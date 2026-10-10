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
    /// The raw record, for fields read only by input rules (S-0214).
    pub raw: UnitType,
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
}

/// A terrain-object type's resolved fields.
#[derive(Clone, Debug, Default)]
pub struct TerrainRules {
    /// Type name.
    pub name: String,
    /// Occupation pattern, temperate and snow.
    pub occupation: (i32, i32),
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
    /// Passive.
    pub passive: bool,
    /// The wall-owner flag (S-0210 claim 14).
    pub wall_owner: bool,
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
    /// Guard stray distance in leptons (S-0215 F5); absent: Q-0020.
    pub guard_stray: Option<i32>,
    /// Scatter threshold of the house skill level.
    pub iq_scatter: i32,
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
    /// The ruleset's `sim_rules_hash`.
    pub hash: String,
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
            // spec: S-0215 Import mapping ([IQ] GuardArea; the absent default is request Q-0020, interim 0)
            iq_guard_area: g.iq_guard_area.unwrap_or(0),
            // spec: S-0215 Import mapping (GuardModeStray: cells × 256, truncated; absent default open, Q-0020)
            guard_stray: g.guard_mode_stray.as_ref().map(|d| {
                d.value(mode)
                    .mul(F64::from_i64(256, mode), mode)
                    .to_i32_common()
            }),
            // spec: S-0209 Import mapping ([IQ] Scatter absent: 3)
            iq_scatter: g.iq_scatter.unwrap_or(3),
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
                    passive: c.multiplay_passive.unwrap_or(false),
                    // spec: S-0210 Constants (wall-owner flag absent: yes)
                    wall_owner: c.wall_owner.unwrap_or(true),
                }
            })
            .collect();
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
                building_rules(n, s.buildings.get(n).cloned().unwrap_or_default(), &quirks)
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
                    // spec: S-0209 Import mapping (occupation bits absent: 7)
                    occupation: (
                        t.temperate_occupation_bits.unwrap_or(7),
                        t.snow_occupation_bits.unwrap_or(7),
                    ),
                }
            })
            .collect();
        Self {
            quirks,
            mode,
            general,
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

fn unit_rules(kind: Kind, name: &str, u: UnitType, q: &Quirks, mode: Rounding) -> UnitRules {
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
        raw: u,
    }
}

fn building_rules(name: &str, bt: BuildingType, q: &Quirks) -> BuildingRules {
    let f = bt.foundation.unwrap_or([1, 1]);
    BuildingRules {
        name: name.to_owned(),
        strength: bt.strength.unwrap_or(0),
        // spec: S-0108 Import mapping (Foundation absent: 1x1)
        foundation: (f[0].max(1), f[1].max(1)),
        turret: bt.turret.unwrap_or(false),
        turn_rate: turn_rate(bt.rot.unwrap_or(0), q.turn_rate_negative_wrap),
        gate: bt.gate.unwrap_or(false),
        invisible: bt.invisible_in_game.unwrap_or(false),
        raw: bt,
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
