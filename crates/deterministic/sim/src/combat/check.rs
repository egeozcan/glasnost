//! The firing check (S-0301 claim 9) with the aiming answers of S-0310:
//! the per-attempt test whose answer says whether a shot can be made now
//! and, if not, why.

use glasnost_map::terrain::land;
use glasnost_num::F64;
use glasnost_num::geom::{Facing, delta, direction};

use crate::combat::data::WeaponId;
use crate::combat::range;
use crate::combat::veterancy::type_combat;
use crate::combat::weapon::{airborne_by_height, slot_weapon, versus};
use crate::rules::Kind;
use crate::world::{Layer, ObjectId, Pos, Target, World};

/// The answers of the firing check (S-0301 claim 9).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    /// The shot can be made now.
    CanFire,
    /// The rearm countdown runs, or a gate holds the shot.
    Rearming,
    /// Ammunition 0.
    OutOfAmmo,
    /// Not pointed at the target (S-0310).
    NotFacing,
    /// The turret is still turning (S-0310 claim 3).
    StillTurning,
    /// May not fire while moving (S-0310 claims 7, 8).
    Moving,
    /// Needs to be uncloaked.
    Cloaked,
    /// Beyond the weapon's range (S-0307).
    OutOfRange,
    /// Not a legal target.
    Illegal,
    /// Cannot fire this weapon now for a reason of its own.
    Impossible,
    /// Launching spawned aircraft.
    Busy,
    /// Must deploy first (S-0310 claim 9).
    MustDeploy,
}

/// The aiming facing of a firer (S-0310 claims 1, 4, 5) and whether it is
/// aimed at all.
fn aiming_facing(w: &World, id: ObjectId) -> Facing {
    let t = w.tick;
    let o = w.obj(id).expect("firer");
    let turret = type_combat(w, id).is_some()
        && match o.kind {
            Kind::Vehicle => w
                .rules()
                .unit(o.kind, o.type_index)
                .is_some_and(|u| u.turret),
            Kind::Building => true,
            _ => true,
        };
    // spec: S-0310 claim 1 (a vehicle aims with its turret facing if its type has the turret flag, else its body)
    if turret {
        o.turret.value(t)
    } else {
        o.body.value(t)
    }
}

/// The aiming point of a turreted building (S-0310 claim 4).
pub fn building_aim_point(w: &World, id: ObjectId) -> Pos {
    let p = w.reference_point(id);
    let Some(o) = w.obj(id) else { return p };
    let b = &w.rules().buildings[o.type_index as usize].building;
    // spec: S-0310 claim 4 (the primary fire pixel shift; else the turret anchor's when not zero; else none)
    let shift = match b.fire_pixel {
        Some(px) => Some(px),
        None if b.turret_anim != [0, 0] => Some(b.turret_anim),
        None => None,
    };
    match shift {
        Some([px, py]) => {
            let (dx, dy) = crate::combat::position::pixel_to_leptons(w, px, py);
            Pos::new(p.x + dx, p.y + dy, p.z)
        }
        None => p,
    }
}

/// The direction of a target from a point (S-0204 claim 8).
pub(crate) fn direction_to(w: &World, from: Pos, to: Pos) -> Facing {
    direction(
        i64::from(to.x - from.x),
        i64::from(to.y - from.y),
        w.rules().quirks.table_direction,
    )
}

/// The aiming answer (S-0310 claims 2, 4, 5): `None` when pointing.
fn facing_answer(w: &World, firer: ObjectId, weapon: WeaponId, t: Target) -> Option<Answer> {
    let o = w.obj(firer)?;
    let r = w.rules();
    let wr = r.combat.weapon(weapon);
    let target_pt = range::target_point(w, t);
    let homing = r.combat.projectile_of(weapon).is_some_and(|p| p.rot != 0);
    let tol = match o.kind {
        // spec: S-0310 claim 5 (infantry never get not facing)
        Kind::Infantry => return None,
        Kind::Vehicle => {
            let u = r.unit(o.kind, o.type_index)?;
            // spec: S-0310 claim 2 (an omni-fire weapon, and every weapon of a visceroid type, is never not facing)
            if wr.omni_fire || u.visceroid {
                return None;
            }
            // spec: S-0310 claim 2 (τ = 2048, or 4096 for a homing projectile)
            if homing { 4096 } else { 2048 }
        }
        Kind::Building => {
            let b = &r.buildings[o.type_index as usize];
            // spec: S-0310 claim 4 (a building that is not turreted fires in any direction)
            if !b.turret {
                return None;
            }
            if b.building.voxel_turret && r.quirks.voxel_turret_exact_aim {
                0 // spec: S-0310 claim 4 (a voxel turret: τ = 0)
            } else {
                2048 // spec: S-0310 claim 4 (a sprite turret: τ = 2048)
            }
        }
        // spec: S-0310 claim 5 (an aircraft: τ = 2048; its flight control exemptions belong to the aircraft specs)
        Kind::Aircraft => 2048,
        Kind::Terrain => return None,
    };
    let from = if o.kind == Kind::Building {
        building_aim_point(w, firer)
    } else {
        w.reference_point(firer)
    };
    let d = direction_to(w, from, target_pt);
    // spec: S-0310 F1 (ε = |δ(a, d)|, the magnitude of −32768 is 32768; pointing ⇔ ε ≤ τ)
    let eps = delta(aiming_facing(w, firer), d).abs();
    (eps > tol).then_some(Answer::NotFacing)
}

