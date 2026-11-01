//! Claim tests for magnetic lift revisions 5 and 6 (S-0607, M7a step 1o):
//! the jumpjet stop when a grip ends (claim 13, with S-0502 revision 14's
//! claims 5, 9, 17 and 24), restoration while a movement commitment lasts
//! (claim 20), a building lifter's docking point (claim 7) and claim 4's
//! harvester case under a warhead that names another locomotor. Synthetic
//! data only.

#[path = "lift_common/mod.rs"]
mod lift_common;

use glasnost_map::CellXY;
use glasnost_num::F64;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::BuildingType;
use glasnost_sim::jump::{self, Phase};
use glasnost_sim::lift;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{ObjectId, Pos, Target, World};
use lift_common::*;

/// LIFTER (house 0) at (0, 0) and a victim VICT (house 1) at (6, 0), seized.
fn seized(tweak: impl FnOnce(&mut Ruleset)) -> (World, ObjectId, ObjectId) {
    let mut w = world(tweak);
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    beam(&mut w, l, v);
    assert!(lift::seized(&w, v));
    (w, l, v)
}

/// The cell of the movement point fixed at the seize (claim 7).
fn fixed_cell(w: &World, v: ObjectId) -> CellXY {
    match w.obj(v).unwrap().dest {
        Some(Target::Cell(c)) => c,
        other => panic!("the destination is not a cell: {other:?}"),
    }
}

fn point(w: &World, v: ObjectId) -> Option<Pos> {
    w.obj(v).unwrap().jump.as_ref().unwrap().dest
}

/// Orders the lifter one cell away (claim 12 row 2: the lifter ends the grip).
fn order_lifter(w: &mut World, l: ObjectId) {
    glasnost_sim::econ::assign_destination(w, l, Some(Target::Cell(at(0, 1))));
}

/// Records each update of a dropped unit's fall: (height before, height
/// after, the crash rate used).
fn fall_steps(w: &mut World, v: ObjectId) -> Vec<(i32, i32, F64)> {
    let mut steps = Vec::new();
    while lift::dropped(w, v) && w.tick < 3000 {
        let before = height(w, v);
        let rate = lift::rate(w, v).map(|r| r.to_f64());
        let crashing = jump::phase(w, v) == Some(Phase::Crashing);
        run(w, 1);
        if let (true, Some(k), Some(_)) = (crashing, rate, w.obj(v)) {
            steps.push((before, height(w, v), k));
        }
    }
    steps
}

/// Whether a fall step ended above what any descent towards the bare
/// ground allows (after − before − 1 + k > 0): the ordinary height step
/// rose towards a clearance before the crash drop.
fn held(w: &World, (before, after, k): (i32, i32, F64)) -> bool {
    let mode = w.rules().mode;
    F64::from_i64(i64::from(after - before - 1), mode)
        .add(k, mode)
        .gt(F64::ZERO)
}

// spec: S-0607 claim 13 (revision 6): after claim 8's arrival over a cell centre the stop's found point and the movement commitment are kept when the fall begins: the jumpjet movement stays moving
#[test]
fn claim_13_drop_keeps_the_found_point_and_commitment() {
    let (mut w, l, v) = seized(|_| {});
    assert!(run_until(&mut w, 400, |w| !lift::grips(w, l, v)));
    assert!(lift::dropped(&w, v), "an arrival drop");
    assert_eq!(jump::phase(&w, v), Some(Phase::Crashing));
    let p = w.obj(v).unwrap().pos;
    assert_eq!((p.x.rem_euclid(256), p.y.rem_euclid(256)), (128, 128));
    let d = point(&w, v).expect("the found point is kept");
    assert_eq!(d.cell(), p.cell(), "its own cell is free: found there");
    assert!(jump::moving(&w, v), "the commitment is kept");
    assert_eq!(w.obj(v).unwrap().dest, None, "no order-level destination");
}

// spec: S-0607 claim 13 (revision 6) and S-0502 claim 9 and F3 (revision 14; scenario L3 run f, J27 b): a dropped unit over its cell centre whose kept point lies in another cell (its own cell holds a parked vehicle, so the stop's search found a neighbour) uses the look-ahead at zero speed: its ordinary height step rises towards its own cell's clearance before the crash drop
#[test]
fn claim_13_kept_point_elsewhere_holds_the_fall_at_the_clearance() {
    let (mut w, l, v) = seized(|_| {});
    let c = fixed_cell(&w, v);
    // Parked after the seize's point search, before the arrival.
    spawn(&mut w, Kind::Vehicle, "VICT", 0, c);
    assert!(run_until(&mut w, 400, |w| !lift::grips(w, l, v)));
    assert!(lift::dropped(&w, v));
    let own = w.obj(v).unwrap().pos.cell();
    assert_eq!(own, c, "dropped over the parked vehicle's cell");
    let d = point(&w, v).expect("a point is kept");
    assert_ne!(d.cell(), own, "the found point lies in another cell");
    let steps = fall_steps(&mut w, v);
    assert!(
        steps.iter().any(|&s| held(&w, s)),
        "the fall is held at the clearance: {steps:?}"
    );
}

