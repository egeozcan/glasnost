//! The tick pipeline (S-0203): the parts of a tick in their fixed places.
//!
//! The parts that have no topic yet (planted bombs, the
//! weather storm, chrono flashes, diving aircraft, teams, disc lasers,
//! radiation, pulses, blast rings and crates) are empty seams in their places
//! of S-0203 claim 1, so that their draws land where the original takes them
//! once they are written. Resource growth (row 9), the production lines (row
//! 6) and the houses (row 7) run the M5a topics (`crate::econ`).

use crate::rules::{Kind, Locomotor};
use crate::world::{ObjectId, World};
use crate::{drive, infantry, occupancy, orders, process, turret, walk};

/// The number of ticks that have run: the tick number, and one more once
/// the final tick has run, since that tick adds nothing to the tick number
/// (S-0201 claim 1).
pub fn ticks_run(w: &World) -> i32 {
    // spec: S-0201 claim 1 (the final tick runs but leaves the tick number at itself)
    w.tick.wrapping_add(i32::from(w.econ.end.finished))
}

/// Runs one tick (S-0203 claim 1, the draw order of its rows).
pub fn step(w: &mut World) {
    // spec: S-0203 claim 1 (draw order: row 1, 9, 2, 10, 11, 12, 3, 13, 14, then the turn pass, row 5)
    // spec: S-0203 row 1 and S-0011 claim 6 (the poll of every polled tag, at the start of the tick)
    crate::triggers::poll(w);
    // spec: S-0203 row 9 (growth and spread of resources, S-0401)
    crate::econ::resource::part(w);
    // Rows 2, 10, 11, 12, 3, 13 and 14 (and row 4) have no topic yet: empty seams.
    // spec: S-0203 claim 1 (other work once per tick, no draws: the regrowth of shroud; S-0409 claim 18)
    // spec: S-0409 claim 18 and State and ordering (a regrowth step due in the same tick as the re-cover sweep, tick 0 included, has its effect before the sweep)
    crate::econ::shroud::regrowth(w);
    // spec: S-0409 claim 13 (the re-cover sweep, before the turn pass of its tick)
    crate::econ::shroud::sweep(w);
    turn_pass(w);
    // spec: S-0203 claim 1 (after the turn pass: rows 15, 16, 6, 7, then the orders, row 8)
    // Rows 15 and 16 (blast rings, crate respawns): empty seams.
    // spec: S-0203 claim 8 and row 6 (the production lines see the whole turn pass)
    crate::econ::production::part(w);
    // spec: S-0203 claim 8 and row 7 (the houses, in house order, after the production lines)
    crate::econ::houses::part(w);
    // spec: S-0203 claim 7 and S-0213 claim 1 (orders take effect after the turn pass)
    orders::apply_pending(w);
    // spec: S-0011 claim 11 (a one-time tag that fired is deleted at the end of the tick)
    crate::triggers::end_tick(w);
    if w.econ.end.over == Some(w.tick) || w.econ.end.finished {
        // spec: S-0201 claim 1 and S-0203 State and ordering (the final tick runs in full but does not add 1: afterwards the tick number is the last tick's; no next tick runs, S-0414 claim 8)
        w.econ.end.finished = true;
    } else {
        // spec: S-0203 claim 1 and S-0201 claim 1 (after the last part the tick number grows by 1, except in the final tick)
        w.tick = w.tick.wrapping_add(1);
    }
    // Derived data, not state: the regions are brought up to date between
    // ticks, so that input rules can read them (crate::targeting).
    crate::regions::ensure(w);
}

/// The turn pass (S-0203 claims 2 to 4, 10 and 11).
fn turn_pass(w: &mut World) {
    w.update.begin_pass();
    while let Some(id) = w.update.next_turn() {
        object_turn(w, id);
    }
    w.update.end_pass();
}

