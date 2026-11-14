//! The simulation's view of a resolved ruleset: every type indexed by its
//! registration index (S-0004) and every field resolved, with the engine
//! default that the owning spec states wherever the data gives none.
//!
//! [`SimRules`] is built once per game from `glasnost_rules` and never
//! changes during play. It is not part of the saved state: a save names the
//! `sim_rules_hash` it was made with.

use std::collections::BTreeMap;

use glasnost_num::{F32, F64, Rounding};
use glasnost_rules::quirks::Quirks;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    self as rs, BuildingType, Decimal, LandType, LocomotionClass, MovementZone, UnitType,
};

/// The kinds of type the simulation indexes.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum Kind {
    /// Infantry.
    Infantry,
    /// Vehicle.
    Vehicle,
    /// Aircraft.
    Aircraft,
    /// Building.
    Building,
    /// Terrain object.
    Terrain,
}

impl Kind {
    /// Whether the kind is a mobile kind (S-0211 Terms, *mobile type*).
    pub fn is_mobile(self) -> bool {
        matches!(self, Kind::Infantry | Kind::Vehicle | Kind::Aircraft)
    }

    fn registry(self) -> &'static str {
        match self {
            Kind::Infantry => "infantry",
            Kind::Vehicle => "vehicles",
            Kind::Aircraft => "aircraft",
            Kind::Building => "buildings",
            Kind::Terrain => "terrain_objects",
        }
    }
}

/// How the simulation moves a mobile type: the type's locomotor (S-0207
/// claim 14) for the kinds whose movement a spec gives.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Locomotor {
    /// The driving locomotor (S-0207).
    Drive,
    /// The walking locomotor of infantry (S-0208).
    Walk,
    /// The teleporting locomotor of a vehicle or infantry unit (S-0207 claim
    /// 20, S-0208 claim 19).
    Teleport,
    /// The flying locomotor of an aircraft (S-0501 Terms).
    Fly,
    /// The rocket locomotor of a spawned missile (S-0506 Terms).
    Rocket,
    /// The ship locomotor of a vehicle (S-0504 Terms).
    Ship,
    /// The hovering locomotor of a vehicle (S-0503 Terms).
    Hover,
    /// The jumpjet locomotor of an infantry unit or a vehicle (S-0502).
    Jump,
    /// Not moved in M3: every other locomotor, and a locomotor on a kind
    /// whose movement with it no spec gives (S-0207 Open questions).
    None,
}

/// A type's jumpjet fields (S-0502 Terms, claim 1): copied to each unit
/// that gets the jumpjet locomotor.
#[derive(Clone, Copy, Debug)]
pub struct JumpFields {
    /// The speed *S*, leptons per tick.
    pub speed: i32,
    /// The acceleration *a*, single precision held as a double.
    pub accel: F64,
    /// The climb rate *c*, single precision held as a double.
    pub climb: F64,
    /// The crash rate *k*, single precision held as a double.
    pub crash: F64,
    /// The cruise height *H* in leptons, after S-0102's minimum.
    pub height: i32,
    /// The wobble rate *W*, single precision held as a double.
    pub wobbles: F64,
    /// The wobble amplitude *d* in leptons.
    pub deviation: i32,
    /// The no-wobble flag.
    pub no_wobble: bool,
    /// The jumpjet turn-rate field *T* as imported (F6).
    pub turn_field: i32,
    /// The turn rate ω of the movement heading after S-0206 claim 6.
    pub omega: i32,
}

impl JumpFields {
    /// The jumpjet fields of an imported type, with the engine defaults of
    /// S-0502 Import mapping.
    pub fn from_type(u: &UnitType, mode: Rounding, negative_wrap: bool) -> Self {
        let single = |d: &Option<Decimal>, text: &str| -> F64 {
            d.clone()
                .unwrap_or_else(|| Decimal(text.into()))
                .single_value(mode)
        };
        // spec: S-0502 Import mapping (JumpjetTurnRate absent: 4)
        let turn_field = u.jumpjet_turn_rate.unwrap_or(4);
        Self {
            // spec: S-0502 Import mapping (JumpjetSpeed absent: 14)
            speed: u.jumpjet_speed.unwrap_or(14),
            // spec: S-0502 Import mapping (the acceleration absent: 2.0, single precision)
            accel: single(&u.jumpjet_accel, "2.0"),
            // spec: S-0502 Import mapping (JumpjetClimb absent: 5.0, single precision)
            climb: single(&u.jumpjet_climb, "5.0"),
            // spec: S-0502 Import mapping (JumpjetCrash absent: 5.0, single precision)
            crash: single(&u.jumpjet_crash, "5.0"),
            // spec: S-0102 Import mapping (JumpjetHeight absent: 500, raised to 208 when smaller) and S-0502 claim 1 (H is the type's field after that minimum)
            height: u.jumpjet_height.unwrap_or(500).max(208),
            // spec: S-0502 Import mapping (JumpjetWobbles absent: 0.15, single precision)
            wobbles: single(&u.jumpjet_wobbles, "0.15"),
            // spec: S-0502 Import mapping (JumpjetDeviation absent: 40)
            deviation: u.jumpjet_deviation.unwrap_or(40),
            // spec: S-0502 Import mapping (JumpjetNoWobbles absent: no)
            no_wobble: u.jumpjet_no_wobbles.unwrap_or(false),
            turn_field,
            // spec: S-0206 claims 6 and 14 (the jumpjet turn-rate field passed through claim 6: the movement heading's rate ω)
            omega: turn_rate(turn_field, negative_wrap),
        }
    }
}

/// A mobile type's resolved fields.
#[derive(Clone, Debug)]
pub struct UnitRules {
    /// Type name.
    pub name: String,
    /// Kind.
    pub kind: Kind,
    /// Strength (hit points).
    pub strength: i32,
    /// The type speed V or base speed b, 0 … 255 (S-0207 F1, S-0208 F4).
    pub speed: i32,
    /// Turn rate ω after S-0206 claim 6 (0: set at once).
    pub turn_rate: i32,
    /// Turret flag.
    pub turret: bool,
    /// Turret-spins flag.
    pub turret_spins: bool,
    /// Locomotion class; `None`: no class (S-0211 claim 3).
    pub class: Option<LocomotionClass>,
    /// Movement zone.
    pub zone: MovementZone,
    /// Locomotor.
    pub locomotor: Locomotor,
    /// Restricted land type (S-0211 claim 14).
    pub restricted_to: Option<u8>,
    /// Crusher flag.
    pub crusher: bool,
    /// Omni-crusher flag.
    pub omni_crusher: bool,
    /// Omni-crush resistant.
    pub omni_crush_resistant: bool,
    /// Crushable flag.
    pub crushable: bool,
    /// Stays crushable when deployed.
    pub deployed_crushable: bool,
    /// Acceleration flag.
    pub accelerates: bool,
    /// Acceleration factor a (double).
    pub acceleration: F64,
    /// Deceleration factor η (double).
    pub deceleration: F64,
    /// Slowdown distance Dₛ, leptons.
    pub slowdown_distance: i32,
    /// Passive flag.
    pub passive: bool,
    /// Train flag.
    pub train: bool,
    /// Fraidycat.
    pub fraidycat: bool,
    /// Cyborg.
    pub cyborg: bool,
    /// Fearless.
    pub fearless: bool,
    /// Can crawl (art).
    pub crawls: bool,
    /// Engineer.
    pub engineer: bool,
    /// Vehicle thief.
    pub vehicle_thief: bool,
    /// Harvester.
    pub harvester: bool,
    /// Defaults to area guard.
    pub default_to_guard_area: bool,
    /// Has a primary weapon (any version).
    pub primary: Option<String>,
    /// Secondary weapon.
    pub secondary: Option<String>,
    /// Sight, cells.
    pub sight: i32,
    /// Guard range, leptons (0: none given).
    pub guard_range: i32,
    /// Threat-avoidance coefficient k (single held as double).
    pub threat_avoidance: F64,
    /// Jumpjet flag.
    pub jumpjet: bool,
    /// Whether the type's imported locomotor is the jumpjet locomotor
    /// (S-0511 claim 15, S-0502).
    pub jumpjet_locomotor: bool,
    /// The jumpjet fields of the type (S-0502 claim 1).
    pub jump: JumpFields,
    /// The landing-first flag, `DeployToLand` (S-0502 Import mapping,
    /// S-0517 Terms).
    pub deploy_to_land: bool,
    /// The deploying presentation's animation type name (S-0517 Import
    /// mapping, `DeployingAnim`).
    pub deploying_anim: Option<String>,
    /// The deployed image type's name (S-0517 Import mapping,
    /// `UnloadingClass`); a name matching no vehicle type draws nothing.
    pub unloading_class: Option<String>,
    /// Veteran and elite ability names.
    pub abilities: (Vec<String>, Vec<String>),
    /// Passengers: the capacity of a transport (S-0507 Terms).
    pub passengers: i32,
    /// Size, a double (S-0507 Terms, claim 1).
    pub size: F64,
    /// Size limit, a double (S-0507 Terms, claim 1).
    pub size_limit: F64,
    /// Open-topped transport (S-0507 claim 14).
    pub open_topped: bool,
    /// Dropship flag (S-0507 claim 13).
    pub dropship: bool,
    /// Turret count (S-0507 claim 11).
    pub turret_count: i32,
    /// Infantry sequence entries by kind (S-0107 claim 1).
    pub sequences: BTreeMap<String, rs::SequenceEntry>,
    /// Deployer flag.
    pub deployer: bool,
    /// Undeploy delay (ticks; negative: none).
    pub undeploy_delay: i32,
    /// Simple deployer.
    pub simple_deployer: bool,
    /// Opportunity-fire flag.
    pub opportunity_fire: bool,
    /// Move into shroud.
    pub move_to_shroud: bool,
    /// Cost in credits (S-0009 claim 45).
    pub cost: i32,
    /// Tech level (S-0009 claim 45).
    pub tech_level: i32,
    /// Allowed to start in multiplayer games (S-0009 claim 45).
    pub allowed_to_start: bool,
    /// The owner list: country registration indices (S-0009 claim 45).
    pub owners: Vec<u32>,
    /// Either visceroid flag (S-0009 claim 46, S-0202 claim 10).
    pub visceroid: bool,
    /// The raw record, for fields read only by input rules (S-0214).
    pub raw: UnitType,
    /// Combat fields (M4).
    pub combat: crate::combat::data::TypeCombat,
    /// Vehicle-only combat fields (M4; defaults for other kinds).
    pub vehicle: crate::combat::data::VehicleCombat,
}

impl UnitRules {
    /// The type's locomotor (S-0207 claim 14): the teleporting locomotor
    /// when no rules layer gives one.
    pub fn type_locomotor(&self) -> rs::Locomotor {
        // spec: S-0207 claim 14 (absent: the teleporting locomotor)
        self.raw.locomotor.unwrap_or(rs::Locomotor::Teleport)
    }
}

/// A building type's resolved fields.
#[derive(Clone, Debug)]
pub struct BuildingRules {
    /// Type name.
    pub name: String,
    /// Strength.
    pub strength: i32,
    /// Foundation width and height in cells.
    pub foundation: (i32, i32),
    /// Turret flag.
    pub turret: bool,
    /// Turn rate ω.
    pub turn_rate: i32,
    /// Gate.
    pub gate: bool,
    /// Invisible in play.
    pub invisible: bool,
    /// The raw record.
    pub raw: BuildingType,
    /// A gate's door time Td in ticks (S-0215 F9; a negative result is
    /// kept and behaves as 0).
    pub gate_door_time: i32,
    /// A gate's close delay Tc in ticks (S-0215 F9).
    pub gate_close_delay: i32,
    /// Combat fields (M4).
    pub combat: crate::combat::data::TypeCombat,
    /// Building-only combat fields (M4).
    pub building: crate::combat::data::BuildingCombat,
    /// The kind of object it makes, for a factory building (S-0403 Terms).
    pub factory: Option<FactoryKind>,
    /// The imported factory category, producing or not (S-0003 claim 21;
    /// S-0213 claim 13, S-0214 claim 17); `None` for no factory.
    pub factory_category: Option<FactoryCategory>,
    /// The naval flag (S-0403 Terms: a naval factory building of vehicles;
    /// S-0406 claim 3 d).
    pub naval: bool,
    /// The absorbing-building fields (S-0605 Terms and Import mapping).
    pub absorb: AbsorbRules,
}

