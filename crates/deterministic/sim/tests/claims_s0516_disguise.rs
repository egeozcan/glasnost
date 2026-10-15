//! Claim tests of S-0516 (cloak and stealth), disguise: claims 30 to 39, on
//! synthetic rules (every name and number is invented; see `cloak_common`).

mod cloak_common;

use cloak_common::*;
use glasnost_rules::schema::{Ruleset, Side};
use glasnost_sim::cloak::view::{self, Look};
use glasnost_sim::cloak::{Disguise, DisguiseAs, appears_disguised, disguise};
use glasnost_sim::occupancy::{self, Outcome};
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

/// The cloak ruleset with two sides (Alphaland first, Betaland second), a
/// second-side default disguise SHADE, and an unarmed WALKER.
fn rules() -> Ruleset {
    let mut r = ruleset();
    r.sides = vec![
        Side {
            name: "First".into(),
            members: vec!["Alphaland".into()],
        },
        Side {
            name: "Second".into(),
            members: vec!["Betaland".into()],
        },
    ];
    r.sim.countries.get_mut("Alphaland").unwrap().econ.side = Some("First".into());
    r.sim.countries.get_mut("Betaland").unwrap().econ.side = Some("Second".into());
    r.sim.general.soviet_disguise = Some("SHADE".into());
    r.sim.infantry.get_mut("WALKER").unwrap().primary = None;
    r
}

fn idx(w: &World, k: Kind, n: &str) -> u32 {
    w.rules().index_of(k, n).unwrap()
}

fn disguise_of(w: &World, id: ObjectId) -> Option<Disguise> {
    w.obj(id).unwrap().cloak.disguise
}

fn set_disguise(w: &mut World, id: ObjectId, what: DisguiseAs, house: Option<HouseId>) {
    let t = w.tick;
    w.obj_mut(id).unwrap().cloak.disguise = Some(Disguise {
        what: Some(what),
        house,
        since: t,
    });
}

/// Runs until `id` is still and has taken a disguise.
fn settle(w: &mut World, id: ObjectId) {
    until(w, 100, |w| w.obj(id).unwrap().cloak.disguise.is_some());
}

// spec: S-0516 claim 30 (a unit is disguised or not; while disguised it has a type and a house, possibly none)
#[test]
fn claim_30_disguise_state() {
    let r = rules();
    let mut w = game(&r);
    let m = spawn(&mut w, "MIRAGE", ALPHA, 0, 0);
    let p = spawn(&mut w, "PLAIN", ALPHA, 4, 0);
    settle(&mut w, m);
    let d = disguise_of(&w, m).unwrap();
    assert!(matches!(d.what, Some(DisguiseAs::Terrain(_))));
    assert_eq!(d.house, None);
    assert_eq!(disguise_of(&w, p), None);
}

