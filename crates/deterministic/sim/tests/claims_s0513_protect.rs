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
