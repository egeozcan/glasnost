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

// spec: S-0207 claim 7 (the second cell of a curve being entered by another vehicle, a moving blocker: the straight track into the first cell)
// spec: S-0207 table T5 (moving blocker: the straight or diagonal track; the stored path is kept)
#[test]
fn occupied_second_cell_takes_straight_track() {
    let s = scene(car(WHEEL, 10, false), 0);
    let (mut w, id) = start(&s, &[N, NE, NE]);
    // Another vehicle's step into the second cell sets its flag (S-0209 claim 1, row VR5).
    w.cells
        .claim_vehicle(cell(1, -2), glasnost_sim::world::Layer::Ground, true);
    let sn = first_moving_tick(&mut w, id);
    let (t, left) = (sn.track.expect("track"), sn.steps);
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

// ---------------------------------------------------------------------------
// Which locomotor a type has

// spec: S-0207 claim 14 (absent: the teleporting locomotor whatever the kind; the driving locomotor drives a vehicle)
// spec: S-0207 claim 20 and S-0208 claim 19 (a vehicle or infantry unit with the teleporting locomotor jumps)
#[test]
fn locomotor_field_selects_how_a_type_moves() {
    use glasnost_rules::sim::Locomotor as L;
    use glasnost_sim::rules::Locomotor;
    let mut s = Scene::new();
    let mut plain = car(WHEEL, 30, false);
    plain.locomotor = None;
    s.raw_unit("vehicles", "NOLOCO", plain.clone());
    let mut walker = unit_type(FOOT, 10, 0);
    walker.locomotor = None;
    s.raw_unit("infantry", "NOLOCOINF", walker.clone());
    let mut drives = plain.clone();
    drives.locomotor = Some(L::Drive);
    s.raw_unit("vehicles", "DRIVES", drives);
    let mut walks = plain;
    walks.locomotor = Some(L::Walk);
    s.raw_unit("vehicles", "WALKS", walks);
    s.place_unit(0, "NOLOCO", cell(0, 0), 0);
    s.place_unit(0, "DRIVES", cell(4, 0), 0);
    s.place_infantry(0, "NOLOCOINF", cell(8, 0), 1);
    let r = s.sim_rules();
    let unit = |kind: Kind, i: u32| r.unit(kind, i).expect("type").clone();
    assert_eq!(unit(Kind::Vehicle, 0).type_locomotor(), L::Teleport);
    assert_eq!(unit(Kind::Infantry, 0).type_locomotor(), L::Teleport);
    assert_eq!(unit(Kind::Vehicle, 0).locomotor, Locomotor::Teleport);
    assert_eq!(unit(Kind::Infantry, 0).locomotor, Locomotor::Teleport);
    assert_eq!(unit(Kind::Vehicle, 1).locomotor, Locomotor::Drive);
    // A vehicle type may name the walking locomotor; how it moves has no spec (Open questions).
    assert_eq!(unit(Kind::Vehicle, 2).type_locomotor(), L::Walk);
    assert_eq!(unit(Kind::Vehicle, 2).locomotor, Locomotor::None);

    let mut w = s.build();
    let vs = objects_of(&w, Kind::Vehicle);
    let (teleporter, driver) = (vs[0], vs[1]);
    assert!(
        w.obj(teleporter).expect("v").drive.is_none(),
        "no drive state without the driving locomotor"
    );
    open_orders(&mut w);
    order_move(&mut w, teleporter, cell(0, -6));
    order_move(&mut w, driver, cell(4, -6));
    let f0 = w.obj(teleporter).expect("v").body.value(w.tick);
    let d0 = w.obj(driver).expect("v").pos;
    run(&mut w, 40);
    let t = w.obj(teleporter).expect("v");
    assert_eq!(
        t.pos.cell(),
        cell(0, -6),
        "a teleporting vehicle jumps to its destination"
    );
    assert_eq!(t.body.value(w.tick), f0, "a jump keeps the facing");
    assert_ne!(
        w.obj(driver).expect("v").pos,
        d0,
        "a driving vehicle drives"
    );
}

// ---------------------------------------------------------------------------
// A refused step (claim 15) and vehicle scatter (claim 16)

/// A corridor one cell wide along map north through the middle column
/// (cells (0, 2) to (0, −14)); every cell beside it is rock, whose row is
/// left at the start-up row (every multiplier 0, S-0211 claim 2).
fn corridor() -> Scene {
    let mut s = Scene::new();
    // spec: S-0211 claim 5 (terrain code 15: rock)
    let rock = code_tile(15);
    for y in -15..=3 {
        for x in [-1, 1] {
            s.set_cell(cell(x, y), rock, 0);
        }
    }
    s.set_cell(cell(0, -15), rock, 0);
    s.set_cell(cell(0, 3), rock, 0);
    let mut armed = car(WHEEL, 30, false);
    armed.primary = Some("Pea".into());
    s.vehicle("MOVER", car(WHEEL, 30, false))
        .vehicle("ARMED", armed)
        .vehicle("IDLER", car(WHEEL, 30, false));
    let sim = &mut s.ruleset.sim;
    sim.weapons.insert(
        "Pea".into(),
        glasnost_rules::sim::WeaponType {
            damage: Some(5),
            warhead: Some("Soft".into()),
            ..Default::default()
        },
    );
    sim.warheads
        .insert("Soft".into(), glasnost_rules::sim::WarheadType::default());
    s
}

/// Places `mover` of house 0 at (0, 0) facing map north and `blocker` of
/// `house` at (0, −1); builds and orders the mover to `dest`. Returns the
/// world, mover and blocker.
fn blocked_scene(
    s: &mut Scene,
    mover: &str,
    blocker: &str,
    house: u8,
    dest: CellXY,
) -> (World, ObjectId, ObjectId) {
    s.place_unit(0, mover, cell(0, 0), 0);
    s.place_unit(house, blocker, cell(0, -1), 0);
    let mut w = s.build();
    let at = |w: &World, c: CellXY| object_in(w, c).expect("object");
    let (m, b) = (at(&w, cell(0, 0)), at(&w, cell(0, -1)));
    open_orders(&mut w);
    order_move(&mut w, m, dest);
    (w, m, b)
}

/// Runs until the mover reacts to the blocker (gives up, attacks, or the
/// blocker gets a destination).
fn until_blocked(w: &mut World, m: ObjectId, b: ObjectId) {
    let b0 = w.obj(b).map(|o| (o.dest, o.target));
    for _ in 0..40 {
        tick::step(w);
        let o = w.obj(m).expect("mover");
        let bo = w.obj(b).map(|o| (o.dest, o.target));
        if o.dest.is_none() || bo != b0 || o.target.is_some() {
            return;
        }
    }
    panic!("no reaction");
}

// spec: S-0207 claim 15 table T4 (stationary friendly: the path is replaced, a forced move-anywhere request without threat point reaches the friend; no track)
// spec: S-0207 claim 16 (no threat point: no draw, the nearby-cell search, behaviours unchanged)
#[test]
fn stationary_friend_gets_a_scatter_request() {
    let mut s = corridor();
    let (mut w, m, v) = blocked_scene(&mut s, "MOVER", "IDLER", 0, cell(0, -10));
    let beh = w.obj(v).expect("v").beh.current;
    until_blocked(&mut w, m, v);
    let mo = w.obj(m).expect("m");
    assert!(mo.dest.is_some(), "the mover keeps its move");
    assert_eq!(mo.pos.cell(), cell(0, 0));
    assert!(
        mo.drive.as_ref().expect("d").track.is_none(),
        "no track on that tick"
    );
    let vo = w.obj(v).expect("v");
    let Some(glasnost_sim::world::Target::Cell(c)) = vo.dest else {
        panic!("the friend got no destination: {:?}", vo.dest);
    };
    assert_ne!(c, cell(0, -1));
    assert_eq!(vo.beh.current, beh, "its behaviour is unchanged");
}

// spec: S-0207 claim 15 table T4 and F7 (within the close-enough distance the repeated check gives up the move; no request)
#[test]
fn stationary_friend_near_the_destination_gives_up() {
    let mut s = corridor();
    let (mut w, m, v) = blocked_scene(&mut s, "MOVER", "IDLER", 0, cell(0, -2));
    until_blocked(&mut w, m, v);
    assert_eq!(w.obj(m).expect("m").dest, None, "the move is given up");
    assert_eq!(w.obj(v).expect("v").dest, None, "no request was sent");
}

// spec: S-0207 Edge cases (`quirks.drive_giveup_ignores_contact`: default gives up in contact; off it requires no contact)
// spec: S-0207 claim 16 (a): a vehicle in sticky refuses the request
#[test]
fn give_up_and_contact_quirk() {
    for ignores in [true, false] {
        let mut s = corridor();
        s.ruleset.globals.quirks.drive_giveup_ignores_contact = ignores;
        let (mut w, m, v) = blocked_scene(&mut s, "MOVER", "IDLER", 0, cell(0, -2));
        w.obj_mut(v).expect("v").beh.current = Some(glasnost_sim::rules::Behaviour::Sticky);
        // In contact with the friend (S-0215 Terms).
        w.obj_mut(m).expect("m").contact = Some(v);
        let mut gave_up = false;
        for _ in 0..40 {
            tick::step(&mut w);
            w.obj_mut(m).expect("m").contact = Some(v);
            if w.obj(m).expect("m").dest.is_none() {
                gave_up = true;
                break;
            }
        }
        assert_eq!(gave_up, ignores, "quirk {ignores}");
        assert_eq!(w.obj(v).expect("v").dest, None, "sticky refuses");
    }
}

// spec: S-0207 claim 15 table T4 (obstacle at the repeated check: waits and attacks the first listed object not allied, by an override)
#[test]
fn enemy_obstacle_is_attacked() {
    let mut s = corridor();
    let (mut w, m, e) = blocked_scene(&mut s, "ARMED", "IDLER", 1, cell(0, -10));
    until_blocked(&mut w, m, e);
    let mo = w.obj(m).expect("m");
    assert_eq!(mo.target, Some(glasnost_sim::world::Target::Object(e)));
    assert_eq!(mo.beh.current, Some(glasnost_sim::rules::Behaviour::Attack));
    assert_eq!(mo.pos.cell(), cell(0, 0));
}

// spec: S-0207 claim 15 table T4 (blocked: the first check clears the path; the mover keeps its destination and does not enter)
#[test]
fn blocked_step_keeps_the_destination() {
    let mut s = corridor();
    let (mut w, m, e) = blocked_scene(&mut s, "MOVER", "IDLER", 1, cell(0, -10));
    let mut seen = false;
    for _ in 0..40 {
        tick::step(&mut w);
        let mo = w.obj(m).expect("m");
        assert_eq!(mo.pos.cell(), cell(0, 0), "never enters");
        assert!(mo.target.is_none(), "an unarmed mover attacks nothing");
        // The first check's tick: stored path cleared, destination kept.
        seen |= mo.path.steps.is_empty() && mo.dest.is_some();
    }
    assert!(seen);
    assert_eq!(w.obj(e).expect("e").pos.cell(), cell(0, -1));
}

// spec: S-0212 claim 22 (at the first check of a track's first cell, blocked clears the stored path and makes a fresh search on the same tick, whatever the path retry timer)
// spec: S-0207 claim 15 table T4 (first check, blocked: as S-0212 claim 22 gives)
#[test]
fn blocked_first_cell_searches_at_once() {
    // Facing map east, a stored path due north: the vehicle turns first,
    // and the first check comes when it faces the step.
    let s = scene(car(WHEEL, 30, false), 64);
    let (mut w, id) = start(&s, &[N, N, N]);
    // The first cell turns to rock after the path was stored.
    let rock = cell(0, -1);
    w.cells.grid.get_mut(rock).expect("cell").land = glasnost_map::terrain::land::ROCK;
    glasnost_sim::regions::cell_changed(&mut w, rock);
    // The retry timer never runs out: only the immediate search can help.
    w.obj_mut(id).expect("vehicle").path.retry_at = i32::MAX;
    let mut replaced = false;
    for _ in 0..80 {
        tick::step(&mut w);
        let o = w.obj(id).expect("vehicle");
        assert_ne!(o.pos.cell(), rock, "never enters");
        assert!(o.dest.is_some(), "the move is kept");
        if let Some(&d) = o.path.steps.first()
            && o.path.steps.as_slice() != [N, N, N]
        {
            let (dx, dy) = STEPS[usize::from(d)];
            assert_ne!(CellXY::new(cell(0, 0).x + dx, cell(0, 0).y + dy), rock);
            replaced = true;
            break;
        }
    }
    assert!(replaced, "a fresh path around the rock on the check's tick");
}

// spec: S-0207 claim 16 (the scatter order moves an idle vehicle to the nearby-cell search's cell; one with a destination refuses; sleep and the teleporting locomotor refuse)
#[test]
fn vehicle_scatter_order() {
    use glasnost_sim::world::Target;
    let mut s = scene(car(WHEEL, 30, false), 0);
    let mut tele = car(WHEEL, 30, false);
    tele.locomotor = Some(glasnost_rules::sim::Locomotor::Teleport);
    s.vehicle("TELE", tele);
    s.place_unit(0, "TESTCAR", cell(6, 0), 0);
    s.place_unit(0, "TELE", cell(-6, 0), 0);
    s.place_unit(0, "TESTCAR", cell(0, 6), 0);
    let mut w = s.build();
    open_orders(&mut w);
    let at = |w: &World, c: CellXY| object_in(w, c).expect("object");
    let (idle, driving, tp, sleeper) = (
        at(&w, cell(0, 0)),
        at(&w, cell(6, 0)),
        at(&w, cell(-6, 0)),
        at(&w, cell(0, 6)),
    );
    w.obj_mut(sleeper).expect("s").beh.current = Some(glasnost_sim::rules::Behaviour::Sleep);
    order_move(&mut w, driving, cell(6, -12));
    run(&mut w, 2);
    let far = w.obj(driving).expect("d").dest;
    let beh = w.obj(idle).expect("i").beh.current;
    // A request without a threat point takes no draw (checked on a copy).
    let mut copy = w.clone();
    let rng = copy.rng.clone();
    assert!(glasnost_sim::scatter::vehicle_request(
        &mut copy,
        idle,
        glasnost_sim::scatter::ScatterRequest {
            threat: None,
            forced: true,
            anywhere: false,
        }
    ));
    assert_eq!(copy.rng, rng, "no draw");
    for id in [idle, driving, tp, sleeper] {
        assert!(orders::issue(
            &mut w,
            HouseId(0),
            Order::Scatter { actor: id }
        ));
    }
    tick::step(&mut w);
    let io = w.obj(idle).expect("i");
    let Some(Target::Cell(c)) = io.dest else {
        panic!("the idle vehicle got no destination");
    };
    assert!(
        (c.x - 40).abs() <= 1 && (c.y - 40).abs() <= 1 && c != cell(0, 0),
        "a neighbour: {c:?}"
    );
    assert_eq!(io.beh.current, beh, "no move is queued");
    assert_eq!(
        w.obj(driving).expect("d").dest,
        far,
        "a destination refuses the order"
    );
    assert_eq!(
        w.obj(tp).expect("t").dest,
        None,
        "the teleporting locomotor refuses"
    );
    assert_eq!(w.obj(sleeper).expect("s").dest, None, "sleep refuses");
}

// spec: S-0207 claim 16 (a threat point: one draw over 0 to 2, the neighbours from d = (d₀ + j − 1) mod 8, move queued)
#[test]
fn vehicle_scatter_with_threat_point() {
    use glasnost_sim::scatter::{ScatterRequest, vehicle_request};
    use glasnost_sim::world::{Pos, Target};
    let s = scene(car(WHEEL, 30, false), 0);
    let mut w = s.build();
    let id = objects_of(&w, Kind::Vehicle)[0];
    let mut probe = w.rng.clone();
    // spec: S-0202 claim 4 (ranged draws)
    let j = probe.ranged(0, 2, true);
    // The threat point lies map west of the vehicle: d₀ is map east (2).
    let o = w.obj(id).expect("v").pos;
    let threat = Pos::new(o.x - 300, o.y, o.z);
    assert!(vehicle_request(
        &mut w,
        id,
        ScatterRequest {
            threat: Some(threat),
            forced: true,
            anywhere: false,
        }
    ));
    assert_eq!(w.rng, probe, "exactly one draw");
    let d = (2 + j - 1).rem_euclid(8) as usize;
    let (dx, dy) = STEPS[d];
    let ob = w.obj(id).expect("v");
    assert_eq!(ob.dest, Some(Target::Cell(CellXY::new(40 + dx, 40 + dy))));
    assert_eq!(ob.beh.queued, Some(glasnost_sim::rules::Behaviour::Move));
}

// spec: S-0207 Terms (vehicle scatter centre cell) and claim 16 (scenario D18 part i: on a track after a stop order the search is centred on the track target's cell; at rest on its own cell; never around the move destination)
#[test]
fn vehicle_scatter_centre_cell() {
    use glasnost_sim::path::{NearbyUse, nearby};
    use glasnost_sim::scatter::{ScatterRequest, vehicle_request};
    use glasnost_sim::world::{Layer, Target};
    let mut w = scene(car(WHEEL, 4, false), 0).build();
    let id = objects_of(&w, Kind::Vehicle)[0];
    open_orders(&mut w);
    let far = cell(0, -12);
    order_move(&mut w, id, far);
    let req = ScatterRequest {
        threat: None,
        forced: true,
        anywhere: false,
    };
    let around = |w: &World, c: CellXY| {
        nearby(
            w,
            c,
            NearbyUse::Scatter {
                class: Some(WHEEL),
                deck: false,
            },
        )
    };
    // Run until the vehicle is on a track into a cell other than its own.
    let mut on_track = false;
    for _ in 0..40 {
        tick::step(&mut w);
        let o = w.obj(id).expect("v");
        if o.drive
            .as_ref()
            .and_then(|d| d.track.as_ref())
            .is_some_and(|t| t.target_cell != o.pos.cell())
        {
            on_track = true;
            break;
        }
    }
    assert!(on_track, "the vehicle starts a track");
    assert!(orders::issue(&mut w, HouseId(0), Order::Stop { actor: id }));
    let mut tracked = false;
    let mut rested = false;
    for _ in 0..400 {
        tick::step(&mut w);
        let o = w.obj(id).expect("v").clone();
        if o.dest.is_some() {
            continue;
        }
        assert_eq!(o.layer, Layer::Ground);
        let track = o.drive.as_ref().and_then(|d| d.track.as_ref());
        let centre = track.map_or(o.pos.cell(), |t| t.target_cell);
        let mut copy = w.clone();
        let expect = around(&copy, centre);
        assert!(vehicle_request(&mut copy, id, req));
        let got = copy.obj(id).expect("v").dest;
        assert_eq!(got, expect.map(Target::Cell), "centred on {centre:?}");
        assert_ne!(got, around(&copy, far).map(Target::Cell));
        if track.is_some() {
            // Part i's case: K is a cell ahead of the vehicle's own cell.
            tracked |= centre != o.pos.cell();
        } else {
            rested = true;
            break;
        }
    }
    assert!(
        tracked,
        "the vehicle had no destination while still on its track"
    );
    assert!(rested, "the vehicle comes to rest");
}

// spec: S-0207 claim 15 (the first cell of a new path: stationary friends make the vehicle give up, or get the request, on the search's tick, before any turn)
#[test]
fn first_cell_of_a_new_path_acts_before_turning() {
    for (dest, gives_up) in [(cell(0, -2), true), (cell(0, -10), false)] {
        let mut s = corridor();
        s.place_unit(0, "MOVER", cell(0, 0), 64); // facing map east
        s.place_unit(0, "IDLER", cell(0, -1), 0);
        let mut w = s.build();
        let m = object_in(&w, cell(0, 0)).expect("m");
        let v = object_in(&w, cell(0, -1)).expect("v");
        open_orders(&mut w);
        order_move(&mut w, m, dest);
        let east = w.obj(m).expect("m").body.target;
        for _ in 0..3 {
            tick::step(&mut w);
            let reacted =
                w.obj(m).expect("m").dest.is_none() || w.obj(v).expect("v").dest.is_some();
            if reacted {
                break;
            }
        }
        let mo = w.obj(m).expect("m");
        // The tick just run is w.tick − 1: the search, the reaction and (when
        // it goes on) the start of the turn all fall on it.
        let turn = mo.drive.as_ref().and_then(|d| d.turn);
        if gives_up {
            assert_eq!(
                mo.body.target, east,
                "a vehicle that gives up does not turn"
            );
            assert_eq!(mo.dest, None);
            assert_eq!(w.obj(v).expect("v").dest, None);
        } else {
            assert!(mo.dest.is_some());
            assert!(
                w.obj(v).expect("v").dest.is_some(),
                "the friend got the request"
            );
            assert_eq!(
                turn.map(|t| t.0),
                Some(w.tick - 1),
                "the turn starts on the request's tick"
            );
        }
    }
}

// spec: S-0207 F8 (dₙ from the cell's top corner: top 90, centre 181, right and left 202, bottom 271)
// spec: S-0207 Edge cases (`quirks.drive_nearest_from_top_corner` off: from the centre, the centre nearest and the outer positions alike)
#[test]
fn nearest_object_distance_from_the_top_corner() {
    use glasnost_sim::cells::position;
    for on in [true, false] {
        let mut s = scene(car(WHEEL, 10, false), 0);
        s.ruleset.globals.quirks.drive_nearest_from_top_corner = on;
        let w = s.build();
        let at = |p: u8| {
            let (u, v) = glasnost_sim::walk::standing_point(p);
            // spec: S-0207 F8 (u = x mod 256, w = y mod 256: the cell does not matter)
            glasnost_sim::drive::nearest_distance(&w, 256 * 41 + u, 256 * 37 + v)
        };
        let got = [
            at(position::TOP),
            at(position::CENTRE),
            at(position::RIGHT),
            at(position::LEFT),
            at(position::BOTTOM),
        ];
        if on {
            assert_eq!(got, [90, 181, 202, 202, 271]);
        } else {
            assert_eq!(got[1], 0, "the centre is nearest");
            assert!(got[0] == got[2] && got[2] == got[3] && got[3] == got[4]);
        }
    }
}

/// The corridor with a crushing mover of house 0 at (0, 0) facing map east
/// (1/4 turn away from X = (0, −1)), X holding `others` (house, type,
/// standing position; position 0 places a vehicle at the centre). Orders
/// the mover far up the corridor. Returns the world, the mover and the
/// objects in X.
fn nearest_scene(others: &[(u8, &str, i32)]) -> (World, ObjectId, Vec<ObjectId>) {
    nearest_scene_q(others, true)
}

/// [`nearest_scene`] with `quirks.drive_nearest_from_top_corner` set to `top_corner`.
fn nearest_scene_q(
    others: &[(u8, &str, i32)],
    top_corner: bool,
) -> (World, ObjectId, Vec<ObjectId>) {
    let mut s = corridor();
    s.ruleset.globals.quirks.drive_nearest_from_top_corner = top_corner;
    let mut crusher = car(WHEEL, 30, false);
    crusher.crusher = Some(true);
    s.vehicle("CRUSHER", crusher);
    let mut grunt = unit_type(glasnost_rules::sim::LocomotionClass::Foot, 4, 8);
    grunt.crushable = Some(true);
    s.infantry("GRUNT", grunt);
    s.place_unit(0, "CRUSHER", cell(0, 0), 64);
    for &(h, ty, sub) in others {
        if ty == "IDLER" {
            s.place_unit(h, ty, cell(0, -1), 0);
        } else {
            s.place_infantry(h, ty, cell(0, -1), sub);
        }
    }
    let mut w = s.build();
    let m = object_in(&w, cell(0, 0)).expect("mover");
    let inx = w
        .cells
        .list(cell(0, -1), glasnost_sim::world::Layer::Ground)
        .to_vec();
    open_orders(&mut w);
    order_move(&mut w, m, cell(0, -10));
    (w, m, inx)
}

/// Runs until the mover's request reaches X (an object there gets a
/// destination); returns whether that happened no later than the
/// tick on which the mover's turn in place towards X began, that is before
/// its facing first changed.
fn request_before_turning(w: &mut World, m: ObjectId, inx: &[ObjectId]) -> bool {
    let mut turn_began = None;
    for _ in 0..60 {
        tick::step(w);
        let t = w.tick - 1;
        if turn_began.is_none() {
            turn_began = w
                .obj(m)
                .and_then(|o| o.drive.as_ref())
                .and_then(|d| d.turn)
                .map(|x| x.0);
        }
        let sent = inx
            .iter()
            .any(|&x| w.obj(x).is_some_and(|o| o.dest.is_some()));
        if sent {
            return turn_began.is_none_or(|t0| t <= t0);
        }
    }
    panic!("no request was sent");
}

// spec: S-0207 claim 15 (the look at a new path's first cell: the nearest object by F8 decides; an enemy at the top position is nearer than a friend at the centre, so nothing happens until the vehicle faces the cell)
// spec: S-0207 F8 (top position 90 against the centre's 181; bottom 271)
#[test]
fn nearest_object_decides_the_look_at_the_first_cell() {
    // Run 1: the enemy infantry at the top position is nearest: no request
    // until the mover has turned to face X (then T4 sends it).
    let (mut w, m, inx) = nearest_scene(&[(0, "IDLER", 0), (1, "GRUNT", 1)]);
    assert!(
        !request_before_turning(&mut w, m, &inx),
        "the top position's enemy is the nearest object"
    );
    // Run 2: the enemy at the bottom position: the friend at the centre is
    // nearest, and the look sends the request before the turn.
    let (mut w, m, inx) = nearest_scene(&[(0, "IDLER", 0), (1, "GRUNT", 4)]);
    assert!(request_before_turning(&mut w, m, &inx));
    // spec: S-0207 Edge cases (`quirks.drive_nearest_from_top_corner` off: measured from the centre, the friend at the centre wins in run 1 too)
    let (mut w, m, inx) = nearest_scene_q(&[(0, "IDLER", 0), (1, "GRUNT", 1)], false);
    assert!(request_before_turning(&mut w, m, &inx));
}

// spec: S-0207 claim 15 (a tie of dₙ goes to the object listed first, S-0209 claim 13)
#[test]
fn nearest_object_tie_goes_to_the_first_listed() {
    // A friend and an enemy at the right and left positions (202 each).
    for first_friend in [true, false] {
        // The one placed last is listed first.
        let others: [(u8, &str, i32); 2] = if first_friend {
            [(1, "GRUNT", 3), (0, "GRUNT", 2)]
        } else {
            [(0, "GRUNT", 2), (1, "GRUNT", 3)]
        };
        let (mut w, m, inx) = nearest_scene(&others);
        let friendly = w.obj(inx[0]).and_then(|o| o.owner) == Some(HouseId(0));
        assert_eq!(friendly, first_friend, "listing order as placed");
        assert_eq!(
            request_before_turning(&mut w, m, &inx),
            first_friend,
            "friend listed first: {first_friend}"
        );
    }
}
/// The corridor forked after X = (0, −1): (0, −2) is rock, and two branches
/// leave X, through (−1, −2) to the north-west and through (1, −2) and
/// (1, −3) to the north-east, back into the corridor at (0, −3) and
/// (0, −4).
fn fork() -> Scene {
    let mut s = corridor();
    // spec: S-0211 claim 5 (terrain code 15: rock)
    s.set_cell(cell(0, -2), code_tile(15), 0);
    for c in [cell(-1, -2), cell(1, -2), cell(1, -3)] {
        s.set_cell(c, 0, 0);
    }
    s
}

// spec: S-0207 claim 15 table T5 (obstacle in a curve's second cell: the path is discarded; with the retry timer run out, the search on that tick, the look, and a step check of the same kind that begins only the straight or diagonal track)
#[test]
fn after_a_curve_s_obstacle_the_new_path_begins_straight() {
    let mut s = fork();
    // An armed mover (so the enemy gives an obstacle, not blocked) facing map east, so it turns before its first step check.
    s.place_unit(0, "ARMED", cell(0, 0), 64);
    s.place_unit(1, "IDLER", cell(-1, -2), 0);
    let mut w = s.build();
    let m = object_in(&w, cell(0, 0)).expect("m");
    let e = object_in(&w, cell(-1, -2)).expect("e");
    open_orders(&mut w);
    order_move(&mut w, m, cell(0, -10));
    tick::step(&mut w);
    // While it turns, its stored path is made to curve into the enemy's
    // cell: N into X, then NW (a 1/8 turn, a two-cell curve).
    {
        let o = w.obj_mut(m).expect("m");
        assert!(o.drive.as_ref().is_some_and(|d| d.track.is_none()));
        o.path.steps = vec![N, 7, 1, N, N, N, N, N, N, N];
        o.path.retry_at = 0;
    }
    let mut began = None;
    for _ in 0..40 {
        tick::step(&mut w);
        let o = w.obj(m).expect("m");
        if let Some(t) = o.drive.as_ref().and_then(|d| d.track.as_ref()) {
            began = Some((t.d, t.e, t.target_cell, o.path.steps.first().copied()));
            break;
        }
    }
    let (d, e_step, target, next) = began.expect("a track begins");
    assert_eq!(d, N);
    assert_eq!(
        e_step, None,
        "on the tick of the search the track is straight, not a curve"
    );
    assert_eq!(target, cell(0, -1));
    assert_eq!(next, Some(1), "the new path takes the north-east branch");
    assert_eq!(w.obj(e).expect("e").pos.cell(), cell(-1, -2));
}

// spec: S-0207 claim 15 (the override of an obstacle's repeated check gives no destination; reverted, the move destination is restored and the vehicle drives on)
#[test]
fn obstacle_override_puts_the_destination_aside() {
    let mut s = corridor();
    let (mut w, m, e) = blocked_scene(&mut s, "ARMED", "IDLER", 1, cell(0, -10));
    until_blocked(&mut w, m, e);
    let mo = w.obj(m).expect("m");
    assert_eq!(mo.target, Some(glasnost_sim::world::Target::Object(e)));
    assert_eq!(mo.dest, None, "the override gives no destination");
    assert_eq!(
        mo.beh.aside.map(|a| a.1),
        Some(Some(glasnost_sim::world::Target::Cell(cell(0, -10)))),
        "the move destination is put aside"
    );
    // While the override lasts, no destination and no second override.
    let aside = mo.beh.aside;
    for _ in 0..5 {
        tick::step(&mut w);
        let mo = w.obj(m).expect("m");
        if mo.target.is_none() {
            break;
        }
        assert_eq!(mo.dest, None);
        assert_eq!(mo.beh.aside, aside);
    }
    // Reverted: the destination put aside is the vehicle's again.
    let restored = w.obj_mut(m).expect("m").beh.revert();
    let (t, d) = restored.expect("an override to revert");
    let mo = w.obj_mut(m).expect("m");
    mo.target = t;
    mo.dest = d;
    assert_eq!(
        mo.dest,
        Some(glasnost_sim::world::Target::Cell(cell(0, -10)))
    );
}

// ---------------------------------------------------------------------------
// Engine defaults of the speed-fraction fields

// spec: S-0207 claim 17 (absent: a and η the doubles nearest 0.03 and 0.002, not widened singles; Dₛ 500)
// spec: S-0207 Constants (absent-key defaults of a, η and Dₛ)
#[test]
fn acceleration_defaults_are_doubles() {
    let mut s = Scene::new();
    s.vehicle("BARE", car(WHEEL, 20, true));
    let mut given = car(WHEEL, 20, true);
    given.acceleration_factor = dec("0.03");
    given.deacceleration_factor = dec("0.002");
    given.slowdown_distance = Some(500);
    s.vehicle("GIVEN", given);
    let r = s.sim_rules();
    let bare = r.unit(Kind::Vehicle, 0).expect("bare").clone();
    let given = r.unit(Kind::Vehicle, 1).expect("given").clone();
    // The values claim 17 states, exactly.
    let a = nearest_double("0.0299999999999999988897769753748434595763683319091796875");
    let eta = nearest_double("0.00200000000000000004163336342344337026588618755340576171875");
    assert_eq!(bare.acceleration, a);
    assert_eq!(bare.deceleration, eta);
    assert_eq!(bare.slowdown_distance, 500);
    // A layer that writes 0.03 gives the single nearest 0.03, widened (S-0003 claim 6).
    assert_eq!(
        given.acceleration,
        nearest_double("0.02999999932944774627685546875")
    );
    assert_ne!(given.acceleration, bare.acceleration);
    assert_ne!(given.deceleration, bare.deceleration);
    // Scenario D19 run A: the first braking value 1 − V × η with V = 51, rounded toward zero.
    let mode = r.mode;
    let v = F64::from_i64(51, mode);
    assert_eq!(
        ONE.sub(v.mul(bare.deceleration, mode), mode),
        nearest_double("0.8979999999999999094058011905872263014316558837890625")
    );
    // The first tick from rest far from the destination: ρ = a (case 4).
    let mut s2 = scene(car(WHEEL, 20, true), 0);
    s2.ruleset
        .sim
        .vehicles
        .get_mut("TESTCAR")
        .expect("car")
        .slowdown_distance = None;
    let (mut w, id) = start(&s2, &[N; 20]);
    assert_eq!(first_moving_tick(&mut w, id).rho, a);
}

// ---------------------------------------------------------------------------
// Getting onto and off a bridge deck (claim 5)

/// A high bridge along map north over ground at level 0: the vehicle's
/// column x = 0 has land at level 4 at y = 1, 0, −5 and −6 and the four
/// high-bridge cells y = −1 … −4 at level 0 between them, laid by overlays
/// of index 25 in each of those cells (S-0211 claim 15: each marks x − 2 …
/// x + 1 of its row; S-0212 Terms: the bridge axis along cell y, the
/// deck-entry mark on x − 1 … x + 1).
fn bridge_scene() -> Scene {
    let mut s = Scene::new();
    for i in 0..=25 {
        let name = format!("OV{i}");
        s.register("overlays", &name);
        s.ruleset.sim.overlays.insert(
            name,
            glasnost_rules::sim::OverlayType {
                has_art: Some(true),
                ..Default::default()
            },
        );
    }
    for y in [0, 1, -5, -6] {
        s.set_cell(cell(0, y), 0, 4);
    }
    for y in -4..=-1 {
        s.cell_mut(cell(0, y)).overlay = 25;
    }
    s
}

// spec: S-0207 claim 5 (onto the deck from land 4 levels higher, world height H + 416 on the deck, kept between high-bridge cells; off the deck into a cell that is not a high-bridge cell)
// spec: S-0212 claim 10 (steps of 4 levels onto a marked deck and off it; deck to deck along marked cells)
#[test]
fn driving_along_and_off_a_bridge_deck() {
    use glasnost_sim::world::Layer;
    let mut s = bridge_scene();
    s.vehicle("TESTCAR", car(WHEEL, 30, false));
    s.place_unit(0, "TESTCAR", cell(0, 0), 0);
    let (mut w, id) = start(&s, &[N; 5]);
    let mut seen_deck = 0;
    for _ in 0..300 {
        tick::step(&mut w);
        let o = w.obj(id).expect("v");
        let y = o.pos.cell().y - 40;
        if (-4..=-1).contains(&y) {
            assert_eq!(o.layer, Layer::Deck, "on the deck at y = {y}");
            assert_eq!(o.pos.z, 416, "H + 416 on the deck");
            seen_deck += 1;
        } else {
            assert_eq!(o.layer, Layer::Ground, "off the deck at y = {y}");
        }
        if y == -5 && o.drive.as_ref().is_some_and(|d| d.track.is_none()) {
            break;
        }
    }
    assert!(seen_deck > 0);
    assert_eq!(w.obj(id).expect("v").pos.cell(), cell(0, -5));
}

// spec: S-0207 claim 5 (a move into a high-bridge cell from ground at its own level keeps the vehicle off the deck)
#[test]
fn driving_under_a_bridge_deck() {
    let mut s = bridge_scene();
    s.vehicle("TESTCAR", car(WHEEL, 30, false));
    s.place_unit(0, "TESTCAR", cell(-2, -2), 64);
    let mut w = s.build();
    let id = objects_of(&w, Kind::Vehicle)[0];
    open_orders(&mut w);
    order_move(&mut w, id, cell(2, -2));
    tick::step(&mut w);
    w.obj_mut(id).expect("vehicle").path.steps = vec![E; 4];
    for _ in 0..200 {
        tick::step(&mut w);
        let o = w.obj(id).expect("v");
        assert_eq!(o.layer, glasnost_sim::world::Layer::Ground);
        assert_eq!(o.pos.z, 0, "under the deck: the ground's height");
        if o.pos.cell() == cell(2, -2) {
            return;
        }
    }
    panic!("the vehicle did not pass under the deck");
}

// spec: S-0207 claim 3 (the budget is emptied only when the vehicle is due to begin a track and waits or is refused; without a path it is kept)
#[test]
fn budget_kept_while_waiting_for_a_path() {
    let s = scene(car(WHEEL, 30, false), 0);
    let (mut w, id) = start(&s, &[N, N]);
    let t = w.tick;
    let ob = w.obj_mut(id).expect("v");
    ob.path.steps.clear();
    ob.path.retry_at = t + 50;
    ob.drive.as_mut().expect("d").budget = 40;
    run(&mut w, 5);
    assert_eq!(snap(&w, id).budget, 40);
}

// spec: S-0207 claim 16 (a): unload, sleep or sticky as the current behaviour, or as the queued one when there is no current one, refuses every request
#[test]
fn vehicle_scatter_refused_in_unload() {
    use glasnost_sim::rules::Behaviour;
    use glasnost_sim::scatter::{ScatterRequest, vehicle_request};
    let s = scene(car(WHEEL, 30, false), 0);
    let base = s.build();
    let id = object_in(&base, cell(0, 0)).expect("vehicle");
    let reqs = [
        ScatterRequest {
            threat: None,
            forced: true,
            anywhere: true,
        },
        ScatterRequest {
            threat: Some(glasnost_sim::world::Pos::centre(cell(3, 0), 0)),
            forced: true,
            anywhere: true,
        },
    ];
    for b in [Behaviour::Unload, Behaviour::Sleep, Behaviour::Sticky] {
        for (current, queued) in [(Some(b), None), (None, Some(b))] {
            for req in reqs {
                let mut w = base.clone();
                let o = w.obj_mut(id).expect("v");
                o.beh.current = current;
                o.beh.queued = queued;
                let rng = w.rng.clone();
                assert!(!vehicle_request(&mut w, id, req), "{b:?} {current:?}");
                assert_eq!(w.obj(id).expect("v").dest, None);
                assert_eq!(w.rng, rng, "no draw");
            }
        }
    }
    // A guard current with unload queued: the current behaviour decides.
    let mut w = base.clone();
    let o = w.obj_mut(id).expect("v");
    o.beh.current = Some(Behaviour::Guard);
    o.beh.queued = Some(Behaviour::Unload);
    assert!(vehicle_request(&mut w, id, reqs[0]));
}

// ---------------------------------------------------------------------------
// Speed estimate for lead

// spec: S-0207 claim 18 (the estimate is F2 at the fraction held now, 0 from rest; reading it updates no fraction)
// spec: S-0207 F2a
#[test]
fn speed_estimate_for_lead_is_f2_at_the_held_fraction() {
    // The scene of claim 2's test: s = 21 per moving tick at ρ = 0.9.
    let mut s = scene(car(WHEEL, 10, false), 0);
    s.ruleset
        .sim
        .countries
        .get_mut("Alphaland")
        .expect("country")
        .speed_units_mult = dec("0.99");
    set_land(&mut s, LandType::Clear, "0.9", "0.9");
    let (mut w, id) = start(&s, &[N; 8]);
    assert_eq!(glasnost_sim::drive::speed_estimate(&w, id), 0, "from rest");
    let first = first_moving_tick(&mut w, id);
    assert_eq!(glasnost_sim::drive::speed_estimate(&w, id), 21);
    assert_eq!(glasnost_sim::drive::speed_estimate(&w, id), 21);
    assert_eq!(snap(&w, id).rho, first.rho, "no fraction update");
}

// spec: S-0207 claim 18 (a turn in place between tracks keeps the held fraction, so the estimate stays nonzero while the position does not change)
#[test]
fn speed_estimate_nonzero_while_turning_in_place() {
    let s = scene(car(WHEEL, 30, true), 0);
    let path = [N, N, N, SE, SE, SE, SE];
    let (mut w, id) = start(&s, &path);
    let mut sn = first_moving_tick(&mut w, id);
    let mut guard = 0;
    while sn.track.is_some() {
        sn = step(&mut w, id);
        guard += 1;
        assert!(guard < 200);
    }
    assert!(sn.rho.gt(F64::ZERO));
    let next = step(&mut w, id);
    assert!(next.track.is_none(), "still turning");
    assert_eq!(next.pos, sn.pos);
    assert!(glasnost_sim::drive::speed_estimate(&w, id) > 0);
}
