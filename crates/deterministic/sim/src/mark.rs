//! A building's charge flash and air-strike mark (S-0519 claims 25 and 26,
//! Formulas F6 and F7).
//!
//! Game state: the flash count *c* (claim 25) and the mark's phase sequence
//! with its mark clock (claim 26, Terms), both stepped in the building's own
//! turn right after the protection envelope of S-0513 claim 29 (S-0519
//! State and ordering). The mark's Initial dimming takes one ranged draw
//! from the scenario stream, so the sequence is hashed state. The flash
//! count is presentation, kept here because it counts the building's turns.
//!
//! Presentation: the flash pattern and step (F6), the mark factor *m* of
//! claim 26's table and the combined building brightness (F7), which the
//! drawing reads ([`flash_on`], [`factor`], [`brightness`]).

use serde::{Deserialize, Serialize};

use crate::rules::Kind;
use crate::world::{ObjectId, World};

/// The mark clock (S-0519 Terms, claim 26), ticks.
pub const MARK_CLOCK: i32 = 100_000; // spec: S-0519 Constants (mark clock 100000 ticks)
/// Initial rise and Final rise, ticks.
pub const MARK_RISE: i32 = 6; // spec: S-0519 Constants (mark phase lengths 6, 4, 20, 64, 64, 6, 4, 20)
/// Bright hold and Final bright hold, ticks.
pub const MARK_HOLD: i32 = 4; // spec: S-0519 Constants (mark phase lengths 6, 4, 20, 64, 64, 6, 4, 20)
/// Initial dimming's base length and Return's length, ticks.
pub const MARK_DIMMING: i32 = 20; // spec: S-0519 Constants (mark phase lengths 6, 4, 20, 64, 64, 6, 4, 20)
/// Pulse rise and Pulse fall, ticks.
pub const MARK_PULSE: i32 = 64; // spec: S-0519 Constants (mark phase lengths 6, 4, 20, 64, 64, 6, 4, 20)
/// Pulse fall goes to Low hold when the mark clock has less than this.
pub const MARK_LOW_ENTRY: i32 = 158; // spec: S-0519 Constants (mark low-hold and final thresholds 158, 31)
/// Low hold goes to Final rise when a later turn finds less than this.
pub const MARK_LOW_EXIT: i32 = 31; // spec: S-0519 Constants (mark low-hold and final thresholds 158, 31)
/// The dimming draw's inclusive range, ticks.
const MARK_DRAW: (i32, i32) = (-5, 5); // spec: S-0519 Constants (mark dimming draw range −5 to 5, inclusive)
/// Factor units per 1× brightness.
pub const FACTOR_SCALE: i32 = 256; // spec: S-0519 Constants (mark factor scale 256)
/// The brightness cap of the factor-bearing phases, ‰.
pub const CAP: i32 = 2000; // spec: S-0519 Constants (brightness cap 2000 ‰), F7
/// The flash step, ‰.
pub const FLASH_STEP: i32 = 500; // spec: S-0519 Constants (flash step 500 ‰), F6
/// Above this brightness the flash darkens, ‰.
pub const FLASH_THRESHOLD: i32 = 1500; // spec: S-0519 Constants (flash step-down threshold 1500 ‰), F6
/// The flash pattern's period, ticks.
const FLASH_PERIOD: i32 = 4; // spec: S-0519 Constants (flash pattern period 4 ticks, 2 on, 2 off)

/// The mark's phase sequence (S-0519 claim 26), each phase with the tick it
/// was entered in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// Before the first marked turn *q* after a restart.
    #[default]
    Before,
    /// Initial rise, entered in this tick.
    Rise(i32),
    /// Bright hold, entered in this tick.
    BrightHold(i32),
    /// Initial dimming, entered in tick `since` with its draw taken, for
    /// `length` = 20 + the draw ticks.
    Dimming {
        /// The tick it was entered in.
        since: i32,
        /// Its length in ticks.
        length: i32,
    },
    /// Pulse rise, entered in this tick.
    PulseRise(i32),
    /// Pulse fall, entered in this tick.
    PulseFall(i32),
    /// Low hold, entered in this tick.
    LowHold(i32),
    /// Final rise, entered in this tick.
    FinalRise(i32),
    /// Final bright hold, entered in this tick.
    FinalHold(i32),
    /// Return, entered in this tick.
    Return(i32),
    /// Ordinary until unmarked or restarted.
    Ordinary,
}

/// A building's flash and mark state (S-0519 Terms: flash count, mark clock).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Look {
    /// The flash count *c*, ticks of flashing still to come (claim 25).
    pub flash: i32,
    /// The mark's phase sequence (claim 26).
    pub phase: Phase,
    /// The tick the mark clock was last started (claim 26: by a start or
    /// a redirect that marks the building).
    pub clock_start: i32,
}

