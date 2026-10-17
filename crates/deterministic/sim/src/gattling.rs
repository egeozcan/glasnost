//! Gattling spin-up (S-0609): the spin value and gattling stage of an
//! object of a gattling-flagged type, the spin-up and spin-down updates
//! (claims 2 to 4, F1 and F2), when vehicles and buildings take them
//! (claims 5 to 9), the resets (claim 11), the spin sound requests (claim
//! 13) and the turret animation counter (claim 14).
//!
//! State: [`crate::combat::state::CombatState`] holds the spin value
//! (`spin`), the stage (`gattling_stage`, which S-0308 claim 7 reads to
//! choose a gattling type's weapon) and the animation-frame counter
//! (`anim_frame`); [`crate::behaviour::Behaviours`] holds the elapsed-tick
//! count (`elapsed`) that a building's updates use as their multiplier. All
//! of it is saved and hashed.
//!
//! The spin sound's playing mark and its draw on the unsynchronised stream
//! never reach game state (claim 13, S-0202 claim 11): the simulation
//! reports each update as an [`Effect::Spin`] and the audio crate keeps the
//! mark, takes the draw and plays the sound. Passive spin-down (claim 9)
//! and the numeric reset (claim 11) report nothing; leaving the map reports
//! [`SpinEvent::Left`].
//!
//! Seams:
//! - S-0605 (absorbing buildings, not built): an object entering a building
//!   that absorbs it resets its spin (claim 11). [`reset`] is the call the
//!   absorption path will make; nothing absorbs yet.

use glasnost_rules::sim::CombatFields;
use serde::{Deserialize, Serialize};

use crate::combat::check::Answer;
use crate::combat::data::WeaponId;
use crate::combat::effects::Effect;
use crate::combat::veterancy::{self, Rank, type_combat};
use crate::rules::{Behaviour, Kind};
use crate::world::{ObjectId, World};

/// The largest stage count our engine reads (Edge cases: a larger count is
/// used as 6).
pub const MAX_STAGES: usize = 6; // spec: S-0609 Constants (largest stage count supported: 6 stages)

/// The margin added to the guard-area targeting delay before the passive
/// spin-down of a building starts (claim 9).
pub const PASSIVE_MARGIN: i32 = 5; // spec: S-0609 Constants (passive spin-down margin 5 ticks)

/// A vehicle's spin multiplier (claim 5: one update per firing step).
const VEHICLE_MULTIPLIER: i32 = 1; // spec: S-0609 Constants (vehicle spin multiplier 1 tick)

/// The passive spin-down's multiplier (claim 9).
const PASSIVE_MULTIPLIER: i32 = 1; // spec: S-0609 Constants (passive spin-down multiplier 1 tick)

/// A gattling-flagged type's spin settings (Terms, Import mapping).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SpinType {
    /// The stage count *N*, 0 … [`MAX_STAGES`].
    pub stages: i32,
    /// The spin-up amount *u*.
    pub up: i32,
    /// The spin-down amount *d*.
    pub down: i32,
    /// The normal thresholds *K*₁ … *K*₆ (0 past *N*).
    pub normal: [i32; MAX_STAGES],
    /// The elite thresholds *E*₁ … *E*₆ (0 past *N*).
    pub elite: [i32; MAX_STAGES],
}

