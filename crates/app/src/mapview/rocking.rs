//! Body rocking (S-0303 claim 20, S-0614 claims 9 and 10, S-0601 F4,
//! S-0210 claim 12, S-0504 claim 15): presentation state the view keeps beside the game.
//!
//! The simulation reports what rocks (`Effect::Rock`, `Effect::DroneRock`,
//! `Effect::GrapplePush`, `Effect::OverloadRock`) and keeps none of it: S-0303 claim 20 believes
//! rocking does not reach game state, and S-0504 claim 16 allows an engine
//! to hold the angles as presentation state. The view sets each rocked
//! body's pitch and roll rates, and moves the angles once per game tick
//! (never per drawn frame), so a rock looks the same at any frame rate.
//!
//! **Interim (our design, M8 decision of 2026-10-25; Q-2081).** No approved
//! spec says how a strength becomes rates, how the rates move the angles
//! back to level, how the type's weight scales them, or the sideways limit
//! (S-0504 Open questions). The rules here are ours, marked `design:`:
//! a hit sets both rates to [`RATE_PER_STRENGTH`] × strength ÷ weight,
//! split between pitch and roll by the push's direction relative to the
//! body; each tick the angle adds its rate and then a damped spring
//! ([`SPRING`], [`DAMPING`]) pulls the rate back towards level.
//!
//! Only drawing reads any of this. A seam S-0504 claim 16 names is left: the
//! passengers of an open-topped transport would fire from the rolled body
//! of a sinking transport, but S-0313 claim 5's fire point does not take the
//! body transform, so no gameplay rule reads the roll.

use std::collections::BTreeMap;
use std::f64::consts::{FRAC_PI_4, PI};

use glasnost_num::F32;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::rules::Kind as SimKind;
use glasnost_sim::world::{ObjectId, Pos, World};

/// The forward rocking angle's usual limit, radians (S-0210 claim 12).
pub const FORWARD_LIMIT: f64 = FRAC_PI_4; // spec: S-0210 claim 12 and Constants (π/4 otherwise)
/// The forward rocking angle's limit while crush-slowed, radians.
pub const SLOWED_FORWARD_LIMIT: f64 = PI / 10.0; // spec: S-0210 claim 12 and Constants (π/10 in the crush-slowdown state)
/// The sideways rocking angle's limit, radians. S-0210 claim 12 says only
/// that it keeps its usual limit, which no spec states.
// design: Q-2081 question 4 (interim: the same π/4 as the forward angle's usual limit)
pub const SIDEWAYS_LIMIT: f64 = FRAC_PI_4;

/// A rocking strength's factor per damage point (S-0303 claim 20).
const STRENGTH_PER_DAMAGE: f64 = 0.01; // spec: S-0303 Constants (rocking strength factor 0.01 per damage point)
/// The rocking strength's cap (S-0303 claim 20).
const STRENGTH_CAP: f64 = 4.0; // spec: S-0303 Constants (rocking strength cap 4.0)
/// A drone parasite's rocking strength (S-0614 claim 9, F2).
pub const DRONE_STRENGTH: f64 = 1.5; // spec: S-0614 Constants (drone rock strength 1.5)
/// The drone's rocking point's displacement from the host, leptons
/// (S-0614 F2).
const DRONE_DISPLACEMENT: i32 = 2; // spec: S-0614 Constants (drone rock displacement 2 leptons)

