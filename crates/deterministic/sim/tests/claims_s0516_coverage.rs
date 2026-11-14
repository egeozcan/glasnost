//! Claim tests of S-0516 (cloak and stealth), fields and coverage: claims
//! 19 to 23 and 40, on synthetic rules (every name and number is invented;
//! see `cloak_common`).

mod cloak_common;

use std::collections::BTreeSet;

use cloak_common::*;
use glasnost_sim::cloak::{self, CloakState, FieldPhase};
use glasnost_sim::objects;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, ObjectId, Pos, World};

fn radius(w: &World, g: ObjectId) -> i32 {
    w.obj(g).unwrap().cloak.field.map_or(0, |f| f.radius)
}

/// The cells of house `h`'s cloak field, as offsets from `at(0, 0)`.
fn field_offsets(w: &World, h: HouseId) -> BTreeSet<(i32, i32)> {
    let o = at(0, 0);
    w.cloak
        .fields
        .iter()
        .filter(|(hh, _)| *hh == h.0)
        .map(|(_, c)| (c.x - o.x, c.y - o.y))
        .collect()
}

/// Places a building switched off, so that it is not in operation at its
/// first placement.
fn place_off(w: &mut World, name: &str, h: HouseId, dx: i32, dy: i32) -> ObjectId {
    let ti = w.rules().index_of(Kind::Building, name).unwrap();
    let id = objects::create(w, Kind::Building, ti, Some(h));
    w.obj_mut(id).unwrap().econ.building.switched_off = true;
    let p = Pos::centre(at(dx, dy), 0);
    let z = w.surface_at(p.x, p.y);
    objects::place(w, id, Pos::new(p.x, p.y, z), 0);
    id
}

// spec: S-0516 claim 19 and F5 (the midpoint half-width table; r = 20 is exactly dx² + dy² ≤ r² + r, r = 1 differs)
#[test]
fn claim_19_field_shape() {
    let disc = |r: i32| -> BTreeSet<(i32, i32)> {
        let mut s = BTreeSet::new();
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r + r {
                    s.insert((dx, dy));
                }
            }
        }
        s
    };
    assert_eq!(cloak::field_circle(20), disc(20));
    // r = 1: one step (0, 1), so the plus shape of 5 cells, not the 9 of
    // the disc.
    let plus: BTreeSet<_> = [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)].into();
    assert_eq!(cloak::field_circle(1), plus);
    assert_ne!(cloak::field_circle(1), disc(1));
    // A grown generator covers exactly the circle of its radius.
    let r = ruleset();
    let mut w = game(&r);
    spawn(&mut w, "GENNY", ALPHA, 0, 0);
    run(&mut w, 10);
    assert_eq!(field_offsets(&w, ALPHA), cloak::field_circle(GEN_RADIUS));
}

// spec: S-0516 claim 20 (growth one ring per tick to the radius; shrinking one ring per tick, cells leaving at once, the update reaching 0 stopping after the generator's own centre; removal and capture vanish at once, capture regrows for the new owner; a field covers its owner only)
#[test]
fn claim_20_growth_shrink_ownership() {
    let r = ruleset();
    let mut w = game(&r);
    let g = spawn(&mut w, "GENNY", ALPHA, 0, 0);
    let mut radii = Vec::new();
    for _ in 0..5 {
        run(&mut w, 1);
        radii.push(radius(&w, g));
    }
    assert_eq!(radii, vec![1, 2, 3, 3, 3]);
    switch_off(&mut w, g);
    let mut radii = Vec::new();
    for _ in 0..2 {
        run(&mut w, 1);
        radii.push(radius(&w, g));
        assert_eq!(
            field_offsets(&w, ALPHA),
            cloak::field_circle(*radii.last().unwrap())
        );
    }
    for _ in 0..2 {
        run(&mut w, 1);
        radii.push(radius(&w, g));
    }
    assert_eq!(radii, vec![2, 1, 0, 0]);
    // The update reaching 0 stops after clearing the generator's own
    // centre (in Y, then X order), and the next one does not retry.
    let rest: BTreeSet<_> = [(1, 0), (0, 1)].into();
    assert_eq!(field_offsets(&w, ALPHA), rest);
    // Removal.
    let mut w = game(&r);
    let g = spawn(&mut w, "GENNY", ALPHA, 0, 0);
    run(&mut w, 5);
    objects::remove(&mut w, g);
    assert!(field_offsets(&w, ALPHA).is_empty());
    // Capture.
    let mut w = game(&r);
    let g = spawn(&mut w, "GENNY", ALPHA, 0, 0);
    run(&mut w, 5);
    change_owner(&mut w, g, BETA);
    assert!(field_offsets(&w, ALPHA).is_empty());
    assert!(field_offsets(&w, BETA).is_empty());
    run(&mut w, 5);
    assert_eq!(field_offsets(&w, BETA), cloak::field_circle(GEN_RADIUS));
    // Owner only, allies not.
    let mut w = game(&r);
    ally(&mut w, ALPHA, BETA);
    ally(&mut w, BETA, ALPHA);
    spawn(&mut w, "GENNY", ALPHA, 0, 0);
    let friend = spawn(&mut w, "PLAIN", BETA, 1, 0);
    let own = spawn(&mut w, "PLAIN", ALPHA, -1, 0);
    run(&mut w, 30);
    assert_eq!(state(&w, own), CloakState::Cloaked);
    assert_eq!(state(&w, friend), CloakState::Uncloaked);
}

