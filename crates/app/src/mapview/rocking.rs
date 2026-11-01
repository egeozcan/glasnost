//! Body rocking (S-0113): the pitch and roll of a rocking body, their
//! rates, the pushes that set the rates and the rocking step that moves the
//! angles back to level, kept by the view beside the game.
//!
//! S-0113 claim 19 makes the angles and rates game state, advanced in each
//! turn of the body, with fire points built on them and claim 9's
//! destruction. Our simulation does not keep them yet (a seam of
//! docs/m8-claims/intake53-S-0107-S-0108-S-0113.md): it reports what rocks
//! (`Effect::Rock`, `Effect::DroneRock`, `Effect::GrapplePush`,
//! `Effect::OverloadRock`, `Effect::CrashTumble`), and the view applies S-0113 to those reports in
//! the simulation's own soft-float arithmetic and rounding mode, once per
//! game tick and never per drawn frame, so the rules here can move into the
//! simulation unchanged. Only drawing reads any of it.
//!
//! Within a tick the bodies step first and that tick's pushes then set the
//! rates (S-0113 claim 19 and State and ordering: the rocking step is the
//! first part of the turn, and a push shows from the next turn).

use std::collections::BTreeMap;

use glasnost_num::{F32, F64, Rounding, geom, trig};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::rules::Kind as SimKind;
use glasnost_sim::world::{ObjectId, Pos, World};

/// 0.04, F1's push base (single, widened).
const PUSH_BASE: F32 = F32(0x3d23_d70a); // spec: S-0113 F1 and Constants (push base 0.04, single)
/// 2.5 × 10⁻⁵, F1's fall-off per lepton (single, widened).
const PUSH_FALL_OFF: F32 = F32(0x37d1_b717); // spec: S-0113 F1 and Constants (fall-off 2.5 × 10⁻⁵ per lepton, single)
/// 0.01, the smallest push (single, widened).
const SMALLEST_PUSH: F32 = F32(0x3c23_d70a); // spec: S-0113 F1 and Constants (smallest push 0.01, single)
/// 0.05, the push cap (single).
const PUSH_CAP: F32 = F32(0x3d4c_cccd); // spec: S-0113 F1 and Constants (push cap 0.05, single)
/// ½, the pitch factor of area damage and drones (single, widened).
const HALF: F32 = F32(0x3f00_0000); // spec: S-0113 claims 1, 11 and 13 and Constants (pitch factor ½)
/// 0.0002, F1's side sign tolerance (double).
const SIDE_TOLERANCE: F64 = F64(0x3f2a_36e2_eb1c_432d); // spec: S-0113 F1 and Constants (side sign tolerance 0.0002, double)
/// 2 × 10⁻⁵, the level band's half-width (double).
const LEVEL_BAND: F64 = F64(0x3ef4_f8b5_88e3_68f1); // spec: S-0113 Terms and Constants (level band ±2 × 10⁻⁵, double)
/// 0.002, the small pull (single, widened).
const SMALL_PULL: F32 = F32(0x3b03_126f); // spec: S-0113 claims 7 and 8 and Constants (small pull 0.002, single)
/// 0.0050000002374872565, the large pull (double, not a single).
const LARGE_PULL: F64 = F64(0x3f74_7ae1_5800_0000); // spec: S-0113 claim 8 and Constants (large pull 0.0050000002374872565, double)
/// π/4, the usual limit (single).
pub const USUAL_LIMIT: F32 = F32(0x3f49_0fdb); // spec: S-0113 claim 6 and Constants (usual limit π/4, single)
/// π/10, the crush-slowdown pitch limit (single).
pub const SLOWED_PITCH_LIMIT: F32 = F32(0x3ea0_d97c); // spec: S-0113 claim 6 and Constants (crush-slowdown pitch limit π/10, single)
/// π/2, the edge (single).
const EDGE: F32 = F32(0x3fc9_0fdb); // spec: S-0113 claim 9 and Constants (edge π/2, single)
/// π, the rolling-over angle (single).
const OVER: F32 = F32(0x4049_0fdb); // spec: S-0113 claim 9 and Constants (rolling-over angle π, single)
/// −π/4 (single): a balloon-hover body's lowest pitch and roll while it
/// crashes.
const MINUS_QUARTER: F32 = F32(0xbf49_0fdb); // spec: S-0113 claim 16 (the pitch raised to −π/4 when below it, the roll held within ±π/4; π/4 a single, claim 8)
/// 0.15, F5's crash roll rate spread (double).
const CRASH_ROLL_SPREAD: F64 = F64(0x3fc3_3333_3333_3333); // spec: S-0113 F5 and Constants (crash roll rate spread 0.15, a double-precision constant)
/// 0.1, F5's crash roll rate base and crash pitch rate spread (double).
const CRASH_TENTH: F64 = F64(0x3fb9_9999_9999_999a); // spec: S-0113 F5 and Constants (crash roll rate base 0.1 and pitch rate spread 0.1, double-precision constants)
/// π/10 for F4's frame angle (double).
const GRAPPLE_STEP: F64 = F64(0x3fd4_1b2f_769c_f0e0); // spec: S-0113 F4 (π/10 a double-precision constant)

