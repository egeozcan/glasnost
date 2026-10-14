//! Where combat enters the behaviours: the vehicle's firing step (S-0310
//! claim 9), infantry firing (S-0107 claim 11), buildings in guard and
//! attack (S-0215 claim 32) and animation-delayed building shots (S-0301
//! claim 14).

use glasnost_num::geom::{delta, direction};

use crate::combat::check::{Answer, building_aim_point, check};
use crate::combat::range::target_point;
use crate::combat::weapon::{choose_slot, slot_weapon};
use crate::infantry::Seq;
use crate::rules::{Behaviour, Kind};
use crate::world::{ObjectId, Pos, Target, World};

/// The direction of a target from a point (S-0204 claim 8).
fn dir(w: &World, from: Pos, t: Target) -> u16 {
    let to = target_point(w, t);
    direction(
        i64::from(to.x - from.x),
        i64::from(to.y - from.y),
        w.rules().quirks.table_direction,
    )
}

/// A vehicle's own firing step (S-0310 claim 9). Returns whether the check
/// gave a facing error (for the turret rows of S-0206 claim 9).
pub fn vehicle_step(w: &mut World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    if o.kind != Kind::Vehicle || o.combat.killed {
        return false;
    }
    let Some(t) = o.target else { return false };
    let rules = w.rules_arc();
    let Some(u) = rules.unit(o.kind, o.type_index) else {
        return false;
    };
    // spec: S-0310 claim 9 (the firing check, range test included, for the weapon it would use)
    let slot = choose_slot(w, id, Some(t));
    let answer = check(w, id, Some(t), slot, true);
    match answer {
        Answer::CanFire => {
            if u.vehicle.deploy_to_fire {
                // spec: S-0310 claim 9 (a deploy-to-fire type deploys instead of firing; the deploy specs own the deploy)
                w.note("S-0310 claim 9: deploy-to-fire vehicles do not deploy yet (deploy specs)");
            } else {
                // spec: S-0310 claim 9 (any other vehicle fires; a visceroid's animation stage is presentation)
                crate::combat::fire::shot(w, id, slot, t);
            }
            false
        }
        Answer::NotFacing => {
            if u.vehicle.deploy_to_fire {
                return false;
            }
            if u.turret {
                // spec: S-0310 claim 9 (with the turret flag: the turret's target facing becomes the direction of the target)
                true
            } else {
                let moving = crate::occupancy::in_progress(w, id);
                if o.dest.is_none() && !moving {
                    // spec: S-0310 claim 9 and S-0206 claim 9 (without a turret: the body turns, the turret follows)
                    let d = dir(w, w.reference_point(id), t);
                    let tick = w.tick;
                    let omega = u.turn_rate;
                    if let Some(ob) = w.obj_mut(id) {
                        ob.body.turn_to(d, tick, omega);
                        let bt = ob.body.target;
                        ob.turret.turn_to(bt, tick, omega);
                    }
                }
                false
            }
        }
        Answer::Illegal => {
            // spec: S-0310 claim 11 (a vehicle whose default weapon heals: drops a target that is not a vehicle or is at full health)
            let default_heals = slot_weapon(w, id, choose_slot(w, id, None))
                .is_some_and(|x| rules.combat.weapon(x).damage < 0);
            if default_heals {
                let keep = matches!(t, Target::Object(x)
                    if w.obj(x).is_some_and(|ob| ob.kind == Kind::Vehicle)
                        && crate::combat::damage::strength(w, x) > w.obj(x).map_or(0, |ob| ob.health));
                if !keep && let Some(ob) = w.obj_mut(id) {
                    ob.target = None;
                    ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
                }
            }
            false
        }
        _ => false,
    }
}