/// The rate one unit of strength sets, radians per tick.
// design: Q-2081 question 1 (interim, chosen for play at 30 ticks a second: the cap of 4.0 rocks an unweighted body about 0.45 radians, the least rocking strength above 0.3 about 0.035, a drone's 1.5 about 0.17)
pub const RATE_PER_STRENGTH: f64 = 0.05;
/// The spring's pull towards level per tick: ω² for ω = 2π / 18 ticks.
// design: Q-2081 question 2 (interim: a swing period of 18 ticks, 0.6 seconds of game time at 30 ticks a second)
pub const SPRING: f64 = (2.0 * PI / 18.0) * (2.0 * PI / 18.0);
/// The damping per tick: 2ζω with ζ = 0.2.
// design: Q-2081 question 2 (interim: damping ratio 0.2; the cap's rock settles below S-0106's 0.005 radians in about 60 ticks, two seconds of game time)
pub const DAMPING: f64 = 2.0 * 0.2 * (2.0 * PI / 18.0);
/// Below this magnitude an angle and its rate count as settled, radians
/// (and radians per tick), and the body leaves the rocking list.
// design: half of S-0106's rocking threshold (0.005 radians), so a settled body is drawn level well before its entry goes
const SETTLED: f64 = 0.0025;
/// The step the drawn angles are held to, radians: S-0106's rocking
/// threshold, so a drawn angle rocks exactly when the held one is at least
/// 0.005 radians (S-0106 Terms), and a body is drawn under few distinct
/// angles (each pair is an image of its own, S-0105 claim 26).
// spec: S-0106 Terms (rocking: a pitch or roll of 0.005 radians or more); design: S-0303 claim 20 (the strength's precision is not fixed), so the drawn angles are held to that step, truncated toward level
const DRAWN_STEP: f32 = glasnost_render::voxel::ROCKING_THRESHOLD;

/// The most ticks one [`Rocking::advance`] catches up.
// design: longer than any rock lasts (about 60 ticks from the strength cap)
const MAX_CATCH_UP: i32 = 120;

/// A level-1 overload pulse's sideways rocking rate, radians per tick
/// (S-0601 F4), the nearest single-precision value.
const OVERLOAD_RATE_LOW: f32 = 0.015; // spec: S-0601 F4 and Constants (rocking magnitude, level 1: 0.015 radians per tick, the nearest binary32)
/// The rate from level 2 on (S-0601 F4).
const OVERLOAD_RATE_HIGH: f32 = 0.03; // spec: S-0601 F4 and Constants (rocking magnitude, level ≥ 2: 0.03 radians per tick, the nearest binary32)
/// The draw from which an overload's rate is positive (S-0601 F4).
const OVERLOAD_SIGN_DRAW: i32 = 50; // spec: S-0601 F4 and Constants (rocking sign threshold 50)

/// A grappling parasite's push: 10 frames of 4 ticks (S-0614 F3).
const PUSH_FRAME_TICKS: i32 = 4; // spec: S-0614 F3 and Constants (push of 10 frames × 4 ticks)
/// The frames of one push (S-0614 F3).
const PUSH_FRAMES: i32 = 10; // spec: S-0614 F3 (push of 10 frames × 4 ticks)

/// The rocking strength of a detonation's damage amount *D*:
/// min(*D* × 0.01, 4.0), formed in double precision (S-0303 claim 20).
pub fn strength(damage: i32) -> f64 {
    // spec: S-0303 claim 20 (strength min(D × 0.01, 4.0), double precision from the integer D)
    (f64::from(damage) * STRENGTH_PER_DAMAGE).min(STRENGTH_CAP)
}

/// One rocking body's angles and rates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Body {
    /// The pitch, radians; positive lowers the bow (S-0504 Terms).
    pub pitch: f64,
    /// The roll, radians; positive lowers the body's right side (S-0105's
    /// *Rx* about the forward axis lifts the left, +*y*, side).
    pub roll: f64,
    /// The pitch rate, radians per tick (S-0210's body tilt, S-0504 Terms).
    pub pitch_rate: f64,
    /// The roll rate, radians per tick.
    pub roll_rate: f64,
}

impl Body {
    /// Sets both rates for a push of `strength` towards `push` (a facing,
    /// S-0204) on a body facing `body`, for a type of weight `weight`.
    /// The push lowers the end or side it points to: a push straight ahead
    /// lowers the bow, one to the body's right lowers its right side.
    pub fn push(&mut self, body: u16, push: u16, strength: f64, weight: f64) {
        // spec: S-0204 F5 (the signed facing difference, positive clockwise: towards the body's right)
        let rel = f64::from(glasnost_num::geom::delta(body, push)) * (2.0 * PI / 65536.0); // spec: S-0204 Terms (65536 facing units per turn)
        // design: Q-2081 questions 1 and 3 (interim: the rates are set, replacing what they were, as S-0303 claim 20 "sets" them; the strength is divided by the type's weight, so a heavier hull rocks less)
        let rate = RATE_PER_STRENGTH * strength / weight;
        self.pitch_rate = rate * rel.cos();
        self.roll_rate = rate * rel.sin();
    }

