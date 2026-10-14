//! Claim tests for the vehicle harvester's harvesting display (S-0402 claim
//! 5a): when it is eligible and when it is suppressed, on synthetic data
//! (the invented harvest ruleset of `harvest_common`, no stock data).

mod harvest_common;

use glasnost_map::CellXY;
use glasnost_sim::econ::harvest::{self, Phase};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::Behaviour;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use harvest_common::*;

fn refinery_cell() -> CellXY {
    at(-6, -1)
}

/// A 3 × 3 field of overlay `o` at stage `d` with its top-left cell at (x, y).
fn field(o: u8, x: i32, y: i32, d: u8) -> Vec<(CellXY, u8, u8)> {
    let mut v = Vec::new();
    for dy in 0..3 {
        for dx in 0..3 {
            v.push((at(x + dx, y + dy), o, d));
        }
    }
    v
}

fn harvest_now(w: &mut World, h: ObjectId) {
    let t = w.tick;
    w.obj_mut(h).unwrap().beh.force(Behaviour::Harvest, t, true);
}

fn until(w: &mut World, max: i32, mut f: impl FnMut(&World) -> bool) -> Option<i32> {
    for _ in 0..max {
        testkit::run(w, 1);
        if f(w) {
            return Some(w.tick);
        }
    }
    None
}

fn total(w: &World, h: ObjectId) -> i32 {
    harvest::state_of(w, h).total()
}

fn moving(w: &World, h: ObjectId) -> bool {
    w.obj(h)
        .unwrap()
        .drive
        .as_ref()
        .is_some_and(|d| d.track.is_some())
}

// spec: S-0402 claim 5a (eligible from the processing that starts loading, before the first bale, through
// the 19-tick wait; still eligible while full until the failing load step changes the phase to returning)
#[test]
fn claim_5a_eligible_from_loading_start_until_the_failing_step() {
    let r = ruleset();
    let mut w = game_with_ore(&r, &field(ORE, 0, 0, 11), 7);
    place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, at(1, 1), 0);
    assert!(!harvest::display_eligible(&w, h));
    harvest_now(&mut w, h);
    // Seeking does not show it.
    let start = until(&mut w, 40, |w| {
        assert_eq!(
            harvest::display_eligible(w, h),
            harvest::phase(w, h) == Phase::Loading
        );
        harvest::phase(w, h) == Phase::Loading
    })
    .expect("loading");
    // At loading start: eligible and shown before any bale.
    assert_eq!(total(&w, h), 0);
    assert!(harvest::display_eligible(&w, h));
    assert!(harvest::display_shown(&w, h));
    // The whole wait before the first bale.
    let first = until(&mut w, 40, |w| {
        if total(w, h) == 0 {
            assert!(harvest::display_shown(w, h));
        }
        total(w, h) == 1
    })
    .expect("first bale");
    assert_eq!(first - start, 19);
    // Full: the filling bale does not end it.
    until(&mut w, 200, |w| {
        assert!(harvest::display_eligible(w, h));
        total(w, h) == CAPACITY
    })
    .expect("full");
    assert_eq!(harvest::phase(&w, h), Phase::Loading);
    assert!(harvest::display_eligible(&w, h));
    let full_at = w.tick;
    // Until the failing step changes the phase to returning.
    let ret = until(&mut w, 100, |w| {
        let loading = harvest::phase(w, h) == Phase::Loading;
        assert_eq!(harvest::display_eligible(w, h), loading);
        !loading
    })
    .expect("returning");
    assert!(ret > full_at);
    assert_eq!(harvest::phase(&w, h), Phase::Returning);
    assert!(!harvest::display_eligible(&w, h));
}

