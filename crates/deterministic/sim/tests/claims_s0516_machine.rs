//! Claim tests of S-0516 (cloak and stealth), the cloak state machine:
//! claims 1 to 18 and 41, on synthetic rules (every name and number is
//! invented; see `cloak_common`).

mod cloak_common;

use cloak_common::*;
use glasnost_num::F32;
use glasnost_sim::cloak::{self, CloakState, machine};
use glasnost_sim::world::Target;

const ONE: u32 = 0x3F80_0000; // 1.0, veteran

fn stages_of(
    w: &mut glasnost_sim::world::World,
    id: glasnost_sim::world::ObjectId,
    n: i32,
) -> Vec<(CloakState, i32)> {
    let mut out = Vec::new();
    for _ in 0..n {
        run(w, 1);
        out.push((state(w, id), stage(w, id)));
    }
    out
}

// spec: S-0516 claim 1 (uncloaked → cloaking → cloaked → uncloaking → uncloaked; the stage counts up while cloaking, down while uncloaking, and is 0 when uncloaked or cloaked)
#[test]
fn claim_1_states_and_stages() {
    let r = ruleset();
    let mut w = game(&r);
    let id = spawn(&mut w, "SHADE", ALPHA, 0, 0);
    assert_eq!(state(&w, id), CloakState::Uncloaked);
    assert_eq!(stage(&w, id), 0);
    let seen = stages_of(&mut w, id, 12);
    let cloaking: Vec<i32> = seen
        .iter()
        .filter(|(s, _)| *s == CloakState::Cloaking)
        .map(|&(_, k)| k)
        .collect();
    assert!(!cloaking.is_empty());
    assert!(cloaking.windows(2).all(|p| p[1] >= p[0]), "counts up");
    assert_eq!(state(&w, id), CloakState::Cloaked);
    assert_eq!(stage(&w, id), 0);
    hit(&mut w, id, 1);
    assert_eq!(state(&w, id), CloakState::Uncloaking);
    let seen = stages_of(&mut w, id, 20);
    let uncloaking: Vec<i32> = seen
        .iter()
        .filter(|(s, _)| *s == CloakState::Uncloaking)
        .map(|&(_, k)| k)
        .collect();
    assert!(uncloaking.windows(2).all(|p| p[1] <= p[0]), "counts down");
    assert!(seen.iter().any(|&(s, _)| s == CloakState::Uncloaked));
    let after: Vec<_> = seen
        .iter()
        .skip_while(|(s, _)| *s != CloakState::Uncloaked)
        .take_while(|(s, _)| *s == CloakState::Uncloaked)
        .collect();
    assert!(after.iter().all(|&&(_, k)| k == 0));
}

// spec: S-0516 claim 2 (may cloak: (a) the vehicle's or infantry's own flag from its type, (b) a cloaking veterancy ability at its rank, (c) its owner's cloak field, (d) a crate-set own flag; aircraft and buildings do not take the type's flag)
#[test]
fn claim_2_where_the_ability_comes_from() {
    let mut r = ruleset();
    r.sim.vehicles.get_mut("PLAIN").unwrap().veteran_abilities = vec!["CLOAK".into()];
    let mut w = game(&r);
    let shade = spawn(&mut w, "SHADE", ALPHA, -6, -6);
    let plain = spawn(&mut w, "PLAIN", ALPHA, -6, 6);
    let vet = spawn(&mut w, "PLAIN", ALPHA, 6, -6);
    let crate_ = spawn(&mut w, "PLAIN", ALPHA, 6, 6);
    let wraith = spawn(&mut w, "WRAITH", ALPHA, 0, -8);
    let veil = spawn(&mut w, "VEIL", ALPHA, 0, 8);
    // (a) and the type flag of aircraft and buildings.
    assert!(w.obj(shade).unwrap().cloak.own);
    assert!(!w.obj(plain).unwrap().cloak.own);
    assert!(!w.obj(wraith).unwrap().cloak.own);
    assert!(!w.obj(veil).unwrap().cloak.own);
    // (b) a veteran with the ability; (d) the crate's flag.
    w.obj_mut(vet).unwrap().combat.experience = F32(ONE);
    machine::grant_own_flag(&mut w, crate_);
    run(&mut w, 40);
    assert_eq!(state(&w, shade), CloakState::Cloaked);
    assert_eq!(state(&w, plain), CloakState::Uncloaked);
    assert_eq!(state(&w, vet), CloakState::Cloaked);
    assert_eq!(state(&w, crate_), CloakState::Cloaked);
    assert_eq!(state(&w, wraith), CloakState::Uncloaked);
    assert_eq!(state(&w, veil), CloakState::Uncloaked);
    // (c) the owner's cloak field.
    let mut w = game(&r);
    let plain = spawn(&mut w, "PLAIN", ALPHA, 1, 0);
    spawn(&mut w, "GENNY", ALPHA, 0, 0);
    run(&mut w, 30);
    assert_eq!(state(&w, plain), CloakState::Cloaked);
}

