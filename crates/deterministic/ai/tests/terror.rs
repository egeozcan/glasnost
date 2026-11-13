//! The own AI and the special attacks of S-0519 (PLAN §4.7): an idle bomb
//! planter is sent at the costliest enemy it sees that carries no bomb, a
//! suicide unit at the nearest enemy, a disarmer at an own bombed object,
//! and a busy unit or an unseen enemy is left alone.

use std::sync::Arc;

use glasnost_ai::brain::Brain;
use glasnost_ai::fixture;
use glasnost_ai::game::Game;
use glasnost_map::CellXY;
use glasnost_rules::sim::{Decimal, WarheadType, WeaponType};
use glasnost_sim::bomb::Bomb;
use glasnost_sim::combat::veterancy::actual_cost;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

fn weapon(warhead: &str, suicide: bool) -> WeaponType {
    WeaponType {
        damage: Some(1),
        rof: Some(40),
        range: Some(d("2")),
        speed: Some(100),
        projectile: Some("Bullet".into()),
        warhead: Some(warhead.into()),
        suicide: suicide.then_some(true),
        ..Default::default()
    }
}

/// The fixture's rules with the planter PLANTER, the disarmer DISARMER and
/// the suicide infantry KAMI (tech level beyond the sidebar's reach).
fn rules() -> Arc<SimRules> {
    let mut r = fixture::ruleset();
    for (n, wp) in [
        ("PLANTER", "PlantW"),
        ("DISARMER", "DisarmW"),
        ("KAMI", "KamiW"),
    ] {
        r.registries
            .entry("infantry".into())
            .or_default()
            .push(n.into());
        let mut u = r.sim.infantry["WALKER"].clone();
        u.primary = Some(wp.into());
        u.tech_level = Some(99);
        r.sim.infantry.insert(n.into(), u);
    }
    for (n, f) in [("IVAN", true), ("DISARM", false), ("DEMO", false)] {
        r.sim.warheads.insert(
            n.into(),
            WarheadType {
                ivan_bomb: f.then_some(true),
                bomb_disarm: (n == "DISARM").then_some(true),
                verses: Some(vec![d("1"); 11]),
                ..Default::default()
            },
        );
    }
    r.sim.weapons.insert("PlantW".into(), weapon("IVAN", false));
    r.sim
        .weapons
        .insert("DisarmW".into(), weapon("DISARM", false));
    r.sim.weapons.insert("KamiW".into(), weapon("DEMO", true));
    Arc::new(SimRules::new(&r, "synthetic-skirmish"))
}