// spec: S-0502 claim 9 and F3 (revision 14) and S-0607 claims 13 and 15: a dropped unit whose point lies in its own cell (the cell-centre detour replaced the found point) uses the direct reference, even with a vehicle listed in its cell: no height step rises towards that vehicle's clearance
#[test]
fn claim_13_point_in_own_cell_falls_by_the_direct_reference() {
    let (mut w, l, v) = seized(|_| {});
    spawn(&mut w, Kind::Vehicle, "VICT", 0, at(3, 0));
    assert!(run_until(&mut w, 400, |w| w
        .obj(v)
        .is_some_and(|o| o.pos.cell() == at(3, 0))
        && height(w, v) > 300));
    order_lifter(&mut w, l);
    assert!(lift::dropped(&w, v));
    assert!(run_until(&mut w, 100, |w| jump::phase(w, v) == Some(Phase::Crashing)));
    assert_eq!(point(&w, v).map(|d| d.cell()), Some(at(3, 0)));
    let steps = fall_steps(&mut w, v);
    assert!(!steps.is_empty(), "it fell");
    assert!(
        steps.iter().all(|&s| !held(&w, s)),
        "no step held at a clearance: {steps:?}"
    );
}

// spec: S-0607 claims 13 and 20 (revision 6) and S-0502 claims 5 and 24 (revision 14; scenario L3 run b, J27 a): a unit set down before its first jumpjet update keeps the found point and its commitment; its next eligible update takes off, it flies to the point and lands there, and it is not restored while the commitment lasts, and is restored at the end of the turn in which the commitment ends (revision 14, with no order)
#[test]
fn claim_13_committed_set_down_takes_off_then_restores() {
    let (mut w, l, v) = seized(|_| {});
    assert_eq!(height(&w, v), 0);
    assert_eq!(
        jump::phase(&w, v),
        Some(Phase::Grounded),
        "before its first update"
    );
    assert!(jump::moving(&w, v), "committed");
    order_lifter(&mut w, l);
    assert!(!lift::dropped(&w, v), "set down, not dropped");
    assert!(w.obj(v).unwrap().lift.set_down);
    let d = point(&w, v).expect("the found point");
    assert!(jump::moving(&w, v), "set down clears no commitment");
    run(&mut w, 1);
    assert_eq!(
        jump::phase(&w, v),
        Some(Phase::Ascending),
        "its next eligible update takes off"
    );
    // Orders while committed: each fall-back to rest is refused restoration.
    let mut flew = false;
    while jump::moving(&w, v) && w.tick < 2000 {
        flew |= height(&w, v) > 0;
        let c = w.obj(v).unwrap().pos.cell();
        order_move(&mut w, v, CellXY { x: c.x + 2, y: c.y });
        run(&mut w, 1);
        if jump::moving(&w, v) {
            assert!(lift::seized(&w, v), "not restored while committed");
        }
    }
    assert!(flew, "it flew");
    let p = w.obj(v).unwrap().pos;
    assert_eq!(p.cell(), d.cell(), "it landed at the point");
    assert_eq!(height(&w, v), 0);
    assert!(
        !lift::seized(&w, v),
        "restored at the end of the turn in which the commitment ended"
    );
}

// spec: S-0607 claim 9 (revision 5): a gripped unit given no destination is set down where it is; claim 13's jumpjet stop applies, so it flies to the stop's found point, the centre of its own free cell, before it descends and lands
#[test]
fn claim_9_no_destination_flies_to_the_found_point() {
    let (mut w, l, v) = seized(|_| {});
    assert!(run_until(&mut w, 400, |w| w
        .obj(v)
        .is_some_and(|o| o.pos.cell() == at(4, 0))
        && height(w, v) > 300));
    glasnost_sim::econ::assign_destination(&mut w, v, None);
    run(&mut w, 1);
    assert!(!lift::grips(&w, l, v));
    let c = w.obj(v).unwrap().pos.cell();
    let d = point(&w, v).expect("the found point");
    assert_eq!(d.cell(), c);
    assert_eq!((d.x, d.y), (c.x * 256 + 128, c.y * 256 + 128));
    assert!(jump::moving(&w, v));
    // spec: S-0309 r70 claim 29 and its Changes for S-0304 claim 2 (the lifter, a covered holder in attack, keeps the airborne victim, which its firing check refuses, until the next check tick rather than dropping it at its attack processing; the later area-guard search then picks the victim again and grips it as it lands). This claim is the victim's own stop, so the lifter leaves play once the grip has ended.
    glasnost_sim::objects::remove(&mut w, l);
    assert!(run_until(&mut w, 400, |w| height(w, v) == 0 && !jump::moving(w, v)));
    let p = w.obj(v).unwrap().pos;
    assert_eq!((p.x, p.y), (d.x, d.y), "landed at the point");
    assert_eq!(health(&w, v), 100);
}