// spec: S-0516 claim 3 (cloak-stop: the own flag starts a cloak only while the locomotor is not moving; once cloaked, moving does not uncloak)
#[test]
fn claim_3_cloak_stop() {
    let mut r = ruleset();
    r.sim.infantry.get_mut("SHADE").unwrap().combat.cloak_stop = Some(true);
    let mut w = game(&r);
    let id = spawn(&mut w, "SHADE", ALPHA, -6, 0);
    // The order takes effect after this tick's turn pass: hold the cloak for
    // that turn.
    let t = w.tick;
    w.obj_mut(id).unwrap().cloak.delay_until = t + 5;
    move_to(&mut w, id, at(6, 0));
    let mut moved_uncloaked = 0;
    for _ in 0..400 {
        run(&mut w, 1);
        if !glasnost_sim::occupancy::in_progress(&w, id) {
            if moved_uncloaked > 0 {
                break;
            }
            continue;
        }
        assert_eq!(
            state(&w, id),
            CloakState::Uncloaked,
            "no cloak while moving"
        );
        moved_uncloaked += 1;
    }
    assert!(moved_uncloaked > 10);
    run(&mut w, 30);
    assert_eq!(state(&w, id), CloakState::Cloaked);
    move_to(&mut w, id, at(-6, 0));
    for _ in 0..60 {
        run(&mut w, 1);
        assert_eq!(
            state(&w, id),
            CloakState::Cloaked,
            "moving keeps it cloaked"
        );
    }
}

// spec: S-0516 claim 4 (not ready: fully cloaked, a running rearm countdown, a target in range, a non-building at a stage other than 0, a running cloak delay, a height of 1 lepton or more)
#[test]
fn claim_4_ready_to_cloak() {
    let r = ruleset();
    let mut w = game(&r);
    let id = spawn(&mut w, "SHADE", ALPHA, 0, 0);
    assert!(machine::ready(&w, id));
    let t = w.tick;
    w.obj_mut(id).unwrap().combat.rearm_until = t + 50;
    assert!(!machine::ready(&w, id), "rearm countdown");
    w.obj_mut(id).unwrap().combat.rearm_until = 0;
    w.obj_mut(id).unwrap().cloak.delay_until = t + 50;
    assert!(!machine::ready(&w, id), "cloak delay");
    w.obj_mut(id).unwrap().cloak.delay_until = 0;
    w.obj_mut(id).unwrap().cloak.state = CloakState::Uncloaking;
    w.obj_mut(id).unwrap().cloak.stage = 3;
    assert!(!machine::ready(&w, id), "a stage other than 0");
    force_uncloaked(&mut w, id);
    force_cloaked(&mut w, id);
    assert!(!machine::ready(&w, id), "fully cloaked");
    force_uncloaked(&mut w, id);
    assert!(machine::ready(&w, id));
    // A target within range of the armed copier type.
    let mut r = ruleset();
    r.sim.infantry.get_mut("COPIER").unwrap().combat.cloakable = Some(true);
    let mut w = game(&r);
    let c = spawn(&mut w, "COPIER", ALPHA, 0, 0);
    let near = spawn(&mut w, "PLAIN", BETA, 2, 0);
    let far = spawn(&mut w, "PLAIN", BETA, 8, 0);
    w.obj_mut(c).unwrap().target = Some(Target::Object(far));
    assert!(machine::ready(&w, c), "a target out of range");
    w.obj_mut(c).unwrap().target = Some(Target::Object(near));
    assert!(!machine::ready(&w, c), "a target in range");
    // Height: an aircraft off the ground.
    let mut w = game(&r);
    let a = spawn(&mut w, "WRAITH", ALPHA, 0, 0);
    machine::grant_own_flag(&mut w, a);
    assert!(machine::ready(&w, a), "landed");
    w.obj_mut(a).unwrap().pos.z += 1;
    assert!(!machine::ready(&w, a), "1 lepton up");
}

