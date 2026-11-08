//! Mind control (S-0601): control records and capacity (claims 1 and 2),
//! the takeover test, firing refusal and detonation (claims 3 to 5), the
//! takeover and its capacity-1 replacement (claims 6 and 7), the fate
//! decision of the neutral and special houses (claim 8), release and
//! release-all (claims 9 and 10), the owner-change and removal rules
//! (claims 11 and 12), permanent control and the original-owner query
//! (claims 13 and 14), deploying (claim 15), overload pulses (claim 16) and
//! the read-only state of link lines and control rings (claims 17 and 18).
//!
//! State is world-level ([`MindWorld`]): a record per controller, a record
//! per controlled victim and the set of permanently controlled objects. It
//! is saved and hashed. Every exit path of an object cleans it:
//! [`killed`] (the killing hit), [`removing`] (removal from the game),
//! [`building_turn`] (a building not in operation), [`erasing_hit`],
//! [`trigger_owner_change`] and [`boarding`].
//!
//! Seams:
//! - S-0608 (drain links): [`drain_link_formed`] releases every victim of
//!   a drained controlling building, called by `drain` when a link forms.
//! - S-0612 (psychic domination): [`dominate`] sets the permanent state of
//!   claim 13 for each object the sweep takes over (`crate::domination`).
//!   Its own-house switch `quirks.dominator_marks_own_objects` is S-0612's
//!   and is wired there.
//! - The grinder fate of claim 8 ([`grinder_fate`]) is S-0603 claim 14's
//!   search (`structures::grinder`); none found falls back to *hunt*, as
//!   claim 8 says for a failed assignment. The bio-reactor fate
//!   ([`bio_reactor_fate`]) is S-0605 claim 16's search; accepted absorption
//!   (S-0605 claim 7) calls [`release_one`].
//! - S-0612 (the Psychic Dominator): its takeover of a bunkered vehicle
//!   keeps the bunker link (S-0606 claim 15), which an owner change never
//!   ends here.
//! - Teams: our engine has no scripted teams, so the team fate falls back to
//!   *hunt* and leaving a team is nothing (claim 6).
//! - Particle systems: the overload sparks take their ten placement draws
//!   (F3); the systems themselves are not created. The rocking rate (F4) is
//!   a presentation input of S-0106 claim 9: its draw is taken here and
//!   reported (`Effect::OverloadRock`), and the view sets the rate.

use std::collections::{BTreeMap, BTreeSet};

use glasnost_num::{F64, Rounding};
use serde::{Deserialize, Serialize};

use crate::combat::damage::Hit;
use crate::combat::effects::Effect;
use crate::combat::veterancy::type_combat;
use crate::econ::TypeRef;
use crate::rules::{Behaviour, Kind};
use crate::world::{HouseId, ObjectId, Pos, Target, World};

/// The alternate link anchors that link lines cycle through (F5).
pub const ANCHORS: usize = 5; // spec: S-0601 Constants (alternate anchors cycled by link lines: 5)

/// A new record's overload countdown (claim 1).
pub const INITIAL_COUNTDOWN: i32 = 30; // spec: S-0601 Constants (initial overload countdown 30 ticks)

/// The pip highlight after a damaging pulse (claim 16).
pub const PULSE_HIGHLIGHT: i32 = 10; // spec: S-0601 Constants (pip highlight after a damaging pulse 10 ticks)

/// Spark systems per damaging pulse (claim 16, F3).
const SPARKS: usize = 5; // spec: S-0601 Constants (sparks per damaging pulse 5)

/// The spark displacement range on each horizontal axis (F3).
const SPARK_SPREAD: i32 = 200; // spec: S-0601 Constants (spark displacement −200 to 200 leptons)

/// The spark height (F3).
const SPARK_HEIGHT: i32 = 100; // spec: S-0601 Constants (spark height 100 leptons)

/// The rocking draw's range (F4).
const ROCK_DRAW: (i32, i32) = (0, 100); // spec: S-0601 F4 (one draw r in [0, 100], inclusive)

/// The fate draw's range (claim 8).
const FATE_DRAW: (i32, i32) = (1, 100); // spec: S-0601 Constants (fate draw range 1 to 100)

/// A controller's record (claim 1).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Controller {
    /// The control capacity: the primary weapon's damage at creation.
    pub capacity: i32,
    /// Unlimited: the primary weapon's unlimited-control flag.
    pub unlimited: bool,
    /// The victims in the order taken.
    pub victims: Vec<ObjectId>,
    /// The overload countdown (claim 16).
    pub countdown: i32,
    /// The pip highlight (claim 16; S-0111 claim 13 reads it).
    pub highlight: i32,
    /// The overload-sound mark (claim 16).
    pub sound_mark: bool,
}

/// A controlled object's record (claim 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Victim {
    /// Its controller.
    pub controller: ObjectId,
    /// The house that owned it at the takeover.
    pub original_owner: Option<HouseId>,
    /// The tick at which its link-line timer ends (claim 17): the timer
    /// runs while the tick is below it.
    pub line_until: i32,
}

/// The mind-control state of the world.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MindWorld {
    /// Controller records by object.
    pub controllers: BTreeMap<ObjectId, Controller>,
    /// Victim records by object.
    pub victims: BTreeMap<ObjectId, Victim>,
    /// The permanently controlled objects (claim 13).
    pub permanent: BTreeSet<ObjectId>,
}

/// A sound of this spec (presentation).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MindSound {
    /// The take-over sound (claim 5): heard when the local player owns the
    /// controller or owned the target before the takeover (`houses`).
    Takeover,
    /// The release sound (claim 9): the victim type's, else the rules-wide
    /// one, for every player.
    Cleared,
    /// The overload sound (claim 16), for every player.
    Overload,
}

// ---------------------------------------------------------------------------
// Queries.