/// 0.01, the area strength's factor per damage point (double).
const STRENGTH_PER_DAMAGE: F64 = F64(0x3f84_7ae1_47ae_147b); // spec: S-0113 F2 (0.01 a double-precision constant)
/// 4.0, the strength cap (double).
const STRENGTH_CAP: F64 = F64(0x4010_0000_0000_0000); // spec: S-0113 F2 and Constants (strength cap 4.0, double)
/// A drone delivery's strength, 1.5 (S-0614 F2).
const DRONE_STRENGTH: F32 = F32(0x3fc0_0000); // spec: S-0614 F2 and Constants (drone rock strength 1.5); S-0113 claim 13
/// The drone's push point distance, leptons.
const DRONE_DISTANCE: i32 = 2; // spec: S-0113 F3 and Constants (σ₂ = ±2 leptons)
/// The distance of a centre-cell body's push point toward the source,
/// leptons, per axis truncated.
const SOURCE_DISTANCE: i64 = 10; // spec: S-0113 claim 11 and Constants (10 leptons toward the source)

/// A level-1 overload pulse's roll rate, 0.015 (S-0601 F4), single.
const OVERLOAD_RATE_LOW: F32 = F32(0x3c75_c28f); // spec: S-0601 F4 and Constants (rocking magnitude, level 1: 0.015 radians per tick, the nearest binary32)
/// The rate from level 2 on, 0.03 (S-0601 F4), single.
const OVERLOAD_RATE_HIGH: F32 = F32(0x3cf5_c28f); // spec: S-0601 F4 and Constants (rocking magnitude, level ≥ 2: 0.03 radians per tick, the nearest binary32)
/// The draw from which an overload's rate is positive (S-0601 F4).
const OVERLOAD_SIGN_DRAW: i32 = 50; // spec: S-0601 F4 and Constants (rocking sign threshold 50)

/// A grappling push phase's turns per frame *k* (S-0113 claim 14).
const GRAPPLE_FRAME_TURNS: i32 = 4; // spec: S-0113 claim 14 (k goes up by 1 every 4 of the host's turns)
/// The turn of a push phase in which it ends with a roll of 0.
const GRAPPLE_END_TURN: i32 = 40; // spec: S-0113 claim 14 (turn 40, where k would reach 10: the phase ends and the roll is set to 0)

/// The step the drawn angles are held to, radians: S-0106's rocking
/// threshold, so a drawn angle rocks exactly when the held one is at least
/// 0.005 radians (S-0106 Terms), and a body is drawn under few distinct
/// angles (each pair is an image of its own, S-0105 claim 26).
// spec: S-0106 Terms (rocking: a pitch or roll of 0.005 radians or more); design: the drawn angles are held to that step, truncated toward level, so few distinct images are drawn
const DRAWN_STEP: f32 = glasnost_render::voxel::ROCKING_THRESHOLD;

/// The most ticks one [`Rocking::advance`] catches up.
// design: a jump of many ticks (a load or a seek) steps at most this many; a push from level is level again within 42 steps (S-0113 State and ordering)
const MAX_CATCH_UP: i32 = 120;

/// F2's strength of an area push of damage *D*: RZ₂₄(min(RZ₅₃(*D* × 0.01),
/// 4.0)) (S-0113 claim 11).
pub fn area_strength(damage: i32, mode: Rounding) -> F32 {
    // spec: S-0113 F2 (s = RZ₂₄(min(RZ₅₃(D × 0.01), 4.0)))
    F64::from_i64(i64::from(damage), mode)
        .mul(STRENGTH_PER_DAMAGE, mode)
        .min(STRENGTH_CAP)
        .to_f32(mode)
}

/// F5's crash tumble rates *(pitch rate, roll rate)* from S-0312 claim 16's
/// three draws: *u*₁, the sense draw *b* (0 or 1) and *u*₃ (S-0113 claim 16).
pub fn crash_rates(u1: F64, sense: i32, u3: F64, mode: Rounding) -> (F32, F32) {
    // spec: S-0113 F5 (roll rate = RZ₂₄(RZ₅₃(u₁ × 0.15) + 0.1), negated when b = 0)
    let roll = u1
        .mul(CRASH_ROLL_SPREAD, mode)
        .add(CRASH_TENTH, mode)
        .to_f32(mode);
    let roll = if sense == 0 {
        roll.to_f64().negated().to_f32(mode)
    } else {
        roll
    };
    // spec: S-0113 F5 (pitch rate = RZ₂₄(u₃ × 0.1))
    let pitch = u3.mul(CRASH_TENTH, mode).to_f32(mode);
    (pitch, roll)
}

/// One rocking body's angles and rates (S-0113 Terms), single precision.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Body {
    /// The pitch, radians; positive lowers the bow and raises the stern.
    pub pitch: F32,
    /// The roll, radians; positive raises the body's left side.
    pub roll: F32,
    /// The pitch rate, radians per tick.
    pub pitch_rate: F32,
    /// The roll rate, radians per tick.
    pub roll_rate: F32,
}

impl Body {
    /// Whether all four values are exactly 0.
    fn level(&self) -> bool {
        [self.pitch, self.roll, self.pitch_rate, self.roll_rate]
            .iter()
            .all(|v| v.to_f64().eq_num(F64::ZERO))
    }

    /// Whether either angle is beyond ±π (S-0113 claim 9).
    fn over(&self) -> bool {
        let over = OVER.to_f64();
        [self.pitch, self.roll].iter().any(|a| {
            let a = a.to_f64();
            a.gt(over) || a.lt(over.negated())
        })
    }
}

