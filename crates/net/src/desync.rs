//! Desync evidence and reports (M9 N4), with no I/O.
//!
//! Every peer keeps, in memory and bounded, what it would need to show how
//! its game went apart from another's: a snapshot of its game (the game's
//! own save bytes, which load back to the same state hash) at the latest
//! checkpoint every peer agreed on, the snapshots of every checkpoint since,
//! the merged orders of every tick since that snapshot and its own state
//! hash after each of those ticks ([`Evidence`]).
//!
//! On a mismatch ([`crate::Event::Desync`]) the peer makes a [`Bundle`]: the
//! build, the start values, the snapshot before the mismatch, its own
//! snapshot at the mismatched checkpoint, the ticks from the first snapshot
//! to a few ticks past the mismatch with their orders and its hashes. The
//! caller writes it out (the client: a file natively, a download in the
//! browser) and tells the other peers its digest
//! ([`crate::Session::set_report`]). The desync tool (`glasnost-headless
//! desync`) loads two bundles, replays both and prints the first differing
//! tick and a per-object diff.
//!
//! **Snapshots.** One per checkpoint (`hash_every` ticks), taken right after
//! the tick that ends it: the snapshot's state hash is then the checkpoint
//! hash every peer compares, so the snapshot of the last agreed checkpoint
//! is known to be the same on every peer, and the snapshot of a mismatched
//! checkpoint is the peer's own state at that checkpoint. A snapshot of
//! the start (no tick run) covers a mismatch at the first checkpoint. A
//! mismatch is found only once the other peer's hash has arrived, which is
//! at most about two largest delays later (the frame window), so the peer
//! keeps every snapshot from the newest agreed one on, at most
//! [`Evidence::max_snapshots`] of them.
//!
//! **A test-only perturbation** ([`Perturb`]) changes one object's health
//! or one house's credits before a tick, from outside the game's rules, so
//! that tests can force a desync. A bundle records the perturbation it
//! applied, so the tool replays it too. The client honours it only in a
//! debug build, and a perturbed client names another build, so a real room
//! refuses it (docs/development.md, *Desync reports (N4)*).
//!
//! The report format, the snapshot keeping and the diff are our own design
//! (`// nocite: our own netcode`).

use std::collections::VecDeque;

use glasnost_ai::game::Game;
use glasnost_sim::orders::Order;
use glasnost_sim::world::{ObjectId, World};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::session::Tick;
use crate::sim::hash_from_hex;
use crate::start::StartInfo;
use crate::wire::Hash;

/// The first four bytes of a bundle file.
// nocite: our own netcode
pub const BUNDLE_MAGIC: [u8; 4] = *b"GDSY";

/// The bundle format's version. Bump it whenever the layout changes.
// nocite: our own netcode
pub const BUNDLE_VERSION: u8 = 1;

/// The bundle's file name extension.
// nocite: our own netcode
pub const BUNDLE_EXTENSION: &str = "gdsync";

/// How many ticks past the mismatch a bundle holds at least. A mismatch is
/// found only after both hashes are in, so a peer has usually run these
/// ticks already when it finds one.
// nocite: our own netcode
pub const PAST_TICKS: u32 = 5;

/// A test-only change to the game from outside its rules, made before the
/// tick `tick` runs (after tick `tick` − 1 and its hash), so the first state
/// hash that differs is the one after tick `tick`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Perturb {
    /// An object's health changes by `by`.
    Health {
        /// The tick it is made before.
        tick: u32,
        /// The object.
        object: u32,
        /// The change.
        by: i32,
    },
    /// A house's credits change by `by`.
    Credits {
        /// The tick it is made before.
        tick: u32,
        /// The house, by house index.
        house: u8,
        /// The change.
        by: i32,
    },
}

impl Perturb {
    /// The tick it is made before.
    pub fn tick(&self) -> u32 {
        match *self {
            Perturb::Health { tick, .. } | Perturb::Credits { tick, .. } => tick,
        }
    }

