//! Quirk switches of S-0516 (cloak and stealth), each on and off, and the
//! save round trip of the cloak state, on synthetic rules (every name and
//! number is invented; see `cloak_common`).

mod cloak_common;

use cloak_common::*;
use glasnost_num::F32;
use glasnost_sim::cloak::{CloakState, machine};
use glasnost_sim::world::World;
use glasnost_sim::{objects, save, tick};

const ONE: u32 = 0x3F80_0000; // 1.0, veteran

// spec: S-0516 Edge cases (quirks.cloak_flag_ground_mobile_only: on, aircraft and buildings ignore the type's flag; off, they cloak by themselves)
#[test]
fn quirk_cloak_flag_ground_mobile_only() {
    for on in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.cloak_flag_ground_mobile_only = on;
        let mut w = game(&r);
        let b = spawn(&mut w, "VEIL", ALPHA, 0, 0);
        let a = spawn(&mut w, "WRAITH", ALPHA, 5, 0);
        assert_eq!(w.obj(b).unwrap().cloak.own, !on);
        assert_eq!(w.obj(a).unwrap().cloak.own, !on);
        run(&mut w, 30);
        let want = if on {
            CloakState::Uncloaked
        } else {
            CloakState::Cloaked
        };
        assert_eq!(state(&w, b), want);
        assert_eq!(state(&w, a), want);
    }
}

// spec: S-0516 claim 7 and Edge cases (quirks.uncloak_from_cloaking_full_duration: on, an uncloak during cloaking starts at N − 1; off, at the current stage)
#[test]
fn quirk_uncloak_from_cloaking_full_duration() {
    for on in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.uncloak_from_cloaking_full_duration = on;
        let mut w = game(&r);
        let s = spawn(&mut w, "SHADE", ALPHA, 0, 0);
        run(&mut w, 5);
        assert_eq!(state(&w, s), CloakState::Cloaking);
        let k = stage(&w, s);
        assert!(k > 0 && k < STAGES - 1);
        hit(&mut w, s, 1);
        assert_eq!(state(&w, s), CloakState::Uncloaking);
        assert_eq!(stage(&w, s), if on { STAGES - 1 } else { k });
    }
}

// spec: S-0516 Edge cases (quirks.cloak_zero_speed_stall: with speed 0 cloaking advances one stage per tick; on, uncloaking never advances; off, one stage per tick)
#[test]
fn quirk_cloak_zero_speed_stall() {
    for on in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.cloak_zero_speed_stall = on;
        r.sim
            .infantry
            .get_mut("SHADE")
            .unwrap()
            .combat
            .cloaking_speed = Some(0);
        let mut w = game(&r);
        let s = spawn(&mut w, "SHADE", ALPHA, 0, 0);
        let t = transitions(&mut w, s, 10);
        assert_eq!(t[0].1, CloakState::Cloaking);
        assert_eq!(t[1], (t[0].0 + 4, CloakState::Cloaked));
        hit(&mut w, s, 1);
        let t = transitions(&mut w, s, 30);
        if on {
            assert!(t.is_empty());
            assert_eq!(stage(&w, s), STAGES - 1);
        } else {
            assert_eq!(t[0], (STAGES - 1, CloakState::Uncloaked));
        }
    }
}

// spec: S-0516 Edge cases and claim 43 (quirks.cloak_zero_speed_stall stalls uncloaking only from a positive stage: with N = 1 uncloaking starts at stage 0 and completes at the next update, on or off)
#[test]
fn quirk_cloak_zero_speed_stall_one_stage() {
    let mut seen = Vec::new();
    for on in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.cloak_zero_speed_stall = on;
        r.sim.general.cloaking_stages = Some(1);
        r.sim
            .infantry
            .get_mut("SHADE")
            .unwrap()
            .combat
            .cloaking_speed = Some(0);
        let mut w = game(&r);
        let s = spawn(&mut w, "SHADE", ALPHA, 0, 0);
        force_cloaked(&mut w, s);
        hit(&mut w, s, 1);
        assert_eq!(stage(&w, s), 0);
        let t = transitions(&mut w, s, 30);
        assert_eq!(t.first().map(|x| x.1), Some(CloakState::Uncloaked), "{on}");
        seen.push(t[0]);
    }
    assert_eq!(seen[0], seen[1]);
    assert_eq!(seen[0].0, 0, "at the next update");
}

