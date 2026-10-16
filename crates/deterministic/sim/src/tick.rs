//! The tick pipeline (S-0203): the parts of a tick in their fixed places.
//!
//! The parts that are not built yet (planted-bomb maintenance, S-0519, the
//! weather storm, chrono flashes, diving aircraft, teams, disc lasers,
//! pulses, blast rings and crates) are empty seams in their places
//! of S-0203 claim 1, so that their draws land where the original takes them
//! once they are written. Resource growth (row 9), the production lines (row
//! 6) and the houses (row 7) run the M5a topics (`crate::econ`). Radiation
//! maintenance (row 13) takes no draws and is not built yet.

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
    // spec: S-0203 claim 1 (draw order: row 1, 9, 2, 10, 11, 12, 3, 14, then the turn pass, row 5; row 13, radiation maintenance, takes no draws)
    // spec: S-0203 row 1 and S-0011 claim 6 (the poll of every polled tag, at the start of the tick)
    crate::triggers::poll(w);
    // spec: S-0203 row 9 (growth and spread of resources, S-0401)
    crate::econ::resource::part(w);
    // Row 2 (planted-bomb maintenance, S-0519 claim 10; no draws) is not built: an empty seam. Timed
    // bombs detonate in their carrier's turn in row 5 (S-0203 claim 1, S-0519 claim 4), not here.
    // spec: S-0506 claim 26 and S-0203 claim 1 row 17 (the shared kamikaze re-order cycle: once per tick, outside object turns, after planted-bomb maintenance (row 2), before diving aircraft (row 10) and the turn pass (row 5); it takes no draws)
    crate::spawn::kamikaze_orders(w);
    // Rows 10, 11 and 12 have no topic yet: empty seams.
    // spec: S-0203 claim 1 row 3 (the weather storm's own update and bolt strikes, after rows 10 to 12 and before row 14 and the turn pass; S-0513 claims 14 to 19)
    crate::superweapons::storm::update(w);
    // Rows 14 (and row 4) have no topic yet: empty seams. Row 13 (radiation
    // maintenance) takes no draws (S-0203 row 13) and is not built yet.
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
    // spec: S-0513 claim 29 (the protection tint's phases advance in the object's turns; its draw is game state)
    crate::superweapons::protect::tint_turn(w, id);
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
        if crate::air::crashing(w, id) {
            // spec: S-0501 claim 9 (a crashing aircraft keeps its flying movement until it hits the ground)
            crate::air::tick(w, id);
            crate::air::end_turn(w, id);
            return;
        }
        if crate::naval::sinking(w, id) {
            // spec: S-0312 claims 15 and 32 (a vessel destroyed while sinking stays in play at 0 health because it is sinking); S-0504 claim 4 (each own update of a sinking vessel descends, and it leaves play below −400 leptons); S-0504 claim 13 (no ordinary living updates are asserted for it, so it takes no others)
            crate::naval::sinking_turn(w, id);
            return;
        }
        // A killed object that stays in play (an infantry unit in its death sequence) only plays it out.
        crate::combat::death::infantry_death_turn(w, id);
        return;
    }
    // spec: S-0513 State and ordering and T2 (a warp member's warp runs in its own updates); S-0513 claims 48 and 51 and S-0203 claim 4 (a member still leaving takes no other part of its turn, confirmed; engine design for the other stages before the arrival hold: no ordinary healing, behaviour, movement, firing or turret turning, and (Q-0870 interim) nothing else of the turn either, so the warp update is the whole turn; no ordering of the warp after the protection tint is implied, and ours puts it after)
    if crate::superweapons::warp::turn(w, id) {
        return;
    }
    // spec: S-0504 claim 4 (a sinking vessel's own update first descends, and may leave play at once)
    if crate::naval::sinking_turn(w, id) {
        return;
    }
    // spec: S-0314 claims 9 and 10 (self-healing, in the object's own turn) and claim 19 (a sinking vehicle still self-heals; health does not end sinking)
    crate::combat::heal::turn(w, id);
    let sinking = crate::naval::sinking(w, id);
    // spec: S-0511 claim 10 (a dropped object falls on each of its own updates; the fall and any landing hit precede that turn's due behaviour processing, ordinary locomotion advance and firing check)
    let fell = crate::fall::falling(w, id) && crate::fall::turn(w, id);
    if w.obj(id).is_none_or(crate::combat::damage::dead) {
        return;
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
    if kind.is_mobile() {
        // spec: S-0516 claim 42 and S-0203 (the ordinary cloak update follows due behaviour processing, its shots included, and precedes ordinary locomotion and the separate infantry and vehicle firing)
        crate::cloak::turn(w, id);
    }
    if kind == Kind::Building {
        // spec: S-0513 claims 8 and 13 (a silo's accepted launch leaves on its next update)
        crate::superweapons::nuke::silo_turn(w, id);
        crate::combat::behave::delayed_shot_turn(w, id);
        // spec: S-0405 claims 7 and 9, S-0409 claim 8 (the building's own turn)
        crate::econ::events::turn(w, id, false);
        // spec: S-0501 claim 26 (a pad's service step, in the pad's own turn)
        crate::air::pad_turn(w, id);
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
    // spec: S-0511 claim 10 (while still falling after the fall step, no ordinary locomotion advance and no firing; a surviving landing lifts both refusals in the same turn)
    let falling = crate::fall::falling(w, id);
    let moved = match loco {
        // spec: S-0506 Terms (a rocket missile moves in the rocket-flight movement mode, claims 14 to 19)
        _ if rocket => crate::rocket::tick(w, id),
        _ if falling => fell,
        // spec: S-0507 claim 15 and S-0203 claim 3 (a passenger in play inside an open-topped transport has no movement of its own; its position is the transport's)
        _ if crate::transport::riding(w, id).is_some() => false,
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
        // spec: S-0503 claims 2 to 11 (the hovering locomotor; S-0614 claim 8 lists no hold for it)
        Some(Locomotor::Hover) => crate::hover::tick(w, id),
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
    // spec: S-0507 claim 15 (whenever the transport moves to a new position, its passengers' positions become its position at once); S-0314 claim 23 (with the position change, before the queued-entry retry)
    crate::transport::sync(w, id);
    if kind == Kind::Vehicle && !crate::teleport::phasing_in(w, id) {
        // spec: S-0314 claim 23 (the queued-entry retry, once per ordinary living vehicle turn, after that turn's locomotor movement, self-healing and due behaviour processing; not before movement and not an extra behaviour processing; skipped, the queued entry kept, only while chrono phase-in is still active, not for a hold, pulse disable, deactivation, lift or locomotor replacement; before the attached parasite's step)
        crate::depot::vehicle_turn(w, id);
    }
    // spec: S-0614 claim 18 (an attached parasite acts late in its host's turn: after self-healing, behaviour processing and movement, before the separate firing update; a host killed by it does nothing more)
    crate::parasite::host_turn(w, id);
    if w.obj(id).is_none_or(crate::combat::damage::dead) {
        return;
    }
    // spec: S-0405 claim 13, S-0409 claim 8 (the unit's own turn, after its movement of the tick)
    crate::econ::events::turn(w, id, moved);
    if kind == Kind::Aircraft {
        // spec: S-0501 claims 15, 31 and 37 (the end of an aircraft's update)
        crate::air::end_turn(w, id);
    }
    if kind == Kind::Vehicle {
        crate::combat::fire::reload_turn(w, id);
        crate::combat::fire::firing_frame_turn(w, id);
        // spec: S-0310 claim 9 (the vehicle's own firing step each tick it has a target); S-0301 claim 9 rank 4 and S-0511 claim 10 (a unit still falling cannot fire; the check refuses it)
        let facing_error = crate::combat::behave::vehicle_step(w, id);
        // The turret's target at the end of the tick (S-0206 claim 9), after the movement.
        turret::tick(w, id, facing_error);
    }
    if kind == Kind::Infantry && w.obj(id).is_some() {
        let in_progress = occupancy::in_progress(w, id);
        let has_dest = w.obj(id).is_some_and(|o| o.dest.is_some());
        infantry::update(w, id, moved, in_progress, has_dest);
        // spec: S-0107 claim 11 (firing, after the unit's sequence rules of the tick); S-0301 claim 9 rank 4 and S-0511 claim 10 (a unit still falling cannot fire, even with a firing sequence begun before the drop)
        crate::combat::behave::infantry_step(w, id);
    }
}