// spec: S-0402 claim 5a (returning, docking and unloading do not show it)
#[test]
fn claim_5a_not_while_returning_docking_or_unloading() {
    let r = ruleset();
    let mut w = game_with_ore(&r, &field(ORE, 0, 0, 11), 7);
    place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, at(1, 1), 0);
    harvest_now(&mut w, h);
    until(&mut w, 300, |w| harvest::phase(w, h) == Phase::Returning).expect("returning");
    let mut saw_unload = false;
    until(&mut w, 1500, |w| {
        let cur = w.obj(h).unwrap().beh.current;
        if cur != Some(Behaviour::Harvest) || harvest::phase(w, h) != Phase::Loading {
            assert!(!harvest::display_eligible(w, h), "{cur:?}");
        }
        saw_unload |= cur == Some(Behaviour::Unload);
        // Back to loading after the delivery.
        saw_unload && harvest::phase(w, h) == Phase::Loading && cur == Some(Behaviour::Harvest)
    })
    .expect("a delivery and a new loading");
    assert!(harvest::display_eligible(&w, h));
}

// spec: S-0402 claim 5a (after an exhausted cell, finding another keeps it eligible, also while driving there
// and before the next bale; movement suppresses it, a stop shows it again)
#[test]
fn claim_5a_drive_on_after_an_exhausted_cell() {
    let r = ruleset();
    // A lone cell of stage 1 (one bale delivered), and a neighbour two cells on.
    let mut w = game_with_ore(&r, &[(at(0, 0), ORE, 1), (at(2, 0), ORE, 5)], 7);
    place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, at(0, 0), 0);
    harvest_now(&mut w, h);
    until(&mut w, 100, |w| total(w, h) == 1).expect("first bale");
    let mut suppressed = 0;
    until(&mut w, 300, |w| {
        assert_eq!(harvest::phase(w, h), Phase::Loading);
        assert!(harvest::display_eligible(w, h));
        if moving(w, h) && harvest::display_suppressed(w, h) {
            suppressed += 1;
        }
        w.obj(h).unwrap().pos.cell() == at(2, 0) && !moving(w, h)
    })
    .expect("arrived");
    assert!(suppressed > 0, "suppressed while driving");
    // Stopped on the new cell, still loading: shown again.
    assert_eq!(harvest::phase(&w, h), Phase::Loading);
    assert!(harvest::display_shown(&w, h));
}

// spec: S-0402 claim 5a (finding no cell and returning with a partial load ends it)
#[test]
fn claim_5a_partial_return_ends_it() {
    let r = ruleset();
    let mut w = game_with_ore(&r, &[(at(0, 0), ORE, 1)], 7);
    place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, at(0, 0), 0);
    harvest_now(&mut w, h);
    until(&mut w, 100, |w| harvest::display_eligible(w, h)).expect("loading");
    until(&mut w, 300, |w| harvest::phase(w, h) == Phase::Returning).expect("returning");
    assert!(total(&w, h) > 0 && total(&w, h) < CAPACITY);
    assert!(!harvest::display_eligible(&w, h));
}

// spec: S-0402 claim 5a (leaving harvest ends it by the vehicle's next turn; merely queueing a replacement
// behaviour does not end loading)
#[test]
fn claim_5a_leaving_harvest_ends_it() {
    let r = ruleset();
    let mut w = game_with_ore(&r, &field(ORE, 0, 0, 11), 7);
    place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, at(1, 1), 0);
    harvest_now(&mut w, h);
    until(&mut w, 40, |w| harvest::phase(w, h) == Phase::Loading).expect("loading");
    // A queued replacement alone does not end it.
    w.obj_mut(h).unwrap().beh.queued = Some(Behaviour::Guard);
    assert!(harvest::display_eligible(&w, h));
    w.obj_mut(h).unwrap().beh.queued = None;
    while w.tick <= w.orders_blocked_through {
        testkit::run(&mut w, 1);
    }
    assert!(orders::issue(&mut w, HouseId(0), Order::Stop { actor: h }));
    until(&mut w, 30, |w| {
        w.obj(h).unwrap().beh.current != Some(Behaviour::Harvest)
    })
    .expect("harvest left");
    assert!(!harvest::display_eligible(&w, h));
    // A new harvest starts in seeking: the old loading phase does not count.
    let t = w.tick;
    w.obj_mut(h).unwrap().beh.force(Behaviour::Harvest, t, true);
    assert_eq!(harvest::phase(&w, h), Phase::Loading);
    assert!(!harvest::display_eligible(&w, h));
}

