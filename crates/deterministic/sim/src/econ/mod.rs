//! Economy, production, structures, visibility and game setup (M5a:
//! S-0401 to S-0416), in one module per topic. Every topic keeps its own
//! state in the structs below, which the world, its houses and its objects
//! carry; the topics meet only through the event hooks of [`events`], the
//! house update of [`houses`] and the orders of `crate::orders`.
//!
//! Behaviour that belongs to batch 5 and later (aircraft, naval units,
//! garrisons, transports, engineers, spies, superweapons, crates) is a seam
//! here: a named function or field with a comment that cites the claim.

use serde::{Deserialize, Serialize};

use crate::world::ObjectId;

pub mod deploy;
pub mod events;
pub mod harvest;
pub mod houses;
pub mod minimap;
pub mod placement;
pub mod power;
pub mod production;
pub mod radar;
pub mod resource;
pub mod sell;
pub mod setup;
pub mod shroud;
pub mod sidebar;
pub mod tech;
pub mod victory;

/// A type of a production kind: infantry, vehicle, aircraft or building,
/// with its registration index (what production orders name, S-0404 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TypeRef {
    /// The kind.
    pub kind: crate::rules::Kind,
    /// Registration index within the kind.
    pub index: u32,
}

/// A house's economy state (M5a), one field per topic.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseEcon {
    /// The house's buildings in the order it acquired them (built, placed,
    /// captured): S-0404 claims 2 and 7, S-0407 claim 8 (c). Kept by
    /// [`events`].
    pub acquired: Vec<ObjectId>,
    /// Power totals and blackouts (S-0405).
    pub power: power::HousePower,
    /// The tech tree's offer, counts and flags (S-0407).
    pub tech: tech::HouseTech,
    /// Radar availability (S-0410).
    pub radar: radar::HouseRadar,
    /// Defeat and the game's result for this house (S-0414).
    pub result: victory::HouseResult,
    /// Production-side state of the house that is not a line: the cost
    /// bonus product and the like (S-0404).
    pub production: production::HouseProduction,
    /// Game-setup values of this house (S-0412): team, handicap, difficulty
    /// index.
    pub setup: setup::HouseSetupState,
}

/// An object's economy state (M5a), one field per topic.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectEcon {
    /// A building's power switch, primary mark and upgrades (S-0405,
    /// S-0213 claim 13, S-0406 claim 12).
    pub building: power::BuildingSwitches,
    /// A sale in progress (S-0408).
    pub sale: Option<sell::Sale>,
    /// A harvester's load and cycle (S-0402).
    pub harvest: Option<harvest::HarvestState>,
    /// A refinery's docks (S-0402).
    pub dock: Option<harvest::DockState>,
    /// Deploying or packing up (S-0415).
    pub deploy: deploy::DeployState,
    /// Sight applied for each house (S-0409).
    pub sight: shroud::ObjectSight,
    /// Counts this object is in (S-0414 claim 4, S-0407 claim 9).
    pub counted: victory::Counted,
    /// A secret lab's secret type (S-0407 claims 4 and 14).
    pub secret: Option<TypeRef>,
}

/// The world's economy state (M5a), one field per topic.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldEcon {
    /// The game's setup values (S-0412).
    pub settings: setup::Settings,
    /// Resource timers and growth and spread lists (S-0401).
    pub resources: resource::Resources,
    /// Every production line of every house, in creation order (S-0404
    /// claim 1).
    pub lines: production::Lines,
    /// Per-house shroud (S-0409).
    pub shroud: shroud::Shroud,
    /// The end of the game (S-0414).
    pub end: victory::GameEnd,
}

/// The typed rules of the M5a topics, built from the ruleset once
/// ([`crate::rules::SimRules`] holds them). Each topic owns its part and the
/// absent-key defaults its spec gives.
#[derive(Clone, Debug, Default)]
pub struct EconRules {
    /// Power (S-0405).
    pub power: power::Rules,
    /// Shroud and sight (S-0409).
    pub shroud: shroud::Rules,
    /// Radar (S-0410).
    pub radar: radar::Rules,
    /// Build time, price and lines (S-0403, S-0404).
    pub production: production::Rules,
    /// The tech tree (S-0407).
    pub tech: tech::Rules,
}

impl EconRules {
    /// The typed rules of a resolved ruleset.
    pub fn new(r: &glasnost_rules::schema::Ruleset, mode: glasnost_num::Rounding) -> Self {
        Self {
            power: power::Rules::new(r, mode),
            shroud: shroud::Rules::new(r, mode),
            radar: radar::Rules::new(r, mode),
            production: production::Rules::new(r, mode),
            tech: tech::Rules::new(r, mode),
        }
    }
}
