//! Typed simulation data: the fields of the user's rules and art data that
//! the simulation reads, converted by the importer (`convert::sim`) or written
//! by hand in our format.
//!
//! Every field is optional: an absent field takes the engine default that the
//! owning spec states, which the simulation applies (and cites) when it
//! builds its indexed tables. Rules layering (S-0006) has already happened:
//! a field holds the value in force after every layer, and the "not given"
//! values that keep a held value never appear here.
//!
//! Decimals are quoted strings in our format (PLAN.md 4.6), read exactly as
//! S-0003 reads the same text ([`Decimal`]). Type records are keyed by type
//! name; their registration order comes from `registries` (S-0004).

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::Deserialize;

/// A decimal written as text, `-?digits[.digits][e±digits][%]`, whose value
/// is the one S-0003 gives that text (claims 6 and 7).
#[derive(Clone, Debug, Default, JsonSchema, PartialEq, Eq)]
#[serde(transparent)]
pub struct Decimal(pub String);

impl<'de> Deserialize<'de> for Decimal {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        glasnost_num::decimal::parse_text(&s).map_err(serde::de::Error::custom)?;
        Ok(Decimal(s))
    }
}

impl Decimal {
    /// The value, read as S-0003 reads the text (a decimal field).
    pub fn value(&self, mode: glasnost_num::Rounding) -> glasnost_num::F64 {
        glasnost_num::decimal::parse(&self.0, glasnost_num::decimal::Field::Decimal, mode)
            .unwrap_or_default()
    }

    /// The value of a single-precision field (S-0003 claim 7).
    pub fn single_value(&self, mode: glasnost_num::Rounding) -> glasnost_num::F64 {
        glasnost_num::decimal::parse(&self.0, glasnost_num::decimal::Field::Single, mode)
            .unwrap_or_default()
    }
}

/// The twelve land types (S-0211 Terms).
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum LandType {
    /// Clear.
    Clear,
    /// Road.
    Road,
    /// Water.
    Water,
    /// Rock.
    Rock,
    /// Wall.
    Wall,
    /// Tiberium (resource ground).
    Tiberium,
    /// Beach.
    Beach,
    /// Rough.
    Rough,
    /// Ice.
    Ice,
    /// Railroad.
    Railroad,
    /// Tunnel.
    Tunnel,
    /// Weeds.
    Weeds,
}

/// The eight locomotion classes (S-0211 Terms).
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum LocomotionClass {
    /// Foot.
    Foot,
    /// Track.
    Track,
    /// Wheel.
    Wheel,
    /// Hover.
    Hover,
    /// Winged.
    Winged,
    /// Float.
    Float,
    /// Amphibious.
    Amphibious,
    /// Float-on-beach.
    FloatBeach,
}

/// The thirteen movement zones (S-0211 claim 12).
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MovementZone {
    /// Normal.
    Normal,
    /// Crusher.
    Crusher,
    /// Destroyer.
    Destroyer,
    /// Amphibious destroyer.
    AmphibiousDestroyer,
    /// Amphibious crusher.
    AmphibiousCrusher,
    /// Amphibious.
    Amphibious,
    /// Subterranean.
    Subterranean,
    /// Infantry.
    Infantry,
    /// Infantry destroyer.
    InfantryDestroyer,
    /// Fly.
    Fly,
    /// Water.
    Water,
    /// Water and beach.
    WaterBeach,
    /// Crusher-all.
    CrusherAll,
}

/// The eleven locomotors a type's locomotor field can select (S-0207 claim
/// 14, table T3). Absent from every rules layer: the teleporting locomotor.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Locomotor {
    /// Driving (S-0207).
    Drive,
    /// Hovering.
    Hover,
    /// Tunnelling.
    Tunnel,
    /// Walking (infantry walking, S-0208).
    Walk,
    /// Drop pod.
    DropPod,
    /// Flying.
    Fly,
    /// Teleporting.
    Teleport,
    /// Walking vehicle (mechanical walking).
    Mech,
    /// Ship.
    Ship,
    /// Jumpjet.
    Jumpjet,
    /// Rocket.
    Rocket,
}

/// Every simulation field of the ruleset.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SimData {
    /// Game-wide values.
    #[serde(default)]
    pub general: General,
    /// Land-type rows by land type (S-0211 claim 2). A missing row has the
    /// start-up row.
    #[serde(default)]
    pub land_types: BTreeMap<LandType, LandRow>,
    /// Behaviour properties by behaviour name (S-0215 Import mapping: our
    /// names, snake case).
    #[serde(default)]
    pub behaviours: BTreeMap<String, BehaviourProps>,
    /// Countries by type name.
    #[serde(default)]
    pub countries: BTreeMap<String, Country>,
    /// Infantry types by type name.
    #[serde(default)]
    pub infantry: BTreeMap<String, UnitType>,
    /// Vehicle types by type name.
    #[serde(default)]
    pub vehicles: BTreeMap<String, UnitType>,
    /// Aircraft types by type name.
    #[serde(default)]
    pub aircraft: BTreeMap<String, UnitType>,
    /// Building types by type name.
    #[serde(default)]
    pub buildings: BTreeMap<String, BuildingType>,
    /// Overlay types by type name.
    #[serde(default)]
    pub overlays: BTreeMap<String, OverlayType>,
    /// Terrain-object types by type name.
    #[serde(default)]
    pub terrain_objects: BTreeMap<String, TerrainObjectType>,
    /// Weapons by name.
    #[serde(default)]
    pub weapons: BTreeMap<String, WeaponType>,
    /// Projectiles by name.
    #[serde(default)]
    pub projectiles: BTreeMap<String, ProjectileType>,
    /// Warheads by name.
    #[serde(default)]
    pub warheads: BTreeMap<String, WarheadType>,
    /// Animation types by type name (their art sections, S-0109).
    #[serde(default)]
    pub animations: BTreeMap<String, AnimationType>,
    /// Particle types by type name (S-0202 claim 15, S-0613 Import
    /// mapping).
    #[serde(default)]
    pub particles: BTreeMap<String, ParticleType>,
    /// Particle-system types by type name (S-0613 Import mapping); the
    /// registry `particle_systems` gives their order.
    #[serde(default)]
    pub particle_systems: BTreeMap<String, ParticleSystemType>,
    /// Voxel-animation types (debris pieces) by type name (S-0317 Import
    /// mapping); the registry `voxel_animations` gives their order.
    #[serde(default)]
    pub voxel_animations: BTreeMap<String, VoxelAnimationType>,
    /// The difficulty rows by our name (`easy`, `normal`, `difficult`;
    /// S-0302 Terms, glossary *difficulty skill block*).
    #[serde(default)]
    pub difficulty: BTreeMap<String, DifficultyRow>,
    /// Tiberiums by section name (S-0401 Import mapping); the registry
    /// `tiberiums` gives their order (S-0004 claim 10).
    #[serde(default)]
    pub tiberiums: BTreeMap<String, crate::sim_econ::TiberiumType>,
    /// Superweapon types by type name (S-0407 and S-0412 Import mapping).
    #[serde(default)]
    pub superweapons: BTreeMap<String, crate::sim_econ::SuperweaponType>,
}

/// The eleven armour classes in class-index order (S-0305 claim 1).
#[derive(
    Clone, Copy, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[serde(rename_all = "snake_case")]
pub enum ArmourClass {
    /// *none* (0), also the class of a type no layer has given one.
    #[default]
    None,
    /// *flak* (1).
    Flak,
    /// *plate* (2).
    Plate,
    /// *light* (3).
    Light,
    /// *medium* (4).
    Medium,
    /// *heavy* (5).
    Heavy,
    /// *wood* (6).
    Wood,
    /// *steel* (7).
    Steel,
    /// *concrete* (8).
    Concrete,
    /// *special 1* (9).
    #[serde(rename = "special_1")]
    Special1,
    /// *special 2* (10).
    #[serde(rename = "special_2")]
    Special2,
}

impl ArmourClass {
    /// The classes in class-index order (S-0305 claim 1).
    // spec: S-0305 claim 1 (eleven classes, fixed order)
    pub const ALL: [ArmourClass; 11] = [
        ArmourClass::None,
        ArmourClass::Flak,
        ArmourClass::Plate,
        ArmourClass::Light,
        ArmourClass::Medium,
        ArmourClass::Heavy,
        ArmourClass::Wood,
        ArmourClass::Steel,
        ArmourClass::Concrete,
        ArmourClass::Special1,
        ArmourClass::Special2,
    ];

    /// The class index, 0 to 10 (S-0305 Terms).
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|c| *c == self).unwrap_or(0)
    }

    /// The armour words in class-index order (S-0305 Import mapping).
    // spec: S-0305 Import mapping (the eleven armour words)
    pub const WORDS: [&'static str; 11] = [
        "none",
        "flak",
        "plate",
        "light",
        "medium",
        "heavy",
        "wood",
        "steel",
        "concrete",
        "special_1",
        "special_2",
    ];

    /// The class an armour word names, comparing the whole text and folding
    /// A to Z only (S-0305 claims 3 and 4: an unknown word gives *none*).
    pub fn from_word(v: &[u8]) -> (ArmourClass, bool) {
        for (i, w) in Self::WORDS.iter().enumerate() {
            if v.eq_ignore_ascii_case(w.as_bytes()) {
                return (Self::ALL[i], true);
            }
        }
        // spec: S-0305 claim 4 (a word that matches nothing gives class none)
        (ArmourClass::None, false)
    }
}

/// One difficulty row's fields (S-0301 claim 3, S-0302 claim 3, S-0309
/// claim 25 part i).
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DifficultyRow {
    /// `ROF`, the row's rate-of-fire factor ℓ (a double).
    #[serde(default)]
    pub rof: Option<Decimal>,
    /// `FirePower`, the row's firepower factor φᵣ (a double).
    #[serde(default)]
    pub firepower: Option<Decimal>,
    /// `DestroyWalls`, the destroy-walls flag.
    #[serde(default)]
    pub destroy_walls: Option<bool>,
}

/// A muzzle position (F, L, H) in leptons (S-0313 Terms).
// spec: S-0313 Terms (three integers: forward, lateral, up)
pub type Flh = [i32; 3];

/// An animation type's fields that its creation draws read (S-0109; S-0202
/// claim 10).
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AnimationType {
    /// `RandomRate`: the random rate pair (ρ₁, ρ₂), a −1 already replaced by
    /// the end held (S-0109 claim 3).
    // spec: S-0109 Import mapping (RandomRate: two integers)
    #[serde(default)]
    pub random_rate: Option<[i32; 2]>,
    /// `Bouncer`.
    #[serde(default)]
    pub bouncer: Option<bool>,
    /// `IsMeteor`.
    #[serde(default)]
    pub is_meteor: Option<bool>,
    /// `TiberiumChainReaction`: the start event's resource chain reaction.
    #[serde(default)]
    pub tiberium_chain_reaction: Option<bool>,
    /// `Scorch`: a middle-event effect.
    #[serde(default)]
    pub scorch: Option<bool>,
    /// `Crater`: a middle-event effect.
    #[serde(default)]
    pub crater: Option<bool>,
    /// `SpawnsParticle`: the particle type the middle event requests, by
    /// type name (S-0109 claim 14, S-0202 claim 15).
    #[serde(default)]
    pub spawns_particle: Option<String>,
    /// `NumParticles`: particles requested per middle event (S-0109 claim
    /// 14); absent 0.
    #[serde(default)]
    pub num_particles: Option<i32>,
    /// μ: half the image-frame count of the sprite of the type's last
    /// sprite-found moment, 0 when none was found (S-0109 claims 14 and
    /// 25); absent when the import had no art.
    #[serde(default)]
    pub middle_frame: Option<i32>,
    /// The image-frame count *F* of the type's current (drawn) sprite
    /// (S-0513 claim 46, not twice the middle frame); absent when the import
    /// had no such sprite.
    #[serde(default)]
    pub frame_count: Option<i32>,
    /// The canvas height in pixels of that sprite's header (S-0513 F4);
    /// absent when the import had no such sprite.
    #[serde(default)]
    pub canvas_height: Option<i32>,
    /// `Rate`, the image frames per game minute (S-0109 claim 2); absent
    /// or −1: not given.
    #[serde(default)]
    pub rate: Option<i32>,
    /// `Normalized`, the game-speed normalisation flag (S-0109 claim 4);
    /// absent: no.
    #[serde(default)]
    pub normalized: Option<bool>,
    /// `Start`, the initial image frame *a* (S-0109 Import mapping; S-0517
    /// F1 reads the presentation's start, end and delay).
    #[serde(default)]
    pub start: Option<i32>,
    /// `End`, the end field *E* as written (S-0109 Import mapping: absent
    /// or −1 take the sprite's value).
    #[serde(default)]
    pub end: Option<i32>,
    /// `Shadow`, the shadow-type flag (S-0109 Import mapping); absent: no.
    #[serde(default)]
    pub shadow: Option<bool>,
    /// `IsFlamingGuy`, the burning-walker flag (S-0615 Import mapping, art
    /// data); absent: no.
    #[serde(default)]
    pub is_flaming_guy: Option<bool>,
    /// `RunningFrames`, the running frames *R* of a burning walker (S-0615
    /// Import mapping, art data); absent: 0.
    #[serde(default)]
    pub running_frames: Option<i32>,
    /// `Damage`, the damage value δ of S-0109 claim 17 (art data, a
    /// decimal); absent: 0.
    #[serde(default)]
    pub damage: Option<Decimal>,
    /// `MakeInfantry`, the make-infantry index *i* (S-0611 F4, S-0109
    /// Import mapping; art data); absent: −1, none.
    #[serde(default)]
    pub make_infantry: Option<i32>,
    /// `LoopStart`, λₛ (S-0109 Import mapping); absent: 0.
    #[serde(default)]
    pub loop_start: Option<i32>,
    /// `LoopEnd`, λₑ as written (S-0109 Import mapping and F2); absent:
    /// the sprite end.
    #[serde(default)]
    pub loop_end: Option<i32>,
    /// `LoopCount`, *L* (S-0109 Import mapping); absent: 0.
    #[serde(default)]
    pub loop_count: Option<i32>,
    /// `Reverse`, the reverse flag (S-0109 claim 8); absent: no.
    #[serde(default)]
    pub reverse: Option<bool>,
    /// `PingPong`, the ping-pong flag (S-0109 claim 9); absent: no.
    #[serde(default)]
    pub ping_pong: Option<bool>,
    /// `RandomLoopDelay`, the loop-delay pair in updates (S-0109 claim
    /// 11); absent: 0, 0.
    #[serde(default)]
    pub random_loop_delay: Option<[i32; 2]>, // spec: S-0109 claim 11 (a pair)
    /// `Next`, the follow-up type, by type name (S-0109 claim 15); absent:
    /// none.
    #[serde(default)]
    pub next: Option<String>,
    /// A new-theater type: the new-theater flag without the theater flag
    /// (S-0109 Terms and F2); absent: no.
    #[serde(default)]
    pub new_theater: Option<bool>,
    /// `MaxXYVel`, the horizontal speed limit *M* of a bouncing animation,
    /// a double, leptons per tick (S-0319 F1, F2; absent 15.0).
    #[serde(default)]
    pub max_xy_vel: Option<Decimal>,
    /// `MinZVel`, the lower vertical limit *Z*ₘᵢₙ, a double (S-0319 F1,
    /// F2; absent 3.5).
    #[serde(default)]
    pub min_z_vel: Option<Decimal>,
    /// `MaxZVel`, the type's upper vertical speed, a double, read only
    /// under `quirks.anim_fixed_upper_vertical_speed` off (S-0319 claim 2;
    /// absent 3.5).
    #[serde(default)]
    pub max_z_vel: Option<Decimal>,
    /// `Elasticity`, the elasticity *e*, single precision (S-0319 claim 7;
    /// absent 0.8).
    #[serde(default)]
    pub elasticity: Option<Decimal>,
    /// `DamageRadius`, the landing hits' radius in leptons (S-0319 claim
    /// 7; absent 0).
    #[serde(default)]
    pub damage_radius: Option<i32>,
    /// `Warhead`, the landing hits' and expiry damage's warhead, by name
    /// (S-0319 claims 7 and 8; absent none).
    #[serde(default)]
    pub warhead: Option<String>,
    /// `BounceAnim`, an animation name (S-0319 claim 7; absent none).
    #[serde(default)]
    pub bounce_anim: Option<String>,
    /// `ExpireAnim`, an animation name (S-0319 claim 8; absent none).
    #[serde(default)]
    pub expire_anim: Option<String>,
    /// `TrailerAnim`, an animation name (S-0319 claim 12; absent none).
    #[serde(default)]
    pub trailer_anim: Option<String>,
    /// `TrailerSeperation`, the trailer separation *s* in ticks (S-0319
    /// claim 12; absent 0, which acts as 1).
    #[serde(default)]
    pub trailer_separation: Option<i32>,
    /// `Spawns`, the spawn type, an animation name (S-0319 claim 9; absent
    /// none).
    #[serde(default)]
    pub spawns: Option<String>,
    /// `SpawnCount`, the spawn count *c* (S-0319 claim 9; absent 0).
    #[serde(default)]
    pub spawn_count: Option<i32>,
    /// `IsTiberium`, the resource flag (S-0319 claim 10; absent no).
    #[serde(default)]
    pub is_tiberium: Option<bool>,
    /// `TiberiumSpawnType`, the resource spawn overlay type, by name
    /// (S-0319 claim 10; absent none).
    #[serde(default)]
    pub tiberium_spawn_type: Option<String>,
    /// `TiberiumSpreadRadius`, the resource spread radius *R* in cells
    /// (S-0319 claim 10, F5; absent 0).
    #[serde(default)]
    pub tiberium_spread_radius: Option<i32>,
}