/// The inputs of one push (S-0113 F1).
#[derive(Clone, Copy, Debug)]
pub struct Push {
    /// The body's position *B*, leptons.
    pub body: (i64, i64, i64),
    /// The push point *Q*, leptons.
    pub from: (i64, i64, i64),
    /// The strength *s*.
    pub strength: F32,
    /// The pitch factor *h*: 1 or ½.
    pub half: bool,
    /// The weight *W*.
    pub weight: F64,
    /// The body's facing *f*.
    pub facing: u16,
}

/// F1: the pitch and roll rates a push sets, or `None` when it changes
/// nothing (S-0113 claims 1 to 4). `table` is S-0204's
/// `quirks.table_root_distance` and `direction` its `quirks.table_direction`.
pub fn push_rates(p: &Push, table: bool, direction: bool, mode: Rounding) -> Option<(F32, F32)> {
    let d64 = |v: i64| F64::from_i64(v, mode);
    // spec: S-0113 F1 (w = B − Q per axis, integers)
    let w = (
        p.body.0 - p.from.0,
        p.body.1 - p.from.1,
        p.body.2 - p.from.2,
    );
    // spec: S-0113 F1 (each component squared and summed in double precision: exact within F1's input domain)
    let sq = |v: i64| u128::from(v.unsigned_abs()) * u128::from(v.unsigned_abs());
    let all = sq(w.0) + sq(w.1) + sq(w.2);
    let flat = sq(w.0) + sq(w.1);
    // spec: S-0113 F1 (no change when RZ₂₄(d) < 2 × 10⁻⁵, that is when w = 0) and claim 3 (Q at the body's own position)
    if all == 0 {
        return None;
    }
    let root = |n: u128| {
        if table {
            // spec: S-0113 F1 (r, S-0204's table square root of the exact sum, a single value)
            geom::table_root(u64::try_from(n).unwrap_or(u64::MAX), mode).to_f64()
        } else {
            // spec: S-0113 F1 (quirks.table_root_distance off: RZ₅₃(√…), not truncated)
            exact_root(F64::from_parts(false, n, 0, mode), mode)
        }
    };
    let d = root(all);
    // spec: S-0113 F1 (k = RZ₂₄(((0.04 − d × 2.5 × 10⁻⁵) × s) / W))
    let k = PUSH_BASE
        .to_f64()
        .sub(d.mul(PUSH_FALL_OFF.to_f64(), mode), mode)
        .mul(p.strength.to_f64(), mode)
        .div(p.weight, mode)
        .to_f32(mode)
        .to_f64();
    // spec: S-0113 F1 and claim 3 (no change when k < 0.01 or k is not a number)
    if k.is_nan() || k.lt(SMALLEST_PUSH.to_f64()) {
        return None;
    }
    // spec: S-0113 F1 and claim 3 (k above 0.05 is set to 0.05)
    let k = k.min(PUSH_CAP.to_f64());
    if flat == 0 {
        // spec: S-0113 F1 and claim 4 (ℓ = 0: both rates become 0)
        return Some((F32::ZERO, F32::ZERO));
    }
    let l = root(flat);
    // spec: S-0113 F1 (aₓ = RZ₂₄(wₓ / ℓ), aᵧ = RZ₂₄(−wᵧ / ℓ))
    let ax = d64(w.0).div(l, mode).to_f32(mode).to_f64();
    let ay = d64(w.1).negated().div(l, mode).to_f32(mode).to_f64();
    let theta = facing_angle(p.facing, direction, mode);
    // spec: S-0113 F1 (c = cosT(θ), n = sinT(θ))
    let c = trig::cos_t(theta, mode);
    let n = trig::sin_t(theta, mode);
    // spec: S-0113 F1 (forward = RZ₂₄(aₓ × c + aᵧ × n))
    let forward = ax
        .mul(c, mode)
        .add(ay.mul(n, mode), mode)
        .to_f32(mode)
        .to_f64();
    // spec: S-0113 F1 (e = RZ₅₃(aₓ × n − aᵧ × c), not rounded to single)
    let e = ax.mul(n, mode).sub(ay.mul(c, mode), mode);
    let e2 = e.mul(e, mode);
    let m = if table {
        // spec: S-0113 F1 (m = √ₜ(RZ₅₃(e²)), S-0105's table square root, a single value)
        geom::table_root_real(e2, true, mode)
    } else {
        // spec: S-0113 F1 (quirks.table_root_distance off: m = RZ₅₃(√RZ₅₃(e²)))
        exact_root(e2, mode)
    };
    // spec: S-0113 F1 (side = −m when |forward × c − m × n − aₓ| > 0.0002 or |m × c + forward × n − aᵧ| > 0.0002, else +m)
    let off_x = forward
        .mul(c, mode)
        .sub(m.mul(n, mode), mode)
        .sub(ax, mode)
        .abs();
    let off_y = m
        .mul(c, mode)
        .add(forward.mul(n, mode), mode)
        .sub(ay, mode)
        .abs();
    let side = if off_x.gt(SIDE_TOLERANCE) || off_y.gt(SIDE_TOLERANCE) {
        m.negated()
    } else {
        m
    };
    // spec: S-0113 claim 1 (h = 1 for a direct-rocking hit, ½ for an area push and a drone delivery)
    let h = if p.half {
        HALF.to_f64()
    } else {
        F64::from_i64(1, mode)
    };
    // spec: S-0113 F1 (pitch rate = RZ₂₄(forward × k × h), roll rate = RZ₂₄(−(side × k)))
    let pitch = forward.mul(k, mode).mul(h, mode).to_f32(mode);
    let roll = side.mul(k, mode).negated().to_f32(mode);
    Some((pitch, roll))
}