// spec: S-0516 claim 5 (b) and (d) (no start in contact with a war factory; below the red ratio one R(0, 99) per tick, starting below 4)
#[test]
fn claim_5_starting_by_itself() {
    let r = ruleset();
    // (b) contact with a war factory.
    let mut w = game(&r);
    let warf = spawn(&mut w, "WARF", ALPHA, 1, 0);
    let id = spawn(&mut w, "SHADE", ALPHA, 0, 0);
    for _ in 0..20 {
        w.obj_mut(id).unwrap().contact = Some(warf);
        run(&mut w, 1);
        assert_eq!(state(&w, id), CloakState::Uncloaked, "war-factory contact");
    }
    w.obj_mut(id).unwrap().contact = None;
    run(&mut w, 1);
    assert_eq!(state(&w, id), CloakState::Cloaking);
    // (d) above red starts on the first turn; at health 1 it waits for a draw.
    let mut w = game(&r);
    let well = spawn(&mut w, "SHADE", ALPHA, 0, 0);
    let low = spawn(&mut w, "SHADE", ALPHA, 4, 0);
    set_health(&mut w, low, 1);
    run(&mut w, 1);
    assert_eq!(state(&w, well), CloakState::Cloaking);
    let mut waited = 1;
    while state(&w, low) == CloakState::Uncloaked {
        assert!(waited < 2000, "a start below red eventually");
        run(&mut w, 1);
        waited += 1;
    }
    assert!(waited > 1, "below red does not start at once (seeded draw)");
}

// spec: S-0516 claim 6 (inside the owner's cloak field a ready object starts at once: no health test, no draw)
#[test]
fn claim_6_cloaking_inside_own_field() {
    let r = ruleset();
    let mut w = game(&r);
    let genny = spawn(&mut w, "GENNY", ALPHA, 0, 0);
    let p = spawn(&mut w, "PLAIN", ALPHA, 2, 0);
    let foe = spawn(&mut w, "PLAIN", BETA, 0, 2);
    set_health(&mut w, p, 1);
    let mut covered_at = None;
    let mut started_at = None;
    for n in 1..=10 {
        run(&mut w, 1);
        if covered_at.is_none() && field_at(&w, ALPHA, 2, 0) {
            covered_at = Some(n);
        }
        if started_at.is_none() && state(&w, p) != CloakState::Uncloaked {
            started_at = Some(n);
        }
    }
    let _ = genny;
    assert!(covered_at.is_some());
    assert_eq!(
        started_at, covered_at,
        "starts in the tick its cell is covered"
    );
    assert_eq!(
        state(&w, foe),
        CloakState::Uncloaked,
        "another house's field"
    );
}

// spec: S-0516 claim 7 and F1, F2, F3 (fully cloaked c·S ticks after the start, c = ⌈N/2⌉; uncloaking from N − 1 takes (N − 1)·S; then the cloak delay D = trunc(d × 900))
#[test]
fn claim_7_durations() {
    assert_eq!(cloak::cloaked_stage(STAGES), 4);
    assert_eq!(cloak::cloak_time(STAGES, SPEED), 8);
    assert_eq!(cloak::uncloak_time(STAGES, SPEED), 14);
    let r = ruleset();
    let mut w = game(&r);
    assert_eq!(w.rules().cloak.delay, DELAY);
    let id = spawn(&mut w, "SHADE", ALPHA, 0, 0);
    let t = transitions(&mut w, id, 20);
    assert_eq!(t[0].1, CloakState::Cloaking);
    assert_eq!(t[1], (t[0].0 + 8, CloakState::Cloaked));
    hit(&mut w, id, 1);
    assert_eq!(stage(&w, id), STAGES - 1, "uncloaking starts at N − 1");
    let t = transitions(&mut w, id, 40);
    assert_eq!(t[0], (14, CloakState::Uncloaked));
    assert_eq!(t[1], (14 + DELAY, CloakState::Cloaking));
}

// spec: S-0516 claim 8 (in the 50 % band at or below red, one R(0, 99) per tick; below 10 starts uncloaking from N − 1, silently)
#[test]
fn claim_8_abort_at_low_health() {
    let r = ruleset();
    let mut w = game(&r);
    let id = spawn(&mut w, "SHADE", ALPHA, 0, 0);
    set_health(&mut w, id, 1);
    let mut aborted = false;
    for _ in 0..300 {
        let t = w.tick;
        let o = w.obj_mut(id).unwrap();
        o.cloak.state = CloakState::Cloaking;
        o.cloak.stage = 2; // x = 256 · 2 / 8 = 64
        o.cloak.stepped = t;
        w.effects.clear();
        run(&mut w, 1);
        if state(&w, id) == CloakState::Uncloaking {
            assert_eq!(stage(&w, id), STAGES - 1);
            assert_eq!(cloak_sounds(&w), 0, "the abort is silent");
            aborted = true;
            break;
        }
    }
    assert!(aborted);
    // Above red it never aborts.
    let mut w = game(&r);
    let id = spawn(&mut w, "SHADE", ALPHA, 0, 0);
    for _ in 0..300 {
        let t = w.tick;
        let o = w.obj_mut(id).unwrap();
        o.cloak.state = CloakState::Cloaking;
        o.cloak.stage = 2;
        o.cloak.stepped = t;
        run(&mut w, 1);
        assert_ne!(state(&w, id), CloakState::Uncloaking);
    }
}

