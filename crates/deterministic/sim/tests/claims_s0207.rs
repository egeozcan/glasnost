//! Claim tests for ground vehicle movement, the drive locomotor (S-0207), on
//! synthetic data: invented vehicle types and values on the flat scene of
//! `common`. The vehicle's path steps are set directly after the move order,
//! so each test controls which tracks are driven.

mod common;

use common::*;
use glasnost_map::CellXY;
use glasnost_num::F64;
use glasnost_num::decimal::nearest_double;
use glasnost_rules::sim::{Decimal, LandRow, LandType, MovementZone, OverlayType, UnitType};
use glasnost_sim::drive::Track;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::regions::STEPS;
use glasnost_sim::rules::Kind;
use glasnost_sim::tick;
use glasnost_sim::tracks;
use glasnost_sim::world::{HouseId, ObjectId, World};

// 8-directions (S-0207 Terms).
const N: u8 = 0;
const NE: u8 = 1;
const E: u8 = 2;
const SE: u8 = 3;

/// 1.0, 0.5, 0.375 and 0.25 as doubles (exact).
const ONE: F64 = F64(0x3ff0_0000_0000_0000);
const HALF: F64 = F64(0x3fe0_0000_0000_0000);
const THREE_EIGHTHS: F64 = F64(0x3fd8_0000_0000_0000);
const QUARTER: F64 = F64(0x3fd0_0000_0000_0000);
/// F₀.₃: single-precision 0.3 widened to double.
// spec: S-0207 Constants (braking floor 0.300000011920928955078125)
const F03: F64 = F64(0x3fd3_3333_4000_0000);

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

/// A drive vehicle type; `accel` is the acceleration flag.
fn car(class: glasnost_rules::sim::LocomotionClass, speed: i32, accel: bool) -> UnitType {
    let mut t = unit_type(class, speed, 5);
    t.accelerates = Some(accel);
    t
}

/// A scene with the vehicle type TESTCAR placed at the middle cell, facing
/// map north (facing value 0) unless `facing` says otherwise.
fn scene(t: UnitType, facing: i32) -> Scene {
    let mut s = Scene::new();
    s.vehicle("TESTCAR", t);
    s.place_unit(0, "TESTCAR", cell(0, 0), facing);
    s
}

/// The cell reached from the middle cell by `steps`.
fn end_cell(steps: &[u8]) -> CellXY {
    let mut c = cell(0, 0);
    for &d in steps {
        let (dx, dy) = STEPS[usize::from(d)];
        c = CellXY::new(c.x + dx, c.y + dy);
    }
    c
}

/// Builds the scene, orders the first vehicle to the end of `steps`, runs
/// one tick and sets its path steps to `steps`.
fn start(s: &Scene, steps: &[u8]) -> (World, ObjectId) {
    let mut w = s.build();
    let id = objects_of(&w, Kind::Vehicle)[0];
    open_orders(&mut w);
    order_move(&mut w, id, end_cell(steps));
    tick::step(&mut w);
    w.obj_mut(id).expect("vehicle").path.steps = steps.to_vec();
    (w, id)
}

/// The drive state of a vehicle after a tick.
#[derive(Clone, Debug)]
struct Snap {
    pos: (i32, i32, i32),
    budget: i32,
    rho: F64,
    rho_t: F64,
    track: Option<Track>,
    facing: u16,
    steps: usize,
}

fn snap(w: &World, id: ObjectId) -> Snap {
    let o = w.obj(id).expect("vehicle");
    let d = o.drive.as_ref().expect("drive state");
    Snap {
        pos: (o.pos.x, o.pos.y, o.pos.z),
        budget: d.budget,
        rho: d.rho,
        rho_t: d.rho_t,
        track: d.track.clone(),
        facing: o.body.value(w.tick),
        steps: o.path.steps.len(),
    }
}

fn step(w: &mut World, id: ObjectId) -> Snap {
    tick::step(w);
    snap(w, id)
}

/// Runs until the vehicle has begun its first track; returns that tick's
/// state.
fn first_moving_tick(w: &mut World, id: ObjectId) -> Snap {
    for _ in 0..100 {
        let sn = step(w, id);
        if sn.track.is_some() {
            return sn;
        }
    }
    panic!("the vehicle never began a track");
}

/// Track cost by curve, identified by its number of points (S-0207 F3).
fn cost(t: &Track) -> i32 {
    let (pts, _) = tracks::track(t.d, t.e);
    // spec: S-0207 F3 (A 163, B 227, C 388, D 273, E 437, F 399)
    match pts.len() {
        23 => 163,
        31 => 227,
        54 => 388,
        38 => 273,
        61 => 437,
        56 => 399,
        n => panic!("unknown track of {n} points"),
    }
}