// spec: S-0516 claim 23 and Edge cases (quirks.sensor_building_activation_once: on, set at first placement only if powered, capture leaves it with the old owner; off, it follows in operation and moves to the new owner)
#[test]
fn quirk_sensor_building_activation_once() {
    for on in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.sensor_building_activation_once = on;
        let mut w = game(&r);
        let a = spawn(&mut w, "ARRAY", ALPHA, 0, 0);
        run(&mut w, 1);
        assert!(sensed_at(&w, ALPHA, 0, 0));
        switch_off(&mut w, a);
        run(&mut w, 2);
        assert_eq!(sensed_at(&w, ALPHA, 0, 0), on, "power off");
        w.obj_mut(a).unwrap().econ.building.switched_off = false;
        run(&mut w, 2);
        assert!(sensed_at(&w, ALPHA, 0, 0), "power back");
        change_owner(&mut w, a, BETA);
        run(&mut w, 2);
        assert_eq!(sensed_at(&w, ALPHA, 0, 0), on);
        assert_eq!(sensed_at(&w, BETA, 0, 0), !on);
    }
}

// spec: S-0516 claim 23 and Edge cases (quirks.sensor_array_removal_radius: on, removal decrements over the cloak-radius disc; off, over the disc it added)
#[test]
fn quirk_sensor_array_removal_radius() {
    for on in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.sensor_array_removal_radius = on;
        r.sim
            .buildings
            .get_mut("ARRAY")
            .unwrap()
            .cloak_radius_in_cells = Some(6);
        let mut w = game(&r);
        let a = spawn(&mut w, "ARRAY", ALPHA, 0, 0);
        objects::remove(&mut w, a);
        assert_eq!(count_at(&w, ALPHA, 0, 0), 0);
        assert_eq!(count_at(&w, ALPHA, -5, 0), if on { -1 } else { 0 });
    }
}

// spec: S-0516 claim 23 and Edge cases (quirks.sensor_count_negative: on, building removals have no floor; off, removal undoes exactly what was added)
#[test]
fn quirk_sensor_count_negative() {
    for on in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.sensor_count_negative = on;
        let mut w = game(&r);
        let a = spawn(&mut w, "ARRAY", ALPHA, 0, 0);
        change_owner(&mut w, a, BETA);
        objects::remove(&mut w, a);
        if on {
            assert_eq!(count_at(&w, BETA, 0, 0), -1);
            assert!(sensed_at(&w, ALPHA, 0, 0));
        } else {
            assert!(w.cloak.sensors.values().all(|&n| n >= 0));
            assert!(!sensed_at(&w, ALPHA, 0, 0));
        }
    }
}

// spec: S-0516 claim 20 and Edge cases (quirks.cloak_field_overlap_loss: on, the shared cells of a shrinking generator leave the field; off, a cell stays while any powered generator of the house covers it)
#[test]
fn quirk_cloak_field_overlap_loss() {
    for on in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.cloak_field_overlap_loss = on;
        let mut w = game(&r);
        let a = spawn(&mut w, "GENNY", ALPHA, 0, 0);
        spawn(&mut w, "GENNY", ALPHA, 2, 0);
        run(&mut w, 5);
        switch_off(&mut w, a);
        run(&mut w, 1);
        // (3, 0) left the shrinking generator's circle (radius 2 now); the
        // other generator, 1 cell away, still covers it.
        assert_eq!(field_at(&w, ALPHA, 3, 0), !on);
    }
}

// spec: S-0516 claim 20 and Edge cases (quirks.cloak_field_overlap_loss: on, a collapse clears its whole circle, shared cells included; off, exactly the cells another generator of the house covers stay; #389: the other generators taken once per collapse)
#[test]
fn quirk_cloak_field_overlap_loss_collapse() {
    use glasnost_map::CellXY;
    use std::collections::BTreeSet;
    for on in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.cloak_field_overlap_loss = on;
        // Every growth update covers its whole circle.
        r.globals.quirks.cloak_field_building_interrupt = false;
        let mut w = game(&r);
        let a = spawn(&mut w, "GENNY", ALPHA, 0, 0);
        let b = spawn(&mut w, "GENNY", ALPHA, 3, 0);
        let c = spawn(&mut w, "GENNY", ALPHA, -2, 3);
        run(&mut w, GEN_RADIUS + 3);
        let circle = |w: &World, id| -> BTreeSet<CellXY> {
            let g = w.obj(id).unwrap().cloak.field.unwrap();
            assert_eq!(g.radius, GEN_RADIUS);
            glasnost_sim::cloak::field_circle(g.radius)
                .into_iter()
                .map(|(dx, dy)| CellXY::new(g.centre.x + dx, g.centre.y + dy))
                .collect()
        };
        let (ca, cb, cc) = (circle(&w, a), circle(&w, b), circle(&w, c));
        let field = |w: &World| -> BTreeSet<CellXY> {
            w.cloak
                .fields
                .iter()
                .filter(|(h, _)| *h == ALPHA.0)
                .map(|&(_, x)| x)
                .collect()
        };
        let all: BTreeSet<CellXY> = ca.iter().chain(&cb).chain(&cc).copied().collect();
        assert_eq!(field(&w), all, "on = {on}");
        assert!(
            ca.iter().any(|x| cb.contains(x))
                && ca.iter().any(|x| !cb.contains(x) && !cc.contains(x))
        );
        kill(&mut w, a);
        let others: BTreeSet<CellXY> = cb.union(&cc).copied().collect();
        let want: BTreeSet<CellXY> = if on {
            others.difference(&ca).copied().collect()
        } else {
            others
        };
        assert_eq!(field(&w), want, "on = {on}");
    }
}

