//! Saved games, the state hash and replays.
//!
//! A save holds the whole game state ([`World`]) in a fixed binary form
//! (postcard over serde, ordered containers only) with the
//! `sim_rules_hash` of the rules it ran under. The rules and the theater's
//! tile catalogue are given again on load; derived data (regions) is
//! rebuilt. The state hash is SHA-256 of the same bytes, so a state and its
//! save/load round trip hash alike.

use std::sync::Arc;

use glasnost_map::Map;
use glasnost_map::terrain::TileCatalogue;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::load::{self, LoadError};
use crate::orders::{self, Order};
use crate::rules::SimRules;
use crate::world::{GameSetup, HouseId, World};

/// Format version of saves and replays.
// nocite: version number of our own save format
pub const FORMAT_VERSION: u32 = 1;

/// A saved game.
#[derive(Serialize, Deserialize)]
struct Save {
    version: u32,
    rules_hash: String,
    world: World,
}

/// Why a save or replay could not be read.
#[derive(Debug, PartialEq, Eq)]
pub enum SaveError {
    /// Not a save of this format.
    Format(String),
    /// Saved under other rules.
    RulesMismatch {
        /// The hash in the save.
        saved: String,
        /// The hash of the rules given.
        given: String,
    },
    /// The replay's game could not be set up.
    Load(LoadError),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::Format(e) => write!(f, "not a readable save: {e}"),
            SaveError::RulesMismatch { saved, given } => {
                write!(f, "saved under rules {saved}, but the rules given are {given}")
            }
            SaveError::Load(e) => write!(f, "{e:?}"),
        }
    }
}

impl std::error::Error for SaveError {}

/// The bytes of a world's state.
fn state_bytes(w: &World) -> Vec<u8> {
    postcard::to_allocvec(w).unwrap_or_default()
}

/// The state hash: SHA-256 of the state's bytes, hexadecimal.
pub fn state_hash(w: &World) -> String {
    hex(&Sha256::digest(state_bytes(w)))
}

// nocite: hexadecimal text encoding of a digest
fn hex(b: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(b.len() * 2);
    for &x in b {
        s.push(char::from(DIGITS[usize::from(x >> 4)]));
        s.push(char::from(DIGITS[usize::from(x & 15)]));
    }
    s
}

/// Saves a game.
pub fn save(w: &World) -> Vec<u8> {
    let s = Save {
        version: FORMAT_VERSION,
        rules_hash: w.rules().hash.clone(),
        world: w.clone(),
    };
    postcard::to_allocvec(&s).unwrap_or_default()
}

/// Loads a game saved by [`save`] under the same rules.
pub fn load(bytes: &[u8], rules: Arc<SimRules>, catalogue: Arc<TileCatalogue>) -> Result<World, SaveError> {
    let s: Save = postcard::from_bytes(bytes).map_err(|e| SaveError::Format(e.to_string()))?;
    if s.version != FORMAT_VERSION {
        return Err(SaveError::Format(format!("format version {}", s.version)));
    }
    if s.rules_hash != rules.hash {
        return Err(SaveError::RulesMismatch {
            saved: s.rules_hash,
            given: rules.hash.clone(),
        });
    }
    let mut w = s.world;
    w.rules = Some(rules);
    w.catalogue = Some(catalogue);
    crate::regions::rebuild(&mut w);
    Ok(w)
}

/// A replay: the rules hash, the game setup and every order with the tick
/// in which it was issued. The map is named, not stored.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Replay {
    /// Format version.
    pub version: u32,
    /// `sim_rules_hash` of the rules.
    pub rules_hash: String,
    /// The map's name, for the reader.
    pub map: String,
    /// The setup (seed included).
    pub setup: GameSetup,
    /// Orders as (tick issued, house, order), in issue order.
    pub orders: Vec<(i32, HouseId, Order)>,
    /// Number of ticks the replay runs.
    pub ticks: i32,
}

impl Replay {
    /// A new, empty replay.
    pub fn new(rules: &SimRules, map: &str, setup: GameSetup) -> Self {
        Self {
            version: FORMAT_VERSION,
            rules_hash: rules.hash.clone(),
            map: map.to_owned(),
            setup,
            orders: Vec::new(),
            ticks: 0,
        }
    }

    /// The replay's bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        postcard::to_allocvec(self).unwrap_or_default()
    }

    /// Reads a replay.
    pub fn from_bytes(b: &[u8]) -> Result<Self, SaveError> {
        let r: Replay = postcard::from_bytes(b).map_err(|e| SaveError::Format(e.to_string()))?;
        if r.version != FORMAT_VERSION {
            return Err(SaveError::Format(format!("format version {}", r.version)));
        }
        Ok(r)
    }

    /// Runs the replay, calling `each` with the world after every tick.
    pub fn run(
        &self,
        rules: Arc<SimRules>,
        map: &Map,
        catalogue: Arc<TileCatalogue>,
        mut each: impl FnMut(&World),
    ) -> Result<World, SaveError> {
        if self.rules_hash != rules.hash {
            return Err(SaveError::RulesMismatch {
                saved: self.rules_hash.clone(),
                given: rules.hash.clone(),
            });
        }
        let mut w = load::new_game(rules, map, catalogue, &self.setup).map_err(SaveError::Load)?;
        let mut next = 0;
        for _ in 0..self.ticks {
            while next < self.orders.len() && self.orders[next].0 == w.tick {
                let (_, h, o) = self.orders[next].clone();
                orders::issue(&mut w, h, o);
                next += 1;
            }
            // Orders recorded for a tick already past are skipped.
            while next < self.orders.len() && self.orders[next].0 < w.tick {
                next += 1;
            }
            crate::tick::step(&mut w);
            each(&w);
        }
        Ok(w)
    }
}