/// Whether a vehicle is "moving" for a weapon (S-0310 claim 7).
fn vehicle_moving(w: &World, firer: ObjectId, weapon: WeaponId) -> bool {
    let Some(o) = w.obj(firer) else { return false };
    let r = w.rules();
    let Some(u) = r.unit(o.kind, o.type_index) else {
        return false;
    };
    let wr = r.combat.weapon(weapon);
    let dest = o.dest.is_some();
    // spec: S-0310 claim 7 (the type lacks the mobile-fire flag and has a destination)
    if !u.combat.mobile_fire && dest {
        return true;
    }
    // spec: S-0310 claim 7 (the weapon lacks fire-while-moving: a destination; a balloon-hover type: its locomotor moving it)
    let balloon = u.raw.balloon_hover.unwrap_or(false);
    let moving_cond = if balloon {
        crate::occupancy::in_progress(w, firer)
    } else {
        dest
    };
    if !wr.fire_while_moving && moving_cond {
        return true;
    }
    // spec: S-0310 claim 7 (a spark- or fire-particle weapon with a destination)
    if (wr.spark || wr.fire_particles) && dest {
        return true;
    }
    // spec: S-0310 claim 7 (the vehicle has a temporal beam and a destination; attachment is irrelevant)
    has_temporal_beam(w, firer) && dest
}

/// Whether an object has a temporal beam (S-0310 claim 7, S-0514 Terms):
/// supplied at creation when its primary weapon's warhead is temporal. An
/// object is never elite at creation (S-0311 claim 11; a map's veterancy
/// is applied after it is created), so that primary is its type's slot 0,
/// not an elite replacement, and the beam stays when a promotion changes
/// the primary. A temporal secondary weapon alone supplies none. Gunner
/// hand-over (S-0308 claim 6) can also supply one; gunner transports are
/// not built, so no beam comes from it yet.
pub fn has_temporal_beam(w: &World, id: ObjectId) -> bool {
    // spec: S-0514 Terms and S-0310 claim 7 (a primary weapon whose warhead is temporal supplies a beam at creation; a gunner vehicle carries its first passenger's)
    crate::temporal::has_beam(w, id)
}

/// Whether an infantry unit is "moving" for a weapon (S-0310 claim 8).
fn infantry_moving(w: &World, firer: ObjectId, weapon: WeaponId) -> bool {
    let Some(o) = w.obj(firer) else { return false };
    // spec: S-0310 claim 8 and Terms (speed fraction above 0.1: it has taken a walking step and not yet stopped)
    let walking = o.walk.as_ref().is_some_and(|s| s.fraction);
    let interruptible = o
        .infantry
        .as_ref()
        .and_then(|s| s.seq)
        .is_none_or(|q| q.table().0);
    let dest = o.dest.is_some();
    // spec: S-0310 claim 8 (a destination and a current sequence that is not interruptible)
    if walking || (dest && !interruptible) {
        return true;
    }
    if let Some(fraction) = crate::jump::speed_fraction(w, firer) {
        // spec: S-0310 claim 8 and Constants (the infantry speed fraction above 0.1, compared in double precision) and S-0502 claims 6 and 20 (a jumpjet's fraction is v_c / S)
        if fraction.gt(glasnost_num::decimal::nearest_double("0.1")) {
            return true;
        }
        // spec: S-0310 claim 8 and S-0502 claim 20 (a jumpjet infantry type with the jumpjet-turn flag gets moving while its locomotor is moving now; grounded and hovering do not)
        if w.rules()
            .unit(o.kind, o.type_index)
            .is_some_and(|u| u.vehicle.jump_jet_turn)
            && crate::jump::moving_now(w, firer)
        {
            return true;
        }
    }
    // spec: S-0310 claim 8 (a fire-particle weapon while the unit has a destination)
    w.rules().combat.weapon(weapon).fire_particles && dest
}

