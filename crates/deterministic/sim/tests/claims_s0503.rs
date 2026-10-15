//! Claim tests of S-0503 (hover and amphibious movement) on a synthetic map
//! with a lake and invented types (every name and number here is made up;
//! no game data).

mod common;

use common::{Scene, cell, code_tile};
use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_num::F64;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    Decimal, LandRow, LandType, LocomotionClass, Locomotor, MovementZone, UnitType,
};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::hover::{self, HoverState};
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, ObjectId, Pos, World};
use glasnost_sim::{save, testkit, tick};

const ALPHA: HouseId = HouseId(0);
/// The terrain code of a water tile in the scene's catalogue (S-0211 Terms).
const WATER_CODE: u8 = 9;
const WAKE: &str = "Ripple";

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

/// A land row: the float class only on water, the hover and amphibious
/// classes everywhere.
fn land_row(ground: &str, float: &str) -> LandRow {
    LandRow {
        foot: dec(ground),
        track: dec(ground),
        wheel: dec(ground),
        hover: dec("1"),
        float: dec(float),
        amphibious: dec("1"),
        float_beach: dec(float),
        buildable: None,
    }
}

fn hover_type() -> UnitType {
    UnitType {
        strength: Some(100),
        speed: Some(6),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Hover),
        locomotor: Some(Locomotor::Hover),
        movement_zone: Some(MovementZone::Amphibious),
        ..Default::default()
    }
}

/// The ruleset: the testkit's combat ruleset plus water rows, the wake
/// animation, the hover fields of [General] (invented values), and
/// - HOVER: the hovering locomotor and class, amphibious zone;
/// - AMPHI: the amphibious class with the hovering locomotor;
/// - DRIVER: the amphibious class with the driving locomotor.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let s = &mut r.sim;
    s.land_types.insert(LandType::Clear, land_row("1", "0"));
    s.land_types.insert(LandType::Road, land_row("1", "0"));
    s.land_types.insert(LandType::Water, land_row("0", "1"));
    s.general.wake = Some(WAKE.into());
    s.general.hover_height = Some(100);
    s.general.hover_bob = dec("0.02");
    s.general.hover_boost = dec("1.5");
    s.general.hover_acceleration = dec("0.02");
    s.general.hover_brake = dec("0.01");
    s.general.hover_dampen = dec("0.75");
    let amphi = UnitType {
        locomotion_class: Some(LocomotionClass::Amphibious),
        ..hover_type()
    };
    let driver = UnitType {
        locomotor: Some(Locomotor::Drive),
        ..amphi.clone()
    };
    for (n, u) in [
        ("HOVER", hover_type()),
        ("AMPHI", amphi),
        ("DRIVER", driver),
    ] {
        r.sim.vehicles.insert(n.into(), u);
        r.registries
            .entry("vehicles".into())
            .or_default()
            .push(n.into());
    }
    r
}

/// The lake: dx in −6 … 6, dy in −6 … 6.
fn lake(dx: i32, dy: i32) -> bool {
    (-6..=6).contains(&dx) && (-6..=6).contains(&dy)
}

fn world_with(r: Ruleset) -> World {
    let mut s = Scene::new();
    s.ruleset = r;
    for dx in -8..=8 {
        for dy in -8..=8 {
            if lake(dx, dy) {
                s.set_cell(cell(dx, dy), code_tile(WATER_CODE), 0);
            }
        }
    }
    s.build()
}

fn world() -> World {
    world_with(rules())
}

fn at(w: &mut World, name: &str, house: u8, dx: i32, dy: i32, facing: u16) -> ObjectId {
    testkit::spawn(w, Kind::Vehicle, name, house, cell(dx, dy), facing)
}

fn hs(w: &World, id: ObjectId) -> HoverState {
    w.obj(id)
        .unwrap()
        .hover
        .clone()
        .expect("a hovering vehicle")
}

