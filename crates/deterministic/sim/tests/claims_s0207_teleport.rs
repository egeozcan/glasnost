//! Claim tests for the teleporting locomotor's own move (S-0207 claim 20,
//! F9) on synthetic data: invented vehicle types and rules values on the
//! flat scene of `common`.

mod common;

use common::*;
use glasnost_map::CellXY;
use glasnost_num::geom::root_floor;
use glasnost_rules::sim::{Locomotor, UnitType};
use glasnost_sim::rules::Kind;
use glasnost_sim::teleport;
use glasnost_sim::world::{Layer, ObjectId, Pos, Target, World};

/// A vehicle type with the teleporting locomotor; `harvester` sets the
/// harvester flag.
fn jumper(harvester: bool) -> UnitType {
    let mut t = unit_type(WHEEL, 6, 5);
    t.locomotor = Some(Locomotor::Teleport);
    t.harvester = Some(harvester);
    t
}

/// A scene with JUMPER placed at the middle cell facing map east.
fn scene(harvester: bool) -> Scene {
    let mut s = Scene::new();
    s.vehicle("JUMPER", jumper(harvester));
    s.place_unit(0, "JUMPER", cell(0, 0), 64);
    s
}

fn start(s: &Scene) -> (World, ObjectId) {
    let mut w = s.build();
    open_orders(&mut w);
    let id = objects_of(&w, Kind::Vehicle)[0];
    (w, id)
}

fn centre(w: &World, c: CellXY) -> Pos {
    let p = Pos::centre(c, 0);
    Pos::centre(c, w.surface_at(p.x, p.y))
}

fn marked(w: &World, c: CellXY) -> bool {
    w.cells
        .occ(c, Layer::Ground)
        .is_some_and(|o| o.vehicle_present())
}

/// F9 with the rules' values, computed here from the formula.
fn f9(w: &World, from: Pos, to: Pos) -> i64 {
    let g = &w.rules().general;
    let (dx, dy, dz) = (
        i64::from(to.x - from.x),
        i64::from(to.y - from.y),
        i64::from(to.z - from.z),
    );
    let d = root_floor(
        (dx * dx + dy * dy + dz * dz) as u64,
        w.rules().quirks.table_root_distance,
        w.rules().mode,
    ) as i64;
    let k = i64::from(g.chrono_distance_factor);
    let q = if g.chrono_trigger && k != 0 { d / k } else { 0 };
    let m = i64::from(g.chrono_minimum_delay);
    if d < i64::from(g.chrono_range_minimum) {
        m
    } else if q.max(0) > m {
        q
    } else {
        m
    }
}

// spec: S-0207 claim 20 (a) (the landing point fixed and the occupancy mark moved when the order applies; the object stays listed in its old cell until the jump)
// spec: S-0207 claim 20 (d) and (e) (the jump on the next movement update: no turn, the centre of the destination cell at H, listed there, the facing kept, the destination removed)
// spec: S-0207 claim 20 (f) (phasing in from the jump)
#[test]
fn claim_20_jump_timing_and_effects() {
    let s = scene(false);
    let (mut w, id) = start(&s);
    let from = w.obj(id).unwrap().pos;
    let f0 = w.obj(id).unwrap().body.value(w.tick);
    let dc = cell(0, -15);
    order_move(&mut w, id, dc);
    run(&mut w, 1);
    // The order applied after the turn pass: the mark moved, the object did not.
    let o = w.obj(id).unwrap();
    assert_eq!(o.pos, from);
    assert_eq!(o.dest, Some(Target::Cell(dc)));
    assert!(marked(&w, dc) && !marked(&w, cell(0, 0)));
    assert!(w.cells.list(cell(0, 0), Layer::Ground).contains(&id));
    assert!(glasnost_sim::occupancy::counts_as_moving(&w, id));
    run(&mut w, 1);
    let t0 = w.tick - 1;
    let o = w.obj(id).unwrap();
    assert_eq!(o.pos, centre(&w, dc));
    assert_eq!(o.layer, Layer::Ground);
    assert_eq!(o.body.value(w.tick), f0);
    assert_eq!(o.dest, None);
    assert!(w.cells.list(dc, Layer::Ground).contains(&id));
    assert!(!w.cells.list(cell(0, 0), Layer::Ground).contains(&id));
    assert!(marked(&w, dc) && !marked(&w, cell(0, 0)));
    assert!(teleport::phasing_in(&w, id));
    // spec: S-0207 F9 (D from the distance before the jump; k = 32 and m = g = 0 when no layer gives them)
    let d = f9(&w, from, centre(&w, dc));
    assert_eq!(d, (15 * 256) / 32);
    assert_eq!(teleport::delay(&w, from, centre(&w, dc)), d);
    while w.tick < t0 + d as i32 {
        assert!(teleport::phasing_in(&w, id), "tick {}", w.tick);
        run(&mut w, 1);
    }
    // The turn of tick t₀ + D ends it.
    assert!(teleport::phasing_in(&w, id));
    run(&mut w, 1);
    assert!(!teleport::phasing_in(&w, id));
}