// spec: S-0402 claim 5a (suppressed during body turning; a destination alone does not suppress it)
#[test]
fn claim_5a_turning_suppresses_and_a_destination_alone_does_not() {
    let r = ruleset();
    let mut w = game_with_ore(&r, &field(ORE, 0, 0, 11), 7);
    place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, at(1, 1), 0);
    harvest_now(&mut w, h);
    until(&mut w, 40, |w| harvest::phase(w, h) == Phase::Loading).expect("loading");
    assert!(harvest::display_shown(&w, h));
    // A destination without a track.
    w.obj_mut(h).unwrap().dest = Some(Target::Cell(at(2, 2)));
    assert!(!harvest::display_suppressed(&w, h));
    w.obj_mut(h).unwrap().dest = None;
    // The body starts turning.
    let t = w.tick;
    let o = w.obj_mut(h).unwrap();
    assert!(o.body.turn_to(16384, t, 6));
    assert!(harvest::display_eligible(&w, h));
    assert!(harvest::display_suppressed(&w, h));
    assert!(!harvest::display_shown(&w, h));
    // Turning done: shown again.
    w.tick += 200;
    assert!(harvest::display_shown(&w, h));
}

// spec: S-0402 claim 5a (the same display for every harvester type and resource kind: a chrono miner and
// gems included)
#[test]
fn claim_5a_every_harvester_and_resource_kind() {
    for (name, o) in [("HARV", GEM), ("CHRONO", ORE), ("CHRONO", GEM)] {
        let r = ruleset();
        let mut w = game_with_ore(&r, &field(o, 0, 0, 11), 7);
        place_building(&mut w, "REFN", 0, refinery_cell());
        let h = place_vehicle(&mut w, name, 0, at(1, 1), 0);
        harvest_now(&mut w, h);
        until(&mut w, 40, |w| harvest::phase(w, h) == Phase::Loading).expect("loading");
        assert!(harvest::display_shown(&w, h), "{name} on {o}");
    }
}

// spec: S-0402 claim 5a (a teleporting harvester: suppressed while a teleport journey is in progress and while
// phasing in; shown again once neither holds)
#[test]
fn claim_5a_teleport_journey_and_phasing_in_suppress_it() {
    use glasnost_sim::teleport::{self, TeleportState};
    use glasnost_sim::world::Pos;
    let mut r = ruleset();
    // The chrono miner with the teleporting locomotor, standing on the field (no journey needed).
    r.sim.vehicles.get_mut("CHRONO").unwrap().locomotor =
        Some(glasnost_rules::sim::Locomotor::Teleport);
    let mut w = game_with_ore(&r, &field(ORE, 0, 0, 11), 7);
    place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "CHRONO", 0, at(1, 1), 0);
    assert!(teleport::is_teleporter(&w, h));
    harvest_now(&mut w, h);
    until(&mut w, 40, |w| harvest::phase(w, h) == Phase::Loading).expect("loading");
    assert!(harvest::display_shown(&w, h));
    let saved = w.obj(h).unwrap().teleport.clone();
    let check = |w: &World| {
        assert!(harvest::display_eligible(w, h));
        assert!(harvest::display_suppressed(w, h));
        assert!(!harvest::display_shown(w, h));
    };
    // A journey in progress: a landing point fixed and not yet jumped to.
    let landing = Pos::centre(at(2, 2), 0);
    w.obj_mut(h).unwrap().teleport = Some(TeleportState {
        landing: Some(landing),
        ..Default::default()
    });
    check(&w);
    // Phasing in after a jump.
    let t = w.tick;
    w.obj_mut(h).unwrap().teleport = Some(TeleportState {
        phasing: Some((t, 10)),
        ..Default::default()
    });
    assert!(teleport::phasing_in(&w, h));
    check(&w);
    // Neither: shown again.
    w.obj_mut(h).unwrap().teleport = saved;
    assert!(harvest::display_shown(&w, h));
}
