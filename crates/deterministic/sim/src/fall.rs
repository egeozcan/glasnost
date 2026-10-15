//! Falling objects (S-0511 claim 10 and F3; S-0522 claim 12; S-0312
//! Terms, the bomb mark).
//!
//! An object dropped from a lost bridge deck (S-0511 claim 9), or a hovering
//! vehicle that sinks (S-0503 claim 11), falls on each of its own updates:
//! its height changes by its fall rate, the rate then follows F3's
//! recurrence, and when it reaches the terrain the fall ends. A dropped
//! paratrooper (S-0522 claim 12) falls the same way, under the parachute
//! recurrence while its parachute is attached, and lands on the bridge deck
//! when it is on the deck layer. One that lands
//! with the bomb mark and positive health takes the demolition hit.

use glasnost_num::F64;
use serde::{Deserialize, Serialize};

use crate::combat::damage::{self, Hit};
use crate::world::{Layer, ObjectId, World};

/// An object's falling state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FallState {
    /// The fall rate *v*, leptons per update (S-0511 F3). It is kept after a
    /// fall ends: the next fall starts from it (claim 10).
    pub rate: i32,
    /// Whether the object is falling.
    pub falling: bool,
    /// The bomb mark (S-0312 Terms): a marked object is killed when it lands
    /// (S-0511 claim 10).
    pub bomb_mark: bool,
    /// A parachute animation is attached (S-0522 claims 12 and 13): the
    /// parachute recurrence of S-0511 F3 applies. The simulation keeps no
    /// playback clock (S-0109 claim 28 seam), so the parachute stays
    /// attached until the landing.
    #[serde(default)]
    pub chute: bool,
    /// The absent-type transient of S-0522 claim 13: `chute` is set for the
    /// release-tick fall update only, and both are cleared by it.
    #[serde(default)]
    pub chute_once: bool,
}

/// Whether `id` is falling.
pub fn falling(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.fall.falling)
}

/// Whether `id` carries the bomb mark of a fall (S-0312 Terms).
pub fn bomb_mark(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.fall.bomb_mark)
}

/// Starts a fall (S-0511 claims 9 and 10): the object keeps the fall rate it
/// already holds, and gets the bomb mark when `bomb` is set.
pub fn start(w: &mut World, id: ObjectId, bomb: bool) {
    if let Some(o) = w.obj_mut(id) {
        o.fall.falling = true;
        // spec: S-0511 claim 10 (its fall rate starts from the value it already holds)
        if bomb {
            o.fall.bomb_mark = true;
        }
    }
}

/// The rate recurrence of S-0511 F3 for an object with no attached
/// animation: *v*ₙ₊₁ = max(trunc(*v*ₙ − 1.4), *v*_min), in double precision.
pub fn next_rate(w: &World, v: i32) -> i32 {
    let mode = w.rules().mode;
    let v_min = w.rules().combat.general.no_parachute_max_fall_rate;
    // spec: S-0511 F3 (trunc(v − 1.4) of a double-precision value)
    let t = F64::from_i64(i64::from(v), mode)
        .sub(crate::consts::FALL_STEP, mode)
        .to_i32_common();
    // spec: S-0511 F3 (the floor v_min, the imported no-parachute maximum fall rate)
    t.max(v_min)
}

/// The rate recurrence of S-0511 F3 while an animation is attached:
/// *v*ₙ₊₁ = max(*v*ₙ − 1, the imported parachute maximum fall rate), in
/// exact integers.
pub fn next_chute_rate(w: &World, v: i32) -> i32 {
    // spec: S-0511 F3 (the attached-animation recurrence: exact integer subtraction of 1 and the parachute floor)
    v.saturating_sub(1)
        .max(w.rules().combat.general.parachute_max_fall_rate)
}

/// Whether the parachute recurrence applies to `o` (S-0522 claim 12). The
/// only animation our simulation attaches to an object is a paratrooper's
/// parachute (claim 13), so both settings of
/// `quirks.paratrooper_any_animation_slow_fall` (any attached animation, or
/// the parachute only) select the same objects.
fn slow(w: &World, o: &crate::world::Object) -> bool {
    // spec: S-0522 Edge cases (quirks.paratrooper_any_animation_slow_fall: any attached animation, or only the parachute; the parachute is the only attachment, so both coincide for a held type; for an absent type the quirk selects the release-tick transient of claim 13, set at the release)
    let _ = w;
    o.fall.chute
}

