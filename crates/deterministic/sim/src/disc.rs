//! The disc laser (S-0615 claims 12 to 18): a shot with a disc-laser weapon
//! makes no projectile. It makes a disc laser, which draws a ring of beams
//! round its firer on every second update and lands its damage 17 ticks
//! after the shot, in its own part of the tick (S-0203 row 12).
//!
//! A disc laser is not an object: it has no number and no place in the
//! update order. Disc lasers in play are kept newest first.

use glasnost_num::F64;
use glasnost_num::geom::{direction, root_floor};
use glasnost_num::trig::{cos_t, sin_t};
use serde::{Deserialize, Serialize};

use crate::combat::area::{self, Detonation};
use crate::combat::data::WeaponId;
use crate::combat::effects::{DiscEvent, Effect};
use crate::rules::Kind;
use crate::world::{HouseId, ObjectId, Pos, Target, World};

/// A colour: three integer channels.
pub type Colour = [i32; 3]; // spec: S-0615 Import mapping (three integer colour channels, S-0003)

/// The inner, outer and spread colours of a beam.
pub type Colours = [Colour; 3]; // spec: S-0615 claim 15 (inner, outer and spread colours)

/// The two arc beams of one step, as ring point pairs.
pub type ArcBeams = [(i32, i32); 2]; // spec: S-0615 claim 15 (two arc beams per step)

/// A disc laser in play (S-0615 claim 12).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Disc {
    /// The firer.
    pub firer: ObjectId,
    /// The target.
    pub target: Target,
    /// The weapon fired.
    pub weapon: WeaponId,
    /// The shot damage *D*ₛ, fixed at the shot (S-0302 claim 2).
    pub damage: i32,
    /// The start ring point *i*₀ (F4), fixed at the shot.
    pub start: u8,
    /// The updates it has had.
    pub updates: u8,
}

/// The number of ring points (S-0615 claim 17).
pub const RING_POINTS: i32 = 16; // spec: S-0615 Constants (ring points: 16)

/// The ring radius in leptons (S-0615 claim 17).
pub const RING_RADIUS: i32 = 240; // spec: S-0615 Constants (ring radius: 240 leptons)

/// The arc steps before the final step (S-0615 claim 13).
pub const ARC_STEPS: u8 = 8; // spec: S-0615 Constants (arc steps: 8, plus a final step)

/// The update in which a disc laser is removed: tick *T* + 18 (F5).
const REMOVAL_UPDATE: u8 = 18; // spec: S-0615 F5 (removal in tick T + 18; its first update is in T + 1)

/// π / 180 as the nearest double, for F6's conversion of whole degrees to
/// radians.
// spec: S-0615 F6 (φ converted to radians in double precision)
pub const DEGREE: F64 = F64(0x3f91_df46_a252_9d39);

/// The start ring point *i*₀ of F4 for the direction `f` from the firer to
/// the target.
pub fn start_point(f: u16) -> u8 {
    // spec: S-0615 F4 (i₀ = (⌊(⌊f / 2048⌋ + 1) / 2⌋ mod 16 + 8) mod 16)
    let sixteenth = ((i32::from(f) / 2048 + 1) / 2).rem_euclid(16);
    (sixteenth + 8).rem_euclid(16) as u8 // spec: S-0615 F4 (turned half a turn, mod 16)
}

/// The displacement of ring point `i` from the ring centre (F6): whole
/// leptons, Δ*z* = 0.
pub fn ring_point(i: i32, mode: glasnost_num::Rounding) -> (i32, i32) {
    let i = i.rem_euclid(RING_POINTS);
    // spec: S-0615 F6 (φᵢ = (270 + ⌊22.5 i⌋) mod 360 whole degrees)
    let phi = (270 + (45 * i) / 2).rem_euclid(360);
    let rad = F64::from_i64(i64::from(phi), mode).mul(DEGREE, mode);
    let r = F64::from_i64(i64::from(RING_RADIUS), mode);
    // spec: S-0615 F6 (Δx = trunc(240 cos φ), Δy = trunc(240 sin φ), table trigonometry of S-0105 F1, truncation toward zero)
    let dx = r.mul(cos_t(rad, mode), mode).to_i32_common();
    let dy = r.mul(sin_t(rad, mode), mode).to_i32_common();
    (dx, dy)
}

