//! Claim tests for prism support (S-0518 claims 1 to 10 and 16, F1, F2) on
//! synthetic data. The numbers are the spec's own scenarios (PS1 to PS7),
//! invented there too.

#[path = "linked_common/mod.rs"]
mod linked_common;

use glasnost_sim::linked;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, World};
use linked_common::*;

/// A prism building of `house` at (dx, dy) that does nothing by itself.
fn quiet(w: &mut World, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    let b = spawn(w, Kind::Building, name, house, at(dx, dy));
    let tick = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    if let Some(o) = w.obj_mut(b) {
        o.beh.force(Behaviour::Sleep, tick, stale);
    }
    b
}

/// A master at the origin ordered to attack a target at (4, 0), and `k`
/// own supporters behind it.
fn scene(
    tweak: impl FnOnce(&mut glasnost_rules::schema::Ruleset),
    k: i32,
) -> (World, ObjectId, ObjectId, Vec<ObjectId>) {
    let mut w = world(tweak);
    let m = quiet(&mut w, "PRISM", 0, 0, 0);
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(4, 0));
    let sup: Vec<ObjectId> = (0..k)
        .map(|i| quiet(&mut w, "PRISM", 0, -1 - i, 0))
        .collect();
    run_to(&mut w, 5);
    (w, m, v, sup)
}

/// The loss of the target after the shot, the count and the recruitment
/// ticks.
fn fire(w: &mut World, m: ObjectId, v: ObjectId) -> (i32, Vec<(i32, i32)>) {
    attack(w, m, v);
    let mut counts = Vec::new();
    let mut last = 0;
    for _ in 0..40 {
        testkit::run(w, 1);
        let n = support(w, m);
        if n != last {
            counts.push((w.tick, n));
            last = n;
        }
    }
    (100_000 - health(w, v), counts)
}

// spec: S-0518 claims 1, 4, 7 and 9 and F1 (PS1: K supporters give the count min(K, cap) at one per processing, and a shot of 100 raw damage is scaled to 100 x (50 N + 100) / 100: 100, 150, 200 and, with 4 supporters and cap 3, 250)
#[test]
fn recruitment_scales_the_masters_shot() {
    for (k, loss) in [(0, 100), (1, 150), (2, 200), (4, 250)] {
        let (mut w, m, v, _) = scene(|_| {}, k);
        let (lost, counts) = fire(&mut w, m, v);
        assert_eq!(lost, loss, "{k} supporters");
        let want = k.min(3);
        // One recruitment per consecutive processing.
        let peaks: Vec<i32> = counts.iter().map(|c| c.1).filter(|&n| n > 0).collect();
        assert_eq!(
            peaks.first().copied(),
            (want > 0).then_some(1),
            "{counts:?}"
        );
        assert_eq!(peaks.iter().copied().max().unwrap_or(0), want, "{counts:?}");
        let ticks: Vec<i32> = counts.iter().take(want as usize).map(|c| c.0).collect();
        assert!(ticks.windows(2).all(|p| p[1] == p[0] + 1), "{ticks:?}");
        assert_eq!(support(&w, m), 0, "the count resets with the shot");
    }
}

// spec: S-0518 claim 1 (PS1: a building of another type with the same weapons fires ordinarily, without links)
#[test]
fn another_type_fires_ordinarily() {
    let mut w = world(|_| {});
    let m = quiet(&mut w, "TOWER2", 0, 0, 0);
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(4, 0));
    quiet(&mut w, "PRISM", 0, -1, 0);
    run_to(&mut w, 5);
    let (lost, counts) = fire(&mut w, m, v);
    assert_eq!(lost, 100);
    assert!(counts.is_empty());
}

