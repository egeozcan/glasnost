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
    /// Not moved in M3: every other locomotor, and a locomotor on a kind
    /// whose movement with it no spec gives (S-0207 Open questions).
    None,
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
    /// Veteran and elite ability names.
    pub abilities: (Vec<String>, Vec<String>),
    /// Passengers.
    pub passengers: i32,
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
    /// The naval flag (S-0403 Terms: a naval factory building of vehicles;
    /// S-0406 claim 3 d).
    pub naval: bool,
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
    /// `aircraft`, `building`). The importer writes none of them yet: the
    /// original's words are asked in Q-0100.
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
#[derive(Clone, Debug, Default)]
pub struct AnimRules {
    /// Type name.
    pub name: String,
    /// The random delay range [d₁′, d₂′] in ticks (S-0109 claim 3, F1).
    pub random_delay: (i32, i32),
    /// A bouncing type: the bouncer or the meteor flag (S-0109 Terms).
    pub bouncing: bool,
    /// Its start event can take draws that no approved spec states: a
    /// resource chain reaction or a middle-event effect (request Q-0046).
    pub start_effects: bool,
}

/// The stage delay d of a rate value ρ (S-0109 F1).
pub fn anim_delay(rho: i32) -> i32 {
    // spec: S-0109 F1 (d = ⌊900 / ρ⌋ for ρ > 0, 0 for ρ ≤ 0)
    if rho > 0 { 900 / rho } else { 0 }
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
    /// Maximum cheer rate, ticks.
    pub cheer_rate: i32,
    /// Area-guard threshold of the house skill level.
    pub iq_guard_area: i32,
    /// Guard stray distance in leptons (S-0215 F5); `None` when no rules
    /// layer gives it (S-0215 claim 36 (b): the leash is used instead).
    pub guard_stray: Option<i32>,
    /// Scatter threshold of the house skill level.
    pub iq_scatter: i32,
    /// The auto-deploy delay A of S-0215 claim 31 for difficulty index 1,
    /// ticks: the list's second entry, the only one our engine reads (the
    /// neutral and special houses have index 1); `None` when the list has no
    /// second entry.
    pub auto_deploy_delay: Option<i32>,
    /// The rules' lobby game-speed index (S-0201 claim 7).
    pub lobby_game_speed: u8,
    /// The engineer-capture health ratio c (S-0214 row I7), single
    /// precision widened.
    pub engineer_capture: F64,
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
    /// The ruleset's `sim_rules_hash`.
    pub hash: String,
    /// The combat rules (M4).
    pub combat: crate::combat::data::CombatRules,
    /// The rules of the M5a topics (economy, production, structures,
    /// visibility, setup).
    pub econ: crate::econ::EconRules,
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
            // spec: S-0107 Import mapping (ConditionYellow absent: 0.5)
            condition_yellow: dec(&g.condition_yellow, mode).unwrap_or_else(|| constant("0.5")),
            // spec: S-0107 Import mapping (ConditionRed absent: 0.5)
            condition_red: dec(&g.condition_red, mode).unwrap_or_else(|| constant("0.5")),
            // spec: S-0107 Import mapping (IdleActionFrequency absent: the double nearest 0.083)
            idle_frequency: dec(&g.idle_action_frequency, mode)
                .unwrap_or_else(|| nearest_double("0.083")),
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
            // spec: S-0215 claim 31 and Import mapping (AIAutoDeployFrameDelay: the entry for difficulty index 1; no second entry, or no list: none, and the row does not apply)
            auto_deploy_delay: g
                .ai_auto_deploy_frame_delay
                .as_ref()
                .and_then(|l| l.get(1).copied()),
            // spec: S-0201 Constants (rules lobby game speed absent: index 6; claim 7 covers 0 to 6)
            lobby_game_speed: g.lobby_game_speed.map_or(6, |v| v.clamp(0, 6) as u8),
            // spec: S-0214 Import mapping (EngineerCaptureLevel: stored in single precision; absent 1.0)
            engineer_capture: g
                .engineer_capture_level
                .as_ref()
                .map_or(one, |d| d.single_value(mode)),
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
        let (weapons_c, projectiles_c, warheads_c, weapon_index) =
            crate::combat::data::tables(&s.weapons, &s.projectiles, &s.warheads, gravity, mode);
        // spec: S-0309 Import mapping (the game-wide default coefficients; engine default 0)
        let coefficient_defaults: [F64; 5] = g
            .coefficient_defaults
            .as_ref()
            .map_or([F64::ZERO; 5], |v| v.clone().map(|d| d.value(mode)));
        let cx = CombatCx {
            weapons: &weapon_index,
            coefficients: coefficient_defaults,
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
        let overlays = registry("overlays")
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
        let animations = registry("animations")
            .iter()
            .map(|n| {
                let a = s.animations.get(n).cloned().unwrap_or_default();
                // spec: S-0109 Import mapping (RandomRate absent: no random rate, the range (0, 0))
                let [r1, r2] = a.random_rate.unwrap_or([0, 0]);
                let (d1, d2) = (anim_delay(r1), anim_delay(r2));
                // spec: S-0109 F1 (d₂′ = max(d₂, 0), d₁′ = min(d₁, d₂′))
                let d2p = d2.max(0);
                let d1p = d1.min(d2p);
                AnimRules {
                    name: n.clone(),
                    random_delay: (d1p, d2p),
                    // spec: S-0109 Terms (a bouncing type: the bouncer flag or the meteor flag)
                    bouncing: a.bouncer.unwrap_or(false) || a.is_meteor.unwrap_or(false),
                    // spec: S-0109 claims 13 and 14 (the start event's chain reaction and middle-event effects)
                    start_effects: a.tiberium_chain_reaction.unwrap_or(false)
                        || a.scorch.unwrap_or(false)
                        || a.crater.unwrap_or(false)
                        || a.spawns_particle.unwrap_or(false),
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
        let econ = crate::econ::EconRules::new(r, mode);
        Self {
            econ,
            combat,
            quirks,
            mode,
            general,
            animations,
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
        self.animations
            .iter()
            .find(|a| a.name.eq_ignore_ascii_case(name))
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
        abilities: (u.veteran_abilities.clone(), u.elite_abilities.clone()),
        passengers: u.passengers.unwrap_or(0),
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
        0,
        false,
        (&[], &[]),
        cx.coefficients,
        cx.weapons,
        mode,
    );
    combat.immune = bt.immune.unwrap_or(false);
    // spec: S-0302 Import mapping (ImmuneToRadiation absent: no)
    combat.immune_radiation = bt.immune_to_radiation.unwrap_or(false);
    let b = |x: Option<bool>| x.unwrap_or(false);
    let building = crate::combat::data::BuildingCombat {
        defense: b(bt.defense),
        // spec: S-0302 Constants (can-be-demolished flag absent: yes)
        can_c4: bt.can_c4.unwrap_or(true),
        delay_kill: b(bt.eligible_for_delay_kill),
        leave_rubble: b(bt.leave_rubble),
        helipad: b(bt.helipad),
        artillary: b(bt.artillary),
        can_be_occupied: b(bt.can_be_occupied),
        can_occupy_fire: b(bt.can_occupy_fire),
        // spec: S-0313 Import mapping (MaxNumberOccupants absent: 0)
        max_occupants: bt.max_number_occupants.unwrap_or(0),
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
    let naval = bt.naval.unwrap_or(false);
    BuildingRules {
        factory,
        naval,
        name: name.to_owned(),
        strength: bt.strength.unwrap_or(0),
        // spec: S-0108 Import mapping (Foundation absent: 1x1)
        foundation: (f[0].max(1), f[1].max(1)),
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
        // spec: S-0009 Import mapping (CellAnim absent: no cell animation)
        cell_anim: o.cell_anim,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
