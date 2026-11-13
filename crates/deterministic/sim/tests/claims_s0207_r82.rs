//! Claim tests of S-0207 revision 82 (answers Q-1759): claim 28, which
//! locomotor a vehicle with the teleport-capable flag uses; claim 21's
//! cancellation of a pending jump at the install of driving; claim 20 a's
//! clearing of the old mark before the entry test and part e's later
//! fall-back; claim 16 b read on the current locomotor; with S-0402
//! revision 76's chrono-miner rules (claims 10, 10a and 17). The fixture is
//! the harvest ruleset with CHRONOT (a harvester with the teleporting
//! locomotor and the teleport-capable flag) and TELEQ (the same without the
//! flag, S-0402 H8's type Q and D22's type T). Every name and number is
//! invented; no game data.

mod harvest_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::Locomotor;
use glasnost_sim::econ::harvest::{self, HarvestState, Phase};
use glasnost_sim::econ::sell::Stage;
use glasnost_sim::econ::{assign_destination, release};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::Behaviour;
use glasnost_sim::scatter::{self, ScatterRequest};
use glasnost_sim::teleport;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, Layer, ObjectId, Pos, Target, World};
use harvest_common::*;

fn ruleset() -> Ruleset {
    let mut r = harvest_common::ruleset();
    let mut chrono = harvester(CAPACITY);
    chrono.teleporter = Some(true);
    chrono.locomotor = Some(Locomotor::Teleport);
    let mut q = chrono.clone();
    q.teleporter = None;
    for (n, u) in [("CHRONOT", chrono), ("TELEQ", q)] {
        r.registries.get_mut("vehicles").unwrap().push(n.into());
        r.sim.vehicles.insert(n.into(), u);
    }
    r
}

/// REFN covers (−6 … −4, −1 … 1) from the middle; its docking cell is
/// (−3, 0).
fn refinery_cell() -> CellXY {
    at(-6, -1)
}

fn dock_cell() -> CellXY {
    at(-3, 0)
}

fn game(ore: &[(CellXY, u8, u8)]) -> (World, ObjectId) {
    let mut w = game_with_ore(&ruleset(), ore, 7);
    testkit::run(&mut w, 4);
    let refn = place_building(&mut w, "REFN", 0, refinery_cell());
    (w, refn)
}

/// Whether the vehicle drives temporarily over its teleporting locomotor.
fn driving(w: &World, h: ObjectId) -> bool {
    release::temporary_drive(w, h)
}

/// A vehicle of `name` at `c`, with its own teleporting locomotor active
/// (restored when its placement installed driving), in *guard*.
fn teleporting(w: &mut World, name: &str, c: CellXY) -> ObjectId {
    let h = place_vehicle(w, name, 0, c, 0);
    if driving(w, h) {
        assert!(release::restore(w, h));
    }
    assert!(teleport::is_teleporter(w, h));
    h
}

fn landing(w: &World, h: ObjectId) -> Option<Pos> {
    w.obj(h).unwrap().teleport.as_ref().and_then(|t| t.landing)
}

fn marked(w: &World, c: CellXY) -> bool {
    w.cells
        .occ(c, Layer::Ground)
        .is_some_and(|o| o.vehicle_present())
}

fn cell(w: &World, h: ObjectId) -> CellXY {
    w.obj(h).unwrap().pos.cell()
}

/// A move of more than 2 cells in one tick (S-0207 scenario D34).
fn jumped(a: CellXY, b: CellXY) -> bool {
    (a.x - b.x).abs() > 2 || (a.y - b.y).abs() > 2
}

fn in_enter(w: &mut World, h: ObjectId, refn: ObjectId) {
    let t = w.tick;
    let o = w.obj_mut(h).unwrap();
    o.contact = Some(refn);
    o.beh.force(Behaviour::Enter, t, true);
    o.econ.harvest = Some(HarvestState {
        load: vec![CAPACITY, 0],
        phase: Phase::Returning,
        ..Default::default()
    });
}

