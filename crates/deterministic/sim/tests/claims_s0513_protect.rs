//! Claim tests of S-0513 claims 4 and 22 to 32 (invulnerability and the
//! force shield) and of the protection consumers in S-0302 claim 8,
//! S-0303 claim 15, S-0209 row WP3 and S-0210 claim 2, on synthetic rules
//! (`tests/sw_common`).

mod sw_common;

use glasnost_sim::combat::area::{self, Detonation, DetonationResult};
use glasnost_sim::combat::damage::{self, Hit, HitResult};
use glasnost_sim::superweapons::{self, ProtectKind, SwEvent, protect};
use glasnost_sim::world::{ObjectId, World};
use sw_common::*;

fn health(w: &World, id: ObjectId) -> i32 {
    w.obj(id).map_or(0, |o| o.health)
}

fn gone_or_dead(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_none_or(|o| o.health <= 0)
}

fn plain_hit(amount: i32, ignore: bool) -> Hit {
    Hit {
        amount,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: ignore,
        no_escape: false,
    }
}

fn warhead(w: &World, name: &str) -> Option<u32> {
    w.rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == name)
        .map(|i| i as u32)
}

// --- Claims 22 to 25 -----------------------------------------------------------

#[test]
fn claim22_23_25_curtain_kills_infantry_and_protects_the_rest() {
    // spec: S-0513 claims 22, 23 and 25 (infantry die; vehicles and buildings of any house are protected, curtain kind, for the curtain duration)
    let r = ruleset();
    let mut w = game(&r);
    let walker = spawn(&mut w, "WALKER", BETA, 0, 0);
    let cart = spawn(&mut w, "CART", BETA, 1, 0);
    let tower = spawn(&mut w, "TOWER", ALPHA, -1, -1);
    let outside = spawn(&mut w, "BUGGY", BETA, 2, 0);
    let t0 = w.tick;
    w.effects.clear();
    protect::curtain(&mut w, Some(ALPHA), at(0, 0), false);
    assert!(gone_or_dead(&w, walker));
    for id in [cart, tower] {
        let p = w.obj(id).unwrap().sw.protection.unwrap();
        assert_eq!(
            (p.start, p.duration, p.kind),
            (t0, CURTAIN_D, ProtectKind::Curtain)
        );
        assert!(superweapons::protected(&w, id));
    }
    // spec: S-0513 claim 4 (objects outside the nine cells are not reached)
    assert!(!superweapons::protected(&w, outside));
    // spec: S-0513 claim 22 (the curtain invoke animation)
    assert_eq!(anims(&w).iter().filter(|a| *a == "CURTA").count(), 1);
}

#[test]
fn claim23_infantry_hit_ignores_an_existing_protection() {
    // spec: S-0513 claim 23 (the hit ignores defences, so an existing protection does not stop it; S-0302 claim 8 column I)
    let r = ruleset();
    let mut w = game(&r);
    let walker = spawn(&mut w, "WALKER", BETA, 0, 0);
    protect::start(&mut w, walker, ProtectKind::ForceShield, 50);
    protect::curtain(&mut w, Some(ALPHA), at(0, 0), false);
    assert!(gone_or_dead(&w, walker));
}

#[test]
fn claim24_organic_vehicle_takes_a_defended_hit_and_no_protection() {
    // spec: S-0513 claim 24 (an organic vehicle: whole-strength demolition hit that does not ignore defences; not protected)
    let r = ruleset();
    let mut w = game(&r);
    let blob = spawn(&mut w, "BLOB", BETA, 0, 0);
    protect::curtain(&mut w, Some(ALPHA), at(0, 0), false);
    assert!(w.obj(blob).is_none_or(|o| o.sw.protection.is_none()));
    // A protected organic vehicle takes nothing and keeps its old protection.
    let mut w = game(&r);
    let blob = spawn(&mut w, "BLOB", BETA, 0, 0);
    protect::start(&mut w, blob, ProtectKind::ForceShield, 500);
    let h0 = health(&w, blob);
    protect::curtain(&mut w, Some(ALPHA), at(0, 0), false);
    assert_eq!(health(&w, blob), h0);
    assert_eq!(
        superweapons::protection_kind(&w, blob),
        Some(ProtectKind::ForceShield)
    );
}

#[test]
fn claim24_quirk_off_organic_vehicle_hit_ignores_defences() {
    // spec: S-0513 claim 24 (quirks.curtain_organic_vehicle_defended off: the hit ignores defences)
    let mut r = ruleset();
    r.globals.quirks.curtain_organic_vehicle_defended = false;
    let mut w = game(&r);
    let blob = spawn(&mut w, "BLOB", BETA, 0, 0);
    protect::start(&mut w, blob, ProtectKind::ForceShield, 500);
    protect::curtain(&mut w, Some(ALPHA), at(0, 0), false);
    assert!(gone_or_dead(&w, blob));
}