// spec: S-0516 claim 18 and Edge cases (quirks.building_sensor_ring_owner_only: on, an ally's sensor unit stops a building cloaking and a sensors ability does not; off, only non-allied sensor units count, the ability included)
#[test]
fn quirk_building_sensor_ring_owner_only() {
    for on in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.cloak_flag_ground_mobile_only = false;
        r.globals.quirks.building_sensor_ring_owner_only = on;
        r.sim.vehicles.get_mut("PLAIN").unwrap().veteran_abilities = vec!["SENSORS".into()];
        // An allied sensor unit in the ring.
        let mut w = game(&r);
        ally(&mut w, ALPHA, BETA);
        ally(&mut w, BETA, ALPHA);
        let b = spawn(&mut w, "VEIL", ALPHA, 0, 0);
        spawn(&mut w, "SEEKER", BETA, 2, 0);
        run(&mut w, 30);
        let want = if on {
            CloakState::Uncloaked
        } else {
            CloakState::Cloaked
        };
        assert_eq!(state(&w, b), want, "allied sensor, on = {on}");
        // An enemy unit with the sensors ability only.
        let mut w = game(&r);
        let b = spawn(&mut w, "VEIL", ALPHA, 0, 0);
        let p = spawn(&mut w, "PLAIN", BETA, 2, 0);
        w.obj_mut(p).unwrap().combat.experience = F32(ONE);
        run(&mut w, 30);
        let want = if on {
            CloakState::Cloaked
        } else {
            CloakState::Uncloaked
        };
        assert_eq!(state(&w, b), want, "ability, on = {on}");
    }
}

// spec: S-0516 claim 12 and Edge cases (quirks.heal_reveals_cloaked: on, healing or a zero hit uncloaks; off, only hits that lower health)
#[test]
fn quirk_heal_reveals_cloaked() {
    for on in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.heal_reveals_cloaked = on;
        let mut w = game(&r);
        let a = spawn_cloaked(&mut w, "GHOST", ALPHA, 0, 0);
        let b = spawn_cloaked(&mut w, "GHOST", ALPHA, 4, 0);
        set_health(&mut w, b, 100);
        hit(&mut w, a, 0);
        hit(&mut w, b, -10);
        let want = if on {
            CloakState::Uncloaking
        } else {
            CloakState::Cloaked
        };
        assert_eq!(state(&w, a), want);
        assert_eq!(state(&w, b), want);
    }
}

// spec: S-0516 claim 33 (b) and Edge cases (quirks.mirage_adjacent_check_cadence: on, the neighbouring-infantry check runs on ticks not a multiple of 8; off, only on multiples of 8)
#[test]
fn quirk_mirage_adjacent_check_cadence() {
    for on in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.mirage_adjacent_check_cadence = on;
        let mut w = game(&r);
        let m = spawn(&mut w, "MIRAGE", ALPHA, 0, 0);
        until(&mut w, 100, |w| w.obj(m).unwrap().cloak.disguise.is_some());
        until(&mut w, 8, |w| w.tick % 8 == 1);
        spawn(&mut w, "SHADE", BETA, 1, 1);
        let mut lost_at = None;
        for _ in 0..8 {
            let t = w.tick;
            run(&mut w, 1);
            if w.obj(m).unwrap().cloak.disguise.is_none() {
                lost_at = Some(t);
                break;
            }
        }
        let t = lost_at.expect("lost");
        if on {
            assert_eq!(t % 8, 1);
        } else {
            assert_eq!(t % 8, 0);
        }
    }
}

