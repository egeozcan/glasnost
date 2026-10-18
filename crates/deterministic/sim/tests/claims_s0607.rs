//! Claim tests for magnetic lift (S-0607): the seize, the pull, the drop,
//! the fall and the impact, restoration and the wave. Synthetic data only.

#[path = "lift_common/mod.rs"]
mod lift_common;

use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::lift;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::Pos;
use glasnost_sim::world::Target;
use lift_common::*;

// spec: S-0607 claims 1 and 5: a seizing hit does no damage; the target stops, is seized with its destination the lifter, starts guard and is deselected
#[test]
fn claim_5_the_seize() {
    let mut w = world(|_| {});
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    beam(&mut w, l, v);
    assert_eq!(health(&w, v), 100, "no damage");
    assert!(lift::seized(&w, v));
    assert_eq!(lift::lifter_of(&w, v), Some(l));
    assert!(lift::grips(&w, l, v));
    let o = w.obj(v).unwrap();
    assert_eq!(o.dest, Some(Target::Object(l)));
    assert!(o.jump.is_some(), "the jumpjet locomotor is linked");
}

/// Destructions credited to house `h` (S-0302 claim 14), by object or by
/// house alone.
fn kills(w: &glasnost_sim::world::World, h: usize) -> i32 {
    w.houses[h].combat.kills
}

/// A world with LIFTER (house 0) at (0, 0) and a victim of `name` (house 1)
/// at (6, 0), seized.
fn seized_world(
    tweak: impl FnOnce(&mut glasnost_rules::schema::Ruleset),
    name: &str,
) -> (
    glasnost_sim::world::World,
    glasnost_sim::world::ObjectId,
    glasnost_sim::world::ObjectId,
) {
    let mut w = world(tweak);
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, name, 1, at(6, 0));
    beam(&mut w, l, v);
    assert!(lift::seized(&w, v));
    (w, l, v)
}

/// Runs until the pulled victim is over `cell` at least 300 leptons up.
fn pull_over(w: &mut glasnost_sim::world::World, v: glasnost_sim::world::ObjectId, dx: i32) {
    let c = at(dx, 0);
    assert!(
        run_until(w, 400, |w| w.obj(v).is_some_and(|o| o.pos.cell() == c)
            && height(w, v) > 300),
        "the victim did not pass over the cell"
    );
}

/// Orders the lifter to move one cell away (S-0607 claim 12 row 2).
fn order_lifter(w: &mut glasnost_sim::world::World, l: glasnost_sim::world::ObjectId) {
    glasnost_sim::econ::assign_destination(w, l, Some(Target::Cell(at(0, 1))));
}

// spec: S-0607 claim 5: the seized unit is deselected (the seize effect is announced)
#[test]
fn claim_5_deselection_effect() {
    let mut w = world(|_| {});
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    assert!(glasnost_sim::combat::fire::shot(
        &mut w,
        l,
        0,
        Target::Object(v)
    ));
    let mut seen = false;
    for _ in 0..40 {
        run(&mut w, 1);
        seen |= w
            .effects
            .iter()
            .any(|e| matches!(e, Effect::Seized { unit } if *unit == v));
    }
    assert!(seen);
}

// spec: S-0607 claim 3: infantry, a type whose size equals the damage and an already-seized unit are not seized
#[test]
fn claim_3_immune_targets() {
    let mut w = world(|_| {});
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let i = spawn(&mut w, Kind::Infantry, "TROOPER", 1, at(5, 0));
    beam(&mut w, l, i);
    assert!(!lift::seized(&w, i), "infantry");
    assert_eq!(health(&w, i), 100, "and no damage");
    let h = spawn(&mut w, Kind::Vehicle, "HEAVY", 1, at(5, 3));
    run(&mut w, 25);
    beam(&mut w, l, h);
    assert!(!lift::seized(&w, h), "F1: size 10 is not below damage 10");
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(5, -3));
    run(&mut w, 25);
    beam(&mut w, l, v);
    assert!(lift::seized(&w, v));
    let l2 = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, -6));
    beam(&mut w, l2, v);
    assert_eq!(
        lift::lifter_of(&w, v),
        Some(l),
        "already seized: no second grip"
    );
    assert!(lift::gripped_unit(&w, l2).is_none());
}

// spec: S-0607 claim 3 and F1: a type whose size is below the damage is seized, whatever its weight
#[test]
fn f1_size_below_damage_seizes() {
    let (w, _, v) = seized_world(
        |r| {
            r.sim.vehicles.get_mut("VICT").unwrap().size = Some(d("9.99"));
        },
        "VICT",
    );
    assert!(lift::seized(&w, v));
}