/// A particle type's fields that its creation draws read (S-0202 claim 15
/// and Import mapping).
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ParticleType {
    /// `BehavesLike`: `gas`, `smoke`, `fire`, `spark` or `railgun`; absent
    /// for no recognised behaviour.
    #[serde(default)]
    pub behaves_like: Option<String>,
    /// `MaxEC`: the lifetime field (absent 1).
    #[serde(default)]
    pub max_ec: Option<i32>,
    /// The number of entries of `ColorList` as S-0202 claim 17 reads it
    /// (absent 0).
    #[serde(default)]
    pub color_count: Option<i32>,
    /// `StartColor1`, components 0 … 255 (absent black).
    #[serde(default)]
    pub start_color1: Option<[i32; 3]>, // spec: S-0202 Import mapping (an RGB triple)
    /// `StartColor2`, components 0 … 255 (absent black).
    #[serde(default)]
    pub start_color2: Option<[i32; 3]>, // spec: S-0202 Import mapping (an RGB triple)
    /// `MaxDC`, the damage interval in ticks (S-0613 Import mapping;
    /// absent 0).
    #[serde(default)]
    pub max_dc: Option<i32>,
    /// `Damage`, health points per hit before damage resolution (S-0613
    /// Import mapping; absent 0).
    #[serde(default)]
    pub damage: Option<i32>,
    /// `Warhead`, a warhead name (S-0613 Import mapping; absent none).
    #[serde(default)]
    pub warhead: Option<String>,
    /// `StartFrame` (S-0613 Import mapping; absent 0; not added for gas
    /// drawing, claim 15).
    #[serde(default)]
    pub start_frame: Option<i32>,
    /// `StartStateAI`, the start stage as written (S-0613 Import mapping;
    /// absent 0; the simulation reduces it to a signed byte).
    #[serde(default)]
    pub start_state_ai: Option<i32>,
    /// `EndStateAI`, the end stage as written (S-0613 Import mapping;
    /// absent 0; reduced to a signed byte).
    #[serde(default)]
    pub end_state_ai: Option<i32>,
    /// `StateAIAdvance`, the stage-advance period in ticks as written
    /// (S-0613 Import mapping; absent 4; reduced to a signed byte).
    #[serde(default)]
    pub state_ai_advance: Option<i32>,
    /// `DeleteOnStateLimit` (S-0613 Import mapping; absent no).
    #[serde(default)]
    pub delete_on_state_limit: Option<bool>,
    /// `NextParticle`, a particle type name (S-0613 Import mapping; absent
    /// or `<none>`: none).
    #[serde(default)]
    pub next_particle: Option<String>,
    /// `NextParticleOffset`, leptons per axis (S-0613 Import mapping;
    /// absent 0, 0, 0).
    #[serde(default)]
    pub next_particle_offset: Option<[i32; 3]>, // spec: S-0613 Import mapping (three integers, leptons)
    /// `WindEffect`, the wind cadence divisor (S-0613 Import mapping;
    /// absent 0).
    #[serde(default)]
    pub wind_effect: Option<i32>,
    /// `Translucency`, percentage points as written (S-0613 Import mapping;
    /// absent 0; reduced to a signed byte for drawing, claim 16).
    #[serde(default)]
    pub translucency: Option<i32>,
    /// `Velocity`, the speed field *S* in leptons per tick, a
    /// single-precision decimal (S-0616 Import mapping; absent 0.0).
    #[serde(default)]
    pub velocity: Option<Decimal>,
    /// `Deacc`, the slowing field, a single-precision decimal (S-0616
    /// Import mapping; absent 0.0).
    #[serde(default)]
    pub deacc: Option<Decimal>,
    /// `FinalDamageState`, the last damaging stage as written (S-0616
    /// Import mapping; absent: the end stage's value; reduced to a signed
    /// byte).
    #[serde(default)]
    pub final_damage_state: Option<i32>,
    /// `Normalized`, the normalised flag (S-0616 claim 19; absent no).
    #[serde(default)]
    pub normalized: Option<bool>,
    /// `Radius`, the radius field ρ in leptons (S-0616 F3; absent 0).
    #[serde(default)]
    pub radius: Option<i32>,
    /// `XVelocity`, the spark *X* field (S-0616 F10; absent 1).
    #[serde(default)]
    pub x_velocity: Option<i32>,
    /// `YVelocity`, the spark *Y* field (S-0616 F10; absent 1).
    #[serde(default)]
    pub y_velocity: Option<i32>,
    /// `MinZVelocity`, the spark *Z*ₘ field (S-0616 F10; absent 0).
    #[serde(default)]
    pub min_z_velocity: Option<i32>,
    /// `ZVelocityRange`, the spark *Z*ᵣ field (S-0616 F10; absent 1).
    #[serde(default)]
    pub z_velocity_range: Option<i32>,
    /// `ColorList`, the entries in order, read as comma-separated items
    /// taken in threes, each component reduced modulo 256 (S-0202 claim 17
    /// and Import mapping, `quirks.colour_list_item_triples`; S-0616 claim
    /// 29, F16; absent empty).
    #[serde(default)]
    pub color_list: Option<Vec<[i32; 3]>>, // spec: S-0202 claim 17 and Import mapping (RGB entries, components 0 … 255)
    /// `ColorSpeed`, the colour speed *v*, a double (S-0616 F16; absent
    /// 0.0).
    #[serde(default)]
    pub color_speed: Option<Decimal>,
    /// `Translucent25State`, the fire stage that sets translucency 25
    /// (S-0616 claim 26; absent 255).
    #[serde(default)]
    pub translucent25_state: Option<i32>,
    /// `Translucent50State`, the fire stage that sets translucency 50
    /// (S-0616 claim 26; absent 255).
    #[serde(default)]
    pub translucent50_state: Option<i32>,
}

/// A particle-system type's fields (S-0613 Import mapping).
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ParticleSystemType {
    /// `BehavesLike`: `gas`, `smoke`, `fire`, `spark` or `railgun`; absent
    /// for no behaviour kind (no update).
    #[serde(default)]
    pub behaves_like: Option<String>,
    /// `HoldsWhat`, the held particle type name; absent: none.
    #[serde(default)]
    pub holds_what: Option<String>,
    /// `Lifetime`, ticks; absent −1 (never times out).
    #[serde(default)]
    pub lifetime: Option<i32>,
    /// `SpawnFrames`, the smoke spawn interval *T*₀ and the fire emission
    /// interval, ticks (S-0616 Import mapping; absent 1).
    #[serde(default)]
    pub spawn_frames: Option<i32>,
    /// `Slowdown`, the smoke emission timer's growth, a single-precision
    /// decimal (S-0616 Import mapping; absent 0.0).
    #[serde(default)]
    pub slowdown: Option<Decimal>,
    /// `SpawnCutoff`, the smoke spawn cut-off, a single-precision decimal
    /// (S-0616 Import mapping; absent 0.0).
    #[serde(default)]
    pub spawn_cutoff: Option<Decimal>,
    /// `SpawnTranslucencyCutoff`, the smoke translucency cut-off τ, a
    /// single-precision decimal (S-0616 claim 9, F4; absent 0.0).
    #[serde(default)]
    pub spawn_translucency_cutoff: Option<Decimal>,
    /// `SpawnRadius`, the smoke spawn radius σ in leptons (S-0616 Import
    /// mapping; absent 0).
    #[serde(default)]
    pub spawn_radius: Option<i32>,
    /// `ParticleCap`, the spark particle cap κ (S-0616 Import mapping;
    /// absent 50).
    #[serde(default)]
    pub particle_cap: Option<i32>,
    /// `SparkSpawnFrames`, the spark counter start *K* (S-0616 Import
    /// mapping; absent 0).
    #[serde(default)]
    pub spark_spawn_frames: Option<i32>,
    /// `SpawnSparkPercentage`, the spark chance χ as a fraction, a double
    /// (S-0616 Import mapping; absent 0.0).
    #[serde(default)]
    pub spawn_spark_percentage: Option<Decimal>,
    /// `SpawnDirection`, the spark spawn direction *s*, three
    /// single-precision decimals (S-0616 Import mapping; absent 0, 0, 0).
    #[serde(default)]
    pub spawn_direction: Option<Vec<Decimal>>,
    /// `ParticlesPerCoord`, the railgun particles per lepton *p*, a double
    /// (S-0616 Import mapping; absent 0.1).
    #[serde(default)]
    pub particles_per_coord: Option<Decimal>,
}

/// A voxel-animation type's fields: one piece type of S-0317 (Import
/// mapping). Absent fields take the Import mapping's defaults.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VoxelAnimationType {
    /// `Duration`, the lifetime counter at creation in piece turns
    /// (S-0317 claim 2; absent 30).
    #[serde(default)]
    pub duration: Option<i32>,
    /// `IsMeteor`, a meteor piece (claim 4; absent no).
    #[serde(default)]
    pub is_meteor: Option<bool>,
    /// `MaxXYVel`, the horizontal speed limit *M*, a double, leptons per
    /// tick (F1, F2; absent 15.0).
    #[serde(default)]
    pub max_xy_vel: Option<Decimal>,
    /// `MinZVel`, the lower vertical limit *Z*ₘᵢₙ, a double (absent 3.5).
    #[serde(default)]
    pub min_z_vel: Option<Decimal>,
    /// `MaxZVel`, the upper vertical limit *Z*ₘₐₓ, a double (absent 5.0).
    #[serde(default)]
    pub max_z_vel: Option<Decimal>,
    /// `Elasticity`, the elasticity *e*, single precision (claim 10;
    /// absent 0.8).
    #[serde(default)]
    pub elasticity: Option<Decimal>,
    /// `MinAngularVelocity`, the lower spin limit in degrees per tick as
    /// the rules layers leave it (S-0006 claims 3 and 5: −1 when a section
    /// omits it); absent here: the engine default before any reading, 0.
    #[serde(default)]
    pub min_angular_velocity: Option<Decimal>,
    /// `MaxAngularVelocity`, the upper spin limit in degrees per tick, as
    /// the lower; absent here: the engine default before any reading, 10.
    #[serde(default)]
    pub max_angular_velocity: Option<Decimal>,
    /// `Damage`, the impact and expiry damage amount (claims 7, 11;
    /// absent 0).
    #[serde(default)]
    pub damage: Option<i32>,
    /// `DamageRadius`, leptons (claim 7; absent 0).
    #[serde(default)]
    pub damage_radius: Option<i32>,
    /// `Warhead`, a warhead name (absent none).
    #[serde(default)]
    pub warhead: Option<String>,
    /// `BounceAnim`, an animation name (claim 7; absent none).
    #[serde(default)]
    pub bounce_anim: Option<String>,
    /// `ExpireAnim`, an animation name (claim 11; absent none).
    #[serde(default)]
    pub expire_anim: Option<String>,
    /// `TrailerAnim`, an animation name (claim 6; absent none).
    #[serde(default)]
    pub trailer_anim: Option<String>,
    /// `Spawns`, a voxel-animation name (claim 12; absent none).
    #[serde(default)]
    pub spawns: Option<String>,
    /// `SpawnCount`, the spawn count *c* (claim 12; absent 0).
    #[serde(default)]
    pub spawn_count: Option<i32>,
    /// `IsTiberium`, the resource flag (claim 12; absent no).
    #[serde(default)]
    pub is_tiberium: Option<bool>,
    /// `AttachedSystem`, a particle-system name (claim 13; absent none).
    #[serde(default)]
    pub attached_system: Option<String>,
    /// `StartSound`, a sound name (claim 14; absent none). The stop sound
    /// has no effect (claim 14) and is not kept.
    #[serde(default)]
    pub start_sound: Option<String>,
}