/// Whether a warhead-bearing weapon is a takeover weapon (Terms).
pub fn takeover_weapon(w: &World, weapon: crate::combat::data::WeaponId) -> bool {
    w.rules()
        .combat
        .warhead_of(weapon)
        .is_some_and(|h| h.mind_control)
}

/// The controller record of an object.
pub fn controller(w: &World, id: ObjectId) -> Option<&Controller> {
    w.mind.controllers.get(&id)
}

/// Whether an object is controlled (Terms), and by whom.
pub fn controlled_by(w: &World, id: ObjectId) -> Option<ObjectId> {
    w.mind.victims.get(&id).map(|v| v.controller)
}

/// Whether an object is permanently controlled (claim 13).
pub fn permanent(w: &World, id: ObjectId) -> bool {
    w.mind.permanent.contains(&id)
}

/// Mind-controlled (Terms): controlled or permanently controlled. The
/// consumers of S-0214, S-0507 claim 4, S-0508 claim 1 and S-0309 read
/// this.
pub fn mind_controlled(w: &World, id: ObjectId) -> bool {
    // spec: S-0601 claim 13 and Terms (mind-controlled: controlled or permanently controlled)
    w.mind.victims.contains_key(&id) || permanent(w, id)
}

/// A controller that controls at least one object now (S-0507 claim 4).
pub fn controls_objects(w: &World, id: ObjectId) -> bool {
    controller(w, id).is_some_and(|c| !c.victims.is_empty())
}

/// At capacity (claim 2, F1): not unlimited and holding at least as many
/// victims as its capacity. S-0309 claims 2 and 18 read it.
pub fn at_capacity(w: &World, id: ObjectId) -> bool {
    // spec: S-0601 claim 2 and F1 (at capacity ⇔ ¬unlimited ∧ N ≥ capacity)
    controller(w, id)
        .is_some_and(|c| !c.unlimited && c.victims.len() as i64 >= i64::from(c.capacity))
}

/// The original-owner query (claim 14): a controlled object's recorded
/// original owner; any other object's current owner (a trigger's house
/// takeover is campaign-only and not modelled).
pub fn original_owner(w: &World, id: ObjectId) -> Option<HouseId> {
    if let Some(v) = w.mind.victims.get(&id) {
        // spec: S-0601 claim 14 (controlled: the recorded original owner)
        return v.original_owner;
    }
    // spec: S-0601 claim 14 (otherwise, including permanently controlled: its current owner)
    w.obj(id).and_then(|o| o.owner)
}

/// Redirects a controlled object's original owner (S-0414 claim 5: a
/// defeated house's controlled objects pass to the first civilian-side
/// house).
pub fn redirect_original_owner(w: &mut World, id: ObjectId, to: Option<HouseId>) {
    if let Some(v) = w.mind.victims.get_mut(&id) {
        v.original_owner = to;
    }
}

/// The bunker row of claim 3: a vehicle that sits in a tank bunker
/// (S-0606 Terms, bunker link).
pub fn bunkered(w: &World, id: ObjectId) -> bool {
    // spec: S-0601 claim 3 (a vehicle that sits in a tank bunker)
    w.obj(id).is_some_and(|o| o.kind == Kind::Vehicle) && crate::structures::bunker::bunkered(w, id)
}

/// The takeover test (claim 3): controller `c` can take object `t` now.
pub fn can_take(w: &World, c: ObjectId, t: ObjectId) -> bool {
    let (Some(co), Some(to)) = (w.obj(c), w.obj(t)) else {
        return false;
    };
    // spec: S-0601 claim 4 (a firer with no control record: never legal, a departure)
    let Some(rec) = controller(w, c) else {
        return false;
    };
    // spec: S-0601 claim 3 (an infantry unit, vehicle, aircraft or building)
    if !matches!(
        to.kind,
        Kind::Infantry | Kind::Vehicle | Kind::Aircraft | Kind::Building
    ) {
        return false;
    }
    // spec: S-0601 claim 3 (T's owner is not C's owner; an ally may be taken)
    if to.owner == co.owner {
        return false;
    }
    // spec: S-0601 claim 3 and S-0610 claim 1 (T's type has no psionics immunity)
    if type_combat(w, t).is_some_and(|tc| tc.immune_psionics) {
        return false;
    }
    // spec: S-0601 claim 3 (not a vehicle in a tank bunker)
    if bunkered(w, t) {
        return false;
    }
    // spec: S-0601 claim 3 (not mind-controlled; a trigger's house takeover is campaign-only and not modelled)
    if mind_controlled(w, t) {
        return false;
    }
    // spec: S-0601 claim 3 and S-0513 claim 27 (not under protection)
    if crate::superweapons::protected(w, t) {
        return false;
    }
    // spec: S-0601 claim 3 and F1 (unlimited ∨ N < capacity ∨ capacity = 1)
    if !(rec.unlimited || (rec.victims.len() as i64) < i64::from(rec.capacity) || rec.capacity == 1)
    {
        return false;
    }
    // spec: S-0601 claim 3 (T's current behaviour is neither construction nor selling)
    !matches!(
        to.beh.current,
        Some(Behaviour::Construction | Behaviour::Selling)
    )
}

/// The firing check's takeover refusal (claim 4, at rank 9 of S-0301 claim
/// 9): a takeover weapon whose takeover test fails, or aimed at a cell, is
/// illegal.
pub fn firing_illegal(
    w: &World,
    firer: ObjectId,
    weapon: crate::combat::data::WeaponId,
    target: Target,
) -> bool {
    if !takeover_weapon(w, weapon) {
        return false;
    }
    match target {
        // spec: S-0601 claim 4 (illegal when the takeover test fails)
        Target::Object(x) => !can_take(w, firer, x),
        // spec: S-0601 claim 3 (a cell is not a takeover target)
        Target::Cell(_) => true,
    }
}