/// Progress accounting: tracks finished so far are summed at their cost,
/// plus 7 per point passed on the current track, plus the budget.
struct Ledger {
    finished: i32,
    last: Option<Track>,
}

impl Ledger {
    fn new() -> Self {
        Ledger {
            finished: 0,
            last: None,
        }
    }

    fn total(&mut self, sn: &Snap) -> i32 {
        if let Some(prev) = &self.last
            && sn.track.as_ref().is_none_or(|t| t.target != prev.target)
        {
            self.finished += cost(prev);
        }
        self.last = sn.track.clone();
        let on = sn.track.as_ref().map_or(0, |t| 7 * t.next as i32);
        self.finished + on + sn.budget
    }
}

fn set_land(s: &mut Scene, land: LandType, wheel: &str, track: &str) {
    s.ruleset.sim.land_types.insert(
        land,
        LandRow {
            wheel: dec(wheel),
            track: dec(track),
            ..Default::default()
        },
    );
}

// ---------------------------------------------------------------------------
// Progress per tick

// spec: S-0207 claim 2 (s = trunc(trunc(V × κ × b) × ρ), truncated after each product)
// spec: S-0207 F2
#[test]
fn progress_per_tick_truncates_country_factor_then_fraction() {
    // v = 10 gives V = 25 (claim 1). κ = 0.99 (single) gives s₁ = trunc(24.75…)
    // = 24; ρ = 0.9 (land multiplier, acceleration off) gives s = trunc(21.6) =
    // 21, where one product of all three would give 22.
    let mut s = scene(car(WHEEL, 10, false), 0);
    s.ruleset
        .sim
        .countries
        .get_mut("Alphaland")
        .expect("country")
        .speed_units_mult = dec("0.99");
    set_land(&mut s, LandType::Clear, "0.9", "0.9");
    let (mut w, id) = start(&s, &[N; 8]);
    let mut ledger = Ledger::new();
    let first = first_moving_tick(&mut w, id);
    assert_eq!(ledger.total(&first), 21);
    for n in 2..=40 {
        let sn = step(&mut w, id);
        assert_eq!(ledger.total(&sn), 21 * n, "after {n} moving ticks");
    }
}

// spec: S-0207 claim 2 (the speed bonus factor b multiplies the type speed)
#[test]
fn progress_per_tick_uses_speed_bonus_factor() {
    let s = scene(car(WHEEL, 10, false), 0);
    let (mut w, id) = start(&s, &[N; 8]);
    let mode = w.rules().mode;
    w.obj_mut(id).expect("vehicle").speed_bonus = F64::from_i64(2, mode);
    let mut ledger = Ledger::new();
    let first = first_moving_tick(&mut w, id);
    assert_eq!(ledger.total(&first), 50);
    for n in 2..=12 {
        let sn = step(&mut w, id);
        assert_eq!(ledger.total(&sn), 50 * n);
    }
}

// ---------------------------------------------------------------------------
// Spending progress on track points

// spec: S-0207 claim 3 (a budget of exactly 7 buys no point; above 7 it keeps at least 1)
#[test]
fn budget_of_seven_buys_no_point() {
    // v = 3 gives V = 7: the first tick leaves A = 7 and no point bought;
    // every later tick has A = 14, buys one point and is back at 7.
    let s = scene(car(WHEEL, 3, false), 0);
    let (mut w, id) = start(&s, &[N; 4]);
    let first = first_moving_tick(&mut w, id);
    assert_eq!(first.budget, 7);
    assert_eq!(first.track.as_ref().map(|t| t.next), Some(0));
    for n in 1..=20 {
        let sn = step(&mut w, id);
        assert_eq!(sn.budget, 7);
        assert_eq!(sn.track.as_ref().map(|t| t.next), Some(n));
    }
}