// spec: S-0516 claim 31 (a disguise-flag projectile disguises its firer and does no damage: infantry copy infantry, their disguise if disguised, else type and owner; vehicles copy a disguised vehicle, a rock overlay or a terrain object, house none)
#[test]
fn claim_31_disguising_by_a_hit() {
    let r = rules();
    let mut w = game(&r);
    let c = spawn(&mut w, "COPIER", ALPHA, 0, 0);
    let t = spawn(&mut w, "WALKER", BETA, 2, 0);
    assert!(attack(&mut w, c, t));
    until(&mut w, 200, |w| w.obj(c).unwrap().cloak.disguise.is_some());
    let d = disguise_of(&w, c).unwrap();
    assert_eq!(
        d.what,
        Some(DisguiseAs::Infantry(idx(&w, Kind::Infantry, "WALKER")))
    );
    assert_eq!(d.house, Some(BETA));
    assert_eq!(w.obj(t).unwrap().health, 100, "no damage");
    // A disguised infantry target: its disguise is copied.
    let spy = spawn(&mut w, "SPY", BETA, 0, 3);
    disguise::disguise_hit(&mut w, Some(c), Some(Target::Object(spy)));
    let d = disguise_of(&w, c).unwrap();
    assert_eq!(
        d.what,
        Some(DisguiseAs::Infantry(idx(&w, Kind::Infantry, "SHADE")))
    );
    assert_eq!(d.house, Some(BETA));
    // A vehicle target is not copied by infantry.
    let v = spawn(&mut w, "PLAIN", BETA, 0, -3);
    disguise::disguise_hit(&mut w, Some(c), Some(Target::Object(v)));
    assert_eq!(disguise_of(&w, c).unwrap().house, Some(BETA));
    assert_eq!(
        disguise_of(&w, c).unwrap().what,
        Some(DisguiseAs::Infantry(idx(&w, Kind::Infantry, "SHADE")))
    );
    // A vehicle firer: a disguised vehicle's disguise; an undisguised one
    // leaves it unchanged.
    let m = spawn(&mut w, "MIRAGE", ALPHA, -5, 0);
    let other = spawn(&mut w, "PLAIN", BETA, -5, 3);
    disguise::disguise_hit(&mut w, Some(m), Some(Target::Object(other)));
    assert_eq!(disguise_of(&w, m), None);
    let bush = idx(&w, Kind::Terrain, "BUSH");
    set_disguise(&mut w, other, DisguiseAs::Terrain(bush), None);
    disguise::disguise_hit(&mut w, Some(m), Some(Target::Object(other)));
    let d = disguise_of(&w, m).unwrap();
    assert_eq!((d.what, d.house), (Some(DisguiseAs::Terrain(bush)), None));
}

// spec: S-0516 claim 32 (a permanent disguiser is created disguised as its side's default, its house its owner; a cleared infantry returns to the default; a vehicle becomes undisguised)
#[test]
fn claim_32_permanent_disguise() {
    let mut r = rules();
    let mut pv = r.sim.vehicles.get("PLAIN").unwrap().clone();
    pv.combat.can_disguise = Some(true);
    pv.perma_disguise = Some(true);
    r.sim.vehicles.insert("PERMAV".into(), pv);
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("PERMAV".into());
    let mut w = game(&r);
    let a = spawn(&mut w, "SPY", ALPHA, 0, 0);
    let b = spawn(&mut w, "SPY", BETA, 4, 0);
    let walker = idx(&w, Kind::Infantry, "WALKER");
    let shade = idx(&w, Kind::Infantry, "SHADE");
    let da = disguise_of(&w, a).unwrap();
    assert_eq!(
        (da.what, da.house),
        (Some(DisguiseAs::Infantry(walker)), Some(ALPHA))
    );
    let db = disguise_of(&w, b).unwrap();
    assert_eq!(
        (db.what, db.house),
        (Some(DisguiseAs::Infantry(shade)), Some(BETA))
    );
    assert!(
        !appears_disguised(&w, a, Some(BETA)),
        "fools no other house"
    );
    set_disguise(&mut w, a, DisguiseAs::Infantry(shade), Some(BETA));
    disguise::clear(&mut w, a);
    let da = disguise_of(&w, a).unwrap();
    assert_eq!(
        (da.what, da.house),
        (Some(DisguiseAs::Infantry(walker)), Some(ALPHA))
    );
    let v = spawn(&mut w, "PERMAV", ALPHA, -4, 0);
    assert!(disguise_of(&w, v).is_some());
    disguise::clear(&mut w, v);
    assert_eq!(disguise_of(&w, v), None);
}