// spec: S-0516 claim 20 and quirks.cloak_field_overlap_loss (a shrinking generator's shared cells leave the field; at an uninterrupted radius 0 a powered full generator with ΔX² + ΔY² < 4(r + 2)² restarts from 0)
#[test]
fn claim_20_overlap_and_restart() {
    let mut r = ruleset();
    // Without building interruptions, so the collapse reaching 0 is
    // uninterrupted.
    r.globals.quirks.cloak_field_building_interrupt = false;
    let mut w = game(&r);
    let a = spawn(&mut w, "GENNY", ALPHA, 0, 0);
    let b = spawn(&mut w, "GENNY", ALPHA, 2, 0);
    let far = spawn(&mut w, "GENNY", ALPHA, -10, 0);
    run(&mut w, 5);
    assert!(field_at(&w, ALPHA, 1, 0));
    switch_off(&mut w, a);
    run(&mut w, 3);
    assert_eq!(radius(&w, a), 0);
    // The shared cell (−1, 0) (covered by b at distance 3) left with a.
    assert!(!field_at(&w, ALPHA, -1, 0));
    // b (distance 2 < 10) restarted from 0; far (distance 10) did not.
    assert!(radius(&w, b) < GEN_RADIUS);
    assert_eq!(radius(&w, far), GEN_RADIUS);
    run(&mut w, 5);
    assert_eq!(radius(&w, b), GEN_RADIUS);
}

// spec: S-0516 claim 21 (a signed count per house and cell, sensed above 0; coverage counts for the sensor's owner only)
#[test]
fn claim_21_sensor_counts() {
    let r = ruleset();
    let mut w = game(&r);
    ally(&mut w, ALPHA, BETA);
    ally(&mut w, BETA, ALPHA);
    spawn(&mut w, "SEEKER", ALPHA, 0, 0);
    spawn(&mut w, "SEEKER", ALPHA, 1, 0);
    spawn(&mut w, "SEEKER", BETA, 0, 6);
    assert_eq!(count_at(&w, ALPHA, 0, 0), 2);
    assert!(sensed_at(&w, ALPHA, 0, 0));
    assert!(sensed_at(&w, BETA, 0, 6));
    assert!(!sensed_at(&w, ALPHA, 0, 6), "an ally's sensor");
    assert!(!sensed_at(&w, BETA, 0, 0));
}

