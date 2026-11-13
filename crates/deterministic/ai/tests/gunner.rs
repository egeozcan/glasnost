//! The own AI and gunner vehicles (S-0308 claims 5 and 6; PLAN §4.7): an
//! infantry unit whose mode gives an empty gunner vehicle a healing weapon
//! is sent into it, one for every `PER_HEALER` armed vehicles.

use std::sync::Arc;

use glasnost_ai::brain::Brain;
use glasnost_ai::gunner::PER_HEALER;
use glasnost_ai::{fixture, game::Game};
use glasnost_rules::schema::Ruleset;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{combat::weapon, transport};

/// The fixture's rules with GUNR a switching gunner transport (slot 0 its
/// gun, slot 1 a healing weapon) and MEDI an unarmed infantry type of mode
/// `mode`.
fn ruleset(mode: i32) -> Ruleset {
    let mut r = fixture::ruleset();
    let mut heal = r.sim.weapons["Gun"].clone();
    heal.damage = Some(-50);
    r.sim.weapons.insert("Heal".into(), heal);
    let g = r.sim.vehicles.get_mut("GUNR").expect("GUNR");
    g.passengers = Some(1);
    g.size_limit = Some(glasnost_rules::sim::Decimal("2".into()));
    g.turret_count = Some(2);
    g.weapon_count = Some(2);
    g.combat.weapon_slots = Some(vec!["Gun".into(), "Heal".into()]);
    g.gunner = Some(true);
    let mut medi = r.sim.infantry["WALKER"].clone();
    medi.primary = None;
    medi.secondary = None;
    medi.ifv_mode = Some(mode);
    r.sim.infantry.insert("MEDI".into(), medi);
    r.registries
        .get_mut("infantry")
        .expect("infantry registry")
        .push("MEDI".into());
    r
}

