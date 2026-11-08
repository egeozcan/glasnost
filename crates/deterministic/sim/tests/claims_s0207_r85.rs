//! Claim tests of S-0207 revision 85 (Q-2781 answered), claim 16: a seized
//! vehicle that receives a scatter request. It has no refusal of its own;
//! cases b and g read the locomotor it has at the request; a request with a
//! threat point takes its draws, queues *move* without an immediate start
//! and then refuses the candidate as its destination (S-0607 claim 9); a
//! gripped unit with the jumpjet lift judges its candidates around its
//! heading point's cell. Synthetic data only (the lift fixture of
//! `lift_common`); no stock value. Ledger: `docs/m8-claims/intake112.md`.

#[path = "lift_common/mod.rs"]
mod lift_common;

use glasnost_rules::sim::Locomotor;
use glasnost_sim::lift;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::scatter::{self, ScatterRequest};
use glasnost_sim::world::{ObjectId, Pos, World};
use lift_common::*;

/// LIFTER (house 0) at (0, 0) and a victim VICT (house 1) at (6, 0),
/// seized and gripped with the jumpjet lift.
fn gripped() -> (World, ObjectId, ObjectId) {
    let mut w = world(|_| {});
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    beam(&mut w, l, v);
    assert!(lift::seized(&w, v));
    assert!(lift::grips(&w, l, v));
    assert_eq!(lift::stand_in(&w, v), None, "the jumpjet lift");
    (w, l, v)
}

fn threat_at(w: &World, id: ObjectId) -> ScatterRequest {
    let p = w.obj(id).unwrap().pos;
    ScatterRequest {
        threat: Some(Pos::new(p.x - 512, p.y, p.z)),
        forced: true,
        anywhere: false,
    }
}

/// The cell of the gripped unit's heading point, the movement point that
/// the seize fixed (S-0607 claim 7).
fn heading_cell(w: &World, v: ObjectId) -> glasnost_map::CellXY {
    w.obj(v)
        .unwrap()
        .jump
        .as_ref()
        .and_then(|j| j.dest)
        .expect("a fixed movement point")
        .cell()
}

// spec: S-0207 claim 16 (r85, Q-2781: a gripped unit with the jumpjet stand-in judges its candidates around its vehicle scatter centre cell, the cell of its heading point, the movement point the seize fixed, S-0607 claim 7, not the cell it is flying over) and Terms (vehicle scatter centre cell)
#[test]
fn claim_16_a_gripped_unit_scatters_around_its_heading_point() {
    let (w, _, v) = gripped();
    let c = heading_cell(&w, v);
    assert_eq!(scatter::vehicle_centre_cell(&w, v), Some(c));
}

// spec: S-0207 claim 16 (r85, Q-2781: while a seized vehicle has a destination, case e refuses every request that is not move-anywhere)
#[test]
fn claim_16_case_e_refuses_a_gripped_unit_with_its_destination() {
    let (mut w, _, v) = gripped();
    assert!(w.obj(v).unwrap().dest.is_some(), "the grip's destination");
    let rng = w.rng.clone();
    let req = threat_at(&w, v);
    assert!(!scatter::request(&mut w, v, req));
    assert_eq!(w.rng, rng, "no draw");
}

// spec: S-0207 claim 16 (r85, Q-2781: a seized vehicle with no destination that accepts a request with a threat point takes its draws, has move queued without an immediate start, and the candidate is not taken as its destination, S-0607 claim 9: its lack of one is unchanged)
#[test]
fn claim_16_a_gripped_unit_without_a_destination_queues_move_only() {
    let (mut w, _, v) = gripped();
    w.obj_mut(v).unwrap().dest = None;
    let before = w.obj(v).unwrap().beh.current;
    let rng = w.rng.clone();
    let req = threat_at(&w, v);
    assert!(scatter::request(&mut w, v, req), "accepted");
    let o = w.obj(v).unwrap();
    assert_ne!(w.rng, rng, "the direction draw is taken");
    assert_eq!(o.beh.queued, Some(Behaviour::Move), "move queued");
    assert_eq!(o.beh.current, before, "no immediate start");
    assert_eq!(o.dest, None, "the candidate is refused as its destination");
    assert!(
        lift::grips(&w, lift::lifter_of(&w, v).unwrap(), v),
        "still gripped"
    );
}

