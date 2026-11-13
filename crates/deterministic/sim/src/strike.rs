//! The air-strike designation (S-0519 claims 16 to 21): a shot with an
//! air-strike warhead calls a group of aircraft onto an object. A designator
//! has at most one strike (created with its first designation; the state
//! before that is the idle strike with no target); the strike links its
//! aircraft, holds a strike target and the end tick of its last run.
//! Sounds and the mark's colour and brightness are presentation; its phase
//! sequence, restarted here, is game state (`crate::mark`, claim 26). The
//! readiness test, a cancel and a redirect take no random draw; a start
//! takes the arrival cell's and the aircraft's (claim 21); the mark's own
//! draw comes later, in the building's turn.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use glasnost_map::CellXY;
use glasnost_rules::sim::{LocomotionClass, MovementZone};

use crate::combat::veterancy::{self, Rank};
use crate::rules::{Behaviour, Kind};
use crate::world::{ObjectId, Pos, Target, World};

/// One designator's strike (S-0519 Terms).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Strike {
    /// The linked aircraft, oldest placed first.
    pub aircraft: Vec<ObjectId>,
    /// The strike target (the marked building when it is one).
    pub target: Option<ObjectId>,
    /// The end tick *e* of the last strike that ended.
    pub end: i32,
    /// Whether a strike has ended at least once (claim 18).
    pub ended: bool,
    /// In progress (Terms): from a start until the last aircraft is unlinked.
    pub progress: bool,
    /// Active (claim 20): from a start that placed aircraft, or a redirect
    /// that re-aimed one, until the next cancel or end.
    pub active: bool,
    /// The designator's target at the last look (claim 19: a change cancels).
    pub watched: Option<Target>,
    /// The designator's count of target changes at the last look.
    #[serde(default)]
    pub watched_epoch: u32,
}

/// All strikes of a game, by designator.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrikeWorld {
    /// The strikes.
    pub strikes: BTreeMap<ObjectId, Strike>,
}

/// The size, aircraft type name and recharge for the designator's rank
/// (claim 17, 18); `None` when the object is not a designator (Terms).
fn parameters(w: &World, id: ObjectId) -> Option<(i32, Option<String>, i32)> {
    let tc = veterancy::type_combat(w, id)?;
    // spec: S-0519 Terms (a designator's type has a positive normal team size) and Edge cases (no strike aircraft type: no designator; the importer warns)
    if tc.airstrike_team <= 0 || tc.airstrike_team_type[0].is_none() {
        return None;
    }
    // spec: S-0519 Terms (the elite values apply while the designator's rank is elite)
    let e = usize::from(veterancy::rank(w, id) == Rank::Elite);
    Some((
        if e == 1 {
            tc.elite_airstrike_team
        } else {
            tc.airstrike_team
        },
        tc.airstrike_team_type[e].clone(),
        tc.airstrike_recharge[e],
    ))
}

/// Whether the object is a designator.
pub fn designator(w: &World, id: ObjectId) -> bool {
    parameters(w, id).is_some()
}

/// Whether the object is a strike aircraft (linked to a strike).
pub fn linked(w: &World, id: ObjectId) -> bool {
    w.strike.strikes.values().any(|s| s.aircraft.contains(&id))
}

/// Whether the building is marked by a strike (presentation, claim 17).
pub fn marked(w: &World, b: ObjectId) -> bool {
    w.obj(b).is_some_and(|o| o.kind == Kind::Building)
        && w.strike.strikes.values().any(|s| s.target == Some(b))
}

/// The designator's turn: a change of its target cancels (claim 19).
pub fn turn(w: &mut World, id: ObjectId) {
    if w.strike.strikes.contains_key(&id) {
        sync(w, id);
    }
}

/// Looks at every designator's target after an order or an object's turn:
/// a change cancels the strike in the tick it applies (claim 19, revision 3),
/// not at the strike's next look, the designator's next turn or the next
/// designation impact.
pub fn sweep(w: &mut World) {
    if w.strike.strikes.is_empty() {
        return;
    }
    let ds: Vec<ObjectId> = w.strike.strikes.keys().copied().collect();
    for d in ds {
        sync(w, d);
    }
}

