//! An in-memory network for tests: a seeded, simulated set of links between
//! peers, with per-link latency, jitter, loss, reordering and duplicates,
//! and a way to stall or cut one peer. Time is passed in; nothing here reads
//! a clock, so a run is the same for the same seed.
//!
//! A message a peer sends goes to every other peer (as through the relay).
//! Lost messages are not resent by the network: the protocol resends
//! (`session`), as it must over an unreliable channel.

use std::collections::{BTreeMap, BTreeSet};

use crate::transport::Transport;

/// One direction of a link between two peers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Link {
    /// Base one-way latency, in milliseconds.
    pub latency_ms: u32,
    /// Extra latency drawn uniformly from 0 to this, per message.
    pub jitter_ms: u32,
    /// Messages lost, per mille.
    pub loss_permille: u32,
    /// Messages delivered twice, per mille.
    pub dup_permille: u32,
    /// Messages held back by one more latency and jitter, per mille (they
    /// arrive after later ones).
    pub reorder_permille: u32,
}

impl Link {
    /// A link with only a fixed latency.
    pub fn latency(ms: u32) -> Self {
        Self {
            latency_ms: ms,
            ..Self::default()
        }
    }
}

/// SplitMix64, a small public-domain generator: the network's own random
/// stream, never the simulation's.
#[derive(Clone, Debug)]
pub struct SplitMix(u64);

impl SplitMix {
    /// A stream from a seed.
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// The next 64 bits.
    pub fn next_u64(&mut self) -> u64 {
        // nocite: SplitMix64's published constants (our own test tooling)
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9); // nocite: SplitMix64
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb); // nocite: SplitMix64
        z ^ (z >> 31) // nocite: SplitMix64
    }

    /// A value from 0 to `n - 1` (0 when `n` is 0).
    pub fn below(&mut self, n: u64) -> u64 {
        if n == 0 { 0 } else { self.next_u64() % n }
    }

    /// True with probability `permille` / 1000.
    pub fn chance(&mut self, permille: u32) -> bool {
        self.below(1000) < u64::from(permille) // nocite: per mille
    }
}

/// Counts of what the network did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    /// Copies sent (one per receiver).
    pub sent: u64,
    /// Copies lost.
    pub lost: u64,
    /// Extra copies delivered.
    pub duplicated: u64,
    /// Copies held back to arrive out of order.
    pub reordered: u64,
    /// Copies delivered.
    pub delivered: u64,
}

/// The simulated network.
#[derive(Clone, Debug)]
pub struct SimNet {
    peers: Vec<u8>,
    default: Link,
    links: BTreeMap<(u8, u8), Link>,
    rng: SplitMix,
    /// Per receiver: messages in flight by (arrival time, sequence).
    flight: BTreeMap<u8, BTreeMap<(u64, u64), Vec<u8>>>,
    seq: u64,
    stalled: BTreeSet<u8>,
    /// Messages a stalled peer sent, released when it is no longer stalled.
    held: BTreeMap<u8, Vec<Vec<u8>>>,
    cut: BTreeSet<u8>,
    stats: Stats,
}

impl SimNet {
    /// A network between `peers` (their slots), every link `default`.
    pub fn new(seed: u64, peers: &[u8], default: Link) -> Self {
        Self {
            peers: peers.to_vec(),
            default,
            links: BTreeMap::new(),
            rng: SplitMix::new(seed),
            flight: peers.iter().map(|&p| (p, BTreeMap::new())).collect(),
            seq: 0,
            stalled: BTreeSet::new(),
            held: BTreeMap::new(),
            cut: BTreeSet::new(),
            stats: Stats::default(),
        }
    }

    /// Sets the link from `from` to `to`.
    pub fn set_link(&mut self, from: u8, to: u8, link: Link) {
        self.links.insert((from, to), link);
    }

    /// Sets every link (both directions of every pair) to `link`.
    pub fn set_all(&mut self, link: Link) {
        self.default = link;
        self.links.clear();
    }

    /// Stalls a peer's network or ends the stall: while stalled, nothing it
    /// sends leaves and nothing reaches it; at the end of the stall what it
    /// sent leaves and what waited for it arrives.
    pub fn stall(&mut self, peer: u8, stalled: bool, now_ms: u64) {
        if stalled {
            self.stalled.insert(peer);
        } else if self.stalled.remove(&peer) {
            for m in self.held.remove(&peer).unwrap_or_default() {
                self.send_from(peer, now_ms, &m);
            }
        }
    }

    /// Cuts a peer off for good: everything it sends and everything sent to
    /// it from now on is lost.
    pub fn cut(&mut self, peer: u8) {
        self.cut.insert(peer);
        if let Some(q) = self.flight.get_mut(&peer) {
            q.clear();
        }
    }

    /// What the network has done so far.
    pub fn stats(&self) -> Stats {
        self.stats
    }

    fn link(&self, from: u8, to: u8) -> Link {
        self.links.get(&(from, to)).copied().unwrap_or(self.default)
    }

