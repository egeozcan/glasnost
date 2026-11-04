//! Claim tests of S-0513 revision 62, claim 39 (Q-2213 and Q-2211
//! answered) and claim 51 (M8 step A3 intake 71): the firing check's
//! *illegal* answer about a superweapon warp member from the launch until
//! its departure update, what S-0309 claim 29's periodic check does with
//! it, a firer whose temporal beam is attached to the member, a beam begun
//! before the launch, the override reverts, and the member's own periodic
//! check through its warp. Synthetic rules (`tests/sw_common`): every name
//! and number is invented. The ledger is
//! `docs/m8-claims/intake71-S-0513-S-0216.md`.
//!
//! A drop in tick *k* shows after `tick::step` has run that tick, when the
//! tick number reads *k* + 1. The launch order is applied after the turn
//! pass of tick *t*_L (S-0203 claim 7), so no turn of *t*_L sees the warp.

mod sw_common;

use glasnost_rules::schema::Ruleset;
use glasnost_sim::combat::check::{Answer, check};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::combat::held;
use glasnost_sim::combat::projectile;
use glasnost_sim::orders;
use glasnost_sim::rules::Behaviour;
use glasnost_sim::superweapons::warp::Stage;
use glasnost_sim::superweapons::{self, ProtectKind, protect};
use glasnost_sim::temporal;
use glasnost_sim::world::{ObjectId, Target, World};
use sw_common::*;

/// S-0309 claim 29's check period, written out for the expected ticks.
const PERIOD: i32 = 16;

fn rules() -> Ruleset {
    let mut r = ruleset();
    {
        let g = &mut r.sim.general.superweapons;
        g.chrono_placement = Some("MARK".into());
        g.chrono_blast = Some("BLASTS".into());
        g.chrono_blast_dest = Some("BLASTD".into());
        g.chrono_delay = Some(7);
    }
    r.sim.general.warp_out = Some("WARPA".into());
    for n in ["MARK", "BLASTS", "BLASTD", "WARPA"] {
        r.sim
            .animations
            .insert(n.into(), glasnost_rules::sim::AnimationType::default());
        r.registries
            .entry("animations".into())
            .or_default()
            .push(n.into());
    }
    r
}

/// The same rules with every weapon of warhead `HE` a temporal weapon
/// (S-0301 Terms: its warhead has the temporal flag).
fn temporal_rules() -> Ruleset {
    let mut r = rules();
    r.sim.warheads.get_mut("HE").unwrap().temporal = Some(true);
    r
}

fn stage(w: &World, id: ObjectId) -> Option<Stage> {
    w.obj(id).and_then(|o| o.sw.warp.as_ref()).map(|m| m.stage)
}

/// A game with ALPHA's teleport source ready.
fn ready(r: &Ruleset) -> World {
    let mut w = game(r);
    spawn(&mut w, "CHRONOB", ALPHA, -9, -9);
    ready_tick(&mut w, ALPHA, SOURCE, 300);
    w
}

/// Launches ALPHA's warp of the block at `at(0, 0)` to `at(5, 5)` in the
/// first tick from now whose number leaves `rem` when divided by 16 and in
/// which orders are taken; returns t_L, the tick that applied the warp
/// order, which has run.
fn launch_at(w: &mut World, rem: i32) -> i32 {
    fire_now(w, ALPHA, SOURCE, at(0, 0));
    while orders::blocked(w) || w.tick.rem_euclid(PERIOD) != rem {
        step(w);
    }
    fire(w, ALPHA, WARP, at(5, 5));
    let tl = w.tick;
    step(w);
    tl
}

/// Runs until tick `t` has run (the next tick to run is `t + 1`).
fn run_through(w: &mut World, t: i32) {
    while w.tick <= t {
        step(w);
    }
}

/// The first check tick after tick `n` (S-0309 claim 29, Phase).
fn next_check(n: i32) -> i32 {
    (n / PERIOD + 1) * PERIOD
}

fn target(w: &World, id: ObjectId) -> Option<Target> {
    w.obj(id).unwrap().target
}

/// Gives `id` target `x` and current behaviour `b` directly, as an order
/// or a search would, its targeting timer 200 ticks away.
fn hold(w: &mut World, id: ObjectId, x: ObjectId, b: Behaviour) {
    let t = w.tick;
    let o = w.obj_mut(id).unwrap();
    o.beh.force(b, t, false);
    o.target = Some(Target::Object(x));
    o.combat.opportunity = false;
    o.targeting_timer = t + 200;
}

