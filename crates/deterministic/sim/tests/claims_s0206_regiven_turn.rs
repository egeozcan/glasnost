//! S-0206 claims 3 and 12 with S-0207 State and ordering (sim batch S13,
//! #261): a turn in place given again while the body is already turning
//! to the step's facing keeps that turn's start and duration (claim 3), so
//! the track begins in the tick the body reaches the step's facing, not
//! a whole duration after the re-given tick. Synthetic type (wheel class,
//! speed 40, turn-rate field 1).

mod common;

use common::{Scene, WHEEL, cell, objects_of, open_orders, order_move, unit_type};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::Kind;
use glasnost_sim::tick;
use glasnost_sim::world::HouseId;

// spec: S-0206 claim 3 (the same target again changes nothing: the turn keeps its start, duration and timing) and claim 12 (the track begins in the tick in which the body reaches 8192 × d); S-0207 State and ordering (turning in place: the facing reaches the step's direction, begin a track in that tick)
#[test]
fn a_regiven_turn_in_place_begins_the_track_when_the_body_arrives() {
    let mut s = Scene::new();
    s.vehicle("TESTCAR", unit_type(WHEEL, 40, 1));
    s.place_unit(0, "TESTCAR", cell(0, 0), 0);
    let mut w = s.build();
    open_orders(&mut w);
    let id = objects_of(&w, Kind::Vehicle)[0];
    let start = w.obj(id).unwrap().pos;

    order_move(&mut w, id, cell(6, 0));
    let mut first = None;
    for _ in 0..20 {
        tick::step(&mut w);
        first = w.obj(id).unwrap().drive.as_ref().and_then(|d| d.turn);
        if first.is_some() {
            break;
        }
    }
    let (t0, theta) = first.expect("a turn in place was given");
    assert!(theta > 6, "a turn long enough to stop and re-give inside it");
    let arrives = t0 + theta;

    // Into the turn: stop. The body keeps its turn.
    tick::step(&mut w);
    tick::step(&mut w);
    assert!(orders::issue(&mut w, HouseId(0), Order::Stop { actor: id }));
    tick::step(&mut w);
    tick::step(&mut w);
    let o = w.obj(id).unwrap();
    assert_eq!(o.body.t0 + o.body.duration, arrives);

    // The same move again.
    order_move(&mut w, id, cell(6, 0));
    let mut moved_at = None;
    for _ in 0..200 {
        tick::step(&mut w);
        if w.obj(id).unwrap().pos != start {
            moved_at = Some(w.tick - 1);
            break;
        }
    }
    assert_eq!(moved_at.expect("the vehicle moved"), arrives);
}
