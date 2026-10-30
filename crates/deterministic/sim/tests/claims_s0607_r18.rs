//! S-0607 revision 18 (M8 step A3 intake 49; answers Q-2113): claim 11's
//! rule that a mobile lifter takes no approach step against the unit it
//! grips while that unit is its target, in every behaviour, with S-0215
//! claim 37 part h's approaches as the one exception; the per-tick drop of
//! part a that still applies, keeping the grip; and claim 12's row 2 with
//! the same exception. Synthetic data only (the lift fixture of
//! `lift_common`); no stock value.

#[path = "lift_common/mod.rs"]
mod lift_common;

use glasnost_rules::schema::Ruleset;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::lift;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{ObjectId, Target, World};
use lift_common::*;

/// The lifter's beam gets a minimum range of 3 cells (synthetic), so that
/// the gripped unit comes inside it before it arrives.
fn min_range(r: &mut Ruleset) {
    let b = r.sim.weapons.get_mut("MagBeam").unwrap();
    b.minimum_range = Some(d("3"));
}

/// A lifter of house 0 (computer-controlled) at (0, 0) seizes VICT of house
/// 1 at (6, 0), then has `b` current with the victim as its target.
fn gripping(b: Behaviour, tweak: impl FnOnce(&mut Ruleset)) -> (World, ObjectId, ObjectId) {
    let mut w = world(|r| {
        min_range(r);
        tweak(r);
    });
    w.houses[0].human = false;
    w.houses[1].human = false;
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    beam(&mut w, l, v);
    assert!(lift::grips(&w, l, v), "the lifter grips the victim");
    let t = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    let o = w.obj_mut(l).unwrap();
    o.target = Some(Target::Object(v));
    o.beh.force(b, t, stale);
    // The seizing shot is not a shot of the hold.
    w.effects.clear();
    (w, l, v)
}

/// The cells east of the lifter, inside its minimum range, that the
/// per-tick test makes water.
fn wet() -> [glasnost_map::CellXY; 6] {
    [at(1, 0), at(2, 0), at(1, 1), at(2, 1), at(1, -1), at(2, -1)]
}

/// What one hold showed, tick by tick, until the grip ended.
#[derive(Debug, Default)]
struct Hold {
    /// Ticks on which the lifter gripped the victim, its target the victim,
    /// and the victim inside the weapon's minimum range.
    inside: usize,
    /// A destination the lifter had on a tick of the hold.
    dest: Option<(i32, Target)>,
    /// A tick of the hold on which the lifter's target was not the victim.
    lost: Option<i32>,
    /// Shots the lifter made on ticks that ended with the grip held.
    shots: usize,
    /// Whether the grip ended by the victim's arrival (claim 8).
    arrived: bool,
}

fn watch_hold(w: &mut World, l: ObjectId, v: ObjectId, limit: i32) -> Hold {
    let mut h = Hold::default();
    for _ in 0..limit {
        glasnost_sim::tick::step(w);
        let shots = std::mem::take(&mut w.effects)
            .iter()
            .filter(|e| matches!(e, Effect::Shot { firer, .. } if *firer == l))
            .count();
        let o = w.obj(l).unwrap();
        if !lift::grips(w, l, v) {
            if let Some(d) = o.dest {
                h.dest.get_or_insert((w.tick, d));
            }
            // Claim 8: arrival ends the grip with the victim placed on its
            // movement point, next to the lifter, and dropped there; the
            // lifter accepted no destination.
            let (a, b) = (o.pos.cell(), w.obj(v).unwrap().pos.cell());
            h.arrived = o.dest.is_none()
                && lift::dropped(w, v)
                && (a.x - b.x).abs() <= 1
                && (a.y - b.y).abs() <= 1;
            return h;
        }
        // Shots on a tick that ends with the grip held (the lifter's
        // target is the gripped unit on all of them, below).
        h.shots += shots;
        if let Some(d) = o.dest {
            h.dest.get_or_insert((w.tick, d));
        }
        if o.target == Some(Target::Object(v)) {
            if !glasnost_sim::approach::in_range(w, l, Target::Object(v)) {
                h.inside += 1;
            }
        } else {
            h.lost.get_or_insert(w.tick);
        }
    }
    panic!("the hold did not end in {limit} ticks: {h:?}");
}