fn shots_by(w: &World, id: ObjectId) -> usize {
    w.effects
        .iter()
        .filter(|e| matches!(e, Effect::Shot { firer, .. } if *firer == id))
        .count()
}

// --- The answer before the member's first warp update -----------------------------

#[test]
fn claim39_the_turns_of_t_l_plus_1_before_the_first_update_answer_illegal() {
    // spec: S-0513 r62 claim 39 (from the launch until the member's departure update a firing check about the member answers illegal for every weapon; this covers the turns of tick t_L + 1 that come before the member's first warp update) and its Changes for S-0301 (rank 4, ahead of the rearm countdown of rank 5)
    let mut w = ready(&rules());
    let h = spawn(&mut w, "CART", ALPHA, 0, 3);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    let other = spawn(&mut w, "BUGGY", BETA, 3, 3);
    let tl = launch_at(&mut w, 3);
    assert_eq!(w.tick, tl + 1);
    assert_eq!(
        stage(&w, m),
        Some(Stage::Pending),
        "before its first update"
    );
    let t = w.tick;
    w.obj_mut(h).unwrap().combat.rearm_until = t + 100;
    assert_eq!(
        check(&w, h, Some(Target::Object(m)), 0, true),
        Answer::Illegal
    );
    assert_eq!(
        check(&w, h, Some(Target::Object(other)), 0, true),
        Answer::Rearming,
        "the control target, in no warp"
    );
}

#[test]
fn claim39_a_holder_whose_check_tick_is_t_l_plus_1_drops_the_member_there() {
    // spec: S-0513 r62 claim 39 (a holder covered by S-0309 claim 29 drops the member at its first checking turn in the check ticks after the launch; the answer holds in t_L + 1 before the member's first update, so a check tick t_L + 1 is the first check tick) and S-0309 r70 claim 29 (no draw)
    let mut w = ready(&rules());
    let h = spawn(&mut w, "CART", ALPHA, 0, 3);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    let tl = launch_at(&mut w, PERIOD - 1);
    hold(&mut w, h, m, Behaviour::Guard);
    let timer = w.obj(h).unwrap().targeting_timer;
    assert!(held::check_tick(tl + 1));
    step(&mut w);
    assert_eq!(target(&w, h), None, "dropped in tick t_L + 1");
    assert_eq!(w.obj(h).unwrap().targeting_timer, timer, "no draw");
    assert!(matches!(stage(&w, m), Some(Stage::Leaving(_))));
}

// --- SW12 part j: holders in attack and guard ---------------------------------------

#[test]
fn claim39_sw12_j_holders_in_attack_and_guard_drop_at_the_first_check_tick() {
    // spec: S-0513 r62 claim 39 (every holder covered by S-0309 claim 29, of any kind, attack included, drops the member at its first checking turn in the check ticks after the launch, with no draw over 4 to 8 ticks; before that tick an infantry holder whose weapon does not heal keeps it in its firing step and makes no shot at it) and SW12 part j (infantry and vehicle holders in guard and attack: the same drop at the first check tick and no shot)
    for rem in [1, 5, 9, 13] {
        let mut w = ready(&rules());
        let hv_a = spawn(&mut w, "CART", ALPHA, 0, 3);
        let hv_g = spawn(&mut w, "CART", ALPHA, 0, -3);
        let hi_a = spawn(&mut w, "WALKER", ALPHA, 3, 0);
        let hi_g = spawn(&mut w, "WALKER", ALPHA, -3, 0);
        let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
        let tl = launch_at(&mut w, rem);
        let holders = [
            (hv_a, Behaviour::Attack),
            (hv_g, Behaviour::Guard),
            (hi_a, Behaviour::Attack),
            (hi_g, Behaviour::Guard),
        ];
        for (h, b) in holders {
            hold(&mut w, h, m, b);
        }
        let timers: Vec<i32> = holders
            .iter()
            .map(|&(h, _)| w.obj(h).unwrap().targeting_timer)
            .collect();
        w.effects.clear();
        let c = next_check(tl);
        assert!(c > tl && c <= tl + PERIOD);
        while w.tick < c {
            step(&mut w);
            for (h, b) in holders {
                assert_eq!(
                    target(&w, h),
                    Some(Target::Object(m)),
                    "{rem} {b:?}: kept to the check tick (tick {})",
                    w.tick - 1
                );
            }
        }
        step(&mut w);
        for (i, (h, b)) in holders.into_iter().enumerate() {
            assert_eq!(target(&w, h), None, "{rem} {b:?}: dropped in tick {c}");
            // spec: S-0513 r62 claim 39 (none of these drops is S-0309 claim 27's or takes a draw over 4 to 8 ticks)
            assert_eq!(w.obj(h).unwrap().targeting_timer, timers[i], "{rem}");
            assert_eq!(shots_by(&w, h), 0, "{rem} {b:?}: no shot");
        }
        assert!(matches!(stage(&w, m), Some(Stage::Leaving(_))));
    }
}

