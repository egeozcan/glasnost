//! Chrono weapons (S-0514): temporal erasure. An erasing shot attaches the
//! firer's temporal beam to its target; the target is *being erased*
//! while at least one beam is attached, loses *R* = 10 × strength erasure
//! units by the raw weapon damages of its attackers at each of its erasure
//! updates, and is removed without death when *R* reaches 0 or less.
//!
//! Beams are kept by holder: an attacker holds at most one, and a gunner
//! vehicle carries the beam of its first passenger (S-0308 claim 6). No
//! site takes a draw, but the animations a completion or a sparkle creates
//! take their creation draws like any animation (S-0202 claim 10).

use std::collections::{BTreeMap, BTreeSet};

use glasnost_map::CellXY;
use glasnost_num::geom::root_floor;
use serde::{Deserialize, Serialize};

use crate::combat::damage::{self, Hit};
use crate::combat::effects;
use crate::combat::veterancy::{self, type_combat};
use crate::rules::Kind;
use crate::world::{ObjectId, Pos, Target, World};

/// The erasure of one target (S-0514 Terms).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lock {
    /// The leading attacker.
    pub leader: ObjectId,
    /// The joined attackers, oldest first.
    pub joiners: Vec<ObjectId>,
    /// The erasure amount *R*.
    pub amount: i32,
    /// Whether a building was switched off before the first lock (for
    /// `quirks.erasure_release_switches_on` off).
    pub was_off: bool,
    /// Whether the target, a superweapon warp member, stopped being erased
    /// at its departure while this beam stays attached (S-0513 claim 39a):
    /// set by the departure, kept through a leadership handover (S-0514
    /// claim 9), cleared by any later lock.
    #[serde(default)]
    pub suspended: bool,
}

/// A beam attached to a target.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Beam {
    /// The target.
    pub target: ObjectId,
    /// The cell its holder stood in when it locked (claim 14: arriving in
    /// a new cell lets go).
    pub cell: CellXY,
}

/// All erasure state of a game.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalWorld {
    /// The erasures by target.
    pub locks: BTreeMap<ObjectId, Lock>,
    /// The attached beams by holder.
    pub beams: BTreeMap<ObjectId, Beam>,
    /// The gunner vehicles that carry a passenger's beam, with that
    /// passenger.
    pub carried: BTreeMap<ObjectId, ObjectId>,
    /// The passengers whose beam a vehicle carries.
    pub given: BTreeSet<ObjectId>,
}

/// Whether an object has a temporal beam (S-0514 Terms): its type supplies
/// one from its primary weapon unless a gunner vehicle carries it, and a
/// gunner vehicle carries its first passenger's.
pub fn has_beam(w: &World, id: ObjectId) -> bool {
    (type_beam(w, id) && !w.temporal.given.contains(&id)) || w.temporal.carried.contains_key(&id)
}

/// Whether the object's type supplies a beam (primary weapon's warhead is
/// temporal).
pub fn type_beam(w: &World, id: ObjectId) -> bool {
    let r = w.rules();
    type_combat(w, id)
        .and_then(|tc| tc.slots[0])
        .and_then(|p| r.combat.weapon(p).warhead)
        .is_some_and(|h| r.combat.warhead(h).temporal)
}

/// Whether the object is being erased by a beam (the beam-free leaving
/// stage of S-0513 claim 39 is `superweapons::being_erased`'s): a beam is
/// attached and the target has not stopped being erased at a warp
/// departure (S-0513 claim 39a).
pub fn erased(w: &World, id: ObjectId) -> bool {
    w.temporal.locks.get(&id).is_some_and(|l| !l.suspended)
}

/// Whether a temporal beam is attached to the object (S-0514 Terms),
/// being erased or not.
pub fn attached(w: &World, id: ObjectId) -> bool {
    w.temporal.locks.contains_key(&id)
}

