//! Behaviour processing (S-0215): the per-tick start, readiness, falling
//! back to rest and the processings of the general-purpose behaviours.
//!
//! Seams for milestone M4 (combat): the target-acquisition specs are not
//! written, so no automatic search runs (claims 26, 27 and 33 see "may not
//! search"), and weapon ranges and rearm times are unknown (the close-range
//! shortening of F3 applies only through the infantry close-range flag; no
//! actor is ever rearming). Approaching a target is the approach step of
//! [`crate::approach`] (claim 37), whose position search (S-0307) finds
//! nothing yet. *Capture*, *sabotage* and *eaten* share claim 28's table
//! ([`process_capture`]). A behaviour whose processing is not built yet is
//! processed as a behaviour without a processing rule; that is recorded as
//! a note on the world.
//!
//! Gates (claim 35): a gate's *open* runs its stages here, the door moves at
//! every turn of the gate before its per-tick start and processing (claim
//! 35 (d)), [`gate_open`] gives the passage state that S-0209's gate rows
//! read, and [`gate_met`] is the effect of the *closed friendly gate*
//! outcome, for the entry rules to call.
//!
//! The rules that S-0215 leaves to the harvest cycle, power and the other
//! M5a topics call [`crate::seams`].

use glasnost_map::CellXY;
use glasnost_num::geom::root_floor;

use crate::behaviour::Door;
use crate::infantry::{self, Seq};
use crate::occupancy;
use crate::rules::{Behaviour, Kind};
use crate::world::{AttackMoveMemory, Layer, ObjectId, Pos, Target, World};

/// The fixed delay of a behaviour without a processing rule (S-0215 claim 4).
// spec: S-0215 claim 4 (nothing happens; a delay of exactly 450 ticks, no draw)
const NO_RULE_DELAY: i32 = 450;

/// Whether the actor is moving on this tick for readiness (S-0215 claim
/// 11), read from its current movement state, not from a record of
/// displacement earlier in the update. A vehicle with the driving
/// locomotor has the claim's own predicate ([`drive_moving`]); every other
/// actor is moving while its locomotor is moving it.
fn moving(w: &World, id: ObjectId) -> bool {
    if driving(w, id) {
        return drive_moving(w, id);
    }
    if w.obj(id).is_some_and(|o| o.kind == Kind::Vehicle)
        && crate::lift::current_locomotor(w, id) == Some(crate::rules::Locomotor::Hover)
    {
        // spec: S-0215 claim 11 (the vehicle's current locomotor's moving-now report) and S-0503 claim 19 and Terms (a hover is held back while it is moving now: a destination or heading point and a throttle that is not 0; a tick with throttle 0 and the heading point present is not moving now)
        return crate::hover::moving_now(w, id);
    }
    // spec: S-0215 claim 11 (other locomotors: the actor is moving on this tick while its locomotor moves it; the driving predicate is the driving locomotor's only)
    // spec: S-0107 Terms (*is moving now*, true while the unit is physically moving on this tick, differs from *has movement in progress*, which S-0417 claim 17 gives a walker waiting for its search; readiness reads the first, occupancy::moving_on_tick)
    occupancy::moving_on_tick(w, id)
}

/// Whether `id` is a vehicle with the driving locomotor, by type or
/// temporarily (S-0417 claim 5).
fn driving(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if o.kind != Kind::Vehicle {
        return false;
    }
    // spec: S-0215 claim 11 (revision 68: the vehicle's current locomotor is asked; the predicate is for a vehicle whose current locomotor is the driving one, a ship, hovering, jumpjet or other locomotor is not covered; S-0417 claim 5: a vehicle that drives temporarily has the driving locomotor); S-0607 claim 20 (a seized unit uses its temporary jumpjet movement, not its saved driving movement) and claims 22 and 23 (a driving stand-in is its current locomotor)
    crate::lift::current_locomotor(w, id) == Some(crate::rules::Locomotor::Drive)
}

/// "Moving" for the readiness of a vehicle whose current locomotor is the
/// driving one (S-0215 claim 11, revision 67, which answers Q-1160 and
/// Q-1161; revision 68 asks the current locomotor): its
/// body is still turning, for any reason; or all three of: its locomotor
/// is moving it in S-0206's sense, it has a heading point (the track
/// target of the track it is on, S-0207 Terms) and its integer movement
/// step *s* (S-0207 claim 2) is above 0. A retained locomotor
/// destination, a stored path or a turn in place alone does not make it
/// moving.
fn drive_moving(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    // spec: S-0215 claim 11 (revision 67: a body still turning, for any reason, is moving; a turn ends by tick number, S-0206 claim 1, so it is no longer turning at either start of tick t0 + Θ)
    if o.body.turning(w.tick) {
        return true;
    }
    let Some(track) = o.drive.as_ref().and_then(|d| d.track.as_ref()) else {
        // spec: S-0215 claim 11 (revision 67: no heading point, so not moving without a body turn, whatever destination or path steps it keeps; S-0207 Terms: a drive vehicle that stands still, turns in place before a step, waits or is between two tracks has no heading point)
        return false;
    };
    // spec: S-0215 claim 11 (revision 67: S-0206's locomotor moving the vehicle: a locomotor destination, or a next point whose map x and y differ from the vehicle's, height ignored)
    let locomotor_moving = crate::drive::goal_present(w, id) || track.target != (o.pos.x, o.pos.y);
    let Some(u) = w.rules().unit(o.kind, o.type_index) else {
        return false;
    };
    // spec: S-0215 claim 11 (revision 67: and the integer movement step s of S-0207 claim 2 strictly above 0, the progress earned per tick and not the speed fraction)
    locomotor_moving && crate::drive::progress(w, id, u) > 0
}

/// Ready to change (S-0215 claim 11).
pub fn ready(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let cur = o.beh.current;
    // spec: S-0215 claim 11 (sticky or rescue is never ready, every kind)
    if matches!(cur, Some(Behaviour::Sticky | Behaviour::Rescue)) {
        return false;
    }
    match o.kind {
        Kind::Infantry => {
            // spec: S-0215 claim 11 (infantry: falling, for example from a parachute; measurable with a fresh paratrooper, scenario B44: its queued rest does not start in the air)
            if o.fall.falling {
                return false;
            }
            // spec: S-0215 r118 claim 11 (infantry: in the middle of firing, which is S-0107 claim 11's attack state and nothing else; Q-2566 answered)
            if o.infantry.as_ref().is_some_and(|s| s.attack) {
                return false;
            }
            let seq = o.infantry.as_ref().and_then(|s| s.seq);
            // spec: S-0215 claim 11 (infantry: moving while not in guard, hunt or attack without a target)
            let excused = matches!(cur, Some(Behaviour::Guard | Behaviour::Hunt))
                || (cur == Some(Behaviour::Attack) && o.target.is_none());
            if moving(w, id) && !excused {
                return false;
            }
            // spec: S-0215 claim 11 (infantry: a current sequence that is not interruptible)
            !seq.is_some_and(|q| !q.table().0)
        }
        Kind::Vehicle => {
            // spec: S-0215 claim 11 (vehicles: moving and not below the surface, except the three cases)
            let excused = o.beh.queued == Some(Behaviour::Enter)
                || cur == Some(Behaviour::Guard)
                || (cur == Some(Behaviour::Attack) && o.target.is_none())
                || (o.beh.queued == Some(Behaviour::Guard) && o.beh.allowance);
            let below = o.pos.z < w.surface_at(o.pos.x, o.pos.y);
            // spec: S-0215 claim 11 (letting out under its own unload processing) and S-0402 claim 11 (a harvester in the unloading state is not ready to change)
            if crate::econ::harvest::in_unloading_state(w, id) {
                return false;
            }
            // spec: S-0215 claim 11 (in the middle of letting its passengers out under its own unload processing, S-0507 table U)
            if crate::transport::letting_out(w, id) {
                return false;
            }
            // spec: S-0215 claim 11 (the two vehicle-factory conditions)
            if crate::econ::release::holds_back(w, id) {
                return false;
            }
            // spec: S-0517 claim 8 (a vehicle deploying or undeploying is not ready to change)
            if !crate::econ::inplace::ready(w, id) {
                return false;
            }
            !(moving(w, id) && !below && !excused)
        }
        Kind::Building => o.beh.ready_mark,
        // spec: S-0501 claim 31 (an aircraft's readiness to change)
        Kind::Aircraft if crate::air::is_aircraft(w, id) => crate::air::ready(w, id),
        Kind::Aircraft | Kind::Terrain => false,
    }
}

/// The standard delay D = D_r + U(0, 2) (S-0215 F2), the draw last.
pub(crate) fn standard_delay(w: &mut World, id: ObjectId) -> i32 {
    let dr = rate_delay(w, id);
    let half = w.rules().quirks.ranged_draw_half_span;
    // spec: S-0215 F2 and F0 (D = D_r + U(0, 2), wrapping 32-bit)
    dr.wrapping_add(w.rng.ranged(0, 2, half))
}

/// D_r of the behaviour current now (S-0215 claim 6, F1).
fn rate_delay(w: &World, id: ObjectId) -> i32 {
    let cur = w.obj(id).and_then(|o| o.beh.current);
    match cur {
        Some(b) => w.rules().behaviour(b).rate_delay,
        // A current behaviour of none has no rate; it takes the claim 4 delay instead.
        None => NO_RULE_DELAY,
    }
}

/// The close-range condition of S-0215 F3 for an actor with a target.
fn close_range(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let Some(t) = o.target else { return false };
    let r = w.rules();
    // spec: S-0215 claim 23 and F3 (an infantry type with the close-range flag, or a primary weapon range of at most 512)
    let flag = o.kind == Kind::Infantry
        && r.unit(o.kind, o.type_index)
            .is_some_and(|u| u.raw.close_range.unwrap_or(false));
    let short = crate::combat::weapon::slot_weapon(w, id, 0)
        .is_some_and(|x| r.combat.weapon(x).range <= 512); // spec: S-0215 F3 (a primary weapon range of at most 512 leptons)
    if !flag && !short {
        return false;
    }
    let tp = target_pos(w, t);
    let dx = i64::from(o.pos.x - tp.x);
    let dy = i64::from(o.pos.y - tp.y);
    // spec: S-0215 F3 and F4 (282 ≤ ρ₂ ≤ 768, ρ₂ the table-root 2-D distance)
    let rho = root_floor(
        (dx * dx + dy * dy) as u64,
        r.quirks.table_root_distance,
        r.mode,
    );
    (282..=768).contains(&rho) // spec: S-0215 F3 (282 ≤ ρ₂ ≤ 768)
}

/// A target's position: an object's position, or a cell's centre at its
/// height (S-0215 F4).
pub(crate) fn target_pos(w: &World, t: Target) -> Pos {
    match t {
        Target::Object(x) => w.obj(x).map_or(Pos::default(), |o| o.pos),
        Target::Cell(c) => {
            let raised = w.rules().quirks.negative_level_floor_raised;
            // spec: S-0215 F4 (a cell guard spot: the cell's centre at the cell's height, B(ℓ) of S-0106 F1)
            let z = w.cells.cell(c).map_or(0, |cell| {
                crate::surface::surface_height(i32::from(cell.level), 0, 0, 0, raised)
            });
            Pos::centre(c, z)
        }
    }
}