// spec: S-0516 claim 9 (a cloaked object stays cloaked through its own flag or its owner's field; a field-only object uncloaks, with the sound, when the field shrinks away)
#[test]
fn claim_9_what_keeps_it_cloaked() {
    let r = ruleset();
    let mut w = game(&r);
    let genny = spawn(&mut w, "GENNY", ALPHA, 0, 0);
    let p = spawn(&mut w, "PLAIN", ALPHA, 2, 0);
    let s = spawn(&mut w, "SHADE", ALPHA, -8, 0);
    run(&mut w, 30);
    assert_eq!(state(&w, p), CloakState::Cloaked);
    assert_eq!(state(&w, s), CloakState::Cloaked);
    switch_off(&mut w, genny);
    w.effects.clear();
    let t = transitions(&mut w, p, 6);
    assert_eq!(t.first().map(|x| x.1), Some(CloakState::Uncloaking));
    assert!(cloak_sounds(&w) >= 1);
    run(&mut w, 50);
    assert_eq!(
        state(&w, s),
        CloakState::Cloaked,
        "own flag outside a field"
    );
}

// spec: S-0516 claim 10 (an uncloaking unit completes its uncloak and waits out the delay before a new cloak, even inside its own field)
#[test]
fn claim_10_no_recloak_while_uncloaking() {
    let r = ruleset();
    let mut w = game(&r);
    spawn(&mut w, "GENNY", ALPHA, 0, 0);
    let p = spawn(&mut w, "PLAIN", ALPHA, 1, 0);
    run(&mut w, 30);
    assert_eq!(state(&w, p), CloakState::Cloaked);
    hit(&mut w, p, 1);
    let t = transitions(&mut w, p, 40);
    assert_eq!(t[0], (14, CloakState::Uncloaked));
    assert_eq!(t[1], (14 + DELAY, CloakState::Cloaking));
}

// spec: S-0516 claim 11 (a), (c), (e) (firing with a weapon that answers cloaked, a vehicle crushing, and a blocked mover reveal every cloaked object of the blocking cell)
#[test]
fn claim_11_uncloak_causes() {
    let r = ruleset();
    // (a) firing.
    let mut w = game(&r);
    let g = spawn_cloaked(&mut w, "GUNSHIP", ALPHA, 0, 0);
    let foe = spawn(&mut w, "PLAIN", BETA, 3, 0);
    assert_eq!(state(&w, g), CloakState::Cloaked);
    assert!(attack(&mut w, g, foe));
    let t = transitions(&mut w, g, 30);
    assert_eq!(t.first().map(|x| x.1), Some(CloakState::Uncloaking));
    // (c) crushing: a vehicle, not an infantry unit.
    let mut w = game(&r);
    let v = spawn_cloaked(&mut w, "GHOST", ALPHA, 0, 0);
    let i = spawn(&mut w, "SHADE", ALPHA, 4, 0);
    force_cloaked(&mut w, i);
    machine::crushed(&mut w, v);
    machine::crushed(&mut w, i);
    assert_eq!(state(&w, v), CloakState::Uncloaking);
    assert_eq!(state(&w, i), CloakState::Cloaked);
    // (e) a blocked mover.
    let mut w = game(&r);
    let hidden = spawn_cloaked(&mut w, "GHOST", BETA, 3, 0);
    let mover = spawn(&mut w, "PLAIN", ALPHA, -3, 0);
    move_to(&mut w, mover, at(3, 0));
    let t = transitions(&mut w, hidden, 200);
    assert_eq!(t.first().map(|x| x.1), Some(CloakState::Uncloaking));
}

// spec: S-0516 claim 12 and quirks.heal_reveals_cloaked (a surviving hit uncloaks a cloaked or cloaking object: a zero hit and healing included; an uncloaked object is unaffected)
#[test]
fn claim_12_hits_uncloak() {
    let r = ruleset();
    let mut w = game(&r);
    let a = spawn_cloaked(&mut w, "GHOST", ALPHA, -4, 0);
    let b = spawn_cloaked(&mut w, "GHOST", ALPHA, 0, 0);
    let c = spawn(&mut w, "SHADE", ALPHA, 4, 0);
    let d = spawn(&mut w, "PLAIN", ALPHA, 0, 4);
    set_health(&mut w, b, 100);
    hit(&mut w, a, 0);
    hit(&mut w, b, -10);
    assert_eq!(state(&w, a), CloakState::Uncloaking, "zero hit");
    assert_eq!(state(&w, b), CloakState::Uncloaking, "healing");
    run(&mut w, 1);
    assert_eq!(state(&w, c), CloakState::Cloaking);
    hit(&mut w, c, 5);
    assert_eq!(state(&w, c), CloakState::Uncloaking, "a cloaking object");
    hit(&mut w, d, 5);
    assert_eq!(state(&w, d), CloakState::Uncloaked);
}

