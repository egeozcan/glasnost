//! Structures and special infantry abilities of M6c: garrisons (S-0508),
//! engineers and capture (S-0509) and spies and infiltration (S-0510); of
//! M7a: absorbing buildings (S-0605), recycling buildings (S-0603) and
//! vehicle bunkers (S-0606).
//!
//! [`entry`] is the one place where an infantry unit's arrival at a building
//! is turned into an entry (garrison, capture, infiltration or refusal);
//! [`garrison`] holds the occupants of garrisonable buildings;
//! [`absorb`] the passengers of absorbing buildings; [`grinder`] the
//! consumption of entrants by recycling buildings; [`bunker`] the entry,
//! occupancy and release of a tank bunker's vehicle.

use serde::{Deserialize, Serialize};

use crate::world::ObjectId;

pub mod absorb;
pub mod bunker;
pub mod engineer;
pub mod entry;
pub mod garrison;
pub mod grinder;
pub mod ownership;
pub mod spy;
pub mod tech;

/// The structure-topic state of an object (S-0508 Terms): a garrisonable
/// building's occupants and shared firing index, an occupant's building.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructState {
    /// A building's occupants in order: the first is the one that has been
    /// inside longest (S-0508 Terms). An occupant that dies inside has its
    /// place removed in the same update (claim 23, an engine design rule).
    pub occupants: Vec<ObjectId>,
    /// The shared occupant and muzzle index (S-0313 claim 6, S-0508 claim
    /// 18), counted from 0.
    pub index: u32,
    /// For an occupant: the building it is inside.
    pub inside: Option<ObjectId>,
    /// An absorbing building's passengers in the passenger order, the front
    /// (the most recent entrant) first (S-0605 Terms).
    pub absorbed: Vec<ObjectId>,
    /// For a passenger: the absorbing building it is inside (S-0605 claim
    /// 9).
    pub absorbed_in: Option<ObjectId>,
    /// The absorbing building an actor is waiting in line to enter (S-0605
    /// claim 4).
    pub waiting: Option<ObjectId>,
    /// The enter-nearest instruction for absorbing buildings (S-0215
    /// Terms), which only the bio-reactor fate of S-0605 claim 16 gives:
    /// while held, every *guard* and *area guard* processing searches again
    /// (S-0215 claims 24 and 26 row 3).
    pub enter_nearest: bool,
    /// The tick in which claim 16's fate put this vehicle in line for a
    /// vehicle absorber: it sends no contact request in that tick (S-0605
    /// claim 3).
    #[serde(default)]
    pub fate_tick: Option<i32>,
    /// A tank bunker's entry stage (S-0606 Terms, table S1); *clearing*
    /// when it takes in no vehicle.
    #[serde(default)]
    pub bunker_stage: bunker::Stage,
    /// A tank bunker's side of the bunker link: its bunkered vehicle
    /// (S-0606 Terms, claim 7).
    #[serde(default)]
    pub bunker_occupant: Option<ObjectId>,
    /// A vehicle's side of the bunker link: the tank bunker it is bunkered
    /// in (S-0606 Terms, claim 7).
    #[serde(default)]
    pub bunkered_in: Option<ObjectId>,
}