// spec: S-0207 claim 28 table row 2 (any other assignment installs temporary driving, which receives the destination; behaviour and queue are left as they are; no landing point is fixed) and scenario D34 parts a and d (the flag-off type jumps)
#[test]
fn claim_28_any_other_assignment_installs_driving() {
    let (mut w, _) = game(&[]);
    let x = at(12, 6);
    let h = teleporting(&mut w, "CHRONOT", at(2, 6));
    let q = teleporting(&mut w, "TELEQ", at(2, 8));
    let qx = at(12, 8);
    assign_destination(&mut w, h, Some(Target::Cell(x)));
    assign_destination(&mut w, q, Some(Target::Cell(qx)));
    assert!(driving(&w, h), "driving is installed at the assignment");
    assert_eq!(landing(&w, h), None, "no landing point");
    assert_eq!(w.obj(h).unwrap().beh.current, Some(Behaviour::Guard));
    assert_eq!(w.obj(h).unwrap().dest, Some(Target::Cell(x)));
    assert!(landing(&w, q).is_some(), "the flag-off type fixes one");
    let mut q_jumped = false;
    for _ in 0..200 {
        let (a, qa) = (cell(&w, h), cell(&w, q));
        testkit::run(&mut w, 1);
        assert!(!jumped(a, cell(&w, h)), "the flagged miner never jumps");
        q_jumped |= jumped(qa, cell(&w, q));
    }
    assert!(q_jumped, "TELEQ jumps (D34 part d)");
    assert_eq!(cell(&w, h), x, "it drove to the ordered cell");
}

// spec: S-0207 claim 28 (an empty assignment counts: it installs driving) and claim 27 (an empty assignment to a vehicle with no destination is rejected as unchanged)
#[test]
fn claim_28_an_empty_assignment_installs_driving() {
    let (mut w, _) = game(&[]);
    let h = teleporting(&mut w, "CHRONOT", at(2, 6));
    assign_destination(&mut w, h, None);
    assert!(!driving(&w, h), "unchanged: rejected");
    // A destination held (set here without an assignment), then removed.
    w.obj_mut(h).unwrap().dest = Some(Target::Cell(at(12, 6)));
    assign_destination(&mut w, h, None);
    assert!(driving(&w, h));
}

// spec: S-0207 claim 28 (under the exclusions, here seized, the rule installs nothing: the destination goes to the current locomotor)
#[test]
fn claim_28_a_seized_vehicle_has_nothing_installed() {
    let (mut w, _) = game(&[]);
    let h = teleporting(&mut w, "CHRONOT", at(2, 6));
    w.obj_mut(h).unwrap().lift.seized = true;
    assign_destination(&mut w, h, None);
    assert!(!driving(&w, h));
}

// spec: S-0207 claim 28, the placement bullet (the placement clear installs driving: a placed miner drives from its first tick) and scenario D34 part f
#[test]
fn claim_28_placement_installs_driving() {
    let (mut w, _) = game(&[]);
    let h = place_vehicle(&mut w, "CHRONOT", 0, at(2, 6), 0);
    assert!(driving(&w, h));
    let q = place_vehicle(&mut w, "TELEQ", 0, at(2, 8), 0);
    assert!(!driving(&w, q), "the rule does not cover the flag-off type");
}

// spec: S-0207 claim 28 table row 1 (a qualified docking assignment with teleporting current: nothing changes and teleporting takes it under claim 20) and the jump bullet (the jump's removal of the destination installs driving from the jump tick on)
#[test]
fn claim_28_a_docking_assignment_jumps_and_the_jump_installs_driving() {
    let (mut w, refn) = game(&[]);
    let h = teleporting(&mut w, "CHRONOT", at(6, 0));
    w.obj_mut(h).unwrap().contact = Some(refn);
    assign_destination(&mut w, h, Some(Target::Cell(dock_cell())));
    assert!(!driving(&w, h));
    assert_eq!(landing(&w, h).map(|l| l.cell()), Some(dock_cell()));
    testkit::run(&mut w, 1);
    assert_eq!(cell(&w, h), dock_cell(), "the jump");
    assert!(driving(&w, h), "driving from the jump tick on");
    assert_eq!(w.obj(h).unwrap().dest, None);
    assert!(marked(&w, dock_cell()), "the cell it stands on is marked");
}