/// An absorbing building type's fields (S-0605 Terms, F1 and Import
/// mapping).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AbsorbRules {
    /// The infantry-absorb flag: an infantry absorber.
    pub infantry: bool,
    /// The vehicle-absorb flag: a vehicle absorber.
    pub vehicle: bool,
    /// The capacity *c*, an integer.
    pub capacity: i32,
    /// The size limit *L*, a double.
    pub size_limit: F64,
}

impl AbsorbRules {
    /// An absorbing building: either flag (S-0605 Terms).
    pub fn absorbing(&self) -> bool {
        self.infantry || self.vehicle
    }
}

/// A building's imported factory category (S-0003 claim 21): one of the
/// four producing kinds, or a recognised category that produces none of
/// them (S-0214 claim 17).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum FactoryCategory {
    /// A producing kind.
    Produces(FactoryKind),
    /// The terrain-type word.
    Terrain,
    /// The infantry-object word.
    InfantryObject,
    /// The vehicle-object word.
    VehicleObject,
    /// The aircraft-object word.
    AircraftObject,
    /// The building-object word.
    BuildingObject,
}

impl FactoryCategory {
    /// The category of our format's factory word.
    pub fn from_word(w: &str) -> Option<Self> {
        if let Some(k) = FactoryKind::from_word(w) {
            return Some(Self::Produces(k));
        }
        Some(match w {
            "terrain" => Self::Terrain,
            "infantry_object" => Self::InfantryObject,
            "vehicle_object" => Self::VehicleObject,
            "aircraft_object" => Self::AircraftObject,
            "building_object" => Self::BuildingObject,
            _ => return None,
        })
    }
}

/// The kind of object a factory building makes (S-0403 Terms, *factory
/// building of a kind*).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum FactoryKind {
    /// Infantry.
    Infantry,
    /// Vehicles.
    Vehicle,
    /// Aircraft.
    Aircraft,
    /// Buildings.
    Building,
}

impl FactoryKind {
    /// The kind of our format's factory word (`infantry`, `vehicle`,
    /// `aircraft`, `building`), which the importer writes for the
    /// original's words (S-0003 claim 21).
    pub fn from_word(w: &str) -> Option<Self> {
        Some(match w {
            "infantry" => Self::Infantry,
            "vehicle" => Self::Vehicle,
            "aircraft" => Self::Aircraft,
            "building" => Self::Building,
            _ => return None,
        })
    }

    /// The object kind it makes.
    pub fn kind(self) -> Kind {
        match self {
            Self::Infantry => Kind::Infantry,
            Self::Vehicle => Kind::Vehicle,
            Self::Aircraft => Kind::Aircraft,
            Self::Building => Kind::Building,
        }
    }
}

/// An overlay type's resolved fields.
#[derive(Clone, Debug, Default)]
pub struct OverlayRules {
    /// Type name.
    pub name: String,
    /// Land type label (S-0211 claim 4; tiberium for ore with clear).
    pub land: u8,
    /// Own-land-type flag.
    pub own_land: bool,
    /// Wall flag.
    pub wall: bool,
    /// Tiberium flag.
    pub tiberium: bool,
    /// Crushable flag.
    pub crushable: bool,
    /// Rock flag.
    pub rock: bool,
    /// Rubble flag.
    pub rubble: bool,
    /// Crate flag.
    pub crate_: bool,
    /// Crate-trigger flag (S-0520 claim 7).
    pub crate_trigger: bool,
    /// Damage-stage count D.
    pub damage_levels: i32,
    /// Strength S.
    pub strength: i32,
    /// Armour is wood.
    pub wood: bool,
    /// Has artwork or a cell animation (S-0009 claim 12).
    pub has_art: bool,
    /// The veins flag (S-0202 Terms).
    pub veins: bool,
    /// Armour class index (S-0305 claims 3 to 5).
    pub armour: usize,
    /// Chain-reaction flag (S-0303 claim 17).
    pub chain_reaction: bool,
    /// Explodes flag (S-0303 claim 19).
    pub explodes: bool,
    /// A legal target (S-0214 *attackable cell* (i)).
    pub legal_target: bool,
    /// The cell animation's type name (S-0009 claim 12), if any.
    pub cell_anim: Option<String>,
}

/// A terrain-object type's resolved fields.
#[derive(Clone, Debug, Default)]
pub struct TerrainRules {
    /// Type name.
    pub name: String,
    /// Occupation pattern, temperate and snow.
    pub occupation: (i32, i32),
    /// The spawns-resources flag (S-0203 claim 2).
    pub spawns_resources: bool,
    /// Strength (S-0302 claim 20; S-0102 claim 23): −1 when unset.
    pub strength: i32,
    /// Armour class index (S-0305).
    pub armour: usize,
    /// Immune (S-0302 claim 20).
    pub immune: bool,
    /// A legal target (S-0214 Import mapping, claim 6 (a)).
    pub legal_target: bool,
    /// Accepts bombs (S-0214 Import mapping, row I19).
    pub bombable: bool,
    /// The animated flag of an ore mine (S-0401 claim 15).
    pub animated: bool,
    /// The animation rate *R* of an ore mine, ticks (S-0401 claim 15).
    pub animation_rate: i32,
    /// The animation probability *p* of an ore mine, a single widened to
    /// double (S-0401 F8).
    pub animation_probability: F64,
    /// The image frame count *N* of the type's sprite (S-0401 claim 15).
    pub image_frames: i32,
}

/// One behaviour's properties (S-0215 claims 6, 16 to 21).
#[derive(Clone, Copy, Debug)]
pub struct BehaviourRules {
    /// No-threat.
    pub no_threat: bool,
    /// Paralysed.
    pub paralysed: bool,
    /// Recruitable.
    pub recruitable: bool,
    /// Retaliate.
    pub retaliate: bool,
    /// Scatter.
    pub scatter: bool,
    /// Zombie.
    pub zombie: bool,
    /// Rate delay Dᵣ in ticks (S-0215 F1).
    pub rate_delay: i32,
    /// Anti-air rate delay D_aa in ticks.
    pub aa_rate_delay: i32,
}

/// A country's resolved fields.
#[derive(Clone, Copy, Debug)]
pub struct CountryRules {
    /// κ for vehicles (single held as double).
    pub speed_units: F64,
    /// k꜀ for infantry (single held as double).
    pub speed_infantry: F64,
    /// κ for aircraft (single held as double; S-0413 claim 3).
    pub speed_aircraft: F64,
    /// Passive.
    pub passive: bool,
    /// The wall-owner flag (S-0210 claim 14).
    pub wall_owner: bool,
    /// Passive as the base rules alone give it, the passivity the early
    /// phase of start positions sees (S-0009 claim 33).
    pub passive_base: bool,
}

/// An animation type's resolved creation-draw fields (S-0109; S-0202
/// claim 10).
#[derive(Clone, Debug)]
pub struct AnimRules {
    /// Type name.
    pub name: String,
    /// The random delay range [d₁′, d₂′] in ticks (S-0109 claim 3, F1).
    pub random_delay: (i32, i32),
    /// A bouncing type: the bouncer or the meteor flag (S-0109 Terms).
    pub bouncing: bool,
    /// μ, the middle image frame (S-0109 claims 14 and 25); `None` when the
    /// import had no art to find it from.
    pub mu: Option<i32>,
    /// The image-frame count *F* of the type's sprite (S-0513 claim 46);
    /// `None` when the import had no art for it.
    pub frames: Option<i32>,
    /// The canvas height of that sprite's header in pixels (S-0513 F4).
    pub canvas_height: Option<i32>,
    /// The rate field ρ (S-0109 claim 2); `None` when not given.
    pub rate: Option<i32>,
    /// The game-speed normalisation flag (S-0109 claim 4).
    pub normalized: bool,
    /// The start field *a* (S-0109 Import mapping, absent 0).
    pub start: i32,
    /// The end field as written; `None` when absent (S-0109 Import mapping).
    pub end: Option<i32>,
    /// The shadow-type flag (S-0109 Import mapping).
    pub shadow: bool,
    /// The particles its middle event requests: the particle type's draw
    /// fields and the count (S-0109 claim 14, S-0202 claim 15); `None` for
    /// no particle type.
    pub particles: Option<(ParticleRules, i32)>,
    /// The scorch flag (S-0109 claim 14).
    pub scorch: bool,
    /// The crater flag (S-0109 claim 14).
    pub crater: bool,
    /// The resource-chain-reaction flag (S-0109 claim 13, S-0401 claim 20).
    pub chain_reaction: bool,
    /// The burning-walker flag (S-0615 Terms and Import mapping).
    pub flaming: bool,
    /// The running frames *R* of a burning walker (S-0615 Terms).
    pub running_frames: i32,
    /// The damage value δ of S-0109 claim 17, a single held exactly in a
    /// double (S-0109 F5).
    pub damage: glasnost_num::F64,
    /// The make-infantry index *i* (S-0611 F4); −1 for none.
    pub make_infantry: i32,
    /// Loop start λₛ (S-0109 Import mapping, absent 0).
    pub loop_start: i32,
    /// Loop end λₑ as written; `None` when absent (S-0109 F2).
    pub loop_end: Option<i32>,
    /// Loop count *L* (S-0109 claim 7, absent 0).
    pub loop_count: i32,
    /// The reverse flag (S-0109 claim 8).
    pub reverse: bool,
    /// The ping-pong flag (S-0109 claim 9).
    pub ping_pong: bool,
    /// The loop-delay pair in updates (S-0109 claim 11; (0, 0) none).
    pub random_loop_delay: (i32, i32),
    /// The follow-up type's name as given (S-0109 claim 15); `None` for
    /// none.
    pub next: Option<String>,
    /// The follow-up type by registration index; `None` for none, or for a
    /// name no type is registered under (it plays with absent-key defaults
    /// and no sprite, S-0109 Edge cases).
    pub next_index: Option<u32>,
    /// A new-theater type (S-0109 Terms).
    pub new_theater: bool,
    /// The bouncing-animation and trailer fields (S-0319 Import mapping).
    pub bounce: crate::bounce::BounceType,
    /// The built-in invisibility-animation type: its stored name is exactly
    /// [`INVISIBILITY_ANIM_NAME`], byte for byte (S-0109 claim 17 and
    /// Import mapping). Its damage takes the demolition warhead.
    pub invisibility: bool,
}

/// The name of the built-in invisibility-animation type, compared byte for
/// byte, letter case included, with a type's stored name: the spelling it
/// was first registered with (S-0004 claims 2 and 7).
pub const INVISIBILITY_ANIM_NAME: &[u8] = b"INVISO"; // spec: S-0109 Import mapping (the built-in invisibility-animation type's name, six ASCII capital letters)

impl Default for AnimRules {
    fn default() -> Self {
        Self {
            name: String::new(),
            random_delay: (0, 0),
            bouncing: false,
            mu: None,
            frames: None,
            canvas_height: None,
            rate: None,
            normalized: false,
            start: 0,
            end: None,
            shadow: false,
            particles: None,
            scorch: false,
            crater: false,
            chain_reaction: false,
            flaming: false,
            running_frames: 0,
            damage: glasnost_num::F64::ZERO,
            // spec: S-0611 Terms and S-0109 Import mapping (MakeInfantry absent: −1, none)
            make_infantry: -1,
            loop_start: 0,
            loop_end: None,
            loop_count: 0,
            reverse: false,
            ping_pong: false,
            random_loop_delay: (0, 0),
            next: None,
            next_index: None,
            new_theater: false,
            bounce: crate::bounce::BounceType::default(),
            invisibility: false,
        }
    }
}

/// A particle type's creation-draw fields (S-0202 claim 15 and Import
/// mapping).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ParticleRules {
    /// Smoke behaviour: the starting-speed raw value.
    pub smoke: bool,
    /// Railgun behaviour: the lifetime modulus is 10.
    pub railgun: bool,
    /// The lifetime field, the modulus of other behaviours.
    pub max_ec: i32,
    /// A nonempty colour list with a nonblack starting colour: the
    /// starting-colour draw.
    pub colour_draw: bool,
    /// The particle type's registration index (S-0613 claim 2: the middle
    /// event creates particles of it); `None` for a type the rules do not
    /// list.
    pub index: Option<u32>,
}

