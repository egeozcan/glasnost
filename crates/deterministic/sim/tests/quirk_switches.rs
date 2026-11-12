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

/// The soldier's positions over 40 ticks of a walk to `to`.
fn walked(bias: bool, to: CellXY) -> Vec<Pos> {
    let (mut w, id) = soldier(&[], |q| q.table_trig_negative_bias = bias);
    open_orders(&mut w);
    order_move(&mut w, id, to);
    (0..40)
        .map(|_| {
            run(&mut w, 1);
            w.obj(id).unwrap().pos
        })
        .collect()
}

// spec: S-0208 Edge cases (quirks.table_trig_negative_bias, S-0105 F1: θ is negative exactly for facings 16384 to 32767, so the switch can change some steps of walkers heading between map east and map south; it changes nothing for other headings) and S-0105 Edge cases (off: exact rounding)
#[test]
fn table_trig_negative_bias_both_positions() {
    let south_east = [cell(5, 3), cell(4, 1), cell(3, 5), cell(6, 2), cell(2, 5)];
    assert!(
        south_east.iter().any(|&c| walked(true, c) != walked(false, c)),
        "some walk between map east and map south takes a different step"
    );
    for c in [cell(-5, 3), cell(-4, -1), cell(3, -5)] {
        assert_eq!(walked(true, c), walked(false, c), "{c:?}");
    }
}
