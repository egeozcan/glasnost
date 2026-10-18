//! Magnetic lift (S-0607): a shot whose warhead seizes locomotors replaces an
//! eligible vehicle's own movement with jumpjet movement and links it to the
//! firer, which pulls it in; the grip ends by the events of claim 12, a unit
//! high enough is dropped and falls, and the impact decides what happens to
//! it and to what lies underneath. The magnetic wave of claim 21 lives here
//! too.
//!
//! The jumpjet movement itself is S-0502's (`crate::jump`); this module
//! decides when it starts, what destination it has and what happens at the
//! ends of a lift. Every rule that tests the seized state reads [`seized`].
//!
//! A seizing warhead that names another locomotor follows claim 22: one that
//! can stand in temporarily seizes with the unit's own movement state kept
//! (Q-1334), one that cannot (or an unknown identifier) stops the target
//! without a seize. The team-membership removal of claim 5 belongs to the
//! original's team system, which is out of scope.

use glasnost_num::{F32, F64};
use serde::{Deserialize, Serialize};

use crate::combat::damage::{self, Hit};
use crate::combat::data::SeizeLocomotor;
use crate::rules::{Behaviour, Kind};
use crate::world::{HouseId, HouseRole, Layer, ObjectId, Pos, Target, World};

/// The magnetic wave of a magnetic-beam weapon's shot (S-0607 claim 21,
/// F4): its firer, its target, the two fade accumulators and whether it has
/// ended. A wave is an entry of the update order of its own
/// (`World::waves`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wave {
    /// The firer whose shot made it.
    pub firer: ObjectId,
    /// The shot's target (claim 21: the wave ends when its firer's target
    /// is no longer this one).
    pub target: Target,
    /// The fade-in accumulator *a* (F4), a double.
    pub a: F64,
    /// The fade-out accumulator *b* (F4), a double.
    pub b: F64,
    /// The wave has ended and fades out.
    pub ended: bool,
}

/// An object's magnetic-lift state (S-0607 Terms).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiftState {
    /// The unit is *seized* (Terms): from the seize (claim 5) until
    /// restoration (claim 20).
    pub seized: bool,
    /// A gripped unit's lifter.
    pub lifter: Option<ObjectId>,
    /// A lifter's gripped unit.
    pub gripped: Option<ObjectId>,
    /// A lifter's destination when it made the grip: accepting another,
    /// non-empty destination ends the grip (claim 12 row 2).
    pub grip_dest: Option<Target>,
    /// The unit is *dropped* (Terms, claim 13).
    pub dropped: bool,
    /// A dropped unit's dropper (claims 13 and 14).
    pub dropper: Option<ObjectId>,
    /// A dropped unit's dropper house (claims 13 and 14).
    pub dropper_house: Option<HouseId>,
    /// The unit is *set down* (Terms).
    pub set_down: bool,
    /// A dropped unit flies to the centre of its cell before its fall
    /// (claim 15).
    pub detour: bool,
    /// A falling dropped unit's crash rate, a single (S-0502 claim 18).
    pub rate: Option<F32>,
    /// The unit was seized with a locomotor other than jumpjet that can
    /// stand in temporarily (S-0607 claim 22); it keeps its own movement
    /// state (Q-1334).
    #[serde(default)]
    pub stand_in: bool,
    /// A unit set sinking by claim 17: its coming death is credited.
    pub sink_credit: bool,
}

// ---------------------------------------------------------------------------
// Queries

fn st(w: &World, id: ObjectId) -> Option<&LiftState> {
    w.obj(id).map(|o| &o.lift)
}

fn upd(w: &mut World, id: ObjectId, f: impl FnOnce(&mut LiftState)) {
    if let Some(o) = w.obj_mut(id) {
        f(&mut o.lift);
    }
}

/// Whether the unit is *seized* (S-0607 Terms): held by magnetic lift, from
/// the seize until its own locomotor returns.
pub fn seized(w: &World, id: ObjectId) -> bool {
    st(w, id).is_some_and(|l| l.seized)
}

/// The unit a lifter grips (S-0607 Terms).
pub fn gripped_unit(w: &World, lifter: ObjectId) -> Option<ObjectId> {
    st(w, lifter)
        .and_then(|l| l.gripped)
        .filter(|&x| w.obj(x).is_some())
}

/// A gripped unit's lifter (S-0607 Terms).
pub fn lifter_of(w: &World, unit: ObjectId) -> Option<ObjectId> {
    st(w, unit).and_then(|l| l.lifter)
}

/// Whether `lifter` grips `unit` (S-0301 Terms, *holding a target* with a
/// magnetic-beam weapon).
pub fn grips(w: &World, lifter: ObjectId, unit: ObjectId) -> bool {
    gripped_unit(w, lifter) == Some(unit)
}

/// Whether a seized unit was seized with a stand-in locomotor (S-0607
/// claim 22).
pub fn stand_in(w: &World, id: ObjectId) -> bool {
    st(w, id).is_some_and(|l| l.seized && l.stand_in)
}

/// Whether the unit is *dropped* (S-0607 Terms).
pub fn dropped(w: &World, id: ObjectId) -> bool {
    st(w, id).is_some_and(|l| l.dropped)
}

/// A falling dropped unit's crash rate (S-0502 claim 18).
pub fn rate(w: &World, id: ObjectId) -> Option<F32> {
    st(w, id).and_then(|l| l.rate)
}

/// Whether the firer's magnetic wave is alive (S-0607 claim 21).
pub fn wave_alive(w: &World, id: ObjectId) -> bool {
    wave_of(w, id).is_some()
}

/// The live magnetic wave of a firer, by its number in the update order
/// (S-0607 claim 21).
pub fn wave_of(w: &World, firer: ObjectId) -> Option<ObjectId> {
    w.waves
        .iter()
        .find(|(_, v)| v.firer == firer)
        .map(|(&id, _)| id)
}

// ---------------------------------------------------------------------------
// Seizing (claims 1 to 6)

/// A seizing projectile detonates (S-0607 claims 1 to 6): the seize takes
/// the place of ordinary damage. `damage` is the projectile damage *D*.
pub fn seize_hit(
    w: &mut World,
    firer: Option<ObjectId>,
    target: Option<Target>,
    damage: i32,
    warhead: crate::combat::data::WarheadId,
) {
    // spec: S-0607 claim 2 (the hit does nothing at all when the firer has left play)
    let Some(f) = firer.filter(|&f| w.obj(f).is_some_and(|o| o.on_map)) else {
        return;
    };
    // spec: S-0607 claim 2 (the hit does nothing at all when the firer is itself seized)
    if seized(w, f) {
        return;
    }
    // spec: S-0606 claim 9 and S-0309 claim 24 (a locomotor-warhead attack on a bunkered vehicle is redirected to the bunker)
    let x = match target {
        Some(Target::Object(x)) => Some(crate::structures::bunker::bunkered_in(w, x).unwrap_or(x)),
        _ => None,
    };
    // spec: S-0607 claim 2 (the hit does nothing at all when the firer already grips this very target)
    if let Some(x) = x
        && grips(w, f, x)
    {
        return;
    }
    // spec: S-0607 claim 2 (otherwise, before the target is tested, the firer's current grip ends as claim 13 says, the firer keeping its target, even when the target then turns out immune) and claim 12 (a new seizing detonation: the lifter's target is not cleared)
    end_grip(w, f, false);
    let Some(x) = x else { return };
    // spec: S-0607 claim 22 (claims 1 to 3 decide the seize the same way whatever locomotor the warhead names)
    if !eligible(w, x, damage) {
        return;
    }
    // spec: S-0607 State and ordering (a resource gatherer made immune by claim 3 keeps its temporary locomotor: claim 3's conditions come before claim 4's special case); Q-1336 interim (claim 4's special case comes first whatever locomotor the warhead names)
    if harvester_case(w, x) {
        return;
    }
    match w.rules().combat.warhead(warhead).seize_locomotor {
        SeizeLocomotor::Jumpjet => seize(w, f, x, false),
        // spec: S-0607 claim 22 (a locomotor that can stand in temporarily: the target is seized as in claim 5, with that locomotor in place of its own)
        SeizeLocomotor::StandIn => seize(w, f, x, true),
        // spec: S-0607 claim 22 (a locomotor that cannot stand in temporarily, or an unknown identifier: claim 5's other effects, no seize)
        SeizeLocomotor::NoStandIn => not_seized(w, x),
    }
}

