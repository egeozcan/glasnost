//! Both positions of simulation quirk switches that a spec names and that
//! no code read before #327: `table_trig_negative_bias` (S-0105 F1, as it
//! bears on S-0208 F3), `unchecked_substitute`, `repeat_direction_snap` and
//! `air_death_fall_generic_follow_up` (S-0107). Synthetic data only: an
//! invented soldier type with invented sequence entries.

mod common;

use common::{FOOT, Scene, cell, open_orders, order_move, run, unit_type};
use glasnost_map::CellXY;
use glasnost_rules::sim::SequenceEntry;
use glasnost_sim::infantry::{self, Seq};
use glasnost_sim::rules::Kind;
use glasnost_sim::turning::Turning;
use glasnost_sim::world::{ObjectId, Pos, World};

fn entry(count: i32, direction: Option<u8>) -> SequenceEntry {
    SequenceEntry {
        start: 0,
        count,
        multiplier: 0,
        direction,
    }
}

/// A scene with one soldier SOLDIER at cell (0, 0), whose type has the
/// given sequence entries; `quirks` adjusts the ruleset's switches.
fn soldier(
    entries: &[(&str, SequenceEntry)],
    quirks: impl FnOnce(&mut glasnost_rules::quirks::Quirks),
) -> (World, ObjectId) {
    let mut s = Scene::new();
    let mut t = unit_type(FOOT, 4, 0);
    for (k, e) in entries {
        t.sequences.insert((*k).into(), e.clone());
    }
    s.infantry("SOLDIER", t);
    quirks(&mut s.ruleset.globals.quirks);
    s.place_infantry(0, "SOLDIER", cell(0, 0), 0);
    let w = s.build();
    let id = common::objects_of(&w, Kind::Infantry)[0];
    (w, id)
}

fn seq(w: &World, id: ObjectId) -> Option<Seq> {
    w.obj(id).unwrap().infantry.as_ref().unwrap().seq
}

/// Makes `q` the current kind, started `ago` ticks back with stage delay 1.
fn current(w: &mut World, id: ObjectId, q: Seq, ago: i32) {
    let t = w.tick;
    let st = w.obj_mut(id).unwrap().infantry.as_mut().unwrap();
    st.seq = Some(q);
    st.seq_t0 = t - ago;
    st.seq_delay = 1;
}

/// The kind a forced walk request starts for a soldier at fear 250 whose
/// type has no *panic* frames.
fn panicked_walk(unchecked: bool) -> Option<Seq> {
    let (mut w, id) = soldier(
        &[("stand", entry(4, None)), ("walk", entry(4, None))],
        |q| q.unchecked_substitute = unchecked,
    );
    current(&mut w, id, Seq::Stand, 0);
    w.obj_mut(id).unwrap().infantry.as_mut().unwrap().fear = 250;
    infantry::request(&mut w, id, Seq::Walk, true);
    seq(&w, id)
}

// spec: S-0107 claim 7 and Edge cases (quirks.unchecked_substitute on: a substitute with C = 0 is still started; off: the requested kind is kept)
#[test]
fn unchecked_substitute_both_positions() {
    assert_eq!(panicked_walk(true), Some(Seq::Panic));
    assert_eq!(panicked_walk(false), Some(Seq::Walk));
}

/// The body facing after a completed *stand* with declared direction 2 is
/// snapped, the facing is then set to 0, and one more tick's sequence
/// rules run.
fn facing_after_a_second_tick(repeat: bool) -> (u16, u16) {
    let (mut w, id) = soldier(&[("stand", entry(1, Some(2)))], |q| {
        q.repeat_direction_snap = repeat
    });
    current(&mut w, id, Seq::Stand, 5);
    infantry::sequence_turn(&mut w, id, false, false);
    let t = w.tick;
    let first = w.obj(id).unwrap().body.value(t);
    assert_eq!(seq(&w, id), Some(Seq::Stand), "stand stays current");
    w.obj_mut(id).unwrap().body = Turning::at(0);
    w.tick += 1;
    infantry::sequence_turn(&mut w, id, false, false);
    let t = w.tick;
    (first, w.obj(id).unwrap().body.value(t))
}

// spec: S-0107 claim 9 and Edge cases (quirks.repeat_direction_snap on: a completed looping kind with a declared direction re-applies it every tick; off: once, on the completion tick)
#[test]
fn repeat_direction_snap_both_positions() {
    assert_eq!(facing_after_a_second_tick(true), (16384, 16384));
    assert_eq!(facing_after_a_second_tick(false), (16384, 0));
}