// ---------------------------------------------------------------------------
// Creation (claim 1).

/// Gives a newly created object its control record when its type's primary
/// weapon, as the type gives it, has a takeover warhead (claim 1).
pub fn created(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    // spec: S-0601 claim 1 (an infantry unit, vehicle, aircraft or building)
    if !matches!(
        o.kind,
        Kind::Infantry | Kind::Vehicle | Kind::Aircraft | Kind::Building
    ) {
        return;
    }
    // spec: S-0601 claim 1 (only the type's primary slot, without any veterancy upgrade)
    let Some(wp) = type_combat(w, id).and_then(|tc| tc.slots[0]) else {
        return;
    };
    if !takeover_weapon(w, wp) {
        return;
    }
    let wr = w.rules().combat.weapon(wp);
    let rec = Controller {
        // spec: S-0601 claim 1 and F1 (capacity: the primary weapon's damage, fixed for the object's lifetime)
        capacity: wr.damage,
        // spec: S-0601 claim 1 (unlimited: the primary weapon's unlimited-control flag)
        unlimited: wr.infinite_mind_control,
        victims: Vec::new(),
        // spec: S-0601 claim 1 (a new record: no victims, countdown 30, cleared highlight and sound mark)
        countdown: INITIAL_COUNTDOWN,
        highlight: 0,
        sound_mark: false,
    };
    w.mind.controllers.insert(id, rec);
}

// ---------------------------------------------------------------------------
// Owner changes.

/// Whether an object is a simple-deployer vehicle whose current behaviour
/// is *unload* (claims 6 and 9).
pub(crate) fn deploying_in_place(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    // spec: S-0601 claims 6 and 9 and S-0517 claim 1 (a vehicle that deploys in place with unload current)
    o.kind == Kind::Vehicle
        && o.beh.current == Some(Behaviour::Unload)
        && w.rules()
            .unit(o.kind, o.type_index)
            .is_some_and(|u| u.simple_deployer)
}

/// Changes an object's owner with every change-of-owner consumer (claim
/// 6): a building through S-0509's ownership change, which requests the
/// capture or loss announcement (claims 6 and 9), a mobile object through `econ::events::owner_changed` and
/// S-0506 claim 13's spawner reset. No change when the owner is the same.
pub fn set_owner(w: &mut World, id: ObjectId, new: Option<HouseId>) {
    set_owner_by(w, id, new, None);
}

/// As [`set_owner`], made in the turn of `actor` (S-0108 claim 26: S-0601
/// claims 6 and 9 make a building's takeover and release in the controller's turn).
pub fn set_owner_by(w: &mut World, id: ObjectId, new: Option<HouseId>, actor: Option<ObjectId>) {
    let Some(o) = w.obj(id) else { return };
    if o.owner == new {
        return;
    }
    if o.kind == Kind::Building {
        // spec: S-0601 claims 6 and 9 and S-0509 claim 12 (every consumer of a change of owner; the takeover and the release of a building request the capture or loss announcement, at the owner change itself, after the trigger events of claim 5)
        crate::structures::ownership::change_by(
            w,
            id,
            new,
            crate::structures::ownership::Cause::MindControl,
            actor,
        );
        return;
    }
    let old = o.owner;
    let spawner = o.spawn.is_some();
    if let Some(ob) = w.obj_mut(id) {
        ob.owner = new;
    }
    // spec: S-0601 claim 6 (S-0414 claims 4, 5 and 21; S-0311 claim 13: the object keeps its experience; S-0516 claim 34)
    crate::econ::events::owner_changed(w, id, old);
    if spawner {
        // spec: S-0601 claim 6 and S-0506 claim 13 (a spawner that changes house)
        crate::spawn::owner_changed(w, id);
    }
}

/// The parts of claim 6 that do not depend on the owner (claims 6, 8 and
/// 9): leaving a team (no teams in our engine), the temporal let-go and the
/// open-topped passengers' targets.
fn owner_independent(w: &mut World, id: ObjectId) {
    // spec: S-0601 claim 6 and S-0514 claim 14 (a temporal attacker lets go of its target)
    crate::temporal::control_changed(w, id);
    // spec: S-0601 claim 6 (an open-topped object clears the targets of its passengers)
    crate::transport::clear_passenger_targets(w, id);
}

/// The guard reset of a takeover (claim 6, S-0215 claim 9).
fn guard_reset(w: &mut World, id: ObjectId) {
    // spec: S-0601 claim 6 (a simple-deployer vehicle with unload current keeps its behaviour and gets no guard reset)
    if deploying_in_place(w, id) {
        return;
    }
    // spec: S-0601 claim 6 and S-0215 claim 9 (target, destination and guard spot cleared, guard forced; wait refuses it)
    crate::econ::power::guard_reset(w, id);
}

// ---------------------------------------------------------------------------
// Takeover (claims 5 to 8).