/// Falling back to rest (S-0215 claims 13 to 15). `placement` marks the
/// placement fall-back.
pub fn fall_back(w: &mut World, id: ObjectId, placement: bool) {
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else { return };
    if !o.kind.is_mobile() {
        return;
    }
    // spec: S-0501 claim 18 and table T2 (the first three rows come before the common effects)
    // spec: S-0514 claim 14 (revision 5: an aircraft early exit skips the common effects and so causes no release)
    if crate::air::fall_back_early(w, id) {
        return;
    }
    // spec: S-0215 claim 13 (the common effects at most once per actor per tick)
    if o.beh.last_common_effects != Some(t) {
        if let Some(ob) = w.obj_mut(id) {
            ob.beh.last_common_effects = Some(t);
        }
        // spec: S-0514 claim 14 and S-0215 claim 13 (revision 5: an actor holding an attached temporal beam, whichever weapon is selected, lets its victim go with the common effects of falling back to rest, at most once per tick; clearing a target, the stop order, and processing stop or sleep are not this event)
        crate::temporal::let_go(w, id);
        // spec: S-0309 claim 23 (b) (the attack-move refresh is among the common effects, at most once per tick)
        // spec: S-0215 claim 13 (attack-move memory is refreshed before the table's effects; the rows judge the state it leaves)
        refresh_attack_move(w, id);
        // spec: S-0215 claim 13 row 1 and S-0607 claim 20 (a seized unit, set down and not moving, gets its own locomotor back; revision 14: the same test is also made at the end of each of its turns, `lift::end_of_turn`)
        let lifted = crate::lift::restore_at_rest(w, id);
        // spec: S-0607 claim 12 (row 3: a vehicle lifter falling back to rest lets go, unless an override is in effect or wait is current)
        crate::lift::lifter_rest(w, id);
        // spec: S-0215 claim 13 row 1 (a temporarily replaced locomotor is given back; S-0417 claim 5)
        let restored = lifted || crate::econ::release::restore_locomotor(w, id);
        // The approach row is a seam: our engine keeps no approach destinations list.
        // spec: S-0215 claim 13 (the route row: it follows a route, the approach row does not apply and it is not in a patrol already begun: it issues the action of the next route point as an order, taking effect at the order boundary); our engine has no patrol progress, so no patrol has begun
        crate::routes::fall_back(w, id);
        // spec: S-0215 claim 13 (an infantry unit that is not a harvesting slave, guard spot on another cell, not given its locomotor back: it becomes the destination)
        let Some(o2) = w.obj(id).cloned() else { return };
        if o2.kind == Kind::Infantry
            && !restored
            && !crate::seams::is_harvesting_slave(w, id)
            && let Some(gs) = o2.guard_spot
        {
            let gc = match gs {
                Target::Cell(c) => Some(c),
                Target::Object(x) => w.obj(x).map(|g| g.pos.cell()),
            };
            if gc.is_some_and(|c| c != o2.pos.cell())
                && let Some(ob) = w.obj_mut(id)
            {
                // spec: S-0215 claim 13 (Q-0202: the guard spot replaces a resumed attack-move destination)
                ob.dest = Some(gs);
                if ob.beh.current != Some(Behaviour::AreaGuard) {
                    // spec: S-0215 claim 13 (unless area guard is current, it queues move without an immediate start, claim 7, and forgets the guard spot)
                    ob.beh.queue(Some(Behaviour::Move));
                    ob.guard_spot = None;
                }
                if placement {
                    // spec: S-0208 claim 20 (the placement fall-back supplies a destination to a teleporting infantry unit: placement-time clearing before the landing choice)
                    crate::teleport::placement_destination(w, id);
                }
                // spec: S-0107 claim 15 (a destination assigned: the destination-driven get-up)
                crate::infantry::destination_assigned(w, id, o2.dest);
            }
        }
    }
    match o.kind {
        Kind::Infantry => fall_back_infantry(w, id, placement),
        Kind::Vehicle => fall_back_vehicle(w, id, placement),
        // spec: S-0501 claim 18 and table T2 (the aircraft rows)
        Kind::Aircraft => crate::air::fall_back_rows(w, id),
        _ => {}
    }
}

/// The no-threat fall-back on damage (S-0215 claim 16): called for a hit
/// from a known attacker that lowered the actor's health and reported
/// unchanged, now yellow or now red, after the hit's retaliation, so an
/// actor that retaliated is tested on *attack* (claim 10).
/// A mobile actor whose current behaviour has the no-threat property and
/// not the zombie property falls back to rest at once (claims 13 to 15;
/// S-0501 claim 18 for an aircraft). The rest tables then decide, so the
/// fall-back queues what they give, without an immediate start.
pub fn no_threat_fall_back(w: &mut World, id: ObjectId) {
    if no_threat_applies(w, id) {
        // spec: S-0215 claim 16 (it falls back to rest at once, claims 13 to 15; an ordinary fall-back, not the placement one)
        fall_back(w, id, false);
    }
}

/// Whether the no-threat fall-back of S-0215 claim 16 would apply to `id`
/// now: judged on the behaviour it sees. A berserk hit judges it before the
/// berserk entry queues *hunt* (S-0302 claim 10).
pub fn no_threat_applies(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    // spec: S-0215 claim 16 (a mobile actor; not a team member: no house of our engine has teams, Terms)
    if !o.kind.is_mobile() {
        return false;
    }
    // spec: S-0215 claim 16 and claim 1 (the current behaviour, or with none current the queued one)
    let Some(b) = o.beh.seen() else { return false };
    let p = w.rules().behaviour(b);
    // spec: S-0215 claim 16 and claim 17 (the no-threat property, and no fall-back while the zombie property holds)
    p.no_threat && !p.zombie
}

/// The guard-area veterancy ability at the actor's current rank (S-0311 claim 3).
fn guard_area_ability(w: &World, id: ObjectId) -> bool {
    crate::combat::veterancy::has_ability(w, id, crate::combat::data::Ability::GuardArea)
}

/// Claim 14 (infantry).
fn fall_back_infantry(w: &mut World, id: ObjectId, placement: bool) {
    let r = w.rules_arc();
    let Some(o) = w.obj(id).cloned() else { return };
    let Some(u) = r.unit(o.kind, o.type_index) else {
        return;
    };
    let cur = o.beh.current;
    // spec: S-0215 claim 14 (nothing is queued while the current behaviour is patrol or area guard)
    let blocked = matches!(cur, Some(Behaviour::Patrol | Behaviour::AreaGuard));
    let again = matches!(cur, Some(Behaviour::Sabotage | Behaviour::Capture));
    let queue = |w: &mut World, b: Behaviour| {
        if !blocked && let Some(ob) = w.obj_mut(id) {
            ob.beh.queue(Some(b));
        }
    };
    // spec: S-0215 claim 14 row 1 (attack-move memory, or wait: nothing changes)
    if o.attack_move.is_some() || cur == Some(Behaviour::Wait) {
        return;
    }
    // spec: S-0215 claim 14 row 2 (a target: attack, or the current sabotage or capture again)
    if o.target.is_some() {
        queue(
            w,
            if again {
                cur.unwrap_or(Behaviour::Attack)
            } else {
                Behaviour::Attack
            },
        );
        return;
    }
    let mut dest = o.dest;
    if dest.is_none() && !o.queued_dests.is_empty() {
        // spec: S-0215 claim 14 row 3 (the first queued destination becomes the destination)
        if let Some(ob) = w.obj_mut(id) {
            let first = ob.queued_dests.remove(0);
            ob.dest = Some(first);
            if placement {
                // spec: S-0208 claim 20 (the placement fall-back supplies a destination to a teleporting infantry unit: placement-time clearing before the landing choice)
                crate::teleport::placement_destination(w, id);
            }
            // spec: S-0107 claim 15 (a queued destination assigned: the destination-driven get-up)
            crate::infantry::destination_assigned(w, id, None);
            // spec: S-0208 claim 19 (a teleporting infantry unit's refused landing removes the destination)
            dest = w.obj(id).and_then(|o| o.dest);
        }
    }
    // spec: S-0215 claim 14 row 4 (a destination: move, or the current sabotage or capture again)
    if dest.is_some() {
        queue(
            w,
            if again {
                cur.unwrap_or(Behaviour::Move)
            } else {
                Behaviour::Move
            },
        );
        return;
    }
    // spec: S-0215 claim 14 rows 5 and 6 (guard or area guard; zombie or paralysed: nothing changes)
    if matches!(cur, Some(Behaviour::Guard | Behaviour::AreaGuard)) {
        return;
    }
    if cur.is_some_and(|b| r.behaviour(b).zombie || r.behaviour(b).paralysed) {
        return;
    }
    let human = o.owner.is_none_or(|h| w.houses[usize::from(h.0)].human);
    if !human {
        // The neutral and special houses keep rows 7 to 10 (S-0215 Terms, houses in our engine); no unit is a team member.
        let skill = o.owner.map_or(0, |h| w.houses[usize::from(h.0)].skill);
        let armed = u.primary.is_some();
        let b = if skill < r.general.iq_guard_area {
            // spec: S-0215 claim 14 row 7 (not human-controlled, skill below the area-guard threshold: guard)
            Behaviour::Guard
        } else if crate::seams::is_harvesting_slave(w, id) {
            // spec: S-0215 claim 14 row 8 (a harvesting slave: guard)
            Behaviour::Guard
        } else if armed || u.engineer || u.vehicle_thief {
            // spec: S-0215 claim 14 row 9 (armed, an engineer or a vehicle thief: area guard)
            Behaviour::AreaGuard
        } else {
            // spec: S-0215 claim 14 row 10 (otherwise guard)
            Behaviour::Guard
        };
        queue(w, b);
        return;
    }
    // spec: S-0215 claim 14 rows 11 and 12 (a player house: area guard with the ability or the type default, else guard)
    if guard_area_ability(w, id) || u.default_to_guard_area {
        queue(w, Behaviour::AreaGuard);
    } else {
        queue(w, Behaviour::Guard);
    }
}

/// Claim 15 (vehicles).
fn fall_back_vehicle(w: &mut World, id: ObjectId, placement: bool) {
    let r = w.rules_arc();
    let Some(o) = w.obj(id).cloned() else { return };
    let Some(u) = r.unit(o.kind, o.type_index) else {
        return;
    };
    let cur = o.beh.current;
    // spec: S-0215 claim 15 (nothing is queued while patrol, area guard, unload or eaten is current, or while enter towards a bunker it is in contact with and not yet standing on that bunker's cell)
    let blocked = matches!(
        cur,
        Some(Behaviour::Patrol | Behaviour::AreaGuard | Behaviour::Unload | Behaviour::Eaten)
    ) || (cur == Some(Behaviour::Enter)
        && o.contact.is_some_and(|k| {
            crate::structures::bunker::is_bunker(w, k)
                && !crate::structures::bunker::standing_on(w, k, id)
        }));
    let queue = |w: &mut World, b: Behaviour| {
        if !blocked && let Some(ob) = w.obj_mut(id) {
            ob.beh.queue(Some(b));
        }
    };
    let clear = |w: &mut World| {
        if let Some(ob) = w.obj_mut(id) {
            crate::spawn::clear_own_target(ob);
            ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
            ob.dest = None;
        }
        if placement {
            // spec: S-0207 claim 27 (Q-1090: the placement fall-back's destination clear is an effective assignment, empty-to-empty included, while placement's refresh is enabled: it stops locomotion, consumes refresh and enables no point suppression)
            crate::econ::assign_destination(w, id, None);
        }
    };
    // spec: S-0215 claim 15 row 1 (attack-move memory, or wait: nothing changes)
    if o.attack_move.is_some() || cur == Some(Behaviour::Wait) {
        return;
    }
    let mut dest = o.dest;
    if dest.is_none() && !o.queued_dests.is_empty() {
        // spec: S-0215 claim 15 row 2 (the first queued destination becomes the destination)
        if let Some(ob) = w.obj_mut(id) {
            let first = ob.queued_dests.remove(0);
            ob.dest = Some(first);
            dest = Some(first);
        }
    }
    // spec: S-0215 claim 15 row 3 (a destination: queue move)
    if dest.is_some() {
        queue(w, Behaviour::Move);
        return;
    }
    if u.harvester {
        // spec: S-0215 claim 15 row 4 (harvesters; the resource land test is skipped in the placement fall-back)
        if o.contact.is_some()
            || cur == Some(Behaviour::Harvest)
            || o.beh.queued == Some(Behaviour::Harvest)
        {
            return;
        }
        clear(w);
        let on_ore = w
            .cells
            .cell(o.pos.cell())
            .is_some_and(|c| c.land == glasnost_map::terrain::land::TIBERIUM);
        // spec: S-0215 claim 15 row 4 and Terms (the exception off the resource land type is for human-controlled harvesters; the neutral and special houses' harvesters queue harvest wherever they stand)
        let human = o.owner.is_none_or(|h| w.houses[usize::from(h.0)].human);
        queue(
            w,
            if placement || on_ore || !human {
                Behaviour::Harvest
            } else {
                Behaviour::Guard
            },
        );
        return;
    }
    let armed = u.primary.is_some();
    if !armed {
        // spec: S-0215 claim 15 row 6 (unarmed and deploying, or with area guard current: nothing changes)
        if cur == Some(Behaviour::AreaGuard) {
            return;
        }
        // spec: S-0215 claim 15 row 7 (unarmed: clear target and destination, queue guard)
        clear(w);
        queue(w, Behaviour::Guard);
        return;
    }
    // spec: S-0215 claim 15 rows 8 and 9 (armed with guard or area guard; zombie or paralysed: nothing changes)
    if matches!(cur, Some(Behaviour::Guard | Behaviour::AreaGuard)) {
        return;
    }
    if cur.is_some_and(|b| r.behaviour(b).zombie || r.behaviour(b).paralysed) {
        return;
    }
    let skill = o.owner.map_or(0, |h| w.houses[usize::from(h.0)].skill);
    let area = guard_area_ability(w, id) || u.default_to_guard_area;
    // spec: S-0215 claim 15 row 10 (skill below the area-guard threshold, without the ability or the type default)
    if skill < r.general.iq_guard_area && !area {
        queue(w, Behaviour::Guard);
        return;
    }
    // spec: S-0215 claim 15 row 11 (armed, a harvesting slave or a slave master: guard; no vehicle is a team member)
    if crate::seams::is_harvesting_slave(w, id) || crate::seams::is_slave_master(w, id) {
        queue(w, Behaviour::Guard);
        return;
    }
    // spec: S-0215 claim 15 row 12 and S-0606 claim 7 (otherwise area guard, or guard if the vehicle sits in a bunker)
    if crate::structures::bunker::bunkered(w, id) {
        queue(w, Behaviour::Guard);
    } else {
        queue(w, Behaviour::AreaGuard);
    }
}