// spec: S-0516 claim 13 (a cloaked mover entering a cell next to a non-allied sensor unit uncloaks; a sensor walking up to a still cloaked object reveals nothing)
#[test]
fn claim_13_adjacent_sensor_reveal() {
    let r = ruleset();
    let mut w = game(&r);
    let g = spawn_cloaked(&mut w, "GHOST", ALPHA, -5, 0);
    spawn(&mut w, "SEEKER", BETA, 0, 1);
    move_to(&mut w, g, at(5, 0));
    let t = transitions(&mut w, g, 200);
    assert_eq!(t.first().map(|x| x.1), Some(CloakState::Uncloaking));
    // An allied sensor does not reveal.
    let mut w = game(&r);
    ally(&mut w, ALPHA, BETA);
    ally(&mut w, BETA, ALPHA);
    let g = spawn_cloaked(&mut w, "GHOST", ALPHA, -5, 0);
    spawn(&mut w, "SEEKER", BETA, 0, 1);
    move_to(&mut w, g, at(5, 0));
    assert!(transitions(&mut w, g, 200).is_empty());
    // A sensor that walks up to a still cloaked object.
    let mut w = game(&r);
    let g = spawn_cloaked(&mut w, "GHOST", ALPHA, 0, 0);
    let s = spawn(&mut w, "SEEKER", BETA, 6, 1);
    move_to(&mut w, s, at(1, 1));
    assert!(transitions(&mut w, g, 200).is_empty());
}

// spec: S-0516 claim 14 (at the start of a cloak, attackers keep the target only when of the same owner or when their house senses its cell)
#[test]
fn claim_14_targets_when_cloaking() {
    let r = ruleset();
    let mut w = game(&r);
    let s = spawn(&mut w, "SHADE", ALPHA, 0, 0);
    let foe = spawn(&mut w, "PLAIN", BETA, 6, 0);
    let own = spawn(&mut w, "PLAIN", ALPHA, -6, 0);
    for x in [foe, own] {
        w.obj_mut(x).unwrap().target = Some(Target::Object(s));
    }
    run(&mut w, 1);
    assert_eq!(state(&w, s), CloakState::Cloaking);
    assert_eq!(
        w.obj(foe).unwrap().target,
        None,
        "a non-sensing foe drops it"
    );
    assert_eq!(
        w.obj(own).unwrap().target,
        Some(Target::Object(s)),
        "same owner"
    );
    // The attacker's house senses the cell: kept through the start and the
    // full cloak.
    let mut w = game(&r);
    let s = spawn(&mut w, "SHADE", ALPHA, 0, 0);
    let foe = spawn(&mut w, "PLAIN", BETA, 6, 0);
    spawn(&mut w, "SEEKER", BETA, 2, 0);
    w.obj_mut(foe).unwrap().target = Some(Target::Object(s));
    run(&mut w, 12);
    assert_eq!(state(&w, s), CloakState::Cloaked);
    assert_eq!(
        w.obj(foe).unwrap().target,
        Some(Target::Object(s)),
        "sensed"
    );
    // A foe whose target is set while it is cloaking drops it at the full
    // cloak.
    let mut w = game(&r);
    let s = spawn(&mut w, "SHADE", ALPHA, 0, 0);
    let foe = spawn(&mut w, "PLAIN", BETA, 6, 0);
    run(&mut w, 2);
    assert_eq!(state(&w, s), CloakState::Cloaking);
    w.obj_mut(foe).unwrap().target = Some(Target::Object(s));
    run(&mut w, 10);
    assert_eq!(state(&w, s), CloakState::Cloaked);
    assert_eq!(
        w.obj(foe).unwrap().target,
        None,
        "dropped at the full cloak"
    );
}

