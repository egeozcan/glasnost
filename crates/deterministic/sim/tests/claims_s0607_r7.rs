//! Claim tests for S-0607 revision 7 (M7a step 1p): the drop-pod row of
//! claim 12, the ends of a stand-in's grip (the teleporting jump, a jump
//! that does not happen, a landing search with no cell), the occupancy of a
//! unit seized with a ground stand-in (`quirks.lift_standin_unmarked_cell`)
//! and a landed aircraft seized with a ground stand-in. Synthetic data only.

#[path = "lift_common/mod.rs"]
mod lift_common;

use glasnost_map::CellXY;
use glasnost_rules::sim::{Locomotor as RawLocomotor, WeaponType};
use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::lift::{self, StandIn};
use glasnost_sim::occupancy::{self, Outcome};
use glasnost_sim::rules::Kind;
use glasnost_sim::save;
use glasnost_sim::testkit;
use glasnost_sim::world::{Layer, ObjectId, Pos, Target, World};
use lift_common::*;

/// A ruleset tweak: the seizing warhead names `loco`, the victims carry a
/// gun, CHRONO is a teleporting victim, CRUSHER crushes, SOFT is
/// crushable, PLANE is a flying aircraft type, JLIFTER seizes with the
/// jumpjet, and the two stand-in quirks are `stays` and `unmarked`.
fn rules(
    loco: &'static str,
    stays: bool,
    unmarked: bool,
) -> impl FnOnce(&mut glasnost_rules::schema::Ruleset) {
    move |r| {
        r.sim.warheads.get_mut("MAG").unwrap().locomotor = Some(loco.to_owned());
        r.sim.vehicles.get_mut("VICT").unwrap().primary = Some("Gun".into());
        vehicle(r, "CHRONO", |t| t.locomotor = Some(RawLocomotor::Teleport));
        vehicle(r, "CRUSHER", |t| {
            t.crusher = Some(true);
            t.primary = Some("Gun".into());
        });
        vehicle(r, "SOFT", |t| t.crushable = Some(true));
        let mut jh = r.sim.warheads["MAG"].clone();
        jh.locomotor = Some("jumpjet".into());
        r.sim.warheads.insert("MAGJ".into(), jh);
        let mut jw: WeaponType = r.sim.weapons["MagBeam"].clone();
        jw.warhead = Some("MAGJ".into());
        r.sim.weapons.insert("MagBeamJ".into(), jw);
        vehicle(r, "JLIFTER", |t| {
            t.strength = Some(1000);
            t.primary = Some("MagBeamJ".into());
        });
        let mut p = r.sim.vehicles["VICT"].clone();
        p.locomotor = Some(RawLocomotor::Fly);
        r.registries
            .entry("aircraft".into())
            .or_default()
            .push("PLANE".into());
        r.sim.aircraft.insert("PLANE".into(), p);
        r.globals.quirks.lift_standin_stays_seized = stays;
        r.globals.quirks.lift_standin_unmarked_cell = unmarked;
    }
}

/// Fires the lifter's beam once at `target` and runs until it has
/// detonated, without giving the lifter its target again afterwards.
fn beam_once(w: &mut World, lifter: ObjectId, target: ObjectId) {
    let t0 = w.tick;
    attack(w, lifter, target);
    assert!(glasnost_sim::combat::fire::shot(
        w,
        lifter,
        0,
        Target::Object(target)
    ));
    while !w.projectiles.is_empty() {
        testkit::run(w, 1);
        assert!(w.tick < t0 + 40, "the beam did not detonate");
    }
}

/// LIFTER (house 0) at (0, 0) and a `name` victim (house 1) at (6, 0),
/// hit once by the beam.
fn scene(
    loco: &'static str,
    stays: bool,
    unmarked: bool,
    kind: Kind,
    name: &str,
) -> (World, ObjectId, ObjectId) {
    let mut w = world(rules(loco, stays, unmarked));
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, kind, name, 1, at(6, 0));
    beam_once(&mut w, l, v);
    assert!(lift::seized(&w, v), "{loco}: seized");
    (w, l, v)
}