/// Looks at the designator's target (claim 19): any change to anything
/// else, including none, cancels the strike; the same target does not.
fn sync(w: &mut World, d: ObjectId) {
    let now = w.obj(d).and_then(|o| o.target);
    let epoch = w.obj(d).map_or(0, |o| o.combat.target_epoch);
    let Some(s) = w.strike.strikes.get(&d) else {
        return;
    };
    // spec: S-0519 claim 19 (a change from A to B and back to A within one tick still cancels: the changes are counted; restoring A does not undo the retreat orders, ammunition loss or forgotten strike target)
    if s.watched == now && s.watched_epoch == epoch {
        return;
    }
    cancel(w, d);
    if let Some(s) = w.strike.strikes.get_mut(&d) {
        s.watched = now;
        s.watched_epoch = epoch;
    }
}

/// A designation shot detonates (claim 16): the firer must still exist and
/// be a designator, the target an object of a kind that can be hit.
pub fn designation(w: &mut World, firer: Option<ObjectId>, target: Option<Target>) {
    let (Some(d), Some(Target::Object(x))) = (firer, target) else {
        return;
    };
    // spec: S-0519 claim 16 (the target is a unit, an infantry unit, an aircraft or a building)
    if w.obj(x).is_none_or(|o| {
        !matches!(
            o.kind,
            Kind::Vehicle | Kind::Infantry | Kind::Aircraft | Kind::Building
        )
    }) {
        return;
    }
    // spec: S-0519 claim 16 (the firer still exists and is a designator)
    let Some((_, _, recharge)) = parameters(w, d) else {
        return;
    };
    // The strike exists from the designator's first look at its target.
    let look = w.obj(d).and_then(|o| o.target);
    let epoch = w.obj(d).map_or(0, |o| o.combat.target_epoch);
    w.strike.strikes.entry(d).or_insert_with(|| Strike {
        watched: look,
        watched_epoch: epoch,
        ..Default::default()
    });
    sync(w, d);
    let q = w.rules().quirks;
    let tick = w.tick;
    let s = w.strike.strikes.get(&d).cloned().unwrap_or_default();
    // spec: S-0519 claim 18 and F5 (ready: the target is not the strike target, or the strike is idle and t >= e + R; exact integer comparison)
    let recharged = !s.progress && tick.wrapping_sub(s.end.wrapping_add(recharge)) >= 0;
    let different = s.target != Some(x);
    let ready = if !q.airstrike_recharge_same_target_only && s.ended {
        // spec: S-0519 F5 and quirks.airstrike_recharge_same_target_only off (an idle strike that has ended before is ready only after e + R, whatever its target; in progress: unchanged)
        (!s.progress && recharged) || (s.progress && different)
    } else {
        different || recharged
    };
    if !ready {
        return;
    }
    if s.progress {
        redirect(w, d, x);
    } else {
        start(w, d, x);
    }
}