/// Whether a target is gone or out of play (no object). A departed warp
/// member that the firer's temporal beam holds is still a target, off the
/// map.
fn target_exists(w: &World, firer: ObjectId, t: Target) -> bool {
    match t {
        Target::Object(x) => w.obj(x).is_some_and(|o| {
            o.health > 0
                // spec: S-0513 r89 claim 39a (The firer, Q-2317 answered: the firing check answers S-0514 claim 19's rearming for a firer whose beam is attached to the member, before and after its departure, unless a reason that outranks it holds) and S-0514 claim 19; S-0309 r95 claim 27 (the exception for a firer whose temporal beam is attached: rearming, unless an answer ranked above it holds, a deactivated firer's illegal among them); S-0513 r91 claim 39a (Q-2376 answered: the same for a building firer, unless one of the building's own answers ranked above it holds, and for a firer beyond its weapon's range of the old position, whatever the range)
                && (o.on_map
                    || (crate::superweapons::warp::departed(w, x)
                        && crate::temporal::holds(w, firer, x)))
        }),
        Target::Cell(_) => true,
    }
}

/// The illegal-target tests of S-0306 claim 23 (rank 4).
fn projectile_illegal(w: &World, weapon: WeaponId, t: Target) -> bool {
    let r = w.rules();
    let Some(p) = r.combat.projectile_of(weapon) else {
        return false;
    };
    match t {
        Target::Object(x) => {
            let Some(o) = w.obj(x) else { return true };
            let air = airborne_by_height(w, x);
            // spec: S-0306 claim 23 (an airborne target and a projectile that is not anti-air)
            if air && !p.aa {
                return true;
            }
            // Render layers other than the ground layer (flying, jumping, falling, digging) are not modelled: every unit is in the ground layer.
            // spec: S-0306 claim 23 (not an infantry, vehicle, aircraft or building, not airborne, projectile not anti-ground)
            o.kind == Kind::Terrain && !air && !p.ag
        }
        // spec: S-0306 claim 23 (a cell target, not airborne, and a projectile that is not anti-ground)
        Target::Cell(_) => !p.ag,
    }
}

/// The water and land rows of S-0306 claim 23: whether the firer's type's
/// naval-targeting or land-targeting mode refuses target `t`, whatever the
/// weapon and the distance. Our engine has no overlay targets; a terrain
/// object meets neither row.
fn targeting_mode_illegal(w: &World, firer: ObjectId, t: Target) -> bool {
    let tc = type_combat(w, firer);
    // spec: S-0306 claim 23 and Import mapping (LandTargeting, NavalTargeting: absent 0; any land mode other than 1 acts as 0 here)
    let land_mode = tc.map_or(0, |c| c.land_targeting);
    let naval_mode = tc.map_or(0, |c| c.naval_targeting);
    let land_of = |c| w.cells.cell(c).map_or(land::CLEAR, |x| x.land);
    // spec: S-0306 claim 23 (a cell whose land type is water or beach)
    let wet = |l: u8| l == land::WATER || l == land::BEACH;
    match t {
        // spec: S-0306 claim 23 (land row: land mode 1 and a cell whose land type is neither water nor beach)
        Target::Cell(c) => land_mode == 1 && !wet(land_of(c)),
        Target::Object(x) => {
            let Some(o) = w.obj(x) else { return false };
            // spec: S-0306 claim 23 (both rows need an infantry, vehicle, aircraft or building that is on the map and not airborne, S-0307 Terms; a terrain object meets neither)
            if o.kind == Kind::Terrain || !o.on_map || airborne_by_height(w, x) {
                return false;
            }
            let bridge = o.layer == Layer::Deck;
            if !bridge && wet(land_of(o.pos.cell())) {
                // spec: S-0306 claim 23 (water row: not on a bridge, on a water or beach cell, and the naval mode gives "none" by S-0308 table N)
                let underwater = type_combat(w, x).is_some_and(|c| c.underwater);
                match naval_mode {
                    // spec: S-0306 claim 23 and S-0308 table N (mode 0: an underwater target that is cloaking, cloaked or uncloaking)
                    0 => underwater && crate::cloak::not_uncloaked(w, x),
                    // spec: S-0306 claim 23 and S-0308 table N (mode 2: a target whose type is not underwater)
                    2 => !underwater,
                    // spec: S-0306 claim 23 and S-0308 table N (mode 6: every target)
                    6 => true,
                    _ => false,
                }
            } else {
                // spec: S-0306 claim 23 (land row: land mode 1 and a floor-bound object on a bridge or on a cell neither water nor beach)
                land_mode == 1
            }
        }
    }
}

/// Whether the weapon is a healing weapon (S-0310 Terms: negative damage).
fn healing(w: &World, weapon: WeaponId) -> bool {
    w.rules().combat.weapon(weapon).damage < 0
}

/// The health ratio of an object is at least 1 (S-0214 Terms, full health).
fn full_health(w: &World, x: ObjectId) -> bool {
    let Some(o) = w.obj(x) else { return true };
    let strength = crate::combat::damage::strength(w, x);
    o.health >= strength
}

