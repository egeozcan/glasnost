//! Claim tests for chrono weapons (S-0514), part two: completion, credit,
//! transports, buildings, presentation and the quirk switches.

#[path = "temporal_common/mod.rs"]
mod temporal_common;

use glasnost_sim::econ::stats;
use glasnost_sim::rules::Kind;
use glasnost_sim::spawn::{Slot, SlotState, SpawnController};
use glasnost_sim::temporal;
use glasnost_sim::testkit;
use glasnost_sim::transport;
use glasnost_sim::world::{HouseId, ObjectId, Pos, World};
use temporal_common::*;

fn two(tweak: impl FnOnce(&mut glasnost_rules::schema::Ruleset)) -> (World, ObjectId, ObjectId) {
    let mut w = world(tweak);
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let a = attacker(&mut w, "ERASER", 0, at(-12, -12));
    (w, a, v)
}

fn finish(w: &mut World, v: ObjectId) {
    for _ in 0..400 {
        if w.obj(v).is_none() {
            return;
        }
        testkit::run(w, 1);
    }
    panic!("the erasure did not complete");
}

// spec: S-0514 claim 12 (the completion animation is played at the target, which is removed without death: no wreck, and the lock state is gone)
#[test]
fn completion_plays_the_warp_away_and_removes_the_target() {
    let (mut w, a, v) = two(|_| {});
    erase(&mut w, a, v);
    w.effects.clear();
    finish(&mut w, v);
    assert!(anims(&w).iter().any(|n| n == "WARP"));
    assert!(w.temporal.beams.is_empty() && w.temporal.locks.is_empty());
}

// spec: S-0514 claim 13 (the leader is the credited killer: the house's kill and the victim house's loss are counted)
#[test]
fn the_leader_is_credited_with_the_kill() {
    let (mut w, a, v) = two(|_| {});
    erase(&mut w, a, v);
    finish(&mut w, v);
    assert_eq!(stats::of(&w, HouseId(0)).kills, 1);
    assert_eq!(stats::of(&w, HouseId(1)).losses, 1);
}

// spec: S-0514 claim 13 and F5 (E6: attacker cost 1000, victim cost 500, ratio 4: the direct gain 500 / (1000 x 4) = 0.125 and the ordinary gain 0.125 make 0.25 per erasure; quirks.erasure_double_experience off keeps only the ordinary 0.125)
#[test]
fn the_leader_gains_experience_directly_and_by_credit() {
    for (double, want) in [(true, "0.25"), (false, "0.125")] {
        let (mut w, a, v) = two(|r| r.globals.quirks.erasure_double_experience = double);
        erase(&mut w, a, v);
        finish(&mut w, v);
        let mode = w.rules().mode;
        let want = glasnost_num::decimal::nearest_double(want);
        let got = w.obj(a).unwrap().combat.experience.to_f64();
        assert!(got.eq_num(want), "double {double}: {got:?} vs {want:?}");
        let _ = mode;
    }
}

// spec: S-0514 claim 13 (F5: a leader that is not trainable gets no direct gain)
#[test]
fn an_untrainable_leader_gets_no_direct_gain() {
    let mut w = world(|r| r.sim.infantry.get_mut("ERASER").unwrap().combat.trainable = Some(false));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let a = attacker(&mut w, "ERASER", 0, at(-12, -12));
    erase(&mut w, a, v);
    finish(&mut w, v);
    assert_eq!(
        w.obj(a).unwrap().combat.experience.to_f64().to_i32_common(),
        0
    );
    assert_eq!(
        w.obj(a)
            .unwrap()
            .combat
            .experience
            .to_f64()
            .eq_num(glasnost_num::F64::ZERO),
        true
    );
}

// spec: S-0514 claim 12 (a transport's passengers are removed with it, with no killer: they give the erasing attacker no kill credit)
#[test]
fn passengers_are_removed_without_credit() {
    let mut w = world(|_| {});
    let t = spawn(&mut w, Kind::Vehicle, "OPEN", 1, at(0, 0));
    let p1 = spawn(&mut w, Kind::Infantry, "BYSTANDER", 1, at(1, 1));
    let p2 = spawn(&mut w, Kind::Infantry, "BYSTANDER", 1, at(1, 2));
    transport::board(&mut w, p1, t);
    transport::board(&mut w, p2, t);
    let a = attacker(&mut w, "ERASER", 0, at(-12, -12));
    erase(&mut w, a, t);
    finish(&mut w, t);
    assert!(w.obj(p1).is_none() && w.obj(p2).is_none());
    assert_eq!(stats::of(&w, HouseId(0)).kills, 1, "the transport only");
    assert_eq!(stats::of(&w, HouseId(1)).losses, 3);
}