/// Turns every cell water except the lifter's (0, 0), so that a wheeled
/// unit's stop finds no cell (S-0502 claim 17).
fn flood(w: &mut World) {
    let mut cells = Vec::new();
    for dx in -SIZE..SIZE {
        for dy in -SIZE..SIZE {
            let c = at(dx, dy);
            if c != at(0, 0) && w.cells.grid.get(c).is_some() {
                cells.push(c);
            }
        }
    }
    water(w, &cells);
}

fn quirk(on: bool) -> impl FnOnce(&mut Ruleset) {
    move |r| r.globals.quirks.jumpjet_no_cell_self_destruct = on
}

// spec: S-0607 claim 13 (revision 5, design) and S-0502 claim 17 (revision 14, magnetic-release safety departure; scenario J27 c): with no cell for the stop, a unit its lifter drops does not destroy itself, with either setting of quirks.jumpjet_no_cell_self_destruct; it keeps no point and falls
#[test]
fn claim_13_no_cell_lifter_drop_does_not_self_destruct() {
    for on in [true, false] {
        let (mut w, l, v) = seized(quirk(on));
        flood(&mut w);
        assert!(run_until(&mut w, 400, |w| height(w, v) > 300));
        order_lifter(&mut w, l);
        assert!(lift::dropped(&w, v), "quirk {on}: dropped");
        assert_eq!(health(&w, v), 100, "quirk {on}: no self-destruction");
        assert!(w.obj(v).is_some());
        assert!(
            run_until(&mut w, 100, |w| jump::phase(w, v) == Some(Phase::Crashing)),
            "quirk {on}: it falls"
        );
    }
}

// spec: S-0607 claim 13 (revision 5, design) and S-0502 claim 17 (revision 14, magnetic-release safety departure): with no cell for the stop, a unit its lifter sets down stops where it is, with no movement point and no commitment, and does not destroy itself, with either setting of the quirk; claim 20 restores it
#[test]
fn claim_13_no_cell_lifter_set_down_stops() {
    for on in [true, false] {
        let (mut w, l, v) = seized(quirk(on));
        flood(&mut w);
        let p = w.obj(v).unwrap().pos;
        assert!(jump::moving(&w, v), "quirk {on}: committed at release");
        order_lifter(&mut w, l);
        assert!(w.obj(v).unwrap().lift.set_down, "quirk {on}");
        assert_eq!(health(&w, v), 100, "quirk {on}: no self-destruction");
        assert_eq!(point(&w, v), None, "quirk {on}: no movement point");
        assert!(!jump::moving(&w, v), "quirk {on}: no longer moving");
        run(&mut w, 5);
        assert_eq!(w.obj(v).unwrap().pos, p, "quirk {on}: it stays");
        glasnost_sim::objects::remove(&mut w, l);
        assert!(restored(&mut w, v), "quirk {on}: restored");
    }
}

// spec: S-0607 claim 9 (revisions 5 and 8) and S-0502 claim 17 and Edge cases (revision 16, quirks.jumpjet_no_cell_self_destruct; scenario J27 c): a grip ended by the no-destination exception keeps S-0502 claim 17's ordinary no-cell outcome: on, the unit destroys itself with no credit; off, in the air, it hovers over its current cell with no movement point and no commitment, neither descends nor flies anywhere, and stays set down and seized; claim 20 does not restore it while it hovers, and a failed retry at the end of each of its turns (revision 14) changes nothing, nor do its fall-backs to rest
#[test]
fn claim_9_no_cell_keeps_the_ordinary_outcome() {
    for on in [true, false] {
        let (mut w, l, v) = seized(quirk(on));
        flood(&mut w);
        assert!(run_until(&mut w, 400, |w| height(w, v) > 300));
        let kills_before = w.houses[0].combat.kills;
        glasnost_sim::econ::assign_destination(&mut w, v, None);
        run(&mut w, 1);
        assert!(!lift::grips(&w, l, v), "quirk {on}: the grip ends");
        if on {
            assert!(
                w.obj(v).is_none_or(|o| o.health <= 0),
                "quirk on: destroyed"
            );
            assert_eq!(w.houses[0].combat.kills, kills_before, "no credit");
            continue;
        }
        assert_eq!(health(&w, v), 100, "quirk off: alive");
        assert_eq!(jump::phase(&w, v), Some(Phase::Hovering));
        assert!(!jump::moving(&w, v), "quirk off: no movement commitment");
        assert_eq!(point(&w, v), None, "quirk off: no movement point");
        let o = w.obj(v).unwrap();
        assert!(
            o.lift.set_down && lift::seized(&w, v),
            "set down and seized"
        );
        let (p, z) = (o.pos, height(&w, v));
        // Refused move orders make it fall back to rest, which neither tests
        // restoration nor retries (claim 20, revision 14); the retry at the
        // end of each turn finds nothing.
        for _ in 0..5 {
            order_move(
                &mut w,
                v,
                CellXY {
                    x: p.cell().x + 2,
                    y: p.cell().y,
                },
            );
            run(&mut w, 20);
            assert!(lift::seized(&w, v), "not restored while it hovers");
            assert_eq!(jump::phase(&w, v), Some(Phase::Hovering));
            assert!(!jump::moving(&w, v));
            assert_eq!(w.obj(v).unwrap().pos, p, "it neither flies nor descends");
            assert_eq!(height(&w, v), z);
        }
        assert_eq!(health(&w, v), 100, "not destroyed");
    }
}