fn vehicles_at(w: &World, c: CellXY) -> u8 {
    w.cells.occ(c, Layer::Ground).map_or(0, |o| o.vehicles)
}

fn others_at(w: &World, c: CellXY) -> u8 {
    w.cells.occ(c, Layer::Ground).map_or(0, |o| o.others)
}

/// The sum of a record count over the cells near the middle of the map.
fn total(w: &World, f: impl Fn(&World, CellXY) -> u8) -> u32 {
    let mut n = 0;
    for dx in -12..=12 {
        for dy in -12..=12 {
            n += u32::from(f(w, at(dx, dy)));
        }
    }
    n
}

/// Whether the firer's wave has ended (or is gone).
fn wave_ended(w: &World, firer: ObjectId) -> bool {
    lift::wave_of(w, firer).is_none_or(|id| w.waves[&id].ended)
}

// ---------------------------------------------------------------------------
// Claim 12 and the drop-pod stand-in (Q-1337 answered)

// spec: S-0607 claim 12 (revision 7, the row for a gripped unit's drop-pod stand-in ending: the lifter's target is cleared) and claim 23 (our engine ends the drop-pod stand-in at once with the original's touchdown result: the lifter is no longer gripping, so claim 11's refusals stop at once, and its wave ends at its next update, claim 21)
#[test]
fn claim_12_drop_pod_end_clears_the_lifter_target() {
    let (mut w, l, v) = scene("drop_pod", true, true, Kind::Vehicle, "VICT");
    assert_eq!(lift::stand_in(&w, v), Some(StandIn::DropPod));
    assert!(!lift::grips(&w, l, v), "the grip has ended");
    assert_eq!(
        w.obj(l).unwrap().target,
        None,
        "the lifter's target is cleared"
    );
    assert!(lift::seized(&w, v), "the unit stays seized (claim 24)");
    assert_eq!(
        lift::firing_answer(&w, l, true, Some(Target::Cell(at(3, 3)))),
        None,
        "claim 11's refusal of other targets stops: the lifter grips nothing"
    );
    run(&mut w, 1);
    assert!(wave_ended(&w, l), "the wave ends at its next update");
}

// spec: S-0607 claim 23 (revision 7, drop-pod, design): our engine reproduces none of the original's touchdown target drop: an object of another house that targets the unit (one the original would make drop it) keeps it as its target
#[test]
fn claim_23_drop_pod_other_attackers_keep_their_target() {
    let mut w = world(rules("drop_pod", true, true));
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    let other = spawn(&mut w, Kind::Vehicle, "VICT", 0, at(6, 4));
    w.obj_mut(other).unwrap().target = Some(Target::Object(v));
    attack(&mut w, l, v);
    assert!(glasnost_sim::combat::fire::shot(
        &mut w,
        l,
        0,
        Target::Object(v)
    ));
    assert!(run_until(&mut w, 40, |w| lift::seized(w, v)));
    assert!(lift::seized(&w, v));
    assert_eq!(w.obj(l).unwrap().target, None);
    assert_eq!(w.obj(other).unwrap().target, Some(Target::Object(v)));
}

// spec: S-0607 claim 12 (revision 7: a lifter whose grip ended without clearing its target may fire at the same unit again; the unit is still seized, so the hit seizes nothing, claim 3, and does no damage, claim 1)
#[test]
fn claim_12_a_second_hit_on_a_seized_unit_seizes_nothing() {
    let (mut w, l, v) = scene("walk", true, true, Kind::Vehicle, "VICT");
    glasnost_sim::econ::assign_destination(&mut w, v, None);
    assert!(!lift::grips(&w, l, v), "the grip ends");
    assert_eq!(w.obj(l).unwrap().target, Some(Target::Object(v)), "kept");
    run(&mut w, 25);
    beam_once(&mut w, l, v);
    assert!(lift::seized(&w, v));
    assert!(!lift::grips(&w, l, v), "seized nothing");
    assert_eq!(health(&w, v), 100, "no damage");
}

// ---------------------------------------------------------------------------
// The ends of a stand-in's grip (Q-1340 answered)

