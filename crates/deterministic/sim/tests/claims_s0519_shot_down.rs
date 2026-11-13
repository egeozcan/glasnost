//! Claim test for S-0519 claim 20 (#161): a strike aircraft stops being
//! linked when it is destroyed, at its killing hit, not when its wreck
//! leaves play. Synthetic data (`terror_common`: a designator with a strike
//! of 2 aircraft and a recharge of 100 ticks).

#[path = "terror_common/mod.rs"]
mod terror_common;

use glasnost_sim::air;
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::objects;
use glasnost_sim::rules::Kind;
use glasnost_sim::strike;
use glasnost_sim::testkit;
use glasnost_sim::world::Target;
use terror_common::*;

// spec: S-0519 claim 20 (a strike aircraft stops being linked when it is destroyed or removed from play; when the last one stops being linked the strike becomes idle and its end tick is that tick) and claims 16 to 18 (an idle strike starts for an object that is not its strike target)
#[test]
fn a_shot_down_strike_aircraft_is_unlinked_at_its_killing_hit() {
    let mut w = world(|_| {});
    let a = quiet(&mut w, Kind::Building, "BLDG", 1, at(0, 0));
    let b = quiet(&mut w, Kind::Building, "BLDG", 1, at(4, 4));
    let d = quiet(&mut w, Kind::Vehicle, "DESIG", 0, at(-6, 0));
    designate(&mut w, d, a);
    let jets = linked(&w, d);
    assert_eq!(jets.len(), 2);
    // Let the aircraft climb.
    let t0 = w.tick;
    while air::height(&w, jets[0]) < 200 && w.tick < t0 + 60 {
        testkit::run(&mut w, 1);
    }
    assert!(air::height(&w, jets[0]) > 0, "the aircraft is in the air");
    assert_eq!(linked(&w, d).len(), 2, "both still linked");
    // One aircraft leaves play without a kill; the other is shot down.
    objects::remove(&mut w, jets[1]);
    let x = jets[0];
    let warhead = w.rules().combat.general.c4_warhead;
    let amount = w.obj(x).unwrap().health;
    damage::apply(
        &mut w,
        x,
        Hit {
            amount,
            warhead,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: true,
        },
    );
    let k = w.tick;
    let o = w.obj(x).expect("the wreck stays in play while it falls");
    assert!(damage::dead(o) && air::crashing(&w, x), "destroyed, crashing");
    let s = w.strike.strikes[&d].clone();
    assert!(!strike::linked(&w, x), "claim 20: unlinked at its killing hit");
    assert!(
        !s.progress && s.end == k && s.ended,
        "claim 20: idle at the kill tick {k} with e = {k}; got progress {} end {}",
        s.progress,
        s.end
    );
    assert_eq!(s.target, Some(a), "it keeps its strike target");
    // The designator is ordered onto B and its designation shot detonates:
    // the idle strike starts for B with new aircraft.
    w.obj_mut(d).unwrap().target = Some(Target::Object(b));
    strike::designation(&mut w, Some(d), Some(Target::Object(b)));
    let s2 = w.strike.strikes[&d].clone();
    let live: Vec<_> = s2
        .aircraft
        .iter()
        .filter(|&&j| w.obj(j).is_some_and(|o| !damage::dead(o)))
        .collect();
    assert_eq!(
        live.len(),
        2,
        "claims 16 to 18: the idle strike starts for B with 2 new aircraft"
    );
    assert!(!s2.aircraft.contains(&x), "the wreck is not linked again");
    assert_eq!(s2.target, Some(b));
    assert_ne!(
        w.obj(x).and_then(|o| o.target),
        Some(Target::Object(b)),
        "the wreck is not redirected"
    );
    // The wreck's removal at its impact moves neither the end tick nor the new strike.
    let t1 = w.tick;
    while w.obj(x).is_some() && w.tick < t1 + 400 {
        testkit::run(&mut w, 1);
    }
    assert!(w.obj(x).is_none(), "the wreck has landed");
    let s3 = w.strike.strikes[&d].clone();
    assert_eq!(s3.end, k, "the impact does not move e");
    assert_eq!(s3.target, Some(b));
}
