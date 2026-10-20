//! Claim tests of S-0102 claim 4 (the moments at which a mobile object's
//! render layer is evaluated), as the simulation reports them to the
//! drawing (`Effect::LayerMoment`), on the synthetic jumpjet ruleset.

mod jump_common;

use glasnost_num::F64;
use glasnost_sim::combat::effects::{Effect, LayerHeights};
use glasnost_sim::fall;
use glasnost_sim::jump::{self, Phase};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{ObjectId, World};
use jump_common::*;

/// The moments of `id` in the effects: `None` for a re-file, the heights
/// before and after for a check.
fn moments(w: &World, id: ObjectId) -> Vec<Option<(LayerHeights, LayerHeights)>> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::LayerMoment { object, check } if *object == id => Some(*check),
            _ => None,
        })
        .collect()
}

/// One tick, with the effects of the tick only.
fn tick(w: &mut World) {
    w.effects.clear();
    step(w);
}

// spec: S-0102 claim 4 (a jumpjet's movement step is a check with the heights evaluated at the step's start and after it; a jumpjet with no movement step reports nothing)
#[test]
fn claim_4_jumpjet_steps_are_checks() {
    let mut w = game();
    let u = put(&mut w, "JJ", Kind::Infantry, 0, 0, 0);
    let idle = put(&mut w, "JJ", Kind::Infantry, 0, 0, 4);
    go(&mut w, u, at(12, 0));
    let mut climbing = 0;
    for _ in 0..120 {
        let r0 = height(&w, u);
        tick(&mut w);
        assert!(moments(&w, idle).is_empty(), "an idle jumpjet");
        let m = moments(&w, u);
        assert!(m.len() <= 1, "one step a tick: {m:?}");
        if let Some(Some((before, after))) = m.first() {
            assert_eq!(before.r, r0, "before: the height at the step's start");
            assert_eq!(after.r, height(&w, u), "after: the height after the step");
            if after.r > before.r {
                climbing += 1;
            }
        }
    }
    assert!(climbing > 0, "climbing steps were reported");
}

// spec: S-0102 claim 4 (every tick of a jumpjet's crash is a re-file)
#[test]
fn claim_4_jumpjet_crash_ticks_are_refiles() {
    let mut w = game();
    let u = put(&mut w, "JV", Kind::Vehicle, 0, -4, 0);
    go(&mut w, u, at(10, 0));
    let t = w.tick;
    w.obj_mut(u).unwrap().beh.force(Behaviour::Move, t, true);
    jump::start_in_flight(&mut w, u, Phase::Cruising, 500);
    w.obj_mut(u).unwrap().jump.as_mut().unwrap().vc = F64::from_i64(14, w.rules().mode);
    assert!(jump::start_crash(&mut w, u, None));
    let mut crashing = 0;
    for _ in 0..80 {
        if w.obj(u).is_none() {
            break;
        }
        tick(&mut w);
        if w.obj(u).is_some_and(|_| phase(&w, u) == Phase::Crashing) {
            assert_eq!(moments(&w, u), vec![None], "a crash tick");
            crashing += 1;
        }
    }
    assert!(crashing > 1, "crash ticks were seen");
}

// spec: S-0102 claim 4 (the start of a dropped explosive's fall is a re-file; each falling update is a check with the heights before and after it)
#[test]
fn claim_4_fall_start_and_fall_checks() {
    let mut w = game();
    let u = put(&mut w, "JJ", Kind::Infantry, 0, 0, 0);
    w.obj_mut(u).unwrap().pos.z += 900;
    w.effects.clear();
    fall::start(&mut w, u, true);
    assert_eq!(moments(&w, u), vec![None], "the fall start");
    // A second start while falling is no new moment.
    w.effects.clear();
    fall::start(&mut w, u, true);
    assert!(moments(&w, u).is_empty());
    let r0 = height(&w, u);
    tick(&mut w);
    let checks: Vec<_> = moments(&w, u).into_iter().flatten().collect();
    assert_eq!(checks.len(), 1, "one falling update");
    assert_eq!(checks[0].0.r, r0);
    assert!(checks[0].1.r <= r0);
}