// spec: S-0607 claim 23 (revision 7, the teleporting jump): one jump on the movement update after the seize; the grip ends in the jump tick with the unit set down at its landing point, the jump clears the lifter's target in that tick (S-0207 claim 20 e), its wave begins fading at its next update, and no further jump follows
#[test]
fn claim_23_teleport_jump_ends_the_grip_and_clears_the_target() {
    let (mut w, l, v) = scene("teleport", true, true, Kind::Vehicle, "VICT");
    let start = w.obj(v).unwrap().pos;
    assert!(lift::grips(&w, l, v));
    assert!(
        run_until(&mut w, 5, |w| w.obj(v).unwrap().pos != start),
        "it jumps"
    );
    let landed = w.obj(v).unwrap().pos;
    assert!(!lift::grips(&w, l, v), "the grip ends in the jump tick");
    assert_eq!(
        w.obj(l).unwrap().target,
        None,
        "the jump clears the lifter's target"
    );
    assert_eq!(w.obj(v).unwrap().dest, None, "set down: no destination");
    assert!(lift::seized(&w, v));
    run(&mut w, 1);
    assert!(wave_ended(&w, l), "the wave fades from its next update");
    run(&mut w, 60);
    assert_eq!(w.obj(v).unwrap().pos, landed, "no further jump");
}

/// Fills every neighbour of `c` but `free` with house-0 vehicles.
fn ring(w: &mut World, c: CellXY, free: &[CellXY]) {
    for dx in -1..=1 {
        for dy in -1..=1 {
            let n = CellXY {
                x: c.x + dx,
                y: c.y + dy,
            };
            if (dx, dy) != (0, 0) && !free.contains(&n) {
                spawn(w, Kind::Vehicle, "SOFT", 0, n);
            }
        }
    }
}

// spec: S-0607 claim 23 (revision 7, no jump: when the landing point equals the unit's own position, S-0207 claim 20 removes the destination at its first movement update without a jump; the grip ends in that update and the lifter keeps its target)
#[test]
fn claim_23_teleport_without_a_jump_keeps_the_target() {
    let mut w = world(rules("teleport", true, true));
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(1, 0));
    ring(&mut w, at(0, 0), &[at(1, 0)]);
    beam_once(&mut w, l, v);
    let p = w.obj(v).unwrap().pos;
    let landing = w.obj(v).unwrap().teleport.as_ref().unwrap().landing;
    assert_eq!(landing, Some(p), "the landing point is its own position");
    run(&mut w, 2);
    assert!(!lift::grips(&w, l, v), "the grip ends");
    assert_eq!(w.obj(v).unwrap().pos, p, "no jump");
    assert_eq!(w.obj(l).unwrap().target, Some(Target::Object(v)), "kept");
    assert!(lift::seized(&w, v));
}

/// LIFTER on a one-cell island at (0, 0) and a teleporting-stand-in victim
/// on other land at (3, 0), seized: the landing search finds no cell.
fn island_scene() -> (World, ObjectId, ObjectId) {
    let mut w = world(rules("teleport", true, true));
    let mut cells = Vec::new();
    for dx in -2..=2 {
        for dy in -2..=2 {
            if (dx, dy) != (0, 0) {
                cells.push(at(dx, dy));
            }
        }
    }
    water(&mut w, &cells);
    for &c in &cells {
        glasnost_sim::regions::cell_changed(&mut w, c);
    }
    glasnost_sim::regions::ensure(&mut w);
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(3, 0));
    beam_once(&mut w, l, v);
    (w, l, v)
}