/// F1's facing angle θ: *g* = *f* as a signed 16-bit value minus 16383,
/// θ = *g* × (−π / 32767); with `quirks.table_direction` off, S-0204's
/// exact scale (*f̂* − 16384) × (−2π / 65536).
fn facing_angle(f: u16, direction: bool, mode: Rounding) -> F64 {
    // spec: S-0113 F1 (g = f as a signed 16-bit value)
    let g = i64::from(f as i16);
    if direction {
        // spec: S-0113 F1 (θ = (g − 16383) × (−π / 32767), the double of S-0204's short scale)
        F64::from_i64(g - 16383, mode).mul(glasnost_sim::consts::WALK_C2, mode)
    } else {
        // spec: S-0113 F1 and S-0204 (quirks.table_direction off: (f̂ − 16384) × (−2π / 65536))
        F64::from_i64(g - 16384, mode).mul(glasnost_sim::consts::WALK_C, mode)
    }
}

/// RZ₅₃(√x) of a double x ≥ 0 under `mode`: the exact root, rounded once.
fn exact_root(x: F64, mode: Rounding) -> F64 {
    let Some((neg, m, e)) = x.parts() else {
        return F64::ZERO;
    };
    if neg {
        return F64::ZERO;
    }
    // x = m × 2^e; make the exponent even and widen the radicand so its
    // root has more than 53 significant bits.
    let mut m = u128::from(m);
    let mut e = e;
    if e % 2 != 0 {
        m <<= 1;
        e -= 1;
    }
    let shift = (126 - (128 - m.leading_zeros())) & !1;
    let radicand = m << shift;
    let r = geom::isqrt(radicand);
    let exp = (e - shift as i32) / 2;
    if r * r == radicand {
        F64::from_parts(false, r, exp, mode)
    } else {
        // A sticky bit below the root's last place, so a nearest rounding
        // never meets a false tie.
        F64::from_parts(false, (r << 1) | 1, exp - 1, mode)
    }
}

/// One axis's rocking step, table R (S-0113 claims 5 to 9, 15): the angle
/// `a` before the step, its rate `r` after any push, its limit and whether
/// it is the roll. `escape` is `quirks.rocking_limit_escape`. Bodies are
/// never melee-linked in our engine yet (claim 12 is a seam), so the linked
/// rows are not taken.
pub fn axis_step(
    a: F32,
    r: F32,
    limit: F32,
    roll: bool,
    escape: bool,
    mode: Rounding,
) -> (F32, F32) {
    let a = a.to_f64();
    let mut r = r.to_f64();
    // spec: S-0113 table R (move: r = 0 exactly, the angle becomes 0 and the rest of the step does nothing)
    if r.eq_num(F64::ZERO) {
        return (F32::ZERO, F32::ZERO);
    }
    // spec: S-0113 table R and claim 8 (new angle = RZ₂₄(a + r))
    let mut new = a.add(r, mode).to_f32(mode).to_f64();
    let edge = EDGE.to_f64();
    let past = a.gt(edge) || a.lt(edge.negated());
    let l = limit.to_f64();
    if !past {
        // spec: S-0113 table R (hold: not linked, a not past the edge, a < L, new angle > L: angle L, r = 0) and claim 15; Edge cases (quirks.rocking_limit_escape off: held whatever the angle before the step)
        if (!escape || a.lt(l)) && new.gt(l) {
            new = l;
            r = F64::ZERO;
        } else if (!escape || a.gt(l.negated())) && new.lt(l.negated()) {
            new = l.negated();
            r = F64::ZERO;
        }
    }
    let small = SMALL_PULL.to_f64();
    let band = LEVEL_BAND;
    // spec: S-0113 table R (pull: the first matching row, chosen from the angle before the step) and claim 9 (past the edge: away from level)
    let pulled = if a.gt(edge) {
        r.add(small, mode)
    } else if a.lt(edge.negated()) {
        r.sub(small, mode)
    } else if roll {
        // spec: S-0113 table R and claim 7 (the roll: 0.005 while moving back toward level or at rate 0, 0.002 while still moving away)
        match (a.gt(band), r.le(F64::ZERO), r.ge(F64::ZERO)) {
            (true, true, _) => r.sub(LARGE_PULL, mode),
            (true, false, _) => r.sub(small, mode),
            (false, _, true) => r.add(LARGE_PULL, mode),
            (false, _, false) => r.add(small, mode),
        }
    } else if a.gt(band) {
        // spec: S-0113 table R and claim 7 (the pitch: lowered by 0.005 above the band)
        r.sub(LARGE_PULL, mode)
    } else {
        // spec: S-0113 table R and claim 7 (raised by 0.005 within the band or below it)
        r.add(LARGE_PULL, mode)
    };
    // spec: S-0113 table R (each pulled rate is rounded to single)
    let mut rate = pulled.to_f32(mode);
    let mut angle = new.to_f32(mode);
    // spec: S-0113 table R (stop: an angle that comes back into the level band from outside becomes 0 with its rate)
    if (a.gt(band) && new.lt(band)) || (a.lt(band.negated()) && new.gt(band.negated())) {
        angle = F32::ZERO;
        rate = F32::ZERO;
    }
    (angle, rate)
}

