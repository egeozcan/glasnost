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
    let rules = w.rules_arc();
    let Some(u) = rules.unit(o.kind, o.type_index) else {
        return false;
    };
    // spec: S-0309 claim 27 (DECIDED: a restored object off the map or destroyed in play is no target in the firing step)
    let Some(t) = crate::combat::held::firing_target(w, &o) else {
        // spec: S-0609 claim 5 (a firing step with no target: spin-down)
        crate::gattling::vehicle_step(w, id, None, false);
        return false;
    };
    // spec: S-0310 claim 9 (the firing check, range test included, for the weapon it would use)
    let slot = choose_slot(w, id, Some(t));
    let answer = check(w, id, Some(t), slot, true);
    let (facing_error, unloads) = vehicle_reply(w, id, &o, u, t, slot, answer);
    // spec: S-0609 claim 5 (one update after the step's own reply, so a shot of this step uses the stage before it; an empty slot 0 spins down)
    let slot0 = slot_weapon(w, id, 0).is_some();
    crate::gattling::vehicle_step(w, id, slot0.then_some(answer), unloads);
    facing_error
}

/// The reply of a vehicle's firing step to the firing check's answer
/// (S-0310 claim 9): whether the answer was a facing error, and whether a
/// deploy-to-fire vehicle queued *unload* instead (S-0609 claim 5's early
/// exit).
fn vehicle_reply(
    w: &mut World,
    id: ObjectId,
    o: &crate::world::Object,
    u: &crate::rules::UnitRules,
    t: Target,
    slot: usize,
    answer: Answer,
) -> (bool, bool) {
    let rules = w.rules_arc();
    // spec: S-0506 claim 11 (an impossible answer to an attempt with the spawner weapon resets the controller)
    crate::spawn::firing_answer(w, id, slot, answer);
    // spec: S-0516 claim 11 (a) (cloaked: a vehicle within range starts uncloaking)
    crate::cloak::machine::firing_answer(w, id, slot, t, answer);
    match answer {
        Answer::CanFire => {
            if crate::econ::inplace::queues_unload(w, id) {
                // spec: S-0310 claim 9 and S-0517 claim 13 (a deploy-to-fire type queues *unload* instead of firing, without starting it immediately; a deployed one with quirks.deploy_to_fire_repacks off fires normally)
                if let Some(ob) = w.obj_mut(id) {
                    ob.beh.queue(Some(Behaviour::Unload));
                }
                (false, true)
            } else {
                // spec: S-0310 claim 9 (any other vehicle fires; a visceroid's animation stage is presentation)
                crate::combat::fire::shot(w, id, slot, t);
                (false, false)
            }
        }
        Answer::NotFacing => {
            if crate::econ::inplace::queues_unload(w, id) {
                // spec: S-0310 claim 9 and S-0517 claim 13 (not facing: it queues *unload* without starting it immediately, and none of the three facing replies applies)
                if let Some(ob) = w.obj_mut(id) {
                    ob.beh.queue(Some(Behaviour::Unload));
                }
                return (false, true);
            }
            if u.turret {
                // spec: S-0310 claim 9 (with the turret flag: the turret's target facing becomes the direction of the target)
                (true, false)
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
                (false, false)
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
                    crate::spawn::clear_own_target(ob);
                    ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
                }
            }
            (false, false)
        }
        _ => (false, false),
    }
}

/// The turret target rows of S-0206 claim 9 for a vehicle with a target,
/// after its firing step; `facing_error` is the step's answer. Returns the
/// target facing of the row that wins, if any.
pub fn turret_target(w: &World, id: ObjectId, facing_error: bool) -> Option<u16> {
    turret_target_rows(w, id, facing_error).last().copied()
}