/// The drain refusal of S-0608 claim 3 (a): the target is the building
/// the firer itself drains (its current, newest link; a stranded earlier
/// building is not refused by this part), whatever the weapon.
fn own_drained_building(w: &World, firer: ObjectId, x: ObjectId) -> bool {
    // spec: S-0608 claim 3 (a) (the target is the building that the firer itself drains, whatever the weapon; only its current building, not a stranded earlier one)
    crate::drain::partner(w, firer) == Some(x)
}

/// The drain refusals of S-0608 claim 3 (b) and (c) against object `x`: a
/// draining weapon against an object already being drained, by any drainer
/// (a stranded earlier building of the firer's own included), or whose type
/// is not drainable. No alliance or ownership test.
fn drain_weapon_refused(w: &World, drain: bool, x: ObjectId) -> bool {
    // spec: S-0608 claim 3 (b) and (c) (a draining weapon against an object already being drained, or whose type is not drainable)
    drain && (crate::drain::being_drained(w, x) || !crate::drain::drainable(w, x))
}

/// The firing check for the weapon in `slot` against `target`, with or
/// without the range test (S-0301 claim 9; S-0310 for the aiming answers).
pub fn check(
    w: &World,
    firer: ObjectId,
    target: Option<Target>,
    slot: usize,
    with_range: bool,
) -> Answer {
    answer(w, firer, target, slot, with_range, Ask::Every)
}

/// The own AI's lasting test (our design, PLAN §4.7): the firing check
/// without the range test and without the reasons of the moment that rank
/// ahead of the target tests, the materialising firer's *rearming* of rank
/// 4 and ranks 5 to 8 (the rearm countdown and the other rearming gates,
/// ammunition 0, the cloak; S-0301 claim 9). It answers whether the target
/// is worth an attack past this moment, so that a unit whose countdown
/// runs or that phases in does not take a target its weapon can never
/// hurt. The attack rule of S-0214 claim 6 (e) does not use it: since r78
/// it takes the ordinary check's answer ([`check`]), or with
/// `quirks.moment_reason_attack_cursor` off asks [`illegal_at_any_rank`].
pub fn check_lasting(w: &World, firer: ObjectId, target: Option<Target>, slot: usize) -> Answer {
    answer(w, firer, target, slot, false, Ask::Lasting)
}

/// Whether a reason that gives *illegal* holds for the weapon in `slot`
/// against `target`, at any rank, every other answer passed over: S-0214
/// claim 6 (e) with `quirks.moment_reason_attack_cursor` off. No range
/// test is asked (part (d) is the range part), and with no weapon in the
/// slot nothing is (part (e) holds for an actor with no weapon). Which
/// answers the switch passes over is under Q-2426 (interim: every answer
/// that is not *illegal*).
pub fn illegal_at_any_rank(
    w: &World,
    firer: ObjectId,
    target: Option<Target>,
    slot: usize,
) -> bool {
    answer(w, firer, target, slot, false, Ask::IllegalOnly) == Answer::Illegal
}

/// What a firing check asks.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Ask {
    /// Every reason, by S-0301 claim 9's ranks.
    Every,
    /// The own AI's lasting test: the materialising *rearming* and ranks 5
    /// to 8 passed over.
    Lasting,
    /// Only the reasons that give *illegal*, at any rank (S-0214 claim 6
    /// (e) with `quirks.moment_reason_attack_cursor` off; Q-2426 interim).
    IllegalOnly,
}