// spec: S-0516 claim 22 (a mobile sensor's disc of its sensors-sight: added on entry, moved with the cell, moved to a new owner, removed on leaving, floored at 0)
#[test]
fn claim_22_mobile_sensors() {
    let r = ruleset();
    let mut w = game(&r);
    let s = spawn(&mut w, "SEEKER", ALPHA, 0, 0);
    let want: BTreeSet<(i32, i32)> = cloak::sensor_disc(SEEKER_SIGHT).into_iter().collect();
    let got = |w: &World, h: HouseId| -> BTreeSet<(i32, i32)> {
        let o = at(0, 0);
        w.cloak
            .sensors
            .iter()
            .filter(|((hh, _), n)| *hh == h.0 && **n > 0)
            .map(|((_, c), _)| (c.x - o.x, c.y - o.y))
            .collect()
    };
    assert_eq!(got(&w, ALPHA), want);
    // dx² + dy² < R²: the cells at distance R along an axis are out.
    assert!(
        sensed_at(&w, ALPHA, 2, 0) && !sensed_at(&w, ALPHA, 3, 0) && !sensed_at(&w, ALPHA, -3, 0)
    );
    change_owner(&mut w, s, BETA);
    assert!(got(&w, ALPHA).is_empty());
    assert_eq!(got(&w, BETA), want);
    move_to(&mut w, s, at(5, 0));
    run(&mut w, 150);
    assert_eq!(w.obj(s).unwrap().pos.cell(), at(5, 0));
    let moved: BTreeSet<_> = want.iter().map(|&(x, y)| (x + 5, y)).collect();
    assert_eq!(got(&w, BETA), moved);
    objects::remove(&mut w, s);
    assert!(w.cloak.sensors.is_empty(), "removed, and no count below 0");
}

// spec: S-0516 claim 23, quirks.sensor_building_activation_once, quirks.sensor_array_removal_radius and quirks.sensor_count_negative (an array senses only if in operation at its first placement; removal decrements its current owner's counts over the cloak-radius disc with no floor; capture leaves the coverage with the old owner)
#[test]
fn claim_23_sensor_arrays() {
    let mut r = ruleset();
    r.sim
        .buildings
        .get_mut("ARRAY")
        .unwrap()
        .cloak_radius_in_cells = Some(6);
    let mut w = game(&r);
    let a = spawn(&mut w, "ARRAY", ALPHA, 0, 0);
    assert!(sensed_at(&w, ALPHA, -3, 0) && !sensed_at(&w, ALPHA, -4, 0));
    switch_off(&mut w, a);
    run(&mut w, 3);
    assert!(sensed_at(&w, ALPHA, 0, 0), "power changes do nothing");
    objects::remove(&mut w, a);
    assert_eq!(count_at(&w, ALPHA, 0, 0), 0);
    assert_eq!(
        count_at(&w, ALPHA, -5, 0),
        -1,
        "the cloak-radius disc, no floor"
    );
    // Placed out of operation: never.
    let mut w = game(&r);
    let a = place_off(&mut w, "ARRAY", ALPHA, 0, 0);
    w.obj_mut(a).unwrap().econ.building.switched_off = false;
    run(&mut w, 5);
    assert!(!sensed_at(&w, ALPHA, 0, 0));
    // Capture.
    let mut w = game(&r);
    let a = spawn(&mut w, "ARRAY", ALPHA, 0, 0);
    change_owner(&mut w, a, BETA);
    run(&mut w, 3);
    assert!(sensed_at(&w, ALPHA, 0, 0));
    assert!(!sensed_at(&w, BETA, 0, 0));
    objects::remove(&mut w, a);
    assert!(sensed_at(&w, ALPHA, 0, 0), "the old owner keeps it");
    assert_eq!(count_at(&w, BETA, 0, 0), -1);
}

// spec: S-0516 claim 40 (a detector building adds the disc of its detection range to its owner's detector coverage; removal uses the detection range; detector coverage does not sense)
#[test]
fn claim_40_disguise_detection() {
    let r = ruleset();
    let mut w = game(&r);
    let d = spawn(&mut w, "DETECTOR", ALPHA, 0, 0);
    let want: BTreeSet<(i32, i32)> = cloak::sensor_disc(DETECT_RANGE).into_iter().collect();
    let o = at(0, 0);
    let got: BTreeSet<(i32, i32)> = w
        .cloak
        .detectors
        .iter()
        .filter(|((h, _), n)| *h == ALPHA.0 && **n > 0)
        .map(|((_, c), _)| (c.x - o.x, c.y - o.y))
        .collect();
    assert_eq!(got, want);
    assert!(cloak::detected(&w, Some(ALPHA), at(0, 0)));
    assert!(!sensed_at(&w, ALPHA, 0, 0), "detection does not sense");
    objects::remove(&mut w, d);
    assert!(w.cloak.detectors.is_empty());
}