/// Runs two worlds `n` ticks and compares their hashes every tick.
fn lockstep(a: &mut World, b: &mut World, n: i32) {
    for _ in 0..n {
        tick::step(a);
        tick::step(b);
        assert_eq!(save::state_hash(a), save::state_hash(b));
    }
}

// spec: S-0516 claim 22 and Edge cases (quirks.mobile_sensor_removal_centre: on, a mobile sensor's disc is removed around its current cell, leaving counts where it differs from the last-added disc; off, the disc actually added is removed)
#[test]
fn quirk_mobile_sensor_removal_centre() {
    for on in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.mobile_sensor_removal_centre = on;
        let mut w = game(&r);
        let s = spawn(&mut w, "SEEKER", ALPHA, 0, 0);
        assert!(sensed_at(&w, ALPHA, 0, 0));
        // The sensor's position moves on to the next cell without a
        // completed-cell movement event.
        let p = glasnost_sim::world::Pos::centre(at(1, 0), 0);
        let z = w.obj(s).unwrap().pos.z;
        w.obj_mut(s).unwrap().pos = glasnost_sim::world::Pos::new(p.x, p.y, z);
        objects::remove(&mut w, s);
        let left = w
            .cloak
            .sensors
            .iter()
            .filter(|((h, _), n)| *h == ALPHA.0 && **n > 0)
            .count();
        if on {
            assert!(left > 0, "a residue of the added disc");
        } else {
            assert_eq!(left, 0);
        }
        assert!(w.cloak.sensors.values().all(|&n| n >= 0), "floored at 0");
    }
}

// spec: S-0516 State and ordering (all cloak state is saved: a fade, a growing field, sensor counts and a disguise survive a save and load, and the copies run on identically)
#[test]
fn save_round_trip_mid_fade() {
    let r = ruleset();
    let mut w = game(&r);
    spawn(&mut w, "SEEKER", BETA, 6, 6);
    spawn(&mut w, "ARRAY", ALPHA, -6, -6);
    spawn(&mut w, "DETECTOR", BETA, 6, -6);
    let s = spawn(&mut w, "SHADE", ALPHA, 0, 0);
    let m = spawn(&mut w, "MIRAGE", ALPHA, -6, 6);
    until(&mut w, 100, |w| w.obj(m).unwrap().cloak.disguise.is_some());
    let g = spawn(&mut w, "GENNY", ALPHA, 0, 4);
    machine::uncloak(&mut w, s, true);
    run(&mut w, 2);
    assert!(
        w.obj(g)
            .unwrap()
            .cloak
            .field
            .is_some_and(|f| f.radius > 0 && f.radius < GEN_RADIUS)
    );
    assert!(!w.cloak.sensors.is_empty());
    assert!(w.obj(m).unwrap().cloak.disguise.is_some());
    let bytes = save::save(&w);
    let mut l = save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
    lockstep(&mut w, &mut l, 60);
}

// spec: S-0516 claim 22 and Edge cases (quirks.flying_sensor_disc_stationary: on, ordinary flight never relocates a sensor aircraft's disc; off, each horizontal cell crossing in flight moves it to the new cell)
#[test]
fn quirk_flying_sensor_disc_stationary() {
    for on in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.flying_sensor_disc_stationary = on;
        let mut eye = r.sim.aircraft["WRAITH"].clone();
        eye.combat.cloakable = None;
        eye.combat.sensors = Some(true);
        eye.combat.sensors_sight = Some(SEEKER_SIGHT);
        r.sim.aircraft.insert("EYE".into(), eye);
        r.registries.get_mut("aircraft").unwrap().push("EYE".into());
        let mut w = game(&r);
        let a = spawn(&mut w, "EYE", ALPHA, 0, 0);
        assert!(sensed_at(&w, ALPHA, 0, 0));
        move_to(&mut w, a, at(10, 0));
        until(&mut w, 400, |w| {
            w.obj(a).unwrap().pos.cell().x >= at(6, 0).x
        });
        let x = w.obj(a).unwrap().pos.cell().x - at(0, 0).x;
        if on {
            assert!(sensed_at(&w, ALPHA, 0, 0), "the disc stays at the start");
            assert!(!sensed_at(&w, ALPHA, x, 0));
        } else {
            assert!(!sensed_at(&w, ALPHA, 0, 0), "the disc left the start");
            assert!(sensed_at(&w, ALPHA, x, 0), "the disc follows the flight");
        }
    }
}