/// A particle type's creation-draw fields (S-0202 claim 15 and Import
/// mapping), without its index.
pub fn particle_draw_rules(t: &glasnost_rules::sim::ParticleType) -> ParticleRules {
    let behaves = t.behaves_like.as_deref();
    // spec: S-0202 Import mapping (StartColor1, StartColor2 black at type creation)
    let lit = |c: Option<[i32; 3]>| c.is_some_and(|c| c.iter().any(|&x| x != 0));
    ParticleRules {
        smoke: behaves == Some("smoke"),
        railgun: behaves == Some("railgun"),
        // spec: S-0202 Import mapping (MaxEC: 1 at type creation)
        max_ec: t.max_ec.unwrap_or(1),
        // spec: S-0202 claim 15 (the colour draw: a nonempty colour list and a nonzero component in either starting colour)
        colour_draw: t.color_count.unwrap_or(0) > 0 && (lit(t.start_color1) || lit(t.start_color2)),
        index: None,
    }
}

/// The stage delay d of a rate value ρ (S-0109 F1), with
/// `quirks.anim_fast_rate_freezes` as `fast_freezes`.
pub fn anim_delay(rho: i32, fast_freezes: bool) -> i32 {
    if rho <= 0 {
        // spec: S-0109 F1 and Edge cases (0 for ρ ≤ 0 in both settings)
        return 0;
    }
    // spec: S-0109 F1 (d = ⌊900 / ρ⌋ for ρ > 0; a rate above 900 gives 0)
    let d = 900 / rho;
    if fast_freezes {
        d
    } else {
        // spec: S-0109 Edge cases (quirks.anim_fast_rate_freezes off: max(1, ⌊900 / ρ⌋) for ρ > 0)
        d.max(1)
    }
}

/// The random delay range of a random rate pair (ρ₁, ρ₂) (S-0109 claim 3,
/// F1), with `quirks.anim_fast_rate_freezes` and
/// `quirks.anim_random_rate_collapse`.
pub fn anim_random_delay(r1: i32, r2: i32, q: &Quirks) -> (i32, i32) {
    let (d1, d2) = (
        anim_delay(r1, q.anim_fast_rate_freezes),
        anim_delay(r2, q.anim_fast_rate_freezes),
    );
    if q.anim_random_rate_collapse {
        // spec: S-0109 claim 3 and F1 (d₂′ = max(d₂, 0), d₁′ = min(d₁, d₂′): an ascending pair collapses to d₂)
        let d2p = d2.max(0);
        (d1.min(d2p), d2p)
    } else {
        // spec: S-0109 Edge cases (quirks.anim_random_rate_collapse off: the converted pair in order, [min, max])
        (d1.min(d2), d1.max(d2))
    }
}

/// The gate and wall tower types that walls join (S-0112 claim 7), by
/// building registration index.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GateTypes {
    /// The first Allied gate type.
    pub allied_one: Option<u32>,
    /// The second Allied gate type.
    pub allied_two: Option<u32>,
    /// The first Soviet gate type.
    pub soviet_one: Option<u32>,
    /// The second Soviet gate type.
    pub soviet_two: Option<u32>,
    /// The wall tower type.
    pub tower: Option<u32>,
}

/// The hovering locomotor's game-wide values (S-0503 Import mapping).
#[derive(Clone, Copy, Debug)]
pub struct HoverRules {
    /// The hover height field *H*ₕ, leptons (F6).
    pub height: i32,
    /// The bob field *q*, game minutes (F6).
    pub bob: F64,
    /// The boost factor (claim 3).
    pub boost: F64,
    /// The acceleration field *a*, game minutes (F2).
    pub acceleration: F64,
    /// The brake field *b*, game minutes (F2).
    pub brake: F64,
    /// The dampen field *k* (F6).
    pub dampen: F64,
    /// Gravity *g*, leptons per tick² (F6).
    pub gravity: i32,
}

/// Game-wide values.
#[derive(Clone, Copy, Debug)]
pub struct GeneralRules {
    /// Close-enough distance, leptons (S-0209 F2).
    pub close_enough: i32,
    /// Blockage delay, ticks (S-0209).
    pub blockage_delay: i32,
    /// Retry delay R, ticks (S-0212 F3).
    pub retry_delay: i32,
    /// Player-scatter rule.
    pub player_scatter: bool,
    /// Player return fire.
    pub player_return_fire: bool,
    /// Player auto-crush rule.
    pub player_auto_crush: bool,
    /// Crush range, leptons (S-0210 F2).
    pub crush_range: i32,
    /// Guard-area targeting delay G, ticks.
    pub guard_area_targeting_delay: i32,
    /// Normal targeting delay, ticks.
    pub normal_targeting_delay: i32,
    /// Approach reset multiplier m (S-0215 F10).
    pub approach_reset_multiplier: i32,
    /// Veteran speed factor ν.
    pub veteran_speed: F64,
    /// Tracked uphill, downhill; wheeled uphill, downhill.
    pub slopes: [F64; 4], // spec: S-0207 Constants (four slope factors: tracked and wheeled, uphill and downhill)
    /// Cliff-back setting.
    pub cliff_back: i32,
    /// Yellow ratio.
    pub condition_yellow: F64,
    /// Red ratio.
    pub condition_red: F64,
    /// Idle-action frequency R, game minutes (double).
    pub idle_frequency: F64,
    /// The resource-glint frequency f.
    pub ore_twinkle_chance: i32,
    /// The teleport distance factor k, leptons per tick (S-0207 F9).
    pub chrono_distance_factor: i32,
    /// The teleport trigger flag (S-0207 F9).
    pub chrono_trigger: bool,
    /// The teleport minimum delay m, ticks (S-0207 F9).
    pub chrono_minimum_delay: i32,
    /// The teleport range minimum g, leptons (S-0207 F9).
    pub chrono_range_minimum: i32,
    /// Maximum cheer rate, ticks.
    pub cheer_rate: i32,
    /// Area-guard threshold of the house skill level.
    pub iq_guard_area: i32,
    /// Guard stray distance in leptons (S-0215 F5); `None` when no rules
    /// layer gives it (S-0215 claim 36 (b): the leash is used instead).
    pub guard_stray: Option<i32>,
    /// Scatter threshold of the house skill level.
    pub iq_scatter: i32,
    /// Auto-crush threshold of the house skill level (S-0210 claim 11: the
    /// original's computer-controlled row, kept by the neutral and special
    /// houses and a test-only computer-controlled player house).
    pub iq_auto_crush: i32,
    /// The maximum house skill level (S-0209 claim 11, S-0412 claim 3): the
    /// level of a computer player house of the original, which our engine
    /// gives only to a test-only computer-controlled player house (glossary).
    pub iq_max: i32,
    /// The auto-deploy delay list of S-0215 claim 31, ticks, for difficulty
    /// indexes 0, 1 and 2: the neutral and special houses read entry 1, a
    /// test-only computer-controlled player house its oracle house's index;
    /// `None` where the list has no entry.
    pub auto_deploy_delays: [Option<i32>; 3], // spec: S-0215 Terms (difficulty index: 0, 1 or 2)
    /// The rules' lobby game-speed index (S-0201 claim 7).
    pub lobby_game_speed: u8,
    /// The engineer-capture health ratio c (S-0214 row I7), single
    /// precision widened.
    pub engineer_capture: F64,
    /// The hovering locomotor's game-wide values (S-0503 Import mapping).
    pub hover: HoverRules,
}

/// The behaviours of S-0215 table T1, in our order.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[allow(missing_docs)]
pub enum Behaviour {
    Ambush,
    AreaGuard,
    Attack,
    AttackMove,
    Capture,
    Construction,
    Eaten,
    Enter,
    Guard,
    Harmless,
    Harvest,
    Hunt,
    Missile,
    Move,
    Open,
    ParadropApproach,
    ParadropOverfly,
    Patrol,
    QueuedMove,
    Repair,
    Rescue,
    Retreat,
    Return,
    Sabotage,
    Selling,
    Sleep,
    SpyPlaneApproach,
    SpyPlaneOverfly,
    Sticky,
    Stop,
    Unload,
    Wait,
}

impl Behaviour {
    /// All 32 behaviours.
    // spec: S-0215 Terms (the 32 behaviours of table T1)
    pub const ALL: [Behaviour; 32] = [
        Behaviour::Ambush,
        Behaviour::AreaGuard,
        Behaviour::Attack,
        Behaviour::AttackMove,
        Behaviour::Capture,
        Behaviour::Construction,
        Behaviour::Eaten,
        Behaviour::Enter,
        Behaviour::Guard,
        Behaviour::Harmless,
        Behaviour::Harvest,
        Behaviour::Hunt,
        Behaviour::Missile,
        Behaviour::Move,
        Behaviour::Open,
        Behaviour::ParadropApproach,
        Behaviour::ParadropOverfly,
        Behaviour::Patrol,
        Behaviour::QueuedMove,
        Behaviour::Repair,
        Behaviour::Rescue,
        Behaviour::Retreat,
        Behaviour::Return,
        Behaviour::Sabotage,
        Behaviour::Selling,
        Behaviour::Sleep,
        Behaviour::SpyPlaneApproach,
        Behaviour::SpyPlaneOverfly,
        Behaviour::Sticky,
        Behaviour::Stop,
        Behaviour::Unload,
        Behaviour::Wait,
    ];

    /// Our snake-case name (the key of `sim.behaviours`).
    pub fn key(self) -> &'static str {
        match self {
            Behaviour::Ambush => "ambush",
            Behaviour::AreaGuard => "area_guard",
            Behaviour::Attack => "attack",
            Behaviour::AttackMove => "attack_move",
            Behaviour::Capture => "capture",
            Behaviour::Construction => "construction",
            Behaviour::Eaten => "eaten",
            Behaviour::Enter => "enter",
            Behaviour::Guard => "guard",
            Behaviour::Harmless => "harmless",
            Behaviour::Harvest => "harvest",
            Behaviour::Hunt => "hunt",
            Behaviour::Missile => "missile",
            Behaviour::Move => "move",
            Behaviour::Open => "open",
            Behaviour::ParadropApproach => "paradrop_approach",
            Behaviour::ParadropOverfly => "paradrop_overfly",
            Behaviour::Patrol => "patrol",
            Behaviour::QueuedMove => "queued_move",
            Behaviour::Repair => "repair",
            Behaviour::Rescue => "rescue",
            Behaviour::Retreat => "retreat",
            Behaviour::Return => "return",
            Behaviour::Sabotage => "sabotage",
            Behaviour::Selling => "selling",
            Behaviour::Sleep => "sleep",
            Behaviour::SpyPlaneApproach => "spy_plane_approach",
            Behaviour::SpyPlaneOverfly => "spy_plane_overfly",
            Behaviour::Sticky => "sticky",
            Behaviour::Stop => "stop",
            Behaviour::Unload => "unload",
            Behaviour::Wait => "wait",
        }
    }

    fn index(self) -> usize {
        Self::ALL.iter().position(|b| *b == self).unwrap_or(0)
    }
}

/// A land type's row of multipliers (S-0211 claim 2), by locomotion class in
/// the order foot, track, wheel, hover, winged, float, amphibious,
/// float-on-beach.
#[derive(Clone, Copy, Debug)]
pub struct LandRules {
    /// M(ℓ, c), single precision.
    pub multiplier: [F32; 8], // spec: S-0211 Terms (eight locomotion classes)
    /// Buildable flag.
    pub buildable: bool,
}

/// Index of a locomotion class in a land row.
pub fn class_index(c: LocomotionClass) -> usize {
    match c {
        LocomotionClass::Foot => 0,
        LocomotionClass::Track => 1,
        LocomotionClass::Wheel => 2, // spec: S-0211 Terms (locomotion classes in their column order)
        LocomotionClass::Hover => 3, // spec: S-0211 Terms (column order)
        LocomotionClass::Winged => 4, // spec: S-0211 Terms (column order)
        LocomotionClass::Float => 5, // spec: S-0211 Terms (column order)
        LocomotionClass::Amphibious => 6, // spec: S-0211 Terms (column order)
        LocomotionClass::FloatBeach => 7, // spec: S-0211 Terms (column order)
    }
}