/// The two arc beams of arc step `k` (claim 15): the ring point pairs
/// (*a*, *a* + 1) and (*b*, *b* − 1), modulo 16.
pub fn arc_pairs(start: u8, k: u8) -> ArcBeams {
    let (i0, k) = (i32::from(start), i32::from(k));
    // spec: S-0615 claim 15 (a = (i₀ + k) mod 16 to a + 1; b = (i₀ − k) mod 16 to b − 1)
    let a = (i0 + k).rem_euclid(RING_POINTS);
    let b = (i0 - k).rem_euclid(RING_POINTS);
    [
        (a, (a + 1).rem_euclid(RING_POINTS)),
        (b, (b - 1).rem_euclid(RING_POINTS)),
    ]
}

/// The beam colours with the house-colour flag (claim 15): the house laser
/// colour inside, each of its channels halved toward zero outside, the
/// weapon's spread colour kept. The drawing applies it; the simulation
/// carries no house laser colours.
pub fn house_colours(house: Colour, spread: Colour) -> Colours {
    // spec: S-0615 claim 15 (outer = the house colour with each channel halved, toward zero, integer arithmetic; spread kept)
    [house, house.map(|c| c / 2), spread]
}

/// A shot with a disc-laser weapon (claim 12): the disc laser is made, newest
/// first, holding the start ring point of F4.
pub fn shot(w: &mut World, firer: ObjectId, weapon: WeaponId, target: Target, damage: i32) {
    let q = w.rules().quirks;
    let from = w.reference_point(firer);
    let to = crate::combat::range::target_point(w, target);
    // spec: S-0615 F4 (f, the direction from the firer to the target at the shot, S-0204 claim 8)
    let f = direction(
        i64::from(to.x - from.x),
        i64::from(to.y - from.y),
        q.table_direction,
    );
    let disc = Disc {
        firer,
        target,
        weapon,
        damage,
        start: start_point(f),
        updates: 0,
    };
    // spec: S-0203 claim 1 row 12 (disc lasers, the most recently fired first)
    w.discs.insert(0, disc);
    w.effects.push(Effect::Disc {
        firer,
        weapon,
        event: DiscEvent::Shot,
        pos: from,
    });
}

/// Whether an object is in play for a disc laser (claim 14): it exists, is
/// on the map, is not inside a building or bunker and is not killed.
fn in_play(w: &World, x: ObjectId) -> bool {
    // spec: S-0615 claim 14 (cancelled as soon as its firer or target leaves play: off the map, removed, or no longer a valid object to point at; a killed object counts as gone, interim, Q-1422)
    w.obj(x).is_some_and(|o| {
        o.on_map && o.structure.inside.is_none() && !crate::combat::damage::dead(o)
    }) && crate::structures::bunker::bunkered_in(w, x).is_none()
}

/// Whether the firer or the target has left play (claim 14).
fn lost(w: &World, d: &Disc) -> bool {
    !in_play(w, d.firer) || matches!(d.target, Target::Object(t) if !in_play(w, t))
}