/// A takeover projectile's detonation (claim 5): it replaces damage and the
/// other special hits (`combat::projectile`). Only the projectile's target
/// can change owner, and only an infantry unit, vehicle, aircraft or
/// building; a firer gone or without a record does nothing.
pub fn detonation(w: &mut World, attacker: Option<ObjectId>, target: Option<Target>) {
    // spec: S-0601 claim 5 (a shot aimed at a cell takes nothing)
    let Some(Target::Object(t)) = target else {
        return;
    };
    let Some(to) = w.obj(t) else { return };
    if !matches!(
        to.kind,
        Kind::Infantry | Kind::Vehicle | Kind::Aircraft | Kind::Building
    ) {
        return;
    }
    // spec: S-0601 claim 5 (the firer no longer exists or has no control record: no takeover and no attack-tag notification)
    let Some(c) = attacker.filter(|&a| w.obj(a).is_some() && controller(w, a).is_some()) else {
        return;
    };
    if to.health > 0 && w.triggers.objects.contains_key(&t) {
        // spec: S-0601 claim 5 (a live target's tag hears attack events 6 and then 44, both before the takeover test of the landing and whether or not the takeover then succeeds; both name the firer as causing object)
        for code in [6, 44] {
            // spec: S-0601 claim 5 and S-0011 claim 8 (every action of the event-6 raise has completed before event 44 is judged: the tag is looked up again, so an action that changed the target's tag or removed the target is seen)
            // spec: S-0601 claim 5 and Edge cases (tag removed by the first raise: event 44 is raised only if the target still carries a tag, on that tag, whichever tag it is; a departure with no switch, the original faults there, lead decision on Q-1150)
            let Some(&tag) = w.triggers.objects.get(&t) else {
                break;
            };
            crate::triggers::raise(
                w,
                tag,
                crate::triggers::Raise {
                    code: Some(code),
                    causer: Some(c),
                    carrier: Some(crate::triggers::Carrier::Object(t)),
                },
            );
        }
    }
    // spec: S-0601 claim 5 (the takeover test is made again at landing)
    if !can_take(w, c, t) {
        return;
    }
    let before = w.obj(t).and_then(|o| o.owner);
    if take(w, c, t) {
        let pos = w.obj(t).map(|o| o.pos);
        let owner = w.obj(c).and_then(|o| o.owner);
        // spec: S-0601 claim 5 (on success the take-over sound plays at the target when the local player owns the controller or owned the target before)
        w.effects.push(Effect::Mind {
            sound: MindSound::Takeover,
            of: None,
            houses: [owner, before],
            pos,
        });
    }
}

/// The takeover of `t` by `c` (claims 6 and 7). Returns whether `t` is now
/// controlled by `c`.
pub fn take(w: &mut World, c: ObjectId, t: ObjectId) -> bool {
    let Some(rec) = controller(w, c) else {
        return false;
    };
    if rec.capacity == 1 {
        // spec: S-0601 claims 6 and 7 (capacity exactly 1: the current victim is released first, with the full effects of claim 9)
        release_all(w, c);
    }
    let Some(new) = w.obj(c).map(|o| o.owner) else {
        return false;
    };
    // spec: S-0601 claim 6 (the original owner is its owner at this moment)
    let original = w.obj(t).and_then(|o| o.owner);
    // spec: S-0601 claim 6 (T changes owner to C's house, with every consumer of a change of owner)
    set_owner_by(w, t, new, Some(c));
    if w.obj(t).is_none_or(|o| o.owner != new) {
        // spec: S-0601 claim 7 (the transfer itself failed: the old victim is already released and the new target is not controlled)
        return false;
    }
    let frames = w.rules().combat.general.mind.line_frames;
    let tick = w.tick;
    // spec: S-0601 claim 6 (T becomes controlled by C and is appended to the end of C's victim list; its link-line timer starts, claim 17)
    w.mind.victims.insert(
        t,
        Victim {
            controller: c,
            original_owner: original,
            line_until: tick.saturating_add(frames),
        },
    );
    if let Some(r) = w.mind.controllers.get_mut(&c) {
        r.victims.push(t);
    }
    if w.obj(t).is_some_and(|o| o.kind != Kind::Building) {
        // spec: S-0309 claim 31 (the takeover's change resets the orders inside it, after claim 30's target drop; a building's change does it in S-0509's ownership change)
        crate::structures::ownership::reset_orders(w, t);
    }
    // spec: S-0601 claim 6 (a guard reset, S-0215 claim 9, after the change); S-0309 claim 31 (it clears the destination and target that a resumption or an aircraft's revert gave, and keeps the attack-move memory)
    guard_reset(w, t);
    // spec: S-0601 claim 6 (leaves its team, temporal let-go, open-topped passengers' targets)
    owner_independent(w, t);
    // spec: S-0601 claim 6 (the fate decision of claim 8 runs)
    fate(w, t, c);
    // The control ring of claim 18 is derived from the record (`ring`).
    true
}

/// The four fates of claim 8, in the order of the tables' weights.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fate {
    /// Join the controller's team.
    Team,
    /// Go to the nearest grinder.
    Grinder,
    /// Go to the nearest bio reactor.
    BioReactor,
    /// Hunt.
    Hunt,
}

/// The grinder fate of claim 8: go to the nearest recycling building of
/// the object's owner, S-0603 claim 14's search. None held: the assignment
/// fails and the fate falls back to *hunt*.
pub fn grinder_fate(w: &mut World, id: ObjectId) -> bool {
    // spec: S-0601 claim 8 (go to the nearest grinder; a failed assignment falls back to hunt) and S-0603 claim 14 (the search)
    crate::structures::grinder::grinder_fate(w, id)
}

/// The bio-reactor fate of claim 8: enter the nearest absorbing building,
/// S-0605 claim 16. None found: the fate fails and falls back to *hunt*.
pub fn bio_reactor_fate(w: &mut World, id: ObjectId) -> bool {
    // spec: S-0601 claim 8 (go to the nearest bio reactor; a failed assignment falls back to hunt) and S-0605 claim 16 (the search, the instruction and claim 3)
    crate::structures::absorb::bio_reactor_fate(w, id)
}

/// The fate table claim 8 selects for controller house `h` and object `v`:
/// low money, then insufficient power, then wounded, otherwise normal.
fn fate_table(w: &World, h: HouseId, v: ObjectId) -> Vec<i32> {
    let m = &w.rules().combat.general.mind;
    let money = crate::econ::production::available_credits(w, h);
    // spec: S-0601 claim 8 (low money first: the controlling house's money below the low-money mark; revision 9 design rule: an absent mark never selects its table, the others keep their places)
    if m.low_money_mark.is_some_and(|mark| money < mark) {
        return m.fate[3].clone();
    }
    // spec: S-0601 claim 8 (then insufficient power of the controlling house, S-0405 Terms)
    if crate::econ::power::low_power(w, h) {
        return m.fate[2].clone();
    }
    // spec: S-0601 claim 8 (then wounded: health ratio strictly below the mark; revision 9 design rule: an absent mark never selects its table, the others keep their places)
    if let Some(mark) = m.wounded_mark
        && wounded_ratio(w, v).is_some_and(|q| q.lt(mark))
    {
        return m.fate[1].clone();
    }
    // spec: S-0601 claim 8 (otherwise normal)
    m.fate[0].clone()
}

