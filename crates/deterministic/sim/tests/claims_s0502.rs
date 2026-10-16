//! Claim tests of S-0502 (movement of jumpjets) on synthetic rules: invented
//! types and numbers, no game data. Each test cites the claim it checks.

mod jump_common;

use glasnost_num::{F64, Rounding};
use glasnost_sim::jump::{self, JumpState, Phase};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, World};
use jump_common::*;

fn j(w: &World, id: ObjectId) -> &JumpState {
    w.obj(id).unwrap().jump.as_ref().unwrap()
}

fn mode(w: &World) -> Rounding {
    w.rules().mode
}

fn fl(w: &World, v: i32) -> F64 {
    F64::from_i64(i64::from(v), mode(w))
}

/// A double from a ratio of two integers (exact in the cases used).
fn ratio(w: &World, a: i64, b: i64) -> F64 {
    F64::from_i64(a, mode(w)).div(F64::from_i64(b, mode(w)), mode(w))
}

/// Sets the current and maximum speeds, each given as a ratio.
fn set_speeds(w: &mut World, id: ObjectId, vc: (i64, i64), vm: (i64, i64)) {
    let (a, b) = (ratio(w, vc.0, vc.1), ratio(w, vm.0, vm.1));
    let o = w.obj_mut(id).unwrap().jump.as_mut().unwrap();
    o.vc = a;
    o.vm = b;
}

/// A jumpjet in flight, cruising at 500 above the surface towards a far
/// destination to the east.
fn cruiser(w: &mut World, name: &str, kind: Kind) -> ObjectId {
    let u = put(w, name, kind, 0, -16, 0);
    go(w, u, at(16, 0));
    jump::start_in_flight(w, u, Phase::Cruising, 500);
    u
}

// spec: S-0502 claim 1 (each unit copies its type's jumpjet fields; absent keys take the engine defaults; H is the cruise height after S-0102's minimum of 208)
#[test]
fn claim_1_fields_and_defaults() {
    let w = game();
    let mut c = w.clone();
    let u = put(&mut c, "JJ", Kind::Infantry, 0, 0, 0);
    let f = jump::fields(&c, u).unwrap();
    assert_eq!(
        (f.speed, f.height, f.deviation, f.turn_field),
        (14, 500, 40, 4)
    );
    assert!(f.accel.eq_num(fl(&w, 2)) && f.climb.eq_num(fl(&w, 5)) && f.crash.eq_num(fl(&w, 6)));
    // A type that sets no field has the defaults; a small height is raised to 208.
    let mut r = ruleset();
    r.sim.vehicles.get_mut("JV").unwrap().jumpjet_height = Some(100);
    let bare = r.sim.vehicles.get_mut("DEPN").unwrap();
    *bare = glasnost_rules::sim::UnitType {
        locomotor: Some(glasnost_rules::sim::Locomotor::Jumpjet),
        ..Default::default()
    };
    let mut w = testkit::game_with(&r, SIZE, 7);
    testkit::run(&mut w, 4);
    let low = put(&mut w, "JV", Kind::Vehicle, 0, 0, 0);
    assert_eq!(jump::fields(&w, low).unwrap().height, 208);
    let d = put(&mut w, "DEPN", Kind::Vehicle, 0, 2, 0);
    let f = jump::fields(&w, d).unwrap();
    assert_eq!(
        (f.speed, f.height, f.deviation, f.turn_field),
        (14, 500, 40, 4)
    );
    assert!(f.accel.eq_num(fl(&w, 2)) && f.climb.eq_num(fl(&w, 5)) && f.crash.eq_num(fl(&w, 5)));
    // 0.15 in single precision, widened.
    let want = glasnost_rules::sim::Decimal("0.15".into()).single_value(mode(&w));
    assert!(f.wobbles.eq_num(want));
    // The movement heading of a new jumpjet is east (S-0206 claim 14).
    assert_eq!(j(&w, d).heading.value(w.tick), 16384);
    assert_eq!(f.omega, 4);
}