/// Game-wide values (keys of `[General]`, `[AI]`, `[IQ]`, `[AudioVisual]`,
/// the combat-damage section and `[MultiplayerDialogSettings]`).
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct General {
    /// `CloseEnough`, cells (S-0209 F2).
    #[serde(default)]
    pub close_enough: Option<Decimal>,
    /// `[AI] BlockagePathDelay`, ticks (S-0209).
    #[serde(default)]
    pub blockage_path_delay: Option<i32>,
    /// `[AI] PathDelay`, game minutes (S-0212 F3).
    #[serde(default)]
    pub path_delay: Option<Decimal>,
    /// `PlayerScatter` (S-0208 claim 11).
    #[serde(default)]
    pub player_scatter: Option<bool>,
    /// `PlayerReturnFire` (S-0215 claim 18).
    #[serde(default)]
    pub player_return_fire: Option<bool>,
    /// `PlayerAutoCrush` (S-0210 claim 11).
    #[serde(default)]
    pub player_auto_crush: Option<bool>,
    /// `Crush`, cells (S-0210 F2).
    #[serde(default)]
    pub crush_range: Option<Decimal>,
    /// `GuardAreaTargetingDelay`, ticks (S-0206 claim 9, S-0215 F6).
    #[serde(default)]
    pub guard_area_targeting_delay: Option<i32>,
    /// `NormalTargetingDelay`, ticks (S-0215 F6).
    #[serde(default)]
    pub normal_targeting_delay: Option<i32>,
    /// `GuardModeStray`, cells (S-0215 F5).
    #[serde(default)]
    pub guard_mode_stray: Option<Decimal>,
    /// `ApproachTargetResetMultiplier`, the approach reset multiplier m
    /// (S-0215 claim 37 e, F10).
    #[serde(default)]
    pub approach_target_reset_multiplier: Option<i32>,
    /// `VeteranSpeed` (S-0207 claim 2, S-0208 F5).
    #[serde(default)]
    pub veteran_speed: Option<Decimal>,
    /// `TrackedUphill` (S-0207 claim 9).
    #[serde(default)]
    pub tracked_uphill: Option<Decimal>,
    /// `TrackedDownhill` (S-0207 claim 9).
    #[serde(default)]
    pub tracked_downhill: Option<Decimal>,
    /// `WheeledUphill` (S-0207 claim 9).
    #[serde(default)]
    pub wheeled_uphill: Option<Decimal>,
    /// `WheeledDownhill` (S-0207 claim 9).
    #[serde(default)]
    pub wheeled_downhill: Option<Decimal>,
    /// `CliffBackImpassability` (S-0211 claim 8).
    #[serde(default)]
    pub cliff_back_impassability: Option<i32>,
    /// `[AudioVisual] ConditionYellow` (S-0108 F3, S-0207 claim 9).
    #[serde(default)]
    pub condition_yellow: Option<Decimal>,
    /// `[AudioVisual] ConditionRed` (S-0107 F5).
    #[serde(default)]
    pub condition_red: Option<Decimal>,
    /// `[AudioVisual] IdleActionFrequency`, game minutes (S-0107 F4).
    #[serde(default)]
    pub idle_action_frequency: Option<Decimal>,
    /// `OreTwinkle`, the resource-glint animation type (S-0202 claim 14).
    // spec: S-0202 Import mapping (OreTwinkle: an animation-type name in [General])
    #[serde(default)]
    pub ore_twinkle: Option<String>,
    /// `[AudioVisual] OreTwinkleChance`, the resource-glint frequency f
    /// (S-0202 claim 14).
    // spec: S-0202 Import mapping (OreTwinkleChance: a signed integer in [AudioVisual])
    #[serde(default)]
    pub ore_twinkle_chance: Option<i32>,
    /// `ChronoDistanceFactor`, the teleport distance factor k, leptons per
    /// tick (S-0207 F9).
    #[serde(default)]
    pub chrono_distance_factor: Option<i32>,
    /// `ChronoTrigger`, the teleport trigger flag (S-0207 F9).
    #[serde(default)]
    pub chrono_trigger: Option<bool>,
    /// `ChronoMinimumDelay`, the teleport minimum delay m, ticks (S-0207
    /// F9).
    #[serde(default)]
    pub chrono_minimum_delay: Option<i32>,
    /// `ChronoRangeMinimum`, the teleport range minimum g, leptons (S-0207
    /// F9).
    #[serde(default)]
    pub chrono_range_minimum: Option<i32>,
    /// `WarpOut`, the warp animation of a teleporting jump (S-0207 claim 20
    /// e).
    #[serde(default)]
    pub warp_out: Option<String>,
    /// `[General] Wake`, the game-wide wake animation: driving wakes (S-0207
    /// claim 23; engine default none) and the grappling wakes (S-0614 claim
    /// 10; S-0312).
    #[serde(default)]
    pub wake: Option<String>,
    /// `[AudioVisual] ChronoOutSound`, the game-wide departure sound of a
    /// teleporting jump (S-0207 claim 20 e).
    #[serde(default)]
    pub chrono_out_sound: Option<String>,
    /// `[AudioVisual] ChronoInSound`, the game-wide arrival sound of a
    /// teleporting jump (S-0207 claim 20 e).
    #[serde(default)]
    pub chrono_in_sound: Option<String>,
    /// `[AudioVisual] EnterBioReactorSound`, the entry sound at an infantry
    /// absorber (S-0605 claim 7 and Import mapping; absent: none).
    #[serde(default)]
    pub enter_bio_reactor_sound: Option<String>,
    /// `[AudioVisual] LeaveBioReactorSound`, the exit sound at an infantry
    /// absorber (S-0605 claim 13 and Import mapping; absent: none).
    #[serde(default)]
    pub leave_bio_reactor_sound: Option<String>,
    /// `[AudioVisual] BunkerWallsUpSound`, the sound at a tank bunker's
    /// position when a vehicle becomes bunkered (S-0606 claim 7 and Import
    /// mapping; absent: none).
    #[serde(default)]
    pub bunker_walls_up_sound: Option<String>,
    /// `[AudioVisual] BunkerWallsDownSound`, the sound at a tank bunker's
    /// position when its walls go down (S-0606 claims 12 and 14 and Import
    /// mapping; absent: none).
    #[serde(default)]
    pub bunker_walls_down_sound: Option<String>,
    /// `[AudioVisual] EnterGrinderSound`, the rules-wide sound at a
    /// recycling building's consumption (S-0603 claim 7 and Import mapping;
    /// absent: none).
    #[serde(default)]
    pub enter_grinder_sound: Option<String>,
    /// `[AudioVisual] LeaveGrinderSound`, read and not played during a
    /// consumption (S-0603 claim 7 and Import mapping; absent: none).
    #[serde(default)]
    pub leave_grinder_sound: Option<String>,
    /// `MaximumCheerRate`, ticks (S-0213 claim 19).
    #[serde(default)]
    pub maximum_cheer_rate: Option<i32>,
    /// `[IQ] Scatter` (S-0209 claim 11).
    #[serde(default)]
    pub iq_scatter: Option<i32>,
    /// `[IQ] GuardArea` (S-0215 claims 14 and 15).
    #[serde(default)]
    pub iq_guard_area: Option<i32>,
    /// `[IQ] AutoCrush` (S-0210 claim 11; the original's computer houses only).
    #[serde(default)]
    pub iq_auto_crush: Option<i32>,
    /// `[IQ] MaxIQLevels` (S-0209 claim 11).
    #[serde(default)]
    pub iq_max: Option<i32>,
    /// `[General] AIAutoDeployFrameDelay`, ticks: the auto-deploy delay list
    /// of S-0215 claim 31, entries for difficulty indexes 0, 1 and 2,
    /// read as S-0003 claim 22's decimal-prefix items.
    #[serde(default)]
    pub ai_auto_deploy_frame_delay: Option<Vec<i32>>,
    /// `[MultiplayerDialogSettings] GameSpeed` (S-0201 claim 7).
    #[serde(default)]
    pub lobby_game_speed: Option<i32>,
    /// `EngineerCaptureLevel` (S-0214 F2).
    #[serde(default)]
    pub engineer_capture_level: Option<Decimal>,
    /// `[AudioVisual] ScrollMultiplier`, the edge-scroll multiplier μ, a
    /// double (S-0101 F9 and Import mapping); absent: 1.0. Presentation
    /// only: the simulation never reads it.
    #[serde(default)]
    pub scroll_multiplier: Option<Decimal>,
    /// `BaseUnit`: the base-unit list, vehicle type keys (S-0009 claim 45).
    #[serde(default)]
    pub base_unit: Option<Vec<String>>,
    /// `[MultiplayerDialogSettings] TechLevel` as the base rules alone give
    /// it: the game's tech level (S-0009 Terms).
    #[serde(default)]
    pub multiplayer_tech_level: Option<i32>,
    /// `[Maximums] Players` as the base rules alone give it: the maximum
    /// player count of the unused-house cleanup (S-0009 claim 28).
    #[serde(default)]
    pub max_players: Option<i32>,
    /// `GDIGateOne`: the first Allied gate type, a building key (S-0112).
    #[serde(default)]
    pub allied_gate_one: Option<String>,
    /// `GDIGateTwo`: the second Allied gate type (S-0112).
    #[serde(default)]
    pub allied_gate_two: Option<String>,
    /// `NodGateOne`: the first Soviet gate type (S-0112).
    #[serde(default)]
    pub soviet_gate_one: Option<String>,
    /// `NodGateTwo`: the second Soviet gate type (S-0112).
    #[serde(default)]
    pub soviet_gate_two: Option<String>,
    /// `WallTower`: the wall tower type (S-0112).
    #[serde(default)]
    pub wall_tower: Option<String>,
    // --- Combat (M4): the combat-damage section unless stated.
    /// `MaxDamage` (S-0302 claim 6).
    #[serde(default)]
    pub max_damage: Option<i32>,
    /// `OccupyROFMultiplier` m (S-0301 claim 7), a single.
    #[serde(default)]
    pub occupy_rof_multiplier: Option<Decimal>,
    /// `BunkerROFMultiplier` b (S-0301 claim 7), a single.
    #[serde(default)]
    pub bunker_rof_multiplier: Option<Decimal>,
    /// `OccupyDamageMultiplier` M_g (S-0302 F1), a single.
    #[serde(default)]
    pub occupy_damage_multiplier: Option<Decimal>,
    /// `BunkerDamageMultiplier` M_b (S-0302 F1), a single.
    #[serde(default)]
    pub bunker_damage_multiplier: Option<Decimal>,
    /// `OpenToppedDamageMultiplier` M_t (S-0302 F1), a single.
    #[serde(default)]
    pub open_topped_damage_multiplier: Option<Decimal>,
    /// `C4Warhead`, the demolition warhead (S-0302 claims 1 and 20).
    #[serde(default)]
    pub c4_warhead: Option<String>,
    /// `[CombatDamage] FlameDamage2`, the warhead of an animation's damage
    /// over time (S-0109 claim 17 and Import mapping); absent: none.
    #[serde(default)]
    pub flame_damage2: Option<String>,
    /// `CrushWarhead` (S-0303 claim 9).
    #[serde(default)]
    pub crush_warhead: Option<String>,
    /// `IonCannonWarhead` (S-0303 claim 18).
    #[serde(default)]
    pub ion_cannon_warhead: Option<String>,
    /// `BridgeStrength` (S-0303 claim 18).
    #[serde(default)]
    pub bridge_strength: Option<i32>,
    /// `AmmoCrateDamage` (S-0303 claim 19).
    #[serde(default)]
    pub ammo_crate_damage: Option<i32>,
    /// `[CombatDamage] railgundamageradius`, integer leptons: the railgun
    /// hit distance (S-0306 claim 27).
    #[serde(default)]
    pub railgun_damage_radius: Option<i32>,
    /// `[CombatDamage] CollapseChance`, integer percentage points: the
    /// railgun cliff-collapse chance (S-0306 claim 27).
    #[serde(default)]
    pub collapse_chance: Option<i32>,
    /// `SplashList`, animation names (S-0303 F5).
    #[serde(default)]
    pub splash_list: Option<Vec<String>>,
    /// `BallisticScatter`, cells (S-0306 claim 4; lepton form of S-0307 claim 1).
    #[serde(default)]
    pub ballistic_scatter: Option<Decimal>,
    /// `OccupyWeaponRange`, cells (S-0307 claim 4).
    #[serde(default)]
    pub occupy_weapon_range: Option<i32>,
    /// `BunkerWeaponRangeBonus`, cells (S-0307 claim 5).
    #[serde(default)]
    pub bunker_weapon_range_bonus: Option<i32>,
    /// `OpenToppedRangeBonus`, cells (S-0307 claim 5).
    #[serde(default)]
    pub open_topped_range_bonus: Option<i32>,
    /// `FireSupress`, cells (S-0309 F6; lepton form).
    #[serde(default)]
    pub fire_supress: Option<Decimal>,
    /// `DeathWeapon`, the game-wide death weapon (S-0312 claim 6).
    #[serde(default)]
    pub death_weapon: Option<String>,
    /// `[General] VeteranROF` v (S-0301 claim 6), a double.
    #[serde(default)]
    pub veteran_rof: Option<Decimal>,
    /// `[General] VeteranCombat` (S-0302 F1), a double.
    #[serde(default)]
    pub veteran_combat: Option<Decimal>,
    /// `[General] VeteranArmor` (S-0302 F2), a double.
    #[serde(default)]
    pub veteran_armor: Option<Decimal>,
    /// `[General] VeteranSight` (S-0311 F4), a double.
    #[serde(default)]
    pub veteran_sight: Option<Decimal>,
    /// `[General] VeteranRatio` (S-0311 F3), a double.
    #[serde(default)]
    pub veteran_ratio: Option<Decimal>,
    /// `[General] VeteranCap` (S-0311 F3), a double.
    #[serde(default)]
    pub veteran_cap: Option<Decimal>,
    /// `[General] DamageFireTypes`, animation names (S-0302 claim 13).
    #[serde(default)]
    pub damage_fire_types: Option<Vec<String>>,
    /// `[General] InfantryExplode`, an animation name (S-0302 claim 15;
    /// death kind 3 of S-0615 claim 1).
    #[serde(default)]
    pub infantry_explode: Option<String>,
    /// `[General] DeadBodies`, the general corpse list, animation names
    /// (S-0107 claim 17 and Import mapping; absent empty).
    #[serde(default)]
    pub dead_bodies: Option<Vec<String>>,
    /// `[General] FlamingInfantry`, the death-kind 4 animation (S-0615
    /// claim 1); absent: none.
    #[serde(default)]
    pub flaming_infantry: Option<String>,
    /// `[General] InfantryHeadPop`, the death-kind 6 animation (S-0615
    /// claim 1); absent: none.
    #[serde(default)]
    pub infantry_head_pop: Option<String>,
    /// `[General] InfantryNuked`, the death-kind 7 animation (S-0615 claim
    /// 1); absent: none.
    #[serde(default)]
    pub infantry_nuked: Option<String>,
    /// `[General] InfantryBrute`, the death-kind 10 animation (S-0615
    /// claim 1); absent: none.
    #[serde(default)]
    pub infantry_brute: Option<String>,
    /// `[AudioVisual] DiskLaserChargeUp`, the disc laser's charge-up sound
    /// (S-0615 claim 15); absent: silent.
    #[serde(default)]
    pub disk_laser_charge_up: Option<String>,
    /// `[General] MissileROTVar` m (S-0306 claim 13), a double.
    #[serde(default)]
    pub missile_rot_var: Option<Decimal>,
    /// `[General] MissileSafetyAltitude`, leptons (S-0306 claim 17).
    #[serde(default)]
    pub missile_safety_altitude: Option<i32>,
    /// `[General] HarvesterUnit`: vehicle type keys (S-0303 claim 9).
    #[serde(default)]
    pub harvester_unit: Option<Vec<String>>,
    /// `[General] BarrelExplode`, an animation name (S-0303 claim 19).
    #[serde(default)]
    pub barrel_explode: Option<String>,
    /// `[General] BarrelDebris`, animation names (S-0303 claim 19).
    #[serde(default)]
    pub barrel_debris: Option<Vec<String>>,
    /// `[General] BarrelParticle`, a particle-system name (S-0303 claim 19).
    #[serde(default)]
    pub barrel_particle: Option<String>,
    /// `[General] MetallicDebris`, animation names (S-0303 F6, S-0312 F1).
    #[serde(default)]
    pub metallic_debris: Option<Vec<String>>,
    /// `[General] BridgeExplosions`, animation names (S-0511 claim 9, F2).
    #[serde(default)]
    pub bridge_explosions: Option<Vec<String>>,
    /// `[General] NoParachuteMaxFallRate`, leptons per update (S-0511 F3).
    #[serde(default)]
    pub no_parachute_max_fall_rate: Option<i32>,
    /// `[General] ParachuteMaxFallRate`, leptons per update (S-0511 F3).
    #[serde(default)]
    pub parachute_max_fall_rate: Option<i32>,
    /// `[AudioVisual] RepairBridgeSound`, a sound name (S-0511 claim 17).
    #[serde(default)]
    pub repair_bridge_sound: Option<String>,
    /// `[CombatDamage] DefaultSparkSystem`, a particle-system name: the drone
    /// delivery's spark effect (S-0614 claim 9); absent-key default not
    /// established.
    #[serde(default)]
    pub default_spark_system: Option<String>,
    /// `[General] RefinerySmokeFrames`, the refinery-smoke system lifetime
    /// in ticks (S-0616 claim 1 and Import mapping; absent 25).
    #[serde(default)]
    pub refinery_smoke_frames: Option<i32>,
    /// `[General] ConditionRedSparkingProbability`, the red sparking
    /// probability of damage sparks, a double (S-0616 claim 1; absent 0.02).
    #[serde(default)]
    pub condition_red_sparking_probability: Option<Decimal>,
    /// `[General] ConditionYellowSparkingProbability`, the yellow sparking
    /// probability of damage sparks, a double (S-0616 claim 1; absent 0.01).
    #[serde(default)]
    pub condition_yellow_sparking_probability: Option<Decimal>,
    /// `[AudioVisual] MindClearedSound`, the rules-wide release sound
    /// (S-0601 claim 9).
    #[serde(default)]
    pub mind_cleared_sound: Option<String>,
    /// `[AudioVisual] YuriMindControlSound`, the take-over sound (S-0601
    /// claim 5).
    #[serde(default)]
    pub yuri_mind_control_sound: Option<String>,
    /// `[AudioVisual] MasterMindOverloadDeathSound`, the overload sound
    /// (S-0601 claim 16).
    #[serde(default)]
    pub master_mind_overload_death_sound: Option<String>,
    /// `[CombatDamage] ControlledAnimationType`, the control ring (S-0601
    /// claim 18).
    #[serde(default)]
    pub controlled_animation_type: Option<String>,
    /// `[CombatDamage] PermaControlledAnimationType`, the permanent control
    /// ring (S-0601 claim 18).
    #[serde(default)]
    pub perma_controlled_animation_type: Option<String>,
    /// `[CombatDamage] MindControlAttackLineFrames`, the link-line timer in
    /// ticks (S-0601 claim 17); absent: 15.
    #[serde(default)]
    pub mind_control_attack_line_frames: Option<i32>,
    /// `[CombatDamage] OverloadCount`, *count* of S-0601 F2; absent: empty.
    #[serde(default)]
    pub overload_count: Option<Vec<i32>>,
    /// `[CombatDamage] OverloadDamage`, *damage* of S-0601 F2; absent: empty.
    #[serde(default)]
    pub overload_damage: Option<Vec<i32>>,
    /// `[CombatDamage] OverloadFrames`, *frames* of S-0601 F2; absent: empty.
    #[serde(default)]
    pub overload_frames: Option<Vec<i32>>,
    /// `[General] AICaptureNormal`, the normal fate table (S-0601 claim 8):
    /// weights for team, grinder, bio reactor, hunt; absent: empty.
    #[serde(default)]
    pub ai_capture_normal: Option<Vec<i32>>,
    /// `[General] AICaptureWounded` (S-0601 claim 8).
    #[serde(default)]
    pub ai_capture_wounded: Option<Vec<i32>>,
    /// `[General] AICaptureLowPower` (S-0601 claim 8).
    #[serde(default)]
    pub ai_capture_low_power: Option<Vec<i32>>,
    /// `[General] AICaptureLowMoney` (S-0601 claim 8).
    #[serde(default)]
    pub ai_capture_low_money: Option<Vec<i32>>,
    /// `[General] AICaptureLowMoneyMark`, credits (S-0601 claim 8); absent:
    /// none, which never selects the low-money table (claim 8's design rule).
    #[serde(default)]
    pub ai_capture_low_money_mark: Option<i32>,
    /// `[General] AICaptureWoundedMark`, a fraction read as a single
    /// (S-0601 claim 8); absent: none, which never selects the wounded table
    /// (claim 8's design rule).
    #[serde(default)]
    pub ai_capture_wounded_mark: Option<Decimal>,
    /// `[CombatDamage] IvanWarhead`, the bomb warhead (S-0519 Import mapping).
    #[serde(default)]
    pub ivan_warhead: Option<String>,
    /// `[CombatDamage] IvanDamage` (S-0519 Import mapping).
    #[serde(default)]
    pub ivan_damage: Option<i32>,
    /// `[CombatDamage] IvanTimedDelay`, ticks (S-0519 Import mapping).
    #[serde(default)]
    pub ivan_timed_delay: Option<i32>,
    /// `[CombatDamage] IvanIconFlickerRate`, ticks (S-0519 Import mapping).
    #[serde(default)]
    pub ivan_icon_flicker_rate: Option<i32>,
    /// `[CombatDamage] CanDetonateTimeBomb` (S-0519 Import mapping).
    #[serde(default)]
    pub can_detonate_time_bomb: Option<bool>,
    /// `[CombatDamage] CanDetonateDeathBomb` (S-0519 Import mapping).
    #[serde(default)]
    pub can_detonate_death_bomb: Option<bool>,
    /// `[CombatDamage] C4Delay`, minutes, a single (S-0519 Import mapping).
    #[serde(default)]
    pub c4_delay: Option<Decimal>,
    /// `[CombatDamage] FallingDamageMultiplier`, a single (S-0607 F2;
    /// absent: 1.0).
    #[serde(default)]
    pub falling_damage_multiplier: Option<Decimal>,
    /// `[CombatDamage] CurrentStrengthDamage` (S-0607 F2; absent: yes).
    #[serde(default)]
    pub current_strength_damage: Option<bool>,
    /// `[General] PrismType`, the building type that supports and is
    /// supported (S-0518 Import mapping).
    #[serde(default)]
    pub prism_type: Option<String>,
    /// `[General] PrismSupportModifier`: the held whole-number percentage
    /// after the layers' F4 reads (S-0518 Import mapping, F4).
    #[serde(default)]
    pub prism_support_modifier: Option<i32>,
    /// `[General] PrismSupportMax`, the support cap (S-0518 Import mapping).
    #[serde(default)]
    pub prism_support_max: Option<i32>,
    /// `[General] PrismSupportDelay`, the support rearm in ticks (S-0518
    /// Import mapping).
    #[serde(default)]
    pub prism_support_delay: Option<i32>,
    /// `[General] PrismSupportDuration`, the beam duration in ticks (S-0518
    /// Import mapping).
    #[serde(default)]
    pub prism_support_duration: Option<i32>,
    /// `[General] WarpAway`, the completion animation (S-0514 Import
    /// mapping).
    #[serde(default)]
    pub warp_away: Option<String>,
    /// `[General] ChronoSparkle1`, the sparkle animation (S-0514 Import
    /// mapping).
    #[serde(default)]
    pub chrono_sparkle: Option<String>,
    /// `[CombatDamage] OpenToppedWarpDistance`, cells (S-0514 Import
    /// mapping).
    #[serde(default)]
    pub open_topped_warp_distance: Option<i32>,
    /// `[CombatDamage] DrainMoneyFrameDelay`, the payout period *P* in
    /// ticks (S-0608 Import mapping, F1); absent: 100.
    #[serde(default)]
    pub drain_money_frame_delay: Option<i32>,
    /// `[CombatDamage] DrainMoneyAmount`, the payout amount *A* in credits
    /// (S-0608 Import mapping, F2); absent: 0.
    #[serde(default)]
    pub drain_money_amount: Option<i32>,
    /// `[CombatDamage] DrainAnimationType`, the drain animation (S-0608
    /// claim 5); absent: none.
    #[serde(default)]
    pub drain_animation_type: Option<String>,
    /// `[Radiation] RadDurationMultiple`, the duration multiple *M*
    /// (S-0525 Import mapping, F2).
    #[serde(default)]
    pub rad_duration_multiple: Option<i32>,
    /// `[Radiation] RadApplicationDelay`, the application delay *A* in ticks
    /// (S-0525 Import mapping, claim 9).
    #[serde(default)]
    pub rad_application_delay: Option<i32>,
    /// `[Radiation] RadLevelMax`, the level cap (S-0525 Import mapping, F6).
    #[serde(default)]
    pub rad_level_max: Option<i32>,
    /// `[Radiation] RadLevelDelay`, the level delay in ticks (S-0525 Import
    /// mapping, claim 5).
    #[serde(default)]
    pub rad_level_delay: Option<i32>,
    /// `[Radiation] RadLightDelay`, the light delay in ticks (S-0525 Import
    /// mapping, claim 14).
    #[serde(default)]
    pub rad_light_delay: Option<i32>,
    /// `[AudioVisual] AmbientChangeRate`, the normal ambient fade interval
    /// in game minutes (S-0104 claim 4, Import mapping); absent: 0.2.
    #[serde(default)]
    pub ambient_change_rate: Option<Decimal>,
    /// `[AudioVisual] AmbientChangeStep`, the ambient fade step as an
    /// ambient factor (S-0104 claim 4, Import mapping); absent: 0.1.
    #[serde(default)]
    pub ambient_change_step: Option<Decimal>,
    /// `[Radiation] RadLevelFactor`, a double (S-0525 Import mapping, F6).
    #[serde(default)]
    pub rad_level_factor: Option<Decimal>,
    /// `[Radiation] RadLightFactor`, a double (S-0525 Import mapping, F7).
    #[serde(default)]
    pub rad_light_factor: Option<Decimal>,
    /// `[Radiation] RadTintFactor`, a double (S-0525 Import mapping, F7).
    #[serde(default)]
    pub rad_tint_factor: Option<Decimal>,
    /// `[Radiation] RadColor`, a byte triple (S-0525 Import mapping, F7).
    #[serde(default)]
    pub rad_color: Option<[i32; 3]>, // spec: S-0525 Import mapping (RadColor: a byte triple)
    /// `[Radiation] RadSiteWarhead`, a warhead name (S-0525 Import mapping,
    /// claim 10).
    #[serde(default)]
    pub rad_site_warhead: Option<String>,
    /// `[General] WeaponNullifyAnim` (S-0303 claim 15).
    #[serde(default)]
    pub weapon_nullify_anim: Option<String>,
    /// `[General] LightningWarhead` (S-0303 F5).
    #[serde(default)]
    pub lightning_warhead: Option<String>,
    /// `[General] WeatherConBoltExplosion` (S-0303 F5).
    #[serde(default)]
    pub weather_con_bolt_explosion: Option<String>,
    /// `[General] CrewEscape` (S-0312 claim 13), a double.
    #[serde(default)]
    pub crew_escape: Option<Decimal>,
    /// `[General] AlliedCrew`, an infantry type key (S-0312 F4).
    #[serde(default)]
    pub allied_crew: Option<String>,
    /// `[General] SovietCrew` (S-0312 F4).
    #[serde(default)]
    pub soviet_crew: Option<String>,
    /// `[General] ThirdCrew` (S-0312 F4).
    #[serde(default)]
    pub third_crew: Option<String>,
    /// `[General] Technician` (S-0312 F4).
    #[serde(default)]
    pub technician: Option<String>,
    /// `[General] Engineer` (S-0312 claim 18).
    #[serde(default)]
    pub engineer: Option<String>,
    /// `[General] AlliedSurvivorDivisor` (S-0312 F5).
    #[serde(default)]
    pub allied_survivor_divisor: Option<i32>,
    /// `[General] SovietSurvivorDivisor` (S-0312 F5).
    #[serde(default)]
    pub soviet_survivor_divisor: Option<i32>,
    /// `[General] ThirdSurvivorDivisor` (S-0312 F5).
    #[serde(default)]
    pub third_survivor_divisor: Option<i32>,
    /// `[General] ShipSinkingWeight` (S-0312 claim 10).
    #[serde(default)]
    pub ship_sinking_weight: Option<Decimal>,
    /// `[General] RepairRate` r_b (S-0314 F1), a double.
    #[serde(default)]
    pub repair_rate: Option<Decimal>,
    /// `[General] URepairRate` r_u (S-0314 F1).
    #[serde(default)]
    pub u_repair_rate: Option<Decimal>,
    /// `[General] IRepairRate` r_i (S-0314 F1).
    #[serde(default)]
    pub i_repair_rate: Option<Decimal>,
    /// `[General] RepairStep` (S-0314 claim 4).
    #[serde(default)]
    pub repair_step: Option<i32>,
    /// `[General] IRepairStep` (S-0314 claim 4).
    #[serde(default)]
    pub i_repair_step: Option<i32>,
    /// `[General] RepairPercent` (S-0314 F2), a double.
    #[serde(default)]
    pub repair_percent: Option<Decimal>,
    /// `[General] SelfHealInfantryFrames`, ticks (S-0314 claim 10).
    #[serde(default)]
    pub self_heal_infantry_frames: Option<i32>,
    /// `[General] SelfHealUnitFrames`, ticks (S-0314 claim 10).
    #[serde(default)]
    pub self_heal_unit_frames: Option<i32>,
    /// `[General] SelfHealInfantryAmount` (S-0314 claim 10).
    #[serde(default)]
    pub self_heal_infantry_amount: Option<i32>,
    /// `[General] SelfHealUnitAmount` (S-0314 claim 10).
    #[serde(default)]
    pub self_heal_unit_amount: Option<i32>,
    /// `[General] ThreatPerOccupant` (S-0309 Terms, *threat value*).
    #[serde(default)]
    pub threat_per_occupant: Option<i32>,
    /// `[General] EnemyHouseThreatBonus` B (S-0309 F4), a double.
    #[serde(default)]
    pub enemy_house_threat_bonus: Option<Decimal>,
    /// The five `[General] …CoefficientDefault` values, in the order
    /// own-effectiveness, target-effectiveness, special-threat, strength,
    /// distance (S-0309 Import mapping), doubles.
    // spec: S-0309 Import mapping (five game-wide default coefficients)
    #[serde(default)]
    pub coefficient_defaults: Option<[Decimal; 5]>,
    /// `[General] DisabledDisguiseDetectionPercent` (S-0309 claim 25 part g).
    #[serde(default)]
    pub disguise_detection_percent: Option<Vec<i32>>,
    /// `[General] CloakingStages` *N*, the game-wide number of cloaking stages (S-0516 claim 1).
    #[serde(default)]
    pub cloaking_stages: Option<i32>,
    /// `[General] CloakDelay` *d*, minutes, a decimal (S-0516 F3).
    #[serde(default)]
    pub cloak_delay: Option<Decimal>,
    /// `[AudioVisual] CloakSound`, the game-wide cloak sound (S-0516 claim 15).
    #[serde(default)]
    pub cloak_sound: Option<String>,
    /// `[General] AlliedDisguise`, an infantry type name (S-0516 claim 32).
    #[serde(default)]
    pub allied_disguise: Option<String>,
    /// `[General] SovietDisguise`, an infantry type name (S-0516 claim 32).
    #[serde(default)]
    pub soviet_disguise: Option<String>,
    /// `[General] ThirdDisguise`, an infantry type name (S-0516 claim 32).
    #[serde(default)]
    pub third_disguise: Option<String>,
    /// `[General] DefaultMirageDisguises`, terrain-object type names (S-0516 claim 33 (c)).
    #[serde(default)]
    pub default_mirage_disguises: Option<Vec<String>>,
    /// `[General] InfantryBlinkDisguiseTime`, ticks (S-0516 claim 33 (b)).
    #[serde(default)]
    pub infantry_blink_disguise_time: Option<i32>,
    /// `[General] AttackCursorOnDisguise` (S-0516 Import mapping, S-0214 claim 6).
    #[serde(default)]
    pub attack_cursor_on_disguise: Option<bool>,
    /// `[AudioVisual] Gravity` (S-0306 Terms; *g* of S-0503 F6).
    #[serde(default)]
    pub gravity: Option<i32>,
    /// `[General] InfantryVirus`, the virus-death animation name (S-0613
    /// claims 2 and 3).
    #[serde(default)]
    pub infantry_virus: Option<String>,
    /// `[General] WindDirection`, the wind table index 0 … 7 (S-0613 claim
    /// 12); absent −1.
    #[serde(default)]
    pub wind_direction: Option<i32>,
    /// `[General] HoverHeight` *H*ₕ, leptons (S-0503 F6).
    #[serde(default)]
    pub hover_height: Option<i32>,
    /// `[General] HoverBob` *q*, game minutes (S-0503 F6).
    #[serde(default)]
    pub hover_bob: Option<Decimal>,
    /// `[General] HoverBoost`, the boost factor (S-0503 claim 3).
    #[serde(default)]
    pub hover_boost: Option<Decimal>,
    /// `[General] HoverAcceleration` *a*, game minutes (S-0503 F2).
    #[serde(default)]
    pub hover_acceleration: Option<Decimal>,
    /// `[General] HoverBrake` *b*, game minutes (S-0503 F2).
    #[serde(default)]
    pub hover_brake: Option<Decimal>,
    /// `[General] HoverDampen` *k* (S-0503 F6).
    #[serde(default)]
    pub hover_dampen: Option<Decimal>,
    /// `[WallModel] AlliedWallTransparency` (S-0306 claim 22).
    #[serde(default)]
    pub allied_wall_transparency: Option<bool>,
    /// `[ElevationModel] ElevationIncrement` I (S-0307 F2).
    #[serde(default)]
    pub elevation_increment: Option<i32>,
    /// `[ElevationModel] ElevationIncrementBonus` b (S-0307 F2), a double.
    #[serde(default)]
    pub elevation_increment_bonus: Option<Decimal>,
    /// `[ElevationModel] ElevationBonusCap` c (S-0307 F2), a double.
    #[serde(default)]
    pub elevation_bonus_cap: Option<Decimal>,
    /// `[IQ] RepairSell` (S-0314 claim 8).
    #[serde(default)]
    pub iq_repair_sell: Option<i32>,
    /// `[AI] CreditReserve` (S-0314 claim 12).
    #[serde(default)]
    pub credit_reserve: Option<i32>,
    /// `[IQ] SellBack` (S-0314 claim 14).
    #[serde(default)]
    pub iq_sell_back: Option<i32>,
    /// Game-wide economy values (M5, S-0401 to S-0416).
    #[serde(default)]
    pub economy: crate::sim_econ::EconGeneral,
    /// Game-wide superweapon values (M6b, S-0512 and S-0513).
    #[serde(default)]
    pub superweapons: crate::sim_econ::SwGeneral,
    /// `V3RocketType`, the rocket type of set A (aircraft type name) (S-0506 Import mapping).
    #[serde(default)]
    pub v3_rocket_type: Option<String>,
    /// `V3RocketPauseFrames`, the pause time of set A, ticks (S-0506 Import mapping).
    #[serde(default)]
    pub v3_rocket_pause_frames: Option<i32>,
    /// `V3RocketTiltFrames`, the tilt (raise) time of set A, ticks (S-0506 Import mapping).
    #[serde(default)]
    pub v3_rocket_tilt_frames: Option<i32>,
    /// `V3RocketPitchInitial`, the initial pitch of set A, quarter turns, a single (S-0506 Import mapping).
    #[serde(default)]
    pub v3_rocket_pitch_initial: Option<Decimal>,
    /// `V3RocketPitchFinal`, the final pitch of set A, quarter turns, a single (S-0506 Import mapping).
    #[serde(default)]
    pub v3_rocket_pitch_final: Option<Decimal>,
    /// `V3RocketTurnRate`, the pitch turn rate of set A, radians per tick, a single (S-0506 Import mapping).
    #[serde(default)]
    pub v3_rocket_turn_rate: Option<Decimal>,
    /// `V3RocketRaiseRate`, the raise rate of set A, leptons per tick, truncated (S-0506 Import mapping).
    #[serde(default)]
    pub v3_rocket_raise_rate: Option<Decimal>,
    /// `V3RocketAcceleration`, the acceleration of set A, leptons per tick per tick, a single (S-0506 Import mapping).
    #[serde(default)]
    pub v3_rocket_acceleration: Option<Decimal>,
    /// `V3RocketAltitude`, the cruise altitude of set A, leptons (S-0506 Import mapping).
    #[serde(default)]
    pub v3_rocket_altitude: Option<i32>,
    /// `V3RocketDamage`, the damage of set A (S-0506 Import mapping).
    #[serde(default)]
    pub v3_rocket_damage: Option<i32>,
    /// `V3RocketEliteDamage`, the elite damage of set A (S-0506 Import mapping).
    #[serde(default)]
    pub v3_rocket_elite_damage: Option<i32>,
    /// `V3RocketBodyLength`, the body length of set A, leptons (S-0506 Import mapping).
    #[serde(default)]
    pub v3_rocket_body_length: Option<i32>,
    /// `V3RocketLazyCurve`, the lazy-curve flag of set A (S-0506 Import mapping).
    #[serde(default)]
    pub v3_rocket_lazy_curve: Option<bool>,
    /// `DMislType`, the rocket type of set B (aircraft type name) (S-0506 Import mapping).
    #[serde(default)]
    pub d_misl_type: Option<String>,
    /// `DMislPauseFrames`, the pause time of set B, ticks (S-0506 Import mapping).
    #[serde(default)]
    pub d_misl_pause_frames: Option<i32>,
    /// `DMislTiltFrames`, the tilt (raise) time of set B, ticks (S-0506 Import mapping).
    #[serde(default)]
    pub d_misl_tilt_frames: Option<i32>,
    /// `DMislPitchInitial`, the initial pitch of set B, quarter turns, a single (S-0506 Import mapping).
    #[serde(default)]
    pub d_misl_pitch_initial: Option<Decimal>,
    /// `DMislPitchFinal`, the final pitch of set B, quarter turns, a single (S-0506 Import mapping).
    #[serde(default)]
    pub d_misl_pitch_final: Option<Decimal>,
    /// `DMislTurnRate`, the pitch turn rate of set B, radians per tick, a single (S-0506 Import mapping).
    #[serde(default)]
    pub d_misl_turn_rate: Option<Decimal>,
    /// `DMislRaiseRate`, the raise rate of set B, leptons per tick, truncated (S-0506 Import mapping).
    #[serde(default)]
    pub d_misl_raise_rate: Option<Decimal>,
    /// `DMislAcceleration`, the acceleration of set B, leptons per tick per tick, a single (S-0506 Import mapping).
    #[serde(default)]
    pub d_misl_acceleration: Option<Decimal>,
    /// `DMislAltitude`, the cruise altitude of set B, leptons (S-0506 Import mapping).
    #[serde(default)]
    pub d_misl_altitude: Option<i32>,
    /// `DMislDamage`, the damage of set B (S-0506 Import mapping).
    #[serde(default)]
    pub d_misl_damage: Option<i32>,
    /// `DMislEliteDamage`, the elite damage of set B (S-0506 Import mapping).
    #[serde(default)]
    pub d_misl_elite_damage: Option<i32>,
    /// `DMislBodyLength`, the body length of set B, leptons (S-0506 Import mapping).
    #[serde(default)]
    pub d_misl_body_length: Option<i32>,
    /// `DMislLazyCurve`, the lazy-curve flag of set B (S-0506 Import mapping).
    #[serde(default)]
    pub d_misl_lazy_curve: Option<bool>,
    /// `CMislType`, the rocket type of set C (aircraft type name) (S-0506 Import mapping).
    #[serde(default)]
    pub c_misl_type: Option<String>,
    /// `CMislPauseFrames`, the pause time of set C, ticks (S-0506 Import mapping).
    #[serde(default)]
    pub c_misl_pause_frames: Option<i32>,
    /// `CMislTiltFrames`, the tilt (raise) time of set C, ticks (S-0506 Import mapping).
    #[serde(default)]
    pub c_misl_tilt_frames: Option<i32>,
    /// `CMislPitchInitial`, the initial pitch of set C, quarter turns, a single (S-0506 Import mapping).
    #[serde(default)]
    pub c_misl_pitch_initial: Option<Decimal>,
    /// `CMislPitchFinal`, the final pitch of set C, quarter turns, a single (S-0506 Import mapping).
    #[serde(default)]
    pub c_misl_pitch_final: Option<Decimal>,
    /// `CMislTurnRate`, the pitch turn rate of set C, radians per tick, a single (S-0506 Import mapping).
    #[serde(default)]
    pub c_misl_turn_rate: Option<Decimal>,
    /// `CMislRaiseRate`, the raise rate of set C, leptons per tick, truncated (S-0506 Import mapping).
    #[serde(default)]
    pub c_misl_raise_rate: Option<Decimal>,
    /// `CMislAcceleration`, the acceleration of set C, leptons per tick per tick, a single (S-0506 Import mapping).
    #[serde(default)]
    pub c_misl_acceleration: Option<Decimal>,
    /// `CMislAltitude`, the cruise altitude of set C, leptons (S-0506 Import mapping).
    #[serde(default)]
    pub c_misl_altitude: Option<i32>,
    /// `CMislDamage`, the damage of set C (S-0506 Import mapping).
    #[serde(default)]
    pub c_misl_damage: Option<i32>,
    /// `CMislEliteDamage`, the elite damage of set C (S-0506 Import mapping).
    #[serde(default)]
    pub c_misl_elite_damage: Option<i32>,
    /// `CMislBodyLength`, the body length of set C, leptons (S-0506 Import mapping).
    #[serde(default)]
    pub c_misl_body_length: Option<i32>,
    /// `CMislLazyCurve`, the lazy-curve flag of set C (S-0506 Import mapping).
    #[serde(default)]
    pub c_misl_lazy_curve: Option<bool>,
    /// `V3Warhead` (combat-damage section), the warhead of rocket set A (S-0506 Import mapping).
    #[serde(default)]
    pub v3_warhead: Option<String>,
    /// `V3EliteWarhead` (combat-damage section), the elite warhead of rocket set A (S-0506 Import mapping).
    #[serde(default)]
    pub v3_elite_warhead: Option<String>,
    /// `DMislWarhead` (combat-damage section), the warhead of rocket set B (S-0506 Import mapping).
    #[serde(default)]
    pub d_misl_warhead: Option<String>,
    /// `DMislEliteWarhead` (combat-damage section), the elite warhead of rocket set B (S-0506 Import mapping).
    #[serde(default)]
    pub d_misl_elite_warhead: Option<String>,
    /// `CMislWarhead` (combat-damage section), the warhead of rocket set C (S-0506 Import mapping).
    #[serde(default)]
    pub c_misl_warhead: Option<String>,
    /// `CMislEliteWarhead` (combat-damage section), the elite warhead of rocket set C (S-0506 Import mapping).
    #[serde(default)]
    pub c_misl_elite_warhead: Option<String>,
}

