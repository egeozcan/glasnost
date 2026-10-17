//! Gas clouds (S-0613): the shared gas system, the warhead-made particle
//! systems, the virus-death clouds and the gas particle's update, movement
//! and damage.
//!
//! A particle system is a world object in the update order
//! ([`GasWorld::systems`]): it takes one turn per tick (S-0203 claim 2) in
//! which its particles update and move (claim 6). Each particle has an
//! object number from the world's counter (S-0306 Terms), which only F8's
//! parity reads.
//!
//! The virus-death animation of claim 2 needs its middle event, which comes
//! later in playback (S-0109 claim 14, μ ≠ 0). Animations have no playback
//! in the simulation otherwise, so this module keeps a narrow record for
//! that animation only ([`VirusAnim`]): one forward pass, the stage delay of
//! S-0109 claims 2 to 4, the first-update rule of claim 5, the middle event
//! of claim 14 and the end of claim 16. Loops, reversed and ping-pong play,
//! follow-up types and loop delays (S-0109 claims 6 to 9, 11 and 15) are a
//! seam, as for the storm's clouds (S-0513 claim 46).
//!
//! Provisional rules (requests): the shared system enters play before the
//! map is read, so it is first in the update order (Q-1240); the order
//! within one update follows the claim numbers (Q-1241); a new particle's
//! remaining lifetime is the drawn value itself (Q-1242); a warhead's system
//! of another behaviour kind is not kept (Q-1243); a system's lifetime
//! counts down at the end of its turn, only when positive (Q-1244).
//!
//! Seams: a particle of another behaviour kind in a gas system only counts
//! its lifetime down (its own update and movement belong to other specs);
//! the drawing of claim 16 belongs to the view.

use glasnost_map::CellXY;
use glasnost_num::{F32, F64, Rounding};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::combat::damage::Hit;
use crate::combat::data::WarheadId;
use crate::rules::{AnimRules, Kind, ParticleRules};
use crate::world::{HouseId, Layer, ObjectId, Pos, World};

/// The engine's fixed name of the shared gas system's type (Import
/// mapping).
const SHARED_SYSTEM: &str = "GasCloudSys"; // spec: S-0613 Import mapping (the particle-system type named GasCloudSys)

/// Hover height above the terrain, leptons (F4, F5).
const HOVER: i32 = 5; // spec: S-0613 Constants (hover height 5 leptons)

/// The most a particle sinks in one odd tick, leptons (F5).
const SINK_STEP: i32 = 2; // spec: S-0613 Constants (sink step 2 leptons)

/// The fall limit of the vertical speed, leptons per tick (F4).
const FALL_LIMIT: i32 = -5; // spec: S-0613 Constants (fall limit −5 leptons per tick)

/// The drift clamp, leptons per tick (F2).
const DRIFT_CLAMP: i32 = 2; // spec: S-0613 Constants (drift clamp ±2 leptons per tick)

/// The probe depth above gravity, leptons (F3: 2 + G).
const PROBE_BASE: i32 = 2; // spec: S-0613 Constants (probe depth 2 + gravity)

/// The obstacle band above the terrain, leptons (F3).
const OBSTACLE_BAND: i32 = 150; // spec: S-0613 Constants (obstacle band 150 leptons)

/// The bridge deck above the terrain, leptons (F3).
const DECK: i32 = 416; // spec: S-0613 Constants (bridge deck height 416 leptons, as in S-0102)

/// The wind cadence numerator, ticks (F6).
const WIND_CADENCE: i32 = 10; // spec: S-0613 Constants (wind cadence numerator 10 ticks)

/// The wind table, (Δₓ, Δᵧ) by direction 0 … 7, with direction 3 short.
// spec: S-0613 Constants (wind table Δx 0, 2, 2, 1, 0, −2, −2, −2 and Δy −2, −2, 0, 2, 2, 2, 0, −2)
const WIND: [(i32, i32); 8] = [
    (0, -2),
    (2, -2),
    (2, 0),
    (1, 2),
    (0, 2),
    (-2, 2),
    (-2, 0),
    (-2, -2),
];

/// Δₓ of direction 3 with `quirks.wind_southeast_short` off.
const WIND_3_FULL: i32 = 2; // spec: S-0613 Edge cases (quirks.wind_southeast_short off: Δx = 2 for direction 3)

/// The railgun lifetime modulus (S-0202 claim 15).
const RAILGUN_MODULUS: i32 = 10; // spec: S-0202 claim 15 (modulo 10 for railgun behaviour)

/// The wall overlays that are obstacles in a cell without a building (claim
/// 10, by position in the imported overlay list).
const WALLS: [u8; 3] = [2, 26, 243]; // spec: S-0613 claim 10 and S-0306 Constants (wall overlays 2, 26, 243)

/// A behaviour kind of a particle or particle-system type (Import mapping).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Behaviour {
    /// Gas.
    Gas,
    /// Smoke.
    Smoke,
    /// Fire.
    Fire,
    /// Spark.
    Spark,
    /// Railgun.
    Railgun,
}

fn behaviour(name: Option<&str>) -> Option<Behaviour> {
    // spec: S-0613 Import mapping (BehavesLike: Gas, Smoke, Fire, Spark, Railgun; absent none)
    match name? {
        "gas" => Some(Behaviour::Gas),
        "smoke" => Some(Behaviour::Smoke),
        "fire" => Some(Behaviour::Fire),
        "spark" => Some(Behaviour::Spark),
        "railgun" => Some(Behaviour::Railgun),
        _ => None,
    }
}

/// A particle type's fields (S-0613 Import mapping, S-0202 claim 15).
#[derive(Clone, Debug)]
pub struct ParticleType {
    /// Type name.
    pub name: String,
    /// The behaviour kind.
    pub kind: Option<Behaviour>,
    /// The creation-draw fields (S-0202 claim 15).
    pub draw: ParticleRules,
    /// The lifetime field *E*ₘₐₓ as imported (F8).
    pub max_ec: i32,
    /// The damage interval reduced modulo 65536 (claim 5, F7).
    pub max_dc: u16,
    /// Damage per hit before damage resolution.
    pub damage: i32,
    /// The warhead of its hits.
    pub warhead: Option<WarheadId>,
    /// Start, end and stage-advance fields as signed bytes (F8).
    pub start_stage: i8,
    /// End stage.
    pub end_stage: i8,
    /// Stage advance *A*.
    pub advance: i8,
    /// The delete-at-stage-limit flag (claim 15).
    pub delete_at_limit: bool,
    /// The next particle type, by registration index (claim 7).
    pub next: Option<u32>,
    /// The next-particle displacement, leptons.
    pub next_offset: [i32; 3], // nocite: vector and matrix dimensions (three axes)
    /// The wind effect *W* (F6).
    pub wind_effect: i32,
    /// Translucency reduced modulo 256 (claim 16; the drawing reads it).
    pub translucency: i8,
}