// spec: S-0207 claim 21 (installing temporary driving cancels a pending jump: the landing point's reservation stays and no jump is pending at restoration)
#[test]
fn claim_21_installing_driving_cancels_a_pending_jump() {
    let (mut w, refn) = game(&[]);
    let here = at(6, 0);
    let h = teleporting(&mut w, "CHRONOT", here);
    w.obj_mut(h).unwrap().contact = Some(refn);
    assign_destination(&mut w, h, Some(Target::Cell(dock_cell())));
    assert!(landing(&w, h).is_some());
    assign_destination(&mut w, h, None);
    assert!(driving(&w, h));
    assert_eq!(landing(&w, h), None, "the jump is cancelled");
    assert!(marked(&w, dock_cell()), "its reservation stays");
    assert!(release::restore(&mut w, h));
    testkit::run(&mut w, 3);
    assert_eq!(cell(&w, h), here, "no jump at restoration");
}

// spec: S-0207 claim 28, An occupied docking cell (the offer of an occupied cell is driven; once the cell clears the repeated dock test qualifies, a vehicle with a movement report gets claim 27's fallback, and at the next processing at which it stands still it jumps) and S-0402 claim 10 (the repeated dock test)
#[test]
fn claim_28_an_occupied_docking_cell_then_the_repeated_dock_test() {
    let (mut w, refn) = game(&[]);
    let occupant = place_vehicle(&mut w, "HARV", 0, dock_cell(), 0);
    let h = place_vehicle(&mut w, "CHRONOT", 0, at(6, 0), 0);
    in_enter(&mut w, h, refn);
    harvest::process_enter(&mut w, h);
    let o = w.obj(h).unwrap();
    assert_eq!(o.dest, Some(Target::Cell(dock_cell())));
    assert!(driving(&w, h), "the occupied cell is driven to");
    assert!(o.drive.as_ref().unwrap().goal.is_some());
    assert!(!o.econ.refresh);
    glasnost_sim::objects::remove(&mut w, occupant);
    // The unchanged offer is ignored; the repeated dock test qualifies and the driving point is a movement report.
    harvest::process_enter(&mut w, h);
    let o = w.obj(h).unwrap();
    assert!(o.econ.refresh, "claim 27's fallback");
    assert_eq!(o.beh.queued, Some(Behaviour::Enter));
    assert!(driving(&w, h));
    // Standing still at the next processing: the offer passes through refresh and teleporting is restored.
    harvest::process_enter(&mut w, h);
    assert!(!driving(&w, h));
    assert_eq!(landing(&w, h).map(|l| l.cell()), Some(dock_cell()));
    testkit::run(&mut w, 1);
    assert_eq!(cell(&w, h), dock_cell());
    assert!(driving(&w, h));
}

// spec: S-0207 claim 28, One jump (the offer restores teleporting and fixes the landing point at the docking cell's centre; the repeated dock test then finds teleporting current and fixes it again, the old mark cleared first, so the point is the same centre) and claim 20 (a) (the old mark is cleared before part b's entry test)
#[test]
fn claim_28_one_jump_when_the_offer_and_the_test_both_act() {
    let (mut w, refn) = game(&[]);
    let h = place_vehicle(&mut w, "CHRONOT", 0, at(6, 0), 0);
    assert!(driving(&w, h));
    in_enter(&mut w, h, refn);
    harvest::process_enter(&mut w, h);
    let c = dock_cell();
    let centre = Pos::centre(c, w.surface_at(256 * c.x + 128, 256 * c.y + 128));
    assert!(!driving(&w, h));
    assert_eq!(
        landing(&w, h).map(|l| (l.x, l.y)),
        Some((centre.x, centre.y))
    );
    assert!(marked(&w, c));
    assert!(!marked(&w, at(6, 0)), "the old mark moved");
    testkit::run(&mut w, 1);
    assert_eq!(cell(&w, h), c, "one jump, to the docking cell");
}