/// One land type's row (S-0211 claim 2): multipliers per locomotion class and
/// the buildable flag. A present row was given by some rules layer; its
/// missing keys are 1.0 and *no*.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LandRow {
    /// `Foot`.
    #[serde(default)]
    pub foot: Option<Decimal>,
    /// `Track`.
    #[serde(default)]
    pub track: Option<Decimal>,
    /// `Wheel`.
    #[serde(default)]
    pub wheel: Option<Decimal>,
    /// `Hover`.
    #[serde(default)]
    pub hover: Option<Decimal>,
    /// `Float`.
    #[serde(default)]
    pub float: Option<Decimal>,
    /// `Amphibious`.
    #[serde(default)]
    pub amphibious: Option<Decimal>,
    /// `FloatBeach`.
    #[serde(default)]
    pub float_beach: Option<Decimal>,
    /// `Buildable`.
    #[serde(default)]
    pub buildable: Option<bool>,
}

/// A behaviour's properties (S-0215 claims 6 and 16 to 21).
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BehaviourProps {
    /// `NoThreat`.
    #[serde(default)]
    pub no_threat: Option<bool>,
    /// `Paralyzed`.
    #[serde(default)]
    pub paralysed: Option<bool>,
    /// `Recruitable`.
    #[serde(default)]
    pub recruitable: Option<bool>,
    /// `Retaliate`.
    #[serde(default)]
    pub retaliate: Option<bool>,
    /// `Scatter`.
    #[serde(default)]
    pub scatter: Option<bool>,
    /// `Zombie`.
    #[serde(default)]
    pub zombie: Option<bool>,
    /// `Rate`, game minutes.
    #[serde(default)]
    pub rate: Option<Decimal>,
    /// `AARate`, game minutes (already reset to the rate where S-0215 claim
    /// 21 says).
    #[serde(default)]
    pub aa_rate: Option<Decimal>,
}