// spec: S-0607 claims 7, 8, 16 and 18: the pull keeps the body facing; arrival drops the unit over its lifter, whose cell is blocked by the lifter; the impact hurts the lifter and removes the unit, credited to the lifter's house
#[test]
fn claim_8_arrival_drop_on_the_lifter() {
    let (mut w, l, v) = seized_world(|_| {}, "VICT");
    let facing = w.obj(v).unwrap().body.value(w.tick);
    let mut max_h = 0;
    let gone = run_until(&mut w, 600, |w| {
        if let Some(o) = w.obj(v) {
            assert_eq!(
                o.body.value(w.tick),
                facing,
                "claim 7: the body facing is kept"
            );
        }
        max_h = max_h.max(height(w, v));
        w.obj(v).is_none()
    });
    assert!(gone, "the dropped unit leaves play");
    assert!(max_h > 300, "it was lifted");
    assert!(
        health(&w, l) < 1000,
        "claim 18: the lifter underneath is hurt"
    );
    assert_eq!(
        kills(&w, 0),
        2,
        "claim 18: credited to the dropper's house, twice (quirks.lift_impact_double_credit)"
    );
}

// spec: S-0607 claim 10: a seized unit gets illegal for every weapon and target
#[test]
fn claim_10_no_firing() {
    let (w, l, v) = seized_world(
        |r| {
            r.sim.vehicles.get_mut("VICT").unwrap().primary = Some("Gun".into());
        },
        "VICT",
    );
    assert_eq!(
        check::check(&w, v, Some(Target::Object(l)), 0, false),
        Answer::Illegal
    );
    assert_eq!(
        check::check(&w, v, Some(Target::Cell(at(3, 3))), 0, false),
        Answer::Illegal
    );
}

// spec: S-0607 claims 12, 13, 15, 17 and 20: a lifter's accepted order drops the unit mid-pull; it flies to its cell centre, falls with a growing rate, finds the cell clear, descends and touches down unhurt, set down; its next fall-back to rest restores its own locomotor
#[test]
fn claim_17_release_over_clear_land() {
    let (mut w, l, v) = seized_world(|_| {}, "VICT");
    pull_over(&mut w, v, 3);
    order_lifter(&mut w, l);
    assert!(
        lift::gripped_unit(&w, l).is_none(),
        "claim 12: the grip ends"
    );
    assert!(
        w.obj(l).unwrap().target.is_none(),
        "claim 12: the lifter's target is cleared"
    );
    assert!(lift::dropped(&w, v), "claim 13: dropped");
    assert!(w.obj(v).unwrap().dest.is_none());
    let mut rates = Vec::new();
    assert!(run_until(&mut w, 400, |w| {
        if let Some(r) = lift::rate(w, v) {
            rates.push(r);
        }
        !lift::dropped(w, v)
    }));
    assert!(
        rates.len() > 2 && rates.windows(2).all(|p| p[1].to_f64().gt(p[0].to_f64())),
        "the crash rate grows"
    );
    let c = w.obj(v).unwrap().pos;
    assert_eq!(
        (c.x.rem_euclid(256), c.y.rem_euclid(256)),
        (128, 128),
        "claim 15: over the cell centre"
    );
    assert!(restored(&mut w, v), "claim 20: restored");
    assert_eq!(health(&w, v), 100, "unhurt");
    assert!(
        w.obj(v).unwrap().jump.is_none(),
        "its own locomotor is back"
    );
    assert_eq!(height(&w, v), 0);
}

// spec: S-0607 claim 13: a grip that ends at height 0 sets the unit down with no fall; claim 20 restores it
#[test]
fn claim_13_set_down_at_height_zero() {
    let (mut w, l, v) = seized_world(|_| {}, "VICT");
    assert_eq!(height(&w, v), 0);
    order_lifter(&mut w, l);
    assert!(!lift::dropped(&w, v), "not dropped");
    assert!(restored(&mut w, v));
    assert_eq!(health(&w, v), 100);
}