/// The turret target rows of S-0206 claim 9 for a vehicle with a target,
/// after its firing step; `facing_error` is the step's answer. Returns the
/// target facing to give, if any.
pub fn turret_target(w: &World, id: ObjectId, facing_error: bool) -> Option<u16> {
    let o = w.obj(id)?;
    let t = o.target?;
    let r = w.rules();
    let u = r.unit(o.kind, o.type_index)?;
    let tick = w.tick;
    // spec: S-0206 Terms (the turret weapon: the numbered weapon at the weapon number, else the primary)
    let tw = if u.combat.multi_weapon {
        slot_weapon(w, id, crate::combat::weapon::weapon_number(w, id))
    } else {
        slot_weapon(w, id, 0)
    };
    let was_turning = o.turret.turning(tick.wrapping_sub(1));
    let d = dir(w, w.reference_point(id), t);
    let omni = tw.is_some_and(|x| r.combat.weapon(x).omni_fire);
    // spec: S-0206 claim 9 (a target, not turning at the end of the previous tick, a turret weapon that is not omni-fire: the direction of the target; turret-locked weapons are not imported)
    if !was_turning && tw.is_some() && !omni {
        return Some(d);
    }
    // spec: S-0206 claim 9 (the facing-error row)
    if facing_error {
        return Some(d);
    }
    // spec: S-0206 claim 9 (otherwise no change, quirks.turret_reaim_after_turn)
    if !r.quirks.turret_reaim_after_turn && tw.is_some() && !omni {
        return Some(d);
    }
    None
}

/// Infantry firing (S-0107 claim 11), after the unit's sequence rules of
/// the tick.
// spec: S-0107 claim 11 (four fire-frame fields)
pub fn infantry_step(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else { return };
    if o.kind != Kind::Infantry || o.combat.killed || o.health <= 0 {
        return;
    }
    let rules = w.rules_arc();
    let Some(u) = rules.unit(o.kind, o.type_index) else {
        return;
    };
    let tick = w.tick;
    let Some(s) = o.infantry.clone() else { return };
    let begins_frozen = s.attack && s.seq_delay == 0;
    let end_attack = |w: &mut World, ask: bool| {
        let prone = w
            .obj(id)
            .and_then(|o| o.infantry.as_ref())
            .is_some_and(|s| s.prone);
        let deploying = w
            .obj(id)
            .and_then(|o| o.infantry.as_ref())
            .and_then(|s| s.seq)
            .is_some_and(Seq::is_deploy);
        if let Some(st) = w.obj_mut(id).and_then(|o| o.infantry.as_mut()) {
            st.attack = false;
        }
        if ask {
            // spec: S-0107 claim 11 (the unit asks, not forced, for deployed, prone or stand)
            let q = if deploying {
                Seq::Deployed
            } else if prone {
                Seq::Prone
            } else {
                Seq::Stand
            };
            crate::infantry::request(w, id, q, false);
        }
    };
    if begins_frozen {
        // spec: S-0107 claim 11 (a tick that begins with a current sequence whose stage delay is 0 ends the attack state and asks for stand)
        end_attack(w, true);
    }
    let Some(t) = o.target else {
        if s.attack {
            // spec: S-0107 claim 11 (the attack state ends when the unit has no target)
            end_attack(w, false);
        }
        return;
    };
    if s.attack && o.combat.last_shot_target != Some(t) {
        // spec: S-0107 claim 11 (the attack state ends when its target changes)
        end_attack(w, false);
    }
    let attack = w
        .obj(id)
        .and_then(|o| o.infantry.as_ref())
        .is_some_and(|s| s.attack);
    let slot = choose_slot(w, id, Some(t));
    if !attack {
        // spec: S-0107 claim 11 (with a target, not in the attack state and the weapon ready: enter it)
        if check(w, id, Some(t), slot, true) != Answer::CanFire {
            return;
        }
        let d = dir(w, w.reference_point(id), t);
        let keeps = rules.quirks.set_facing_keeps_target;
        let secondary = slot == 1;
        let prone = s.prone;
        let has = |q: Seq| u.sequences.get(q.key()).is_some_and(|e| e.count > 0);
        let deploying = s.seq.is_some_and(Seq::is_deploy);
        // spec: S-0107 claim 11 (the firing kind by priority: jumpjet, deploying, secondary prone, secondary, prone, standing)
        let q = if u.jumpjet {
            Seq::FireAirborne
        } else if deploying {
            Seq::DeployedFire
        } else if secondary && prone && has(Seq::SecondaryFireProne) {
            Seq::SecondaryFireProne
        } else if secondary && has(Seq::SecondaryFire) {
            Seq::SecondaryFire
        } else if prone {
            Seq::FireProne
        } else {
            Seq::FireStanding
        };
        if let Some(ob) = w.obj_mut(id) {
            // spec: S-0107 claim 11 (the body facing is set at once to the direction of the target)
            ob.body.set(d, tick, keeps);
            if let Some(st) = ob.infantry.as_mut() {
                st.attack = true;
            }
            ob.combat.last_shot_target = Some(t);
            // spec: S-0208 claim 17 (exception: a destination that is the target object itself is lost, and any movement still in progress ends, as the firing sequence starts)
            if matches!(t, Target::Object(_)) && ob.dest == Some(t) {
                ob.dest = None;
                ob.path.steps.clear();
                if let Some(s) = ob.walk.as_mut() {
                    s.fraction = false;
                }
            }
        }
        crate::infantry::request(w, id, q, false);
    }
    // The shot: on every tick in the attack state on which the stage equals F.
    let Some(st) = w.obj(id).and_then(|o| o.infantry.clone()) else {
        return;
    };
    if !st.attack {
        return;
    }
    let ff = u.raw.fire_frames.unwrap_or([0; 4]);
    let secondary = slot == 1;
    let has = |q: Seq| u.sequences.get(q.key()).is_some_and(|e| e.count > 0);
    // spec: S-0107 claim 11 (the fire frame F by stance and weapon, quirks.fire_frame_by_stance)
    let f = match (secondary, st.prone) {
        (true, true) if has(Seq::SecondaryFireProne) => ff[3],
        (true, false) if has(Seq::SecondaryFire) => ff[2],
        (_, true) => ff[1],
        (_, false) => ff[0],
    };
    if st.stage(tick) == f {
        if check(w, id, Some(t), slot, true) == Answer::CanFire {
            crate::combat::fire::shot(w, id, slot, t);
        } else {
            // spec: S-0107 claim 11 (not ready at the fire frame: the attack state ends)
            end_attack(w, true);
        }
    }
}

