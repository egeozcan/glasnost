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
//! A seizing warhead that names another locomotor follows claims 22 to 24:
//! one that can stand in temporarily seizes with that locomotor installed as
//! the unit's stand-in, which moves it towards the lifter by its own rules
//! and, under `quirks.lift_standin_stays_seized`, never lets go; one that
//! cannot (or an unknown identifier) stops the target without a seize. The
//! team-membership removal of claim 5 belongs to the original's team
//! system, which is out of scope.

use glasnost_map::CellXY;
use glasnost_num::{F32, F64};
use serde::{Deserialize, Serialize};

use crate::combat::damage::{self, Hit};
use crate::combat::data::SeizeLocomotor;
use crate::rules::{Behaviour, Kind, Locomotor};
use crate::world::{HouseId, Layer, ObjectId, Pos, Target, World};

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

/// A stand-in locomotor other than the jumpjet's (S-0607 Terms, claims 22
/// to 24): the locomotor a seize installs in place of the unit's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StandIn {
    /// The driving locomotor (S-0207).
    Drive,
    /// The walking locomotor: our engine keeps the unit where it is
    /// (claim 23, design).
    Walk,
    /// The ship locomotor (S-0504).
    Ship,
    /// The teleporting locomotor (S-0207 claim 20).
    Teleport,
    /// The drop-pod locomotor: our engine ends it at once at the seize
    /// (claim 23, design).
    DropPod,
}

/// A seized unit's own movement states, kept aside while a stand-in is
/// installed (S-0607 claim 24 with `quirks.lift_standin_stays_seized` off;
/// S-0215 claim 13: restoration retains the suspended movement state).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnLocomotor {
    /// Its own driving or ship state.
    pub drive: Option<crate::drive::DriveState>,
    /// Its own walking state.
    pub walk: Option<crate::walk::WalkState>,
    /// Its own hovering state.
    pub hover: Option<crate::hover::HoverState>,
    /// Its own teleporting state.
    pub teleport: Option<crate::teleport::TeleportState>,
    /// Its own jumpjet state.
    pub jump: Option<crate::jump::JumpState>,
    /// It was driving temporarily for a factory exit (S-0417 claim 5).
    pub temporary_drive: bool,
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
    /// The stand-in locomotor of a unit seized with a locomotor other than
    /// jumpjet that can stand in temporarily (S-0607 claims 22 to 24);
    /// none for the jumpjet's lift. A drop-pod stand-in stays recorded
    /// after it has ended, since the unit stays seized (claim 24).
    #[serde(default)]
    pub stand_in: Option<StandIn>,
    /// The unit's own movement states while a stand-in is installed.
    #[serde(default)]
    pub own: Option<Box<OwnLocomotor>>,
    /// A gripped unit whose teleporting stand-in found no landing cell
    /// while the seize assigned its destination (S-0607 claim 23): that
    /// removal preceded the seized state, so the grip lasts with no
    /// destination until another event of claim 12.
    #[serde(default)]
    pub grip_without_dest: bool,
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

/// The stand-in locomotor of a unit seized with one (S-0607 claims 22 to
/// 24); none for a unit that is not seized or is seized with the jumpjet.
pub fn stand_in(w: &World, id: ObjectId) -> Option<StandIn> {
    st(w, id).filter(|l| l.seized).and_then(|l| l.stand_in)
}

/// The stand-in that moves a seized unit now (S-0607 claim 23): none for a
/// unit not seized with a stand-in, and for the drop-pod stand-in, which
/// has already ended (the unit has its own locomotor back).
pub fn moving_stand_in(w: &World, id: ObjectId) -> Option<StandIn> {
    stand_in(w, id).filter(|&k| k != StandIn::DropPod)
}