// spec: S-0607 claim 23 (revision 7, no cell: the landing search finds no cell, so the destination is removed while the seize assigns it, before the seized state begins; that removal does not end the grip: the unit stays where it is with no destination, never jumps, and the grip lasts until another event of claim 12) and S-0207 claim 20 (b) (revision 41)
#[test]
fn claim_23_teleport_no_cell_keeps_the_grip() {
    let (mut w, l, v) = island_scene();
    let p = w.obj(v).unwrap().pos;
    assert!(lift::seized(&w, v));
    assert!(lift::grips(&w, l, v), "the grip lasts");
    assert_eq!(w.obj(v).unwrap().dest, None, "no destination");
    run(&mut w, 100);
    assert!(lift::grips(&w, l, v), "still gripped");
    assert_eq!(w.obj(v).unwrap().pos, p, "never jumps");
    assert_eq!(w.obj(v).unwrap().dest, None);
    assert_eq!(
        lift::firing_answer(&w, l, true, Some(Target::Cell(at(3, 3)))),
        Some(Answer::Rearming),
        "the gripping lifter refuses other targets (claim 11)"
    );
    // spec: S-0607 claim 9 (revision 33: a gripped vehicle that has no destination gets none again from such a rule: the assignment leaves its destination unchanged, so it is not the exception, and the grip holds, whatever its stand-in)
    glasnost_sim::econ::assign_destination(&mut w, v, None);
    run(&mut w, 2);
    assert!(
        lift::grips(&w, l, v),
        "a redundant assignment: the grip holds"
    );
    assert_eq!(w.obj(v).unwrap().pos, p, "and it still never jumps");
}

// spec: S-0607 claims 22 to 24 (revision 7: the grip kept with no destination from the seize survives a save/load round trip with the same state hash)
#[test]
fn claim_23_no_cell_grip_save_round_trip() {
    let (mut w, l, v) = island_scene();
    run(&mut w, 3);
    assert!(w.obj(v).unwrap().lift.grip_without_dest);
    let h = save::state_hash(&w);
    let mut x = save::load(&save::save(&w), w.rules_arc(), testkit::catalogue()).unwrap();
    assert_eq!(save::state_hash(&x), h);
    run(&mut x, 20);
    assert!(lift::grips(&x, l, v), "the loaded grip lasts too");
}

// ---------------------------------------------------------------------------
// Occupancy (Q-1338 confirmed; quirks.lift_standin_unmarked_cell)

// spec: S-0607 claim 23 and quirks.lift_standin_unmarked_cell (on: the seize gives up the vehicle-present mark and installing the driving, walking or ship stand-in marks none; off: the unit keeps its own cell's mark at the seize, as a vehicle at rest does) and S-0209 claim 1 (revision 43)
#[test]
fn quirk_lift_standin_unmarked_cell_both_positions() {
    for unmarked in [true, false] {
        for loco in ["drive", "walk", "ship"] {
            let (w, l, v) = scene(loco, true, unmarked, Kind::Vehicle, "VICT");
            assert!(lift::grips(&w, l, v));
            let c = w.obj(v).unwrap().pos.cell();
            assert_eq!(
                vehicles_at(&w, c),
                u8::from(!unmarked),
                "{loco}, unmarked {unmarked}: its own cell's mark"
            );
        }
    }
}

// spec: S-0607 claim 23 (revision 7, teleporting: fixing the landing point at the seize moves the mark at once to the landing cell; the unit's own cell stays unmarked until the jump, and the landing cell stays marked after it; the teleporting stand-in is the same in both positions of quirks.lift_standin_unmarked_cell). The seize has already given up the own mark, so the landing point clears nothing else: another mark in the unit's cell survives
#[test]
fn claim_23_teleport_marks_the_landing_cell() {
    for unmarked in [true, false] {
        let mut w = world(rules("teleport", true, unmarked));
        let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
        let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
        let c = w.obj(v).unwrap().pos.cell();
        w.cells.claim_vehicle(c, Layer::Ground, true); // another, foreign mark
        let base = total(&w, vehicles_at);
        beam_once(&mut w, l, v);
        let landing = w
            .obj(v)
            .unwrap()
            .teleport
            .as_ref()
            .unwrap()
            .landing
            .unwrap();
        assert_eq!(
            vehicles_at(&w, c),
            1,
            "unmarked {unmarked}: only the foreign mark"
        );
        assert_eq!(
            vehicles_at(&w, landing.cell()),
            1,
            "the landing cell is marked"
        );
        assert_eq!(total(&w, vehicles_at), base, "the mark moved");
        assert!(run_until(&mut w, 5, |w| w.obj(v).unwrap().pos == landing));
        assert_eq!(
            vehicles_at(&w, landing.cell()),
            1,
            "still marked after the jump"
        );
        assert_eq!(vehicles_at(&w, c), 1);
    }
}