// spec: S-0607 claim 5 and S-0215 claim 11 (revision 67) and Q-1331: a seized vehicle's movement is its jumpjet locomotor, so a set-down seized vehicle standing still in move is ready while its body turns
#[test]
fn claim_5_seized_turning_vehicle_is_ready() {
    let (mut w, l, v) = seized_world(|_| {}, "VICT");
    order_lifter(&mut w, l);
    assert!(run_until(&mut w, 400, |w| {
        w.height_above_surface(v) == 0 && !glasnost_sim::occupancy::in_progress(w, v)
    }));
    assert!(
        lift::seized(&w, v),
        "guard never falls back, so still seized"
    );
    // As after the owner's move click (scenario L11 part a): *move* is
    // current, which claim 11 does not excuse.
    let t = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    w.obj_mut(v).unwrap().beh.force(Behaviour::Move, t, stale);
    let rate = w
        .rules()
        .unit(Kind::Vehicle, w.obj(v).unwrap().type_index)
        .unwrap()
        .turn_rate;
    let face = w.obj(v).unwrap().body.value(t).wrapping_add(32768);
    assert!(w.obj_mut(v).unwrap().body.turn_to(face, t, rate));
    assert!(w.obj(v).unwrap().body.turning(t));
    assert!(glasnost_sim::process::ready(&w, v), "turning but ready");
}

// spec: S-0607 claim 9: a seized unit ignores a new destination
#[test]
fn claim_9_destinations_ignored() {
    let (mut w, l, v) = seized_world(|_| {}, "VICT");
    glasnost_sim::econ::assign_destination(&mut w, v, Some(Target::Cell(at(10, 10))));
    assert_eq!(w.obj(v).unwrap().dest, Some(Target::Object(l)));
}

// spec: S-0607 claim 9: no destination ends the grip without a drop: the unit is set down where it is, then descends and lands unhurt
#[test]
fn claim_9_no_destination_sets_down() {
    let (mut w, l, v) = seized_world(|_| {}, "VICT");
    pull_over(&mut w, v, 4);
    glasnost_sim::econ::assign_destination(&mut w, v, None);
    run(&mut w, 1);
    assert!(lift::gripped_unit(&w, l).is_none(), "the grip ends");
    assert!(
        w.obj(l).unwrap().target.is_some(),
        "the lifter keeps its target"
    );
    assert!(!lift::dropped(&w, v), "no drop");
    assert!(restored(&mut w, v));
    assert_eq!(health(&w, v), 100);
}

// spec: S-0607 claim 17 and S-0504 claims 4 and 5: a drop onto blocked water sets the unit sinking; its death is credited to the dropper
#[test]
fn claim_17_blocked_water_sinks() {
    let (mut w, l, v) = seized_world(|_| {}, "VICT");
    let cells: Vec<_> = (1..=5)
        .flat_map(|x| (-1..=1).map(move |y| at(x, y)))
        .collect();
    water(&mut w, &cells);
    pull_over(&mut w, v, 3);
    order_lifter(&mut w, l);
    assert!(run_until(&mut w, 400, |w| glasnost_sim::naval::sinking(
        w, v
    ) || w.obj(v).is_none()));
    assert!(w.obj(v).is_some(), "sinking, not removed by an impact");
    assert_eq!(health(&w, v), 100, "no falling damage");
    assert!(
        run_until(&mut w, 400, |w| w.obj(v).is_none()),
        "it sinks out of play"
    );
    assert_eq!(kills(&w, 0), 1, "its death is credited to the dropper");
}

// spec: S-0607 claim 16: infantry underneath are removed without death effects, credited to the dropper; claim 17 then finds the cell clear
#[test]
fn claim_16_infantry_underneath_are_erased() {
    let (mut w, l, v) = seized_world(|_| {}, "VICT");
    let i = spawn(&mut w, Kind::Infantry, "TROOPER", 1, at(3, 0));
    pull_over(&mut w, v, 3);
    order_lifter(&mut w, l);
    assert!(
        run_until(&mut w, 400, |w| w.obj(i).is_none()),
        "the infantry unit is removed"
    );
    assert_eq!(kills(&w, 0), 1);
    assert!(restored(&mut w, v));
    assert_eq!(
        health(&w, v),
        100,
        "the dropped unit survives on the cleared cell"
    );
}

/// A world whose impact warheads do no damage (verses 0), with the switch
/// `removes` for quirks.lift_impact_removes_survivor.
fn armoured(removes: bool) -> impl FnOnce(&mut glasnost_rules::schema::Ruleset) {
    move |r| {
        for h in ["CRUSH", "C4"] {
            r.sim.warheads.get_mut(h).unwrap().verses = Some(vec![d("0"); 11]);
        }
        r.globals.quirks.lift_impact_removes_survivor = removes;
    }
}

/// Drops the victim onto a vehicle at (3, 0) (blocked land) and returns
/// once the impact has happened.
fn drop_on_blocker(
    w: &mut glasnost_sim::world::World,
    l: glasnost_sim::world::ObjectId,
    v: glasnost_sim::world::ObjectId,
) {
    spawn(w, Kind::Vehicle, "VICT", 0, at(3, 0));
    pull_over(w, v, 3);
    order_lifter(w, l);
    assert!(run_until(w, 400, |w| !lift::dropped(w, v)));
}