/// One actor's behaviour part of its turn (S-0215 claims 2, 3, 11 and 33):
/// [`head`] then [`tail`].
pub fn turn(w: &mut World, id: ObjectId) {
    head(w, id);
    tail(w, id);
}

/// The first part of the behaviour part: the first per-tick start and the
/// berserk countdown (S-0215 claim 11, S-0610 claim 5). An unlimited mind
/// controller runs it earlier in its turn, before its overload update and
/// self-healing (S-0601 claim 16; `tick::object_turn`).
pub fn head(w: &mut World, id: ObjectId) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    let Some(o) = w.obj(id) else { return };
    let kind = o.kind;
    if kind == Kind::Aircraft {
        // spec: S-0610 claim 5 (the berserk countdown precedes the update's processing and shots; an aircraft has no first per-tick start before its processing; a landed aircraft being erased never gets here, S-0514 claim 17)
        crate::psychic::countdown_turn(w, id);
        return;
    }
    if kind == Kind::Terrain {
        return;
    }
    if kind == Kind::Vehicle {
        // spec: S-0517 claim 9 (a destination written by any rule while the vehicle is deployed or in transition is replaced by none)
        crate::econ::inplace::enforce(w, id);
        // spec: S-0606 claim 18 (while the bunker link holds, a destination written by any rule is refused: it is replaced by none)
        crate::structures::bunker::enforce(w, id);
    }
    // spec: S-0607 claims 9 and 23 (a unit seized with a stand-in locomotor has no jumpjet steering: while gripped its destination stays the lifter and a removed one ends the grip; afterwards a destination written by any rule is replaced by none here)
    crate::lift::enforce(w, id);
    if kind == Kind::Building {
        door_update(w, id);
    }
    if kind == Kind::Building {
        // spec: S-0406 claim 17 (the construction state sets the ready mark on a building turn)
        crate::econ::placement::construction_turn(w, id);
        // spec: S-0215 claim 11 (the body range of a building without a turret sets the ready mark)
        crate::econ::body::turn(w, id);
    }
    // spec: S-0215 claim 11 (the per-tick start of infantry, vehicles and buildings); S-0406 claim 17 (no first start while the construction state lasts)
    let ready_now = ready(w, id) && !crate::econ::placement::first_start_withheld(w, id);
    let Some(o) = w.obj(id) else { return };
    if ready_now {
        if kind == Kind::Infantry && o.beh.current.is_none() && o.beh.queued.is_none() {
            // spec: S-0215 claim 11 (an infantry unit with neither falls back to rest at its per-tick start)
            fall_back(w, id, false);
        }
        if let Some(ob) = w.obj_mut(id)
            && ob.beh.start_next(t, stale)
            && kind == Kind::Building
        {
            // spec: S-0215 claim 11 (each successful per-tick start of a building clears its ready mark)
            ob.beh.ready_mark = false;
        }
    }
    // spec: S-0610 claim 5 and F2 (the berserk countdown, one step in each countdown update: after the first per-tick start, so that a behaviour the expiry queues is not made current and processed in the expiry update; before the processing, the per-tick search and every shot of the update, so that the expiry update already treats the unit as ordinary; not dependent on the processing timer being due. Excluded and warp-only updates never reach here: S-0215 claim 2, S-0513 claim 51)
    crate::psychic::countdown_turn(w, id);
}

/// The rest of the behaviour part, after [`head`]: the drop rule, the
/// processing when due, the per-tick search and approach (S-0215 claims 2,
/// 3, 33 and 37).
pub fn tail(w: &mut World, id: ObjectId) {
    let t = w.tick;
    let Some(o) = w.obj(id) else { return };
    let kind = o.kind;
    if kind == Kind::Aircraft {
        // spec: S-0501 claims 31 and 32 (an aircraft has no per-tick start; its processing)
        crate::air::behaviour_turn(w, id);
        return;
    }
    if kind == Kind::Terrain {
        return;
    }
    // spec: S-0215 claim 33 (the drop rule: an opportunity target is dropped in these behaviours, judged after the per-tick start)
    if let Some(ob) = w.obj_mut(id)
        && ob.target.is_some()
        && ob.combat.opportunity
        && matches!(
            // spec: S-0406 claim 17 (a building with none current is seen in its queued construction)
            crate::econ::placement::seen_behaviour(ob),
            Some(
                Behaviour::Ambush
                    | Behaviour::Construction
                    | Behaviour::Enter
                    | Behaviour::Harmless
                    | Behaviour::Missile
                    | Behaviour::Open
                    | Behaviour::Repair
                    | Behaviour::Selling
                    | Behaviour::Sleep
                    | Behaviour::Stop
                    | Behaviour::Unload
                    | Behaviour::Wait
            )
        )
    {
        crate::spawn::clear_own_target(ob);
        ob.combat.opportunity = false;
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
    // spec: S-0609 Terms (the elapsed-tick count grows by 1 just before the behaviour may be processed, after the per-tick start; only gattling-flagged objects read it, so only they keep it)
    if crate::gattling::flagged(w, id) {
        crate::gattling::elapse(w, id);
    }
    let Some(o) = w.obj(id) else { return };
    // spec: S-0215 claim 2 (alive with health above 0 and the timer expired)
    if o.health <= 0 || !o.beh.due(t) {
        if kind == Kind::Building {
            second_start(w, id);
            // spec: S-0309 claim 31 (an opened building's idle body follows at the end of the turn)
            crate::econ::placement::opened_turn_end(w, id);
        }
        // spec: S-0215 claim 33 (b) and (d) (the per-tick search after the processing)
        crate::combat::acquire::per_tick(w, id);
        // spec: S-0215 claim 37 (a) and (a2) (the per-tick approach of an actor that cannot reach its target, after the per-tick search on a tick without processing too)
        crate::approach::per_tick(w, id);
        return;
    }
    let d = process(w, id);
    if let Some(ob) = w.obj_mut(id) {
        ob.beh.processed(t, d);
    }
    if kind == Kind::Building {
        second_start(w, id);
        // spec: S-0309 claim 31 (an opened building's idle body follows at the end of the turn)
        crate::econ::placement::opened_turn_end(w, id);
    }
    // spec: S-0215 claim 33 (b) and (d) (the per-tick search after the processing, judged on the behaviour it leaves)
    crate::combat::acquire::per_tick(w, id);
    // spec: S-0215 claim 37 (a) and (a2) (the per-tick approach of an actor that cannot reach its target follows the processing and the per-tick search; revision 48 specifies the resulting state, not a count of internal evaluations)
    crate::approach::per_tick(w, id);
}

/// The late start of an infantry unit or vehicle (S-0215 claim 11): one
/// more ready-and-start chance in its update, after its movement, judged
/// with the movement state that update has left (revision 67: the current
/// predicate, not a remembered displacement earlier in the update). It makes a queued behaviour
/// current; a ready infantry unit with neither a current nor a queued
/// behaviour first falls back to rest, as at the first per-tick start
/// (revision 59, which answers Q-1166 question 1). A behaviour made
/// current here is first processed on the unit's next update (claim 3b),
/// and the drop rule and the per-tick search of this update have already
/// run (claim 33 b).
pub fn late_start(w: &mut World, id: ObjectId) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    let Some(o) = w.obj(id) else { return };
    let kind = o.kind;
    // spec: S-0215 claim 11 (infantry and vehicles have a late start; buildings and aircraft do not)
    if !matches!(kind, Kind::Infantry | Kind::Vehicle) || o.health <= 0 {
        return;
    }
    let empty = o.beh.current.is_none() && o.beh.queued.is_none();
    if o.beh.queued.is_none() && !(kind == Kind::Infantry && empty) {
        return;
    }
    // spec: S-0215 claim 11 (the late start: ready under the readiness rules, judged with the movement state this update has left, revision 67)
    if !ready(w, id) {
        return;
    }
    if kind == Kind::Infantry && empty {
        // spec: S-0215 claim 11 (revision 59: an infantry unit ready at the late start with neither a current nor a queued behaviour falls back to rest in that update; what the fall-back queues becomes current at that same late start)
        fall_back(w, id, false);
    }
    if let Some(ob) = w.obj_mut(id) {
        ob.beh.start_next(t, stale);
    }
}

/// The attack-move refresh among the common effects of a fall-back to rest
/// (S-0309 claim 23 (b)): resumed when engaged (which it then no longer is)
/// or in *guard*, else removed.
fn refresh_attack_move(w: &mut World, id: ObjectId) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    let Some(ob) = w.obj_mut(id) else { return };
    let Some(mem) = ob.attack_move else { return };
    // spec: S-0309 claim 23 (b) (resumed when engaged or in guard; otherwise the memory is removed, with its engaged mark)
    if !(ob.combat.engaged || ob.beh.seen() == Some(Behaviour::Guard)) {
        ob.attack_move = None;
        ob.combat.engaged = false; // spec: S-0309 claim 23 (c) (removing the memory ends the engaged mark)
        return;
    }
    // spec: S-0309 claim 23 (c) (resuming ends the engaged mark)
    ob.combat.engaged = false;
    match mem {
        AttackMoveMemory::Dest(d) => {
            // spec: S-0309 claim 23 (b) (a destination: move queued with an immediate start, the memory's destination the object's)
            let before = ob.dest;
            ob.beh.queue_and_start_at_once(Behaviour::Move, t, stale);
            ob.dest = Some(d);
            // spec: S-0107 claim 15 (a destination assigned: the destination-driven get-up)
            crate::infantry::destination_assigned(w, id, before);
        }
        AttackMoveMemory::Target(x) => {
            // spec: S-0309 claim 23 (b) (only a target: attack queued with an immediate start, the memory's target the object's)
            ob.beh.queue_and_start_at_once(Behaviour::Attack, t, stale);
            let before = ob.target;
            ob.target = Some(x);
            // spec: S-0309 claim 24 (every setting of a target, even to the same one, leaves a target that is not an opportunity target)
            ob.combat.opportunity = false;
            // spec: S-0215 r118 claim 11 (a remembered target put back is no change of target)
            crate::infantry::watch_carry(w, id, before);
            // spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: at the change of the firer's target)
            crate::emitters::retarget_let_go(w, id);
        }
    }
}