/// A drawn angle held to the drawing step, truncated toward level.
pub fn drawn(angle: F32) -> f32 {
    let a = f32::from_bits(angle.0);
    let steps = (a / DRAWN_STEP).trunc();
    if steps == 0.0 || steps.is_nan() {
        // Level, never a negative zero (its bits would key images apart).
        return 0.0;
    }
    steps * DRAWN_STEP
}

/// A grappled host's push phase (S-0113 claim 14).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Grapple {
    /// The current push phase: the tick of its way draw (its turn 1) and
    /// whether its way draw was 1 (σ = −1).
    push: Option<(i32, bool)>,
    /// A push has started since the grab: every later push follows a
    /// strike (S-0614 claim 10).
    pushed: bool,
}

/// The view's rocking state.
#[derive(Clone, Debug, Default)]
pub struct Rocking {
    /// The bodies that rock, by object.
    bodies: BTreeMap<ObjectId, Body>,
    /// Each host of a grappling parasite.
    grapples: BTreeMap<ObjectId, Grapple>,
    /// The lean of a sinking vessel that was rocked when it began to sink
    /// (S-0504 claim 15, *Starting value*): its pitch, stepped here from
    /// the rocked pitch at each descent by F4, in place of the
    /// simulation's, which starts from level.
    leans: BTreeMap<ObjectId, F32>,
    /// The last tick moved.
    last: Option<i32>,
}

impl Rocking {
    /// Takes one tick's rocking reports, after that tick's step
    /// ([`Rocking::advance`], which counts the turn as `w.tick`).
    pub fn take(&mut self, w: &World, effects: &[Effect]) {
        let mode = w.rules().mode;
        for e in effects {
            match e {
                Effect::Rock { damage, pushed } => {
                    // spec: S-0113 claim 11 and F2 (an area push: h = ½, s = min(D × 0.01, 4.0))
                    let s = area_strength(*damage, mode);
                    let sources = source_points(w, pushed);
                    for (i, &(id, from)) in pushed.iter().enumerate() {
                        self.area(w, id, from, sources[i], s);
                    }
                }
                Effect::DroneRock { host, side } => self.drone(w, *host, *side),
                Effect::OverloadRock {
                    controller,
                    level,
                    draw,
                } => self.overload(w, *controller, *level, *draw),
                Effect::CrashTumble { id, u1, sense, u3 } => {
                    // spec: S-0113 claim 16 and F5 (an object that is not infantry starts to crash: the three draws set both rates; the angles keep their values)
                    let (pitch, roll) = crash_rates(*u1, *sense, *u3, mode);
                    let b = self.bodies.entry(*id).or_default();
                    b.pitch_rate = pitch;
                    b.roll_rate = roll;
                }
                Effect::GrapplePush { host, way } => {
                    let g = self.grapples.entry(*host).or_default();
                    if g.pushed {
                        // spec: S-0113 claim 14 (the roll is set to 0 in each strike turn; a later push follows a strike, S-0614 claim 10)
                        if let Some(b) = self.bodies.get_mut(host) {
                            b.roll = F32::ZERO;
                        }
                    }
                    // spec: S-0113 claim 14 (each push has its own way draw; the turn of the draw is the phase's turn 1)
                    g.push = Some((w.tick, *way == 1));
                    g.pushed = true;
                }
                _ => {}
            }
        }
    }

    /// An area push of body `id` from the reported point `from`, which is
    /// the source object's position when `source` (S-0113 claim 11).
    fn area(&mut self, w: &World, id: ObjectId, from: Pos, source: bool, s: F32) {
        let Some(o) = w.obj(id) else { return };
        let b = (i64::from(o.pos.x), i64::from(o.pos.y), i64::from(o.pos.z));
        let q = if source {
            match toward(
                w,
                b,
                (i64::from(from.x), i64::from(from.y), i64::from(from.z)),
            ) {
                Some(q) => q,
                // spec: S-0113 claim 11 (a source at exactly the body's position changes nothing, claim 3)
                None => return,
            }
        } else {
            // spec: S-0113 claim 11 (every other body, and a centre-cell body when there is no source object, is pushed from P)
            (i64::from(from.x), i64::from(from.y), i64::from(from.z))
        };
        self.push(w, id, q, s, true);
    }

    /// A drone parasite's delivery to `host` (S-0113 claim 13, F3).
    fn drone(&mut self, w: &World, host: ObjectId, side: i32) {
        let Some(o) = w.obj(host) else { return };
        let mode = w.rules().mode;
        let direction = w.rules().quirks.table_direction;
        let f = o.body.value(w.tick);
        // spec: S-0113 F3 (σ₂ = +2 when r = 0 and −2 when r = 1)
        let s = if side == 0 {
            DRONE_DISTANCE
        } else {
            -DRONE_DISTANCE
        };
        // spec: S-0113 F3 (g = (f − 16383) as a signed 16-bit value, t = (g − 16383) × (−π / 32767); Q = (trunc(X + σ₂ cosT t), trunc(Y − σ₂ sinT t), Z)), which is S-0208 F3's step along facing f − 16383; with quirks.table_direction off, f − 16384 on S-0204's exact scale
        let (qx, qy) = if direction {
            glasnost_sim::walk::step_point(o.pos.x, o.pos.y, f.wrapping_sub(16383), s, true, mode)
        } else {
            glasnost_sim::walk::step_point(o.pos.x, o.pos.y, f.wrapping_sub(16384), s, false, mode)
        };
        let q = (i64::from(qx), i64::from(qy), i64::from(o.pos.z));
        // spec: S-0113 claim 13 (h = ½, S-0614 F2's strength, the host's weight)
        self.push(w, host, q, DRONE_STRENGTH, true);
    }