/// A particle-system type's fields (S-0613 Import mapping).
#[derive(Clone, Debug)]
pub struct SystemType {
    /// Type name.
    pub name: String,
    /// The behaviour kind.
    pub kind: Option<Behaviour>,
    /// The held particle type, by registration index.
    pub holds: Option<u32>,
    /// The system lifetime, ticks (−1: never times out).
    pub lifetime: i32,
}

/// The gas rules.
#[derive(Clone, Debug, Default)]
pub struct GasRules {
    /// Particle types by registration index.
    pub particles: Vec<ParticleType>,
    /// Particle-system types by registration index.
    pub systems: Vec<SystemType>,
    /// The shared gas system's type (claim 1); `None` only for rules that
    /// list no particle-system types at all.
    pub shared: Option<u32>,
    /// The virus-death animation (claims 2 and 3).
    pub virus: Option<String>,
    /// The wind push (Δₓ, Δᵧ) of the rules' wind direction; `None` outside
    /// 0 … 7 (Edge cases).
    pub wind: Option<(i32, i32)>,
    /// Gravity *G*, leptons per tick² (F3).
    pub gravity: i32,
    /// The ramp matrices of S-0106 table C4 by slope shape 0 … 20.
    pub ramps: Vec<[[F32; 3]; 3]>, // nocite: vector and matrix dimensions (three axes)
}

/// The ramp matrices of S-0106 table C4, rows top to bottom, for shapes
/// 1 … 8 and 13 … 16.
// spec: S-0106 Constants table C4 (original tilt transforms; shapes 9 … 12 equal 5 … 8; 0 and 17 … 20 the identity)
const C4: [(u8, [&str; 9]); 12] = [
    (
        1,
        [
            "0.8719697",
            "-0.00133758574",
            "-0.489557832",
            "0.00153398002",
            "0.999998808",
            "1.19903125e-16",
            "0.489557236",
            "-0.000750971958",
            "0.871970773",
        ],
    ),
    (
        2,
        [
            "0.999998808",
            "-0.00153398002",
            "5.99515626e-17",
            "0.00133758574",
            "0.8719697",
            "0.489557832",
            "-0.000750971958",
            "-0.489557236",
            "0.871970773",
        ],
    ),
    (
        3,
        [
            "0.8719697",
            "-0.00133758574",
            "0.489557832",
            "0.00153398002",
            "0.999998808",
            "-5.99515626e-17",
            "-0.489557236",
            "0.000750971958",
            "0.871970773",
        ],
    ),
    (
        4,
        [
            "1",
            "0",
            "0",
            "0",
            "0.871970773",
            "-0.489557832",
            "0",
            "0.489557832",
            "0.871970773",
        ],
    ),
    (
        5,
        [
            "0.965714812",
            "0.0328553095",
            "-0.257517487",
            "0.0358179212",
            "0.965609491",
            "0.257517487",
            "0.257122159",
            "-0.257912219",
            "0.931326687",
        ],
    ),
    (
        6,
        [
            "0.965609491",
            "-0.0358179212",
            "0.257517487",
            "-0.0328553095",
            "0.965714812",
            "0.257517487",
            "-0.257912219",
            "-0.257122159",
            "0.931326687",
        ],
    ),
    (
        7,
        [
            "0.965714812",
            "0.0328553095",
            "0.257517487",
            "0.0358179212",
            "0.965609491",
            "-0.257517487",
            "-0.257122159",
            "0.257912219",
            "0.931326687",
        ],
    ),
    (
        8,
        [
            "0.965609491",
            "-0.0358179212",
            "-0.257517487",
            "-0.0328553095",
            "0.965714812",
            "-0.257517487",
            "0.257912219",
            "0.257122159",
            "0.931326687",
        ],
    ),
    (
        13,
        [
            "0.936082423",
            "0.0625787526",
            "-0.346169651",
            "0.0654503107",
            "0.935886025",
            "0.346169651",
            "0.345638216",
            "-0.346700251",
            "0.871970773",
        ],
    ),
    (
        14,
        [
            "0.935886025",
            "-0.0654503107",
            "0.346169651",
            "-0.0625787526",
            "0.936082423",
            "0.346169651",
            "-0.346700251",
            "-0.345638216",
            "0.871970773",
        ],
    ),
    (
        15,
        [
            "0.936082423",
            "0.0625787526",
            "0.346169651",
            "0.0654503107",
            "0.935886025",
            "-0.346169651",
            "-0.345638216",
            "0.346700251",
            "0.871970773",
        ],
    ),
    (
        16,
        [
            "0.935886025",
            "-0.0654503107",
            "-0.346169651",
            "-0.0625787526",
            "0.936082423",
            "-0.346169651",
            "0.346700251",
            "0.345638216",
            "0.871970773",
        ],
    ),
];

/// The single nearest to decimal text (S-0106 C4: "the single-precision
/// numbers nearest to them"), through the nearest double; the table test
/// checks that no entry's double is a midpoint between two singles.
pub fn nearest_single(text: &str) -> F32 {
    crate::rules::nearest_double(text).to_f32(Rounding::Nearest)
}

fn identity() -> [[F32; 3]; 3] { // nocite: vector and matrix dimensions (three axes)
    let one = nearest_single("1");
    let zero = nearest_single("0");
    [[one, zero, zero], [zero, one, zero], [zero, zero, one]]
}

/// The ramp matrices by shape 0 … 20 (S-0106 table C4).
pub fn ramps() -> Vec<[[F32; 3]; 3]> { // nocite: vector and matrix dimensions (three axes)
    let mut out = vec![identity(); 21]; // spec: S-0106 table C4 (ramp shapes 0 … 20)
    for (s, e) in C4 {
        let m = [
            [nearest_single(e[0]), nearest_single(e[1]), nearest_single(e[2])], // nocite: vector and matrix dimensions (three axes)
            [nearest_single(e[3]), nearest_single(e[4]), nearest_single(e[5])], // nocite: vector and matrix dimensions (three axes)
            [nearest_single(e[6]), nearest_single(e[7]), nearest_single(e[8])], // nocite: vector and matrix dimensions (three axes)
        ];
        out[usize::from(s)] = m;
        if (5..=8).contains(&s) { // spec: S-0106 C4 (shapes 9 … 12 equal 5 … 8)
            // spec: S-0106 C4 (shapes 9 … 12 equal 5 … 8)
            out[usize::from(s) + 4] = m;
        }
    }
    out
}