/// A building's second per-tick start (S-0215 claim 11).
fn second_start(w: &mut World, id: ObjectId) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    if let Some(ob) = w.obj_mut(id)
        && ob.beh.ready_mark
        && ob.beh.start_next(t, stale)
    {
        // spec: S-0215 claim 11 (the second start, after the processing; it clears the mark when it succeeds)
        ob.beh.ready_mark = false;
    }
}

/// One processing; returns the delay chosen.
fn process(w: &mut World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    let cur = o.beh.current;
    // spec: S-0215 claim 31 (deploying infantry: its rules take precedence in guard and area guard)
    if o.kind == Kind::Infantry
        && matches!(
            cur,
            Some(Behaviour::Guard | Behaviour::Sticky | Behaviour::AreaGuard)
        )
        && let Some(d) = deployer_rule(w, id)
    {
        return d;
    }
    match (cur, o.kind) {
        // spec: S-0215 claim 4 and table T1 (move has no processing rule for buildings: nothing, 450 ticks, no draw); S-0402 claim 27a (a field-seeking building's move does not move it, request packing or clear its guard spot)
        (Some(Behaviour::Move), Kind::Building) => NO_RULE_DELAY,
        (Some(Behaviour::Move), _) => process_move(w, id),
        (Some(Behaviour::Attack), Kind::Infantry | Kind::Vehicle) => process_attack(w, id),
        // spec: S-0215 claim 5 (sticky is processed exactly as guard)
        (Some(Behaviour::Guard | Behaviour::Sticky), Kind::Infantry | Kind::Vehicle) => {
            process_guard(w, id)
        }
        (Some(Behaviour::AreaGuard), Kind::Infantry | Kind::Vehicle) => process_area_guard(w, id),
        (Some(Behaviour::Hunt), Kind::Infantry | Kind::Vehicle) => process_hunt(w, id),
        // spec: S-0215 claim 28 and claim 5 (capture; sabotage is processed exactly as capture)
        (Some(Behaviour::Capture | Behaviour::Sabotage), Kind::Infantry | Kind::Vehicle) => {
            process_capture(w, id, false)
        }
        // spec: S-0215 claim 4 and table T1 (capture, sabotage and eaten have no processing rule for buildings: nothing, 450 ticks, no draw)
        (Some(Behaviour::Capture | Behaviour::Sabotage | Behaviour::Eaten), Kind::Building) => {
            NO_RULE_DELAY
        }
        (Some(Behaviour::Unload), Kind::Infantry) => process_unload_infantry(w, id),
        // spec: S-0402 State and ordering (a harvester's harvest, enter and unload); claim 22 (a slave master's harvest)
        (Some(Behaviour::Harvest), Kind::Vehicle)
            if crate::econ::harvest::is_harvester(w, id)
                || crate::econ::slaves::is_master(w, id) =>
        {
            crate::econ::harvest::process_harvest(w, id)
        }
        // spec: S-0402 claim 14 (a slave's harvest)
        (Some(Behaviour::Harvest), Kind::Infantry) if crate::econ::slaves::is_slave(w, id) => {
            crate::econ::slaves::process_slave(w, id)
        }
        // spec: S-0215 claim 29 and S-0314 claims 16 to 18 (a vehicle's enter at a repair depot)
        (Some(Behaviour::Enter), Kind::Vehicle) if crate::depot::entering_depot(w, id) => {
            crate::depot::process_enter(w, id)
        }
        // spec: S-0606 claim 20 (revision 7, Q-1163 answered: a type that is both a repair depot and a bunker carries out at most one row of table S1, then the depot's repair decision on the contact, behaviour and movement that row left; the depot's own queued behaviour replaces S1's queued guard, S-0215 claim 7, otherwise that guard stays; the delay is the depot's)
        (Some(Behaviour::Repair), Kind::Building)
            if crate::depot::is_depot(w, id) && crate::structures::bunker::is_bunker(w, id) =>
        {
            crate::structures::bunker::process_repair(w, id);
            crate::depot::process_repair(w, id)
        }
        // spec: S-0314 claim 6 and S-0504 claim 8 (a repair depot's repair processing)
        (Some(Behaviour::Repair), Kind::Building) if crate::depot::is_depot(w, id) => {
            crate::depot::process_repair(w, id)
        }
        // spec: S-0606 claims 3 to 5 and S-0215 claim 29 (a vehicle's enter toward a tank bunker; a bunker that is also a depot takes the depot's enter, S-0606 claim 20)
        (Some(Behaviour::Enter), Kind::Vehicle)
            if crate::structures::bunker::entering_bunker(w, id) =>
        {
            crate::structures::bunker::process_enter(w, id)
        }
        // spec: S-0606 claim 6 (a tank bunker's repair processing: the entry stages of table S1)
        (Some(Behaviour::Repair), Kind::Building)
            if crate::structures::bunker::is_bunker(w, id) =>
        {
            crate::structures::bunker::process_repair(w, id)
        }
        // spec: S-0215 claim 29 and S-0507 claim 5 (enter toward a transport: the loading exchange)
        (Some(Behaviour::Enter), Kind::Infantry | Kind::Vehicle)
            if crate::transport::entering_transport(w, id) =>
        {
            crate::transport::process_enter(w, id)
        }
        // spec: S-0605 claim 5 and S-0215 claim 29 (enter toward an absorbing building: the loading request at each processing)
        (Some(Behaviour::Enter), Kind::Infantry | Kind::Vehicle)
            if crate::structures::absorb::entering_absorber(w, id) =>
        {
            crate::structures::absorb::process_enter(w, id)
        }
        // spec: S-0603 claim 3 and S-0215 claim 28 (eaten: the capture and sabotage table with its eaten differences)
        (Some(Behaviour::Eaten), Kind::Infantry | Kind::Vehicle) => {
            crate::structures::grinder::process_eaten(w, id)
        }
        (Some(Behaviour::Enter), Kind::Vehicle) if crate::econ::harvest::is_harvester(w, id) => {
            crate::econ::harvest::process_enter(w, id)
        }
        // spec: S-0215 claim 29 (the entry request goes to the contact, otherwise to the remembered entry object: a vehicle in enter with neither, and no object destination, gets no accepting reply and falls back to rest) and S-0314 claim 20 (revision 28: the docked vehicle refused for standing on its depot falls back at its next processing)
        (Some(Behaviour::Enter), Kind::Vehicle)
            if w.obj(id).is_some_and(|o| {
                o.contact.is_none()
                    && o.depot.entering.is_none()
                    && !matches!(o.dest, Some(Target::Object(_)))
            }) =>
        {
            fall_back(w, id, false);
            standard_delay(w, id)
        }
        // spec: S-0606 claim 12 and S-0415 claim 1 (a bunkered vehicle's unload is the bunker release, not passenger unloading)
        (Some(Behaviour::Unload), Kind::Vehicle) if crate::structures::bunker::bunkered(w, id) => {
            crate::structures::bunker::process_unload(w, id)
        }
        // spec: S-0415 claim 1 rank 1 and S-0507 claims 8 to 12 (a vehicle with passenger capacity above 0 unloads passengers)
        (Some(Behaviour::Unload), Kind::Vehicle) if crate::transport::vehicle_transport(w, id) => {
            crate::transport::process_unload(w, id)
        }
        (Some(Behaviour::Unload), Kind::Vehicle)
            if crate::econ::harvest::unload_is_harvesting(w, id) =>
        {
            crate::econ::harvest::process_unload(w, id)
        }
        // spec: S-0415 claim 1 (unload of a vehicle: rank 2 is the deploy-fire rule with its own delay, rank 4 deploys with the standard delay of S-0215 claim 6; ranks 1 and 3 are other topics' rules)
        (Some(Behaviour::Unload), Kind::Vehicle) => {
            match crate::econ::deploy::process_unload(w, id) {
                Some(crate::econ::deploy::UnloadRule::DeployFire(d)) => d,
                Some(crate::econ::deploy::UnloadRule::Deploy) => standard_delay(w, id),
                // spec: S-0517 claims 3 to 8 (the in-place cycle: 1 tick in a transition or landing for deploy, otherwise guard started and 450 ticks, no draw)
                Some(crate::econ::deploy::UnloadRule::InPlace(d)) => d,
                None => {
                    w.note(
                    "S-0215: the processing of unload is not implemented yet; it waits as a behaviour without a processing rule",
                );
                    NO_RULE_DELAY
                }
            }
        }
        // spec: S-0605 claim 11 (unload of an absorbing building: occupants first, then the front passenger)
        (Some(Behaviour::Unload), Kind::Building)
            if crate::structures::absorb::is_absorber(w, id) =>
        {
            crate::structures::absorb::process_unload(w, id)
        }
        // spec: S-0508 claim 13 (a deploy order given to a garrisonable building: its occupants are evicted without hunting or inside fallback)
        (Some(Behaviour::Unload), Kind::Building)
            if crate::structures::garrison::garrisonable(w, id) =>
        {
            crate::structures::garrison::process_unload(w, id)
        }
        // spec: S-0417 claims 2 and 9 (a factory building's unloading)
        (Some(Behaviour::Unload), Kind::Building) => crate::econ::release::process_unload(w, id),
        // spec: S-0408 claim 16 (selling: a processing delay of 1, no draw)
        (Some(Behaviour::Selling), Kind::Building) => crate::econ::sell::process_selling(w, id),
        // spec: S-0406 claims 14 and 15 (construction: processed every tick, handed over when the build-up reports completion)
        (Some(Behaviour::Construction), Kind::Building) => {
            crate::econ::deploy::process_construction(w, id)
        }
        (
            Some(Behaviour::Guard | Behaviour::Sticky | Behaviour::AreaGuard | Behaviour::Attack),
            Kind::Building,
        ) => crate::combat::behave::building_process(w, id),
        // spec: S-0215 claim 35 (c) (open for buildings)
        (Some(Behaviour::Open), Kind::Building) => process_open(w, id),
        // spec: S-0215 claim 4 and table T1 (open for infantry and vehicles: no processing rule, 450 ticks)
        (Some(Behaviour::Open), _) => NO_RULE_DELAY,
        // spec: S-0215 claim 5 (queued move and attack move are processed as sleep, quirks.queued_move_sleeps)
        (Some(Behaviour::QueuedMove | Behaviour::AttackMove), _)
            if w.rules().quirks.queued_move_sleeps =>
        {
            NO_RULE_DELAY
        }
        // spec: S-0215 claim 4 (sleep, harmless, ambush, return, stop, wait and none: 450 ticks, no draw)
        (
            None
            | Some(
                Behaviour::Sleep
                | Behaviour::Harmless
                | Behaviour::Ambush
                | Behaviour::Return
                | Behaviour::Stop
                | Behaviour::Wait,
            ),
            _,
        ) => NO_RULE_DELAY,
        (Some(b), _) => {
            w.note(format!(
                "S-0215: the processing of {} is not implemented yet; it waits as a behaviour without a processing rule",
                b.key()
            ));
            NO_RULE_DELAY
        }
    }
}

/// The undeploy delay of an infantry unit's type (S-0214 Import mapping:
/// absent −1, none).
fn undeploy_delay(w: &World, id: ObjectId) -> i32 {
    let o = w.obj(id);
    // spec: S-0214 Import mapping (UndeployDelay absent: −1, none)
    o.and_then(|o| w.rules().unit(o.kind, o.type_index))
        .map_or(-1, |u| u.raw.undeploy_delay.unwrap_or(-1))
}

/// The frame count of an infantry sequence (S-0107 claim 1).
fn frame_count(w: &World, id: ObjectId, q: Seq) -> i32 {
    w.obj(id)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        .and_then(|u| u.sequences.get(q.key()))
        .map_or(0, |e| e.count)
}

