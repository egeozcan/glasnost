//! Claim tests of S-0517 revision 7 (deployable units) on synthetic rules:
//! claim 5, a landing-first vehicle hovering when the deploy order clears its
//! destination (S-0502 claims 13 and 24). Invented types and numbers, no game
//! data.

mod deploy_common;

use deploy_common::*;
use glasnost_sim::econ::inplace::Form;
use glasnost_sim::jump::{self, Phase};
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{ObjectId, Pos, Target, World};

fn j(w: &World, id: ObjectId) -> &glasnost_sim::jump::JumpState {
    w.obj(id).unwrap().jump.as_ref().unwrap()
}

fn js(w: &mut World, id: ObjectId) -> &mut glasnost_sim::jump::JumpState {
    w.obj_mut(id).unwrap().jump.as_mut().unwrap()
}

/// Runs ticks until `f` holds, at most `max`.
fn until(w: &mut World, max: i32, f: impl Fn(&World) -> bool) -> bool {
    for _ in 0..max {
        if f(w) {
            return true;
        }
        step(w);
    }
    f(w)
}

/// A landing-first vehicle hovering over its cell `(dx, dy)` with a movement
/// commitment and the point of a completed flight.
fn hovering_committed(w: &mut World, dx: i32, dy: i32) -> ObjectId {
    let v = put(w, "DLAND0", Kind::Vehicle, 0, dx, dy);
    jump::start_in_flight(w, v, Phase::Hovering, 300);
    commit_here(w, v);
    v
}

// spec: S-0517 claim 5 and S-0502 claim 24 (revision 7): an effective deploy-order clear of a committed hovering vehicle reroutes and keeps the replacement point and the commitment; an equal horizontal point lets the landing request produce descent, and touchdown is at the retained point
#[test]
fn claim_5_effective_clear_keeps_the_replacement_point() {
    let mut w = game();
    let v = hovering_committed(&mut w, 0, 0);
    // The order-level destination it holds (an effective clear when the deploy order empties it).
    w.obj_mut(v).unwrap().dest = Some(Target::Cell(at(0, 0)));
    js(&mut w, v).order = Some(Target::Cell(at(0, 0)));
    deploy(&mut w, v);
    assert!(until(&mut w, 40, |w| jump::landing_requested(w, v)));
    assert!(j(&w, v).moving, "the commitment survives the clear");
    assert_eq!(j(&w, v).dest.map(|d| d.cell()), Some(at(0, 0)));
    assert!(w.obj(v).unwrap().dest.is_none());
    assert!(until(&mut w, 200, |w| phase(w, v) == Phase::Grounded));
    assert_eq!(w.obj(v).unwrap().pos.cell(), at(0, 0));
    assert!(until(&mut w, 40, |w| inplace_form(w, v) == Form::Deployed));
}

fn inplace_form(w: &World, id: ObjectId) -> Form {
    glasnost_sim::econ::inplace::form(w, id)
}

// spec: S-0517 claim 5 and S-0502 claim 24 (revision 7): a different replacement point first causes cruise to that point, then descent; touchdown uses the point then retained (the cell below is occupied, so the reroute's search picks another cell)
#[test]
fn claim_5_a_different_replacement_cruises_before_descent() {
    let mut w = game();
    let v = hovering_committed(&mut w, 0, 0);
    let _blocker = put(&mut w, "TANK", Kind::Vehicle, 0, 0, 0);
    w.obj_mut(v).unwrap().dest = Some(Target::Cell(at(0, 0)));
    js(&mut w, v).order = Some(Target::Cell(at(0, 0)));
    deploy(&mut w, v);
    let mut cruised = false;
    for _ in 0..300 {
        step(&mut w);
        cruised |= phase(&w, v) == Phase::Cruising;
        if phase(&w, v) == Phase::Grounded {
            break;
        }
    }
    assert!(cruised, "it first cruises to the replacement");
    assert_eq!(phase(&w, v), Phase::Grounded);
    assert_ne!(w.obj(v).unwrap().pos.cell(), at(0, 0));
}

// spec: S-0517 claim 5 and S-0502 claim 24 (revision 7): when the order-level destination was already empty the repeated assignment of an ordinary vehicle is suppressed: no reroute, and the preceding movement point and commitment remain; descent and touchdown use that retained point
#[test]
fn claim_5_a_suppressed_assignment_keeps_the_preceding_point() {
    let mut w = game();
    let v = hovering_committed(&mut w, 0, 0);
    // The preceding point is two cells east: the vehicle flies there, whatever the deploy order does.
    let far = Pos::centre(at(2, 0), 0);
    js(&mut w, v).dest = Some(far);
    js(&mut w, v).order = None;
    deploy(&mut w, v);
    assert!(until(&mut w, 40, |w| jump::landing_requested(w, v)
        || phase(w, v) == Phase::Cruising));
    assert_eq!(
        j(&w, v).dest,
        Some(far),
        "the point is not erased or replaced"
    );
    assert!(j(&w, v).moving);
    assert!(until(&mut w, 400, |w| phase(w, v) == Phase::Grounded));
    assert_eq!(w.obj(v).unwrap().pos.cell(), at(2, 0));
}

// spec: S-0517 claim 5 and S-0502 claims 3 and 24 (revision 7): without a movement commitment the clear cannot create one or wake idle hover, and the landing request alone does not bypass the update eligibility
#[test]
fn claim_5_no_commitment_the_request_does_not_wake_an_idle_hover() {
    let mut w = game();
    let v = put(&mut w, "DLAND0", Kind::Vehicle, 0, 0, 0);
    jump::start_in_flight(&mut w, v, Phase::Hovering, 300);
    assert!(!j(&w, v).moving);
    let h = height(&w, v);
    deploy(&mut w, v);
    assert!(until(&mut w, 40, |w| jump::landing_requested(w, v)));
    for _ in 0..60 {
        step(&mut w);
    }
    assert_eq!(phase(&w, v), Phase::Hovering);
    assert!(!j(&w, v).moving, "no commitment was created");
    assert_eq!(height(&w, v), h, "an idle hover takes no update");
    assert!(jump::landing_requested(&w, v));
}