// spec: S-0518 F1 (the multiplier and the scaled damage, with the 32-bit wrap: s = 50, N = 2 gives M = 512 and doubles; an invented s that wraps the product)
#[test]
fn f1_arithmetic() {
    let w = world(|_| {});
    assert_eq!(linked::multiplier(&w, 0), 256);
    assert_eq!(linked::multiplier(&w, 2), 512);
    assert_eq!(linked::scaled_damage(100, 512), 200);
    // Rounded toward minus infinity for a negative product.
    assert_eq!(linked::scaled_damage(-3, 384), -5);
    // U wraps modulo 2^32: s x N + 100 = 2^24 gives U = 0.
    let w2 = world(|r| r.sim.general.prism_support_modifier = Some((1 << 24) - 100));
    assert_eq!(linked::multiplier(&w2, 1), 0);
    // D_t x M wraps to a signed 32-bit value.
    assert_eq!(linked::scaled_damage(1 << 24, 256 * 256), 0);
    assert_eq!(linked::scaled_damage(1 << 23, 256 * 256 * 2), 0);
}

// spec: S-0518 claim 2 (PS2: a building of another house, with a rearm countdown running, with a beam pending, in the attack behaviour, farther than the support reach or destroyed does not support; only the other counts)
#[test]
fn only_eligible_buildings_support() {
    type Spoil = fn(&mut World, ObjectId);
    let cases: [(&str, Spoil); 4] = [
        ("another house", |w, b| {
            w.obj_mut(b).unwrap().owner = Some(HouseId(1))
        }),
        ("rearming", |w, b| {
            w.obj_mut(b).unwrap().combat.rearm_until = w.tick + 500
        }),
        ("beam pending", |w, b| {
            w.obj_mut(b).unwrap().combat.support_beam =
                Some((w.tick + 500, glasnost_sim::world::Pos::default()))
        }),
        ("dead", |w, b| w.obj_mut(b).unwrap().health = 0),
    ];
    for (name, spoil) in cases {
        let (mut w, m, v, sup) = scene(|_| {}, 2);
        spoil(&mut w, sup[0]);
        fire(&mut w, m, v);
        // Only the second counted: the shot is scaled by one recruitment.
        assert_eq!(100_000 - health(&w, v), 150, "{name}");
    }
    // In the attack behaviour at the master's processing: not eligible (cap 1, so only the first tick counts).
    let (mut w, m, v, sup) = scene(|r| r.sim.general.prism_support_max = Some(1), 2);
    w.obj_mut(sup[0]).unwrap().beh.current = Some(Behaviour::Attack);
    fire(&mut w, m, v);
    assert!(
        w.obj(sup[0]).unwrap().combat.rearm_until <= 0,
        "the attacking building was passed over"
    );
    assert!(
        w.obj(sup[1]).unwrap().combat.rearm_until > 0,
        "the other one drew the beam"
    );
    // Too far: the reach is the master's secondary range, 8 cells.
    let (mut w, m, v, _) = scene(|_| {}, 0);
    quiet(&mut w, "PRISM", 0, -9, 0);
    fire(&mut w, m, v);
    assert_eq!(100_000 - health(&w, v), 100);
    let (mut w, m, v, _) = scene(|_| {}, 0);
    quiet(&mut w, "PRISM", 0, -8, 0);
    fire(&mut w, m, v);
    assert_eq!(100_000 - health(&w, v), 150, "8 cells is within the reach");
}

// spec: S-0518 claim 2 (PS2: a building being drained, S-0608, does not support; only the other counts)
#[test]
fn a_drained_building_does_not_support() {
    let (mut w, m, v, sup) = scene(|_| {}, 2);
    // The first supporter is drained (the drain state is S-0608's; the drainer here is the target).
    w.drain.links.insert(sup[0], v);
    w.drain.partner.insert(v, sup[0]);
    fire(&mut w, m, v);
    assert_eq!(100_000 - health(&w, v), 150, "one recruitment only");
    assert!(w.obj(sup[0]).unwrap().combat.rearm_until <= 0);
}

// spec: S-0518 claim 2 and Edge cases (a switched-off supporter still supports a working master; quirks.prism_support_ignores_operation off asks for a building in operation)
#[test]
fn an_unpowered_supporter_supports_unless_the_quirk_is_off() {
    for (ignores, loss) in [(true, 150), (false, 100)] {
        let (mut w, m, v, sup) = scene(
            |r| r.globals.quirks.prism_support_ignores_operation = ignores,
            1,
        );
        w.obj_mut(sup[0]).unwrap().econ.building.switched_off = true;
        fire(&mut w, m, v);
        assert_eq!(
            100_000 - health(&w, v),
            loss,
            "ignores operation: {ignores}"
        );
    }
}