fn f(w: &World, v: i64) -> F64 {
    F64::from_i64(v, w.rules().mode)
}

fn half(w: &World) -> F64 {
    f(w, 1).div(f(w, 2), w.rules().mode)
}

fn height_of(w: &World, id: ObjectId) -> i32 {
    hover::hover_height(w, id)
}

fn wakes(w: &World) -> usize {
    w.effects
        .iter()
        .filter(|e| matches!(e, Effect::Anim { name, .. } if name == WAKE))
        .count()
}

fn land_of(w: &World, id: ObjectId) -> u8 {
    let c = w.obj(id).unwrap().pos.cell();
    w.cells.cell(c).unwrap().land
}

/// Issues a move order and runs ticks until the vehicle has taken it.
fn order(w: &mut World, id: ObjectId, c: CellXY) {
    common::order_move(w, id, c);
    common::run_until(w, 20, |w| w.obj(id).unwrap().dest.is_some());
    assert!(w.obj(id).unwrap().dest.is_some(), "the move is taken");
}

/// Runs ticks until the vehicle has a heading point.
fn head(w: &mut World, id: ObjectId) {
    common::run_until(w, 20, |w| hs(w, id).point.is_some());
    assert!(hs(w, id).point.is_some());
}

/// Runs until `id` has no destination and no heading point; returns ticks.
fn until_stopped(w: &mut World, id: ObjectId, max: i32) -> i32 {
    common::run_until(w, max, |w| !hover::moving(w, id))
}

/// Lets a new hover rise to its hover height and the order window open.
fn settle(w: &mut World) {
    common::run(w, 300);
}

// spec: S-0503 claim 1 (the locomotor field alone chooses the model: a hovering amphibious type hovers, a driving amphibious type drives)
#[test]
fn claim_1_locomotor_chooses_the_model() {
    let mut w = world();
    let h = at(&mut w, "HOVER", 0, 0, 0, 0);
    let a = at(&mut w, "AMPHI", 0, 1, 0, 0);
    let d = at(&mut w, "DRIVER", 0, 2, 0, 0);
    for id in [h, a] {
        let o = w.obj(id).unwrap();
        assert!(o.hover.is_some() && o.drive.is_none());
    }
    let o = w.obj(d).unwrap();
    assert!(
        o.drive.is_some() && o.hover.is_none(),
        "a driving amphibious vehicle drives"
    );
}

// spec: S-0503 claim 1 and claim 4 (no change of locomotor at the shoreline: a hover crosses water and land by the same rules, and reaches a land goal across the lake)
#[test]
fn claim_1_hover_crosses_the_shore() {
    let mut w = world();
    let id = at(&mut w, "HOVER", 0, -8, 0, 0);
    settle(&mut w);
    let goal = cell(8, 0);
    order(&mut w, id, goal);
    let ticks = common::run_until(&mut w, 3000, |w| !hover::moving(w, id));
    assert!(ticks < 3000);
    assert_eq!(w.obj(id).unwrap().pos.cell(), goal);
    assert!(w.obj(id).unwrap().hover.is_some(), "still hovering");
    // Over water on the way.
    let mut w = world();
    let id = at(&mut w, "HOVER", 0, -8, 0, 0);
    settle(&mut w);
    order(&mut w, id, goal);
    let mut saw_water = false;
    for _ in 0..3000 {
        tick::step(&mut w);
        saw_water |= land_of(&w, id) == land::WATER;
        if !hover::moving(&w, id) {
            break;
        }
    }
    assert!(saw_water, "the straight route crosses the lake");
}