    /// One tick: each angle adds its rate and stops at its limit with its
    /// rate level (S-0210 claim 12), then the spring pulls the rate back
    /// towards level. While `sinking`, the pitch belongs to the lean
    /// (S-0504 claim 15): the forward rate is not added and the pitch does
    /// not return to level; the roll is left free.
    pub fn step(&mut self, forward_limit: f64, sinking: bool) {
        if sinking {
            // spec: S-0504 claim 15 (while sinking the forward rocking rate is not added to the pitch, and the pitch does not return to level)
            self.pitch_rate = 0.0;
        } else {
            axis(&mut self.pitch, &mut self.pitch_rate, forward_limit);
        }
        // spec: S-0504 claim 15 (sinking adds no rule of its own for the roll and does not hold it)
        axis(&mut self.roll, &mut self.roll_rate, SIDEWAYS_LIMIT);
    }

    /// Whether every angle and rate is settled.
    pub fn settled(&self) -> bool {
        [self.pitch, self.roll, self.pitch_rate, self.roll_rate]
            .iter()
            .all(|v| v.abs() < SETTLED)
    }
}

/// One angle's tick.
fn axis(angle: &mut f64, rate: &mut f64, limit: f64) {
    // spec: S-0210 Terms (the angle is what the rate per tick builds up)
    *angle += *rate;
    if angle.abs() >= limit {
        // spec: S-0210 claim 12 (when the angle reaches the limit it stays at the limit and the rate becomes level)
        *angle = limit.copysign(*angle);
        *rate = 0.0;
    }
    // design: Q-2081 question 2 (interim return to level: a damped spring on the rate)
    *rate -= SPRING * *angle + DAMPING * *rate;
}

/// A drawn angle held to the drawing step, truncated toward level.
pub fn drawn(angle: f64) -> f32 {
    let steps = (angle / f64::from(DRAWN_STEP)).trunc();
    if steps == 0.0 {
        // Level, never a negative zero (its bits would key images apart).
        return 0.0;
    }
    steps as f32 * DRAWN_STEP
}

/// The view's rocking state.
#[derive(Clone, Debug, Default)]
pub struct Rocking {
    /// The bodies that rock, by object.
    bodies: BTreeMap<ObjectId, Body>,
    /// Each grappled host's current push: its first tick and whether its
    /// way is the positive one (S-0614 claim 10).
    pushes: BTreeMap<ObjectId, (i32, bool)>,
    /// The lean of a sinking vessel that was rocked when it began to sink
    /// (S-0504 claim 15, *Starting value*): its pitch, stepped here from
    /// the rocked pitch at each descent by F4, in place of the
    /// simulation's, which starts from level.
    leans: BTreeMap<ObjectId, F32>,
    /// The last tick moved.
    last: Option<i32>,
}

impl Rocking {
    /// Takes one tick's rocking events (the tick shown is `shown`).
    pub fn take(&mut self, w: &World, effects: &[Effect], shown: i32) {
        for e in effects {
            match e {
                Effect::Rock { damage, pushed } => {
                    let s = strength(*damage);
                    for &(id, from) in pushed {
                        self.rock_from(w, id, from, s);
                    }
                }
                Effect::DroneRock { host, side } => self.drone(w, *host, *side),
                Effect::OverloadRock {
                    controller,
                    level,
                    draw,
                } => self.overload(w, *controller, *level, *draw),
                Effect::GrapplePush { host, way } => {
                    // design: Q-2081 question 7 (interim: the way's draw 1 rocks towards +π/10 per frame, 0 towards −π/10)
                    self.pushes.insert(*host, (shown, *way == 1));
                }
                _ => {}
            }
        }
    }