/// The anti-air delay D_aa + U(0, 2) of a building (S-0215 claims 21, 32).
fn aa_delay(w: &mut World, id: ObjectId) -> i32 {
    let half = w.rules().quirks.ranged_draw_half_span;
    let cur = w.obj(id).and_then(|o| o.beh.current);
    let daa = cur.map_or(450, |b| w.rules().behaviour(b).aa_rate_delay); // spec: S-0215 claim 4 (no behaviour: 450 ticks)
    // spec: S-0215 claim 32 (D_aa + U(0, 2))
    daa.wrapping_add(w.rng.ranged(0, 2, half))
}

/// A building's processing of *guard*, *sticky*, *area guard* and
/// *attack* (S-0215 claim 32). Returns the delay.
// spec: S-0215 F8 (b = |256 × w₈|, w₈ the value modulo 256 in −128 … 127)
pub fn building_process(w: &mut World, id: ObjectId) -> i32 {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let Some(o) = w.obj(id).cloned() else {
        return 450; // spec: S-0215 claim 4 (450 ticks)
    };
    let rules = w.rules_arc();
    let half = rules.quirks.ranged_draw_half_span;
    let tick = w.tick;
    let b = &rules.buildings[o.type_index as usize];
    let armed = crate::combat::weapon::slot_weapon(w, id, 0).is_some();
    let cur = o.beh.current;
    if cur == Some(Behaviour::Attack) {
        if b.building.sam {
            return sam_attack(w, id);
        }
        let Some(t) = o.target else {
            // spec: S-0215 claim 32 (no target: queues guard and starts it at once, 1 tick)
            if let Some(ob) = w.obj_mut(id) {
                ob.beh
                    .queue_and_start_at_once(Behaviour::Guard, tick, stale);
            }
            return 1;
        };
        if let Some(ob) = w.obj_mut(id) {
            ob.beh.ready_mark = true;
        }
        let slot = choose_slot(w, id, Some(t));
        let mut answer = check(w, id, Some(t), slot, true);
        if answer == Answer::NotFacing && b.turret && b.building.voxel_turret {
            let v = b.raw.rot.unwrap_or(0);
            let (zero, bound) = if rules.quirks.turret_snap_bound_wraps {
                // spec: S-0215 claim 32 and F8 (turret snap: v = 0 or ε ≤ b, b = |256 w₈|, w₈ the field modulo 256)
                let w8 = i32::from(v.rem_euclid(256) as u8 as i8);
                (v == 0, (256 * w8).abs())
            } else {
                // spec: S-0215 Edge cases (quirks.turret_snap_bound_wraps off: b = 256 × ω, the snap when ω = 0 or ε ≤ b)
                (b.turn_rate == 0, 256 * b.turn_rate)
            };
            let d = dir(w, building_aim_point(w, id), t);
            let eps = delta(o.turret.value(tick), d).abs();
            if zero || eps <= bound {
                let keeps = rules.quirks.set_facing_keeps_target;
                if let Some(ob) = w.obj_mut(id) {
                    ob.turret.set(d, tick, keeps);
                }
                answer = check(w, id, Some(t), slot, true);
            }
        }
        let rotate = |w: &mut World| {
            if b.turret {
                let d = dir(w, building_aim_point(w, id), t);
                let omega = b.turn_rate;
                if let Some(ob) = w.obj_mut(id) {
                    ob.turret.turn_to(d, tick, omega);
                }
            }
        };
        // spec: S-0215 claim 32 (the table of answers: fire 1; rotate 2; drop and guard 1; others rotate 1)
        return match answer {
            Answer::CanFire => {
                if b.building.anim_delayed_fire {
                    // spec: S-0301 claim 14 (an animation-delayed building starts its delayed shot instead)
                    let d = b.building.delayed_fire_delay;
                    if let Some(ob) = w.obj_mut(id) {
                        ob.combat.delayed_shot = Some(tick.wrapping_add((d - 1).max(0)));
                    }
                } else {
                    crate::combat::fire::shot(w, id, slot, t);
                }
                1
            }
            Answer::NotFacing | Answer::Rearming => {
                rotate(w);
                2 // spec: S-0215 claim 32 (not facing or rearming: 2 ticks)
            }
            Answer::Illegal | Answer::Impossible | Answer::OutOfAmmo | Answer::OutOfRange => {
                if let Some(ob) = w.obj_mut(id) {
                    ob.target = None;
                    ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
                    ob.beh
                        .queue_and_start_at_once(Behaviour::Guard, tick, stale);
                }
                1
            }
            _ => {
                rotate(w);
                1
            }
        };
    }
    // Guard, sticky and area guard (table T1: the same rule).
    if armed {
        if let Some(ob) = w.obj_mut(id) {
            // spec: S-0215 claim 32 (every processing of an armed building in guard sets its ready mark)
            ob.beh.ready_mark = true;
        }
        // spec: S-0215 claim 32 (a pulse cannon, and a building that can be occupied but has no occupants, choose the anti-air delay whatever their target)
        // Occupants are not modelled (garrisons are a seam), so a building that can be occupied has none.
        if o.target.is_some() && !b.building.emp_cannon && !b.building.can_be_occupied {
            // spec: S-0215 claim 32 (a target: queues attack and starts it at once, 1 tick)
            if let Some(ob) = w.obj_mut(id) {
                ob.beh
                    .queue_and_start_at_once(Behaviour::Attack, tick, stale);
            }
            return 1;
        }
        return aa_delay(w, id);
    }
    if b.building.stupid_guard {
        return 100; // spec: S-0215 claim 32 (the stupid-guard-mode flag: 100 ticks)
    }
    // The factory, repair-bay and reload rules of unarmed buildings belong to the structures specs.
    let dr = cur.map_or(450, |c| rules.behaviour(c).rate_delay); // spec: S-0215 claim 4 (no behaviour: 450 ticks)
    if b.building.unit_repair {
        // spec: S-0215 F2 (the standard delay D_r + U(0, 2))
        dr.wrapping_add(w.rng.ranged(0, 2, half))
    } else {
        // spec: S-0417 claims 12 and 13 (idle apron clearing of a vehicle factory, its scatter draws before the delay draw)
        crate::econ::release::idle_apron(w, id, o.beh.expired);
        // spec: S-0215 claim 32 (3 × D_r + U(0, 2))
        dr.wrapping_mul(3).wrapping_add(w.rng.ranged(0, 2, half))
    }
}