/// The land label of a rules land type (S-0211 Terms numbering).
pub fn land_label(l: LandType) -> u8 {
    use glasnost_map::terrain::land;
    match l {
        LandType::Clear => land::CLEAR,
        LandType::Road => land::ROAD,
        LandType::Water => land::WATER,
        LandType::Rock => land::ROCK,
        LandType::Wall => land::WALL,
        LandType::Tiberium => land::TIBERIUM,
        LandType::Beach => land::BEACH,
        LandType::Rough => land::ROUGH,
        LandType::Ice => land::ICE,
        LandType::Railroad => land::RAILROAD,
        LandType::Tunnel => land::TUNNEL,
        LandType::Weeds => land::WEEDS,
    }
}

// spec: S-0211 Terms (the twelve land types)
const LAND_TYPES: [LandType; 12] = [
    LandType::Clear,
    LandType::Road,
    LandType::Water,
    LandType::Rock,
    LandType::Wall,
    LandType::Tiberium,
    LandType::Beach,
    LandType::Rough,
    LandType::Ice,
    LandType::Railroad,
    LandType::Tunnel,
    LandType::Weeds,
];

/// The resolved rules of one game.
#[derive(Clone, Debug)]
pub struct SimRules {
    /// Quirk switches.
    pub quirks: Quirks,
    /// Rounding of floating-point results (S-0205 claim 5).
    pub mode: Rounding,
    /// Game-wide values.
    pub general: GeneralRules,
    /// Land rows by land label.
    pub land: [LandRules; 12], // spec: S-0211 Terms (the twelve land types)
    /// Behaviour properties, in [`Behaviour::ALL`] order.
    pub behaviours: Vec<BehaviourRules>,
    /// Countries by registration index.
    pub countries: Vec<CountryRules>,
    /// Country names by registration index.
    pub country_names: Vec<String>,
    /// Infantry types.
    pub infantry: Vec<UnitRules>,
    /// Vehicle types.
    pub vehicles: Vec<UnitRules>,
    /// Aircraft types.
    pub aircraft: Vec<UnitRules>,
    /// Building types.
    pub buildings: Vec<BuildingRules>,
    /// Overlay types by registration index (the overlay list).
    pub overlays: Vec<OverlayRules>,
    /// Terrain-object types.
    pub terrain: Vec<TerrainRules>,
    /// Weapons by name.
    pub weapons: BTreeMap<String, rs::WeaponType>,
    /// Projectiles by name.
    pub projectiles: BTreeMap<String, rs::ProjectileType>,
    /// Warheads by name.
    pub warheads: BTreeMap<String, rs::WarheadType>,
    /// Type names by kind, to registration index.
    pub names: BTreeMap<(Kind, String), u32>,
    /// The base-unit list: vehicle registration indices (S-0009 claim 45).
    pub base_units: Vec<u32>,
    /// The game's tech level (S-0009 Terms).
    pub game_tech_level: i32,
    /// The maximum player count of the unused-house cleanup (S-0009 claim
    /// 28).
    pub max_players: i32,
    /// The gate and wall tower types (S-0112 claim 7).
    pub gates: GateTypes,
    /// Animation types by registration index.
    pub animations: Vec<AnimRules>,
    /// The registration index of each animation type name, keyed by the
    /// name with ASCII letters folded to lower case; the first registered
    /// type of a folded name wins (#407). Derived from `animations`.
    anim_by_name: BTreeMap<Vec<u8>, u32>,
    /// The length of `animations` when `anim_by_name` was built.
    anim_by_name_len: usize,
    /// Each kind's type names with ASCII letters folded to lower case, to
    /// the registration index of the last name in `names` order that folds
    /// to it (#400). Derived from `names`.
    names_folded: BTreeMap<Kind, BTreeMap<Vec<u8>, u32>>,
    /// The length of `names` when `names_folded` was built.
    names_folded_len: usize,
    /// Particle and particle-system types, the shared gas system and the
    /// wind (S-0613).
    pub gas: crate::gas::GasRules,
    /// The debris-piece types (S-0317).
    pub pieces: crate::pieces::PieceRules,
    /// The loops in the links between types that make one another
    /// (#224, #174, #203, #198; [`crate::loops`]).
    pub loops: crate::loops::LinkLoops,
    /// The resource-glint animation type, if one is configured (S-0202
    /// claim 14).
    pub ore_twinkle: Option<String>,
    /// The warp animation of a teleporting jump, if one is in force
    /// (S-0207 claim 20 e).
    pub warp_out: Option<String>,
    /// The game-wide wake animation, if one is defined (S-0207 claim 23).
    pub wake: Option<String>,
    /// The ruleset's `sim_rules_hash`.
    pub hash: String,
    /// The crate rules (S-0520).
    pub crates: crate::crates::CrateRules,
    /// The combat rules (M4).
    pub combat: crate::combat::data::CombatRules,
    /// The rules of the M5a topics (economy, production, structures,
    /// visibility, setup).
    pub econ: crate::econ::EconRules,
    /// Map triggers (S-0011).
    pub triggers: crate::triggers::Rules,
    /// Spawners and rocket parameter sets (S-0506).
    pub missiles: crate::spawn::MissileRules,
    /// Aircraft flight and pads (S-0501).
    pub air: crate::air::AirRules,
    /// Cloak and disguise (S-0516).
    pub cloak: crate::cloak::CloakRules,
    /// Superweapons (S-0512, S-0513).
    pub superweapons: crate::superweapons::SwRules,
}

fn dec(d: &Option<Decimal>, mode: Rounding) -> Option<F64> {
    d.as_ref().map(|d| d.value(mode))
}

/// A double from the decimal text of an engine constant.
fn constant(text: &str) -> F64 {
    glasnost_num::decimal::parse(
        text,
        glasnost_num::decimal::Field::Decimal,
        Rounding::TowardZero,
    )
    .unwrap_or_default()
}

/// The double nearest to the decimal `text` (an engine constant written as a
/// decimal that the spec calls a double).
pub fn nearest_double(text: &str) -> F64 {
    glasnost_num::decimal::nearest_double(text)
}

/// The turn rate ω from a turn-rate field (S-0206 claim 6, F3).
pub fn turn_rate(v: i32, negative_wrap: bool) -> i32 {
    // spec: S-0206 F3 (ω = 127 when v ≥ 127; v when 0 ≤ v ≤ 126; v mod 256 rule below 0)
    if v >= 127 {
        127
    } else if v >= 0 {
        v
    } else if !negative_wrap {
        // spec: S-0206 Edge cases (quirks.turn_rate_negative_wrap off: every negative value sets at once)
        0
    } else {
        let b = v.rem_euclid(256);
        if (1..=127).contains(&b) { b } else { 0 }
    }
}

/// The type speed V of a speed field (S-0207 claim 1, F1; S-0208 F4).
pub fn type_speed(v: i32) -> i32 {
    // spec: S-0207 F1 (V = min(⌊256 × clamp(v, 0, 100) / 100⌋, 255))
    let c = v.clamp(0, 100);
    (256 * c / 100).min(255) // spec: S-0207 F1 (V = min(⌊256 × c / 100⌋, 255))
}

/// The rate delay of S-0215 F1: trunc(RZ₅₃(r × 900)), wrapped, 0 at 2⁶³.
pub fn rate_delay(r: F64, mode: Rounding) -> i32 {
    // spec: S-0215 F1 (p = RZ₅₃(r × 900), 900 ticks per game minute; Dᵣ = w(trunc(p)), 0 at 2⁶³)
    r.mul(F64::from_i64(900, mode), mode).to_i32_common()
}