// spec: S-0516 claim 33 (vehicles only: moving loses the disguise; a non-allied first infantry in a neighbouring cell on ticks not a multiple of 8 removes it with the infantry-blink lockout; a still, contact-free vehicle past its lockout draws one from the default list, house none)
#[test]
fn claim_33_disguise_when_still() {
    let r = rules();
    let mut w = game(&r);
    let m = spawn(&mut w, "MIRAGE", ALPHA, 0, 0);
    let copier = spawn(&mut w, "COPIER", ALPHA, 4, 4);
    settle(&mut w, m);
    let list = [
        idx(&w, Kind::Terrain, "SHRUB"),
        idx(&w, Kind::Terrain, "BUSH"),
    ];
    let d = disguise_of(&w, m).unwrap();
    assert!(matches!(d.what, Some(DisguiseAs::Terrain(t)) if list.contains(&t)));
    assert_eq!(disguise_of(&w, copier), None, "never infantry");
    // (a) Moving.
    move_to(&mut w, m, at(-6, 0));
    until(&mut w, 20, |w| glasnost_sim::occupancy::in_progress(w, m));
    assert_eq!(disguise_of(&w, m), None);
    for _ in 0..20 {
        run(&mut w, 1);
        if glasnost_sim::occupancy::in_progress(&w, m) {
            assert_eq!(disguise_of(&w, m), None);
        }
    }
    // (b) A neighbouring enemy infantry unit, checked on ticks t mod 8 ≠ 0.
    let mut w = game(&r);
    let m = spawn(&mut w, "MIRAGE", ALPHA, 0, 0);
    settle(&mut w, m);
    until(&mut w, 8, |w| w.tick % 8 == 0);
    let foe = spawn(&mut w, "WALKER", BETA, 1, 1);
    run(&mut w, 1);
    assert!(
        disguise_of(&w, m).is_some(),
        "not checked on a multiple of 8"
    );
    let t = w.tick;
    run(&mut w, 1);
    assert_eq!(disguise_of(&w, m), None);
    assert_eq!(w.obj(m).unwrap().cloak.lockout_until, t + INFANTRY_BLINK);
    glasnost_sim::objects::remove(&mut w, foe);
    let back = until(&mut w, 50, |w| w.obj(m).unwrap().cloak.disguise.is_some());
    assert!(back >= INFANTRY_BLINK - 1);
    // An allied infantry unit does not count.
    let mut w = game(&r);
    let m = spawn(&mut w, "MIRAGE", ALPHA, 0, 0);
    settle(&mut w, m);
    spawn(&mut w, "WALKER", ALPHA, 1, 1);
    run(&mut w, 16);
    assert!(disguise_of(&w, m).is_some());
}

// spec: S-0516 claim 34 (a) and (e) (a hit clears the disguise with a 2-per-point lockout; the disguise house leaving clears it; an owner change keeps it)
#[test]
fn claim_34_losing_a_disguise() {
    let r = rules();
    let mut w = game(&r);
    let m = spawn(&mut w, "MIRAGE", ALPHA, 0, 0);
    settle(&mut w, m);
    let t = w.tick;
    let sa = r.sim.warheads.keys().position(|k| k == "SA").unwrap() as u32;
    glasnost_sim::combat::damage::apply(
        &mut w,
        m,
        glasnost_sim::combat::damage::Hit {
            amount: 10,
            warhead: Some(sa),
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: false,
            no_escape: false,
        },
    );
    assert_eq!(w.obj(m).unwrap().health, 190);
    assert_eq!(disguise_of(&w, m), None);
    assert_eq!(w.obj(m).unwrap().cloak.lockout_until, t + 20);
    let back = until(&mut w, 50, |w| w.obj(m).unwrap().cloak.disguise.is_some());
    assert!(back >= 19, "the lockout holds");
    // (e) The disguise house leaves.
    let c = spawn(&mut w, "COPIER", ALPHA, 4, 4);
    let walker = idx(&w, Kind::Infantry, "WALKER");
    set_disguise(&mut w, c, DisguiseAs::Infantry(walker), Some(BETA));
    change_owner(&mut w, m, BETA);
    assert!(disguise_of(&w, m).is_some(), "kept through an owner change");
    disguise::house_left(&mut w, BETA);
    assert_eq!(disguise_of(&w, c), None);
}