// spec: S-0207 claim 3 (k points for 7k; at the target A′ = A + trunc(7 × (1 − m/11)); leftover kept at rest)
// spec: S-0207 F3
#[test]
fn track_costs_and_leftover_kept_at_rest() {
    // N, N, NE, NE, NE: straight A, curve C (N then NE), diagonal B twice,
    // no turn in place. Total cost 163 + 388 + 227 + 227 = 1005.
    let s = scene(car(WHEEL, 10, false), 0);
    let (mut w, id) = start(&s, &[N, N, NE, NE, NE]);
    let mut ledger = Ledger::new();
    let first = first_moving_tick(&mut w, id);
    let mut ticks = 1;
    let mut prev = first.clone();
    assert_eq!(ledger.total(&first), 25);
    loop {
        let sn = step(&mut w, id);
        if prev.track.is_none() && sn.track.is_none() {
            break;
        }
        ticks += 1;
        if sn.track.is_some() {
            assert_eq!(ledger.total(&sn), 25 * ticks, "tick {ticks}");
        }
        prev = sn;
        assert!(ticks < 100);
    }
    let end = snap(&w, id);
    assert_eq!(
        (end.pos.0, end.pos.1),
        (256 * (40 + 3) + 128, 256 * (40 - 5) + 128)
    );
    assert_eq!(end.budget, 25 * ticks - 1005, "leftover kept at rest");
    assert!(end.budget > 0);
}

// ---------------------------------------------------------------------------
// Position between points

// spec: S-0207 claim 4 (s = 2 on a straight track: 3, 7, 10, 12, 16, 20, 21, 25 leptons)
// spec: S-0207 F4
#[test]
fn position_between_points_compounds() {
    // v = 1 gives V = 2.
    let s = scene(car(WHEEL, 1, false), 0);
    let (mut w, id) = start(&s, &[N; 3]);
    let y0 = snap(&w, id).pos.1;
    let mut got = vec![y0 - first_moving_tick(&mut w, id).pos.1];
    for _ in 1..8 {
        got.push(y0 - step(&mut w, id).pos.1);
    }
    assert_eq!(got, [3, 7, 10, 12, 16, 20, 21, 25]);
}

// ---------------------------------------------------------------------------
// Height and facing on a track

// spec: S-0207 claim 5 (facing 256 f of the last point passed; at a point target + displacement; at the end the target)
#[test]
fn facing_and_position_follow_curve_points() {
    // N then E: curve D to the cell one up and one right. v = 3 gives V = 7,
    // which buys one point per tick (claim 3), so every point is visited.
    let s = scene(car(WHEEL, 3, false), 0);
    let (mut w, id) = start(&s, &[N, E]);
    let mut sn = first_moving_tick(&mut w, id);
    let (pts, two) = tracks::track(N, Some(E));
    assert!(two);
    assert_eq!(pts.len(), 38);
    let mut at_last_point = 0;
    let mut checked = 0;
    while let Some(t) = sn.track.clone() {
        if t.next > 0 {
            let p = pts[t.next - 1];
            assert_eq!(sn.facing, u16::from(p.2) * 256, "point {}", t.next - 1);
            checked += 1;
        } else {
            assert_eq!(sn.facing, 0, "no point passed yet");
        }
        if t.next == pts.len() {
            let p = pts[pts.len() - 1];
            assert_eq!((sn.pos.0, sn.pos.1), (t.target.0 + p.0, t.target.1 + p.1));
            at_last_point += 1;
        }
        sn = step(&mut w, id);
    }
    assert_eq!(checked, 38);
    assert!(at_last_point > 0, "never stood at the last point");
    let c = cell(1, -1);
    assert_eq!((sn.pos.0, sn.pos.1), (256 * c.x + 128, 256 * c.y + 128));
    assert_eq!(sn.facing, 64 * 256, "the curve ends facing east");
}

// spec: S-0207 claim 5 (the height follows the surface height at the point)
#[test]
fn height_follows_surface_at_points() {
    // A ramp in the cell ahead, driven onto and stopped on. v = 3 gives V = 7,
    // so the vehicle stands exactly on the last point for a tick. The height
    // between points is not given by the spec and is not checked.
    let mut s = scene(car(WHEEL, 3, false), 0);
    s.set_cell(cell(0, -1), 1, 0);
    let (mut w, id) = start(&s, &[N]);
    let mut sn = first_moving_tick(&mut w, id);
    let mut on_last = 0;
    while let Some(t) = sn.track.clone() {
        let (pts, _) = tracks::track(t.d, t.e);
        if t.next == pts.len() {
            let p = pts[pts.len() - 1];
            let (x, y) = (t.target.0 + p.0, t.target.1 + p.1);
            assert_eq!((sn.pos.0, sn.pos.1), (x, y));
            let h = w.surface_at(x, y);
            assert!(h > 0, "the last point lies on the ramp");
            assert_eq!(sn.pos.2, h);
            on_last += 1;
        }
        sn = step(&mut w, id);
    }
    assert!(on_last > 0);
    // At rest on the track target, the ramp's centre.
    let c = cell(0, -1);
    let (x, y) = (256 * c.x + 128, 256 * c.y + 128);
    assert_eq!((sn.pos.0, sn.pos.1), (x, y));
    assert!(w.surface_at(x, y) > 0);
    assert_eq!(sn.pos.2, w.surface_at(x, y));
}

