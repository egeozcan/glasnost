//! Claim tests for magnetic lift revision 33 (S-0607, M8 step A3 intake 57;
//! answers Q-2151): a seized unit cannot be selected (claim 5); a gripped
//! unit whose approach step clears its destination for being too far from
//! its target keeps its grip, and a later rule giving it no destination is
//! redundant (claim 9); the jumpjet stop on a drop and on a set-down
//! (claim 13), with the balloon-hover retention (S-0502 claim 28) in the
//! grip, at a drop and at a set-down; the attack-move memory kept by the
//! seize and by restoration and the charging scan before restoration
//! (claims 9 and 20). Synthetic data only (the lift fixture of
//! `lift_common`, the charger fixture of `linked_common`); no stock value.
//! Ledger: `docs/m8-claims/intake57-S-0407-S-0607-S-0307-S-0215-S-0408.md`.

#[path = "lift_common/mod.rs"]
mod lift_common;
#[path = "linked_common/mod.rs"]
mod linked_common;

use glasnost_rules::schema::Ruleset;
use glasnost_sim::jump;
use glasnost_sim::lift;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting;
use glasnost_sim::world::{AttackMoveMemory, HouseId, ObjectId, Pos, Target, World};
use lift_common::*;

/// LIFTER (house 0) at (0, 0) and a victim VICT (house 1) at (6, 0), seized.
fn seized(tweak: impl FnOnce(&mut Ruleset)) -> (World, ObjectId, ObjectId) {
    let mut w = world(tweak);
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    beam(&mut w, l, v);
    assert!(lift::seized(&w, v));
    assert!(lift::grips(&w, l, v));
    (w, l, v)
}

/// [`seized`], with the victim's type unable to approach (S-0215 claim 37
/// (d)'s *may approach* false) and of a computer-controlled owner, hunting
/// with its lifter as its target: part d keeps the target, and part e
/// clears the grip's destination, the lifter's cell, which lies farther
/// from the lifter than m × r′ of the unarmed unit (as in
/// `claims_s0215_r88.rs`).
fn hunting_its_lifter(tweak: impl FnOnce(&mut Ruleset)) -> (World, ObjectId, ObjectId) {
    let (mut w, l, v) = seized(|r| {
        r.sim.vehicles.get_mut("VICT").unwrap().can_approach_target = Some(false);
        tweak(r);
    });
    w.houses[1].human = false;
    hunt(&mut w, v, l);
    (w, l, v)
}

fn hunt(w: &mut World, v: ObjectId, l: ObjectId) {
    let t = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    let o = w.obj_mut(v).unwrap();
    o.target = Some(Target::Object(l));
    o.beh.force(Behaviour::Hunt, t, stale);
}

/// The jumpjet movement point.
fn point(w: &World, v: ObjectId) -> Option<Pos> {
    w.obj(v).unwrap().jump.as_ref().unwrap().dest
}

/// Orders the lifter one cell away (S-0607 claim 12 row 2: the grip ends).
fn order_lifter(w: &mut World, l: ObjectId) {
    glasnost_sim::econ::assign_destination(w, l, Some(Target::Cell(at(0, 1))));
}

/// Runs until the pulled victim is over `at(dx, 0)` at least 300 leptons up.
fn pull_over(w: &mut World, v: ObjectId, dx: i32) {
    let c = at(dx, 0);
    assert!(
        run_until(w, 400, |w| w.obj(v).is_some_and(|o| o.pos.cell() == c)
            && height(w, v) > 300),
        "the victim did not pass over the cell"
    );
}

fn selectable_by_owner(w: &World, v: ObjectId) -> bool {
    targeting::selectable_by(w, v, Some(HouseId(1)))
}

fn balloon(r: &mut Ruleset) {
    r.sim.vehicles.get_mut("VICT").unwrap().balloon_hover = Some(true);
}

/// Gives the victim, hunting, a target that is the first ground-layer
/// object listed in its destination's cell (S-0502 claim 28 row 2): a
/// vehicle of house 0 placed there.
fn row_2_target(w: &mut World, v: ObjectId) -> ObjectId {
    let Some(Target::Cell(c)) = w.obj(v).unwrap().dest else {
        panic!("the grip's destination is a cell");
    };
    let x = spawn(w, Kind::Vehicle, "VICT", 0, c);
    assert_eq!(
        w.cells.list(c, glasnost_sim::world::Layer::Ground).first(),
        Some(&x)
    );
    hunt(w, v, x);
    x
}

/// Runs until the pulled victim is in the air, out of its own cell.
fn pull_up(w: &mut World, v: ObjectId) {
    assert!(
        run_until(w, 400, |w| w
            .obj(v)
            .is_some_and(|o| o.pos.cell() != at(6, 0))
            && height(w, v) >= 1),
        "the victim was not pulled up"
    );
}

