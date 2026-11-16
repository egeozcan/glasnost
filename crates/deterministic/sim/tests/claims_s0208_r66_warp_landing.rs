//! A teleporting infantry unit's landing choice while a superweapon warp is
//! pending, and its standing positions at the warp's departure, landing
//! check and appearance (#363; S-0208 revision 66, claim 19, *Landing
//! choice while a warp is pending* and *Appearance of a teleporting
//! infantry warp member*, Q-3142; `quirks.warp_pending_infantry_landing`).
//! On the synthetic superweapon rules of `sw_common` plus an invented
//! infantry type with the teleporting locomotor; no game data.

mod sw_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{LocomotionClass, Locomotor, UnitType};
use glasnost_sim::cells::Marker;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::Behaviour;
use glasnost_sim::superweapons::warp;
use glasnost_sim::world::{Layer, ObjectId, Target, World};
use sw_common::*;

fn rules() -> Ruleset {
    let mut r = ruleset();
    r.sim.general.superweapons.chrono_delay = Some(7);
    // An infantry type with the teleporting locomotor, not killed by a warp.
    r.sim.infantry.insert(
        "BLINK".into(),
        UnitType {
            strength: Some(100),
            speed: Some(4),
            locomotion_class: Some(LocomotionClass::Foot),
            locomotor: Some(Locomotor::Teleport),
            teleporter: Some(true),
            ..Default::default()
        },
    );
    r.registries
        .entry("infantry".into())
        .or_default()
        .push("BLINK".into());
    r
}

fn rules_off() -> Ruleset {
    let mut r = rules();
    r.globals.quirks.warp_pending_infantry_landing = false;
    r
}

/// Every standing-position mark held in the name of infantry unit `id`.
fn marks_of(w: &World, id: ObjectId) -> Vec<(CellXY, Layer)> {
    let mut v = Vec::new();
    let cells: Vec<CellXY> = w.cells.grid.iter().map(|(c, _)| c).collect();
    for c in cells {
        for l in [Layer::Ground, Layer::Deck] {
            if let Some(rec) = w.cells.occ(c, l)
                && rec
                    .marks
                    .iter()
                    .flatten()
                    .any(|m| matches!(m.by, Marker::Infantry(x, _) if x == id))
            {
                v.push((c, l));
            }
        }
    }
    v
}

fn marked(w: &World, id: ObjectId, c: CellXY) -> bool {
    marks_of(w, id).iter().any(|&(m, _)| m == c)
}

fn move_order(w: &mut World, id: ObjectId, c: CellXY) {
    let owner = w.obj(id).unwrap().owner.unwrap();
    assert!(orders::issue(
        w,
        owner,
        Order::Behaviour {
            actor: id,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(c)),
        }
    ));
}

/// A game with ALPHA's and BETA's teleport sources ready and BLINK at
/// `at(0, 0)` for ALPHA.
fn scene(r: &Ruleset) -> (World, ObjectId) {
    let mut w = game(r);
    spawn(&mut w, "CHRONOB", ALPHA, -9, -9);
    spawn(&mut w, "CHRONOB", BETA, -9, 9);
    ready_tick(&mut w, ALPHA, SOURCE, 300);
    ready_tick(&mut w, BETA, SOURCE, 300);
    let b = spawn(&mut w, "BLINK", ALPHA, 0, 0);
    (w, b)
}

/// Runs until the member has departed (off the map, before its appearance).
fn until_departed(w: &mut World, b: ObjectId) {
    until(w, 200, |w| warp::departed(w, b));
}

/// Runs until the member's warp is over.
fn until_over(w: &mut World, b: ObjectId) {
    until(w, 300, |w| w.obj(b).is_some_and(|o| o.sw.warp.is_none()));
}

