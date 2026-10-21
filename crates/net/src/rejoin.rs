//! The game a rejoining peer is sent (M9 N5), with no I/O.
//!
//! When a person's peer drops and comes back, the relay asks one live peer
//! to serve it ([`crate::wire::Body::Serve`]). That peer sends a
//! [`Transfer`]: the game's own save bytes after the ticks it has run
//! ([`Game::save`], which loads back to the same state hash), that state
//! hash, and its session's view of every person's slot then
//! ([`Session::catch_up`]: the frames that count and those it holds but has
//! not run). The bytes go in parts of at most [`PART_BYTES`] through the
//! relay ([`crate::lobby::ToRelay::Snapshot`]); the rejoining peer puts
//! them together ([`Assembler`]), loads the game, checks its hash and
//! starts its session with [`Session::rejoin`]. Every lockstep message a
//! live peer sends from the moment the relay asked reaches the rejoining
//! peer too, so with what the serving peer held then, it has every frame
//! it needs to run the ticks since.
//!
//! The format is our own: the magic `GRJN`, [`TRANSFER_VERSION`], then the
//! [`Transfer`] in postcard (`// nocite: our own netcode`).

use glasnost_ai::game::Game;
use serde::{Deserialize, Serialize};

use crate::session::{Catchup, Session};
use crate::sim::{hash_from_hex, hex};
use crate::wire::Hash;

/// The first four bytes of a transfer.
// nocite: our own netcode
pub const TRANSFER_MAGIC: [u8; 4] = *b"GRJN";

/// The transfer format's version. Bump it whenever its layout changes.
// nocite: our own netcode
pub const TRANSFER_VERSION: u8 = 1;

/// The largest part, in bytes: a part's lobby message stays under the
/// relay's message limit (64 KiB) with room for its header.
// nocite: our own netcode
pub const PART_BYTES: usize = 60_000;

/// The largest transfer a rejoining peer takes, in bytes.
// nocite: our own netcode (far above a large map's save)
pub const MAX_TRANSFER: usize = 64 * 1024 * 1024;

/// What a rejoining peer is sent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transfer {
    /// The sender's session after the ticks it has run.
    pub catchup: Catchup,
    /// The game's state hash after those ticks.
    pub hash: Hash,
    /// The game's save bytes after those ticks.
    pub save: Vec<u8>,
}

/// Why a transfer cannot be used.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RejoinError {
    /// Not a transfer, or one of another version.
    Format(String),
    /// A part out of order, of another count, or too large.
    Part(String),
    /// The save does not load.
    Load(String),
    /// The loaded game's state hash is not the sender's.
    Hash {
        /// The sender's.
        sent: String,
        /// The loaded game's.
        loaded: String,
    },
}

impl std::fmt::Display for RejoinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RejoinError::Format(e) => write!(f, "not a rejoin transfer: {e}"),
            RejoinError::Part(e) => write!(f, "a bad part of the game: {e}"),
            RejoinError::Load(e) => write!(f, "the game sent does not load: {e}"),
            RejoinError::Hash { sent, loaded } => write!(
                f,
                "the game sent loads to state hash {loaded}, the sender's was {sent}"
            ),
        }
    }
}

impl std::error::Error for RejoinError {}

impl Transfer {
    /// The transfer of a serving peer: its session's catch-up and its game
    /// after the same ticks. `None` before the session runs.
    pub fn of(session: &Session, game: &Game) -> Option<Self> {
        let catchup = session.catch_up()?;
        let hash = hash_from_hex(&game.state_hash())?;
        Some(Self {
            catchup,
            hash,
            save: game.save(),
        })
    }

    /// The transfer's bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.save.len() + 1024); // nocite: our own netcode (a first guess at the session's part)
        out.extend_from_slice(&TRANSFER_MAGIC);
        out.push(TRANSFER_VERSION);
        postcard::to_extend(self, out).expect("a transfer serializes")
    }

    /// Reads a transfer. Never panics on hostile bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, RejoinError> {
        let head = TRANSFER_MAGIC.len() + 1;
        if bytes.len() < head || bytes[..TRANSFER_MAGIC.len()] != TRANSFER_MAGIC {
            return Err(RejoinError::Format("no transfer header".into()));
        }
        let v = bytes[TRANSFER_MAGIC.len()];
        if v != TRANSFER_VERSION {
            return Err(RejoinError::Format(format!(
                "version {v}; this build reads {TRANSFER_VERSION}"
            )));
        }
        let (t, rest): (Transfer, &[u8]) = postcard::take_from_bytes(&bytes[head..])
            .map_err(|e| RejoinError::Format(e.to_string()))?;
        if !rest.is_empty() {
            return Err(RejoinError::Format(format!(
                "{} trailing bytes",
                rest.len()
            )));
        }
        Ok(t)
    }

    /// Loads the game under the rejoining peer's own rules and tile
    /// catalogue, those of `like` (its own copy of the game at the start),
    /// and checks its state hash against the sender's.
    pub fn load(&self, like: &Game) -> Result<Game, RejoinError> {
        let catalogue = like
            .world
            .catalogue
            .clone()
            .ok_or_else(|| RejoinError::Load("no tile catalogue".into()))?;
        let g = Game::load(&self.save, like.world.rules_arc(), catalogue)
            .map_err(|e| RejoinError::Load(format!("{e:?}")))?;
        let loaded = g.state_hash();
        if hash_from_hex(&loaded) != Some(self.hash) {
            return Err(RejoinError::Hash {
                sent: hex(&self.hash),
                loaded,
            });
        }
        Ok(g)
    }
}