/// Makes `cells` land again, with the tile of the untouched cell (0, 0).
fn dry(w: &mut World, cells: &[CellXY]) {
    let tile = w.cells.grid.get(at(0, 0)).unwrap().tile;
    for &c in cells {
        w.cells.grid.get_mut(c).unwrap().tile = tile;
    }
    glasnost_sim::load::recalc_cells(w, cells);
}

// spec: S-0607 claim 20 (revision 8, the hovering set-down unit, design) and S-0502 Edge cases (revision 16; scenario J27 c, a successful retry): once a cell is free again, the end of its next turn (revision 14) repeats the stop's search, which finds it, with no order given: the unit re-orders itself to that cell's centre (a movement point, no order-level destination), releases its hover slot, flies there, descends and lands, and is restored at the end of the turn in which it lands
#[test]
fn claim_20_hovering_set_down_retries_the_search() {
    let (mut w, l, v) = seized(quirk(false));
    flood(&mut w);
    assert!(run_until(&mut w, 400, |w| height(w, v) > 300));
    glasnost_sim::econ::assign_destination(&mut w, v, None);
    run(&mut w, 1);
    assert!(!lift::grips(&w, l, v));
    assert_eq!(jump::phase(&w, v), Some(Phase::Hovering));
    let c = w.obj(v).unwrap().pos.cell();
    assert_eq!(
        w.obj(v).unwrap().jump.as_ref().unwrap().slot,
        Some(c),
        "it holds its cell's hover slot"
    );
    run(&mut w, 5);
    assert!(lift::seized(&w, v));
    assert_eq!(jump::phase(&w, v), Some(Phase::Hovering), "no cell yet");
    dry(&mut w, &[c]);
    run(&mut w, 1);
    assert!(
        jump::moving(&w, v),
        "the end of its next turn retried the search"
    );
    assert!(lift::seized(&w, v), "not restored by the retry");
    let d = point(&w, v).expect("the found point");
    assert_eq!(d.cell(), c);
    assert_eq!((d.x, d.y), (c.x * 256 + 128, c.y * 256 + 128), "the centre");
    assert_eq!(w.obj(v).unwrap().dest, None, "no order-level destination");
    run(&mut w, 2);
    assert!(height(&w, v) > 0, "still in the air");
    assert_eq!(
        w.obj(v).unwrap().jump.as_ref().unwrap().slot,
        None,
        "the hover slot is released"
    );
    assert!(run_until(&mut w, 400, |w| height(w, v) == 0 && !jump::moving(w, v)));
    let p = w.obj(v).unwrap().pos;
    assert_eq!((p.x, p.y), (d.x, d.y), "landed at the point");
    assert_eq!(health(&w, v), 100);
    assert!(
        !lift::seized(&w, v),
        "restored at the end of its landing turn"
    );
}

// spec: S-0607 claim 9 (revision 8, design: on the ground, height 0 or less, the unit still moving because it was released before its first jumpjet update) and S-0502 Edge cases (revision 16): with no cell and the switch off it stays grounded where it is with no movement point and no commitment, and claim 20 restores it as any set-down unit, revision 14: at the end of the same turn, so the grounded state is not seen between ticks; with the switch on it is destroyed
#[test]
fn claim_9_no_cell_on_the_ground_stays_grounded() {
    for on in [true, false] {
        let (mut w, l, v) = seized(quirk(on));
        flood(&mut w);
        assert_eq!(height(&w, v), 0);
        assert!(jump::moving(&w, v), "quirk {on}: before its first update");
        let p = w.obj(v).unwrap().pos;
        glasnost_sim::econ::assign_destination(&mut w, v, None);
        run(&mut w, 1);
        assert!(!lift::grips(&w, l, v), "quirk {on}: the grip ends");
        if on {
            assert!(
                w.obj(v).is_none_or(|o| o.health <= 0),
                "quirk on: destroyed"
            );
            continue;
        }
        // The lifter keeps its target (claim 12): out of play, it cannot seize the unit again.
        glasnost_sim::objects::remove(&mut w, l);
        assert_eq!(health(&w, v), 100);
        assert!(
            !lift::seized(&w, v),
            "restored at the end of the turn in which it stopped grounded"
        );
        assert!(
            w.obj(v).unwrap().jump.is_none(),
            "its own locomotor: the jumpjet movement is gone"
        );
        assert_eq!(w.obj(v).unwrap().pos, p, "it stays where it is");
        run(&mut w, 5);
        assert_eq!(w.obj(v).unwrap().pos, p, "it stays where it is");
        assert_eq!(height(&w, v), 0);
    }
}