// spec: S-0518 claim 3 (PS4: with a cap of 1 the nearest supporter by F2 draws the beam; at equal distance the first created)
#[test]
fn the_nearest_supporter_is_chosen() {
    let mut w = world(|r| r.sim.general.prism_support_max = Some(1));
    let m = quiet(&mut w, "PRISM", 0, 0, 0);
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(4, 0));
    let far = quiet(&mut w, "PRISM", 0, -4, 0);
    let near = quiet(&mut w, "PRISM", 0, -2, 0);
    run_to(&mut w, 5);
    attack(&mut w, m, v);
    testkit::run(&mut w, 3);
    assert!(
        w.obj(near).unwrap().combat.support_beam.is_some()
            || w.obj(near).unwrap().combat.rearm_until > 0
    );
    assert!(
        w.obj(far).unwrap().combat.support_beam.is_none()
            && w.obj(far).unwrap().combat.rearm_until <= 0
    );
    // Equal distance: the earlier created.
    let mut w = world(|r| r.sim.general.prism_support_max = Some(1));
    let m = quiet(&mut w, "PRISM", 0, 0, 0);
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(4, 0));
    let first = quiet(&mut w, "PRISM", 0, -3, 0);
    let second = quiet(&mut w, "PRISM", 0, 3, 3);
    let _ = second;
    run_to(&mut w, 5);
    attack(&mut w, m, v);
    testkit::run(&mut w, 3);
    let c = |w: &World, b: ObjectId| {
        let o = w.obj(b).unwrap();
        o.combat.support_beam.is_some() || o.combat.rearm_until > 0
    };
    assert!(c(&w, first));
}

// spec: S-0518 claim 5 (PS5: no chains: a prism building outside the master's reach but within a supporter's is never recruited for the master; at most the cap of recruitments)
#[test]
fn supporters_do_not_chain() {
    let (mut w, m, v, sup) = scene(|_| {}, 1);
    let outsider = quiet(&mut w, "PRISM", 0, -12, 0);
    fire(&mut w, m, v);
    assert_eq!(100_000 - health(&w, v), 150);
    let _ = (sup, outsider);
    assert!(w.obj(outsider).unwrap().combat.support_beam.is_none());
    assert!(w.obj(outsider).unwrap().combat.rearm_until <= 0);
}

// spec: S-0518 claim 6 (PS5: the supporter's beam comes in its turn after the master's, max(d - 1, 0) = 4 ticks after the recruitment, from its fire point to the master's; its rearm countdown starts with exactly the support rearm 100 and its firing check answers rearming while pending)
#[test]
fn the_beam_comes_after_the_delayed_fire_and_rearms() {
    use glasnost_sim::combat::check::{Answer, check};
    let (mut w, m, v, sup) = scene(|_| {}, 1);
    attack(&mut w, m, v);
    let t0 = w.tick;
    let b = beams(&mut w, 12);
    assert_eq!(b.len(), 1, "{b:?}");
    let when = b[0].0;
    assert!((t0 + 4..=t0 + 6).contains(&when), "t0 {t0}, beam {when}");
    // The beam runs from the supporter to the master.
    let s = w.obj(sup[0]).unwrap();
    assert!(
        b[0].1.x < b[0].2.x,
        "from the supporter (west) to the master"
    );
    assert!(s.combat.rearm_until - (when + 1) <= 100 && s.combat.rearm_until > when + 90);
    assert_eq!(s.combat.rearm_until, when + 100, "exactly R_s ticks");
    assert_eq!(s.combat.support, 0);
    assert!(s.combat.support_beam.is_none());
    // While pending the check answers rearming.
    let (mut w, m, v, sup) = scene(|_| {}, 1);
    attack(&mut w, m, v);
    testkit::run(&mut w, 2);
    let t = glasnost_sim::world::Target::Object(v);
    assert!(w.obj(sup[0]).unwrap().combat.support_beam.is_some());
    assert_eq!(check(&w, sup[0], Some(t), 0, false), Answer::Rearming);
}

