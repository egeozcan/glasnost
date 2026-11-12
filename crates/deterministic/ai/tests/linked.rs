//! The own AI and linked weapons (S-0518, PLAN §4.7): it orders prism
//! buildings, up to three, and sends idle chargers at its own overpowerable
//! buildings that have fewer than three chargers.

use std::sync::Arc;

use glasnost_ai::brain::Brain;
use glasnost_ai::fixture;
use glasnost_ai::game::Game;
use glasnost_map::CellXY;
use glasnost_rules::sim::{Decimal, WarheadType, WeaponType};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

fn rules() -> Arc<SimRules> {
    rules_with(false)
}

/// The rules; with `coil_offered` the overpowerable building is buildable
/// and there is no prism type.
fn rules_with(coil_offered: bool) -> Arc<SimRules> {
    let mut r = fixture::ruleset();
    for (reg, n) in [
        ("buildings", "PRSM"),
        ("buildings", "COIL"),
        ("infantry", "ZAPPER"),
    ] {
        r.registries.entry(reg.into()).or_default().push(n.into());
    }
    let mut prism = fixture::building(2, 2, 500, -20);
    prism.primary = Some("Gun".into());
    prism.secondary = Some("Gun".into());
    r.sim.buildings.insert("PRSM".into(), prism);
    let mut coil = fixture::building(2, 2, 500, -20);
    if !coil_offered {
        coil.tech_level = Some(99);
    }
    coil.overpowerable = Some(true);
    r.sim.buildings.insert("COIL".into(), coil);
    r.sim.warheads.insert(
        "ELEC".into(),
        WarheadType {
            verses: Some(vec![d("1"); 11]),
            electric_assault: Some(true),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "Zap".into(),
        WeaponType {
            damage: Some(0),
            rof: Some(40),
            range: Some(d("2")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("ELEC".into()),
            ..Default::default()
        },
    );
    let mut z = r.sim.infantry["WALKER"].clone();
    z.secondary = Some("Zap".into());
    z.tech_level = Some(99);
    r.sim.infantry.insert("ZAPPER".into(), z);
    // A vehicle with the charge weapon (#14): its charge shots register
    // nothing (S-0518 claim 11).
    let mut car = r.sim.vehicles["GUNR"].clone();
    car.secondary = Some("Zap".into());
    car.tech_level = Some(99);
    r.sim.vehicles.insert("ZAPCAR".into(), car);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("ZAPCAR".into());
    if !coil_offered {
        r.sim.general.prism_type = Some("PRSM".into());
    }
    Arc::new(SimRules::new(&r, "synthetic-skirmish"))
}

fn game() -> Game {
    game_with(rules())
}

fn game_with(rules: Arc<SimRules>) -> Game {
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

fn builds(orders: &[Order], name: &str, w: &World) -> usize {
    let idx = w.rules().index_of(Kind::Building, name);
    orders
        .iter()
        .filter(|o| matches!(o, Order::Build { what } if Some(what.index) == idx && what.kind == Kind::Building))
        .count()
}

// S-0518, PLAN §4.7: a house that can build the prism type and can pay twice its price orders one; the counter counts it.
#[test]
fn orders_a_prism_building() {
    let mut g = game();
    for _ in 0..900 {
        g.step();
    }
    let mut w = g.world.clone();
    w.houses[0].credits = 100_000;
    // The building line is free (what the house was building is dropped).
    for l in w
        .econ
        .lines
        .lines
        .iter_mut()
        .filter(|l| l.house == HouseId(0))
    {
        if l.item.as_ref().is_some_and(|i| i.t.kind == Kind::Building) {
            l.item = None;
            l.queue.clear();
        }
    }
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    let n = builds(&orders, "PRSM", &w);
    assert_eq!(n, 1, "{orders:?}");
    assert_eq!(brain.special_uses().prisms as usize, n);
    // With nothing to pay with it orders none.
    w.houses[0].credits = 0;
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    assert_eq!(builds(&brain.think(&w), "PRSM", &w), 0);
}

/// A world of [`game_with`] after 900 ticks with plenty of credits and the
/// house's building line free.
fn free_line(g: &mut Game) -> World {
    for _ in 0..900 {
        g.step();
    }
    let mut w = g.world.clone();
    w.houses[0].credits = 100_000;
    for l in w
        .econ
        .lines
        .lines
        .iter_mut()
        .filter(|l| l.house == HouseId(0))
    {
        if l.item.as_ref().is_some_and(|i| i.t.kind == Kind::Building) {
            l.item = None;
            l.queue.clear();
        }
    }
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    w
}

// S-0518, PLAN §4.7 (M7a step 2c): a house with a charger whose sidebar offers an overpowerable building other than the prism type orders one (it has none), as a person who built the charger would; a house with no charger orders none, and one that has the building orders no second.
#[test]
fn a_house_with_a_charger_orders_an_overpowerable_building() {
    let mut g = game_with(rules_with(true));
    let mut w = free_line(&mut g);
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    assert_eq!(
        builds(&brain.think(&w), "COIL", &w),
        0,
        "no charger, no order"
    );
    let h = home(&w);
    testkit::spawn(
        &mut w,
        Kind::Infantry,
        "ZAPPER",
        0,
        CellXY::new(h.x, h.y + 8),
        0,
    );
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    assert_eq!(builds(&orders, "COIL", &w), 1, "{orders:?}");
    assert_eq!(brain.special_uses().overpowerables, 1);
    testkit::spawn(
        &mut w,
        Kind::Building,
        "COIL",
        0,
        CellXY::new(h.x + 4, h.y + 8),
        0,
    );
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    assert_eq!(builds(&brain.think(&w), "COIL", &w), 0, "it has one");
}

// S-0518, PLAN §4.7: while the building line holds another building the prism building is not ordered (a person waits for the line; the order would be given again at every think).
#[test]
fn no_prism_order_while_the_building_line_is_busy() {
    let mut g = game();
    for _ in 0..900 {
        g.step();
    }
    let mut w = g.world.clone();
    w.houses[0].credits = 100_000;
    let power = glasnost_sim::econ::TypeRef {
        kind: Kind::Building,
        index: w.rules().index_of(Kind::Building, "POWR").expect("POWR"),
    };
    glasnost_sim::orders::issue(&mut w, HouseId(0), Order::Build { what: power });
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let line = glasnost_sim::econ::production::line_of(&w, HouseId(0), power).expect("a line");
    assert!(
        line.item.is_some() || !line.queue.is_empty(),
        "the line is busy"
    );
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    assert_eq!(builds(&brain.think(&w), "PRSM", &w), 0);
    assert_eq!(brain.special_uses().prisms, 0);
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

// S-0518, PLAN §4.7: an idle charger is sent at an own overpowerable building with fewer than three chargers; one with three is left alone.
#[test]
fn sends_a_charger_at_an_own_coil() {
    let g = game();
    let mut w = g.world.clone();
    let h = home(&w);
    let at = CellXY::new(h.x, h.y + 8);
    let coil = testkit::spawn(
        &mut w,
        Kind::Building,
        "COIL",
        0,
        CellXY::new(at.x + 4, at.y),
        0,
    );
    let z = testkit::spawn(&mut w, Kind::Infantry, "ZAPPER", 0, at, 0);
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    assert_eq!(attack_of(&orders, z), Some(coil), "{orders:?}");
    assert!(brain.special_uses().charges >= 1);
    for _ in 0..3 {
        let c = testkit::spawn(&mut w, Kind::Infantry, "ZAPPER", 0, at, 0);
        w.obj_mut(coil).unwrap().combat.chargers.push(c);
    }
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    assert_eq!(attack_of(&orders, z), None);
}

/// A world of [`game`] with an own COIL eight cells south of home and
/// `n` idle chargers of type `name` (`kind`) by it; the coil and the chargers.
fn coil_and(n: usize, kind: Kind, name: &str) -> (World, ObjectId, Vec<ObjectId>) {
    let g = game();
    let mut w = g.world.clone();
    let h = home(&w);
    let at = CellXY::new(h.x, h.y + 8);
    let coil = testkit::spawn(
        &mut w,
        Kind::Building,
        "COIL",
        0,
        CellXY::new(at.x + 4, at.y),
        0,
    );
    let chargers = (0..n)
        .map(|i| {
            let i = i32::try_from(i).unwrap();
            testkit::spawn(&mut w, kind, name, 0, CellXY::new(at.x - i % 3, at.y + i / 3), 0)
        })
        .collect();
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    (w, coil, chargers)
}

// design: own AI (#14) and S-0518 Constants (three chargers overpower the building): of six idle chargers by an own coil no more than three are sent at it in one think (the cap read only the chargers whose shots had hit, so every idle one went).
#[test]
fn sends_no_more_than_three_chargers_at_a_coil() {
    let (w, coil, chargers) = coil_and(6, Kind::Infantry, "ZAPPER");
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    let sent = chargers
        .iter()
        .filter(|&&z| attack_of(&orders, z) == Some(coil))
        .count();
    assert!((1..=3).contains(&sent), "{sent} sent: {orders:?}");
}

// design: own AI (#14): chargers already aiming at the coil count toward the three; with three of them none more is sent.
#[test]
fn chargers_on_their_way_count_toward_the_three() {
    let (mut w, coil, chargers) = coil_and(4, Kind::Infantry, "ZAPPER");
    // The orders go in on the tick before a think, so that the three are
    // still on their way (none of them has hit yet).
    testkit::run(&mut w, 14);
    for &z in &chargers[..3] {
        glasnost_sim::orders::issue(
            &mut w,
            HouseId(0),
            Order::Behaviour {
                actor: z,
                behaviour: Behaviour::Attack,
                attack_move: false,
                target: Some(Target::Object(coil)),
                dest: None,
            },
        );
    }
    testkit::run(&mut w, 1);
    assert_eq!(w.tick % 15, 0);
    assert_eq!(glasnost_sim::linked::chargers(&w, coil), 0, "none has hit");
    assert!(
        chargers[..3]
            .iter()
            .all(|&z| w.obj(z).unwrap().target == Some(Target::Object(coil))),
        "three aim at the coil"
    );
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    assert_eq!(attack_of(&orders, chargers[3]), None, "{orders:?}");
}

// design: own AI (#14) and S-0518 claim 11 (a firer that is not infantry registers nothing): an idle vehicle with the charge weapon by an own coil is not sent at it.
#[test]
fn a_vehicle_charger_is_not_sent_at_a_coil() {
    let (w, _coil, cars) = coil_and(1, Kind::Vehicle, "ZAPCAR");
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    assert_eq!(attack_of(&orders, cars[0]), None, "{orders:?}");
}

// PLAN §4.7 (M8 B4): the charger looks for an own coil across the base, 18 cells off, but not 26.
#[test]
fn sends_a_charger_at_a_coil_across_the_base() {
    for (off, sent) in [(18, true), (26, false)] {
        let g = game();
        let mut w = g.world.clone();
        let h = home(&w);
        let at = CellXY::new(h.x, h.y + 8);
        let coil = testkit::spawn(
            &mut w,
            Kind::Building,
            "COIL",
            0,
            CellXY::new(at.x + off, at.y),
            0,
        );
        let z = testkit::spawn(&mut w, Kind::Infantry, "ZAPPER", 0, at, 0);
        testkit::run(&mut w, 1);
        while w.tick % 15 != 0 {
            testkit::run(&mut w, 1);
        }
        let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
        let orders = brain.think(&w);
        assert_eq!(
            attack_of(&orders, z),
            sent.then_some(coil),
            "{off} cells: {orders:?}"
        );
    }
}