/// The locomotor the unit moves with now (S-0215 claim 11, revision 68:
/// "the vehicle's current locomotor"; S-0607 claims 5, 20, 23 and 24): the
/// temporary jumpjet while seized with it, the stand-in while one is
/// installed, otherwise its own (the driving locomotor while it drives
/// temporarily, S-0417 claim 5).
pub fn current_locomotor(w: &World, id: ObjectId) -> Option<Locomotor> {
    let o = w.obj(id)?;
    if o.lift.seized {
        // spec: S-0607 claim 20 and S-0215 claim 11 (while seized the current locomotor is the temporary jumpjet movement)
        match o.lift.stand_in {
            None => return Some(Locomotor::Jump),
            // spec: S-0607 claims 22 and 23 (the stand-in installed in place of its own)
            Some(StandIn::Drive) => return Some(Locomotor::Drive),
            Some(StandIn::Walk) => return Some(Locomotor::Walk),
            Some(StandIn::Ship) => return Some(Locomotor::Ship),
            Some(StandIn::Teleport) => return Some(Locomotor::Teleport),
            // spec: S-0607 claims 23 and 24 (the drop-pod stand-in has ended: the unit has its own locomotor back)
            Some(StandIn::DropPod) => {}
        }
    }
    if crate::econ::release::temporary_drive(w, id) {
        return Some(Locomotor::Drive);
    }
    w.rules().unit(o.kind, o.type_index).map(|u| u.locomotor)
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
    // spec: S-0306 claim 17 and Questions for the project lead (engine design, no switch: a locomotor-seizing hit does nothing to a superweapon warp member from its departure until its arrival hold ends; the row still replaces ordinary damage; Q-2231)
    if crate::superweapons::warp::phasing(w, x) {
        return;
    }
    // spec: S-0607 claim 22 (claims 1 to 3 decide the seize the same way whatever locomotor the warhead names)
    if !eligible(w, x, damage) {
        return;
    }
    // spec: S-0607 State and ordering (a resource gatherer made immune by claim 3 keeps its temporary locomotor: claim 3's conditions come before claim 4's special case) and claims 4 and 22 (revision 5: claim 4's case is decided before the warhead's locomotor is considered, whichever locomotor it names, an unknown identifier included)
    if harvester_case(w, x) {
        return;
    }
    match w.rules().combat.warhead(warhead).seize_locomotor {
        SeizeLocomotor::Jumpjet => seize(w, f, x, None),
        // spec: S-0607 claim 22 (a locomotor that can stand in temporarily: the target is seized as in claim 5, with that locomotor installed as its stand-in in place of its own)
        SeizeLocomotor::StandIn(k) => seize(w, f, x, Some(k)),
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
    // spec: S-0607 r39 claim 3 (the layer is the one the target's state and height give it, S-0102 claim 3, not the list it is filed in: an aircraft whose type cannot land, neither rocket nor drop-pod movement, is not seized by a hit while it is above height 0, although it stays in the ground list, S-0306 claim 17)
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
/// and 20: the neutral and special houses and a test-only
/// computer-controlled player house (glossary; our AI houses take the human
/// rule).
fn non_human_rule(w: &World, h: Option<HouseId>) -> bool {
    h.and_then(|h| w.houses.get(usize::from(h.0)))
        .is_some_and(|x| !x.human)
}

/// The seize itself (S-0607 claim 5; claims 22 and 23 for a stand-in).
fn seize(w: &mut World, lifter: ObjectId, x: ObjectId, stand_in: Option<StandIn>) {
    let t = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    // spec: S-0607 claim 5 (it stops: its route is cleared, it gives up the occupancy mark of its cell and it stands still; claims 22 and 23 replace only the link and the destination for a stand-in)
    suspend_own(w, x);
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
    match stand_in {
        // spec: S-0607 claim 5 and claim 7 (its jumpjet locomotor is linked, with the jumpjet fields of the unit's own type)
        None => crate::jump::link_lift(w, x),
        // spec: S-0607 claims 22 and 23 (that locomotor is installed as its stand-in in place of its own)
        Some(k) => install_stand_in(w, x, k),
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
    let p = lifter_point(w, lifter, x);
    if let Some(k) = stand_in {
        if let Some(o) = w.obj_mut(x) {
            // spec: S-0607 claim 23 (at the seize the unit is given the lifter as its order-level destination, exactly as in claim 5, before the seized state begins; with another stand-in it stays the lifter itself, an object destination, not the cell of a point)
            o.dest = Some(Target::Object(lifter));
        }
        // spec: S-0607 claim 23 (the stand-in receives the lifter's point of claim 7 as its movement instruction; no nearby-cell search and no fixed movement point of its own)
        instruct_stand_in(w, x, k, lifter, p);
    }
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
        l.grip_without_dest = false;
    });
    let harvester = resource_gatherer(w, x);
    let hunt = non_human_rule(w, w.obj(x).and_then(|o| o.owner))
        && !w
            .obj(x)
            .and_then(|o| o.owner)
            .is_some_and(|h| crate::econ::setup::passive(w, h))
        && !harvester;
    if stand_in.is_none() {
        // spec: S-0607 claims 5 and 7 (the grip is made and the unit is given the lifter as its destination; its jumpjet movement turns this at once into a movement point fixed at the seize, from the lifter's point)
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
    match stand_in {
        Some(StandIn::DropPod) => drop_pod_ends(w, lifter, x),
        Some(_) if w.obj(x).is_some_and(|o| o.dest.is_none()) => {
            // spec: S-0607 claim 23 (revision 7: when the teleporting stand-in's landing search finds no cell, S-0207 claim 20 b removes the destination while the seize itself assigns it, before the seized state begins; that removal does not end the grip: the unit stays where it is with no destination, never jumps, and the grip lasts until another event of claim 12)
            upd(w, x, |l| l.grip_without_dest = true);
        }
        Some(_) | None => {}
    }
}

/// Installs a stand-in locomotor on a seized unit (S-0607 claims 22 and
/// 23): its own movement states are kept aside and the stand-in starts
/// afresh, at rest in the unit's cell, holding no occupancy mark until its
/// own movement next marks a cell. A drop-pod stand-in ends at once and
/// installs nothing ([`drop_pod_ends`]); a landed aircraft keeps its own
/// states and stays where it is.
fn install_stand_in(w: &mut World, x: ObjectId, k: StandIn) {
    let Some(o) = w.obj(x) else { return };
    if k == StandIn::Teleport && o.kind == Kind::Aircraft {
        // spec: S-0607 claim 23 (revision 7: a landed aircraft seized with the teleporting stand-in receives the lifter's point as any unit does; S-0207 claim 20 covers aircraft)
        let Some(o) = w.obj_mut(x) else { return };
        let own = OwnLocomotor {
            teleport: o.teleport.take(),
            ..OwnLocomotor::default()
        };
        o.lift.own = Some(Box::new(own));
        o.teleport = Some(crate::teleport::TeleportState::default());
        return;
    }
    if k == StandIn::DropPod || o.kind != Kind::Vehicle {
        // spec: S-0607 claim 23 (revision 7: a landed aircraft seized with the driving, walking or ship stand-in, design: no movement spec covers it; our engine keeps it where it stands, it does not move or turn under that stand-in)
        return;
    }
    if o.jump.is_some() {
        // spec: S-0607 claim 5 (it gives up the occupancy mark of its cell: a jumpjet type's own marks, reservation and slot go)
        crate::jump::give_up_marks(w, x);
    }
    let Some(o) = w.obj_mut(x) else { return };
    let own = OwnLocomotor {
        drive: o.drive.take(),
        walk: o.walk.take(),
        hover: o.hover.take(),
        teleport: o.teleport.take(),
        jump: o.jump.take(),
        temporary_drive: std::mem::take(&mut o.econ.temporary_drive),
    };
    o.lift.own = Some(Box::new(own));
    match k {
        StandIn::Drive | StandIn::Ship => o.drive = Some(crate::drive::DriveState::new()),
        StandIn::Teleport => o.teleport = Some(crate::teleport::TeleportState::default()),
        StandIn::Walk | StandIn::DropPod => {}
    }
    // spec: S-0607 claim 23 and quirks.lift_standin_unmarked_cell (on: installing the stand-in marks no cell; a driving or ship stand-in marks cells from the start of its first step into a neighbouring cell, S-0209 claim 1, and a walking one never does)
    if !w.rules().quirks.lift_standin_unmarked_cell
        && matches!(k, StandIn::Drive | StandIn::Walk | StandIn::Ship)
        && let Some(o) = w.obj(x)
    {
        let (c, layer) = (o.pos.cell(), o.layer);
        // spec: S-0607 claim 23 and Edge cases (quirks.lift_standin_unmarked_cell off: the unit keeps the vehicle-present mark of its own cell at the seize, as a vehicle at rest does; its stand-in's movement then clears and marks cells as usual)
        w.cells.claim_vehicle(c, layer, true);
        if let Some(d) = w.obj_mut(x).and_then(|o| o.drive.as_mut()) {
            d.claims = vec![c];
        }
    }
}

/// The stand-in takes the lifter's point as its movement instruction
/// (S-0607 claim 23).
fn instruct_stand_in(w: &mut World, x: ObjectId, k: StandIn, lifter: ObjectId, p: Pos) {
    let Some(kind) = w.obj(x).map(|o| o.kind) else {
        return;
    };
    // spec: S-0607 claim 23 (revision 7: a landed aircraft takes the lifter's point only with the teleporting stand-in; with the driving, walking or ship stand-in it stays where it stands)
    if kind != Kind::Vehicle && !(kind == Kind::Aircraft && k == StandIn::Teleport) {
        return;
    }
    let dest = Some(Target::Object(lifter));
    match k {
        StandIn::Drive | StandIn::Ship => {
            if let Some(d) = w.obj_mut(x).and_then(|o| o.drive.as_mut()) {
                // spec: S-0607 claim 23 and S-0207 claim 25 (the locomotor destination point is the lifter's point, taken for the lifter as the order-level destination)
                d.for_dest = dest;
                d.goal = Some(p);
            }
        }
        // spec: S-0607 claim 23 and S-0207 claim 20 (a) and (b) (the teleporting stand-in fixes its landing point from the cell of the lifter's point)
        StandIn::Teleport => {
            crate::teleport::stand_in_instruction(w, x, Target::Object(lifter), p.cell())
        }
        // spec: S-0607 claim 23 (walking, design: the unit does not move or turn under that stand-in)
        StandIn::Walk | StandIn::DropPod => {}
    }
}

/// The drop-pod stand-in ends at once at the seize (S-0607 claim 23,
/// design, with the original's touchdown result): the unit stays where it
/// is, gets its own locomotor back and the grip ends with it set down and
/// the lifter's target cleared. It stays seized (claim 24).
fn drop_pod_ends(w: &mut World, lifter: ObjectId, x: ObjectId) {
    // spec: S-0607 claims 23 and 24 (our engine ends the drop-pod stand-in at once at the seize: the unit stays where it is and gets its own locomotor back, marking its cell again as it returns)
    own_marks_back(w, x);
    // spec: S-0607 claim 12 (revision 7, the row for a gripped unit's drop-pod stand-in ending: the lifter's target is cleared) and claim 23 (the grip ends with the unit set down; the lifter is no longer gripping, so claim 11's refusals stop at once, and the cleared target ends the wave at its next update, claim 21). Our engine reproduces none of the original's touchdown target drop for other objects: every object other than the lifter keeps its target.
    upd(w, lifter, |s| {
        s.gripped = None;
        s.grip_dest = None;
    });
    if let Some(o) = w.obj_mut(lifter) {
        crate::spawn::clear_own_target(o);
        o.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
    upd(w, x, |s| s.lifter = None);
    set_down_here(w, x);
}

/// The lifter's point at the seize (S-0607 claim 7): a mobile lifter's
/// heading point (Terms: its next movement point, or its position when it
/// has none); a building's foundation centre, or its docking point
/// ([`docking_point`]) when its type has the pad, the repair-depot or the
/// vehicle-bunker flag.
pub fn lifter_point(w: &World, lifter: ObjectId, x: ObjectId) -> Pos {
    let Some(o) = w.obj(lifter) else {
        return Pos::default();
    };
    if o.kind == Kind::Building {
        let flags = w
            .rules()
            .buildings
            .get(o.type_index as usize)
            .map(|b| {
                (
                    b.raw.helipad.unwrap_or(false),
                    b.raw.unit_repair.unwrap_or(false),
                    b.raw.bunker.unwrap_or(false),
                )
            })
            .unwrap_or_default();
        // spec: S-0607 claim 7 and Import mapping (revision 5: a building gives its docking point only when its type has the pad flag, the repair-depot flag or the vehicle-bunker flag, each absent no; every other building, a refinery included, gives the centre of its foundation)
        if flags.0 || flags.1 || flags.2 {
            return docking_point(w, lifter, x);
        }
        // spec: S-0607 claim 7 (a building: the centre of its foundation, at its world height)
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

/// The docking point a building lifter gives (S-0607 claim 7, revision
/// 5): the one it would give the seized unit itself, by claim 7's
/// precedence: the obsolete-unloading flag's fixed point, else the
/// resource-unloading flag's point, else the vehicle-bunker quadrant
/// point, else the docking-place rule.
fn docking_point(w: &World, k: ObjectId, x: ObjectId) -> Pos {
    let Some(b) = w.obj(k) else {
        return Pos::default();
    };
    let Some(t) = w.rules().buildings.get(b.type_index as usize) else {
        return w.reference_point(k);
    };
    let centre = w.reference_point(k);
    // spec: S-0607 claim 7 (first, with the obsolete-unloading flag: the fixed point of S-0314 claim 18, the centre of the cell displaced by (2, 1) cells from the top-left foundation cell, at the building's height) and Import mapping (Weeder, absent no)
    if t.raw.econ.weeder.unwrap_or(false) {
        let c = b.pos.cell();
        return Pos::centre(
            CellXY::new(c.x.wrapping_add(2), c.y.wrapping_add(1)),
            b.pos.z,
        );
    }
    // spec: S-0607 claim 7 (else with the resource-unloading flag: the foundation centre displaced by +128 leptons along x) and Import mapping (Refinery, absent no)
    if t.raw.econ.refinery.unwrap_or(false) {
        return Pos::new(centre.x + 128, centre.y, centre.z);
    }
    // spec: S-0607 claim 7 (else with the vehicle-bunker flag: the bunker's quadrant point on the seized unit's side, S-0606 claim 4, from the seized unit's position at the seize)
    if t.raw.bunker.unwrap_or(false) {
        return crate::structures::bunker::docking_point(w, k, x);
    }
    // spec: S-0607 claim 7 (else the docking-place rule of S-0314 claim 18 and S-0404 claim 15: with a greater count than 1, the place the seized unit holds through a contact with that building, the foundation centre when it holds none; the seize breaks that contact only after this point is taken, claim 5)
    // spec: S-0404 Import mapping (NumberOfDocks, rules, absent 1)
    let count = t.raw.econ.number_of_docks.unwrap_or(1);
    let held = crate::depot::link(w, x)
        .filter(|l| l.depot == k)
        .map(|l| l.place)
        .or_else(|| {
            w.obj(x)
                .filter(|o| o.contact == Some(k))
                .and_then(|o| o.econ.exit)
                .filter(|e| e.factory == k)
                .map(|e| e.place)
        });
    match (count, held) {
        // spec: S-0607 claim 7 (a docking-place count of 0: the foundation centre; of 1: the centre plus the sole docking displacement)
        (c, _) if c <= 1 => crate::econ::release::pad_position_at(w, k, 0),
        // spec: S-0607 claim 7 (a greater count: the displacement of the place held through a contact with that building)
        (_, Some(place)) => crate::econ::release::pad_position_at(w, k, place),
        // spec: S-0607 claim 7 (a greater count and no place held: the foundation centre)
        (_, None) => centre,
    }
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
        // spec: S-0607 claim 13 (a grip ended by its lifter: the jumpjet stop with the design rule for no cell)
        set_down_released(w, x, true);
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
        l.grip_without_dest = false;
    });
    // spec: S-0607 claim 13 (revision 33, a dropped balloon-hover vehicle: the stop on dropping first gives it no destination while it still holds its target, which S-0502 claim 28 can reject, making attack current; it then clears the target and gives no destination again, which that claim cannot reject, so it still ends with none and the direct stop; the target's clearing does not undo the change of behaviour)
    let _ = crate::jump::clear_rejected(w, x);
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
    // spec: S-0607 claim 13 (revision 33: on a drop the stop always runs: the stop on dropping gives its jumpjet movement a direct instruction to stop, not a destination assignment, so it runs whether or not the unit still had a destination; S-0502 claim 17's stopped-movement rule in the same update; a grip ended by its lifter takes the design rule for no cell: no self-destruction, it falls as claim 15 says)
    crate::jump::lift_release_stop(w, x, true);
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

/// A gripped unit is set down where it is at the end of its grip (S-0607
/// claims 9 and 13): its destination becomes none and the jumpjet stop of
/// claim 13 runs (`lifter_ended`: with its design rule for no cell; claim
/// 9's no-destination exception: S-0502 claim 17 unchanged). A unit whose
/// jumpjet movement is still moving afterwards keeps the found point and
/// its commitment: S-0502 claims 5 and 24 take it off, fly it to the point
/// and land it there, and claim 20 restores it only after that. A claim 9
/// unit in the air that the stop left hovering with no cell (the switch
/// off) keeps hovering, set down and seized. Otherwise it descends and
/// lands from above the surface, or stays where it is.
fn set_down_released(w: &mut World, x: ObjectId, lifter_ended: bool) {
    // spec: S-0607 claim 13 (revision 33: a set-down unit that already has no destination, left so by part e's clearing in the grip, claim 9, takes no stop: the assignment is redundant)
    let effective = w.obj(x).is_some_and(|o| !o.lift.grip_without_dest);
    upd(w, x, |l| {
        l.set_down = true;
        l.dropped = false;
        l.detour = false;
        l.rate = None;
        l.grip_without_dest = false;
    });
    // spec: S-0607 claim 13 (revision 33, a balloon-hover type: the set-down's assignment of no destination, made after the grip has ended, is rejected under S-0502 claim 28 when the vehicle still holds a destination and a target and a row holds; it keeps its destination, movement point and commitment, takes no stop, and attack becomes current: a set-down unit still moving)
    let rejected = effective && crate::jump::clear_rejected(w, x);
    if !rejected && let Some(o) = w.obj_mut(x) {
        // spec: S-0607 claims 9 and 13 (set down: its destination becomes none)
        o.dest = None;
    }
    if w.obj(x).is_none_or(|o| o.jump.is_none()) {
        // spec: S-0607 claim 23 (a unit on another stand-in is set down where it stands: no jumpjet stop, descent or touchdown)
        return;
    }
    let was_moving = crate::jump::moving(w, x);
    if effective && !rejected {
        // spec: S-0607 claim 13 (revision 33: on a set-down the stop runs only through the destination's clearing, an effective order-level clear, S-0502 claim 24, which runs S-0502 claim 17's stopped-movement rule in the same update) and claim 9 (the no-destination exception: the grip has already ended when the stop runs, so S-0502 claim 17's self-destruction applies unchanged, with its quirk switch and no credit)
        crate::jump::lift_release_stop(w, x, lifter_ended);
    }
    if w.obj(x).is_none_or(damage::dead) {
        return;
    }
    if crate::jump::moving(w, x) {
        // spec: S-0607 claim 13 (revision 6: a set-down unit still moving keeps the found point and its movement commitment; being set down clears neither) and S-0502 claims 5 and 24 (revision 14: its next eligible jumpjet update takes off, then ordinary flight and arrival towards that point)
        return;
    }
    let above = crate::jump::height_above(w, x) >= 1; // spec: S-0607 claim 9 (revision 8: in the air is a height above the surface of 1 lepton or more)
    if was_moving && !lifter_ended && above {
        // spec: S-0607 claim 9 (revision 8, design: no cell found with quirks.jumpjet_no_cell_self_destruct off, in the air: it re-enters hovering over its current cell with no movement point and no commitment, neither descends nor flies anywhere, and stays set down and seized; claim 20 retries the search at the end of each of its turns, not at a fall-back to rest, `end_of_turn`) and S-0502 Edge cases (revision 21: the retry at the end of each of its turns, S-0203)
        return;
    }
    if above {
        // spec: S-0607 claim 13 (a gripped unit is not moving only when the seize's search found no cell, and it is then still grounded); an unmoving unit set down above the surface descends and lands where it is (claim 17's clear-cell rule, as for claim 23's set-down)
        crate::jump::lift_descend(w, x);
    } else {
        // spec: S-0607 claim 13 (no cell found for a grip ended by its lifter, design: a set-down unit stops where it is, with no movement point, its jumpjet movement no longer moving) and claim 9 (revision 9, switch off, on the ground, height above the surface 0 or less: it stays where it is and becomes grounded, even from ascending at height 0, with no movement point, no longer moving, and claim 20 restores it as any grounded set-down unit)
        crate::jump::lift_settle(w, x);
    }
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
    if w.obj(x).is_some_and(|o| o.jump.is_none()) {
        // spec: S-0607 claim 23 (a unit on another stand-in, or with its own locomotor back from a drop-pod stand-in, is set down where it stands: no jumpjet descent or touchdown)
        return;
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
    // spec: S-0607 claim 12 (row 2: the lifter is a mobile unit and accepts a non-empty destination, from an order, an approach step or another rule: the grip ends and the lifter's target is cleared; revision 18: an approach step against its gripped unit gives none, claim 11, `approach::step`, except S-0215 claim 37 part h's, whose destination is the gripped unit and ends the grip here)
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
/// ignored; no destination given while it had one ends the grip without a
/// drop. A gripped unit left with no destination by a direct removal
/// ([`destination_removed`]) keeps none, and its grip holds. A seized unit
/// that is not gripped keeps no destination, apart from one a balloon-hover
/// set-down kept (claim 13): the destination its locomotor still holds.
pub fn steer(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let dest = o.dest;
    let without = o.lift.grip_without_dest;
    match lifter_of(w, id) {
        Some(l) if w.obj(l).is_some() && without => {
            if dest.is_some()
                && let Some(ob) = w.obj_mut(id)
            {
                // spec: S-0607 claim 9 (revision 33, Q-2151: a seized unit ignores every new destination; after part e's clearing a gripped unit has none, and a later rule giving it none is a redundant assignment: the grip holds, its jumpjet movement keeping its movement point and commitment, claim 7)
                ob.dest = None;
            }
        }
        Some(l) if w.obj(l).is_some() => {
            match dest {
                None => {
                    // spec: S-0607 claim 9 (a rule that gives a gripped unit no destination ends its grip without a drop: set down where it is) and claim 12 (the lifter's target is not cleared)
                    upd(w, l, |s| {
                        s.gripped = None;
                        s.grip_dest = None;
                    });
                    upd(w, id, |s| s.lifter = None);
                    // spec: S-0607 claim 9 (claim 13's jumpjet stop applies, but not its design rule for no cell)
                    set_down_released(w, id, false);
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
            // spec: S-0607 claim 13 (revision 33: a set-down balloon-hover vehicle whose set-down assignment S-0502 claim 28 rejected keeps its destination, movement point and commitment; its locomotor still holds that destination)
            let kept = dest.is_some() && dest == crate::jump::lift_order(w, id);
            if kept {
                return;
            }
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

/// A rule removes a unit's destination directly, not by assigning it an
/// empty one (S-0215 claim 37 part e's clearing; S-0607 claim 9, revision
/// 33, Q-2151): for a gripped unit that is no effective clear and not claim
/// 9's exception. Its grip holds, whatever its stand-in, and it keeps no
/// destination until another event of claim 12 ends the grip.
pub fn destination_removed(w: &mut World, id: ObjectId) {
    // spec: S-0607 claim 9 (revision 33: part e removes the destination and nothing else: the grip holds, whatever the unit's stand-in, since the removal is not a change by assignment; with the jumpjet stand-in the movement keeps its point and commitment, claim 7)
    if lifter_of(w, id).is_some_and(|l| w.obj(l).is_some()) {
        upd(w, id, |l| l.grip_without_dest = true);
    }
}

/// Enforces claims 9 and 23 for a unit seized with a stand-in locomotor
/// (S-0607 claims 22 to 24), at the start of its behaviour turn and after
/// its movement: while gripped its order-level destination stays the
/// lifter, and a destination removed by any rule ends the grip with the
/// unit set down; once the grip has ended a destination written without
/// the assignment is replaced by none.
// spec: S-0607 claim 23 (revision 7, arrival: the teleporting jump removes the destination through S-0207 claim 20, so the grip ends on the tick of the jump, the unit set down at its landing point; the jump itself has already cleared the lifter's target, S-0207 claim 20 e. A landing point equal to the unit's own position removes it at the first movement update without a jump, and S-0207 claim 26's guard row removes a driving or ship stand-in's at an exact point: the grip ends the same way and the lifter keeps its target)
pub fn enforce(w: &mut World, id: ObjectId) {
    if stand_in(w, id).is_none() {
        return;
    }
    let Some(o) = w.obj(id) else { return };
    let dest = o.dest;
    if o.lift.grip_without_dest && lifter_of(w, id).is_some_and(|l| w.obj(l).is_some()) {
        if dest.is_some()
            && let Some(ob) = w.obj_mut(id)
        {
            // spec: S-0607 claims 9 and 23 (revision 7: a unit whose teleporting stand-in found no landing cell at the seize keeps no destination and never jumps; a seized unit ignores every new destination)
            ob.dest = None;
            ob.path.steps.clear();
        }
        return;
    }
    match lifter_of(w, id) {
        Some(l) if w.obj(l).is_some() => match dest {
            None => {
                // spec: S-0607 claims 9, 12 and 23 (a gripped unit given no destination, here when the stand-in's own rules remove it through the ordinary destination-setting rule: the grip ends without a drop and the unit is set down; the lifter's target is not cleared)
                upd(w, l, |s| {
                    s.gripped = None;
                    s.grip_dest = None;
                });
                upd(w, id, |s| s.lifter = None);
                set_down_here(w, id);
            }
            Some(t) if t != Target::Object(l) => {
                if let Some(ob) = w.obj_mut(id) {
                    // spec: S-0607 claims 9 and 23 (a seized unit ignores every new destination: the lifter stays its destination)
                    ob.dest = Some(Target::Object(l));
                }
            }
            Some(_) => {}
        },
        _ => {
            if dest.is_some()
                && let Some(ob) = w.obj_mut(id)
            {
                // spec: S-0607 claim 9 (a seized unit ignores every new destination)
                ob.dest = None;
                ob.path.steps.clear();
            }
        }
    }
}

/// A unit is given no destination by an assignment (S-0607 claims 9, 12
/// and 23): for a unit gripped with a stand-in that had a destination that
/// is claim 12's event. A gripped unit with none (its teleporting stand-in
/// found no landing cell at the seize, or part e's clearing) gets none
/// again: the assignment is redundant and its grip holds.
pub fn given_no_destination(w: &mut World, id: ObjectId) {
    // spec: S-0607 claim 12 (the row for a gripped unit given no destination, claim 9) and claim 9 (revision 33: a gripped vehicle that has no destination gets none again from such a rule; the assignment leaves its destination unchanged, so it is not the exception and the grip holds, whatever its stand-in) and claim 23 (the grip with no destination from the seize lasts until another event of claim 12)
    enforce(w, id);
}

/// Whether a seized unit refuses a destination given to it (S-0607 claim
/// 9, `econ::assign_destination`): a non-empty one is ignored; an empty
/// one is let through, so a gripped unit that had a destination is set down
/// by [`steer`]. S-0502 claim 28's balloon-hover retention is the callers'
/// (`jump::clear_rejected`), tested before this (claim 9, revision 33;
/// S-0502 claim 28, revision 21: the rule applies to a seized vehicle too,
/// before these refusals).
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

/// Claim 20's restoration test at a fall-back to rest (S-0215 claim 13 row
/// 1). Returns whether the unit was restored.
pub fn restore_at_rest(w: &mut World, id: ObjectId) -> bool {
    restoration_test(w, id, false)
}

/// Claim 20's restoration test at the end of each of the unit's turns
/// (revision 14; `tick::object_turn`): after the point at which it may
/// process its behaviour, whether or not it did, and after its movement,
/// whatever its behaviour and house. A turn that ends early (a lost turn, a
/// dead unit, being erased, a warp-only update) has no test.
pub fn end_of_turn(w: &mut World, id: ObjectId) {
    if seized(w, id) {
        restoration_test(w, id, true);
    }
}

/// The restoration test (claim 20; claim 24 for another stand-in): a seized
/// unit whose jumpjet movement is not *moving* and that is set down gets its
/// own locomotor back. With the no-cell switch off, a set-down unit hovering
/// with no movement point and no commitment, by any route, is not tested:
/// at the end of a turn it repeats the stop's search instead, at a fall-back
/// to rest nothing happens. Returns whether it was restored.
fn restoration_test(w: &mut World, id: ObjectId, turn_end: bool) -> bool {
    let Some(l) = st(w, id).cloned() else {
        return false;
    };
    if !l.seized {
        return false;
    }
    // spec: S-0607 claim 20 (revision 14, units being erased: not restored while being erased or in the leaving stage of a chrono move, S-0215 claim 2, at any test; design: the same for a landed aircraft being erased, which the spec leaves open)
    if crate::superweapons::being_erased(w, id) {
        return false;
    }
    if let Some(k) = l.stand_in {
        // spec: S-0607 claim 24 and quirks.lift_standin_stays_seized (on: a unit seized with the driving, walking, ship or teleporting stand-in is never restored, and one seized with the drop-pod stand-in stays seized)
        if w.rules().quirks.lift_standin_stays_seized {
            return false;
        }
        // spec: S-0607 claim 24 (quirk off: claim 20 applies to every stand-in, at any restoration test, with that stand-in's own moving test in place of the jumpjet's)
        if stand_in_moving(w, id, k) || !l.set_down || l.lifter.is_some() {
            return false;
        }
        restore(w, id);
        return true;
    }
    if l.set_down
        && l.lifter.is_none()
        && !w.rules().quirks.jumpjet_no_cell_self_destruct
        && crate::jump::hovering_uncommitted(w, id)
    {
        // spec: S-0607 claim 20 (revision 14, the hovering set-down unit, design: a seized set-down unit hovering with no movement point and no movement commitment after S-0502 claim 17's no-cell outcome with quirks.jumpjet_no_cell_self_destruct off, whatever route brought it there, is not tested for restoration, neither at the end of a turn nor at a fall-back to rest; at the end of each of its turns it repeats the stop's search instead) and S-0607 Changes for other specs (S-0502 Edge cases: the retry at the end of each turn, not at a fall-back to rest)
        if turn_end {
            crate::jump::lift_retry_stop(w, id);
        }
        return false;
    }
    // spec: S-0607 claim 20 (its jumpjet movement is not moving, S-0502's moving, not moving now; and it is set down)
    if crate::jump::moving(w, id) || !l.set_down || l.lifter.is_some() {
        return false;
    }
    restore(w, id);
    true
}

/// Gives a seized unit its own locomotor back (S-0607 claim 20; claim 24
/// for another stand-in).
fn restore(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else { return };
    if o.lift.stand_in.is_some() {
        // spec: S-0607 claim 24 (quirks.lift_standin_stays_seized off: claim 20 applies to every stand-in; the stand-in ends and its own locomotor returns)
        end_stand_in(w, id);
    } else {
        let jumpjet_type = w
            .rules()
            .unit(o.kind, o.type_index)
            .is_some_and(|u| u.locomotor == Locomotor::Jump);
        crate::jump::lift_restore(w, id, jumpjet_type);
        if !jumpjet_type {
            own_marks_back(w, id);
        }
    }
    // spec: S-0607 claim 20 (restoration clears seized, set down and the dropper) and claim 24 (for every stand-in when the quirk is off)
    upd(w, id, |l| {
        l.seized = false;
        l.set_down = false;
        l.dropped = false;
        l.detour = false;
        l.dropper = None;
        l.dropper_house = None;
        l.rate = None;
        l.stand_in = None;
        l.own = None;
        l.grip_without_dest = false;
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

/// The own locomotor's marks at the unit's cell when it returns (S-0215
/// claim 13: restoration retains the suspended movement state; S-0209
/// claim 1).
fn own_marks_back(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else { return };
    if o.jump.is_some() {
        // A jumpjet type kept its own jumpjet locomotor and its marks (`suspend_own`).
        return;
    }
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

/// A stand-in other than the drop-pod's ends (S-0607 claim 24 with the
/// quirk off): it gives up its marks and the unit's own movement states
/// return, with their marks at its cell.
fn end_stand_in(w: &mut World, id: ObjectId) {
    let Some(own) = w.obj_mut(id).and_then(|o| o.lift.own.take()) else {
        // The drop-pod stand-in, or a landed aircraft with the driving, walking or ship stand-in: nothing was installed.
        if w.obj(id).is_some_and(|o| o.kind == Kind::Aircraft) {
            // spec: S-0607 claim 24 (quirk off: its own locomotor returns) and S-0209 claim 1 (an aircraft on the ground sets its object-present mark)
            own_marks_back(w, id);
        }
        return;
    };
    if w.obj(id).is_some_and(|o| o.kind == Kind::Aircraft) {
        // spec: S-0209 claim 1 and S-0207 claim 20 (a) and (e) (the teleporting stand-in of a landed aircraft clears the object-present records it holds)
        for (c, layer) in crate::teleport::removal_marks(w, id) {
            if let Some(rec) = w.cells.occ_mut(c, layer) {
                rec.others = rec.others.saturating_sub(1);
            }
        }
    } else {
        // spec: S-0209 claim 1 (the stand-in clears the marks it holds)
        crate::drive::release_claims(w, id);
    }
    let jumpjet = own.jump.is_some();
    if let Some(o) = w.obj_mut(id) {
        let own = *own;
        o.drive = own.drive;
        o.walk = own.walk;
        o.hover = own.hover;
        o.teleport = own.teleport;
        o.jump = own.jump;
        o.econ.temporary_drive = own.temporary_drive;
    }
    if jumpjet {
        // spec: S-0502 claim 4 (a jumpjet type's own ground mark at its present position)
        crate::jump::retake_marks(w, id);
    } else {
        own_marks_back(w, id);
    }
}

/// The stand-in's own moving test (S-0607 claim 24, quirk off: in place of
/// the jumpjet's *moving* of claim 20).
fn stand_in_moving(w: &World, id: ObjectId, k: StandIn) -> bool {
    match k {
        // spec: S-0607 claim 24 and S-0207 claim 27 (the driving or ship locomotor's movement report: a locomotor destination point, or a heading point horizontally different from its position)
        StandIn::Drive | StandIn::Ship => crate::drive::movement_report(w, id, false),
        // spec: S-0607 claim 24 and S-0207 claim 20 (d) (the teleporting locomotor is moving while a jump to a landing point other than its position is pending)
        StandIn::Teleport => w.obj(id).is_some_and(|o| {
            o.teleport
                .as_ref()
                .and_then(|t| t.landing)
                .is_some_and(|l| l != o.pos)
        }),
        // spec: S-0607 claim 23 (walking: our engine never moves the unit; drop-pod: the stand-in has already ended)
        StandIn::Walk | StandIn::DropPod => false,
    }
}

/// The occupancy records a vehicle or aircraft seized with the walking or
/// the teleporting stand-in holds, where they differ from the ordinary
/// teleporting removal rule (S-0607 claim 23, S-0209 claim 1): a walking
/// stand-in holds its own cell's mark only with
/// `quirks.lift_standin_unmarked_cell` off, since it never moves; a
/// teleporting stand-in that has fixed no landing point holds none. `None`:
/// the ordinary rule applies.
pub fn stand_in_marks(w: &World, id: ObjectId) -> Option<Vec<(CellXY, Layer)>> {
    let o = w.obj(id)?;
    match moving_stand_in(w, id)? {
        StandIn::Walk if o.kind == Kind::Vehicle => {
            // spec: S-0607 claim 23 and quirks.lift_standin_unmarked_cell (on: a walking stand-in never marks a cell; off: it kept its own cell's mark at the seize)
            Some(if w.rules().quirks.lift_standin_unmarked_cell {
                Vec::new()
            } else {
                vec![(o.pos.cell(), o.layer)]
            })
        }
        StandIn::Teleport => {
            let t = o.teleport.as_ref()?;
            // spec: S-0607 claims 5 and 23 (the seize gave up the unit's own mark; a teleporting stand-in with no landing point fixed, its search having found no cell, marks none)
            (t.mark.is_none() && t.held.is_none() && t.deck_mark.is_none()).then(Vec::new)
        }
        _ => None,
    }
}

/// Whether a vehicle seized with the walking or the teleporting stand-in
/// is *between steps* (S-0209 Terms; S-0607 claim 23, revision 7: before
/// its first step a seized unit is between steps). Neither stand-in ever
/// advances along a track, so it never leaves the state.
pub fn stand_in_between_steps(w: &World, id: ObjectId) -> bool {
    // spec: S-0607 claim 23 (before its first step the seized unit is between steps) and S-0209 Terms (a vehicle that has not advanced along a track since it was created or put down elsewhere, a teleport included, is between steps; vehicles that move by jumping are not known ever to leave the state)
    matches!(
        moving_stand_in(w, id),
        Some(StandIn::Walk | StandIn::Teleport)
    )
}

/// The movement update of a unit seized with a stand-in (S-0607 claim 23):
/// the stand-in's ordinary rules move it, then a destination they removed
/// ends the grip. Returns whether it moved.
pub fn stand_in_tick(w: &mut World, id: ObjectId, k: StandIn, held: bool) -> bool {
    let Some(kind) = w.obj(id).map(|o| o.kind) else {
        return false;
    };
    if kind == Kind::Aircraft && k == StandIn::Teleport {
        // spec: S-0607 claim 23 (revision 7: a landed aircraft with the teleporting stand-in jumps under S-0207 claim 20, which covers aircraft, and its grip ends at the jump)
        let moved = crate::teleport::tick(w, id);
        enforce(w, id);
        return moved;
    }
    if kind != Kind::Vehicle {
        // spec: S-0607 claims 3 and 23 (revision 7: a landed aircraft with the driving, walking or ship stand-in, design: our engine keeps it where it stands; its grip ends only by a lifter-side event of claim 12)
        return false;
    }
    let moved = match k {
        // spec: S-0607 claim 23 (driving: S-0207) and S-0614 claim 8 (a held unit does not move under the driving locomotor)
        StandIn::Drive => {
            let moved = !held && crate::drive::tick(w, id);
            // spec: S-0207 claim 23 (a drive vehicle's wake, created right after its drive update)
            crate::drive::driving_wake(w, id);
            moved
        }
        // spec: S-0607 claim 23 (ship: S-0504's ship locomotor) and S-0614 claim 8
        StandIn::Ship => !held && crate::naval::tick(w, id),
        // spec: S-0607 claim 23 (teleporting: S-0207 claim 20)
        StandIn::Teleport => crate::teleport::tick(w, id),
        // spec: S-0607 claim 23 (walking, design: the unit does not move or turn while it is seized; drop-pod: ended at the seize)
        StandIn::Walk | StandIn::DropPod => false,
    };
    // spec: S-0607 claim 23 (the grip ends when the stand-in's own rules remove the unit's order-level destination, for example S-0207 claim 25's give-up row)
    enforce(w, id);
    moved
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
    // spec: S-0607 claim 25 (the picture's shift direction comes from the shot's firing and impact points)
    w.effects.push(crate::combat::effects::Effect::Wave {
        id,
        from: fire_point,
        to: impact,
    });
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