impl SpinType {
    /// The settings the imported fields give.
    pub fn from_fields(c: &CombatFields) -> Self {
        // spec: S-0609 Import mapping (WeaponStages absent: 0) and Edge cases (a count above 6 is used as 6; engine rule: a negative stage count is used as 0)
        let stages = c.weapon_stages.unwrap_or(0).clamp(0, MAX_STAGES as i32);
        // spec: S-0609 Import mapping (RateUp and RateDown absent: 0) and Edge cases (engine rule: a negative spin-up or spin-down amount is used as 0)
        let up = c.rate_up.unwrap_or(0).max(0);
        let down = c.rate_down.unwrap_or(0).max(0);
        let row = |v: &Option<Vec<i32>>| {
            let mut out = [0; MAX_STAGES];
            // spec: S-0609 Edge cases (stage count 0 or 1: the thresholds stay 0; revision 5, answering Q-1151: the final stage count governs, so a type a later layer lowered below 2 stages uses none of its held thresholds)
            if stages >= 2 {
                for (i, t) in v.iter().flatten().take(stages as usize).enumerate() {
                    // spec: S-0609 Import mapping (absent thresholds: 0, the elite ones not the normal ones); Edge cases (engine rule: a negative threshold is used as 0)
                    out[i] = (*t).max(0);
                }
            }
            out
        };
        Self {
            stages,
            up,
            down,
            normal: row(&c.stage_thresholds),
            elite: row(&c.elite_stage_thresholds),
        }
    }
}

/// What a spin update asks of the spin sound (claim 13), for the audio
/// crate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpinEvent {
    /// A spin-up update. `raised`: it raised the stage. `weapon`: the
    /// weapon in slot 2*s*′ after elite replacement (S-0308 claim 2), the
    /// ground slot whatever the target, `None` for an empty slot (Edge
    /// cases: an empty report list).
    Up {
        /// The update raised the stage.
        raised: bool,
        /// The weapon whose report list the sound comes from.
        weapon: Option<WeaponId>,
    },
    /// A spin-down update of claims 3, 7, 8 or S-0514 claim 4: the sound
    /// ends after its present loop and the mark is cleared.
    Down,
    /// The object left the map: the sound ends looping and the mark is
    /// cleared.
    Left,
}

/// The spin settings of an object of a gattling-flagged type; `None` for
/// any other object.
pub fn spin_type(w: &World, id: ObjectId) -> Option<&SpinType> {
    type_combat(w, id).filter(|tc| tc.gattling).map(|tc| &tc.spin)
}

/// Whether an object's type has the gattling flag (Terms).
pub fn flagged(w: &World, id: ObjectId) -> bool {
    spin_type(w, id).is_some()
}

/// The thresholds in force: the elite row while the object is of elite
/// rank, the normal row otherwise, read at this moment (Terms, claim 4).
fn in_force(w: &World, id: ObjectId, st: &SpinType) -> [i32; MAX_STAGES] {
    // spec: S-0609 Terms and claim 4 (the elite thresholds while elite, S-0311; each update reads the rank at that moment)
    if veterancy::rank(w, id) == Rank::Elite {
        st.elite
    } else {
        st.normal
    }
}

/// The spin cap *T*_N (Terms); 0 for a stage count of 0.
fn cap(st: &SpinType, thr: &[i32; MAX_STAGES]) -> i32 {
    match st.stages {
        // spec: S-0609 Edge cases (stage count 0: the value never grows)
        0 => 0,
        // spec: S-0609 Terms (T_N is the spin cap)
        n => thr[(n - 1) as usize],
    }
}

/// F1 on a value and stage: the new value and stage, and whether the stage
/// rose.
pub fn f1(st: &SpinType, thr: &[i32; MAX_STAGES], v: i32, s: i32, m: i32) -> (i32, i32, bool) {
    // spec: S-0609 claim 2 and F1 (v′ = v + u × m when v < T_N, else v; no clamp to the cap) and Edge cases (engine rule: the product and the sum saturate at 2³¹ − 1, nothing wraps)
    let v2 = if v < cap(st, thr) {
        v.saturating_add(st.up.saturating_mul(m))
    } else {
        v
    };
    // spec: S-0609 claim 2 and F1 (s′ = s + 1 when s < N − 1 and v ≥ T_{s+1}, both read before the update; at most one stage)
    let rise = s >= 0 && s < st.stages - 1 && v >= thr[s as usize];
    (v2, if rise { s + 1 } else { s }, rise)
}

