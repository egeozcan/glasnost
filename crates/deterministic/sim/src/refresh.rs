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

/// Whether a vehicle has a pending one-application point suppression (S-0215
/// Terms, *pending suppression*: S-0207 claim 27's suppression not yet
/// consumed). Reading it does not consume it.
pub fn suppression_pending(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .is_some_and(|o| o.kind == Kind::Vehicle && o.econ.point_suppression)
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
    assignment(w, id, new, &mut token);
    token
}

/// An assignment that is treated as new even when unchanged (S-0207 claim
/// 28, *The repeated dock test in enter*): the dock test and claim 28's
/// rule act on the destination the vehicle holds, and the teleporting and
/// driving locomotors take it. Nothing else of an assignment runs (no depot
/// or get-up hook): the order-level destination does not change.
pub fn reassign(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if o.kind != Kind::Vehicle {
        return;
    }
    let new = o.dest;
    let mut token = Token {
        consumes: o.econ.refresh,
        newly: false,
    };
    assignment(w, id, new, &mut token);
    // spec: S-0207 claim 28 (the held destination is treated as a new assignment, even when unchanged: the teleporting locomotor fixes its landing point again under claim 20 a)
    crate::teleport::take_destination_forced(w, id);
    // spec: S-0207 claim 25 (a driving locomotor installs the supplied point at the assignment)
    crate::drive::take_destination(w, id);
    finish(w, id, token);
}

/// A destination set without the assignment hook, by a path computation's
/// substitution (S-0212 claim 24) or a vehicle scatter request (S-0207
/// claim 16): for a vehicle with the teleport-capable flag it is an
/// assignment (S-0402 claim 10a, *A chrono miner*; S-0207 claim 28, *A
/// sale of the refinery*), so the dock test and claim 28's rule act on it
/// and the locomotors take it. `before` is the destination it replaced.
pub fn adopted(w: &mut World, id: ObjectId, before: Option<Target>) {
    if !flagged(w, id) {
        return;
    }
    let token = begin(w, id, before);
    crate::teleport::take_destination(w, id);
    crate::drive::take_destination(w, id);
    finish(w, id, token);
}

/// Claim 27's dock test and claim 28's rule for one assignment of `new`
/// that got past the unchanged-destination rejection.
fn assignment(w: &mut World, id: ObjectId, new: Option<Target>, token: &mut Token) {
    // spec: S-0207 claim 28 (the rule covers vehicles whose type has the teleport-capable flag, whatever their current locomotor; claim 27's dock test needs the flag too)
    if !flagged(w, id) {
        return;
    }
    // spec: S-0207 claims 27 and 28 (no warp-pending immobilization, not gripped and not seized: under any of them neither rule installs or replaces anything and the destination goes to the current locomotor)
    if excluded(w, id) {
        return;
    }
    if docking(w, id, new) {
        // spec: S-0207 claim 28 table row 1 (a qualified docking assignment: claim 27 decides; when the current locomotor is already teleporting claim 27's last predicate fails, nothing changes and teleporting movement takes the destination under claim 20)
        // spec: S-0207 claim 27 (its current locomotor is not teleporting)
        if crate::teleport::is_teleporter(w, id) {
            return;
        }
        if restorable_now(w, id) {
            // spec: S-0207 claim 27 table row 1 (temporary driving with a suspended predecessor, no movement report, no restoration lock and not seized: the predecessor is restored without enabling either pending state; eligibility uses the driving points already present, not the offered destination)
            crate::econ::release::restore(w, id);
            // spec: S-0207 claim 27 (The restoring assignment's destination: the restored predecessor, not driving movement, receives this assignment's otherwise due point, so a pending earlier teleport can be replaced by the offer; the unchanged-destination rejection was this assignment's, made above)
            if let Some(t) = w.obj_mut(id).and_then(|o| o.teleport.as_mut()) {
                t.for_dest = None;
            }
        } else {
            fallback(w, id);
            token.newly = true;
        }
        return;
    }
    // spec: S-0207 claim 28 table row 2 (any other assignment, empty assignments included: temporary driving is installed whenever the current locomotor is not driving, and then receives the destination; behaviour and queue are left as they are)
    install_drive(w, id);
}

/// Claim 28's install of temporary driving over the teleporting locomotor
/// (S-0207 claims 21 and 28), when the current locomotor is teleporting.
fn install_drive(w: &mut World, id: ObjectId) {
    // design: S-0207 claim 28 (driving is installed over a teleporting current locomotor only: our engine runs a hovering or walking own locomotor before temporary driving, so a flagged type with another own locomotor that is not driving keeps it, a seam with no such type known)
    if crate::teleport::is_teleporter(w, id) {
        crate::econ::release::install_temporary_drive(w, id);
    }
}