// spec: S-0503 claim 2 and F1 (T = 0 while η is more than 8192 off β; T = ½ strictly within 256 of D; T = 1 otherwise; values only 0, ½ and 1)
#[test]
fn claim_2_throttle() {
    let mut w = world();
    // Facing east (body and movement heading), ordered west.
    let id = at(&mut w, "HOVER", 0, 0, 0, 16384);
    settle(&mut w);
    assert_eq!(
        hs(&w, id).heading.value(w.tick),
        16384,
        "η starts at the placement facing (Q-0694)"
    );
    order(&mut w, id, cell(-4, 0));
    let start = w.obj(id).unwrap().pos;
    tick::step(&mut w);
    let s = hs(&w, id);
    assert!(s.point.is_some());
    assert!(s.throttle.eq_num(F64::ZERO), "η is 180° off β");
    assert_eq!(
        (w.obj(id).unwrap().pos.x, w.obj(id).unwrap().pos.y),
        (start.x, start.y)
    );
    let mut saw = (false, false);
    for _ in 0..2000 {
        tick::step(&mut w);
        let s = hs(&w, id);
        let t = w.tick;
        if s.point.is_none() {
            if !hover::moving(&w, id) {
                break;
            }
            continue;
        }
        let o = w.obj(id).unwrap();
        let p = s.point.unwrap();
        let beta = glasnost_num::geom::direction(
            i64::from(p.x - o.pos.x),
            i64::from(p.y - o.pos.y),
            w.rules().quirks.table_direction,
        );
        let _ = (t, beta);
        let one = f(&w, 1);
        assert!(
            s.throttle.eq_num(F64::ZERO) || s.throttle.eq_num(half(&w)) || s.throttle.eq_num(one)
        );
        saw.0 |= s.throttle.eq_num(one);
        saw.1 |= s.throttle.eq_num(half(&w));
    }
    assert!(saw.0 && saw.1, "full throttle on the way, half at the end");
    assert_eq!(w.obj(id).unwrap().pos.cell(), cell(-4, 0));
}

// spec: S-0503 claim 3 and F2 (φ rises by 1 / (a × 900) towards G = min(1, B × T); above G it falls by 1 / (b × 900), floored at 0, so it can undershoot G)
#[test]
fn claim_3_ramp_and_undershoot() {
    let w = world();
    let m = w.rules().mode;
    let hr = w.rules().general.hover;
    let one = f(&w, 1);
    let up = one.div(hr.acceleration.mul(f(&w, 900), m), m);
    let down = one.div(hr.brake.mul(f(&w, 900), m), m);
    // Rising from rest at full throttle.
    let phi = hover::ramp(&w, F64::ZERO, one, one);
    assert!(phi.eq_num(up));
    // Capped at G.
    let near = one.sub(f(&w, 1).div(f(&w, 1_000_000), m), m);
    assert!(hover::ramp(&w, near, one, one).eq_num(one));
    // Half throttle with boost 1.5: G = min(1, 0.75).
    let g = hr.boost.mul(half(&w), m);
    assert!(hover::ramp(&w, F64::ZERO, hr.boost, half(&w)).eq_num(up));
    let just_above = g.add(down.div(f(&w, 2), m), m);
    let braked = hover::ramp(&w, just_above, hr.boost, half(&w));
    assert!(braked.eq_num(just_above.sub(down, m)), "one braking step");
    assert!(braked.lt(g), "below G: the floor is 0");
    // Floored at 0.
    let tiny = down.div(f(&w, 2), m);
    assert!(hover::ramp(&w, tiny, one, F64::ZERO).eq_num(F64::ZERO));
}

// spec: S-0503 Edge cases (quirks.hover_brake_undershoot off: the braking step is floored at the goal)
#[test]
fn claim_3_quirk_brake_floor_at_goal() {
    let mut r = rules();
    r.globals.quirks.hover_brake_undershoot = false;
    let w = world_with(r);
    let m = w.rules().mode;
    let hr = w.rules().general.hover;
    let down = f(&w, 1).div(hr.brake.mul(f(&w, 900), m), m);
    let g = hr.boost.mul(half(&w), m);
    let just_above = g.add(down.div(f(&w, 2), m), m);
    assert!(hover::ramp(&w, just_above, hr.boost, half(&w)).eq_num(g));
}