/// House 0 of a skirmish under `rules` once its construction yard stands,
/// with `guns` GUNR and one MEDI spawned at home.
fn scene(rules: &Ruleset, guns: usize) -> (World, ObjectId, Vec<ObjectId>) {
    let map = fixture::map();
    let mut g = Game::new(
        Arc::new(SimRules::new(rules, "synthetic-gunner")),
        &map,
        "synthetic-gunner",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the skirmish loads");
    let yard = |w: &World| {
        w.objects.values().any(|o| {
            o.owner == Some(HouseId(0))
                && o.kind == Kind::Building
                && w.rules().buildings[o.type_index as usize].name == "YARD"
        })
    };
    while g.world.tick < 600 && !yard(&g.world) {
        g.step();
    }
    assert!(yard(&g.world), "the yard stands");
    let mut w = g.world.clone();
    let x = -fixture::START_DX;
    let guns: Vec<ObjectId> = (0..guns)
        .map(|k| {
            testkit::spawn(
                &mut w,
                Kind::Vehicle,
                "GUNR",
                0,
                fixture::at(x - 4 + 2 * k as i32, -5),
                0,
            )
        })
        .collect();
    let medi = testkit::spawn(&mut w, Kind::Infantry, "MEDI", 0, fixture::at(x + 4, 4), 0);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    (w, medi, guns)
}

fn think(w: &World) -> (Vec<Order>, u32) {
    let mut b = Brain::with_plan(w, &fixture::map(), HouseId(0), fixture::plan());
    let orders = b.think(w);
    (orders, b.special_uses().gunner_loads)
}

fn sent_into(orders: &[Order], unit: ObjectId) -> Option<ObjectId> {
    orders.iter().find_map(|o| match o {
        Order::Behaviour {
            actor,
            behaviour: Behaviour::Enter,
            target: Some(Target::Object(v)),
            ..
        } if *actor == unit => Some(*v),
        Order::Behaviour {
            actor,
            behaviour: Behaviour::Enter,
            dest: Some(Target::Object(v)),
            ..
        } if *actor == unit => Some(*v),
        _ => None,
    })
}

// S-0308 claims 5 and 6, PLAN §4.7: with PER_HEALER armed vehicles and an infantry unit whose mode gives the empty gunner vehicle a healing weapon, the unit is sent in; once aboard, the vehicle's weapon number is the mode.
#[test]
fn loads_a_healer_into_an_empty_gunner_vehicle() {
    let (mut w, medi, guns) = scene(&ruleset(1), PER_HEALER);
    let (orders, loads) = think(&w);
    let v = sent_into(&orders, medi).expect("MEDI is sent into a gunner vehicle");
    assert!(guns.contains(&v));
    assert_eq!(loads, 1);
    for o in orders
        .into_iter()
        .filter(|o| matches!(o, Order::Behaviour { actor, .. } if *actor == medi))
    {
        orders::issue(&mut w, HouseId(0), o);
    }
    for _ in 0..600 {
        testkit::run(&mut w, 1);
        if transport::count(&w, v) == 1 {
            break;
        }
    }
    assert_eq!(transport::count(&w, v), 1, "MEDI is aboard");
    assert_eq!(weapon::weapon_number(&w, v), 1);
}

// PLAN §4.7: a person keeps one healer for every PER_HEALER armed vehicles; with fewer, nobody is sent.
#[test]
fn no_healer_for_a_small_army() {
    let (w, medi, _) = scene(&ruleset(1), PER_HEALER - 1);
    let (orders, loads) = think(&w);
    assert_eq!(sent_into(&orders, medi), None);
    assert_eq!(loads, 0);
}

// S-0308 claim 6, PLAN §4.7: an infantry unit whose mode gives the vehicle its gun, not a healing weapon, is not sent.
#[test]
fn no_load_without_a_healing_mode() {
    let (w, medi, _) = scene(&ruleset(0), PER_HEALER);
    let (orders, loads) = think(&w);
    assert_eq!(sent_into(&orders, medi), None);
    assert_eq!(loads, 0);
}

// S-0308 claims 5 and 6, S-0509, PLAN §4.7: when the healer is an engineer and a capturable enemy building is in sight, the engineer still goes into the gunner vehicle (a person who has picked it for the vehicle does not send it to capture); the M7b lane found every such engineer sent to capture.
#[test]
fn keeps_an_engineer_healer_from_capturing() {
    let mut r = ruleset(1);
    r.sim.infantry.get_mut("MEDI").expect("MEDI").engineer = Some(true);
    r.registries
        .get_mut("buildings")
        .expect("buildings registry")
        .push("CAPT".into());
    r.sim.buildings.insert(
        "CAPT".into(),
        glasnost_rules::sim::BuildingType {
            strength: Some(400),
            foundation: Some([2, 2]),
            capturable: Some(true),
            tech_level: Some(99),
            ..Default::default()
        },
    );
    let (mut w, medi, guns) = scene(&r, PER_HEALER);
    let at = fixture::at(-fixture::START_DX + 7, 7);
    testkit::spawn(&mut w, Kind::Building, "CAPT", 1, at, 0);
    let p = glasnost_sim::world::Pos::centre(at, 0);
    glasnost_sim::econ::shroud::area_reveal(&mut w, HouseId(0), p, 4);
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let mut b = Brain::with_plan(&w, &fixture::map(), HouseId(0), fixture::plan());
    let orders = b.think(&w);
    let v = sent_into(&orders, medi).expect("the engineer is sent into a gunner vehicle");
    assert!(guns.contains(&v));
    assert_eq!(b.special_uses().gunner_loads, 1);
    assert_eq!(b.special_uses().engineers, 0);
}

// PLAN §4.7: with an empty gunner vehicle waiting, PER_HEALER armed vehicles and no infantry that would heal in it, the house queues one such infantry unit even with no credits to spare (the line pays as money comes in; the M7b lane found the army spending every credit).
#[test]
fn queues_a_healer_without_credits_to_spare() {
    let (mut w, medi, _) = scene(&ruleset(1), PER_HEALER);
    glasnost_sim::objects::remove(&mut w, medi);
    // PILL is the fixture's infantry factory; POWR keeps it powered.
    let x = -fixture::START_DX;
    testkit::spawn(&mut w, Kind::Building, "POWR", 0, fixture::at(x + 6, -6), 0);
    testkit::spawn(&mut w, Kind::Building, "PILL", 0, fixture::at(x + 6, 6), 0);
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    w.houses[0].credits = 0;
    let (orders, _) = think(&w);
    let medi_t = w.rules().index_of(Kind::Infantry, "MEDI").expect("MEDI");
    let builds = orders
        .iter()
        .filter(|o| matches!(o, Order::Build { what } if what.kind == Kind::Infantry && what.index == medi_t))
        .count();
    assert_eq!(builds, 1, "{orders:?}");
}

// design: own AI (#25) and S-0507 claim 1 (a rider larger than the transport's size limit is refused): a healer the gunner vehicle refuses (too large for it) is not picked for it; the healer that fits is sent instead.
#[test]
fn a_healer_the_vehicle_refuses_is_not_picked() {
    let mut r = ruleset(1);
    let mut big = r.sim.infantry["MEDI"].clone();
    big.size = Some(glasnost_rules::sim::Decimal("3".into()));
    r.sim.infantry.insert("BIGMEDI".into(), big);
    r.registries
        .get_mut("infantry")
        .expect("infantry registry")
        .push("BIGMEDI".into());
    let (mut w, medi, guns) = scene(&r, PER_HEALER);
    // Nearer to the first gunner vehicle than MEDI.
    let x = -fixture::START_DX;
    let big = testkit::spawn(&mut w, Kind::Infantry, "BIGMEDI", 0, fixture::at(x - 4, -3), 0);
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    assert_ne!(
        transport::answer(&w, guns[0], big),
        transport::Answer::Accepts,
        "the vehicle refuses the large healer"
    );
    let (orders, loads) = think(&w);
    assert_eq!(sent_into(&orders, big), None, "{orders:?}");
    let v = sent_into(&orders, medi).expect("MEDI is sent into a gunner vehicle");
    assert!(guns.contains(&v));
    assert_eq!(loads, 1);
}

/// The special abilities' think on `w` for house 0, with `earlier` move
/// orders for `unit` given before it in the same think (the script's rally
/// and the brain's ferry, say).
fn special_think(
    s: &mut glasnost_ai::special::Special,
    w: &World,
    unit: ObjectId,
    earlier: usize,
) -> Vec<Order> {
    let script = glasnost_ai::scripted::Script::new(HouseId(0), fixture::plan());
    let at = w.obj(unit).expect("the unit").pos.cell();
    let mut out: Vec<Order> = (0..earlier)
        .map(|_| Order::Behaviour {
            actor: unit,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(at)),
        })
        .collect();
    s.think(&script, w, &mut out);
    out
}

