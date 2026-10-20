//! Claim tests of S-0516 (cloak and stealth), hidden objects in game state
//! and in presentation: claims 24 to 29, on synthetic rules (every name and
//! number is invented; see `cloak_common`).

mod cloak_common;

use cloak_common::*;
use glasnost_rules::sim::WeaponType;
use glasnost_sim::cloak::machine::CloakEvent;
use glasnost_sim::cloak::view::{self, Look};
use glasnost_sim::cloak::{self, CloakState};
use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::targeting;
use glasnost_sim::world::Target;

// spec: S-0516 claim 24 (impossible on a target hidden from the firer's house when the mean weapon damage is above 0 or the target is not allied; an own hidden unit is refused too; an allied hidden target passes at mean damage 0)
#[test]
fn claim_24_firing_at_hidden() {
    let mut r = ruleset();
    r.sim.weapons.insert(
        "Tickle".into(),
        WeaponType {
            damage: Some(0),
            rof: Some(30),
            range: Some(glasnost_rules::sim::Decimal("5".into())),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("SA".into()),
            ..Default::default()
        },
    );
    let mut tickler = r.sim.vehicles.get("PLAIN").unwrap().clone();
    tickler.primary = Some("Tickle".into());
    tickler.combat.primary_fire_flh = Some([90, 0, 70]);
    r.sim.vehicles.insert("TICKLER".into(), tickler);
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("TICKLER".into());
    let mut w = game(&r);
    let cart = spawn(&mut w, "CART", ALPHA, 0, 0);
    let tick = spawn(&mut w, "TICKLER", ALPHA, 0, 2);
    let foe = spawn_cloaked(&mut w, "GHOST", BETA, 3, 0);
    let own = spawn_cloaked(&mut w, "GHOST", ALPHA, -3, 0);
    let ans = |w: &glasnost_sim::world::World, f, t| {
        check::check(w, f, Some(Target::Object(t)), 0, false)
    };
    assert!(cloak::hidden_from(&w, foe, Some(ALPHA)));
    assert_eq!(ans(&w, cart, foe), Answer::Impossible);
    assert_eq!(ans(&w, cart, own), Answer::Impossible, "even an own unit");
    assert_eq!(ans(&w, tick, foe), Answer::Impossible, "not allied");
    assert_ne!(
        ans(&w, tick, own),
        Answer::Impossible,
        "allied, mean damage 0"
    );
    // Sensed: not hidden.
    spawn(&mut w, "SEEKER", ALPHA, 4, 2);
    assert!(!cloak::hidden_from(&w, foe, Some(ALPHA)));
    assert_ne!(ans(&w, cart, foe), Answer::Impossible);
}

// spec: S-0516 claim 25 (an aircraft drops a target that is fully cloaked, not allied with it and not sensed by its house)
#[test]
fn claim_25_aircraft_drop_hidden_targets() {
    let r = ruleset();
    let mut w = game(&r);
    let a = spawn(&mut w, "WRAITH", ALPHA, 0, 0);
    let foe = spawn_cloaked(&mut w, "GHOST", BETA, 4, 0);
    w.obj_mut(a).unwrap().target = Some(Target::Object(foe));
    run(&mut w, 1);
    assert_eq!(w.obj(a).unwrap().target, None);
    // Sensed by its house: kept by this rule.
    let mut w = game(&r);
    let a = spawn(&mut w, "WRAITH", ALPHA, 0, 0);
    let foe = spawn_cloaked(&mut w, "GHOST", BETA, 4, 0);
    spawn(&mut w, "SEEKER", ALPHA, 4, 2);
    w.obj_mut(a).unwrap().target = Some(Target::Object(foe));
    run(&mut w, 1);
    assert_eq!(w.obj(a).unwrap().target, Some(Target::Object(foe)));
}