#[test]
fn claim25_restart_replaces_timer_and_kind() {
    // spec: S-0513 claim 25 (quirks.protection_restart_replaces: a shorter new duration shortens; the kind follows the latest start)
    let r = ruleset();
    let mut w = game(&r);
    let cart = spawn(&mut w, "CART", BETA, 0, 0);
    protect::start(&mut w, cart, ProtectKind::Curtain, 300);
    run(&mut w, 3);
    let t = w.tick;
    protect::start(&mut w, cart, ProtectKind::ForceShield, 5);
    let p = w.obj(cart).unwrap().sw.protection.unwrap();
    assert_eq!(
        (p.start, p.duration, p.kind),
        (t, 5, ProtectKind::ForceShield)
    );
    run(&mut w, 5);
    assert!(!superweapons::protected(&w, cart));
}

#[test]
fn claim25_quirk_off_keeps_the_longer_time_and_its_kind() {
    // spec: S-0513 claim 25 (quirk off: the longer remaining time is kept, with the kind of the effect that gave it)
    let mut r = ruleset();
    r.globals.quirks.protection_restart_replaces = false;
    let mut w = game(&r);
    let cart = spawn(&mut w, "CART", BETA, 0, 0);
    protect::start(&mut w, cart, ProtectKind::Curtain, 300);
    protect::start(&mut w, cart, ProtectKind::ForceShield, 5);
    assert_eq!(
        superweapons::protection_kind(&w, cart),
        Some(ProtectKind::Curtain)
    );
    protect::start(&mut w, cart, ProtectKind::ForceShield, 400);
    assert_eq!(
        superweapons::protection_kind(&w, cart),
        Some(ProtectKind::ForceShield)
    );
}

#[test]
fn claim25_building_delayed_kill_cancelled() {
    // spec: S-0513 claim 25 (a building: a pending delayed-kill countdown is cancelled)
    let r = ruleset();
    let mut w = game(&r);
    let tower = spawn(&mut w, "TOWER", BETA, 0, 0);
    let until = w.tick + 5;
    {
        let o = w.obj_mut(tower).unwrap();
        o.health = 1;
        o.combat.killed = true;
        o.combat.delay_kill_until = Some(until);
    }
    protect::curtain(&mut w, Some(ALPHA), at(0, 0), false);
    run(&mut w, 10);
    let o = w.obj(tower).expect("the building survives");
    assert_eq!(o.combat.delay_kill_until, None);
    assert_eq!(o.health, 1);
}

#[test]
fn claim22_waypoint_action_has_no_house_and_still_protects() {
    // spec: S-0513 claim 22 (the waypoint trigger action: no launching house, the ground layer of the block)
    let r = ruleset();
    let mut w = game(&r);
    let cart = spawn(&mut w, "CART", BETA, 0, 1);
    protect::curtain(&mut w, None, at(0, 0), true);
    assert!(superweapons::protected(&w, cart));
}

// --- Claim 26 ------------------------------------------------------------------

#[test]
fn claim26_protected_for_exactly_d_ticks() {
    // spec: S-0513 claim 26 and F7 (protected in t₀ ≤ t < t₀ + D)
    let r = ruleset();
    let mut w = game(&r);
    let cart = spawn(&mut w, "CART", BETA, 0, 0);
    let t0 = w.tick;
    protect::start(&mut w, cart, ProtectKind::Curtain, 7);
    let mut seen = Vec::new();
    for _ in 0..10 {
        seen.push((w.tick - t0, superweapons::protected(&w, cart)));
        step(&mut w);
    }
    for (dt, p) in seen {
        assert_eq!(p, dt < 7, "tick t0 + {dt}");
    }
}

#[test]
fn claim26_zero_duration_protects_nothing() {
    // spec: S-0513 claim 26 (a duration of 0 or less protects nothing; an infantry unit still dies)
    let mut r = ruleset();
    r.sim.general.superweapons.iron_curtain_duration = Some(0);
    let mut w = game(&r);
    let cart = spawn(&mut w, "CART", BETA, 0, 0);
    let walker = spawn(&mut w, "WALKER", BETA, 1, 0);
    protect::curtain(&mut w, Some(ALPHA), at(0, 0), false);
    assert!(!superweapons::protected(&w, cart));
    assert!(gone_or_dead(&w, walker));
}