// spec: S-0607 claim 18 and quirks.lift_impact_removes_survivor: a unit that survives both detonations is removed with the switch on; with it off it stays in play, set down, and claim 20 restores it
#[test]
fn quirk_lift_impact_removes_survivor_both_positions() {
    let (mut w, l, v) = seized_world(armoured(true), "VICT");
    drop_on_blocker(&mut w, l, v);
    assert!(w.obj(v).is_none(), "on: removed");
    let (mut w, l, v) = seized_world(armoured(false), "VICT");
    drop_on_blocker(&mut w, l, v);
    assert!(w.obj(v).is_some(), "off: stays in play");
    assert_eq!(health(&w, v), 100);
    assert!(restored(&mut w, v), "off: restored");
}

// spec: S-0607 claim 13: the stop on dropping clears the dropped unit's destination, route and target, breaks off its contact and deselects it
#[test]
fn claim_13_stop_on_dropping() {
    let (mut w, l, v) = seized_world(|_| {}, "VICT");
    let other = spawn(&mut w, Kind::Vehicle, "VICT", 0, at(0, 6));
    pull_over(&mut w, v, 3);
    if let Some(o) = w.obj_mut(v) {
        o.target = Some(Target::Object(other));
        o.contact = Some(other);
    }
    w.obj_mut(other).unwrap().contact = Some(v);
    w.effects.clear();
    order_lifter(&mut w, l);
    assert!(lift::dropped(&w, v));
    let o = w.obj(v).unwrap();
    assert_eq!(o.dest, None);
    assert_eq!(o.target, None, "target cleared");
    assert_eq!(o.contact, None, "contact broken");
    assert_eq!(w.obj(other).unwrap().contact, None, "on both sides");
    assert!(
        w.effects
            .iter()
            .any(|e| matches!(e, Effect::Dropped { unit } if *unit == v)),
        "deselected"
    );
}

// spec: S-0607 claim 15: while seized and crashing the target height is 0 in every update (not the ordinary crash's -5), and the rate starts afresh at every seize
#[test]
fn claim_15_crash_target_height_zero() {
    let (mut w, l, v) = seized_world(|_| {}, "VICT");
    pull_over(&mut w, v, 3);
    order_lifter(&mut w, l);
    assert!(run_until(&mut w, 40, |w| lift::dropped(w, v)));
    let mut seen = 0;
    while lift::dropped(&w, v) && w.tick < 2000 {
        if lift::rate(&w, v).is_some() {
            seen += 1;
            assert_eq!(
                w.obj(v).unwrap().jump.as_ref().unwrap().ht,
                0,
                "target height 0"
            );
        }
        run(&mut w, 1);
    }
    assert!(seen > 0, "it fell");
}

// spec: S-0607 claim 18: a survivor removed under quirks.lift_impact_removes_survivor is credited once, to the dropper
#[test]
fn claim_18_removed_survivor_credited_once() {
    for double in [true, false] {
        let (mut w, l, v) = seized_world(
            |r| {
                armoured(true)(r);
                r.globals.quirks.lift_impact_double_credit = double;
            },
            "VICT",
        );
        drop_on_blocker(&mut w, l, v);
        assert!(w.obj(v).is_none(), "removed");
        assert_eq!(w.houses[1].combat.losses, 1, "one loss (double {double})");
        assert_eq!(kills(&w, 0), 1, "one kill to the dropper (double {double})");
    }
}

/// A world whose falling damage kills VICT outright (F2 with the type's
/// strength and a multiplier of 1), with the switch `double` for
/// quirks.lift_impact_double_credit.
fn deadly(double: bool) -> impl FnOnce(&mut glasnost_rules::schema::Ruleset) {
    move |r| {
        r.sim.general.falling_damage_multiplier = Some(d("1.0"));
        r.sim.general.current_strength_damage = Some(false);
        r.globals.quirks.lift_impact_double_credit = double;
        r.sim.vehicles.get_mut("VICT").unwrap().cost = Some(500);
    }
}