// ---------------------------------------------------------------------------
// Turn in place

// spec: S-0207 claim 6 (no track and no progress until the body faces the step exactly; budget 0 while turning)
#[test]
fn turns_in_place_before_a_track() {
    // Facing east (64), the first step is north.
    let s = scene(car(WHEEL, 10, false), 64);
    let (mut w, id) = start(&s, &[N, N]);
    let p0 = snap(&w, id).pos;
    let mut turning = 0;
    loop {
        let sn = step(&mut w, id);
        if let Some(t) = &sn.track {
            assert_eq!(t.d, N);
            // The track begins with the body facing map north exactly.
            assert_eq!(snap(&w, id).facing, 0);
            break;
        }
        assert_eq!(sn.pos, p0, "no progress while turning");
        assert_eq!(sn.budget, 0);
        turning += 1;
        assert!(turning < 100);
    }
    assert!(turning >= 1);
}

// spec: S-0207 claim 6 (the speed fraction is kept through a turn in place)
// spec: S-0207 claim 7 (a 3/8 turn: the straight track for the current step, then a turn in place)
#[test]
fn speed_fraction_kept_through_turn_in_place() {
    // N, N, N then SE (3/8 turn): straight to the third cell, turn, go on.
    let s = scene(car(WHEEL, 30, true), 0);
    let path = [N, N, N, SE, SE, SE, SE];
    let (mut w, id) = start(&s, &path);
    let mut sn = first_moving_tick(&mut w, id);
    // Drive until the track for the third north step has ended.
    let mut guard = 0;
    while sn.track.as_ref().is_none_or(|t| t.d == N) {
        sn = step(&mut w, id);
        guard += 1;
        assert!(guard < 200);
        if sn.track.is_none() {
            break;
        }
    }
    // Now at the third cell's centre with no track, turning towards SE.
    let c = cell(0, -3);
    assert_eq!((sn.pos.0, sn.pos.1), (256 * c.x + 128, 256 * c.y + 128));
    assert!(sn.track.is_none());
    let kept = sn.rho;
    assert!(kept.gt(F64::ZERO));
    let mut turning = 0;
    loop {
        let next = step(&mut w, id);
        if next.track.is_some() {
            assert_eq!(next.track.as_ref().map(|t| t.d), Some(SE));
            break;
        }
        assert_eq!(next.rho, kept, "ρ kept while turning");
        assert_eq!(next.budget, 0);
        turning += 1;
        assert!(turning < 100);
    }
    assert!(turning >= 1);
}

// ---------------------------------------------------------------------------
// Choice of track

/// The first track a vehicle takes for `path`.
fn first_track(s: &Scene, path: &[u8]) -> (Track, usize) {
    let (mut w, id) = start(s, path);
    let sn = first_moving_tick(&mut w, id);
    (sn.track.expect("track"), sn.steps)
}

// spec: S-0207 claim 7 (same step: straight or diagonal; 1/8 turn: C or F; 1/4 turn: D or E; both steps used)
// spec: S-0207 table T2
#[test]
fn next_step_chooses_the_track() {
    let s = scene(car(WHEEL, 10, false), 0);
    // (path, expected points, two-cell curve, target)
    let cases: [(&[u8], usize, bool, CellXY); 6] = [
        (&[N, N, N], 23, false, cell(0, -1)),
        (&[N, NE, NE], 54, true, cell(1, -2)),
        (&[N, E, E], 38, true, cell(1, -1)),
        (&[N, SE], 23, false, cell(0, -1)),
        (&[N, 4], 23, false, cell(0, -1)),
        (&[N], 23, false, cell(0, -1)),
    ];
    for (path, len, two, target) in cases {
        let (t, left) = first_track(&s, path);
        let (pts, is_two) = tracks::track(t.d, t.e);
        assert_eq!(pts.len(), len, "path {path:?}");
        assert_eq!(is_two, two, "path {path:?}");
        assert_eq!(t.target_cell, target, "path {path:?}");
        let used = if two { 2 } else { 1 };
        assert_eq!(left, path.len() - used, "steps used for {path:?}");
    }
    // Diagonal current steps (placed facing NE): F for a 1/8 turn and E for a
    // 1/4 turn.
    let s = scene(car(WHEEL, 10, false), 32);
    for (path, len, target) in [
        (&[NE, NE][..], 31, cell(1, -1)),
        (&[NE, E][..], 56, cell(2, -1)),
        (&[NE, SE][..], 61, cell(2, 0)),
    ] {
        let (t, _) = first_track(&s, path);
        let (pts, _) = tracks::track(t.d, t.e);
        assert_eq!(pts.len(), len, "path {path:?}");
        assert_eq!(t.target_cell, target, "path {path:?}");
    }
}