    /// A damaging overload pulse sets its controller's roll rate (S-0113
    /// claim 17, S-0601 F4), replacing it, unscaled.
    fn overload(&mut self, w: &World, controller: ObjectId, level: i32, draw: i32) {
        let Some(o) = w.obj(controller) else { return };
        if !kept(o.kind) {
            return;
        }
        // spec: S-0601 F4 (0.015 radians per tick at level 1, 0.03 from level 2)
        let magnitude = if level == 1 {
            OVERLOAD_RATE_LOW
        } else {
            OVERLOAD_RATE_HIGH
        };
        // spec: S-0601 F4 (−magnitude when r < 50, otherwise +magnitude; exact)
        let rate = if draw < OVERLOAD_SIGN_DRAW {
            F32(magnitude.0 ^ 0x8000_0000) // spec: S-0205 (negation flips the sign bit, exact)
        } else {
            magnitude
        };
        // spec: S-0113 claims 10 and 17 (the overload sets the roll rate, replacing it, unscaled by the weight)
        self.bodies.entry(controller).or_default().roll_rate = rate;
    }

    /// Sets the rates of body `id` for a push from `q` with strength `s`
    /// (S-0113 claims 1 to 4); `half` is the pitch factor ½.
    fn push(&mut self, w: &World, id: ObjectId, q: (i64, i64, i64), s: F32, half: bool) {
        let Some(o) = w.obj(id) else { return };
        if !kept(o.kind) {
            return;
        }
        let r = w.rules();
        let p = Push {
            body: (i64::from(o.pos.x), i64::from(o.pos.y), i64::from(o.pos.z)),
            from: q,
            strength: s,
            half,
            weight: weight_of(w, id),
            facing: o.body.value(w.tick),
        };
        // spec: S-0113 F1 and Changes for other specs (S-0204's quirks.table_root_distance covers d, ℓ and m; quirks.table_direction covers θ)
        let Some((pitch, roll)) = push_rates(
            &p,
            r.quirks.table_root_distance,
            r.quirks.table_direction,
            r.mode,
        ) else {
            // spec: S-0113 claim 2 (a push that changes nothing leaves the earlier rates)
            return;
        };
        // spec: S-0113 claim 2 (a push replaces both rates; the angles keep their values)
        let b = self.bodies.entry(id).or_default();
        b.pitch_rate = pitch;
        b.roll_rate = roll;
    }

    /// Moves every rocking body to the game's current tick, one step per
    /// tick since the last call, whatever the drawing's frame rate.
    pub fn advance(&mut self, w: &World) {
        let now = w.tick;
        // design: a jump of many ticks (a load or a seek) steps at most MAX_CATCH_UP ticks
        let steps = self.last.map_or(1, |l| (now - l).clamp(0, MAX_CATCH_UP));
        self.last = Some(now);
        for i in 0..steps {
            self.step(w, now - (steps - 1 - i));
        }
    }

    /// One turn of every body, in tick `t` (S-0113 table S).
    fn step(&mut self, w: &World, t: i32) {
        let mode = w.rules().mode;
        let escape = w.rules().quirks.rocking_limit_escape;
        let leans = &mut self.leans;
        self.bodies.retain(|&id, b| {
            let Some(o) = w.obj(id) else { return false };
            if o.sw.warp.as_ref().is_some_and(|m| {
                !matches!(m.stage, glasnost_sim::superweapons::warp::Stage::Hold(_))
            }) {
                // spec: S-0113 table S and claim 5 (no rocking step during a superweapon warp from the first pending update through the arrival effects; again in arrival hold)
                return true;
            }
            if glasnost_sim::naval::sinking(w, id) {
                if !leans.contains_key(&id) && !b.pitch.to_f64().eq_num(F64::ZERO) {
                    // spec: S-0504 claim 15 (Starting value: the lean steps from the pitch the vehicle had when sinking began, left over from rocking)
                    leans.insert(id, b.pitch);
                }
                // spec: S-0113 claim 5 and table S (sinking: the pitch takes the lean, its rate is left as it is; the roll takes table R's roll step)
                (b.roll, b.roll_rate) =
                    axis_step(b.roll, b.roll_rate, USUAL_LIMIT, true, escape, mode);
                return !b.level();
            }
            if glasnost_sim::jump::crashing(w, id) || glasnost_sim::air::crashing(w, id) {
                // spec: S-0113 claim 16 and table S (crashing: each angle adds its rate, rounded to single, with no limit, pull, stop or destruction)
                b.roll = b.roll.to_f64().add(b.roll_rate.to_f64(), mode).to_f32(mode);
                b.pitch = b
                    .pitch
                    .to_f64()
                    .add(b.pitch_rate.to_f64(), mode)
                    .to_f32(mode);
                if glasnost_sim::jump::balloon_hover_type(w, id) {
                    // spec: S-0113 claim 16 (balloon hover: the pitch raised to −π/4 when below it, the roll held within ±π/4)
                    if b.pitch.to_f64().lt(MINUS_QUARTER.to_f64()) {
                        b.pitch = MINUS_QUARTER;
                    }
                    let l = USUAL_LIMIT.to_f64();
                    if b.roll.to_f64().gt(l) {
                        b.roll = USUAL_LIMIT;
                    } else if b.roll.to_f64().lt(l.negated()) {
                        b.roll = MINUS_QUARTER;
                    }
                }
                return !b.level();
            }
            // spec: S-0113 claim 6 (the pitch limit π/10 while in S-0210's crush-slowdown state, π/4 otherwise; the roll limit π/4)
            let pitch_limit = if o.crush_slowdown {
                SLOWED_PITCH_LIMIT
            } else {
                USUAL_LIMIT
            };
            // spec: S-0113 table S (table R for the roll, then for the pitch)
            (b.roll, b.roll_rate) = axis_step(b.roll, b.roll_rate, USUAL_LIMIT, true, escape, mode);
            (b.pitch, b.pitch_rate) =
                axis_step(b.pitch, b.pitch_rate, pitch_limit, false, escape, mode);
            // spec: S-0113 claim 9 (after both angles have stepped, a roll or pitch beyond ±π destroys the body); design: the simulation keeps no rocking yet (a seam), so the view drops the body and draws it level
            if b.over() {
                return false;
            }
            !b.level()
        });
        leans.retain(|&id, p| {
            let Some(n) = w.obj(id).and_then(|o| o.naval.as_ref()) else {
                return false;
            };
            if !n.sinking {
                return false;
            }
            if let Some(lean) = n.lean {
                // spec: S-0504 claim 15 and F4 (one step per descent, the simulation's kept direction)
                *p = glasnost_sim::naval::pitch_step(*p, lean, mode);
            }
            true
        });
        self.grapple_step(w, t, mode);
    }