#[test]
fn claim39_a_building_holder_drops_the_member_at_the_first_check_tick() {
    // spec: S-0513 r62 claim 39 (every holder covered by S-0309 claim 29, of any kind, drops the member at its first checking turn in the check ticks after the launch, with no draw over 4 to 8 ticks) and S-0309 claim 29 (a building holding a target in attack or guard makes the check); the building's own attack processing (S-0215 claim 32) is held back so the periodic drop shows alone (M8 seam sweep 5a)
    for (rem, b) in [(1, Behaviour::Guard), (9, Behaviour::Attack)] {
        let mut w = ready(&rules());
        let h = spawn(&mut w, "TOWER", ALPHA, 0, 3);
        let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
        let tl = launch_at(&mut w, rem);
        hold(&mut w, h, m, b);
        let t = w.tick;
        w.obj_mut(h).unwrap().beh.processed(t, 1000);
        let timer = w.obj(h).unwrap().targeting_timer;
        w.effects.clear();
        let c = next_check(tl);
        while w.tick < c {
            step(&mut w);
            assert_eq!(
                target(&w, h),
                Some(Target::Object(m)),
                "{b:?}: kept to the check tick (tick {})",
                w.tick - 1
            );
        }
        step(&mut w);
        assert_eq!(target(&w, h), None, "{b:?}: dropped in tick {c}");
        assert_eq!(w.obj(h).unwrap().targeting_timer, timer, "{b:?}: no draw");
        assert_eq!(shots_by(&w, h), 0, "{b:?}: no shot");
    }
}

#[test]
fn claim39_an_attacking_holder_stops_its_attack_at_its_next_processing() {
    // spec: S-0513 r62 claim 39 (an attacking holder then stops its attack at its next processing, S-0309 claim 29) and S-0309 r70 claim 29 (the drop itself changes no behaviour)
    let mut w = ready(&rules());
    let h = spawn(&mut w, "CART", ALPHA, 0, 3);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    let tl = launch_at(&mut w, 6);
    hold(&mut w, h, m, Behaviour::Attack);
    let t = w.tick;
    w.obj_mut(h).unwrap().beh.processed(t, 1000);
    run_through(&mut w, next_check(tl));
    assert_eq!(target(&w, h), None);
    assert_eq!(
        w.obj(h).unwrap().beh.current,
        Some(Behaviour::Attack),
        "the drop itself changes no behaviour"
    );
    w.obj_mut(h).unwrap().beh.expired = true;
    step(&mut w);
    assert_ne!(w.obj(h).unwrap().beh.current, Some(Behaviour::Attack));
}

// --- Temporal weapons and beams ---------------------------------------------------------

#[test]
fn claim39_a_temporal_firer_without_a_beam_drops_the_member_at_the_check_tick() {
    // spec: S-0513 r62 claim 39 (illegal for every weapon, a temporal weapon included: the test is made on the pending warp itself and makes no exception for a temporal weapon; Q-2213) and S-0309 r70 claim 29 (an illegal answer drops the target at the check tick)
    let mut w = ready(&temporal_rules());
    let h = spawn(&mut w, "CART", ALPHA, 0, 3);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    // h makes no shot before the launch: a temporal shot in flight at the launch locks a beam
    // on the member at its detonation (S-0513 r86 claim 39a), and h would then get S-0514 claim 19's
    // rearming. The countdown itself ranks below the warp-pending answer.
    let t0 = w.tick;
    w.obj_mut(h).unwrap().combat.rearm_until = t0 + 1000;
    let tl = launch_at(&mut w, 2);
    hold(&mut w, h, m, Behaviour::Guard);
    assert_eq!(
        check(&w, h, Some(Target::Object(m)), 0, false),
        Answer::Illegal
    );
    run_through(&mut w, next_check(tl) - 1);
    assert_eq!(target(&w, h), Some(Target::Object(m)));
    step(&mut w);
    assert_eq!(target(&w, h), None, "dropped at the first check tick");
    assert!(!temporal::erased(&w, m), "no beam was locked on it");
}