/// A country's simulation fields.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Country {
    /// Economy fields (M5).
    #[serde(default)]
    pub econ: crate::sim_econ::CountryEcon,
    /// `SpeedUnitsMult` (S-0207 claim 2), a single.
    #[serde(default)]
    pub speed_units_mult: Option<Decimal>,
    /// `SpeedInfantryMult` (S-0208 F5), a single.
    #[serde(default)]
    pub speed_infantry_mult: Option<Decimal>,
    /// `MultiplayPassive` (S-0009, S-0214).
    #[serde(default)]
    pub multiplay_passive: Option<bool>,
    /// The wall-owner flag (S-0210 claim 14, Import mapping).
    #[serde(default)]
    pub wall_owner: Option<bool>,
    /// `MultiplayPassive` as the base rules alone give it: the passive field
    /// the early phase of start positions sees (S-0009 claim 33).
    #[serde(default)]
    pub multiplay_passive_base: Option<bool>,
    /// `Firepower` φ_c (S-0302 claim 3), a double.
    #[serde(default)]
    pub firepower: Option<Decimal>,
    /// `ROF` κ (S-0301 claim 3), a double.
    #[serde(default)]
    pub rof: Option<Decimal>,
    /// `ArmorInfantryMult` (S-0302 F2), a single.
    #[serde(default)]
    pub armor_infantry_mult: Option<Decimal>,
    /// `ArmorUnitsMult` (S-0302 F2), a single.
    #[serde(default)]
    pub armor_units_mult: Option<Decimal>,
    /// `ArmorAircraftMult` (S-0302 F2), a single.
    #[serde(default)]
    pub armor_aircraft_mult: Option<Decimal>,
    /// `ArmorBuildingsMult` (S-0302 F2), a single.
    #[serde(default)]
    pub armor_buildings_mult: Option<Decimal>,
    /// `ArmorDefensesMult` (S-0302 F2), a single.
    #[serde(default)]
    pub armor_defenses_mult: Option<Decimal>,
    /// `VeteranInfantry`: infantry type keys (S-0311 claim 11).
    #[serde(default)]
    pub veteran_infantry: Option<Vec<String>>,
    /// `VeteranUnits`: vehicle type keys (S-0311 claim 11).
    #[serde(default)]
    pub veteran_units: Option<Vec<String>>,
    /// `VeteranAircraft`: aircraft type keys (S-0311 claim 11).
    #[serde(default)]
    pub veteran_aircraft: Option<Vec<String>>,
}

/// One sequence entry of an infantry type (S-0107 claim 1).
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SequenceEntry {
    /// Start image frame S.
    #[serde(default)]
    pub start: i32,
    /// Frame count C.
    #[serde(default)]
    pub count: i32,
    /// Facing multiplier M.
    #[serde(default)]
    pub multiplier: i32,
    /// Declared direction, 0 = N … 7 = NW.
    #[serde(default)]
    pub direction: Option<u8>,
}

/// An infantry, vehicle or aircraft type's simulation fields.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UnitType {
    /// `Strength`.
    #[serde(default)]
    pub strength: Option<i32>,
    /// `Speed`, the speed field v (S-0207 claim 1, S-0208 F4).
    #[serde(default)]
    pub speed: Option<i32>,
    /// `ROT`, the turn-rate field (S-0206 claim 6).
    #[serde(default)]
    pub rot: Option<i32>,
    /// `JumpjetTurnRate` (S-0206 claim 14).
    #[serde(default)]
    pub jumpjet_turn_rate: Option<i32>,
    /// `JumpjetSpeed`, the jumpjet speed *S*, an integer in leptons per
    /// tick (S-0502 Import mapping, absent 14).
    #[serde(default)]
    pub jumpjet_speed: Option<i32>,
    /// `JumpjetClimb`, the climb rate *c*, a single-precision decimal
    /// (S-0502 Import mapping, absent 5.0).
    #[serde(default)]
    pub jumpjet_climb: Option<Decimal>,
    /// `JumpjetCrash`, the crash rate *k*, a single-precision decimal
    /// (S-0502 Import mapping, absent 5.0).
    #[serde(default)]
    pub jumpjet_crash: Option<Decimal>,
    /// `JumpjetHeight`, the jumpjet cruise height *H* in leptons (S-0502
    /// Import mapping; S-0102 owns its minimum and default).
    #[serde(default)]
    pub jumpjet_height: Option<i32>,
    /// `JumpjetAccel`, the acceleration *a*, a single-precision decimal
    /// (S-0502 Import mapping, absent 2.0). A key written with different
    /// letter case, such as `JumpJetAccel`, does not set it (S-0001 claim 13).
    #[serde(default)]
    pub jumpjet_accel: Option<Decimal>,
    /// `JumpjetWobbles`, the wobble rate *W*, a single-precision decimal
    /// (S-0502 Import mapping, absent 0.15).
    #[serde(default)]
    pub jumpjet_wobbles: Option<Decimal>,
    /// `JumpjetDeviation`, the wobble amplitude *d* in leptons (S-0502
    /// Import mapping, absent 40).
    #[serde(default)]
    pub jumpjet_deviation: Option<i32>,
    /// `JumpjetNoWobbles`, the no-wobble flag (S-0502 Import mapping).
    #[serde(default)]
    pub jumpjet_no_wobbles: Option<bool>,
    /// `DeployToLand`, the landing-first flag of an in-place deploying
    /// vehicle (S-0502 Import mapping, S-0517 Import mapping).
    #[serde(default)]
    pub deploy_to_land: Option<bool>,
    /// `DeployingAnim`, the deploying presentation: an animation type name
    /// (S-0517 Import mapping, claims 4, 6 and 12).
    #[serde(default)]
    pub deploying_anim: Option<String>,
    /// `UnloadingClass`, the deployed image type: a vehicle type name
    /// (S-0517 Import mapping, claim 12).
    #[serde(default)]
    pub unloading_class: Option<String>,
    /// `DeployTime`, game minutes: read, with no effect on any vehicle
    /// (S-0517 claim 15).
    #[serde(default)]
    pub deploy_time: Option<Decimal>,
    /// `Turret`.
    #[serde(default)]
    pub turret: Option<bool>,
    /// `TurretSpins`.
    #[serde(default)]
    pub turret_spins: Option<bool>,
    /// The locomotion class as S-0211 claim 3 leaves it; absent: no class.
    #[serde(default)]
    pub locomotion_class: Option<LocomotionClass>,
    /// `MovementZone` (S-0211 claim 4).
    #[serde(default)]
    pub movement_zone: Option<MovementZone>,
    /// `Locomotor`, the locomotor its class identifier selects (S-0207
    /// claim 14); absent: the teleporting locomotor.
    #[serde(default)]
    pub locomotor: Option<Locomotor>,
    /// `MovementRestrictedTo` (S-0211 claim 14); absent: no restriction.
    #[serde(default)]
    pub movement_restricted_to: Option<LandType>,
    /// `Crusher`.
    #[serde(default)]
    pub crusher: Option<bool>,
    /// `OmniCrusher`.
    #[serde(default)]
    pub omni_crusher: Option<bool>,
    /// `OmniCrushResistant`.
    #[serde(default)]
    pub omni_crush_resistant: Option<bool>,
    /// `Crushable`.
    #[serde(default)]
    pub crushable: Option<bool>,
    /// `DeployedCrushable`.
    #[serde(default)]
    pub deployed_crushable: Option<bool>,
    /// `Accelerates`.
    #[serde(default)]
    pub accelerates: Option<bool>,
    /// `AccelerationFactor`.
    #[serde(default)]
    pub acceleration_factor: Option<Decimal>,
    /// `DeaccelerationFactor`.
    #[serde(default)]
    pub deacceleration_factor: Option<Decimal>,
    /// `SlowdownDistance`, leptons.
    #[serde(default)]
    pub slowdown_distance: Option<i32>,
    /// `Passive`.
    #[serde(default)]
    pub passive: Option<bool>,
    /// `IsTrain`.
    #[serde(default)]
    pub is_train: Option<bool>,
    /// `IsSimpleDeployer`.
    #[serde(default)]
    pub is_simple_deployer: Option<bool>,
    /// `Harvester`.
    #[serde(default)]
    pub harvester: Option<bool>,
    /// `Weeder`.
    #[serde(default)]
    pub weeder: Option<bool>,
    /// `JumpJet`.
    #[serde(default)]
    pub jumpjet: Option<bool>,
    /// `Fraidycat`.
    #[serde(default)]
    pub fraidycat: Option<bool>,
    /// `Cyborg`.
    #[serde(default)]
    pub cyborg: Option<bool>,
    /// `Fearless`.
    #[serde(default)]
    pub fearless: Option<bool>,
    /// `NotHuman`.
    #[serde(default)]
    pub not_human: Option<bool>,
    /// `DeadBodies`: the type's corpse list, animation names (S-0107
    /// claim 17 and Import mapping; absent empty).
    #[serde(default)]
    pub dead_bodies: Option<Vec<String>>,
    /// `DeathAnims`: the type's death animations, entry *w* for death kind
    /// *w*, every listed entry kept in its position (S-0107 claim 17 row 3
    /// and Import mapping; absent empty).
    #[serde(default)]
    pub death_anims: Option<Vec<String>>,
    /// `VehicleThief`.
    #[serde(default)]
    pub vehicle_thief: Option<bool>,
    /// `Engineer`.
    #[serde(default)]
    pub engineer: Option<bool>,
    /// `Occupier`.
    #[serde(default)]
    pub occupier: Option<bool>,
    /// `Assaulter`.
    #[serde(default)]
    pub assaulter: Option<bool>,
    /// `Infiltrate`.
    #[serde(default)]
    pub infiltrate: Option<bool>,
    /// `Agent`.
    #[serde(default)]
    pub agent: Option<bool>,
    /// `Thief`, the money-thief flag (S-0309 claim 5, Import mapping).
    #[serde(default)]
    pub thief: Option<bool>,
    /// `C4`.
    #[serde(default)]
    pub c4: Option<bool>,
    /// `Ivan`.
    #[serde(default)]
    pub ivan: Option<bool>,
    /// `Doggie`, the animal marker: no effect of its own (S-0519 claim 1).
    #[serde(default)]
    pub doggie: Option<bool>,
    /// `Deployer`.
    #[serde(default)]
    pub deployer: Option<bool>,
    /// `DeployFire`.
    #[serde(default)]
    pub deploy_fire: Option<bool>,
    /// `UndeployDelay`, ticks.
    #[serde(default)]
    pub undeploy_delay: Option<i32>,
    /// `DefaultToGuardArea`.
    #[serde(default)]
    pub default_to_guard_area: Option<bool>,
    /// `StupidHunt`.
    #[serde(default)]
    pub stupid_hunt: Option<bool>,
    /// `HoverAttack`.
    #[serde(default)]
    pub hover_attack: Option<bool>,
    /// `CloseRange`.
    #[serde(default)]
    pub close_range: Option<bool>,
    /// `CanApproachTarget`, the can-approach property (S-0215 claim 37 d).
    #[serde(default)]
    pub can_approach_target: Option<bool>,
    /// `DistributedFire`.
    #[serde(default)]
    pub distributed_fire: Option<bool>,
    /// `OpportunityFire`.
    #[serde(default)]
    pub opportunity_fire: Option<bool>,
    /// `OpenTopped`.
    #[serde(default)]
    pub open_topped: Option<bool>,
    /// `NonVehicle`.
    #[serde(default)]
    pub non_vehicle: Option<bool>,
    /// `PreventAttackMove`.
    #[serde(default)]
    pub prevent_attack_move: Option<bool>,
    /// `VoiceSelect`: the type's selection responses, in the listed order,
    /// every entry kept (S-0214 Terms, F5).
    #[serde(default)]
    pub voice_select: Option<Vec<String>>,
    /// `MoveToShroud`.
    #[serde(default)]
    pub move_to_shroud: Option<bool>,
    /// `Pushy`.
    #[serde(default)]
    pub pushy: Option<bool>,
    /// `PermaDisguise`.
    #[serde(default)]
    pub perma_disguise: Option<bool>,
    /// `AttackFriendlies`.
    #[serde(default)]
    pub attack_friendlies: Option<bool>,
    /// `AttackCursorOnFriendlies`.
    #[serde(default)]
    pub attack_cursor_on_friendlies: Option<bool>,
    /// `SmallVisceroid`.
    #[serde(default)]
    pub small_visceroid: Option<bool>,
    /// `LargeVisceroid` (S-0009 claim 46, S-0202 Import mapping).
    #[serde(default)]
    pub large_visceroid: Option<bool>,
    /// `Cost`, credits (S-0009 claim 45).
    #[serde(default)]
    pub cost: Option<i32>,
    /// `TechLevel` (S-0009 claim 45).
    #[serde(default)]
    pub tech_level: Option<i32>,
    /// `AllowedToStartInMultiplayer` (S-0009 claim 45).
    #[serde(default)]
    pub allowed_to_start_in_multiplayer: Option<bool>,
    /// `Owner`: the countries that own the type, as country keys (S-0009
    /// claim 45; unknown names dropped by the importer, S-0004 claim 3).
    #[serde(default)]
    pub owner: Option<Vec<String>>,
    /// `Immune`.
    #[serde(default)]
    pub immune: Option<bool>,
    /// `LegalTarget`.
    #[serde(default)]
    pub legal_target: Option<bool>,
    /// `Bombable`.
    #[serde(default)]
    pub bombable: Option<bool>,
    /// `ImmuneToRadiation`.
    #[serde(default)]
    pub immune_to_radiation: Option<bool>,
    /// `Warpable`, the erasable-target flag (S-0514 Import mapping); absent:
    /// yes.
    #[serde(default)]
    pub warpable: Option<bool>,
    /// `BalloonHover`.
    #[serde(default)]
    pub balloon_hover: Option<bool>,
    /// `Bunkerable`, the bunkerable flag of a tank bunker's entrant (S-0606
    /// claim 1 and Import mapping); absent: on for vehicle types, off for
    /// infantry and aircraft types.
    #[serde(default)]
    pub bunkerable: Option<bool>,
    /// `SprayAttack`.
    #[serde(default)]
    pub spray_attack: Option<bool>,
    /// `TiltsWhenCrushes`.
    #[serde(default)]
    pub tilts_when_crushes: Option<bool>,
    /// `AutoCrush`.
    #[serde(default)]
    pub auto_crush: Option<bool>,
    /// `Carryall`.
    #[serde(default)]
    pub carryall: Option<bool>,
    /// `Spawned`.
    #[serde(default)]
    pub spawned: Option<bool>,
    /// `GuardRange`, cells.
    #[serde(default)]
    pub guard_range: Option<Decimal>,
    /// `Sight`, cells.
    #[serde(default)]
    pub sight: Option<i32>,
    /// `Passengers`.
    #[serde(default)]
    pub passengers: Option<i32>,
    /// `Size`, the size a passenger counts (S-0507 Import mapping).
    #[serde(default)]
    pub size: Option<Decimal>,
    /// `SizeLimit`, the largest size of one passenger a transport takes
    /// (S-0507 Import mapping).
    #[serde(default)]
    pub size_limit: Option<Decimal>,
    /// `IsDropship`, the dropship flag of an air transport (S-0507 Import
    /// mapping).
    #[serde(default)]
    pub is_dropship: Option<bool>,
    /// `EnterTransportSound`, a sound name (S-0507 Import mapping).
    #[serde(default)]
    pub enter_transport_sound: Option<String>,
    /// `LeaveTransportSound`, a sound name (S-0507 Import mapping).
    #[serde(default)]
    pub leave_transport_sound: Option<String>,
    /// `Storage`.
    #[serde(default)]
    pub storage: Option<i32>,
    /// `ThreatAvoidanceCoefficient`, a single.
    #[serde(default)]
    pub threat_avoidance_coefficient: Option<Decimal>,
    /// `DeploysInto`.
    #[serde(default)]
    pub deploys_into: Option<String>,
    /// `ChronoOutSound`, the type's departure sound of a teleporting jump
    /// (S-0207 claim 20 e).
    #[serde(default)]
    pub chrono_out_sound: Option<String>,
    /// `ChronoInSound`, the type's arrival sound of a teleporting jump
    /// (S-0207 claim 20 e).
    #[serde(default)]
    pub chrono_in_sound: Option<String>,
    /// `Primary`.
    #[serde(default)]
    pub primary: Option<String>,
    /// `Secondary`.
    #[serde(default)]
    pub secondary: Option<String>,
    /// `ElitePrimary`.
    #[serde(default)]
    pub elite_primary: Option<String>,
    /// `EliteSecondary`.
    #[serde(default)]
    pub elite_secondary: Option<String>,
    /// `TurretCount`.
    #[serde(default)]
    pub turret_count: Option<i32>,
    /// `WeaponCount`.
    #[serde(default)]
    pub weapon_count: Option<i32>,
    /// `VeteranAbilities`, upper-case ability names.
    #[serde(default)]
    pub veteran_abilities: Vec<String>,
    /// `EliteAbilities`, upper-case ability names.
    #[serde(default)]
    pub elite_abilities: Vec<String>,
    /// Art `Crawls` (S-0107 Import mapping).
    #[serde(default)]
    pub crawls: Option<bool>,
    /// Art sequence entries by our kind name (S-0107 claim 1).
    #[serde(default)]
    pub sequences: BTreeMap<String, SequenceEntry>,
    /// Art fire frames: standing, prone, secondary, secondary-prone (S-0107
    /// claim 11).
    // spec: S-0107 claim 11 (four fire-frame fields)
    #[serde(default)]
    pub fire_frames: Option<[i32; 4]>,
    /// `Teleporter`: a self-teleporting type (S-0214 claim 11).
    #[serde(default)]
    pub teleporter: Option<bool>,
    /// `Selectable`: the type permits selection (S-0214 Terms and claim
    /// 16; absent: yes).
    #[serde(default)]
    pub selectable: Option<bool>,
    /// The combat fields shared with buildings (M4).
    #[serde(default)]
    pub combat: CombatFields,
    /// Economy fields (M5).
    #[serde(default)]
    pub econ: crate::sim_econ::EconFields,
    /// `BurstDelay0` … `BurstDelay3` (S-0301 claim 4), ticks; −1: none.
    // spec: S-0301 Constants (four burst-delay fields)
    #[serde(default)]
    pub burst_delay: Option<[i32; 4]>,
    /// Art `FiringFrames` F (S-0301 claim 13).
    #[serde(default)]
    pub firing_frames: Option<i32>,
    /// Art `FiringSyncFrame0`, `FiringSyncFrame1` (S-0301 claim 13).
    // spec: S-0301 Constants (two firing sync frames)
    #[serde(default)]
    pub firing_sync_frames: Option<[i32; 2]>,
    /// `HunterSeeker` (S-0301 claim 9).
    #[serde(default)]
    pub hunter_seeker: Option<bool>,
    /// `TargetLaser`, the target-line flag (S-0301 claim 24; only a vehicle
    /// shows the line); absent: no.
    #[serde(default)]
    pub target_laser: Option<bool>,
    /// `MobileFire` (S-0301 claim 11, S-0310 claim 7).
    #[serde(default)]
    pub mobile_fire: Option<bool>,
    /// `Gunner` (S-0308 claim 6).
    #[serde(default)]
    pub gunner: Option<bool>,
    /// `IFVMode` (S-0308 claim 6).
    #[serde(default)]
    pub ifv_mode: Option<i32>,
    /// `OpenTransportWeapon` (S-0308 claims 8, 10).
    #[serde(default)]
    pub open_transport_weapon: Option<i32>,
    /// `DeployFireWeapon` (S-0308 claims 8, 9).
    #[serde(default)]
    pub deploy_fire_weapon: Option<i32>,
    /// `OccupyWeapon` (S-0308 claim 13).
    #[serde(default)]
    pub occupy_weapon: Option<String>,
    /// `EliteOccupyWeapon` (S-0308 claim 13).
    #[serde(default)]
    pub elite_occupy_weapon: Option<String>,
    /// `DeployToFire` (S-0310 claim 9).
    #[serde(default)]
    pub deploy_to_fire: Option<bool>,
    /// `JumpJetTurn` (S-0310 claim 8).
    #[serde(default)]
    pub jump_jet_turn: Option<bool>,
    /// `Crewed` (S-0312 claim 13, F4).
    #[serde(default)]
    pub crewed: Option<bool>,
    /// `CarriesCrate` (S-0312 claim 14).
    #[serde(default)]
    pub carries_crate: Option<bool>,
    /// `CrateGoodie`: a vehicle type a unit crate may give (S-0520 claim
    /// 13).
    #[serde(default)]
    pub crate_goodie: Option<bool>,
    /// `Crashable` (S-0312 claims 12, 15).
    #[serde(default)]
    pub crashable: Option<bool>,
    /// `Weight` (S-0312 claim 10).
    #[serde(default)]
    pub weight: Option<Decimal>,
    /// `SuppressionThreshold` (S-0315 claim 6).
    #[serde(default)]
    pub suppression_threshold: Option<i32>,
    /// `MissileSpawn` (S-0311 claim 8).
    #[serde(default)]
    pub missile_spawn: Option<bool>,
    /// Art `Trailer` of an aircraft type's image section, the trailer
    /// animation of a rocket missile (S-0506 claim 21, S-0004 claim 14).
    #[serde(default)]
    pub trailer: Option<String>,
    /// `Parasiteable`, the can-carry-a-parasite flag (S-0614 claim 2); absent: yes.
    #[serde(default)]
    pub parasiteable: Option<bool>,
    /// `ReselectIfLimboed`, the re-select flag (S-0614 claim 19); absent: no.
    #[serde(default)]
    pub reselect_if_limboed: Option<bool>,
}

