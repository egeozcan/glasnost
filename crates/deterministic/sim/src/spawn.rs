//! The spawn controller and its slots (S-0506 claims 1 to 13): an object whose type names a spawned type keeps, launches, regenerates and reloads its slot objects (V3 and Dreadnought missiles, a carrier's aircraft).

use serde::{Deserialize, Serialize};

/// A spawner's spawn controller (S-0506 Terms).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpawnController {}