// spec: S-0516 claim 35 (appears disguised to H: disguised, its owner not allying H, no detector coverage of H on its cell, the disguise house H, none or an ally of H; a building or aircraft to every house)
#[test]
fn claim_35_appears_disguised() {
    let r = rules();
    let mut w = game(&r);
    let c = spawn(&mut w, "COPIER", ALPHA, 0, 0);
    let walker = idx(&w, Kind::Infantry, "WALKER");
    assert!(!appears_disguised(&w, c, Some(BETA)), "not disguised");
    set_disguise(&mut w, c, DisguiseAs::Infantry(walker), Some(BETA));
    assert!(appears_disguised(&w, c, Some(BETA)));
    assert!(!appears_disguised(&w, c, Some(ALPHA)), "its owner");
    set_disguise(&mut w, c, DisguiseAs::Infantry(walker), None);
    assert!(appears_disguised(&w, c, Some(BETA)), "house none");
    set_disguise(&mut w, c, DisguiseAs::Infantry(walker), Some(ALPHA));
    assert!(!appears_disguised(&w, c, Some(BETA)), "the owner's house");
    set_disguise(&mut w, c, DisguiseAs::Infantry(walker), Some(BETA));
    ally(&mut w, ALPHA, BETA);
    assert!(!appears_disguised(&w, c, Some(BETA)), "the owner allies H");
    let mut w = game(&r);
    let c = spawn(&mut w, "COPIER", ALPHA, 0, 0);
    set_disguise(&mut w, c, DisguiseAs::Infantry(walker), Some(BETA));
    spawn(&mut w, "DETECTOR", BETA, 2, 0);
    assert!(!appears_disguised(&w, c, Some(BETA)), "detector coverage");
    let a = spawn(&mut w, "WRAITH", ALPHA, -4, 0);
    set_disguise(&mut w, a, DisguiseAs::Infantry(walker), None);
    assert!(
        appears_disguised(&w, a, Some(ALPHA)),
        "an aircraft, to every house"
    );
}

// spec: S-0516 claim 36 (appearing disguised: a scenery disguise cannot be selected; acquisition skips it; infantry entry row WP25 gives stationary friendly)
#[test]
fn claim_36_what_appearing_disguised_changes() {
    let r = rules();
    let mut w = game(&r);
    let m = spawn(&mut w, "MIRAGE", ALPHA, 0, 0);
    settle(&mut w, m);
    assert!(!glasnost_sim::targeting::selectable_by(&w, m, Some(BETA)));
    assert!(glasnost_sim::targeting::selectable_by(&w, m, Some(ALPHA)));
    // Acquisition: an armed guard of the other house does not pick it.
    let gun = spawn(&mut w, "CART", BETA, 3, 0);
    run(&mut w, 60);
    assert_ne!(w.obj(gun).unwrap().target, Some(Target::Object(m)));
    let p = spawn(&mut w, "PLAIN", ALPHA, 3, 3);
    run(&mut w, 60);
    let t = w.obj(gun).unwrap().target;
    assert!(t == Some(Target::Object(p)) || w.obj(p).is_none_or(|o| o.health < 200));
    // Row WP25.
    let mut r2 = rules();
    r2.sim.infantry.get_mut("WALKER").unwrap().primary = Some("Rifle".into());
    let mut w = game(&r2);
    let walker = spawn(&mut w, "WALKER", ALPHA, 0, 0);
    let spy = spawn(&mut w, "COPIER", BETA, 1, 0);
    let wi = idx(&w, Kind::Infantry, "WALKER");
    set_disguise(&mut w, spy, DisguiseAs::Infantry(wi), Some(ALPHA));
    let lvl = occupancy::reference_level(&w, walker);
    assert_eq!(
        occupancy::infantry_outcome(&w, walker, at(1, 0), lvl),
        Outcome::StationaryFriendly
    );
    w.obj_mut(spy).unwrap().cloak.disguise = None;
    assert_ne!(
        occupancy::infantry_outcome(&w, walker, at(1, 0), lvl),
        Outcome::StationaryFriendly
    );
}