// --- Protection consumers ----------------------------------------------------------

#[test]
fn s0302_claim8_protection_stops_plus_hits_only() {
    // spec: S-0302 claim 8 (stage 1: temporarily invulnerable stops + hits, not ignoring-defences hits)
    let r = ruleset();
    let mut w = game(&r);
    let cart = spawn(&mut w, "CART", BETA, 0, 0);
    protect::start(&mut w, cart, ProtectKind::Curtain, 100);
    let h0 = health(&w, cart);
    let res = damage::apply(&mut w, cart, plain_hit(50, false));
    assert_eq!(res, HitResult::Unaffected);
    assert_eq!(health(&w, cart), h0);
    damage::apply(&mut w, cart, plain_hit(30, true));
    assert_eq!(health(&w, cart), h0 - 30);
}

#[test]
fn s0303_claim15_curtain_nullifies_a_small_detonation() {
    // spec: S-0303 claim 15 (a small detonation near a curtain-protected object in the centre cell hits only protected objects)
    let r = ruleset();
    let mut w = game(&r);
    let cart = spawn(&mut w, "CART", BETA, 0, 0);
    let walker = spawn(&mut w, "WALKER", BETA, 0, 0);
    protect::start(&mut w, cart, ProtectKind::Curtain, 100);
    let hw = health(&w, walker);
    let point = w.obj(cart).unwrap().pos;
    let sa = warhead(&w, "SA");
    let res = area::detonate(
        &mut w,
        Detonation {
            point,
            damage: 40,
            warhead: sa,
            source: None,
            house: Some(ALPHA),
            resources: false,
        },
    );
    assert_eq!(res, DetonationResult::Nullified);
    assert_eq!(health(&w, walker), hw);
}

#[test]
fn s0303_claim15_force_shield_does_not_nullify() {
    // spec: S-0303 claim 15 (the curtain kind only, not the force-shield kind)
    let r = ruleset();
    let mut w = game(&r);
    let cart = spawn(&mut w, "CART", BETA, 0, 0);
    let walker = spawn(&mut w, "WALKER", BETA, 0, 0);
    protect::start(&mut w, cart, ProtectKind::ForceShield, 100);
    let hw = health(&w, walker);
    let point = w.obj(cart).unwrap().pos;
    let sa = warhead(&w, "SA");
    let res = area::detonate(
        &mut w,
        Detonation {
            point,
            damage: 40,
            warhead: sa,
            source: None,
            house: Some(ALPHA),
            resources: false,
        },
    );
    assert_ne!(res, DetonationResult::Nullified);
    assert!(health(&w, walker) < hw);
}

#[test]
fn s0303_claim15_quirk_off_others_hit_normally() {
    // spec: S-0303 Edge cases (quirks.curtain_nullifies_small_detonation off: a curtain-protected object only cancels its own hits)
    let mut r = ruleset();
    r.globals.quirks.curtain_nullifies_small_detonation = false;
    let mut w = game(&r);
    let cart = spawn(&mut w, "CART", BETA, 0, 0);
    let walker = spawn(&mut w, "WALKER", BETA, 0, 0);
    protect::start(&mut w, cart, ProtectKind::Curtain, 100);
    let hw = health(&w, walker);
    let point = w.obj(cart).unwrap().pos;
    let sa = warhead(&w, "SA");
    area::detonate(
        &mut w,
        Detonation {
            point,
            damage: 40,
            warhead: sa,
            source: None,
            house: Some(ALPHA),
            resources: false,
        },
    );
    assert!(health(&w, walker) < hw);
}

#[test]
fn claim27_protected_object_cannot_be_crushed() {
    // spec: S-0513 claim 27 and S-0210 claim 2 (a protected object is not crushed by an object not allied with it)
    let r = ruleset();
    let mut w = game(&r);
    let crusher = spawn(&mut w, "CRUSHER", ALPHA, 0, 0);
    let walker = spawn(&mut w, "WALKER", BETA, 1, 0);
    let before = glasnost_sim::occupancy::crushable_by(&w, crusher, walker);
    protect::start(&mut w, walker, ProtectKind::Curtain, 100);
    assert!(!glasnost_sim::occupancy::crushable_by(&w, crusher, walker));
    let _ = before;
}