/// The firing check, with the reasons of the moment asked or passed over
/// as `ask` says.
fn answer(
    w: &World,
    firer: ObjectId,
    target: Option<Target>,
    slot: usize,
    with_range: bool,
    ask: Ask,
) -> Answer {
    // The reasons of ranks 5 to 8 and the materialising rearming are asked.
    let moment = ask == Ask::Every;
    // An answer other than illegal is passed over, so that an illegal reason ranked below it still answers.
    let pass = ask == Ask::IllegalOnly;
    let Some(o) = w.obj(firer) else {
        return Answer::Impossible;
    };
    let r = w.rules();
    let kind = o.kind;
    let weapon = slot_weapon(w, firer, slot);
    let target = target.filter(|t| target_exists(w, firer, *t));
    // spec: S-0301 claims 9 and 14 (a building's pending delayed shot: rearming at rank 4, yielding only to the building's own answers of S-0405 table T1 and ahead of every other reason, the absent-weapon and hidden-target impossible, the drain refusals, every row of S-0306 claim 23 and the later ranks; Q-1770 answered)
    // Passed over for illegal only: the building's own answers are asked again below.
    if !pass && kind == Kind::Building && o.combat.delayed_shot.is_some() {
        return crate::econ::power::firing_answer(w, firer).unwrap_or(Answer::Rearming);
    }
    // Ranks 1 and 3 (a vehicle playing death frames, melee-linked) need states that no topic of our engine sets yet.
    // spec: S-0301 claim 9 rank 2 and S-0506 claim 10 (a vehicle whose launching count is above 0: busy)
    if !pass && kind == Kind::Vehicle && crate::spawn::launching_count(w, firer) > 0 {
        return Answer::Busy;
    }
    // spec: S-0310 claim 11 and S-0301 r69 claim 9 (A materialising firer within rank 4: an infantry healer gets illegal when its target is absent, not infantry or at full health; the answer outranks every answer of the general part, the materialising and own-gripped-unit rearming among them, as one of the answers an infantry firer is asked first)
    if kind == Kind::Infantry
        && let Some(wp) = weapon
        && healing(w, wp)
    {
        let ok = matches!(target, Some(Target::Object(x))
            if w.obj(x).is_some_and(|t| t.kind == Kind::Infantry) && !full_health(w, x));
        if !ok {
            return Answer::Illegal;
        }
    }
    // The keep-acting illegal (S-0301 r69 claim 9: an infantry firer whose
    // type has the keep-acting flag, against a vehicle target melee-linked
    // to another object) is asked here, before the materialising rearming;
    // it needs S-0214's melee link, which no topic of our engine sets yet.
    // spec: S-0301 r69 claim 9 (A materialising firer within rank 4: rearming for every target and weapon; it yields only to the slave-bound firer's illegal, not in our engine, for vehicles to ranks 1 to 3 and the tube transit, and for infantry to the answers an infantry firer is asked first, above; it outranks every other rank-4 answer, among them the projectile refusals of S-0306 claim 23, the hidden-target impossible, the parasite warhead's illegal against a protected object, the empty slot's impossible and the own-gripped-unit rearming) and S-0207 claim 20 (f) (phasing in)
    // The own AI's lasting test passes over it as a reason of the moment
    // (our design). The attack rule asks it (S-0214 r78 claim 6 (e), Q-2356
    // answered), unless quirks.moment_reason_attack_cursor is off.
    if moment && crate::teleport::phasing_in(w, firer) {
        return Answer::Rearming;
    }
    // spec: S-0301 r69 claim 9 (Grip refusals within rank 4) and S-0607 r38 claim 11 (a firing check by the lifter against its own gripped unit answers rearming, with any weapon, ahead of every test of the target's layer, height, kind or armour and of every projectile test; of S-0301 claim 9's rank-4 answers only the materialising firer's rearming, the refusal of a firer bound to a slave master, which our engine does not have, and for an infantry lifter the answers an infantry firer is asked first precede it, all asked above)
    if !pass
        && weapon.is_some()
        && let Some(Target::Object(x)) = target
        && crate::lift::grips(w, firer, x)
    {
        return Answer::Rearming;
    }
    // spec: S-0301 claim 9 (Deactivated firers within rank 4) and S-0405 claim 20: a deactivated vehicle gets illegal for every target and weapon, a force-fire target included; the refusal yields to the slave-bound firer's illegal (not in our engine), the materialising firer's rearming and the own-gripped-target rearming (both above), and outranks every other rank-4 answer
    if kind == Kind::Vehicle && crate::econ::power::unit_disabled(w, firer) {
        return Answer::Illegal;
    }
    // spec: S-0608 claim 3 (Precedence relevant to drain: the own-current-building refusal follows the own-gripped-unit and materialisation rearming answers, both above, and precedes the held-temporal-target rearming, hidden-target impossible, absence-of-weapon and power-sensitive-weapon impossible, the other weapon's live-effect impossible and the conditional magnetic-wave rearming)
    if let Some(Target::Object(x)) = target
        && own_drained_building(w, firer, x)
    {
        return Answer::Illegal;
    }
    // spec: S-0301 claim 9 rank 4 (no weapon in the slot: impossible)
    let Some(weapon) = weapon else {
        return Answer::Impossible;
    };
    // spec: S-0405 claim 9 and T1 (a building's own answers, at rank 4)
    if kind == Kind::Building
        && let Some(a) = crate::econ::power::firing_answer(w, firer)
        && (!pass || a == Answer::Illegal)
    {
        return a;
    }
    let wr = r.combat.weapon(weapon);
    if let Some(t) = target
        && projectile_illegal(w, weapon, t)
    {
        // spec: S-0301 claim 9 rank 4 (most target tests give illegal)
        return Answer::Illegal;
    }
    // spec: S-0301 claim 9 rank 4 and S-0610 claims 3 and 6 (a berserk-causing warhead against a psionics-immune type or a bunkered vehicle; a berserk firer against a berserk-friendly type: illegal)
    if let Some(t) = target
        && crate::psychic::firing_refused(
            w,
            firer,
            r.combat.warhead_of(weapon).is_some_and(|h| h.psychedelic),
            t,
        )
    {
        return Answer::Illegal;
    }
    if let Some(Target::Object(x)) = target
        && crate::superweapons::protected(w, x)
    {
        // spec: S-0513 claim 27 (a protected object is an illegal target for a weapon whose warhead is a parasite warhead)
        let parasite = r.combat.warhead_of(weapon).is_some_and(|h| h.parasite);
        // spec: S-0513 claim 28 (departure: every player house takes the human rule; the neutral and special houses, and a test-only computer-controlled player house (glossary), get illegal against a protected object)
        let player = o
            .owner
            .and_then(|h| w.houses.get(usize::from(h.0)))
            .is_none_or(|h| h.human);
        if parasite || !player {
            return Answer::Illegal;
        }
    }
    // spec: S-0301 claim 9 rank 4 and S-0516 claim 24 (a target hidden from the firer's house: impossible when the mean weapon damage is above 0 or the target is not allied with the firer)
    if !pass
        && let Some(Target::Object(x)) = target
        && crate::cloak::hidden_from(w, x, o.owner)
        && (crate::combat::weapon::mean_damage(w, firer) > 0
            || !w.allied(o.owner, w.obj(x).and_then(|t| t.owner)))
    {
        return Answer::Impossible;
    }
    // spec: S-0301 claim 9 rank 4 and S-0514 claim 19 (a firer that already holds this very target with a temporal weapon: rearming)
    if !pass
        && let Some(Target::Object(x)) = target
        && r.combat.warhead_of(weapon).is_some_and(|h| h.temporal)
        && crate::temporal::holds(w, firer, x)
    {
        return Answer::Rearming;
    }
    if let Some(Target::Object(x)) = target
        && drain_weapon_refused(w, wr.drain, x)
    {
        // spec: S-0608 claim 3 (Precedence relevant to drain: the draining-weapon refusals (b) and (c) follow the held-temporal-target, hidden-target, absence-of-weapon and power-sensitive-weapon answers and precede the berserk-warhead target refusals, projectile eligibility, the conditional magnetic-wave refusal, the other weapon's live-effect refusal and the empty-spawner rearming; their order among reasons that all answer illegal is not observable)
        return Answer::Illegal;
    }
    // spec: S-0301 r69 claim 9 (Targets with a superweapon warp pending within rank 4) and S-0513 r62 claim 39 (Q-2213 answered: a target that has a superweapon warp pending, from the launch until its departure, answers illegal for every weapon, a temporal weapon included; the test is on the pending warp itself, apart from being erased, so another firer's beam on the member does not change it; rank 4, after every other rank-4 reason, the held-temporal-target rearming of S-0514 claim 19 above included, except the spawner-weapon answers, the other weapon's live-effect impossible, every row of S-0306 claim 23 and the magnetic-beam rearming against another target, which it outranks, and ahead of ranks 5 and 7 to 16; the earlier projectile and berserk rows also answer illegal, so their order before it is not observable)
    // The sinking, falling, deploy-transition, seized-firer and S-0517
    // magnetic-beam refusals below also answer illegal, so their place after
    // this reason is not observable.
    if let Some(Target::Object(x)) = target
        && crate::superweapons::warp_pending_or_leaving(w, x)
    {
        return Answer::Illegal;
    }
    // spec: S-0301 claim 9 rank 4 and S-0614 claim 8 (a spawner weapon of a held mobile firer: impossible, whatever its organic type)
    if !pass && crate::parasite::held_spawner(w, firer, weapon) {
        return Answer::Impossible;
    }
    // spec: S-0301 claim 9 rank 4 and S-0506 claim 10 (a spawner weapon none of whose slot objects is alive: rearming)
    if !pass && wr.spawner && !crate::spawn::any_alive(w, firer) {
        return Answer::Rearming;
    }
    // spec: S-0301 claim 9 rank 4 and S-0504 claim 5 (a sinking firer: illegal; earlier reasons keep their precedence)
    if crate::naval::sinking(w, firer) {
        return Answer::Illegal;
    }
    // spec: S-0301 claim 9 rank 4 and S-0511 claim 10 (a falling firer: illegal)
    if crate::fall::falling(w, firer) {
        return Answer::Illegal;
    }
    // spec: S-0301 claim 9 rank 4 and S-0517 claim 11 (a vehicle in a deploy or undeploy transition: illegal for every weapon and target; higher-ranked refusals win)
    if kind == Kind::Vehicle && crate::econ::inplace::refuses_fire(w, firer) {
        return Answer::Illegal;
    }
    // spec: S-0517 claim 11 (a magnetic beam is illegal against a target vehicle in transition or an in-place deploying vehicle whose current behaviour is unload)
    if wr.mag_beam
        && let Some(Target::Object(x)) = target
        && w.obj(x).is_some_and(|t| t.kind == Kind::Vehicle)
        && crate::econ::inplace::beam_refused(w, x)
    {
        return Answer::Illegal;
    }
    // spec: S-0306 claim 23 (the water and land rows: illegal at rank 4, after the other weapon's live-effect impossible and every earlier rank-4 answer, ahead of the magnetic-beam rearming; a building's pending delayed shot comes ahead of them, at the top of this check: S-0301 claim 9)
    if let Some(t) = target
        && targeting_mode_illegal(w, firer, t)
    {
        // spec: S-0306 claim 23 and S-0301 claim 5 (the other weapon's live effect: impossible, ahead of the two rows)
        if !pass && crate::emitters::other_effect_alive(w, firer, slot) {
            return Answer::Impossible;
        }
        return Answer::Illegal;
    }
    // spec: S-0301 claim 9 rank 4 and S-0607 claims 10 and 11 (a seized firer: illegal; a magnetic-beam firer gripping a unit with a live wave against a target that is neither that unit nor a building: rearming, after the target's layer and projectile tests, so against an airborne target the beam cannot hit that refusal wins; placed last among the rank-4 answers)
    if let Some(a) = crate::lift::firing_answer(w, firer, wr.mag_beam, target)
        && (!pass || a == Answer::Illegal)
    {
        return a;
    }
    // spec: S-0301 claim 5 (a live effect of the kind of the asked weapon's other weapon: impossible, rank 4) and S-0616 claim 1 (the gate applies while the emitter is held)
    if !pass && crate::emitters::other_effect_alive(w, firer, slot) {
        return Answer::Impossible;
    }
    let t = w.tick;
    let c = &o.combat;
    // Ranks 5 to 8, the rearming, out-of-ammunition and cloaked answers that come before illegal, are passed over by the own AI's lasting test (our design) and, with quirks.moment_reason_attack_cursor off, by S-0214 claim 6 (e), so the target tests of rank 9 answer there; the attack rule asks them otherwise (S-0214 r78 claim 6 (e))
    if moment {
        // spec: S-0301 claim 13 (the sync tick: the counter equals the sync frame for k mod B)
        let sync = sync_tick(w, firer, slot, weapon);
        // spec: S-0301 claim 1 and claim 9 rank 5 (the countdown runs below t + D, except in the sync tick)
        if t < c.rearm_until && !sync {
            return Answer::Rearming;
        }
        // spec: S-0301 claim 5 (a live effect of the asked weapon's own kind: rearming, rank 5)
        if crate::emitters::own_effect_alive(w, firer, weapon) {
            return Answer::Rearming;
        }
        // spec: S-0301 claim 13 (with a sync frame, the primary weapon gets rearming until the counter equals it)
        if slot == 0 && gate_closed(w, firer, weapon) {
            return Answer::Rearming;
        }
        // spec: S-0518 claim 6 (a supporter's pending beam: rearming; a building's pending delayed shot is answered at rank 4 above, S-0301 claim 9)
        if c.support_beam.is_some() {
            return Answer::Rearming;
        }
        // spec: S-0301 claim 9 rank 7 (ammunition 0)
        if c.ammo == 0 {
            return Answer::OutOfAmmo;
        }
        // spec: S-0301 claim 9 rank 8 and claim 12 (a weapon that needs the firer uncloaked: cloaked; an aircraft only while fully cloaked)
        if crate::cloak::machine::needs_uncloak(w, firer, wr.decloak_to_fire) {
            return Answer::Cloaked;
        }
    }
    if let Some(tg) = target {
        // spec: S-0301 claim 9 rank 9 (a seeker type answers out of range, whatever the distance)
        if !pass
            && kind == Kind::Vehicle
            && r.unit(kind, o.type_index)
                .is_some_and(|u| u.vehicle.hunter_seeker)
        {
            return Answer::OutOfRange;
        }
        // spec: S-0301 claim 9 rank 9 and S-0601 claims 3 and 4 (a takeover warhead whose takeover test fails: illegal; a passing test goes on to the later tests, the zero-versus test included)
        if crate::mind::firing_illegal(w, firer, weapon, tg) {
            return Answer::Illegal;
        }
        // spec: S-0304 claim 1 and S-0301 claim 9 rank 9 (a versus value of exactly zero makes a unit or building illegal)
        if let Target::Object(x) = tg
            && w.obj(x).is_some_and(|ob| ob.kind != Kind::Terrain)
            && versus(w, weapon, tg).is_some_and(|v| v.eq_num(F64::ZERO))
        {
            return Answer::Illegal;
        }
        // spec: S-0301 claim 9 rank 9 and S-0614 claim 5 (the parasite tests of a latching weapon against a mobile unit)
        if let Target::Object(x) = tg
            && w.obj(x).is_some_and(|ob| ob.kind.is_mobile())
            && crate::parasite::latching(w, weapon)
            && crate::parasite::shot_illegal(w, firer, x)
        {
            return Answer::Illegal;
        }
        // spec: S-0301 claim 9 rank 9 and S-0519 claim 3 (a bomb-planting weapon against a carrier, a disarm weapon against a target with no bomb: illegal)
        if crate::bomb::illegal_target(w, weapon, tg) {
            return Answer::Illegal;
        }
        // spec: S-0301 claim 9 rank 9 and S-0504 claim 5 (a sinking target, among the late target tests: illegal)
        if let Target::Object(x) = tg
            && crate::naval::sinking(w, x)
        {
            return Answer::Illegal;
        }
    }
    // spec: S-0301 claim 9 rank 9 and S-0614 claim 8 (a held organic firer: illegal)
    if crate::parasite::held_organic(w, firer) {
        return Answer::Illegal;
    }
    // spec: S-0301 claim 9 rank 10 (the range test, when asked for)
    if with_range && !range::in_range(w, firer, Some(weapon), target, None) {
        return Answer::OutOfRange;
    }
    if kind == Kind::Vehicle {
        let u = r.unit(kind, o.type_index).expect("vehicle type");
        // spec: S-0301 claim 9 rank 11 and S-0310 claim 9 (must deploy: a deploy-to-fire vehicle whose own cell fails S-0517 claim 14's cell test, when no reason of ranks 1 to 10 holds; it outranks moving, still turning and not facing)
        if !pass && u.vehicle.deploy_to_fire && !crate::econ::inplace::deploy_test(w, firer) {
            return Answer::MustDeploy;
        }
        // spec: S-0310 claim 11 and S-0301 claim 9 rank 12 (a vehicle healer: illegal when the target is absent, does not count as a vehicle, or is at full health)
        if healing(w, weapon) {
            let ok = matches!(target, Some(Target::Object(x))
                if crate::targeting::counts_as_vehicle(w, x) && !full_health(w, x));
            if !ok {
                return Answer::Illegal;
            }
        }
        // spec: S-0301 claim 9 rank 13 (moving)
        if vehicle_moving(w, firer, weapon) {
            return Answer::Moving;
        }
        let homing = r.combat.projectile_of(weapon).is_some_and(|p| p.rot != 0);
        // spec: S-0310 claim 3 (the turret was turning at the end of the previous tick; not for a homing projectile; never without a turret or with a spinning one)
        if u.turret && !u.turret_spins && !homing && o.turret.turning(t.wrapping_sub(1)) {
            return Answer::StillTurning;
        }
    }
    if kind == Kind::Infantry && infantry_moving(w, firer, weapon) {
        // spec: S-0310 claim 8 (infantry moving)
        return Answer::Moving;
    }
    // spec: S-0501 claim 19 (a passenger carrier that now carries none answers out of ammunition)
    if crate::air::empty_passenger_carrier(w, firer) {
        return Answer::OutOfAmmo;
    }
    // spec: S-0501 claim 19 (a fighter never answers not facing)
    let fighter = crate::air::is_aircraft(w, firer) && crate::air::fighter(w, firer);
    if let Some(tg) = target
        && !fighter
        && let Some(a) = facing_answer(w, firer, weapon, tg)
    {
        // spec: S-0301 claim 9 rank 15 (not facing)
        return a;
    }
    Answer::CanFire
}

