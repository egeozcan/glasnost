//! The teleport (S-0513 claims 33 to 41).

use glasnost_map::CellXY;
use serde::{Deserialize, Serialize};

use crate::world::{HouseId, World};

/// A warp member's state (T2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Member {
    /// The stage.
    pub stage: u8,
}

impl Member {
    /// In the leaving stage.
    pub fn leaving(&self) -> bool {
        false
    }

    /// Warp pending or leaving.
    pub fn pending_or_leaving(&self) -> bool {
        false
    }
}

/// A source pick (claim 33).
pub fn pick_source(w: &mut World, h: HouseId, i: u32, cell: CellXY) {
    let _ = (w, h, i, cell);
}

/// A warp launch (claim 34).
pub fn launch(w: &mut World, h: HouseId, source: CellXY, dest: CellXY) {
    let _ = (w, h, source, dest);
}
