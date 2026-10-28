//! Claim tests of S-0402 revision 28 (answers Q-1130 question 3 with
//! S-0207 revision 36): the seeking drop of a chrono miner with active
//! teleporting locomotion (claim 17 and the claim 3 table rows). The
//! fixture is the harvest ruleset with CHRONOT, a harvester whose own
//! locomotor teleports and whose type lacks the teleport-capable flag (with
//! the flag it would drive from placement, S-0207 claim 28). Every name and
//! number is invented; no game data.

mod harvest_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::Locomotor;
use glasnost_sim::econ::harvest::{self, HarvestState, Phase};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{Layer, ObjectId, Target, World};
use harvest_common::*;

fn ruleset() -> Ruleset {
    let mut r = harvest_common::ruleset();
    let mut chrono = harvester(CAPACITY);
    chrono.locomotor = Some(Locomotor::Teleport);
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("CHRONOT".into());
    r.sim.vehicles.insert("CHRONOT".into(), chrono);
    r
}

fn game(ore: &[(CellXY, u8, u8)]) -> World {
    let mut w = game_with_ore(&ruleset(), ore, 7);
    testkit::run(&mut w, 4);
    place_building(&mut w, "REFN", 0, at(-9, -9));
    w
}

/// A harvester of type `name` at `c` in *harvest*, seeking, empty, with
/// `remembered`.
fn seeker(w: &mut World, name: &str, c: CellXY, remembered: Option<CellXY>) -> ObjectId {
    let h = place_vehicle(w, name, 0, c, 0);
    let t = w.tick;
    let o = w.obj_mut(h).unwrap();
    o.beh.current = Some(Behaviour::Harvest);
    o.beh.start_tick = t;
    o.econ.harvest = Some(HarvestState {
        load: vec![0, 0],
        phase: Phase::Seeking,
        harvest_start: Some(t),
        remembered,
        ..Default::default()
    });
    h
}

fn marked(w: &World, c: CellXY) -> bool {
    w.cells
        .occ(c, Layer::Ground)
        .is_some_and(|o| o.vehicle_present())
}

fn phase(w: &World, h: ObjectId) -> Phase {
    w.obj(h).unwrap().econ.harvest.as_ref().unwrap().phase
}

fn landing(w: &World, h: ObjectId) -> Option<CellXY> {
    w.obj(h)
        .unwrap()
        .teleport
        .as_ref()
        .and_then(|t| t.landing)
        .map(|l| l.cell())
}

// spec: S-0402 claim 17 (the seeking drop, row "not full, remembered cell": the adopted cell is removed at once, so the miner never jumps to it; the ring search runs from where it stands; the landing point fixed for the adopted cell and its reservation stay)
// spec: S-0402 State and ordering (seeking, not full, has a remembered cell: for a chrono miner with active teleporting locomotion the processing then takes the ring-search rows; here its own cell, so loading after 1 tick)
#[test]
fn claim_17_the_remembered_cell_is_dropped() {
    let here = at(0, 0);
    let far = at(10, 0);
    let mut w = game(&[(here, ORE, 5), (far, ORE, 5)]);
    let h = seeker(&mut w, "CHRONOT", here, Some(far));
    let d = harvest::process_harvest(&mut w, h);
    let o = w.obj(h).unwrap();
    assert_eq!(o.dest, None, "the adopted cell is dropped");
    assert_eq!(o.pos.cell(), here);
    assert_eq!(
        phase(&w, h),
        Phase::Loading,
        "the search found its own cell"
    );
    assert_eq!(d, 1);
    assert_eq!(o.econ.harvest.as_ref().unwrap().remembered, None);
    assert_eq!(landing(&w, h), None, "no pending jump");
    assert!(marked(&w, far), "the reservation stays at the adopted cell");
    for _ in 0..3 {
        testkit::run(&mut w, 1);
    }
    assert_eq!(w.obj(h).unwrap().pos.cell(), here, "never jumps there");
}

// spec: S-0402 claim 17 (the seeking drop, row "not full, no remembered cell, destination present": the order-level destination is removed and the pending jump cancelled; the search runs from where it stands)
// spec: S-0402 State and ordering (seeking, not full, no remembered cell, has a destination: for such a chrono miner the next processing is that of the row the ring search then reaches, here another cell: drives there, standard delay)
#[test]
fn claim_17_a_present_destination_is_dropped() {
    let here = at(0, 0);
    let near = at(2, 0);
    let ordered = at(10, 4);
    let mut w = game(&[(near, ORE, 5), (ordered, ORE, 5)]);
    let h = seeker(&mut w, "CHRONOT", here, None);
    glasnost_sim::econ::assign_destination(&mut w, h, Some(Target::Cell(ordered)));
    assert_eq!(landing(&w, h), Some(ordered));
    harvest::process_harvest(&mut w, h);
    let o = w.obj(h).unwrap();
    assert_ne!(
        o.dest,
        Some(Target::Cell(ordered)),
        "the ordered cell is dropped"
    );
    assert_eq!(phase(&w, h), Phase::Seeking);
    let found = match o.dest {
        Some(Target::Cell(c)) => c,
        other => panic!("the search from where it stands sets a cell: {other:?}"),
    };
    assert_eq!(found, near, "found from where the miner stands");
    assert_eq!(landing(&w, h), Some(found));
    assert!(!marked(&w, ordered), "replaced by the new landing point");
}

