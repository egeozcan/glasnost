//! Claim tests for chrono weapons (S-0514) revision 5 (answers Q-0984): the
//! one selected special hit (claim 20), a firer without a beam (claim 21),
//! boarding without hand-over (claim 22) and the fall-back release
//! (claim 14). Synthetic data only.

#[path = "temporal_common/mod.rs"]
mod temporal_common;

use glasnost_rules::sim::WarheadType;
use glasnost_sim::process;
use glasnost_sim::rules::Kind;
use glasnost_sim::temporal;
use glasnost_sim::transport;
use glasnost_sim::world::{ObjectId, World};
use temporal_common::*;

/// A world whose erasing warhead also carries the flags `set` changes (it
/// is temporal unless `erasing` is false), with a victim vehicle of strength
/// 100 (its raw damage versus 0.1 gives visible health changes) and an
/// erasing attacker.
fn flagged(erasing: bool, set: impl FnOnce(&mut WarheadType)) -> (World, ObjectId, ObjectId) {
    let mut w = world(|r| {
        let wh = r.sim.warheads.get_mut("TEMP").unwrap();
        wh.temporal = Some(erasing);
        wh.verses = Some(vec![d("1.0"); 11]);
        set(wh);
    });
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let a = attacker(&mut w, "ERASER", 0, at(-12, -12));
    (w, a, v)
}

fn locked(w: &World, v: ObjectId) -> bool {
    amount(w, v).is_some()
}

// spec: S-0514 claim 20 (revision 5): the first enabled row of the table wins; takeover, bomb planting, charging and parasite attachment outrank temporal erasure, which selects nothing when its effect cannot succeed or is not built, and ordinary damage is suppressed as well
#[test]
fn rows_above_erasure_suppress_it_and_ordinary_damage() {
    type Setter = fn(&mut WarheadType);
    let rows: [(&str, Setter); 4] = [
        ("takeover", |w| w.mind_control = Some(true)),
        ("bomb planting", |w| w.ivan_bomb = Some(true)),
        ("charging", |w| w.electric_assault = Some(true)),
        ("parasite", |w| w.parasite = Some(true)),
    ];
    for (name, set) in rows {
        let (mut w, a, v) = flagged(true, set);
        erase(&mut w, a, v);
        assert!(!locked(&w, v), "{name}: no erasure");
        assert_eq!(health(&w, v), 100, "{name}: no ordinary damage either");
    }
}

// spec: S-0514 claim 20 (revision 5): erasure outranks locomotor replacement, designation, disarming, disguise-making and nuke-making; a flag of the lower rows does not stop it
#[test]
fn erasure_outranks_the_lower_rows() {
    type Setter = fn(&mut WarheadType);
    let rows: [(&str, Setter); 5] = [
        ("locomotor", |w| w.is_locomotor = Some(true)),
        ("designation", |w| w.airstrike = Some(true)),
        ("disarming", |w| w.bomb_disarm = Some(true)),
        ("disguise", |w| w.makes_disguise = Some(true)),
        ("direct rocking", |w| w.direct_rocker = Some(true)),
    ];
    for (name, set) in rows {
        let (mut w, a, v) = flagged(true, set);
        erase(&mut w, a, v);
        assert!(locked(&w, v), "{name}: the erasing effect is selected");
    }
}

// spec: S-0514 claim 20 (revision 5): a row without a built effect (locomotor replacement here) still replaces ordinary damage and every lower row; with no row enabled ordinary damage applies
#[test]
fn a_selected_row_replaces_ordinary_damage_even_unbuilt() {
    let (mut w, a, v) = flagged(false, |w| w.is_locomotor = Some(true));
    erase(&mut w, a, v);
    assert_eq!(health(&w, v), 100, "locomotor replacement: no damage");
    let (mut w, a, v) = flagged(false, |_| {});
    erase(&mut w, a, v);
    assert!(health(&w, v) < 100, "no row: ordinary damage");
}