    /// Rocks body `id` away from `from` with `strength` (S-0303 claim 20).
    fn rock_from(&mut self, w: &World, id: ObjectId, from: Pos, strength: f64) {
        let Some(o) = w.obj(id) else { return };
        let (dx, dy) = (
            i64::from(o.pos.x) - i64::from(from.x),
            i64::from(o.pos.y) - i64::from(from.y),
        );
        if dx == 0 && dy == 0 {
            // design: Q-2081 question 5 (interim: a body exactly at the point it is pushed from has no direction and does not rock)
            return;
        }
        // spec: S-0303 claim 20 (pushed away from the point); S-0204 F3 (the direction of the body from it)
        let push = glasnost_num::geom::direction(dx, dy, false);
        self.rock(w, id, push, strength);
    }

    /// A drone parasite's delivery to `host` (S-0614 claim 9, F2): the
    /// rocking point lies 2 leptons to one side, perpendicular to the
    /// host's facing, and the host is pushed away from it with strength 1.5.
    fn drone(&mut self, w: &World, host: ObjectId, side: i32) {
        let Some(o) = w.obj(host) else { return };
        let facing = o.body.value(w.tick);
        // spec: S-0614 F2 (s = +2 when the draw gives 0, −2 when it gives 1)
        let s = if side == 0 {
            DRONE_DISPLACEMENT
        } else {
            -DRONE_DISPLACEMENT
        };
        // design: Q-2081 question 6 (interim: +s lies to the host's right, a quarter turn clockwise; the push is away from the point, so to the other side)
        let quarter = 16384; // spec: S-0204 Terms (a quarter turn is 16384 facing units)
        let push = if s > 0 {
            facing.wrapping_sub(quarter)
        } else {
            facing.wrapping_add(quarter)
        };
        self.rock(w, host, push, DRONE_STRENGTH);
    }

    /// A damaging overload pulse sets its controller's sideways rocking
    /// rate (S-0601 claim 16, F4); the pitch rate is left as it was.
    fn overload(&mut self, w: &World, controller: ObjectId, level: i32, draw: i32) {
        let Some(o) = w.obj(controller) else { return };
        // design: only vehicles are kept, as in `rock`
        if o.kind != SimKind::Vehicle {
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
            -magnitude
        } else {
            magnitude
        };
        // spec: S-0601 claim 16 (the sideways body-rocking rate takes the F4 value: set, not added, and given as a rate, not scaled by weight); S-0504 Terms (the roll's sense)
        self.bodies.entry(controller).or_default().roll_rate = f64::from(rate);
    }

    /// Sets the rates of body `id` for a push towards facing `push`.
    fn rock(&mut self, w: &World, id: ObjectId, push: u16, strength: f64) {
        let Some(o) = w.obj(id) else { return };
        // design: S-0106 claim 9 and S-0504 claim 16 (rocking is drawn for the driving, ship, hovering and jumpjet locomotors, which move vehicles); an aircraft's body is drawn by its flight attitude (S-0105 claims 41, 42), and buildings and infantry from sprites, so only vehicles are kept
        if o.kind != SimKind::Vehicle {
            return;
        }
        let weight = weight_of(w, id);
        self.bodies
            .entry(id)
            .or_default()
            .push(o.body.value(w.tick), push, strength, weight);
    }

    /// Moves every rocking body to the game's current tick, one step per
    /// tick since the last call, whatever the drawing's frame rate.
    pub fn advance(&mut self, w: &World) {
        let now = w.tick;
        // design: a jump of many ticks (a load or a seek) steps at most MAX_CATCH_UP ticks; every rock has settled long before
        let steps = self.last.map_or(1, |l| (now - l).clamp(0, MAX_CATCH_UP));
        self.last = Some(now);
        for _ in 0..steps {
            self.step(w);
        }
    }