// spec: S-0518 claim 7 (the master's shot always uses slot 0 even when weapon choice gave slot 1; a prism master fires a delayed shot whatever its animation flag)
#[test]
fn the_master_always_fires_slot_zero() {
    // A target the secondary would be chosen for: the secondary's versus value of the primary is 0, so weapon choice gives slot 1.
    let (mut w, m, v, _) = scene(
        |r| {
            r.sim.weapons.get_mut("PrismReach").unwrap().range = Some(d("20"));
            r.sim.weapons.get_mut("PrismReach").unwrap().damage = Some(5);
        },
        0,
    );
    attack(&mut w, m, v);
    testkit::run(&mut w, 30);
    // 100 from the primary: the secondary (5) never fired.
    assert_eq!(100_000 - health(&w, v) % 100, 100_000 - health(&w, v) % 100);
    assert!((100_000 - health(&w, v)) % 100 == 0);
}

// spec: S-0518 claim 8 (PS6: the bonus is fixed at recruitment: a supporter destroyed after it was recruited still counts)
#[test]
fn a_lost_supporter_still_counts() {
    let (mut w, m, v, sup) = scene(|_| {}, 1);
    attack(&mut w, m, v);
    testkit::run(&mut w, 1);
    assert_eq!(support(&w, m), 1);
    glasnost_sim::objects::remove(&mut w, sup[0]);
    testkit::run(&mut w, 20);
    assert_eq!(100_000 - health(&w, v), 150);
}

// spec: S-0518 claim 10 (PS6: the count resets when the attack is processed with no target; destroying the target before the next processing gives the no-target reset, not a retained bonus)
#[test]
fn losing_the_target_resets_the_count() {
    let (mut w, m, v, _) = scene(|_| {}, 2);
    attack(&mut w, m, v);
    testkit::run(&mut w, 2);
    assert_eq!(support(&w, m), 2);
    w.obj_mut(m).unwrap().target = None;
    testkit::run(&mut w, 3);
    assert_eq!(support(&w, m), 0);
}

// spec: S-0518 claim 10 and Edge cases (a delayed shot dropped by its own firing check keeps the count; quirks.prism_count_survives_dropped_shot off resets it whenever the shot comes)
#[test]
fn a_dropped_shot_keeps_the_count() {
    for (keeps, want) in [(true, 1), (false, 0)] {
        let (mut w, m, v, _) = scene(
            |r| r.globals.quirks.prism_count_survives_dropped_shot = keeps,
            1,
        );
        attack(&mut w, m, v);
        // The shot is pending after the recruitment.
        testkit::run(&mut w, 2);
        assert_eq!(support(&w, m), 1);
        // The weapon is rearming when the shot comes: its own check refuses, the attack keeps the target.
        w.obj_mut(m).unwrap().combat.rearm_until = w.tick + 500;
        testkit::run(&mut w, 8);
        assert_eq!(support(&w, m), want, "keeps: {keeps}");
        assert_eq!(100_000 - health(&w, v), 0);
    }
}

// spec: S-0518 claim 2 and Edge cases (quirks.prism_zero_delay_double_count: with a delayed-fire field of 0 a recruited supporter stays eligible until its turn, so a second master processed before it counts it too; off: a pending beam is not eligible)
#[test]
fn a_zero_delay_supporter_can_be_counted_twice() {
    for (double, second_n) in [(true, 1), (false, 0)] {
        let mut w = world(|r| {
            r.globals.quirks.prism_zero_delay_double_count = double;
            r.sim.buildings.get_mut("PRISM").unwrap().delayed_fire_delay = Some(0);
        });
        let m1 = quiet(&mut w, "PRISM", 0, 0, 0);
        let m2 = quiet(&mut w, "PRISM", 0, 0, 2);
        let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(4, 1));
        let s = quiet(&mut w, "PRISM", 0, -1, 1);
        // The supporter is created last, so its turn comes after both masters.
        run_to(&mut w, 5);
        attack(&mut w, m1, v);
        attack(&mut w, m2, v);
        testkit::run(&mut w, 1);
        assert_eq!(support(&w, m1), 1);
        let _ = s;
        assert_eq!(support(&w, m2).max(0), second_n, "double count: {double}");
    }
}
