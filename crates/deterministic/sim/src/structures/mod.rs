//! Structures and special infantry abilities of M6c: garrisons (S-0508),
//! engineers and capture (S-0509) and spies and infiltration (S-0510).
//!
//! [`entry`] is the one place where an infantry unit's arrival at a building
//! is turned into an entry (garrison, capture, infiltration or refusal);
//! [`garrison`] holds the occupants of garrisonable buildings.

use serde::{Deserialize, Serialize};

use crate::world::ObjectId;

pub mod entry;
pub mod garrison;
pub mod ownership;

/// The structure-topic state of an object (S-0508 Terms): a garrisonable
/// building's occupants and shared firing index, an occupant's building.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructState {
    /// A building's occupants in order: the first is the one that has been
    /// inside longest (S-0508 Terms). An identifier that names no object is
    /// an occupant killed inside by the defeat of its own house (claim 19):
    /// it keeps its place (Q-0970).
    pub occupants: Vec<ObjectId>,
    /// The shared occupant and muzzle index (S-0313 claim 6, S-0508 claim
    /// 18), counted from 0.
    pub index: u32,
    /// For an occupant: the building it is inside.
    pub inside: Option<ObjectId>,
}
