//! The destination-refresh state, the one-application movement-point
//! suppression and the teleport-capable dock fallback of a vehicle (S-0207
//! claim 27, S-0502 claim 27, S-0402 claim 10 row 2).
//!
//! A vehicle's destination assignment is *effective* when it changes the
//! order-level destination or when the refresh state is pending; the dock
//! test belongs to an effective assignment, evaluated before the
//! locomotors take the destination. Two flags live in the object's economy
//! state ([`crate::econ::ObjectEcon`]): `refresh` and `point_suppression`.

use crate::rules::{Behaviour, Kind};
use crate::world::{Layer, ObjectId, Target, World};

/// What [`begin`] found, handed back to [`finish`].
#[derive(Clone, Copy, Debug, Default)]
pub struct Token {
    /// Refresh was pending at the assignment and is consumed by it.
    consumes: bool,
    /// The dock fallback newly enabled refresh in this assignment.
    newly: bool,
}

/// Whether the refresh state is pending (it makes an unchanged assignment
/// effective).
pub fn pending(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.econ.refresh)
}

/// Enables or clears the refresh state.
fn set_refresh(w: &mut World, id: ObjectId, on: bool) {
    if let Some(o) = w.obj_mut(id) {
        o.econ.refresh = on;
    }
}

/// Successful placement enables refresh for the length of initial
/// idle-behaviour setup (S-0207 claim 27, *Placement*).
pub fn placement_begin(w: &mut World, id: ObjectId) {
    // spec: S-0207 claim 27 (successful placement enables refresh during initial idle-behaviour setup)
    if w.obj(id).is_some_and(|o| o.kind == Kind::Vehicle) {
        set_refresh(w, id, true);
    }
}

/// The initial idle-behaviour setup ends: refresh is cleared (S-0207 claim
/// 27).
pub fn placement_end(w: &mut World, id: ObjectId) {
    // spec: S-0207 claim 27 (it clears when that setup finishes)
    if w.obj(id).is_some_and(|o| o.kind == Kind::Vehicle) {
        set_refresh(w, id, false);
    }
}

/// The start of a destination assignment of a vehicle, after the order-level
/// destination has been stored and before any locomotor takes it. `before`
/// is the destination before the assignment. Runs the dock test of an
/// effective assignment (S-0207 claim 27); an assignment that is neither
/// changed nor refreshed is ignored by this rule (S-0402 claim 10 row 2).
pub fn begin(w: &mut World, id: ObjectId, before: Option<Target>) -> Token {
    let Some(o) = w.obj(id) else {
        return Token::default();
    };
    if o.kind != Kind::Vehicle {
        return Token::default();
    }
    let new = o.dest;
    let had = o.econ.refresh;
    // spec: S-0207 claim 27 (the dock test belongs to an otherwise eligible assignment after unchanged-destination rejection: it needs a changed destination or pending refresh)
    if before == new && !had {
        return Token::default();
    }
    let mut token = Token {
        consumes: had,
        newly: false,
    };
    if qualifies(w, id, new) {
        if restorable_now(w, id) {
            // spec: S-0207 claim 27 table row 1 (temporary driving with a suspended predecessor, no movement report, no restoration lock and not seized: the predecessor is restored without enabling either pending state)
            crate::econ::release::restore(w, id);
        } else {
            fallback(w, id);
            token.newly = true;
        }
    }
    token
}

/// The end of a destination assignment: the refresh it consumed is gone,
/// unless the dock fallback newly enabled it in this very assignment.
pub fn finish(w: &mut World, id: ObjectId, token: Token) {
    // spec: S-0207 claim 27 (an effective assignment consumes refresh already pending; refresh newly enabled by the dock fallback remains after that very assignment)
    if token.consumes && !token.newly {
        set_refresh(w, id, false);
    }
}

/// Whether the next nonempty movement-point application is suppressed: true
/// once, consuming the suppression (S-0207 claim 27, *Separate consumption
/// events*). The caller is a locomotor that is about to apply a nonempty
/// point (driving, S-0207 claim 25, or jumpjet movement, S-0502 claim 16).
pub fn skip_point(w: &mut World, id: ObjectId) -> bool {
    let Some(o) = w.obj_mut(id) else { return false };
    if !o.econ.point_suppression {
        return false;
    }
    // spec: S-0207 claim 27 (the next nonempty assignment that would apply a point consumes the one-application suppression instead of applying it; an empty destination stops movement and does not consume it)
    o.econ.point_suppression = false;
    true
}

/// The predicates of the dock fallback (S-0207 claim 27, *Dock evaluation
/// event*), for the destination `new` just offered.
fn qualifies(w: &World, id: ObjectId, new: Option<Target>) -> bool {
    let Some(o) = w.obj(id) else { return false };
    // spec: S-0207 claim 27 (the vehicle's type has the teleport-capable flag, whatever its current locomotor)
    let flagged = w
        .rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.raw.teleporter.unwrap_or(false));
    if !flagged {
        return false;
    }
    // spec: S-0207 claim 27 (not immobilized by teleport aftereffects, no magnetic-lift source, not seized) and Q-1110 (the aftereffects are a seam that answers no)
    if immobilized(w, id) || crate::jump::seized(w, id) {
        return false;
    }
    // spec: S-0207 claim 27 (its current contact is a building permitting dock unloading)
    if !o
        .contact
        .is_some_and(|b| crate::econ::harvest::permits_dock_unload(w, b))
    {
        return false;
    }
    // spec: S-0207 claim 27 (the offered destination is a cell: a building destination is not the required cell destination)
    let Some(Target::Cell(c)) = new else {
        return false;
    };
    // spec: S-0207 claim 27 (a vehicle of any house on the ground layer disqualifies the cell, the approaching vehicle itself included)
    let occupied = w.cells.list(c, Layer::Ground).iter().any(|&x| {
        w.obj(x)
            .is_some_and(|v| v.kind == Kind::Vehicle && v.on_map)
    });
    if occupied {
        return false;
    }
    // spec: S-0207 claim 27 (its current locomotor is not teleporting)
    !crate::teleport::is_teleporter(w, id)
}

/// The teleport aftereffects that immobilize a vehicle (S-0207 claim 27).
/// The spec does not say which state this names (Q-1110); the seam answers
/// no.
fn immobilized(_w: &World, _id: ObjectId) -> bool {
    false
}

/// Whether restoring the suspended predecessor is possible now (S-0207
/// claim 27 table row 1).
fn restorable_now(w: &World, id: ObjectId) -> bool {
    crate::econ::release::restorable(w, id, true)
}

/// The fallback of a qualified assignment when no restoration is possible
/// (S-0207 claim 27 table rows 2 and 3): the active locomotion stops, the
/// current behaviour is cleared, *enter* is queued without an immediate
/// start, and both pending states are enabled.
fn fallback(w: &mut World, id: ObjectId) {
    // spec: S-0207 claim 27 (stop the active locomotion: a driving locomotor's destination point is absent at once, a jumpjet's movement point is cleared unless it is moving now)
    crate::drive::stop_point(w, id);
    crate::jump::no_point(w, id);
    if let Some(o) = w.obj_mut(id) {
        // spec: S-0207 claim 27 (current behaviour is cleared, enter is queued without an immediate start)
        o.beh.current = None;
        o.beh.queue(Some(Behaviour::Enter));
        // spec: S-0207 claim 27 (both destination refresh and suppression of one nonempty movement-point application become pending)
        o.econ.refresh = true;
        o.econ.point_suppression = true;
    }
}