    fn enqueue(&mut self, to: u8, at: u64, bytes: Vec<u8>) {
        self.seq += 1;
        let seq = self.seq;
        self.flight.entry(to).or_default().insert((at, seq), bytes);
    }

    /// Sends `bytes` from `from` to every other peer.
    pub fn send_from(&mut self, from: u8, now_ms: u64, bytes: &[u8]) {
        if self.cut.contains(&from) {
            return;
        }
        if self.stalled.contains(&from) {
            self.held.entry(from).or_default().push(bytes.to_vec());
            return;
        }
        let peers = self.peers.clone();
        for to in peers.into_iter().filter(|&p| p != from) {
            self.stats.sent += 1;
            if self.cut.contains(&to) {
                self.stats.lost += 1;
                continue;
            }
            let l = self.link(from, to);
            if self.rng.chance(l.loss_permille) {
                self.stats.lost += 1;
                continue;
            }
            let mut at =
                now_ms + u64::from(l.latency_ms) + self.rng.below(u64::from(l.jitter_ms) + 1);
            if self.rng.chance(l.reorder_permille) {
                self.stats.reordered += 1;
                at += u64::from(l.latency_ms) + u64::from(l.jitter_ms) + 1;
            }
            self.enqueue(to, at, bytes.to_vec());
            if self.rng.chance(l.dup_permille) {
                self.stats.duplicated += 1;
                let again = at + self.rng.below(u64::from(l.jitter_ms) + 2); // nocite: our own test tooling (a duplicate up to a jitter later)
                self.enqueue(to, again, bytes.to_vec());
            }
        }
    }

    /// The next message for `to` that has arrived by `now_ms`.
    pub fn recv_for(&mut self, to: u8, now_ms: u64) -> Option<Vec<u8>> {
        if self.stalled.contains(&to) || self.cut.contains(&to) {
            return None;
        }
        let q = self.flight.get_mut(&to)?;
        let (&key, _) = q.iter().next()?;
        if key.0 > now_ms {
            return None;
        }
        self.stats.delivered += 1;
        q.remove(&key)
    }

    /// The transport of one peer.
    pub fn endpoint(&mut self, peer: u8) -> Endpoint<'_> {
        Endpoint { net: self, peer }
    }
}

/// One peer's end of the simulated network.
pub struct Endpoint<'a> {
    net: &'a mut SimNet,
    peer: u8,
}

impl Transport for Endpoint<'_> {
    fn send(&mut self, now_ms: u64, bytes: &[u8]) {
        self.net.send_from(self.peer, now_ms, bytes);
    }

    fn recv(&mut self, now_ms: u64) -> Option<Vec<u8>> {
        self.net.recv_for(self.peer, now_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delivers_after_latency_to_every_other_peer() {
        let mut n = SimNet::new(1, &[0, 1, 2], Link::latency(10));
        n.send_from(0, 100, b"x");
        assert_eq!(n.recv_for(1, 109), None);
        assert_eq!(n.recv_for(1, 110).as_deref(), Some(&b"x"[..]));
        assert_eq!(n.recv_for(2, 110).as_deref(), Some(&b"x"[..]));
        assert_eq!(n.recv_for(0, 1000), None, "not to itself");
    }

    #[test]
    fn same_seed_same_network() {
        let link = Link {
            latency_ms: 20,
            jitter_ms: 30,
            loss_permille: 200,
            dup_permille: 100,
            reorder_permille: 100,
        };
        let run = |seed| {
            let mut n = SimNet::new(seed, &[0, 1], link);
            let mut got = Vec::new();
            for t in 0..200u64 {
                n.send_from(0, t, &t.to_le_bytes());
                while let Some(m) = n.recv_for(1, t) {
                    got.push((t, m));
                }
            }
            (got, n.stats())
        };
        assert_eq!(run(7), run(7));
        assert_ne!(run(7), run(8));
        let (_, s) = run(7);
        assert!(s.lost > 0 && s.duplicated > 0 && s.reordered > 0, "{s:?}");
    }

    #[test]
    fn stall_holds_both_ways_and_cut_drops() {
        let mut n = SimNet::new(1, &[0, 1], Link::latency(5));
        n.stall(1, true, 0);
        n.send_from(0, 0, b"a");
        n.send_from(1, 0, b"b");
        assert_eq!(n.recv_for(1, 100), None);
        assert_eq!(n.recv_for(0, 100), None);
        n.stall(1, false, 100);
        assert_eq!(n.recv_for(1, 100).as_deref(), Some(&b"a"[..]));
        assert_eq!(n.recv_for(0, 105).as_deref(), Some(&b"b"[..]));
        n.cut(1);
        n.send_from(0, 200, b"c");
        n.send_from(1, 200, b"d");
        assert_eq!(n.recv_for(1, 300), None);
        assert_eq!(n.recv_for(0, 300), None);
    }
}