// spec: S-0516 claim 14 (the start of a cloak: a non-sensing foe drops the target); S-0215 claim 10 (an override's end puts the earlier target back); S-0107 r46 claim 11, Changes of target (the cloak drop's loss of the target, or the earlier target put back, ends an infantry holder's attack state and makes its request, not forced, for stand, at once) (seam sweep 6: intake 107's reverts row)
#[test]
fn seams6_a_cloak_drop_or_revert_ends_an_infantry_holders_attack_state() {
    use glasnost_sim::infantry::Seq;
    use glasnost_sim::rules::Behaviour;
    let mut r = ruleset();
    let walker = r.sim.infantry.get_mut("WALKER").unwrap();
    for k in ["stand", "fire_standing"] {
        walker.sequences.insert(
            k.into(),
            glasnost_rules::sim::SequenceEntry {
                start: 0,
                count: 6,
                multiplier: 0,
                direction: None,
            },
        );
    }
    for revert in [false, true] {
        let mut w = game(&r);
        let s = spawn(&mut w, "SHADE", BETA, 0, 0);
        let man = spawn(&mut w, "WALKER", ALPHA, -6, 0);
        let earlier = spawn(&mut w, "PLAIN", BETA, 9, 0);
        let t = w.tick;
        let o = w.obj_mut(man).unwrap();
        o.target = Some(Target::Object(s));
        o.beh.current = Some(Behaviour::Attack);
        o.beh.queued = None;
        if revert {
            o.beh.suspended = Some(Behaviour::Guard);
            o.beh.aside = Some((Some(Target::Object(earlier)), None));
        }
        let st = o.infantry.as_mut().unwrap();
        st.watch = Some(Target::Object(s));
        st.attack = true;
        st.seq = Some(Seq::FireStanding);
        st.seq_delay = 3;
        st.seq_t0 = t;
        run(&mut w, 1);
        assert_eq!(state(&w, s), CloakState::Cloaking);
        let o = w.obj(man).unwrap();
        let st = o.infantry.as_ref().unwrap();
        if revert {
            assert_eq!(o.target, Some(Target::Object(earlier)), "put back");
            assert_eq!(o.beh.current, Some(Behaviour::Guard), "the suspended one");
        } else {
            assert_eq!(o.target, None, "dropped");
        }
        assert!(!st.attack, "the attack state ends (revert {revert})");
        assert_eq!(st.seq, Some(Seq::Stand), "its request for stand");
    }
}

// spec: S-0516 claim 15 (the cloak sound at a self-start and at an uncloak cause; none for a repeated cause while uncloaking)
#[test]
fn claim_15_cloak_sound() {
    let r = ruleset();
    let mut w = game(&r);
    let s = spawn(&mut w, "SHADE", ALPHA, 0, 0);
    w.effects.clear();
    run(&mut w, 1);
    assert_eq!(state(&w, s), CloakState::Cloaking);
    assert_eq!(cloak_sounds(&w), 1);
    w.effects.clear();
    hit(&mut w, s, 1);
    assert_eq!(cloak_sounds(&w), 1);
    w.effects.clear();
    hit(&mut w, s, 1);
    assert_eq!(cloak_sounds(&w), 0, "repeated cause");
}

// spec: S-0516 claim 16 (ordinary placement within the usable area does not force a vehicle cloaked, even with its own flag; infantry and aircraft start uncloaked; only a vehicle with its own flag placed outside the usable area is forced fully cloaked, tested afresh at each placement)
#[test]
fn claim_16_vehicles_placed_cloaked() {
    let r = ruleset();
    let mut w = game(&r);
    let v = spawn(&mut w, "GHOST", ALPHA, 0, 0);
    let i = spawn(&mut w, "SHADE", ALPHA, 3, 0);
    let a = spawn(&mut w, "WRAITH", ALPHA, -3, 0);
    machine::grant_own_flag(&mut w, a);
    assert_eq!(state(&w, v), CloakState::Uncloaked);
    assert_eq!(state(&w, i), CloakState::Uncloaked);
    assert_eq!(state(&w, a), CloakState::Uncloaked);
    // Outside the usable area: forced fully cloaked; a vehicle without the
    // flag and infantry are not.
    // A usable cell of the top row of the usable area, raised one level so
    // that it falls outside it (S-0009 claim 4).
    let width = i64::from(w.cells.footprint().width);
    let outside = w
        .cells
        .grid
        .iter()
        .map(|(c, _)| c)
        .find(|&c| w.cells.usable(c) && c.s() == width + 1)
        .expect("a cell of the usable area's top row");
    w.cells.grid.get_mut(outside).unwrap().level = 1;
    assert!(!w.cells.usable(outside));
    let place = |w: &mut glasnost_sim::world::World, name: &str| {
        use glasnost_sim::rules::Kind;
        let kind = [Kind::Vehicle, Kind::Infantry]
            .into_iter()
            .find(|&k| w.rules().index_of(k, name).is_some())
            .unwrap();
        let ti = w.rules().index_of(kind, name).unwrap();
        let id = glasnost_sim::objects::create(w, kind, ti, Some(ALPHA));
        let p = glasnost_sim::world::Pos::centre(outside, 0);
        let z = w.surface_at(p.x, p.y);
        glasnost_sim::objects::place(w, id, glasnost_sim::world::Pos::new(p.x, p.y, z), 0);
        id
    };
    let v = place(&mut w, "GHOST");
    let p = place(&mut w, "PLAIN");
    let s = place(&mut w, "SHADE");
    assert_eq!(state(&w, v), CloakState::Cloaked);
    assert_eq!(stage(&w, v), 0);
    assert_eq!(state(&w, p), CloakState::Uncloaked);
    assert_eq!(state(&w, s), CloakState::Uncloaked);
}