// spec: S-0207 claim 7 (each two-cell curve ends facing e; another step after it needs a turn in place)
#[test]
fn curve_ends_facing_its_second_step_then_turns() {
    // N then NE (curve C), then E: at the curve's target the body faces NE
    // and the vehicle turns in place to E before going on.
    let s = scene(car(WHEEL, 10, false), 0);
    let (mut w, id) = start(&s, &[N, NE, E, E]);
    let mut sn = first_moving_tick(&mut w, id);
    while sn.track.as_ref().is_some_and(|t| t.e == Some(NE)) {
        sn = step(&mut w, id);
    }
    assert!(sn.track.is_none());
    // The turn in place towards E starts from the curve's final facing NE.
    assert_eq!(w.obj(id).expect("vehicle").body.start, 32 * 256);
    let c = cell(1, -2);
    assert_eq!((sn.pos.0, sn.pos.1), (256 * c.x + 128, 256 * c.y + 128));
    let mut turned = false;
    loop {
        let next = step(&mut w, id);
        if let Some(t) = next.track {
            assert_eq!(t.d, E);
            break;
        }
        turned = true;
        assert_eq!(next.budget, 0);
    }
    assert!(turned);
}

// spec: S-0207 claim 7 (the second cell of a curve occupied: the straight track into the first cell)
#[test]
fn occupied_second_cell_takes_straight_track() {
    let mut s = scene(car(WHEEL, 10, false), 0);
    s.place_unit(0, "TESTCAR", cell(1, -2), 0);
    let (t, left) = first_track(&s, &[N, NE, NE]);
    assert!(!t.crush, "the occupancy fallback, not crush mode");
    assert_eq!(tracks::track(t.d, t.e).0.len(), 23);
    assert_eq!(t.target_cell, cell(0, -1));
    assert_eq!(left, 2);
}

/// A scene with a crushable overlay TESTFENCE in `c`.
fn with_fence(s: &mut Scene, c: CellXY) {
    s.register("overlays", "TESTFENCE");
    s.ruleset.sim.overlays.insert(
        "TESTFENCE".into(),
        OverlayType {
            crushable: Some(true),
            ..Default::default()
        },
    );
    s.map.has_overlay_data = true;
    s.cell_mut(c).overlay = 0;
}

// spec: S-0207 claim 7 (crush mode: a crushable overlay in the second or first cell; drive straight, one step)
#[test]
fn crush_mode_drives_straight() {
    for fence in [cell(1, -2), cell(0, -1)] {
        let mut s = scene(car(WHEEL, 10, false), 0);
        with_fence(&mut s, fence);
        let (t, left) = first_track(&s, &[N, NE, NE]);
        assert!(t.crush, "fence at {fence:?}");
        assert_eq!(tracks::track(t.d, t.e).0.len(), 23, "fence at {fence:?}");
        assert_eq!(t.target_cell, cell(0, -1));
        assert_eq!(left, 2);
    }
    // No fence: the curve.
    let s = scene(car(WHEEL, 10, false), 0);
    let (t, _) = first_track(&s, &[N, NE, NE]);
    assert!(!t.crush);
    assert_eq!(tracks::track(t.d, t.e).0.len(), 54);
}

// spec: S-0207 claim 7 (crusher-all: a vehicle in the second cell gives crush mode)
#[test]
fn crusher_all_vehicle_in_second_cell() {
    let mut t = car(WHEEL, 10, false);
    t.movement_zone = Some(MovementZone::CrusherAll);
    // A vehicle in the second cell (straight ahead with no next step).
    let mut s = scene(t.clone(), 0);
    s.place_unit(1, "TESTCAR", cell(0, -2), 0);
    let (tr, _) = first_track(&s, &[N]);
    assert!(tr.crush);
    // The same type with no vehicle there.
    let s = scene(t, 0);
    let (tr, _) = first_track(&s, &[N]);
    assert!(!tr.crush);
}

// ---------------------------------------------------------------------------
// No chaining of curves