#[test]
fn claim28_player_houses_may_fire_at_protected_objects() {
    // spec: S-0513 claim 28 (departure: every player house takes the human rule; the hits do nothing)
    use glasnost_sim::combat::check::{self, Answer};
    use glasnost_sim::world::Target;
    let r = ruleset();
    let mut w = glasnost_sim::testkit::game_with_non_players(&r, SIZE, 1);
    while glasnost_sim::orders::blocked(&w) {
        step(&mut w);
    }
    let cart = spawn(&mut w, "CART", BETA, 2, 0);
    let mine = spawn(&mut w, "TOWER", ALPHA, 0, 0);
    let neutral = glasnost_sim::testkit::spawn(
        &mut w,
        glasnost_sim::rules::Kind::Building,
        "TOWER",
        2,
        at(0, 2),
        0,
    );
    protect::start(&mut w, cart, ProtectKind::Curtain, 1000);
    let t = Some(Target::Object(cart));
    assert_ne!(check::check(&w, mine, t, 0, false), Answer::Illegal);
    // spec: S-0513 claim 28 (the neutral and special houses keep the original's rule: illegal target)
    assert_eq!(check::check(&w, neutral, t, 0, false), Answer::Illegal);
}

// --- Claim 29 ------------------------------------------------------------------

/// Steps both worlds until their streams differ (at most `max` ticks);
/// returns the tick in which they first differ.
fn first_divergence(a: &mut World, b: &mut World, max: i32) -> Option<i32> {
    for _ in 0..max {
        let t = a.tick;
        step(a);
        step(b);
        if a.rng != b.rng {
            return Some(t);
        }
    }
    None
}

#[test]
fn claim29_tint_draw_in_turn_q_plus_ten() {
    // spec: S-0513 claim 29 (one ranged draw R(−5, 5) on entering Initial dimming, in turn q + 10 with uninterrupted turns; q the first protected turn, here the tick of the start, before its turn pass)
    let r = ruleset();
    let mut a = game(&r);
    let mut b = game(&r);
    let ca = spawn(&mut a, "CART", BETA, 0, 0);
    let _cb = spawn(&mut b, "CART", BETA, 0, 0);
    let q = a.tick;
    protect::start(&mut a, ca, ProtectKind::Curtain, 100);
    assert_eq!(first_divergence(&mut a, &mut b, 30), Some(q + 10));
    // Exactly one draw per application: the gap stays through the rest of
    // the protection and after it.
    let gap = a.rng.taken() - b.rng.taken();
    assert!(gap > 0);
    for _ in 0..120 {
        step(&mut a);
        step(&mut b);
    }
    assert_eq!(a.rng.taken() - b.rng.taken(), gap);
}

#[test]
fn claim29_launch_after_the_turn_pass_draws_at_t0_plus_eleven() {
    // spec: S-0513 claim 29 (a launch applied after the turn pass in tick t₀: q = t₀ + 1, the draw in tick t₀ + 11, not t₀ + 10)
    let r = ruleset();
    let mut a = game(&r);
    spawn(&mut a, "IRONB", ALPHA, -5, -5);
    let cart = spawn(&mut a, "CART", BETA, 0, 0);
    ready_tick(&mut a, ALPHA, CURTAIN, 200);
    fire(&mut a, ALPHA, CURTAIN, at(0, 0));
    let t0 = a.tick;
    step(&mut a);
    let p = a.obj(cart).unwrap().sw.protection.unwrap();
    assert_eq!(p.start, t0, "applied in the order step of tick t₀");
    assert_eq!(p.tint, superweapons::Tint::BeforeFirstTurn);
    // The twin: the same world without the protection takes no tint draw.
    let mut b = a.clone();
    b.obj_mut(cart).unwrap().sw.protection = None;
    assert_eq!(first_divergence(&mut a, &mut b, 30), Some(t0 + 11));
}

#[test]
fn claim29_reapplication_restarts_the_envelope() {
    // spec: S-0513 claim 29 (reapplication restarts the envelope, even with another kind; expiry or restart before turn q + 10 prevents that application's draw)
    let r = ruleset();
    let mut a = game(&r);
    let mut b = game(&r);
    let ca = spawn(&mut a, "CART", BETA, 0, 0);
    let _cb = spawn(&mut b, "CART", BETA, 0, 0);
    protect::start(&mut a, ca, ProtectKind::Curtain, 100);
    for _ in 0..7 {
        step(&mut a);
        step(&mut b);
    }
    assert!(matches!(
        a.obj(ca).unwrap().sw.protection.unwrap().tint,
        superweapons::Tint::BrightHold(_)
    ));
    let q = a.tick;
    protect::start(&mut a, ca, ProtectKind::ForceShield, 100);
    assert_eq!(first_divergence(&mut a, &mut b, 30), Some(q + 10));
    let gap = a.rng.taken() - b.rng.taken();
    for _ in 0..30 {
        step(&mut a);
        step(&mut b);
    }
    assert_eq!(a.rng.taken() - b.rng.taken(), gap, "one draw in all");
}