// spec: S-0607 claim 20 (revision 9, the hovering exclusion for every route, design; scenario L14; answers Q-1344) and S-0502 Edge cases (revision 17; scenario J27 c): a set-down unit that kept its movement commitment flies towards its point; a stop there with no cell and the switch off leaves it hovering in the air with no movement point and no commitment; it is not restored at its fall-backs to rest or at the end of its turns, whose retries find nothing and change nothing (revision 14); once a cell is free again the retry at the end of its next turn finds it, it flies there and lands, and is restored at the end of its landing turn
#[test]
fn claim_20_hovering_exclusion_for_a_committed_set_down() {
    let (mut w, l, v) = seized(quirk(false));
    order_lifter(&mut w, l);
    assert!(w.obj(v).unwrap().lift.set_down, "set down at height 0");
    assert!(jump::moving(&w, v), "it keeps its commitment");
    assert!(run_until(&mut w, 400, |w| height(w, v) > 100));
    assert!(jump::moving(&w, v), "still flying to its point");
    // The lifter leaves play (it cannot seize the unit again) and no land is left.
    glasnost_sim::objects::remove(&mut w, l);
    let land_tile = w.cells.grid.get(at(0, 0)).unwrap().tile;
    flood(&mut w);
    water(&mut w, &[at(0, 0)]);
    // A stop on the way (S-0502 claim 17): its search finds no cell.
    jump::stop(&mut w, v);
    assert_eq!(health(&w, v), 100, "not destroyed: the switch is off");
    assert_eq!(jump::phase(&w, v), Some(Phase::Hovering), "it hovers");
    assert!(!jump::moving(&w, v), "no movement commitment");
    assert_eq!(point(&w, v), None, "no movement point");
    assert!(w.obj(v).unwrap().lift.set_down && lift::seized(&w, v));
    let (p, z) = (w.obj(v).unwrap().pos, height(&w, v));
    for _ in 0..3 {
        order_move(
            &mut w,
            v,
            CellXY {
                x: p.cell().x + 2,
                y: p.cell().y,
            },
        );
        run(&mut w, 20);
        assert!(lift::seized(&w, v), "not restored while it hovers");
        assert_eq!(jump::phase(&w, v), Some(Phase::Hovering));
        assert!(!jump::moving(&w, v), "the retry found nothing");
        assert_eq!((w.obj(v).unwrap().pos, height(&w, v)), (p, z));
    }
    // A neighbouring cell becomes land again (its own cell keeps the mark of its low ascent, S-0502 claim 4).
    let here = p.cell();
    let c = CellXY {
        x: here.x + 1,
        y: here.y,
    };
    w.cells.grid.get_mut(c).unwrap().tile = land_tile;
    glasnost_sim::load::recalc_cells(&mut w, &[c]);
    run(&mut w, 1);
    assert!(
        jump::moving(&w, v),
        "the end of its next turn retried the search"
    );
    assert!(lift::seized(&w, v), "not restored by the retry");
    let d = point(&w, v).expect("the found point");
    assert_eq!((d.x, d.y), (c.x * 256 + 128, c.y * 256 + 128), "its centre");
    assert!(run_until(&mut w, 400, |w| height(w, v) == 0 && !jump::moving(w, v)));
    assert!(
        !lift::seized(&w, v),
        "restored at the end of its landing turn"
    );
}

// spec: S-0607 claim 9 (revision 9, design: on the ground, height above the surface 0 or less, it stays where it is and its flight state becomes grounded, even if it was ascending at height 0; answers Q-1344) and S-0502 Edge cases (revision 17): a seized unit given no destination in the tick after its take-off update, still at height 0 and ascending, with no cell and the switch off, stays put grounded with no point and no commitment, and claim 20 restores it as any grounded set-down unit (revision 14: at the end of that same turn)
#[test]
fn claim_9_no_cell_ascending_at_height_zero_becomes_grounded() {
    let (mut w, l, v) = seized(quirk(false));
    flood(&mut w);
    run(&mut w, 1);
    assert_eq!(jump::phase(&w, v), Some(Phase::Ascending), "took off");
    assert_eq!(height(&w, v), 0, "still at height 0");
    assert!(jump::moving(&w, v));
    let p = w.obj(v).unwrap().pos;
    glasnost_sim::econ::assign_destination(&mut w, v, None);
    run(&mut w, 1);
    assert!(!lift::grips(&w, l, v), "the grip ends");
    glasnost_sim::objects::remove(&mut w, l);
    assert_eq!(health(&w, v), 100);
    // spec: S-0607 claim 20 (revision 14: the restoration test at the end of the same turn restores it, grounded and not moving)
    assert!(
        !lift::seized(&w, v),
        "restored as any grounded set-down unit, at the end of that turn"
    );
    assert!(
        w.obj(v).unwrap().jump.is_none(),
        "its own locomotor: the jumpjet movement is gone"
    );
    assert_eq!(height(&w, v), 0);
    run(&mut w, 5);
    assert_eq!(w.obj(v).unwrap().pos, p, "it stays where it is");
}