impl SimRules {
    /// Builds the rules of one game from a resolved ruleset.
    pub fn new(r: &Ruleset, hash: &str) -> Self {
        let quirks = r.globals.quirks;
        let mode = quirks.rounding();
        let s = &r.sim;
        let g = &s.general;
        let one = F64::from_i64(1, mode);
        let cells = |d: &Option<Decimal>, default: i32| -> i32 {
            // spec: S-0209 F2 (trunc(256 × c), 256 leptons per cell; exact product)
            d.as_ref().map_or(default, |d| {
                d.value(mode)
                    .mul(F64::from_i64(256, mode), mode)
                    .to_i32_common()
            })
        };
        let general = GeneralRules {
            // spec: S-0209 Import mapping (CloseEnough absent: 640 leptons)
            close_enough: cells(&g.close_enough, 640),
            // spec: S-0209 Import mapping (BlockagePathDelay absent: 60 ticks)
            blockage_delay: g.blockage_path_delay.unwrap_or(60),
            // spec: S-0212 F3 (R = trunc(trunc₅₃(P × 900)); absent: the double nearest 0.016)
            retry_delay: rate_delay(
                dec(&g.path_delay, mode).unwrap_or_else(|| nearest_double("0.016")),
                mode,
            ),
            // spec: S-0208 Import mapping (PlayerScatter absent: no)
            player_scatter: g.player_scatter.unwrap_or(false),
            // spec: S-0215 Import mapping (PlayerReturnFire)
            player_return_fire: g.player_return_fire.unwrap_or(false),
            // spec: S-0210 Constants (player-auto-crush rule absent: off)
            player_auto_crush: g.player_auto_crush.unwrap_or(false),
            // spec: S-0210 F2 (crush range absent, or exactly −1, which counts as no value: 384 leptons)
            crush_range: cells(
                &g.crush_range
                    .clone()
                    .filter(|d| !d.value(mode).eq_num(F64::from_i64(-1, mode))),
                384,
            ),
            // spec: S-0206 Import mapping (GuardAreaTargetingDelay absent: 36 ticks)
            guard_area_targeting_delay: g.guard_area_targeting_delay.unwrap_or(36),
            // spec: S-0215 Constants (normal targeting delay absent: 27 ticks)
            normal_targeting_delay: g.normal_targeting_delay.unwrap_or(27),
            // spec: S-0215 Constants (approach reset multiplier absent: 1)
            approach_reset_multiplier: g.approach_target_reset_multiplier.unwrap_or(1),
            // spec: S-0207 Constants (veteran speed factor absent: 1.0)
            veteran_speed: dec(&g.veteran_speed, mode).unwrap_or(one),
            // spec: S-0207 Constants (uphill and downhill factors absent: 1.0)
            slopes: [
                dec(&g.tracked_uphill, mode).unwrap_or(one),
                dec(&g.tracked_downhill, mode).unwrap_or(one),
                dec(&g.wheeled_uphill, mode).unwrap_or(one),
                dec(&g.wheeled_downhill, mode).unwrap_or(one),
            ],
            // spec: S-0211 Import mapping (CliffBackImpassability absent: 0)
            cliff_back: g.cliff_back_impassability.unwrap_or(0),
            // spec: S-0107 Import mapping (ConditionYellow absent: 0.5); S-0302 Import mapping (read as a single, widened exactly, a `%` product taken as a double)
            condition_yellow: dec(&g.condition_yellow, mode).unwrap_or_else(|| constant("0.5")),
            // spec: S-0107 Import mapping (ConditionRed absent: 0.5); S-0302 Import mapping (read as a single, widened exactly, a `%` product taken as a double)
            condition_red: dec(&g.condition_red, mode).unwrap_or_else(|| constant("0.5")),
            // spec: S-0107 Import mapping (IdleActionFrequency absent: the double nearest 0.083)
            idle_frequency: dec(&g.idle_action_frequency, mode)
                .unwrap_or_else(|| nearest_double("0.083")),
            // spec: S-0202 Import mapping (OreTwinkleChance: 50 at database creation)
            ore_twinkle_chance: g.ore_twinkle_chance.unwrap_or(50),
            // spec: S-0207 Constants (absent-key defaults: k = 32, trigger on, m = 0, g = 0)
            chrono_distance_factor: g.chrono_distance_factor.unwrap_or(32),
            chrono_trigger: g.chrono_trigger.unwrap_or(true),
            chrono_minimum_delay: g.chrono_minimum_delay.unwrap_or(0),
            chrono_range_minimum: g.chrono_range_minimum.unwrap_or(0),
            // spec: S-0213 Import mapping (MaximumCheerRate absent: 1 tick)
            cheer_rate: g.maximum_cheer_rate.unwrap_or(1),
            // spec: S-0215 claim 36 (a) and Constants ([IQ] GuardArea absent from every layer: engine default 4)
            iq_guard_area: g.iq_guard_area.unwrap_or(4),
            // spec: S-0215 Import mapping (GuardModeStray: w(trunc(c × 256)) leptons, 0 at 2⁶³; absent from every layer: no value, claim 36 (b))
            guard_stray: g.guard_mode_stray.as_ref().map(|d| {
                d.value(mode)
                    .mul(F64::from_i64(256, mode), mode)
                    .to_i32_common()
            }),
            // spec: S-0209 Import mapping ([IQ] Scatter absent: 3)
            iq_scatter: g.iq_scatter.unwrap_or(3),
            // spec: S-0210 Constants and Import mapping ([IQ] AutoCrush absent: 2)
            iq_auto_crush: g.iq_auto_crush.unwrap_or(2),
            // spec: S-0209 Import mapping ([IQ] MaxIQLevels absent: 5 skill levels)
            iq_max: g.iq_max.unwrap_or(5),
            // spec: S-0215 claim 31 and Import mapping (AIAutoDeployFrameDelay: the list by difficulty index; no list: empty, and the row does not apply)
            auto_deploy_delays: {
                let l = g.ai_auto_deploy_frame_delay.as_deref().unwrap_or_default();
                [0, 1, 2].map(|i| l.get(i).copied())
            },
            // spec: S-0201 Constants (rules lobby game speed absent: index 6; claim 7 covers 0 to 6)
            lobby_game_speed: g.lobby_game_speed.map_or(6, |v| v.clamp(0, 6) as u8),
            // spec: S-0214 Import mapping (EngineerCaptureLevel: stored in single precision; absent 1.0)
            engineer_capture: g
                .engineer_capture_level
                .as_ref()
                .map_or(one, |d| d.single_value(mode)),
            hover: HoverRules {
                // spec: S-0503 Import mapping (HoverHeight absent: engine default 120 leptons)
                height: g.hover_height.unwrap_or(120),
                // spec: S-0503 Import mapping (HoverBob: single precision widened to double; absent 30.0, exact binary64)
                bob: dec(&g.hover_bob, mode).unwrap_or_else(|| nearest_double("30.0")),
                // spec: S-0503 Import mapping (HoverBoost absent: the binary64 nearest exact decimal 1.3, not through the single-precision text conversion; Q-0690 answered)
                boost: dec(&g.hover_boost, mode).unwrap_or_else(|| nearest_double("1.3")),
                // spec: S-0503 Import mapping (HoverAcceleration absent: the binary64 nearest exact decimal 0.03)
                acceleration: dec(&g.hover_acceleration, mode)
                    .unwrap_or_else(|| nearest_double("0.03")),
                // spec: S-0503 Import mapping (HoverBrake absent: the binary64 nearest exact decimal 0.03)
                brake: dec(&g.hover_brake, mode).unwrap_or_else(|| nearest_double("0.03")),
                // spec: S-0503 Import mapping (HoverDampen absent: the binary64 nearest exact decimal 0.8)
                dampen: dec(&g.hover_dampen, mode).unwrap_or_else(|| nearest_double("0.8")),
                // spec: S-0503 Import mapping and S-0306 Constants (Gravity absent: 3 leptons per tick²)
                gravity: g.gravity.unwrap_or(3),
            },
        };
        // spec: S-0211 claim 2 (start-up row: M = 0 for all eight classes, not buildable)
        let mut land = [LandRules {
            multiplier: [F32::ZERO; 8],
            buildable: false,
        }; 12];
        for lt in LAND_TYPES {
            if let Some(row) = s.land_types.get(&lt) {
                let m = |d: &Option<Decimal>| -> F32 {
                    // spec: S-0211 Formulas (M = σ(min(d, 1.0)); a missing key gives 1.0)
                    let v = d.as_ref().map_or(one, |d| d.value(mode)).min(one);
                    v.to_f32(mode)
                };
                land[usize::from(land_label(lt))] = LandRules {
                    multiplier: [
                        m(&row.foot),
                        m(&row.track),
                        m(&row.wheel),
                        m(&row.hover),
                        // spec: S-0211 claim 2 (b) (the winged multiplier is 1.0 whatever the section says)
                        one.to_f32(mode),
                        m(&row.float),
                        m(&row.amphibious),
                        m(&row.float_beach),
                    ],
                    // spec: S-0211 claim 2 (c) (a missing buildable key gives no)
                    buildable: row.buildable.unwrap_or(false),
                };
            }
        }
        let behaviours = Behaviour::ALL
            .iter()
            .map(|b| {
                let p = s.behaviours.get(b.key()).cloned().unwrap_or_default();
                // spec: S-0215 Constants (rate absent: the double nearest 0.016)
                let rate = dec(&p.rate, mode).unwrap_or_else(|| nearest_double("0.016"));
                // spec: S-0215 claim 21 (anti-air rate absent or 0: the rate)
                let aa = dec(&p.aa_rate, mode)
                    .filter(|v| !v.eq_num(F64::ZERO))
                    .unwrap_or(rate);
                BehaviourRules {
                    // spec: S-0215 Constants (flag defaults: no-threat no, paralysed no, recruitable yes, retaliate yes, scatter yes, zombie no)
                    no_threat: p.no_threat.unwrap_or(false),
                    paralysed: p.paralysed.unwrap_or(false),
                    recruitable: p.recruitable.unwrap_or(true),
                    retaliate: p.retaliate.unwrap_or(true),
                    scatter: p.scatter.unwrap_or(true),
                    zombie: p.zombie.unwrap_or(false),
                    rate_delay: rate_delay(rate, mode),
                    aa_rate_delay: rate_delay(aa, mode),
                }
            })
            .collect();
        let registry =
            |key: &str| -> Vec<String> { r.registries.get(key).cloned().unwrap_or_default() };
        let country_names = registry("countries");
        let countries = country_names
            .iter()
            .map(|n| {
                let c = s.countries.get(n).cloned().unwrap_or_default();
                CountryRules {
                    // spec: S-0207 Import mapping (SpeedUnitsMult absent: 1.0, single precision)
                    speed_units: c.speed_units_mult.map_or(one, |d| d.single_value(mode)),
                    // spec: S-0208 Import mapping (SpeedInfantryMult absent: 1.0, single precision)
                    speed_infantry: c.speed_infantry_mult.map_or(one, |d| d.single_value(mode)),
                    // spec: S-0413 Import mapping (SpeedAircraftMult absent: 1.0, single precision)
                    speed_aircraft: c
                        .econ
                        .speed_aircraft_mult
                        .clone()
                        .map_or(one, |d| d.single_value(mode)),
                    // spec: S-0009 Import mapping (MultiplayPassive absent: false)
                    passive: c.multiplay_passive.unwrap_or(false),
                    // spec: S-0210 Constants (wall-owner flag absent: yes)
                    wall_owner: c.wall_owner.unwrap_or(true),
                    // spec: S-0009 claim 33 and Import mapping (the base rules' passive field; absent: false)
                    passive_base: c.multiplay_passive_base.unwrap_or(false),
                }
            })
            .collect();
        // spec: S-0306 Constants (gravity absent: 3 leptons per tick²)
        let gravity = g.gravity.unwrap_or(3);
        // spec: S-0520 Edge cases (the gas warhead is the one registered under the fixed name; when the rules name none, a warhead with every field at its absent-key default is created)
        let (weapons_c, projectiles_c, mut warheads_c, weapon_index) = crate::combat::data::tables(
            &s.weapons,
            &s.projectiles,
            &s.warheads,
            gravity,
            quirks.negative_range_fields,
            mode,
        );
        if !s.warheads.keys().any(|k| k.eq_ignore_ascii_case("GAS")) {
            // The default warhead goes last, so no other warhead's index moves.
            let one: BTreeMap<String, rs::WarheadType> =
                [("GAS".to_owned(), Default::default())].into();
            let (_, _, mut added, _) = crate::combat::data::tables(
                &BTreeMap::new(),
                &BTreeMap::new(),
                &one,
                gravity,
                quirks.negative_range_fields,
                mode,
            );
            warheads_c.append(&mut added);
        }
        // spec: S-0309 Import mapping (the game-wide default coefficients; engine default 0)
        let coefficient_defaults: [F64; 5] = g
            .coefficient_defaults
            .as_ref()
            .map_or([F64::ZERO; 5], |v| v.clone().map(|d| d.value(mode)));
        let cx = CombatCx {
            weapons: &weapon_index,
            coefficients: coefficient_defaults,
            negative_range_fields: quirks.negative_range_fields,
        };
        let mut names = BTreeMap::new();
        let units = |kind: Kind,
                     map: &BTreeMap<String, UnitType>,
                     names: &mut BTreeMap<(Kind, String), u32>| {
            registry(kind.registry())
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    names.insert((kind, n.clone()), i as u32);
                    unit_rules(
                        kind,
                        n,
                        map.get(n).cloned().unwrap_or_default(),
                        &quirks,
                        mode,
                        &country_names,
                        &cx,
                    )
                })
                .collect::<Vec<_>>()
        };
        let infantry = units(Kind::Infantry, &s.infantry, &mut names);
        let vehicles = units(Kind::Vehicle, &s.vehicles, &mut names);
        let aircraft = units(Kind::Aircraft, &s.aircraft, &mut names);
        let buildings = registry("buildings")
            .iter()
            .enumerate()
            .map(|(i, n)| {
                names.insert((Kind::Building, n.clone()), i as u32);
                building_rules(
                    n,
                    s.buildings.get(n).cloned().unwrap_or_default(),
                    &quirks,
                    mode,
                    &cx,
                )
            })
            .collect();
        let overlays: Vec<OverlayRules> = registry("overlays")
            .iter()
            .map(|n| overlay_rules(n, s.overlays.get(n).cloned().unwrap_or_default()))
            .collect();
        let terrain = registry("terrain_objects")
            .iter()
            .enumerate()
            .map(|(i, n)| {
                names.insert((Kind::Terrain, n.clone()), i as u32);
                let t = s.terrain_objects.get(n).cloned().unwrap_or_default();
                TerrainRules {
                    name: n.clone(),
                    // spec: S-0211 Import mapping and S-0209 Import mapping (occupation bits absent: 7, an engine constant; category 2 by S-0211 claim 11 rule 4)
                    occupation: (
                        t.temperate_occupation_bits.unwrap_or(7),
                        t.snow_occupation_bits.unwrap_or(7),
                    ),
                    // spec: S-0203 Import mapping (SpawnsTiberium absent: no)
                    spawns_resources: t.spawns_tiberium.unwrap_or(false),
                    // spec: S-0102 claim 23 and S-0009 claim 42 (a terrain type's strength left unset, which the importer writes as absent: health −1)
                    strength: t.strength.unwrap_or(-1),
                    // spec: S-0305 claim 2 (a type no layer has given the field: class none)
                    armour: t.armor.unwrap_or_default().index(),
                    immune: t.immune.unwrap_or(false),
                    // spec: S-0214 Import mapping (object type LegalTarget absent: yes)
                    legal_target: t.legal_target.unwrap_or(true),
                    // spec: S-0214 Import mapping (object type Bombable absent: yes)
                    bombable: t.bombable.unwrap_or(true),
                    // spec: S-0401 Import mapping (IsAnimated absent: no)
                    animated: t.is_animated.unwrap_or(false),
                    // spec: S-0401 Import mapping (AnimationRate absent: 0)
                    animation_rate: t.animation_rate.unwrap_or(0),
                    // spec: S-0401 Import mapping and F8 (AnimationProbability, a single; absent: 0)
                    animation_probability: t
                        .animation_probability
                        .as_ref()
                        .map_or(F64::ZERO, |d| d.single_value(mode)),
                    // spec: S-0401 claim 15 (N, the sprite's image frames; absent: no frames known, so the mine never makes resource, as with fewer than 2)
                    image_frames: t.image_frames.unwrap_or(0),
                }
            })
            .collect();
        let index = |kind: Kind, n: &Option<String>| -> Option<u32> {
            n.as_ref()
                .and_then(|n| names.get(&(kind, n.clone())).copied())
        };
        let base_units = g
            .base_unit
            .iter()
            .flatten()
            .filter_map(|n| names.get(&(Kind::Vehicle, n.clone())).copied())
            .collect();
        // spec: S-0112 Import mapping (gate and wall tower types; absent: none)
        let gates = GateTypes {
            allied_one: index(Kind::Building, &g.allied_gate_one),
            allied_two: index(Kind::Building, &g.allied_gate_two),
            soviet_one: index(Kind::Building, &g.soviet_gate_one),
            soviet_two: index(Kind::Building, &g.soviet_gate_two),
            tower: index(Kind::Building, &g.wall_tower),
        };
        let particle_names = registry("particles");
        let anim_names = registry("animations");
        let overlay_names: Vec<String> = overlays.iter().map(|o| o.name.clone()).collect();
        let mut animations: Vec<AnimRules> = anim_names
            .iter()
            .map(|n| {
                let a = s.animations.get(n).cloned().unwrap_or_default();
                // spec: S-0109 Import mapping (RandomRate absent: no random rate, the range (0, 0))
                let [r1, r2] = a.random_rate.unwrap_or([0, 0]);
                AnimRules {
                    name: n.clone(),
                    random_delay: anim_random_delay(r1, r2, &quirks),
                    // spec: S-0109 Terms (a bouncing type: the bouncer flag or the meteor flag)
                    bouncing: a.bouncer.unwrap_or(false) || a.is_meteor.unwrap_or(false),
                    mu: a.middle_frame,
                    // spec: S-0513 claim 46 and Import mapping (F, the image-frame count of the sprite; absent without art)
                    frames: a.frame_count,
                    // spec: S-0513 F4 (the image header's canvas height)
                    canvas_height: a.canvas_height,
                    // spec: S-0109 Import mapping (Rate absent: delay 1; Normalized absent: no)
                    rate: a.rate,
                    normalized: a.normalized.unwrap_or(false),
                    // spec: S-0109 Import mapping (Start absent: 0; End absent: the sprite's E_s; Shadow absent: no)
                    start: a.start.unwrap_or(0),
                    end: a.end,
                    shadow: a.shadow.unwrap_or(false),
                    particles: a.spawns_particle.as_ref().map(|p| {
                        let t = s.particles.get(p).cloned().unwrap_or_default();
                        let mut rules = particle_draw_rules(&t);
                        rules.index = particle_names
                            .iter()
                            .position(|n| n == p)
                            .and_then(|i| u32::try_from(i).ok());
                        // spec: S-0202 Import mapping (NumParticles absent 0)
                        (rules, a.num_particles.unwrap_or(0))
                    }),
                    // spec: S-0109 Import mapping (Scorch, Crater, TiberiumChainReaction: booleans, absent no)
                    scorch: a.scorch.unwrap_or(false),
                    crater: a.crater.unwrap_or(false),
                    chain_reaction: a.tiberium_chain_reaction.unwrap_or(false),
                    // spec: S-0615 Import mapping (IsFlamingGuy absent: no; RunningFrames absent: 0 frames)
                    flaming: a.is_flaming_guy.unwrap_or(false),
                    running_frames: a.running_frames.unwrap_or(0),
                    // spec: S-0109 Import mapping and F5 (Damage absent: 0; δ in the S-0003 decimal form, a single held exactly in binary64)
                    damage: a
                        .damage
                        .as_ref()
                        .map_or(glasnost_num::F64::ZERO, |d| d.single_value(mode)),
                    // spec: S-0611 Import mapping and S-0109 Import mapping (MakeInfantry absent: −1, none)
                    make_infantry: a.make_infantry.unwrap_or(-1),
                    // spec: S-0109 Import mapping (LoopStart and LoopCount absent 0; LoopEnd absent: the sprite end, F2; Reverse and PingPong absent no; RandomLoopDelay absent 0,0; Next absent none)
                    loop_start: a.loop_start.unwrap_or(0),
                    loop_end: a.loop_end,
                    loop_count: a.loop_count.unwrap_or(0),
                    reverse: a.reverse.unwrap_or(false),
                    ping_pong: a.ping_pong.unwrap_or(false),
                    random_loop_delay: a.random_loop_delay.map_or((0, 0), |[x, y]| (x, y)),
                    // spec: S-0004 claim 13 (a type name compared without regard to letter case)
                    next: a.next.clone(),
                    next_index: a.next.as_ref().and_then(|x| {
                        anim_names
                            .iter()
                            .position(|n| n.eq_ignore_ascii_case(x))
                            .and_then(|i| u32::try_from(i).ok())
                    }),
                    new_theater: a.new_theater.unwrap_or(false),
                    bounce: crate::bounce::BounceType::new(&a, &overlay_names, &quirks, mode),
                    // spec: S-0109 claim 17 and Import mapping (an exact, byte-for-byte comparison with the stored name, the first-registered spelling: a type first registered in another letter case never matches)
                    invisibility: n.as_bytes() == INVISIBILITY_ANIM_NAME,
                }
            })
            .collect();
        let combat = crate::combat::data::combat_rules(
            s,
            (weapons_c, projectiles_c, warheads_c, weapon_index),
            gravity,
            &names,
            &country_names,
            mode,
        );
        let crates = crate::crates::CrateRules::new(r, &names, &overlays, &combat, mode);
        let econ = crate::econ::EconRules::new(r, mode);
        let mut missiles = crate::spawn::MissileRules::new(s, &names, &combat.warheads, mode);
        // spec: S-0611 F5 (a chain of spawned and slave types that leads back to a type already in it: the importer warns)
        missiles.warnings = crate::objects::creation_cycles(&missiles, &econ.harvest);
        let superweapons = crate::superweapons::SwRules::new(r, &names, &combat, &animations, mode);
        let warhead_names: Vec<String> = combat.warheads.iter().map(|h| h.name.clone()).collect();
        for a in &mut animations {
            a.bounce.resolve(&warhead_names);
        }
        let gas = crate::gas::GasRules::new(r, &warhead_names, &quirks);
        let pieces = crate::pieces::PieceRules::new(r, &warhead_names, &gas, mode);
        // spec: S-0319 r30 claim 15, S-0317 r59 claim 29, S-0306 r87 claim 21a, S-0616 r29 claim 8a (the loops of the links between types)
        let loops = crate::loops::LinkLoops::new(&animations, &pieces, &combat, &gas);
        let mut anim_by_name = BTreeMap::new();
        for (i, a) in animations.iter().enumerate() {
            if let Ok(i) = u32::try_from(i) {
                anim_by_name
                    .entry(a.name.as_bytes().to_ascii_lowercase())
                    .or_insert(i);
            }
        }
        let mut names_folded: BTreeMap<Kind, BTreeMap<Vec<u8>, u32>> = BTreeMap::new();
        for ((k, n), &i) in &names {
            names_folded
                .entry(*k)
                .or_default()
                .insert(n.as_bytes().to_ascii_lowercase(), i);
        }
        Self {
            anim_by_name,
            anim_by_name_len: animations.len(),
            names_folded,
            names_folded_len: names.len(),
            loops,
            crates,
            superweapons,
            missiles,
            air: crate::air::AirRules::new(r, mode),
            cloak: crate::cloak::CloakRules::new(g, &names, mode),
            econ,
            triggers: crate::triggers::Rules::new(r),
            combat,
            quirks,
            mode,
            general,
            animations,
            gas,
            pieces,
            // spec: S-0202 Import mapping (OreTwinkle: no animation at database creation)
            ore_twinkle: g.ore_twinkle.clone(),
            // spec: S-0207 Import mapping (WarpOut: with none in force, no animation is played)
            warp_out: g.warp_out.clone(),
            // spec: S-0207 Import mapping ([General] Wake: engine default none)
            wake: g.wake.clone(),
            base_units,
            // spec: S-0009 Terms and Import mapping (the game's tech level: the base rules' value, 10 when they omit it)
            game_tech_level: g.multiplayer_tech_level.unwrap_or(10),
            // spec: S-0009 Constants and Import mapping (maximum player count when the base rules omit it: 8)
            max_players: g.max_players.unwrap_or(8),
            gates,
            land,
            behaviours,
            countries,
            country_names,
            infantry,
            vehicles,
            aircraft,
            buildings,
            overlays,
            terrain,
            weapons: s.weapons.clone(),
            projectiles: s.projectiles.clone(),
            warheads: s.warheads.clone(),
            names,
            hash: hash.to_owned(),
        }
    }

    /// A behaviour's properties.
    pub fn behaviour(&self, b: Behaviour) -> &BehaviourRules {
        &self.behaviours[b.index()]
    }

    /// A mobile type's rules.
    pub fn unit(&self, kind: Kind, index: u32) -> Option<&UnitRules> {
        let v = match kind {
            Kind::Infantry => &self.infantry,
            Kind::Vehicle => &self.vehicles,
            Kind::Aircraft => &self.aircraft,
            _ => return None,
        };
        v.get(index as usize)
    }

    /// The animation type a name names (S-0003 claim 14: ASCII case
    /// folding), if the rules list it.
    pub fn animation(&self, name: &str) -> Option<&AnimRules> {
        self.anim_index(name.as_bytes())
            .and_then(|i| self.animations.get(i as usize))
    }

    /// The registration index of the animation type a name names: the
    /// first registered type whose name matches with ASCII letters folded
    /// (S-0003 claim 14, S-0004 claim 13), if the rules list one.
    pub fn anim_index(&self, name: &[u8]) -> Option<u32> {
        if self.anim_by_name_len != self.animations.len() {
            // The list changed after the rules were built: search it.
            return self
                .animations
                .iter()
                .position(|a| a.name.as_bytes().eq_ignore_ascii_case(name))
                .and_then(|i| u32::try_from(i).ok());
        }
        let mut buf = [0u8; 64];
        if let Some(b) = buf.get_mut(..name.len()) {
            b.copy_from_slice(name);
            b.make_ascii_lowercase();
            self.anim_by_name.get(&*b).copied()
        } else {
            self.anim_by_name.get(&name.to_ascii_lowercase()).copied()
        }
    }

    /// The registration index of the type of `kind` whose name, with ASCII
    /// letters folded to lower case, equals `folded` (already folded); of
    /// several such types, the one whose name comes last in `names` order
    /// (#400: what a scan of `names` keeping its last match finds).
    pub fn index_of_folded(&self, kind: Kind, folded: &[u8]) -> Option<u32> {
        if self.names_folded_len != self.names.len() {
            // The names changed after the rules were built: search them.
            return self
                .names
                .iter()
                .filter(|((k, n), _)| *k == kind && n.as_bytes().eq_ignore_ascii_case(folded))
                .map(|(_, &i)| i)
                .last();
        }
        self.names_folded.get(&kind)?.get(folded).copied()
    }

    /// A type's registration index by kind and name.
    pub fn index_of(&self, kind: Kind, name: &str) -> Option<u32> {
        self.names.get(&(kind, name.to_owned())).copied()
    }

    /// M(ℓ, c) for a land label and locomotion class (S-0211 claim 2).
    pub fn multiplier(&self, land: u8, class: LocomotionClass) -> F32 {
        self.land
            .get(usize::from(land))
            .map_or(F32::ZERO, |row| row.multiplier[class_index(class)])
    }
}

