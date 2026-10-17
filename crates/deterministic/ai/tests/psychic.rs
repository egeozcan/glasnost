//! The own AI and psychic weapons (S-0610, PLAN §4.7): an idle unit with a
//! berserk weapon is sent at the seen enemy with the most enemies around
//! it, passing over targets its weapon may not fire at; no order goes to a
//! berserk unit; a house with no psychic sensor builds one; a ready
//! psychic reveal is fired at an enemy start cell the house does not see.
//! The tests use invented types on the synthetic skirmish.

use std::sync::Arc;

use glasnost_ai::brain::Brain;
use glasnost_ai::fixture;
use glasnost_ai::game::Game;
use glasnost_map::CellXY;
use glasnost_rules::sim::{Decimal, WarheadType, WeaponType};
use glasnost_rules::sim_econ::SuperweaponType;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};
use glasnost_sim::{psychic, testkit};

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

/// The fixture's rules with the berserk-causing warhead MAD, the weapon
/// MADRAY, the rifleman MADMAN that carries it (beyond the sidebar's tech
/// level), the psionics-immune vehicle STOIC, the sensor building EYE
/// (radius 6, buildable when `sensor`), and the psychic reveal REVEAL with
/// its giving building MINDB (not buildable).
fn rules(sensor: bool) -> Arc<SimRules> {
    let mut r = fixture::ruleset();
    let reg = |r: &mut glasnost_rules::schema::Ruleset, k: &str, n: &str| {
        r.registries.entry(k.into()).or_default().push(n.into());
    };
    r.sim.warheads.insert(
        "MAD".into(),
        WarheadType {
            psychedelic: Some(true),
            verses: Some(vec![d("1"); 11]),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "MADRAY".into(),
        WeaponType {
            damage: Some(200),
            rof: Some(40),
            range: Some(d("6")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("MAD".into()),
            ..Default::default()
        },
    );
    let mut m = r.sim.infantry["WALKER"].clone();
    m.primary = Some("MADRAY".into());
    m.secondary = None;
    m.tech_level = Some(99);
    r.sim.infantry.insert("MADMAN".into(), m);
    reg(&mut r, "infantry", "MADMAN");
    let mut s = r.sim.vehicles["GUNR"].clone();
    s.combat.immune_to_psionics = Some(true);
    s.tech_level = Some(99);
    r.sim.vehicles.insert("STOIC".into(), s);
    reg(&mut r, "vehicles", "STOIC");
    let mut eye = fixture::building(1, 1, 150, 0);
    eye.econ.psychic_detection_radius = Some(6);
    if !sensor {
        eye.tech_level = Some(99);
    }
    r.sim.buildings.insert("EYE".into(), eye);
    reg(&mut r, "buildings", "EYE");
    r.sim.superweapons.insert(
        "REVEAL".into(),
        SuperweaponType {
            kind: Some("PsychicReveal".into()),
            action: Some("PsychicReveal".into()),
            recharge_time: Some(d("0.1")),
            is_powered: Some(false),
            ..Default::default()
        },
    );
    reg(&mut r, "superweapons", "REVEAL");
    let mut mind = fixture::building(1, 1, 500, 0);
    mind.tech_level = Some(99);
    mind.econ.super_weapon = Some("REVEAL".into());
    r.sim.buildings.insert("MINDB".into(), mind);
    reg(&mut r, "buildings", "MINDB");
    Arc::new(SimRules::new(&r, "synthetic-psychic"))
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

fn home(w: &World) -> CellXY {
    w.objects
        .values()
        .find(|o| o.owner == Some(HouseId(0)) && o.on_map)
        .map(|o| o.pos.cell())
        .unwrap()
}

/// A world with a MADMAN of house 0 six cells south of its base and the
/// enemy vehicles `enemies` of house 1 at the given offsets from it, all
/// revealed for house 0.
fn scene(g: &Game, enemies: &[(&str, i32, i32)]) -> (World, ObjectId, Vec<ObjectId>) {
    let mut w = g.world.clone();
    let h = home(&w);
    let at = CellXY::new(h.x, h.y + 6);
    let unit = testkit::spawn(&mut w, Kind::Infantry, "MADMAN", 0, at, 0);
    let mut ids = Vec::new();
    for &(n, dx, dy) in enemies {
        let c = CellXY::new(at.x + dx, at.y + dy);
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

fn brain(w: &World) -> Brain {
    Brain::new(w, &fixture::map(), HouseId(0))
}

// S-0610 claims 4 and 6, PLAN §4.7: a berserk weapon goes on the enemy among others (it turns on its neighbours), not on a lone nearer one; the counter counts the order.
#[test]
fn sends_a_berserk_weapon_at_an_enemy_group() {
    let g = game(rules(false));
    let (w, unit, ids) = scene(
        &g,
        &[
            ("GUNR", 3, 0),
            ("GUNR", 7, 0),
            ("GUNR", 8, 0),
            ("GUNR", 7, 1),
        ],
    );
    let mut b = brain(&w);
    let orders = b.think(&w);
    let t = attack_of(&orders, unit).expect("an attack order");
    assert!(ids[1..].contains(&t), "a member of the group");
    assert_eq!(b.special_uses().berserks, 1);
}

// S-0610 claims 3 and 4, PLAN §4.7: a psionics-immune enemy (the firing check refuses it) and an enemy already berserk are passed over.
#[test]
fn passes_over_immune_and_berserk_enemies() {
    let g = game(rules(false));
    let (w, unit, _) = scene(&g, &[("STOIC", 3, 0)]);
    let mut b = brain(&w);
    assert_eq!(attack_of(&b.think(&w), unit), None);
    assert_eq!(b.special_uses().berserks, 0);
    let (mut w, unit, ids) = scene(&g, &[("GUNR", 3, 0)]);
    psychic::enter(&mut w, ids[0], 10_000);
    let mut b = brain(&w);
    assert_eq!(attack_of(&b.think(&w), unit), None);
}

// S-0214 claim 15 and S-0610 claim 6: the brain gives no order to a berserk unit of its house.
#[test]
fn no_orders_to_a_berserk_unit() {
    let g = game(rules(false));
    let (mut w, unit, _) = scene(&g, &[("GUNR", 3, 0)]);
    psychic::enter(&mut w, unit, 10_000);
    let mut b = brain(&w);
    let orders = b.think(&w);
    assert!(orders.iter().all(|o| o.actor() != Some(unit)), "{orders:?}");
}

/// The number of buildings of type `name` house `h` has on the map.
fn count(w: &World, h: u8, name: &str) -> usize {
    w.objects
        .values()
        .filter(|o| o.owner == Some(HouseId(h)) && o.on_map && o.health > 0)
        .filter(|o| {
            o.kind == Kind::Building && w.rules().buildings[o.type_index as usize].name == name
        })
        .count()
}

// S-0610 claim 9, PLAN §4.7: a house that can build a psychic sensor builds one, and only one.
#[test]
fn builds_one_psychic_sensor() {
    let mut g = game(rules(true));
    while g.world.tick < 15_000 && g.outcome().is_none() && count(&g.world, 0, "EYE") == 0 {
        g.step();
    }
    assert_eq!(count(&g.world, 0, "EYE"), 1, "by tick {}", g.world.tick);
    let uses = g.special_uses();
    assert!(uses.iter().any(|(_, u)| u.sensors >= 1));
    for _ in 0..3000 {
        g.step();
    }
    assert!(count(&g.world, 0, "EYE") <= 1);
}

// S-0610 claims 13 and 14, PLAN §4.7: a ready psychic reveal is fired at the enemy start cell the house does not see, and counted; with every start seen at an unexplored cell; with the whole map explored it is kept.
#[test]
fn fires_the_psychic_reveal_at_an_unseen_enemy_start() {
    let g = game(rules(false));
    let mut w = g.world.clone();
    let h = home(&w);
    let at = (2..12)
        .map(|k| CellXY::new(h.x - k, h.y))
        .find(|&c| w.cells.usable(c) && w.objects.values().all(|o| o.pos.cell() != c))
        .unwrap();
    testkit::spawn(&mut w, Kind::Building, "MINDB", 0, at, 0);
    let i = w
        .rules()
        .superweapons
        .types
        .iter()
        .position(|t| t.name == "REVEAL")
        .unwrap() as u32;
    let mut n = 0;
    while !glasnost_sim::superweapons::charge::get(&w, HouseId(0), i).is_some_and(|r| r.ready) {
        testkit::run(&mut w, 1);
        n += 1;
        assert!(n < 2000, "the reveal never got ready");
    }
    let enemy = fixture::at(fixture::START_DX, 0);
    let mut b = brain(&w);
    let orders = b.think(&w);
    let fired: Vec<CellXY> = orders
        .iter()
        .filter_map(|o| match *o {
            Order::FireSuperweapon { sw, cell } if sw == i => Some(cell),
            _ => None,
        })
        .collect();
    assert_eq!(fired.len(), 1, "{orders:?}");
    assert!(!glasnost_sim::econ::shroud::seen(&w, HouseId(0), fired[0]));
    assert!(
        dist(fired[0], enemy) <= 25,
        "{:?} near the enemy start {:?}",
        fired[0],
        enemy
    );
    assert_eq!(b.special_uses().reveals, 1);
    // Every start seen: it scouts an unexplored cell instead.
    for c in testkit_starts(&w) {
        glasnost_sim::econ::shroud::area_reveal(&mut w, HouseId(0), Pos::centre(c, 0), 1);
    }
    let fired = fires(&brain(&w).think(&w), i);
    assert_eq!(fired.len(), 1);
    assert!(!glasnost_sim::econ::shroud::explored(
        &w,
        HouseId(0),
        fired[0]
    ));
    // The whole map explored: the reveal is kept.
    glasnost_sim::econ::shroud::crate_reveal(&mut w, HouseId(0));
    assert!(fires(&brain(&w).think(&w), i).is_empty());
}

fn fires(orders: &[Order], i: u32) -> Vec<CellXY> {
    orders
        .iter()
        .filter_map(|o| match *o {
            Order::FireSuperweapon { sw, cell } if sw == i => Some(cell),
            _ => None,
        })
        .collect()
}

fn dist(a: CellXY, b: CellXY) -> i32 {
    (a.x - b.x).pow(2) + (a.y - b.y).pow(2)
}

fn testkit_starts(_: &World) -> Vec<CellXY> {
    glasnost_ai::skirmish::start_cells(&fixture::map())
}