/// Whether the object's turns are cut short by a beam (S-0514 claim 17,
/// S-0215 claim 2): while it is being erased, and while it phases in with
/// a beam attached for a vehicle or aircraft, or for an infantry unit with
/// `quirks.phase_in_beam_spares_infantry` off (S-0513 claim 39a).
pub fn cut_short(w: &World, id: ObjectId) -> bool {
    if erased(w, id) {
        return true;
    }
    if !attached(w, id) || !crate::teleport::phasing_in(w, id) {
        return false;
    }
    match w.obj(id).map(|o| o.kind) {
        // spec: S-0513 claim 39a (Departure: while it phases in, a vehicle or aircraft with a temporal beam attached has its turns cut short whether or not it is being erased) and S-0514 claim 17
        Some(Kind::Vehicle | Kind::Aircraft) => true,
        // spec: S-0513 claim 39a and Edge cases (an infantry unit's turns are cut short only while it is being erased; quirks.phase_in_beam_spares_infantry off: also while a beam is attached)
        Some(Kind::Infantry) => !w.rules().quirks.phase_in_beam_spares_infantry,
        _ => false,
    }
}

/// Whether `holder`'s beam is attached to `target` (S-0301 Terms, *holding
/// a target*).
pub fn holds(w: &World, holder: ObjectId, target: ObjectId) -> bool {
    w.temporal
        .beams
        .get(&holder)
        .is_some_and(|b| b.target == target)
}

/// The attackers of a lock, leader first.
fn attackers(l: &Lock) -> Vec<ObjectId> {
    let mut v = vec![l.leader];
    v.extend(l.joiners.iter().rev());
    v
}

/// The render-layer test of S-0102 claim 3 for delivery (S-0514 claim 1):
/// a flying or jumpjet unit above the ground, or a rocket, is not in the
/// ground layer.
pub fn ground_layer(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if o.rocket.is_some() {
        return false;
    }
    let fly = w
        .rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.locomotor == crate::rules::Locomotor::Fly);
    let in_air = w.height_above_surface(id) > 0;
    // spec: S-0102 State and ordering (flying and jumpjet movement are in the ground layer on the ground, and in another above it; every other kind is in the ground layer when not in the air) and S-0607 claim 5 (a seized unit moves with the jumpjet locomotor)
    !(in_air && (o.kind == Kind::Aircraft || fly || crate::jump::is_jumpjet(w, id)))
}

/// A temporal detonation (S-0514 claims 1 to 4): the firer and the shot's
/// target. Does nothing unless both are present.
pub fn deliver(w: &mut World, firer: Option<ObjectId>, target: Option<Target>) {
    // spec: S-0514 claim 1 (a temporal detonation needs its firer and its target still present)
    let Some(f) = firer.filter(|&f| w.obj(f).is_some()) else {
        return;
    };
    let Some(target) = target else { return };
    let Target::Object(x) = target else {
        // spec: S-0514 claim 1 (a cell only makes the firer's old beam let go)
        let_go(w, f);
        return;
    };
    // spec: S-0514 claim 1 and S-0309 claim 24 (a bunkered vehicle is redirected to its bunker, S-0606 claim 9)
    let x = crate::structures::bunker::bunkered_in(w, x).unwrap_or(x);
    let Some(t) = w.obj(x) else { return };
    if !matches!(
        t.kind,
        Kind::Infantry | Kind::Vehicle | Kind::Aircraft | Kind::Building
    ) {
        // spec: S-0514 claim 1 (a non-combat object only makes the firer's old beam let go)
        let_go(w, f);
        return;
    }
    // spec: S-0514 claim 1 (an object target must be in the ground render layer, otherwise this effect does nothing) and S-0306 claim 17 (a warp member is in the ground render layer from its warp's launch until at least the end of its arrival hold, so the test passes for it, off the map or phasing in); S-0514 r13 claim 1 (a departed warp member target, off the map or phasing in, is an ordinary object target for claims 1 to 4)
    if !ground_layer(w, x) && !crate::superweapons::warp::in_ground_layer(w, x) {
        return;
    }
    // Q-0984 (interim: a firer whose type supplies no beam, and none carried, makes no lock and no side effect)
    if !has_beam(w, f) {
        return;
    }
    // spec: S-0514 claims 2 and 14 (the firer's old beam lets go, also at the same target)
    let_go(w, f);
    let refused = refusal(w, f, x);
    let side_effects_first = w.rules().quirks.erasure_refused_hit_side_effects;
    if side_effects_first {
        // spec: S-0514 claim 2 (a delivered hit clears the spawner slots and frees controlled units, even when the lock is refused)
        side_effects(w, x);
    }
    if refused {
        return;
    }
    lock(w, f, x);
    if !side_effects_first {
        // spec: S-0514 claim 2 (quirks.erasure_refused_hit_side_effects off: only a successful lock has the side effects)
        side_effects(w, x);
    }
}