impl GasRules {
    /// The gas rules of a ruleset: `warheads` the warhead names by index,
    /// `draws` the creation-draw fields of a particle type.
    pub fn new(
        r: &glasnost_rules::schema::Ruleset,
        warheads: &[String],
        quirks: &glasnost_rules::quirks::Quirks,
    ) -> Self {
        let s = &r.sim;
        let registry =
            |key: &str| -> Vec<String> { r.registries.get(key).cloned().unwrap_or_default() };
        let particle_names = registry("particles");
        let particle_index = |n: &Option<String>| -> Option<u32> {
            let n = n.as_ref()?;
            particle_names
                .iter()
                .position(|p| p == n)
                .and_then(|i| u32::try_from(i).ok())
        };
        let particles = particle_names
            .iter()
            .enumerate()
            .map(|(i, n)| {
                let t = s.particles.get(n).cloned().unwrap_or_default();
                let mut draw = crate::rules::particle_draw_rules(&t);
                draw.index = u32::try_from(i).ok();
                ParticleType {
                    name: n.clone(),
                    kind: behaviour(t.behaves_like.as_deref()),
                    draw,
                    // spec: S-0613 Import mapping (MaxEC absent 1)
                    max_ec: t.max_ec.unwrap_or(1),
                    // spec: S-0613 claim 5 and Import mapping (MaxDC absent 0, reduced modulo 65536)
                    max_dc: t.max_dc.unwrap_or(0) as u16,
                    // spec: S-0613 Import mapping (Damage absent 0)
                    damage: t.damage.unwrap_or(0),
                    warhead: t
                        .warhead
                        .as_ref()
                        .and_then(|h| warheads.iter().position(|w| w == h))
                        .and_then(|i| u32::try_from(i).ok()),
                    // spec: S-0613 Import mapping and F8 (StartStateAI, EndStateAI absent 0, StateAIAdvance absent 4; each stored as a signed 8-bit value)
                    start_stage: t.start_state_ai.unwrap_or(0) as i8,
                    end_stage: t.end_state_ai.unwrap_or(0) as i8,
                    advance: t.state_ai_advance.unwrap_or(4) as i8, // spec: S-0613 Import mapping (StateAIAdvance absent: 4 ticks)
                    // spec: S-0613 Import mapping (DeleteOnStateLimit absent no)
                    delete_at_limit: t.delete_on_state_limit.unwrap_or(false),
                    next: particle_index(&t.next_particle),
                    // spec: S-0613 Import mapping (NextParticleOffset absent 0, 0, 0)
                    next_offset: t.next_particle_offset.unwrap_or([0; 3]),
                    // spec: S-0613 Import mapping (WindEffect absent 0)
                    wind_effect: t.wind_effect.unwrap_or(0),
                    // spec: S-0613 claim 16 and Import mapping (Translucency absent 0, reduced modulo 256)
                    translucency: t.translucency.unwrap_or(0) as i8,
                }
            })
            .collect();
        let systems: Vec<SystemType> = registry("particle_systems")
            .iter()
            .map(|n| {
                let t = s.particle_systems.get(n).cloned().unwrap_or_default();
                SystemType {
                    name: n.clone(),
                    kind: behaviour(t.behaves_like.as_deref()),
                    holds: particle_index(&t.holds_what),
                    // spec: S-0613 Import mapping (Lifetime absent −1)
                    lifetime: t.lifetime.unwrap_or(-1),
                }
            })
            .collect();
        let shared = systems
            .iter()
            .position(|t| t.name.eq_ignore_ascii_case(SHARED_SYSTEM))
            .and_then(|i| u32::try_from(i).ok());
        let g = &s.general;
        // spec: S-0613 Import mapping and Edge cases (WindDirection absent −1; outside 0 … 7 no push)
        let direction = g.wind_direction.unwrap_or(-1);
        let wind = usize::try_from(direction)
            .ok()
            .and_then(|d| WIND.get(d).copied())
            .map(|(dx, dy)| {
                if direction == 3 && !quirks.wind_southeast_short { // spec: S-0613 Edge cases (the short step of direction 3)
                    (WIND_3_FULL, dy)
                } else {
                    (dx, dy)
                }
            });
        GasRules {
            particles,
            systems,
            shared,
            virus: g.infantry_virus.clone(),
            wind,
            // spec: S-0613 Import mapping and Constants (Gravity absent 3 leptons per tick²)
            gravity: g.gravity.unwrap_or(3),
            ramps: ramps(),
        }
    }

    /// The system type a name names, if the rules list it.
    pub fn system(&self, name: &str) -> Option<u32> {
        self.systems
            .iter()
            .position(|t| t.name == name)
            .and_then(|i| u32::try_from(i).ok())
    }
}

/// One particle (S-0613 Terms).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Particle {
    /// Its object number (S-0306 Terms).
    pub id: ObjectId,
    /// Its type, by registration index.
    pub type_index: u32,
    /// Position, leptons.
    pub pos: Pos,
    /// Velocity (vₓ, vᵧ, v_z), leptons per tick.
    pub vel: [i32; 3], // nocite: vector and matrix dimensions (three axes)
    /// The stored deflection (dₓ, dᵧ), single precision.
    pub defl: [F32; 2], // nocite: the two horizontal axes
    /// Remaining lifetime *E*, 16 bits.
    pub life: u16,
    /// Damage countdown *C*, 16 bits.
    pub countdown: u16,
    /// Particle stage *s*, signed 8 bits.
    pub stage: i8,
    /// The dead mark.
    pub dead: bool,
}

/// One particle system (S-0613 Terms).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParticleSystem {
    /// Its type, by registration index.
    pub type_index: u32,
    /// The owner house.
    pub owner: Option<HouseId>,
    /// Its position (no observable effect, claim 1).
    pub pos: Pos,
    /// The system lifetime, ticks (−1: never times out).
    pub lifetime: i32,
    /// Marked to end (claim 6).
    pub ending: bool,
    /// Its particles, oldest first.
    pub particles: Vec<Particle>,
}

/// The virus-death animation of claim 2 while it plays (one forward pass,
/// S-0109 claims 2 to 5, 14 and 16).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirusAnim {
    /// The animation type, by registration index.
    pub anim: u32,
    /// Its position.
    pub pos: Pos,
    /// Its owner house (claim 2; it changes nothing here).
    pub owner: Option<HouseId>,
    /// The effective stage delay *d*.
    pub delay: i32,
    /// The stage *k*.
    pub k: i32,
    /// The tick of the last advance, or of creation before the first.
    pub last: i32,
    /// The first update (S-0109 claim 5) has passed.
    pub updated: bool,
}

/// Gas state.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GasWorld {
    /// Particle systems in play by object number.
    pub systems: BTreeMap<ObjectId, ParticleSystem>,
    /// The shared gas system (claim 1), while in play.
    pub shared: Option<ObjectId>,
    /// Virus-death animations playing, by object number.
    pub anims: BTreeMap<ObjectId, VirusAnim>,
}

