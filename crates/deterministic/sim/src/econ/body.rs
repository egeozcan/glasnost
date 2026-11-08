//! A building's *idle* body range as far as the simulation needs it: the
//! body-range ready mark of S-0215 claim 11 for a building without a
//! turret (S-0108 claim 6 for the stages and delays).
//!
//! Only the *idle* state is followed. A building enters it at game set-up
//! (S-0215 claim 1, S-0009 claim 40) and at its construction handover
//! (S-0406 claim 15), and leaves it when it enters construction or selling.
//! An owner change requests it again (S-0309 claim 31), also for a building
//! that is selling or being built up, where it can take the place of the
//! build-down or the build-up.
//! The other body states (*active*, *aux-1*, *aux-2*, *full*) and what
//! puts a building in them are not modelled: the range keeps running as
//! *idle* through them (interim, docs/development.md).

use serde::{Deserialize, Serialize};

use crate::world::{ObjectId, World};

/// The running *idle* body range of a building: its stage *k*, the tick
/// of its last stage change (entry, advance or wrap) and the delay it
/// runs with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdleBody {
    /// The stage *k*.
    pub stage: i32,
    /// The tick of the last stage change.
    pub since: i32,
    /// The delay in ticks the stage runs with.
    pub delay: i32,
    /// The range has set the ready mark while the building is selling: an
    /// idle range that took the place of a sale's build-down at an owner
    /// change has made its report (S-0309 claim 31, S-0408 claim 17).
    #[serde(default)]
    pub sale_report: bool,
}

/// The *idle* range (*a*, *c*, *d*) of a building type (S-0108 Import
/// mapping, `AnimIdle`) and its normalised mark (`Normalized`).
// spec: S-0108 Import mapping (AnimIdle: start, count, delay)
fn range(w: &World, t: u32) -> ([i32; 3], bool) {
    let r = w.rules();
    let Some(b) = r.buildings.get(t as usize) else {
        // spec: S-0108 Constants (default state range a = 0, c = 1, d = 0)
        return ([0, 1, 0], false);
    };
    // spec: S-0108 Import mapping (AnimIdle absent: 0, 1, 0; Normalized absent: false)
    (
        b.raw.econ.anim_idle.unwrap_or([0, 1, 0]),
        b.raw.econ.normalized.unwrap_or(false),
    )
}

/// N(*d*, *s*) of S-0107 F3 at the game speed, or the raw *d* with
/// `quirks.speed_normalised_delays` off (S-0108 Edge cases).
fn normalised(w: &World, d: i32) -> i32 {
    if w.rules().quirks.speed_normalised_delays {
        crate::infantry::normalised(d, w.game_speed)
    } else {
        // spec: S-0108 Edge cases (quirks.speed_normalised_delays off: the raw d)
        d
    }
}

/// Whether a building is a turreted building (S-0215 Terms): its type, or
/// an upgrade installed on it, has the turret flag.
pub fn turreted(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let r = w.rules();
    let flag = |t: u32| r.buildings.get(t as usize).is_some_and(|b| b.turret);
    // spec: S-0215 Terms (turreted building: the type or an installed upgrade has the turret flag)
    flag(o.type_index) || o.econ.building.upgrades.iter().flatten().any(|&u| flag(u))
}

/// Enters the *idle* state (S-0108 claim 6): at stage *a*. `immediate`
/// is a change during the game's set-up, whose delay is N(*d*, *s*) for
/// art marked normalised and *d* otherwise; a deferred change into *idle*
/// takes N(*d*, *s*) whatever the mark.
pub fn enter_idle(w: &mut World, id: ObjectId, immediate: bool) {
    let Some(o) = w.obj(id) else { return };
    let ([a, _, d], norm) = range(w, o.type_index);
    let wrap_rule = w.rules().quirks.building_wrap_normalisation;
    // spec: S-0108 claim 6 (immediate change, art marked normalised: N(d, s), else d; a deferred change into idle: N(d, s) whatever the mark); Edge cases (quirks.building_wrap_normalisation off: the immediate-entry rule everywhere)
    let delay = if (immediate || !wrap_rule) && !norm {
        d
    } else {
        normalised(w, d)
    };
    let t = w.tick;
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0108 claim 6 (the stage is a after the change)
        ob.econ.idle_body = Some(IdleBody {
            stage: a,
            since: t,
            delay,
            sale_report: false,
        });
    }
}