/// What the resolution of a type's combat fields needs from the rules.
struct CombatCx<'a> {
    weapons: &'a BTreeMap<String, u32>,
    coefficients: [F64; 5], // spec: S-0309 F4 (five coefficients)
    /// `quirks.negative_range_fields` (S-0307 Edge cases), for the air range bonus.
    negative_range_fields: bool,
}

#[allow(clippy::too_many_arguments)]
fn unit_rules(
    kind: Kind,
    name: &str,
    u: UnitType,
    q: &Quirks,
    mode: Rounding,
    countries: &[String],
    cx: &CombatCx<'_>,
) -> UnitRules {
    let b = |x: Option<bool>| x.unwrap_or(false);
    // spec: S-0211 claim 3 (infantry start as foot and aircraft as winged before their sections)
    let class = match kind {
        Kind::Infantry => Some(u.locomotion_class.unwrap_or(LocomotionClass::Foot)),
        Kind::Aircraft => Some(u.locomotion_class.unwrap_or(LocomotionClass::Winged)),
        _ => u.locomotion_class,
    };
    // spec: S-0206 claim 7 (infantry bodies turn at 127 with quirks.infantry_fixed_turn_rate)
    let turn = if kind == Kind::Infantry && q.infantry_fixed_turn_rate {
        127
    } else {
        // spec: S-0206 Import mapping (ROT absent: 0)
        turn_rate(u.rot.unwrap_or(0), q.turn_rate_negative_wrap)
    };
    // spec: S-0207 claim 14 (absent from every layer: the teleporting locomotor, whatever the kind)
    let loco = u.locomotor.unwrap_or(rs::Locomotor::Teleport);
    let locomotor = match (kind, loco) {
        // spec: S-0207 Terms (a drive vehicle: a vehicle with the driving locomotor)
        (Kind::Vehicle, rs::Locomotor::Drive) => Locomotor::Drive,
        // spec: S-0208 (an infantry unit with the walking locomotor)
        (Kind::Infantry, rs::Locomotor::Walk) => Locomotor::Walk,
        // spec: S-0207 claim 20 and S-0208 claim 19 (every object with the teleporting locomotor moves by it: vehicles, infantry and aircraft)
        (Kind::Vehicle | Kind::Infantry | Kind::Aircraft, rs::Locomotor::Teleport) => {
            Locomotor::Teleport
        }
        // spec: S-0501 Terms (an aircraft: an object of an aircraft type that moves with the flying locomotor)
        (Kind::Aircraft, rs::Locomotor::Fly) => Locomotor::Fly,
        // spec: S-0506 Terms (a rocket missile: an aircraft-kind object in the rocket-flight movement mode)
        (Kind::Aircraft, rs::Locomotor::Rocket) => Locomotor::Rocket,
        // spec: S-0504 Terms (a ship: a vehicle with the ship locomotor)
        (Kind::Vehicle, rs::Locomotor::Ship) => Locomotor::Ship,
        // spec: S-0503 Terms and claim 1 (a hovering vehicle: a vehicle with the hovering locomotor; the locomotor alone chooses the model)
        (Kind::Vehicle, rs::Locomotor::Hover) => Locomotor::Hover,
        // spec: S-0502 claim 1 and Terms (a unit that gets the jumpjet locomotor: an infantry or vehicle type whose imported locomotor is the jumpjet one; the locomotor, not the JumpJet flag, chooses the model)
        (Kind::Vehicle | Kind::Infantry, rs::Locomotor::Jumpjet) => Locomotor::Jump,
        _ => Locomotor::None,
    };
    let cells = |d: &Option<Decimal>| -> i32 {
        // spec: S-0215 Import mapping (GuardRange: cells × 256, truncated)
        d.as_ref().map_or(0, |d| {
            d.value(mode)
                .mul(F64::from_i64(256, mode), mode)
                .to_i32_common()
        })
    };
    UnitRules {
        name: name.to_owned(),
        kind,
        // spec: S-0107 Import mapping (Strength absent: 0)
        strength: u.strength.unwrap_or(0),
        // spec: S-0207 claim 1 (speed field absent: type speed 0)
        speed: type_speed(u.speed.unwrap_or(0)),
        turn_rate: turn,
        turret: b(u.turret),
        turret_spins: b(u.turret_spins),
        class,
        // spec: S-0211 Import mapping (MovementZone absent: normal)
        zone: u.movement_zone.unwrap_or(MovementZone::Normal),
        locomotor,
        restricted_to: u.movement_restricted_to.map(land_label),
        crusher: b(u.crusher),
        omni_crusher: b(u.omni_crusher),
        omni_crush_resistant: b(u.omni_crush_resistant),
        crushable: b(u.crushable),
        // spec: S-0107 Import mapping (DeployedCrushable absent: true)
        deployed_crushable: u.deployed_crushable.unwrap_or(true),
        // spec: S-0207 Constants (acceleration flag absent: on)
        accelerates: u.accelerates.unwrap_or(true),
        // spec: S-0207 claim 17 (acceleration factor absent: the double nearest 0.03)
        acceleration: dec(&u.acceleration_factor, mode).unwrap_or_else(|| nearest_double("0.03")),
        // spec: S-0207 claim 17 (deceleration factor absent: the double nearest 0.002)
        deceleration: dec(&u.deacceleration_factor, mode)
            .unwrap_or_else(|| nearest_double("0.002")),
        // spec: S-0207 claim 17 (slowdown distance absent: the integer 500 leptons)
        slowdown_distance: u.slowdown_distance.unwrap_or(500),
        passive: b(u.passive),
        train: b(u.is_train),
        fraidycat: b(u.fraidycat),
        cyborg: b(u.cyborg),
        fearless: b(u.fearless),
        // spec: S-0107 Import mapping (Crawls absent: true)
        crawls: u.crawls.unwrap_or(true),
        engineer: b(u.engineer),
        vehicle_thief: b(u.vehicle_thief),
        harvester: b(u.harvester),
        default_to_guard_area: b(u.default_to_guard_area),
        primary: u.primary.clone(),
        secondary: u.secondary.clone(),
        sight: u.sight.unwrap_or(0),
        guard_range: cells(&u.guard_range),
        // spec: S-0212 Import mapping (ThreatAvoidanceCoefficient absent: 0.0, single precision)
        threat_avoidance: u
            .threat_avoidance_coefficient
            .as_ref()
            .map_or(F64::ZERO, |d| d.single_value(mode)),
        jumpjet: b(u.jumpjet),
        // spec: S-0511 claim 15 (a type with jumpjet locomotion)
        jumpjet_locomotor: loco == rs::Locomotor::Jumpjet,
        jump: JumpFields::from_type(&u, mode, q.turn_rate_negative_wrap),
        // spec: S-0502 Import mapping (DeployToLand absent: no)
        deploy_to_land: b(u.deploy_to_land),
        // spec: S-0517 Import mapping (DeployingAnim absent: none; UnloadingClass absent: none, a name matching no vehicle type is none)
        deploying_anim: u.deploying_anim.clone(),
        unloading_class: u.unloading_class.clone(),
        abilities: (u.veteran_abilities.clone(), u.elite_abilities.clone()),
        // spec: S-0507 Import mapping (Passengers absent: 0, not a transport)
        passengers: u.passengers.unwrap_or(0),
        // spec: S-0507 Import mapping (Size absent: 1.0)
        size: dec(&u.size, mode).unwrap_or_else(|| F64::from_i64(1, mode)),
        // spec: S-0507 Import mapping (SizeLimit absent: 0.0, no replacement value)
        size_limit: dec(&u.size_limit, mode).unwrap_or(F64::ZERO),
        // spec: S-0507 Import mapping (OpenTopped absent: no)
        open_topped: b(u.open_topped),
        // spec: S-0507 Import mapping (IsDropship absent: no)
        dropship: b(u.is_dropship),
        // spec: S-0507 Import mapping (TurretCount absent: 0)
        turret_count: u.turret_count.unwrap_or(0),
        sequences: u.sequences.clone(),
        deployer: b(u.deployer),
        // spec: S-0214 Import mapping (UndeployDelay absent: −1, none)
        undeploy_delay: u.undeploy_delay.unwrap_or(-1),
        simple_deployer: b(u.is_simple_deployer),
        opportunity_fire: b(u.opportunity_fire),
        // spec: S-0214 Import mapping (MoveToShroud absent: yes)
        move_to_shroud: u.move_to_shroud.unwrap_or(true),
        // spec: S-0009 Import mapping (Cost absent: 0)
        cost: u.cost.unwrap_or(0),
        // spec: S-0009 Import mapping (TechLevel absent: 255)
        tech_level: u.tech_level.unwrap_or(255),
        // spec: S-0009 Import mapping (AllowedToStartInMultiplayer absent: yes)
        allowed_to_start: u.allowed_to_start_in_multiplayer.unwrap_or(true),
        // spec: S-0009 Import mapping (Owner absent: empty, no country)
        owners: u
            .owner
            .iter()
            .flatten()
            .filter_map(|n| countries.iter().position(|c| c == n).map(|i| i as u32))
            .collect(),
        // spec: S-0009 Import mapping (SmallVisceroid, LargeVisceroid: either one on; absent: no)
        visceroid: u.small_visceroid.unwrap_or(false) || u.large_visceroid.unwrap_or(false),
        combat: unit_combat(kind, &u, cx, mode),
        vehicle: crate::combat::data::vehicle_combat(&u, mode),
        raw: u,
    }
}