// spec: S-0207 claim 27 and claim 28, An occupied docking cell (the fallback stops a driving vehicle: it finishes the track it is on and begins no other; at the first later processing of enter at which it stands still it jumps) and claim 11
#[test]
fn claim_27_the_fallback_stops_a_moving_miner() {
    let (mut w, refn) = game(&[]);
    let start = at(10, 6);
    let h = place_vehicle(&mut w, "CHRONOT", 0, start, 0);
    assign_destination(&mut w, h, Some(Target::Cell(at(10, -8))));
    testkit::run(&mut w, 6);
    assert!(w.obj(h).unwrap().drive.as_ref().unwrap().track.is_some());
    let moving_from = cell(&w, h);
    in_enter(&mut w, h, refn);
    harvest::process_enter(&mut w, h);
    assert!(w.obj(h).unwrap().econ.refresh, "the fallback");
    let mut jump = None;
    for _ in 0..200 {
        let a = cell(&w, h);
        testkit::run(&mut w, 1);
        if jumped(a, cell(&w, h)) {
            jump = Some(a);
            break;
        }
    }
    let stood = jump.expect("it jumps");
    assert_eq!(cell(&w, h), dock_cell());
    assert!(
        (stood.x - moving_from.x).abs() <= 2 && (stood.y - moving_from.y).abs() <= 2,
        "it stopped where it was, not driving on"
    );
}

// spec: S-0207 claim 16 (b) (the teleporting locomotor, read as the current one: a chrono miner driving accepts, one teleporting refuses)
#[test]
fn claim_16_b_reads_the_current_locomotor() {
    let (mut w, _) = game(&[]);
    let req = ScatterRequest {
        threat: None,
        forced: true,
        anywhere: false,
    };
    let d = place_vehicle(&mut w, "CHRONOT", 0, at(2, 6), 0);
    assert!(driving(&w, d));
    assert!(scatter::request(&mut w, d, req));
    let t = teleporting(&mut w, "CHRONOT", at(2, 9));
    assert!(!scatter::request(&mut w, t, req));
    assert_eq!(w.obj(t).unwrap().dest, None);
}

// spec: S-0402 claim 17, Who reaches the drop, case (c) (a chrono miner at rest with teleporting restored adopts its remembered cell by an assignment that installs driving before the drop's test: the cell is not dropped and it drives there)
#[test]
fn claim_17_case_c_the_remembered_cell_is_driven_to() {
    let here = at(0, 4);
    let far = at(10, 4);
    let (mut w, _) = game(&[(here, ORE, 5), (far, ORE, 5)]);
    let h = teleporting(&mut w, "CHRONOT", here);
    let t = w.tick;
    let o = w.obj_mut(h).unwrap();
    o.beh.force(Behaviour::Harvest, t, true);
    o.econ.harvest = Some(HarvestState {
        load: vec![0, 0],
        phase: Phase::Seeking,
        harvest_start: Some(t),
        remembered: Some(far),
        ..Default::default()
    });
    harvest::process_harvest(&mut w, h);
    assert!(driving(&w, h));
    assert_eq!(w.obj(h).unwrap().dest, Some(Target::Cell(far)), "kept");
    assert_eq!(landing(&w, h), None);
}