/// Claim 3: whether no lock attaches.
fn refusal(w: &World, firer: ObjectId, target: ObjectId) -> bool {
    // spec: S-0514 claim 3 (the target type is not erasable)
    if type_combat(w, target).is_some_and(|tc| !tc.warpable) {
        return true;
    }
    // spec: S-0514 claim 3 (iron curtain or force shield protects it, S-0513 claim 27)
    if crate::superweapons::protected(w, target) {
        return true;
    }
    // spec: S-0514 claim 3 (a vehicle still standing in the cell of the vehicle factory it is in contact with)
    if let Some(o) = w.obj(target)
        && o.kind == Kind::Vehicle
        && let Some(b) = o.contact.and_then(|c| w.obj(c))
        && b.kind == Kind::Building
        && crate::econ::placement::foundation_cells(w, b.type_index, b.pos.cell())
            .contains(&o.pos.cell())
    {
        return true;
    }
    // spec: S-0514 claim 3 and S-0513 claim 39a (quirks.warp_departure_keeps_beam off: a target's pending superweapon warp, from the launch until its departure update, is one more refusal; on, it is none)
    if !w.rules().quirks.warp_departure_keeps_beam
        && crate::superweapons::warp_pending_or_leaving(w, target)
    {
        return true;
    }
    // spec: S-0514 claim 3 (a beam already attached to the firer refuses the new lock, being erased or not; the firer's own old beam was let go before)
    attached(w, firer)
}

/// Claim 2's target side effects.
fn side_effects(w: &mut World, x: ObjectId) {
    // spec: S-0514 claim 2 (a delivered hit on an infantry unit, vehicle, aircraft or building clears its spawner slots, S-0506 claim 13)
    if w.obj(x).is_some_and(|o| o.spawn.is_some()) {
        crate::spawn::clear_slots(w, x);
    }
    // spec: S-0514 claim 2 and S-0601 claim 10 (and frees the units it mind-controls)
    crate::mind::erasing_hit(w, x);
}

/// The raw damage an attacker would deal by one update (claim 7).
fn raw_damage(w: &World, attacker: ObjectId) -> i32 {
    // spec: S-0514 claim 7 (the weapon each attacker would select with no target; raw damage, no multipliers)
    crate::combat::weapon::would_use(w, attacker, None)
        .1
        .map_or(0, |wp| w.rules().combat.weapon(wp).damage)
}

/// Attaches `f`'s beam to `x` (claim 4).
fn lock(w: &mut World, f: ObjectId, x: ObjectId) {
    let cell = w.obj(f).map_or(CellXY::new(0, 0), |o| o.pos.cell());
    if let Some(l) = w.temporal.locks.get_mut(&x) {
        // spec: S-0514 claim 4 (a lock on an erased target joins without resetting the amount)
        l.joiners.push(f);
        // spec: S-0513 claim 39a (A lock after the departure: the lock makes the member being erased again)
        l.suspended = false;
        // spec: S-0514 claim 4 and S-0609 claim 11 (a joining lock also gives a gattling target one spin-down update with multiplier 1)
        crate::gattling::temporal_lock(w, x);
    } else {
        // spec: S-0514 claim 4 (the target releases any beam of its own)
        let_go(w, x);
        let strength = damage::strength(w, x);
        let was_off = w
            .obj(x)
            .is_some_and(|o| o.kind == Kind::Building && o.econ.building.switched_off);
        w.temporal.locks.insert(
            x,
            Lock {
                leader: f,
                joiners: Vec::new(),
                // spec: S-0514 F1 (R₀ = 10 × S, wrapping; the type's strength, not the current health, claim 6)
                amount: strength.wrapping_mul(10),
                was_off,
                suspended: false,
            },
        );
        if w.obj(x).is_some_and(|o| o.kind == Kind::Building) {
            // spec: S-0514 claim 4 (a building is switched off and its house reconsiders power and radar)
            set_switch(w, x, true);
        }
        // spec: S-0514 claim 4 and S-0609 claim 11 (a successful lock gives a gattling target one spin-down update with multiplier 1, with its sound effect)
        crate::gattling::temporal_lock(w, x);
        // spec: S-0514 claim 5 (first-lock notices: no notice mechanism exists in our engine yet)
    }
    w.temporal.beams.insert(f, Beam { target: x, cell });
}