/// A hit whose warhead names a locomotor that cannot stand in temporarily
/// (S-0607 claim 22): the eligible target stops, is put in *guard* and
/// loses its spawned objects and its parasite, but is not seized: no
/// locomotor is installed, it keeps moving with its own and can be hit
/// again.
fn not_seized(w: &mut World, x: ObjectId) {
    let t = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    // spec: S-0607 claims 5 and 22 (it stops: its route is cleared and its destination becomes none, its own locomotor stopping by its own rules)
    if let Some(o) = w.obj_mut(x) {
        o.path.steps.clear();
        o.queued_dests.clear();
    }
    crate::econ::assign_destination(w, x, None);
    crate::drive::stop(w, x);
    if let Some(o) = w.obj_mut(x) {
        // spec: S-0607 claims 5 and 22 (it is put in guard with no target)
        crate::spawn::clear_own_target(o);
        o.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
        o.beh.force(Behaviour::Guard, t, stale);
    }
    // spec: S-0607 claims 5 and 22 (it loses its spawned objects and its parasite)
    cleanup(w, x);
}

/// Whether a target can be seized by a hit of `damage` (S-0607 claim 3
/// and F1).
pub fn eligible(w: &World, x: ObjectId, damage: i32) -> bool {
    let Some(o) = w.obj(x) else { return false };
    // spec: S-0607 claim 3 (a vehicle or an aircraft; infantry and buildings are never seized)
    if !matches!(o.kind, Kind::Vehicle | Kind::Aircraft) || !o.on_map {
        return false;
    }
    // spec: S-0607 claim 3 (in the ground render layer, S-0102: an aircraft in flight and a jumpjet in the air are immune, a landed aircraft is not)
    if !crate::temporal::ground_layer(w, x) {
        return false;
    }
    // spec: S-0607 claim 3 and S-0513 claim 27 (not temporarily invulnerable)
    if crate::superweapons::protected(w, x) {
        return false;
    }
    // spec: S-0607 claim 3 (not already seized)
    if seized(w, x) {
        return false;
    }
    // spec: S-0607 claim 3 (not protected by its factory: in contact with a building of a war-factory type that covers the vehicle's cell)
    if o.kind == Kind::Vehicle
        && let Some(k) = o.contact
        && crate::econ::release::vehicle_factory(w, k)
        && w.obj(k).is_some_and(|b| {
            crate::objects::foundation_cells(b.pos.cell(), b.foundation).contains(&o.pos.cell())
        })
    {
        return false;
    }
    let mode = w.rules().mode;
    let Some(u) = w.rules().unit(o.kind, o.type_index) else {
        return false;
    };
    // spec: S-0607 claim 3 and F1 (s < D, exact and strict: the size held in double precision, the damage widened exactly; an equal size is immune)
    u.size.lt(F64::from_i64(i64::from(damage), mode))
}

fn resource_gatherer(w: &World, x: ObjectId) -> bool {
    w.obj(x)
        .filter(|o| o.kind == Kind::Vehicle)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        .is_some_and(|u| u.raw.econ.resource_gatherer.unwrap_or(false))
}

/// The cleanup of claim 5 that also applies to claim 4: spawned objects lost
/// (S-0506 claim 13) and a naval parasite released (S-0614 claim 12).
fn cleanup(w: &mut World, x: ObjectId) {
    // spec: S-0607 claims 4 and 5 and S-0506 claim 13 (the seized unit loses its spawned objects)
    crate::spawn::clear_slots(w, x);
    // spec: S-0607 claims 4 and 5 and S-0614 claim 12 (a replaced locomotor: a naval parasite is released)
    crate::parasite::locomotor_replaced(w, x);
}

/// A resource gatherer whose own locomotor is already temporarily replaced
/// and which is not gripped (S-0607 claim 4). Returns whether it applied;
/// the gatherer is then not seized.
fn harvester_case(w: &mut World, x: ObjectId) -> bool {
    // spec: S-0607 claim 4 (a resource-gathering vehicle whose own locomotor is already temporarily replaced, for example a teleporting harvester while it drives, and which is not gripped)
    if !resource_gatherer(w, x)
        || !crate::econ::release::temporary_drive(w, x)
        || lifter_of(w, x).is_some()
    {
        return false;
    }
    if crate::econ::release::restorable(w, x, false) {
        // spec: S-0607 claim 4 (a replacement that may end at this moment ends: it gets its own locomotor back and is not seized)
        crate::econ::release::restore(w, x);
    } else {
        // spec: S-0607 claim 4 (otherwise it stops, its current behaviour is cleared and it queues stop)
        crate::drive::stop(w, x);
        if let Some(o) = w.obj_mut(x) {
            o.beh.current = None;
            o.beh.queue(Some(Behaviour::Stop));
        }
    }
    cleanup(w, x);
    true
}

/// Whether the house follows the original's non-human rule of claims 6
/// and 20: the neutral and special houses (our AI houses take the human
/// rule).
fn non_human_rule(w: &World, h: Option<HouseId>) -> bool {
    h.and_then(|h| w.houses.get(usize::from(h.0)))
        .is_some_and(|x| !x.human && x.role != HouseRole::Player)
}