// spec: S-0607 claim 18 and quirks.lift_impact_double_credit: a falling unit killed by a detonation is credited a second time by the impact credit, in the kill and loss tallies and the score; off: once
#[test]
fn quirk_lift_impact_double_credit_both_positions() {
    let mut tallies = Vec::new();
    for double in [true, false] {
        let (mut w, l, v) = seized_world(deadly(double), "VICT");
        drop_on_blocker(&mut w, l, v);
        assert!(w.obj(v).is_none_or(|o| o.health <= 0), "killed");
        run(&mut w, 5);
        tallies.push((
            w.houses[1].combat.losses,
            kills(&w, 0),
            w.houses[0].combat.score,
        ));
    }
    let (on, off) = (tallies[0], tallies[1]);
    assert_eq!(on.0, 2, "on: the victim's house counts two losses");
    assert_eq!(off.0, 1, "off: one loss");
    assert_eq!(on.1, off.1 + 1, "on: one more kill for the dropper's house");
    assert!(off.2 > 0);
    assert_eq!(on.2, 2 * off.2, "on: the score is doubled");
}

// spec: S-0607 claim 18: a crashable vehicle type takes the ordinary crash end instead (destroyed whatever its armour)
#[test]
fn claim_18_crashable_type_crashes() {
    let (mut w, l, v) = seized_world(armoured(false), "CRASHER");
    drop_on_blocker(&mut w, l, v);
    assert!(
        run_until(&mut w, 5, |w| w.obj(v).is_none()),
        "destroyed by the ordinary crash end"
    );
}

// spec: S-0607 F2: D_f = trunc(B × m) with B the current health (switch on) or the type's strength (switch off)
#[test]
fn f2_falling_damage() {
    let quarter = |current: bool| {
        move |r: &mut glasnost_rules::schema::Ruleset| {
            r.globals.quirks.lift_impact_removes_survivor = false;
            r.sim.general.falling_damage_multiplier = Some(d("0.25"));
            r.sim.general.current_strength_damage = Some(current);
        }
    };
    let (mut w, l, v) = seized_world(quarter(true), "VICT");
    w.obj_mut(v).unwrap().health = 80;
    drop_on_blocker(&mut w, l, v);
    // Two detonations of trunc(80 × 0.25) = 20 at the unit's own point.
    assert_eq!(health(&w, v), 40);
    let (mut w, l, v) = seized_world(quarter(false), "VICT");
    w.obj_mut(v).unwrap().health = 80;
    drop_on_blocker(&mut w, l, v);
    // The type's strength 100: two of 25.
    assert_eq!(health(&w, v), 30);
}

// spec: S-0607 claim 2: a new hit by a firer that already grips this very target does nothing
#[test]
fn claim_2_hit_on_the_gripped_target_does_nothing() {
    let (mut w, l, v) = seized_world(|_| {}, "VICT");
    pull_over(&mut w, v, 4);
    run(&mut w, 25);
    beam(&mut w, l, v);
    assert!(lift::grips(&w, l, v), "still gripped");
    assert!(!lift::dropped(&w, v));
}

// spec: S-0607 claims 2 and 12: a new seizing hit ends the current grip before the target is tested, even when the target is immune; the dropped unit falls; the firer keeps its target
#[test]
fn claim_2_new_hit_at_an_immune_target_drops() {
    let (mut w, l, v) = seized_world(|_| {}, "VICT");
    let i = spawn(&mut w, Kind::Infantry, "TROOPER", 1, at(0, 4));
    pull_over(&mut w, v, 3);
    assert!(glasnost_sim::combat::fire::shot(
        &mut w,
        l,
        0,
        Target::Object(i)
    ));
    attack(&mut w, l, v);
    assert!(run_until(&mut w, 40, |w| w.projectiles.is_empty()));
    assert!(!lift::seized(&w, i), "infantry are immune");
    assert!(lift::gripped_unit(&w, l).is_none(), "the grip ended");
    assert!(lift::dropped(&w, v), "the old unit falls");
    assert!(
        w.obj(l).unwrap().target.is_some(),
        "the firer keeps its target"
    );
}

// spec: S-0607 claim 5 and State and ordering: the firer's old gripped unit is released, and starts its fall, in the same tick in which the new unit is seized
#[test]
fn claim_5_second_seize_drops_the_first() {
    let (mut w, l, v) = seized_world(|_| {}, "VICT");
    let v2 = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 6));
    pull_over(&mut w, v, 3);
    run(&mut w, 25);
    assert!(glasnost_sim::combat::fire::shot(
        &mut w,
        l,
        0,
        Target::Object(v2)
    ));
    assert!(run_until(&mut w, 40, |w| lift::seized(w, v2)));
    assert!(lift::grips(&w, l, v2));
    assert!(lift::dropped(&w, v), "dropped in the tick of the new seize");
}