// design: own AI (#3): a healer with two orders from earlier in the think (the script's and the brain's) is loaded into the gunner vehicle; its earlier orders go and the special step does not panic on its own start index.
#[test]
fn a_healer_with_two_earlier_orders_is_loaded() {
    let (w, medi, guns) = scene(&ruleset(1), PER_HEALER);
    let mut s = glasnost_ai::special::Special::default();
    let out = special_think(&mut s, &w, medi, 2);
    let v = sent_into(&out, medi).expect("MEDI is sent into a gunner vehicle");
    assert!(guns.contains(&v));
    assert_eq!(
        out.iter().filter(|o| o.actor() == Some(medi)).count(),
        1,
        "only the load is left for MEDI: {out:?}"
    );
}

// design: own AI (#3) and the memory of PLAN §4.7: a load given to a healer that had an earlier order in the same think is noted like any special order, so a load that does not take (here: nothing is issued) is given up at the next think instead of being clicked again.
#[test]
fn a_load_after_an_earlier_order_is_noted() {
    let (w, medi, _) = scene(&ruleset(1), PER_HEALER);
    let mut s = glasnost_ai::special::Special::default();
    let first = special_think(&mut s, &w, medi, 1);
    assert!(sent_into(&first, medi).is_some(), "loaded: {first:?}");
    let mut w2 = w.clone();
    testkit::run(&mut w2, 15);
    let again = special_think(&mut s, &w2, medi, 0);
    assert_eq!(s.given_up(), 1, "the load that did not take is given up");
    assert_eq!(
        sent_into(&again, medi),
        None,
        "not clicked again: {again:?}"
    );
}