#[test]
fn claim29_buildings_and_both_kinds_draw_in_turn_q_plus_ten() {
    // spec: S-0513 claim 29 (Q-0733: the draw is identical for the curtain and the force shield and for buildings and mobile objects that take protected turns: turn q + 10)
    for kind in [ProtectKind::Curtain, ProtectKind::ForceShield] {
        for name in ["TOWER", "CART"] {
            let r = ruleset();
            let mut a = game(&r);
            let mut b = game(&r);
            let id = spawn(&mut a, name, BETA, 0, 0);
            let _ = spawn(&mut b, name, BETA, 0, 0);
            let q = a.tick;
            protect::start(&mut a, id, kind, 100);
            assert_eq!(
                first_divergence(&mut a, &mut b, 30),
                Some(q + 10),
                "{name} {kind:?}"
            );
        }
    }
}

#[test]
fn claim29_missed_turns_postpone_the_phase_and_schedule_no_draw_of_their_own() {
    // spec: S-0513 claim 29 (Q-0733: a missed due turn postpones the transition to the next protected turn; after entering Bright hold, an object absent across its expiry draws on its first protected turn back; one absent across the expiry of Initial rise enters Bright hold on its return, starts a fresh 4-tick hold then, and draws only when that hold expires)
    let r = ruleset();
    // Absent across the expiry of Bright hold.
    let mut w = game(&r);
    let id = spawn(&mut w, "CART", BETA, 0, 0);
    let q = w.tick;
    protect::start(&mut w, id, ProtectKind::Curtain, 200);
    protect::tint_turn(&mut w, id);
    assert!(matches!(
        w.obj(id).unwrap().sw.protection.unwrap().tint,
        superweapons::Tint::Rise(s) if s == q
    ));
    w.tick = q + 6;
    protect::tint_turn(&mut w, id);
    assert!(matches!(
        w.obj(id).unwrap().sw.protection.unwrap().tint,
        superweapons::Tint::BrightHold(s) if s == q + 6
    ));
    let taken = w.rng.taken();
    // Absent for a long while: no draw of its own at q + 10.
    w.tick = q + 30;
    assert_eq!(w.rng.taken(), taken);
    protect::tint_turn(&mut w, id);
    assert!(matches!(
        w.obj(id).unwrap().sw.protection.unwrap().tint,
        superweapons::Tint::Dimming { since, .. } if since == q + 30
    ));
    assert_eq!(
        w.rng.taken(),
        taken + 1,
        "drawn on the first protected turn back"
    );

    // Absent across the expiry of Initial rise.
    let mut w = game(&r);
    let id = spawn(&mut w, "CART", BETA, 0, 0);
    let q = w.tick;
    protect::start(&mut w, id, ProtectKind::ForceShield, 200);
    protect::tint_turn(&mut w, id);
    w.tick = q + 20;
    let taken = w.rng.taken();
    protect::tint_turn(&mut w, id);
    assert!(matches!(
        w.obj(id).unwrap().sw.protection.unwrap().tint,
        superweapons::Tint::BrightHold(s) if s == q + 20
    ));
    assert_eq!(
        w.rng.taken(),
        taken,
        "no draw on return, a fresh hold starts"
    );
    w.tick = q + 20 + 3;
    protect::tint_turn(&mut w, id);
    assert_eq!(w.rng.taken(), taken);
    w.tick = q + 20 + 4;
    protect::tint_turn(&mut w, id);
    assert_eq!(
        w.rng.taken(),
        taken + 1,
        "drawn when the fresh hold expires"
    );
}

#[test]
fn claim29_protection_ended_before_the_phase_takes_no_draw() {
    // spec: S-0513 claim 29 (protection that expires before the pulsing phase takes no draw)
    let r = ruleset();
    let mut a = game(&r);
    let mut b = game(&r);
    let ca = spawn(&mut a, "CART", BETA, 0, 0);
    let _cb = spawn(&mut b, "CART", BETA, 0, 0);
    protect::start(&mut a, ca, ProtectKind::Curtain, 3);
    for _ in 0..20 {
        step(&mut a);
        step(&mut b);
    }
    assert_eq!(a.rng, b.rng);
}

/// The envelope phase of `id` after a turn at tick `t`.
fn turn_at(w: &mut World, id: ObjectId, t: i32) -> superweapons::Tint {
    w.tick = t;
    protect::tint_turn(w, id);
    w.obj(id).unwrap().sw.protection.unwrap().tint
}