/// A mobile type's combat fields (S-0308 claim 1 for the slots).
fn unit_combat(
    kind: Kind,
    u: &UnitType,
    cx: &CombatCx<'_>,
    mode: Rounding,
) -> crate::combat::data::TypeCombat {
    let mut c = crate::combat::data::type_combat(
        &u.combat,
        [
            u.primary.as_ref(),
            u.secondary.as_ref(),
            u.elite_primary.as_ref(),
            u.elite_secondary.as_ref(),
        ],
        // spec: S-0308 Import mapping (TurretCount absent: 0)
        u.turret_count.unwrap_or(0),
        kind == Kind::Infantry,
        (&u.veteran_abilities, &u.elite_abilities),
        cx.coefficients,
        cx.weapons,
        mode,
    );
    // spec: S-0307 Edge cases (quirks.negative_range_fields, the air range bonus)
    c.air_range_bonus =
        crate::combat::data::range_field(c.air_range_bonus, cx.negative_range_fields, false);
    let w = |n: &Option<String>| n.as_ref().and_then(|n| cx.weapons.get(n).copied());
    c.gunner = u.gunner.unwrap_or(false);
    // spec: S-0308 Constants (mode index absent 0, passenger weapon index absent −1, deploy weapon index absent 1)
    c.ifv_mode = u.ifv_mode.unwrap_or(0);
    c.open_transport_weapon = u.open_transport_weapon.unwrap_or(-1);
    c.deploy_fire_weapon = u.deploy_fire_weapon.unwrap_or(1);
    c.deploy_fire = u.deploy_fire.unwrap_or(false);
    c.occupy_weapon = w(&u.occupy_weapon);
    c.elite_occupy_weapon = w(&u.elite_occupy_weapon);
    // spec: S-0301 Import mapping (MobileFire absent: yes)
    c.mobile_fire = u.mobile_fire.unwrap_or(true);
    c.immune = u.immune.unwrap_or(false);
    c.immune_radiation = u.immune_to_radiation.unwrap_or(false);
    // spec: S-0514 Import mapping (Warpable absent: yes)
    c.warpable = u.warpable.unwrap_or(true);
    c
}