// spec: S-0607 claim 13 (revision 8, design; answers Q-1342) and S-0502 claims 9 and 17 (revision 16; scenario J27 c): a unit dropped over its cell centre by its lifter with no cell for the stop loses the point fixed at the seize and its commitment, and falls with no movement point by the direct ground reference, even with a vehicle listed in its own cell; it takes its jumpjet update every tick of its fall
#[test]
fn claim_13_no_cell_drop_over_the_centre_falls_by_the_direct_reference() {
    let (mut w, l, v) = seized(quirk(true));
    let c = fixed_cell(&w, v);
    let mut cells = Vec::new();
    for dx in -SIZE..SIZE {
        for dy in -SIZE..SIZE {
            let x = at(dx, dy);
            if x != at(0, 0) && x != c && w.cells.grid.get(x).is_some() {
                cells.push(x);
            }
        }
    }
    water(&mut w, &cells);
    // Parked after the seize's point search, before the arrival.
    spawn(&mut w, Kind::Vehicle, "VICT", 0, c);
    assert!(run_until(&mut w, 400, |w| !lift::grips(w, l, v)));
    assert!(lift::dropped(&w, v), "an arrival drop");
    let p = w.obj(v).unwrap().pos;
    assert_eq!(p.cell(), c);
    assert_eq!((p.x.rem_euclid(256), p.y.rem_euclid(256)), (128, 128));
    assert_eq!(point(&w, v), None, "the seize's point is cleared");
    assert!(!jump::moving(&w, v), "the commitment is lost");
    assert_eq!(health(&w, v), 100, "no self-destruction");
    let steps = fall_steps(&mut w, v);
    assert!(!steps.is_empty(), "it fell");
    assert!(
        steps.iter().all(|&s| !held(&w, s)),
        "no step held at the parked vehicle's clearance: {steps:?}"
    );
    assert!(
        steps.iter().all(|&(before, after, _)| after < before),
        "an update every tick of the fall: {steps:?}"
    );
}

// spec: S-0607 claims 13 and 15 (revision 8, design) and S-0502 claims 3 and 17 (revision 16, the no-cell magnetic-release safety fall): a unit dropped while hovering with no cell for the stop still takes its next update: over its cell centre its fall starts at once with no movement point; off its centre the detour gives it that centre, which lies in its own cell, as its point
#[test]
fn claim_13_no_cell_drop_while_hovering_still_falls() {
    for centred in [true, false] {
        let (mut w, l, v) = seized(quirk(true));
        flood(&mut w);
        assert!(run_until(&mut w, 400, |w| height(w, v) > 300));
        let o = w.obj_mut(v).unwrap();
        let c = o.pos.cell();
        if centred {
            o.pos.x = c.x * 256 + 128;
            o.pos.y = c.y * 256 + 128;
        } else if o.pos.x.rem_euclid(256) == 128 && o.pos.y.rem_euclid(256) == 128 {
            o.pos.x += 10;
        }
        o.jump.as_mut().unwrap().phase = Phase::Hovering;
        order_lifter(&mut w, l);
        assert!(lift::dropped(&w, v), "centred {centred}: dropped");
        let before = (w.obj(v).unwrap().pos, height(&w, v));
        if centred {
            assert_eq!(point(&w, v), None, "no movement point");
            assert!(!jump::moving(&w, v), "no commitment");
        } else {
            let d = point(&w, v).expect("the detour's point");
            assert_eq!(d.cell(), c, "its own cell");
            assert_eq!((d.x, d.y), (c.x * 256 + 128, c.y * 256 + 128));
        }
        run(&mut w, 1);
        let after = (w.obj(v).unwrap().pos, height(&w, v));
        assert_ne!(before, after, "centred {centred}: its next update ran");
        if centred {
            assert!(after.1 < before.1, "the fall started");
        }
        assert!(
            run_until(&mut w, 400, |w| !lift::dropped(w, v)),
            "centred {centred}: the fall ends"
        );
    }
}

// ---------------------------------------------------------------------------
// Claim 7: a building lifter's point

