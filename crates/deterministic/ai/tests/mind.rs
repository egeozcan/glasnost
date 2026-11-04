//! The own AI and mind control (S-0601, PLAN §4.7): an idle controller is
//! sent at the costliest seen enemy it may take, passing over the ones the
//! firing check refuses; a controller at capacity is not sent; an
//! unlimited controller stops before an overload level that damages it;
//! two controllers are not sent at the same enemy. The tests use invented
//! types on the synthetic skirmish.

use std::sync::Arc;

use glasnost_ai::brain::Brain;
use glasnost_ai::fixture;
use glasnost_ai::game::Game;
use glasnost_map::CellXY;
use glasnost_rules::sim::{Decimal, WarheadType, WeaponType};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};
use glasnost_sim::{mind, testkit};

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

/// The fixture's rules with the takeover warhead MIND, the weapons RAY
/// (capacity 2), RAYU (unlimited, capacity 2) and RAY1 (capacity 1), the
/// infantry CLONE (RAY) and SOLO (RAY1)
/// and the vehicle MASTER (RAYU), both beyond the sidebar's tech level;
/// the psionics-immune vehicle STOIC and the costly vehicle RICH; the
/// overload lists count (1, 9), damage (0, 5), frames (10, 10).
fn rules() -> Arc<SimRules> {
    let mut r = fixture::ruleset();
    let reg = |r: &mut glasnost_rules::schema::Ruleset, k: &str, n: &str| {
        r.registries.entry(k.into()).or_default().push(n.into());
    };
    r.sim.warheads.insert(
        "MIND".into(),
        WarheadType {
            mind_control: Some(true),
            verses: Some(vec![d("1"); 11]),
            ..Default::default()
        },
    );
    for (name, unlimited, capacity) in [("RAY", false, 2), ("RAYU", true, 2), ("RAY1", false, 1)] {
        r.sim.weapons.insert(
            name.into(),
            WeaponType {
                damage: Some(capacity),
                rof: Some(40),
                range: Some(d("2")),
                speed: Some(100),
                projectile: Some("Bullet".into()),
                warhead: Some("MIND".into()),
                infinite_mind_control: unlimited.then_some(true),
                ..Default::default()
            },
        );
    }
    let mut c = r.sim.infantry["WALKER"].clone();
    c.primary = Some("RAY".into());
    c.secondary = None;
    c.tech_level = Some(99);
    let mut one = c.clone();
    one.primary = Some("RAY1".into());
    r.sim.infantry.insert("CLONE".into(), c);
    reg(&mut r, "infantry", "CLONE");
    r.sim.infantry.insert("SOLO".into(), one);
    reg(&mut r, "infantry", "SOLO");
    let mut m = r.sim.vehicles["GUNR"].clone();
    m.primary = Some("RAYU".into());
    m.secondary = None;
    m.tech_level = Some(99);
    r.sim.vehicles.insert("MASTER".into(), m);
    reg(&mut r, "vehicles", "MASTER");
    let mut s = r.sim.vehicles["GUNR"].clone();
    s.combat.immune_to_psionics = Some(true);
    s.cost = Some(9000);
    s.tech_level = Some(99);
    r.sim.vehicles.insert("STOIC".into(), s);
    reg(&mut r, "vehicles", "STOIC");
    let mut rich = r.sim.vehicles["GUNR"].clone();
    rich.cost = Some(5000);
    rich.tech_level = Some(99);
    r.sim.vehicles.insert("RICH".into(), rich);
    reg(&mut r, "vehicles", "RICH");
    r.sim.general.overload_count = Some(vec![1, 9]);
    r.sim.general.overload_damage = Some(vec![0, 5]);
    r.sim.general.overload_frames = Some(vec![10, 10]);
    Arc::new(SimRules::new(&r, "synthetic-mind"))
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

fn home(w: &World) -> CellXY {
    w.objects
        .values()
        .find(|o| o.owner == Some(HouseId(0)) && o.on_map)
        .map(|o| o.pos.cell())
        .unwrap()
}

/// A world with controllers `ctrl` of house 0 six cells south of its base
/// (one cell apart) and the enemy vehicles `enemies` of house 1 at the
/// given offsets from the first, all revealed for house 0.
fn scene(
    g: &Game,
    ctrl: &[(Kind, &str)],
    enemies: &[(&str, i32, i32)],
) -> (World, Vec<ObjectId>, Vec<ObjectId>) {
    let mut w = g.world.clone();
    let h = home(&w);
    let at = CellXY::new(h.x, h.y + 6);
    let units = ctrl
        .iter()
        .enumerate()
        .map(|(i, &(k, n))| testkit::spawn(&mut w, k, n, 0, CellXY::new(at.x - i as i32, at.y), 0))
        .collect();
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
    (w, units, ids)
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

// S-0601 claims 3 to 6, PLAN §4.7: a controller goes for the costliest enemy it may take, passing over a costlier psionics-immune one (the firing check refuses it); the counter counts the order.
#[test]
fn sends_a_controller_at_the_costliest_enemy_it_may_take() {
    let g = game();
    let (w, u, ids) = scene(
        &g,
        &[(Kind::Infantry, "CLONE")],
        &[("GUNR", 2, 0), ("RICH", 4, 0), ("STOIC", 3, 0)],
    );
    let mut b = brain(&w);
    let orders = b.think(&w);
    assert_eq!(attack_of(&orders, u[0]), Some(ids[1]), "{orders:?}");
    assert_eq!(b.special_uses().takeovers, 1);
}

// S-0601 claim 2, PLAN §4.7: a controller at capacity is not sent.
#[test]
fn no_order_to_a_controller_at_capacity() {
    let g = game();
    let (mut w, u, ids) = scene(
        &g,
        &[(Kind::Infantry, "CLONE")],
        &[("GUNR", 2, 0), ("GUNR", 2, 2), ("RICH", 4, 0)],
    );
    assert!(mind::take(&mut w, u[0], ids[0]));
    assert!(mind::take(&mut w, u[0], ids[1]));
    assert!(mind::at_capacity(&w, u[0]));
    let mut b = brain(&w);
    assert_eq!(attack_of(&b.think(&w), u[0]), None);
    assert_eq!(b.special_uses().takeovers, 0);
}

// S-0601 claims 2, 7 and 9, PLAN §4.7 (our own AI's design, M8 seam trace 4): a capacity-1 controller may still fire at a new target, and the new takeover releases its victim back to its owner. No part of the brain sends one that holds a victim at another enemy (here the army's attack on a near enemy did), as a person keeps the victim rather than swapping; in the lane a controller swapped a power plant and a tank back and forth for some 1 500 ticks. A free controller is still sent (the tests above).
#[test]
fn no_part_of_the_brain_swaps_a_capacity_one_controllers_victim() {
    let g = game();
    // The gunner stands beyond the controller's reach, so its own search does not pick it.
    let (mut w, u, ids) = scene(
        &g,
        &[(Kind::Infantry, "SOLO")],
        &[("GUNR", 6, 0), ("RICH", 2, 0)],
    );
    assert_eq!(
        mind::controlled_by(&w, ids[1]),
        Some(u[0]),
        "its own shot took the near one"
    );
    assert!(mind::at_capacity(&w, u[0]));
    // S-0309 claim 29: a target of its own house (the victim) is dropped at a check tick.
    testkit::run(&mut w, 32);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    assert_eq!(w.obj(u[0]).expect("the controller").target, None);
    let mut b = brain(&w);
    let orders = b.think(&w);
    assert_eq!(attack_of(&orders, u[0]), None, "{orders:?}");
    assert_eq!(mind::controlled_by(&w, ids[1]), Some(u[0]));
}

// S-0601 claim 16, PLAN §4.7: an unlimited controller is sent while one more victim keeps it at a harmless overload level, and not once the next victim would bring damaging pulses.
#[test]
fn unlimited_controller_stops_before_overload() {
    let g = game();
    let (w, u, ids) = scene(
        &g,
        &[(Kind::Vehicle, "MASTER")],
        &[("RICH", 3, 0), ("GUNR", 4, 2)],
    );
    assert!(
        glasnost_ai::mind::safe_more(&w, u[0]),
        "N + 1 = 1: level 0, no damage"
    );
    let mut b = brain(&w);
    assert_eq!(attack_of(&b.think(&w), u[0]), Some(ids[0]));
    let mut w = w;
    assert!(mind::take(&mut w, u[0], ids[0]));
    assert!(
        !glasnost_ai::mind::safe_more(&w, u[0]),
        "N + 1 = 2: level 1, damage 5"
    );
    let mut b = brain(&w);
    let orders = b.think(&w);
    assert_eq!(attack_of(&orders, u[0]), None, "{orders:?}");
}

// PLAN §4.7: two controllers are not sent at the same enemy in one think.
#[test]
fn two_controllers_pick_two_enemies() {
    let g = game();
    let (w, u, ids) = scene(
        &g,
        &[(Kind::Infantry, "CLONE"), (Kind::Infantry, "CLONE")],
        &[("RICH", 3, 0), ("GUNR", 3, 2)],
    );
    let mut b = brain(&w);
    let orders = b.think(&w);
    let a = attack_of(&orders, u[0]);
    let c = attack_of(&orders, u[1]);
    assert!(a.is_some() && c.is_some(), "{orders:?}");
    assert_ne!(a, c);
    assert!(ids.contains(&a.unwrap()) && ids.contains(&c.unwrap()));
}