// spec: S-0516 claim 37 and F6, table B (the owner's 256-tick cycle; others see the disguise without detector coverage; the fake blink shows others the real unit)
#[test]
fn claim_37_drawn_as_disguise() {
    for (t, show, look) in [
        (0, false, Look::Translucent25),
        (4, false, Look::Translucent50),
        (8, true, Look::Translucent50),
        (12, true, Look::Translucent25),
        (16, true, Look::Normal),
        (47, true, Look::Normal),
        (48, true, Look::Translucent25),
        (52, true, Look::Translucent50),
        (55, true, Look::Translucent50),
        (56, false, Look::Translucent50),
        (60, false, Look::Translucent25),
        (64, false, Look::Normal),
        (255, false, Look::Normal),
        (256 + 20, true, Look::Normal),
    ] {
        assert_eq!(view::owner_phase(t), (show, look), "t = {t}");
    }
    let r = rules();
    let mut w = game(&r);
    let m = spawn(&mut w, "MIRAGE", ALPHA, 0, 0);
    settle(&mut w, m);
    assert!(view::shown_disguise(&w, m, Some(BETA)).is_some());
    let t = w.tick;
    w.obj_mut(m).unwrap().cloak.blink_until = t + 3;
    assert!(
        view::shown_disguise(&w, m, Some(BETA)).is_none(),
        "fake blink"
    );
    w.obj_mut(m).unwrap().cloak.blink_until = 0;
    spawn(&mut w, "DETECTOR", BETA, 2, 0);
    assert!(
        view::shown_disguise(&w, m, Some(BETA)).is_none(),
        "detector coverage"
    );
}

// spec: S-0516 claim 38 (radar: the owner's colour when the owner allies V, one way; otherwise the disguise house's)
#[test]
fn claim_38_radar_colour() {
    let r = rules();
    let mut w = game(&r);
    let c = spawn(&mut w, "COPIER", ALPHA, 0, 0);
    let walker = idx(&w, Kind::Infantry, "WALKER");
    set_disguise(&mut w, c, DisguiseAs::Infantry(walker), Some(BETA));
    assert_eq!(view::radar_house(&w, c, Some(ALPHA)), Some(ALPHA));
    assert_eq!(view::radar_house(&w, c, Some(BETA)), Some(BETA));
    ally(&mut w, BETA, ALPHA);
    assert_eq!(
        view::radar_house(&w, c, Some(BETA)),
        Some(BETA),
        "viewer → owner does not count"
    );
    ally(&mut w, ALPHA, BETA);
    assert_eq!(view::radar_house(&w, c, Some(BETA)), Some(ALPHA));
}

// spec: S-0516 claim 39 (a disguise-when-still vehicle that fires sets its fake blink to the weapon's time and keeps its disguise; infantry firing never sets it)
#[test]
fn claim_39_fake_blink() {
    let r = rules();
    let mut w = game(&r);
    let m = spawn(&mut w, "MIRAGE", ALPHA, 0, 0);
    let foe = spawn(&mut w, "PLAIN", BETA, 3, 0);
    settle(&mut w, m);
    assert!(attack(&mut w, m, foe));
    let mut fired_at = None;
    for _ in 0..100 {
        let t = w.tick;
        run(&mut w, 1);
        if w.obj(m).unwrap().cloak.blink_until > 0 {
            fired_at = Some(t);
            break;
        }
    }
    let t = fired_at.expect("fired");
    assert_eq!(w.obj(m).unwrap().cloak.blink_until, t + BLINK);
    assert!(disguise::blinking(&w, m));
    assert!(disguise_of(&w, m).is_some(), "the disguise stays");
    // Infantry firing.
    let c = spawn(&mut w, "COPIER", ALPHA, -4, 0);
    let x = spawn(&mut w, "WALKER", BETA, -4, 2);
    assert!(attack(&mut w, c, x));
    until(&mut w, 200, |w| w.obj(c).unwrap().cloak.disguise.is_some());
    assert_eq!(w.obj(c).unwrap().cloak.blink_until, 0);
}