/// Combat fields that infantry, vehicle, aircraft and building types share
/// (M4). Each field's spec is in its doc comment.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CombatFields {
    /// `Armor`, the armour class (S-0305; the importer resolves the word).
    #[serde(default)]
    pub armor: Option<ArmourClass>,
    /// The weapon slots 0 … 17 of a multi-weapon type as `Weapon1` …
    /// `Weapon18` fill them (S-0308 claim 1), the empty string for a slot
    /// with no weapon; absent: slots 0 and 1 come from `primary` and
    /// `secondary`.
    #[serde(default)]
    pub weapon_slots: Option<Vec<String>>,
    /// The elite slots of a multi-weapon type (S-0308 claim 1).
    #[serde(default)]
    pub elite_weapon_slots: Option<Vec<String>>,
    /// `IsGattling` (S-0308 claim 7).
    #[serde(default)]
    pub is_gattling: Option<bool>,
    /// `WeaponStages`, the gattling stage count *N* (S-0609 Import mapping).
    #[serde(default)]
    pub weapon_stages: Option<i32>,
    /// `RateUp`, the gattling spin-up amount *u* (S-0609 Import mapping).
    #[serde(default)]
    pub rate_up: Option<i32>,
    /// `RateDown`, the gattling spin-down amount *d* (S-0609 Import
    /// mapping).
    #[serde(default)]
    pub rate_down: Option<i32>,
    /// `Stage1` … `Stage6`, the normal gattling thresholds *K*₁ … as the
    /// layers leave them (S-0609 Import mapping: a layer reads them only
    /// with the gattling flag and a stage count of at least 2).
    #[serde(default)]
    pub stage_thresholds: Option<Vec<i32>>,
    /// `EliteStage1` … `EliteStage6`, the elite gattling thresholds
    /// (S-0609 Import mapping; absent: 0, not the normal threshold).
    #[serde(default)]
    pub elite_stage_thresholds: Option<Vec<i32>>,
    /// `IsChargeTurret` (S-0301 claim 16, S-0308 claim 5).
    #[serde(default)]
    pub is_charge_turret: Option<bool>,
    /// `LandTargeting` (S-0308 rows 13, 18).
    #[serde(default)]
    pub land_targeting: Option<i32>,
    /// `NavalTargeting` (S-0308 table N).
    #[serde(default)]
    pub naval_targeting: Option<i32>,
    /// `Underwater` (S-0308 table N, S-0303 claim 21).
    #[serde(default)]
    pub underwater: Option<bool>,
    /// `Cloakable`: whether an underwater type can be submerged (S-0504 Terms; S-0516, wave 2).
    #[serde(default)]
    pub cloakable: Option<bool>,
    /// `CloakStop`, the cloak-stop flag (S-0516 claim 3).
    #[serde(default)]
    pub cloak_stop: Option<bool>,
    /// `CloakingSpeed` *S*, ticks per stage (S-0516 claim 1).
    #[serde(default)]
    pub cloaking_speed: Option<i32>,
    /// `Sensors`, the sensors flag (S-0516 claims 13 and 18).
    #[serde(default)]
    pub sensors: Option<bool>,
    /// `SensorsSight` *R*, cells (S-0516 claims 22 and 23).
    #[serde(default)]
    pub sensors_sight: Option<i32>,
    /// `CanDisguise` (S-0516 claims 30 to 34).
    #[serde(default)]
    pub can_disguise: Option<bool>,
    /// `DisguiseWhenStill` (S-0516 claim 33).
    #[serde(default)]
    pub disguise_when_still: Option<bool>,
    /// `Organic` (S-0308 table N, S-0314 claim 10).
    #[serde(default)]
    pub organic: Option<bool>,
    /// `Unnatural` (S-0308 table N).
    #[serde(default)]
    pub unnatural: Option<bool>,
    /// `Naval` (S-0308 row 13, S-0303 claim 21, S-0312 claim 10).
    #[serde(default)]
    pub naval: Option<bool>,
    /// `Ammo` A (S-0301 claim 10).
    #[serde(default)]
    pub ammo: Option<i32>,
    /// `InitialAmmo` (S-0301 claim 10).
    #[serde(default)]
    pub initial_ammo: Option<i32>,
    /// `Reload` L (S-0301 F3).
    #[serde(default)]
    pub reload: Option<i32>,
    /// `EmptyReload` E (S-0301 F3).
    #[serde(default)]
    pub empty_reload: Option<i32>,
    /// `ReloadIncrement` I (S-0301 F3).
    #[serde(default)]
    pub reload_increment: Option<i32>,
    /// `PipWrap` W (S-0301 F3).
    #[serde(default)]
    pub pip_wrap: Option<i32>,
    /// `ManualReload` (S-0301 claim 11).
    #[serde(default)]
    pub manual_reload: Option<bool>,
    /// `DamageReducesReadiness` (S-0301 claim 17, S-0302 claim 10).
    #[serde(default)]
    pub damage_reduces_readiness: Option<bool>,
    /// `ReadinessReductionMultiplier` κ (S-0302 F6), a single.
    #[serde(default)]
    pub readiness_reduction_multiplier: Option<Decimal>,
    /// `AirRangeBonus`, cells (S-0307 claim 3; lepton form).
    #[serde(default)]
    pub air_range_bonus: Option<Decimal>,
    /// `RadialFireSegments` (S-0306 claims 2, 7).
    #[serde(default)]
    pub radial_fire_segments: Option<i32>,
    /// `TypeImmune` (S-0302 claim 8).
    #[serde(default)]
    pub type_immune: Option<bool>,
    /// `ImmuneToPsionicWeapons` (S-0302 claim 8). Absent from every layer:
    /// yes for building types, no for infantry, vehicle and aircraft types
    /// (S-0610 claim 1).
    #[serde(default)]
    pub immune_to_psionic_weapons: Option<bool>,
    /// `ImmuneToPoison` (S-0302 claim 8).
    #[serde(default)]
    pub immune_to_poison: Option<bool>,
    /// `ImmuneToPsionics` (S-0302 claim 10). Absent from every layer: yes
    /// for building types, no for infantry, vehicle and aircraft types
    /// (S-0610 claim 1).
    #[serde(default)]
    pub immune_to_psionics: Option<bool>,
    /// `BerserkFriendly`: a berserk unit may not fire at objects of this
    /// type (S-0610 claim 6); absent: no.
    #[serde(default)]
    pub berserk_friendly: Option<bool>,
    /// `DamageSelf` (S-0302 claim 8, S-0303 claim 9).
    #[serde(default)]
    pub damage_self: Option<bool>,
    /// `DamageParticleSystems`: the particle-system names that are
    /// eligible (behaviour *smoke*, S-0302 claim 13), in list order.
    #[serde(default)]
    pub damage_smoke_systems: Option<Vec<String>>,
    /// `DamageSmokeOffset`, the damage-smoke displacement in leptons per
    /// axis (S-0616 claim 1 and Import mapping; absent 0, 0, 0).
    #[serde(default)]
    pub damage_smoke_offset: Option<[i32; 3]>, // spec: S-0616 Import mapping (three integers, leptons)
    /// `DamageParticleSystems`: the particle-system names of the spark
    /// kind, in list order (S-0616 claim 1, damage sparks).
    #[serde(default)]
    pub damage_spark_systems: Option<Vec<String>>,
    /// `NaturalParticleSystem`, a particle-system name (S-0616 claim 1,
    /// natural smoke; absent none).
    #[serde(default)]
    pub natural_particle_system: Option<String>,
    /// `NaturalParticleLocation`, the natural-smoke displacement in leptons
    /// per axis (S-0616 Import mapping; absent 0, 0, 0).
    #[serde(default)]
    pub natural_particle_location: Option<[i32; 3]>, // spec: S-0616 Import mapping (three integers, leptons)
    /// `DestroySmokeOffset`, the destruction smoke displacement in leptons
    /// per axis (S-0312 claim 19 f and Import mapping; absent 0, 0, 0).
    #[serde(default)]
    pub destroy_smoke_offset: Option<[i32; 3]>, // spec: S-0312 Import mapping (three integers in leptons)
    /// `MaxDebris` X (S-0312 F1).
    #[serde(default)]
    pub max_debris: Option<i32>,
    /// `MinDebris` Y (S-0312 F1).
    #[serde(default)]
    pub min_debris: Option<i32>,
    /// `DebrisTypes` (S-0312 F1).
    #[serde(default)]
    pub debris_types: Option<Vec<String>>,
    /// `DebrisMaximums` (S-0312 F1).
    #[serde(default)]
    pub debris_maximums: Option<Vec<i32>>,
    /// `DebrisAnims` (S-0312 F1).
    #[serde(default)]
    pub debris_anims: Option<Vec<String>>,
    /// `Explodes` (S-0312 Terms).
    #[serde(default)]
    pub explodes: Option<bool>,
    /// `DeathWeapon` (S-0312 claim 6).
    #[serde(default)]
    pub death_weapon: Option<String>,
    /// `DeathWeaponDamageModifier` φ (S-0312 F3), a single.
    #[serde(default)]
    pub death_weapon_damage_modifier: Option<Decimal>,
    /// `Explosion`, animation names (S-0312 claims 9, 16, 19).
    #[serde(default)]
    pub explosion: Option<Vec<String>>,
    /// `DestroyAnim`, animation names (S-0312 claims 9, 19).
    #[serde(default)]
    pub destroy_anim: Option<Vec<String>>,
    /// `DestroyParticleSystems`, particle-system names, `none` and
    /// `<none>` omitted (S-0312 claim 19 f; S-0003 claim 19, row C18).
    #[serde(default)]
    pub destroy_particle_systems: Option<Vec<String>>,
    /// `Trainable` (S-0311 claim 8).
    #[serde(default)]
    pub trainable: Option<bool>,
    /// `DontScore` (S-0311 claim 7).
    #[serde(default)]
    pub dont_score: Option<bool>,
    /// `SelfHealing` (S-0314 claim 9).
    #[serde(default)]
    pub self_healing: Option<bool>,
    /// `CanPassiveAquire` (S-0309 claim 2).
    #[serde(default)]
    pub can_passive_aquire: Option<bool>,
    /// `CanRetaliate` (S-0309 claim 18).
    #[serde(default)]
    pub can_retaliate: Option<bool>,
    /// `DetectDisguise` (S-0309 table C).
    #[serde(default)]
    pub detect_disguise: Option<bool>,
    /// `NoAutoFire` (S-0309 claim 5).
    #[serde(default)]
    pub no_auto_fire: Option<bool>,
    /// `VHPScan`: `none`, `normal` or `strong` (S-0309 claim 14).
    #[serde(default)]
    pub vhp_scan: Option<String>,
    /// `SpecialThreatValue` (S-0309 F4), a double.
    #[serde(default)]
    pub special_threat_value: Option<Decimal>,
    /// The five threat coefficients, in the order own-effectiveness,
    /// target-effectiveness, special-threat, strength, distance (S-0309
    /// F4), doubles; an entry given as 0 already replaced by the importer
    /// where S-0309 Edge cases say.
    // spec: S-0309 Import mapping (five threat coefficients)
    #[serde(default)]
    pub coefficients: Option<[Decimal; 5]>,
    /// `Insignificant` (S-0309 table C).
    #[serde(default)]
    pub insignificant: Option<bool>,
    /// `ThreatPosed` (S-0309 Terms).
    #[serde(default)]
    pub threat_posed: Option<i32>,
    /// `AirstrikeTeam` (S-0309 claim 24).
    #[serde(default)]
    pub airstrike_team: Option<i32>,
    /// `EliteAirstrikeTeam`, the elite strike size (S-0519 Import mapping).
    #[serde(default)]
    pub elite_airstrike_team: Option<i32>,
    /// `AirstrikeTeamType`, the strike aircraft type (S-0519 Import mapping).
    #[serde(default)]
    pub airstrike_team_type: Option<String>,
    /// `EliteAirstrikeTeamType` (S-0519 Import mapping).
    #[serde(default)]
    pub elite_airstrike_team_type: Option<String>,
    /// `AirstrikeRechargeTime`, ticks (S-0519 Import mapping).
    #[serde(default)]
    pub airstrike_recharge_time: Option<i32>,
    /// `EliteAirstrikeRechargeTime`, ticks (S-0519 Import mapping).
    #[serde(default)]
    pub elite_airstrike_recharge_time: Option<i32>,
    /// `BombSight`, cells (S-0519 Import mapping).
    #[serde(default)]
    pub bomb_sight: Option<i32>,
    /// `Drainable` (S-0308 row 8).
    #[serde(default)]
    pub drainable: Option<bool>,
    /// `Spawns`, the spawned aircraft type (S-0506 Import mapping).
    #[serde(default)]
    pub spawns: Option<String>,
    /// `SpawnsNumber`, the spawn count (S-0506 Import mapping).
    #[serde(default)]
    pub spawns_number: Option<i32>,
    /// `SpawnRegenRate`, the regeneration time in ticks (S-0506 Import mapping).
    #[serde(default)]
    pub spawn_regen_rate: Option<i32>,
    /// `SpawnReloadRate`, the reload time in ticks (S-0506 Import mapping).
    #[serde(default)]
    pub spawn_reload_rate: Option<i32>,
    /// Art `PrimaryFireFLH` (S-0313).
    #[serde(default)]
    pub primary_fire_flh: Option<Flh>,
    /// Art `SecondaryFireFLH` (S-0313).
    #[serde(default)]
    pub secondary_fire_flh: Option<Flh>,
    /// Art `ElitePrimaryFireFLH` (S-0313); absent, the primary slot's
    /// muzzle position (claim 1a).
    #[serde(default)]
    pub elite_primary_fire_flh: Option<Flh>,
    /// Art `EliteSecondaryFireFLH` (S-0313); absent, the secondary slot's
    /// muzzle position (claim 1a).
    #[serde(default)]
    pub elite_secondary_fire_flh: Option<Flh>,
    /// Art `Weapon`n`FLH`, by slot (S-0313).
    #[serde(default)]
    pub weapon_flh: Option<Vec<Flh>>,
    /// Art `EliteWeapon`n`FLH`, by slot (S-0313); an absent entry holds
    /// weapon n's muzzle position as imported (claim 1a).
    #[serde(default)]
    pub elite_weapon_flh: Option<Vec<Flh>>,
    /// Art `TurretOffset` δ (S-0313).
    #[serde(default)]
    pub turret_offset: Option<i32>,
    /// Art `AlternateFLH0` … `AlternateFLH4`, the five alternate link
    /// anchors (S-0601 claim 17, F5), by index; a missing one is the zero
    /// triple.
    #[serde(default)]
    pub alternate_flh: Option<Vec<Flh>>,
    /// `MindClearedSound`, the type's release sound (S-0601 claim 9).
    #[serde(default)]
    pub mind_cleared_sound: Option<String>,
    /// `LeptonMindControlOffset`, the link-line end height in leptons
    /// (S-0601 claim 17); absent: 70.
    #[serde(default)]
    pub lepton_mind_control_offset: Option<i32>,
    /// `MindControlRingOffset`, the mobile ring height in leptons (S-0601
    /// claim 18); absent: 140.
    #[serde(default)]
    pub mind_control_ring_offset: Option<i32>,
}