#[test]
fn claim39_a_firer_whose_beam_holds_the_member_gets_rearming_and_keeps_it() {
    // spec: S-0513 r62 claim 39 (a firer that has a temporal beam attached to the member gets S-0514 claim 19's rearming, which ranks above; for a firer with no beam attached to it, another firer's beam on the member does not change the answer) and S-0309 r70 claim 29 (a rearming answer keeps the target at the check tick); S-0513 r86 claim 39a (The firer; The erasure goes on: removal at the k-th update, r = d + k − 1 for a lock before the member's turn of tick d)
    let mut w = ready(&temporal_rules());
    let h = spawn(&mut w, "CART", ALPHA, 0, 3);
    let h2 = spawn(&mut w, "CART", ALPHA, 3, 0);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    let tl = launch_at(&mut w, 4);
    // The firers' own opportunity shots at the member, made before the launch, are taken out of
    // play: a detonation would lock a beam again (S-0514 claim 4) and restart the amount.
    let shells: Vec<ObjectId> = w.projectiles.keys().copied().collect();
    for p in shells {
        projectile::remove(&mut w, p);
    }
    // A beam on the member after the launch: this delivery stands for the detonation of a
    // temporal projectile in flight at the launch (S-0513 r86 claim 39a, The lock routes; the
    // pending warp is no refusal of S-0514 claim 3), made before the member's turn of t_L + 1.
    temporal::deliver(&mut w, Some(h), Some(Target::Object(m)));
    assert!(temporal::holds(&w, h, m));
    hold(&mut w, h, m, Behaviour::Guard);
    hold(&mut w, h2, m, Behaviour::Guard);
    // The temporal beam is held by `h`: S-0514 claim 19's rearming, for the warp's whole span.
    assert_eq!(
        check(&w, h, Some(Target::Object(m)), 0, false),
        Answer::Rearming
    );
    assert_eq!(
        check(&w, h2, Some(Target::Object(m)), 0, false),
        Answer::Illegal,
        "another firer's beam on the member does not change the answer"
    );
    let amount = w.temporal.locks[&m].amount;
    run_through(&mut w, next_check(tl));
    assert_eq!(target(&w, h), Some(Target::Object(m)), "kept: rearming");
    assert_eq!(target(&w, h2), None, "dropped: illegal");
    assert!(matches!(stage(&w, m), Some(Stage::Leaving(_))));
    // S-0513 r86 claim 39a (The erasure goes on): an erasure update in each member turn, from
    // t_L + 1, before its warp step; the Gun's raw damage is 30 (S-0514 claim 7).
    assert_eq!(
        w.temporal.locks[&m].amount,
        amount - 30 * (next_check(tl) - tl)
    );
    // R₀ = 1200 (S-0514 F1): k = 40 updates; a lock before the member's turn of d = t_L + 1
    // removes it in d + k − 1 = t_L + 40, in its leaving stage.
    run_through(&mut w, tl + 39);
    assert!(w.obj(m).is_some());
    run_through(&mut w, tl + 40);
    assert!(w.obj(m).is_none(), "removed in t_L + 40");
}

#[test]
fn claim39_a_beam_begun_before_the_launch_keeps_the_object_out_of_the_group() {
    // spec: S-0513 r62 claim 39 (a beam begun before the launch never holds a member: an object on which a temporal beam is locked is being erased, S-0514 claim 4 and the glossary, so claim 34 leaves it in place, out of the group) and claim 34
    let mut w = ready(&temporal_rules());
    let h = spawn(&mut w, "CART", ALPHA, 0, 3);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    let n = spawn(&mut w, "BUGGY", BETA, 1, 0);
    temporal::deliver(&mut w, Some(h), Some(Target::Object(m)));
    assert!(temporal::erased(&w, m));
    assert!(superweapons::being_erased(&w, m));
    launch_at(&mut w, 0);
    assert_eq!(stage(&w, m), None, "left in place, out of the group");
    assert_eq!(stage(&w, n), Some(Stage::Pending), "the control joins");
}

// --- Overrides --------------------------------------------------------------------------

/// Puts `h` into an override: guard suspended with `aside` put aside,
/// attack on `m` current.
fn overridden(w: &mut World, h: ObjectId, m: ObjectId, aside: ObjectId) {
    hold(w, h, aside, Behaviour::Guard);
    let o = w.obj_mut(h).unwrap();
    o.beh.aside = Some((Some(Target::Object(aside)), None));
    assert!(o.beh.override_with(Behaviour::Attack, false, false));
    o.target = Some(Target::Object(m));
}