/// The deployer rules of S-0215 claim 31, applied when one applies
/// (returning its delay); `None` when none applies.
fn deployer_rule(w: &mut World, id: ObjectId) -> Option<i32> {
    let o = w.obj(id)?;
    let u = w.rules().unit(o.kind, o.type_index)?;
    if !u.deployer {
        return None;
    }
    let deployed = o
        .infantry
        .as_ref()
        .and_then(|s| s.seq)
        .is_some_and(Seq::is_deploy);
    if !deployed {
        return auto_deploy(w, id);
    }
    if undeploy_delay(w, id) >= 0 {
        // spec: S-0215 claim 31 row 1 (deployed, undeploy delay 0 or more: starts undeploy; its frame count)
        infantry::request(w, id, Seq::Undeploy, false);
        return Some(frame_count(w, id, Seq::Undeploy));
    }
    let fires = w
        .obj(id)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        .is_some_and(|u| {
            u.raw.deploy_fire.unwrap_or(false) && !u.raw.immune_to_radiation.unwrap_or(false)
        });
    if fires {
        // spec: S-0215 claim 31 row 2 (deploys to fire, not immune to radiation: the deployed-firing rule, S-0517 claim 16; the standard delay)
        deployed_firing(w, id);
        return Some(standard_delay(w, id));
    }
    let immune_fires = w
        .obj(id)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        .is_some_and(|u| {
            u.raw.deploy_fire.unwrap_or(false) && u.raw.immune_to_radiation.unwrap_or(false)
        });
    if immune_fires {
        // spec: S-0215 claim 31 rows 3 to 5 (deploys to fire, immune to radiation)
        return Some(immune_deployed(w, id));
    }
    // A type that does not deploy to fire follows claims 24 and 26.
    None
}

/// The radiation-immune deployed rows of S-0215 claim 31: a unit on the
/// centre cell of a field whose current field level is at least
/// *R* = trunc(*W* / 3), with *W* the radiation level of its secondary
/// weapon (S-0525 claim 7), does nothing; otherwise it fires at its own
/// cell when the firing rules let it (the standing row), starting its
/// deployed-fire sequence; a refused shot only drops the target. The delay
/// is the sequence's frame count for a shot and *D*<sub>r</sub> + *U*(10, 20)
/// for the other two rows.
fn immune_deployed(w: &mut World, id: ObjectId) -> i32 {
    use crate::combat::check::{Answer, check};
    let Some(cell) = w.obj(id).map(|o| o.pos.cell()) else {
        return 0;
    };
    // spec: S-0215 claim 31 (R = trunc(W / 3), integer division by the radiation threshold divisor 3; W the secondary slot's radiation level)
    let level = crate::combat::weapon::slot_weapon(w, id, 1)
        .map_or(0, |wp| w.rules().combat.weapon(wp).rad_level);
    let threshold = level / 3; // spec: S-0215 claim 31 (R = trunc(W / 3), the radiation threshold divisor)
    // spec: S-0215 claim 31 and S-0525 claim 7 (only a field centred on the unit's cell counts, by its current integer field level)
    let suppressed = crate::radiation::centre_field_level(w, cell).is_some_and(|l| l >= threshold);
    let target = Target::Cell(cell);
    if !suppressed && check(w, id, Some(target), 1, true) == Answer::CanFire {
        // spec: S-0215 claim 31 (fires at its own cell, drops the target, starts its deployed-fire sequence; Q-0983: the shot is made at once)
        crate::combat::fire::shot(w, id, 1, target);
        if let Some(ob) = w.obj_mut(id) {
            crate::spawn::clear_own_target(ob);
            ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
        }
        infantry::request(w, id, Seq::DeployedFire, false);
        return frame_count(w, id, Seq::DeployedFire);
    }
    if !suppressed && let Some(ob) = w.obj_mut(id) {
        // spec: S-0215 claim 31 (the firing rules refuse: it drops the target)
        crate::spawn::clear_own_target(ob);
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
    let half = w.rules().quirks.ranged_draw_half_span;
    let dr = rate_delay(w, id);
    // spec: S-0215 claim 31 (D_r + U(10, 20))
    dr.wrapping_add(w.rng.ranged(10, 20, half))
}

/// The deployed-firing rule (S-0517 claim 16) of a deployed infantry unit:
/// its target stays when it is within the range of the weapon selected with
/// no target supplied; otherwise an S-0309 range search centred on its own
/// position runs without waiting for the targeting timer, and a found target
/// replaces an out-of-range one; with none found an out-of-range target is
/// dropped and a unit left targetless falls back to rest unless its current
/// behaviour is *guard* or its type is a jumpjet type. It never moves the
/// unit.
fn deployed_firing(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    // spec: S-0517 claim 16 (the weapon selected with no target supplied, S-0308 claims 3 and 8: the deploy weapon of a deployed deploy-fire type)
    let (_, weapon) = crate::combat::weapon::would_use(w, id, None);
    let had = o.target;
    let in_range =
        had.is_some_and(|t| crate::combat::range::in_range(w, id, weapon, Some(t), None));
    // spec: S-0517 claim 16 (a target within the weapon's range under S-0307: no targeting change)
    if in_range {
        return;
    }
    if had.is_some()
        && let Some(ob) = w.obj_mut(id)
    {
        // spec: S-0517 claim 16 (an out-of-range target is replaced by what the search finds, or dropped when it finds none)
        crate::spawn::clear_own_target(ob);
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
    // spec: S-0517 claim 16 (an S-0309 range search centred on its own position, without waiting for its targeting timer)
    let _ = crate::combat::acquire::search(
        w,
        id,
        crate::combat::acquire::SearchKind::Range,
        crate::combat::acquire::SearchContext::PerTick,
    );
    let Some(o) = w.obj(id) else { return };
    if o.target.is_some() {
        return;
    }
    let jumpjet = w
        .rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.jumpjet);
    // spec: S-0517 claim 16 (a unit left targetless falls back to rest under S-0215 unless its current behaviour is guard or its type is a jumpjet type)
    if o.beh.current != Some(Behaviour::Guard) && !jumpjet {
        fall_back(w, id, false);
    }
}

/// The last row of S-0215 claim 31 for an undeployed unit: the auto-deploy
/// of an owner that is not human-controlled. In our engine no player house
/// reaches it; the neutral and special houses keep it, with A the list's
/// entry for their difficulty index 1, and without that entry the row does
/// not apply (claim 31, a documented departure).
fn auto_deploy(w: &mut World, id: ObjectId) -> Option<i32> {
    let t = w.tick;
    let o = w.obj(id)?.clone();
    let r = w.rules_arc();
    let u = r.unit(o.kind, o.type_index)?;
    // spec: S-0215 claim 31 (the last row: an owner that is not human-controlled; Terms, houses in our engine: the neutral and special houses, and a test-only computer-controlled player house)
    let owner = o.owner.map(|h| &w.houses[usize::from(h.0)]);
    if owner.is_none_or(|h| h.human) {
        return None;
    }
    // spec: S-0215 claim 31 (A: the entry for the owner's difficulty index, 1 for the neutral and special houses; S-0309 Terms, Test-only exception: a marked house's oracle index; no such entry: the row does not apply)
    let idx = owner.map_or(1, |h| h.econ.setup.difficulty);
    let a = usize::try_from(idx)
        .ok()
        .and_then(|i| r.general.auto_deploy_delays.get(i).copied().flatten())?;
    let fires = u.raw.deploy_fire.unwrap_or(false);
    let immune = u.raw.immune_to_radiation.unwrap_or(false);
    let on_spot = match o.guard_spot {
        None => true,
        Some(Target::Cell(c)) => c == o.pos.cell(),
        Some(Target::Object(x)) => w.obj(x).is_some_and(|g| g.pos.cell() == o.pos.cell()),
    };
    // spec: S-0215 claim 31 (deploys to fire, not immune, undeploy delay negative, no destination, no guard spot or on its cell, and t > t_s + A with the sum wrapping)
    let due = t > o.beh.start_tick.wrapping_add(a);
    if !(fires && !immune && undeploy_delay(w, id) < 0 && o.dest.is_none() && on_spot && due) {
        return None;
    }
    if moving(w, id) {
        // spec: S-0215 claim 31 (moving: it is told to stop, the locomotor stop of S-0213 claim 8; the standard delay)
        if let Some(ob) = w.obj_mut(id) {
            ob.path.steps.clear();
        }
        return Some(standard_delay(w, id));
    }
    // spec: S-0215 claim 31 (otherwise it starts its deploy sequence; the delay is that sequence's frame count)
    infantry::request(w, id, Seq::Deploy, false);
    Some(frame_count(w, id, Seq::Deploy))
}

/// Unload for infantry (S-0215 claim 30).
fn process_unload_infantry(w: &mut World, id: ObjectId) -> i32 {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    let deployer = w
        .rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.deployer);
    if !deployer {
        // spec: S-0215 claim 30 (any other infantry unit does nothing in unload: 450 ticks, no draw)
        return NO_RULE_DELAY;
    }
    let deployed = o
        .infantry
        .as_ref()
        .and_then(|s| s.seq)
        .is_some_and(Seq::is_deploy);
    let delay = undeploy_delay(w, id);
    let d = if deployed {
        if delay < 0 {
            // spec: S-0215 claim 30 (a deployed unit undeploys only if its undeploy delay is negative)
            infantry::request(w, id, Seq::Undeploy, false);
        }
        // spec: S-0215 claim 30 (deployed: 450 ticks)
        NO_RULE_DELAY
    } else {
        // spec: S-0215 claim 30 (an undeployed unit starts its deploy sequence; area-fire weapons wait for M4)
        infantry::request(w, id, Seq::Deploy, false);
        // spec: S-0215 claim 30 (undeployed: the undeploy delay when 0 or more, else 450 ticks)
        if delay >= 0 { delay } else { NO_RULE_DELAY }
    };
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0215 claim 30 (in every case it forces guard and clears its destination)
        ob.beh.force(Behaviour::Guard, t, stale);
        ob.dest = None;
    }
    d
}

/// Move (S-0215 claim 22).
fn process_move(w: &mut World, id: ObjectId) -> i32 {
    // spec: S-0517 claim 10 (move processed for a vehicle that is deployed or in transition: guard becomes its queued behaviour, 1 tick)
    if let Some(d) = crate::econ::inplace::move_fallback(w, id) {
        return d;
    }
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    let deployed = o.kind == Kind::Infantry
        && o.infantry
            .as_ref()
            .and_then(|s| s.seq)
            .is_some_and(Seq::is_deploy);
    if deployed {
        // spec: S-0215 claim 22 (deployed infantry: clears its destination, 1 tick)
        if let Some(ob) = w.obj_mut(id) {
            ob.dest = None;
        }
        return 1;
    }
    // spec: S-0215 claim 22 (no destination, not moving, nothing queued: falls back to rest, 1 tick)
    if o.dest.is_none() && !moving(w, id) && o.beh.queued.is_none() {
        fall_back(w, id, false);
        return 1;
    }
    standard_delay(w, id)
}

/// S-0215 claim 22's end-of-update check of an infantry unit, made at the
/// end of each of its updates after its late start: a unit whose current
/// behaviour is *move*, or that has none and *move* queued, that is not
/// moving and has no destination falls back to rest (claims 13 and 14).
/// What that queues is first started at a later per-tick start, since the
/// late start has passed. The caller leaves out the updates that end before
/// their late start, which have no check.
pub fn end_of_update_check(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    // spec: S-0215 claim 22 (r132, End-of-update check: infantry only; whether a vehicle in move has a like check is open, Open questions, and our engine makes it for infantry only)
    if o.kind != Kind::Infantry || crate::combat::damage::dead(o) {
        return;
    }
    // spec: S-0215 claim 22 (r132: the current behaviour is move, or there is none and move is queued, the behaviour seen by claim 1)
    if o.beh.seen() != Some(Behaviour::Move) {
        return;
    }
    // spec: S-0215 claim 22 (r132: no destination; a unit in move holding one, not moving, is left alone by our engine, Open questions)
    if o.dest.is_some() {
        return;
    }
    // spec: S-0215 claim 22 (r132: not moving, the same test as the general rule of the move processing)
    if moving(w, id) {
        return;
    }
    // spec: S-0215 claim 22 (r132: it falls back to rest by claims 13 and 14, whatever is queued and whether or not move was processed in that update)
    fall_back(w, id, false);
}