/// A teleporting jump or a claim 20 removal has just removed the
/// destination of a vehicle (claim 20 a, b, d, e): that removal is an empty
/// assignment, so claim 28's rule installs driving (S-0207 claim 28, *The
/// vehicle's own jump*).
pub fn removed(w: &mut World, id: ObjectId) {
    if w.obj(id).is_none_or(|o| o.kind != Kind::Vehicle) {
        return;
    }
    // spec: S-0207 claim 28 (a jump's removal of the destination, claim 20 b's removal without a jump and claim 20 a's drop are empty assignments: driving is the active movement kind from that tick on)
    if flagged(w, id) && !excluded(w, id) {
        install_drive(w, id);
    }
}

/// A destination set or removed without the assignment hook, reaching a
/// teleporting vehicle with the teleport-capable flag (a fall-back's queued
/// destination, S-0215 claim 15 row 2): claim 28's rule acts on it before
/// the teleporting locomotor would take it. Returns whether driving was
/// installed (the teleporting locomotor then takes nothing).
pub fn untaken(w: &mut World, id: ObjectId, new: Option<Target>) -> bool {
    if w.obj(id).is_none_or(|o| o.kind != Kind::Vehicle) {
        return false;
    }
    if !flagged(w, id) || excluded(w, id) || docking(w, id, new) {
        return false;
    }
    // spec: S-0207 claim 28 table row 2 (an assignment that is not a qualified docking assignment installs driving, which receives the destination)
    install_drive(w, id);
    !crate::teleport::is_teleporter(w, id)
}

/// The end of a destination assignment: the refresh it consumed is gone,
/// unless the dock fallback newly enabled it in this very assignment.
pub fn finish(w: &mut World, id: ObjectId, token: Token) {
    // spec: S-0207 claim 27 (an effective assignment consumes refresh already pending; refresh newly enabled by the dock fallback remains after that very assignment)
    if token.consumes && !token.newly {
        set_refresh(w, id, false);
    }
}

/// Whether a driving vehicle stands stopped by claim 27's fallback: refresh
/// is pending and its locomotor holds no destination point for the
/// order-level destination it has taken, so it begins no track until a
/// later effective assignment supplies a point (S-0207 claims 25, 27 and
/// 28, *An occupied docking cell*).
pub fn stopped(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let Some(d) = o.drive.as_ref() else {
        return false;
    };
    // spec: S-0207 claim 27 (the fallback stops the active locomotion; the next equal offer reaches the test through refresh) and claim 28 (it stops, and at the first later processing of enter at which it stands still the refinery offers the docking cell again)
    o.econ.refresh && o.dest.is_some() && o.dest == d.for_dest && d.goal.is_none()
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

/// Whether the vehicle's type has the teleport-capable flag (S-0207 claims
/// 27 and 28; Import mapping, `Teleporter`), whatever its current locomotor.
pub fn flagged(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    // spec: S-0207 claims 27 and 28 and Import mapping (the vehicle's type has the teleport-capable flag, whatever its current locomotor)
    o.kind == Kind::Vehicle
        && w.rules()
            .unit(o.kind, o.type_index)
            .is_some_and(|u| u.raw.teleporter.unwrap_or(false))
}

/// The three exclusions shared by claim 27's dock test and claim 28's rule:
/// a warp-pending immobilization, being gripped and being seized.
fn excluded(w: &World, id: ObjectId) -> bool {
    // spec: S-0207 claims 27 and 28 (no warp-pending immobilization, read whatever the current locomotor; not gripped, glossary; not seized, and a gripped unit is always seized)
    immobilized(w, id) || crate::lift::lifter_of(w, id).is_some() || crate::lift::seized(w, id)
}

/// The remaining predicates of a qualified docking assignment (S-0207
/// claim 28's table, claim 27's dock test without its last predicate), for
/// the destination `new` just offered.
fn docking(w: &World, id: ObjectId, new: Option<Target>) -> bool {
    let Some(o) = w.obj(id) else { return false };
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
    !occupied
}

/// The warp-pending immobilization of a vehicle (S-0207 Terms and claim
/// 27, S-0502 Terms): the external-teleport commitment of an accepted
/// superweapon warp, from acceptance through the leaving wait until the
/// object leaves its old position. Phasing in (after an own-locomotor jump
/// or a warp's departure, the arrival hold included), the parasite-related
/// held countdown and the restoration lock are other states.
fn immobilized(w: &World, id: ObjectId) -> bool {
    // spec: S-0207 Terms, Warp-pending immobilization (from the accepted warp, before its leaving stage, through the leaving wait until departure; S-0513 claims 34 and 39 own the stages)
    crate::superweapons::warp_pending_or_leaving(w, id)
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
    // spec: S-0207 claims 11 and 28 (a driving vehicle stops: it finishes the track it is on and begins no other)
    crate::drive::stop(w, id);
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
