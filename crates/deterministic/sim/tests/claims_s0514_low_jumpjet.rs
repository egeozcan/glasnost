//! Claim test for S-0514 claim 1 (#213): for a jumpjet the ground-layer
//! test of an erasing delivery is S-0102's render-layer test, so a jumpjet
//! 1 to 207 leptons up is in the ground layer and is locked. Synthetic data
//! (`temporal_common`); every name and number is invented.

#[path = "temporal_common/mod.rs"]
mod temporal_common;

use glasnost_rules::sim::{Locomotor, MovementZone};
use glasnost_sim::jump::{self, Phase};
use glasnost_sim::rules::Kind;
use glasnost_sim::temporal;
use glasnost_sim::world::Target;
use temporal_common::*;

/// (listing predicate, `temporal::ground_layer`, locked after a delivery)
fn at_height(h: i32) -> (bool, bool, bool) {
    let mut w = world(|r| {
        infantry(r, "JJT", |t| {
            t.locomotor = Some(Locomotor::Jumpjet);
            t.movement_zone = Some(MovementZone::Fly);
            t.jumpjet = Some(true);
            t.jumpjet_speed = Some(14);
            t.jumpjet_climb = Some(d("5.0"));
            t.jumpjet_crash = Some(d("6.0"));
            t.jumpjet_height = Some(500);
            t.jumpjet_accel = Some(d("2.0"));
            t.jumpjet_no_wobbles = Some(true);
            t.jumpjet_turn_rate = Some(4);
        })
    });
    let u = spawn(&mut w, Kind::Infantry, "JJT", 1, at(0, 0));
    let a = attacker(&mut w, "ERASER", 0, at(-3, -3));
    assert!(jump::is_jumpjet(&w, u), "a unit with jumpjet movement");
    if h > 0 {
        jump::start_in_flight(&mut w, u, Phase::Ascending, h);
    }
    assert_eq!(w.height_above_surface(u), h);
    let qualifies = jump::placement_qualifies(&w, u);
    let ground = temporal::ground_layer(&w, u);
    temporal::deliver(&mut w, Some(a), Some(Target::Object(u)));
    (qualifies, ground, erased(&w, u))
}

// spec: S-0514 claim 1 (for a jumpjet the ground-layer test is S-0102 claim 3's render-layer test) and S-0102 State and ordering (r < 208 leptons or r' = 0: flat-land heights 1, 50 and 100 evaluate to ground, 208 and 250 do not)
#[test]
fn a_low_jumpjet_is_in_the_ground_layer_for_the_erasure_test() {
    for h in [0, 1, 50, 100, 207, 208, 250] {
        let (qualifies, ground, locked) = at_height(h);
        let want = h < 208;
        assert_eq!(qualifies, want, "the two-height predicate at {h}");
        assert_eq!(ground, want, "temporal::ground_layer at {h}");
        assert_eq!(locked, want, "the lock at {h}");
    }
}