// spec: S-0402 claim 17 (a search that sent the miner away produces a drop at the next processing only if the jump was blocked; then the pending jump is cancelled and the search runs again from where it stands; the dropped landing point keeps its reservation until the next accepted destination, S-0207 claim 20 h)
#[test]
fn claim_17_a_blocked_jump_is_dropped_at_the_next_processing() {
    let here = at(0, 0);
    let mut w = game(&[(at(2, 0), ORE, 5), (at(0, 2), ORE, 5)]);
    let h = seeker(&mut w, "CHRONOT", here, None);
    harvest::process_harvest(&mut w, h);
    let first = match w.obj(h).unwrap().dest {
        Some(Target::Cell(c)) => c,
        other => panic!("the first search sets a cell: {other:?}"),
    };
    assert!(marked(&w, first));
    // No movement update in between: the jump has not been made.
    harvest::process_harvest(&mut w, h);
    let o = w.obj(h).unwrap();
    assert_eq!(o.pos.cell(), here, "no jump");
    let second = match o.dest {
        Some(Target::Cell(c)) => c,
        other => panic!("the second search sets a cell: {other:?}"),
    };
    // The retained reservation leaves the first cell's entry outcome a moving blocker: the miner is not listed there, so it is not excused from the flag (S-0209 claim 5 rows VP1 and VR5); the search from here gives the other ore cell.
    assert_ne!(second, first);
    assert_eq!(landing(&w, h), Some(second));
    assert!(
        !marked(&w, first),
        "the next accepted destination cleared it from the remembered point"
    );
    assert!(marked(&w, second));
    // In a turn the due processing (its timer was not set by these direct calls) drops and searches again before the jump, which then lands on the searched cell and removes the destination (claim 3b; S-0207 claim 20 d).
    testkit::run(&mut w, 1);
    let o = w.obj(h).unwrap();
    assert!(
        [first, second].contains(&o.pos.cell()),
        "jumped to an ore cell"
    );
    assert_eq!(o.dest, None);
}

// spec: S-0402 claim 17 (the remembered cell is adopted by an ordinary destination assignment: pending point suppression, S-0207 claim 27, keeps it without a landing point, so no landing point or mark appears for it; the drop then removes it)
// spec: S-0207 claim 27 (S-0402's seeking drop adopts a remembered cell by an ordinary assignment: the drop that follows fixes no landing point for the adopted cell)
#[test]
fn claim_17_suppression_fixes_no_point_for_the_adopted_cell() {
    let here = at(0, 0);
    let far = at(10, 0);
    let mut w = game(&[(here, ORE, 5), (far, ORE, 5)]);
    let h = seeker(&mut w, "CHRONOT", here, Some(far));
    w.obj_mut(h).unwrap().econ.point_suppression = true;
    harvest::process_harvest(&mut w, h);
    let o = w.obj(h).unwrap();
    assert_eq!(o.dest, None);
    assert!(!o.econ.point_suppression, "consumed by the adoption");
    assert!(!marked(&w, far), "no reservation for the adopted cell");
    assert_eq!(phase(&w, h), Phase::Loading);
}

// spec: S-0402 claim 17 (the rule holds for a harvester whose current locomotor is the teleport locomotor, not for one whose type has the teleporter flag but that moves by another locomotor) and claim 3 (a destination: stays seeking, no search)
#[test]
fn claim_17_a_driving_flagged_miner_keeps_its_destination() {
    let here = at(0, 0);
    let ordered = at(10, 4);
    let mut w = game(&[(at(2, 0), ORE, 5), (ordered, ORE, 5)]);
    let h = seeker(&mut w, "CHRONO", here, None);
    glasnost_sim::econ::assign_destination(&mut w, h, Some(Target::Cell(ordered)));
    harvest::process_harvest(&mut w, h);
    let o = w.obj(h).unwrap();
    assert_eq!(o.dest, Some(Target::Cell(ordered)), "remains seeking");
    assert_eq!(phase(&w, h), Phase::Seeking);
}

// spec: S-0402 claim 17 (the seeking drop applies at every seeking processing past the full test, not only the first after the miner enters harvest; a full miner goes returning instead)
#[test]
fn claim_17_full_is_not_dropped() {
    let here = at(0, 0);
    let ordered = at(10, 4);
    let mut w = game(&[(ordered, ORE, 5)]);
    let h = seeker(&mut w, "CHRONOT", here, None);
    w.obj_mut(h).unwrap().econ.harvest.as_mut().unwrap().load = vec![CAPACITY, 0];
    glasnost_sim::econ::assign_destination(&mut w, h, Some(Target::Cell(ordered)));
    harvest::process_harvest(&mut w, h);
    assert_eq!(phase(&w, h), Phase::Returning);
    assert_eq!(
        w.obj(h).unwrap().dest,
        Some(Target::Cell(ordered)),
        "the full row drops nothing"
    );
    let _ = Kind::Vehicle;
}