/// One object's turn. Within a turn: the behaviour part (S-0215 claims 2,
/// 11 and 33), then the locomotor, then the infantry sequence rules
/// (S-0107 claims 8 to 10 and 15 run after the movement of the tick).
fn object_turn(w: &mut World, id: ObjectId) {
    let t = w.tick;
    if w.projectiles.contains_key(&id) {
        // spec: S-0203 claim 5 (a shot is resolved in its projectile's own turn)
        crate::combat::projectile::turn(w, id);
        return;
    }
    let Some(o) = w.obj(id) else { return };
    // A building in a delayed-kill countdown is killed but alive (S-0302 claim 16): it takes its whole turn.
    let dead = crate::combat::damage::dead(o);
    // spec: S-0309 claim 15 (the expected remaining health changes in the object's own turn)
    crate::combat::acquire::expected_health_turn(w, id);
    let Some(o) = w.obj(id) else { return };
    let kind = o.kind;
    if let Some(ob) = w.obj_mut(id) {
        ob.last_turn = t;
    }
    if dead {
        if kind == Kind::Building {
            // spec: S-0312 claim 21 (a building awaiting removal: the general part of its turn, then its removal)
            process::turn(w, id);
            crate::combat::death::removal_turn(w, id);
            return;
        }
        if w.obj(id).is_some_and(|o| o.rocket.is_some()) {
            // spec: S-0506 claim 20 (a destroyed rocket missile above the terrain keeps the rocket-flight movement mode)
            crate::rocket::tick(w, id);
            return;
        }
        // A killed object that stays in play (an infantry unit in its death sequence) only plays it out.
        crate::combat::death::infantry_death_turn(w, id);
        return;
    }
    // spec: S-0504 claim 4 (a sinking vessel's own update first descends, and may leave play at once)
    if crate::naval::sinking_turn(w, id) {
        return;
    }
    let sinking = crate::naval::sinking(w, id);
    if !sinking {
        // spec: S-0314 claims 9 and 10 (self-healing, in the object's own turn); a sinking vessel does not heal (Q-0642, interim)
        crate::combat::heal::turn(w, id);
    }
    process::turn(w, id);
    if w.obj(id).is_none() {
        return;
    }
    // spec: S-0506 claim 2 (a spawner's launch pass, when one is due, in its own turn; claim 24: a vehicle or ship spawner's own firing step comes later in the turn, so its shot waits for the next pass)
    crate::spawn::turn(w, id);
    if w.obj(id).is_none() {
        return;
    }
    if kind == Kind::Building {
        crate::combat::behave::delayed_shot_turn(w, id);
        // spec: S-0405 claims 7 and 9, S-0409 claim 8 (the building's own turn)
        crate::econ::events::turn(w, id, false);
        // spec: S-0302 claim 16 (the countdown's expiry hit, in the building's own turn, after the actions due earlier in that turn: self-healing, behaviour, ordinary and animation-delayed shots; Q-0170)
        crate::combat::damage::delay_kill_expiry_turn(w, id);
        return;
    }
    if kind == Kind::Vehicle {
        // spec: S-0203 claim 12 (a vehicle master's due group update comes after that turn's behaviour start and processing, whose destination and group-state changes it sees, and before its movement; a destination it assigns is available to that movement)
        crate::econ::slaves::group_turn(w, id);
        if w.obj(id).is_none() {
            return;
        }
    }
    let loco = w
        .obj(id)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        .map(|u| u.locomotor);
    let rocket = w.obj(id).is_some_and(|o| o.rocket.is_some());
    // spec: S-0614 claim 8 (a held unit does not move under the driving, ship, flying, teleporting and tunnelling locomotors; walking and rocket ignore the hold)
    let held = crate::parasite::held(w, id);
    let moved = match loco {
        // spec: S-0506 Terms (a rocket missile moves in the rocket-flight movement mode, claims 14 to 19)
        _ if rocket => crate::rocket::tick(w, id),
        // spec: S-0504 claim 5 (a sinking vessel makes no progress by its locomotor, turning and speed-fraction updates included)
        _ if sinking => false,
        Some(Locomotor::Drive) if held => false,
        Some(Locomotor::Drive) => {
            let moved = drive::tick(w, id);
            // spec: S-0207 claim 23 (a drive vehicle's wake, created right after its drive update; Q-0600 answered)
            drive::driving_wake(w, id);
            moved
        }
        Some(Locomotor::Walk) => walk::tick(w, id),
        // spec: S-0417 claim 5 (a tunnelling or teleporting vehicle drives temporarily for its factory exit)
        _ if crate::econ::release::temporary_drive(w, id) => !held && drive::tick(w, id),
        // spec: S-0207 claim 20 (d) (the jump on the first movement update after L is fixed; the behaviour is processed before it)
        Some(Locomotor::Teleport) => crate::teleport::tick(w, id),
        // spec: S-0501 claims 1 to 10 (aircraft flight)
        Some(Locomotor::Fly) => !held && crate::air::tick(w, id),
        // A rocket-locomotor type outside the rocket-flight mode has no movement of its own (S-0506 Terms).
        // spec: S-0504 claim 1 (the ship locomotor)
        Some(Locomotor::Ship) => !held && crate::naval::tick(w, id),
        _ => false,
    };
    // spec: S-0614 claim 18 (an attached parasite acts late in its host's turn: after self-healing, behaviour processing and movement, before the separate firing update; a host killed by it does nothing more)
    crate::parasite::host_turn(w, id);
    if w.obj(id).is_none_or(crate::combat::damage::dead) {
        return;
    }
    // spec: S-0405 claim 13, S-0409 claim 8 (the unit's own turn, after its movement of the tick)
    crate::econ::events::turn(w, id, moved);
    if kind == Kind::Vehicle {
        crate::combat::fire::reload_turn(w, id);
        crate::combat::fire::firing_frame_turn(w, id);
        // spec: S-0310 claim 9 (the vehicle's own firing step each tick it has a target)
        let facing_error = crate::combat::behave::vehicle_step(w, id);
        // The turret's target at the end of the tick (S-0206 claim 9), after the movement.
        turret::tick(w, id, facing_error);
    }
    if kind == Kind::Infantry && w.obj(id).is_some() {
        let in_progress = occupancy::in_progress(w, id);
        let has_dest = w.obj(id).is_some_and(|o| o.dest.is_some());
        infantry::update(w, id, moved, in_progress, has_dest);
        // spec: S-0107 claim 11 (firing, after the unit's sequence rules of the tick)
        crate::combat::behave::infantry_step(w, id);
    }
}