/// The health ratio of claim 8: health narrowed to binary32 toward zero,
/// divided by the exact integer maximum health rounded toward zero to a
/// 53-bit significand; not narrowed again.
fn wounded_ratio(w: &World, v: ObjectId) -> Option<F64> {
    let o = w.obj(v)?;
    let max = crate::combat::damage::strength(w, v);
    if max <= 0 {
        return None;
    }
    let rz = Rounding::TowardZero;
    // spec: S-0601 claim 8 and S-0205 claim 5 (health narrowed to binary32 toward zero; the quotient RZ to 53 bits, compared without narrowing)
    let h = F64::from_i64(i64::from(o.health), rz).to_f32(rz).to_f64();
    Some(h.div(F64::from_i64(i64::from(max), rz), rz))
}

/// The fate decision of claim 8 for object `v` after a takeover or release
/// by controller `c`. It runs only for a computer-controlled new owner:
/// in our engine no player house is (glossary, *AI house*), so only the
/// neutral and special houses take it, with its draw.
pub fn fate(w: &mut World, v: ObjectId, c: ObjectId) {
    let Some(owner) = w.obj(v).and_then(|o| o.owner) else {
        return;
    };
    // spec: S-0601 claim 8 (the decision runs only when the new owner is computer-controlled; no player house of our engine is)
    if w.houses.get(usize::from(owner.0)).is_none_or(|h| h.human) {
        return;
    }
    // spec: S-0601 claim 8 (money and power belong to the controller's house, even on release)
    let Some(ch) = w.obj(c).and_then(|o| o.owner) else {
        return;
    };
    let table = fate_table(w, ch, v);
    let half = w.rules().quirks.ranged_draw_half_span;
    // spec: S-0601 claim 8 (one draw from the scenario random stream in 1 to 100, inclusive)
    let r = w.rng.ranged(FATE_DRAW.0, FATE_DRAW.1, half);
    // spec: S-0601 claim 8 (cumulative weights in the order team, grinder, bio reactor, hunt; exact integer addition)
    let mut sum: i64 = 0;
    let mut picked = None;
    // spec: S-0601 claim 8 (four weights: team, grinder, bio reactor, hunt)
    for (i, &wt) in table.iter().take(4).enumerate() {
        sum += i64::from(wt);
        if i64::from(r) <= sum {
            picked = Some([Fate::Team, Fate::Grinder, Fate::BioReactor, Fate::Hunt][i]);
            break;
        }
    }
    let Some(f) = picked else {
        // spec: S-0601 claim 8 (a draw beyond every cumulative weight: nothing)
        return;
    };
    let done = match f {
        // spec: S-0601 claim 8 (a team assignment needs the new house to be the controller's and the controller to have a team; our engine has no teams, so hunt)
        Fate::Team => false,
        Fate::Grinder => grinder_fate(w, v),
        Fate::BioReactor => bio_reactor_fate(w, v),
        Fate::Hunt => false,
    };
    if !done {
        // spec: S-0601 claim 8 (hunt, and the fall-back of a failed team, grinder or bio-reactor assignment)
        if let Some(ob) = w.obj_mut(v) {
            ob.beh.queue(Some(Behaviour::Hunt));
        }
    }
}

// ---------------------------------------------------------------------------
// Ending control (claims 9 to 12).

/// Releases victim `v` from controller `c` (claim 9). The controller may
/// have changed owner (claim 11): money and power for the fate decision
/// are its house's.
pub fn release(w: &mut World, c: ObjectId, v: ObjectId) {
    let Some(rec) = w.mind.victims.get(&v).copied() else {
        return;
    };
    if rec.controller != c {
        return;
    }
    // The ring goes with the record (`ring`); it is removed first in the
    // original, which only presentation sees.
    let (of, pos) = match w.obj(v) {
        Some(o) => (
            Some(TypeRef {
                kind: o.kind,
                index: o.type_index,
            }),
            Some(o.pos),
        ),
        None => (None, None),
    };
    // spec: S-0601 claim 9 (the release sound at V: the type's when set, else the rules-wide one)
    w.effects.push(Effect::Mind {
        sound: MindSound::Cleared,
        of,
        houses: [None, None],
        pos,
    });
    // The record ends before the owner change so that no consumer of the
    // change sees V as controlled.
    w.mind.victims.remove(&v);
    if let Some(r) = w.mind.controllers.get_mut(&c) {
        // spec: S-0601 claim 9 (V leaves C's victim list; later victims close the gap, F5)
        r.victims.retain(|&x| x != v);
    }
    let changes = w.obj(v).is_some_and(|o| o.owner != rec.original_owner);
    // spec: S-0601 claim 9 (V changes owner back to its recorded original owner, with every consumer)
    set_owner_by(w, v, rec.original_owner, Some(c));
    if changes && w.obj(v).is_some_and(|o| o.kind != Kind::Building) {
        // spec: S-0601 claim 9 and S-0309 claim 31 (the ownership change clears target and destination and requests guard: the orders reset of claim 31; a building's change does it in S-0509's ownership change)
        crate::structures::ownership::reset_orders(w, v);
    }
    // spec: S-0601 claim 9 (the owner-independent effects of claim 6 also apply)
    owner_independent(w, v);
    // spec: S-0601 claim 9 (for that owner the fate decision of claim 8 runs)
    fate(w, v, c);
}