// spec: S-0607 claim 23 (revision 7, occupancy: a unit that never moves under its stand-in stands unmarked, also after its grip ends; when it leaves play it clears no mark it does not hold, so another mark in its cell survives)
#[test]
fn claim_23_unmoved_stand_in_clears_no_foreign_mark() {
    for loco in ["walk", "drive"] {
        let mut w = world(rules(loco, true, true));
        let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
        let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
        beam_once(&mut w, l, v);
        let c = w.obj(v).unwrap().pos.cell();
        glasnost_sim::objects::remove(&mut w, l);
        run(&mut w, 1);
        assert!(lift::seized(&w, v));
        assert_eq!(vehicles_at(&w, c), 0, "{loco}: unmarked after the grip");
        w.cells.claim_vehicle(c, Layer::Ground, true);
        glasnost_sim::objects::remove(&mut w, v);
        run(&mut w, 1);
        assert_eq!(vehicles_at(&w, c), 1, "{loco}: the foreign mark survives");
    }
}

// spec: S-0607 claim 23 (revision 7, driving: the unit marks and clears cells from the start of its first step as S-0209 claim 1 says, and a unit that has moved and stops keeps the mark of its last cell, as any vehicle at rest does)
#[test]
fn claim_23_driven_stand_in_keeps_its_last_mark() {
    let (mut w, l, v) = scene("drive", true, true, Kind::Vehicle, "VICT");
    let start = w.obj(v).unwrap().pos.cell();
    assert!(run_until(&mut w, 600, |w| !lift::grips(w, l, v)));
    run(&mut w, 30);
    let c = w.obj(v).unwrap().pos.cell();
    assert_ne!(c, start, "it drove");
    assert_eq!(vehicles_at(&w, c), 1, "its last cell is marked");
    assert_eq!(vehicles_at(&w, start), 0);
}

// spec: S-0607 claim 23 (revision 7, what the missing mark changes: the unit stays listed and the listed occupants decide first; while it has the lifter as its destination it counts as moving and, before its first step, is between steps, so an allied mover meets moving blocker, row VP22; once its destination is gone an allied mover meets stationary friendly, row VP20) and S-0209 claim 5, in both positions of quirks.lift_standin_unmarked_cell
#[test]
fn claim_23_allied_entry_outcomes() {
    for unmarked in [true, false] {
        for loco in ["walk", "drive", "ship"] {
            let (mut w, l, v) = scene(loco, true, unmarked, Kind::Vehicle, "VICT");
            let c = w.obj(v).unwrap().pos.cell();
            let other = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 3));
            let r = occupancy::reference_level(&w, other);
            assert_eq!(
                occupancy::vehicle_outcome(&w, other, c, r),
                Outcome::MovingBlocker,
                "{loco}, unmarked {unmarked}: while gripped"
            );
            glasnost_sim::econ::assign_destination(&mut w, v, None);
            assert!(!lift::grips(&w, l, v));
            assert_eq!(
                occupancy::vehicle_outcome(&w, other, c, r),
                Outcome::StationaryFriendly,
                "{loco}, unmarked {unmarked}: after the grip"
            );
        }
    }
}

// spec: S-0607 claim 23 (revision 7: a crushable seized unit gives clear to a crusher with or without the mark, rows VR3 and VR2 of S-0209 claim 5; an enemy mover meets the seized unit's enemy row)
#[test]
fn claim_23_enemy_entry_outcomes() {
    for unmarked in [true, false] {
        let (mut w, _, v) = scene("walk", true, unmarked, Kind::Vehicle, "SOFT");
        let c = w.obj(v).unwrap().pos.cell();
        let crusher = spawn(&mut w, Kind::Vehicle, "CRUSHER", 0, at(6, 3));
        let r = occupancy::reference_level(&w, crusher);
        assert_eq!(
            occupancy::vehicle_outcome(&w, crusher, c, r),
            Outcome::Clear,
            "unmarked {unmarked}: a crusher may enter"
        );
        let (mut w, _, v) = scene("walk", true, unmarked, Kind::Vehicle, "VICT");
        let c = w.obj(v).unwrap().pos.cell();
        let enemy = spawn(&mut w, Kind::Vehicle, "VICT", 0, at(6, 3));
        let r = occupancy::reference_level(&w, enemy);
        assert_eq!(
            occupancy::vehicle_outcome(&w, enemy, c, r),
            Outcome::EnemyObstacle,
            "unmarked {unmarked}: the enemy row"
        );
    }
}