/// Starts a strike (claim 17).
fn start(w: &mut World, d: ObjectId, x: ObjectId) {
    let Some((n, ty, _)) = parameters(w, d) else {
        return;
    };
    let Some(house) = w.obj(d).and_then(|o| o.owner) else {
        return;
    };
    let ti = ty.and_then(|t| w.rules().index_of(Kind::Aircraft, &t));
    let mut placed: Vec<ObjectId> = Vec::new();
    if n > 0
        && let Some(ti) = ti
    {
        // spec: S-0519 claim 17 (one arrival cell for the whole strike, chosen before the first aircraft is created, as a paradrop plane's: S-0522 claims 4 and 5)
        let arrival = crate::air::house_edge_cell(w, house);
        crate::regions::ensure(w);
        // spec: S-0519 claim 17 (the aircraft's target is the cell holding the designated object's reference point, each coordinate divided by 256 and rounded toward zero)
        let rp = w.reference_point(x);
        let aim = CellXY::new(rp.x / 256, rp.y / 256); // spec: S-0519 Constants (strike target cell divisor 256 leptons per cell)
        for _ in 0..n {
            // spec: S-0519 claims 17 and 21 (each aircraft in creation order: its creation draws, then its placement; no credits, factory, prerequisite, limit or queue)
            let id = crate::objects::create(w, Kind::Aircraft, ti, Some(house));
            // spec: S-0519 claim 17 (placed at the nearby-cell search's result centred on the arrival cell: class winged, zone normal, no reach, no deck, no height condition, no further test, bridges allowed, no reference)
            let settings = crate::path::NearbySettings {
                class: Some(LocomotionClass::Winged),
                zone: MovementZone::Normal,
                reach: None,
                deck: false,
                height: false,
                reference: None,
                no_bridges: false,
                no_overlay: false,
            };
            let cell = crate::path::nearby(w, arrival, crate::path::NearbyUse::Settings(settings));
            let Some(cell) = cell else {
                // spec: S-0519 Edge cases (an aircraft that cannot be placed is discarded and no further aircraft is created; we count the aircraft actually placed)
                crate::objects::remove(w, id);
                break;
            };
            if let Some(o) = w.obj_mut(id) {
                // spec: S-0519 Terms (a strike aircraft is a visiting aircraft)
                o.air = Some(crate::air::AirState {
                    visiting: true,
                    ..Default::default()
                });
            }
            // spec: S-0519 claim 17 (placed at the centre of that cell, facing 0, S-0501 claim 1)
            crate::objects::place(w, id, Pos::centre(cell, 0), 0);
            let (t, stale) = (w.tick, w.rules().quirks.stale_suspended_revert);
            if let Some(o) = w.obj_mut(id) {
                // spec: S-0519 claim 17 (each starts attack, with no destination, aimed at the cell, not the object)
                o.target = Some(Target::Cell(aim));
                o.dest = None;
                o.beh.force(Behaviour::Attack, t, stale);
            }
            placed.push(id);
        }
    }
    let stuck = w.rules().quirks.airstrike_empty_strike_stuck;
    let Some(s) = w.strike.strikes.get_mut(&d) else {
        return;
    };
    if placed.is_empty() {
        // spec: S-0519 Edge cases and quirks.airstrike_empty_strike_stuck (a start that places no aircraft leaves the strike in progress for ever; off: idle, target and end unchanged)
        if stuck {
            s.progress = true;
        }
        return;
    }
    // spec: S-0519 claim 17 (at least one aircraft placed: the object becomes the strike target and the strike is in progress and active; the attack sound and the mark are presentation)
    s.aircraft = placed;
    s.target = Some(x);
    s.progress = true;
    s.active = true;
    // spec: S-0519 claims 17 and 26 (a designated building is marked: its phase sequence restarts and its mark clock starts)
    crate::mark::restart(w, x);
}

/// Cancels the strike (claim 19): ammunition, retreat, forgotten target.
pub fn cancel(w: &mut World, d: ObjectId) {
    let Some(s) = w.strike.strikes.get(&d) else {
        return;
    };
    let linked: Vec<ObjectId> = s
        .aircraft
        .iter()
        .copied()
        .filter(|&a| w.obj(a).is_some())
        .collect();
    // spec: S-0519 claim 19 (if any linked aircraft had no ammunition, every linked aircraft now has 0 rounds)
    if linked.iter().any(|&a| !crate::air::ammo_left(w, a)) {
        for &a in &linked {
            if let Some(o) = w.obj_mut(a) {
                o.combat.ammo = 0;
            }
        }
    }
    // spec: S-0519 claim 19 and Constants (the most recently placed linked aircraft, at most 101, have no target or destination and queue retreat)
    for &a in linked.iter().rev().take(101) {
        crate::air::forget(w, a);
        crate::air::queue(w, a, Behaviour::Retreat);
    }
    if let Some(s) = w.strike.strikes.get_mut(&d) {
        // spec: S-0519 claim 19 (the strike forgets its strike target; the aircraft stay linked; the mark passes to another strike on the same building by being derived from the targets)
        s.target = None;
        s.active = false;
    }
}