/// The target facings of the turret target rows that apply in this tick
/// (S-0206 claim 9), in F5's numbering: from the lowest row of the table
/// to the highest, so the last one is the target at the end of the tick
/// unless the spin row applies too.
pub fn turret_target_rows(w: &World, id: ObjectId, facing_error: bool) -> Vec<u16> {
    let mut rows = Vec::new();
    let Some(o) = w.obj(id) else { return rows };
    let Some(t) = o.target else { return rows };
    let r = w.rules();
    let Some(u) = r.unit(o.kind, o.type_index) else {
        return rows;
    };
    let tick = w.tick;
    // spec: S-0206 Terms (the turret weapon: the numbered weapon at the weapon number, else the primary)
    let tw = if u.combat.multi_weapon {
        slot_weapon(w, id, crate::combat::weapon::weapon_number(w, id))
    } else {
        slot_weapon(w, id, 0)
    };
    // spec: S-0206 claim 9 (a spinning turret never counts as turning for the target rows)
    let was_turning = !u.turret_spins && o.turret.turning(tick.wrapping_sub(1));
    let d = dir(w, w.reference_point(id), t);
    let omni = tw.is_some_and(|x| r.combat.weapon(x).omni_fire);
    // spec: S-0206 claim 9 and F5 (the facing-error row is the lowest of the rows that can apply together)
    if facing_error {
        rows.push(d);
    }
    // spec: S-0206 claim 9 (a target, not turning at the end of the previous tick, a turret weapon that is not omni-fire: the direction of the target; turret-locked weapons are not imported)
    if !was_turning && tw.is_some() && !omni {
        rows.push(d);
    } else if !facing_error && !r.quirks.turret_reaim_after_turn && tw.is_some() && !omni {
        // spec: S-0206 claim 9 (otherwise no change, quirks.turret_reaim_after_turn off: the direction of the target)
        rows.push(d);
    }
    rows
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
    // The stage-delay-0 ending runs before the late start (`tick`,
    // `infantry::stage_delay_zero_end`), and a change of target by an order,
    // the target's leaving or the unit's own turn at once
    // (`infantry::target_changed`), so the firing turn sees them ended.
    let end_attack = |w: &mut World, ask: bool| crate::infantry::end_attack(w, id, ask);
    // spec: S-0309 claim 27 (DECIDED: a restored object off the map or destroyed in play is no target in the firing step)
    let Some(t) = crate::combat::held::firing_target(w, &o) else {
        if s.attack {
            // spec: S-0107 claim 11 (the attack state ends when the unit has no target) and S-0215 r118 claim 11 (Other endings: no target at the firing turn, in the firing turn)
            end_attack(w, false);
        }
        return;
    };
    if s.attack && o.combat.last_shot_target != Some(t) {
        // spec: S-0107 claim 11 (the attack state ends when its target changes) and Q-2616 interim (a change by a route that S-0215 claim 11 does not place, such as a retaliation in another actor's turn, ends the state here, with no request)
        end_attack(w, false);
    }
    let attack = w
        .obj(id)
        .and_then(|o| o.infantry.as_ref())
        .is_some_and(|s| s.attack);
    let slot = choose_slot(w, id, Some(t));
    if !attack {
        // spec: S-0107 claim 11 (with a target, not in the attack state and the weapon ready: enter it)
        let answer = check(w, id, Some(t), slot, true);
        // spec: S-0516 claim 11 (a) (cloaked: the infantry unit starts uncloaking)
        crate::cloak::machine::firing_answer(w, id, slot, t, answer);
        if answer == Answer::Illegal {
            // spec: S-0310 r8 claim 11, from S-0309 r70's Changes for S-0310 claim 11 (an infantry unit whose default weapon heals replies to illegal in its own firing step, in each of its turns: it drops a target that is not infantry or is at full health, and keeps an infantry target below full health and does nothing in that step)
            let default_heals = slot_weapon(w, id, choose_slot(w, id, None))
                .is_some_and(|x| rules.combat.weapon(x).damage < 0);
            if default_heals {
                let keep = matches!(t, Target::Object(x)
                    if w.obj(x).is_some_and(|ob| ob.kind == Kind::Infantry)
                        && crate::combat::damage::strength(w, x) > w.obj(x).map_or(0, |ob| ob.health));
                if !keep && let Some(ob) = w.obj_mut(id) {
                    crate::spawn::clear_own_target(ob);
                    ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
                }
            }
        }
        if answer != Answer::CanFire {
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
            // spec: S-0107 r40 claim 11 (End of the attack state: when the weapon fires at the fire frame F, the attack state ends with that shot) and S-0215 r118 claim 11 (Ends at the shot, in the same firing turn; Q-2566 answered)
            end_attack(w, false);
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
    // spec: S-0609 claim 7 (the elapsed-tick count c of this processing, read before any behaviour start resets it)
    let c = o.beh.elapsed;
    if cur == Some(Behaviour::Attack) {
        if b.building.sam {
            return sam_attack(w, id);
        }
        // spec: S-0309 claim 27 (DECIDED: a restored object off the map or destroyed in play is no target in the firing step); S-0513 r91 claim 39a (Q-2376 answered: a departed warp member that the building's temporal beam holds is a target here, so the building takes the rearming row, no shot, the 2-tick delay and a rotation towards the old position, and no guard is queued for the departure)
        let Some(t) = crate::combat::held::firing_target(w, &o) else {
            // spec: S-0518 claim 10 (a prism master's attack processed with no target resets its support count)
            crate::linked::reset_count(w, id);
            // spec: S-0213 claim 16d (revision 84, Q-2192: a building without the SAM flag issues its next route point only when its attack has no target, first, with claim 16c's queue of wait with an immediate start; when wait is then current it queues no guard) and Changes for other specs (S-0215 claim 32)
            if crate::routes::issue_later(w, id)
                && w.obj(id)
                    .is_some_and(|x| x.beh.current == Some(Behaviour::Wait))
            {
                return 1; // spec: S-0215 claim 32 (the no-target case's delay: 1 tick)
            }
            // spec: S-0215 claim 32 (no target: queues guard and starts it at once, 1 tick; also when the building has no route point to issue or wait is not current, S-0213 claim 16d)
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
        // spec: S-0506 claim 11 (an impossible answer to an attempt with the spawner weapon resets the controller)
        crate::spawn::firing_answer(w, id, slot, answer);
        // spec: S-0516 claim 11 (a) (cloaked: the building starts uncloaking)
        crate::cloak::machine::firing_answer(w, id, slot, t, answer);
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
        let delay = match answer {
            Answer::CanFire => {
                let gate = upgrade_gate(w, id);
                if gate == Some(false) && rules.quirks.upgrade_attack_weapon_gate {
                    // spec: S-0215 claim 32 and S-0301 claim 14 (a failed upgrade weapon gate: no firing attempt and no delayed shot, with the original quirk setting; target, attack and ready mark kept, 1 tick)
                } else if gate.is_none() && crate::linked::master_path(w, id) {
                    // spec: S-0518 claims 1 to 7 (a prism master recruits one supporter per processing, and when it finds none, or the count has reached the cap, starts its own delayed shot, slot 0)
                    if !crate::linked::recruit(w, id) {
                        crate::linked::start_shot(w, id);
                    }
                } else if gate == Some(true) {
                    // spec: S-0215 claim 32 and S-0301 claims 9 and 14 (a passing gate: a primary attempt, then a secondary attempt with no firing check of its own, whatever rearm the primary began; immediately, also for an animation-delayed building, and starting no delayed shot)
                    // spec: S-0301 claim 9 (a secondary slot that resolves to no weapon: the attempt does nothing, `fire::shot` returns before any change or draw)
                    crate::combat::fire::shot(w, id, 0, t);
                    crate::combat::fire::shot(w, id, 1, t);
                } else if b.building.anim_delayed_fire {
                    // spec: S-0301 claim 14 (the ordinary path, the gate not applying or quirks.upgrade_attack_weapon_gate off with a failed gate: an animation-delayed building starts its delayed shot instead)
                    let d = b.building.delayed_fire_delay;
                    if let Some(ob) = w.obj_mut(id) {
                        ob.combat.delayed_shot = Some(tick.wrapping_add((d - 1).max(0)));
                        // spec: S-0301 claim 14 and S-0308 claim 14 (the weapon slot chosen when the delay starts is kept for the check and the shot that end it)
                        ob.combat.delayed_slot = slot as u8;
                    }
                } else {
                    // spec: S-0215 claim 32 (no applicable gate, or quirks.upgrade_attack_weapon_gate off with a failed gate: the ordinary selected-weapon attempt)
                    crate::combat::fire::shot(w, id, slot, t);
                }
                1
            }
            Answer::NotFacing | Answer::Rearming => {
                rotate(w);
                // spec: S-0215 claim 32 (revision 76: while the check answers rearming, for any cause, the rearm countdown or a pending animation-delayed shot included, the building keeps its target, also out of range)
                2 // spec: S-0215 claim 32 (not facing or rearming: 2 ticks)
            }
            // spec: S-0215 claim 32 (revision 76: a target out of range is dropped at the first processing whose firing check answers out of range)
            Answer::Illegal | Answer::Impossible | Answer::OutOfAmmo | Answer::OutOfRange => {
                // spec: S-0518 claim 10 (a prism master whose attack drops the target resets its support count)
                crate::linked::reset_count(w, id);
                if let Some(ob) = w.obj_mut(id) {
                    crate::spawn::clear_own_target(ob);
                    ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
                    ob.beh
                        .queue_and_start_at_once(Behaviour::Guard, tick, stale);
                }
                1
            }
            Answer::StillTurning
            | Answer::Moving
            | Answer::MustDeploy
            | Answer::Cloaked
            | Answer::Busy => {
                rotate(w);
                1
            }
        };
        // spec: S-0609 claim 7 (the spin update after the processing's shot, by the final answer, with multiplier c; a SAM building makes none)
        crate::gattling::building_attack(w, id, answer, c);
        return delay;
    }
    // spec: S-0609 claim 8 (every guard, sticky and area guard processing of a gattling building starts with a spin-down by c, armed or unarmed)
    if matches!(
        cur,
        Some(Behaviour::Guard | Behaviour::Sticky | Behaviour::AreaGuard)
    ) {
        crate::gattling::building_guard(w, id);
    }
    // Guard, sticky and area guard (table T1: the same rule).
    if armed {
        if let Some(ob) = w.obj_mut(id) {
            // spec: S-0215 claim 32 (every processing of an armed building in guard sets its ready mark)
            ob.beh.ready_mark = true;
        }
        // spec: S-0215 claim 32 (a pulse cannon, and a building that can be occupied but has no occupants, choose the anti-air delay whatever their target)
        let empty_garrison =
            b.building.can_be_occupied && crate::structures::garrison::count(w, id) == 0;
        if o.target.is_some() && !b.building.emp_cannon && !empty_garrison {
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
    if !b.building.unit_repair
        && let Some(d) = crate::structures::bunker::early_start(w, id)
    {
        // spec: S-0606 claim 5 and S-0215 claim 29 (the earlier guard transition also applies to a bunker: 1 tick, no delay draw)
        return d;
    }
    if b.building.unit_repair {
        if let Some(d) = crate::depot::early_start(w, id) {
            // spec: S-0215 claim 29 and S-0314 claim 17 (the earlier guard transition: 1 tick, no delay draw)
            return d;
        }
        // spec: S-0215 F2 (the standard delay D_r + U(0, 2))
        dr.wrapping_add(w.rng.ranged(0, 2, half))
    } else {
        // spec: S-0417 claims 12 and 13 (idle apron clearing of a vehicle factory, its scatter draws before the delay draw)
        crate::econ::release::idle_apron(w, id, o.beh.expired);
        // spec: S-0215 claim 32 (3 × D_r + U(0, 2))
        dr.wrapping_mul(3).wrapping_add(w.rng.ranged(0, 2, half))
    }
}

/// The upgrade weapon gate of a non-SAM building (S-0215 claim 32):
/// `None` when the gate does not apply (a zero upgrade count or an empty
/// position 1), else whether the upgrade in position 1 passes with its
/// ordinary weapons.
pub fn upgrade_gate(w: &World, id: ObjectId) -> Option<bool> {
    use crate::combat::weapon::{building_type_slot, gate_upgrade};
    // spec: S-0215 claim 32 and S-0406 claim 12 (position 1 only, even when another position supplies the weapon chosen; never the first occupied higher position nor the latest installation)
    let u = gate_upgrade(w, id)?;
    // spec: S-0215 claim 32 (its ordinary primary and secondary, not their elite replacements)
    let Some(p) = building_type_slot(w, u, 0) else {
        // spec: S-0215 claim 32 (no primary: the gate fails)
        return Some(false);
    };
    let s = building_type_slot(w, u, 1);
    let burst = |x| w.rules().combat.weapon(x).burst;
    // spec: S-0215 claim 32 (both slots the same weapon type, the primary's burst count above 1, or a secondary whose burst count is above 1; integer comparisons)
    Some(s == Some(p) || burst(p) > 1 || s.is_some_and(|s| burst(s) > 1))
}

/// A SAM building's *attack* (S-0215 claim 32).
fn sam_attack(w: &mut World, id: ObjectId) -> i32 {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let Some(o) = w.obj(id).cloned() else {
        return 1;
    };
    let tick = w.tick;
    // spec: S-0513 r91 claim 39a (Q-2376 answered: a SAM-flag building whose beam holds a warp member that waits to land takes these rows as for a target on the map at the member's old position, at the world height it had there, which the departure keeps; so it keeps an aircraft member above height 0, its rearming one of the aimed stage's other answers, keeps any member while the power row applies, and otherwise drops the member and starts guard)
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
                crate::spawn::clear_own_target(ob);
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
            crate::spawn::clear_own_target(ob);
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
                crate::spawn::clear_own_target(ob);
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
    if crate::linked::master_path(w, id) {
        // spec: S-0518 claims 7 to 10 (a prism master's shot: slot 0, the support multiplier, the count)
        crate::linked::master_shot(w, id);
        return;
    }
    // spec: S-0301 claim 14 (the check and the shot use the building's target at the end of the delay; Q-1770 answered) and S-0309 claim 27 (with no target then, no check and no shot, and the shot is no longer pending; DECIDED: a restored object off the map or destroyed in play is no target)
    let Some(t) = crate::combat::held::firing_target(w, &o) else {
        return;
    };
    // spec: S-0301 claim 14 and S-0308 claim 14 (the slot chosen when the delay started, not chosen again)
    let slot = usize::from(o.combat.delayed_slot);
    // spec: S-0301 claim 14 (the firing check again, with range, not refused by this shot itself, which is no longer pending: a shot on can fire, else dropped with no projectile, countdown or draw)
    if check(w, id, Some(t), slot, true) == Answer::CanFire {
        crate::combat::fire::shot(w, id, slot, t);
    }
}