/// Creates the shared gas system at game start (claim 1): no owner, the
/// lifetime of its type, empty. It enters the update order before the map
/// is read (Q-1240).
pub fn create_shared(w: &mut World) {
    let Some(ti) = w.rules().gas.shared else {
        return;
    };
    let lifetime = w.rules().gas.systems[ti as usize].lifetime;
    let id = w.new_id();
    // spec: S-0613 claim 1 (no owner house, no target, the system lifetime of its type; it starts empty)
    w.gas.systems.insert(
        id,
        ParticleSystem {
            type_index: ti,
            owner: None,
            pos: Pos::default(),
            lifetime,
            ending: false,
            particles: Vec::new(),
        },
    );
    w.gas.shared = Some(id);
    w.update.enter(id);
}

/// The signed magnitude *m*(*r*) of F1 (wrapping at −2³¹).
fn magnitude(r: i32) -> i32 {
    // spec: S-0613 F1 (m(r) = |r|, except m(−2³¹) = −2³¹)
    r.wrapping_abs()
}

/// Creates one particle of type `ti` in system `sys` at `at` (claim 5),
/// with its creation draws (S-0202 claim 15). Returns its object number.
fn add_particle(w: &mut World, sys: ObjectId, ti: u32, at: Pos) -> Option<ObjectId> {
    let rules = w.rules_arc();
    let t = rules.gas.particles.get(ti as usize)?;
    let life = creation_draws(w, t.draw);
    let h = w.surface_at(at.x, at.y);
    let id = w.new_id();
    let p = Particle {
        id,
        type_index: ti,
        // spec: S-0613 claim 5 (a height at or below H is raised to H)
        pos: Pos::new(at.x, at.y, at.z.max(h)),
        // spec: S-0613 claim 5 (velocity and deflection start at zero)
        vel: [0; 3],
        defl: [F32(0); 2], // nocite: the two horizontal axes
        life,
        // spec: S-0613 claim 5 (C starts at the damage interval modulo 65536, the stage at the start-stage field)
        countdown: t.max_dc,
        stage: t.start_stage,
        dead: false,
    };
    // spec: S-0613 claim 5 (a new particle is added at the end of its system's list)
    w.gas.systems.get_mut(&sys)?.particles.push(p);
    Some(id)
}

/// A particle's creation draws (S-0202 claim 15) in table order; returns
/// the remaining lifetime *E* (claim 5, Q-1242).
fn creation_draws(w: &mut World, p: ParticleRules) -> u16 {
    let half = w.rules().quirks.ranged_draw_half_span;
    if p.smoke {
        // spec: S-0202 claim 15 (smoke: one raw value for a starting-speed variation)
        let _ = w.rng.raw();
    }
    // spec: S-0202 claim 15 (every particle: one raw value for its lifetime variation, magnitude after signed remainder, modulo 10 for railgun behaviour, otherwise modulo the lifetime field)
    let r = w.rng.raw() as i32;
    let modulus = if p.railgun {
        RAILGUN_MODULUS
    } else {
        p.max_ec
    };
    // spec: S-0205 claim 3 (a remainder by 0 is the dividend, our engine's departure; the importer warns)
    let e = magnitude(glasnost_num::int::rem(r, modulus));
    if p.colour_draw {
        // spec: S-0202 claim 15 (one ranged draw R(0, 2147483646) for the starting-colour interpolation)
        let _ = w.rng.ranged(0, 2147483646, half);
    }
    // spec: S-0613 claim 5 (the lifetime is reduced modulo 65536)
    e as u16
}

/// One particle requested by an animation's middle event (S-0109 claim 14):
/// created in the shared gas system at `at`, whatever its behaviour kind
/// (S-0613 claim 2). Without a position (a resource glint, S-0202 claim 14)
/// or with no shared system in play, only its creation draws are taken.
pub fn middle_event_particle(w: &mut World, p: ParticleRules, at: Option<(i32, i32, i32)>) {
    match (at, p.index, w.gas.shared) {
        (Some((x, y, z)), Some(ti), Some(sys)) => {
            add_particle(w, sys, ti, Pos::new(x, y, z));
        }
        (None, ..) => {
            creation_draws(w, p);
        }
        (_, None, _) => {
            creation_draws(w, p);
            w.note("S-0004 claim 2: a middle event's particle type is not a listed type; its creation draws are taken and no particle is made");
        }
        (_, _, None) => {
            creation_draws(w, p);
            w.note("Q-1244: no shared gas system is in play (its lifetime ran out, or the rules list no particle-system type); a requested particle takes its creation draws and is not made");
        }
    }
}

/// The particle system a detonation's warhead names (S-0303 claim 20,
/// S-0613 claim 4): created at `p` owned by `house`, with one particle of
/// its held type at once. A system of another behaviour kind than gas, or
/// none, is not kept (Q-1243): its first particle takes its creation draws.
pub fn warhead_system(w: &mut World, name: &str, p: Pos, house: Option<HouseId>) {
    let rules = w.rules_arc();
    let Some(ti) = rules.gas.system(name) else {
        w.note(format!(
            "S-0303 claim 20: the warhead's particle system {name} is not a listed type; no system is made"
        ));
        return;
    };
    let t = &rules.gas.systems[ti as usize];
    match t.kind {
        Some(Behaviour::Gas) | None => {
            let id = w.new_id();
            // spec: S-0303 claim 20 and S-0613 claim 4 (created at P, owned by the source house, with its type's lifetime)
            w.gas.systems.insert(
                id,
                ParticleSystem {
                    type_index: ti,
                    owner: house,
                    pos: p,
                    lifetime: t.lifetime,
                    ending: false,
                    particles: Vec::new(),
                },
            );
            w.update.enter(id);
            if let Some(pt) = t.holds {
                // spec: S-0613 claim 4 (one particle of the held type at P at once, subject to claim 5's terrain floor)
                add_particle(w, id, pt, p);
            }
        }
        Some(_) => {
            if let Some(pt) = t.holds
                && let Some(pr) = rules.gas.particles.get(pt as usize)
            {
                creation_draws(w, pr.draw);
            }
            w.note("Q-1243: a warhead's particle system of the smoke, fire, spark or railgun kind is not kept; its first particle takes its creation draws only");
        }
    }
}