/// The surface a falling object lands on at its position (S-0522 claim 12):
/// the bridge deck, 416 leptons above the terrain surface, for an object on
/// the deck layer; the terrain surface otherwise.
fn landing_surface(w: &World, o: &crate::world::Object) -> i32 {
    let ground = w.surface_at(o.pos.x, o.pos.y);
    if o.layer == Layer::Deck {
        // spec: S-0522 claim 12 (the deck on a bridge cell); S-0207 Constants (416 leptons)
        ground + crate::teleport::DECK_HEIGHT
    } else {
        ground
    }
}

/// One falling update of `id` (S-0511 claim 10, F3), before its behaviour
/// processing; while it still falls its locomotor does not advance.
/// Returns whether it moved.
pub fn turn(w: &mut World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let v = o.fall.rate;
    let pos = o.pos;
    // spec: S-0522 claim 12 (the parachute recurrence while an animation is attached, the no-parachute one otherwise)
    let next = if slow(w, o) {
        next_chute_rate(w, v)
    } else {
        next_rate(w, v)
    };
    // spec: S-0511 F3 (h_{n+1} = h_n + v_n: the height changes by v before v is updated)
    let z = pos.z + v;
    let ground = landing_surface(w, o);
    // spec: S-0522 claim 12 and Constants (a height above the surface below 1 lepton after a fall step lands); S-0511 claim 10
    let landed = z - ground < 1;
    if let Some(ob) = w.obj_mut(id) {
        ob.pos.z = z;
        // spec: S-0511 F3 (v_{n+1} follows every update, the landing update included; the landing test follows the move)
        ob.fall.rate = next;
        if ob.fall.chute_once {
            // spec: S-0522 claim 13 (an absent parachute type: the parachute recurrence applied once, in the release-tick update; there is no durable attachment, so the no-parachute recurrence follows)
            ob.fall.chute_once = false;
            ob.fall.chute = false;
        }
        if landed {
            // spec: S-0511 claim 10 (height above the terrain 0 or less after the move: height 0, the fall ends)
            ob.pos.z = ground;
            ob.fall.falling = false;
        }
    }
    if landed {
        // spec: S-0516 claim 22 (a completed fall landing is a cell-arrival update, also within the same cell, after the height becomes 0 and falling ends and before the parachute is told to end: the sensor disc moves and the adjacent sensor test applies; the descent itself moved nothing)
        crate::cloak::machine::cell_movement_completed(w, id);
        if let Some(ob) = w.obj_mut(id) {
            // spec: S-0522 claim 13 and State and ordering (a parachute still present on landing makes its current pass the last; the simulation keeps no playback clock, S-0109 claim 28, so it is detached here)
            ob.fall.chute = false;
        }
        land(w, id);
    }
    v != 0
}

/// The landing of a falling object (S-0511 claim 10): one with the bomb mark
/// and positive health takes a hit of its current health, distance 0, with
/// the demolition warhead, ignoring defences and carrying the no-escape
/// marker, with no attacker and no attacking house.
fn land(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let marked = o.fall.bomb_mark;
    let health = o.health;
    // spec: S-0511 claim 10 (one hit per landing, not one on every later grounded update; landing does not spend or clear the bomb mark, even when the object survives, so a later fall without a new mark still ends with this hit)
    if marked && health > 0 {
        landing_hit(w, id, health);
    }
}

/// The landing hit of a marked object (S-0511 claim 10).
fn landing_hit(w: &mut World, id: ObjectId, health: i32) {
    // spec: S-0511 claim 10 (the landing hit: current health, distance 0, the demolition warhead, ignoring defences, no-escape, no attacker, no house)
    let hit = Hit {
        amount: health,
        warhead: w.rules().combat.general.c4_warhead,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: true,
    };
    damage::apply(w, id, hit);
}
