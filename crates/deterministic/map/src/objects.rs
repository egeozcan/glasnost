//! Objects a map places (S-0009 claim 27).
//!
//! The field layout of the object lists has no spec yet (S-0009 Import
//! mapping; request Q-0010). The importer fills these records by the
//! provisional layout given in Q-0010's workaround; the values are kept as
//! read, and every field that a spec does not interpret stays raw.

use serde::{Deserialize, Serialize};

use crate::geometry::CellXY;

/// The fields every mobile or building entry shares.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Placed {
    /// The entry's key in its section (kept for object info and warnings).
    pub key: Vec<u8>,
    /// Owner house name, matched case-sensitively by the caller (claim 27).
    pub owner: Vec<u8>,
    /// Type name.
    pub type_name: Vec<u8>,
    /// Health value as written (its scale: Q-0010).
    pub health: i32,
    /// Cell.
    pub cell: CellXY,
    /// Facing value as written. For vehicles it is the map facing value of
    /// glossary claim G1 (0 … 255 covered); for other kinds its scale is
    /// open (Q-0010).
    pub facing: i32,
    /// Every value of the entry, as written, for object info.
    pub raw: Vec<Vec<u8>>,
}

/// A vehicle.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unit {
    /// Shared fields.
    pub placed: Placed,
}

/// An aircraft.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aircraft {
    /// Shared fields.
    pub placed: Placed,
}

/// An infantry unit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Infantry {
    /// Shared fields.
    pub placed: Placed,
    /// Position within the cell, as written (its meaning: Q-0010).
    pub sub_cell: i32,
}

/// A building.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Structure {
    /// Shared fields; the cell is taken as the foundation's top-left cell
    /// (Q-0010).
    pub placed: Placed,
}

/// A terrain object (tree and similar).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerrainObject {
    /// Cell.
    pub cell: CellXY,
    /// Type name.
    pub type_name: Vec<u8>,
}

/// A smudge.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Smudge {
    /// Type name.
    pub type_name: Vec<u8>,
    /// Cell.
    pub cell: CellXY,
}

/// Every object a map places, in entry order per list.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Objects {
    /// `[Units]`.
    pub units: Vec<Unit>,
    /// `[Infantry]`.
    pub infantry: Vec<Infantry>,
    /// `[Aircraft]`.
    pub aircraft: Vec<Aircraft>,
    /// `[Structures]`.
    pub structures: Vec<Structure>,
    /// `[Terrain]`.
    pub terrain: Vec<TerrainObject>,
    /// `[Smudge]`.
    pub smudges: Vec<Smudge>,
}

impl Objects {
    /// Total number of objects.
    pub fn len(&self) -> usize {
        self.units.len()
            + self.infantry.len()
            + self.aircraft.len()
            + self.structures.len()
            + self.terrain.len()
            + self.smudges.len()
    }

    /// Whether the map places no object.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