/// A virus death's clouds (S-0613 claims 2 and 3) for an infantry unit at
/// `pos`: `extra` for the extra-animation cases of S-0107 claim 17 (claim
/// 3), otherwise the human row's animation (claim 2). `owner` is the
/// animation's owner house.
pub fn virus_death(w: &mut World, pos: Pos, owner: Option<HouseId>, extra: bool) {
    let rules = w.rules_arc();
    let Some(name) = rules.gas.virus.clone() else {
        return;
    };
    let Some((index, a)) = rules
        .animations
        .iter()
        .enumerate()
        .find(|(_, a)| a.name.eq_ignore_ascii_case(&name))
    else {
        return;
    };
    let at = (pos.x, pos.y, pos.z);
    if extra {
        // spec: S-0613 claim 3 (created with no loop delay: its start event at once, with the middle event when μ = 0)
        crate::load::anim_created(w, &name, Some(at), true);
        // spec: S-0613 claim 3 (then exactly one particle of its particle type in the shared gas system at the unit's position; the animation is removed in the same step)
        match a.particles {
            Some((p, _)) => middle_event_particle(w, p, Some(at)),
            // spec: S-0613 Edge cases (no particle type: no particle, a departure; the importer warns)
            None => w.note("S-0613 Edge cases: the virus-death animation names no particle type; no extra particle is made"),
        }
        return;
    }
    // spec: S-0613 claim 2 (the animation is created at the unit's position; its middle event's particles go into the shared gas system)
    let delay = crate::load::anim_created(w, &name, Some(at), true);
    w.effects.push(crate::combat::effects::Effect::Anim {
        name: name.clone(),
        pos,
        delay,
        reverse: false,
    });
    // Only a later middle event (μ ≠ 0) with effects needs the playback.
    let effects = a.particles.is_some_and(|(_, n)| n > 0) || a.scorch || a.crater;
    if !(effects && a.mu.is_some_and(|m| m != 0)) {
        return;
    }
    if end_limit(a).is_none() {
        w.note("S-0109 claim 25: the virus-death animation has no image, so its playback and later middle event are not modelled");
        return;
    }
    let t = w.tick;
    let d = crate::superweapons::storm::stage_delay(w, a, delay);
    let id = w.new_id();
    w.gas.anims.insert(
        id,
        VirusAnim {
            anim: u32::try_from(index).unwrap_or(u32::MAX),
            pos,
            owner,
            delay: d,
            k: 0,
            last: t,
            updated: false,
        },
    );
    // spec: S-0109 claim 5 (created during the turn pass: its first turn comes in that tick after every object already in the order)
    w.update.enter(id);
}

/// The end limit *E* of one pass (S-0109 claims 6, 10 and 25): the end
/// field, or the sprite end (⌊N / 2⌋ for a shadow type) when absent or
/// −1; `None` without an image.
fn end_limit(a: &AnimRules) -> Option<i32> {
    let n = a.frames?;
    // spec: S-0109 claim 10 (a shadow type's sprite end is ⌊N / 2⌋ instead of N)
    let sprite_end = if a.shadow { n / 2 } else { n };
    // spec: S-0109 claim 25 (an absent end field, or −1, takes the sprite end)
    Some(match a.end {
        None | Some(-1) => sprite_end,
        Some(e) => e,
    })
}

/// A virus-death animation's turn (S-0109 claims 5, 14 and 16).
pub fn anim_turn(w: &mut World, id: ObjectId) {
    let t = w.tick;
    let rules = w.rules_arc();
    let Some(v) = w.gas.anims.get_mut(&id) else {
        return;
    };
    if !v.updated {
        // spec: S-0109 claim 5 (the first update produces no advance)
        v.updated = true;
        return;
    }
    // spec: S-0109 claims 2 and 5 (advance when d > 0 and at least d ticks passed since the last advance, or creation)
    if v.delay <= 0 || t.wrapping_sub(v.last) < v.delay {
        return;
    }
    v.k += 1;
    v.last = t;
    let v = v.clone();
    let Some(a) = rules.animations.get(v.anim as usize) else {
        return;
    };
    // spec: S-0109 claim 14 (the middle event on an advance after which a + k = μ, μ ≠ 0, not for a bouncing type; also on an ending advance)
    if !a.bouncing
        && a.mu
            .is_some_and(|m| m != 0 && a.start.wrapping_add(v.k) == m)
    {
        crate::load::middle_event(w, a, Some((v.pos.x, v.pos.y, v.pos.z)));
    }
    // spec: S-0109 claims 6 and 16 (the pass ends on the advance after which k ≥ E; with no follow-up the animation ends and is removed)
    if end_limit(a).is_none_or(|e| v.k >= e) {
        w.gas.anims.remove(&id);
        let skip = w.rules().quirks.update_order_removal_skip;
        // spec: S-0203 claim 10 (a removal from the update order, counting during the turn pass)
        w.update.leave(id, skip);
    }
}

/// Whether tick `t` is even.
fn even(t: i32) -> bool {
    t.rem_euclid(2) == 0 // spec: S-0613 claims 8 to 11 (even tick numbers)
}

/// A particle system's turn (claim 6).
pub fn system_turn(w: &mut World, id: ObjectId) {
    let rules = w.rules_arc();
    let Some(s) = w.gas.systems.get(&id) else {
        return;
    };
    let kind = rules.gas.systems[s.type_index as usize].kind;
    if kind == Some(Behaviour::Gas) {
        update_pass(w, id);
        second_pass(w, id);
    }
    // Q-1244: the lifetime counts down, and the removal is tested, at the end of the turn.
    let Some(s) = w.gas.systems.get_mut(&id) else {
        return;
    };
    if s.lifetime > 0 {
        // spec: S-0613 claim 6 (the system lifetime goes down by 1 each turn; reaching exactly 0 marks it to end; −1 never reaches 0)
        s.lifetime -= 1;
        if s.lifetime == 0 {
            s.ending = true;
        }
    }
    if s.ending && s.particles.is_empty() {
        // spec: S-0613 claim 6 (a marked system with no particles left is removed)
        w.gas.systems.remove(&id);
        if w.gas.shared == Some(id) {
            w.gas.shared = None;
        }
        let skip = w.rules().quirks.update_order_removal_skip;
        // spec: S-0203 claim 10 (a removal from the update order, counting during the turn pass)
        w.update.leave(id, skip);
    }
}

/// The update pass (claim 6): every particle there at the start, newest
/// first. Particles a hit's death adds meanwhile are not updated.
fn update_pass(w: &mut World, id: ObjectId) {
    let n = w.gas.systems.get(&id).map_or(0, |s| s.particles.len());
    // spec: S-0613 claim 6 (the newer particle's update effects, draws included, precede the older's)
    for i in (0..n).rev() {
        update_particle(w, id, i);
    }
}

fn particle(w: &World, sys: ObjectId, i: usize) -> Option<Particle> {
    w.gas.systems.get(&sys)?.particles.get(i).cloned()
}

fn put(w: &mut World, sys: ObjectId, i: usize, p: Particle) {
    if let Some(slot) = w
        .gas
        .systems
        .get_mut(&sys)
        .and_then(|s| s.particles.get_mut(i))
    {
        *slot = p;
    }
}