/// A SAM building's *attack* (S-0215 claim 32).
fn sam_attack(w: &mut World, id: ObjectId) -> i32 {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let Some(o) = w.obj(id).cloned() else {
        return 1;
    };
    let tick = w.tick;
    let suitable = matches!(o.target, Some(Target::Object(x))
        if w.obj(x).is_some_and(|t| t.kind == Kind::Aircraft && t.pos.z != 0));
    let stage = o.beh.progress;
    // spec: S-0215 claim 32 (the out-of-stage rule: nothing, the anti-air delay)
    if stage != 0 && stage != 1 {
        return aa_delay(w, id);
    }
    if stage == 0 {
        // spec: S-0215 claim 32 (aiming: the type needs power, drains power, and the owner is short of power: nothing, 1 tick)
        if crate::seams::sam_held_by_low_power(w, id) {
            return 1;
        }
        if !suitable {
            if let Some(ob) = w.obj_mut(id) {
                ob.target = None;
                ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
                ob.beh
                    .queue_and_start_at_once(Behaviour::Guard, tick, stale);
            }
            return 1;
        }
        let Some(t) = o.target else { return 1 };
        let b = &w.rules().buildings[o.type_index as usize];
        let omega = b.turn_rate;
        let d = dir(w, building_aim_point(w, id), t);
        let aimed = {
            let ob = w.obj_mut(id).expect("building");
            ob.turret.turn_to(d, tick, omega);
            !ob.turret.turning(tick) && delta(ob.turret.value(tick), d).abs() <= 8192 // spec: S-0215 claim 32 (within 8192 facing units)
        };
        if aimed && let Some(ob) = w.obj_mut(id) {
            ob.beh.progress = 1;
        }
        return 1;
    }
    let Some(t) = o.target.filter(|_| suitable) else {
        if let Some(ob) = w.obj_mut(id) {
            ob.target = None;
            ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
            ob.beh.progress = 0;
        }
        return 1;
    };
    let answer = check(w, id, Some(t), 0, true);
    let next = match answer {
        Answer::CanFire => {
            // spec: S-0215 claim 32 (both weapons fire in this processing; the secondary's own state never holds it back)
            crate::combat::fire::shot(w, id, 0, t);
            crate::combat::fire::shot(w, id, 1, t);
            0
        }
        Answer::NotFacing => 0,
        Answer::Illegal | Answer::Impossible | Answer::OutOfRange => {
            if let Some(ob) = w.obj_mut(id) {
                ob.target = None;
                ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
            }
            0
        }
        _ => 1,
    };
    if let Some(ob) = w.obj_mut(id) {
        ob.beh.progress = next;
    }
    1
}

/// An animation-delayed building's pending shot in its turn (S-0301
/// claim 14).
pub fn delayed_shot_turn(w: &mut World, id: ObjectId) {
    let tick = w.tick;
    let Some(o) = w.obj(id).cloned() else { return };
    let Some(due) = o.combat.delayed_shot else {
        return;
    };
    if tick < due {
        return;
    }
    if let Some(ob) = w.obj_mut(id) {
        ob.combat.delayed_shot = None;
    }
    let Some(t) = o.target else { return };
    let slot = choose_slot(w, id, Some(t));
    // spec: S-0301 claim 14 (the firing check again, with range: a shot on can fire, else dropped)
    if check(w, id, Some(t), slot, true) == Answer::CanFire {
        crate::combat::fire::shot(w, id, slot, t);
    }
}