// spec: S-0207 claim 28 (a destination that reaches no assignment hook, here a fall-back's queued destination, S-0215 claim 15 row 2, installs driving, which receives it; the teleporting locomotor neither jumps nor drops it)
#[test]
fn claim_28_a_queued_destination_is_driven() {
    let (mut w, _) = game(&[]);
    let h = teleporting(&mut w, "CHRONOT", at(2, 6));
    let first = at(12, 6);
    w.obj_mut(h).unwrap().queued_dests = vec![Target::Cell(first)];
    glasnost_sim::process::fall_back(&mut w, h, false);
    assert_eq!(w.obj(h).unwrap().dest, Some(Target::Cell(first)));
    testkit::run(&mut w, 1);
    assert!(driving(&w, h), "driving installed at its first locomotion");
    for _ in 0..200 {
        let a = cell(&w, h);
        testkit::run(&mut w, 1);
        assert!(!jumped(a, cell(&w, h)));
    }
    assert_eq!(cell(&w, h), first, "driven there");
}

// spec: S-0207 claim 20 (e) (a behaviour that ends once its destination is gone falls back to rest at a later tick, not within the jump tick) and scenario D22 run 1
#[test]
fn claim_20_e_the_fall_back_comes_after_the_jump_tick() {
    let (mut w, _) = game(&[]);
    let q = teleporting(&mut w, "TELEQ", at(2, 6));
    let t = w.tick;
    w.obj_mut(q).unwrap().beh.force(Behaviour::Move, t, true);
    assign_destination(&mut w, q, Some(Target::Cell(at(12, 6))));
    let mut jumped_at = None;
    for _ in 0..20 {
        let a = cell(&w, q);
        testkit::run(&mut w, 1);
        if jumped(a, cell(&w, q)) {
            jumped_at = Some(w.tick);
            break;
        }
    }
    jumped_at.expect("the jump");
    assert_eq!(w.obj(q).unwrap().beh.current, Some(Behaviour::Move));
    assert_eq!(w.obj(q).unwrap().dest, None);
}

// spec: S-0207 claim 28 and scenario D34 part c (an untouched cycle: the only jumps are to the docking cell, during enter, driving again from each jump tick; trips to resource cells are driven) and claim 21 (no reservation is left behind by the installs and restorations: every marked cell is a driving claim or the teleporting mark)
#[test]
fn claim_28_d34_c_the_cycle_jumps_only_to_the_docking_cell() {
    let mut ore = Vec::new();
    for dy in 0..3 {
        for dx in 0..3 {
            ore.push((at(10 + dx, 6 + dy), ORE, 2));
        }
    }
    let (mut w, _) = game(&ore);
    let h = place_vehicle(&mut w, "CHRONOT", 0, at(8, 6), 0);
    let t = w.tick;
    w.obj_mut(h).unwrap().beh.force(Behaviour::Harvest, t, true);
    let mut jumps = 0;
    let mut unloads = 0;
    let mut was_unloading = false;
    for _ in 0..6000 {
        let a = cell(&w, h);
        testkit::run(&mut w, 1);
        let b = cell(&w, h);
        if jumped(a, b) {
            jumps += 1;
            assert_eq!(b, dock_cell(), "a jump lands on the docking cell");
            assert_eq!(w.obj(h).unwrap().beh.current, Some(Behaviour::Enter));
            assert!(driving(&w, h), "driving from the jump tick on");
        }
        let unloading = w.obj(h).unwrap().beh.current == Some(Behaviour::Unload);
        if unloading {
            assert!(driving(&w, h), "driving through the unloading");
        }
        if was_unloading && !unloading {
            unloads += 1;
        }
        was_unloading = unloading;
        // Every marked cell is one its driving claims or its teleporting state's mark.
        let o = w.obj(h).unwrap();
        let mut own: Vec<CellXY> = o
            .drive
            .as_ref()
            .map(|d| d.claims.iter().map(|x| x.0).collect())
            .unwrap_or_default();
        own.extend(o.teleport.as_ref().and_then(|t| t.mark));
        let stale: Vec<CellXY> = w
            .cells
            .grid
            .iter()
            .map(|(c, _)| c)
            .filter(|&c| marked(&w, c) && !own.contains(&c))
            .collect();
        assert!(stale.is_empty(), "no reservation is left behind: {stale:?}");
        if unloads == 3 {
            break;
        }
    }
    assert_eq!(unloads, 3, "three loads");
    assert_eq!(jumps, 3, "one jump per load");
}