/// One particle's update (claims 8, 9, 10, 13 and 15, then the lifetime of
/// claim 6; the order follows Q-1241).
fn update_particle(w: &mut World, sys: ObjectId, i: usize) {
    let rules = w.rules_arc();
    let Some(mut p) = particle(w, sys, i) else {
        return;
    };
    let Some(t) = rules.gas.particles.get(p.type_index as usize) else {
        return;
    };
    let gas = t.kind == Some(Behaviour::Gas);
    if gas {
        if even(w.tick) {
            drift(w, &mut p);
            vertical(w, &mut p);
            p.defl = deflection(w, p.pos);
        }
        put(w, sys, i, p.clone());
        damage_step(w, sys, i);
        let Some(q) = particle(w, sys, i) else { return };
        p = q;
        stage_step(t, &mut p);
    } else {
        w.note("S-0613 Edge cases: a particle of another behaviour kind in a gas system only counts its lifetime down; its own update and movement belong to other specs");
    }
    // spec: S-0613 claim 6 (the lifetime goes down by one modulo 65536; 0 marks it dead)
    p.life = p.life.wrapping_sub(1);
    if p.life == 0 {
        p.dead = true;
    }
    put(w, sys, i, p);
}

/// Claim 8, F2: the random drift on an even tick.
fn drift(w: &mut World, p: &mut Particle) {
    // spec: S-0613 F2 (one raw value r₁; with the low three bits of m(r₁) zero, two more)
    let r1 = w.rng.raw() as i32;
    if magnitude(r1) & 7 != 0 { // spec: S-0613 F2 (the low three bits of m(r₁))
        return;
    }
    let r2 = w.rng.raw() as i32;
    let r3 = w.rng.raw() as i32;
    // spec: S-0613 F2 (k = (m(r₃) rem 3) − 1, a truncated remainder)
    let k = glasnost_num::int::rem(magnitude(r3), 3) - 1;
    let c = |u: i32| u.clamp(-DRIFT_CLAMP, DRIFT_CLAMP);
    let [vx, vy, _] = p.vel;
    // spec: S-0613 F2 (m(r₂) even: k goes to vₓ, odd: to vᵧ; both are clamped)
    if magnitude(r2) & 1 == 0 {
        p.vel[0] = c(vx.wrapping_add(k));
        p.vel[1] = c(vy);
    } else {
        p.vel[0] = c(vx);
        p.vel[1] = c(vy.wrapping_add(k));
    }
}

/// Claim 9, F4: the vertical speed on an even tick.
fn vertical(w: &World, p: &mut Particle) {
    let a = p.pos.z - w.surface_at(p.pos.x, p.pos.y);
    // spec: S-0613 F4 (v_z ← max(v_z − 1, −5) when a > 5, otherwise max(v_z, 0))
    p.vel[2] = if a > HOVER {
        (p.vel[2] - 1).max(FALL_LIMIT)
    } else {
        p.vel[2].max(0)
    };
}

/// Whether a cell holds an obstacle (claim 10).
fn obstacle(w: &World, c: CellXY) -> bool {
    let rules = w.rules();
    let mut building = false;
    for &id in w.cells.list(c, Layer::Ground) {
        let Some(o) = w.obj(id) else { continue };
        if o.kind != Kind::Building {
            continue;
        }
        building = true;
        // spec: S-0613 claim 10 (an inactive laser-fence section is no obstacle; section states are not modelled, so every section counts as inactive)
        if rules.buildings[o.type_index as usize].building.laser_fence {
            continue;
        }
        // spec: S-0613 claim 10 (a vehicle-like building, S-0214 Terms, is no obstacle)
        if crate::targeting::vehicle_like(w, id) {
            continue;
        }
        return true;
    }
    // spec: S-0613 claim 10 (in a cell without a building, one of the three wall overlays)
    !building && w.cells.cell(c).is_some_and(|cell| WALLS.contains(&cell.overlay))
}

/// m · v for a single-precision matrix and vector (F3): products and sums
/// in double precision, grouped (a_y + a_z) + a_x for x and (a_y + a_x) +
/// a_z for y and z, each completed component stored as a single.
fn transform(m: &[[F32; 3]; 3], v: [F32; 3], mode: Rounding) -> [F32; 3] { // nocite: vector and matrix dimensions (three axes)
    let mut out = [F32(0); 3]; // nocite: vector and matrix dimensions (three axes)
    for (i, row) in m.iter().enumerate() {
        let a: [F64; 3] = [0, 1, 2].map(|j| row[j].to_f64().mul(v[j].to_f64(), mode)); // nocite: vector and matrix dimensions (three axes)
        // spec: S-0613 F3 (the sums are grouped (a_y + a_z) + a_x for x, (a_y + a_x) + a_z for y and z, with no fused operations)
        let sum = if i == 0 {
            a[1].add(a[2], mode).add(a[0], mode)
        } else {
            a[1].add(a[0], mode).add(a[2], mode)
        };
        // spec: S-0613 F3 (each completed transformed component is single precision, rounded in the mode)
        out[i] = sum.to_f32(mode);
    }
    out
}

fn transposed(m: &[[F32; 3]; 3]) -> [[F32; 3]; 3] { // nocite: vector and matrix dimensions (three axes)
    let mut t = *m;
    for (i, row) in t.iter_mut().enumerate() {
        for (j, e) in row.iter_mut().enumerate() {
            *e = m[j][i];
        }
    }
    t
}

fn diagonal(a: &str, b: &str, c: &str) -> [[F32; 3]; 3] { // nocite: vector and matrix dimensions (three axes)
    let z = nearest_single("0");
    [
        [nearest_single(a), z, z],
        [z, nearest_single(b), z],
        [z, z, nearest_single(c)],
    ]
}