fn game() -> Game {
    let mut g = Game::new(
        rules(),
        &fixture::map(),
        "synthetic",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the skirmish loads");
    while g.world.tick <= g.world.orders_blocked_through || g.world.tick % 15 != 0 {
        g.step();
    }
    g
}

/// A world with a unit `who` of house 0 near its base and the enemy
/// vehicles `enemies` of house 1, all of them on explored cells.
fn scene(g: &Game, who: &str, enemies: &[&str]) -> (World, ObjectId, Vec<ObjectId>) {
    let mut w = g.world.clone();
    let home = w
        .objects
        .values()
        .find(|o| o.owner == Some(HouseId(0)) && o.on_map)
        .map(|o| o.pos.cell())
        .unwrap();
    let at = CellXY::new(home.x, home.y + 6);
    let unit = testkit::spawn(&mut w, Kind::Infantry, who, 0, at, 0);
    let mut ids = Vec::new();
    for (i, n) in enemies.iter().enumerate() {
        let c = CellXY::new(at.x + 4 + i as i32 * 2, at.y);
        ids.push(testkit::spawn(&mut w, Kind::Vehicle, n, 1, c, 0));
        glasnost_sim::econ::shroud::area_reveal(&mut w, HouseId(0), Pos::centre(c, 0), 2);
    }
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    (w, unit, ids)
}

fn attack_of(orders: &[Order], who: ObjectId) -> Option<ObjectId> {
    orders.iter().rev().find_map(|o| match o {
        Order::Behaviour {
            actor,
            behaviour: Behaviour::Attack,
            target: Some(Target::Object(t)),
            ..
        } if *actor == who => Some(*t),
        _ => None,
    })
}

fn bomb(w: &mut World, on: ObjectId) {
    let tick = w.tick;
    w.obj_mut(on).unwrap().combat.bomb = Some(Bomb {
        planter: None,
        house: Some(HouseId(1)),
        planted: tick,
    });
}

// S-0519, PLAN §4.7: an idle planter attacks the costliest seen enemy; the counter counts the order.
#[test]
fn sends_a_planter_at_the_costliest_enemy() {
    let g = game();
    let (w, unit, ids) = scene(&g, "PLANTER", &["GUNR", "MINR"]);
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    let best = *ids.iter().max_by_key(|&&i| actual_cost(&w, i)).unwrap();
    assert_ne!(
        actual_cost(&w, ids[0]),
        actual_cost(&w, ids[1]),
        "costs differ"
    );
    assert_eq!(attack_of(&orders, unit), Some(best));
    assert_eq!(brain.special_uses().bombs, 1);
}

// S-0519 claim 3, PLAN §4.7: a carrier is no target for a planter; the other enemy is.
#[test]
fn passes_over_a_bombed_enemy() {
    let g = game();
    let (mut w, unit, ids) = scene(&g, "PLANTER", &["GUNR", "MINR"]);
    let best = *ids.iter().max_by_key(|&&i| actual_cost(&w, i)).unwrap();
    let other = *ids.iter().find(|&&i| i != best).unwrap();
    bomb(&mut w, best);
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    assert_eq!(attack_of(&brain.think(&w), unit), Some(other));
}

// S-0519 claim 11, PLAN §4.7: a suicide unit goes for the nearest enemy.
#[test]
fn throws_a_suicide_unit_at_the_nearest_enemy() {
    let g = game();
    let (w, unit, ids) = scene(&g, "KAMI", &["GUNR", "MINR"]);
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    assert_eq!(attack_of(&brain.think(&w), unit), Some(ids[0]));
    assert_eq!(brain.special_uses().suicides, 1);
}

// S-0519 claims 3 and 10, PLAN §4.7: a disarmer goes for an own object that carries a bomb its house sees, and for nothing else.
#[test]
fn sends_a_disarmer_at_an_own_bombed_object() {
    let g = game();
    let (mut w, unit, _) = scene(&g, "DISARMER", &[]);
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    assert_eq!(attack_of(&brain.think(&w), unit), None, "no bomb, no order");
    let own = w
        .objects
        .values()
        .find(|o| o.owner == Some(HouseId(0)) && o.kind == Kind::Vehicle && o.on_map)
        .map(|o| o.id);
    let Some(own) = own else { return };
    bomb(&mut w, own);
    w.obj_mut(own).unwrap().combat.bomb.as_mut().unwrap().house = Some(HouseId(0));
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    assert_eq!(attack_of(&brain.think(&w), unit), Some(own));
    assert_eq!(brain.special_uses().disarms, 1);
}

// S-0519, PLAN §4.7: a busy planter is left alone and nothing is ordered without an enemy in sight.
#[test]
fn leaves_a_busy_planter_and_an_empty_field() {
    let g = game();
    let (mut w, unit, ids) = scene(&g, "PLANTER", &["GUNR"]);
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let tick = w.tick;
    let o = w.obj_mut(unit).unwrap();
    o.target = Some(Target::Object(ids[0]));
    o.beh.force(Behaviour::Attack, tick, true);
    brain.think(&w);
    assert_eq!(brain.special_uses().bombs, 0);
    let (w2, unit2, _) = scene(&g, "PLANTER", &[]);
    let mut brain2 = Brain::new(&w2, &fixture::map(), HouseId(0));
    assert_eq!(attack_of(&brain2.think(&w2), unit2), None);
}

// design: own AI (#26) and S-0519 claim 3 (a carrier is no target for a planting weapon): two idle planters in one think go for two different enemies, not both for the costliest one.
#[test]
fn two_planters_take_two_targets() {
    let g = game();
    let (mut w, unit, ids) = scene(&g, "PLANTER", &["GUNR", "MINR"]);
    let at = w.obj(unit).unwrap().pos.cell();
    // Placed at the think's tick: the first planter stays idle.
    let second = testkit::spawn(&mut w, Kind::Infantry, "PLANTER", 0, CellXY::new(at.x, at.y + 1), 0);
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    let (a, b) = (attack_of(&orders, unit), attack_of(&orders, second));
    assert!(a.is_some() && b.is_some(), "{orders:?}");
    assert_ne!(a, b, "both planters go for one enemy");
    assert!(ids.contains(&a.unwrap()) && ids.contains(&b.unwrap()));
    assert_eq!(brain.special_uses().bombs, 2);
}

// design: own AI (#26): an idle planter passes over the enemy another own planter is already sent at.
#[test]
fn a_planter_leaves_a_claimed_enemy_to_its_planter() {
    let g = game();
    let (mut w, unit, ids) = scene(&g, "PLANTER", &["GUNR", "MINR"]);
    let best = *ids.iter().max_by_key(|&&i| actual_cost(&w, i)).unwrap();
    let other = *ids.iter().find(|&&i| i != best).unwrap();
    let at = w.obj(unit).unwrap().pos.cell();
    let busy = testkit::spawn(&mut w, Kind::Infantry, "PLANTER", 0, CellXY::new(at.x, at.y + 1), 0);
    let tick = w.tick;
    let o = w.obj_mut(busy).unwrap();
    o.target = Some(Target::Object(best));
    o.beh.force(Behaviour::Attack, tick, true);
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    assert_eq!(attack_of(&brain.think(&w), unit), Some(other));
}

// design: own AI (#26) and S-0519 claim 10: two idle disarmers in one think go for two different bombed own objects.
#[test]
fn two_disarmers_take_two_bombs() {
    let g = game();
    let (mut w, unit, _) = scene(&g, "DISARMER", &[]);
    let at = w.obj(unit).unwrap().pos.cell();
    let second = testkit::spawn(&mut w, Kind::Infantry, "DISARMER", 0, CellXY::new(at.x, at.y + 1), 0);
    let carriers: Vec<ObjectId> = [3, 5]
        .into_iter()
        .map(|dx| testkit::spawn(&mut w, Kind::Vehicle, "GUNR", 0, CellXY::new(at.x + dx, at.y), 0))
        .collect();
    for &c in &carriers {
        bomb(&mut w, c);
        w.obj_mut(c).unwrap().combat.bomb.as_mut().unwrap().house = Some(HouseId(0));
    }
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    let (a, b) = (attack_of(&orders, unit), attack_of(&orders, second));
    assert!(a.is_some() && b.is_some(), "{orders:?}");
    assert_ne!(a, b, "both disarmers go for one bomb");
    assert_eq!(brain.special_uses().disarms, 2);
}
