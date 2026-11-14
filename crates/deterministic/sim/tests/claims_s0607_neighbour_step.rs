//! Claim test for S-0607 claims 5, 7 and 9 with S-0502 claim 21, sim issues
//! S15 (#280): a neighbour step taken by a gripped unit keeps its
//! order-level destination, the cell of its fixed movement point, and the
//! locomotor's record of it. Synthetic data of `lift_common`; every name and
//! number is invented. Ledger: `docs/m8-claims/sim-issues.md`, *Batch S15*.

#[path = "lift_common/mod.rs"]
mod lift_common;

use glasnost_map::CellXY;
use glasnost_rules::sim::{LocomotionClass, Locomotor, MovementZone};
use glasnost_sim::jump::{self, Phase};
use glasnost_sim::lift;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{ObjectId, Target, World};
use lift_common::*;

/// LIFTER at (0, 0), a victim VICT at (8, 0), seized, and a jumpjet vehicle
/// HOV hovering over `hover_at`, holding that cell's hover slot.
fn scene(hover_at: CellXY) -> (World, ObjectId, ObjectId) {
    let mut w = world(|r| {
        vehicle(r, "HOV", |t| {
            t.locomotion_class = Some(LocomotionClass::Hover);
            t.movement_zone = Some(MovementZone::Fly);
            t.locomotor = Some(Locomotor::Jumpjet);
            t.jumpjet = Some(true);
            t.jumpjet_speed = Some(14);
            t.jumpjet_climb = Some(d("5.0"));
            t.jumpjet_height = Some(500);
            t.jumpjet_accel = Some(d("2.0"));
            t.jumpjet_turn_rate = Some(4);
        });
    });
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(8, 0));
    let h = spawn(&mut w, Kind::Vehicle, "HOV", 0, hover_at);
    jump::start_in_flight(&mut w, h, Phase::Hovering, 500);
    assert_eq!(w.jump_slots.get(&hover_at), Some(&h));
    beam(&mut w, l, v);
    assert!(lift::seized(&w, v));
    assert!(lift::grips(&w, l, v));
    (w, l, v)
}

fn phase(w: &World, id: ObjectId) -> Phase {
    w.obj(id).unwrap().jump.as_ref().unwrap().phase
}

// spec: S-0607 claim 5 (the seized unit's order-level destination becomes the cell of its movement point), claim 7 (the movement point is fixed at the seize) and claim 9 (a seized unit ignores every new destination, from orders, approach steps or any other rule); S-0502 claim 21 (a neighbour step gives its new destination through the move-order rule, with one draw, and enters cruising)
#[test]
fn issue_280_a_neighbour_step_leaves_a_gripped_unit_its_fixed_destination() {
    // Dry run: where the gripped unit finishes its ascent with a free slot.
    let (mut w, l, v) = scene(at(-12, -12));
    let fixed = w.obj(v).unwrap().dest;
    assert!(matches!(fixed, Some(Target::Cell(_))));
    assert!(run_until(&mut w, 400, |w| phase(w, v) == Phase::Hovering
        || !lift::grips(w, l, v)));
    assert!(lift::grips(&w, l, v));
    assert_eq!(phase(&w, v), Phase::Hovering);
    let over = w.obj(v).unwrap().pos.cell();
    assert_ne!(Some(Target::Cell(over)), fixed);
    assert_eq!(w.obj(v).unwrap().dest, fixed, "no step: the destination is kept");

    // The same scene with another jumpjet hovering over that cell.
    let (mut w, l, v) = scene(over);
    let fixed = w.obj(v).unwrap().dest;
    let point = w.obj(v).unwrap().jump.as_ref().unwrap().dest;
    assert_eq!(jump::lift_order(&w, v), fixed);
    let draws = w.rng.taken();
    assert!(run_until(&mut w, 400, |w| phase(w, v) == Phase::Cruising
        || !lift::grips(w, l, v)));
    assert!(lift::grips(&w, l, v));
    assert!(w.rng.taken() > draws, "the step's draw is taken");
    assert_eq!(w.obj(v).unwrap().pos.cell(), over, "the step's cell");
    assert_eq!(
        w.obj(v).unwrap().jump.as_ref().unwrap().dest,
        point,
        "the movement point stays fixed"
    );
    // A few more turns: the steering has had its chance to restore it.
    run(&mut w, 3);
    assert!(lift::grips(&w, l, v));
    assert_eq!(
        w.obj(v).unwrap().dest,
        fixed,
        "the gripped unit's order-level destination is the cell of its fixed movement point"
    );
    assert_eq!(jump::lift_order(&w, v), fixed);
}