// spec: S-0207 claim 8 (an ordinary vehicle runs every curve to its target's centre, even when the next step turns again)
#[test]
fn ordinary_vehicle_does_not_chain_curves() {
    // N, NE, E, SE: every pair could curve. After curve C the step E turns
    // again; the vehicle still reaches the centre of the curve's target.
    let s = scene(car(WHEEL, 10, false), 0);
    let (mut w, id) = start(&s, &[N, NE, E, SE]);
    let mut sn = first_moving_tick(&mut w, id);
    let target = sn.track.as_ref().expect("track").target;
    let mut reached = false;
    for _ in 0..200 {
        if (sn.pos.0, sn.pos.1) == target {
            reached = true;
            break;
        }
        assert_eq!(sn.track.as_ref().map(|t| t.target), Some(target));
        sn = step(&mut w, id);
    }
    assert!(reached, "the curve ran to its target's centre");
}

// spec: S-0207 claim 8 (a passive vehicle with the acceleration flag on keeps ρ = 0 and never moves from rest)
#[test]
fn passive_accelerating_vehicle_never_moves() {
    let mut t = car(WHEEL, 10, true);
    t.passive = Some(true);
    let s = scene(t, 0);
    let (mut w, id) = start(&s, &[N, N, N]);
    let p0 = snap(&w, id).pos;
    for _ in 0..60 {
        let sn = step(&mut w, id);
        assert_eq!(sn.rho, F64::ZERO);
        assert_eq!(sn.pos, p0);
    }
}

// ---------------------------------------------------------------------------
// Target fraction

/// The target fraction of the first track for `path`.
fn first_rho_t(s: &Scene, path: &[u8]) -> F64 {
    let (mut w, id) = start(s, path);
    first_moving_tick(&mut w, id).rho_t
}

// spec: S-0207 claim 9 (land factor g = min(M, 1) for the vehicle's locomotion class)
// spec: S-0207 F6
#[test]
fn target_fraction_land_factor() {
    for (wheel, want) in [("0.5", HALF), ("1.5", ONE), ("0.25", QUARTER)] {
        let mut s = scene(car(WHEEL, 10, false), 0);
        set_land(&mut s, LandType::Clear, wheel, "1");
        assert_eq!(first_rho_t(&s, &[N, N]), want, "wheel {wheel}");
    }
    // A tracked vehicle reads the track column.
    let mut s = scene(car(TRACK, 10, false), 0);
    set_land(&mut s, LandType::Clear, "1", "0.25");
    assert_eq!(first_rho_t(&s, &[N, N]), QUARTER);
    // With the acceleration flag off ρ follows at once (claim 10), so ρ = ρₜ.
    let mut s = scene(car(WHEEL, 10, false), 0);
    set_land(&mut s, LandType::Clear, "0.5", "1");
    let (mut w, id) = start(&s, &[N, N]);
    assert_eq!(first_moving_tick(&mut w, id).rho, HALF);
}

// spec: S-0207 claim 9 (the entered cell's land type when its level differs by less than 2, not road)
#[test]
fn target_fraction_from_entered_cell_land_type() {
    // The vehicle stands on clear (1.0); the entered cell is rough (0.5);
    // road has its own value (0.25) and is not used at equal levels. A level
    // difference of 1 or more is a cliff on flat cells, so only equal levels
    // are reachable here; the road case needs a bridge deck.
    let mut s = scene(car(WHEEL, 10, false), 0);
    set_land(&mut s, LandType::Rough, "0.5", "0.5");
    set_land(&mut s, LandType::Road, "0.25", "0.25");
    // Terrain code 14 is rough (S-0211 claim 6).
    s.set_cell(cell(0, -1), code_tile(14), 0);
    assert_eq!(first_rho_t(&s, &[N, N]), HALF);
}

// spec: S-0207 claim 9 (slope factor: uphill or downhill by surface heights; tracked or wheeled factors)
#[test]
fn target_fraction_slope_factor() {
    let set_slopes = |s: &mut Scene| {
        let g = &mut s.ruleset.sim.general;
        g.wheeled_uphill = dec("0.5");
        g.wheeled_downhill = dec("0.25");
        g.tracked_uphill = dec("0.375");
        g.tracked_downhill = dec("1");
    };
    // Uphill: the ramp ahead has a higher surface at its centre.
    let mut s = scene(car(WHEEL, 10, false), 0);
    set_slopes(&mut s);
    s.set_cell(cell(0, -1), 1, 0);
    assert_eq!(first_rho_t(&s, &[N, N]), HALF);
    let mut s = scene(car(TRACK, 10, false), 0);
    set_slopes(&mut s);
    s.set_cell(cell(0, -1), 1, 0);
    assert_eq!(first_rho_t(&s, &[N, N]), THREE_EIGHTHS);
    // Downhill: the vehicle stands on the ramp, the cell ahead is flat.
    let mut s = scene(car(WHEEL, 10, false), 0);
    set_slopes(&mut s);
    s.set_cell(cell(0, 0), 1, 0);
    assert_eq!(first_rho_t(&s, &[N, N]), QUARTER);
    // Level: factor 1.
    let mut s = scene(car(WHEEL, 10, false), 0);
    set_slopes(&mut s);
    assert_eq!(first_rho_t(&s, &[N, N]), ONE);
}