/// Switches a locked building off, or on again (claims 4 and 10).
fn set_switch(w: &mut World, b: ObjectId, off: bool) {
    let Some(ob) = w.obj_mut(b) else { return };
    ob.econ.building.switched_off = off;
    if off {
        crate::econ::shroud::gap_switched_off(w, b);
        crate::cloak::field::switched_off(w, b);
    }
    crate::econ::events::switched(w, b);
}

/// Lets `holder`'s beam go (claims 9, 10 and 14).
pub fn let_go(w: &mut World, holder: ObjectId) {
    let Some(beam) = w.temporal.beams.remove(&holder) else {
        return;
    };
    let Some(l) = w.temporal.locks.get_mut(&beam.target) else {
        return;
    };
    if l.leader != holder {
        // spec: S-0514 claim 9 (a departing joiner simply stops contributing)
        l.joiners.retain(|&j| j != holder);
        return;
    }
    if let Some(next) = l.joiners.pop() {
        // spec: S-0514 claim 9 (the most recent remaining joiner leads, with the same amount; exception: a warp member that stopped being erased at its departure stays not being erased, S-0513 claim 39a)
        l.leader = next;
        return;
    }
    end_lock(w, beam.target);
}

/// The last attacker let go before completion (claim 10): the target
/// stops being erased and loses its progress.
fn end_lock(w: &mut World, x: ObjectId) {
    let Some(l) = w.temporal.locks.remove(&x) else {
        return;
    };
    if w.obj(x).is_some_and(|o| o.kind == Kind::Building) {
        // spec: S-0514 claim 10 (a building is switched on, unconditionally; quirks.erasure_release_switches_on off restores its earlier state)
        let off = !w.rules().quirks.erasure_release_switches_on && l.was_off;
        set_switch(w, x, off);
    }
}

/// F4: whether the distance of an attacker to its target exceeds the warp
/// distance.
fn too_far(w: &World, attacker: ObjectId, target: ObjectId) -> bool {
    let (Some(a), Some(t)) = (w.obj(attacker), w.obj(target)) else {
        return false;
    };
    let sq = |d: i32| i64::from(d) * i64::from(d);
    // spec: S-0514 F4 (the differences of the objects' own positions, 32-bit with wrap; no building-size reduction)
    let n = sq(a.pos.x.wrapping_sub(t.pos.x))
        + sq(a.pos.y.wrapping_sub(t.pos.y))
        + sq(a.pos.z.wrapping_sub(t.pos.z));
    let r = w.rules();
    // spec: S-0514 F4 (dist = trunc(r(q)), the table square root of S-0204 claim 5; quirks.table_root_distance)
    let dist = root_floor(n as u64, r.quirks.table_root_distance, r.mode) as i64;
    // spec: S-0514 F4 and Constants (release when dist > 256 × W, in 32-bit integers with wrap)
    dist > i64::from(256i32.wrapping_mul(r.combat.general.warp_distance))
}

/// A target's erasure update and presentation (claims 7, 8, 11, 17, 18),
/// in its own turn. Returns whether the rest of an infantry unit's,
/// vehicle's or aircraft's turn is skipped (claim 17, and S-0513 claim 39a
/// for a member phasing in with a beam attached: [`cut_short`]); a
/// building skips its behaviour processing and firing through
/// [`building_erased`]. `update` is false when the turn's erasure update
/// was already made before its warp step ([`member_update`]).
pub fn turn(w: &mut World, id: ObjectId, update: bool) -> bool {
    if !attached(w, id) {
        return false;
    }
    // The sparkle of this turn is made by [`turn_sparkle`], before this.
    let short = cut_short(w, id) && kind_of(w, id) != Kind::Building;
    if short {
        // spec: S-0514 claim 17 and S-0215 claim 2 (a mobile object being erased drops its target and destination on each such tick) and S-0513 claim 39a (so does a vehicle or aircraft phasing in with a beam attached)
        inactivity_clear(w, id);
    }
    if update {
        erasure_update(w, id);
    }
    short
}

fn kind_of(w: &World, id: ObjectId) -> Kind {
    w.obj(id).map_or(Kind::Terrain, |o| o.kind)
}