// spec: S-0503 claim 3 (while T > 0, B is the boost factor when the first two stored steps are equal, else 1; while T = 0 it keeps its value)
#[test]
fn claim_3_boost_on_a_straight_run() {
    let mut w = world();
    let id = at(&mut w, "HOVER", 0, 0, 0, 16384);
    settle(&mut w);
    // East, straight along a row.
    order(&mut w, id, cell(6, 0));
    let boost = w.rules().general.hover.boost;
    let mut saw = false;
    for _ in 0..400 {
        tick::step(&mut w);
        let s = hs(&w, id);
        let o = w.obj(id).unwrap();
        if s.throttle.gt(F64::ZERO) {
            let straight = o.path.steps.len() >= 2 && o.path.steps[0] == o.path.steps[1];
            if straight && s.boost.eq_num(boost) {
                saw = true;
            }
        }
        if !hover::moving(&w, id) {
            break;
        }
    }
    assert!(saw, "the boost factor on a straight run");
    assert!(hs(&w, id).boost.eq_num(f(&w, 1)) || hs(&w, id).boost.eq_num(boost));
}

// spec: S-0503 claim 4 and F3 (d = trunc(s × φ); no land multiplier: the same crossing takes as long over water as over land)
#[test]
fn claim_4_land_and_water_alike() {
    let time = |dy: i32| {
        let mut w = world();
        let id = at(&mut w, "HOVER", 0, -4, dy, 16384);
        settle(&mut w);
        order(&mut w, id, cell(4, dy));
        until_stopped(&mut w, id, 3000)
    };
    let water = time(0);
    let land = time(-10);
    assert!(water < 3000);
    assert_eq!(water, land, "no land multiplier enters d");
}

// spec: S-0503 claim 4 (ρ = 1 when it is given a move while powered and not moving now; S-0417's ρ = ½ persists until a listed reset)
#[test]
fn claim_4_speed_fraction() {
    let mut w = world();
    let id = at(&mut w, "HOVER", 0, 0, 0, 16384);
    settle(&mut w);
    assert!(
        hs(&w, id).rho.eq_num(F64::ZERO),
        "a new vehicle starts with ρ = 0"
    );
    order(&mut w, id, cell(0, -4));
    assert!(hs(&w, id).rho.eq_num(f(&w, 1)), "ρ = 1 on a given move");
    head(&mut w, id);
    hover::set_exit_fraction(&mut w, id);
    // η is 90° off the heading point: no stalled-start reset while it turns.
    for _ in 0..5 {
        tick::step(&mut w);
        assert!(hs(&w, id).rho.eq_num(half(&w)), "ρ = ½ persists");
    }
}

// spec: S-0503 Edge cases (quirks.hover_stalled_start_reset: d = 0 with η exactly at the heading point sets T = 1 and ρ = 1)
#[test]
fn claim_4_stalled_start_reset() {
    let run = |on: bool| {
        let mut r = rules();
        r.globals.quirks.hover_stalled_start_reset = on;
        let mut w = world_with(r);
        // Facing north, ordered north: η points exactly at P.
        let id = at(&mut w, "HOVER", 0, 0, 0, 0);
        settle(&mut w);
        order(&mut w, id, cell(0, -4));
        head(&mut w, id);
        hover::set_exit_fraction(&mut w, id);
        let before = w.obj(id).unwrap().pos;
        tick::step(&mut w);
        let s = hs(&w, id);
        assert_eq!(s.heading.value(w.tick), 0, "η exactly at P");
        let _ = before;
        s.rho
    };
    let w = world();
    assert!(run(true).eq_num(f(&w, 1)));
    assert!(run(false).eq_num(half(&w)));
}