/// Sets building `b`'s flash count to `c` (S-0519 claim 25): a later flash
/// of the same building replaces the count; counts do not add.
pub fn start_flash(w: &mut World, b: ObjectId, c: i32) {
    if let Some(o) = w.obj_mut(b) {
        // spec: S-0519 claim 25 (planting sets c to L; a later flash replaces the count, counts do not add)
        o.combat.look.flash = c;
    }
}

/// A start or redirect that marks building `b` (S-0519 claims 17, 19 and
/// 26): the phase sequence goes back to *before the first marked turn* and
/// the mark clock starts.
pub fn restart(w: &mut World, b: ObjectId) {
    let t = w.tick;
    if let Some(o) = w.obj_mut(b).filter(|o| o.kind == Kind::Building) {
        // spec: S-0519 claim 26 (Restart: back to before the first marked turn; the mark clock starts)
        o.combat.look.phase = Phase::Before;
        o.combat.look.clock_start = t;
    }
}

/// The building's turn (S-0519 claims 25 and 26, State and ordering): the
/// flash count drops by 1 while positive, and the mark's phase sequence
/// advances by at most one phase, taking its one draw on leaving Bright
/// hold. Called right after the protection envelope's turn.
pub fn turn(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if o.kind != Kind::Building {
        return;
    }
    let look = o.combat.look;
    let t = w.tick;
    let marked = crate::strike::marked(w, id);
    // spec: S-0519 claim 25, F6 (each building turn with c > 0 sets c to c − 1)
    let flash = if look.flash > 0 {
        look.flash - 1
    } else {
        look.flash
    };
    let phase = if !marked {
        // spec: S-0519 claim 26 (at any turn while not marked the sequence returns to before the first marked turn, without a draw)
        Phase::Before
    } else {
        // spec: S-0519 claim 26 (M, the mark clock's remaining ticks)
        let m = look.clock_start + MARK_CLOCK - t;
        // spec: S-0519 claim 26 (a phase entered in tick s ends at the first turn with r = max(0, ℓ − (n − s)) = 0; one phase per turn; the M tests only at the transitions shown)
        match look.phase {
            // spec: S-0519 claim 26 (Initial rise starts at q, the first turn while marked after a restart)
            Phase::Before => Phase::Rise(t),
            Phase::Rise(s) if t - s >= MARK_RISE => Phase::BrightHold(t),
            Phase::BrightHold(s) if t - s >= MARK_HOLD => {
                let half = w.rules().quirks.ranged_draw_half_span;
                // spec: S-0519 claims 21 and 26 (exactly one inclusive ranged draw from −5 through 5 on the scenario stream, in the turn that ends Bright hold; Initial dimming lasts 20 + the draw)
                let d = w.rng.ranged(MARK_DRAW.0, MARK_DRAW.1, half);
                Phase::Dimming {
                    since: t,
                    length: MARK_DIMMING + d,
                }
            }
            Phase::Dimming { since, length } if t - since >= length => Phase::PulseRise(t),
            Phase::PulseRise(s) if t - s >= MARK_PULSE => Phase::PulseFall(t),
            // spec: S-0519 claim 26 (Pulse fall: Low hold if M < 158 ticks, otherwise Pulse rise)
            Phase::PulseFall(s) if t - s >= MARK_PULSE && m < MARK_LOW_ENTRY => Phase::LowHold(t),
            Phase::PulseFall(s) if t - s >= MARK_PULSE => Phase::PulseRise(t),
            // spec: S-0519 claim 26 (Low hold until a later turn finds M < 31 ticks)
            Phase::LowHold(s) if t > s && m < MARK_LOW_EXIT => Phase::FinalRise(t),
            Phase::FinalRise(s) if t - s >= MARK_RISE => Phase::FinalHold(t),
            Phase::FinalHold(s) if t - s >= MARK_HOLD => Phase::Return(t),
            Phase::Return(s) if t - s >= MARK_DIMMING => Phase::Ordinary,
            p => p,
        }
    };
    if let Some(o) = w.obj_mut(id) {
        o.combat.look.flash = flash;
        o.combat.look.phase = phase;
    }
}

/// Whether the flash is on with flash count `c` (F6): *c* mod 4 is 2 or 3.
pub fn flash_on(c: i32) -> bool {
    // spec: S-0519 F6 (on = (c mod 4 ≥ 2))
    c > 0 && c.rem_euclid(FLASH_PERIOD) >= 2
}

/// The flashed brightness *b*<sub>F</sub> of brightness `b` ‰ (F6): +500
/// while on and *b* ≤ 1500, −500 while on and *b* > 1500; integer, no clamp.
pub fn flashed(b: i32, on: bool) -> i32 {
    if !on {
        // spec: S-0519 F6 (b when off)
        return b;
    }
    if b > FLASH_THRESHOLD {
        // spec: S-0519 F6 (b − 500 when on and b > 1500)
        b - FLASH_STEP
    } else {
        // spec: S-0519 F6 (b + 500 when on and b ≤ 1500)
        b + FLASH_STEP
    }
}