/// S-0514 claim 17's clearing of an infantry unit, vehicle or aircraft
/// whose turn is cut short: its target and destination, the destination
/// kept under a pending point suppression. A building clears nothing.
fn inactivity_clear(w: &mut World, id: ObjectId) {
    if kind_of(w, id) == Kind::Building {
        return;
    }
    // spec: S-0215 claim 2 and S-0514 claim 17 (a vehicle under a pending point suppression, S-0207 claim 27, keeps its destination; the target is dropped in every case)
    let keeps = crate::refresh::suppression_pending(w, id);
    if let Some(ob) = w.obj_mut(id) {
        crate::spawn::clear_own_target(ob);
        if !keeps {
            ob.dest = None;
            ob.path.steps.clear();
        }
    }
}

/// The drop of a warp member after its turn's warp step (S-0513 claim 39a,
/// *Sparkles and drops in the warp's turns*): when the member is still
/// being erased after the warp step, or its turns are cut short by a beam,
/// it clears its target and destination as S-0514 claim 17 gives. The
/// arrival hold, whose ordinary turn follows the warp step, makes this
/// drop through [`turn`].
pub fn member_drop(w: &mut World, id: ObjectId) {
    // spec: S-0513 claim 39a (Q-2316: in each turn in which the member is still being erased after the turn's warp step, or whose turns are cut short by a beam, it clears its target and destination as S-0514 claim 17 gives: its leaving updates from t_L + 1 to the expiry update, the expiry's target search included, with or without a beam; the departure update of a vehicle or aircraft with a beam attached; every turn after a later lock) and claims 48 and 51; S-0514 claim 17
    if crate::superweapons::being_erased(w, id) || cut_short(w, id) {
        inactivity_clear(w, id);
    }
}

/// The sparkle of an object's turn (S-0514 claim 18): one in each turn of
/// an object being erased, phasing in, or both. A warp member's comes
/// before its turn's warp step (S-0513 claims 39a and 51); another
/// object's at its place in the ordinary turn. Both come before the turn's
/// erasure update (interim Q-2377).
pub fn turn_sparkle(w: &mut World, id: ObjectId) {
    // spec: S-0513 claim 39a (Q-2316: each turn of a warp member at every stage is one of its turns for S-0514 claim 18, with or without a beam; in its leaving stage it counts as being erased, from its departure it phases in; its state before the turn's warp step decides; one sparkle per turn whether it is being erased, phasing in or both) and S-0514 claim 18
    if crate::superweapons::being_erased(w, id) || crate::teleport::phasing_in(w, id) {
        sparkle(w, id);
    }
}

/// The erasure update of a warp member with a beam attached, made before
/// its turn's warp step (S-0513 claim 39a, *The erasure goes on*; S-0203
/// claim 4's intrinsic exception). Returns whether an update was made, so
/// that the turn's [`turn`] makes no second one.
pub fn member_update(w: &mut World, id: ObjectId) -> bool {
    // spec: S-0513 claim 39a (in each turn of the member after the lock, at every stage of the warp, the turn has its erasure update, before the turn's warp step) and S-0203 claim 4 (an intrinsic exception to claim 51's whole-member-turn contract)
    if !attached(w, id) || !crate::superweapons::warp::is_member(w, id) {
        return false;
    }
    erasure_update(w, id);
    true
}

/// The erasure update proper (claims 7, 8 and 11): the open-topped
/// distance release, or the reduction and any completion.
fn erasure_update(w: &mut World, id: ObjectId) {
    let Some(l) = w.temporal.locks.get(&id).cloned() else {
        return;
    };
    // spec: S-0514 claim 11 (a leading attacker inside an open-topped transport lets go when its distance exceeds the warp distance; that update makes no reduction, even if another attacker becomes leader; quirks.erasure_distance_leader_only off checks every open-topped passenger)
    let all = !w.rules().quirks.erasure_distance_leader_only;
    let checked: Vec<ObjectId> = if all { attackers(&l) } else { vec![l.leader] };
    let mut released = false;
    for a in checked {
        if crate::transport::open_topped_passenger(w, a) && too_far(w, a, id) {
            let_go(w, a);
            released = true;
        }
    }
    if released {
        return;
    }
    // spec: S-0514 claim 7 and F2 (D = the sum of the raw damages of the leader and the 51 most recent joiners)
    let contributors: Vec<ObjectId> = attackers(&l).into_iter().take(52).collect(); // spec: S-0514 Constants (52 contributors: the leader and 51 joiners)
    let step = contributors
        .iter()
        .fold(0i32, |s, &a| s.wrapping_add(raw_damage(w, a)));
    let Some(lock) = w.temporal.locks.get_mut(&id) else {
        return;
    };
    // spec: S-0514 F3 (R ← R − D, signed 32-bit with wrap; completion when the result is at most 0)
    lock.amount = lock.amount.wrapping_sub(step);
    if lock.amount <= 0 {
        complete(w, id);
    }
}