// spec: S-0207 claim 16 (r85, Q-2781: a seized vehicle that accepts a request without a threat point takes no draw, queues no behaviour, and the cell the search gives is not taken as its destination)
#[test]
fn claim_16_a_gripped_unit_without_a_destination_ignores_a_plain_request() {
    let (mut w, _, v) = gripped();
    w.obj_mut(v).unwrap().dest = None;
    let (cur, q) = {
        let o = w.obj(v).unwrap();
        (o.beh.current, o.beh.queued)
    };
    let rng = w.rng.clone();
    let req = ScatterRequest {
        threat: None,
        forced: true,
        anywhere: false,
    };
    assert!(scatter::request(&mut w, v, req), "accepted");
    let o = w.obj(v).unwrap();
    assert_eq!(w.rng, rng, "no draw");
    assert_eq!((o.beh.current, o.beh.queued), (cur, q));
    assert_eq!(o.dest, None);
}

// spec: S-0207 claim 16 (r85, Q-2781: case b reads the locomotor the seized vehicle has at that moment: a teleporting type seized with the jumpjet lift has the jumpjet locomotor, so case b does not refuse it; its own teleporting locomotor refuses it once it is no longer seized)
#[test]
fn claim_16_case_b_reads_the_current_locomotor_of_a_seized_teleporter() {
    let mut w = world(|r| vehicle(r, "TELE", |t| t.locomotor = Some(Locomotor::Teleport)));
    let v = spawn(&mut w, Kind::Vehicle, "TELE", 1, at(6, 0));
    w.obj_mut(v).unwrap().dest = None;
    let req = threat_at(&w, v);
    assert!(
        !scatter::request(&mut w, v, req),
        "control: its own locomotor refuses"
    );
    {
        let o = w.obj_mut(v).unwrap();
        o.lift.seized = true;
        o.lift.stand_in = None;
    }
    assert_eq!(
        lift::current_locomotor(&w, v),
        Some(glasnost_sim::rules::Locomotor::Jump)
    );
    let req = threat_at(&w, v);
    assert!(
        scatter::request(&mut w, v, req),
        "the jumpjet lift is not refused"
    );
    assert_eq!(w.obj(v).unwrap().dest, None);
}

// spec: S-0207 claim 16 (r85, Q-2781: case g reads the locomotor the seized vehicle has at that moment: a hovering type seized with the jumpjet lift is judged by the jumpjet's switch, not by the hover's power, S-0503 Terms)
#[test]
fn claim_16_case_g_reads_the_current_locomotor_of_a_seized_hover() {
    let mut w = world(|r| vehicle(r, "HOVR", |t| t.locomotor = Some(Locomotor::Hover)));
    let v = spawn(&mut w, Kind::Vehicle, "HOVR", 1, at(6, 0));
    w.obj_mut(v).unwrap().dest = None;
    // Set-up: switched off and down at a hover height of 0, so not powered.
    let hh = glasnost_sim::hover::hover_height(&w, v);
    {
        let o = w.obj_mut(v).unwrap();
        o.hover.as_mut().expect("a hover state").off = true;
        o.pos.z -= hh;
    }
    assert!(!glasnost_sim::hover::powered(&w, v), "set-up");
    let req = threat_at(&w, v);
    assert!(
        !scatter::request(&mut w, v, req),
        "control: not powered refuses"
    );
    {
        let o = w.obj_mut(v).unwrap();
        o.lift.seized = true;
        o.lift.stand_in = None;
    }
    let req = threat_at(&w, v);
    assert!(
        scatter::request(&mut w, v, req),
        "the jumpjet lift is switched on"
    );
}