/// The mark factor *m* in tick `t` (claim 26's table); `None` before the
/// first marked turn and in Ordinary, where the brightness is unchanged.
pub fn factor(phase: Phase, t: i32) -> Option<i32> {
    // spec: S-0519 claim 26 (a phase of length ℓ entered in tick s has r = max(0, ℓ − (n − s)) in tick n)
    let r = |s: i32, l: i32| (l - (t - s)).max(0);
    // spec: S-0519 claim 26 (the factor column; integer arithmetic, divisions truncated toward zero)
    let m = match phase {
        // spec: S-0519 claim 26 (Before the first marked turn: none)
        Phase::Before => return None,
        // spec: S-0519 claim 26 (Initial rise: trunc((12 − r) × 256 / 6))
        Phase::Rise(s) => (12 - r(s, MARK_RISE)) * FACTOR_SCALE / 6,
        // spec: S-0519 claim 26 (Bright hold: 512)
        Phase::BrightHold(_) => 512,
        // spec: S-0519 claim 26 (Initial dimming: trunc((r + 20) × 256 / 20))
        Phase::Dimming { since, length } => (r(since, length) + 20) * FACTOR_SCALE / 20,
        // spec: S-0519 claim 26 (Pulse rise: 4 × (128 − r))
        Phase::PulseRise(s) => 4 * (128 - r(s, MARK_PULSE)),
        // spec: S-0519 claim 26 (Pulse fall: 4 × (r + 64))
        Phase::PulseFall(s) => 4 * (r(s, MARK_PULSE) + 64),
        // spec: S-0519 claim 26 (Low hold: 256)
        Phase::LowHold(_) => 256,
        // spec: S-0519 claim 26 (Final rise: trunc((12 − r) × 256 / 6))
        Phase::FinalRise(s) => (12 - r(s, MARK_RISE)) * FACTOR_SCALE / 6,
        // spec: S-0519 claim 26 (Final bright hold: 512)
        Phase::FinalHold(_) => 512,
        // spec: S-0519 claim 26 (Return: trunc((r + 20) × 256 / 20))
        Phase::Return(s) => (r(s, MARK_DIMMING) + 20) * FACTOR_SCALE / 20,
        // spec: S-0519 claim 26 (Ordinary: none)
        Phase::Ordinary => return None,
    };
    Some(m)
}

/// What F7 needs of a building in the shown tick: the flash count, the
/// protection factor (`Some(None)`: protected, no factor; `None`:
/// unprotected), and the mark (`Some(m)`: marked, with factor `m` or none).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Adjust {
    /// The flash count *c*.
    pub flash: i32,
    /// Protected, with the envelope's factor if in a factor phase.
    pub protection: Option<Option<i32>>,
    /// Marked, with the mark factor if in a factor phase.
    pub mark: Option<Option<i32>>,
}

impl Adjust {
    /// Building `b`'s adjustments in tick `t`.
    pub fn of(w: &World, b: ObjectId, t: i32) -> Self {
        let Some(o) = w.obj(b) else {
            return Self::default();
        };
        let protection =
            o.sw.protection
                .as_ref()
                .and_then(|p| crate::superweapons::protect::appearance(p, t))
                .map(|(_, m)| m);
        let mark = crate::strike::marked(w, b).then(|| factor(o.combat.look.phase, t));
        Self {
            flash: o.combat.look.flash,
            protection,
            mark,
        }
    }

    /// The building brightness *b*₃ of ordinary brightness `b` ‰ (F7).
    pub fn brightness(&self, b: i32) -> i32 {
        // spec: S-0519 F7 (b₁ = F6's result)
        let b1 = flashed(b, flash_on(self.flash));
        // spec: S-0519 F7 (b₂ = P(b₁), the protection-brightness function of S-0513 F9, the identity outside its factor phases)
        let b2 = crate::superweapons::protect::brightness(b1, self.protection.flatten());
        match (self.mark, self.protection) {
            // spec: S-0519 F7 (marked in a phase with a factor: min(2000, ⌊m × b₂ / 256⌋); a negative b₂ rounds toward minus infinity)
            (Some(Some(m)), _) => {
                let v = (i64::from(m) * i64::from(b2)).div_euclid(i64::from(FACTOR_SCALE));
                v.min(i64::from(CAP)) as i32
            }
            // spec: S-0519 F7 (marked in a phase without a factor, or unmarked but protected: b₂)
            (Some(None), _) | (None, Some(_)) => b2,
            // spec: S-0519 F7 (unmarked and unprotected: b₁)
            (None, None) => b1,
        }
    }
}