// spec: S-0607 claim 12: the gripped unit leaving play ends the grip and clears the lifter's target
#[test]
fn claim_12_gripped_unit_leaves_play() {
    let (mut w, l, v) = seized_world(|_| {}, "VICT");
    run(&mut w, 10);
    glasnost_sim::objects::remove(&mut w, v);
    assert!(lift::gripped_unit(&w, l).is_none());
    assert!(w.obj(l).unwrap().target.is_none());
}

// spec: S-0607 claims 12 and 14: a mobile lifter that leaves play drops its unit just before; the unit keeps the dropper house, which takes the credit
#[test]
fn claim_14_lifter_leaves_play() {
    let (mut w, l, v) = seized_world(|_| {}, "VICT");
    spawn(&mut w, Kind::Vehicle, "VICT", 0, at(3, 0));
    pull_over(&mut w, v, 3);
    glasnost_sim::objects::remove(&mut w, l);
    assert!(lift::dropped(&w, v));
    assert!(
        run_until(&mut w, 400, |w| w.obj(v).is_none()),
        "impact removal"
    );
    assert!(kills(&w, 0) >= 1, "credited to the dropper house");
    assert_eq!(kills(&w, 1), 0);
}

// spec: S-0607 claim 6: a seized unit of the neutral or special house (non-human rule) queues hunt; a player house's unit queues guard
#[test]
fn claim_6_non_human_houses_queue_hunt() {
    for (house, want) in [
        (1u8, Behaviour::Guard),
        (2, Behaviour::Hunt),
        (3, Behaviour::Hunt),
    ] {
        let mut w = testkit::game_with_non_players(&ruleset(), SIZE, 1);
        let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
        let v = spawn(&mut w, Kind::Vehicle, "VICT", house, at(6, 0));
        assert!(glasnost_sim::combat::fire::shot(
            &mut w,
            l,
            0,
            Target::Object(v)
        ));
        assert!(run_until(&mut w, 40, |w| lift::seized(w, v)));
        let b = &w.obj(v).unwrap().beh;
        assert!(
            b.queued == Some(want) || (b.queued.is_none() && b.current == Some(want)),
            "house {house}: {b:?}"
        );
    }
}

// spec: S-0607 claim 11: a gripping lifter with a live wave gets rearming against a non-building target other than its gripped unit
#[test]
fn claim_11_rearming_while_gripping() {
    let (mut w, l, v) = seized_world(|_| {}, "VICT");
    let other = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 5));
    assert!(lift::wave_alive(&w, l), "the shot made a wave");
    run(&mut w, 25);
    assert_eq!(
        check::check(&w, l, Some(Target::Object(other)), 0, false),
        Answer::Rearming
    );
    assert_eq!(
        check::check(&w, l, Some(Target::Object(v)), 0, false),
        Answer::Rearming,
        "against its gripped unit: rearming"
    );
}

// spec: S-0607 claim 11: against its own gripped unit the lifter answers rearming with any weapon, ahead of the tests of the target's layer and height (the gripped unit is airborne)
#[test]
fn claim_11_gripped_unit_rearming_with_any_weapon() {
    let (mut w, l, v) = seized_world(
        |r| {
            r.sim.vehicles.get_mut("LIFTER").unwrap().secondary = Some("Doom".into());
        },
        "VICT",
    );
    pull_over(&mut w, v, 3);
    assert!(height(&w, v) > 300, "the gripped unit is airborne");
    for slot in [0, 1] {
        assert_eq!(
            check::check(&w, l, Some(Target::Object(v)), slot, false),
            Answer::Rearming,
            "slot {slot}"
        );
    }
}

fn pos_at(c: glasnost_map::CellXY, dx: i32) -> Pos {
    Pos::new(c.x * 256 + 128 + dx, c.y * 256 + 128, 0)
}

// spec: S-0607 claim 21 and F3: no wave when the horizontal distance between the firing and impact points is under 240 leptons; one at 240
#[test]
fn f3_wave_distance_threshold() {
    let mut w = world(|_| {});
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(1, 0));
    let c = at(0, 0);
    w.obj_mut(l).unwrap().target = Some(Target::Object(v));
    lift::shot(&mut w, l, Target::Object(v), pos_at(c, 0), pos_at(c, 239));
    assert!(!lift::wave_alive(&w, l), "239: no wave");
    lift::shot(&mut w, l, Target::Object(v), pos_at(c, 0), pos_at(c, 240));
    assert!(lift::wave_alive(&w, l), "240: a wave");
}