/// Leaves the *idle* state: the building enters construction or selling.
pub fn leave_idle(w: &mut World, id: ObjectId) {
    if let Some(ob) = w.obj_mut(id) {
        ob.econ.idle_body = None;
    }
}

/// The body part of a building turn (S-0108 claim 6, S-0215 claim 11),
/// before its first per-tick start, for a building without a turret: the
/// *idle* stage advances once its delay has passed and wraps past the
/// range's end; the building gets its ready mark when the stage advances onto the last stage,
/// or on every turn of a zero-delay range, which does not advance. A
/// wrap does not set the mark.
pub fn turn(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let Some(body) = o.econ.idle_body else { return };
    let selling = o.beh.seen() == Some(crate::rules::Behaviour::Selling);
    let building_work = selling || o.beh.seen() == Some(crate::rules::Behaviour::Construction);
    // spec: S-0215 claim 11 (for a turreted building the body-range rule applies only while its behaviour is construction or selling, a queued one seen while none is current included, so its idle range sets no mark otherwise; its stages have no other effect in the simulation)
    if turreted(w, id) && !building_work {
        return;
    }
    let ([a, c, d], norm) = range(w, o.type_index);
    let wrap_rule = w.rules().quirks.building_wrap_normalisation;
    let t = w.tick;
    let mut next = body;
    let mut mark = false;
    if body.delay <= 0 {
        // spec: S-0215 claim 11 (a zero-delay range: every non-advancing turn sets the mark)
        mark = true;
    } else if t.wrapping_sub(body.since) >= body.delay {
        next.stage = body.stage.wrapping_add(1);
        next.since = t;
        if next.stage >= a.wrapping_add(c) {
            // spec: S-0108 claim 6 (past the end of the range, k ≥ a + c, it wraps to a with N(d, s) for idle); Edge cases (quirks.building_wrap_normalisation off: the immediate-entry rule); S-0215 claim 11 (the wrap itself sets no mark)
            next.stage = a;
            next.delay = if !wrap_rule && !norm {
                d
            } else {
                normalised(w, d)
            };
        } else if next.stage == a.wrapping_add(c).wrapping_sub(1) {
            // spec: S-0215 claim 11 (advancing onto the last stage sets the mark)
            mark = true;
        }
    }
    if let Some(ob) = w.obj_mut(id) {
        if mark && selling {
            // spec: S-0309 claim 31 (the sale ends at the first selling processing that finds the ready mark set, which the idle range sets by S-0215 claim 11's rule)
            next.sale_report = true;
        }
        ob.econ.idle_body = Some(next);
        if mark {
            ob.beh.ready_mark = true;
        }
    }
}

/// A building placed at game set-up (S-0215 claim 1, S-0009 claim 40):
/// the idle body at once and *guard* queued, with no current behaviour;
/// placement's readiness test finds it not ready, and then its grand
/// opening sets the ready mark, so *guard* becomes current at its first
/// per-tick start. The opening's free unit is suppressed at set-up (S-0402
/// claim 18).
pub fn set_up(w: &mut World, id: ObjectId) {
    // spec: S-0215 claim 1 and S-0108 claim 6 (at game set-up: the idle body, a change during set-up being immediate)
    enter_idle(w, id, true);
    if let Some(o) = w.obj_mut(id) {
        o.beh.current = None;
        // spec: S-0215 claim 1 (at game set-up a building receives guard queued; the choice of idle body and queued guard does not itself set the ready mark, and a weapon is not a set-up ready-mark setter)
        o.beh.queued = Some(crate::rules::Behaviour::Guard);
        // spec: S-0215 claim 1, S-0108 claim 11 and S-0402 claim 18 (later in the same placement the building's grand opening runs and sets the ready mark, whatever its weapon, turret or upgrades; its free-unit grant is suppressed at set-up)
        o.beh.ready_mark = true;
    }
}