    /// Makes the change; returns whether its object or house exists.
    pub fn apply(&self, w: &mut World) -> bool {
        match *self {
            Perturb::Health { object, by, .. } => match w.objects.get_mut(&ObjectId(object)) {
                Some(o) => {
                    o.health = o.health.saturating_add(by);
                    true
                }
                None => false,
            },
            Perturb::Credits { house, by, .. } => match w.houses.get_mut(usize::from(house)) {
                Some(h) => {
                    h.credits = h.credits.saturating_add(by);
                    true
                }
                None => false,
            },
        }
    }

    /// Reads `health:TICK:OBJECT[:BY]` or `credits:TICK:HOUSE[:BY]` (the
    /// change `BY` is 1 when left out).
    pub fn parse(text: &str) -> Result<Self, String> {
        let parts: Vec<&str> = text.trim().split(':').collect();
        let num = |i: usize, what: &str| -> Result<i64, String> {
            parts
                .get(i)
                .ok_or_else(|| format!("{what} missing"))?
                .trim()
                .parse::<i64>()
                .map_err(|e| format!("{what}: {e}"))
        };
        let by = if parts.len() > 3 {
            i32::try_from(num(3, "the change")?).map_err(|e| e.to_string())?
        } else {
            1
        };
        let tick = u32::try_from(num(1, "the tick")?).map_err(|e| e.to_string())?;
        if parts.len() > 4 {
            return Err(format!("{text:?} has too many parts"));
        }
        match parts.first().map(|p| p.trim()) {
            Some("health") => Ok(Perturb::Health {
                tick,
                object: u32::try_from(num(2, "the object")?).map_err(|e| e.to_string())?,
                by,
            }),
            Some("credits") => Ok(Perturb::Credits {
                tick,
                house: u8::try_from(num(2, "the house")?).map_err(|e| e.to_string())?,
                by,
            }),
            _ => Err(format!(
                "{text:?} is neither health:TICK:OBJECT[:BY] nor credits:TICK:HOUSE[:BY]"
            )),
        }
    }
}

impl std::fmt::Display for Perturb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Perturb::Health { tick, object, by } => {
                write!(f, "object {object}'s health {by:+} before tick {tick}")
            }
            Perturb::Credits { tick, house, by } => {
                write!(f, "house {house}'s credits {by:+} before tick {tick}")
            }
        }
    }
}

/// A peer's game as it was after `ran` ticks of the session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    /// Ticks run when it was taken (0: before the first).
    pub ran: u32,
    /// The game's state hash then.
    pub hash: Hash,
    /// The game's save bytes ([`Game::save`]).
    pub bytes: Vec<u8>,
}

/// One order a tick issued, as the session merged it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Issued {
    /// The slot whose frame held it.
    pub slot: u8,
    /// The slot's house, by house index.
    pub house: u8,
    /// The order.
    pub order: Order,
    /// Whether the game took it.
    pub taken: bool,
}

/// One tick a peer ran.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TickRecord {
    /// The tick (0 for the session's first).
    pub tick: u32,
    /// Its orders, in the order issued.
    pub orders: Vec<Issued>,
    /// The peer's own state hash after it.
    pub hash: Hash,
}

/// A checkpoint whose hashes differ.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mismatch {
    /// The last tick the hashes cover.
    pub tick: u32,
    /// The other peer's slot.
    pub slot: u8,
    /// Our hash.
    pub mine: Hash,
    /// Its hash.
    pub theirs: Hash,
}

/// A desync report: what one peer holds of the ticks around a mismatch.
/// Nothing in it comes from the install but the game's own save bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bundle {
    /// The writer's build.
    pub build: String,
    /// The session's start values.
    pub start: StartInfo,
    /// The writer's slot.
    pub writer: u8,
    /// The mismatch it reports (none for a report made on demand).
    pub mismatch: Option<Mismatch>,
    /// The test-only perturbation the writer applied, if any.
    pub perturb: Option<Perturb>,
    /// The snapshot the ticks start from: the latest one before the
    /// mismatch that every peer agreed on.
    pub base: Snapshot,
    /// The writer's own snapshot at the mismatched checkpoint.
    pub after: Option<Snapshot>,
    /// Every tick from `base.ran` on, in order.
    pub ticks: Vec<TickRecord>,
}