/// Releases object `v` from whatever controller holds it (S-0507 claim 6,
/// S-0605 claim 7).
pub fn release_one(w: &mut World, v: ObjectId) {
    if let Some(c) = controlled_by(w, v) {
        release(w, c, v);
    }
}

/// Releases every victim of controller `c`, the most recently taken first
/// (claim 10).
pub fn release_all(w: &mut World, c: ObjectId) {
    loop {
        // spec: S-0601 claim 10 (all victims at once, the most recently taken first)
        let Some(v) = controller(w, c).and_then(|r| r.victims.last().copied()) else {
            return;
        };
        let before = controller(w, c).map_or(0, |r| r.victims.len());
        release(w, c, v);
        if controller(w, c).map_or(0, |r| r.victims.len()) >= before {
            // A victim without a record: drop it so the loop ends.
            if let Some(r) = w.mind.controllers.get_mut(&c) {
                r.victims.pop();
            }
        }
    }
}

/// A victim leaves its controller's list silently (claim 12): no owner
/// change, no sound and no fate decision.
pub fn drop_silently(w: &mut World, v: ObjectId) {
    let Some(rec) = w.mind.victims.remove(&v) else {
        return;
    };
    if let Some(r) = w.mind.controllers.get_mut(&rec.controller) {
        // spec: S-0601 claim 12 (it silently leaves its controller's list)
        r.victims.retain(|&x| x != v);
    }
}

/// An object receives its killing hit (claim 10, S-0302 claim 14): a
/// controller releases every victim; with `quirks.dying_victim_holds_slot`
/// off a victim leaves its controller's list now.
pub fn killed(w: &mut World, id: ObjectId) {
    // spec: S-0601 claim 10 (the controller receives its killing hit, whatever kind of object it is)
    release_all(w, id);
    if !w.rules().quirks.dying_victim_holds_slot {
        // spec: S-0601 claim 12 and Edge cases (quirks.dying_victim_holds_slot off: a victim leaves the list at its killing hit)
        drop_silently(w, id);
    }
}

/// An object is removed from the game (claims 10 and 12): a controller
/// releases every victim and loses its record; a victim leaves its
/// controller's list silently; the permanent state goes with the object.
pub fn removing(w: &mut World, id: ObjectId) {
    if w.mind.controllers.contains_key(&id) {
        // spec: S-0601 claim 10 (the controller is destroyed in any other way, or a mobile controller is removed from the game)
        release_all(w, id);
        w.mind.controllers.remove(&id);
    }
    // spec: S-0601 claim 12 (a victim removed from the game silently leaves its controller's list)
    drop_silently(w, id);
    w.mind.permanent.remove(&id);
}

/// A building's own turn (claim 10, S-0405 claim 9): a controller building
/// that is not in operation releases every victim. Reconciled every turn.
pub fn building_turn(w: &mut World, id: ObjectId, in_operation: bool) {
    if !in_operation && controls_objects(w, id) {
        // spec: S-0601 claim 10 and S-0405 claim 9 (a controller building not in operation releases its victims in its own turn)
        release_all(w, id);
    }
}

/// An erasing hit is delivered on `x` (claim 10, S-0514 claim 2), even one
/// refused by protection.
pub fn erasing_hit(w: &mut World, x: ObjectId) {
    // spec: S-0601 claim 10 and S-0514 claim 2 (an erasing hit frees the units it controls)
    release_all(w, x);
}

/// A drain link that forms on a controlling building releases all its
/// victims (claim 10; S-0608 claim 5, called by `crate::drain`); a failed
/// attempt releases nothing.
pub fn drain_link_formed(w: &mut World, building: ObjectId) {
    // spec: S-0601 claim 10 (a drain link forms on a controlling building, S-0608 claim 5)
    release_all(w, building);
}

/// A trigger action is about to change object `id`'s house (claim 10,
/// S-0011 claim 24): a controller releases every victim first.
pub fn trigger_owner_change(w: &mut World, id: ObjectId) {
    // spec: S-0601 claim 10 (a trigger action changes the controller's house)
    release_all(w, id);
}

/// Any other owner change of object `id` (claim 11), after the change:
/// with `quirks.mind_control_owner_change_keeps_victims` on (the original)
/// nothing is released; off, a controller releases every victim.
pub fn owner_changed(w: &mut World, id: ObjectId) {
    if w.rules().quirks.mind_control_owner_change_keeps_victims {
        // spec: S-0601 claim 11 (a controller that changes owner keeps its victims)
        return;
    }
    // spec: S-0601 claim 11 and Edge cases (quirks.mind_control_owner_change_keeps_victims off: an owner change releases all victims first)
    release_all(w, id);
}

/// A unit boards a transport (S-0507 claim 6): a controlled unit is
/// released.
pub fn boarding(w: &mut World, id: ObjectId) {
    // spec: S-0601 claim 10 and S-0507 claim 6 (a controlled unit that boards is released)
    release_one(w, id);
}