// spec: S-0516 claim 26 (a fully cloaked object that V neither owns nor senses, allies included, cannot be picked; nor a building at level 15 that V does not sense; a start of cloak leaves non-owner selections, a re-evaluation those of non-owners that do not sense it)
#[test]
fn claim_26_clicks_and_selection() {
    let mut r = ruleset();
    r.globals.quirks.cloak_flag_ground_mobile_only = false;
    let mut w = game(&r);
    ally(&mut w, ALPHA, BETA);
    ally(&mut w, BETA, ALPHA);
    let g = spawn_cloaked(&mut w, "GHOST", BETA, 0, 0);
    assert!(!targeting::selectable_by(&w, g, Some(ALPHA)), "an ally's");
    assert!(view::pickable(&w, g, Some(BETA)), "the owner's");
    let veil = spawn(&mut w, "VEIL", ALPHA, 6, 6);
    run(&mut w, 20);
    assert_eq!(state(&w, veil), CloakState::Cloaked);
    assert!(
        !view::pickable(&w, veil, Some(ALPHA)),
        "level 15, the owner too"
    );
    spawn(&mut w, "SEEKER", ALPHA, 1, 1);
    assert!(view::pickable(&w, g, Some(ALPHA)), "sensed");
    // Selection events.
    let mut w = game(&r);
    spawn(&mut w, "SEEKER", BETA, 2, 0);
    let s = spawn(&mut w, "SHADE", ALPHA, 0, 0);
    w.effects.clear();
    run(&mut w, 1);
    assert!(w.effects.iter().any(|e| matches!(
        e,
        Effect::Cloak { id, event: CloakEvent::Started } if *id == s
    )));
    assert!(
        view::deselects(&w, s, CloakEvent::Started, Some(BETA)),
        "even sensed"
    );
    assert!(!view::deselects(&w, s, CloakEvent::Started, Some(ALPHA)));
    run(&mut w, 10);
    assert_eq!(state(&w, s), CloakState::Cloaked);
    assert!(
        !view::deselects(&w, s, CloakEvent::Reevaluated, Some(BETA)),
        "sensed"
    );
    let mut w2 = game(&r);
    let s2 = spawn_cloaked(&mut w2, "GHOST", ALPHA, 0, 0);
    assert!(view::deselects(
        &w2,
        s2,
        CloakEvent::Reevaluated,
        Some(BETA)
    ));
    assert!(!view::deselects(
        &w2,
        s2,
        CloakEvent::Reevaluated,
        Some(ALPHA)
    ));
}

// spec: S-0516 claim 27 (another house's cloaked object is on V's radar only where V senses its cell, an ally's included)
#[test]
fn claim_27_radar() {
    let r = ruleset();
    let mut w = game(&r);
    ally(&mut w, ALPHA, BETA);
    ally(&mut w, BETA, ALPHA);
    let g = spawn_cloaked(&mut w, "GHOST", BETA, 0, 0);
    let p = spawn(&mut w, "PLAIN", BETA, 0, 5);
    assert!(!view::on_radar(&w, g, Some(ALPHA)));
    assert!(view::on_radar(&w, g, Some(BETA)));
    assert!(view::on_radar(&w, p, Some(ALPHA)));
    spawn(&mut w, "SEEKER", ALPHA, 1, 0);
    assert!(view::on_radar(&w, g, Some(ALPHA)));
}

