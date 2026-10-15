//! Claim tests of S-0516 (cloak and stealth), fields and coverage: claims
//! 19 to 23 and 40, on synthetic rules (every name and number is invented;
//! see `cloak_common`).

mod cloak_common;

use std::collections::BTreeSet;

use cloak_common::*;
use glasnost_sim::cloak::{self, CloakState};
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

// spec: S-0516 claim 20 (growth one ring per tick to the radius; shrinking one ring per tick, cells leaving at once; removal and capture vanish at once, capture regrows for the new owner; a field covers its owner only)
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
    for _ in 0..4 {
        run(&mut w, 1);
        radii.push(radius(&w, g));
        assert_eq!(
            field_offsets(&w, ALPHA),
            cloak::field_circle(*radii.last().unwrap())
        );
    }
    assert_eq!(radii, vec![2, 1, 0, 0]);
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

// spec: S-0516 claim 20 and quirks.cloak_field_overlap_loss (a shrinking generator's shared cells leave the field; at radius 0 a powered full generator closer than 2 (r + 2) restarts from 0)
#[test]
fn claim_20_overlap_and_restart() {
    let r = ruleset();
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