/// The condition of S-0215 claim 23's first infantry row: the infantry
/// unit `id` has explosive charges (by type or veterancy ability) and `b` is
/// a building that accepts charges and is not invisible.
fn charges_building(w: &World, id: ObjectId, b: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    // spec: S-0215 claim 23 (explosive charges by type or veterancy ability)
    let charges = w
        .rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.raw.c4.unwrap_or(false))
        || crate::combat::veterancy::has_ability(w, id, crate::combat::data::Ability::C4);
    let Some(t) = w
        .obj(b)
        .filter(|x| x.kind == Kind::Building)
        .and_then(|x| w.rules().buildings.get(x.type_index as usize))
    else {
        return false;
    };
    // spec: S-0215 claim 23 (a building that accepts charges and is not invisible, the two flags as the rules import them)
    charges && t.building.can_c4 && !t.invisible
}

/// Attack (S-0215 claim 23).
fn process_attack(w: &mut World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    if o.kind == Kind::Infantry {
        if let Some(tg @ Target::Object(b)) = o.target
            && charges_building(w, id, b)
        {
            // spec: S-0215 claim 23 (Infantry, first row: explosive charges against a building that accepts charges and is not invisible: the target becomes the destination and sabotage is queued; 1 tick, no draw, none of the general rules)
            let before = o.dest;
            if let Some(ob) = w.obj_mut(id) {
                ob.dest = Some(tg);
                ob.beh.queue(Some(Behaviour::Sabotage));
            }
            // spec: S-0107 claim 15 (a destination assigned: the destination-driven get-up)
            crate::infantry::destination_assigned(w, id, before);
            return 1;
        }
        if let Some(tg) = computer_taker_target(w, id) {
            // spec: S-0215 claim 23 (Infantry, second row, original only: a computer-controlled owner, its target a building, and its type infiltrates, or occupies or assaults buildings and may occupy this one: the target becomes its destination and capture is forced; 1 tick, no draw; S-0215 Terms, Test-only computer-controlled player house)
            go_for_building(w, id, tg);
            return 1;
        }
        let deployed = o
            .infantry
            .as_ref()
            .and_then(|s| s.seq)
            .is_some_and(Seq::is_deploy);
        if deployed && o.target.is_some() {
            // spec: S-0215 claim 23 (deployed infantry: its deployed-firing rule, S-0517 claim 16, the standard delay not halved)
            deployed_firing(w, id);
            return standard_delay(w, id);
        }
    }
    // spec: S-0518 claim 15 (an infantry unit whose charging mark is set first runs an automatic range search, without the timer gate)
    crate::linked::break_off(w, id);
    // spec: S-0304 r6 claim 2 (This holds for every actor, whatever its owner) and S-0309 r94's Changes for S-0304 claim 2 (Q-2263 answered: claim 29 covers every holder of every owner, so a unit drops an illegal target at that claim's periodic check, a healer also at its firing step's reply; the attack processing's drop is a building's only, S-0215 claim 32, for every owner)
    if w.obj(id).is_some_and(|o| o.target.is_none()) {
        // spec: S-0215 claim 23 (fall-back: no target)
        fall_back(w, id, false);
    }
    if w.obj(id).is_some_and(|o| o.target.is_some()) {
        // spec: S-0215 claim 23 (approach: an actor with a target takes the approach step of claim 37)
        crate::approach::step(w, id);
    }
    let d = standard_delay(w, id);
    // spec: S-0215 claim 23 and F3 (halved under the close-range condition, the draw taken either way)
    if close_range(w, id) { d / 2 } else { d }
}

/// Guard (S-0215 claim 24).
fn process_guard(w: &mut World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    if o.kind == Kind::Vehicle {
        // spec: S-0215 claim 24 vehicle table row 1 (a slave master for which S-0402's wake-up rule holds: wakes its slaves, the standard delay; excludes the rest)
        if crate::seams::slave_wake_rule_holds(w, id) {
            crate::seams::wake_slaves(w, id);
            return standard_delay(w, id);
        }
        // spec: S-0215 claim 24 vehicle table row 2 (a guard rule of S-0402 that queues harvest again: 1 tick, no draw; excludes the rest) and S-0402 claim 14 (claim 13's rows only after a processing that did not wake, and only for a type with the harvester flag: `guard_rule` tests it)
        if crate::seams::harvester_guard_rule(w, id) {
            return 1;
        }
        // Row 3 (the weeder's rule) is left by S-0402 to a later revision; row 4 is the original's computer players' only.
    }
    if crate::structures::absorb::holds_instruction(w, id) {
        // spec: S-0605 claim 16 and S-0215 claim 24 (a holder of the enter-nearest instruction carries it out and nothing else of the general rules: the search again, none found ends it) and Terms (in our engine only the absorbing-building instruction of a neutral or special house unit, given by the bio-reactor fate)
        crate::structures::absorb::instruction_turn(w, id);
        // spec: S-0215 claim 24 (D_r with no draw, the rate of the behaviour being processed: enter is queued without a start)
        return rate_delay(w, id);
    }
    // spec: S-0518 claim 14 (a charger with no target in guard looks at the cells next to it; when it takes a building its idle action does not run, but the delay selection of the table below still does, with the rate of guard; Q-0985 answered by revision 3)
    let charging = o.target.is_none() && crate::linked::start_charging(w, id);
    if !charging && o.target.is_none() && o.kind == Kind::Infantry {
        // spec: S-0215 claim 24 (no target: attempts its idle action, S-0107 claim 14)
        let in_progress = moving(w, id);
        infantry::idle_attempt(w, id, in_progress);
    }
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    // spec: S-0215 claim 24 (the actor's weapon is still rearming: the rearm time remaining, no draw)
    if t < o.combat.rearm_until {
        return o.combat.rearm_until.wrapping_sub(t);
    }
    // spec: S-0215 claim 24 (distributed fire with candidates: 0, no draw)
    let distributed = w
        .rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.raw.distributed_fire.unwrap_or(false));
    if distributed && !o.combat.fired_at.is_empty() {
        return 0;
    }
    standard_delay(w, id)
}

/// Hunt (S-0215 claim 27).
fn process_hunt(w: &mut World, id: ObjectId) -> i32 {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    let r = w.rules_arc();
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    let Some(u) = r.unit(o.kind, o.type_index).cloned() else {
        return NO_RULE_DELAY;
    };
    let stupid = u.raw.stupid_hunt.unwrap_or(false);
    if let Some(tg) = computer_taker_target(w, id) {
        // spec: S-0215 claim 27 (Infantry of a computer-controlled owner, original only, judged on the target held when the processing begins: a building it may take, as claim 23's second row; 1 tick, no draw, no search, no targeting-timer restart, no idle action; S-0215 Terms, Test-only computer-controlled player house)
        go_for_building(w, id, tg);
        return 1;
    }
    if o.kind == Kind::Vehicle && u.raw.deploys_into.is_some() {
        // spec: S-0215 claim 27 (a deploying vehicle seeks a clear spot to deploy; the deploy specs own it)
        w.note("S-0215 claim 27: a hunting vehicle that deploys does not seek a spot yet (deploy specs)");
        return standard_delay(w, id);
    }
    if !stupid {
        // spec: S-0215 claim 27 (the normal search from its own position, unless stupid hunt)
        crate::combat::acquire::search(
            w,
            id,
            crate::combat::acquire::SearchKind::MapWide,
            crate::combat::acquire::SearchContext::Hunt,
        );
    }
    let target = if stupid {
        None
    } else {
        w.obj(id).and_then(|x| x.target)
    };
    match target {
        Some(tg) => {
            let charges = u.raw.c4.unwrap_or(false)
                || crate::combat::veterancy::has_ability(w, id, crate::combat::data::Ability::C4);
            let building = matches!(tg, Target::Object(x) if w.obj(x).is_some_and(|b| b.kind == Kind::Building));
            let ready_now = ready(w, id);
            if o.kind == Kind::Infantry && ((u.engineer && !charges) || u.vehicle_thief) {
                // spec: S-0215 claim 27 (an engineer or vehicle thief: the target as destination, capture queued with an immediate start if ready)
                if let Some(ob) = w.obj_mut(id) {
                    let before = ob.dest;
                    ob.dest = Some(tg);
                    ob.beh
                        .queue_and_start(Some(Behaviour::Capture), ready_now, t, stale);
                    // spec: S-0107 claim 15 (a destination assigned: the destination-driven get-up)
                    crate::infantry::destination_assigned(w, id, before);
                }
            } else if o.kind == Kind::Infantry && charges && building {
                // spec: S-0215 claim 27 (explosive charges against a building: sabotage)
                if let Some(ob) = w.obj_mut(id) {
                    let before = ob.dest;
                    ob.dest = Some(tg);
                    ob.beh
                        .queue_and_start(Some(Behaviour::Sabotage), ready_now, t, stale);
                    // spec: S-0107 claim 15 (a destination assigned: the destination-driven get-up)
                    crate::infantry::destination_assigned(w, id, before);
                }
            } else {
                if o.kind == Kind::Infantry
                    && let Some(ob) = w.obj_mut(id)
                {
                    // spec: S-0215 claim 27 (any other infantry unit clears its destination)
                    ob.dest = None;
                }
                // spec: S-0215 claim 27 (and takes the approach step of claim 37; other kinds take that step)
                crate::approach::step(w, id);
            }
        }
        None => {
            if o.kind == Kind::Infantry {
                // spec: S-0215 claim 27 (without a target: it attempts its idle action)
                let ip = moving(w, id);
                infantry::idle_attempt(w, id, ip);
            }
        }
    }
    // spec: S-0215 claim 27 (the standard delay with the rate of the behaviour current once the outcome took effect)
    standard_delay(w, id)
}

/// The building an infantry unit of a computer-controlled owner goes
/// straight for (S-0215 claim 23, the second row of its infantry table, and
/// claim 27's first infantry rule, both original only): its owner is not
/// human-controlled (the neutral and special houses and a marked test-only
/// computer-controlled player house, S-0215 Terms), its target is a
/// building, and its type infiltrates, or occupies or assaults buildings and
/// may occupy this one (S-0508 claim 1, r19, Q-2927: claim 1's test for an
/// occupier type, claim 14's assault eligibility for an assaulter type that
/// is not an occupier type, never for a type with neither mark; an
/// infiltrator type goes whatever that test gives). `None` when the rule
/// does not apply.
fn computer_taker_target(w: &World, id: ObjectId) -> Option<Target> {
    let o = w.obj(id)?;
    // spec: S-0215 claims 23 and 27 (an infantry unit of a computer-controlled owner; S-0309 Terms, Test-only exception)
    if o.kind != Kind::Infantry || crate::combat::acquire::player_owned(w, id) {
        return None;
    }
    // spec: S-0215 claims 23 and 27 (its target is a building)
    let Some(tg @ Target::Object(b)) = o.target else {
        return None;
    };
    if !w.obj(b).is_some_and(|x| x.kind == Kind::Building) {
        return None;
    }
    // spec: S-0215 claims 23 and 27 (its type infiltrates); S-0510 Import mapping (Infiltrate absent: no)
    let infiltrates = w
        .rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.raw.infiltrate.unwrap_or(false));
    // spec: S-0215 claims 23 and 27 (or occupies or assaults buildings and may occupy this one) and S-0508 claim 1 (r19, Q-2927: "may occupy" is claim 1's test for an occupier type, claim 14's assault eligibility for an assaulter type that is not an occupier type, never met by a type with neither mark; an infiltrator type takes the target whatever this test gives)
    let occupies = crate::structures::garrison::may_occupy(w, id, b);
    (infiltrates || occupies).then_some(tg)
}