// spec: S-0607 claim 11 (revision 18, Q-2113: no approach step against the gripped unit, in every behaviour; the range test, minimum range included, gives no destination towards or away from it) and claim 12 (row 2: no destination, so the grip is not ended by the approach step); S-0215 claim 37 (c), (d) and (f): a computer-controlled lifter in hunt whose gripped unit comes inside its weapon's minimum range takes no destination, keeps its target and the grip, makes no fire event against it, and the grip lasts until the unit arrives (claim 8)
#[test]
fn claim_11_no_approach_against_the_gripped_unit_in_hunt() {
    let (mut w, l, v) = gripping(Behaviour::Hunt, |_| {});
    let h = watch_hold(&mut w, l, v, 600);
    assert!(
        h.inside > 0,
        "the victim came inside the minimum range: {h:?}"
    );
    assert_eq!(h.dest, None, "claim 11: no approach destination");
    assert_eq!(h.lost, None, "claim 11: the target stays the gripped unit");
    assert_eq!(h.shots, 0, "claim 11: no fire event at the gripped unit");
    assert!(h.arrived, "claim 8: the hold ends by arrival: {h:?}");
}

// spec: S-0607 claim 11 (revision 18: in every behaviour) and S-0215 claim 37 (a), (d) and (f): the same in attack and area guard, whose processings take the approach step too; the grip lasts until arrival (claim 8)
#[test]
fn claim_11_no_approach_against_the_gripped_unit_in_attack_and_area_guard() {
    for b in [Behaviour::Attack, Behaviour::AreaGuard] {
        let (mut w, l, v) = gripping(b, |_| {});
        let h = watch_hold(&mut w, l, v, 600);
        assert!(h.inside > 0, "{b:?}: inside the minimum range: {h:?}");
        assert_eq!(h.dest, None, "{b:?}: claim 11: no approach destination");
        assert_eq!(h.lost, None, "{b:?}: claim 11: the target is kept");
        assert!(h.arrived, "{b:?}: claim 8: arrival: {h:?}");
    }
}

// spec: S-0607 claim 11 (revision 18: a human-controlled lifter too, in every behaviour) and S-0215 claim 37 (d): the hunt row for a human-controlled owner that may approach keeps the target, and the step then gives no destination against the gripped unit
#[test]
fn claim_11_no_approach_for_a_human_controlled_lifter() {
    let (mut w, l, v) = gripping(Behaviour::Hunt, |_| {});
    w.houses[0].human = true;
    let h = watch_hold(&mut w, l, v, 600);
    assert!(h.inside > 0, "inside the minimum range: {h:?}");
    assert_eq!(h.dest, None, "claim 11: no approach destination");
    assert!(h.arrived, "claim 8: arrival: {h:?}");
}

/// Runs a hold until the grip ends, at most 600 ticks, and checks that it
/// ended by a part h approach: the lifter's destination is the gripped unit
/// (S-0215 claim 37 h), its target is cleared and the unit no longer has it
/// as its holder (S-0607 claim 12 row 2). Returns the tick and whether the
/// unit was dropped (claim 13: it was 1 lepton or more above the surface).
fn ended_by_part_h(w: &mut World, l: ObjectId, v: ObjectId) -> (i32, bool) {
    for _ in 0..600 {
        let up = height(w, v) >= 1;
        glasnost_sim::tick::step(w);
        if !lift::grips(w, l, v) {
            let o = w.obj(l).unwrap();
            assert_eq!(o.dest, Some(Target::Object(v)), "part h: the destination");
            assert_eq!(o.target, None, "claim 12 row 2: the target is cleared");
            assert_eq!(lift::lifter_of(w, v), None, "claim 12: the grip ended");
            return (w.tick, up && lift::dropped(w, v));
        }
    }
    panic!("part h: the grip did not end");
}

// spec: S-0607 claim 11 (revision 18: the exception: S-0215 claim 37 part h's crushing approaches come first and give the gripped unit as the destination) and claims 12 (row 2, its exception: that destination ends the grip, the target cleared) and 13 (in the air: dropped); S-0210 claim 11 (the attack-target variant of a computer-controlled house in our engine's neutral and special houses' rules: a target it can crush, closer than the crush range, no move destination): the gripped unit comes within the crush range while it is pulled in, and the grip ends there with a drop. Control: the same hold without a crushable victim lasts to arrival (claim 8)
#[test]
fn claim_11_part_h_crushing_approach_ends_the_grip() {
    let crushable = |r: &mut Ruleset| {
        r.sim.vehicles.get_mut("VICT").unwrap().crushable = Some(true);
        // Synthetic: 4 cells, less than the 6 cells the victim starts at.
        r.sim.general.crush_range = Some(d("4"));
    };
    let (mut w, l, v) = gripping(Behaviour::Hunt, crushable);
    let (t, dropped) = ended_by_part_h(&mut w, l, v);
    assert!(dropped, "claim 13: dropped in the air at tick {t}");
    let (mut w, l, v) = gripping(Behaviour::Hunt, |r| {
        crushable(r);
        r.sim.vehicles.get_mut("VICT").unwrap().crushable = Some(false);
    });
    let h = watch_hold(&mut w, l, v, 600);
    assert_eq!(h.dest, None, "control: no part h approach");
    assert!(h.arrived, "control: claim 8: arrival: {h:?}");
}