/// The offsets of the circle of radius `r` in increasing Y, then X.
fn ordered(r: i32) -> Vec<(i32, i32)> {
    let mut v: Vec<_> = cloak::field_circle(r).into_iter().collect();
    v.sort_by_key(|&(x, y)| (y, x));
    v
}

/// The cells of circle `r` up to and including `stop`, in order.
fn prefix(r: i32, stop: (i32, i32)) -> BTreeSet<(i32, i32)> {
    let v = ordered(r);
    let n = v.iter().position(|&c| c == stop).unwrap();
    v[..=n].iter().copied().collect()
}

/// The first cell of row 0 of circle `r` outside circle `r − 1`.
fn first_of_ring(r: i32) -> (i32, i32) {
    let inner = cloak::field_circle(r - 1);
    ordered(r)
        .into_iter()
        .find(|c| c.1 == 0 && !inner.contains(c))
        .unwrap()
}

// spec: S-0516 claim 20 and quirks.cloak_field_building_interrupt (on: the first newly covered own building centre ends a growth update, the radius still increasing; the next growth update examines the whole larger circle; omissions at the final radius stay; off: whole circles)
#[test]
fn claim_20_building_interrupts_growth() {
    let ring2 = first_of_ring(2);
    let mut r = ruleset();
    let mut w = game(&r);
    let g = spawn(&mut w, "GENNY", ALPHA, 0, 0);
    place_off(&mut w, "GENNY", ALPHA, ring2.0, ring2.1);
    run(&mut w, 1);
    assert_eq!(radius(&w, g), 1);
    // spec: S-0516 claim 20 (the own centre interrupts the first radius-1 update: only (0, −1), (−1, 0) and (0, 0))
    let first: BTreeSet<(i32, i32)> = [(0, -1), (-1, 0), (0, 0)].into();
    assert_eq!(field_offsets(&w, ALPHA), first);
    run(&mut w, 1);
    assert_eq!(radius(&w, g), 2);
    let mut want = first.clone();
    want.extend(prefix(2, ring2));
    assert_eq!(field_offsets(&w, ALPHA), want);
    assert!(
        !want.contains(&(1, 0)),
        "omitted radius-1 cells after the stop"
    );
    assert_ne!(want, cloak::field_circle(2), "cells after it omitted");
    run(&mut w, 1);
    assert_eq!(field_offsets(&w, ALPHA), cloak::field_circle(3));
    // An own centre in the final circle: its omissions stay.
    let ring3 = first_of_ring(3);
    let mut w = game(&r);
    let g = spawn(&mut w, "GENNY", ALPHA, 0, 0);
    place_off(&mut w, "GENNY", ALPHA, ring3.0, ring3.1);
    run(&mut w, 10);
    assert_eq!(radius(&w, g), GEN_RADIUS);
    let mut want = cloak::field_circle(2);
    want.extend(prefix(3, ring3));
    assert_eq!(field_offsets(&w, ALPHA), want);
    assert_ne!(want, cloak::field_circle(3));
    // Off: whole circles.
    r.globals.quirks.cloak_field_building_interrupt = false;
    let mut w = game(&r);
    spawn(&mut w, "GENNY", ALPHA, 0, 0);
    place_off(&mut w, "GENNY", ALPHA, ring3.0, ring3.1);
    run(&mut w, 2);
    assert_eq!(field_offsets(&w, ALPHA), cloak::field_circle(2));
    run(&mut w, 10);
    assert_eq!(field_offsets(&w, ALPHA), cloak::field_circle(3));
}

// spec: S-0516 claim 20 and quirks.cloak_field_building_interrupt (on: the shrink update reaching 0 stops after the generator's own centre, leaving cells and suppressing the neighbour restart; off: the collapse is whole and restarts the neighbour, of any owner)
#[test]
fn claim_20_building_interrupts_shrink() {
    for on in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.cloak_field_building_interrupt = on;
        let mut w = game(&r);
        let a = spawn(&mut w, "GENNY", ALPHA, 0, 0);
        let b = spawn(&mut w, "GENNY", BETA, 4, 0);
        run(&mut w, 5);
        switch_off(&mut w, a);
        run(&mut w, GEN_RADIUS);
        assert_eq!(radius(&w, a), 0);
        if on {
            assert!(!field_offsets(&w, ALPHA).is_empty(), "uncleared cells");
            assert_eq!(radius(&w, b), GEN_RADIUS, "no restart");
        } else {
            assert!(field_offsets(&w, ALPHA).is_empty());
            assert!(radius(&w, b) < GEN_RADIUS, "restarted");
        }
    }
}