// ---------------------------------------------------------------------------
// Claim 5: no selection while seized

// spec: S-0607 claim 5 (revision 33: while it is seized it cannot be selected at all: a click on it, a band box around it and its control group's command pass it over, also while it falls after its grip has ended; it can be selected again only once it is no longer seized, claim 20; scenario L18 part a)
#[test]
fn claim_5_a_seized_unit_cannot_be_selected_until_restored() {
    let (mut w, l, v) = seized(|_| {});
    let h = HouseId(1);
    assert!(!selectable_by_owner(&w, v), "gripped: no selection");
    assert!(!targeting::box_eligible(&w, h, &[], v), "nor a band box");
    // Seam (S-0214 Terms, *clicked object*, unchanged): the click still
    // finds it, so a click on it can still give an order to a selection.
    assert!(targeting::clickable_by(&w, v, Some(h)));
    pull_over(&mut w, v, 4);
    order_lifter(&mut w, l);
    assert!(lift::dropped(&w, v), "dropped");
    assert!(!selectable_by_owner(&w, v), "falling: no selection");
    assert!(!targeting::box_eligible(&w, h, &[], v));
    assert!(run_until(&mut w, 600, |w| !lift::seized(w, v)));
    assert!(w.obj(v).is_some(), "it survived the fall");
    assert!(selectable_by_owner(&w, v), "restored: selectable again");
    assert!(targeting::box_eligible(&w, h, &[], v));
}

// spec: S-0607 claim 5 (revision 33: a unit seized with the driving, walking, ship or teleporting stand-in, which claim 24 never restores, is never selectable again)
#[test]
fn claim_5_a_unit_seized_with_a_stand_in_is_never_selectable_again() {
    let mut w = world(|r| {
        r.sim.warheads.get_mut("MAG").unwrap().locomotor = Some("drive".into());
        vehicle(r, "CHRONO", |t| {
            t.locomotor = Some(glasnost_rules::sim::Locomotor::Teleport)
        });
    });
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "CHRONO", 1, at(6, 0));
    beam(&mut w, l, v);
    assert!(lift::seized(&w, v));
    assert!(lift::stand_in(&w, v).is_some(), "a stand-in grip");
    order_lifter(&mut w, l);
    run(&mut w, 200);
    assert!(lift::seized(&w, v));
    assert!(!selectable_by_owner(&w, v));
}

// ---------------------------------------------------------------------------
// Claim 9: the approach step in the grip (Q-2151)

// spec: S-0607 claim 9 (revision 33, Q-2151: part e's clearing removes the destination directly; it is not the exception and not an effective clear: the grip holds and, with the jumpjet stand-in, the jumpjet movement keeps its movement point and commitment, so the unit is still pulled in and dropped on arrival, claim 8) and S-0215 claim 37 (e)
#[test]
fn claim_9_part_e_clearing_keeps_the_grip_and_the_pull() {
    let (mut w, l, v) = hunting_its_lifter(|_| {});
    let before = point(&w, v);
    assert!(jump::moving(&w, v));
    glasnost_sim::approach::step(&mut w, v);
    run(&mut w, 1);
    assert!(lift::grips(&w, l, v), "the grip holds");
    assert_eq!(w.obj(v).unwrap().dest, None, "and it has no destination");
    assert!(w.obj(v).unwrap().lift.grip_without_dest);
    assert_eq!(point(&w, v), before, "the movement point is kept");
    assert!(jump::moving(&w, v), "and the commitment");
    assert!(run_until(&mut w, 400, |w| !lift::grips(w, l, v)));
    assert!(lift::dropped(&w, v), "the arrival drop (claim 8)");
}

// spec: S-0607 claim 9 (revision 33: a gripped vehicle that has no destination gets none again from such a rule: a redundant assignment, so the grip holds; after part e's clearing, a later drop or failed search, here an empty assignment, ends no grip)
#[test]
fn claim_9_no_destination_again_is_redundant() {
    let (mut w, l, v) = hunting_its_lifter(|_| {});
    glasnost_sim::approach::step(&mut w, v);
    run(&mut w, 1);
    let before = point(&w, v);
    glasnost_sim::econ::assign_destination(&mut w, v, None);
    run(&mut w, 1);
    assert!(
        lift::grips(&w, l, v),
        "a redundant assignment: the grip holds"
    );
    assert_eq!(point(&w, v), before);
    // A destination given to it then is ignored, as every new one is.
    glasnost_sim::econ::assign_destination(&mut w, v, Some(Target::Cell(at(10, 10))));
    w.obj_mut(v).unwrap().dest = Some(Target::Cell(at(10, 10)));
    run(&mut w, 1);
    assert_eq!(w.obj(v).unwrap().dest, None, "ignored");
    assert!(lift::grips(&w, l, v));
    // Contrast: with its destination, the empty assignment is the exception.
    let (mut w, l, v) = seized(|_| {});
    glasnost_sim::econ::assign_destination(&mut w, v, None);
    run(&mut w, 1);
    assert!(!lift::grips(&w, l, v), "an effective clear ends the grip");
}