// spec: S-0516 claim 28 (the fade table by x; fully cloaked drawn 50 % for the owner, a sensing viewer or a mutual ally, else not drawn; building levels 1–5 25 %, 6–10 50 %, 11–15 as fully cloaked)
#[test]
fn claim_28_how_v_sees_it() {
    let r = ruleset();
    let mut w = game(&r);
    let g = spawn_cloaked(&mut w, "GHOST", ALPHA, 0, 0);
    let set = |w: &mut glasnost_sim::world::World, s: CloakState, k: i32| {
        let o = w.obj_mut(g).unwrap();
        o.cloak.state = s;
        o.cloak.stage = k;
    };
    let rows = [
        (1, Look::Translucent25, Look::Translucent25), // x = 32
        (2, Look::Translucent50, Look::Translucent50), // x = 64
        (3, Look::Translucent50, Look::Translucent50), // x = 96
        (4, Look::Translucent50, Look::Translucent50), // x = 128
        (5, Look::Translucent50, Look::Translucent50), // x = 160
        (6, Look::Rippling, Look::Translucent50),      // x = 192
        (7, Look::Rippling, Look::Translucent50),      // x = 224
    ];
    for s in [CloakState::Cloaking, CloakState::Uncloaking] {
        for (k, other, owner) in rows {
            set(&mut w, s, k);
            assert_eq!(view::look(&w, g, Some(BETA)), other, "{s:?} {k}");
            assert_eq!(view::look(&w, g, Some(ALPHA)), owner, "{s:?} {k}");
        }
    }
    // spec: S-0516 claim 28 (revision 12: uncloaked, or fading at stage 0, is drawn normally)
    for s in [CloakState::Cloaking, CloakState::Uncloaking] {
        set(&mut w, s, 0);
        assert_eq!(view::look(&w, g, Some(BETA)), Look::Normal, "{s:?} 0");
        assert_eq!(view::look(&w, g, Some(ALPHA)), Look::Normal, "{s:?} 0");
    }
    set(&mut w, CloakState::Uncloaked, 0);
    assert_eq!(view::look(&w, g, Some(BETA)), Look::Normal);
    set(&mut w, CloakState::Cloaked, 0);
    assert_eq!(view::look(&w, g, Some(ALPHA)), Look::Translucent50);
    assert_eq!(view::look(&w, g, Some(BETA)), Look::Hidden);
    ally(&mut w, BETA, ALPHA);
    assert_eq!(view::look(&w, g, Some(BETA)), Look::Hidden, "one-way");
    ally(&mut w, ALPHA, BETA);
    assert_eq!(view::look(&w, g, Some(BETA)), Look::Translucent50, "mutual");
    // x ≥ 255 with a large stage count.
    let mut r2 = ruleset();
    r2.sim.general.cloaking_stages = Some(300);
    let mut w = game(&r2);
    let g = spawn_cloaked(&mut w, "GHOST", ALPHA, 0, 0);
    let o = w.obj_mut(g).unwrap();
    o.cloak.state = CloakState::Uncloaking;
    o.cloak.stage = 299; // x = ⌊256 · 299 / 300⌋ = 255
    assert_eq!(view::look(&w, g, Some(BETA)), Look::Hidden);
    assert_eq!(view::look(&w, g, Some(ALPHA)), Look::Translucent50);
    // Buildings.
    let mut w = game(&r);
    let b = spawn(&mut w, "VEIL", ALPHA, 0, 0);
    for (level, want) in [
        (1, Look::Translucent25),
        (5, Look::Translucent25),
        (6, Look::Translucent50),
        (10, Look::Translucent50),
        (11, Look::Hidden),
        (15, Look::Hidden),
    ] {
        let o = w.obj_mut(b).unwrap();
        o.cloak.state = CloakState::Cloaking;
        o.cloak.stage = level;
        assert_eq!(view::look(&w, b, Some(BETA)), want, "level {level}");
    }
    assert_eq!(view::look(&w, b, Some(ALPHA)), Look::Translucent50);
}

// spec: S-0516 claim 29 (no shadow, for any viewer, while the cloak state is anything but uncloaked)
#[test]
fn claim_29_shadows() {
    let r = ruleset();
    let mut w = game(&r);
    let g = spawn_cloaked(&mut w, "GHOST", ALPHA, 0, 0);
    let p = spawn(&mut w, "PLAIN", ALPHA, 3, 0);
    assert!(!view::shadow(&w, g));
    assert!(view::shadow(&w, p));
    hit(&mut w, g, 1);
    assert!(!view::shadow(&w, g), "uncloaking");
    run(&mut w, 16);
    assert_eq!(state(&w, g), CloakState::Uncloaked);
    assert!(view::shadow(&w, g));
}