/// F2 on a value and stage: the new value and stage. `empties` is
/// `quirks.gattling_zero_spin_down_empties`.
pub fn f2(st: &SpinType, thr: &[i32; MAX_STAGES], v: i32, s: i32, m: i32, empties: bool) -> (i32, i32) {
    // spec: S-0609 claim 3 and F2 (a = d × m) and Edge cases (engine rule: the product saturates at 2³¹ − 1)
    let a = st.down.saturating_mul(m);
    let v2 = if a != 0 {
        // spec: S-0609 F2 (v′ = max(v − a, 0))
        v.saturating_sub(a).max(0)
    } else if empties {
        // spec: S-0609 claim 3, F2 and Edge cases (quirks.gattling_zero_spin_down_empties on: an amount of 0 sets the value to 0)
        0
    } else {
        // spec: S-0609 F2 and Edge cases (quirks.gattling_zero_spin_down_empties off: an amount of 0 leaves the value unchanged)
        v
    };
    // spec: S-0609 claim 3 and F2 (s′ = s − 1 when s > 0 and v′ < T_s, the threshold that raised it; at most one stage)
    let fall = s > 0 && (s as usize) <= MAX_STAGES && v2 < thr[(s - 1) as usize];
    (v2, if fall { s - 1 } else { s })
}

/// A spin-up update with multiplier `m` (claim 2), with its sound request
/// (claim 13).
pub fn spin_up(w: &mut World, id: ObjectId, m: i32) {
    let Some(st) = spin_type(w, id).cloned() else {
        return;
    };
    let thr = in_force(w, id, &st);
    let Some(o) = w.obj(id) else { return };
    let (v, s, raised) = f1(&st, &thr, o.combat.spin, o.combat.gattling_stage, m);
    if let Some(ob) = w.obj_mut(id) {
        ob.combat.spin = v;
        ob.combat.gattling_stage = s;
    }
    // spec: S-0609 claim 13 (the sound from the report list of the weapon in slot 2s, s the stage after the update, after elite replacement; the ground slot even against an airborne target)
    let slot = usize::try_from(s.saturating_mul(2)).unwrap_or(0);
    let weapon = crate::combat::weapon::slot_weapon(w, id, slot);
    let pos = w.obj(id).map(|o| o.pos);
    w.effects.push(Effect::Spin {
        id,
        event: SpinEvent::Up { raised, weapon },
        pos,
    });
}

/// A spin-down update with multiplier `m` (claim 3). `sound`: the update
/// is one of claims 3, 7 and 8 or S-0514 claim 4, which ends the sound's
/// loop and clears the mark (claim 13); the passive spin-down of claim 9
/// does not.
pub fn spin_down(w: &mut World, id: ObjectId, m: i32, sound: bool) {
    let Some(st) = spin_type(w, id).cloned() else {
        return;
    };
    let thr = in_force(w, id, &st);
    let empties = w.rules().quirks.gattling_zero_spin_down_empties;
    let Some(o) = w.obj(id) else { return };
    let (v, s) = f2(&st, &thr, o.combat.spin, o.combat.gattling_stage, m, empties);
    let pos = o.pos;
    if let Some(ob) = w.obj_mut(id) {
        ob.combat.spin = v;
        ob.combat.gattling_stage = s;
    }
    if sound {
        // spec: S-0609 claim 13 (every spin-down update clears the mark and lets the sound end after its present loop)
        w.effects.push(Effect::Spin {
            id,
            event: SpinEvent::Down,
            pos: Some(pos),
        });
    }
}

