//! The session start (handshake): the values every peer must share before
//! the first tick, and which of them differs when they do not.
//!
//! Every peer builds its own [`StartInfo`] from its own install and the
//! lobby's setup, sends it, and compares each other peer's with its own.
//! There is no host: a peer refuses a start whose values differ from its own
//! and names the first differing [`Field`], and since the other peer
//! compares the same two values it refuses too.

use glasnost_sim::world::{GameSetup, HouseRole};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Who plays a slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotKind {
    /// A person, on the peer of the same slot number. Its orders cross the
    /// network in that peer's frames.
    Human,
    /// Our own AI (`glasnost_ai`), run by every peer inside the tick. Its
    /// orders never cross the network and it has no frames.
    Computer,
}

/// One player slot of the session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Slot {
    /// The house the slot plays, by house index.
    pub house: u8,
    /// Who plays it.
    pub kind: SlotKind,
}

/// The lockstep parameters every peer must share.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Params {
    /// The delay of the first ticks: frames 0 to `initial_delay - 1` are
    /// sent empty at the start.
    pub initial_delay: u8,
    /// The smallest delay (at least 1).
    pub min_delay: u8,
    /// The largest delay.
    pub max_delay: u8,
    /// A state-hash checkpoint every this many ticks (at least 1).
    pub hash_every: u32,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            initial_delay: 3, // nocite: our own netcode
            min_delay: 1,     // nocite: our own netcode
            max_delay: 30,    // nocite: our own netcode
            hash_every: 30,   // nocite: our own netcode
        }
    }
}

impl Params {
    /// The parameters made usable: delays of at least 1, the initial one
    /// between the smallest and the largest, checkpoints at least every
    /// tick.
    pub fn sane(self) -> Self {
        let min_delay = self.min_delay.max(1);
        let max_delay = self.max_delay.max(min_delay);
        Self {
            initial_delay: self.initial_delay.clamp(min_delay, max_delay),
            min_delay,
            max_delay,
            hash_every: self.hash_every.max(1),
        }
    }
}

/// The values a session starts from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartInfo {
    /// The engine build, as the client names it (N3 decides the form).
    pub build: String,
    /// `sim_rules_hash` of the ruleset.
    pub rules_hash: String,
    /// The map's content hash (`game::session::content_hash` in the
    /// client).
    pub map_hash: String,
    /// SHA-256 of the whole game setup (countries, colours, teams, starts,
    /// settings), hexadecimal; see [`setup_hash`].
    pub setup_hash: String,
    /// The game seed (the setup's).
    pub seed: u32,
    /// The slot table, slot 0 first.
    pub slots: Vec<Slot>,
    /// The lockstep parameters.
    pub params: Params,
}

/// A start value that differs between two peers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Field {
    /// The wire format's version.
    Protocol,
    /// The engine build.
    Build,
    /// The rules hash.
    RulesHash,
    /// The map hash.
    MapHash,
    /// The game seed.
    Seed,
    /// The slot table.
    Slots,
    /// The rest of the game setup.
    Setup,
    /// The lockstep parameters.
    Params,
    /// The sender claims a slot that is not a person's slot of the table, or
    /// is ours.
    Slot,
}

impl Field {
    /// The field's name, for messages.
    pub fn name(self) -> &'static str {
        match self {
            Field::Protocol => "protocol version",
            Field::Build => "build",
            Field::RulesHash => "rules hash",
            Field::MapHash => "map hash",
            Field::Seed => "seed",
            Field::Slots => "slot table",
            Field::Setup => "game setup",
            Field::Params => "lockstep parameters",
            Field::Slot => "slot",
        }
    }
}

impl std::fmt::Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// The SHA-256 of a setup's saved form, hexadecimal.
pub fn setup_hash(setup: &GameSetup) -> String {
    let bytes = postcard::to_allocvec(setup).expect("a setup serializes");
    // nocite: hexadecimal text of a digest (64 digits)
    Sha256::digest(&bytes)
        .iter()
        .fold(String::with_capacity(64), |mut s, b| {
            use std::fmt::Write as _;
            let _ = write!(s, "{b:02x}"); // nocite: hexadecimal text of a digest
            s
        })
}

/// The slot table of a setup: one slot per player house, in house order,
/// a computer slot for each AI house (`HouseSetup::ai`), a person's slot for
/// the others.
pub fn slots_of(setup: &GameSetup) -> Vec<Slot> {
    setup
        .houses
        .iter()
        .enumerate()
        .filter(|(_, h)| h.role == HouseRole::Player)
        .map(|(i, h)| Slot {
            house: i as u8,
            kind: if h.ai {
                SlotKind::Computer
            } else {
                SlotKind::Human
            },
        })
        .collect()
}

impl StartInfo {
    /// The start values of a game: the seed, the slot table and the setup
    /// hash come from `setup`.
    pub fn new(
        build: &str,
        rules_hash: &str,
        map_hash: &str,
        setup: &GameSetup,
        params: Params,
    ) -> Self {
        Self {
            build: build.to_owned(),
            rules_hash: rules_hash.to_owned(),
            map_hash: map_hash.to_owned(),
            setup_hash: setup_hash(setup),
            seed: setup.seed,
            slots: slots_of(setup),
            params: params.sane(),
        }
    }

    /// The first field in which `other` differs from these values, in the
    /// order of [`Field`].
    pub fn mismatch(&self, other: &StartInfo) -> Option<Field> {
        if self.build != other.build {
            Some(Field::Build)
        } else if self.rules_hash != other.rules_hash {
            Some(Field::RulesHash)
        } else if self.map_hash != other.map_hash {
            Some(Field::MapHash)
        } else if self.seed != other.seed {
            Some(Field::Seed)
        } else if self.slots != other.slots {
            Some(Field::Slots)
        } else if self.setup_hash != other.setup_hash {
            Some(Field::Setup)
        } else if self.params != other.params {
            Some(Field::Params)
        } else {
            None
        }
    }

    /// Whether `slot` is a person's slot of the table.
    pub fn is_human(&self, slot: u8) -> bool {
        self.slots
            .get(usize::from(slot))
            .is_some_and(|s| s.kind == SlotKind::Human)
    }
}
