//! The game-wide lighting state of S-0104 that the simulation keeps: the
//! base, current and target ambient, the ambient fade (claim 4), the
//! lighting re-evaluations and their global choice (claims 2 and 5), and
//! the nuke flash phases that time them.
//!
//! What this holds is presentation input (the renderer reads the current
//! ambient and the chosen set's tint), but one part of it is game state:
//! a psychic domination is over only when the current ambient reaches the
//! target (S-0612 claims 5 and 7), so the fade must run in the tick
//! pipeline. It runs once per tick in S-0203 row 3, just before the
//! domination update ([`crate::domination::update`]); S-0104 lets the fade
//! work run anywhere in the tick, and it takes no draws.
//!
//! Seams:
//! - The chrono flash (S-0104 claims 2, 4 and 5) is not built: nothing
//!   starts one, so it never ranks and never picks the nuke rate.
//! - The per-cell terms (height terms, tables built during an effect,
//!   claims 9, 10 and 16) are the renderer's, read from [`LightWorld`] and
//!   [`crate::domination::in_force`].

use glasnost_map::{Lighting, LightingSet};
use glasnost_num::F64;
use serde::{Deserialize, Serialize};

use crate::world::World;

/// The length of the nuke flash's brightening phase, ticks 0 to 30.
pub const NUKE_BRIGHTENING: i32 = 31; // spec: S-0104 Constants (nuke flash phase lengths 31, then 16 ticks) and claim 5
/// The length of the nuke flash's fading phase, ticks 31 to 46.
pub const NUKE_FADING: i32 = 16; // spec: S-0104 Constants (nuke flash phase lengths 31, then 16 ticks) and claim 5
/// Ticks per game minute of the ambient change rate.
pub const TICKS_PER_MINUTE: i64 = 900; // spec: S-0104 Constants (fade game minutes to ticks: 900)
/// The ambient change step's scale, hundredths.
pub const STEP_SCALE: i64 = 100; // spec: S-0104 Formulas, Ambient fade (step = ⌊s·100⌋ hundredths)

/// The rules' ambient fade values (S-0104 claim 4, Import mapping).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LightRules {
    /// The normal fade interval ⌊m × 900⌋, ticks.
    pub interval: i32,
    /// The fade step ⌊s × 100⌋, hundredths.
    pub step: i32,
    /// The rules' ambient change rate is 0 (Edge cases, "Zero change rate
    /// freezes ambient").
    pub zero_rate: bool,
}

impl Default for LightRules {
    fn default() -> Self {
        Self {
            interval: 180, // spec: S-0104 Constants (default ambient change rate 0.2 game minutes, 180 ticks)
            step: 10, // spec: S-0104 Constants (default ambient change step 0.10, 10 hundredths)
            zero_rate: false,
        }
    }
}

impl LightRules {
    /// Resolves the values from the rules' `[AudioVisual]` keys.
    pub fn new(r: &glasnost_rules::schema::Ruleset, mode: glasnost_num::Rounding) -> Self {
        let g = &r.sim.general;
        let d = Self::default();
        // spec: S-0104 Formulas, Precision of decimal conversions (the single-precision value widened, multiplied in double precision in the rounding mode in force, truncated)
        let rate = g.ambient_change_rate.as_ref().map(|v| v.single_value(mode));
        let interval = rate.map_or(d.interval, |m| {
            // spec: S-0104 Formulas, Ambient fade (P = ⌊m·900⌋ ticks)
            m.mul(F64::from_i64(TICKS_PER_MINUTE, mode), mode)
                .to_i32_common()
        });
        let step = g.ambient_change_step.as_ref().map_or(d.step, |v| {
            // spec: S-0104 Formulas, Ambient fade (step = ⌊s·100⌋ hundredths)
            v.single_value(mode)
                .mul(F64::from_i64(STEP_SCALE, mode), mode)
                .to_i32_common()
        });
        Self {
            interval,
            step,
            // spec: S-0104 claim 4 and Edge cases (the rules' ambient change rate is 0)
            zero_rate: rate.is_some_and(|m| m.eq_num(F64::ZERO)),
        }
    }
}

/// The lighting set a global choice picked (S-0104 claims 2 and 5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Choice {
    /// No effect: the base ambient and every table's own tint.
    #[default]
    Normal,
    /// The ion storm set.
    Ion,
    /// The psychic dominator set.
    Dominator,
    /// The fixed nuke flash set.
    Nuke,
}

