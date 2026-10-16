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
    coil.tech_level = Some(99);
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
    r.sim.general.prism_type = Some("PRSM".into());
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