    /// The grappling parasites' roll writes in tick `t`, after the hosts'
    /// rocking steps (S-0113 claim 14, F4, State and ordering).
    fn grapple_step(&mut self, w: &World, t: i32, mode: Rounding) {
        let hosts: Vec<ObjectId> = w
            .objects
            .values()
            .filter(|o| kept(o.kind))
            .filter(|o| {
                glasnost_sim::parasite::parasite_of(w, o.id)
                    .is_some_and(|p| glasnost_sim::parasite::grappling(w, p))
            })
            .map(|o| o.id)
            .collect();
        for &h in &hosts {
            if let std::collections::btree_map::Entry::Vacant(e) = self.grapples.entry(h) {
                // spec: S-0113 claim 14 (the roll is set to 0 in the turn the grab starts)
                e.insert(Grapple::default());
                if let Some(b) = self.bodies.get_mut(&h) {
                    b.roll = F32::ZERO;
                }
            }
        }
        let bodies = &mut self.bodies;
        self.grapples.retain(|h, g| {
            if !hosts.contains(h) {
                // spec: S-0113 claim 14 (the roll is set to 0 when the parasite leaves)
                if let Some(b) = bodies.get_mut(h) {
                    b.roll = F32::ZERO;
                }
                return false;
            }
            let Some((start, negative)) = g.push else {
                // spec: S-0113 claim 14 (in the rest of the grab the roll moves only by the host's own rocking step)
                return true;
            };
            // spec: S-0113 claim 14 (the turn of the way draw is turn 1)
            let turn = t - start + 1;
            if turn <= 1 {
                // spec: S-0113 claim 14 (the phase writes no roll in turn 1)
                return true;
            }
            let roll = if turn >= GRAPPLE_END_TURN {
                // spec: S-0113 claim 14 (in turn 40 the phase ends and the roll is set to 0)
                g.push = None;
                F32::ZERO
            } else {
                // spec: S-0113 claim 14 (k = 0 in turns 2 and 3, then up by 1 every 4 turns: k = ⌊turn / 4⌋) and F4 (roll = RZ₂₄(sinT(k × σ × π/10) × π/4))
                let k = turn / GRAPPLE_FRAME_TURNS;
                let sigma = if negative { -1 } else { 1 };
                let x = F64::from_i64(i64::from(k * sigma), mode).mul(GRAPPLE_STEP, mode);
                trig::sin_t(x, mode)
                    .mul(USUAL_LIMIT.to_f64(), mode)
                    .to_f32(mode)
            };
            // spec: S-0113 claim 14 (it sets the roll directly and never changes the roll rate; the pitch is unaffected)
            let b = bodies.entry(*h).or_default();
            b.roll = roll;
            true
        });
    }

    /// The drawn pitch and roll of `id`, radians, single precision, held to
    /// the drawing step; `None` when it does not rock.
    pub fn angles(&self, id: ObjectId) -> Option<(f32, f32)> {
        let b = self.bodies.get(&id)?;
        Some((drawn(b.pitch), drawn(b.roll)))
    }

    /// The pitch and roll of `id` as they stand, radians, single precision,
    /// not held to the drawing step: a crashing aircraft's tumble, which
    /// S-0113 claim 20 draws fresh with its exact angles. `None` when it has
    /// no body here.
    pub fn exact_angles(&self, id: ObjectId) -> Option<(f32, f32)> {
        let b = self.bodies.get(&id)?;
        Some((f32::from_bits(b.pitch.0), f32::from_bits(b.roll.0)))
    }

    /// The lean of a sinking vessel that was rocked when it began to sink,
    /// if the view steps it (S-0504 claim 15, *Starting value*).
    pub fn lean(&self, id: ObjectId) -> Option<F32> {
        self.leans.get(&id).copied()
    }

    /// Whether nothing rocks.
    pub fn is_empty(&self) -> bool {
        self.bodies.is_empty() && self.grapples.is_empty() && self.leans.is_empty()
    }