/// A conversion replaces object `old` with `new` (claim 15), before `old`
/// is removed. With `quirks.deploy_makes_control_permanent` on (the
/// original) nothing is done here: `old`'s removal drops it silently and
/// the new object stays the controlling house's. Off: `new` inherits the
/// control link, the original owner and the place on the list.
pub fn converted(w: &mut World, old: ObjectId, new: ObjectId) {
    if w.rules().quirks.deploy_makes_control_permanent {
        // spec: S-0601 claim 15 (deploying ends control for good: the new building is not controlled)
        return;
    }
    let Some(rec) = w.mind.victims.remove(&old) else {
        return;
    };
    // spec: S-0601 claim 15 and Edge cases (quirks.deploy_makes_control_permanent off: the new object inherits the link, the original owner and the list place)
    w.mind.victims.insert(new, rec);
    if let Some(r) = w.mind.controllers.get_mut(&rec.controller) {
        for x in &mut r.victims {
            if *x == old {
                *x = new;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Permanent control (claim 13).

/// Psychic domination by house `h` reaches object `id` (claim 13, S-0612
/// claims 13 and 14). Returns whether it became permanently controlled.
/// The Psychic Dominator's sweep (S-0612 claims 13 and 14,
/// `crate::domination`) calls this for each examined object; its own-house
/// switch `quirks.dominator_marks_own_objects` is tested there. A
/// dominated drainer's link then ends by S-0608 claim 11's alliance test,
/// which reads the owners in the drainer's turn (`crate::drain`).
pub fn dominate(w: &mut World, id: ObjectId, h: HouseId) -> bool {
    if !dominable(w, id) {
        return false;
    }
    // spec: S-0601 claim 13 (a controlled object is first released by its controller, with all of claim 9's effects)
    release_one(w, id);
    // spec: S-0601 claim 13 (then it belongs to the dominating house; no owner change when it already does)
    let changes = w.obj(id).is_some_and(|o| o.owner != Some(h));
    set_owner(w, id, Some(h));
    if changes {
        // spec: S-0309 claims 30 and 31 (a psychic domination is a change of claim 30's list and resets the orders inside it, adding no guard reset; a release before it gave the reset at the release's change too, the domination's only when the dominating house is not then the owner)
        crate::structures::ownership::reset_orders(w, id);
    }
    // spec: S-0601 claim 13 and S-0612 claim 14 (it becomes permanently controlled, never cleared; no original owner is recorded; a set, so one ring however often it is taken)
    w.mind.permanent.insert(id);
    true
}

/// Whether permanent control may take `id` (S-0601 claim 13's tests, the
/// conditions of S-0612 claim 13 and no others).
pub fn dominable(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    // spec: S-0601 claim 13 (not a building)
    if o.kind == Kind::Building || o.kind == Kind::Terrain {
        return false;
    }
    // spec: S-0601 claim 13 and S-0610 claim 1 (no psionics immunity)
    if type_combat(w, id).is_some_and(|tc| tc.immune_psionics) {
        return false;
    }
    // spec: S-0601 claim 13 (not under protection)
    if crate::superweapons::protected(w, id) {
        return false;
    }
    // spec: S-0601 claim 13 (its type does not hover like a balloon)
    if w.rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.raw.balloon_hover.unwrap_or(false))
    {
        return false;
    }
    // spec: S-0601 claim 13 (not in the air)
    !crate::combat::weapon::in_air(w, id)
}

// ---------------------------------------------------------------------------
// Overload (claim 16).

/// The overload level of F2 for victim count `n`: the first index whose
/// count is at least `n`, else the last; `None` for an empty count list.
pub fn overload_level(count: &[i32], n: i64) -> Option<usize> {
    if count.is_empty() {
        // spec: S-0601 Import mapping (an empty count list gives no overload pulses, a design fallback)
        return None;
    }
    // spec: S-0601 F2 (i = the smallest index with countᵢ ≥ N; else L − 1)
    Some(
        count
            .iter()
            .position(|&c| i64::from(c) >= n)
            .unwrap_or(count.len() - 1),
    )
}

/// Whether `id` is an unlimited controller (claim 16; S-0610 Terms).
pub fn unlimited(w: &World, id: ObjectId) -> bool {
    controller(w, id).is_some_and(|c| c.unlimited)
}

/// An unlimited controller's overload update, in its own turn (claim 16):
/// after the first per-tick start and the berserk countdown, before
/// self-healing, behaviour processing, movement and firing
/// (`tick::object_turn`).
pub fn overload_turn(w: &mut World, id: ObjectId) {
    let Some(rec) = w.mind.controllers.get_mut(&id) else {
        return;
    };
    // spec: S-0601 claim 16 (controllers that are not unlimited never pulse)
    if !rec.unlimited {
        return;
    }
    // spec: S-0601 claim 16 (the pip highlight decreases by 1 on each update while positive)
    if rec.highlight > 0 {
        rec.highlight -= 1;
    }
    // spec: S-0601 claim 16 (a positive countdown decreases by 1, with no pulse check, including the update that changes 1 to 0)
    if rec.countdown > 0 {
        rec.countdown -= 1;
        return;
    }
    let n = rec.victims.len() as i64;
    let m = &w.rules().combat.general.mind;
    let Some(i) = overload_level(&m.overload_count, n) else {
        return;
    };
    // spec: S-0601 Import mapping (a missing damage or frames entry counts as 0, a design fallback)
    let frames = m.overload_frames.get(i).copied().unwrap_or(0);
    let d = m.overload_damage.get(i).copied().unwrap_or(0);
    let c4 = w.rules().combat.general.c4_warhead;
    let half = w.rules().quirks.ranged_draw_half_span;
    let Some(rec) = w.mind.controllers.get_mut(&id) else {
        return;
    };
    // spec: S-0601 claim 16 and F2 (the countdown becomes framesᵢ overload updates)
    rec.countdown = frames;
    if d < 1 {
        // spec: S-0601 claim 16 (d < 1: the sound mark is cleared; no damage, highlight, sound, sparks or rocking)
        rec.sound_mark = false;
        return;
    }
    // spec: S-0601 claim 16 (the pip highlight is set to 10)
    rec.highlight = PULSE_HIGHLIGHT;
    let sound = !rec.sound_mark;
    // spec: S-0601 claim 16 (the mark becomes set)
    rec.sound_mark = true;
    let pos = w.obj(id).map_or(Pos::default(), |o| o.pos);
    // spec: S-0601 claim 16 (a hit of d with the demolition warhead, no attacker or attacking house, not ignoring defences; S-0302 resolves it)
    crate::combat::damage::apply(
        w,
        id,
        Hit {
            amount: d,
            warhead: c4,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: false,
            no_escape: false,
        },
    );
    if sound {
        // spec: S-0601 claim 16 (the overload sound plays at the controller when its sound mark was clear)
        w.effects.push(Effect::Mind {
            sound: MindSound::Overload,
            of: None,
            houses: [None, None],
            pos: Some(pos),
        });
    }
    // spec: S-0601 claim 16 and Import mapping (the spark type key absent: no spark systems and no placement draws)
    let sparks = if w.rules().gas.default_spark_given {
        SPARKS
    } else {
        0
    };
    // spec: S-0601 claim 16 and F3 (five spark systems, even after a lethal hit; per system y then x in [−200, 200])
    for _ in 0..sparks {
        let y = w.rng.ranged(-SPARK_SPREAD, SPARK_SPREAD, half);
        let x = w.rng.ranged(-SPARK_SPREAD, SPARK_SPREAD, half);
        // spec: S-0601 F3 (position = controller's position + (x, y, 100) leptons, exact integer additions)
        let at = Pos::new(pos.x + x, pos.y + y, pos.z + SPARK_HEIGHT);
        // spec: S-0601 claim 16 and S-0616 claim 1 (a system of the default spark type; making it takes no draw)
        crate::particles::spark_system(w, at);
    }
    // spec: S-0601 claim 16 and F4 (when i > 0 and the controller survives, one rocking draw r in [0, 100]; the rate is a presentation input)
    if i > 0 && w.obj(id).is_some_and(|o| o.health > 0) {
        let r = w.rng.ranged(ROCK_DRAW.0, ROCK_DRAW.1, half);
        // spec: S-0601 F4 (the rate is a presentation input to S-0106 claim 9: reported to the drawing, which keeps it)
        w.effects.push(Effect::OverloadRock {
            controller: id,
            level: i as i32,
            draw: r,
        });
    }
}

// ---------------------------------------------------------------------------
// Presentation (claims 17 and 18).

/// One link line (claim 17), in world leptons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinkLine {
    /// The controller.
    pub controller: ObjectId,
    /// The victim.
    pub victim: ObjectId,
    /// The controller's house (the line's colour).
    pub house: Option<HouseId>,
    /// The start: the alternate link anchor `k mod 5` (F5).
    pub from: Pos,
    /// The end: the victim's position raised by its type's link height.
    pub to: Pos,
    /// The victim's link-line timer is running.
    pub timer: bool,
}

/// The alternate link anchor `k` of controller `c` (F5): S-0313 claim 1's
/// firing-position transform with the imported anchor in place of the
/// muzzle triple.
pub fn anchor(w: &World, c: ObjectId, k: usize) -> Pos {
    let Some(tc) = type_combat(w, c) else {
        return w.obj(c).map_or(Pos::default(), |o| o.pos);
    };
    // spec: S-0601 F5 (anchor index = k mod 5, exact)
    let flh = tc.alternate_flh[k % ANCHORS];
    crate::combat::position::transform_point(w, c, flh, 0)
}

/// Every link line now, controllers in object order and victims in list
/// order (claim 17). The view draws a line when the controller, a transport
/// carrying it or the victim is selected, or the timer runs; selecting only
/// the carrying transport shows it too (the fixed drawing bug).
pub fn link_lines(w: &World) -> Vec<LinkLine> {
    let mut out = Vec::new();
    for (&c, rec) in &w.mind.controllers {
        let Some(co) = w.obj(c) else { continue };
        for (k, &v) in rec.victims.iter().enumerate() {
            let Some(vo) = w.obj(v) else { continue };
            if !vo.on_map {
                continue;
            }
            let lift = type_combat(w, v).map_or(0, |tc| tc.link_height);
            let timer = w
                .mind
                .victims
                .get(&v)
                .is_some_and(|r| w.tick < r.line_until);
            out.push(LinkLine {
                controller: c,
                victim: v,
                house: co.owner,
                from: anchor(w, c, k),
                // spec: S-0601 claim 17 (the end: the victim's position raised by its type's link height)
                to: Pos::new(vo.pos.x, vo.pos.y, vo.pos.z + lift),
                timer,
            });
        }
    }
    out
}

/// A control ring (claim 18).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ring {
    /// The animation name.
    pub anim: String,
    /// The permanent ring.
    pub permanent: bool,
    /// Where: a mobile object's position raised by its type's ring height;
    /// a building's top-centre point (the exact building-height conversion
    /// is not established, S-0601 Open questions).
    pub at: Pos,
    /// Drawn in front of a building.
    pub building: bool,
}

/// The control ring of an object, if it carries one (claim 18).
pub fn ring(w: &World, id: ObjectId) -> Option<Ring> {
    let o = w.obj(id)?;
    let m = &w.rules().combat.general.mind;
    // spec: S-0601 claim 18 (the controlled-animation type on a controlled object, the permanent one on a permanently controlled object)
    let (anim, permanent) = if permanent(w, id) {
        (m.perma_ring.clone()?, true)
    } else if w.mind.victims.contains_key(&id) {
        (m.ring.clone()?, false)
    } else {
        return None;
    };
    let building = o.kind == Kind::Building;
    let at = if building {
        // MC13 has not established the building-height conversion: the ring
        // stands at the building's reference point, drawn in front of it.
        w.reference_point(id)
    } else {
        let lift = type_combat(w, id).map_or(0, |tc| tc.ring_height);
        // spec: S-0601 claim 18 (raised above a mobile object by its type's ring height in leptons)
        Pos::new(o.pos.x, o.pos.y, o.pos.z + lift)
    };
    Some(Ring {
        anim,
        permanent,
        at,
        building,
    })
}

/// The pip highlight of a controller (claim 16; S-0111 claim 13 draws it).
pub fn highlight(w: &World, id: ObjectId) -> i32 {
    controller(w, id).map_or(0, |c| c.highlight)
}