#[test]
fn claim39_the_periodic_drop_of_the_member_reverts_no_override() {
    // spec: S-0513 r62 claim 39 and its Changes for S-0215 claim 10 (an override reverts only at that claim's causes; a drop by the holder's own processing, a leash or an order does not revert it) and S-0309 r70 claim 29 (the drop: no revert)
    let mut w = ready(&rules());
    let h = spawn(&mut w, "CART", ALPHA, 0, 3);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    let aside = spawn(&mut w, "BUGGY", BETA, -3, 3);
    let tl = launch_at(&mut w, 8);
    overridden(&mut w, h, m, aside);
    let t = w.tick;
    w.obj_mut(h).unwrap().beh.processed(t, 1000);
    run_through(&mut w, next_check(tl));
    let o = w.obj(h).unwrap();
    assert_eq!(o.target, None);
    assert_eq!(o.beh.suspended, Some(Behaviour::Guard), "no revert");
    assert_eq!(o.beh.aside, Some((Some(Target::Object(aside)), None)));
    assert_eq!(o.beh.current, Some(Behaviour::Attack));
}

#[test]
fn claim39_a_landing_check_kill_reverts_the_override_at_the_killing_hit() {
    // spec: S-0513 r62 claim 39 and its Changes for S-0215 claim 10 (a landing-check or claim 38 kill reverts an override at the killing hit, as S-0309 claim 27 gives for a destroyed object)
    let mut w = ready(&rules());
    let h = spawn(&mut w, "CART", ALPHA, 0, 3);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    let aside = spawn(&mut w, "BUGGY", BETA, -3, 3);
    let guard = spawn(&mut w, "BUGGY", BETA, 5, 5);
    let tl = launch_at(&mut w, 0);
    run_through(&mut w, tl + 61);
    // The landing cell is taken by a protected object: the landing check kills the member.
    protect::start(&mut w, guard, ProtectKind::Curtain, 500);
    assert_eq!(stage(&w, m), Some(Stage::Expired));
    overridden(&mut w, h, m, aside);
    glasnost_sim::superweapons::warp::turn(&mut w, m);
    assert!(
        w.obj(m).is_none_or(|o| o.health <= 0),
        "killed by the landing check"
    );
    let o = w.obj(h).unwrap();
    assert_eq!(o.beh.suspended, None, "reverted at the killing hit");
    assert_eq!(o.beh.current, Some(Behaviour::Guard));
    assert_eq!(
        o.target,
        Some(Target::Object(aside)),
        "the target put aside"
    );
}

// --- Claim 51 (SW23 part f): the member's own periodic check ----------------------------

#[test]
fn claim51_sw23_f_the_members_own_periodic_check_is_suppressed_through_arrival_effects() {
    // spec: S-0513 r62 claim 51 (from the first pending update through the arrival effects the member's turn is the warp update alone: no S-0309 claim 29 periodic check) and SW23 part f; S-0203 claim 4 (eligible in the arrival hold after the hold-end test)
    // The rows, by their tick after t_L (S-0513 T2 and claim 40 with held delay 0): leaving
    // (waiting) 2 to 60, wait expiry 61, departure 62, placement 63, arrival effects 64, hold
    // end 65. The first pending update (1) clears the member's own target itself (claim 48),
    // and the placement (63) falls back to rest (S-0215 Terms), so each row is compared with a
    // control run in which tick t_L + k is not a check tick: a check would be the difference.
    let run = |k: i32, check_tick: bool| {
        let mut w = ready(&rules());
        let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
        let x = spawn(&mut w, "CART", ALPHA, -6, 6);
        // The unarmed member answers impossible about any target (S-0301 claim 9 rank 4: no weapon in the slot), so a check would drop it.
        assert_eq!(
            check(&w, m, Some(Target::Object(x)), 0, false),
            Answer::Impossible
        );
        let at = if check_tick { k } else { k + 8 };
        let rem = (PERIOD - at.rem_euclid(PERIOD)).rem_euclid(PERIOD);
        let tl = launch_at(&mut w, rem);
        assert_eq!(held::check_tick(tl + k), check_tick);
        run_through(&mut w, tl + k - 1);
        hold(&mut w, m, x, Behaviour::Guard);
        step(&mut w);
        (stage(&w, m), target(&w, m))
    };
    for (k, row) in [
        (2_i32, "leaving"),
        (40, "leaving"),
        (61, "wait expiry"),
        (62, "departure"),
        (63, "placement"),
        (64, "arrival effects"),
        (65, "hold end"),
    ] {
        let (stage_c, checked) = run(k, true);
        let (_, control) = run(k, false);
        if k < 65 {
            assert_eq!(checked, control, "{row} (tick t_L + {k}): no check");
            if k <= 61 {
                // spec: S-0513 r89 claim 39a (Sparkles and drops in the warp's turns: a member still being erased after its warp step, its leaving updates to the expiry included, clears its target in both runs, so these rows show only that the check adds nothing)
                assert_eq!(checked, None, "{row}: claim 39a's drop");
            } else if k != 63 {
                assert!(checked.is_some(), "{row}: the target is kept");
            }
        } else {
            assert_eq!(stage_c, None, "the hold has ended");
            assert!(control.is_some(), "{row}: kept off a check tick");
            assert_eq!(checked, None, "{row}: checked and dropped");
        }
    }
}