/// A warp member departs (S-0513 claim 39a, *Departure*): the departure
/// releases no beam and keeps the amount, but from its departure update
/// the member is no longer being erased while the beam stays attached.
pub fn departed(w: &mut World, id: ObjectId) {
    if let Some(l) = w.temporal.locks.get_mut(&id) {
        // spec: S-0513 claim 39a (Departure: no longer being erased from its departure update, the beam attached and the amount kept) and S-0514 claim 14 (the target's own superweapon warp is no release cause)
        l.suspended = true;
    }
}

/// Whether a building's behaviour processing and firing are skipped by an
/// erasure (claim 17).
pub fn building_erased(w: &World, id: ObjectId) -> bool {
    erased(w, id) && w.obj(id).is_some_and(|o| o.kind == Kind::Building)
}

/// A sparkle for an erased or phasing-in object (claim 18): every global
/// tick divisible by 24, at the object's turn.
pub fn sparkle(w: &mut World, id: ObjectId) {
    // spec: S-0514 Constants (sparkle period 24 ticks, on the global tick number)
    if w.tick % 24 != 0 {
        return;
    }
    let Some(name) = w.rules().combat.general.chrono_sparkle.clone() else {
        return;
    };
    let Some(o) = w.obj(id) else { return };
    let at = match o.kind {
        // spec: S-0514 claim 18 (infantry and vehicles: 120 leptons farther along each horizontal axis, same height)
        Kind::Infantry | Kind::Vehicle => Pos::new(o.pos.x + 120, o.pos.y + 120, o.pos.z), // spec: S-0514 Constants (mobile sparkle displacement 120 leptons)
        // spec: S-0514 claim 18 (aircraft and buildings without damage-fire positions: their own position; the damage-fire positions are not imported)
        _ => o.pos,
    };
    // spec: S-0514 r14 claim 18 (the sparkle is an ordinary animation with S-0109's simulation effects: its playback draws, claims 3, 11 and 28, its damage, claim 17, its start and middle effects, claim 28; a creation draw comes in the turn's place, before a warp member's warp step, S-0513 claim 39a) and S-0109 claim 29
    effects::anim(w, &name, at);
}

/// Completion (claims 12 and 13): the target is removed without death.
fn complete(w: &mut World, x: ObjectId) {
    let Some(l) = w.temporal.locks.remove(&x) else {
        return;
    };
    let all = attackers(&l);
    for &a in &all {
        w.temporal.beams.remove(&a);
    }
    let Some(o) = w.obj(x).cloned() else { return };
    // spec: S-0514 claim 12 (the warp-away animation at the target)
    if let Some(name) = w.rules().combat.general.warp_away.clone() {
        effects::anim(w, &name, o.pos);
    }
    // spec: S-0514 claim 12 and S-0508 claim 15 (a building's occupants are removed with it)
    if o.kind == Kind::Building {
        crate::structures::garrison::erased(w, x);
        // spec: S-0605 claim 14 and S-0514 claim 12 (an erased absorbing building deletes its passengers)
        crate::structures::absorb::erased(w, x);
        // spec: S-0514 claim 12 and S-0606 claim 13 (an erased bunker lets its vehicle out at the completion of its erasure)
        crate::structures::bunker::let_out(w, x);
    } else {
        // spec: S-0514 claim 12 and S-0606 claim 14 (removing a bunkered vehicle releases its bunker in place)
        crate::structures::bunker::release_in_place(w, x);
    }
    let hit = Hit {
        amount: 0,
        warhead: None,
        distance: 0,
        attacker: Some(l.leader),
        house: None,
        ignore_defences: true,
        no_escape: true,
    };
    // spec: S-0514 claim 13 (the leader is the credited killer; recipients and kill value follow S-0311 claims 7 to 10)
    let direct = w.rules().quirks.erasure_double_experience
        && w.obj(l.leader).is_some()
        && type_combat(w, l.leader).is_some_and(|tc| tc.trainable);
    if direct {
        // spec: S-0514 claim 13 and F5 (a trainable leader's direct gain: the target's cost over the leader's own, composed first; it goes to the leader itself, with no rank multiplier or allied and unscored exclusion)
        let value = veterancy::actual_cost(w, x);
        let own = veterancy::actual_cost(w, l.leader);
        veterancy::gain(w, l.leader, value, own);
    }
    crate::combat::death::mark_killed(w, x);
    damage::credit(w, x, &hit);
    // spec: S-0514 claim 13 (loss announcements and destruction-trigger events accompany completion)
    crate::combat::death::destruction_raises(w, x, &hit);
    // spec: S-0514 claim 12 (removed without its death animation, death weapon, debris, rubble, wreck or survivors; passengers are removed with no killer, slaves are freed, the removal's own rules)
    crate::objects::remove(w, x);
    // spec: S-0514 claim 12 (removing the target makes its attackers enter idle behaviour)
    for a in all {
        crate::process::fall_back(w, a, false);
    }
}

