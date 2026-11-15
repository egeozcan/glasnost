//! S-0519 claim 19 and S-0203 claim 7 (#385): a designator whose target is
//! destroyed outside the turn pass and the order loop (here in the houses'
//! part of the tick) has its strike cancelled in that tick. Synthetic rules
//! (`tests/terror_common`).

#[path = "terror_common/mod.rs"]
mod terror_common;

use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::Target;
use terror_common::*;

// spec: S-0519 claim 19 and S-0203 claim 7 (a designator's lost target cancels its strike when the change applies, in the tick of the cancelling event, not at a shared strike pass)
#[test]
fn a_target_lost_in_the_house_update_cancels_in_that_tick() {
    let mut w = world(|_| {});
    let d = quiet(&mut w, Kind::Vehicle, "DESIG", 0, at(0, 0));
    let v = quiet(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    run_to(&mut w, 100);
    designate(&mut w, d, v);
    let jets = linked(&w, d);
    assert!(!jets.is_empty());
    assert!(w.strike.strikes[&d].active);
    assert_eq!(w.strike.strikes[&d].target, Some(v));
    assert_eq!(w.obj(d).unwrap().target, Some(Target::Object(v)));

    // House 1 is marked to die (a surrender applied in the tick before,
    // S-0414 claim 11): its objects are destroyed in the houses' part of
    // this tick, after the turn pass.
    w.econ.end.active = true;
    w.houses[1].econ.result.doomed = true;
    assert!(w.houses[0].pending.is_empty() && w.houses[1].pending.is_empty());
    let t = w.tick;
    testkit::run(&mut w, 1);
    assert_eq!(w.tick, t + 1, "one tick ran");
    assert!(
        w.obj(v).is_none_or(|o| o.health <= 0),
        "the target was destroyed"
    );
    assert_eq!(
        w.obj(d).unwrap().target,
        None,
        "the designator lost its target"
    );
    let s = w.strike.strikes.get(&d).expect("the designator lives");
    assert!(!s.active, "the strike is no longer active");
    assert_eq!(s.target, None, "the strike target is forgotten");
    for &j in &jets {
        assert_eq!(
            w.obj(j).and_then(|o| o.target),
            None,
            "the strike aircraft lost their target in that tick"
        );
    }
}