/// Bytes cut into parts of at most [`PART_BYTES`] (one empty part for no
/// bytes).
pub fn parts(bytes: &[u8]) -> Vec<Vec<u8>> {
    if bytes.is_empty() {
        return vec![Vec::new()];
    }
    bytes.chunks(PART_BYTES).map(<[u8]>::to_vec).collect()
}

/// Puts a transfer's parts back together, in order, bounded by
/// [`MAX_TRANSFER`].
#[derive(Clone, Debug, Default)]
pub struct Assembler {
    parts: Option<u32>,
    next: u32,
    bytes: Vec<u8>,
}

impl Assembler {
    /// Takes part `part` of `parts`. Returns the whole transfer's bytes
    /// once the last part is in.
    pub fn add(
        &mut self,
        part: u32,
        parts: u32,
        bytes: &[u8],
    ) -> Result<Option<Vec<u8>>, RejoinError> {
        let max_parts = MAX_TRANSFER.div_ceil(PART_BYTES) as u32;
        if parts == 0 || parts > max_parts {
            return Err(RejoinError::Part(format!("{parts} parts")));
        }
        if *self.parts.get_or_insert(parts) != parts {
            return Err(RejoinError::Part("the part count changed".into()));
        }
        if part != self.next {
            return Err(RejoinError::Part(format!(
                "part {part} where {} was due",
                self.next
            )));
        }
        if bytes.len() > PART_BYTES || self.bytes.len() + bytes.len() > MAX_TRANSFER {
            return Err(RejoinError::Part("too many bytes".into()));
        }
        self.bytes.extend_from_slice(bytes);
        self.next += 1;
        if self.next == parts {
            return Ok(Some(std::mem::take(&mut self.bytes)));
        }
        Ok(None)
    }

    /// Bytes taken so far.
    pub fn received(&self) -> usize {
        self.bytes.len()
    }

    /// Parts taken so far, and how many there are (once one is in).
    pub fn progress(&self) -> (u32, Option<u32>) {
        (self.next, self.parts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parts_go_back_together_in_order_only() {
        let bytes: Vec<u8> = (0..(2 * PART_BYTES + 17)).map(|i| i as u8).collect();
        let ps = parts(&bytes);
        assert_eq!(ps.len(), 3);
        let mut a = Assembler::default();
        assert_eq!(a.add(0, 3, &ps[0]), Ok(None));
        assert!(a.add(2, 3, &ps[2]).is_err(), "out of order");
        let mut a = Assembler::default();
        assert_eq!(a.add(0, 3, &ps[0]), Ok(None));
        assert!(a.add(1, 4, &ps[1]).is_err(), "another count");
        let mut a = Assembler::default();
        for (i, p) in ps.iter().enumerate() {
            let got = a.add(i as u32, 3, p).expect("in order");
            assert_eq!(got.is_some(), i == 2);
            if let Some(b) = got {
                assert_eq!(b, bytes);
            }
        }
        assert!(Assembler::default().add(0, u32::MAX, &[]).is_err());
        assert_eq!(parts(&[]), vec![Vec::<u8>::new()]);
    }

    #[test]
    fn hostile_transfer_bytes_never_panic() {
        assert!(Transfer::from_bytes(b"").is_err());
        assert!(Transfer::from_bytes(b"GRJN").is_err());
        assert!(Transfer::from_bytes(&[b'G', b'R', b'J', b'N', 9]).is_err());
        let mut b = vec![b'G', b'R', b'J', b'N', TRANSFER_VERSION];
        for i in 0..64u8 {
            b.push(i.wrapping_mul(37));
            let _ = Transfer::from_bytes(&b);
        }
    }
}