// spec: S-0207 claim 28, A sale of the refinery (a full chrono miner unloading after its docking jump: the sale's scatter request, made while the contact holds, is a qualified docking assignment; driving has no movement report, so teleporting is restored with the landing point at the scatter cell; a full miner's first harvest processing assigns nothing, so it jumps there, and that jump's removal installs driving again) and S-0402 claim 11 and claim 17 case (f)
#[test]
fn claim_28_a_sale_jumps_to_the_scatter_cell() {
    let mut r = ruleset();
    r.sim.buildings.get_mut("REFN").unwrap().econ.buildup = Some(true);
    let mut w = game_with_ore(&r, &[], 7);
    while w.tick <= w.orders_blocked_through {
        testkit::run(&mut w, 1);
    }
    let refn = place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "CHRONOT", 0, at(6, 0), 0);
    w.obj_mut(h).unwrap().econ.harvest = Some(HarvestState {
        load: vec![CAPACITY, 0],
        phase: Phase::Returning,
        ..Default::default()
    });
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: h,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: Some(Target::Object(refn)),
            dest: Some(Target::Object(refn)),
        }
    ));
    let mut to_dock = false;
    for _ in 0..300 {
        let a = cell(&w, h);
        testkit::run(&mut w, 1);
        to_dock |= jumped(a, cell(&w, h)) && cell(&w, h) == dock_cell();
        if w.obj(h).unwrap().beh.current == Some(Behaviour::Unload) {
            break;
        }
    }
    assert!(to_dock, "the docking jump");
    assert!(driving(&w, h), "driving through the unloading");
    assert_eq!(harvest::load(&w, h), vec![CAPACITY, 0]);
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Sell { actor: refn }
    ));
    // The sale's first processing: the request restores teleporting and fixes the landing point;
    // the miner's turn comes later in the same tick (the refinery is earlier in the update order),
    // its harvest processing assigns nothing (full) and its locomotion jumps.
    let mut moved = None;
    for _ in 0..20 {
        testkit::run(&mut w, 1);
        if cell(&w, h) != dock_cell() {
            moved = Some(w.tick);
            break;
        }
    }
    let t = moved.expect("it left the docking cell");
    let begun = w
        .obj(refn)
        .and_then(|b| b.econ.sale.as_ref())
        .map(|x| x.stage);
    assert_eq!(
        begun,
        Some(Stage::Begun),
        "in the tick of the sale's first processing"
    );
    let o = w.obj(h).unwrap();
    let s = cell(&w, h);
    assert!(
        (s.x - dock_cell().x).abs() <= 1 && (s.y - dock_cell().y).abs() <= 1,
        "a cell next to the docking cell"
    );
    let centre = Pos::centre(s, w.surface_at(256 * s.x + 128, 256 * s.y + 128));
    assert_eq!(
        (o.pos.x, o.pos.y),
        (centre.x, centre.y),
        "a jump to the cell's centre at tick {t}"
    );
    assert_eq!(o.dest, None);
    assert_eq!(o.beh.current, Some(Behaviour::Harvest));
    assert_eq!(
        harvest::load(&w, h),
        vec![CAPACITY, 0],
        "full: nothing assigned"
    );
    assert!(driving(&w, h), "the jump installs driving again");
}