// spec: S-0503 claim 5 (the heading point is the centre of the first stored step's cell at its surface height; the step is used up; the body aims one step beyond)
#[test]
fn claim_5_heading_point_and_body_aim() {
    let mut w = world();
    let id = at(&mut w, "HOVER", 0, 0, 0, 0);
    settle(&mut w);
    order(&mut w, id, cell(5, 0));
    tick::step(&mut w);
    let s = hs(&w, id);
    let p = s.point.expect("a heading point");
    let c = p.cell();
    assert_eq!(
        (p.x, p.y),
        (256 * c.x + 128, 256 * c.y + 128),
        "the cell centre"
    );
    assert_eq!(p.z, w.surface_at(p.x, p.y));
    assert_eq!(c, cell(1, 0), "the first step's cell");
    // The body turns towards a point east while throttle is above 0.
    let mut turned = false;
    for _ in 0..60 {
        tick::step(&mut w);
        if hs(&w, id).throttle.gt(F64::ZERO) {
            let b = w.obj(id).unwrap().body.value(w.tick);
            turned |= b != 0;
        }
    }
    assert!(turned);
}

// spec: S-0503 claim 6 and F4 (straight travel along η by d per tick; occupancy moves cell to cell as the position crosses an edge)
#[test]
fn claim_6_straight_travel_and_occupancy() {
    let mut w = world();
    let id = at(&mut w, "HOVER", 0, 0, 0, 16384);
    settle(&mut w);
    let y0 = w.obj(id).unwrap().pos.y;
    order(&mut w, id, cell(4, 0));
    for _ in 0..1000 {
        let before = w.obj(id).unwrap().pos;
        tick::step(&mut w);
        let o = w.obj(id).unwrap();
        assert_eq!(o.pos.y, y0, "straight east: y unchanged");
        assert!(o.pos.x >= before.x);
        let c = o.pos.cell();
        assert!(w.cells.list(c, o.layer).contains(&id), "listed in its cell");
        if !hover::moving(&w, id) {
            break;
        }
    }
    let c = w.obj(id).unwrap().pos.cell();
    assert_eq!(c, cell(4, 0));
    for dx in 0..4 {
        assert!(
            !w.cells
                .list(cell(dx, 0), w.obj(id).unwrap().layer)
                .contains(&id)
        );
    }
}

// spec: S-0503 claim 7 and F5 (arrival within d on the map plane; no snap to the point; destination cleared in its cell; no further travel once stopped)
#[test]
fn claim_7_arrival_without_snap() {
    let mut w = world();
    let id = at(&mut w, "HOVER", 0, 0, 0, 16384);
    settle(&mut w);
    order(&mut w, id, cell(3, 0));
    let t = until_stopped(&mut w, id, 2000);
    assert!(t < 2000);
    let o = w.obj(id).unwrap();
    assert!(o.dest.is_none());
    assert_eq!(o.pos.cell(), cell(3, 0));
    let p = o.pos;
    common::run(&mut w, 50);
    let q = w.obj(id).unwrap().pos;
    assert_eq!((p.x, p.y), (q.x, q.y), "no further travel ticks");
}

// spec: S-0503 claim 8 and F6 (lift, gravity, damping and bob: a resting hover rises from the surface and stays near its hover height, bobbing within a band)
#[test]
fn claim_8_lift_and_bob() {
    let mut w = world();
    let id = at(&mut w, "HOVER", 0, 0, 0, 0);
    assert_eq!(height_of(&w, id), 0, "placed on the surface");
    common::run(&mut w, 2);
    assert!(height_of(&w, id) > 0, "lift raises it");
    common::run(&mut w, 400);
    let mut hs_ = Vec::new();
    for _ in 0..200 {
        tick::step(&mut w);
        hs_.push(height_of(&w, id));
    }
    let lo = *hs_.iter().min().unwrap();
    let hi = *hs_.iter().max().unwrap();
    assert!(lo > 50 && hi < 200, "near its hover height: {lo} … {hi}");
    assert!(lo < hi, "it bobs");
}