/// An object leaves play (claims 12 and 14): as a target it invalidates all
/// attached beams and its attackers go idle; as a holder its beam lets go.
pub fn leaving(w: &mut World, id: ObjectId) {
    if let Some(l) = w.temporal.locks.remove(&id) {
        let all = attackers(&l);
        for &a in &all {
            w.temporal.beams.remove(&a);
        }
        for a in all {
            crate::process::fall_back(w, a, false);
        }
    }
    let_go(w, id);
    if let Some(p) = w.temporal.carried.remove(&id) {
        w.temporal.given.remove(&p);
    }
    if w.temporal.given.remove(&id) {
        w.temporal.carried.retain(|_, &mut p| p != id);
    }
}

/// A trigger destroys the object (claim 15): the beams on it are detached
/// and its attackers idle before the destructive hit; leadership is not
/// handed on and a building is not switched back on.
pub fn detach_for_destruction(w: &mut World, id: ObjectId) {
    if let Some(l) = w.temporal.locks.remove(&id) {
        let all = attackers(&l);
        for &a in &all {
            w.temporal.beams.remove(&a);
        }
        for a in all {
            crate::process::fall_back(w, a, false);
        }
    }
}

/// A holder's turn (claim 14): arriving in a new cell lets its beam go.
pub fn holder_turn(w: &mut World, id: ObjectId) {
    let Some(b) = w.temporal.beams.get(&id).copied() else {
        return;
    };
    if crate::transport::riding(w, id).is_some() {
        // A passenger's cell is its transport's: it does not arrive anywhere itself.
        return;
    }
    if w.obj(id)
        .is_some_and(|o| o.on_map && o.pos.cell() != b.cell)
    {
        // spec: S-0514 claim 14 (an attached attacker lets go on arriving in a new cell)
        let_go(w, id);
    }
}

/// A gunner vehicle's first passenger enters (S-0308 claim 6, S-0514
/// claims 4 and 14): the vehicle takes the passenger's beam, attached or
/// not, and carries on any erasure.
pub fn hand_over(w: &mut World, vehicle: ObjectId, passenger: ObjectId) {
    if !type_beam(w, passenger) || w.temporal.given.contains(&passenger) {
        return;
    }
    w.temporal.given.insert(passenger);
    w.temporal.carried.insert(vehicle, passenger);
    if let Some(b) = w.temporal.beams.remove(&passenger) {
        w.temporal.beams.insert(vehicle, b);
        if let Some(l) = w.temporal.locks.get_mut(&b.target) {
            if l.leader == passenger {
                l.leader = vehicle;
            }
            for j in &mut l.joiners {
                if *j == passenger {
                    *j = vehicle;
                }
            }
        }
    }
}

/// The last passenger leaves a gunner vehicle (S-0308 claim 6): the beam
/// goes back to that passenger and lets go.
pub fn hand_back(w: &mut World, vehicle: ObjectId, passenger: ObjectId) {
    let_go(w, vehicle);
    if let Some(p) = w.temporal.carried.remove(&vehicle) {
        w.temporal.given.remove(&p);
    }
    let _ = passenger;
}

/// An object came under or out of mind control (S-0514 claim 14, S-0601
/// claims 6 and 9): its beam lets go.
pub fn control_changed(w: &mut World, id: ObjectId) {
    let_go(w, id);
}