// spec: S-0208 claim 19, Landing choice while a warp is pending (Q-3142: a landing point chosen before the warp is not jumped to; its reservation is released when the unit appears at the warp's landing point) and Appearance of a teleporting infantry warp member (the appearance frees the chosen point's standing position and takes the one at the warp's landing point)
#[test]
fn issue_363_a_landing_point_chosen_before_the_join_stays_reserved_until_the_appearance() {
    let (mut w, b) = scene(&rules());
    let l = at(0, -6);
    fire_now(&mut w, BETA, SOURCE, at(0, 0));
    // One order step: house 0's move order first, then house 1's warp.
    move_order(&mut w, b, l);
    fire(&mut w, BETA, WARP, at(5, 5));
    step(&mut w);
    assert!(warp::is_member(&w, b), "set-up: the unit joined the warp");
    assert!(marked(&w, b, l), "kept at the join: {:?}", marks_of(&w, b));
    until_departed(&mut w, b);
    assert!(
        marked(&w, b, l),
        "kept through the departure: {:?}",
        marks_of(&w, b)
    );
    until_over(&mut w, b);
    let o = w.obj(b).unwrap();
    assert_eq!(
        o.pos.cell(),
        at(5, 5),
        "set-up: it appeared at the warp's landing point"
    );
    assert!(
        !marked(&w, b, l),
        "freed at the appearance: {:?}",
        marks_of(&w, b)
    );
    assert!(marked(&w, b, at(5, 5)), "the warp's landing point is taken");
}

// spec: S-0208 claim 19, Landing choice while a warp is pending (Q-3142: a destination that reaches the choice while the unit has warp-pending immobilization, before its leaving stage, is chosen, entry-tested and reserved; the unit does not jump to it)
#[test]
fn issue_363_a_move_order_reaching_a_pending_member_reserves_its_choice() {
    let (mut w, b) = scene(&rules());
    let l = at(0, -6);
    fire_now(&mut w, ALPHA, SOURCE, at(0, 0));
    // One order step: the warp, then the move order to the pending member.
    fire(&mut w, ALPHA, WARP, at(5, 5));
    move_order(&mut w, b, l);
    step(&mut w);
    assert!(warp::is_member(&w, b), "set-up: the unit joined the warp");
    assert!(
        marked(&w, b, l),
        "reserved while pending: {:?}",
        marks_of(&w, b)
    );
    assert_eq!(w.obj(b).unwrap().pos.cell(), at(0, 0), "no jump to it");
    until_departed(&mut w, b);
    assert!(marked(&w, b, l), "kept through the departure");
    until_over(&mut w, b);
    assert!(
        !marked(&w, b, l),
        "freed at the appearance: {:?}",
        marks_of(&w, b)
    );
}

// spec: S-0208 Edge cases (quirks.warp_pending_infantry_landing off: a destination reaching the landing choice while warp-pending is treated as S-0207 claim 20 b treats one for other objects: no standing position is chosen or reserved and no draw is taken)
#[test]
fn issue_363_quirk_off_a_pending_member_reserves_nothing() {
    let (mut w, b) = scene(&rules_off());
    let l = at(0, -6);
    fire_now(&mut w, ALPHA, SOURCE, at(0, 0));
    fire(&mut w, ALPHA, WARP, at(5, 5));
    move_order(&mut w, b, l);
    step(&mut w);
    assert!(warp::is_member(&w, b), "set-up: the unit joined the warp");
    assert!(!marked(&w, b, l), "nothing reserved: {:?}", marks_of(&w, b));
}

// spec: S-0208 Edge cases (quirks.warp_pending_infantry_landing off: an earlier landing point keeps its reservation, which the appearance frees) and claim 19, Appearance of a teleporting infantry warp member
#[test]
fn issue_363_quirk_off_an_earlier_landing_point_keeps_its_reservation_until_the_appearance() {
    let (mut w, b) = scene(&rules_off());
    let l = at(0, -6);
    fire_now(&mut w, BETA, SOURCE, at(0, 0));
    move_order(&mut w, b, l);
    fire(&mut w, BETA, WARP, at(5, 5));
    step(&mut w);
    assert!(marked(&w, b, l), "kept at the join");
    until_over(&mut w, b);
    assert!(
        !marked(&w, b, l),
        "freed at the appearance: {:?}",
        marks_of(&w, b)
    );
}

// spec: S-0208 claim 19, Landing choice while a warp is pending (from the leaving stage until the departure, being erased drops the destination before any choice, S-0207 claim 20 a)
#[test]
fn issue_363_a_move_order_in_the_leaving_stage_is_dropped_before_any_choice() {
    let (mut w, b) = scene(&rules());
    let l = at(0, -6);
    fire_now(&mut w, ALPHA, SOURCE, at(0, 0));
    fire(&mut w, ALPHA, WARP, at(5, 5));
    step(&mut w);
    run(&mut w, 5);
    assert!(
        w.obj(b)
            .and_then(|o| o.sw.warp.as_ref())
            .is_some_and(|m| m.leaving()),
        "set-up: in the leaving stage"
    );
    let before = w.rng.clone();
    move_order(&mut w, b, l);
    step(&mut w);
    assert!(!marked(&w, b, l), "no choice: {:?}", marks_of(&w, b));
    assert_eq!(w.obj(b).unwrap().dest, None, "the destination is dropped");
    assert!(w.rng == before, "no draw");
}