/// A building type's simulation fields.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BuildingType {
    /// `Overpowerable`, the type that charging infantry overpower (S-0518
    /// Import mapping); absent: no.
    #[serde(default)]
    pub overpowerable: Option<bool>,
    /// `CloakGenerator` (S-0516 claims 19 and 20).
    #[serde(default)]
    pub cloak_generator: Option<bool>,
    /// `CloakRadiusInCells` *r*, cells, kept as a signed 8-bit value (S-0516 claims 19, 20 and 23).
    #[serde(default)]
    pub cloak_radius_in_cells: Option<i32>,
    /// `SensorArray` (S-0516 claim 23).
    #[serde(default)]
    pub sensor_array: Option<bool>,
    /// `DetectDisguiseRange`, cells (S-0516 claim 40).
    #[serde(default)]
    pub detect_disguise_range: Option<i32>,
    /// `Strength`.
    #[serde(default)]
    pub strength: Option<i32>,
    /// `TurretCount`: above 0 makes a multi-weapon type (S-0308 claim 1
    /// and Import mapping, a type section key; S-0609 Terms for a gattling
    /// building).
    #[serde(default)]
    pub turret_count: Option<i32>,
    /// Art `Foundation`, width and height in cells.
    // nocite: a pair of numbers (width, height), our format's shape
    #[serde(default)]
    pub foundation: Option<[i32; 2]>,
    /// Art `Foundation` is the 8-cell 3 × 3 word `3x3Refinery`, which lacks
    /// the cell (2, 1) (S-0406 claim 2, S-0302 claim 13); absent: no.
    #[serde(default)]
    pub foundation_refinery: Option<bool>,
    /// `Bib` (S-0204 Terms).
    #[serde(default)]
    pub bib: Option<bool>,
    /// `Gate`.
    #[serde(default)]
    pub gate: Option<bool>,
    /// `InvisibleInGame`.
    #[serde(default)]
    pub invisible_in_game: Option<bool>,
    /// `Selectable`: the type permits selection (S-0214 Terms and claim
    /// 16; absent: yes).
    #[serde(default)]
    pub selectable: Option<bool>,
    /// `LaserFence`.
    #[serde(default)]
    pub laser_fence: Option<bool>,
    /// `FirestormWall`.
    #[serde(default)]
    pub firestorm_wall: Option<bool>,
    /// `Naval`.
    #[serde(default)]
    pub naval: Option<bool>,
    /// `Bunker`.
    #[serde(default)]
    pub bunker: Option<bool>,
    /// `UnitRepair`.
    #[serde(default)]
    pub unit_repair: Option<bool>,
    /// `Grinding`: a recycling building (S-0603 Terms and Import mapping;
    /// absent: no).
    #[serde(default)]
    pub grinding: Option<bool>,
    /// `UnitAbsorb`.
    #[serde(default)]
    pub unit_absorb: Option<bool>,
    /// `InfantryAbsorb`.
    #[serde(default)]
    pub infantry_absorb: Option<bool>,
    /// `Passengers` of a building type: an absorbing building's capacity
    /// *c* (S-0605 F1 and Import mapping; absent: 0).
    #[serde(default)]
    pub passengers: Option<i32>,
    /// `SizeLimit` of a building type: an absorbing building's size limit
    /// *L*, floating point (S-0605 F1 and Import mapping; absent: 0.0).
    #[serde(default)]
    pub size_limit: Option<Decimal>,
    /// `BridgeRepairHut`.
    #[serde(default)]
    pub bridge_repair_hut: Option<bool>,
    /// `NumberImpassableRows` (S-0209 Terms); absent: none.
    #[serde(default)]
    pub number_impassable_rows: Option<i32>,
    /// `Factory`, the imported factory category in our words: the kind it
    /// builds (`infantry`, `vehicle`, `aircraft`, `building`) or a
    /// recognised category that builds nothing (`terrain`,
    /// `infantry_object`, `vehicle_object`, `aircraft_object`,
    /// `building_object`); the importer maps the original's words by
    /// S-0003 claim 21.
    #[serde(default)]
    pub factory: Option<String>,
    /// `WeaponsFactory`.
    #[serde(default)]
    pub weapons_factory: Option<bool>,
    /// `ConstructionYard`.
    #[serde(default)]
    pub construction_yard: Option<bool>,
    /// `UndeploysInto`.
    #[serde(default)]
    pub undeploys_into: Option<String>,
    /// `Cloning`.
    #[serde(default)]
    pub cloning: Option<bool>,
    /// `Powered`.
    #[serde(default)]
    pub powered: Option<bool>,
    /// `TogglePower`.
    #[serde(default)]
    pub toggle_power: Option<bool>,
    /// `VoiceSelect`: the type's selection responses, in the listed order,
    /// every entry kept (S-0214 Terms, F5).
    #[serde(default)]
    pub voice_select: Option<Vec<String>>,
    /// `CanBeOccupied`.
    #[serde(default)]
    pub can_be_occupied: Option<bool>,
    /// `CanOccupyFire`.
    #[serde(default)]
    pub can_occupy_fire: Option<bool>,
    /// `Capturable`.
    #[serde(default)]
    pub capturable: Option<bool>,
    /// `ImmuneToRadiation` (S-0302 claim 8).
    #[serde(default)]
    pub immune_to_radiation: Option<bool>,
    /// `Warpable`, the erasable-target flag (S-0514 Import mapping); absent:
    /// yes.
    #[serde(default)]
    pub warpable: Option<bool>,
    /// `NeedsEngineer` (S-0309 claim 25 part e, Import mapping).
    #[serde(default)]
    pub needs_engineer: Option<bool>,
    /// `ProduceCashStartup`, the acquisition bonus in credits (S-0515
    /// claim 3, Import mapping).
    #[serde(default)]
    pub produce_cash_startup: Option<i32>,
    /// `ProduceCashAmount`, the income amount in credits (S-0515 claim 5).
    #[serde(default)]
    pub produce_cash_amount: Option<i32>,
    /// `ProduceCashDelay`, the income delay in ticks (S-0515 claims 4 and 5).
    #[serde(default)]
    pub produce_cash_delay: Option<i32>,
    /// `Storage`, storage capacity (S-0309 table C, row "storage").
    #[serde(default)]
    pub storage: Option<i32>,
    /// `Spyable`.
    #[serde(default)]
    pub spyable: Option<bool>,
    /// `CanC4`.
    #[serde(default)]
    pub can_c4: Option<bool>,
    /// `Repairable`.
    #[serde(default)]
    pub repairable: Option<bool>,
    /// `Hospital`.
    #[serde(default)]
    pub hospital: Option<bool>,
    /// `Armory`.
    #[serde(default)]
    pub armory: Option<bool>,
    /// `SAM`.
    #[serde(default)]
    pub sam: Option<bool>,
    /// `EMPulseCannon`.
    #[serde(default)]
    pub emp_cannon: Option<bool>,
    /// `HasStupidGuardMode`.
    #[serde(default)]
    pub stupid_guard_mode: Option<bool>,
    /// `Turret`.
    #[serde(default)]
    pub turret: Option<bool>,
    /// `ROT`.
    #[serde(default)]
    pub rot: Option<i32>,
    /// `Primary`.
    #[serde(default)]
    pub primary: Option<String>,
    /// `Secondary`.
    #[serde(default)]
    pub secondary: Option<String>,
    /// `TechLevel`.
    #[serde(default)]
    pub tech_level: Option<i32>,
    /// `GapGenerator`.
    #[serde(default)]
    pub gap_generator: Option<bool>,
    /// `Immune`.
    #[serde(default)]
    pub immune: Option<bool>,
    /// `LegalTarget`.
    #[serde(default)]
    pub legal_target: Option<bool>,
    /// `Bombable`, accepts bombs (S-0214 Import mapping, row I19); absent:
    /// yes.
    #[serde(default)]
    pub bombable: Option<bool>,
    /// `Crushable`.
    #[serde(default)]
    pub crushable: Option<bool>,
    /// `DeployTime`, game minutes: a gate's door time (S-0215 F9).
    #[serde(default)]
    pub deploy_time: Option<Decimal>,
    /// `GateCloseDelay`, game minutes: a gate's close delay (S-0215 F9).
    #[serde(default)]
    pub gate_close_delay: Option<Decimal>,
    /// The combat fields shared with mobile types (M4).
    #[serde(default)]
    pub combat: CombatFields,
    /// Economy fields (M5).
    #[serde(default)]
    pub econ: crate::sim_econ::EconFields,
    /// `Owner`, the owner countries (S-0407 claims 1 and 11), country keys.
    #[serde(default)]
    pub owner: Option<Vec<String>>,
    /// `Cost`, credits (S-0311 F1, S-0314 claim 5).
    #[serde(default)]
    pub cost: Option<i32>,
    /// `BuildCat` is `Combat`: a defence building (S-0302 claim 5).
    #[serde(default)]
    pub defense: Option<bool>,
    /// `EligibleForDelayKill` (S-0302 claim 16).
    #[serde(default)]
    pub eligible_for_delay_kill: Option<bool>,
    /// `CrateBeneath` (S-0312 claim 20).
    #[serde(default)]
    pub crate_beneath: Option<bool>,
    /// `CrateBeneathIsMoney` (S-0312 claim 20).
    #[serde(default)]
    pub crate_beneath_is_money: Option<bool>,
    /// `LeaveRubble` (S-0312 claim 20).
    #[serde(default)]
    pub leave_rubble: Option<bool>,
    /// `Helipad` (S-0312 claim 17).
    #[serde(default)]
    pub helipad: Option<bool>,
    /// `Crewed` (S-0312 claim 18 and F5; S-0408 claim 7).
    #[serde(default)]
    pub crewed: Option<bool>,
    /// `Artillary` (S-0309 claim 18).
    #[serde(default)]
    pub artillary: Option<bool>,
    /// `MaxNumberOccupants` (S-0313 claim 6).
    #[serde(default)]
    pub max_number_occupants: Option<i32>,
    /// Art `MuzzleFlash`*n* for *n* from 0 to the occupant capacity − 1: the
    /// garrison muzzle points, pixel shifts (S-0313 claim 6, Import mapping;
    /// a point the art does not give is (0, 0)).
    #[serde(default)]
    pub garrison_muzzles: Option<Vec<[i32; 2]>>, // nocite: pixel pairs, our shape
    /// `InfantryGainSelfHeal` (S-0314 claim 10).
    #[serde(default)]
    pub infantry_gain_self_heal: Option<i32>,
    /// `UnitsGainSelfHeal` (S-0314 claim 10).
    #[serde(default)]
    pub units_gain_self_heal: Option<i32>,
    /// Art `TargetCoordOffset`, leptons (S-0306 Terms, aim shift).
    // spec: S-0306 Import mapping (an integer triple)
    #[serde(default)]
    pub target_coord_offset: Option<[i32; 3]>,
    /// `RefinerySmokeParticleSystem`, a particle-system name (S-0616 claim
    /// 1, refinery smoke; absent none).
    #[serde(default)]
    pub refinery_smoke_particle_system: Option<String>,
    /// `RefinerySmokeOffsetOne` … `RefinerySmokeOffsetFour`, the smoke
    /// points in leptons; a point not given is written as (0, 0, 0), which
    /// counts as unset (S-0616 claim 1 and Import mapping).
    #[serde(default)]
    pub refinery_smoke_offsets: Option<Vec<[i32; 3]>>, // spec: S-0616 Import mapping (three integers each, leptons; absent unset)
    /// Art `Height` H_b (S-0306 claim 6 c).
    #[serde(default)]
    pub art_height: Option<i32>,
    /// `TurretAnimIsVoxel` of the building type's rules section (S-0310
    /// claim 4, S-0313 claims 8 and 10; art has no effect).
    #[serde(default)]
    pub turret_anim_is_voxel: Option<bool>,
    /// `BarrelAnimIsVoxel` of the building type's rules section (S-0313
    /// claims 8 and 10).
    #[serde(default)]
    pub barrel_anim_is_voxel: Option<bool>,
    /// Art `PrimaryFirePixelOffset`, pixels (S-0313 claim 7).
    #[serde(default)]
    pub primary_fire_pixel_offset: Option<[i32; 2]>, // nocite: a pixel pair, our shape
    /// Art `PrimaryFireDualOffset` (S-0313 claim 7).
    #[serde(default)]
    pub primary_fire_dual_offset: Option<bool>,
    /// `TurretAnimX`, `TurretAnimY` of the building type's rules section,
    /// pixels (S-0313 claims 8 and 10, S-0310 claim 4).
    #[serde(default)]
    pub turret_anim_xy: Option<[i32; 2]>, // nocite: a pixel pair, our shape
    /// Art `IsAnimDelayedFire` (S-0301 claim 14).
    #[serde(default)]
    pub is_anim_delayed_fire: Option<bool>,
    /// Art `DelayedFireDelay`, ticks (S-0301 claim 14).
    #[serde(default)]
    pub delayed_fire_delay: Option<i32>,
}

/// An overlay type's simulation fields.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OverlayType {
    /// `Land`; absent: clear (S-0211 claim 4; `<none>` is read as clear by
    /// the importer, with a warning).
    #[serde(default)]
    pub land: Option<LandType>,
    /// `NoUseTileLandType`.
    #[serde(default)]
    pub no_use_tile_land_type: Option<bool>,
    /// `Wall`.
    #[serde(default)]
    pub wall: Option<bool>,
    /// `Tiberium`.
    #[serde(default)]
    pub tiberium: Option<bool>,
    /// `Crushable`.
    #[serde(default)]
    pub crushable: Option<bool>,
    /// `IsARock`.
    #[serde(default)]
    pub rock: Option<bool>,
    /// `IsRubble`.
    #[serde(default)]
    pub rubble: Option<bool>,
    /// `Crate`.
    #[serde(default, rename = "crate")]
    pub crate_: Option<bool>,
    /// `CrateTrigger` (S-0520 claim 7).
    #[serde(default)]
    pub crate_trigger: Option<bool>,
    /// `DamageLevels` of the overlay type's image section in the art data
    /// (S-0210 claim 13).
    #[serde(default)]
    pub damage_levels: Option<i32>,
    /// `Strength`.
    #[serde(default)]
    pub strength: Option<i32>,
    /// `Armor`, the armour name as written.
    #[serde(default)]
    pub armor: Option<String>,
    /// Whether the type has artwork or a cell animation (S-0009 claim 12),
    /// as the importer's art lookup found.
    #[serde(default)]
    pub has_art: Option<bool>,
    /// `IsVeins`, the veins flag (S-0202 Terms).
    #[serde(default)]
    pub is_veins: Option<bool>,
    /// `ChainReaction` (S-0303 claim 17).
    #[serde(default)]
    pub chain_reaction: Option<bool>,
    /// `Explodes` (S-0303 claim 19).
    #[serde(default)]
    pub explodes: Option<bool>,
    /// `LegalTarget`, a legal target (S-0214 Import mapping: the attackable
    /// cell); absent: yes.
    #[serde(default)]
    pub legal_target: Option<bool>,
    /// `CellAnim`, the overlay type's cell animation (S-0009 claim 12 and
    /// Import mapping): an animation type key; absent: none.
    #[serde(default)]
    pub cell_anim: Option<String>,
}

/// A terrain-object type's simulation fields.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TerrainObjectType {
    /// `TemperateOccupationBits`.
    #[serde(default)]
    pub temperate_occupation_bits: Option<i32>,
    /// `SnowOccupationBits`.
    #[serde(default)]
    pub snow_occupation_bits: Option<i32>,
    /// `SpawnsTiberium`, the spawns-resources flag (S-0203 claim 2).
    #[serde(default)]
    pub spawns_tiberium: Option<bool>,
    /// `IsAnimated`, the animated flag of an ore mine (S-0401 claim 15).
    #[serde(default)]
    pub is_animated: Option<bool>,
    /// `AnimationRate`, *R* in ticks (S-0401 claim 15); absent: 0.
    #[serde(default)]
    pub animation_rate: Option<i32>,
    /// `AnimationProbability`, *p*, a single (S-0401 F8); absent: 0.
    #[serde(default)]
    pub animation_probability: Option<Decimal>,
    /// The number of image frames *N* of the type's sprite (S-0401 claim
    /// 15, S-0112 Terms *frame count*), which the importer reads from the
    /// sprite of S-0007 claim 13 for the map's theater; absent (no sprite,
    /// a file that does not parse, or an import without a theater): an
    /// animated ore mine never makes resource, as for a sprite of fewer
    /// than 2 image frames (S-0401 claim 15).
    #[serde(default)]
    pub image_frames: Option<i32>,
    /// `Strength` (S-0302 claim 20), with the game-wide tree strength
    /// already applied by the importer (S-0102 claim 23); absent: unset,
    /// which a placed object starts with as health −1 (S-0009 claim 42).
    #[serde(default)]
    pub strength: Option<i32>,
    /// `Armor`, the armour class (S-0305).
    #[serde(default)]
    pub armor: Option<ArmourClass>,
    /// `Immune` (S-0302 claim 20).
    #[serde(default)]
    pub immune: Option<bool>,
    /// `LegalTarget`, a legal target (S-0214 Import mapping, claim 6 (a));
    /// absent: yes.
    #[serde(default)]
    pub legal_target: Option<bool>,
    /// `Bombable`, accepts bombs (S-0214 Import mapping, row I19); absent:
    /// yes.
    #[serde(default)]
    pub bombable: Option<bool>,
}