/// The distance of the range re-check (claim 14).
pub fn recheck_distance(w: &World, firer: ObjectId, target: Target) -> i32 {
    let r = w.rules();
    // The firer at its reference point, the target at its target point (interim, Q-1422).
    let s = w.reference_point(firer);
    let m = crate::combat::range::target_point(w, target);
    let (dx, dy) = (i64::from(s.x - m.x), i64::from(s.y - m.y));
    // spec: S-0615 claim 14 (the vertical difference is 0 when the firer is in the air)
    let dz = if crate::combat::weapon::airborne_by_height(w, firer) {
        0
    } else {
        i64::from(s.z - m.z)
    };
    // spec: S-0615 claim 14 (the table-root distance in three dimensions, S-0204 claim 5)
    let n = (dx * dx + dy * dy + dz * dz).unsigned_abs();
    let d = i64::try_from(root_floor(n, r.quirks.table_root_distance, r.mode)).unwrap_or(i64::MAX);
    // spec: S-0615 claim 14 (a building target: 64 (W + H) leptons less, clamped at 0)
    let less = match target {
        Target::Object(x) => w
            .obj(x)
            .filter(|o| o.kind == Kind::Building)
            .map_or(0, |o| 64 * i64::from(o.foundation.0 + o.foundation.1)),
        Target::Cell(_) => 0,
    };
    i32::try_from((d - less).max(0)).unwrap_or(i32::MAX)
}

/// Whether a disc laser passes claim 14's re-check.
fn recheck(w: &World, d: &Disc) -> bool {
    if lost(w, d) {
        return false;
    }
    // spec: S-0615 claim 14 (cancelled when the firer has begun a crash, S-0312 claim 16)
    if crate::jump::crashing(w, d.firer) || crate::air::crashing(w, d.firer) {
        return false;
    }
    let range = w.rules().combat.weapon(d.weapon).range;
    // spec: S-0615 claim 14 (removed when the distance exceeds the weapon's range field, with no additional range term of S-0307)
    // An unlimited range field (−512) is compared as it is, so it always cancels (interim, Q-1426).
    recheck_distance(w, d.firer, d.target) <= range
}

/// The detonation of the final step (claim 16): at the target's target
/// point, *D*ₛ and the weapon's warhead, the firer as attacker, no attacking
/// house (`quirks.disc_laser_no_house`; off: the firer's owner), resource
/// reduction allowed.
pub fn detonation(w: &World, d: &Disc) -> Detonation {
    let house: Option<HouseId> = if w.rules().quirks.disc_laser_no_house {
        // spec: S-0615 claim 16 and Edge cases (quirks.disc_laser_no_house: no attacking house)
        None
    } else {
        // spec: S-0615 Edge cases (quirks.disc_laser_no_house off: the firer's owner, as for a projectile's detonation, S-0302 claim 1)
        w.obj(d.firer).and_then(|o| o.owner)
    };
    // spec: S-0615 claim 16 (at the target's target point; D_s and the weapon's warhead; the firer is the attacker; resource reduction allowed, S-0303 claim 17)
    Detonation {
        point: crate::combat::range::target_point(w, d.target),
        damage: d.damage,
        warhead: w.rules().combat.weapon(d.weapon).warhead,
        source: Some(d.firer),
        house,
        resources: true,
    }
}

/// The beam colours of a disc laser's weapon and firer (claim 15): the
/// weapon's inner, outer and spread colours, and the house whose laser
/// colour replaces them when the weapon has the house-colour flag.
fn colours(w: &World, d: &Disc) -> (Colours, Option<HouseId>) {
    let wr = w.rules().combat.weapon(d.weapon);
    // spec: S-0615 claim 15 (the weapon's inner, outer and spread colours; with the house-colour flag the firer's house laser colour, house_colours)
    let house = if wr.house_colour {
        w.obj(d.firer).and_then(|o| o.owner)
    } else {
        None
    };
    (wr.laser_colours, house)
}

/// The disc lasers' part of the tick (S-0203 row 12): each one, newest
/// first, takes its update.
pub fn update(w: &mut World) {
    let discs = std::mem::take(&mut w.discs);
    let mut kept = Vec::with_capacity(discs.len());
    for mut d in discs {
        if step(w, &mut d) {
            kept.push(d);
        }
    }
    // Disc lasers are only made in the turn pass, so none was added here.
    kept.append(&mut w.discs);
    w.discs = kept;
}