// spec: S-0207 claim 20 (f) and (g) (a harvester-flag type does not phase in: a further order is jumped to on the next movement update)
#[test]
fn claim_20_harvester_does_not_phase_in() {
    let s = scene(true);
    let (mut w, id) = start(&s);
    order_move(&mut w, id, cell(5, 0));
    run(&mut w, 2);
    assert_eq!(w.obj(id).unwrap().pos, centre(&w, cell(5, 0)));
    assert!(!teleport::phasing_in(&w, id));
    order_move(&mut w, id, cell(-5, 3));
    run(&mut w, 2);
    assert_eq!(w.obj(id).unwrap().pos, centre(&w, cell(-5, 3)));
}

// spec: S-0207 claim 20 (a) and (f) (a destination given while phasing in is dropped: lost, not delayed)
#[test]
fn claim_20_orders_while_phasing_in_are_lost() {
    let s = scene(false);
    let (mut w, id) = start(&s);
    order_move(&mut w, id, cell(0, -15));
    run(&mut w, 3);
    assert!(teleport::phasing_in(&w, id));
    let at = w.obj(id).unwrap().pos;
    order_move(&mut w, id, cell(6, -15));
    run(&mut w, 1);
    assert_eq!(w.obj(id).unwrap().dest, None);
    run(&mut w, 200);
    assert!(!teleport::phasing_in(&w, id));
    assert_eq!(w.obj(id).unwrap().pos, at);
}

// spec: S-0207 claim 20 (b) (an occupied destination: the nearby-cell search's cell, not the destination cell)
#[test]
fn claim_20_occupied_destination() {
    let mut s = scene(false);
    s.vehicle("BLOCK", unit_type(WHEEL, 6, 5));
    let dc = cell(0, -8);
    s.place_unit(0, "BLOCK", dc, 0);
    let (mut w, id) = start(&s);
    order_move(&mut w, id, dc);
    run(&mut w, 2);
    let c = w.obj(id).unwrap().pos.cell();
    assert_ne!(c, dc);
    assert_eq!(w.obj(id).unwrap().pos, centre(&w, c));
    assert!((c.x - dc.x).abs() <= 1 && (c.y - dc.y).abs() <= 1);
}

// spec: S-0207 claim 20 (d) (L equal to its own position: no jump, the destination removed, no phasing in)
#[test]
fn claim_20_own_cell_destination() {
    let s = scene(false);
    let (mut w, id) = start(&s);
    let p = w.obj(id).unwrap().pos;
    assert_eq!(p, centre(&w, cell(0, 0)));
    order_move(&mut w, id, cell(0, 0));
    run(&mut w, 2);
    let o = w.obj(id).unwrap();
    assert_eq!((o.pos, o.dest), (p, None));
    assert!(!teleport::phasing_in(&w, id));
}