// spec: S-0607 claim 23 (revision 7: the missing mark is seen by rules that read the occupancy record alone, such as the nearby-cell search of S-0212 claim 25: claim 7's movement-point search for another seize can choose the unmoved seized unit's cell; with quirks.lift_standin_unmarked_cell off the unit keeps its mark and the cell is refused)
#[test]
fn claim_7_search_can_choose_an_unmoved_seized_cell() {
    for unmarked in [true, false] {
        let (mut w, _, v) = scene("walk", true, unmarked, Kind::Vehicle, "VICT");
        let c = w.obj(v).unwrap().pos.cell();
        let l2_cell = CellXY { x: c.x + 1, y: c.y };
        let l2 = spawn(&mut w, Kind::Vehicle, "JLIFTER", 0, l2_cell);
        ring(&mut w, l2_cell, &[c]);
        let third = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 6));
        beam_once(&mut w, l2, third);
        assert!(lift::grips(&w, l2, third));
        let chosen = match w.obj(third).unwrap().dest {
            Some(Target::Cell(x)) => x,
            d => panic!("unexpected destination {d:?}"),
        };
        assert_eq!(chosen == c, unmarked, "unmarked {unmarked}: {chosen:?}");
    }
}

// ---------------------------------------------------------------------------
// A landed aircraft (Q-1339 answered)

// spec: S-0607 claim 23 (revision 7, a landed aircraft with the teleporting stand-in: S-0207 claim 20 covers aircraft; its entry outcome is clear, S-0209 claim 14, so its landing point is the centre of the cell of the lifter's point even where the lifter stands; it lands at terrain height and the grip ends at the jump) and claim 24 (quirks.lift_standin_stays_seized on: it never takes off again)
#[test]
fn claim_23_landed_aircraft_teleports() {
    let (mut w, l, p) = scene("teleport", true, true, Kind::Aircraft, "PLANE");
    assert_eq!(lift::stand_in(&w, p), Some(StandIn::Teleport));
    assert!(lift::grips(&w, l, p));
    let lc = w.obj(l).unwrap().pos.cell();
    assert!(
        run_until(&mut w, 5, |w| !lift::grips(w, l, p)),
        "the grip ends"
    );
    let o = w.obj(p).unwrap();
    assert_eq!(o.pos.cell(), lc, "in the cell of the lifter's point");
    assert_eq!(o.pos, Pos::centre(lc, w.surface_at(o.pos.x, o.pos.y)));
    assert_eq!(height(&w, p), 0, "at terrain height");
    assert!(lift::seized(&w, p));
    assert_eq!(others_at(&w, lc), 1, "its object-present reservation");
    let c = w.obj(p).unwrap().pos.cell();
    for _ in 0..5 {
        order_move(&mut w, p, CellXY { x: c.x + 4, y: c.y });
        run(&mut w, 40);
    }
    assert_eq!(height(&w, p), 0, "never takes off");
    assert!(lift::seized(&w, p));
    glasnost_sim::objects::remove(&mut w, p);
    run(&mut w, 1);
    assert_eq!(total(&w, others_at), 0, "no object-present mark left");
}