// spec: S-0607 claim 11 (revision 18: part h's approaches are not excepted, also when the lifter held that destination when it gripped) and claim 12 (row 2: the grip ends, the target cleared); S-0210 claim 11 (the omni-crusher variant, at any distance): an omni-crusher lifter whose crushing approach gave it its target as destination while its seizing shot flew ends the grip at its first processing after the seize
#[test]
fn claim_11_part_h_omni_crusher_ends_the_grip_at_once() {
    let (mut w, l, v) = gripping(Behaviour::Hunt, |r| {
        r.sim.vehicles.get_mut("LIFTER").unwrap().omni_crusher = Some(true);
    });
    let (t, _) = ended_by_part_h(&mut w, l, v);
    assert!(t < 60, "at the first processing: tick {t}");
}

// spec: S-0607 claim 11 (revision 18: the exception also covers part h's balloon-hover approach) and claim 12 (row 2: that destination ends the grip, the target cleared); S-0307 claim 18 (a balloon-hover vehicle whose weapon chosen for the target has a vertical projectile takes the target as its destination)
#[test]
fn claim_11_part_h_balloon_hover_approach_ends_the_grip() {
    let (mut w, l, v) = gripping(Behaviour::Hunt, |r| {
        r.sim.vehicles.get_mut("LIFTER").unwrap().balloon_hover = Some(true);
        let mut p = r.sim.projectiles["Bullet"].clone();
        p.vertical = Some(true);
        r.sim.projectiles.insert("Drop".into(), p);
        r.sim.weapons.get_mut("MagBeam").unwrap().projectile = Some("Drop".into());
    });
    ended_by_part_h(&mut w, l, v);
}

// spec: S-0607 claim 11 (revision 18: the per-tick drop of S-0215 claim 37 part a still applies, with its own range test after the approach step: a lifter with no destination that cannot reach its gripped unit's cell, the unit out of range with minimum range included, drops it as its target) and claims 12 and 21 (the grip stays, claim 12 has no row for a target change; the wave ends)
#[test]
fn claim_11_per_tick_drop_keeps_the_grip_and_ends_the_wave() {
    let (mut w, l, v) = gripping(Behaviour::Hunt, |_| {});
    // The cells next to the lifter, inside its minimum range, are water: the
    // lifter's ground movement zone cannot reach them (S-0212 claim 5: the
    // changed categories mark the reach regions for a rebuild).
    water(&mut w, &wet());
    for c in wet() {
        glasnost_sim::regions::cell_changed(&mut w, c);
    }
    glasnost_sim::regions::ensure(&mut w);
    let mut dropped_at = None;
    for _ in 0..600 {
        glasnost_sim::tick::step(&mut w);
        if !lift::grips(&w, l, v) {
            break;
        }
        let o = w.obj(l).unwrap();
        assert_eq!(o.dest, None, "claim 11: no approach destination");
        if o.target.is_none() {
            dropped_at = Some(w.tick);
            break;
        }
    }
    let t = dropped_at.expect("part a: the target is dropped");
    let vc = w.obj(v).unwrap().pos.cell();
    assert!(
        wet().contains(&vc),
        "dropped over an unreachable cell: {vc:?}"
    );
    assert!(
        !glasnost_sim::approach::in_range(&w, l, Target::Object(v)),
        "dropped out of range"
    );
    assert!(
        lift::grips(&w, l, v),
        "claim 12: the grip stays at tick {t}"
    );
    // Claim 21: the wave ends at its turn, after its firer's in the same
    // tick (its firer's target is no longer the wave's target), and fades.
    assert!(
        w.waves.values().any(|x| x.firer == l && x.ended),
        "claim 21: the wave ends"
    );
    // Control: the same hold over land keeps the target until arrival.
    let (mut w, l, v) = gripping(Behaviour::Hunt, |_| {});
    let h = watch_hold(&mut w, l, v, 600);
    assert_eq!(h.lost, None, "control: over land the target is kept");
}