// ---------------------------------------------------------------------------
// Claim 13: the jumpjet stop

// spec: S-0607 claim 13 (revision 33: on a drop the stop always runs, as a direct instruction, whether or not the unit still had a destination: here after part e's clearing, the found point is the centre of its own free cell)
#[test]
fn claim_13_a_drop_runs_the_stop_without_a_destination() {
    let (mut w, l, v) = hunting_its_lifter(|_| {});
    glasnost_sim::approach::step(&mut w, v);
    run(&mut w, 1);
    let pull = point(&w, v).expect("the pull's point");
    pull_over(&mut w, v, 4);
    assert_eq!(w.obj(v).unwrap().dest, None);
    order_lifter(&mut w, l);
    assert!(lift::dropped(&w, v));
    let c = w.obj(v).unwrap().pos.cell();
    let d = point(&w, v).expect("the stop's found point");
    assert_eq!(d.cell(), c, "found in its own free cell");
    assert_ne!(d.cell(), pull.cell(), "not the pull's point");
}

// spec: S-0607 claim 13 (revision 33: on a set-down the stop runs only through the destination's clearing; a set-down unit that already has none, after part e's clearing, takes no stop: its jumpjet movement keeps its movement point and commitment, the one its pull was heading to, and it is a set-down unit still moving)
#[test]
fn claim_13_a_set_down_without_a_destination_takes_no_stop() {
    let (mut w, l, v) = hunting_its_lifter(|_| {});
    assert_eq!(height(&w, v), 0, "grounded before its first update");
    glasnost_sim::approach::step(&mut w, v);
    assert!(
        w.obj(v).unwrap().lift.grip_without_dest,
        "part e cleared it; part f's cell is ignored"
    );
    let pull = point(&w, v);
    order_lifter(&mut w, l);
    assert!(!lift::grips(&w, l, v));
    assert!(w.obj(v).unwrap().lift.set_down, "set down, not dropped");
    assert_eq!(point(&w, v), pull, "the pull's point is kept: no stop");
    assert!(jump::moving(&w, v), "still moving");
    // Contrast: with its destination the set-down's clearing is effective,
    // and the stop finds the centre of its own free cell.
    let (mut w, l, v) = seized(|_| {});
    let pull = point(&w, v);
    order_lifter(&mut w, l);
    let d = point(&w, v).expect("the found point");
    assert_ne!(Some(d), pull);
    assert_eq!(d.cell(), w.obj(v).unwrap().pos.cell());
}

// spec: S-0607 claim 13 (revision 33, a balloon-hover type: a set-down vehicle that still holds a destination and a target, a row of S-0502 claim 28 holding, here row 2 with its lifter the first object of the destination cell, has the set-down's assignment rejected after the grip has ended: it keeps its destination, movement point and commitment, takes no stop, and attack becomes current)
#[test]
fn claim_13_a_balloon_hover_set_down_keeps_its_destination() {
    let (mut w, l, v) = seized(balloon);
    let dest = w.obj(v).unwrap().dest;
    row_2_target(&mut w, v);
    let pull = point(&w, v);
    order_lifter(&mut w, l);
    assert!(!lift::grips(&w, l, v));
    assert!(w.obj(v).unwrap().lift.set_down);
    assert_eq!(w.obj(v).unwrap().dest, dest, "the assignment is rejected");
    assert_eq!(w.obj(v).unwrap().beh.seen(), Some(Behaviour::Attack));
    assert_eq!(point(&w, v), pull, "no stop");
    assert!(jump::moving(&w, v));
    run(&mut w, 1);
    assert_eq!(w.obj(v).unwrap().dest, dest, "kept after the set-down");
    // Without the target no row can hold: the destination becomes none.
    let (mut w, l, v) = seized(balloon);
    order_lifter(&mut w, l);
    assert_eq!(w.obj(v).unwrap().dest, None);
}