/// The seize itself (S-0607 claim 5).
fn seize(w: &mut World, lifter: ObjectId, x: ObjectId, stand_in: bool) {
    let t = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    if stand_in {
        // spec: S-0607 claim 22 (how a unit seized with a stand-in locomotor moves is outside the spec); Q-1334 interim (it keeps its own movement state and marks, stops by its own rules and stays where it stands)
        crate::drive::stop(w, x);
    } else {
        // spec: S-0607 claim 5 (it stops: its route is cleared, it gives up the occupancy mark of its cell and it stands still)
        suspend_own(w, x);
    }
    if let Some(o) = w.obj_mut(x) {
        o.path.steps.clear();
        o.queued_dests.clear();
        // spec: S-0607 claim 5 and S-0215 claim 9 (it starts guard with no target and no destination)
        crate::spawn::clear_own_target(o);
        o.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
        o.dest = None;
        o.beh.force(Behaviour::Guard, t, stale);
    }
    cleanup(w, x);
    if w.obj(x).is_none() {
        return;
    }
    if !stand_in {
        // spec: S-0607 claim 5 and claim 7 (its jumpjet locomotor is linked, with the jumpjet fields of the unit's own type)
        crate::jump::link_lift(w, x);
    }
    // spec: S-0607 claim 5 (the firer's previous grip ends without clearing the firer's target)
    end_grip(w, lifter, false);
    // spec: S-0607 claim 5 (a grip the unit itself held ends, clearing the unit's own target)
    end_grip(w, x, true);
    let grip_dest = w.obj(lifter).and_then(|o| o.dest);
    upd(w, lifter, |l| {
        l.gripped = Some(x);
        l.grip_dest = grip_dest;
    });
    upd(w, x, |l| {
        l.seized = true;
        l.lifter = Some(lifter);
        l.dropped = false;
        l.set_down = false;
        l.detour = false;
        l.dropper = None;
        l.dropper_house = None;
        l.rate = None;
        l.stand_in = stand_in;
    });
    let harvester = resource_gatherer(w, x);
    let hunt = non_human_rule(w, w.obj(x).and_then(|o| o.owner))
        && !w
            .obj(x)
            .and_then(|o| o.owner)
            .is_some_and(|h| crate::econ::setup::passive(w, h))
        && !harvester;
    if !stand_in {
        // spec: S-0607 claims 5 and 7 (the grip is made and the unit is given the lifter as its destination; its jumpjet movement turns this at once into a movement point fixed at the seize, from the lifter's point)
        let p = lifter_point(w, lifter, x);
        let cell = crate::jump::lift_point(w, x, p);
        if let Some(o) = w.obj_mut(x) {
            // spec: S-0607 claim 5 (its order-level destination becomes the cell of that point, not the lifter)
            o.dest = Some(Target::Cell(cell));
        }
    }
    // spec: S-0607 claim 5 (the unit is deselected)
    w.effects
        .push(crate::combat::effects::Effect::Seized { unit: x });
    // spec: S-0607 claim 5 (it breaks off a contact with a building)
    break_contact(w, x);
    if let Some(o) = w.obj_mut(x) {
        // spec: S-0607 claims 5 and 6 (it queues guard; a seized unit of a non-passive computer-controlled house queues hunt instead, a resource gatherer excepted; our AI houses take the human rule, the neutral and special houses the original's)
        o.beh.queue(Some(if hunt {
            Behaviour::Hunt
        } else {
            Behaviour::Guard
        }));
    }
}

/// The lifter's point at the seize (S-0607 claim 7): a mobile lifter's
/// heading point (Terms: its next movement point, or its position when it
/// has none); a building's foundation centre, or its docking point when its
/// type services docking units.
fn lifter_point(w: &World, lifter: ObjectId, x: ObjectId) -> Pos {
    let Some(o) = w.obj(lifter) else {
        return Pos::default();
    };
    if o.kind == Kind::Building {
        if crate::depot::is_depot(w, lifter) || crate::structures::bunker::is_bunker(w, lifter) {
            // spec: S-0607 claim 7 (a building of a type that services docking units gives its docking point); Q-1333 interim (the repair depot and the bunker are those types, and the point is the one they would give the seized unit)
            return crate::depot::docking_point(w, lifter, x);
        }
        // spec: S-0607 claim 7 (a building: the centre of its foundation)
        return w.reference_point(lifter);
    }
    // spec: S-0607 Terms and claim 7 (a mobile lifter's heading point: the point its own movement is heading for, its next movement point; its position when it has none)
    let next = o
        .drive
        .as_ref()
        .and_then(|d| d.track.as_ref())
        .map(|tr| {
            Pos::new(
                tr.target.0,
                tr.target.1,
                w.surface_at(tr.target.0, tr.target.1),
            )
        })
        .or_else(|| o.walk.as_ref().and_then(|s| s.target))
        .or_else(|| o.hover.as_ref().and_then(|h| h.point))
        .or_else(|| o.jump.as_ref().and_then(|j| j.dest));
    next.unwrap_or(o.pos)
}

/// The seized unit's own locomotor stops and gives up its occupancy (S-0607
/// claim 5); its state is kept suspended for restoration (S-0215 claim 13:
/// restoration retains the suspended movement state).
fn suspend_own(w: &mut World, x: ObjectId) {
    let Some(o) = w.obj(x) else { return };
    if o.jump.is_some() {
        // A jumpjet type keeps its own jumpjet locomotor; `jump::link_lift` handles its marks.
        return;
    }
    match o.kind {
        Kind::Vehicle => {
            // spec: S-0607 claim 5 and S-0209 claim 1 (it gives up the occupancy mark of its cell)
            crate::drive::release_claims(w, x);
            if let Some(ob) = w.obj_mut(x) {
                if ob.drive.is_some() {
                    // spec: S-0607 claim 5 (it stops and stands still: the driving step and its track end)
                    ob.drive = Some(crate::drive::DriveState::new());
                }
                if let Some(h) = ob.hover.as_mut() {
                    h.point = None;
                    h.between = false;
                    h.v = F64::ZERO;
                }
            }
        }
        Kind::Aircraft => {
            // spec: S-0607 claim 5 and S-0209 claim 1 (a landed aircraft gives up its object-present mark)
            crate::air::clear_mark(w, x);
        }
        _ => {}
    }
}

/// The seized unit breaks off a contact with a building (S-0607 claim 5).
fn break_contact(w: &mut World, x: ObjectId) {
    let Some(k) = w.obj(x).and_then(|o| o.contact) else {
        return;
    };
    if w.obj(k).is_none_or(|b| b.kind != Kind::Building) {
        return;
    }
    if crate::depot::link(w, x).is_some() {
        crate::depot::release(w, x);
    }
    if w.obj(x).is_some_and(|o| o.econ.harvest.is_some()) {
        crate::econ::harvest::break_contact(w, x);
    }
    if let Some(o) = w.obj_mut(x) {
        o.contact = None;
        if o.econ.exit.is_some_and(|e| e.factory == k) {
            o.econ.exit = None;
        }
    }
    if let Some(b) = w.obj_mut(k)
        && b.contact == Some(x)
    {
        b.contact = None;
    }
}

/// The dropped unit breaks off any contact (S-0607 claim 13): a contact
/// with a building as claim 5 breaks it, any other on both sides.
fn break_any_contact(w: &mut World, x: ObjectId) {
    let Some(k) = w.obj(x).and_then(|o| o.contact) else {
        return;
    };
    if w.obj(k).is_some_and(|b| b.kind == Kind::Building) {
        break_contact(w, x);
        return;
    }
    if let Some(o) = w.obj_mut(x) {
        o.contact = None;
    }
    if let Some(b) = w.obj_mut(k)
        && b.contact == Some(x)
    {
        b.contact = None;
    }
}

// ---------------------------------------------------------------------------
// Ending the grip (claims 12 to 14)

