//! The own AI and chrono weapons (S-0514, PLAN §4.7): an idle erasing unit
//! is sent against the costliest erasable enemy its house sees, a unit
//! that holds a target is left alone, an unseen enemy is not a target, and
//! a type that cannot be erased is passed over.

use std::sync::Arc;

use glasnost_ai::brain::Brain;
use glasnost_ai::fixture;
use glasnost_ai::game::Game;
use glasnost_map::CellXY;
use glasnost_rules::sim::{Decimal, WarheadType, WeaponType};
use glasnost_sim::combat::veterancy::actual_cost;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

/// The fixture's rules with the temporal warhead TICK, the weapon TIME and
/// the erasing rifleman ERASER (tech level beyond the sidebar's reach).
fn rules(unwarpable: Option<&str>) -> Arc<SimRules> {
    let mut r = fixture::ruleset();
    r.registries
        .entry("infantry".into())
        .or_default()
        .push("ERASER".into());
    r.sim.warheads.insert(
        "TICK".into(),
        WarheadType {
            temporal: Some(true),
            verses: Some(vec![d("1"); 11]),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "TIME".into(),
        WeaponType {
            damage: Some(30),
            rof: Some(40),
            range: Some(d("2")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("TICK".into()),
            ..Default::default()
        },
    );
    let mut e = r.sim.infantry["WALKER"].clone();
    e.primary = Some("TIME".into());
    e.tech_level = Some(99);
    r.sim.infantry.insert("ERASER".into(), e);
    if let Some(n) = unwarpable {
        r.sim.vehicles.get_mut(n).unwrap().warpable = Some(false);
    }
    Arc::new(SimRules::new(&r, "synthetic-skirmish"))
}

fn game(rules: Arc<SimRules>) -> Game {
    let mut g = Game::new(
        rules,
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

/// A world with an eraser of house 0 near its base, and the enemy vehicles
/// `enemies` of house 1, the first `seen` of them on explored cells.
fn scene(g: &Game, enemies: &[&str], seen: usize) -> (World, ObjectId, Vec<ObjectId>) {
    let mut w = g.world.clone();
    let home = w
        .objects
        .values()
        .find(|o| o.owner == Some(HouseId(0)) && o.on_map)
        .map(|o| o.pos.cell())
        .unwrap();
    let at = CellXY::new(home.x, home.y + 6);
    let unit = testkit::spawn(&mut w, Kind::Infantry, "ERASER", 0, at, 0);
    let mut ids = Vec::new();
    for (i, n) in enemies.iter().enumerate() {
        let c = CellXY::new(at.x + 4 + i as i32 * 2, at.y);
        ids.push(testkit::spawn(&mut w, Kind::Vehicle, n, 1, c, 0));
        if i < seen {
            glasnost_sim::econ::shroud::area_reveal(&mut w, HouseId(0), Pos::centre(c, 0), 2);
        }
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

// S-0514, PLAN §4.7: an idle erasing unit attacks the costliest seen enemy; the counter counts the order.
#[test]
fn sends_an_eraser_against_the_costliest_enemy() {
    let g = game(rules(None));
    let (w, unit, ids) = scene(&g, &["GUNR", "MINR"], 2);
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    let best = *ids.iter().max_by_key(|&&i| actual_cost(&w, i)).unwrap();
    assert_ne!(
        actual_cost(&w, ids[0]),
        actual_cost(&w, ids[1]),
        "costs differ"
    );
    assert_eq!(attack_of(&orders, unit), Some(best));
    assert_eq!(brain.special_uses().erasures, 1);
}

// S-0514, PLAN §4.7: it sees only what its house has explored, and passes over a type that cannot be erased.
#[test]
fn passes_over_unseen_and_unerasable_enemies() {
    let g = game(rules(None));
    let (w, unit, ids) = scene(&g, &["GUNR", "MINR"], 1);
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let a = attack_of(&brain.think(&w), unit);
    let seen_second =
        glasnost_sim::econ::shroud::explored(&w, HouseId(0), w.obj(ids[1]).unwrap().pos.cell());
    assert!(a == Some(ids[0]) || (seen_second && a == Some(ids[1])));
    let g2 = game(rules(Some("MINR")));
    let (w2, unit2, ids2) = scene(&g2, &["GUNR", "MINR"], 2);
    let mut brain2 = Brain::new(&w2, &fixture::map(), HouseId(0));
    assert_eq!(attack_of(&brain2.think(&w2), unit2), Some(ids2[0]));
}

// S-0514, PLAN §4.7: a unit that already holds a target is left alone; nothing is ordered without an enemy in sight.
#[test]
fn leaves_a_busy_eraser_and_an_empty_field() {
    let g = game(rules(None));
    let (mut w, unit, ids) = scene(&g, &["GUNR"], 1);
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let tick = w.tick;
    let o = w.obj_mut(unit).unwrap();
    o.target = Some(Target::Object(ids[0]));
    o.beh.force(Behaviour::Attack, tick, true);
    brain.think(&w);
    assert_eq!(brain.special_uses().erasures, 0);
    let (w3, unit3, _) = scene(&g, &[], 0);
    let mut brain3 = Brain::new(&w3, &fixture::map(), HouseId(0));
    assert_eq!(attack_of(&brain3.think(&w3), unit3), None);
    assert_eq!(brain3.special_uses().erasures, 0);
}