/// A ready designation while the strike is in progress (claim 19).
fn redirect(w: &mut World, d: ObjectId, x: ObjectId) {
    let Some(s) = w.strike.strikes.get(&d) else {
        return;
    };
    let linked: Vec<ObjectId> = s
        .aircraft
        .iter()
        .copied()
        .filter(|&a| w.obj(a).is_some())
        .collect();
    let first = linked
        .first()
        .and_then(|&a| w.obj(a))
        .map(|o| o.combat.ammo);
    // spec: S-0519 claim 19 (aircraft holding unequal ammunition: the strike is cancelled again instead and nothing is redirected)
    if linked
        .iter()
        .any(|&a| w.obj(a).map(|o| o.combat.ammo) != first)
    {
        cancel(w, d);
        return;
    }
    let mut aimed = false;
    for &a in &linked {
        if crate::air::ammo_left(w, a)
            && let Some(o) = w.obj_mut(a)
        {
            // spec: S-0519 claim 19 (every aircraft that still has ammunition takes the object itself as its target and queues attack; those without keep retreating)
            o.target = Some(Target::Object(x));
            o.beh.queue(Some(Behaviour::Attack));
            aimed = true;
            // spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: at the change of the firer's target)
            crate::emitters::retarget_let_go(w, a);
        }
    }
    if let Some(s) = w.strike.strikes.get_mut(&d) {
        // spec: S-0519 claim 19 (the object becomes the strike target; active when at least one aircraft was re-aimed)
        s.target = Some(x);
        if aimed {
            s.active = true;
        }
    }
    // spec: S-0519 claims 19 and 26 (a building T is marked as at a start: its phase sequence restarts)
    crate::mark::restart(w, x);
}

/// An object leaves play (claims 19 and 20): a strike aircraft is unlinked,
/// the last one ends the strike; a designator ends its strike; a marked
/// building cancels it. For an object already handled at its killing hit
/// ([`killed`]) only the marked building's cancel can remain.
pub fn leaving(w: &mut World, id: ObjectId) {
    end_for(w, id, true);
}

/// An object is destroyed, at its killing hit (claim 20, #161): a strike
/// aircraft stops being linked, the last one ends the strike; a designator
/// ends its strike. A destroyed marked building is cancelled through the
/// designator's loss of its target in the same tick ([`sweep`], claim 19).
pub fn killed(w: &mut World, id: ObjectId) {
    // spec: S-0519 claim 20 (a strike aircraft stops being linked when it is destroyed or removed from play; the designator's strike ends from its killing hit) (#161)
    end_for(w, id, false);
}

/// Claims 19 and 20 for `id`; `marked`: also the marked building's cancel.
fn end_for(w: &mut World, id: ObjectId, marked: bool) {
    if w.strike.strikes.is_empty() {
        return;
    }
    let tick = w.tick;
    // spec: S-0519 claim 20 (the designator destroyed or removed: the strike is cancelled, all its aircraft stop being linked, the strike is gone; the abort sound is presentation)
    if w.strike.strikes.contains_key(&id) {
        cancel(w, id);
        w.strike.strikes.remove(&id);
    }
    let ds: Vec<ObjectId> = w.strike.strikes.keys().copied().collect();
    for d in ds {
        // spec: S-0519 claim 19 (the marked building destroyed or removed cancels)
        let marked_gone = marked
            && w.strike
                .strikes
                .get(&d)
                .is_some_and(|s| s.target == Some(id))
            && w.obj(id).is_some_and(|o| o.kind == Kind::Building);
        if marked_gone {
            cancel(w, d);
        }
        let Some(s) = w.strike.strikes.get_mut(&d) else {
            continue;
        };
        if let Some(i) = s.aircraft.iter().position(|&a| a == id) {
            s.aircraft.remove(i);
            // spec: S-0519 claim 20 (when the last aircraft stops being linked the strike becomes idle, its end tick is this tick, it keeps its strike target)
            if s.aircraft.is_empty() && s.progress {
                s.progress = false;
                s.active = false;
                s.end = tick;
                s.ended = true;
            }
        }
    }
}