/// The sync tick of claim 13 of S-0301: the firing-frame counter runs and
/// equals the sync frame that applies.
pub fn sync_tick(w: &World, firer: ObjectId, slot: usize, weapon: WeaponId) -> bool {
    if slot != 0 {
        return false;
    }
    let Some(s) = sync_frame(w, firer, weapon) else {
        return false;
    };
    w.obj(firer).and_then(|o| o.combat.firing_frame) == Some(s)
}

/// The sync frame that applies (S-0301 claim 13), if any.
fn sync_frame(w: &World, firer: ObjectId, weapon: WeaponId) -> Option<i32> {
    let o = w.obj(firer)?;
    if o.kind != Kind::Vehicle {
        return None;
    }
    let u = w.rules().unit(o.kind, o.type_index)?;
    if u.vehicle.firing_frames <= 0 {
        return None;
    }
    let b = w.rules().combat.weapon(weapon).burst;
    // spec: S-0301 claim 13 (k mod B 0 or 1: that value's sync frame; 2 or more: none; −1 gives none)
    let s = match o.combat.burst.rem_euclid(b) {
        0 => u.vehicle.sync_frames[0],
        1 => u.vehicle.sync_frames[1],
        _ => -1,
    };
    (s != -1).then_some(s)
}

/// The firing-frame gate (S-0301 claim 13): with a sync frame that applies,
/// a running counter that does not equal it holds the primary weapon.
fn gate_closed(w: &World, firer: ObjectId, weapon: WeaponId) -> bool {
    let Some(s) = sync_frame(w, firer, weapon) else {
        return false;
    };
    matches!(w.obj(firer).and_then(|o| o.combat.firing_frame), Some(k) if k != s)
}