/// Claim 10, F3: the deflection of a particle at `p` on an even tick.
pub fn deflection(w: &World, p: Pos) -> [F32; 2] { // nocite: the two horizontal axes
    let rules = w.rules();
    let g = rules.gas.gravity;
    let mode = rules.quirks.rounding();
    let depth = PROBE_BASE.wrapping_add(g);
    let zq = p.z.wrapping_sub(depth);
    let h = w.surface_at(p.x, p.y);
    let b = h + DECK;
    let c = p.cell();
    // spec: S-0613 F3 (a crossing: a high bridge's cell, z ≥ b > z − 2 − G downward or z < b ≤ z − 2 − G upward)
    let crossing = w.high_bridge(c) && ((p.z >= b && b > zq) || (p.z < b && b <= zq));
    // spec: S-0613 F3 (an obstacle: without a crossing, h ≤ z − 2 − G < h + 150 in a cell holding an obstacle)
    let blocked = !crossing && h <= zq && zq < h + OBSTACLE_BAND && obstacle(w, c);
    // spec: S-0613 F3 (deflect with a crossing, an obstacle, or z − 2 − G < h; otherwise (0, 0))
    if !(crossing || blocked || zq < h) {
        return [F32(0); 2];
    }
    let shape = w.cells.cell(c).map_or(0, |cell| usize::from(cell.slope));
    let m = rules
        .gas
        .ramps
        .get(shape)
        .copied()
        .unwrap_or_else(identity);
    // spec: S-0613 F3 (v⁰ = (0, 0, −2 − G) in single precision)
    let v0 = [
        F32(0),
        F32(0),
        F64::from_i64(-i64::from(depth), mode).to_f32(mode),
    ];
    // spec: S-0613 F3 (S = diag(1, −1, 1), Z = diag(1, 1, −1); w = S · M · Z · Mᵀ · S · v⁰, transformed from right to left)
    let s = diagonal("1", "-1", "1");
    let z = diagonal("1", "1", "-1");
    let v = transform(&s, v0, mode);
    let v = transform(&transposed(&m), v, mode);
    let v = transform(&z, v, mode);
    let v = transform(&m, v, mode);
    let v = transform(&s, v, mode);
    // spec: S-0613 F3 (the stored deflection is (w_x, w_y))
    [v[0], v[1]]
}

/// Claim 13, F7: the damage step of particle `i`.
fn damage_step(w: &mut World, sys: ObjectId, i: usize) {
    let rules = w.rules_arc();
    let Some(mut p) = particle(w, sys, i) else {
        return;
    };
    let Some(t) = rules.gas.particles.get(p.type_index as usize) else {
        return;
    };
    // spec: S-0613 F7 (C ← (C − 1) mod 65536)
    p.countdown = p.countdown.wrapping_sub(1);
    // spec: S-0613 F7 (a hit is due when the result is 0 and damage is nonzero; only then is the countdown reset)
    let due = p.countdown == 0 && t.damage != 0;
    if due {
        p.countdown = t.max_dc;
    }
    let pos = p.pos;
    put(w, sys, i, p);
    if !due {
        return;
    }
    let owner = w.gas.systems.get(&sys).and_then(|s| s.owner);
    let mode = rules.mode;
    // spec: S-0613 claim 13 (every object listed in the ground layer of its cell, in listing order, S-0209 claim 13, whatever the particle's height)
    let list: Vec<ObjectId> = w.cells.list(pos.cell(), Layer::Ground).to_vec();
    for target in list {
        let Some(o) = w.obj(target) else { continue };
        // spec: S-0613 claim 13 (alive with health above 0, judged at the hit)
        let dead = o.combat.killed && o.combat.delay_kill_until.is_none();
        if o.health <= 0 || !o.on_map || dead {
            continue;
        }
        let at = o.pos;
        // spec: S-0613 F7 (D = L(|xₒ − x| + |yₒ − y|), S-0409 F2's lift applied to the horizontal sum, used as the lepton distance)
        let sum = (at.x - pos.x).wrapping_abs().wrapping_add((at.y - pos.y).wrapping_abs());
        let d = crate::econ::shroud::height_lift(sum, mode);
        let hit = Hit {
            amount: t.damage,
            warhead: t.warhead,
            distance: d,
            // spec: S-0613 claims 13 and 14 (no attacker; the attacking house is the system's owner, none for the shared system; defences not ignored, escape allowed)
            attacker: None,
            house: owner,
            ignore_defences: false,
            no_escape: false,
        };
        w.effects
            .push(crate::combat::effects::Effect::GasHit { id: target, pos });
        // spec: S-0613 claim 13 (ordinary damage, not a detonation; each resolved before the next)
        crate::combat::damage::apply(w, target, hit);
    }
}

/// Claim 15, F8: the particle stage step.
fn stage_step(t: &ParticleType, p: &mut Particle) {
    let n = p.id.0 as i32;
    // spec: S-0613 F8 ((n + E_max − E) divisible by (n mod 2) + A, signed 32-bit wraparound; S-0205 claim 3 for a zero divisor)
    let x = n.wrapping_add(t.max_ec).wrapping_sub(i32::from(p.life));
    let divisor = n.rem_euclid(2) + i32::from(t.advance); // spec: S-0613 F8 ((n mod 2) + A)
    if glasnost_num::int::rem(x, divisor) == 0 {
        // spec: S-0613 F8 (an 8-bit signed counter, wrapping from 127 to −128)
        p.stage = p.stage.wrapping_add(1);
    }
    // spec: S-0613 claim 15 (at the end stage: dead with the delete flag, otherwise back to 0)
    if p.stage == t.end_stage {
        if t.delete_at_limit {
            p.dead = true;
        } else {
            p.stage = 0;
        }
    }
}

/// The wind push of type `t` at tick `tick` (F6).
fn wind_push(w: &World, t: &ParticleType, tick: i32) -> (i32, i32) {
    let Some(push) = w.rules().gas.wind else {
        return (0, 0);
    };
    if t.wind_effect <= 0 {
        return (0, 0);
    }
    // spec: S-0613 F6 (due when W > 0 and t rem ⌊10 / W⌋ = 0; a zero divisor gives the remainder t, S-0205 claim 3)
    if glasnost_num::int::rem(tick, WIND_CADENCE / t.wind_effect) == 0 {
        push
    } else {
        (0, 0)
    }
}

/// Claim 11, F5: one live gas particle's move.
fn move_particle(w: &World, t: &ParticleType, p: &mut Particle) {
    let tick = w.tick;
    let (dx, dy) = wind_push(w, t, tick);
    let Pos { x, y, z } = p.pos;
    if even(tick) {
        // spec: S-0613 F5 (even ticks: (x + Δₓ, y + Δᵧ, z), no height clamp)
        p.pos = Pos::new(x.wrapping_add(dx), y.wrapping_add(dy), z);
        return;
    }
    let mode = w.rules().quirks.rounding();
    // spec: S-0613 F5 (x₁ = trunc(x + Δₓ + dₓ): the integer sum and the single deflection added in S-0205's precision, truncated toward zero)
    let x1 = F64::from_i64(i64::from(x) + i64::from(dx), mode)
        .add(p.defl[0].to_f64(), mode)
        .to_i32_common();
    let y1 = F64::from_i64(i64::from(y) + i64::from(dy), mode)
        .add(p.defl[1].to_f64(), mode)
        .to_i32_common();
    let h1 = w.surface_at(x1, y1);
    // spec: S-0613 F5 (z₁ = z − clamp(z − h₁ − 5, 0, 2))
    let z1 = z - (z - h1 - HOVER).clamp(0, SINK_STEP);
    // spec: S-0613 F5 ((x′, y′) = (x₁ + vₓ, y₁ + vᵧ); z′ = max(z₁ + v_z, H(x′, y′) + 5))
    let x2 = x1.wrapping_add(p.vel[0]);
    let y2 = y1.wrapping_add(p.vel[1]);
    let z2 = (z1 + p.vel[2]).max(w.surface_at(x2, y2) + HOVER); // spec: S-0613 F5 (never below the hover height above the terrain)
    p.pos = Pos::new(x2, y2, z2);
}