// spec: S-0516 claim 20 (removal and capture clear the field without interruptions and perform the neighbour-restart test: strictly inside ΔX² + ΔY² < 4(r + 2)² restarts, exactly on it does not; a neighbour below its full radius drops by one)
#[test]
fn claim_20_immediate_collapse_restarts() {
    let r = ruleset();
    // 4(r + 2)² = 100 for r = 3: (6, 8) is on the boundary, (6, 7) inside.
    let mut w = game(&r);
    let a = spawn(&mut w, "GENNY", ALPHA, 0, 0);
    let inside = spawn(&mut w, "GENNY", BETA, 6, 7);
    let on_edge = spawn(&mut w, "GENNY", BETA, 6, 8);
    run(&mut w, 5);
    objects::remove(&mut w, a);
    assert!(field_offsets(&w, ALPHA).is_empty());
    assert_eq!(radius(&w, inside), 0, "full radius: restarts from 0");
    assert_eq!(radius(&w, on_edge), GEN_RADIUS, "on the boundary");
    // spec: S-0516 claim 20 (a growing neighbour is not idle and does not restart; capture restarts an idle one)
    let mut w = game(&r);
    let a = spawn(&mut w, "GENNY", ALPHA, 0, 0);
    let idle = spawn(&mut w, "GENNY", BETA, 0, 4);
    run(&mut w, 5);
    let n = spawn(&mut w, "GENNY", ALPHA, 2, 0);
    run(&mut w, 2);
    assert_eq!(radius(&w, n), 2);
    assert_eq!(phase(&w, idle), FieldPhase::Idle);
    change_owner(&mut w, a, BETA);
    assert_eq!(radius(&w, n), 2, "growing: no restart");
    assert_eq!(radius(&w, idle), 0, "idle at full radius: restarts from 0");
    assert_eq!(phase(&w, idle), FieldPhase::Growing);
}

fn phase(w: &World, g: ObjectId) -> FieldPhase {
    w.obj(g).unwrap().cloak.field.unwrap().phase
}

// spec: S-0516 claim 20 (idle means no growth or shrinking pending: growing at full radius before its next update, a shrinking generator back in operation, and a neighbour not in operation are all ineligible for a restart; a second collapse cannot restart a neighbour the first made growing)
#[test]
fn claim_20_restart_eligibility_by_state() {
    let r = ruleset();
    // Growing at full radius: reached on the third update, idle on the fourth.
    let mut w = game(&r);
    let n = spawn(&mut w, "GENNY", BETA, 2, 0);
    run(&mut w, GEN_RADIUS);
    assert_eq!(radius(&w, n), GEN_RADIUS);
    assert_eq!(phase(&w, n), FieldPhase::Growing);
    let a = place_off(&mut w, "GENNY", ALPHA, 0, 0);
    w.obj_mut(a).unwrap().econ.building.switched_off = false;
    objects::remove(&mut w, a);
    assert_eq!(
        radius(&w, n),
        GEN_RADIUS,
        "growing at full radius: no restart"
    );
    run(&mut w, 1);
    assert_eq!(phase(&w, n), FieldPhase::Idle);
    // Two collapses: the first restarts, the second finds it growing.
    let a = spawn(&mut w, "GENNY", ALPHA, 0, 0);
    let b = spawn(&mut w, "GENNY", ALPHA, 0, 2);
    objects::remove(&mut w, a);
    assert_eq!((radius(&w, n), phase(&w, n)), (0, FieldPhase::Growing));
    run(&mut w, 1);
    objects::remove(&mut w, b);
    assert_eq!(radius(&w, n), 1, "growing: the second collapse leaves it");
    // Shrinking, even back in operation, and not in operation: no restart.
    let mut w = game(&r);
    let s = spawn(&mut w, "GENNY", BETA, 2, 0);
    let off = spawn(&mut w, "GENNY", BETA, 0, 2);
    run(&mut w, 5);
    switch_off(&mut w, s);
    switch_off(&mut w, off);
    run(&mut w, 1);
    assert_eq!(phase(&w, s), FieldPhase::Shrinking);
    w.obj_mut(s).unwrap().econ.building.switched_off = false;
    let a = spawn(&mut w, "GENNY", ALPHA, 0, 0);
    let rs = radius(&w, s);
    let ro = radius(&w, off);
    objects::remove(&mut w, a);
    assert_eq!(radius(&w, s), rs, "shrinking: no restart");
    assert_eq!(radius(&w, off), ro, "not in operation: no restart");
}