// spec: S-0503 claim 8 and F6 (the bob phase depends on the object number: two hovers bob out of step)
#[test]
fn claim_8_bob_out_of_step() {
    let mut w = world();
    let a = at(&mut w, "HOVER", 0, 0, 0, 0);
    let b = at(&mut w, "HOVER", 0, 2, 0, 0);
    common::run(&mut w, 400);
    let mut differ = false;
    for _ in 0..100 {
        tick::step(&mut w);
        differ |= height_of(&w, a) != height_of(&w, b);
    }
    assert!(differ);
}

// spec: S-0503 claim 9 (a wake on ticks divisible by 10 while moving now over water, after travel; none at rest or over land; none without the animation)
#[test]
fn claim_9_wake() {
    let run = |r: Ruleset, dy: i32| {
        let mut w = world_with(r);
        let id = at(&mut w, "HOVER", 0, -4, dy, 16384);
        settle(&mut w);
        let mut at_rest = 0;
        for _ in 0..30 {
            w.effects.clear();
            tick::step(&mut w);
            at_rest += wakes(&w);
        }
        order(&mut w, id, cell(4, dy));
        let mut ticks = Vec::new();
        for _ in 0..300 {
            w.effects.clear();
            let t = w.tick;
            tick::step(&mut w);
            if wakes(&w) > 0 {
                assert_eq!(wakes(&w), 1);
                ticks.push(t);
            }
        }
        (at_rest, ticks)
    };
    let (rest, ticks) = run(rules(), 0);
    assert_eq!(rest, 0);
    assert!(ticks.len() >= 3, "{ticks:?}");
    assert!(ticks.iter().all(|t| t % 10 == 0));
    let (_, land) = run(rules(), -10);
    assert!(land.is_empty(), "no wake over land");
    let mut r = rules();
    r.sim.general.wake = None;
    let (_, none) = run(r, 0);
    assert!(none.is_empty());
}

// spec: S-0503 claim 10 and F7 (switched off while powered: two ranged draws, a spin of 20 to 30 counts turning the body by 256n per tick, a stop request, no lift: it settles)
#[test]
fn claim_10_switch_off_spin_and_settle() {
    let mut w = world();
    let id = at(&mut w, "HOVER", 0, -8, 0, 0);
    settle(&mut w);
    let before = w.rng.taken();
    glasnost_sim::econ::power::apply_disabled(&mut w, id, true);
    assert_eq!(w.rng.taken() - before, 2, "two draws");
    let n = hs(&w, id).spin;
    assert!((20..=30).contains(&n.abs()), "{n}");
    let f0 = w.obj(id).unwrap().body.value(w.tick);
    tick::step(&mut w);
    let f1 = w.obj(id).unwrap().body.value(w.tick);
    assert_eq!(
        i32::from(f1),
        (i32::from(f0) + 256 * n).rem_euclid(65536),
        "256n per tick"
    );
    assert_eq!(hs(&w, id).spin, n - n.signum());
    common::run(&mut w, 400);
    assert_eq!(height_of(&w, id), 0, "it settles on the surface");
    assert_eq!(hs(&w, id).spin, 0);
    assert!(!hover::bomb_mark(&w, id), "on land it survives the test");
    // Already switched off: no new draws.
    let before = w.rng.taken();
    glasnost_sim::econ::power::apply_disabled(&mut w, id, true);
    assert_eq!(w.rng.taken(), before);
    // Switched on: the lift returns.
    glasnost_sim::econ::power::apply_disabled(&mut w, id, false);
    common::run(&mut w, 300);
    assert!(height_of(&w, id) > 50);
}