// spec: S-0502 claim 3 (a jumpjet that is neither moving nor moving now takes no update: its height, wobble phase and speed fraction keep their values)
#[test]
fn claim_3_idle_jumpjets_take_no_update() {
    let mut w = game();
    let g = put(&mut w, "JJ", Kind::Infantry, 0, -4, 0);
    let h = put(&mut w, "JJ", Kind::Infantry, 0, 4, 0);
    jump::start_in_flight(&mut w, h, Phase::Hovering, 500);
    let before = (j(&w, g).clone(), j(&w, h).clone(), pos(&w, h));
    testkit::run(&mut w, 30);
    assert_eq!(&before.0, j(&w, g));
    assert_eq!(&before.1, j(&w, h));
    assert_eq!(
        before.2,
        pos(&w, h),
        "a hovering unit with no order does not wobble"
    );
    assert!(!jump::moving_now(&w, g) && !jump::moving_now(&w, h));
}

// spec: S-0502 claim 5 and scenario J2 (a grounded jumpjet's first update with a move order: speed fraction 1, speeds 0, target height H, ascending; height first rises in the following update by trunc(z + c) − z)
#[test]
fn claim_5_take_off() {
    let mut w = game();
    let u = put(&mut w, "JJ", Kind::Infantry, 0, -8, 0);
    let t0 = w.tick;
    go(&mut w, u, at(2, 0));
    assert!(jump::moving(&w, u) && phase(&w, u) == Phase::Grounded);
    step(&mut w);
    assert_eq!(w.tick, t0 + 1);
    let s = j(&w, u);
    assert_eq!(s.phase, Phase::Ascending);
    assert_eq!(s.ht, 500);
    assert!(s.fraction.eq_num(fl(&w, 1)) && s.vc.eq_num(F64::ZERO) && s.vm.eq_num(F64::ZERO));
    assert_eq!(height(&w, u), 0, "no rise in the grounded update");
    step(&mut w);
    assert_eq!(
        height(&w, u),
        5,
        "the climb rate c of 5.0 in the following update"
    );
    step(&mut w);
    assert_eq!(height(&w, u), 10);
}

// spec: S-0502 claim 6 and F1 (the current speed moves towards the maximum by a per tick, not beyond S, and down by 1.5 a, not below 0; braking also applies in the step in which acceleration overshoots the maximum; the speed fraction is v_c / S)
#[test]
fn claim_6_f1_speed() {
    let mut w = game();
    let u = cruiser(&mut w, "JJ", Kind::Infantry);
    set_speeds(&mut w, u, (0, 1), (14, 1));
    let mut seen = Vec::new();
    for _ in 0..9 {
        step(&mut w);
        seen.push(j(&w, u).vc);
    }
    let want: Vec<F64> = [2, 4, 6, 8, 10, 12, 14, 14, 14]
        .iter()
        .map(|&v| fl(&w, v))
        .collect();
    assert_eq!(seen, want, "a per tick to S");
    assert!(j(&w, u).fraction.eq_num(fl(&w, 1)));
    // Braking by 1.5 × 2 = 3 per tick down to 0.
    set_speeds(&mut w, u, (14, 1), (0, 1));
    let mut seen = Vec::new();
    for _ in 0..6 {
        // The cruise outcome sets the maximum speed again each tick: hold it at 0.
        w.obj_mut(u).unwrap().jump.as_mut().unwrap().vm = F64::ZERO;
        step(&mut w);
        seen.push(j(&w, u).vc);
    }
    let want: Vec<F64> = [11, 8, 5, 2, 0, 0].iter().map(|&v| fl(&w, v)).collect();
    assert_eq!(seen, want);
    // The tick in which acceleration overshoots the maximum speed: v_c 13, v_m 13.5, S 14: q = 14, then 14 − 3 = 11.
    set_speeds(&mut w, u, (13, 1), (27, 2));
    step(&mut w);
    assert!(j(&w, u).vc.eq_num(fl(&w, 11)));
    // The fraction: 11 / 14.
    assert!(j(&w, u).fraction.eq_num(ratio(&w, 11, 14)));
    // Equal speeds: unchanged.
    set_speeds(&mut w, u, (7, 1), (7, 1));
    step(&mut w);
    assert!(j(&w, u).vc.eq_num(fl(&w, 7)));
}

