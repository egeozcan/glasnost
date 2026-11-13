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
use crate::rules::{Kind, SimRules};
use crate::world::{GameSetup, HouseId, World};

/// Format version of saves and replays.
// nocite: version number of our own save format
pub const FORMAT_VERSION: u32 = 133;

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
                write!(
                    f,
                    "saved under rules {saved}, but the rules given are {given}"
                )
            }
            SaveError::Load(e) => write!(f, "{e:?}"),
        }
    }
}

impl std::error::Error for SaveError {}

/// The bytes of a world's state. Serialization of plain data cannot fail;
/// if it ever did, hashing nothing would hide every difference, so it
/// panics instead.
fn state_bytes(w: &World) -> Vec<u8> {
    postcard::to_allocvec(w).expect("the game state serializes")
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
    postcard::to_allocvec(&s).expect("the game state serializes")
}

/// Loads a game saved by [`save`] under the same rules.
pub fn load(
    bytes: &[u8],
    rules: Arc<SimRules>,
    catalogue: Arc<TileCatalogue>,
) -> Result<World, SaveError> {
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
    // nocite: our own input validation (#157): a save from another peer or a damaged file is refused, not a panic
    holds_together(&w, &rules).map_err(SaveError::Format)?;
    w.rules = Some(rules);
    w.catalogue = Some(catalogue);
    crate::regions::rebuild(&mut w);
    Ok(w)
}

/// Whether a decoded world holds together (#157): the indices that the
/// load's region rebuild and the first ticks trust without a check. Each
/// part is decoded on its own, so bytes another peer chose may give tables
/// of other lengths. Not checked (a seam): the overlay indices of the
/// grid's cells, and the ids that objects, routes and other records hold
/// of each other.
fn holds_together(w: &World, rules: &SimRules) -> Result<(), String> {
    if !w.cells.is_consistent() {
        return Err("the cell tables do not agree with the grid".into());
    }
    if !w.rng.is_consistent() || w.map_rng.as_ref().is_some_and(|r| !r.is_consistent()) {
        return Err("a random stream is broken".into());
    }
    for (&id, o) in &w.objects {
        if o.id != id {
            return Err(format!("object {} is filed as {}", o.id.0, id.0));
        }
        if o.owner.is_some_and(|h| usize::from(h.0) >= w.houses.len()) {
            return Err(format!("object {} has a house the game lacks", id.0));
        }
        let types = match o.kind {
            Kind::Infantry => rules.infantry.len(),
            Kind::Vehicle => rules.vehicles.len(),
            Kind::Aircraft => rules.aircraft.len(),
            Kind::Building => rules.buildings.len(),
            Kind::Terrain => rules.terrain.len(),
        };
        if usize::try_from(o.type_index).map_or(true, |t| t >= types) {
            return Err(format!("object {} has a type the rules do not have", id.0));
        }
    }
    for h in &w.houses {
        if !h.pending.iter().all(crate::orders::in_window) {
            return Err("a pending order carries a cell outside the window".into());
        }
    }
    Ok(())
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
    /// Orders as (tick issued, house, order), sorted by tick and in issue
    /// order within a tick ([`Replay::record`] keeps that order).
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
        postcard::to_allocvec(self).expect("a replay serializes")
    }

    /// Reads a replay.
    pub fn from_bytes(b: &[u8]) -> Result<Self, SaveError> {
        let r: Replay = postcard::from_bytes(b).map_err(|e| SaveError::Format(e.to_string()))?;
        if r.version != FORMAT_VERSION {
            return Err(SaveError::Format(format!("format version {}", r.version)));
        }
        Ok(r)
    }

    /// Records an order issued in tick `tick`, after every order recorded
    /// for the same or an earlier tick.
    pub fn record(&mut self, tick: i32, house: HouseId, order: Order) {
        let at = self.orders.partition_point(|o| o.0 <= tick);
        self.orders.insert(at, (tick, house, order));
    }

    /// Issues the orders recorded for the world's current tick.
    pub fn issue_due(&self, w: &mut World) {
        let from = self.orders.partition_point(|o| o.0 < w.tick);
        for (t, h, o) in &self.orders[from..] {
            if *t != w.tick {
                break;
            }
            orders::issue(w, *h, o.clone());
        }
    }

    /// Runs the replay from a new game, calling `each` with the world after
    /// every tick.
    pub fn run(
        &self,
        rules: Arc<SimRules>,
        map: &Map,
        catalogue: Arc<TileCatalogue>,
        each: impl FnMut(&World),
    ) -> Result<World, SaveError> {
        if self.rules_hash != rules.hash {
            return Err(SaveError::RulesMismatch {
                saved: self.rules_hash.clone(),
                given: rules.hash.clone(),
            });
        }
        let mut w = load::new_game(rules, map, catalogue, &self.setup).map_err(SaveError::Load)?;
        self.run_on(&mut w, self.ticks, each);
        Ok(w)
    }

    /// Continues the replay on a world (a new game or a loaded save) up to
    /// tick `until` (counted by [`crate::tick::ticks_run`]), calling `each`
    /// after every tick. No tick runs after the final one (S-0414 claim 8).
    pub fn run_on(&self, w: &mut World, until: i32, mut each: impl FnMut(&World)) {
        // spec: S-0201 claim 1 and S-0414 claim 8 (the final tick leaves the tick number at itself; no tick runs after it)
        while crate::tick::ticks_run(w) < until && !w.econ.end.finished {
            self.issue_due(w);
            crate::tick::step(w);
            each(w);
        }
    }
}