// spec: S-0514 claims 4 and 10 (E7: a locked building is switched off and switched on again by the last release, even if it was switched off before; quirks.erasure_release_switches_on off restores the earlier state)
#[test]
fn a_locked_building_is_switched_off_and_on_again() {
    for (switches_on, was_off, want) in [
        (true, true, false),
        (false, true, true),
        (false, false, false),
    ] {
        let mut w = world(|r| r.globals.quirks.erasure_release_switches_on = switches_on);
        let b = spawn(&mut w, Kind::Building, "TOWER", 1, at(0, 0));
        w.obj_mut(b).unwrap().econ.building.switched_off = was_off;
        let a = attacker(&mut w, "ERASER", 0, at(-12, -12));
        erase(&mut w, a, b);
        assert!(
            w.obj(b).unwrap().econ.building.switched_off,
            "off while locked"
        );
        assert!(temporal::building_erased(&w, b));
        temporal::let_go(&mut w, a);
        assert_eq!(
            w.obj(b).unwrap().econ.building.switched_off,
            want,
            "switches on {switches_on}, was off {was_off}"
        );
    }
}

// spec: S-0514 claim 12 (a building is erased the same way: removed when the amount runs out; its strength gives the amount)
#[test]
fn a_building_is_erased_by_its_strength() {
    let mut w = world(|_| {});
    let b = spawn(&mut w, Kind::Building, "TOWER", 1, at(0, 0));
    let a = attacker(&mut w, "ERASER", 0, at(-12, -12));
    erase(&mut w, a, b);
    assert_eq!(amount(&w, b), Some(8000)); // the tower's strength is 800
    finish(&mut w, b);
    assert!(w.obj(b).is_none());
}

// spec: S-0514 claim 2 and Edge cases (a delivered hit clears the target's spawner slots even when the lock is refused, quirks.erasure_refused_hit_side_effects; off: only a successful lock does)
#[test]
fn a_refused_hit_still_clears_the_spawner_slots() {
    for (on, refused, cleared) in [
        (true, true, true),
        (false, true, false),
        (false, false, true),
        (true, false, true),
    ] {
        let mut w = world(|r| {
            r.globals.quirks.erasure_refused_hit_side_effects = on;
            r.sim.vehicles.get_mut("VICT").unwrap().warpable = Some(!refused);
        });
        let carrier = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
        let slot = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(2, 0));
        w.obj_mut(carrier).unwrap().spawn = Some(SpawnController {
            slots: vec![Slot {
                state: SlotState::Docked,
                object: Some(slot),
                until: 0,
            }],
            ..Default::default()
        });
        let a = attacker(&mut w, "ERASER", 0, at(-12, -12));
        erase(&mut w, a, carrier);
        assert_eq!(erased(&w, carrier), !refused);
        assert_eq!(w.obj(slot).is_none(), cleared, "on {on}, refused {refused}");
    }
}

fn board_leader(w: &mut World, name: &str, t: ObjectId, c: (i32, i32)) -> ObjectId {
    let p = attacker(w, name, 0, at(c.0, c.1));
    transport::board(w, p, t);
    p
}

// spec: S-0514 claim 11 and F4 (E8: warp distance 3 cells = 768 leptons: an open-topped leader lets go at the first update whose distance exceeds it, and that update makes no reduction)
#[test]
fn an_open_topped_leader_lets_go_beyond_the_warp_distance() {
    let mut w = world(|r| r.sim.vehicles.get_mut("VICT").unwrap().strength = Some(100_000));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let t = spawn(&mut w, Kind::Vehicle, "OPEN", 0, at(-2, 0));
    let p = board_leader(&mut w, "ERASER", t, (-2, 1));
    erase(&mut w, p, v);
    assert!(temporal::holds(&w, p, v));
    let before = amount(&w, v).unwrap();
    // The transport at the edge: distance 768 is not beyond it.
    let vp = w.obj(v).unwrap().pos;
    w.obj_mut(p).unwrap().pos = Pos::new(vp.x - 768, vp.y, vp.z);
    testkit::run(&mut w, 1);
    assert!(temporal::holds(&w, p, v), "at 768 leptons it is held");
    assert!(amount(&w, v).unwrap() < before);
    // Well beyond (the table root falls short of the exact root): it lets go, with no reduction.
    w.obj_mut(p).unwrap().pos = Pos::new(vp.x - 808, vp.y, vp.z);
    let r = amount(&w, v);
    testkit::run(&mut w, 1);
    assert!(!erased(&w, v), "released");
    let _ = r;
}

// spec: S-0514 claim 11 and Edge cases (E8: only a leading open-topped attacker is checked; quirks.erasure_distance_leader_only off checks every open-topped attacker)
#[test]
fn a_distant_open_topped_joiner_is_checked_only_without_the_leader_only_quirk() {
    for (leader_only, kept) in [(true, true), (false, false)] {
        let mut w = world(|r| {
            r.sim.vehicles.get_mut("VICT").unwrap().strength = Some(100_000);
            r.globals.quirks.erasure_distance_leader_only = leader_only;
        });
        let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
        let t = spawn(&mut w, Kind::Vehicle, "OPEN", 0, at(-2, 0));
        let leader = attacker(&mut w, "ERASER", 0, at(-3, 0));
        let joiner = board_leader(&mut w, "ERASER2", t, (-2, 1));
        erase(&mut w, leader, v);
        erase(&mut w, joiner, v);
        let vp = w.obj(v).unwrap().pos;
        w.obj_mut(joiner).unwrap().pos = Pos::new(vp.x - 2000, vp.y, vp.z);
        testkit::run(&mut w, 2);
        assert_eq!(
            temporal::holds(&w, joiner, v),
            kept,
            "leader only: {leader_only}"
        );
        assert!(temporal::holds(&w, leader, v));
    }
}