// spec: S-0514 claim 20 (revision 5): direct rocking ranks only when the projectile target is a vehicle: against a vehicle it replaces the later rows and ordinary damage; its flag against another target does not suppress them
#[test]
fn direct_rocking_counts_only_against_a_vehicle() {
    let (mut w, a, v) = flagged(false, |w| w.direct_rocker = Some(true));
    erase(&mut w, a, v);
    assert_eq!(health(&w, v), 100, "a vehicle target");
    let (mut w, a, _v) = flagged(false, |w| w.direct_rocker = Some(true));
    let inf = spawn(&mut w, Kind::Infantry, "BYSTANDER", 1, at(2, 0));
    erase(&mut w, a, inf);
    assert!(
        health(&w, inf) < 100_000,
        "an infantry target takes ordinary damage"
    );
}

// spec: S-0514 claim 21 (revision 5): a firer with no temporal beam makes no lock and causes none of claim 2's target side effects, and ordinary damage stays suppressed; it does not acquire a beam by delivering an erasing shot (a secondary-only erasing weapon)
#[test]
fn a_firer_without_a_beam_makes_no_lock() {
    let mut w = world(|r| {
        infantry(r, "SECONDARY", |t| {
            t.primary = Some("Rifle".into());
            t.secondary = Some("Chrono".into());
        });
    });
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let a = attacker(&mut w, "SECONDARY", 0, at(-12, -12));
    assert!(glasnost_sim::combat::fire::shot(
        &mut w,
        a,
        1,
        glasnost_sim::world::Target::Object(v)
    ));
    while !w.projectiles.is_empty() {
        glasnost_sim::testkit::run(&mut w, 1);
    }
    assert!(!locked(&w, v), "no lock");
    assert!(!temporal::holds(&w, a, v), "no beam acquired");
    assert_eq!(health(&w, v), 100, "ordinary damage stays suppressed");
}

// spec: S-0514 claim 22 (revision 5): boarding a non-gunner transport does not itself release or transfer the passenger's beam: the attachment is retained and keeps reducing the amount while its holder is a passenger
#[test]
fn boarding_a_non_gunner_transport_keeps_the_beam() {
    let mut w = world(|r| {
        vehicle(r, "CLOSED", |t| {
            t.passengers = Some(2);
            t.size_limit = Some(d("2"));
        });
        r.sim.vehicles.get_mut("VICT").unwrap().strength = Some(100_000);
    });
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let t = spawn(&mut w, Kind::Vehicle, "CLOSED", 0, at(-2, 0));
    let p = attacker(&mut w, "ERASER", 0, at(-2, 1));
    erase(&mut w, p, v);
    assert!(temporal::holds(&w, p, v));
    let before = amount(&w, v).unwrap();
    transport::board(&mut w, p, t);
    assert!(temporal::holds(&w, p, v), "retained at boarding");
    glasnost_sim::testkit::run(&mut w, 1);
    assert!(
        amount(&w, v).is_some_and(|a| a < before),
        "it keeps reducing the amount from inside the transport"
    );
}

// spec: S-0514 claim 14 and S-0215 claim 13 (revision 5): the release by falling back to rest belongs to the common effects, which take place at most once per actor per tick; a second fall-back in the same tick does not release
#[test]
fn the_fall_back_release_shares_the_once_per_tick_limit() {
    let (mut w, a, v) = {
        let mut w = world(|_| {});
        let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
        let a = attacker(&mut w, "ERASER", 0, at(-12, -12));
        (w, a, v)
    };
    erase(&mut w, a, v);
    assert!(temporal::holds(&w, a, v));
    // A first fall-back in this tick has already used the common effects.
    let t = w.tick;
    w.obj_mut(a).unwrap().beh.last_common_effects = Some(t);
    process::fall_back(&mut w, a, false);
    assert!(temporal::holds(&w, a, v), "no second set of common effects");
    w.obj_mut(a).unwrap().beh.last_common_effects = None;
    process::fall_back(&mut w, a, false);
    assert!(!temporal::holds(&w, a, v), "the first one releases");
}