/// The kind after a completed *air-death-fall*'s sequence rules.
fn after_air_death_fall(generic: bool) -> Option<Seq> {
    let (mut w, id) = soldier(
        &[
            ("stand", entry(4, None)),
            ("air_death_fall", entry(2, None)),
        ],
        |q| q.air_death_fall_generic_follow_up = generic,
    );
    current(&mut w, id, Seq::AirDeathFall, 10);
    infantry::sequence_turn(&mut w, id, false, false);
    seq(&w, id)
}

// spec: S-0107 claim 21 and Edge cases (quirks.air_death_fall_generic_follow_up on: a completed air-death-fall gets the general follow-up; off: it loops until air-death-end replaces it)
#[test]
fn air_death_fall_follow_up_both_positions() {
    assert_eq!(after_air_death_fall(true), Some(Seq::Stand));
    assert_eq!(after_air_death_fall(false), Some(Seq::AirDeathFall));
}

/// A fast soldier's positions over 40 ticks of a walk to `to` (a fast
/// one, so that one table step of the angle moves a step's truncation).
fn walked(bias: bool, to: CellXY) -> Vec<Pos> {
    let mut s = Scene::new();
    s.infantry("SOLDIER", unit_type(FOOT, 60, 0));
    s.ruleset.globals.quirks.table_trig_negative_bias = bias;
    // spec: S-0208 Edge cases (quirks.walker_overshoot off: a step that would end 17 or more beyond its target ends on it, so a fast walker does not swing about its standing position)
    s.ruleset.globals.quirks.walker_overshoot = false;
    s.place_infantry(0, "SOLDIER", cell(0, 0), 0);
    let mut w = s.build();
    let id = common::objects_of(&w, Kind::Infantry)[0];
    // Off its cell's centre, so that its first heading is none of the
    // eight directions between cells.
    let o = w.obj_mut(id).unwrap();
    o.pos = Pos::new(o.pos.x + 37, o.pos.y + 11, o.pos.z);
    open_orders(&mut w);
    order_move(&mut w, id, to);
    (0..40)
        .map(|_| {
            run(&mut w, 1);
            w.obj(id).unwrap().pos
        })
        .collect()
}

/// sinT of `half_steps` half table steps (I = `half_steps`, S-0105 F1)
/// in a world whose ruleset sets the switch to `bias`.
fn world_sin(bias: bool, half_steps: i64) -> glasnost_num::F64 {
    use glasnost_num::{F64, Rounding};
    let (w, _) = soldier(&[], |q| q.table_trig_negative_bias = bias);
    let rz = Rounding::TowardZero;
    // x = I / 2607, a little over I / 2607.594482421875, so trunc(x × 2607.594…) = I.
    let x = F64::from_i64(half_steps, rz).div(F64::from_i64(2607, rz), rz);
    glasnost_sim::table_trig::sin_q(&w, x, rz)
}

// spec: S-0105 F1 and Quirks (quirks.table_trig_negative_bias on: a negative I is rounded short; off: exact rounding; a non-negative I is the same in both positions)
#[test]
fn table_trig_negative_bias_both_positions() {
    assert_ne!(world_sin(true, -5), world_sin(false, -5));
    assert_eq!(world_sin(true, 5), world_sin(false, 5));
    assert_eq!(world_sin(true, -4), world_sin(false, -4));
}

// spec: S-0208 F3 and Edge cases (quirks.table_trig_negative_bias, S-0105 F1: θ is negative exactly for facings 16384 to 32767, so walks heading elsewhere take the same steps in both positions)
#[test]
fn table_trig_negative_bias_leaves_other_walks() {
    for c in [cell(-12, 7), cell(-12, -3), cell(7, -12)] {
        assert_eq!(walked(true, c), walked(false, c), "{c:?}");
    }
}

// spec: S-0208 F3 (θ = (f̂ − 16383) × c₂ with quirks.table_direction on) and S-0105 F1 and Quirks (quirks.table_trig_negative_bias: only a negative I is rounded differently, so only facings 16384 to 32767 can step differently)
#[test]
fn table_trig_negative_bias_steps_by_facing() {
    let mode = glasnost_num::Rounding::TowardZero;
    let differs = |f: u16| {
        glasnost_sim::walk::step_point_b(0, 0, f, 1000, true, mode, true)
            != glasnost_sim::walk::step_point_b(0, 0, f, 1000, true, mode, false)
    };
    assert!((16384..=32767).any(differs));
    assert!(!(0..16384).any(differs));
    assert!(!(32768..=65535).any(differs));
}