/// Why bytes are not a bundle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BundleError {
    /// Shorter than the header.
    Short,
    /// Not a bundle.
    Magic,
    /// A bundle of another version.
    Version(u8),
    /// The body cannot be read.
    Body(String),
}

impl std::fmt::Display for BundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BundleError::Short => write!(f, "shorter than a desync report's header"),
            BundleError::Magic => write!(f, "not a Glasnost desync report"),
            BundleError::Version(v) => write!(
                f,
                "a desync report of format version {v}; this build reads {BUNDLE_VERSION}"
            ),
            BundleError::Body(e) => write!(f, "an unreadable desync report: {e}"),
        }
    }
}

impl std::error::Error for BundleError {}

impl Bundle {
    /// The bundle's bytes: the magic, the version, then the body in
    /// postcard.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&BUNDLE_MAGIC);
        out.push(BUNDLE_VERSION);
        postcard::to_extend(self, out).expect("a bundle serializes")
    }

    /// Reads a bundle. Never panics on hostile bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, BundleError> {
        let head = BUNDLE_MAGIC.len() + 1;
        if bytes.len() < head {
            return Err(BundleError::Short);
        }
        if bytes[..BUNDLE_MAGIC.len()] != BUNDLE_MAGIC {
            return Err(BundleError::Magic);
        }
        if bytes[BUNDLE_MAGIC.len()] != BUNDLE_VERSION {
            return Err(BundleError::Version(bytes[BUNDLE_MAGIC.len()]));
        }
        let (b, rest): (Bundle, &[u8]) = postcard::take_from_bytes(&bytes[head..])
            .map_err(|e| BundleError::Body(e.to_string()))?;
        if !rest.is_empty() {
            return Err(BundleError::Body(format!("{} trailing bytes", rest.len())));
        }
        Ok(b)
    }

    /// The SHA-256 of bytes (of a bundle's [`Bundle::to_bytes`]).
    pub fn digest(bytes: &[u8]) -> Hash {
        Sha256::digest(bytes).into()
    }

    /// The first tick after the ticks it holds.
    pub fn end(&self) -> u32 {
        self.ticks
            .last()
            .map_or(self.base.ran, |t| t.tick.saturating_add(1))
    }

    /// The tick record of `tick`, if it holds it.
    pub fn tick(&self, tick: u32) -> Option<&TickRecord> {
        let first = self.ticks.first()?.tick;
        let i = tick.checked_sub(first)? as usize;
        self.ticks.get(i).filter(|t| t.tick == tick)
    }

    /// A file name for the bundle: the mismatch's tick (or the last tick
    /// held) and the writer's slot, counted from 1 as the room shows rows.
    pub fn file_name(&self, prefix: &str) -> String {
        let tick = self
            .mismatch
            .map_or(self.end().saturating_sub(1), |m| m.tick);
        format!(
            "{prefix}tick{tick}-slot{}.{BUNDLE_EXTENSION}",
            u32::from(self.writer) + 1
        )
    }
}

/// The state hash of a game, as bytes.
fn hash_of(g: &Game) -> Hash {
    hash_from_hex(&g.state_hash()).expect("the game's state hash is 64 hex digits")
}

fn snapshot(ran: u32, hash: Hash, g: &Game) -> Snapshot {
    Snapshot {
        ran,
        hash,
        bytes: g.save(),
    }
}

/// One peer's evidence: snapshots and ticks back to the latest checkpoint
/// every peer agreed on, bounded.
#[derive(Clone, Debug)]
pub struct Evidence {
    build: String,
    start: StartInfo,
    me: u8,
    every: u32,
    max_snapshots: usize,
    snaps: VecDeque<Snapshot>,
    ticks: VecDeque<TickRecord>,
    mismatch: Option<Mismatch>,
    perturb: Option<Perturb>,
    applied: Option<Perturb>,
    done: bool,
}