/// A chrono miner with `load`, remembering `remembered`, docks by its jump
/// at a buildup refinery, unloads, and the refinery is sold then. Returns
/// the world and the miner, right after the sale order.
fn sold_while_unloading(
    ore: &[(CellXY, u8, u8)],
    load: Vec<i32>,
    remembered: Option<CellXY>,
) -> (World, ObjectId, ObjectId) {
    let mut r = ruleset();
    r.sim.buildings.get_mut("REFN").unwrap().econ.buildup = Some(true);
    let mut w = game_with_ore(&r, ore, 7);
    while w.tick <= w.orders_blocked_through {
        testkit::run(&mut w, 1);
    }
    let refn = place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "CHRONOT", 0, at(6, 0), 0);
    w.obj_mut(h).unwrap().econ.harvest = Some(HarvestState {
        load,
        phase: Phase::Returning,
        remembered,
        ..Default::default()
    });
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: h,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: Some(Target::Object(refn)),
            dest: Some(Target::Object(refn)),
        }
    ));
    for _ in 0..300 {
        testkit::run(&mut w, 1);
        if w.obj(h).unwrap().beh.current == Some(Behaviour::Unload) {
            break;
        }
    }
    assert_eq!(cell(&w, h), dock_cell(), "docked by its jump");
    assert_eq!(
        w.obj(h).unwrap().econ.harvest.as_ref().unwrap().remembered,
        remembered
    );
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Sell { actor: refn }
    ));
    (w, h, refn)
}

/// The tick and the cell of the miner's first move off the docking cell,
/// and whether that move was a jump.
fn first_move(w: &mut World, h: ObjectId) -> (i32, CellXY, bool) {
    for _ in 0..200 {
        let a = cell(w, h);
        testkit::run(w, 1);
        let b = cell(w, h);
        if b != a {
            return (w.tick, b, jumped(a, b));
        }
    }
    panic!("it never left the docking cell");
}

// spec: S-0207 claim 28, A sale of the refinery (a chrono miner not full: the adopted remembered cell is dropped and the seeking drop's empty assignment installs driving, which cancels the scatter jump; the ring-search cell is a qualified docking assignment while the contact holds, so teleporting is restored and the miner jumps to that resource cell in the same turn, without reaching the scatter cell; the jump installs driving again) and S-0402 claim 17 case (f)
#[test]
fn claim_28_a_sale_with_a_part_load_jumps_to_the_ring_search_cell() {
    let remembered = at(4, 3);
    let near = at(0, 2);
    let (mut w, h, refn) = sold_while_unloading(
        &[(remembered, ORE, 5), (near, ORE, 5)],
        vec![2, 0],
        Some(remembered),
    );
    let (t, c, jump) = first_move(&mut w, h);
    assert_eq!(
        c, near,
        "the ring-search cell at tick {t}, not the scatter cell"
    );
    assert!(jump, "a jump");
    let begun = w
        .obj(refn)
        .and_then(|b| b.econ.sale.as_ref())
        .map(|x| x.stage);
    assert_eq!(
        begun,
        Some(Stage::Begun),
        "in the tick of the sale's first processing, while the contact holds"
    );
    assert!(driving(&w, h), "the jump installs driving again");
    assert_eq!(w.obj(h).unwrap().beh.current, Some(Behaviour::Harvest));
}

// spec: S-0207 claim 28, A sale of the refinery (a chrono miner not full whose ring search finds no cell stays where it is with driving installed) and S-0402 claim 17 case (f) (when the search finds nothing the miner goes idle with driving installed and no destination)
#[test]
fn claim_28_a_sale_with_nothing_found_stays_on_the_docking_cell() {
    let (mut w, h, _) = sold_while_unloading(&[], vec![2, 0], Some(at(4, 3)));
    let mut seen_idle = false;
    for _ in 0..20 {
        testkit::run(&mut w, 1);
        assert_eq!(cell(&w, h), dock_cell(), "no jump to the scatter cell");
        let o = w.obj(h).unwrap();
        if o.econ.harvest.as_ref().unwrap().phase == Phase::Idle {
            seen_idle = true;
            assert_eq!(o.dest, None);
            assert!(driving(&w, h));
            assert_eq!(landing(&w, h), None, "the scatter jump is cancelled");
        }
    }
    assert!(seen_idle, "idle");
}