/// The spin update of a vehicle's firing step (claim 5), after the step's
/// own reply. `answer` is the firing check's answer, `None` when the
/// vehicle has no target or its slot 0 is empty; `unloads` marks the
/// deploy-to-fire early exit, which makes no update.
pub fn vehicle_step(w: &mut World, id: ObjectId, answer: Option<Answer>, unloads: bool) {
    if !flagged(w, id) || w.obj(id).is_none_or(|o| o.kind != Kind::Vehicle) {
        return;
    }
    if unloads {
        // spec: S-0609 claim 5 (a deploy-to-fire vehicle that queues unload instead: no update in that tick) and claim 14 (no advance)
        return;
    }
    match answer {
        // spec: S-0609 claim 5 (can fire, rearming, not facing or still turning: spin-up with multiplier 1)
        Some(Answer::CanFire | Answer::Rearming | Answer::NotFacing | Answer::StillTurning) => {
            spin_up(w, id, VEHICLE_MULTIPLIER);
        }
        // spec: S-0609 claim 5 (any other answer, no target or an empty slot 0: spin-down with multiplier 1)
        Some(
            Answer::OutOfAmmo
            | Answer::Illegal
            | Answer::Impossible
            | Answer::Moving
            | Answer::OutOfRange
            | Answer::Cloaked
            | Answer::Busy
            | Answer::MustDeploy,
        )
        | None => spin_down(w, id, VEHICLE_MULTIPLIER, true),
    }
    // spec: S-0609 claim 14 (a vehicle's animation-frame counter advances by one in each completed firing step whose post-update value is at least 1)
    if let Some(ob) = w.obj_mut(id)
        && ob.combat.spin >= 1
    {
        ob.combat.anim_frame = ob.combat.anim_frame.wrapping_add(1);
    }
}

/// What a building's *attack* processing does to its spin (claim 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttackRow {
    /// A spin-up with the elapsed-tick count.
    Up,
    /// A spin-down with the elapsed-tick count.
    Down,
    /// No update; the elapsed ticks are dropped.
    Pause,
}

/// The row of claim 7's table for a firing check's final answer.
pub fn attack_row(answer: Answer) -> AttackRow {
    match answer {
        // spec: S-0609 claim 7 (can fire, whether or not a shot is launched, not facing, rearming: spin-up with m = c)
        Answer::CanFire | Answer::NotFacing | Answer::Rearming => AttackRow::Up,
        // spec: S-0609 claim 7 (out of ammunition, illegal, impossible, out of range: spin-down with m = c, the target dropped by S-0215 claim 32; cloaked, busy: spin-down with m = c)
        Answer::OutOfAmmo
        | Answer::Illegal
        | Answer::Impossible
        | Answer::OutOfRange
        | Answer::Cloaked
        | Answer::Busy => AttackRow::Down,
        // spec: S-0609 claim 7 (still turning, moving, must deploy: no update, c returns to 0)
        Answer::StillTurning | Answer::Moving | Answer::MustDeploy => AttackRow::Pause,
    }
}

/// A building's spin in an *attack* processing with a target (claim 7),
/// after the processing's shot: `c` is the elapsed-tick count read at the
/// start of the processing. Each row sets the count back to 0.
pub fn building_attack(w: &mut World, id: ObjectId, answer: Answer, c: i32) {
    if !flagged(w, id) {
        return;
    }
    match attack_row(answer) {
        AttackRow::Up => spin_up(w, id, c),
        AttackRow::Down => spin_down(w, id, c, true),
        AttackRow::Pause => {}
    }
    // spec: S-0609 claim 7 and Terms (each update, and each paused row, sets c back to 0)
    if let Some(ob) = w.obj_mut(id) {
        ob.beh.elapsed = 0;
    }
}

/// The spin-down that starts every *guard*, *sticky* and *area guard*
/// processing of a building (claim 8), with multiplier *c*, before anything
/// else that processing does.
pub fn building_guard(w: &mut World, id: ObjectId) {
    if !flagged(w, id) {
        return;
    }
    let c = w.obj(id).map_or(0, |o| o.beh.elapsed);
    // spec: S-0609 claim 8 (a spin-down with multiplier c, armed or unarmed, then c returns to 0)
    spin_down(w, id, c, true);
    if let Some(ob) = w.obj_mut(id) {
        ob.beh.elapsed = 0;
    }
}

/// The elapsed-tick count's growth (Terms): by 1 every tick just before
/// the object's behaviour may be processed.
pub fn elapse(w: &mut World, id: ObjectId) {
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0609 Terms (the elapsed-tick count grows by 1 every tick just before the behaviour may be processed) and Edge cases (engine rule: the count saturates at 2³¹ − 1 and stays there until its next reset)
        ob.beh.elapsed = ob.beh.elapsed.saturating_add(1);
    }
}