/// The lighting state (S-0104 State and ordering).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LightWorld {
    /// The map's lighting sets and rates, read once with the map (claim
    /// 14).
    pub map: Lighting,
    /// The base ambient, hundredths.
    pub base: i32,
    /// The current ambient, hundredths.
    pub current: i32,
    /// The target ambient, hundredths.
    pub target: i32,
    /// The last global choice: the target and the tints follow it.
    pub choice: Choice,
    /// The tick of the last fade step: the current interval started then.
    pub last_step: i32,
    /// The current fade interval, chosen at the step that started it.
    pub interval: i32,
    /// A launch or a nuclear detonation re-evaluated: the next tick takes
    /// a fade step however much of the interval has passed (claim 5).
    pub forced: bool,
    /// The tick a nuclear missile last detonated (the nuke flash's tick 0).
    pub nuke: Option<i32>,
}

/// The fixed nuke flash set (S-0104 Constants: ambient 2.00, tint 1.75,
/// 1.50, 1.25, ground and level 0.100).
// spec: S-0104 Constants (the nuke flash's fixed set)
pub const NUKE_SET: LightingSet = LightingSet {
    ambient: 200,          // spec: S-0104 Constants (nuke flash ambient 2.00, fixed)
    tint: [175, 150, 125], // spec: S-0104 Constants (nuke flash tint 1.75, 1.50, 1.25, fixed)
    ground: 100, // spec: S-0104 Constants (nuke flash ground 0.100, stored 100 thousandths)
    level: 100, // spec: S-0104 Constants (nuke flash level 0.100 per height level, stored 100 thousandths)
};

/// The state at game start: base, current and target at the map's normal
/// ambient (S-0104 Import mapping).
pub fn start(map: &Lighting) -> LightWorld {
    let a = map.normal.ambient;
    LightWorld {
        map: *map,
        // spec: S-0104 Import mapping (the normal ambient is the base, current and target at start)
        base: a,
        current: a,
        target: a,
        choice: Choice::Normal,
        last_step: 0,
        interval: 0,
        forced: false,
        nuke: None,
    }
}

/// The nuke flash phase at tick `t`: `Some(true)` brightening, `Some(false)`
/// fading or its tick 47, `None` over or none.
fn nuke_phase(l: &LightWorld, t: i32) -> Option<bool> {
    let d = t - l.nuke?;
    if (0..NUKE_BRIGHTENING).contains(&d) {
        // spec: S-0104 claim 5, Nuke flash timing (brightening: ticks 0 to 30)
        Some(true)
    } else if (NUKE_BRIGHTENING..=NUKE_BRIGHTENING + NUKE_FADING).contains(&d) {
        // spec: S-0104 claim 5, Nuke flash timing (fading: ticks 31 to 46; a step on tick 47 still takes the nuke rate)
        Some(false)
    } else {
        None
    }
}

/// Whether an ion storm is in force (S-0104 Terms: a clearing storm counts
/// until it ends).
pub fn storm_in_force(w: &World) -> bool {
    use crate::superweapons::StormState;
    // spec: S-0104 claim 2 (a clearing ion storm still counts in all three rankings until it ends)
    matches!(w.sw.storm.state, StormState::Raging | StormState::Clearing)
}

/// The global choice from the effects in force now (claim 2, first
/// column).
pub fn global_choice(w: &World) -> Choice {
    let l = &w.lighting;
    if nuke_phase(l, w.tick) == Some(true) {
        // spec: S-0104 claim 2 rank 1 (nuke flash brightening phase; the chrono flash is not built)
        Choice::Nuke
    } else if storm_in_force(w) {
        // spec: S-0104 claim 2 rank 2 (ion storm)
        Choice::Ion
    } else if crate::domination::active(w) {
        // spec: S-0104 claim 2 rank 3 (psychic dominator in an active phase other than its final phase)
        Choice::Dominator
    } else {
        // spec: S-0104 claim 2 rank 4 (normal)
        Choice::Normal
    }
}

/// The set of a choice (`None` for normal).
pub fn set_of(l: &LightWorld, c: Choice) -> Option<LightingSet> {
    match c {
        Choice::Normal => None,
        Choice::Ion => Some(l.map.ion),
        Choice::Dominator => Some(l.map.dominator),
        Choice::Nuke => Some(NUKE_SET),
    }
}

/// A lighting re-evaluation (claim 5): the global choice from the effects
/// in force now sets the target ambient and the tints.
pub fn reevaluate(w: &mut World) {
    let c = global_choice(w);
    let l = &mut w.lighting;
    l.choice = c;
    // spec: S-0104 claim 5 (the target becomes the chosen set's ambient, or the base ambient when the choice is normal)
    l.target = set_of(l, c).map_or(l.base, |s| s.ambient);
}

/// A re-evaluation after which the first fade step comes on the next tick
/// (claim 5: a nuclear detonation and a dominator launch).
pub fn reevaluate_forced(w: &mut World) {
    reevaluate(w);
    // spec: S-0104 claim 5, Nuke flash timing and Dominator timing (the first fade step comes on the next tick, however much of the current fade interval has passed)
    w.lighting.forced = true;
}