/// Ends the grip `lifter` holds, if any (S-0607 claims 12 and 13): the
/// gripped unit is dropped at 1 lepton or more above the surface, set down
/// otherwise. `clear_target`: the lifter's target is cleared (claim 12's
/// table).
pub fn end_grip(w: &mut World, lifter: ObjectId, clear_target: bool) {
    let Some(x) = st(w, lifter).and_then(|l| l.gripped) else {
        return;
    };
    upd(w, lifter, |l| {
        l.gripped = None;
        l.grip_dest = None;
    });
    if clear_target && let Some(o) = w.obj_mut(lifter) {
        // spec: S-0607 claim 12 (the lifter's target is cleared)
        crate::spawn::clear_own_target(o);
        o.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
    if w.obj(x).is_none() || lifter_of(w, x) != Some(lifter) {
        return;
    }
    upd(w, x, |l| l.lifter = None);
    let house = w.obj(lifter).and_then(|o| o.owner);
    // spec: S-0607 claim 13 (height above the surface 1 lepton or more: dropped; 0 or less: set down)
    if crate::jump::height_above(w, x) >= 1 {
        drop(w, x, Some(lifter), house);
    } else {
        set_down_here(w, x);
    }
}

/// A grip ends while the unit is high enough: it is dropped (S-0607 claim
/// 13).
fn drop(w: &mut World, x: ObjectId, dropper: Option<ObjectId>, house: Option<HouseId>) {
    // spec: S-0607 claim 13 (marked as crashing, with the lifter as its dropper and the lifter's house as its dropper house; not a crash start of S-0312 claim 16: passengers stay in, no tumbling draws)
    upd(w, x, |l| {
        l.dropped = true;
        l.dropper = dropper;
        l.dropper_house = house;
        l.set_down = false;
    });
    if let Some(o) = w.obj_mut(x) {
        // spec: S-0607 claim 13 (the stop on dropping: its destination becomes none, its route is cleared and its target is cleared)
        o.dest = None;
        o.path.steps.clear();
        crate::spawn::clear_own_target(o);
        o.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
    // spec: S-0607 claim 13 (the stop on dropping: it breaks off any contact)
    break_any_contact(w, x);
    // spec: S-0607 claim 13 (the stop on dropping: it is deselected)
    w.effects
        .push(crate::combat::effects::Effect::Dropped { unit: x });
    // spec: S-0607 claim 15 (to its cell centre first when it is not exactly over it)
    let detour = crate::jump::lift_dropped(w, x);
    upd(w, x, |l| l.detour = detour);
    if !detour {
        start_fall(w, x);
    }
}

/// The fall itself begins (S-0607 claim 15, S-0502 claim 18).
fn start_fall(w: &mut World, x: ObjectId) {
    let mode = w.rules().mode;
    let k = crate::jump::fields(w, x).map_or(F64::ZERO, |f| f.crash);
    // spec: S-0607 claim 15 (the crash rate starts at the unit's own type's jumpjet crash value, narrowed to single precision, afresh at every seize)
    upd(w, x, |l| {
        l.detour = false;
        l.rate = Some(k.to_f32(mode));
    });
    crate::jump::lift_fall(w, x);
}

/// The unit is set down where it is (S-0607 claims 9 and 13): its
/// destination becomes none; above the surface it descends and lands.
fn set_down_here(w: &mut World, x: ObjectId) {
    upd(w, x, |l| {
        l.set_down = true;
        l.dropped = false;
        l.detour = false;
        l.rate = None;
    });
    if let Some(o) = w.obj_mut(x) {
        // spec: S-0607 claims 9 and 13 (set down: its destination becomes none)
        o.dest = None;
    }
    if crate::jump::height_above(w, x) > 0 {
        // spec: S-0607 claim 9 (set down at whatever height, it then descends and lands under claim 17's clear-cell rule)
        crate::jump::lift_descend(w, x);
    } else {
        crate::jump::lift_settle(w, x);
    }
}

/// The lifter accepts a destination (S-0607 claim 12 row 2): a mobile
/// lifter's grip ends, clearing its target, when the destination is not
/// empty and not the one it had when it gripped.
pub fn lifter_destination(w: &mut World, lifter: ObjectId) {
    let Some(o) = w.obj(lifter) else { return };
    if !o.kind.is_mobile() {
        return;
    }
    let Some(l) = st(w, lifter) else { return };
    if l.gripped.is_none() {
        return;
    }
    // spec: S-0607 claim 12 (row 2: the lifter is a mobile unit and accepts a non-empty destination, from an order, an approach step or another rule: the grip ends and the lifter's target is cleared)
    if o.dest.is_some() && o.dest != l.grip_dest {
        end_grip(w, lifter, true);
    }
}

/// A mobile lifter accepts a non-empty destination through the assignment
/// (S-0607 claim 12 row 2): its grip ends and its target is cleared.
pub fn lifter_accepts(w: &mut World, lifter: ObjectId) {
    if !w.obj(lifter).is_some_and(|o| o.kind.is_mobile()) {
        return;
    }
    // spec: S-0607 claim 12 (row 2: the lifter is a mobile unit and accepts a non-empty destination: the grip ends and the lifter's target is cleared)
    end_grip(w, lifter, true);
}

/// A vehicle falls back to rest (S-0607 claim 12 row 3): a vehicle lifter's
/// grip ends, clearing its target, unless an override is in effect or its
/// current behaviour is *wait*.
pub fn lifter_rest(w: &mut World, lifter: ObjectId) {
    let Some(o) = w.obj(lifter) else { return };
    if o.kind != Kind::Vehicle || st(w, lifter).is_none_or(|l| l.gripped.is_none()) {
        return;
    }
    // spec: S-0607 claim 12 (row 3: unless an override is in effect, S-0215 Terms: a behaviour is suspended, or its current behaviour is wait)
    if o.beh.suspended.is_some() || o.beh.current == Some(Behaviour::Wait) {
        return;
    }
    end_grip(w, lifter, true);
}

/// A building's own turn with its in-operation result (S-0607 claim 12 row
/// 4, S-0405 claim 9): a building lifter not in operation lets go.
pub fn building_turn(w: &mut World, id: ObjectId, in_operation: bool) {
    if !in_operation {
        // spec: S-0607 claim 12 (row 4: a building lifter that stops being in operation: the grip ends and its target is cleared) and S-0405 claim 9 (reconciled in its own turn)
        end_grip(w, id, true);
    }
}

/// A building is destroyed (S-0607 claim 12 row 5).
pub fn building_destroyed(w: &mut World, id: ObjectId) {
    // spec: S-0607 claim 12 (row 5: a building lifter that is destroyed: the grip ends and its target is cleared)
    end_grip(w, id, true);
}

/// An object leaves play (S-0607 claims 12 and 14; `objects::remove`).
pub fn removing(w: &mut World, id: ObjectId) {
    // spec: S-0607 claim 12 (the lifter leaves play: its grip ends; claim 14: a mobile lifter leaving play drops the unit just before)
    end_grip(w, id, false);
    // spec: S-0607 claim 12 (the gripped unit leaves play: the grip ends and the lifter's target is cleared)
    if let Some(l) = lifter_of(w, id) {
        upd(w, id, |s| s.lifter = None);
        if st(w, l).is_some_and(|s| s.gripped == Some(id)) {
            upd(w, l, |s| {
                s.gripped = None;
                s.grip_dest = None;
            });
            if let Some(o) = w.obj_mut(l) {
                crate::spawn::clear_own_target(o);
                o.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
            }
        }
    }
    // spec: S-0607 claim 14 (a dropper that leaves play: its dropped units no longer have a dropper but keep their dropper house)
    let dropped: Vec<ObjectId> = w
        .objects
        .iter()
        .filter(|(_, o)| o.lift.dropper == Some(id))
        .map(|(&k, _)| k)
        .collect();
    for k in dropped {
        upd(w, k, |l| l.dropper = None);
    }
}

// ---------------------------------------------------------------------------
// The lift (claims 7 to 9)

/// The steering of a seized unit before its jumpjet update (S-0607 claims
/// 7 and 9): a gripped unit keeps the movement point fixed at the seize
/// and the cell of that point as its destination; any other destination is
/// ignored; no destination ends the grip without a drop. A seized unit
/// that is not gripped keeps no destination.
pub fn steer(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let dest = o.dest;
    match lifter_of(w, id) {
        Some(l) if w.obj(l).is_some() => {
            match dest {
                None => {
                    // spec: S-0607 claim 9 (a rule that gives a gripped unit no destination ends its grip without a drop: set down where it is) and claim 12 (the lifter's target is not cleared)
                    upd(w, l, |s| {
                        s.gripped = None;
                        s.grip_dest = None;
                    });
                    upd(w, id, |s| s.lifter = None);
                    set_down_here(w, id);
                    return;
                }
                Some(t) => {
                    if let Some(fixed) = crate::jump::lift_order(w, id)
                        && t != fixed
                        && let Some(ob) = w.obj_mut(id)
                    {
                        // spec: S-0607 claim 9 (a seized unit ignores every new destination)
                        ob.dest = Some(fixed);
                    }
                }
            }
            // spec: S-0607 claim 7 (the movement point is fixed at the seize and not refreshed while the grip lasts: when the lifter moves, the gripped unit keeps flying to it)
        }
        _ => {
            if dest.is_some() {
                // spec: S-0607 claim 9 (a seized unit ignores every new destination)
                if let Some(ob) = w.obj_mut(id) {
                    ob.dest = None;
                }
            }
            crate::jump::forget_order(w, id);
        }
    }
}

/// Enforces claim 9 for a unit seized with a stand-in locomotor (S-0607
/// claim 22, Q-1334 interim): a destination written without the assignment
/// is replaced by none at the start of its behaviour turn.
pub fn enforce(w: &mut World, id: ObjectId) {
    if stand_in(w, id)
        && let Some(o) = w.obj_mut(id)
        && o.dest.is_some()
    {
        // spec: S-0607 claim 9 (a seized unit ignores every new destination)
        o.dest = None;
        o.path.steps.clear();
    }
}

/// Whether a seized unit refuses a destination given to it (S-0607 claim
/// 9, `econ::assign_destination`): a non-empty one is ignored; an empty
/// one is let through, so a gripped unit is set down by [`steer`].
pub fn refuses_destination(w: &World, id: ObjectId, dest: Option<Target>) -> bool {
    // spec: S-0607 claim 9 (a seized unit ignores every new destination; no destination is the exception)
    seized(w, id) && dest.is_some()
}

/// A gripped unit or a dropped unit's detour arrives (S-0607 claims 8 and
/// 15; S-0502 claim 12): it is on its destination with target height 0 and
/// *descending*.
pub fn arrived(w: &mut World, id: ObjectId) {
    if let Some(l) = lifter_of(w, id)
        && w.obj(l).is_some()
    {
        // spec: S-0607 claim 8 (arrival: the lifter's grip ends as claim 13 says, in the same update) and claim 12 (row 1: the lifter's target is cleared)
        end_grip(w, l, true);
        return;
    }
    if dropped(w, id) && st(w, id).is_some_and(|l| l.detour) {
        // spec: S-0607 claim 15 (its crash begins once it is over the centre, which the arrival snap puts it on)
        start_fall(w, id);
    }
}

// ---------------------------------------------------------------------------
// The fall and the impact (claims 15 to 19)

/// The credit of claim 16's erasures: the dropper, else the dropper house,
/// else the dropped unit.
fn erase_hit(w: &World, id: ObjectId) -> Hit {
    let l = st(w, id).cloned().unwrap_or_default();
    let dropper = l.dropper.filter(|&d| w.obj(d).is_some());
    // spec: S-0607 claim 16 (each removal is credited as a kill to the dropper, else to the dropper house, else to the dropped unit)
    let (attacker, house) = match (dropper, l.dropper_house) {
        (Some(d), _) => (Some(d), None),
        (None, Some(h)) => (None, Some(h)),
        (None, None) => (Some(id), None),
    };
    Hit {
        amount: 0,
        warhead: None,
        distance: 0,
        attacker,
        house,
        ignore_defences: true,
        no_escape: false,
    }
}

/// A dropped unit's fall has ended (S-0607 claims 15 to 19): what lies
/// underneath, then water, beach, clear and blocked.
pub fn fall_ended(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else { return };
    upd(w, id, |l| l.rate = None);
    if damage::dead(&o) {
        // A dropped unit already destroyed in its fall (a crashable type that started to crash, S-0312 claim 16) ends it as an ordinary crash.
        crate::jump::lift_crashed(w, id);
        crate::jump::impact(w, id);
        return;
    }
    // spec: S-0607 claim 15 (the fall ends on a bridge deck when the unit stands on the deck or its fall crossed the deck from at or above to below)
    let on_deck = o.layer == Layer::Deck || crate::jump::crossed_deck(w, id);
    if on_deck && o.layer != Layer::Deck {
        crate::jump::lift_onto_deck(w, id);
    }
    let o = w.obj(id).cloned().unwrap_or(o);
    let blocked_by = underneath(w, id, on_deck);
    if w.obj(id).is_none() {
        return;
    }
    let c = o.pos.cell();
    let land = w.cells.cell(c).map_or(0, |x| x.land);
    use glasnost_map::terrain::land as lt;
    // spec: S-0607 claim 17 (water and beach count only when the unit did not land on a bridge deck)
    let water = !on_deck && land == lt::WATER;
    let beach = !on_deck && land == lt::BEACH;
    // spec: S-0607 claim 16 (a vehicle or building underneath makes the cell blocked and claim 17's blocked row on land applies, even on water)
    let blocked = blocked_by || blocked(w, id, on_deck);
    if blocked && water && !blocked_by {
        // spec: S-0607 claim 17 (blocked, water and not beach: set sinking, S-0504 claims 4 and 5, and stunned; its slaves freed, credited to the dropper; its own coming death credited to the dropper, else the dropper house; no falling damage)
        sink(w, id);
        return;
    }
    if blocked {
        // spec: S-0607 claim 17 (blocked, on land or beach: it ends its fall in crashed, releases its hover slot, and claim 18 applies)
        crate::jump::lift_crashed(w, id);
        impact(w, id);
        return;
    }
    let _ = beach;
    // spec: S-0607 claim 17 (not blocked: it releases its hover slot and enters descending with no destination; at touchdown it is set down and no longer dropped or crashing)
    crate::jump::lift_descend(w, id);
}

/// Claim 16: the first other object listed in the cell's ground layer
/// decides. Returns whether a vehicle or building made the cell blocked.
fn underneath(w: &mut World, id: ObjectId, on_deck: bool) -> bool {
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    let c = o.pos.cell();
    // spec: S-0607 claim 16 and quirks.lift_deck_impact_reads_ground (the ground layer is read even when the fall ended on a bridge deck; off: the layer the unit landed in)
    let layer = if on_deck && !w.rules().quirks.lift_deck_impact_reads_ground {
        Layer::Deck
    } else {
        Layer::Ground
    };
    let list: Vec<ObjectId> = w.cells.list(c, layer).to_vec();
    // spec: S-0607 claim 16 (the first object listed, in S-0209 claim 13's listing order, other than the unit itself)
    let Some(pos) = list.iter().position(|&x| x != id && w.obj(x).is_some()) else {
        return false;
    };
    let first = list[pos];
    let Some(f) = w.obj(first).cloned() else {
        return false;
    };
    match f.kind {
        // spec: S-0607 claim 16 (a vehicle or a building: no damage from this rule; the cell is blocked)
        Kind::Vehicle | Kind::Building => true,
        Kind::Infantry => {
            let tail = w.rules().quirks.lift_drop_erases_cell_tail;
            // spec: S-0607 claim 16 and quirks.lift_drop_erases_cell_tail (that infantry unit and every object after it in the cell's list are removed, whatever its kind, the dropped unit included when listed after; off: only the infantry units, never the dropped unit)
            let victims: Vec<ObjectId> = list[pos..]
                .iter()
                .copied()
                .filter(|&x| {
                    tail || (x != id && w.obj(x).is_some_and(|v| v.kind == Kind::Infantry))
                })
                .collect();
            let hit = erase_hit(w, id);
            let at = o.pos;
            for v in victims {
                let Some(vo) = w.obj(v).cloned() else {
                    continue;
                };
                // spec: S-0607 claim 16 (removed at once, without death effects or death animations, each credited as a kill)
                crate::combat::death::passenger_death(w, v, &hit);
                // spec: S-0607 claim 16 (the crush sound of each one's type plays at the dropped unit's position)
                w.effects.push(crate::combat::effects::Effect::Sound {
                    sound: crate::combat::effects::Sound::Crush,
                    of: Some(crate::econ::TypeRef {
                        kind: vo.kind,
                        index: vo.type_index,
                    }),
                    house: None,
                    pos: Some(at),
                });
            }
            false
        }
        _ => {
            let l = st(w, id).cloned().unwrap_or_default();
            let dropper = l.dropper.filter(|&d| w.obj(d).is_some());
            // spec: S-0607 claim 16 (anything else: damage equal to its current health with the crushing warhead, ignoring defences; the attacker is the dropper; with no dropper, the dropped unit with the dropper house)
            let (attacker, house) = match dropper {
                Some(d) => (Some(d), None),
                None => (Some(id), l.dropper_house),
            };
            let warhead = w.rules().combat.general.crush_warhead;
            damage::apply(
                w,
                first,
                Hit {
                    amount: f.health,
                    warhead,
                    distance: 0,
                    attacker,
                    house,
                    ignore_defences: true,
                    no_escape: false,
                },
            );
            false
        }
    }
}

/// Claim 17's blocked test: the occupancy test of S-0209 for the unit, and
/// for a jumpjet or aircraft type the extra clearance test.
fn blocked(w: &World, id: ObjectId, on_deck: bool) -> bool {
    let Some(o) = w.obj(id) else { return true };
    let c = o.pos.cell();
    let reference = crate::occupancy::reference_level(w, id);
    // spec: S-0607 claim 17 (the cell is blocked when the entry outcome that S-0209 claim 5 gives the dropped unit for the cell is anything other than clear: cloaked blocker, moving blocker, closed friendly gate, friendly obstacle, enemy obstacle, stationary friendly and blocked all count; a crushable occupant whose row gives clear does not block)
    let outcome = crate::occupancy::vehicle_outcome(w, id, c, reference);
    if outcome != crate::occupancy::Outcome::Clear {
        return true;
    }
    let r = w.rules();
    let flagged =
        o.kind == Kind::Aircraft || r.unit(o.kind, o.type_index).is_some_and(|u| u.jumpjet);
    if !flagged {
        return false;
    }
    let high = w.high_bridge(c);
    // spec: S-0607 claim 17 (the extra clearance test: the bridge-deck occupancy record when the cell has a high bridge, otherwise the ground record)
    let layer = if high { Layer::Deck } else { Layer::Ground };
    let _ = on_deck;
    // spec: S-0607 claim 17 (a building-present mark fails it; infantry and vehicle or object marks do not)
    if w.cells.occ(c, layer).is_some_and(|rec| rec.buildings > 0) {
        return true;
    }
    let Some(cell) = w.cells.cell(c) else {
        return true;
    };
    // spec: S-0607 claim 17 (a wall overlay fails it)
    if cell.overlay != glasnost_map::NO_OVERLAY
        && r.overlays
            .get(usize::from(cell.overlay))
            .is_some_and(|ov| ov.wall)
    {
        return true;
    }
    // spec: S-0607 claim 17 and S-0211 claim 2 (away from a high bridge, a zero wheeled terrain-speed multiplier fails it)
    !high
        && r.multiplier(cell.land, glasnost_rules::sim::LocomotionClass::Wheel)
            .to_f64()
            .eq_num(F64::ZERO)
}

/// The unit is set sinking (S-0607 claim 17, blocked water).
fn sink(w: &mut World, id: ObjectId) {
    crate::jump::lift_sinking(w, id);
    // spec: S-0607 claim 17 and S-0504 claims 4 and 5 (set sinking and stunned; the stun repeats claim 13's stop on dropping, so it adds nothing observable beyond the sinking)
    crate::naval::start_sinking(w, id);
    upd(w, id, |l| {
        l.sink_credit = true;
        l.dropped = false;
        l.detour = false;
    });
    let l = st(w, id).cloned().unwrap_or_default();
    // spec: S-0607 claim 17 and S-0312 claim 30 (its slaves are freed by the death of their controller, credited to the dropper)
    let hit = Hit {
        amount: 0,
        warhead: None,
        distance: 0,
        attacker: l.dropper.filter(|&d| w.obj(d).is_some()),
        house: l.dropper_house,
        ignore_defences: true,
        no_escape: false,
    };
    crate::econ::slaves::master_killed(w, id, &hit);
}

/// A unit set sinking by claim 17 leaves play (`naval::sinking_turn`): its
/// death is credited to the dropper, else the dropper house (S-0607 claim
/// 17).
pub fn sunk(w: &mut World, id: ObjectId) {
    if !st(w, id).is_some_and(|l| l.sink_credit) {
        return;
    }
    upd(w, id, |l| l.sink_credit = false);
    let l = st(w, id).cloned().unwrap_or_default();
    let dropper = l.dropper.filter(|&d| w.obj(d).is_some());
    let hit = Hit {
        amount: 0,
        warhead: None,
        distance: 0,
        attacker: dropper,
        house: if dropper.is_some() {
            None
        } else {
            l.dropper_house
        },
        ignore_defences: true,
        no_escape: false,
    };
    if hit.attacker.is_none() && hit.house.is_none() {
        return;
    }
    if let Some(o) = w.obj_mut(id) {
        o.health = 0;
    }
    crate::combat::death::mark_killed(w, id);
    // spec: S-0607 claim 17 (its own coming death is credited to the dropper, else to the dropper house)
    damage::credit(w, id, &hit);
}

/// Falling damage (S-0607 claims 18 and 19, F2).
fn impact(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else { return };
    let r = w.rules_arc();
    let aircraft = o.kind == Kind::Aircraft;
    let crashable = r
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.vehicle.crashable);
    if crashable && !aircraft {
        // spec: S-0607 claim 18 (a crashable vehicle type: the ordinary crash end of S-0312 claim 16 applies instead) and claim 19 (not for an aircraft)
        crate::jump::impact(w, id);
        return;
    }
    let mode = r.mode;
    let g = &r.combat.general;
    // spec: S-0607 F2 (B: the current health when the current-strength switch is on, else the type's strength)
    let base = if g.current_strength_damage {
        o.health
    } else {
        damage::strength(w, id)
    };
    // spec: S-0607 F2 (D_f = max(0, trunc(RZ53(B × m))), m a single widened, the product one arithmetic result, trunc the common form)
    let df = F64::from_i64(i64::from(base), mode)
        .mul(g.falling_damage_multiplier, mode)
        .to_i32_common()
        .max(0);
    let l = o.lift.clone();
    let dropper = l.dropper.filter(|&d| w.obj(d).is_some());
    // spec: S-0607 claim 18 (the source is the dropper, or the falling unit with no dropper; credit to the dropper, else the retained dropper house)
    let source = dropper.or(Some(id));
    let house = if dropper.is_some() {
        w.obj(dropper.unwrap_or(id)).and_then(|d| d.owner)
    } else {
        l.dropper_house
    };
    let surface = o.pos.z - crate::jump::height_above(w, id);
    let ground = Pos::new(o.pos.x, o.pos.y, surface);
    let crush = g.crush_warhead;
    let c4 = g.c4_warhead;
    let death = g.death_weapon;
    // spec: S-0607 claim 18 (the first detonation at ground level with the crushing warhead) and claim 19 (an aircraft's first detonation uses the demolition warhead)
    let first = if aircraft { c4 } else { crush };
    detonate(w, death, first, df, ground, source, house);
    // spec: S-0607 claim 18 (the second at the unit's own position with the demolition warhead; both use the pre-impact amount)
    let at = w.obj(id).map_or(o.pos, |x| x.pos);
    detonate(w, death, c4, df, at, source, house);
    let killed = w.obj(id).is_none_or(damage::dead);
    if killed {
        if w.rules().quirks.lift_impact_double_credit {
            // spec: S-0607 claim 18 and quirks.lift_impact_double_credit (the impact credit is given whether or not the unit is already dead: a unit a detonation killed is credited a second time, in the kill and loss tallies, the score and the experience); switch off: skipped
            impact_credit(w, id, &o);
        }
        return;
    }
    if w.rules().quirks.lift_impact_removes_survivor {
        // spec: S-0607 claim 18 (a survivor removed under quirks.lift_impact_removes_survivor is credited once, to the dropper, else the retained dropper house)
        impact_credit(w, id, &o);
        // spec: S-0607 claim 18 and quirks.lift_impact_removes_survivor (the falling unit leaves play after these effects even if both detonations left it alive; a surviving transport loses its passengers without attacker credit, S-0507 claim 16)
        crate::combat::death::leave_play(w, id);
    } else {
        // spec: S-0607 Edge cases (switch off: a unit that survives both detonations stays in play, is set down, and is restored by claim 20)
        set_down_here(w, id);
    }
}

/// The impact credit of claim 18: the destruction of the falling unit is
/// credited to the dropper, else to the retained dropper house. `before` is
/// the unit as it was before the detonations; a unit a detonation has
/// already taken out of play is credited from it.
fn impact_credit(w: &mut World, id: ObjectId, before: &crate::world::Object) {
    let l = &before.lift;
    let dropper = l.dropper.filter(|&d| w.obj(d).is_some());
    // spec: S-0607 claim 18 (impact destruction is credited to the dropper, else to the retained dropper house)
    let hit = Hit {
        amount: 0,
        warhead: None,
        distance: 0,
        attacker: dropper,
        house: if dropper.is_some() {
            None
        } else {
            l.dropper_house
        },
        ignore_defences: true,
        no_escape: false,
    };
    if hit.attacker.is_none() && hit.house.is_none() {
        return;
    }
    if w.obj(id).is_some() {
        damage::credit(w, id, &hit);
        return;
    }
    // The credit reads only the victim's owner, type and experience; the
    // snapshot stands in for the removed unit for that read alone.
    w.objects.insert(id, before.clone());
    damage::credit(w, id, &hit);
    w.objects.remove(&id);
}

/// One impact detonation of claim 18: the death weapon's projectile with the
/// given warhead. With no death weapon, a plain area detonation (S-0607
/// claim 18, the design rule).
fn detonate(
    w: &mut World,
    weapon: Option<crate::combat::data::WeaponId>,
    warhead: Option<crate::combat::data::WarheadId>,
    amount: i32,
    at: Pos,
    source: Option<ObjectId>,
    house: Option<HouseId>,
) {
    let Some(h) = warhead else { return };
    match weapon {
        Some(wp) => {
            crate::combat::projectile::detonate_weapon_as(w, wp, h, amount, at, source, house);
        }
        None => {
            // spec: S-0607 claim 18 (no death weapon, design: each detonation becomes a plain area detonation of its warhead at its point, with the same amount, source and house, and no projectile effects)
            crate::combat::projectile::detonate_area(w, h, amount, at, source, house, true);
        }
    }
}

/// A seized unit touches down (S-0607 Terms, claim 17; S-0502 claim 15): it
/// is set down and no longer dropped or crashing.
pub fn touched_down(w: &mut World, id: ObjectId) {
    if !seized(w, id) {
        return;
    }
    // spec: S-0607 Terms (a seized unit is also set down when it touches down at the end of a descending flight) and claim 17 (no longer dropped or crashing)
    upd(w, id, |l| {
        l.set_down = true;
        l.dropped = false;
        l.detour = false;
        l.rate = None;
    });
}

// ---------------------------------------------------------------------------
// Restoration (claim 20)

/// At a fall-back to rest (S-0215 claim 13 row 1): a seized unit whose
/// jumpjet movement is not *moving* and that is set down gets its own
/// locomotor back (S-0607 claim 20). Returns whether it did.
pub fn restore_at_rest(w: &mut World, id: ObjectId) -> bool {
    let Some(l) = st(w, id).cloned() else {
        return false;
    };
    if !l.seized {
        return false;
    }
    // spec: S-0607 claim 20 (its jumpjet movement is not moving, S-0502's moving, not moving now; and it is set down)
    if crate::jump::moving(w, id) || !l.set_down || l.lifter.is_some() {
        return false;
    }
    restore(w, id);
    true
}

/// Gives a seized unit its own locomotor back (S-0607 claim 20).
fn restore(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else { return };
    if o.lift.stand_in {
        // spec: S-0607 claim 22 (when a unit seized with a stand-in locomotor is restored is outside the spec); Q-1334 interim (claim 20's rule; its own movement state and marks were kept, so nothing is put back)
        upd(w, id, |l| {
            l.seized = false;
            l.set_down = false;
            l.dropped = false;
            l.detour = false;
            l.dropper = None;
            l.dropper_house = None;
            l.rate = None;
            l.stand_in = false;
        });
        return;
    }
    let jumpjet_type = w
        .rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.locomotor == crate::rules::Locomotor::Jump);
    crate::jump::lift_restore(w, id, jumpjet_type);
    if !jumpjet_type {
        // spec: S-0215 claim 13 (restoration retains the suspended movement state) and S-0209 claim 1 (the own locomotor's mark at its cell)
        let Some(o) = w.obj(id).cloned() else { return };
        let (c, layer) = (o.pos.cell(), o.layer);
        match o.kind {
            Kind::Vehicle => {
                // spec: S-0207 claim 21 (no occupancy mark is moved merely by restoration: a retained teleport reservation stays where it is)
                let pending = o.teleport.as_ref().is_some_and(|t| t.mark.is_some());
                if !pending {
                    w.cells.claim_vehicle(c, layer, true);
                }
                if let Some(ob) = w.obj_mut(id) {
                    if let Some(d) = ob.drive.as_mut() {
                        d.claims = vec![c];
                    }
                    if let Some(h) = ob.hover.as_mut() {
                        h.claims = vec![(c, layer)];
                    }
                }
            }
            Kind::Aircraft => crate::air::ground_mark(w, id),
            _ => {}
        }
    }
    // spec: S-0607 claim 20 (restoration clears seized, set down and the dropper)
    upd(w, id, |l| {
        l.seized = false;
        l.set_down = false;
        l.dropped = false;
        l.detour = false;
        l.dropper = None;
        l.dropper_house = None;
        l.rate = None;
    });
    let owner = o.owner;
    if resource_gatherer(w, id)
        && non_human_rule(w, owner)
        && let Some(ob) = w.obj_mut(id)
    {
        // spec: S-0607 claim 20 (a seized resource gatherer of a computer-controlled house queues harvest in the original; our AI houses take the human rule, the neutral and special houses the original's)
        ob.beh.queue(Some(Behaviour::Harvest));
    }
}

