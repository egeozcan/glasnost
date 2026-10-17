//! Claim tests for S-0215 revision 59 (claim 11: the late start's
//! fall-back to rest, which answers Q-1166 question 1), on synthetic data:
//! the chrono-weapon fixture `tests/temporal_common`.

#[path = "temporal_common/mod.rs"]
mod temporal_common;

use glasnost_sim::process;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, World};
use temporal_common::*;

const BETA: u8 = 1;

/// A unit of BETA, standing, with both behaviour slots emptied.
fn emptied(w: &mut World, kind: Kind, name: &str) -> ObjectId {
    let id = spawn(w, kind, name, BETA, at(0, 0));
    testkit::run(w, 4);
    let ob = w.obj_mut(id).unwrap();
    ob.beh.current = None;
    ob.beh.queued = None;
    id
}

// spec: S-0215 claim 11 (revision 59: an infantry unit that is ready at the late start with neither a current nor a queued behaviour falls back to rest in that same update; the behaviour the fall-back queues becomes current at that same late start, not processed until the next update)
#[test]
fn claim_11_late_start_falls_back_to_rest() {
    let mut w = game(&ruleset());
    let u = emptied(&mut w, Kind::Infantry, "BYSTANDER");
    process::late_start(&mut w, u);
    let o = w.obj(u).unwrap();
    assert!(o.beh.current.is_some(), "the fall-back's behaviour is current");
    assert_eq!(o.beh.queued, None, "not left queued");
    assert!(o.beh.expired, "made current, not processed");
}

// spec: S-0215 claim 11 (the fall-back to rest at the late start is an infantry rule: a vehicle with both slots empty is left as it is)
#[test]
fn claim_11_no_late_fall_back_for_a_vehicle() {
    let mut w = game(&ruleset());
    let v = emptied(&mut w, Kind::Vehicle, "VICT");
    process::late_start(&mut w, v);
    let o = w.obj(v).unwrap();
    assert_eq!((o.beh.current, o.beh.queued), (None, None));
}