/// A nuclear missile detonates (claim 5): the nuke flash's tick 0.
pub fn nuke_detonated(w: &mut World) {
    w.lighting.nuke = Some(w.tick);
    // spec: S-0104 claim 5 and State and ordering (a nuclear missile detonating re-evaluates and chooses the nuke set)
    reevaluate_forced(w);
}

/// The fade interval chosen at a step taken now (claim 4, Formulas).
fn interval_now(w: &World) -> i32 {
    let l = &w.lighting;
    if nuke_phase(l, w.tick).is_some() {
        // spec: S-0104 Formulas, Ambient fade (nuke flash, either phase or tick 47: the nuke rate)
        l.map.nuke_rate
    } else if crate::domination::in_force(w) {
        if w.rules().quirks.dominator_rate_thousandths {
            // spec: S-0104 Formulas, Ambient fade and Edge cases (the dominator rate stored ×1000 and used directly as ticks; quirks.dominator_rate_thousandths)
            l.map.dominator_rate
        } else {
            // spec: S-0104 Edge cases (quirks.dominator_rate_thousandths off: ⌊v⌋ ticks)
            l.map.dominator_rate_whole
        }
    } else {
        // spec: S-0104 Formulas, Ambient fade (otherwise ⌊m·900⌋; the ion storm uses the normal rate)
        w.rules().superweapons.light.interval
    }
}

/// One fade step from `a` towards `target` by `step` (Formulas, Ambient
/// fade): never overshooting, a negative target counting as 0.
pub fn ambient_step(a: i32, target: i32, step: i32) -> i32 {
    // spec: S-0104 Formulas, Ambient fade (A_t = max(target, 0); A' = A_t within one step, else A + step·sgn(A_t − A))
    let t = target.max(0);
    if (i64::from(t) - i64::from(a)).abs() <= i64::from(step) {
        t
    } else if t > a {
        a.saturating_add(step)
    } else {
        a.saturating_sub(step)
    }
}

/// The lighting's work of a tick (claims 4 and 5), in S-0203 row 3 before
/// the domination update: the fade step when one falls due, then the
/// nuke flash's own phase change.
pub fn update(w: &mut World) {
    let t = w.tick;
    let lr = w.rules().superweapons.light;
    let q = w.rules().quirks;
    let forced = std::mem::take(&mut w.lighting.forced);
    let l = &w.lighting;
    let differs = l.current != l.target.max(0) || l.target < 0;
    let due = forced || t - l.last_step >= l.interval;
    if differs && due {
        if lr.zero_rate && q.zero_rate_freezes_ambient {
            // spec: S-0104 claim 4 and Edge cases (quirks.zero_rate_freezes_ambient: with a zero change rate the current ambient never moves, even for nuke and dominator fades)
        } else {
            let interval = interval_now(w);
            let l = &mut w.lighting;
            // spec: S-0104 claim 4 (a negative target is raised to 0 when a step falls due)
            l.target = l.target.max(0);
            l.current = if lr.zero_rate {
                // spec: S-0104 Edge cases (quirks.zero_rate_freezes_ambient off: jump straight to the target)
                l.target
            } else {
                ambient_step(l.current, l.target, lr.step)
            };
            // spec: S-0104 claim 4 (each step starts a new fade interval, its length chosen from the effects in force at that step)
            l.last_step = t;
            l.interval = interval;
        }
    }
    if w.lighting.nuke.is_some_and(|d| t - d == NUKE_BRIGHTENING) {
        // spec: S-0104 claim 5, Same-tick order and Nuke flash timing (on tick 31 the fading phase starts with a re-evaluation; that tick's fade stepped towards the old target)
        reevaluate(w);
    }
}

/// The set-base-ambient trigger action (code 73, S-0104 claim 14; S-0524
/// claim 7): the base, and the target unless an ion storm is in force.
pub fn set_base(w: &mut World, value: i32) {
    let storm = storm_in_force(w);
    let l = &mut w.lighting;
    // spec: S-0104 claim 14 (the integer slot becomes the base ambient in hundredths, without division or rounding)
    l.base = value;
    if !storm {
        // spec: S-0104 claim 14 (it sets the target as well, unless an ion storm is in force)
        l.target = value;
    }
}

/// The tint the tintable tables show, hundredths (claims 3 and 5): the
/// chosen set's tint, or `None` for every table's own tint.
// spec: S-0104 claim 3 (a tint of three colour components)
pub fn tint(w: &World) -> Option<[i32; 3]> {
    set_of(&w.lighting, w.lighting.choice).map(|s| s.tint)
}