// spec: S-0516 claim 20 (a zero-radius generator gives no field and cannot initiate a restart by shrinking, but when idle and in operation it receives a restart, becomes growing, and its next growth update makes it idle again)
#[test]
fn claim_20_zero_radius_generator() {
    let mut r = ruleset();
    let mut zero = r.sim.buildings["GENNY"].clone();
    zero.cloak_radius_in_cells = Some(0);
    r.sim.buildings.insert("GENNY0".into(), zero);
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("GENNY0".into());
    let mut w = game(&r);
    let z = spawn(&mut w, "GENNY0", BETA, 2, 0);
    let n = spawn(&mut w, "GENNY", ALPHA, 0, 2);
    run(&mut w, 5);
    assert_eq!((radius(&w, z), phase(&w, z)), (0, FieldPhase::Idle));
    assert!(field_offsets(&w, BETA).is_empty());
    // Its shrink initiates nothing.
    switch_off(&mut w, z);
    run(&mut w, 3);
    assert_eq!(radius(&w, n), GEN_RADIUS);
    // Idle in operation, it receives a restart.
    w.obj_mut(z).unwrap().econ.building.switched_off = false;
    run(&mut w, 1);
    let a = spawn(&mut w, "GENNY", ALPHA, 0, 0);
    objects::remove(&mut w, a);
    assert_eq!((radius(&w, z), phase(&w, z)), (0, FieldPhase::Growing));
    run(&mut w, 1);
    assert_eq!((radius(&w, z), phase(&w, z)), (0, FieldPhase::Idle));
    assert!(field_offsets(&w, BETA).is_empty());
}

/// The x offset of the centre of house `h`'s sensor disc on row 0.
fn disc_x(w: &World, h: HouseId) -> i32 {
    let o = at(0, 0);
    let xs: Vec<i32> = w
        .cloak
        .sensors
        .iter()
        .filter(|((hh, c), n)| *hh == h.0 && c.y == o.y && **n > 0)
        .map(|((_, c), _)| c.x - o.x)
        .collect();
    (xs.iter().min().unwrap() + xs.iter().max().unwrap()) / 2
}

// spec: S-0516 claims 13 and 22 (walking and hovering sensors relocate their disc at their locomotor's cell-arrival update, not when the centre cell changes: there are ticks in another cell with the disc still behind; the walker's final arrival moves it to the destination, a hover's last choice uses its pre-travel position)
#[test]
fn claim_22_walk_and_hover_arrival_updates() {
    use glasnost_rules::sim::Locomotor;
    let mut r = ruleset();
    let mut scout = r.sim.infantry["SHADE"].clone();
    scout.combat.cloakable = None;
    scout.combat.sensors = Some(true);
    scout.combat.sensors_sight = Some(SEEKER_SIGHT);
    r.sim.infantry.insert("SCOUT".into(), scout);
    r.registries
        .get_mut("infantry")
        .unwrap()
        .push("SCOUT".into());
    let mut skimmer = r.sim.vehicles["SEEKER"].clone();
    skimmer.locomotor = Some(Locomotor::Hover);
    r.sim.vehicles.insert("SKIMMER".into(), skimmer);
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("SKIMMER".into());
    for name in ["SCOUT", "SKIMMER"] {
        let mut w = game(&r);
        let s = spawn(&mut w, name, ALPHA, 0, 0);
        assert_eq!(disc_x(&w, ALPHA), 0);
        move_to(&mut w, s, at(4, 0));
        let mut lagged = false;
        let mut moves = 0;
        for _ in 0..300 {
            let x0 = w.obj(s).unwrap().pos.cell().x - at(0, 0).x;
            let d0 = disc_x(&w, ALPHA);
            run(&mut w, 1);
            let x = w.obj(s).unwrap().pos.cell().x - at(0, 0).x;
            let d = disc_x(&w, ALPHA);
            if x != d {
                lagged = true;
            }
            if d != d0 {
                moves += 1;
                if name == "SKIMMER" {
                    // A hover's update uses its actual position at the
                    // choice, before that tick's travel.
                    assert_eq!(d, x0, "the pre-travel cell");
                } else {
                    // A walker's arrival snaps it: its cell at the update.
                    assert_eq!(d, x, "the arrival cell");
                }
            }
        }
        assert_eq!(w.obj(s).unwrap().pos.cell(), at(4, 0), "{name}");
        assert!(moves > 0, "{name}");
        if name == "SCOUT" {
            // The walker's final arrival and snap is an update.
            assert_eq!(disc_x(&w, ALPHA), 4);
        }
        assert!(lagged, "{name}: the cell changes before the arrival update");
    }
}