// spec: S-0516 claim 16 (boarding and unloading preserve the passenger's cloak state and stage: an uncloaked self-cloaking vehicle unloads uncloaked, a cloaked one cloaked)
#[test]
fn claim_16_exit_keeps_the_state() {
    let r = ruleset();
    let mut w = game(&r);
    let v = spawn(&mut w, "GHOST", ALPHA, 0, 0);
    machine::entered_map(&mut w, v);
    assert_eq!(state(&w, v), CloakState::Uncloaked);
    let c = spawn_cloaked(&mut w, "GHOST", ALPHA, 4, 0);
    machine::entered_map(&mut w, c);
    assert_eq!(state(&w, c), CloakState::Cloaked);
}

// spec: S-0516 claim 17 (an aircraft cloaks only while landed; once airborne and fully cloaked it stays cloaked)
#[test]
fn claim_17_aircraft() {
    let r = ruleset();
    let mut w = game(&r);
    let a = spawn(&mut w, "WRAITH", ALPHA, 0, 0);
    let up = spawn(&mut w, "WRAITH", ALPHA, 4, 0);
    machine::grant_own_flag(&mut w, a);
    machine::grant_own_flag(&mut w, up);
    for _ in 0..20 {
        w.obj_mut(up).unwrap().pos.z =
            w.surface_at(w.obj(up).unwrap().pos.x, w.obj(up).unwrap().pos.y) + 100;
        run(&mut w, 1);
    }
    assert_eq!(state(&w, a), CloakState::Cloaked);
    assert_eq!(state(&w, up), CloakState::Uncloaked, "airborne");
    for _ in 0..20 {
        w.obj_mut(a).unwrap().pos.z =
            w.surface_at(w.obj(a).unwrap().pos.x, w.obj(a).unwrap().pos.y) + 100;
        run(&mut w, 1);
    }
    assert_eq!(state(&w, a), CloakState::Cloaked, "stays cloaked airborne");
}

// spec: S-0516 claim 18 (a building's fade level rises 1 per tick to 15 and falls 1 per tick to 0, no delay; a foreign sensor unit in the ring stops it)
#[test]
fn claim_18_buildings() {
    let mut r = ruleset();
    r.globals.quirks.cloak_flag_ground_mobile_only = false;
    let mut w = game(&r);
    let b = spawn(&mut w, "VEIL", ALPHA, 0, 0);
    assert!(w.obj(b).unwrap().cloak.own);
    let seen = stages_of(&mut w, b, 20);
    let first = seen
        .iter()
        .position(|&(s, _)| s == CloakState::Cloaking)
        .unwrap();
    for (i, &(s, k)) in seen[first..first + 15].iter().enumerate() {
        assert_eq!(s, CloakState::Cloaking);
        assert_eq!(k, i as i32);
    }
    assert_eq!(seen[first + 15], (CloakState::Cloaked, 15));
    hit(&mut w, b, 1);
    let t = transitions(&mut w, b, 20);
    assert_eq!(t[0], (15, CloakState::Uncloaked));
    assert_eq!(
        t[1],
        (16, CloakState::Cloaking),
        "no cloak delay for a building"
    );
    // The sensor ring.
    let mut w = game(&r);
    let b = spawn(&mut w, "VEIL", ALPHA, 0, 0);
    spawn(&mut w, "SEEKER", BETA, 2, 0);
    run(&mut w, 30);
    assert_eq!(
        state(&w, b),
        CloakState::Uncloaked,
        "a foreign sensor in the ring"
    );
}

// spec: S-0516 claim 41 (an underwater type with the cloak flag is submerged while not uncloaked; placed within the usable area it starts surfaced and submerges on an eligible update)
#[test]
fn claim_41_submerging() {
    let mut r = ruleset();
    let mut diver = r.sim.vehicles.get("GHOST").unwrap().clone();
    diver.combat.underwater = Some(true);
    r.sim.vehicles.insert("DIVER2".into(), diver);
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("DIVER2".into());
    let mut w = game(&r);
    let d = spawn(&mut w, "DIVER2", ALPHA, 0, 0);
    assert!(!glasnost_sim::naval::submerged(&w, d), "starts surfaced");
    run(&mut w, 1);
    assert_eq!(state(&w, d), CloakState::Cloaking);
    assert!(glasnost_sim::naval::submerged(&w, d), "begins submerging");
    force_cloaked(&mut w, d);
    hit(&mut w, d, 1);
    assert!(glasnost_sim::naval::submerged(&w, d), "surfacing");
    // The hit falls between ticks: the uncloak completes in the turn of the
    // 14th tick after the hit's tick, the 15th tick run.
    run(&mut w, 15);
    assert_eq!(state(&w, d), CloakState::Uncloaked);
    assert!(!glasnost_sim::naval::submerged(&w, d));
}