    /// The body state of `id`, for tests.
    pub fn body(&self, id: ObjectId) -> Option<Body> {
        self.bodies.get(&id).copied()
    }
}

/// Whether the view keeps the pushes' rocking for an object of kind `k`:
/// vehicles only. S-0113's rocking bodies are every kind drawn as a voxel
/// model, but an aircraft draws its angles only while it crashes (claim
/// 18), from the tumble rates `Effect::CrashTumble` reports (claim 16), and
/// buildings and infantry are drawn from sprites. The angles an aircraft
/// would carry into its crash from earlier pushes are not kept.
// design: S-0113 claim 18 (only vehicles draw the angles outside a crash); a seam for the other kinds
fn kept(k: SimKind) -> bool {
    k == SimKind::Vehicle
}

/// Which entries of one area push are pushed from the source object's
/// position rather than from the detonation point *P* (S-0113 claim 11).
/// `Effect::Rock` gives each body its point but does not say which it is
/// (a seam): a source exists only for the centre cell's bodies, which
/// stand in *P*'s cell, and every other body is pushed from *P*. With one
/// point, it is the source's position when an object other than the
/// pushed bodies stands exactly there (the source object, in play or not);
/// with two, the source's is the one whose bodies all stand in the other's
/// cell. A misreading changes only how far a push point lies along the
/// same direction, so only the push's magnitude.
// design: interim reading of Effect::Rock until it says which point it gives (docs/m8-claims/intake53-S-0107-S-0108-S-0113.md, seams)
fn source_points(w: &World, pushed: &[(ObjectId, Pos)]) -> Vec<bool> {
    let mut points: Vec<Pos> = Vec::new();
    for &(_, from) in pushed {
        if !points.contains(&from) {
            points.push(from);
        }
    }
    let bodies: Vec<ObjectId> = pushed.iter().map(|(id, _)| *id).collect();
    let cell_of = |id: ObjectId| w.obj(id).map(|o| o.pos.cell());
    // A point where an object other than the pushed bodies stands
    // exactly: the source object's position, not P.
    let occupied = |p: Pos| {
        w.objects
            .values()
            .any(|o| o.pos == p && !bodies.contains(&o.id))
    };
    // All bodies of point x stand in cell c.
    let all_in = |x: Pos, c| {
        pushed
            .iter()
            .filter(|(_, f)| *f == x)
            .all(|(id, _)| cell_of(*id) == Some(c))
    };
    let source = match points.as_slice() {
        // One point: the source's when an object stands exactly at it.
        [f] => occupied(*f).then_some(*f),
        [a, b] => {
            // Two points: the source's bodies all stand in P's cell.
            let a_src = all_in(*a, b.cell());
            let b_src = all_in(*b, a.cell());
            match (a_src, b_src) {
                (true, false) => Some(*a),
                (false, true) => Some(*b),
                (true, true) => match (occupied(*a), occupied(*b)) {
                    (true, false) => Some(*a),
                    (false, true) => Some(*b),
                    _ => None,
                },
                (false, false) => None,
            }
        }
        _ => None,
    };
    pushed.iter().map(|(_, f)| Some(*f) == source).collect()
}

/// A centre-cell body's push point toward a source at `s`: *B* +
/// trunc(10 × *v*) per axis, *v* the three-dimensional unit vector from
/// *B* toward the source (S-0113 claim 11); `None` when the source stands
/// at the body's position.
fn toward(w: &World, b: (i64, i64, i64), s: (i64, i64, i64)) -> Option<(i64, i64, i64)> {
    let mode = w.rules().mode;
    let u = (s.0 - b.0, s.1 - b.1, s.2 - b.2);
    let sq = |v: i64| u128::from(v.unsigned_abs()) * u128::from(v.unsigned_abs());
    let n = sq(u.0) + sq(u.1) + sq(u.2);
    if n == 0 {
        return None;
    }
    // design: S-0113 Open questions (the root and precision of v are open): S-0204's table root under quirks.table_root_distance, the components in double precision, each truncated
    let len = if w.rules().quirks.table_root_distance {
        geom::table_root(u64::try_from(n).unwrap_or(u64::MAX), mode).to_f64()
    } else {
        exact_root(F64::from_parts(false, n, 0, mode), mode)
    };
    let step = |v: i64| {
        let ten = F64::from_i64(SOURCE_DISTANCE, mode);
        i64::from(
            F64::from_i64(v, mode)
                .mul(ten, mode)
                .div(len, mode)
                .to_i32_common(),
        )
    };
    // spec: S-0113 claim 11 (Q = B + trunc(10 × v) leptons, per axis)
    Some((b.0 + step(u.0), b.1 + step(u.1), b.2 + step(u.2)))
}

/// The weight *W* that divides a body's pushes (S-0113 claims 3 and 10): a
/// vehicle type's imported weight, held as a double, 1.0 when absent; 1.0
/// for other kinds, whose weight our importer does not read yet (a seam).
fn weight_of(w: &World, id: ObjectId) -> F64 {
    let one = F64::from_i64(1, Rounding::TowardZero);
    let Some(o) = w.obj(id) else { return one };
    // spec: S-0113 claim 10 and Import mapping (1.0 at type creation when the key is absent); S-0504 claim 11; Edge cases (a weight of 0 or below is used as it is)
    w.rules()
        .unit(o.kind, o.type_index)
        .filter(|_| o.kind == SimKind::Vehicle)
        .and_then(|u| u.vehicle.weight)
        .unwrap_or(one)
}