/// A building's part of its turn after its behaviour processing: the
/// animation advance of claim 14, then the passive spin-down of claim 9.
pub fn building_turn(w: &mut World, id: ObjectId) {
    if !flagged(w, id) {
        return;
    }
    let Some(o) = w.obj(id) else { return };
    if o.kind != Kind::Building {
        return;
    }
    let attacking = o.beh.current == Some(Behaviour::Attack);
    let since = w.tick.wrapping_sub(o.last_fired);
    let g = w.rules().general.guard_area_targeting_delay;
    if let Some(ob) = w.obj_mut(id)
        && ob.combat.spin > 0
    {
        // spec: S-0609 claim 14 (a building's counter advances once per tick when its value after the tick's behaviour processing is positive; the original's second advance after passive decay is corrected without a switch)
        ob.combat.anim_frame = ob.combat.anim_frame.wrapping_add(1);
    }
    // spec: S-0609 claim 9 (not in attack, and more than G + 5 ticks since the last shot, S-0206 claim 9: F2 with m = 1 after the tick's processing, no sound change)
    if !attacking && since > g.saturating_add(PASSIVE_MARGIN) {
        spin_down(w, id, PASSIVE_MULTIPLIER, false);
    }
}

/// The reset of claim 11: spin value, stage and elapsed-tick count become
/// 0 (boarding a transport, S-0507 claim 6; entering an absorbing building,
/// S-0605, a seam). No sound effect: leaving the map is [`leaving_map`].
pub fn reset(w: &mut World, id: ObjectId) {
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0609 claim 11 (boarding or absorption: the spin value, the stage and the elapsed-tick count become 0)
        ob.combat.spin = 0;
        ob.combat.gattling_stage = 0;
        ob.beh.elapsed = 0;
    }
}

/// A successful temporal lock onto an object (S-0514 claim 4): exactly one
/// spin-down update with multiplier 1 (claim 11), with its sound effect.
pub fn temporal_lock(w: &mut World, id: ObjectId) {
    // spec: S-0609 claim 11 and S-0514 claim 4 (each successful lock, a joining one included: one spin-down update with multiplier 1, not a reset)
    spin_down(w, id, 1, true);
}

/// An object of a gattling-flagged type leaves the map: its spin sound ends
/// looping and the mark is cleared (claims 11 and 13).
pub fn leaving_map(w: &mut World, id: ObjectId) {
    if !flagged(w, id) {
        return;
    }
    let pos = w.obj(id).map(|o| o.pos);
    // spec: S-0609 claim 13 (leaving the map tells the spin sound to end looping and clears the mark)
    w.effects.push(Effect::Spin {
        id,
        event: SpinEvent::Left,
        pos,
    });
}

/// The stage, spin value and elapsed-tick count of an object, for tests,
/// the AI and the lane.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spin {
    /// The spin value.
    pub value: i32,
    /// The gattling stage.
    pub stage: i32,
    /// The elapsed-tick count.
    pub elapsed: i32,
}

/// An object's spin state.
pub fn spin(w: &World, id: ObjectId) -> Spin {
    w.obj(id).map_or(Spin::default(), |o| Spin {
        value: o.combat.spin,
        stage: o.combat.gattling_stage,
        elapsed: o.beh.elapsed,
    })
}

/// Whether an object of a gattling-flagged type with at least two stages
/// is at its top stage.
pub fn at_top(w: &World, id: ObjectId) -> bool {
    spin_type(w, id).is_some_and(|st| {
        // spec: S-0609 Edge cases (a stage count of 0 or 1 has no stage above 0) and claim 2 (stages 0 … N − 1)
        st.stages >= 2 && w.obj(id).is_some_and(|o| o.combat.gattling_stage >= st.stages - 1)
    })
}