// spec: S-0207 claim 28, A sale of the refinery (a chrono miner not full whose remembered cell holds a ground-layer vehicle: driving takes the adopted cell, which cancels the scatter jump; nothing is dropped, no ring search follows, and the miner drives to its remembered cell)
#[test]
fn claim_28_a_sale_with_the_remembered_cell_taken_drives_there() {
    let ore_cell = at(4, 3);
    let (mut w, h, _) = sold_while_unloading(&[(ore_cell, ORE, 5)], vec![2, 0], Some(ore_cell));
    let blocker = place_vehicle(&mut w, "HARV", 1, ore_cell, 0);
    assert!(marked(&w, ore_cell));
    let (t, c, jump) = first_move(&mut w, h);
    assert!(!jump, "a driven step at tick {t}, to {c:?}");
    let o = w.obj(h).unwrap();
    assert_eq!(o.dest, Some(Target::Cell(ore_cell)), "its remembered cell");
    assert_eq!(landing(&w, h), None, "the scatter jump is cancelled");
    assert!(driving(&w, h));
    assert_eq!(cell(&w, blocker), ore_cell);
}

// ---- Seams sweep 3 (docs/m8-claims/seams-3.md): the exclusions of claim 28 on a real destination.

/// A teleporting CHRONOT holding a destination, then given another one
/// that is no docking assignment; the state is changed by `exclude`.
fn exclusion_case(exclude: impl Fn(&mut World, ObjectId)) -> (World, ObjectId) {
    let (mut w, _) = game(&[]);
    let h = teleporting(&mut w, "CHRONOT", at(2, 6));
    exclude(&mut w, h);
    w.obj_mut(h).unwrap().dest = Some(Target::Cell(at(12, 6)));
    assign_destination(&mut w, h, Some(Target::Cell(at(12, 8))));
    (w, h)
}

// spec: S-0207 claim 28 (a vehicle with a warp pending is excluded: the rule installs nothing, and the destination goes to the current locomotor)
#[test]
fn claim_28_a_warp_pending_vehicle_has_nothing_installed() {
    use glasnost_sim::superweapons::warp::{Member, Stage};
    for stage in [Stage::Pending, Stage::Leaving(3), Stage::Expired] {
        let (w, h) = exclusion_case(|w, h| {
            let pos = w.obj(h).unwrap().pos;
            w.obj_mut(h).unwrap().sw.warp = Some(Member {
                stage,
                house: HouseId(0),
                landing: pos,
                redirected: false,
                fell_back: false,
                refused: false,
                entry_ok: false,
            });
        });
        assert!(!driving(&w, h), "{stage:?}");
        assert!(teleport::is_teleporter(&w, h), "{stage:?}");
    }
}

// spec: S-0207 claim 28 (a gripped vehicle, glossary, is excluded: the rule installs nothing)
#[test]
fn claim_28_a_gripped_vehicle_has_nothing_installed() {
    let (w, h) = exclusion_case(|w, h| {
        let lifter = place_vehicle(w, "TELEQ", 0, at(6, 6), 0);
        w.obj_mut(h).unwrap().lift.lifter = Some(lifter);
    });
    assert!(!driving(&w, h));
    assert!(teleport::is_teleporter(&w, h));
}

// spec: S-0207 claim 28 (a seized vehicle is excluded; the control: with no exclusion the same assignment installs driving and fixes no landing point)
#[test]
fn claim_28_the_exclusions_against_a_control() {
    let (w, h) = exclusion_case(|_, _| {});
    assert!(driving(&w, h), "control");
    assert_eq!(landing(&w, h), None, "control");
    let (w, h) = exclusion_case(|w, h| w.obj_mut(h).unwrap().lift.seized = true);
    assert!(!driving(&w, h));
}