// spec: S-0207 claim 9 (zero fallback 0.5 where g × u = 0; damage factor 0.75 at or below the yellow ratio, also on the fallback)
#[test]
fn target_fraction_zero_fallback_and_damage() {
    let zero_uphill = |s: &mut Scene| {
        s.ruleset.sim.general.wheeled_uphill = dec("0");
        s.set_cell(cell(0, -1), 1, 0);
    };
    let mut s = scene(car(WHEEL, 10, false), 0);
    zero_uphill(&mut s);
    assert_eq!(first_rho_t(&s, &[N, N]), HALF);

    let damaged = |s: &Scene, health: i32| {
        let (mut w, id) = start(s, &[N, N]);
        w.obj_mut(id).expect("vehicle").health = health;
        first_moving_tick(&mut w, id).rho_t
    };
    // Strength 100, yellow ratio 0.5: health 50 is damaged, 51 is not.
    let mut s = scene(car(WHEEL, 10, false), 0);
    s.ruleset.sim.general.condition_yellow = dec("0.5");
    set_land(&mut s, LandType::Clear, "0.5", "1");
    assert_eq!(damaged(&s, 50), THREE_EIGHTHS);
    assert_eq!(damaged(&s, 51), HALF);
    // Damaged on the zero fallback: 0.75 × 0.5.
    let mut s = scene(car(WHEEL, 10, false), 0);
    s.ruleset.sim.general.condition_yellow = dec("0.5");
    zero_uphill(&mut s);
    assert_eq!(damaged(&s, 50), THREE_EIGHTHS);
}

// spec: S-0207 claim 9 (a two-cell curve takes its target from its first cell only, until the next track)
#[test]
fn curve_target_fraction_from_first_cell() {
    // Curve C (N then NE) over a clear first cell and a rough second cell
    // (0.5), then the other way round.
    for (rough, want) in [(cell(1, -2), ONE), (cell(0, -1), HALF)] {
        let mut s = scene(car(WHEEL, 10, false), 0);
        set_land(&mut s, LandType::Rough, "0.5", "0.5");
        s.set_cell(rough, code_tile(14), 0);
        let (mut w, id) = start(&s, &[N, NE, NE]);
        let mut sn = first_moving_tick(&mut w, id);
        let t = sn.track.clone().expect("track");
        assert_eq!(tracks::track(t.d, t.e).0.len(), 54);
        let mut ticks = 0;
        while sn.track.as_ref().is_some_and(|x| x.target == t.target) {
            assert_eq!(sn.rho_t, want, "rough at {rough:?}");
            sn = step(&mut w, id);
            ticks += 1;
        }
        assert!(ticks > 5);
    }
}

// ---------------------------------------------------------------------------
// Stopping

// spec: S-0207 claim 11 (a stop on a track: ρₜ = min(ρₜ, F₀.₃), destination cleared, the track finished, ρ = 0 at rest)
#[test]
fn stop_finishes_track_and_rests() {
    let s = scene(car(WHEEL, 30, true), 0);
    let (mut w, id) = start(&s, &[N; 10]);
    let mut sn = first_moving_tick(&mut w, id);
    for _ in 0..12 {
        sn = step(&mut w, id);
    }
    let target = sn.track.as_ref().expect("on a track").target;
    let owner = w.obj(id).and_then(|o| o.owner).unwrap_or(HouseId(0));
    assert!(orders::issue(&mut w, owner, Order::Stop { actor: id }));
    let mut sn = step(&mut w, id);
    assert!(w.obj(id).expect("vehicle").dest.is_none());
    assert!(sn.track.is_some(), "still on the track it was on");
    assert_eq!(sn.rho_t, F03);
    let mut guard = 0;
    while sn.track.is_some() {
        assert_eq!(sn.track.as_ref().map(|t| t.target), Some(target));
        sn = step(&mut w, id);
        guard += 1;
        assert!(guard < 100);
    }
    assert_eq!((sn.pos.0, sn.pos.1), target);
    assert_eq!(sn.rho, F64::ZERO);
    // It stays there.
    let later = {
        run(&mut w, 10);
        snap(&w, id)
    };
    assert_eq!(later.pos, sn.pos);
}

