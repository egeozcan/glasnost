//! Claim test for S-0506 claim 14 with S-0207's heading point, sim issues
//! S15 (#279): a rocket missile's aim point at a driving vehicle is the
//! track target at the track's stored height, the vehicle's height when the
//! track began, not the terrain at the target. Synthetic scene of
//! `tests/common`; every name and number is invented. Ledger:
//! `docs/m8-claims/sim-issues.md`, *Batch S15*.

mod common;

use common::*;
use glasnost_sim::rocket;
use glasnost_sim::rules::Kind;
use glasnost_sim::tick;
use glasnost_sim::world::Target;

// spec: S-0506 claim 14 (the aim point is the destination's heading point), S-0207 Terms (heading point: for the driving locomotor the track target, whose stored world height is the vehicle's world height when an ordinary track begins, not the terrain or deck height at the target) and claim 24
#[test]
fn issue_279_rocket_aim_takes_the_heading_points_stored_height() {
    // A flat start cell, a ramp in the cell ahead and one more cell beyond.
    let mut s = Scene::new();
    s.vehicle("TESTCAR", unit_type(WHEEL, 3, 5));
    s.place_unit(0, "TESTCAR", cell(0, 0), 0);
    s.set_cell(cell(0, -1), 1, 0);
    let mut w = s.build();
    let id = objects_of(&w, Kind::Vehicle)[0];
    open_orders(&mut w);
    order_move(&mut w, id, cell(0, -2));
    tick::step(&mut w);
    w.obj_mut(id).expect("vehicle").path.steps = vec![0, 0];
    let start_z = w.obj(id).unwrap().pos.z;
    let mut track = None;
    for _ in 0..100 {
        tick::step(&mut w);
        track = w
            .obj(id)
            .and_then(|o| o.drive.as_ref())
            .and_then(|d| d.track.clone());
        if track.is_some() {
            break;
        }
    }
    let track = track.expect("a track");
    let target_z = w.surface_at(track.target.0, track.target.1);
    assert!(target_z > start_z, "set-up: the target lies higher");
    assert_eq!(track.height, start_z, "set-up: the stored heading height");
    assert!(w.rules().quirks.rocket_aim_at_heading_point);
    let aim = rocket::heading_point(&w, Target::Object(id)).expect("an aim point");
    assert_eq!((aim.x, aim.y), track.target);
    assert_eq!(
        aim.z, track.height,
        "the aim point's height is the heading point's stored height (target surface {target_z})"
    );
}