// spec: S-0207 claim 20 (e) (every object that has it as its target drops it at the jump)
#[test]
fn claim_20_targets_drop_the_jumper() {
    let mut s = scene(false);
    s.vehicle("WATCHER", unit_type(WHEEL, 6, 5));
    s.place_unit(1, "WATCHER", cell(5, 5), 0);
    let (mut w, id) = start(&s);
    let watcher = objects_of(&w, Kind::Vehicle)[1];
    w.obj_mut(watcher).unwrap().target = Some(Target::Object(id));
    order_move(&mut w, id, cell(0, -10));
    run(&mut w, 1);
    assert_eq!(w.obj(watcher).unwrap().target, Some(Target::Object(id)));
    run(&mut w, 1);
    assert_eq!(w.obj(watcher).unwrap().target, None);
}

// spec: S-0207 claim 20 (f) and (i) (at the end of phasing in, with no target and more than 10 ticks on the targeting timer, one ranged draw over 4 to 8 ticks; the jump itself takes none)
#[test]
fn claim_20_draw_at_the_end_of_phasing_in() {
    let s = scene(false);
    // The jump tick takes the draws of the same tick without a jump (an order to the own cell): the move behaviour's own.
    let taken_in_order_tick = |to: CellXY| {
        let (mut w, id) = start(&s);
        order_move(&mut w, id, to);
        run(&mut w, 1);
        let n = w.rng.taken();
        run(&mut w, 1);
        (w.rng.taken() - n, teleport::phasing_in(&w, id))
    };
    let (jump, phasing) = taken_in_order_tick(cell(0, -4));
    let (stay, control) = taken_in_order_tick(cell(0, 0));
    assert!(phasing && !control);
    assert_eq!(jump, stay, "the jump takes no draw");
    // The end tick, with the targeting timer `left` ticks ahead of it: the
    // tick, the draw state before and after it, and the timer after it.
    let end_tick = |left: i32| {
        let (mut w, id) = start(&s);
        order_move(&mut w, id, cell(0, -4));
        run(&mut w, 2);
        assert!(teleport::phasing_in(&w, id));
        loop {
            let t = w.tick;
            w.obj_mut(id).unwrap().targeting_timer = t + left;
            let before = w.rng.clone();
            run(&mut w, 1);
            if !teleport::phasing_in(&w, id) {
                let timer = w.obj(id).unwrap().targeting_timer;
                return (
                    t,
                    before,
                    w.rng.clone(),
                    timer,
                    w.rules().quirks.ranged_draw_half_span,
                );
            }
        }
    };
    // 10 ticks left: no draw of its own, the timer kept.
    let (end, before, short, timer, half) = end_tick(10);
    assert_eq!(timer, end + 10);
    // More than 10 ticks left: the same tick plus one ranged draw over 4 to
    // 8, after the draws of that tick's processing, which sets the timer.
    let (end_long, before_long, long, timer) = {
        let (e, b, a, t, _) = end_tick(11);
        (e, b, a, t)
    };
    assert_eq!((end_long, &before_long), (end, &before));
    let mut r = before.clone();
    for _ in 0..short.taken() - before.taken() {
        r.raw();
    }
    let v = r.ranged(4, 8, half);
    assert_eq!(r, long);
    assert_eq!(timer, end + v);
}

// spec: S-0207 F9 (trigger off: D = m; d below the range minimum: D = m; D = q when max(q, 0) > m, so a negative m gives q) and Edge cases (k = 0 with the trigger on: q = 0)
#[test]
fn f9_phasing_in_delay() {
    let from = Pos::new(0, 0, 0);
    let to = Pos::new(256 * 20, 0, 0);
    let delay = |k: i32, trigger: bool, m: i32, g: i32| {
        let mut s = scene(false);
        let gl = &mut s.ruleset.sim.general;
        gl.chrono_distance_factor = Some(k);
        gl.chrono_trigger = Some(trigger);
        gl.chrono_minimum_delay = Some(m);
        gl.chrono_range_minimum = Some(g);
        let w = s.build();
        teleport::delay(&w, from, to)
    };
    // d = 5120.
    assert_eq!(delay(32, true, 0, 0), 160);
    assert_eq!(delay(32, true, 200, 0), 200);
    assert_eq!(delay(32, false, 7, 0), 7);
    assert_eq!(delay(32, true, 7, 6000), 7);
    assert_eq!(delay(-32, true, -500, 0), -160);
    assert_eq!(delay(0, true, 3, 0), 3);
}