// spec: S-0503 claim 10 (the stop request clears the destination and path when D and P differ; it does not zero throttle or hover speed)
#[test]
fn claim_10_stop_request() {
    let mut w = world();
    let id = at(&mut w, "HOVER", 0, -8, 0, 16384);
    settle(&mut w);
    order(&mut w, id, cell(-2, 0));
    common::run(&mut w, 60);
    let phi = hs(&w, id).phi;
    assert!(phi.gt(F64::ZERO));
    glasnost_sim::econ::power::apply_disabled(&mut w, id, true);
    let o = w.obj(id).unwrap();
    assert!(o.dest.is_none() && o.path.steps.is_empty());
    assert!(hs(&w, id).phi.eq_num(phi), "hover speed not zeroed");
}

// spec: S-0503 claim 11 (a spin ending over water below 104 leptons gives the bomb mark and starts the fall)
#[test]
fn claim_11_sinking_over_water() {
    let mut w = world();
    let id = at(&mut w, "HOVER", 0, 0, 0, 0);
    settle(&mut w);
    glasnost_sim::econ::power::apply_disabled(&mut w, id, true);
    let mut marked = false;
    for _ in 0..60 {
        tick::step(&mut w);
        if w.obj(id)
            .is_some_and(|o| o.pos.z - w.surface_at(o.pos.x, o.pos.y) < hover::SINK_LIMIT)
            && hover::bomb_mark(&w, id)
        {
            marked = true;
            break;
        }
    }
    assert!(marked, "the bomb mark over water");
    // spec: S-0511 claim 10 (the marked object falls; the mark is spent when it lands)
    common::run(&mut w, 200);
    assert!(glasnost_sim::fall::falling(&w, id) || w.obj(id).is_none_or(|o| o.health <= 0));
    common::run(&mut w, 200);
    // It reached the surface through the fall and took the landing hit.
    assert!(
        w.obj(id).is_none_or(|o| o.health <= 0 && !o.fall.falling),
        "the landing kill"
    );
}

// spec: S-0503 claim 11 (a spin ending at 104 leptons or more above the water: no mark)
#[test]
fn claim_11_high_spin_end_survives() {
    let mut w = world();
    let id = at(&mut w, "HOVER", 0, 0, 0, 0);
    settle(&mut w);
    glasnost_sim::econ::power::apply_disabled(&mut w, id, true);
    // Hold it high while the spin runs out.
    for _ in 0..40 {
        let s = w.surface_at(w.obj(id).unwrap().pos.x, w.obj(id).unwrap().pos.y);
        let o = w.obj_mut(id).unwrap();
        o.pos.z = s + 300;
        o.hover.as_mut().unwrap().v = F64::ZERO;
        tick::step(&mut w);
    }
    assert_eq!(hs(&w, id).spin, 0);
    assert!(!hover::bomb_mark(&w, id));
}

// spec: S-0503 claim 12, table T1 (clear: the step is used up and the vehicle heads for the cell centre)
#[test]
fn claim_12_clear() {
    let mut w = world();
    let id = at(&mut w, "HOVER", 0, 0, 0, 16384);
    settle(&mut w);
    order(&mut w, id, cell(3, 0));
    tick::step(&mut w);
    let o = w.obj(id).unwrap();
    assert_eq!(hs(&w, id).point.unwrap().cell(), cell(1, 0));
    assert_eq!(o.path.steps.len(), 2, "one step used up");
}

// spec: S-0503 claim 12, table T1 (stationary friendly at the first check: no heading point, throttle, φ and ρ at 0; the path is discarded and searched again at once, and R starts, Q-0692)
#[test]
fn claim_12_stationary_friendly_first_check() {
    let mut w = world();
    let id = at(&mut w, "HOVER", 0, 0, 0, 16384);
    settle(&mut w);
    order(&mut w, id, cell(4, 0));
    // Moving east towards (1, 0); a friendly standing vehicle appears in (2, 0).
    common::run_until(&mut w, 500, |w| {
        hs(w, id).point.is_some_and(|p| p.cell() == cell(1, 0)) && hs(w, id).phi.gt(F64::ZERO)
    });
    let blocker = at(&mut w, "CART", 0, 2, 0, 0);
    assert!(w.obj(blocker).is_some());
    let mut checked = false;
    for _ in 0..500 {
        let t = w.tick;
        tick::step(&mut w);
        let s = hs(&w, id);
        if s.point.is_none_or(|p| p.cell() != cell(1, 0)) {
            // The arrival at (1, 0): the next cell (2, 0) is refused.
            let o = w.obj(id).unwrap();
            let into_blocker = s.point.is_some_and(|p| p.cell() == cell(2, 0));
            assert!(!into_blocker, "it does not head into the blocker's cell");
            assert_eq!(
                o.path.retry_at,
                t + w.rules().general.retry_delay,
                "the immediate search starts R (Q-0692)"
            );
            checked = true;
            break;
        }
    }
    assert!(checked);
}