/// The outcome of [`computer_taker_target`]'s rule: the target becomes the
/// destination and *capture* is forced (S-0215 claims 23 and 27; claim 9).
fn go_for_building(w: &mut World, id: ObjectId, tg: Target) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    // spec: S-0215 claims 23 and 27 (it takes the target as destination)
    crate::econ::assign_destination(w, id, Some(tg));
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0215 claims 23 and 27 (and forces capture: claim 9, current at once whether or not ready)
        ob.beh.force(Behaviour::Capture, t, stale);
    }
}

/// Whether an actor is a building taker (S-0215 claim 28): an infantry unit
/// with explosive charges, an engineer, an occupier, an assaulter or an
/// infiltrator. For *eaten* (`eaten`) occupiers, assaulters and
/// infiltrators do not count.
fn building_taker(w: &World, id: ObjectId, eaten: bool) -> bool {
    let Some(o) = w.obj(id) else { return false };
    // spec: S-0215 Import mapping (C4, Engineer, Occupier, Assaulter and Infiltrate are infantry type fields)
    if o.kind != Kind::Infantry {
        return false;
    }
    // spec: S-0215 claim 28 (an infantry unit with explosive charges or an engineer)
    if crate::bomb::has_charges(w, id) || crate::structures::engineer::is_engineer(w, id) {
        return true;
    }
    if eaten {
        // spec: S-0215 claim 28 (for eaten, occupiers, assaulters and infiltrators do not count as building takers)
        return false;
    }
    // spec: S-0215 claim 28 (an occupier, an assaulter or an infiltrator); S-0510 Import mapping (Infiltrate absent: no)
    crate::structures::garrison::occupier(w, id)
        || crate::structures::garrison::assaulter(w, id)
        || w.rules()
            .unit(o.kind, o.type_index)
            .is_some_and(|u| u.raw.infiltrate.unwrap_or(false))
}

/// The processing of *capture* and *sabotage*, and with `eaten` of *eaten*
/// (S-0215 claim 28; claim 5: *sabotage* is processed as *capture*), for
/// infantry, vehicles and aircraft. The row is chosen by the destination
/// and target the actor has when the processing begins, and only that row
/// applies. Returns the delay.
pub(crate) fn process_capture(w: &mut World, id: ObjectId, eaten: bool) -> i32 {
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    if o.dest.is_none() {
        let target_building = match o.target {
            Some(Target::Object(t)) => w
                .obj(t)
                .is_some_and(|x| x.kind == Kind::Building && x.on_map),
            _ => false,
        };
        if target_building && building_taker(w, id, eaten) {
            // spec: S-0215 claim 28 row 1 (no destination; the target a building and the actor a building taker: the target becomes its destination)
            crate::econ::assign_destination(w, id, o.target);
        } else {
            // spec: S-0215 claim 28 row 2 (no destination, any other case: it falls back to rest, claims 14 and 15, S-0501 claim 18 for an aircraft)
            fall_back(w, id, false);
            // spec: S-0215 claim 28 (the cell tested is the one that holds the actor's position after its fall-back to rest)
            let on_building = w.obj(id).is_some_and(|x| {
                x.on_map
                    && w.cells
                        .list(x.pos.cell(), Layer::Ground)
                        .iter()
                        .any(|&b| w.obj(b).is_some_and(|y| y.kind == Kind::Building))
            });
            if on_building {
                // spec: S-0215 claim 28 (the scatter off a building: no threat point, forced, not move-anywhere; infantry judge it by S-0208 claims 11 and 13, a vehicle by S-0207 claim 16 without a draw, claim 19; an aircraft by S-0501 claim 49, r93, Q-2251: a fall-back to rest when its current behaviour, the one the first fall-back left, has the scatter property)
                crate::scatter::request(
                    w,
                    id,
                    crate::scatter::ScatterRequest {
                        threat: None,
                        forced: true,
                        anywhere: false,
                    },
                );
            }
            // spec: S-0215 claim 28 row 2 (if it then has no target, the hunt rule applies as well: judged on the target the fall-back to rest leaves it with)
            if w.obj(id).is_some_and(|x| x.target.is_none()) {
                capture_hunt_rule(w, id, eaten);
            }
        }
    } else if o.target.is_none() {
        // spec: S-0215 claim 28 row 4 (a destination and no target: the hunt rule applies)
        capture_hunt_rule(w, id, eaten);
    }
    // spec: S-0215 claim 28 row 3 (a destination and a target: no change)
    // spec: S-0215 claim 28 and T2 (the delay is always the standard delay, with the rate of the behaviour current now, claim 6)
    standard_delay(w, id)
}

/// The hunt rule of S-0215 claim 28 (original only): an actor of a
/// computer-controlled owner, other than one of an occupier infantry type,
/// has its destination cleared and queues *hunt*. In our engine it reaches
/// the neutral and special houses and the marked test-only
/// computer-controlled player houses (S-0215 Terms); every other player
/// house is human-controlled and is left as the other effects leave it.
/// *Eaten* never takes it.
fn capture_hunt_rule(w: &mut World, id: ObjectId, eaten: bool) {
    // spec: S-0215 claim 28 (eaten: the hunt rule never applies)
    if eaten {
        return;
    }
    // spec: S-0215 claim 28 (hunt rule, original only: an actor of a computer-controlled owner; an actor of a human-controlled owner is left as the other effects leave it) and Terms (houses in our engine: the neutral and special houses keep the rule; Test-only computer-controlled player house)
    if crate::combat::acquire::player_owned(w, id) {
        return;
    }
    // spec: S-0215 claim 28 (other than one of an occupier infantry type) and S-0508 claim 4 (r19, Q-2926: every unit of an occupier type is left out by its type alone, whether it was refused, lost its target on the way or never set out; an assaulter type that is not an occupier type is not left out)
    if crate::structures::garrison::occupier(w, id) {
        return;
    }
    // spec: S-0215 claim 28 and S-0501 claim 49 (an aircraft, not a strike aircraft, whose retreat or approach behaviour became current at the fall-back ignores the hunt request, S-0501 claim 34, and ends the processing as the fall-back left it)
    if crate::air::queue_ignored(w, id, Behaviour::Hunt) {
        return;
    }
    // spec: S-0215 claim 28 (has its destination cleared, also one that its fall-back to rest or scatter gave it in the same processing)
    crate::econ::assign_destination(w, id, None);
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0215 claim 28 and claim 7 (queues hunt, which replaces whatever the fall-back or scatter queued; without an immediate start, S-0501 claim 49)
        ob.beh.queue(Some(Behaviour::Hunt));
    }
}

/// Area guard (S-0215 claim 26); an aircraft's below its flight height
/// (S-0501 claim 45).
pub(crate) fn process_area_guard(w: &mut World, id: ObjectId) -> i32 {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    let r = w.rules_arc();
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    let Some(u) = r.unit(o.kind, o.type_index).cloned() else {
        return NO_RULE_DELAY;
    };
    let half = r.quirks.ranged_draw_half_span;
    // spec: S-0215 claim 26 row 1 (a slave master for which S-0402's wake-up rule holds: wakes its slaves, the standard delay)
    if o.kind == Kind::Vehicle && crate::seams::slave_wake_rule_holds(w, id) {
        crate::seams::wake_slaves(w, id);
        return standard_delay(w, id);
    }
    if crate::structures::bunker::bunkered(w, id) {
        // spec: S-0215 claim 26 row 2 and S-0606 claim 7 (in a bunker: guard with an immediate start, readiness-dependent under claim 7; 1 tick)
        let rd = ready(w, id);
        if let Some(ob) = w.obj_mut(id) {
            ob.beh.queue_and_start(Some(Behaviour::Guard), rd, t, stale);
        }
        return 1;
    }
    if crate::structures::absorb::holds_instruction(w, id) {
        // spec: S-0605 claim 16 and S-0215 claim 26 row 3 (the enter-nearest instruction is carried out as in guard; in our engine only the absorbing-building one)
        crate::structures::absorb::instruction_turn(w, id);
        // spec: S-0215 claim 26 row 3 (D_r, no draw, with the rate of the behaviour then current: area guard's, enter being queued without a start)
        return rate_delay(w, id);
    }
    if o.kind == Kind::Vehicle && u.harvester {
        // spec: S-0215 claim 26 row 4 (a harvester: harvest at once; 1 + U(1, 10))
        if let Some(ob) = w.obj_mut(id) {
            ob.beh.queue_and_start_at_once(Behaviour::Harvest, t, stale);
        }
        return 1i32.wrapping_add(w.rng.ranged(1, 10, half)); // spec: S-0215 claim 26 row 4 (1 + U(1, 10))
    }
    // spec: S-0215 claim 26 (no row ended the processing: a slave master wakes its slaves anyway when its slave group is at rest, S-0402 claim 14)
    if o.kind == Kind::Vehicle && crate::seams::slave_group_at_rest(w, id) {
        crate::seams::wake_slaves(w, id);
    }
    let mut o = o;
    if o.guard_spot.is_none() && o.beh.queued.is_none() {
        // spec: S-0215 claim 26 (no guard spot and nothing queued: takes its own cell; a spot so taken already qualifies for the table in this processing, Q-0231)
        let spot = Target::Cell(o.pos.cell());
        o.guard_spot = Some(spot);
        if let Some(ob) = w.obj_mut(id) {
            ob.guard_spot = Some(spot);
        }
    }
    // spec: S-0215 claim 26 (the leash and search table for an actor with a guard spot)
    if let Some(gs) = o.guard_spot {
        let gp = match gs {
            Target::Object(x) => w.reference_point(x),
            Target::Cell(_) => target_pos(w, gs),
        };
        let dx = i64::from(o.pos.x - gp.x);
        let dy = i64::from(o.pos.y - gp.y);
        let dz = i64::from(o.pos.z - gp.z);
        // spec: S-0215 F4 (ρ₃ to the guard spot, minus 64 × (W + H) for a building, not below 0)
        let mut dist = root_floor(
            (dx * dx + dy * dy + dz * dz) as u64,
            r.quirks.table_root_distance,
            r.mode,
        ) as i64;
        if let Target::Object(x) = gs
            && let Some(g) = w.obj(x)
            && g.kind == Kind::Building
        {
            dist = (dist - 64 * i64::from(g.foundation.0 + g.foundation.1)).max(0); // spec: S-0215 F4 (β = 64 × (W + H))
        }
        let mobile_spot =
            matches!(gs, Target::Object(x) if w.obj(x).is_some_and(|g| g.kind.is_mobile()));
        let leash = if mobile_spot {
            // spec: S-0215 F5 (guarding a mobile actor: the guard stray distance)
            // spec: S-0215 claim 36 (b) and Questions for the project lead (decided: given by no layer, the area-guard leash instead)
            r.general
                .guard_stray
                .unwrap_or_else(|| leash(u.guard_range))
        } else {
            leash(u.guard_range)
        };
        let firing = t < o.combat.rearm_until;
        // spec: S-0215 claim 26 (the leash: not firing, no destination, distance above L)
        let holds = !firing && o.dest.is_none() && dist > i64::from(leash);
        // spec: S-0215 claim 26 (may search: passive acquisition allowed and the targeting timer run out)
        let may = crate::combat::acquire::may_search_passively(w, id) && t >= o.targeting_timer;
        let idle = |w: &mut World| {
            if o.kind == Kind::Infantry {
                let ip = moving(w, id);
                infantry::idle_attempt(w, id, ip);
            }
        };
        if holds {
            // spec: S-0215 claim 26 (drops any target, takes the guard spot as destination)
            if let Some(ob) = w.obj_mut(id) {
                crate::spawn::clear_own_target(ob);
                ob.combat.burst = 0;
                ob.dest = Some(gs);
                // spec: S-0107 claim 15 (a destination assigned: the destination-driven get-up; the leash holds only without one)
                crate::infantry::destination_assigned(w, id, None);
            }
        }
        if (holds || o.target.is_none()) && may {
            // spec: S-0215 claim 26 (the area search; a search that finds a target ends the processing with 1 tick)
            let found = crate::combat::acquire::search(
                w,
                id,
                crate::combat::acquire::SearchKind::Area,
                crate::combat::acquire::SearchContext::Area,
            );
            if found.is_some() {
                return 1;
            }
            // spec: S-0518 claim 14 (area guard: after the guard spot and automatic search steps, a charger still without a target looks for a building to charge; when it takes one its idle action does not run and the area guard delay follows: the rate delay plus a draw over 1 to 5, divided by 6 under the close-range condition on the new building; Q-0985 answered by revision 3)
            if !(w.obj(id).is_some_and(|x| x.target.is_none())
                && crate::linked::start_charging(w, id))
            {
                idle(w);
            }
        } else if holds || o.target.is_none() {
            // spec: S-0215 claim 26 (may not search: attempts its idle action)
            idle(w);
        } else {
            // spec: S-0215 claim 26 (the leash does not hold and a target is present: the approach step of claim 37)
            crate::approach::step(w, id);
        }
    }
    if w.obj(id).is_none() {
        return NO_RULE_DELAY;
    }
    let dr = rate_delay(w, id);
    // spec: S-0215 claim 26 and F0 (D = k × D_r + U(1, 5), k = 2 for aircraft and 1 for every other kind)
    let k = if o.kind == Kind::Aircraft { 2 } else { 1 };
    let d = dr.wrapping_mul(k).wrapping_add(w.rng.ranged(1, 5, half)); // spec: S-0215 F0 (U(1, 5))
    // spec: S-0215 F3 (trunc(D / 6) under the close-range condition)
    if close_range(w, id) { d / 6 } else { d }
}