/// A weapon's simulation fields.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WeaponType {
    /// `DisguiseFakeBlinkTime`, ticks (S-0516 claim 39).
    #[serde(default)]
    pub disguise_fake_blink_time: Option<i32>,
    /// `InfiniteMindControl`, the unlimited-control flag (S-0601 claim 1);
    /// absent: no.
    #[serde(default)]
    pub infinite_mind_control: Option<bool>,
    /// `Damage`.
    #[serde(default)]
    pub damage: Option<i32>,
    /// `AmbientDamage`.
    #[serde(default)]
    pub ambient_damage: Option<i32>,
    /// `Projectile`.
    #[serde(default)]
    pub projectile: Option<String>,
    /// `Warhead`.
    #[serde(default)]
    pub warhead: Option<String>,
    /// `OmniFire`.
    #[serde(default)]
    pub omni_fire: Option<bool>,
    /// `AreaFire`.
    #[serde(default)]
    pub area_fire: Option<bool>,
    /// `RadLevel`.
    #[serde(default)]
    pub rad_level: Option<i32>,
    /// `ROF` R, ticks (S-0301).
    #[serde(default)]
    pub rof: Option<i32>,
    /// `Burst` B (S-0301 claim 4).
    #[serde(default)]
    pub burst: Option<i32>,
    /// `Speed`, the imported speed field v (S-0306 Import mapping).
    #[serde(default)]
    pub speed: Option<i32>,
    /// `Range`, cells (S-0307 claim 1; lepton form).
    #[serde(default)]
    pub range: Option<Decimal>,
    /// `MinimumRange`, cells (S-0307 claim 1).
    #[serde(default)]
    pub minimum_range: Option<Decimal>,
    /// `CellRangefinding` (S-0307 claim 8).
    #[serde(default)]
    pub cell_rangefinding: Option<bool>,
    /// `IsSonic` (S-0301 claim 5, S-0302 claim 2).
    #[serde(default)]
    pub is_sonic: Option<bool>,
    /// `UseSparkParticles` (S-0301 claim 5).
    #[serde(default)]
    pub use_spark_particles: Option<bool>,
    /// `UseFireParticles` (S-0301 claim 5, S-0302 claim 2).
    #[serde(default)]
    pub use_fire_particles: Option<bool>,
    /// `IsRailgun` (S-0301 claim 5).
    #[serde(default)]
    pub is_railgun: Option<bool>,
    /// `IsMagBeam` (S-0301 claim 5).
    #[serde(default)]
    pub is_mag_beam: Option<bool>,
    /// `IsElectricBolt`, the electric-bolt flag (S-0316 claims 1 to 13);
    /// absent: no.
    #[serde(default)]
    pub is_electric_bolt: Option<bool>,
    /// `IsAlternateColor`, the alternate-colour flag that only picks the
    /// colour of a bolt's side strands (S-0316 claim 10); absent: no.
    #[serde(default)]
    pub is_alternate_color: Option<bool>,
    /// `IsLaser`, the laser flag that wins over the electric-bolt flag
    /// (S-0316 claim 2, S-0514 claim 23); absent: no.
    #[serde(default)]
    pub is_laser: Option<bool>,
    /// `IsRadBeam`, the radiation-beam flag that the electric-bolt flag
    /// wins over (S-0316 claim 2, S-0514 claim 23); absent: no.
    #[serde(default)]
    pub is_rad_beam: Option<bool>,
    /// `Spawner` (S-0301 claim 9).
    #[serde(default)]
    pub spawner: Option<bool>,
    /// `Camera`, the camera-weapon flag (S-0501 Terms, *visiting
    /// aircraft*); absent: no.
    #[serde(default)]
    pub camera: Option<bool>,
    /// `DiskLaser` (S-0301 claim 8; S-0615 claim 12).
    #[serde(default)]
    pub disk_laser: Option<bool>,
    /// `LaserDuration`, the disc laser's final beam duration in ticks
    /// (S-0615 claim 16); absent: 10.
    #[serde(default)]
    pub laser_duration: Option<i32>,
    /// `LaserInnerColor`, three colour channels (S-0615 claim 15); absent:
    /// (0, 0, 0).
    #[serde(default)]
    pub laser_inner_color: Option<[i32; 3]>, // spec: S-0615 Import mapping (three integer colour channels, S-0003)
    /// `LaserOuterColor`, three colour channels (S-0615 claim 15); absent:
    /// (0, 0, 0).
    #[serde(default)]
    pub laser_outer_color: Option<[i32; 3]>, // spec: S-0615 Import mapping (three integer colour channels, S-0003)
    /// `LaserOuterSpread`, three colour channels (S-0615 claim 15);
    /// absent: (0, 0, 0).
    #[serde(default)]
    pub laser_outer_spread: Option<[i32; 3]>, // spec: S-0615 Import mapping (three integer colour channels, S-0003)
    /// `IsHouseColor`, the house-colour flag (S-0615 claim 15); absent:
    /// no.
    #[serde(default)]
    pub is_house_color: Option<bool>,
    /// `DecloakToFire` (S-0301 claim 12).
    #[serde(default)]
    pub decloak_to_fire: Option<bool>,
    /// `Lobber` (S-0306 claim 6 a).
    #[serde(default)]
    pub lobber: Option<bool>,
    /// `NeverUse` (S-0308 claim 4).
    #[serde(default)]
    pub never_use: Option<bool>,
    /// `DrainWeapon` (S-0308 row 8).
    #[serde(default)]
    pub drain_weapon: Option<bool>,
    /// `FireWhileMoving` (S-0310 claim 7).
    #[serde(default)]
    pub fire_while_moving: Option<bool>,
    /// `Suicide` (S-0312 Terms).
    #[serde(default)]
    pub suicide: Option<bool>,
    /// `MigAttackCursor`, the air-strike cursor flag: cursor only (S-0519
    /// claim 16, S-0214 row I18).
    #[serde(default)]
    pub mig_attack_cursor: Option<bool>,
    /// `Supress` (S-0309 claim 14).
    #[serde(default)]
    pub supress: Option<bool>,
    /// `FireOnce` (S-0309 claim 21).
    #[serde(default)]
    pub fire_once: Option<bool>,
    /// `RevealOnFire`, the reveal-on-fire flag (S-0409 claim 16); absent: yes
    /// (S-0409 Import mapping, revision 16).
    #[serde(default)]
    pub reveal_on_fire: Option<bool>,
    /// `LimboLaunch`, the launch-inside flag (S-0614 Terms); absent: no.
    #[serde(default)]
    pub limbo_launch: Option<bool>,
    /// `OccupantAnim`, the animation shown where an assault removes an
    /// occupant (S-0508 claim 14, Import mapping), and a garrisoned
    /// building's shot animation (S-0301 claim 20 b).
    #[serde(default)]
    pub occupant_anim: Option<String>,
    /// `Anim`, animation names: the per-direction delivery animation
    /// (S-0614 claim 9) and the shot animation list (S-0301 claim 20 a);
    /// absent: empty.
    #[serde(default)]
    pub anim: Option<Vec<String>>,
    /// `OpenToppedAnim`, the open-topped animation of a passenger firing
    /// out (S-0301 claim 20 c); absent: none.
    #[serde(default)]
    pub open_topped_anim: Option<String>,
    /// `AssaultAnim`, the assault animation, which ordinary shots never use
    /// (S-0301 Terms); absent: none.
    #[serde(default)]
    pub assault_anim: Option<String>,
    /// `AttachedParticleSystem`, the effect weapon's emitter type, a
    /// particle-system name (S-0616 claim 1; absent none).
    #[serde(default)]
    pub attached_particle_system: Option<String>,
}

/// A projectile's simulation fields.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProjectileType {
    /// `ROT`.
    #[serde(default)]
    pub rot: Option<i32>,
    /// `AG`.
    #[serde(default)]
    pub ag: Option<bool>,
    /// `AA` (S-0306 claim 23).
    #[serde(default)]
    pub aa: Option<bool>,
    /// `Arcing` (S-0306 Terms).
    #[serde(default)]
    pub arcing: Option<bool>,
    /// `Vertical` (S-0306 Terms).
    #[serde(default)]
    pub vertical: Option<bool>,
    /// `Inviso` (S-0306 Terms).
    #[serde(default)]
    pub inviso: Option<bool>,
    /// `Inaccurate` (S-0306 claim 4).
    #[serde(default)]
    pub inaccurate: Option<bool>,
    /// `FlakScatter` (S-0306 claim 4).
    #[serde(default)]
    pub flak_scatter: Option<bool>,
    /// `Dropping` (S-0306 claim 5).
    #[serde(default)]
    pub dropping: Option<bool>,
    /// `Airburst` (S-0306 claim 21).
    #[serde(default)]
    pub airburst: Option<bool>,
    /// `VeryHigh` (S-0306 claim 15).
    #[serde(default)]
    pub very_high: Option<bool>,
    /// `Level` (S-0306 T1).
    #[serde(default)]
    pub level: Option<bool>,
    /// `Floater` (S-0306 Terms).
    #[serde(default)]
    pub floater: Option<bool>,
    /// `SubjectToCliffs` (S-0306 claim 22).
    #[serde(default)]
    pub subject_to_cliffs: Option<bool>,
    /// `SubjectToWalls` (S-0306 claim 22).
    #[serde(default)]
    pub subject_to_walls: Option<bool>,
    /// `SubjectToElevation` (S-0307 claim 6).
    #[serde(default)]
    pub subject_to_elevation: Option<bool>,
    /// `Ranged` (S-0306 Terms).
    #[serde(default)]
    pub ranged: Option<bool>,
    /// `Degenerates` (S-0306 claim 18).
    #[serde(default)]
    pub degenerates: Option<bool>,
    /// `Scalable` (S-0306 claim 16).
    #[serde(default)]
    pub scalable: Option<bool>,
    /// `Acceleration` α (S-0306 claims 11, 12).
    #[serde(default)]
    pub acceleration: Option<i32>,
    /// `CourseLockDuration` κ (S-0306 claim 12).
    #[serde(default)]
    pub course_lock_duration: Option<i32>,
    /// `Arm`, ticks (S-0306 claim 9).
    #[serde(default)]
    pub arm: Option<i32>,
    /// `DetonationAltitude`, leptons (S-0306 claim 11).
    #[serde(default)]
    pub detonation_altitude: Option<i32>,
    /// `Cluster` n_c (S-0306 claim 21).
    #[serde(default)]
    pub cluster: Option<i32>,
    /// `AirburstWeapon` (S-0306 claim 21).
    #[serde(default)]
    pub airburst_weapon: Option<String>,
    /// `ShrapnelWeapon`, the fragment weapon (S-0318 Import mapping).
    #[serde(default)]
    pub shrapnel_weapon: Option<String>,
    /// `ShrapnelCount`, the fragment count c (S-0318 Import mapping).
    #[serde(default)]
    pub shrapnel_count: Option<i32>,
    /// `Trailer`, an animation name (S-0306 claim 16).
    #[serde(default)]
    pub trailer: Option<String>,
    /// `SpawnDelay`, ticks (S-0306 claim 16).
    #[serde(default)]
    pub spawn_delay: Option<i32>,
    /// Art `Voxel`: drawn as a voxel model (S-0306 claim 6 b).
    #[serde(default)]
    pub voxel: Option<bool>,
}

/// A warhead's simulation fields.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WarheadType {
    /// `MakesDisguise` (S-0516 claim 31).
    #[serde(default)]
    pub makes_disguise: Option<bool>,
    /// `NukeMaker`, the missile-maker flag (S-0513 claim 8).
    #[serde(default)]
    pub nuke_maker: Option<bool>,
    /// `Wall`.
    #[serde(default)]
    pub wall: Option<bool>,
    /// `Wood`.
    #[serde(default)]
    pub wood: Option<bool>,
    /// `IvanBomb`.
    #[serde(default)]
    pub ivan_bomb: Option<bool>,
    /// `BombDisarm`, the disarm warhead (S-0519 Import mapping).
    #[serde(default)]
    pub bomb_disarm: Option<bool>,
    /// `Sonic`.
    #[serde(default)]
    pub sonic: Option<bool>,
    /// `Parasite`, the parasite flag (S-0614 Terms); absent: no.
    #[serde(default)]
    pub parasite: Option<bool>,
    /// `Paralyzes`, the paralysis length P in ticks (S-0614 claims 6, 7); absent: 0.
    #[serde(default)]
    pub paralyzes: Option<i32>,
    /// `Culling`, the cull flag (S-0614 claim 11); absent: no.
    #[serde(default)]
    pub culling: Option<bool>,
    /// `Bright`, the flash permission of the demolition warhead (S-0524
    /// claim 6, I2); absent: false.
    #[serde(default)]
    pub bright: Option<bool>,
    /// `CombatLightSize`, the flash-size fraction (S-0524 claim 6, I2), a
    /// single; absent: 0.
    #[serde(default)]
    pub combat_light_size: Option<Decimal>,
    /// `Temporal`, the warhead's temporal flag (S-0301 Terms and Import
    /// mapping, S-0514 Import mapping); absent: no.
    #[serde(default)]
    pub temporal: Option<bool>,
    /// `Fire`.
    #[serde(default)]
    pub fire: Option<bool>,
    /// `ElectricAssault`.
    #[serde(default)]
    pub electric_assault: Option<bool>,
    /// `InfDeath`.
    #[serde(default)]
    pub inf_death: Option<i32>,
    /// `Verses`: the eleven versus values in class-index order (S-0003
    /// claim 13, S-0305), each written as the exact decimal of its double,
    /// without `%` (the importer has applied claim 13's `%` readings);
    /// validation refuses an item with `%`.
    #[serde(default)]
    pub verses: Option<Vec<Decimal>>,
    /// `CellSpread` S, cells (S-0303), a single.
    #[serde(default)]
    pub cell_spread: Option<Decimal>,
    /// `PercentAtMax` p (S-0303), a single.
    #[serde(default)]
    pub percent_at_max: Option<Decimal>,
    /// `ProneDamage` P (S-0302 F2), a double.
    #[serde(default)]
    pub prone_damage: Option<Decimal>,
    /// `PenetratesBunker` (S-0302 claim 8).
    #[serde(default)]
    pub penetrates_bunker: Option<bool>,
    /// `Radiation` (S-0302 claim 8).
    #[serde(default)]
    pub radiation: Option<bool>,
    /// `PsychicDamage` (S-0302 claim 8).
    #[serde(default)]
    pub psychic_damage: Option<bool>,
    /// `Poison` (S-0302 claim 8).
    #[serde(default)]
    pub poison: Option<bool>,
    /// `AffectsAllies` (S-0302 claim 8).
    #[serde(default)]
    pub affects_allies: Option<bool>,
    /// `Psychedelic`: the berserk warhead (S-0302 claim 10).
    #[serde(default)]
    pub psychedelic: Option<bool>,
    /// `Sparky` (S-0302 claim 13).
    #[serde(default)]
    pub sparky: Option<bool>,
    /// `CausesDelayKill` (S-0302 claim 16).
    #[serde(default)]
    pub causes_delay_kill: Option<bool>,
    /// `DelayKillFrames` F (S-0302 F7).
    #[serde(default)]
    pub delay_kill_frames: Option<i32>,
    /// `DelayKillAtMax` α (S-0302 F7), a single.
    #[serde(default)]
    pub delay_kill_at_max: Option<Decimal>,
    /// `WallAbsoluteDestroyer` (S-0303 claim 16).
    #[serde(default)]
    pub wall_absolute_destroyer: Option<bool>,
    /// `Tiberium`, the resource flag (S-0303 claim 17).
    #[serde(default)]
    pub tiberium: Option<bool>,
    /// `Rocker` (S-0303 claim 20).
    #[serde(default)]
    pub rocker: Option<bool>,
    /// `Particle`, a particle-system name (S-0303 claim 20).
    #[serde(default)]
    pub particle: Option<String>,
    /// `Conventional` (S-0303 F5).
    #[serde(default)]
    pub conventional: Option<bool>,
    /// `EMEffect` (S-0303 F5, S-0306 claim 20 c).
    #[serde(default)]
    pub em_effect: Option<bool>,
    /// `AnimList`, animation names (S-0303 F5).
    #[serde(default)]
    pub anim_list: Option<Vec<String>>,
    /// `MaxDebris` X (S-0303 F6).
    #[serde(default)]
    pub max_debris: Option<i32>,
    /// `MinDebris` Y (S-0303 F6).
    #[serde(default)]
    pub min_debris: Option<i32>,
    /// `DebrisTypes` (S-0303 F6).
    #[serde(default)]
    pub debris_types: Option<Vec<String>>,
    /// `DebrisMaximums` (S-0303 F6).
    #[serde(default)]
    pub debris_maximums: Option<Vec<i32>>,
    /// `Airstrike` (S-0308 row 6).
    #[serde(default)]
    pub airstrike: Option<bool>,
    /// `IsLocomotor` (S-0308 row 7).
    #[serde(default)]
    pub is_locomotor: Option<bool>,
    /// `Locomotor`, the locomotor a seizing warhead installs (S-0607
    /// Import mapping): the name of S-0207 table T3's class identifier
    /// (absent: the teleporting locomotor).
    #[serde(default)]
    pub locomotor: Option<String>,
    /// `MindControl`, the takeover selection of S-0514 claim 20 (absent: no).
    #[serde(default)]
    pub mind_control: Option<bool>,
    /// `directrocker`, the direct-rocking selection of S-0514 claim 20
    /// (absent: no).
    #[serde(default)]
    pub direct_rocker: Option<bool>,
}
