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
//! The turns of the smoke, fire, spark and railgun kinds, and the update
//! and move of a particle of those kinds in a system of any kind (S-0616),
//! are in [`crate::particles`].
//!
//! Seams: the drawing of claim 16 belongs to the view; the importer's
//! warning for a loading-time particle request covers cell animations only
//! (theater tile animations are not seen by the rules import).

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
pub(crate) const WALLS: [u8; 3] = [2, 26, 243]; // spec: S-0613 claim 10 and S-0306 Constants (wall overlays 2, 26, 243)

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
    /// The speed field *S* at creation, single precision (S-0616 Terms).
    pub speed: F32,
    /// The slowing field, single precision (S-0616 claims 6 and 12).
    pub deacc: F32,
    /// The last damaging stage δ_F (S-0616 claim 12).
    pub final_stage: i8,
    /// The normalised flag (S-0616 claim 19).
    pub normalized: bool,
    /// The radius field ρ, leptons (S-0616 F3).
    pub radius: i32,
    /// The spark fields *X*, *Y*, *Z*ₘ and *Z*ᵣ (S-0616 F10).
    pub spark: [i32; 4], // nocite: the four spark velocity fields of S-0616 F10
    /// The look of a particle: colour list, start colours, colour speed
    /// and the fire translucency stages (S-0616 claims 26 and 29).
    pub look: ParticleLook,
}

/// A particle type's drawing fields that its updates change state by
/// (S-0616 claims 26 and 29, F16; Import mapping).
#[derive(Clone, Debug, Default)]
pub struct ParticleLook {
    /// The colour list, 8-bit RGB triples in order.
    pub colours: Vec<[u8; 3]>, // nocite: RGB channels
    /// The two start colours *s*₁ and *s*₂.
    pub start: [[u8; 3]; 2], // nocite: two RGB colours
    /// The colour speed *v*, a double.
    pub speed: F64,
    /// The translucent-25 stage field, an 8-bit unsigned stage.
    pub stage25: u8,
    /// The translucent-50 stage field, an 8-bit unsigned stage.
    pub stage50: u8,
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
    /// The smoke spawn interval *T*₀, also the fire emission interval
    /// (S-0616 claims 9 and 11).
    pub spawn_frames: i32,
    /// The smoke emission timer's growth (S-0616 claim 9).
    pub slowdown: F32,
    /// The smoke spawn cut-off (S-0616 claim 9).
    pub cutoff: F32,
    /// The smoke translucency cut-off τ (S-0616 claim 9, F4).
    pub translucency_cutoff: F32,
    /// The smoke spawn radius σ (S-0616 F4).
    pub spawn_radius: i32,
    /// The particle cap κ (S-0616 F10; the smoke cap quirk).
    pub cap: i32,
    /// The spark counter start *K* (S-0616 claim 15).
    pub spark_turns: i32,
    /// The spark chance χ (S-0616 F10).
    pub chance: F64,
    /// The spark spawn direction *s* (S-0616 F10).
    pub direction: [F32; 3], // nocite: vector and matrix dimensions (three axes)
    /// The railgun particles per lepton *p* (S-0616 F12).
    pub per_coord: F64,
}

