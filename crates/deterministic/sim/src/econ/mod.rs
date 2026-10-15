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

pub mod body;
pub mod deploy;
pub mod events;
pub mod harvest;
pub mod houses;
pub mod minimap;
pub mod placement;
pub mod power;
pub mod production;
pub mod radar;
pub mod release;
pub mod resource;
pub mod sell;
pub mod setup;
pub mod shroud;
pub mod sidebar;
pub mod slaves;
pub mod stats;
pub mod tech;
pub mod victory;

/// Whether a unit is tethered (S-0213 Terms, the glossary's *tether
/// link*): a harvester docked at a refinery (S-0402 claim 10) or a unit
/// released by a factory (S-0417 claims 2 and 10).
pub fn tethered(w: &crate::world::World, id: crate::world::ObjectId) -> bool {
    // spec: S-0314 claim 17 (the completed depot entry exchange tethers too)
    release::tethered(w, id) || harvest::tethered(w, id) || crate::depot::tethered(w, id)
}

/// Gives object `id` the destination `dest` (none clears it) and tells the
/// movement rules at once: the destination-driven get-up of S-0107 claim
/// 15 and the teleporting locomotor's taking of the destination, whose
/// landing point and vehicle-present mark move in the same moment (S-0207
/// claim 20 (a)). Every economy rule that sets a destination goes through
/// here.
pub fn assign_destination(
    w: &mut crate::world::World,
    id: ObjectId,
    dest: Option<crate::world::Target>,
) {
    let Some(o) = w.obj_mut(id) else { return };
    let before = o.dest;
    o.dest = dest;
    // spec: S-0207 claim 20 (a) (the locomotor takes the destination at once) and S-0107 claim 15
    crate::infantry::destination_assigned(w, id, before);
}

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
    /// The harvest record of this house (S-0402 claim 13).
    pub harvest: harvest::HouseHarvest,
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
    /// A produced object's start price (S-0402 Terms, S-0404 claim 2): its
    /// price when its production started; none for an object not produced.
    #[serde(default)]
    pub start_price: Option<i32>,
    /// Deploying or packing up (S-0415).
    pub deploy: deploy::DeployState,
    /// A building's construction state (S-0108, S-0406 claim 1).
    #[serde(default)]
    pub construction: placement::Construction,
    /// Sight applied for each house (S-0409).
    pub sight: shroud::ObjectSight,
    /// Counts this object is in (S-0414 claim 4, S-0407 claim 9).
    pub counted: victory::Counted,
    /// A secret lab's secret type (S-0407 claims 4 and 14).
    pub secret: Option<TypeRef>,
    /// A released unit's contact and tether with its factory building
    /// (S-0417 claims 2, 7 and 10).
    #[serde(default)]
    pub exit: Option<release::Exit>,
    /// Driving temporarily for a factory exit (S-0417 claim 5).
    #[serde(default)]
    pub temporary_drive: bool,
    /// A building's running *idle* body range (S-0108 claim 6, S-0215
    /// claim 11); none outside the idle state.
    #[serde(default)]
    pub idle_body: Option<body::IdleBody>,
    /// A slave master's slave group (S-0402 claims 14 and 19 to 22).
    #[serde(default)]
    pub group: Option<slaves::SlaveGroup>,
    /// A slave's own state: its master and its load (S-0402 claim 14).
    #[serde(default)]
    pub slave: Option<slaves::SlaveState>,
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
    /// Game-start values: the rewritten country flags and the crate count
    /// (S-0412, S-0413).
    #[serde(default)]
    pub start: setup::StartState,
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
    /// Placement, selling and deploying (S-0406, S-0408, S-0415).
    pub placement: placement::Rules,
    /// Build time, price and lines (S-0403, S-0404).
    pub production: production::Rules,
    /// The tech tree (S-0407).
    pub tech: tech::Rules,
    /// Skirmish setup (S-0412, S-0413).
    pub setup: setup::Rules,
    /// Victory and defeat (S-0414).
    pub victory: victory::Rules,
    /// Ore and gems (S-0401).
    pub resource: resource::Rules,
    /// The harvest cycle (S-0402).
    pub harvest: harvest::Rules,
}

impl EconRules {
    /// The typed rules of a resolved ruleset.
    pub fn new(r: &glasnost_rules::schema::Ruleset, mode: glasnost_num::Rounding) -> Self {
        Self {
            power: power::Rules::new(r, mode),
            shroud: shroud::Rules::new(r, mode),
            radar: radar::Rules::new(r, mode),
            placement: placement::Rules::new(r, mode),
            production: production::Rules::new(r, mode),
            tech: tech::Rules::new(r, mode),
            setup: setup::Rules::new(r, mode),
            victory: victory::Rules::new(r, mode),
            resource: resource::Rules::new(r, mode),
            harvest: harvest::Rules::new(r, mode),
        }
    }
}