// spec: S-0607 F3: a squared sum that wraps negative in 32-bit arithmetic takes the root of its negation, r(n) = r(-n), so a far impact still makes a wave
#[test]
fn f3_wrapped_negative_sum() {
    let mut w = world(|_| {});
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(1, 0));
    let c = at(0, 0);
    w.obj_mut(l).unwrap().target = Some(Target::Object(v));
    // 46341 squared is above the largest signed 32-bit value and wraps negative.
    lift::shot(
        &mut w,
        l,
        Target::Object(v),
        pos_at(c, 0),
        pos_at(c, 46_341),
    );
    assert!(lift::wave_alive(&w, l), "a wave");
}

// spec: S-0607 claim 21 and F4: no fixed duration; after a full fade-in a retarget leaves 20 fade updates; target removal also ends it
#[test]
fn f4_wave_fade() {
    let mut w = world(|_| {});
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(1, 0));
    let c = at(0, 0);
    w.obj_mut(l).unwrap().target = Some(Target::Object(v));
    lift::shot(&mut w, l, Target::Object(v), pos_at(c, 0), pos_at(c, 300));
    for _ in 0..200 {
        lift::wave_turn(&mut w, l);
    }
    assert!(lift::wave_alive(&w, l), "no fixed expiry");
    w.obj_mut(l).unwrap().target = None;
    let mut n = 0;
    while lift::wave_alive(&w, l) {
        lift::wave_turn(&mut w, l);
        n += 1;
        assert!(n < 100);
    }
    assert_eq!(n, 20, "20 fade updates after a full fade-in");
    // A retarget on the first update after creation: a = 2q retained, so b catches up in 2 updates.
    w.obj_mut(l).unwrap().target = Some(Target::Object(v));
    lift::shot(&mut w, l, Target::Object(v), pos_at(c, 0), pos_at(c, 300));
    lift::wave_turn(&mut w, l);
    glasnost_sim::objects::remove(&mut w, v);
    let mut n = 0;
    while lift::wave_alive(&w, l) {
        lift::wave_turn(&mut w, l);
        n += 1;
        assert!(n < 100);
    }
    assert_eq!(
        n, 2,
        "the target leaving play ends it; the shorter fade from the retained strength"
    );
}

/// Drops the victim over a cell holding a TROOPER first and a VICT after
/// it in the ground list; the impact warheads do no damage.
fn tail_scene(
    erase_tail: bool,
) -> (
    glasnost_sim::world::World,
    glasnost_sim::world::ObjectId,
    glasnost_sim::world::ObjectId,
    glasnost_sim::world::ObjectId,
) {
    let (mut w, l, v) = seized_world(
        move |r| {
            armoured(true)(r);
            r.globals.quirks.lift_drop_erases_cell_tail = erase_tail;
        },
        "VICT",
    );
    let t = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(3, 0));
    let i = spawn(&mut w, Kind::Infantry, "TROOPER", 1, at(3, 0));
    let list = w
        .cells
        .list(at(3, 0), glasnost_sim::world::Layer::Ground)
        .to_vec();
    assert_eq!(list, vec![i, t], "the infantry unit is listed first");
    pull_over(&mut w, v, 3);
    order_lifter(&mut w, l);
    assert!(run_until(&mut w, 400, |w| w.obj(i).is_none()));
    (w, v, i, t)
}

// spec: S-0607 claim 16 and quirks.lift_drop_erases_cell_tail: on, every object after the infantry unit in the cell's list goes too (the vehicle), and the cell is then clear; off, only infantry go, and the vehicle blocks the cell (claim 17)
#[test]
fn quirk_lift_drop_erases_cell_tail_both_positions() {
    let (mut w, v, _, t) = tail_scene(true);
    assert!(w.obj(t).is_none(), "on: the vehicle after it is erased");
    assert_eq!(kills(&w, 0), 2);
    assert!(
        restored(&mut w, v),
        "on: the cell is clear, the unit survives"
    );
    let (mut w, v, _, t) = tail_scene(false);
    assert!(w.obj(t).is_some(), "off: the vehicle stays");
    assert!(
        run_until(&mut w, 30, |w| w.obj(v).is_none()),
        "off: the cell is blocked; impact removal"
    );
    assert!(
        w.obj(t).is_some(),
        "off: the vehicle survives the armoured impact"
    );
}

