//! Ore and gems (S-0401): resource cells, bales and value, growth and
//! spread, ore mines.

use serde::{Deserialize, Serialize};

use crate::world::World;

/// Resource timers and the growth and spread lists of every tiberium
/// (S-0401 claims 6 to 9 and 13).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resources {}

/// Row 9 of S-0203: growth and spread of resources, growth first.
pub fn part(w: &mut World) {
    let _ = w;
}