// spec: S-0607 claim 23 (revision 7, taking off) and claim 24 (quirks.lift_standin_stays_seized off: a landed aircraft seized with the teleporting stand-in is restored and can take off on a later order; its marks are its own again) and claim 20 (revision 14: the restoration test at the end of each of its turns restores it with no order)
#[test]
fn claim_23_landed_aircraft_restored_takes_off() {
    let (mut w, l, p) = scene("teleport", false, true, Kind::Aircraft, "PLANE");
    assert!(run_until(&mut w, 5, |w| !lift::grips(w, l, p)));
    glasnost_sim::objects::remove(&mut w, l);
    assert!(run_until(&mut w, 2, |w| !lift::seized(w, p)), "restored");
    assert!(w.obj(p).unwrap().lift.own.is_none());
    assert!(w.obj(p).unwrap().teleport.is_none(), "its own (none)");
    assert!(total(&w, others_at) <= 1, "at most its own mark");
    // Past the start-of-game order block (S-0213 claim 1), which refuses the
    // order otherwise.
    let through = w.orders_blocked_through;
    assert!(run_until(&mut w, 10, |w| w.tick > through));
    let c = w.obj(p).unwrap().pos.cell();
    assert!(order_move(&mut w, p, CellXY { x: c.x + 6, y: c.y }));
    assert!(run_until(&mut w, 200, |w| height(w, p) > 0), "it takes off");
    assert!(
        run_until(&mut w, 400, |w| total(w, others_at) == 0),
        "no mark left in flight"
    );
}

// spec: S-0607 claim 23 (revision 7, a landed aircraft with the driving, walking or ship stand-in, design: our engine keeps it where it stands, it does not move or turn under that stand-in, and its grip ends only by a lifter-side event of claim 12)
#[test]
fn claim_23_landed_aircraft_other_stand_ins_stay() {
    for loco in ["drive", "walk", "ship"] {
        let (mut w, l, p) = scene(loco, true, true, Kind::Aircraft, "PLANE");
        let pos = w.obj(p).unwrap().pos;
        let f = w.obj(p).unwrap().body.value(w.tick);
        run(&mut w, 80);
        assert!(lift::grips(&w, l, p), "{loco}: the grip lasts");
        assert_eq!(w.obj(p).unwrap().pos, pos, "{loco}");
        assert_eq!(w.obj(p).unwrap().body.value(w.tick), f, "{loco}");
        assert_eq!(height(&w, p), 0, "{loco}");
        assert_eq!(
            check::check(&w, p, Some(Target::Object(l)), 0, false),
            Answer::Illegal,
            "{loco}: claim 10"
        );
    }
}

// spec: S-0607 claim 23 (revision 7, a landed aircraft with the drop-pod stand-in: as for any unit, the stand-in ends at once, the grip ends with the lifter's target cleared and its own object-present mark is back)
#[test]
fn claim_23_landed_aircraft_drop_pod() {
    let (w, l, p) = scene("drop_pod", true, true, Kind::Aircraft, "PLANE");
    assert!(!lift::grips(&w, l, p));
    assert_eq!(w.obj(l).unwrap().target, None);
    assert!(lift::seized(&w, p));
    assert_eq!(others_at(&w, w.obj(p).unwrap().pos.cell()), 1);
}

// spec: S-0607 claim 23 (revision 7, driving at an exact point: S-0207 claim 26's guard row removes the destination when the unit, at rest in guard, stands exactly on its locomotor destination point in all three coordinates; the grip ends and the lifter keeps its target). The rare layout is made by placing the unit on the point directly.
#[test]
fn claim_23_drive_exact_point_ends_the_grip() {
    let (mut w, l, v) = scene("drive", true, true, Kind::Vehicle, "VICT");
    let goal = w.obj(v).unwrap().drive.as_ref().unwrap().goal.unwrap();
    // Move the lifter aside without an order (an order would end the grip,
    // claim 12) and put the unit on its point.
    let lc = w.obj(l).unwrap().pos.cell();
    w.obj_mut(l).unwrap().pos = Pos::centre(
        CellXY {
            x: lc.x,
            y: lc.y - 3,
        },
        0,
    );
    let vc = w.obj(v).unwrap().pos.cell();
    w.cells.leave(vc, Layer::Ground, v);
    w.obj_mut(v).unwrap().pos = goal;
    w.cells.enter(goal.cell(), Layer::Ground, v, false);
    assert!(lift::grips(&w, l, v));
    assert!(
        run_until(&mut w, 5, |w| !lift::grips(w, l, v)),
        "the grip ends"
    );
    assert_eq!(w.obj(v).unwrap().pos, goal);
    assert_eq!(w.obj(v).unwrap().dest, None);
    assert_eq!(w.obj(l).unwrap().target, Some(Target::Object(v)), "kept");
    assert!(lift::seized(&w, v));
}