/// A single-precision rules decimal (S-0003 claim 7), 0.0 when absent.
fn single(d: &Option<glasnost_rules::sim::Decimal>, mode: Rounding) -> F32 {
    d.as_ref()
        .map_or(F32(0), |d| d.single_value(mode).to_f32(mode))
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
    /// The smoke wind push (Δₓ, Δᵧ) of the rules' wind direction by
    /// S-0616's own table; `None` outside 0 … 7 (S-0616 claim 7).
    pub smoke_wind: Option<(i32, i32)>,
    /// The game-wide spark type (`DefaultSparkSystem`, S-0616 Import
    /// mapping), when the key names a listed system type.
    pub default_spark: Option<u32>,
    /// Whether `DefaultSparkSystem` is present at all (S-0601 claim 16).
    pub default_spark_given: bool,
    /// The game-wide refinery-smoke time, ticks (S-0616 claim 1).
    pub refinery_frames: i32,
    /// The red and yellow sparking probabilities of damage sparks, doubles
    /// (S-0616 claim 1).
    pub sparking: (F64, F64),
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

// nocite: vector and matrix dimensions (three axes)
pub(crate) fn identity() -> [[F32; 3]; 3] {
    let one = nearest_single("1");
    let zero = nearest_single("0");
    [[one, zero, zero], [zero, one, zero], [zero, zero, one]]
}

/// The ramp matrices by shape 0 … 20 (S-0106 table C4).
// nocite: vector and matrix dimensions (three axes)
pub fn ramps() -> Vec<[[F32; 3]; 3]> {
    let mut out = vec![identity(); 21]; // spec: S-0106 table C4 (ramp shapes 0 … 20)
    for (s, e) in C4 {
        let m = [
            [
                nearest_single(e[0]),
                nearest_single(e[1]),
                nearest_single(e[2]),
            ], // nocite: vector and matrix dimensions (three axes)
            [
                nearest_single(e[3]),
                nearest_single(e[4]),
                nearest_single(e[5]),
            ], // nocite: vector and matrix dimensions (three axes)
            [
                nearest_single(e[6]),
                nearest_single(e[7]),
                nearest_single(e[8]),
            ], // nocite: vector and matrix dimensions (three axes)
        ];
        out[usize::from(s)] = m;
        // spec: S-0106 C4 (shapes 9 … 12 equal 5 … 8)
        if (5..=8).contains(&s) {
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
        let mode = quirks.rounding();
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
                    // spec: S-0616 Import mapping (Velocity, Deacc: single-precision numbers, absent 0.0)
                    speed: single(&t.velocity, mode),
                    deacc: single(&t.deacc, mode),
                    // spec: S-0616 Import mapping (FinalDamageState: an 8-bit signed integer, absent the end stage's value as read)
                    final_stage: t.final_damage_state.or(t.end_state_ai).unwrap_or(0) as i8,
                    // spec: S-0616 Import mapping (Normalized absent no)
                    normalized: t.normalized.unwrap_or(false),
                    // spec: S-0616 Import mapping (Radius absent 0)
                    radius: t.radius.unwrap_or(0),
                    // spec: S-0616 Import mapping (XVelocity, YVelocity absent 1; MinZVelocity absent 0; ZVelocityRange absent 1)
                    spark: [
                        t.x_velocity.unwrap_or(1),
                        t.y_velocity.unwrap_or(1),
                        t.min_z_velocity.unwrap_or(0),
                        t.z_velocity_range.unwrap_or(1),
                    ],
                    look: ParticleLook {
                        // spec: S-0616 Import mapping (ColorList: 8-bit RGB triples, absent empty)
                        colours: t
                            .color_list
                            .iter()
                            .flatten()
                            .map(|c| c.map(|x| x as u8))
                            .collect(),
                        // spec: S-0616 Import mapping (StartColor1, StartColor2 absent black)
                        start: [
                            t.start_color1.unwrap_or([0; 3]).map(|x| x as u8),
                            t.start_color2.unwrap_or([0; 3]).map(|x| x as u8),
                        ],
                        // spec: S-0616 Import mapping (ColorSpeed: a double, absent 0.0)
                        speed: t
                            .color_speed
                            .as_ref()
                            .map_or(F64::from_i64(0, mode), |d| d.value(mode)),
                        // spec: S-0616 Import mapping (Translucent25State, Translucent50State: 8-bit unsigned stages, absent 255)
                        stage25: t.translucent25_state.unwrap_or(255) as u8, // spec: S-0616 Import mapping (Translucent25State absent 255)
                        stage50: t.translucent50_state.unwrap_or(255) as u8, // spec: S-0616 Import mapping (Translucent50State absent 255)
                    },
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
                    // spec: S-0616 Import mapping (SpawnFrames absent 1)
                    spawn_frames: t.spawn_frames.unwrap_or(1),
                    // spec: S-0616 Import mapping (Slowdown, SpawnCutoff: single-precision numbers, absent 0.0)
                    slowdown: single(&t.slowdown, mode),
                    cutoff: single(&t.spawn_cutoff, mode),
                    // spec: S-0616 Import mapping (SpawnTranslucencyCutoff: a single-precision number, absent 0.0)
                    translucency_cutoff: single(&t.spawn_translucency_cutoff, mode),
                    // spec: S-0616 Import mapping (SpawnRadius absent 0)
                    spawn_radius: t.spawn_radius.unwrap_or(0),
                    // spec: S-0616 Import mapping (ParticleCap absent 50)
                    cap: t.particle_cap.unwrap_or(50),
                    // spec: S-0616 Import mapping (SparkSpawnFrames absent 0)
                    spark_turns: t.spark_spawn_frames.unwrap_or(0),
                    // spec: S-0616 Import mapping (SpawnSparkPercentage: a double fraction, absent 0.0)
                    chance: t
                        .spawn_spark_percentage
                        .as_ref()
                        .map_or(F64::from_i64(0, mode), |d| d.value(mode)),
                    // spec: S-0616 Import mapping (SpawnDirection: three single-precision numbers, absent 0, 0, 0)
                    direction: {
                        let mut out = [F32(0); 3]; // nocite: vector and matrix dimensions (three axes)
                        for (o, d) in out.iter_mut().zip(t.spawn_direction.iter().flatten()) {
                            *o = d.single_value(mode).to_f32(mode);
                        }
                        out
                    },
                    // spec: S-0616 Import mapping (ParticlesPerCoord: a double, absent 0.1)
                    per_coord: t
                        .particles_per_coord
                        .as_ref()
                        .map_or_else(|| crate::rules::nearest_double("0.1"), |d| d.value(mode)),
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
                // spec: S-0613 Edge cases (the short step of direction 3)
                if direction == 3 && !quirks.wind_southeast_short {
                    (WIND_3_FULL, dy)
                } else {
                    (dx, dy)
                }
            });
        // spec: S-0616 claim 7, F2 and Constants (smoke uses its own wind table; outside 0 … 7 no push, a departure)
        let smoke_wind = usize::try_from(direction)
            .ok()
            .and_then(|d| crate::particles::SMOKE_WIND.get(d).copied());
        // spec: S-0616 Import mapping ([CombatDamage] DefaultSparkSystem: the game-wide spark type, absent none) and S-0601 claim 16
        let default_spark_given = g.default_spark_system.is_some();
        let default_spark = g.default_spark_system.as_ref().and_then(|n| {
            systems
                .iter()
                .position(|t| &t.name == n)
                .and_then(|i| u32::try_from(i).ok())
        });
        // spec: S-0616 Import mapping ([General] ConditionRedSparkingProbability and ConditionYellowSparkingProbability, doubles; absent 0.02 and 0.01)
        let prob = |d: &Option<glasnost_rules::sim::Decimal>, absent: &str| {
            d.as_ref()
                .map_or_else(|| crate::rules::nearest_double(absent), |d| d.value(mode))
        };
        GasRules {
            particles,
            systems,
            shared,
            smoke_wind,
            default_spark,
            default_spark_given,
            // spec: S-0616 Import mapping ([General] RefinerySmokeFrames absent 25)
            refinery_frames: g.refinery_smoke_frames.unwrap_or(25),
            sparking: (
                prob(&g.condition_red_sparking_probability, "0.02"),
                prob(&g.condition_yellow_sparking_probability, "0.01"),
            ),
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
    /// The particle speed *S*, single precision (S-0616 Terms).
    pub speed: F32,
    /// The flight direction *u*, single precision (S-0616 Terms, F1).
    pub dir: [F32; 3], // nocite: vector and matrix dimensions (three axes)
    /// A spark's velocity, single precision (S-0616 F10, F11).
    pub fvel: [F32; 3], // nocite: vector and matrix dimensions (three axes)
    /// A fire particle's step for this tick (S-0616 F8).
    pub step: [i32; 3], // nocite: vector and matrix dimensions (three axes)
    /// Its own stage-advance value (S-0616 claim 19).
    pub advance: i8,
    /// Its translucency value, 8 bits, kept modulo 256 (S-0616 claim 26).
    #[serde(default)]
    pub translucency: u8,
    /// A pixel kind's colour state (S-0616 claim 29, F16).
    #[serde(default)]
    pub colour: ColourState,
    /// Its creation position (S-0616 F15).
    #[serde(default)]
    pub origin: Pos,
    /// The target point it was given at creation (S-0616 F15).
    #[serde(default)]
    pub aim: Pos,
}

/// An 8-bit RGB colour (S-0616 F16).
pub type Rgb = [u8; 3]; // nocite: RGB channels

/// Black, the colour of an empty list (S-0616 F16).
fn own_black() -> Rgb {
    // spec: S-0616 F16 (with an empty list c₀ is black; colours are 8-bit RGB triples)
    [0; 3]
}

impl ColourState {
    /// The state at creation (S-0616 F16): the own colour from the type's
    /// colour list and start colours with the creation colour draw `r`
    /// (taken only when the type has one, S-0202 claim 15), *i* = 0 and
    /// *t* = 0.
    pub fn new(look: &ParticleLook, r: Option<i32>, m: Rounding) -> Self {
        let own = if look.colours.is_empty() {
            // spec: S-0616 F16 (with an empty list c₀ is black)
            [0; 3]
        } else if look.start.iter().flatten().all(|&x| x == 0) {
            // spec: S-0616 F16 (with both start colours black c₀ is list entry 0 and there is no draw)
            look.colours[0]
        } else {
            let r = r.unwrap_or(0);
            let single = |x: F64| x.to_f32(m).to_f64();
            // spec: S-0616 F16 (f = r × 4.656613 × 10⁻¹⁰ in single precision)
            let k = single(crate::consts::COLOUR_START_SCALE); // spec: S-0616 F16 (the single-precision scale 4.656613 × 10⁻¹⁰)
            let f = single(single(F64::from_i64(i64::from(r), m)).mul(k, m));
            let one_minus = single(F64::from_i64(1, m).sub(f, m));
            let [s1, s2] = look.start;
            let mut c = own_black();
            for (ch, out) in c.iter_mut().enumerate() {
                // spec: S-0616 F16 (c₀ = trunc(s₁ × (1 − f) + s₂ × f), clamped to 0 … 255, single precision)
                let a = single(F64::from_i64(i64::from(s1[ch]), m).mul(one_minus, m));
                let b = single(F64::from_i64(i64::from(s2[ch]), m).mul(f, m));
                let v = single(a.add(b, m)).to_i32_common();
                *out = v.clamp(0, 255) as u8; // spec: S-0616 F16 (clamped to 0 … 255)
            }
            c
        };
        ColourState {
            own,
            index: 0,
            t: F64::from_i64(0, m),
        }
    }

    /// One update's colour step (S-0616 claims 16, 18 and 29, F16) with
    /// the update's colour draw `r`.
    pub fn advance(&mut self, look: &ParticleLook, r: i32, m: Rounding) {
        // spec: S-0616 F16 and Constants (t ← (r × 4.656612877414201 × 10⁻¹⁰ × 0.05 + v) + t, double precision, summed in that order)
        let k = crate::consts::DRAW_SCALE; // spec: S-0616 Constants (scale 4.656612877414201 × 10⁻¹⁰)
        let jitter = crate::consts::TWENTIETH; // spec: S-0616 Constants (colour progress jitter 0.05)
        let step = F64::from_i64(i64::from(r), m)
            .mul(k, m)
            .mul(jitter, m)
            .add(look.speed, m);
        self.t = step.add(self.t, m);
        let one = F64::from_i64(1, m);
        if self.t.gt(one) {
            let n = i32::try_from(look.colours.len()).unwrap_or(i32::MAX);
            // spec: S-0616 F16 (when i < n − 2: t ← 0 and i ← i + 1; otherwise t ← 1.0)
            if self.index < n - 2 {
                self.t = F64::from_i64(0, m);
                self.index += 1;
            } else {
                self.t = one;
            }
        }
    }

    /// The drawn colour before the light-alpha step (S-0616 claim 29, F16):
    /// from *A* (the own colour while *i* = 0, else list entry *i*) towards
    /// *B* (entry *i* + 1, or *A* when the list has none).
    pub fn drawn(&self, look: &ParticleLook, m: Rounding) -> Rgb {
        let i = usize::try_from(self.index).unwrap_or(0);
        let a = if i == 0 {
            self.own
        } else {
            look.colours.get(i).copied().unwrap_or(self.own)
        };
        // spec: S-0616 claim 29 (short lists, departure: no entry i + 1 moves towards A)
        let b = look.colours.get(i + 1).copied().unwrap_or(a);
        let one_minus = F64::from_i64(1, m).sub(self.t, m);
        let mut c = own_black();
        for (ch, out) in c.iter_mut().enumerate() {
            // spec: S-0616 F16 (c = trunc(A × (1 − t) + B × t), floating point)
            let v = F64::from_i64(i64::from(a[ch]), m)
                .mul(one_minus, m)
                .add(F64::from_i64(i64::from(b[ch]), m).mul(self.t, m), m)
                .to_i32_common();
            *out = v.clamp(0, 255) as u8; // spec: S-0616 F16 (8-bit channels of colours of the list, so in 0 … 255)
        }
        c
    }
}

/// A particle's colour state (S-0616 claim 29, F16): own colour *c*₀,
/// colour index *i* and colour progress *t*.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColourState {
    /// The own colour *c*₀, 8-bit RGB.
    pub own: [u8; 3], // nocite: RGB channels
    /// The colour index *i*.
    pub index: i32,
    /// The colour progress *t*, a double.
    pub t: F64,
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
    /// The system lifetime, a signed count of turns (claim 17).
    pub lifetime: i32,
    /// Marked to end (claim 17).
    pub ending: bool,
    /// Its particles, oldest first.
    pub particles: Vec<Particle>,
    /// Its owner object, if any (S-0616 Terms, claim 4).
    pub owner_object: Option<ObjectId>,
    /// Its displacement from the owner object at creation (S-0616 claim 5).
    pub owner_offset: [i32; 3], // nocite: vector and matrix dimensions (three axes)
    /// Its target point (S-0616 Terms).
    pub target: Pos,
    /// The smoke emission timer *T* (S-0616 Terms, claim 9).
    pub timer: F32,
    /// The spark counter *K* (S-0616 Terms, claim 15).
    pub sparks: i32,
    /// Its target object, if any (S-0616 Terms and claim 4).
    #[serde(default)]
    pub target_object: Option<ObjectId>,
    /// Its particles made through a next-type link on a loop of particle
    /// types: such a particle does not split through such a link itself
    /// (#198; [`crate::loops`]). Kept here, not on the particle, so the
    /// particle's own fields stay as S-0613 gives them.
    #[serde(default)]
    pub looped: std::collections::BTreeSet<ObjectId>,
}

impl ParticleSystem {
    /// A new system of type `t` at `pos` (S-0616 claim 1): no particle, no
    /// owner object, the type's lifetime, *T* at the spawn interval and *K*
    /// at the spark-turns field.
    pub fn new(ti: u32, t: &SystemType, pos: Pos, owner: Option<HouseId>, mode: Rounding) -> Self {
        ParticleSystem {
            type_index: ti,
            owner,
            pos,
            lifetime: t.lifetime,
            ending: false,
            particles: Vec::new(),
            owner_object: None,
            owner_offset: [0; 3], // nocite: vector and matrix dimensions (three axes)
            // spec: S-0616 Terms (a target point the creator passes; none: the world origin)
            target: Pos::default(),
            // spec: S-0616 Terms (T starts at the type's spawn interval; K at its spark-turns field)
            timer: F64::from_i64(i64::from(t.spawn_frames), mode).to_f32(mode),
            sparks: t.spark_turns,
            target_object: None,
            looped: std::collections::BTreeSet::new(),
        }
    }
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
/// lifetime of its type, empty. The caller creates it last in the
/// game-start step, before only the ore-twinkle animations, so it is
/// appended to the update order after every map-placed object and
/// starting unit.
pub fn create_shared(w: &mut World) {
    let Some(ti) = w.rules().gas.shared else {
        return;
    };
    let rules = w.rules_arc();
    let t = &rules.gas.systems[ti as usize];
    let id = w.new_id();
    // spec: S-0613 claim 1 (no owner house, no target, the system lifetime of its type; it starts empty) and S-0616 claim 1 and Constants (its position, an engine constant: (2688, 2688, 0) leptons, the hypothesis P1 decides)
    let pos = Pos::new(2688, 2688, 0); // spec: S-0616 Constants (shared gas system position (2688, 2688, 0) leptons)
    w.gas
        .systems
        .insert(id, ParticleSystem::new(ti, t, pos, None, rules.mode));
    w.gas.shared = Some(id);
    w.update.enter(id);
}

/// The signed magnitude *m*(*r*) of F1 (wrapping at −2³¹).
fn magnitude(r: i32) -> i32 {
    // spec: S-0613 F1 (m(r) = |r|, except m(−2³¹) = −2³¹)
    r.wrapping_abs()
}

/// Creates one particle of type `ti` in system `sys` at `at` (claim 5),
/// aimed at target point `target` (S-0616 F1), with its creation draws
/// (S-0202 claim 15). Returns its object number.
pub(crate) fn add_particle(
    w: &mut World,
    sys: ObjectId,
    ti: u32,
    at: Pos,
    target: Pos,
) -> Option<ObjectId> {
    let rules = w.rules_arc();
    let t = rules.gas.particles.get(ti as usize)?;
    let (life, smoke, colour) = creation_draws(w, t.draw);
    let h = w.surface_at(at.x, at.y);
    let id = w.new_id();
    let mode = rules.mode;
    // spec: S-0616 Terms (S starts at the type's speed field plus the smoke variation of S-0202 claim 15, formed in double precision and held in single)
    let speed = t
        .speed
        .to_f64()
        .add(F64::from_i64(i64::from(smoke), mode), mode)
        .to_f32(mode);
    // spec: S-0616 F1 (u from the position before the terrain floor to the target point)
    let dir = crate::particles::heading(w, at, target);
    let advance = crate::particles::own_advance(w, t, at, target, dir, speed);
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
        speed,
        dir,
        // spec: S-0616 Terms (a new particle's velocity starts at zero, whatever its kind)
        fvel: [F32(0); 3], // nocite: vector and matrix dimensions (three axes)
        step: [0; 3],      // nocite: vector and matrix dimensions (three axes)
        advance,
        // spec: S-0616 claim 26 (at creation: the type's translucency field, kept modulo 256)
        translucency: t.translucency as u8,
        // spec: S-0616 claim 29 and F16 (own colour, i = 0 and t = 0 at creation)
        colour: ColourState::new(&t.look, colour, mode),
        // spec: S-0616 F15 (the creation position and the target point given at creation)
        origin: Pos::new(at.x, at.y, at.z.max(h)),
        aim: target,
    };
    // spec: S-0613 claim 5 (a new particle is added at the end of its system's list)
    w.gas.systems.get_mut(&sys)?.particles.push(p);
    Some(id)
}

/// A particle's creation draws (S-0202 claim 15) in table order; returns
/// the remaining lifetime *E* (claim 5, F9) and the smoke speed variation
/// (0 for other kinds).
pub(crate) fn creation_draws(w: &mut World, p: ParticleRules) -> (u16, i32, Option<i32>) {
    let half = w.rules().quirks.ranged_draw_half_span;
    let mut smoke = 0;
    if p.smoke {
        // spec: S-0202 claim 15 (smoke: one raw value, signed remainder modulo 2, for a starting-speed variation)
        smoke = glasnost_num::int::rem(w.rng.raw() as i32, 2);
    }
    // spec: S-0202 claim 15 (every particle: one raw value for its lifetime variation, magnitude after signed remainder, modulo 10 for railgun behaviour, otherwise modulo the lifetime field)
    let r = w.rng.raw() as i32;
    let modulus = if p.railgun { RAILGUN_MODULUS } else { p.max_ec };
    // spec: S-0613 F9, S-0202 claim 15 and S-0205 claim 3 (u = |r rem E_max|, in 0 … |m| − 1 for a nonzero modulus m, the remainder first; a remainder by 0 is the dividend, our engine's departure; the importer warns)
    let u = magnitude(glasnost_num::int::rem(r, modulus));
    let colour = if p.colour_draw {
        // spec: S-0202 claim 15 (one ranged draw R(0, 2147483646) for the starting-colour interpolation)
        Some(w.rng.ranged(0, 2147483646, half))
    } else {
        None
    };
    // spec: S-0613 claim 5 and F9 (E = (E_max + u) mod 65536, an unsigned 16-bit count)
    (p.max_ec.wrapping_add(u) as u16, smoke, colour)
}

/// One particle requested by an animation's middle event (S-0109 claim 14):
/// created in the shared gas system at `at`, whatever its behaviour kind
/// (S-0613 claim 2). Without a position (a resource glint, S-0202 claim 14)
/// or with no shared system in play, only its creation draws are taken.
pub fn middle_event_particle(w: &mut World, p: ParticleRules, at: Option<(i32, i32, i32)>) {
    match (at, p.index, w.gas.shared) {
        (Some((x, y, z)), Some(ti), Some(sys)) => {
            let at = Pos::new(x, y, z);
            // spec: S-0616 Terms (an animation's request passes no target point: the world origin)
            add_particle(w, sys, ti, at, Pos::default());
        }
        (None, ..) => {
            creation_draws(w, p);
        }
        (_, None, _) => {
            creation_draws(w, p);
            w.note("S-0004 claim 2: a middle event's particle type is not a listed type; its creation draws are taken and no particle is made");
        }
        (_, _, None) => {
            // spec: S-0613 Edge cases (a particle requested before the shared system exists, at loading) and claim 17 (after it left play): a departure, the original stops; our engine takes the creation draws and makes no particle
            creation_draws(w, p);
            w.note("S-0613 claim 17 and Edge cases: no shared gas system is in play (not yet created while loading, its lifetime ran out, or the rules list no particle-system type); a requested particle takes its creation draws and is not made (a departure: the original stops)");
        }
    }
}

/// The particle system a detonation's warhead names (S-0303 claim 20,
/// S-0613 claim 4, S-0616 claim 1): created at `p` owned by `house`, with no
/// owner object, target point `p`, and one particle of its held type at
/// once, whatever its kind; its turns are S-0613's (gas) and S-0616's.
pub fn warhead_system(w: &mut World, name: &str, p: Pos, house: Option<HouseId>) {
    let rules = w.rules_arc();
    let Some(ti) = rules.gas.system(name) else {
        w.note(format!(
            "S-0303 claim 20: the warhead's particle system {name} is not a listed type; no system is made"
        ));
        return;
    };
    let t = &rules.gas.systems[ti as usize];
    let id = w.new_id();
    // spec: S-0303 claim 20, S-0613 claim 4 and S-0616 claim 1 (created at P, owned by the source house, no owner object, target point the detonation point, with its type's lifetime; every system, whatever its kind, takes a turn each tick)
    let mut sys = ParticleSystem::new(ti, t, p, house, rules.mode);
    sys.target = p;
    w.gas.systems.insert(id, sys);
    w.update.enter(id);
    let Some(pt) = t.holds else {
        return;
    };
    // spec: S-0613 claim 4 and S-0616 claims 1 and 22 (one particle of the held type at P at once, subject to claim 5's terrain floor, whatever the system's kind; its target point is P)
    add_particle(w, id, pt, p, p);
}

/// Whether particle system `id` is a gas cloud: a gas-kind system other than
/// the shared one (for counting).
pub fn is_cloud(w: &World, id: ObjectId) -> bool {
    w.gas.shared != Some(id)
        && w.gas.systems.get(&id).is_some_and(|s| {
            w.rules()
                .gas
                .systems
                .get(s.type_index as usize)
                .is_some_and(|t| t.kind == Some(Behaviour::Gas))
        })
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
        .anim_index(name.as_bytes())
        .and_then(|i| rules.animations.get(i as usize).map(|a| (i as usize, a)))
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
        z_adjust: None,
        id: None,
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
    match kind {
        Some(Behaviour::Gas) => {
            // spec: S-0613 claim 6 (a gas system's turn: the update pass, then the second pass)
            update_pass(w, id);
            second_pass(w, id);
        }
        // spec: S-0616 claims 5, 10, 15 and 17 (the kind work of the other kinds)
        Some(Behaviour::Smoke) => crate::particles::smoke_turn(w, id),
        Some(Behaviour::Fire) => crate::particles::fire_turn(w, id),
        Some(Behaviour::Spark) => crate::particles::spark_turn(w, id),
        Some(Behaviour::Railgun) => crate::particles::railgun_turn(w, id),
        // spec: S-0616 claim 2 and S-0613 claim 17 (no behaviour kind: no kind work)
        None => {}
    }
    if kind != Some(Behaviour::Fire) {
        // spec: S-0616 claims 1, 2 and 22 (revision 27) and S-0613 claim 17 (revision 13: a let-go of the fire place marks the system it holds, a gas-kind or no-kind system included)
        // spec: S-0301 claim 5, place-and-kind rule (revision 63, Q-2181: the fire place's let-go events apply to the system it holds whatever its kind; a fire-kind system makes the death test in its own turn) and Moment of a retarget's let-go (revision 67, Q-2246: the retarget test is made at the change of the firer's target, not here)
        crate::emitters::fire_place_let_go(w, id);
    }
    // spec: S-0613 claim 17 (after the kind's own turn work, whatever the kind, also none)
    let Some(s) = w.gas.systems.get_mut(&id) else {
        return;
    };
    // spec: S-0613 claim 17 (the system lifetime, a signed integer count of turns, goes down by 1 whatever its value; marked to end in the turn the result is exactly 0)
    s.lifetime = s.lifetime.wrapping_sub(1);
    if s.lifetime == 0 {
        s.ending = true;
    }
    if s.ending && s.particles.is_empty() {
        // spec: S-0613 claim 17 (then, in the same turn, a marked system that holds no particles leaves play; the shared system is not created again)
        let gone = w.gas.systems.remove(&id);
        // spec: S-0616 claim 1 and Terms (a system held in an emitter place, damage smoke included, stays there until it leaves play)
        if let Some(o) = gone.and_then(|g| g.owner_object) {
            crate::emitters::release_places(w, o, id);
        }
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
        crate::particles::update_one(w, id, i);
    }
}

pub(crate) fn particle(w: &World, sys: ObjectId, i: usize) -> Option<Particle> {
    w.gas.systems.get(&sys)?.particles.get(i).cloned()
}

pub(crate) fn put(w: &mut World, sys: ObjectId, i: usize, p: Particle) {
    if let Some(slot) = w
        .gas
        .systems
        .get_mut(&sys)
        .and_then(|s| s.particles.get_mut(i))
    {
        *slot = p;
    }
}

/// A gas particle's own update (claim 18): on even ticks the drift,
/// vertical speed and deflection (claims 8 to 10), which only set the
/// velocity and deflection; then the countdown and hits (claim 13) and the
/// stage step (claim 15). The lifetime count-down follows in
/// [`crate::particles::update_one`].
pub(crate) fn gas_update(w: &mut World, sys: ObjectId, i: usize) {
    let rules = w.rules_arc();
    let Some(mut p) = particle(w, sys, i) else {
        return;
    };
    let Some(t) = rules.gas.particles.get(p.type_index as usize) else {
        return;
    };
    // spec: S-0613 claim 18 (the drift draws precede the draws of every death the particle's hits cause; the hits precede its stage step; the lifetime count-down comes last)
    if even(w.tick) {
        drift(w, &mut p);
        vertical(w, &mut p);
        p.defl = deflection(w, p.pos);
    }
    put(w, sys, i, p);
    damage_step(w, sys, i);
    let Some(mut p) = particle(w, sys, i) else {
        return;
    };
    stage_step(t, &mut p);
    put(w, sys, i, p);
}

/// Claim 8, F2: the random drift on an even tick.
fn drift(w: &mut World, p: &mut Particle) {
    // spec: S-0613 F2 (one raw value r₁; with the low three bits of m(r₁) zero, two more)
    let r1 = w.rng.raw() as i32;
    // spec: S-0613 F2 (the low three bits of m(r₁))
    if magnitude(r1) & 7 != 0 {
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

/// The deck height of a high-bridge cell `c` at or below a gas particle at
/// height `z`, when `quirks.gas_through_bridge_deck` is off (claim 19
/// **Off**); `None` otherwise.
fn deck_under(w: &World, c: CellXY, z: i32) -> Option<i32> {
    // spec: S-0613 claim 19 (switch on: a gas particle has no on-deck state)
    if w.rules().quirks.gas_through_bridge_deck || !w.high_bridge(c) {
        return None;
    }
    let level = w.cells.cell(c).map_or(0, |c| i32::from(c.level));
    // spec: S-0613 claim 19 Off and S-0616 claim 14 (the deck height (height level × 104) + 416, one value for the cell)
    let deck = level * 104 + DECK;
    // spec: S-0613 claim 19 Off (a particle at or above the deck height, tested with its height before the move)
    (z >= deck).then_some(deck)
}

/// The height a gas particle at height `z` measures from at (x, y): the
/// terrain *H*, or the deck height by claim 19 **Off**.
fn ground(w: &World, x: i32, y: i32, z: i32) -> i32 {
    // spec: S-0204 claim 2 (the containing cell)
    let c = CellXY::new(x / 256, y / 256);
    // spec: S-0613 claim 19 Off (the deck height in place of H for claims 9 and 11)
    deck_under(w, c, z).unwrap_or_else(|| w.surface_at(x, y))
}

/// Claim 9, F4: the vertical speed on an even tick.
fn vertical(w: &World, p: &mut Particle) {
    let a = p.pos.z - ground(w, p.pos.x, p.pos.y, p.pos.z);
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
    !building
        && w.cells
            .cell(c)
            .is_some_and(|cell| WALLS.contains(&cell.overlay))
}

/// m · v for a single-precision matrix and vector (F3): products and sums
/// in double precision, grouped (a_y + a_z) + a_x for x and (a_y + a_x) +
/// a_z for y and z, each completed component stored as a single.
// nocite: vector and matrix dimensions (three axes)
pub(crate) fn transform(m: &[[F32; 3]; 3], v: [F32; 3], mode: Rounding) -> [F32; 3] {
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

// nocite: vector and matrix dimensions (three axes)
pub(crate) fn transposed(m: &[[F32; 3]; 3]) -> [[F32; 3]; 3] {
    let mut t = *m;
    for (i, row) in t.iter_mut().enumerate() {
        for (j, e) in row.iter_mut().enumerate() {
            *e = m[j][i];
        }
    }
    t
}

// nocite: vector and matrix dimensions (three axes)
pub(crate) fn diagonal(a: &str, b: &str, c: &str) -> [[F32; 3]; 3] {
    let z = nearest_single("0");
    [
        [nearest_single(a), z, z],
        [z, nearest_single(b), z],
        [z, z, nearest_single(c)],
    ]
}

/// Claim 10, F3: the deflection of a particle at `p` on an even tick.
// nocite: the two horizontal axes
pub fn deflection(w: &World, p: Pos) -> [F32; 2] {
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
    let m = rules.gas.ramps.get(shape).copied().unwrap_or_else(identity);
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
    // spec: S-0613 claim 19 Off (the bridge-deck layer for a particle at or above its cell's deck height)
    let layer = match deck_under(w, pos.cell(), pos.z) {
        Some(_) => Layer::Deck,
        None => Layer::Ground,
    };
    let list: Vec<ObjectId> = w.cells.list(pos.cell(), layer).to_vec();
    for target in list {
        let Some(o) = w.obj(target) else { continue };
        // spec: S-0613 claim 13 (alive with health above 0, judged at the hit); S-0504 claim 10 (a sinking vessel with health left is not immune; #172)
        let dead = crate::combat::damage::dead(o);
        if o.health <= 0 || !o.on_map || dead {
            continue;
        }
        let at = o.pos;
        // spec: S-0613 F7 (D = L(|xₒ − x| + |yₒ − y|), S-0409 F2's lift applied to the horizontal sum, used as the lepton distance)
        let sum = (at.x - pos.x)
            .wrapping_abs()
            .wrapping_add((at.y - pos.y).wrapping_abs());
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

/// S-0613 F8's advance test of particle `p` with stage-advance value `a`.
pub(crate) fn stage_due(max_ec: i32, a: i8, p: &Particle) -> bool {
    let n = p.id.0 as i32;
    // spec: S-0613 F8 ((n + E_max − E) divisible by (n mod 2) + A, signed 32-bit wraparound; S-0205 claim 3 for a zero divisor)
    let x = n.wrapping_add(max_ec).wrapping_sub(i32::from(p.life));
    let divisor = n.rem_euclid(2) + i32::from(a); // spec: S-0613 F8 ((n mod 2) + A)
    glasnost_num::int::rem(x, divisor) == 0
}

/// Claim 15, F8: the particle stage step.
fn stage_step(t: &ParticleType, p: &mut Particle) {
    if stage_due(t.max_ec, t.advance, p) {
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
pub(crate) fn move_particle(w: &World, t: &ParticleType, p: &mut Particle) {
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
    // spec: S-0613 claim 19 Off (each position's test uses the height z before the move)
    let h1 = ground(w, x1, y1, z);
    // spec: S-0613 F5 (z₁ = z − clamp(z − h₁ − 5, 0, 2))
    let z1 = z - (z - h1 - HOVER).clamp(0, SINK_STEP);
    // spec: S-0613 F5 ((x′, y′) = (x₁ + vₓ, y₁ + vᵧ); z′ = max(z₁ + v_z, H(x′, y′) + 5))
    let x2 = x1.wrapping_add(p.vel[0]);
    let y2 = y1.wrapping_add(p.vel[1]);
    let z2 = (z1 + p.vel[2]).max(ground(w, x2, y2, z) + HOVER); // spec: S-0613 F5 (never below the hover height above the terrain)
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
    // spec: S-0613 claims 6 and 11 (only particles not marked dead move) and S-0616 claim 3 (the particle's own kind chooses the move: gas F5, smoke claim 7, fire claim 13)
    for p in list.iter_mut().filter(|p| !p.dead) {
        crate::particles::move_one(w, p);
    }
    let dead: Vec<Particle> = list.iter().filter(|p| p.dead).cloned().collect();
    list.retain(|p| !p.dead);
    if let Some(s) = w.gas.systems.get_mut(&sys) {
        s.particles = list;
    }
    // spec: S-0613 claims 6 and 7 (successor creation follows all update effects, newest dead particle first; a dead particle of any kind in a gas system gets one successor) and S-0616 claim 3 (successors follow the system's kind, not the particle's)
    for d in dead.iter().rev() {
        let Some(t) = rules.gas.particles.get(d.type_index as usize) else {
            continue;
        };
        let Some(next) = t.next else { continue };
        let [ox, oy, oz] = t.next_offset;
        // spec: S-0613 claim 7 (at the dead particle's position plus the displacement, by claim 5; then it takes exactly the dead particle's speed and velocity, overwriting its own creation's, and nothing else) and S-0616 claim 3
        let at = Pos::new(
            d.pos.x.wrapping_add(ox),
            d.pos.y.wrapping_add(oy),
            d.pos.z.wrapping_add(oz),
        );
        // spec: S-0616 Terms (a successor's target point is not given for the gas kind: the world origin, which no gas rule reads)
        if add_particle(w, sys, next, at, Pos::default()).is_some()
            && let Some(p) = w
                .gas
                .systems
                .get_mut(&sys)
                .and_then(|s| s.particles.last_mut())
        {
            p.speed = d.speed;
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
            speed: F32(0),
            deacc: F32(0),
            final_stage: 0,
            normalized: false,
            radius: 0,
            spark: [1, 1, 0, 1],
            look: ParticleLook::default(),
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
            speed: F32(0),
            dir: [F32(0); 3],
            fvel: [F32(0); 3],
            step: [0; 3],
            advance: 0,
            translucency: 0,
            colour: ColourState::default(),
            origin: Pos::default(),
            aim: Pos::default(),
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