// spec: S-0607 claims 15 and 16 and quirks.lift_deck_impact_reads_ground: a fall ending on a bridge deck reads the ground layer under it (on: the infantry under the bridge is erased); off: the deck layer (the infantry is untouched)
#[test]
fn quirk_lift_deck_impact_reads_ground_both_positions() {
    for on in [true, false] {
        let (mut w, l, v) = seized_world(
            move |r| r.globals.quirks.lift_deck_impact_reads_ground = on,
            "VICT",
        );
        let i = spawn(&mut w, Kind::Infantry, "TROOPER", 1, at(3, 0));
        let k = w.cells.idx(at(3, 0)).unwrap();
        w.cells.deck[k] = true;
        pull_over(&mut w, v, 3);
        order_lifter(&mut w, l);
        assert!(run_until(&mut w, 400, |w| !lift::dropped(w, v)));
        assert_eq!(w.obj(i).is_none(), on, "quirk {on}");
    }
}

/// A ruleset whose crushing warhead does no damage (verses 0), falling
/// damage a quarter of the type's strength, and survivors kept, with a
/// landed, crashable aircraft type PLANE of strength 100.
fn plane_rules(r: &mut glasnost_rules::schema::Ruleset) {
    r.sim.warheads.get_mut("CRUSH").unwrap().verses = Some(vec![d("0"); 11]);
    r.globals.quirks.lift_impact_removes_survivor = false;
    r.sim.general.falling_damage_multiplier = Some(d("0.25"));
    r.sim.general.current_strength_damage = Some(false);
    let mut t = r.sim.vehicles["VICT"].clone();
    t.crashable = Some(true);
    r.registries
        .entry("aircraft".into())
        .or_default()
        .push("PLANE".into());
    r.sim.aircraft.insert("PLANE".into(), t);
}

// spec: S-0607 claim 19: a dropped aircraft's first impact detonation uses the demolition warhead, and claim 18's crashable exception does not apply to it; a vehicle's first uses the crushing warhead
#[test]
fn claim_19_dropped_aircraft() {
    let mut w = world(plane_rules);
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    beam(&mut w, l, v);
    drop_on_blocker(&mut w, l, v);
    // The crushing detonation does nothing: one demolition hit of 25.
    assert_eq!(health(&w, v), 75, "vehicle");
    let mut w = world(plane_rules);
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let p = spawn(&mut w, Kind::Aircraft, "PLANE", 1, at(6, 0));
    beam(&mut w, l, p);
    assert!(lift::seized(&w, p), "a landed aircraft is seized");
    drop_on_blocker(&mut w, l, p);
    // Two demolition hits of 25, though its type can crash.
    assert_eq!(health(&w, p), 50, "aircraft");
}

/// A resource-gathering vehicle type HARV that drives temporarily
/// (S-0417 claim 5): `moving` keeps a locomotor destination point.
fn temporary_harvester(
    moving: bool,
) -> (
    glasnost_sim::world::World,
    glasnost_sim::world::ObjectId,
    glasnost_sim::world::ObjectId,
) {
    let mut w = world(|r| {
        vehicle(r, "HARV", |t| t.econ.resource_gatherer = Some(true));
    });
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let h = spawn(&mut w, Kind::Vehicle, "HARV", 1, at(6, 0));
    // A harvester whose own locomotor is not driving: the temporary
    // driving locomotor replaces it, as on a factory exit.
    w.obj_mut(h).unwrap().drive = None;
    glasnost_sim::econ::release::install_temporary_drive(&mut w, h);
    if moving {
        let goal = w.obj(h).unwrap().pos;
        let goal = Pos::new(goal.x + 512, goal.y, goal.z);
        w.obj_mut(h).unwrap().drive.as_mut().unwrap().goal = Some(goal);
    }
    assert!(glasnost_sim::econ::release::temporary_drive(&w, h));
    (w, l, h)
}

// spec: S-0607 claim 4: a resource gatherer driving temporarily whose replacement may end gets its own locomotor back and is not seized; otherwise it stops, its behaviour is cleared, it queues stop and is not seized
#[test]
fn claim_4_temporary_harvester() {
    let (mut w, l, h) = temporary_harvester(false);
    beam(&mut w, l, h);
    assert!(!lift::seized(&w, h));
    assert!(
        !glasnost_sim::econ::release::temporary_drive(&w, h),
        "its own locomotor is back"
    );
    assert_eq!(health(&w, h), 100);
    let (mut w, l, h) = temporary_harvester(true);
    beam(&mut w, l, h);
    assert!(!lift::seized(&w, h));
    assert!(
        glasnost_sim::econ::release::temporary_drive(&w, h),
        "still driving temporarily"
    );
    let o = w.obj(h).unwrap();
    assert!(
        o.beh.current == Some(Behaviour::Stop) || o.beh.queued == Some(Behaviour::Stop),
        "{:?}",
        o.beh
    );
    assert!(lift::gripped_unit(&w, l).is_none());
}