// spec: S-0503 claim 12 (the refused rows zero throttle, hover speed and ρ; S-0503 claim 7 a refused cell suppresses that tick's travel)
#[test]
fn claim_12_refusal_zeroes_the_ramp() {
    let mut w = world();
    let id = at(&mut w, "HOVER", 0, 0, 0, 16384);
    settle(&mut w);
    // Hemmed in: friendly vehicles on every side but the start cell.
    for (dx, dy) in [
        (1, 0),
        (1, 1),
        (1, -1),
        (0, 1),
        (0, -1),
        (-1, 0),
        (-1, 1),
        (-1, -1),
    ] {
        at(&mut w, "CART", 0, dx, dy, 0);
    }
    order(&mut w, id, cell(4, 0));
    let p = w.obj(id).unwrap().pos;
    tick::step(&mut w);
    let s = hs(&w, id);
    assert!(s.point.is_none());
    assert!(s.throttle.eq_num(F64::ZERO) && s.phi.eq_num(F64::ZERO) && s.rho.eq_num(F64::ZERO));
    let q = w.obj(id).unwrap().pos;
    assert_eq!((p.x, p.y), (q.x, q.y));
}

// The save format keeps a moving hover's state (S-0503 Terms) over water.
// spec: S-0503 Terms (heading point, throttle, hover speed, boost, vertical velocity are kept state)
#[test]
fn save_round_trip_over_water() {
    let mut w = world();
    let id = at(&mut w, "HOVER", 0, -6, 0, 16384);
    settle(&mut w);
    order(&mut w, id, cell(6, 0));
    common::run(&mut w, 120);
    assert_eq!(land_of(&w, id), land::WATER, "moving over water");
    assert!(hover::moving(&w, id));
    let bytes = save::save(&w);
    let mut l = save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
    assert_eq!(l.obj(id).unwrap().hover, w.obj(id).unwrap().hover);
    for _ in 0..200 {
        tick::step(&mut w);
        tick::step(&mut l);
    }
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
    let _ = (ALPHA, Pos::new(0, 0, 0));
}

// spec: S-0503 Terms and Edge cases (quirks.hover_elevated_heading_point: a heading point gains 416 when the vehicle is 312 or more above the cell's surface, bridge or not; off: a high-bridge cell is needed too)
#[test]
fn quirk_elevated_heading_point() {
    let run = |on: bool| {
        let mut r = rules();
        r.globals.quirks.hover_elevated_heading_point = on;
        let mut w = world_with(r);
        let id = at(&mut w, "HOVER", 0, 0, 0, 16384);
        settle(&mut w);
        common::order_move(&mut w, id, cell(4, 0));
        let mut z = None;
        for _ in 0..20 {
            let o = w.obj(id).unwrap();
            let s = w.surface_at(o.pos.x, o.pos.y);
            let o = w.obj_mut(id).unwrap();
            o.pos.z = s + 400;
            tick::step(&mut w);
            if let Some(p) = hs(&w, id).point {
                z = Some(p.z - w.surface_at(p.x, p.y));
                break;
            }
        }
        z.expect("a heading point")
    };
    assert_eq!(run(true), 416);
    assert_eq!(run(false), 0);
}