#[test]
fn claim51_sw23_f_the_members_allied_drop_is_suppressed_through_arrival_effects() {
    // spec: S-0203 claim 4 (revision 42: the drop of an allied held target at the periodic check's place, S-0309 claim 25 part k, is suppressed in the warp turns as the check is; in the arrival hold after the hold-end test) and S-0513 claim 51, SW23 part f; S-0309 claim 25 part k (not in a warp turn; in a tick of any number otherwise) and Terms (Test-only exception: the marked house takes the computer-owner rules)
    // Each row's tick t_L + k is not a check tick, so only the allied drop could clear the target.
    let run = |k: i32| {
        let mut w = ready(&rules());
        w.houses[usize::from(BETA.0)].human = false;
        let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
        let z = spawn(&mut w, "CART", BETA, -6, 6);
        let at = k + 8;
        let rem = (PERIOD - at.rem_euclid(PERIOD)).rem_euclid(PERIOD);
        let tl = launch_at(&mut w, rem);
        assert!(!held::check_tick(tl + k));
        run_through(&mut w, tl + k - 1);
        hold(&mut w, m, z, Behaviour::Guard);
        step(&mut w);
        (stage(&w, m), target(&w, m))
    };
    for (k, row) in [(62_i32, "departure"), (64, "arrival effects")] {
        let (_, kept) = run(k);
        assert!(kept.is_some(), "{row} (tick t_L + {k}): no allied drop");
    }
    // spec: S-0513 r89 claim 39a (Sparkles and drops in the warp's turns: in its leaving updates, the expiry included, the member clears its target after the warp step whatever the target, so those rows no longer tell an allied drop from none)
    for (k, row) in [(2_i32, "leaving"), (40, "leaving"), (61, "wait expiry")] {
        let (_, dropped) = run(k);
        assert_eq!(dropped, None, "{row} (tick t_L + {k}): claim 39a's drop");
    }
    let (stage_e, dropped) = run(65);
    assert_eq!(stage_e, None, "the hold has ended");
    assert_eq!(dropped, None, "hold end: the allied target is dropped");
}

// --- After departure (Q-2211 question 3, open) ------------------------------------------

#[test]
fn claim39_a_holder_through_departure_drops_the_member_at_its_next_turn_interim() {
    // spec: S-0513 r62 claim 39 (a holder may hold the member through its departure; what the firing check answers about it from then on is open, Q-2211 question 3; S-0309 claim 27's rule for an object a revert gave back does not apply) - interim Q-2211 question 3: our own rule, the holder's next turn drops it with no draw and no revert
    let mut w = ready(&rules());
    let h = spawn(&mut w, "CART", ALPHA, 0, 3);
    let m = spawn(&mut w, "BUGGY", BETA, 0, 0);
    // Departure in t_L + 62 falls between check ticks: t_L + 62 = 16k + 8.
    let tl = launch_at(&mut w, 10);
    assert!(!held::check_tick(tl + 62));
    run_through(&mut w, tl + 61);
    hold(&mut w, h, m, Behaviour::Guard);
    let timer = w.obj(h).unwrap().targeting_timer;
    step(&mut w);
    assert_eq!(stage(&w, m), Some(Stage::Away));
    assert_eq!(
        target(&w, h),
        Some(Target::Object(m)),
        "kept through departure"
    );
    step(&mut w);
    assert_eq!(target(&w, h), None, "dropped at its next turn");
    assert_eq!(w.obj(h).unwrap().targeting_timer, timer, "no draw");
}
