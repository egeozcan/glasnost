//! S-0207 claims 11 and 12 with `quirks.drive_track_end_cap` off (sim
//! batch S13, #262): a tick begun without a track that goes on to begin
//! its path's last track and does not end it keeps that track's fraction;
//! rest with ρ = 0 is only for a vehicle that reaches a track target with
//! no further step. Synthetic type (speed 100, accelerating, factor 0.7).

mod common;

use common::*;
use glasnost_num::F64;
use glasnost_rules::sim::Decimal;
use glasnost_sim::rules::Kind;
use glasnost_sim::tick;
use glasnost_sim::world::{ObjectId, World};

const N: u8 = 0;

fn start(steps: &[u8]) -> (World, ObjectId) {
    let mut t = unit_type(WHEEL, 100, 5);
    t.accelerates = Some(true);
    t.acceleration_factor = Some(Decimal("0.7".into()));
    t.slowdown_distance = Some(1);
    let mut s = Scene::new();
    s.vehicle("TESTCAR", t);
    s.place_unit(0, "TESTCAR", cell(0, 0), 0);
    s.ruleset.globals.quirks.drive_track_end_cap = false;
    let mut w = s.build();
    let id = objects_of(&w, Kind::Vehicle)[0];
    open_orders(&mut w);
    order_move(&mut w, id, cell(0, -(steps.len() as i32)));
    tick::step(&mut w);
    w.obj_mut(id).expect("vehicle").path.steps = steps.to_vec();
    (w, id)
}

/// (y, ρ, on a track, steps left) at the end of the first tick with a track.
fn first_moving_tick(steps: &[u8]) -> (i32, F64, bool, usize) {
    let (mut w, id) = start(steps);
    for _ in 0..40 {
        tick::step(&mut w);
        let o = w.obj(id).expect("vehicle");
        let d = o.drive.as_ref().expect("drive state");
        if d.track.is_some() {
            return (o.pos.y, d.rho, true, o.path.steps.len());
        }
    }
    panic!("the vehicle never began a track");
}

// spec: S-0207 claim 12 (quirks.drive_track_end_cap off: at each further track begun in the tick the fraction is updated once more towards that track's target and the leftover is spent on it) and claim 11 (rest with ρ = 0 only at a track target with no further step)
#[test]
fn cap_off_keeps_the_fraction_on_the_last_track() {
    let three = first_moving_tick(&[N, N, N]);
    let two = first_moving_tick(&[N, N]);
    assert!(three.2 && three.1 != F64::ZERO, "control");
    assert!(two.2, "the vehicle is on its second track");
    assert_eq!(two.3, 0, "with no step left");
    assert_eq!(two.0, three.0, "at the control's position");
    assert_ne!(two.1, F64::ZERO, "ρ is not 0 in the middle of a track");
}