// ---------------------------------------------------------------------------
// Gates (S-0215 claim 35)

/// The stages of a gate's *open*, held in the progress count (S-0215 Terms,
/// *progress count*: start 0, opening 1, waiting 2, closing-start 3,
/// closing 4, finished 5).
mod stage {
    // spec: S-0215 Terms (progress count: the stages of a gate's open are 0 to 5 in the order of its table)
    pub const START: i32 = 0;
    pub const OPENING: i32 = 1; // spec: S-0215 Terms (stage opening 1)
    pub const WAITING: i32 = 2; // spec: S-0215 Terms (stage waiting 2)
    pub const CLOSING_START: i32 = 3; // spec: S-0215 Terms (stage closing-start 3)
    pub const CLOSING: i32 = 4; // spec: S-0215 Terms (stage closing 4)
    pub const FINISHED: i32 = 5; // spec: S-0215 Terms (stage finished 5)
}

/// Whether a building is a gate (S-0209 Terms, the gate flag).
fn is_gate(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        o.kind == Kind::Building
            && w.rules()
                .buildings
                .get(o.type_index as usize)
                .is_some_and(|b| b.gate)
    })
}

/// A gate's door time Td and close delay Tc in ticks, a negative value
/// behaving as 0 (S-0215 F9).
fn gate_times(w: &World, id: ObjectId) -> (i32, i32) {
    w.obj(id)
        .and_then(|o| w.rules().buildings.get(o.type_index as usize))
        // spec: S-0215 F9 (a negative result behaves as 0)
        .map_or((0, 0), |b| {
            (b.gate_door_time.max(0), b.gate_close_delay.max(0))
        })
}

/// A gate's passage state (S-0215 claim 35): open when its current
/// behaviour is *open* and its door is at rest open, closed otherwise. This
/// is what S-0209's gate rows call an open or a closed gate.
pub fn gate_open(w: &World, gate: ObjectId) -> bool {
    // spec: S-0215 claim 35 (passage state open: open current and the door at rest open)
    w.obj(gate)
        .is_some_and(|o| o.beh.current == Some(Behaviour::Open) && o.beh.gate.door == Door::Open)
}

/// A closed gate in cell `c` whose owner is allied with `mover`'s house,
/// the gate owner's view (S-0209 rows WP12 and VP9; S-0207 claim 15).
pub fn closed_friendly_gate(w: &World, mover: ObjectId, c: CellXY) -> Option<ObjectId> {
    let owner = w.obj(mover)?.owner;
    w.cells.list(c, Layer::Ground).iter().copied().find(|&g| {
        is_gate(w, g) && !gate_open(w, g) && w.allied(w.obj(g).and_then(|x| x.owner), owner)
    })
}

/// The *closed friendly gate* outcome of S-0209 has arisen for `gate`
/// (S-0215 claim 35 (b)): unless *open* is current with the door opening or
/// at rest open, the gate drops any queued behaviour and *open* becomes
/// current at once, whatever its readiness (a behaviour start). The entry
/// rules call this when the outcome arises for a unit's move, never while
/// only asking for an outcome (S-0214 claim 2: asking changes nothing).
pub fn gate_met(w: &mut World, gate: ObjectId) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    if !is_gate(w, gate) {
        return;
    }
    let Some(o) = w.obj_mut(gate) else { return };
    let moving_open = matches!(o.beh.gate.door, Door::Opening { .. } | Door::Open);
    // spec: S-0215 claim 35 (b) (open current with the door opening or at rest open: nothing changes)
    if o.beh.current == Some(Behaviour::Open) && moving_open {
        return;
    }
    // spec: S-0215 claim 35 (b) (drops any queued behaviour; open current at once, a behaviour start)
    o.beh.queued = None;
    o.beh.force(Behaviour::Open, t, stale);
}

/// The door motion at every turn of the gate (S-0215 claim 35 (d)): a
/// moving door is at rest on the first tick on which its time has passed
/// since the motion began. It moves whatever the gate's behaviour and
/// whether or not the gate processes in that turn, before its processing,
/// so a processing in the turn in which the door reaches rest finds it at
/// rest.
fn door_update(w: &mut World, id: ObjectId) {
    let t = w.tick;
    let Some(o) = w.obj_mut(id) else { return };
    // spec: S-0215 claim 35 (d) (at rest once Td ticks have passed since the motion began; at every turn of the gate, before its processing)
    o.beh.gate.door = match o.beh.gate.door {
        Door::Opening { since, ticks } if t.wrapping_sub(since) >= ticks => Door::Open,
        Door::Closing { since, ticks } if t.wrapping_sub(since) >= ticks => Door::Closed,
        d => d,
    };
}

/// Whether an object other than the gate is in the ground layer of one of
/// its foundation cells (S-0215 claim 35, stage *waiting*).
fn gate_cells_taken(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    crate::objects::foundation_cells(o.pos.cell(), o.foundation)
        .into_iter()
        .any(|c| {
            w.cells
                .list(c, crate::world::Layer::Ground)
                .iter()
                .any(|&x| x != id)
        })
}

/// *Open* for a building (S-0215 claim 35 (c)); returns the delay.
fn process_open(w: &mut World, id: ObjectId) -> i32 {
    let t = w.tick;
    if !is_gate(w, id) {
        if let Some(ob) = w.obj_mut(id) {
            ob.beh.queue(Some(Behaviour::Guard));
        }
        // spec: S-0215 claim 35 (c) and Constants (a building that is not a gate: queues guard, 1 tick, no draw)
        return 1;
    }
    let (td, tc) = gate_times(w, id);
    let snaps = w.rules().quirks.gate_reverse_snaps;
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    let gate = o.beh.gate;
    let set = |w: &mut World, next: i32, door: Door, restart: bool| {
        if let Some(ob) = w.obj_mut(id) {
            ob.beh.progress = next;
            ob.beh.gate.door = door;
            if restart {
                ob.beh.gate.countdown = t;
            }
        }
    };
    let queue_guard = |w: &mut World| {
        if let Some(ob) = w.obj_mut(id) {
            ob.beh.queue(Some(Behaviour::Guard));
        }
    };
    // spec: S-0215 Constants (the delay of a gate's open stages other than Std: 0 ticks, no draw)
    const NOW: i32 = 0;
    // spec: S-0215 claim 35 (c) (the stage table; Std is the standard delay with the rate of open, one draw)
    match o.beh.progress {
        stage::START => match gate.door {
            Door::Open => {
                // spec: S-0215 claim 35 (c) (start, at rest open: countdown restarted, waiting, Std)
                set(w, stage::WAITING, Door::Open, true);
                standard_delay(w, id)
            }
            Door::Closed => {
                // spec: S-0215 claim 35 (c) (start, at rest closed: the door starts opening, Td ticks; countdown restarted)
                let door = Door::Opening {
                    since: t,
                    ticks: td,
                };
                set(w, stage::OPENING, door, true);
                NOW
            }
            Door::Opening { .. } => {
                // spec: S-0215 claim 35 (c) (start, opening: countdown restarted)
                set(w, stage::OPENING, gate.door, true);
                NOW
            }
            Door::Closing { since, .. } => {
                let door = if snaps {
                    // spec: S-0215 claim 35 d2 (quirks.gate_reverse_snaps on: q = 1 at once, at rest open at the next gate turn)
                    Door::Opening { since: t, ticks: 0 }
                } else {
                    // spec: S-0215 claim 35 d2 (off: closing ran a ticks; the reopening takes a ticks with q = R(min(Td, Td − a + e) / Td), e counted from the reversal; Q-0685 answered)
                    let td = match gate.door {
                        Door::Closing { ticks, .. } => ticks.max(0),
                        _ => 0,
                    };
                    let a = t.wrapping_sub(since).clamp(0, td);
                    Door::Opening {
                        since: t.wrapping_sub(td.wrapping_sub(a)),
                        ticks: td,
                    }
                };
                set(w, stage::OPENING, door, true);
                NOW
            }
        },
        stage::OPENING => {
            match gate.door {
                // spec: S-0215 claim 35 (c) (opening, at rest open: waiting)
                Door::Open => set(w, stage::WAITING, Door::Open, false),
                // spec: S-0215 claim 35 (c) (opening, at rest closed: queues guard, finished)
                Door::Closed => {
                    queue_guard(w);
                    set(w, stage::FINISHED, Door::Closed, false);
                }
                // spec: S-0215 claim 35 (c) (opening, still moving: unchanged)
                _ => {}
            }
            NOW
        }
        stage::WAITING => {
            if gate_cells_taken(w, id) {
                // spec: S-0215 claim 35 (c) (waiting, an object in the ground layer of a foundation cell: countdown restarted)
                set(w, stage::WAITING, gate.door, true);
            } else if t.wrapping_sub(gate.countdown) >= tc {
                // spec: S-0215 claim 35 (c) (waiting, cells empty and the countdown run out: closing-start)
                set(w, stage::CLOSING_START, gate.door, false);
            }
            standard_delay(w, id)
        }
        stage::CLOSING_START => {
            // spec: S-0215 claim 35 (c) (closing-start: the door starts closing, Td ticks)
            let door = Door::Closing {
                since: t,
                ticks: td,
            };
            set(w, stage::CLOSING, door, false);
            NOW
        }
        stage::CLOSING => {
            if gate.door == Door::Closed {
                // spec: S-0215 claim 35 (c) (closing, at rest closed: queues guard, finished)
                queue_guard(w);
                set(w, stage::FINISHED, Door::Closed, false);
            }
            NOW
        }
        // spec: S-0215 claim 35 (c) (a finished gate changes nothing and takes Std)
        _ => standard_delay(w, id),
    }
}

/// The area-guard leash L (S-0215 F5) for a guard range g in leptons;
/// weapon ranges (g = 0) wait for M4 and count as 0.
fn leash(g: i32) -> i32 {
    // spec: S-0215 F5 (G = min(max(2g, 0), 4096); L = ⌊11G / 10⌋)
    let big_g = g.saturating_mul(2).clamp(0, 4096);
    11 * big_g / 10 // spec: S-0215 F5 (L = ⌊11G / 10⌋)
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: S-0215 F5 (L = ⌊11 × G / 10⌋ with G capped at 4096: at most 4505)
    #[test]
    fn leash_values() {
        assert_eq!(leash(0), 0);
        assert_eq!(leash(256), 563);
        assert_eq!(leash(5000), 4505);
        assert_eq!(leash(-3), 0);
    }
}