// ---------------------------------------------------------------------------
// Firing (claims 10 and 11)

/// The magnetic-lift answers of the firing check at S-0301 claim 9's rank 4
/// (S-0607 claims 10 and 11).
pub fn firing_answer(
    w: &World,
    firer: ObjectId,
    mag_beam: bool,
    target: Option<Target>,
) -> Option<crate::combat::check::Answer> {
    use crate::combat::check::Answer;
    // spec: S-0607 claim 10 (a seized unit gets illegal for every weapon and target, at S-0301 claim 9's rank 4, from the seize until restoration)
    if seized(w, firer) {
        return Some(Answer::Illegal);
    }
    if !mag_beam {
        return None;
    }
    let held = gripped_unit(w, firer);
    // spec: S-0607 claim 11 (a magnetic-beam weapon's firer that grips a unit and has a live wave, with a target that is neither the gripped unit nor a building: rearming)
    if held.is_some() && wave_alive(w, firer) {
        let building = matches!(target, Some(Target::Object(x)) if w.obj(x).is_some_and(|t| t.kind == Kind::Building));
        let own = matches!(target, Some(Target::Object(x)) if Some(x) == held);
        if !building && !own {
            return Some(Answer::Rearming);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// The magnetic wave (claim 21)

/// The fade step *q* of F4: the single-precision constant 13421773 /
/// 268435456, widened exactly.
fn fade_step(mode: glasnost_num::Rounding) -> F64 {
    // spec: S-0607 F4 and Constants (q = 13421773 / 268435456, exact)
    F64::from_parts(false, 13_421_773, -28, mode)
}

/// The full-strength threshold of F4: 8827055269646172 / 2^53.
fn full_threshold(mode: glasnost_num::Rounding) -> F64 {
    // spec: S-0607 F4 and Constants (the threshold 8827055269646172 / 9007199254740992)
    F64::from_parts(false, 8_827_055_269_646_172, -53, mode)
}

/// A magnetic-beam weapon's shot (S-0607 claim 21): it creates a wave only
/// when its firer has no live wave and the horizontal distance between the
/// firing and impact points is at least 240 leptons (F3).
pub fn shot(w: &mut World, firer: ObjectId, target: Target, fire_point: Pos, impact: Pos) {
    // spec: S-0607 claim 21 (created only when its firer has no live wave, sonic or magnetic, S-0301 claim 5; sonic waves are not modelled in our engine, so only the magnetic one counts)
    if wave_alive(w, firer) {
        return;
    }
    // spec: S-0607 F3 (the differences, squares and sum in signed 32-bit wrapping arithmetic)
    let dx = impact.x.wrapping_sub(fire_point.x);
    let dy = impact.y.wrapping_sub(fire_point.y);
    let n = dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy));
    let mode = w.rules().mode;
    // spec: S-0607 F3 (d = S-0204 claim 5's table square root, without integer truncation; for a sum that wraps negative, r(n) = r(−n): the root's input is made positive first)
    let d = glasnost_num::geom::table_root(i64::from(n).unsigned_abs(), mode).to_f64();
    // spec: S-0607 F3 and Constants (no wave when d < 240 leptons)
    if d.lt(F64::from_i64(240, mode)) {
        return;
    }
    let id = w.new_id();
    w.waves.insert(
        id,
        Wave {
            firer,
            target,
            a: F64::ZERO,
            b: F64::ZERO,
            ended: false,
        },
    );
    // spec: S-0607 claim 21 (the wave joins the update order when it is created, appended after every object already in play, S-0203 claims 2 and 3)
    w.update.enter(id);
    // spec: S-0607 claim 21 and F4 (one update at creation as well as the per-tick updates)
    wave_update(w, id);
}

/// A wave's own turn in the update order (S-0607 claim 21, F4): each tick
/// after the one it was created in, after its firer's turn.
pub fn wave_turn(w: &mut World, id: ObjectId) {
    wave_update(w, id);
}

/// Whether an object is in play for the wave's end test (S-0607 claim 21):
/// present and on the map, and not inside a transport, a bunker or a
/// building.
fn in_play(w: &World, x: ObjectId) -> bool {
    w.obj(x).is_some_and(|o| {
        o.on_map && o.structure.inside.is_none() && crate::transport::riding(w, x).is_none()
    }) && crate::structures::bunker::bunkered_in(w, x).is_none()
}

fn wave_update(w: &mut World, id: ObjectId) {
    let Some(mut v) = w.waves.get(&id).copied() else {
        return;
    };
    let mode = w.rules().mode;
    let q = fade_step(mode);
    let one = F64::from_i64(1, mode);
    if !v.ended {
        let firer_gone = !in_play(w, v.firer);
        let target_gone = matches!(v.target, Target::Object(t) if !in_play(w, t));
        let retargeted = w.obj(v.firer).and_then(|o| o.target) != Some(v.target);
        // spec: S-0607 claim 21 (the wave ends when its firer or its target leaves play, going inside a transport, a bunker or a building included, or when its firer's target, as it stands at the wave's turn, is no longer the wave's target)
        if firer_gone || target_gone || retargeted {
            v.ended = true;
        }
    }
    if !v.ended {
        // spec: S-0607 F4 (while the wave travels, each update with a < 1 gives a = RZ53(a + q); a result above the threshold becomes exactly 1)
        if v.a.lt(one) {
            v.a = v.a.add(q, mode);
            if v.a.gt(full_threshold(mode)) {
                v.a = one;
            }
        }
    } else {
        let half = F64::from_parts(false, 1, -1, mode);
        // spec: S-0607 F4 (once the wave has ended, each update with b ≤ RZ53(a + RZ53(q × 0.5)) gives b = RZ53(b + q); it is removed in that update if the new b ≥ a)
        if v.b.le(v.a.add(q.mul(half, mode), mode)) {
            v.b = v.b.add(q, mode);
            if v.b.ge(v.a) {
                w.waves.remove(&id);
                let skip = w.rules().quirks.update_order_removal_skip;
                // spec: S-0203 claim 10 (a removal from the update order, counting during the turn pass)
                w.update.leave(id, skip);
                return;
            }
        }
    }
    w.waves.insert(id, v);
}
