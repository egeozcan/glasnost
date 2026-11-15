//! `Replay::run_on` on a loaded save that already holds the orders of its
//! tick as pending (#333): they are not issued a second time. Synthetic
//! data only: the testkit's ruleset with an invented building type HUT.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::BuildingType;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Kind;
use glasnost_sim::save::{self, Replay};
use glasnost_sim::testkit;
use glasnost_sim::tick;
use glasnost_sim::world::{HouseId, ObjectId, World};

const SIZE: i32 = 24;

fn rules() -> Ruleset {
    let mut r = testkit::ruleset();
    r.registries
        .insert("buildings".into(), vec!["HUT".to_string()]);
    r.sim.buildings.insert(
        "HUT".into(),
        BuildingType {
            strength: Some(400),
            foundation: Some([1, 1]),
            ..Default::default()
        },
    );
    r
}

fn pending(w: &World) -> usize {
    w.houses.iter().map(|h| h.pending.len()).sum()
}

/// A game with a damaged HUT of house 0, and a replay that records one
/// repair order for it (a switch: on if off, off if on, S-0213 claim 13) in
/// the world's current tick.
fn game() -> (World, ObjectId, Replay) {
    let mut w = testkit::game_with(&rules(), SIZE, 13);
    testkit::run(&mut w, 4);
    let hut = testkit::spawn(
        &mut w,
        Kind::Building,
        "HUT",
        0,
        testkit::mid(SIZE, 0, 0),
        0,
    );
    w.obj_mut(hut).unwrap().health = 200;
    testkit::run(&mut w, 1);
    let pb = w.rules().combat.general.repair_period;
    // Off a repair-period multiple, so that no repair step can turn it off.
    while w.tick % pb == 0 || (w.tick + 1) % pb == 0 {
        tick::step(&mut w);
    }
    let mut rep = Replay::new(w.rules(), "synthetic", testkit::setup(13));
    rep.record(w.tick, HouseId(0), Order::Repair { actor: hut });
    rep.ticks = glasnost_sim::tick::ticks_run(&w) + 5;
    (w, hut, rep)
}

// design: #333 (run_on on a loaded save taken after its tick's recorded orders were issued does not issue them again)
#[test]
fn issue_333_run_on_a_save_holding_its_ticks_orders_does_not_issue_them_again() {
    let (mut w, hut, rep) = game();
    assert!(!w.obj(hut).unwrap().econ.building.repairing);
    // The live game: the tick's orders are issued, then the game is saved
    // before the tick is stepped.
    rep.issue_due(&mut w);
    assert_eq!(pending(&w), 1, "the order is pending");
    let mut l = save::load(&save::save(&w), w.rules_arc(), w.catalogue_arc()).unwrap();
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
    assert_eq!(pending(&l), 1, "the save holds the pending order");
    // The live game goes on: the tick is stepped, the replay continues.
    tick::step(&mut w);
    assert!(
        w.obj(hut).unwrap().econ.building.repairing,
        "live: the repair switch is on"
    );
    let mut a = vec![save::state_hash(&w)];
    rep.run_on(&mut w, rep.ticks, |w| a.push(save::state_hash(w)));
    // The loaded save continues under run_on.
    let mut b = Vec::new();
    let mut first = None;
    rep.run_on(&mut l, rep.ticks, |w| {
        first.get_or_insert(w.obj(hut).unwrap().econ.building.repairing);
        b.push(save::state_hash(w));
    });
    assert_eq!(first, Some(true), "the order was applied once, not twice");
    assert_eq!(a, b, "the loaded save continues as the live game");
}

// design: #333 (a world at a tick boundary, with no pending orders, still has its tick's orders issued)
#[test]
fn issue_333_run_on_at_a_tick_boundary_issues_the_ticks_orders() {
    let (mut w, hut, rep) = game();
    assert_eq!(pending(&w), 0);
    let until = glasnost_sim::tick::ticks_run(&w) + 1;
    rep.run_on(&mut w, until, |_| {});
    assert!(w.obj(hut).unwrap().econ.building.repairing);
}