// spec: S-0502 claim 7 and scenario J2 (ascending: past one quarter of the target height, one half for a balloon-hover vehicle, the maximum speed becomes S and the heading starts turning; at the target height it claims the slot and hovers)
#[test]
fn claim_7_ascending_thresholds() {
    for (name, kind, quarter) in [
        ("JJ", Kind::Infantry, 125),
        ("BALLOON", Kind::Vehicle, 250),
        ("JV", Kind::Vehicle, 125),
    ] {
        let mut w = game();
        let u = put(&mut w, name, kind, 0, -8, 0);
        go(&mut w, u, at(8, 0));
        let mut first = None;
        for _ in 0..200 {
            step(&mut w);
            if first.is_none() && j(&w, u).vm.gt(F64::ZERO) {
                first = Some(height(&w, u));
            }
        }
        // The maximum speed is set in the update in which h first exceeds the threshold; the height there is the first multiple of c above it.
        let h = first.expect("the maximum speed is set");
        assert!(h > quarter && h <= quarter + 5, "{name}: {h}");
    }
}

// spec: S-0502 claim 7 and scenario J2 (revision 6: when the accepted movement point is in the ascent cell, past the quarter it stops its climb at its present height and cruises, away from a bridge; a balloon-hover vehicle continues to H)
#[test]
fn claim_7_own_cell_early_cruise() {
    let mut w = game();
    // An entering vehicle keeps the exact ordered point (claim 16), so its accepted point is in its own cell.
    let u = put(&mut w, "JV", Kind::Vehicle, 0, 0, 0);
    let b = put(&mut w, "BALLOON", Kind::Vehicle, 0, 4, 0);
    go_here_entering(&mut w, u);
    go(&mut w, b, at(4, 0));
    let mut early = None;
    for _ in 0..140 {
        step(&mut w);
        if early.is_none() && phase(&w, u) == Phase::Cruising {
            early = Some((height(&w, u), j(&w, u).ht));
        }
    }
    let (h, ht) = early.expect("an own-cell order cruises early");
    assert!(
        h > 125 && h < 140,
        "it stops climbing past the quarter, at {h}"
    );
    assert_eq!(ht, h, "the target height is the present height r");
    // The balloon-hover vehicle keeps its target height H and climbs on.
    assert_eq!(j(&w, b).ht, 500);
    assert!(
        height(&w, b) > h - 20,
        "the balloon-hover vehicle climbs on past the early-cruise height"
    );
    for _ in 0..200 {
        step(&mut w);
    }
    assert_eq!(
        phase(&w, b),
        Phase::Hovering,
        "a balloon-hover vehicle hovers over its destination"
    );
    assert!(jump::moving(&w, b));
}

// spec: S-0502 claim 8, F4 and scenario J2 (ground hold: a jumpjet away from its destination moves sideways only once its height above the reference is at least half its desired height)
#[test]
fn claim_8_ground_hold() {
    let mut w = game();
    let u = put(&mut w, "JJ", Kind::Infantry, 0, -8, 0);
    go(&mut w, u, at(8, 0));
    let start = pos(&w, u);
    let mut first_sideways = None;
    for _ in 0..120 {
        step(&mut w);
        if first_sideways.is_none() && pos(&w, u).x != start.x {
            first_sideways = Some(height(&w, u));
        }
    }
    let h = first_sideways.expect("it moves sideways");
    assert!(
        h >= 250,
        "sideways motion starts at half the desired height or more, at {h}"
    );
    assert!(h < 270);
}