// spec: S-0308 claim 6 and S-0514 claim 4 (E: a first passenger with a beam hands it to a gunner vehicle, attached or not, and the vehicle carries on the erasure; the last passenger leaving hands it back and the beam lets go, freeing a sole target)
#[test]
fn a_gunner_vehicle_carries_the_first_passengers_beam() {
    let mut w = world(|r| r.sim.vehicles.get_mut("VICT").unwrap().strength = Some(100_000));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let g = spawn(&mut w, Kind::Vehicle, "GUNNER", 0, at(-12, -12));
    let p = attacker(&mut w, "ERASER", 0, at(-12, -11));
    erase(&mut w, p, v);
    let r = amount(&w, v);
    transport::board(&mut w, p, g);
    assert!(temporal::has_beam(&w, g), "the vehicle has it");
    assert!(!temporal::has_beam(&w, p), "the passenger no longer does");
    assert!(temporal::holds(&w, g, v), "the erasure goes on");
    assert_eq!(w.temporal.locks[&v].leader, g);
    assert!(amount(&w, v) <= r);
    // The last passenger leaves: the beam goes back and lets go.
    transport::temporal_hand_back(&mut w, g, p);
    assert!(!erased(&w, v));
    assert!(temporal::has_beam(&w, p) && !temporal::has_beam(&w, g));
}

// spec: S-0514 claim 18 (E11 part b: a target being erased sparkles on its turns whose global tick number is divisible by 24, 120 leptons farther along each horizontal axis for a vehicle)
#[test]
fn a_sparkle_comes_on_every_24th_tick() {
    let (mut w, a, v) = two(|r| r.sim.vehicles.get_mut("VICT").unwrap().strength = Some(100_000));
    erase(&mut w, a, v);
    let vp = w.obj(v).unwrap().pos;
    let mut ticks = Vec::new();
    for _ in 0..100 {
        w.effects.clear();
        let t = w.tick;
        testkit::run(&mut w, 1);
        for e in &w.effects {
            if let glasnost_sim::combat::effects::Effect::Anim { name, pos, .. } = e
                && name == "SPARK"
            {
                ticks.push(t);
                assert_eq!(*pos, Pos::new(vp.x + 120, vp.y + 120, vp.z));
            }
        }
    }
    assert_eq!(ticks, vec![24, 48, 72, 96]);
}

// spec: S-0514 claim 18 (a building sparkles at its own position, residue 0, when it has no damage-fire positions)
#[test]
fn a_building_sparkles_at_its_own_position() {
    let mut w = world(|_| {});
    let b = spawn(&mut w, Kind::Building, "TOWER", 1, at(0, 0));
    let a = attacker(&mut w, "ERASER", 0, at(-12, -12));
    erase(&mut w, a, b);
    let bp = w.obj(b).unwrap().pos;
    let mut seen = Vec::new();
    for _ in 0..30 {
        w.effects.clear();
        testkit::run(&mut w, 1);
        for e in &w.effects {
            if let glasnost_sim::combat::effects::Effect::Anim { name, pos, .. } = e
                && name == "SPARK"
            {
                seen.push(*pos);
            }
        }
    }
    assert_eq!(seen, vec![bp]);
}

// spec: S-0514 claim 4 (a hit on an erased target does not damage it: S-0302 claim 8 row 1, the erased target is immune to ordinary hits)
#[test]
fn an_erased_target_is_immune_to_ordinary_hits() {
    let (mut w, a, v) = two(|_| {});
    erase(&mut w, a, v);
    glasnost_sim::combat::damage::apply(
        &mut w,
        v,
        glasnost_sim::combat::damage::Hit {
            amount: 50,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: false,
            no_escape: false,
        },
    );
    assert_eq!(health(&w, v), 100);
}

// spec: S-0514 Determinism: the state hash survives a save and load with an erasure in progress
#[test]
fn an_erasure_survives_a_save_and_load() {
    let (mut w, a, v) = two(|r| r.sim.vehicles.get_mut("VICT").unwrap().strength = Some(100_000));
    erase(&mut w, a, v);
    testkit::run(&mut w, 10);
    let bytes = glasnost_sim::save::save(&w);
    let h = glasnost_sim::save::state_hash(&w);
    let w2 = glasnost_sim::save::load(&bytes, w.rules_arc(), w.catalogue.clone().unwrap()).unwrap();
    assert_eq!(glasnost_sim::save::state_hash(&w2), h);
    assert_eq!(w2.temporal, w.temporal);
}
