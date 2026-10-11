//! The tick pipeline (S-0203): the parts of a tick in their fixed places.
//!
//! The parts that M3 has no topic for yet (map triggers, planted bombs, the
//! weather storm, chrono flashes, resource growth, diving aircraft, teams,
//! disc lasers, radiation, pulses, blast rings, crates, production lines and
//! houses) are empty seams in their places of S-0203 claim 1, so that their
//! draws land where the original takes them once they are written.

use crate::rules::{Kind, Locomotor};
use crate::world::{ObjectId, World};
use crate::{drive, infantry, occupancy, orders, process, turret, walk};

/// Runs one tick (S-0203 claim 1, the draw order of its rows).
pub fn step(w: &mut World) {
    // spec: S-0203 claim 1 (draw order: row 1, 9, 2, 10, 11, 12, 3, 13, 14, then the turn pass, row 5)
    // Rows 1, 9, 2, 10, 11, 12, 3, 13 and 14 (and row 4) have no topic in M3: empty seams.
    turn_pass(w);
    // spec: S-0203 claim 1 (after the turn pass: rows 15, 16, 6, 7, then the orders, row 8)
    // Rows 15, 16, 6 and 7: empty seams.
    // spec: S-0203 claim 7 and S-0213 claim 1 (orders take effect after the turn pass)
    orders::apply_pending(w);
    // spec: S-0203 claim 1 (after the last part the tick number grows by 1)
    w.tick = w.tick.wrapping_add(1);
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
    let killed = o.combat.killed;
    // spec: S-0309 claim 15 (the expected remaining health changes in the object's own turn)
    crate::combat::acquire::expected_health_turn(w, id);
    let Some(o) = w.obj(id) else { return };
    let kind = o.kind;
    if let Some(ob) = w.obj_mut(id) {
        ob.last_turn = t;
    }
    if killed {
        // A killed object that stays in play (an infantry unit in its death sequence) only plays it out.
        crate::combat::death::infantry_death_turn(w, id);
        return;
    }
    process::turn(w, id);
    if w.obj(id).is_none() {
        return;
    }
    if kind == Kind::Building {
        crate::combat::behave::delayed_shot_turn(w, id);
        return;
    }
    let loco = w
        .obj(id)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        .map(|u| u.locomotor);
    let moved = match loco {
        Some(Locomotor::Drive) => drive::tick(w, id),
        Some(Locomotor::Walk) => walk::tick(w, id),
        _ => false,
    };
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