fn register(r: &mut Ruleset, name: &str) {
    let l = r.registries.entry("buildings".into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn building(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut BuildingType)) {
    let mut b = BuildingType {
        strength: Some(800),
        foundation: Some([3, 3]),
        number_impassable_rows: Some(0),
        ..Default::default()
    };
    f(&mut b);
    register(r, name);
    r.sim.buildings.insert(name.into(), b);
}

fn docking_rules(r: &mut Ruleset) {
    building(r, "PLAIN", |_| {});
    building(r, "REF", |b| b.econ.refinery = Some(true));
    building(r, "WEEDONLY", |b| b.econ.weeder = Some(true));
    building(r, "FACT", |b| b.factory = Some("vehicle".into()));
    building(r, "DEPOT1", |b| {
        b.unit_repair = Some(true);
        b.econ.docking_offsets = Some(vec![[0, 512, 0]]);
    });
    building(r, "DEPOT0", |b| {
        b.unit_repair = Some(true);
        b.econ.number_of_docks = Some(0);
        b.econ.docking_offsets = Some(vec![[0, 512, 0]]);
    });
    building(r, "DEPOT3", |b| {
        b.unit_repair = Some(true);
        b.econ.number_of_docks = Some(3);
        b.econ.docking_offsets = Some(vec![[-256, 0, 0], [256, 0, 0], [0, 512, 0]]);
    });
    building(r, "PAD1", |b| {
        b.helipad = Some(true);
        b.econ.docking_offsets = Some(vec![[512, 0, 0]]);
    });
    building(r, "BUNK", |b| b.bunker = Some(true));
    building(r, "ALL", |b| {
        b.unit_repair = Some(true);
        b.bunker = Some(true);
        b.econ.weeder = Some(true);
        b.econ.refinery = Some(true);
    });
    building(r, "REFBUNK", |b| {
        b.unit_repair = Some(true);
        b.bunker = Some(true);
        b.econ.refinery = Some(true);
    });
    building(r, "DEPOTBUNK", |b| {
        b.unit_repair = Some(true);
        b.bunker = Some(true);
        b.econ.docking_offsets = Some(vec![[0, 512, 0]]);
    });
}

fn plus(p: Pos, dx: i32, dy: i32) -> Pos {
    Pos::new(p.x + dx, p.y + dy, p.z)
}

// spec: S-0607 claim 7 and Import mapping (revision 5): only a building with the pad, repair-depot or vehicle-bunker flag gives a docking point; every other building (a refinery, a weeder, a war factory included) gives the centre of its foundation
#[test]
fn claim_7_which_buildings_give_a_docking_point() {
    let mut w = world(docking_rules);
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(-8, 0));
    for name in ["PLAIN", "REF", "WEEDONLY", "FACT"] {
        let b = spawn(&mut w, Kind::Building, name, 0, at(0, 0));
        assert_eq!(
            lift::lifter_point(&w, b, v),
            w.reference_point(b),
            "{name}: the foundation centre"
        );
        glasnost_sim::objects::remove(&mut w, b);
    }
    for (name, dx, dy) in [("DEPOT1", 0, 512), ("PAD1", 512, 0)] {
        let b = spawn(&mut w, Kind::Building, name, 0, at(0, 0));
        assert_eq!(
            lift::lifter_point(&w, b, v),
            plus(w.reference_point(b), dx, dy),
            "{name}: the sole docking displacement"
        );
        glasnost_sim::objects::remove(&mut w, b);
    }
}

// spec: S-0607 claim 7 (revision 5, the docking-place rule): a count of 0 gives the foundation centre; with several places, the place the seized unit holds through a contact with the building, else the foundation centre (never the first free place)
#[test]
fn claim_7_docking_place_rule() {
    let mut w = world(docking_rules);
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(-8, 0));
    let b = spawn(&mut w, Kind::Building, "DEPOT0", 0, at(0, 0));
    assert_eq!(
        lift::lifter_point(&w, b, v),
        w.reference_point(b),
        "count 0"
    );
    glasnost_sim::objects::remove(&mut w, b);
    let b = spawn(&mut w, Kind::Building, "DEPOT3", 0, at(0, 0));
    let centre = w.reference_point(b);
    assert_eq!(lift::lifter_point(&w, b, v), centre, "no place held");
    if let Some(o) = w.obj_mut(v) {
        o.contact = Some(b);
        o.depot.link = Some(glasnost_sim::depot::Link {
            depot: b,
            place: 2,
            tethered: true,
            arrived: false,
        });
    }
    assert_eq!(
        lift::lifter_point(&w, b, v),
        plus(centre, 0, 512),
        "the place it holds"
    );
}

// spec: S-0607 claim 7 (revision 5, precedence): the obsolete-unloading flag's fixed point (the centre of the cell (2, 1) from the top-left foundation cell) comes first, then the resource-unloading flag's foundation centre + 128 leptons along x, then the bunker's quadrant point on the seized unit's side, then the docking places
#[test]
fn claim_7_docking_point_precedence() {
    let mut w = world(docking_rules);
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(-8, 0));
    let u = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(10, 1));
    let b = spawn(&mut w, Kind::Building, "ALL", 0, at(0, 0));
    let top = w.obj(b).unwrap().pos;
    let tl = top.cell();
    assert_eq!(
        lift::lifter_point(&w, b, v),
        Pos::centre(CellXY::new(tl.x + 2, tl.y + 1), top.z),
        "the obsolete-unloading point first"
    );
    glasnost_sim::objects::remove(&mut w, b);
    let b = spawn(&mut w, Kind::Building, "REFBUNK", 0, at(0, 0));
    assert_eq!(
        lift::lifter_point(&w, b, v),
        plus(w.reference_point(b), 128, 0),
        "then the resource-unloading point"
    );
    glasnost_sim::objects::remove(&mut w, b);
    for name in ["BUNK", "DEPOTBUNK"] {
        let b = spawn(&mut w, Kind::Building, name, 0, at(0, 0));
        let (pv, pu) = (lift::lifter_point(&w, b, v), lift::lifter_point(&w, b, u));
        assert_eq!(
            pv,
            glasnost_sim::structures::bunker::docking_point(&w, b, v),
            "{name}: the quadrant point"
        );
        assert_eq!(
            pu,
            glasnost_sim::structures::bunker::docking_point(&w, b, u)
        );
        assert_ne!(pv, pu, "{name}: it changes with the side");
        glasnost_sim::objects::remove(&mut w, b);
    }
}