// spec: S-0209 claim 4 (rows WP21, WR2 and WR3: a cloaked enemy infantry unit gives cloaked blocker, then blocked to an unarmed walker and enemy obstacle to an armed one; a cloaked enemy vehicle stays cloaked blocker); S-0516 claim 11 (e)
#[test]
fn claim_11e_cloaked_infantry_walker_rows() {
    use glasnost_sim::occupancy::{self, Outcome};
    let r = ruleset();
    let mut w = game(&r);
    let unarmed = spawn(&mut w, "SHADE", ALPHA, 0, 0);
    w.obj_mut(unarmed).unwrap().cloak.own = false;
    let armed = spawn(&mut w, "WALKER", ALPHA, 0, 4);
    let foe_i = spawn(&mut w, "SHADE", BETA, 1, 0);
    let foe_i2 = spawn(&mut w, "SHADE", BETA, 1, 4);
    let foe_v = spawn_cloaked(&mut w, "GHOST", BETA, -1, 0);
    spawn_cloaked(&mut w, "GHOST", BETA, -1, 4);
    force_cloaked(&mut w, foe_i);
    force_cloaked(&mut w, foe_i2);
    let out = |w: &glasnost_sim::world::World, m, c| {
        occupancy::infantry_outcome(w, m, c, occupancy::reference_level(w, m))
    };
    assert_eq!(out(&w, unarmed, at(1, 0)), Outcome::Blocked);
    assert_eq!(out(&w, armed, at(1, 4)), Outcome::EnemyObstacle);
    assert_eq!(state(&w, foe_v), CloakState::Cloaked);
    assert_eq!(out(&w, unarmed, at(-1, 0)), Outcome::CloakedBlocker);
    assert_eq!(out(&w, armed, at(-1, 4)), Outcome::CloakedBlocker);
}

// spec: S-0516 claim 13 (the candidate distance is the table root of u² + w², u and w the position residues mod 256, height ignored; ties go to the first object in the cell-layer list)
#[test]
fn claim_13_candidate_distance_and_ties() {
    use glasnost_sim::world::Layer;
    let r = ruleset();
    let mut w = game(&r);
    let d = |w: &glasnost_sim::world::World, x, y| machine::sensor_candidate_distance(w, x, y);
    assert_eq!(d(&w, 0, 0), 0);
    assert_eq!(d(&w, 5 * 256 + 30, 7 * 256 + 40), d(&w, 30, 40));
    assert_eq!(d(&w, -1, 0), d(&w, 255, 0), "residues from 0 to 255");
    assert!(d(&w, 30, 40) <= 50 && d(&w, 30, 40) >= 49, "a floored root");
    // Two infantry at the same cell centre position: a tie, the first in
    // the list wins.
    let a = spawn(&mut w, "SHADE", BETA, 2, 0);
    let b = spawn(&mut w, "SHADE", BETA, 2, 0);
    let pa = w.obj(a).unwrap().pos;
    let pb = w.obj(b).unwrap().pos;
    let c = at(2, 0);
    let list = w.cells.list(c, Layer::Ground).to_vec();
    assert!(list.contains(&a) && list.contains(&b));
    if d(&w, pa.x, pa.y) == d(&w, pb.x, pb.y) {
        let first = *list.iter().find(|&&x| x == a || x == b).unwrap();
        assert_eq!(
            machine::nearest_top_corner(&w, c, Layer::Ground),
            Some(first)
        );
    }
}

// spec: S-0516 claim 43 (a negative stage speed advances one stage per ordinary update in both directions, as speed 1 does, not a stall; a nonpositive stage count imports as 1)
#[test]
fn claim_43_degenerate_inputs() {
    let run_with = |s: i32| {
        let mut r = ruleset();
        r.sim
            .infantry
            .get_mut("SHADE")
            .unwrap()
            .combat
            .cloaking_speed = Some(s);
        let mut w = game(&r);
        let id = spawn(&mut w, "SHADE", ALPHA, 0, 0);
        let mut t = transitions(&mut w, id, 60);
        t.truncate(2);
        force_cloaked(&mut w, id);
        hit(&mut w, id, 1);
        let u = transitions(&mut w, id, 60);
        (t, u.first().copied())
    };
    let neg = run_with(-5);
    assert_eq!(neg, run_with(1));
    assert_eq!(neg.0.last().map(|x| x.1), Some(CloakState::Cloaked));
    assert_eq!(neg.1.map(|x| x.1), Some(CloakState::Uncloaked));
    for n in [0, -3] {
        let mut r = ruleset();
        r.sim.general.cloaking_stages = Some(n);
        let w = game(&r);
        assert_eq!(w.rules().cloak.stages, 1);
    }
}