// spec: S-0208 claim 19, Appearance of a teleporting infantry warp member (a unit with no remembered landing point: the entry at the place it left is freed at the departure and taken again when its landing check passes, so with a clean landing it stays taken until the appearance, which frees it)
#[test]
fn issue_363_a_member_with_no_remembered_point_keeps_its_entry_until_the_appearance() {
    let (mut w, b) = scene(&rules());
    let home = at(0, 0);
    assert!(marked(&w, b, home), "set-up: its own entry");
    fire_now(&mut w, ALPHA, SOURCE, at(0, 0));
    fire_now(&mut w, ALPHA, WARP, at(5, 5));
    until_departed(&mut w, b);
    assert!(
        marked(&w, b, home),
        "taken again at the passing landing check: {:?}",
        marks_of(&w, b)
    );
    until_over(&mut w, b);
    assert!(
        !marked(&w, b, home),
        "freed at the appearance: {:?}",
        marks_of(&w, b)
    );
    assert!(marked(&w, b, at(5, 5)));
    let t = w.obj(b).unwrap().teleport.clone().unwrap_or_default();
    assert_eq!(
        t.held,
        Some((at(5, 5), Layer::Ground)),
        "the warp's landing point becomes its remembered landing point"
    );
}

// spec: S-0208 claim 19, Appearance of a teleporting infantry warp member (a unit that completed its jump before the warp: its entry stays taken through the departure; the appearance frees what a later destination choice would free, by its actual position under quirks.teleport_infantry_ground_reservation)
#[test]
fn issue_363_a_member_that_jumped_before_the_warp_keeps_its_entry_until_the_appearance() {
    let (mut w, b) = scene(&rules());
    let l = at(1, 0);
    move_order(&mut w, b, l);
    until(&mut w, 100, |w| w.obj(b).unwrap().pos.cell() == l);
    // Phasing in keeps a unit out of a warp (S-0513 claim 34): wait for its end.
    until(&mut w, 200, |w| {
        w.obj(b)
            .unwrap()
            .teleport
            .as_ref()
            .is_some_and(|t| t.phasing.is_none())
    });
    run(&mut w, 2);
    assert!(
        w.obj(b)
            .unwrap()
            .teleport
            .as_ref()
            .and_then(|t| t.held)
            .is_some(),
        "set-up: a remembered landing point after the jump"
    );
    assert!(
        marked(&w, b, l),
        "set-up: its entry at the point it jumped to"
    );
    fire_now(&mut w, ALPHA, SOURCE, at(0, 0));
    fire_now(&mut w, ALPHA, WARP, at(5, 5));
    until_departed(&mut w, b);
    assert!(
        marked(&w, b, l),
        "kept through the departure: {:?}",
        marks_of(&w, b)
    );
    until_over(&mut w, b);
    assert!(
        !marked(&w, b, l),
        "freed at the appearance: {:?}",
        marks_of(&w, b)
    );
    assert!(marked(&w, b, at(6, 5)) || marked(&w, b, at(5, 5)));
}

// spec: S-0208 claim 19 and Open questions (our engine's provisional rule: a warp member removed before it appears releases the standing position it reserved while its warp was pending)
#[test]
fn issue_363_a_member_removed_before_it_appears_releases_its_reservation() {
    let (mut w, b) = scene(&rules());
    let l = at(0, -6);
    fire_now(&mut w, ALPHA, SOURCE, at(0, 0));
    fire(&mut w, ALPHA, WARP, at(5, 5));
    move_order(&mut w, b, l);
    step(&mut w);
    until_departed(&mut w, b);
    remove(&mut w, b);
    assert!(w.obj(b).is_none(), "set-up: removed");
    assert!(
        marks_of(&w, b).is_empty(),
        "left behind: {:?}",
        marks_of(&w, b)
    );
}