#[test]
fn claim29_the_whole_envelope_with_uninterrupted_turns() {
    // spec: S-0513 claim 29 (the phase table: Initial rise 6, Bright hold 4, Initial dimming 20 + R(−5, 5), Pulse rise 8, Pulse fall 16, Low hold if P < 54 at its end, otherwise Pulse rise; Low hold until a turn finds P < 31; Final rise 6, Final bright hold 4, Return 20, then Ordinary)
    use superweapons::Tint;
    let r = ruleset();
    let mut w = game(&r);
    let id = spawn(&mut w, "CART", BETA, 0, 0);
    let q = w.tick;
    let d = 400;
    protect::start(&mut w, id, ProtectKind::Curtain, d);
    let mut seen = Vec::new();
    let mut last = Tint::BeforeFirstTurn;
    for t in q..q + d {
        let now = turn_at(&mut w, id, t);
        if now != last {
            seen.push((t, now));
            last = now;
        }
    }
    let ticks: Vec<i32> = seen.iter().map(|(t, _)| t - q).collect();
    assert_eq!(seen[0].1, Tint::Rise(q));
    assert_eq!(seen[1].1, Tint::BrightHold(q + 6));
    let Tint::Dimming { since, length } = seen[2].1 else {
        panic!("{seen:?}");
    };
    assert_eq!(since, q + 10, "the draw's turn q + 10");
    assert!((15..=25).contains(&length), "20 + R(−5, 5): {length}");
    assert_eq!(seen[3].1, Tint::PulseRise(q + 10 + length));
    // Pulses of 8 + 16 ticks while P ≥ 54 at the end of a Pulse fall.
    let mut i = 3;
    let mut t = q + 10 + length;
    loop {
        assert_eq!(seen[i].1, Tint::PulseRise(t), "{ticks:?}");
        assert_eq!(seen[i + 1].1, Tint::PulseFall(t + 8), "{ticks:?}");
        t += 24;
        let p = q + d - t;
        if p < 54 {
            assert_eq!(seen[i + 2].1, Tint::LowHold(t), "P = {p}");
            break;
        }
        i += 2;
    }
    let low = seen
        .iter()
        .position(|s| matches!(s.1, Tint::LowHold(_)))
        .unwrap();
    // The first turn with P < 31: t = q + d − 30.
    let f = (q + d - 30).max(t + 1);
    assert_eq!(seen[low + 1].1, Tint::FinalRise(f));
    assert_eq!(seen[low + 2].1, Tint::FinalHold(f + 6));
    assert_eq!(seen[low + 3].1, Tint::Return(f + 10));
    // Return lasts 20 ticks: with P < 31 at Final rise, protection expires
    // before or as Ordinary would begin.
    assert_eq!(seen.len(), low + 4, "{seen:?}");
    assert_eq!(
        protect::appearance(&w.obj(id).unwrap().sw.protection.unwrap(), q + d),
        None
    );
}

#[test]
fn claim29_low_hold_lasts_its_entry_turn_and_tests_strictly() {
    // spec: S-0513 claim 29 (Pulse fall ends in Low hold only if P < 54 at that turn, strictly; Low hold occupies at least its entry turn even when P < 31 already; Final rise starts at a later turn finding P < 31, strictly)
    use superweapons::Tint;
    let r = ruleset();
    let mut w = game(&r);
    let id = spawn(&mut w, "CART", BETA, 0, 0);
    let q = w.tick;
    // P = 54 at the end of Pulse fall: another Pulse rise.
    protect::start(&mut w, id, ProtectKind::Curtain, 100);
    let start = q;
    let set = |w: &mut World, tint: Tint| {
        w.obj_mut(id).unwrap().sw.protection.as_mut().unwrap().tint = tint;
    };
    set(&mut w, Tint::PulseFall(start + 30));
    assert_eq!(
        turn_at(&mut w, id, start + 46),
        Tint::PulseRise(start + 46),
        "P = 54"
    );
    set(&mut w, Tint::PulseFall(start + 31));
    assert_eq!(
        turn_at(&mut w, id, start + 47),
        Tint::LowHold(start + 47),
        "P = 53"
    );
    // Entered with P < 31 already: no Final rise in its entry turn.
    set(&mut w, Tint::PulseFall(start + 54));
    assert_eq!(
        turn_at(&mut w, id, start + 70),
        Tint::LowHold(start + 70),
        "P = 30"
    );
    assert_eq!(turn_at(&mut w, id, start + 70), Tint::LowHold(start + 70));
    assert_eq!(turn_at(&mut w, id, start + 71), Tint::FinalRise(start + 71));
    // P = 31 holds; P = 30 rises.
    set(&mut w, Tint::LowHold(start + 60));
    assert_eq!(
        turn_at(&mut w, id, start + 69),
        Tint::LowHold(start + 60),
        "P = 31"
    );
    assert_eq!(
        turn_at(&mut w, id, start + 70),
        Tint::FinalRise(start + 70),
        "P = 30"
    );
}