/// Puts a building on the map the way a place order does (S-0406 claim
/// 17): no current behaviour and *construction* queued.
fn placed(w: &mut World, name: &str, h: HouseId, dx: i32, dy: i32) -> ObjectId {
    let t = w.rules().index_of(Kind::Building, name).unwrap();
    let id = objects::create(w, Kind::Building, t, Some(h));
    glasnost_sim::econ::placement::put_on_map(w, id, at(dx, dy), true);
    id
}

// spec: S-0516 claims 23 and 40 (r20, Q-3060: activation only at the grand opening; one placed by a place order has construction current at its handover's opening and never adds its disc, whatever its power; removal decrements its owner's counts whether or not it ever added the disc)
#[test]
fn claims_23_40_r20_placed_by_order_never_covers() {
    let mut r = ruleset();
    r.sim
        .buildings
        .get_mut("ARRAY")
        .unwrap()
        .cloak_radius_in_cells = Some(6);
    let mut w = game(&r);
    let a = placed(&mut w, "ARRAY", ALPHA, 0, 0);
    let d = placed(&mut w, "DETECTOR", ALPHA, 6, 6);
    run(&mut w, 60);
    for id in [a, d] {
        let o = w.obj(id).unwrap();
        assert!(!o.econ.construction.building_up, "handed over");
        assert!(glasnost_sim::econ::power::in_operation(&w, id));
    }
    assert!(!sensed_at(&w, ALPHA, 0, 0), "no sensor disc");
    assert!(
        !cloak::detected(&w, Some(ALPHA), at(6, 6)),
        "no detector disc"
    );
    objects::remove(&mut w, a);
    assert_eq!(count_at(&w, ALPHA, 0, 0), -1);
    assert_eq!(count_at(&w, ALPHA, -5, 0), -1, "the cloak-radius disc");
    objects::remove(&mut w, d);
    let o = at(6, 6);
    assert_eq!(
        w.cloak.detectors.get(&(ALPHA.0, o)).copied(),
        Some(-1),
        "the detection range, no floor"
    );
}

// spec: S-0516 claims 23 and 40 (r20, Q-3060: an array or detector whose owner changes during its build-up has its grand opening inside the change, for its new owner, after the change's guard request; with construction still queued that request replaced it, guard is read and the new owner gets the disc) and S-0309 claim 31 (the grand opening step of the orders reset)
#[test]
fn claims_23_40_r20_owner_change_in_build_up_opens_for_the_new_owner() {
    use glasnost_sim::structures::ownership::{self, Cause};
    let r = ruleset();
    let mut w = game(&r);
    let a = placed(&mut w, "ARRAY", ALPHA, 0, 0);
    let d = placed(&mut w, "DETECTOR", ALPHA, 6, 6);
    for id in [a, d] {
        assert_eq!(w.obj(id).unwrap().beh.current, None);
        ownership::change(&mut w, id, Some(BETA), Cause::Trigger);
    }
    assert!(sensed_at(&w, BETA, 0, 0), "the new owner senses");
    assert!(!sensed_at(&w, ALPHA, 0, 0));
    assert!(
        cloak::detected(&w, Some(BETA), at(6, 6)),
        "the new owner detects"
    );
    run(&mut w, 60);
    assert!(!w.obj(a).unwrap().econ.construction.building_up);
    assert_eq!(
        count_at(&w, BETA, 0, 0),
        1,
        "the handover adds no second disc"
    );
}