// spec: S-0607 claim 13 (revision 33, a dropped balloon-hover vehicle: the stop on dropping first gives it no destination while it holds its target, which S-0502 claim 28 rejects; it then clears the target and gives none again, which cannot be rejected: it ends with no destination and the direct stop, and attack stays current with no target)
#[test]
fn claim_13_a_dropped_balloon_hover_vehicle_ends_with_attack_current() {
    let (mut w, l, v) = seized(balloon);
    pull_up(&mut w, v);
    row_2_target(&mut w, v);
    order_lifter(&mut w, l);
    assert!(lift::dropped(&w, v));
    let o = w.obj(v).unwrap();
    assert_eq!((o.dest, o.target), (None, None));
    assert_eq!(
        o.beh.seen(),
        Some(Behaviour::Attack),
        "the rejection's attack"
    );
    let d = point(&w, v).expect("the direct stop's found point");
    assert_eq!(d.cell(), o.pos.cell());
    // An ordinary victim keeps the behaviour it had.
    let (mut w, l, v) = seized(|_| {});
    pull_up(&mut w, v);
    row_2_target(&mut w, v);
    order_lifter(&mut w, l);
    assert!(lift::dropped(&w, v));
    assert_eq!(w.obj(v).unwrap().beh.seen(), Some(Behaviour::Hunt));
}

/// An attack order on `x` for the victim, as an order given in the seize's
/// tick reaches it (S-0607 claim 9): it sets the target first, then asks for
/// no destination (S-0213 claim 21).
fn attack_order(w: &mut World, v: ObjectId, x: ObjectId) {
    w.houses[1]
        .pending
        .push(glasnost_sim::orders::Order::Behaviour {
            actor: v,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(x)),
            dest: None,
        });
}

// spec: S-0607 claim 9 (revision 33, a balloon-hover type: S-0502 claim 28's retention is tested for a seized vehicle as for any other, before a seized unit's refusals: an order giving a gripped vehicle that holds a destination and a target no destination, a row of that claim holding, is rejected: not the exception, the grip holds, the destination is kept and attack becomes current)
#[test]
fn claim_9_a_balloon_hover_grip_rejects_no_destination() {
    let (mut w, l, v) = seized(balloon);
    let dest = w.obj(v).unwrap().dest;
    let x = row_2_target(&mut w, v);
    attack_order(&mut w, v, x);
    run(&mut w, 2);
    assert!(lift::grips(&w, l, v), "the grip holds");
    let o = w.obj(v).unwrap();
    assert_eq!(o.dest, dest, "the destination is kept");
    assert_eq!(o.target, Some(Target::Object(x)));
    assert_eq!(o.beh.seen(), Some(Behaviour::Attack));
    // Without the balloon-hover field the order's empty destination is the
    // exception: the grip ends.
    let (mut w, l, v) = seized(|_| {});
    let x = row_2_target(&mut w, v);
    attack_order(&mut w, v, x);
    run(&mut w, 2);
    assert!(!lift::grips(&w, l, v), "an effective clear ends the grip");
}

// ---------------------------------------------------------------------------
// Claims 9 and 20: the attack-move memory and the charging scan

// spec: S-0607 claim 9 (revision 33: the seize does not remove an attack-move memory, S-0213 claim 6: it is not a behaviour order) and claim 20 (restoration does not itself remove it; scenario L18 part b)
#[test]
fn claims_9_and_20_the_seize_and_restoration_keep_the_attack_move_memory() {
    let mut w = world(|_| {});
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    let memory = AttackMoveMemory::Target(Target::Cell(at(30, 30)));
    w.obj_mut(v).unwrap().attack_move = Some(memory);
    beam(&mut w, l, v);
    assert!(lift::seized(&w, v));
    assert_eq!(
        w.obj(v).unwrap().attack_move,
        Some(memory),
        "kept by the seize"
    );
    order_lifter(&mut w, l);
    assert!(run_until(&mut w, 600, |w| !lift::seized(w, v)));
    assert_eq!(
        w.obj(v).unwrap().attack_move,
        Some(memory),
        "kept through restoration"
    );
}

// spec: S-0607 claims 9 and 20 (revision 33: S-0518 claim 14's charging scan makes no firing check, and neither it nor guard processing makes an exception for a seized unit: a found building of its own house becomes its target, its charging mark is set and attack is queued without an immediate start)
#[test]
fn claims_9_and_20_the_charging_scan_runs_for_a_seized_unit() {
    let mut w = linked_common::world(|_| {});
    let coil = linked_common::spawn(&mut w, Kind::Building, "COIL", 0, linked_common::at(1, 0));
    let car = linked_common::spawn(&mut w, Kind::Vehicle, "ZAPCAR", 0, linked_common::at(0, 0));
    // The seized state alone, set on the synthetic charger: the scan's
    // inputs do not include it.
    w.obj_mut(car).unwrap().lift.seized = true;
    assert!(lift::seized(&w, car));
    assert!(glasnost_sim::linked::start_charging(&mut w, car));
    let o = w.obj(car).unwrap();
    assert_eq!(o.target, Some(Target::Object(coil)));
    assert!(o.combat.charging);
    assert_eq!(o.beh.queued, Some(Behaviour::Attack));
}