#[test]
fn claim29_the_phase_factors() {
    // spec: S-0513 claim 29 (the factor column, r = max(0, L − (t − s)), divisions truncated toward zero)
    use protect::factor;
    use superweapons::Tint;
    assert_eq!(factor(Tint::BeforeFirstTurn, 50), None);
    assert_eq!(factor(Tint::Ordinary, 50), None);
    // Initial rise: 256 at its entry, 512 when it ends; trunc((12 − r) × 256 / 6).
    assert_eq!(factor(Tint::Rise(10), 10), Some(256));
    assert_eq!(factor(Tint::Rise(10), 11), Some(298));
    assert_eq!(factor(Tint::Rise(10), 16), Some(512));
    assert_eq!(factor(Tint::BrightHold(10), 12), Some(512));
    // Initial dimming of any length: trunc((461 r + 1020) / 20), 51 at r = 0.
    let dim = |length| Tint::Dimming { since: 0, length };
    assert_eq!(factor(dim(20), 0), Some(512));
    assert_eq!(factor(dim(25), 0), Some((461 * 25 + 1020) / 20));
    assert_eq!(factor(dim(15), 0), Some((461 * 15 + 1020) / 20));
    assert_eq!(factor(dim(15), 15), Some(51));
    assert_eq!(factor(dim(15), 40), Some(51), "r stays 0 past the end");
    // Pulse rise 51 → 128, Pulse fall 128 → 51, Low hold 51.
    assert_eq!(factor(Tint::PulseRise(0), 0), Some(51));
    assert_eq!(factor(Tint::PulseRise(0), 8), Some(128));
    assert_eq!(factor(Tint::PulseFall(0), 0), Some(128));
    assert_eq!(factor(Tint::PulseFall(0), 16), Some(51));
    assert_eq!(factor(Tint::LowHold(0), 99), Some(51));
    // Final rise 51 → 512, Final bright hold 512, Return 512 → 256.
    assert_eq!(factor(Tint::FinalRise(0), 0), Some(51));
    assert_eq!(factor(Tint::FinalRise(0), 3), Some((3072 - 461 * 3) / 6));
    assert_eq!(factor(Tint::FinalRise(0), 6), Some(512));
    assert_eq!(factor(Tint::FinalHold(0), 2), Some(512));
    assert_eq!(factor(Tint::Return(0), 0), Some(512));
    assert_eq!(factor(Tint::Return(0), 7), Some((13 + 20) * 256 / 20));
    assert_eq!(factor(Tint::Return(0), 20), Some(256));
}

#[test]
fn f9_protection_brightness() {
    // spec: S-0513 F9 (b′ = min(2000, ⌊b × m / 256⌋); unchanged and not capped without a factor)
    use protect::brightness;
    assert_eq!(brightness(1000, Some(512)), 2000);
    assert_eq!(brightness(1500, Some(512)), 2000, "capped");
    assert_eq!(brightness(1000, Some(51)), 199);
    assert_eq!(brightness(999, Some(298)), 999 * 298 / 256);
    assert_eq!(
        brightness(2400, Some(256)),
        2000,
        "a factor of 256 inside a phase caps"
    );
    assert_eq!(brightness(2400, None), 2400, "no factor: not newly capped");
    assert_eq!(brightness(0, Some(512)), 0);
}

#[test]
fn claim29_f7_the_appearance_ends_with_the_protection() {
    // spec: S-0513 claim 29 (expiry immediately removes the appearance regardless of the phase); F7 (protected exactly when t − t₀ < D)
    use superweapons::{Protection, Tint};
    let p = Protection {
        start: 10,
        duration: 30,
        kind: ProtectKind::ForceShield,
        tint: Tint::BrightHold(12),
    };
    assert_eq!(protect::appearance(&p, 9), None);
    assert_eq!(
        protect::appearance(&p, 12),
        Some((ProtectKind::ForceShield, Some(512)))
    );
    assert_eq!(
        protect::appearance(&p, 39),
        Some((ProtectKind::ForceShield, Some(512)))
    );
    assert_eq!(protect::appearance(&p, 40), None);
}

// --- Claims 30 to 32 -------------------------------------------------------------