impl Evidence {
    /// The evidence of the peer of slot `me` of a session from `start`, with
    /// the game `g` as it starts (no tick run): the first snapshot.
    pub fn new(build: &str, start: &StartInfo, me: u8, g: &Game) -> Self {
        let p = start.params.sane();
        let every = p.hash_every;
        // Snapshots from the newest agreed one on: a mismatch is found up to
        // about two largest delays after its checkpoint (the frame window),
        // so this many checkpoints, with the agreed one and two to spare.
        // nocite: our own netcode
        let max_snapshots = 3 + (2 * u32::from(p.max_delay)).div_ceil(every) as usize;
        let mut snaps = VecDeque::new();
        snaps.push_back(snapshot(0, hash_of(g), g));
        Self {
            build: build.to_owned(),
            start: start.clone(),
            me,
            every,
            max_snapshots,
            snaps,
            ticks: VecDeque::new(),
            mismatch: None,
            perturb: None,
            applied: None,
            done: false,
        }
    }

    /// Sets the test-only perturbation this peer makes ([`Evidence::before_tick`]).
    pub fn with_perturb(mut self, p: Option<Perturb>) -> Self {
        self.perturb = p;
        self
    }

    /// The most snapshots it keeps.
    pub fn max_snapshots(&self) -> usize {
        self.max_snapshots
    }

    /// The snapshots it holds, oldest first (tests, the docs' size bound).
    pub fn snapshots(&self) -> impl Iterator<Item = &Snapshot> {
        self.snaps.iter()
    }

    /// The ticks it holds.
    pub fn ticks_held(&self) -> usize {
        self.ticks.len()
    }

    /// The bytes it holds (snapshots and ticks, roughly).
    pub fn bytes_held(&self) -> usize {
        let snaps: usize = self.snaps.iter().map(|s| s.bytes.len() + 40).sum(); // nocite: our own netcode (a snapshot's tick and hash)
        let ticks: usize = self.ticks.iter().map(|t| 40 + 16 * t.orders.len()).sum(); // nocite: our own netcode (a rough size of a tick record)
        snaps + ticks
    }

    /// Whether a report is made or no longer can be.
    pub fn done(&self) -> bool {
        self.done
    }

    /// The perturbation it applied, if any.
    pub fn applied(&self) -> Option<Perturb> {
        self.applied
    }

    /// Call before every try to run the next tick (`ran` ticks run): makes
    /// the test-only perturbation once, before its tick.
    pub fn before_tick(&mut self, ran: u32, g: &mut Game) {
        if let Some(p) = self.perturb
            && self.applied.is_none()
            && ran == p.tick()
        {
            self.perturb = None;
            if p.apply(&mut g.world) {
                self.applied = Some(p);
            }
        }
    }

    /// Call after every tick the session ran, with the game after it and
    /// the session's [`crate::Session::agreed`].
    pub fn after_tick(&mut self, t: &Tick, g: &Game, agreed: u32) {
        if self.done {
            return;
        }
        let hash = t.hash.unwrap_or_else(|| hash_of(g));
        self.ticks.push_back(TickRecord {
            tick: t.tick,
            orders: t
                .orders
                .iter()
                .map(|a| Issued {
                    slot: a.slot,
                    house: a.house.0,
                    order: a.order.clone(),
                    taken: a.taken,
                })
                .collect(),
            hash,
        });
        let ran = t.tick + 1;
        if t.hash.is_some() {
            self.snaps.push_back(snapshot(ran, hash, g));
        }
        if self.mismatch.is_none() {
            self.prune(agreed);
        }
    }