/// The second pass (claims 6, 7 and 11): live particles move; dead ones
/// leave successors, newest dead first, and are removed.
fn second_pass(w: &mut World, sys: ObjectId) {
    let rules = w.rules_arc();
    let Some(s) = w.gas.systems.get(&sys) else {
        return;
    };
    let mut list = s.particles.clone();
    // spec: S-0613 claims 6 and 11 (only particles not marked dead move, by F5)
    for p in list.iter_mut().filter(|p| !p.dead) {
        if let Some(t) = rules.gas.particles.get(p.type_index as usize)
            && t.kind == Some(Behaviour::Gas)
        {
            move_particle(w, t, p);
        }
    }
    let dead: Vec<Particle> = list.iter().filter(|p| p.dead).cloned().collect();
    list.retain(|p| !p.dead);
    if let Some(s) = w.gas.systems.get_mut(&sys) {
        s.particles = list;
    }
    // spec: S-0613 claims 6 and 7 (successor creation follows all update effects, newest dead particle first)
    for d in dead.iter().rev() {
        let Some(t) = rules.gas.particles.get(d.type_index as usize) else {
            continue;
        };
        let Some(next) = t.next else { continue };
        let [ox, oy, oz] = t.next_offset;
        // spec: S-0613 claim 7 (at the dead particle's position plus the displacement, by claim 5; then it takes the dead particle's velocity)
        let at = Pos::new(
            d.pos.x.wrapping_add(ox),
            d.pos.y.wrapping_add(oy),
            d.pos.z.wrapping_add(oz),
        );
        if add_particle(w, sys, next, at).is_some()
            && let Some(p) = w
                .gas
                .systems
                .get_mut(&sys)
                .and_then(|s| s.particles.last_mut())
        {
            p.vel = d.vel;
        }
    }
}

/// The particles in play, with their system's owner and type: for the
/// drawing and the headless runner.
pub fn particles(w: &World) -> impl Iterator<Item = (&Particle, &ParticleSystem)> {
    w.gas
        .systems
        .values()
        .flat_map(|s| s.particles.iter().map(move |p| (p, s)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fog(max_ec: i32, advance: i8, end_stage: i8, delete_at_limit: bool) -> ParticleType {
        ParticleType {
            name: "FOG".into(),
            kind: Some(Behaviour::Gas),
            draw: ParticleRules::default(),
            max_ec,
            max_dc: 0,
            damage: 0,
            warhead: None,
            start_stage: 0,
            end_stage,
            advance,
            delete_at_limit,
            next: None,
            next_offset: [0; 3],
            wind_effect: 0,
            translucency: 0,
        }
    }

    fn particle(id: u32, life: u16, stage: i8) -> Particle {
        Particle {
            id: ObjectId(id),
            type_index: 0,
            pos: Pos::new(0, 0, 0),
            vel: [0; 3],
            defl: [F32(0); 2],
            life,
            countdown: 0,
            stage,
            dead: false,
        }
    }

    /// spec: S-0106 table C4 (the single nearest to each entry; no entry's nearest double lies halfway between two singles, so going through the double gives the nearest single)
    #[test]
    fn c4_entries_are_not_midpoints() {
        for (_, row) in C4 {
            for e in row {
                let d = crate::rules::nearest_double(e).0;
                // A normal double drops 29 mantissa bits on the way to a single.
                assert_ne!(d & ((1 << 29) - 1), 1 << 28, "{e} is a midpoint");
            }
        }
    }

    /// spec: S-0106 table C4 (shapes 9 … 12 equal 5 … 8; shapes it does not list are the identity)
    #[test]
    fn ramp_table_shapes() {
        let r = ramps();
        assert_eq!(r.len(), 21);
        for s in 5..=8 {
            assert_eq!(r[s], r[s + 4]);
        }
        assert_eq!(r[0], identity());
    }

    /// spec: S-0613 F1 (m(r) = |r|, except m(−2³¹) = −2³¹)
    #[test]
    fn magnitude_wraps() {
        assert_eq!(magnitude(-7), 7);
        assert_eq!(magnitude(7), 7);
        assert_eq!(magnitude(i32::MIN), i32::MIN);
    }

    /// spec: S-0613 F8 (the stage advances when (n + E_max − E) is divisible by (n mod 2) + A)
    #[test]
    fn stage_step_divisibility() {
        let t = fog(100, 4, 50, false);
        // n = 2 (even): divisor 4; x = 2 + 100 − 98 = 4.
        let mut p = particle(2, 98, 0);
        stage_step(&t, &mut p);
        assert_eq!(p.stage, 1);
        // x = 2 + 100 − 97 = 5: no advance.
        let mut p = particle(2, 97, 0);
        stage_step(&t, &mut p);
        assert_eq!(p.stage, 0);
        // n = 3 (odd): divisor 5; x = 3 + 100 − 98 = 5.
        let mut p = particle(3, 98, 0);
        stage_step(&t, &mut p);
        assert_eq!(p.stage, 1);
    }

    /// spec: S-0613 F8 and claim 15 (an 8-bit signed counter wrapping from 127 to −128; at the end stage back to 0, or dead with the delete flag)
    #[test]
    fn stage_step_wrap_and_limit() {
        let t = fog(0, 1, -128, false);
        // divisor (0 mod 2) + 1 = 1 always divides.
        let mut p = particle(0, 0, 127);
        stage_step(&t, &mut p);
        assert_eq!(p.stage, 0, "wrapped to −128, the end stage, reset to 0");
        let t = fog(0, 1, 3, true);
        let mut p = particle(0, 0, 2);
        stage_step(&t, &mut p);
        assert_eq!(p.stage, 3);
        assert!(p.dead);
    }

    /// spec: S-0613 F8 and S-0205 claim 3 (a zero divisor: the remainder is the dividend)
    #[test]
    fn stage_step_zero_divisor() {
        let t = fog(10, 0, 50, false);
        // n = 0: divisor 0; x = 0 + 10 − 10 = 0, remainder 0: advance.
        let mut p = particle(0, 10, 0);
        stage_step(&t, &mut p);
        assert_eq!(p.stage, 1);
        let mut p = particle(0, 9, 0);
        stage_step(&t, &mut p);
        assert_eq!(p.stage, 0);
    }
}