// spec: S-0207 claim 11 (a vehicle at rest starts every move from ρ = 0)
// spec: S-0207 claim 12 (the first tick from rest: ρ = min(a, ρₜ))
#[test]
fn move_from_rest_starts_at_zero() {
    let s = scene(car(WHEEL, 30, true), 0);
    let (mut w, id) = start(&s, &[N, N]);
    let mut sn = first_moving_tick(&mut w, id);
    // spec: S-0207 Constants (acceleration factor absent: 0.03)
    let a = nearest_double("0.03");
    assert_eq!(sn.rho, a);
    while sn.track.is_some() {
        sn = step(&mut w, id);
    }
    assert_eq!(sn.rho, F64::ZERO, "at rest at the destination");
    // A second move, straight on (no turn in place).
    let owner = w.obj(id).and_then(|o| o.owner).unwrap_or(HouseId(0));
    let dest = cell(0, -4);
    assert!(orders::issue(&mut w, owner, move_to(id, dest)));
    tick::step(&mut w);
    w.obj_mut(id).expect("vehicle").path.steps = vec![N, N];
    let sn = first_moving_tick(&mut w, id);
    assert_eq!(sn.rho, a);
}

// ---------------------------------------------------------------------------
// Ticks and track boundaries

// spec: S-0207 claim 12 (a tick begun without a track finishes at most one; the extra budget piles up)
#[test]
fn track_end_cap_one_cell_per_tick() {
    // v = 100 gives V = 255 > 163: one straight track per tick, and each
    // tick adds 255 − 163 = 92 to the budget.
    let s = scene(car(WHEEL, 100, false), 0);
    let (mut w, id) = start(&s, &[N; 8]);
    let y0 = snap(&w, id).pos.1;
    let mut cells = 0;
    let mut n = 0;
    for _ in 0..20 {
        let before = snap(&w, id);
        let sn = step(&mut w, id);
        if sn.pos != before.pos {
            n += 1;
            cells += 1;
            assert!(sn.track.is_none(), "the tick ends with no track");
            assert_eq!(y0 - sn.pos.1, 256 * cells, "one cell per tick");
            assert_eq!(sn.budget, 92 * n);
            if cells == 6 {
                break;
            }
        }
    }
    assert_eq!(cells, 6);
}

// spec: S-0207 claim 12 (`quirks.drive_track_end_cap` off: no limit on tracks per tick)
#[test]
fn track_end_cap_off_allows_more() {
    let mut s = scene(car(WHEEL, 100, false), 0);
    s.ruleset.globals.quirks.drive_track_end_cap = false;
    let (mut w, id) = start(&s, &[N; 12]);
    let y0 = snap(&w, id).pos.1;
    first_moving_tick(&mut w, id);
    for _ in 0..3 {
        step(&mut w, id);
    }
    // Four ticks at 255 per tick buy more than four 163-unit tracks.
    let moved = y0 - snap(&w, id).pos.1;
    assert!(moved > 4 * 256, "moved {moved}");
}

// spec: S-0207 claim 12 (on a track: a second update of ρ when the track ends and the move goes on, so 2a in that tick)
#[test]
fn accelerating_vehicle_gains_twice_at_track_boundary() {
    // Far destination (no braking), ρₜ = 1, a = 0.03 (absent-key default).
    let s = scene(car(WHEEL, 100, true), 0);
    let (mut w, id) = start(&s, &[N; 16]);
    let mode = w.rules().mode;
    // spec: S-0207 Constants (acceleration factor absent: 0.03)
    let a = nearest_double("0.03");
    let mut prev = first_moving_tick(&mut w, id);
    let mut doubles = 0;
    for _ in 0..20 {
        let sn = step(&mut w, id);
        let once = prev.rho.add(a, mode).min(ONE);
        let twice = once.add(a, mode).min(ONE);
        // The tick began on a track and that track ended (the next one began,
        // and may itself have ended).
        let boundary = prev.track.is_some()
            && sn.track.as_ref().map(|t| t.target) != prev.track.as_ref().map(|p| p.target);
        if boundary {
            assert_eq!(sn.rho, twice);
            doubles += 1;
        } else {
            assert_eq!(sn.rho, once);
        }
        prev = sn;
    }
    assert!(doubles > 0);
}