    /// Forgets what is older than the newest agreed snapshot, and the
    /// oldest snapshots beyond the bound.
    fn prune(&mut self, agreed: u32) {
        let agreed_ran = agreed.saturating_mul(self.every);
        while self.snaps.len() >= 2 && self.snaps[1].ran <= agreed_ran {
            self.snaps.pop_front();
        }
        while self.snaps.len() > self.max_snapshots {
            self.snaps.pop_front();
        }
        let base = self.snaps.front().map_or(0, |s| s.ran);
        while self.ticks.front().is_some_and(|t| t.tick < base) {
            self.ticks.pop_front();
        }
    }

    /// Notes a desync the session reported ([`crate::Event::Desync`]). The
    /// earliest mismatch counts.
    pub fn desync(&mut self, tick: u32, slot: u8, mine: Hash, theirs: Hash) {
        if self.done {
            return;
        }
        if self.mismatch.is_none_or(|m| tick < m.tick) {
            self.mismatch = Some(Mismatch {
                tick,
                slot,
                mine,
                theirs,
            });
        }
    }

    /// The mismatch noted, if any.
    pub fn mismatch(&self) -> Option<Mismatch> {
        self.mismatch
    }

    /// The report, once a mismatch is noted and `ran` (the ticks run)
    /// reaches [`PAST_TICKS`] past it; afterwards the evidence keeps
    /// nothing more.
    pub fn take(&mut self, ran: u32) -> Option<Bundle> {
        let m = self.mismatch?;
        if self.done || ran < m.tick.saturating_add(1 + PAST_TICKS) {
            return None;
        }
        let at = m.tick + 1;
        let base_ran = at.saturating_sub(self.every);
        let base = self
            .snaps
            .iter()
            .rev()
            .find(|s| s.ran <= base_ran)
            .or(self.snaps.front())?
            .clone();
        let after = self.snaps.iter().find(|s| s.ran == at).cloned();
        let b = self.bundle(Some(m), base, after);
        self.done = true;
        self.snaps.clear();
        self.ticks.clear();
        Some(b)
    }

    /// A report of what it holds now, with no mismatch (tests; a report on
    /// demand): from the oldest snapshot it holds.
    pub fn bundle_now(&self) -> Option<Bundle> {
        let base = self.snaps.front()?.clone();
        Some(self.bundle(self.mismatch, base, None))
    }

    fn bundle(&self, m: Option<Mismatch>, base: Snapshot, after: Option<Snapshot>) -> Bundle {
        Bundle {
            build: self.build.clone(),
            start: self.start.clone(),
            writer: self.me,
            mismatch: m,
            perturb: self.applied,
            ticks: self
                .ticks
                .iter()
                .filter(|t| t.tick >= base.ran)
                .cloned()
                .collect(),
            base,
            after,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perturbations_parse() {
        assert_eq!(
            Perturb::parse("health:100:17"),
            Ok(Perturb::Health {
                tick: 100,
                object: 17,
                by: 1
            })
        );
        assert_eq!(
            Perturb::parse("credits:40:1:-5"),
            Ok(Perturb::Credits {
                tick: 40,
                house: 1,
                by: -5
            })
        );
        assert!(Perturb::parse("speed:1:2").is_err());
        assert!(Perturb::parse("health:x:2").is_err());
        assert!(Perturb::parse("health:1").is_err());
        assert!(Perturb::parse("credits:1:2:3:4").is_err());
    }

    #[test]
    fn hostile_bundle_bytes_never_panic() {
        assert_eq!(Bundle::from_bytes(b"GD"), Err(BundleError::Short));
        assert_eq!(Bundle::from_bytes(b"GNxx\x01"), Err(BundleError::Magic));
        assert_eq!(
            Bundle::from_bytes(b"GDSY\x09"),
            Err(BundleError::Version(9))
        );
        assert!(matches!(
            Bundle::from_bytes(b"GDSY\x01\xff\xff"),
            Err(BundleError::Body(_))
        ));
        let mut x: u64 = 7;
        for n in 0..2_000usize {
            let mut b = vec![b'G', b'D', b'S', b'Y', BUNDLE_VERSION];
            for _ in 0..(n % 97) {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                b.push(x as u8);
            }
            let _ = Bundle::from_bytes(&b);
        }
    }
}