fn building_rules(
    name: &str,
    bt: BuildingType,
    q: &Quirks,
    mode: Rounding,
    cx: &CombatCx<'_>,
) -> BuildingRules {
    let f = bt.foundation.unwrap_or([1, 1]);
    let mut combat = crate::combat::data::type_combat(
        &bt.combat,
        [bt.primary.as_ref(), bt.secondary.as_ref(), None, None],
        // spec: S-0308 Import mapping (TurretCount of a type section, buildings included; absent: 0) and S-0609 Terms (a gattling building that is a multi-weapon type reads its stage)
        bt.turret_count.unwrap_or(0),
        false,
        (&[], &[]),
        cx.coefficients,
        cx.weapons,
        mode,
    );
    // spec: S-0307 Edge cases (quirks.negative_range_fields, the air range bonus)
    combat.air_range_bonus =
        crate::combat::data::range_field(combat.air_range_bonus, cx.negative_range_fields, false);
    combat.immune = bt.immune.unwrap_or(false);
    // spec: S-0302 Import mapping (ImmuneToRadiation absent: no)
    combat.immune_radiation = bt.immune_to_radiation.unwrap_or(false);
    // spec: S-0514 Import mapping (Warpable absent: yes)
    combat.warpable = bt.warpable.unwrap_or(true);
    // spec: S-0610 claim 1 (both psychic immunities absent from every layer: yes for building types)
    combat.immune_psionic_weapons = bt.combat.immune_to_psionic_weapons.unwrap_or(true);
    combat.immune_psionics = bt.combat.immune_to_psionics.unwrap_or(true);
    let b = |x: Option<bool>| x.unwrap_or(false);
    let building = crate::combat::data::BuildingCombat {
        defense: b(bt.defense),
        // spec: S-0302 Constants (can-be-demolished flag absent: yes)
        can_c4: bt.can_c4.unwrap_or(true),
        // spec: S-0308 Import mapping (ResourceGatherer and ResourceDestination absent: no)
        gatherer_destination: b(bt.econ.resource_gatherer) && b(bt.econ.resource_destination),
        delay_kill: b(bt.eligible_for_delay_kill),
        leave_rubble: b(bt.leave_rubble),
        // spec: S-0518 Import mapping (Overpowerable absent: no)
        overpowerable: b(bt.overpowerable),
        helipad: b(bt.helipad),
        // spec: S-0312 Import mapping (Crewed: the default is left to the type specs, which give none for buildings; Q-2135, interim: absent is no)
        crewed: b(bt.crewed),
        artillary: b(bt.artillary),
        can_be_occupied: b(bt.can_be_occupied),
        can_occupy_fire: b(bt.can_occupy_fire),
        // spec: S-0313 Import mapping (MaxNumberOccupants absent: 0)
        max_occupants: bt.max_number_occupants.unwrap_or(0),
        // spec: S-0313 Import mapping (garrison muzzle points absent: (0, 0) each)
        garrison_muzzles: bt.garrison_muzzles.clone().unwrap_or_default(),
        // spec: S-0314 Import mapping (self-heal counts absent: 0)
        infantry_self_heal: bt.infantry_gain_self_heal.unwrap_or(0),
        units_self_heal: bt.units_gain_self_heal.unwrap_or(0),
        // spec: S-0306 Import mapping (TargetCoordOffset absent: 0, 0, 0; Height absent: 2)
        aim_shift: bt.target_coord_offset.unwrap_or([0; 3]),
        art_height: bt.art_height.unwrap_or(2), // spec: S-0306 Import mapping (Height absent: 2)
        voxel_turret: b(bt.turret_anim_is_voxel),
        voxel_barrel: b(bt.barrel_anim_is_voxel),
        // spec: S-0313 claim 7 (a pixel shift of (65535, 65535) counts as not given)
        fire_pixel: bt
            .primary_fire_pixel_offset
            .filter(|p| *p != [65535, 65535]),
        dual_offset: b(bt.primary_fire_dual_offset),
        turret_anim: bt.turret_anim_xy.unwrap_or([0; 2]), // nocite: a pixel pair, absent 0, 0 (S-0313 Import mapping)
        anim_delayed_fire: b(bt.is_anim_delayed_fire),
        // spec: S-0301 Import mapping (DelayedFireDelay absent: 0)
        delayed_fire_delay: bt.delayed_fire_delay.unwrap_or(0),
        laser_fence: b(bt.laser_fence),
        bridge_repair_hut: b(bt.bridge_repair_hut),
        weapons_factory: b(bt.weapons_factory),
        sam: b(bt.sam),
        emp_cannon: b(bt.emp_cannon),
        stupid_guard: b(bt.stupid_guard_mode),
        unit_repair: b(bt.unit_repair),
        capturable: b(bt.capturable),
    };
    // spec: S-0403 Terms and Import mapping (Factory: the kind of object named; absent: none), S-0403 Import mapping (Naval absent: no)
    let factory = bt.factory.as_deref().and_then(FactoryKind::from_word);
    // spec: S-0214 Import mapping (Factory: the imported category, including recognised non-producing ones; absent: no factory)
    let factory_category = bt.factory.as_deref().and_then(FactoryCategory::from_word);
    let naval = bt.naval.unwrap_or(false);
    let absorb = AbsorbRules {
        // spec: S-0605 Import mapping (InfantryAbsorb and UnitAbsorb absent: no)
        infantry: b(bt.infantry_absorb),
        vehicle: b(bt.unit_absorb),
        // spec: S-0605 Import mapping (Passengers of a building type absent: 0)
        capacity: bt.passengers.unwrap_or(0),
        // spec: S-0605 Import mapping (SizeLimit of a building type absent: 0.0)
        size_limit: dec(&bt.size_limit, mode).unwrap_or(F64::ZERO),
    };
    BuildingRules {
        factory,
        factory_category,
        naval,
        absorb,
        name: name.to_owned(),
        strength: bt.strength.unwrap_or(0),
        // spec: S-0108 Import mapping (Foundation absent: 1x1; the zero-size word is 0 × 0), S-0406 claim 2 and S-0009 claim 38 (a zero-size building has no foundation cells)
        foundation: (f[0].max(0), f[1].max(0)),
        turret: bt.turret.unwrap_or(false),
        turn_rate: turn_rate(bt.rot.unwrap_or(0), q.turn_rate_negative_wrap),
        gate: bt.gate.unwrap_or(false),
        invisible: bt.invisible_in_game.unwrap_or(false),
        // spec: S-0215 F9 and Constants (Td = w(trunc(RZ₅₃(td × 900))), 0 at 2⁶³; DeployTime absent: 0)
        gate_door_time: bt
            .deploy_time
            .as_ref()
            .map_or(0, |d| rate_delay(d.value(mode), mode)),
        // spec: S-0215 F9 and Constants (Tc likewise; GateCloseDelay absent: 0)
        gate_close_delay: bt
            .gate_close_delay
            .as_ref()
            .map_or(0, |d| rate_delay(d.value(mode), mode)),
        raw: bt,
        combat,
        building,
    }
}

fn overlay_rules(name: &str, o: rs::OverlayType) -> OverlayRules {
    use glasnost_map::terrain::land;
    let tiberium = o.tiberium.unwrap_or(false);
    // spec: S-0211 claim 4 (absent land type: clear); claim 5 (ore and gems with clear: tiberium)
    let mut l = o.land.map_or(land::CLEAR, land_label);
    if tiberium && l == land::CLEAR {
        l = land::TIBERIUM;
    }
    OverlayRules {
        name: name.to_owned(),
        land: l,
        own_land: o.no_use_tile_land_type.unwrap_or(false),
        wall: o.wall.unwrap_or(false),
        tiberium,
        crushable: o.crushable.unwrap_or(false),
        rock: o.rock.unwrap_or(false),
        rubble: o.rubble.unwrap_or(false),
        crate_: o.crate_.unwrap_or(false),
        crate_trigger: o.crate_trigger.unwrap_or(false),
        // spec: S-0210 claim 13 (damage-stage count absent: 1)
        damage_levels: o.damage_levels.unwrap_or(1),
        // spec: S-0210 claim 13 (wall strength absent: 1)
        strength: o.strength.unwrap_or(1),
        wood: o
            .armor
            .as_deref()
            .is_some_and(|a| a.eq_ignore_ascii_case("wood")),
        // Overlays whose art the importer did not look up are taken as drawn.
        has_art: o.has_art.unwrap_or(true),
        // spec: S-0202 Import mapping (IsVeins absent: no)
        veins: o.is_veins.unwrap_or(false),
        // spec: S-0305 claim 5 (a resource overlay has class wood whatever its field says)
        armour: if tiberium {
            rs::ArmourClass::Wood.index()
        } else {
            o.armor
                .as_deref()
                .map_or(0, |a| rs::ArmourClass::from_word(a.as_bytes()).0.index())
        },
        chain_reaction: o.chain_reaction.unwrap_or(false),
        explodes: o.explodes.unwrap_or(false),
        // spec: S-0214 Import mapping (object type LegalTarget absent: yes)
        legal_target: o.legal_target.unwrap_or(true),
        // spec: S-0009 Import mapping (CellAnim absent: no cell animation)
        cell_anim: o.cell_anim,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The name index answers as the linear case-folded search it replaced
    /// (#407): the first registered type of a folded name, names longer
    /// than the stack buffer, missing names, and a list grown after the
    /// build.
    #[test]
    fn anim_index_matches_linear_search() {
        let long = "L".repeat(80);
        let names = ["Boom", "BOOM", "smoke", "Smoke2", long.as_str(), "a"];
        let mut set = crate::testkit::ruleset();
        set.registries.insert(
            "animations".to_owned(),
            names.iter().map(|n| (*n).to_owned()).collect(),
        );
        let mut r = SimRules::new(&set, "synthetic-anim-index");
        let linear = |r: &SimRules, n: &str| {
            r.animations
                .iter()
                .position(|a| a.name.eq_ignore_ascii_case(n))
                .and_then(|i| u32::try_from(i).ok())
        };
        let long_lower = long.to_ascii_lowercase();
        let probes = [
            "boom",
            "BOOM",
            "Boom",
            "bOoM",
            "SMOKE",
            "smoke2",
            "smoke3",
            "",
            "A",
            long.as_str(),
            long_lower.as_str(),
            "nothing",
        ];
        for n in probes {
            assert_eq!(r.anim_index(n.as_bytes()), linear(&r, n), "{n}");
        }
        assert_eq!(r.anim_index(b"BOOM"), Some(0));
        assert_eq!(r.animation("SMOKE").map(|a| a.name.as_str()), Some("smoke"));
        let mut extra = r.animations[2].clone();
        extra.name = "Late".to_owned();
        r.animations.push(extra);
        assert_eq!(r.anim_index(b"late"), Some(6));
        for n in probes {
            assert_eq!(r.anim_index(n.as_bytes()), linear(&r, n), "{n}");
        }
    }

    // spec: S-0207 claim 1 (v = 100 gives 255, v = 1 gives 2, above 100 acts as 100)
    #[test]
    fn type_speeds() {
        assert_eq!(type_speed(100), 255);
        assert_eq!(type_speed(150), 255);
        assert_eq!(type_speed(1), 2);
        assert_eq!(type_speed(20), 51);
        assert_eq!(type_speed(40), 102);
        assert_eq!(type_speed(-5), 0);
    }

    // spec: S-0206 claim 6 (turn rate from the field: clamp at 127, negative values mod 256)
    #[test]
    fn turn_rates() {
        assert_eq!(turn_rate(200, true), 127);
        assert_eq!(turn_rate(126, true), 126);
        assert_eq!(turn_rate(0, true), 0);
        assert_eq!(turn_rate(-200, true), 56);
        assert_eq!(turn_rate(-5, true), 0);
        assert_eq!(turn_rate(-200, false), 0);
    }

    // spec: S-0003 claim 7 (the red and yellow ratios: the single conversion widened exactly without `%`; with `%` the double product, not narrowed)
    #[test]
    fn health_ratio_precision() {
        let rz = Rounding::TowardZero;
        let ratio = |s: &str| {
            let d = Some(glasnost_rules::sim::Decimal(s.to_owned()));
            glasnost_num::decimal::exact_decimal(dec(&d, rz).unwrap())
        };
        assert_eq!(ratio("6e-1"), "0.60000002384185791015625");
        assert_eq!(
            ratio("10%"),
            "0.09999999999999999167332731531132594682276248931884765625"
        );
    }

    // spec: S-0109 claim 2, F1 and Edge cases (⌊900 / ρ⌋, 0 for ρ ≤ 0; quirks.anim_fast_rate_freezes off: at least 1 for ρ > 0)
    #[test]
    fn anim_delays() {
        assert_eq!(anim_delay(225, true), 4);
        assert_eq!(anim_delay(901, true), 0);
        assert_eq!(anim_delay(901, false), 1);
        assert_eq!(anim_delay(225, false), 4);
        for fast in [true, false] {
            assert_eq!(anim_delay(0, fast), 0);
            assert_eq!(anim_delay(-5, fast), 0);
        }
    }

    // spec: S-0109 claim 3, F1 and Edge cases (d₂′ = max(d₂, 0), d₁′ = min(d₁, d₂′); quirks.anim_random_rate_collapse off: [min, max])
    #[test]
    fn anim_random_delays() {
        let mut q = Quirks::default();
        assert_eq!(anim_random_delay(450, 225, &q), (2, 4));
        assert_eq!(anim_random_delay(225, 450, &q), (2, 2));
        assert_eq!(anim_random_delay(225, 1000, &q), (0, 0));
        q.anim_random_rate_collapse = false;
        assert_eq!(anim_random_delay(450, 225, &q), (2, 4));
        assert_eq!(anim_random_delay(225, 450, &q), (2, 4));
        assert_eq!(anim_random_delay(225, 1000, &q), (0, 4));
        q.anim_fast_rate_freezes = false;
        assert_eq!(anim_random_delay(225, 1000, &q), (1, 4));
    }

    // spec: S-0215 F1 (0.06 gives 53, 0.05 gives 45, the default gives 14)
    #[test]
    fn rate_delays() {
        let rz = Rounding::TowardZero;
        let p = |s: &str| {
            glasnost_num::decimal::parse(s, glasnost_num::decimal::Field::Decimal, rz).unwrap()
        };
        assert_eq!(rate_delay(p("0.06"), rz), 53);
        assert_eq!(rate_delay(p("0.05"), rz), 45);
        assert_eq!(rate_delay(nearest_double("0.016"), rz), 14);
        assert_eq!(rate_delay(p("6%"), rz), 53);
        assert_eq!(rate_delay(p("6%"), Rounding::Nearest), 54);
    }

    #[allow(clippy::disallowed_types)]
    #[test]
    fn nearest_double_matches_host() {
        for (s, v) in [
            ("0.016", 0.016f64),
            ("0.083", 0.083),
            ("0.03", 0.03),
            ("0.002", 0.002),
        ] {
            assert_eq!(nearest_double(s).0, v.to_bits(), "{s}");
        }
    }
}