    /// One tick of every body.
    fn step(&mut self, w: &World) {
        let mode = w.rules().mode;
        self.bodies.retain(|&id, b| {
            let Some(o) = w.obj(id) else { return false };
            let sinking = glasnost_sim::naval::sinking(w, id);
            // spec: S-0210 claim 12 (π/10 while in the crush-slowdown state, whatever the locomotor; π/4 otherwise)
            let limit = if o.crush_slowdown {
                SLOWED_FORWARD_LIMIT
            } else {
                FORWARD_LIMIT
            };
            b.step(limit, sinking);
            !b.settled()
        });
        let leans = &mut self.leans;
        let bodies = &mut self.bodies;
        for (&id, b) in bodies.iter_mut() {
            if glasnost_sim::naval::sinking(w, id) && !leans.contains_key(&id) && b.pitch != 0.0 {
                // spec: S-0504 claim 15 (Starting value: steps start from the pitch the vehicle had when sinking began, left over from earlier rocking)
                leans.insert(id, F32((b.pitch as f32).to_bits()));
                b.pitch = 0.0;
            }
        }
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
        self.pushes.retain(|&host, &mut (start, _)| {
            // spec: S-0614 claim 10 (the push rocks the host while it still has its grappling parasite, 10 frames of 4 ticks, 0 at the strike)
            glasnost_sim::parasite::parasite_of(w, host).is_some()
                && w.tick - start <= PUSH_FRAMES * PUSH_FRAME_TICKS
        });
    }

    /// The drawn pitch and roll of `id` in tick `shown`, radians, single
    /// precision, held to the drawing step; `None` when it does not rock.
    pub fn angles(&self, id: ObjectId, shown: i32) -> Option<(f32, f32)> {
        let body = self.bodies.get(&id).copied().unwrap_or_default();
        let grapple = self.pushes.get(&id).map_or(0.0, |&(start, positive)| {
            // spec: S-0614 claim 10 (the tilt sin(k × (±π/10)) × π/4 at frame k of the push, 0 at the strike)
            let k = (shown - start) / PUSH_FRAME_TICKS;
            if (0..PUSH_FRAMES).contains(&k) {
                let way = if positive { PI / 10.0 } else { -PI / 10.0 }; // spec: S-0614 claim 10 (±π/10 per frame)
                (f64::from(k) * way).sin() * FRAC_PI_4 // spec: S-0614 claim 10 (× π/4)
            } else {
                0.0
            }
        });
        if !self.bodies.contains_key(&id) && grapple == 0.0 {
            return None;
        }
        // design: Q-2081 question 7 (interim: a squid's push is the sideways angle, S-0504 claim 15, added to any other rocking and held to the sideways limit)
        let roll = (body.roll + grapple).clamp(-SIDEWAYS_LIMIT, SIDEWAYS_LIMIT);
        Some((drawn(body.pitch), drawn(roll)))
    }

    /// The lean of a sinking vessel that was rocked when it began to sink,
    /// if the view steps it (S-0504 claim 15, *Starting value*).
    pub fn lean(&self, id: ObjectId) -> Option<F32> {
        self.leans.get(&id).copied()
    }

    /// Whether nothing rocks.
    pub fn is_empty(&self) -> bool {
        self.bodies.is_empty() && self.pushes.is_empty() && self.leans.is_empty()
    }

    /// The body state of `id`, for tests.
    pub fn body(&self, id: ObjectId) -> Option<Body> {
        self.bodies.get(&id).copied()
    }
}

/// The weight that scales a body's rocking (S-0303 claim 20): a vehicle
/// type's imported weight (S-0504 claim 11 and Import mapping, 1.0 when
/// absent); 1.0 for other kinds, which import no weight.
fn weight_of(w: &World, id: ObjectId) -> f64 {
    let Some(o) = w.obj(id) else { return 1.0 };
    // spec: S-0504 claim 11 (an absent vehicle weight is exactly 1.0)
    let weight = w
        .rules()
        .unit(o.kind, o.type_index)
        .filter(|_| o.kind == SimKind::Vehicle)
        .and_then(|u| u.vehicle.weight)
        .map_or(1.0, |x| f64::from_bits(x.0));
    // design: Q-2081 question 3 (interim: a weight that is not a positive number scales as 1.0)
    if weight.is_finite() && weight > 0.0 {
        weight
    } else {
        1.0
    }
}