// ---------------------------------------------------------------------------
// Claim 4 under another named locomotor

// spec: S-0607 claims 4 and 22 (revision 5; scenario L13 part d): claim 4's harvester case is decided before the warhead's locomotor is considered: with a hovering, a driving or an unknown-identifier warhead a temporarily driving harvester whose replacement cannot end stops and queues stop, is not seized and is not put in guard
#[test]
fn claim_4_comes_first_whatever_the_warhead_names() {
    for loco in ["hover", "drive", "unknown"] {
        let mut w = world(|r| {
            r.sim.warheads.get_mut("MAG").unwrap().locomotor = Some(loco.into());
            vehicle(r, "HARV", |t| t.econ.resource_gatherer = Some(true));
        });
        let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
        let h = spawn(&mut w, Kind::Vehicle, "HARV", 1, at(6, 0));
        w.obj_mut(h).unwrap().drive = None;
        glasnost_sim::econ::release::install_temporary_drive(&mut w, h);
        let goal = w.obj(h).unwrap().pos;
        w.obj_mut(h).unwrap().drive.as_mut().unwrap().goal =
            Some(Pos::new(goal.x + 512, goal.y, goal.z));
        beam(&mut w, l, h);
        assert!(!lift::seized(&w, h), "{loco}: not seized");
        let o = w.obj(h).unwrap();
        assert!(
            o.beh.current == Some(Behaviour::Stop) || o.beh.queued == Some(Behaviour::Stop),
            "{loco}: claim 4's stop, not claim 22's guard: {:?}",
            o.beh
        );
        assert!(
            glasnost_sim::econ::release::temporary_drive(&w, h),
            "{loco}: still driving temporarily"
        );
    }
}

// spec: S-0502 claim 29 (revision 18, Q-1500 answered: a unit of any type seized by magnetic lift is registered as a tracked flying object from its first jumpjet update, a take-off) and claim 30 (Q-1501 answered: a seized unit is in the air by the same height test, r ≥ 208 leptons)
#[test]
fn a_seized_unit_is_tracked_and_in_the_air_by_height() {
    let (mut w, _l, v) = seized(|_| {});
    assert!(
        run_until(&mut w, 60, |w| jump::phase(w, v) != Some(Phase::Grounded)),
        "the seized unit takes off"
    );
    assert!(glasnost_sim::combat::flying::tracked(&w, v));
    assert!(
        !glasnost_sim::combat::weapon::in_air(&w, v),
        "low after take-off"
    );
    assert!(run_until(&mut w, 400, |w| height(w, v) >= 208));
    assert!(
        glasnost_sim::combat::weapon::in_air(&w, v),
        "at 208 leptons or more it is in the air"
    );
}

// spec: S-0607 claim 7 (revision 10: when the seize's search finds no cell the unit has no movement commitment and stays grounded, with no take-off registration) and S-0502 claim 29 (revision 19, Seized units)
#[test]
fn a_seized_unit_with_no_cell_stays_grounded_and_untracked() {
    let mut w = world(|_| {});
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(2, 0));
    // Rock everywhere but the lifter's cell: the jumpjet search finds no cell.
    for dx in -45..45 {
        for dy in -45..45 {
            if (dx, dy) != (0, 0)
                && let Some(c) = w.cells.grid.get_mut(at(dx, dy))
            {
                c.land = glasnost_map::terrain::land::ROCK;
            }
        }
    }
    for dx in -45..45 {
        for dy in -45..45 {
            if w.cells.cell(at(dx, dy)).is_some() {
                glasnost_sim::regions::cell_changed(&mut w, at(dx, dy));
            }
        }
    }
    glasnost_sim::regions::ensure(&mut w);
    beam(&mut w, l, v);
    assert!(glasnost_sim::lift::seized(&w, v));
    for _ in 0..30 {
        glasnost_sim::testkit::run(&mut w, 1);
        assert_eq!(jump::phase(&w, v), Some(Phase::Grounded), "no take-off");
        assert!(
            !glasnost_sim::combat::flying::tracked(&w, v),
            "not registered"
        );
    }
}