/// One update of a disc laser (claim 13, F5). Returns whether it stays.
fn step(w: &mut World, d: &mut Disc) -> bool {
    d.updates = d.updates.saturating_add(1);
    let n = d.updates;
    // spec: S-0615 F5 (removed in its update in tick T + 18)
    if n >= REMOVAL_UPDATE {
        return false;
    }
    // spec: S-0615 claim 14 (cancelled, without damage, as soon as its firer or its target leaves play: at any update, not only at a step)
    if lost(w, d) {
        w.effects.push(Effect::Disc {
            firer: d.firer,
            weapon: d.weapon,
            event: DiscEvent::Cancelled,
            pos: w.obj(d.firer).map_or_else(Pos::default, |o| o.pos),
        });
        return false;
    }
    // spec: S-0615 claim 13 and F5 (step k in tick T + 1 + 2k: every second update, from the first)
    if n.is_multiple_of(2) {
        return true;
    }
    let k = (n - 1) / 2; // spec: S-0615 F5 (step k in update 2k + 1)
    // spec: S-0615 claims 13 and 14 (every step, the final one included, first makes the re-check; a failing one removes it with no damage and nothing drawn)
    if !recheck(w, d) {
        w.effects.push(Effect::Disc {
            firer: d.firer,
            weapon: d.weapon,
            event: DiscEvent::Cancelled,
            pos: w.reference_point(d.firer),
        });
        return false;
    }
    let mode = w.rules().mode;
    let (colours, house) = colours(w, d);
    // spec: S-0615 claim 15 (the ring is centred at the slot-0 fire point, S-0313, at the time of the step, at its height)
    let burst = w.obj(d.firer).map_or(0, |o| o.combat.burst);
    let centre = crate::combat::position::fire_point(w, d.firer, 0, burst);
    let at = |i: i32| {
        let (dx, dy) = ring_point(i, mode);
        Pos::new(centre.x + dx, centre.y + dy, centre.z)
    };
    if k < ARC_STEPS {
        if k == 0 {
            // spec: S-0615 claim 15 (the charge-up sound at k = 0 only, if given)
            let sound = w.rules().combat.general.disk_laser_charge_up.clone();
            w.effects.push(Effect::Disc {
                firer: d.firer,
                weapon: d.weapon,
                event: DiscEvent::ChargeUp { sound },
                pos: centre,
            });
        }
        for (p, q) in arc_pairs(d.start, k) {
            // spec: S-0615 claim 15 (each beam lasts 8 − k ticks and fades from full to half intensity)
            w.effects.push(Effect::Beam {
                from: at(p),
                to: at(q),
                duration: i32::from(ARC_STEPS - k),
                fade_out: false,
                colours,
                house,
            });
        }
        return true;
    }
    // spec: S-0615 claim 16 (k = 8: a = b = i₀ + 8 mod 16, the ring point facing the target as of the shot)
    let a = (i32::from(d.start) + 8).rem_euclid(RING_POINTS);
    let det = detonation(w, d);
    let duration = w.rules().combat.weapon(d.weapon).laser_duration;
    // spec: S-0615 claim 16 (a beam from ring point a to the target's target point, lasting the laser-duration field, fading from full to none)
    w.effects.push(Effect::Beam {
        from: at(a),
        to: det.point,
        duration,
        fade_out: true,
        colours,
        house,
    });
    let report = w.obj(d.firer).map_or(0, |o| o.report_value);
    // spec: S-0615 claim 16 (the report sound plays here; its variant from the firer's own stored value, S-0701 claim 21)
    w.effects.push(Effect::Disc {
        firer: d.firer,
        weapon: d.weapon,
        event: DiscEvent::Hit { report },
        pos: det.point,
    });
    // Area damage only: no explosion animation, debris or special-hit selection, which S-0303
    // claims 21 and 22 tie to a projectile (interim, Q-1420).
    area::detonate(w, det);
    // spec: S-0615 claim 16 (removed at its next update)
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn degree_is_the_nearest_double() {
        // spec: S-0615 F6 (radians in double precision)
        assert_eq!(DEGREE, crate::rules::nearest_double("0.017453292519943295"));
    }
}