#[test]
fn claim30_31_force_shield_blackout_and_allied_buildings_in_radius() {
    // spec: S-0513 claims 30 and 31 (blackout of the launching house; allied buildings within the radius, strict F3 test; nothing else)
    let r = ruleset();
    let mut w = game(&r);
    let near = spawn(&mut w, "TOWER", ALPHA, 2, 2);
    let edge = spawn(&mut w, "TOWER", ALPHA, SHIELD_R, 0);
    let enemy = spawn(&mut w, "TOWER", BETA, 1, 0);
    let unit = spawn(&mut w, "CART", ALPHA, 0, 0);
    let shield = w
        .rules()
        .superweapons
        .first_of_kind(superweapons::SwKind::ForceShield)
        .unwrap();
    w.effects.clear();
    protect::force_shield(&mut w, ALPHA, shield, at(0, 0));
    assert!(glasnost_sim::econ::power::blackout_running(&w, ALPHA));
    assert_eq!(
        superweapons::protection_kind(&w, near),
        Some(ProtectKind::ForceShield)
    );
    // 3 cells away: a distance of exactly 768 leptons is not below 256 × 3.
    assert!(!superweapons::protected(&w, edge));
    assert!(!superweapons::protected(&w, enemy));
    assert!(!superweapons::protected(&w, unit));
    assert!(anims(&w).iter().any(|a| a == "SHIELDA"));
    // spec: S-0513 claim 32 (the start sound at the target)
    assert!(events(&w).iter().any(|e| e.0 == SwEvent::ShieldStart));
}

#[test]
fn claim31_allied_owner_view_decides() {
    // spec: S-0513 claim 31 (the building's owner counts the launching house as allied)
    let r = ruleset();
    let mut w = game(&r);
    let theirs = spawn(&mut w, "TOWER", BETA, 1, 0);
    let shield = w
        .rules()
        .superweapons
        .first_of_kind(superweapons::SwKind::ForceShield)
        .unwrap();
    ally(&mut w, ALPHA, BETA);
    protect::force_shield(&mut w, ALPHA, shield, at(0, 0));
    assert!(
        !superweapons::protected(&w, theirs),
        "only Alpha's view says allied"
    );
    ally(&mut w, BETA, ALPHA);
    protect::force_shield(&mut w, ALPHA, shield, at(0, 0));
    assert!(superweapons::protected(&w, theirs));
}

#[test]
fn claim32_fade_sound_after_d_minus_p_house_updates() {
    // spec: S-0513 claim 32 (the special sound D − P house updates after the launch)
    let r = ruleset();
    let mut w = game(&r);
    let shield = w
        .rules()
        .superweapons
        .first_of_kind(superweapons::SwKind::ForceShield)
        .unwrap();
    protect::force_shield(&mut w, ALPHA, shield, at(0, 0));
    let mut heard = None;
    for k in 1..=(SHIELD_D) {
        step(&mut w);
        if events(&w).iter().any(|e| e.0 == SwEvent::ShieldFade) {
            heard = Some(k);
            break;
        }
    }
    assert_eq!(heard, Some(SHIELD_D - FADE));
}

#[test]
fn claim32_negative_lead_never_sounds() {
    // spec: S-0513 claim 32 (never when D − P is negative)
    let mut r = ruleset();
    r.sim.general.superweapons.force_shield_play_fade_sound_time = Some(SHIELD_D + 1);
    let mut w = game(&r);
    let shield = w
        .rules()
        .superweapons
        .first_of_kind(superweapons::SwKind::ForceShield)
        .unwrap();
    protect::force_shield(&mut w, ALPHA, shield, at(0, 0));
    for _ in 0..(2 * SHIELD_D) {
        step(&mut w);
        assert!(!events(&w).iter().any(|e| e.0 == SwEvent::ShieldFade));
    }
}

// --- Through the order path ---------------------------------------------------------

#[test]
fn claim5_curtain_order_announces_and_protects() {
    // spec: S-0513 claims 5 and 22 (activation announcement, radar event at the target, the effect)
    let r = ruleset();
    let mut w = game(&r);
    spawn(&mut w, "IRONB", ALPHA, -5, -5);
    let cart = spawn(&mut w, "CART", BETA, 0, 0);
    ready_tick(&mut w, ALPHA, CURTAIN, 200);
    fire(&mut w, ALPHA, CURTAIN, at(0, 0));
    step(&mut w);
    let ev = events(&w);
    assert!(ev.contains(&(SwEvent::Launch, Some(ALPHA), Some(CURTAIN))));
    assert!(ev.contains(&(SwEvent::Radar, Some(ALPHA), Some(CURTAIN))));
    assert!(superweapons::protected(&w, cart));
}
