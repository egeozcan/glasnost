//! Direct links between the peers of a network game (M9 N7): a WebRTC data
//! channel per pair of persons, beside the relay.
//!
//! The relay stays the game's path and its authority: every lockstep
//! message still goes through it. A peer copies only its own frames
//! (`glasnost_net::wire::is_frames_from`) onto each open direct link, so a
//! frame may come first the short way; the session takes from a link only
//! the frames of the slot it was opened for, and the first copy wins
//! (`glasnost_net::session`, N7). Nothing the players do: a link that
//! cannot be made, or that goes quiet, leaves that pair on the relay, which
//! carried every frame all along, and the lower slot tries again later.
//!
//! The pieces:
//!
//! - [`PairLink`]: one attempt at a link (natively `glasnost-rtc`, in the
//!   browser `glasnost-platform-web`'s RTCPeerConnection, in tests
//!   [`fake`]), opened through an [`OpenPair`].
//! - [`Mesh`]: every pair of this peer: who offers, the attempts and their
//!   timeouts, the ping that tells a live link from a dead one, the
//!   signals for the relay, the numbers.
//! - [`Shape`]: a bad network on the direct path for the soak (latency,
//!   jitter, loss, stalls), seeded from the host's own seed.
//!
//! Signalling goes through the relay as lobby messages
//! (`ToRelay::Signal`, `FromRelay::Signal`): the relay passes them only
//! between live persons of one game. The lower slot of a pair offers. A
//! pair's attempts are numbered by the offering end; a signal of another
//! attempt than the current one is dropped, and an offer of a new attempt
//! replaces the answering end's link.
//!
//! Every timeout and number here is our own netcode; no spec applies.

use std::collections::{BTreeMap, VecDeque};
use std::rc::Rc;

use glasnost_net::Session;
use glasnost_net::lobby::{Signal, ToRelay};
use glasnost_net::wire;

/// How long an attempt may take to open, in milliseconds; then it is
/// given up.
// nocite: our own netcode (ICE on a LAN takes well under a second; a STUN round leaves room)
pub const CONNECT_MS: u64 = 10_000;

/// How long an open link may be silent before it counts as dead, in
/// milliseconds: six pings missed.
// nocite: our own netcode (shorter than a stall the relay path would show as a wait)
pub const STALE_MS: u64 = 3_000;

/// How often an open link is pinged, in milliseconds.
// nocite: our own netcode (two a second: a round-trip figure and a heartbeat)
pub const PING_MS: u64 = 500;

/// How long the offering end waits after a failed attempt before the
/// next, in milliseconds; it doubles with every failure in a row, up to
/// [`RETRY_MAX_MS`].
// nocite: our own netcode
pub const RETRY_MS: u64 = 15_000;

/// The longest wait between two attempts, in milliseconds.
// nocite: our own netcode (two minutes: a pair that never connects costs a signal burst that often)
pub const RETRY_MAX_MS: u64 = 120_000;

/// The most signals one attempt sends; a stack that wants more gives the
/// attempt up (the relay's signal budget is for all of a peer's pairs).
// nocite: our own netcode (an offer and a few dozen candidates)
pub const MAX_SIGNALS: u32 = 40;

/// The magic of a direct link's own control packets.
// nocite: our own netcode ("GP": Glasnost ping)
pub const PING_MAGIC: [u8; 2] = *b"GP";

/// The version of the control packets.
// nocite: our own netcode
pub const PING_VERSION: u8 = 1;

/// A control packet's kind: a ping, answered with [`PONG`].
// nocite: our own netcode
const PING: u8 = 0;
/// A control packet's kind: the answer to a [`PING`].
// nocite: our own netcode
const PONG: u8 = 1;

/// The timeouts of a peer's direct links: [`CONNECT_MS`], [`STALE_MS`],
/// [`PING_MS`], [`RETRY_MS`] and [`RETRY_MAX_MS`] by default (tests take
/// shorter ones).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timeouts {
    /// How long an attempt may take to open.
    pub connect_ms: u64,
    /// How long an open link may be silent.
    pub stale_ms: u64,
    /// How often an open link is pinged.
    pub ping_ms: u64,
    /// The first wait after a failed attempt.
    pub retry_ms: u64,
    /// The longest wait between attempts.
    pub retry_max_ms: u64,
}

impl Default for Timeouts {
    fn default() -> Self {
        Self {
            connect_ms: CONNECT_MS,
            stale_ms: STALE_MS,
            ping_ms: PING_MS,
            retry_ms: RETRY_MS,
            retry_max_ms: RETRY_MAX_MS,
        }
    }
}

/// The weight of a new round-trip sample, in eighths.
// nocite: our own netcode (the usual one-eighth smoothing)
const RTT_NEW_EIGHTHS: u32 = 1;

/// The ICE servers a link may use: STUN or TURN URLs, none by default
/// (host candidates only: a direct link forms on a LAN, not across most
/// routers, and nothing is contacted but the other peer).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ice {
    /// The servers' URLs (`stun:host:port`, `turn:host:port`).
    pub urls: Vec<String>,
    /// The TURN user, if any.
    pub username: Option<String>,
    /// The TURN password, if any.
    pub credential: Option<String>,
}

impl Ice {
    /// The servers of a space-separated list of URLs.
    pub fn from_urls(text: &str) -> Self {
        Self {
            urls: text.split_whitespace().map(str::to_owned).collect(),
            ..Self::default()
        }
    }
}

/// Where an attempt is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PairState {
    /// The offer, the answer and ICE are under way.
    Connecting,
    /// Packets flow.
    Open,
    /// The attempt ended, and why.
    Closed(String),
}

/// One attempt at a direct link to one peer. Never blocks; packets may be
/// lost or reordered.
pub trait PairLink {
    /// Looks at the link (and moves what arrived), with the client's clock.
    fn poll(&mut self, now_ms: u64);
    /// Where the attempt is, as of the last [`PairLink::poll`].
    fn state(&self) -> PairState;
    /// Sends one packet if the link is open.
    fn send(&mut self, bytes: Vec<u8>);
    /// The next packet that arrived.
    fn recv(&mut self) -> Option<Vec<u8>>;
    /// Takes a signal the peer sent through the relay.
    fn signal(&mut self, s: Signal);
    /// The next signal for the peer.
    fn take_signal(&mut self) -> Option<Signal>;
    /// Ends the attempt.
    fn close(&mut self);
}

/// What an attempt is for.
#[derive(Clone, Debug)]
pub struct PairInfo {
    /// This peer's slot.
    pub me: u8,
    /// The other peer's slot.
    pub peer: u8,
    /// Whether this end offers (the lower slot).
    pub offerer: bool,
    /// The attempt's number.
    pub attempt: u32,
    /// The ICE servers.
    pub ice: Ice,
}

/// Opens one attempt.
pub type OpenPair = Rc<dyn Fn(&PairInfo) -> Box<dyn PairLink>>;

/// The platform's own direct links, where it has them built in: natively
/// `glasnost-rtc`. The web shell passes its own.
pub fn platform_direct() -> Option<OpenPair> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Some(Rc::new(native::open))
    }
    #[cfg(target_arch = "wasm32")]
    {
        None
    }
}

/// The native direct link: `glasnost-rtc`'s matchbox socket.
#[cfg(not(target_arch = "wasm32"))]
pub mod native {
    use glasnost_net::lobby::Signal;
    use glasnost_rtc::{Ice, Pair, State};

    use super::{PairInfo, PairLink, PairState};

    /// Opens an attempt.
    pub fn open(info: &PairInfo) -> Box<dyn PairLink> {
        let ice = Ice {
            urls: info.ice.urls.clone(),
            username: info.ice.username.clone(),
            credential: info.ice.credential.clone(),
        };
        Box::new(Native(Pair::open(&ice, info.offerer)))
    }

    struct Native(Pair);

    impl PairLink for Native {
        fn poll(&mut self, _now_ms: u64) {
            self.0.poll();
        }
        fn state(&self) -> PairState {
            match self.0.state() {
                State::Connecting => PairState::Connecting,
                State::Open => PairState::Open,
                State::Closed(why) => PairState::Closed(why),
            }
        }
        fn send(&mut self, bytes: Vec<u8>) {
            self.0.send(bytes);
        }
        fn recv(&mut self) -> Option<Vec<u8>> {
            self.0.recv()
        }
        fn signal(&mut self, s: Signal) {
            self.0.signal(s);
        }
        fn take_signal(&mut self) -> Option<Signal> {
            self.0.take_signal()
        }
        fn close(&mut self) {
            self.0.close();
        }
    }
}

/// A pair's numbers (counts and timings only).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PairStats {
    /// Attempts made (offering end) or answered.
    pub attempts: u32,
    /// Attempts that opened.
    pub opened: u32,
    /// How long the first attempt that opened took, in milliseconds.
    pub connect_ms: Option<u64>,
    /// Open links that died (went silent or closed): each one a fallback
    /// to the relay.
    pub fallbacks: u32,
    /// Attempts that never opened.
    pub failed: u32,
    /// The direct round trip, smoothed, in milliseconds.
    pub rtt_ms: Option<u32>,
    /// The lowest and highest direct round trip seen.
    pub rtt_range: Option<(u32, u32)>,
    /// Packets sent and taken on the link (frames and pings).
    pub packets: (u64, u64),
    /// Packets taken that were neither a ping nor this peer's frames.
    pub refused: u64,
    /// How long a link was open in all, in milliseconds.
    pub open_ms: u64,
}

/// One other person's peer.
struct Peer {
    /// This end offers (it has the lower slot).
    offerer: bool,
    /// The current attempt's number (0: none yet).
    attempt: u32,
    link: Option<Box<dyn PairLink>>,
    /// When the current attempt began.
    since: u64,
    /// When the current link opened.
    open_since: Option<u64>,
    /// When the link last carried anything.
    heard: u64,
    /// When the next ping is due.
    next_ping: u64,
    /// When the offering end tries again.
    retry_at: u64,
    /// Failed attempts in a row (the offering end's back-off).
    fails: u32,
    /// Signals the current attempt sent.
    signals: u32,
    stats: PairStats,
}

impl Peer {
    fn new(offerer: bool) -> Self {
        Self {
            offerer,
            attempt: 0,
            link: None,
            since: 0,
            open_since: None,
            heard: 0,
            next_ping: 0,
            retry_at: 0,
            fails: 0,
            signals: 0,
            stats: PairStats::default(),
        }
    }

    fn open(&self) -> bool {
        self.link.is_some() && self.open_since.is_some()
    }

    /// The current attempt ends: a fallback if it was open, a failure if
    /// not; the offering end waits before the next.
    fn end(&mut self, now: u64, why: &str, slot: u8, t: &Timeouts) {
        let Some(mut link) = self.link.take() else {
            return;
        };
        link.close();
        match self.open_since.take() {
            Some(t) => {
                self.stats.fallbacks += 1;
                self.stats.open_ms += now.saturating_sub(t);
                log::info!("direct link to slot {slot} lost ({why}); the relay carries its frames");
            }
            None => {
                self.stats.failed += 1;
                log::info!("direct link to slot {slot} not made ({why})");
            }
        }
        if self.offerer {
            self.fails = self.fails.saturating_add(1);
            let wait = t
                .retry_ms
                .saturating_mul(1 << (self.fails - 1).min(8)) // nocite: our own netcode (doubling, capped below)
                .min(t.retry_max_ms);
            self.retry_at = now + wait;
        }
    }
}

/// A bad network on the direct path (the soak, M9 N7): each direction of
/// every link of this peer, seeded. A packet is late by `latency +
/// jitter`, lost with a chance of `loss_permille` in a thousand (a lost
/// datagram is gone: the relay's copy counts), and during a stall nothing
/// passes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Shape {
    /// One-way latency, in milliseconds.
    pub latency_ms: u64,
    /// Jitter, drawn in `0..=jitter_ms`.
    pub jitter_ms: u64,
    /// The chance in a thousand that a packet is lost.
    pub loss_permille: u32,
    /// The shortest and longest wait between two stalls; `(0, 0)`: none.
    pub stall_every_ms: (u64, u64),
    /// The shortest and longest stall, in milliseconds.
    pub stall_ms: (u64, u64),
    /// The seed of the draws.
    pub seed: u64,
}

/// A host-side generator for [`Shape`]'s draws (splitmix64): never the
/// simulation's.
#[derive(Clone, Debug)]
struct Draw(u64);

impl Draw {
    fn next(&mut self) -> u64 {
        // nocite: splitmix64's published constants (a host-side generator, not the simulation's)
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn range(&mut self, lo: u64, hi: u64) -> u64 {
        if hi <= lo {
            return lo;
        }
        lo + self.next() % (hi - lo + 1)
    }
}

/// One direction of a shaped link.
#[derive(Debug)]
struct Lane {
    queue: VecDeque<(u64, Vec<u8>)>,
    /// The current or next stall: (start, end).
    stall: Option<(u64, u64)>,
    lost: u64,
}

impl Lane {
    fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            stall: None,
            lost: 0,
        }
    }

    /// Takes a packet at `now`: queued for later, or lost.
    fn push(&mut self, s: &Shape, d: &mut Draw, now: u64, bytes: Vec<u8>) {
        if s.stall_every_ms.1 > 0 {
            let (start, end) = *self.stall.get_or_insert_with(|| {
                let start = now + d.range(s.stall_every_ms.0, s.stall_every_ms.1);
                (start, start + d.range(s.stall_ms.0, s.stall_ms.1))
            });
            if now >= end {
                let start = end + d.range(s.stall_every_ms.0, s.stall_every_ms.1);
                self.stall = Some((start, start + d.range(s.stall_ms.0, s.stall_ms.1)));
            } else if now >= start {
                self.lost += 1;
                return;
            }
        }
        if d.range(0, 999) < u64::from(s.loss_permille) {
            self.lost += 1;
            return;
        }
        let due = now + s.latency_ms + d.range(0, s.jitter_ms);
        // Kept by due time; a later draw may pass an earlier packet.
        let at = self.queue.partition_point(|(t, _)| *t <= due);
        self.queue.insert(at, (due, bytes));
    }

    fn due(&mut self, now: u64) -> Option<Vec<u8>> {
        if self.queue.front().is_some_and(|(t, _)| *t <= now) {
            return self.queue.pop_front().map(|(_, b)| b);
        }
        None
    }
}

/// A link behind a [`Shape`] both ways.
struct Shaped {
    inner: Box<dyn PairLink>,
    shape: Shape,
    draw: Draw,
    up: Lane,
    down: Lane,
    now: u64,
}

impl PairLink for Shaped {
    fn poll(&mut self, now_ms: u64) {
        self.now = now_ms;
        self.inner.poll(now_ms);
        while let Some(b) = self.up.due(now_ms) {
            self.inner.send(b);
        }
        while let Some(b) = self.inner.recv() {
            self.down.push(&self.shape, &mut self.draw, now_ms, b);
        }
    }
    fn state(&self) -> PairState {
        self.inner.state()
    }
    fn send(&mut self, bytes: Vec<u8>) {
        self.up.push(&self.shape, &mut self.draw, self.now, bytes);
    }
    fn recv(&mut self) -> Option<Vec<u8>> {
        self.down.due(self.now)
    }
    fn signal(&mut self, s: Signal) {
        self.inner.signal(s);
    }
    fn take_signal(&mut self) -> Option<Signal> {
        self.inner.take_signal()
    }
    fn close(&mut self) {
        self.inner.close();
    }
}

/// How this peer's direct links are set up for a game.
#[derive(Clone)]
pub struct Direct {
    /// The way to open an attempt; none: relay only.
    pub open: Option<OpenPair>,
    /// The ICE servers.
    pub ice: Ice,
    /// A bad network on the direct path (the soak).
    pub shape: Option<Shape>,
    /// The timeouts.
    pub timeouts: Timeouts,
}

impl Direct {
    /// No direct links: the relay carries everything.
    pub fn off() -> Self {
        Self {
            open: None,
            ice: Ice::default(),
            shape: None,
            timeouts: Timeouts::default(),
        }
    }
}

/// A control packet: magic, version, kind, the stamp.
fn control(kind: u8, stamp: u32) -> Vec<u8> {
    let mut p = Vec::with_capacity(8); // nocite: our own netcode (the packet's size)
    p.extend_from_slice(&PING_MAGIC);
    p.push(PING_VERSION);
    p.push(kind);
    p.extend_from_slice(&stamp.to_le_bytes());
    p
}

/// A control packet's kind and stamp, if `bytes` is one.
fn read_control(bytes: &[u8]) -> Option<(u8, u32)> {
    if bytes.len() != 8 || bytes[..2] != PING_MAGIC || bytes[2] != PING_VERSION {
        return None;
    }
    let stamp = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    Some((bytes[3], stamp))
}

/// This peer's direct links to every other person of its game.
pub struct Mesh {
    me: u8,
    direct: Direct,
    peers: BTreeMap<u8, Peer>,
    signals: Vec<ToRelay>,
}

impl Mesh {
    /// The mesh of `me` in a game of the persons `persons` (every
    /// person's slot; `me` among them or not).
    pub fn new(me: u8, persons: &[u8], direct: Direct) -> Self {
        let peers = persons
            .iter()
            .filter(|&&s| s != me)
            .map(|&s| (s, Peer::new(me < s)))
            .collect();
        Self {
            me,
            direct,
            peers,
            signals: Vec::new(),
        }
    }

    /// Whether this peer makes direct links at all.
    pub fn enabled(&self) -> bool {
        self.direct.open.is_some()
    }

    /// A signal from `from` through the relay.
    pub fn signal_in(&mut self, now: u64, ls: &Session, from: u8, attempt: u32, s: Signal) {
        let Some(open) = self.direct.open.clone() else {
            return;
        };
        if !ls.direct_ok(from) {
            return;
        }
        let me = self.me;
        let Some(p) = self.peers.get_mut(&from) else {
            return;
        };
        if !p.offerer && matches!(s, Signal::Offer(_)) && attempt != p.attempt {
            // A new attempt of the offering end: it replaces ours.
            p.end(now, "the peer tried again", from, &self.direct.timeouts);
            p.attempt = attempt;
            p.since = now;
            p.signals = 0;
            p.stats.attempts += 1;
            p.link = Some(self::opened(&open, &self.direct, me, from, attempt, false));
        }
        if attempt != p.attempt {
            return;
        }
        if let Some(link) = p.link.as_mut() {
            link.signal(s);
        }
    }

    /// Looks at every link: attempts start and time out, links open and
    /// die, pings go and come. Returns the packets that came for the
    /// session, by slot.
    pub fn update(&mut self, now: u64, ls: &Session) -> Vec<(u8, Vec<u8>)> {
        let mut out = Vec::new();
        let Some(open) = self.direct.open.clone() else {
            return out;
        };
        let me = self.me;
        let t = self.direct.timeouts;
        for (&slot, p) in &mut self.peers {
            if !ls.direct_ok(slot) {
                // A slot that left, or this peer not (yet) in the game: no
                // link; a fresh start once it is back.
                if p.link.is_some() {
                    p.end(now, "the slot is not playing", slot, &t);
                }
                p.fails = 0;
                p.retry_at = 0;
                continue;
            }
            if p.link.is_none() && p.offerer && now >= p.retry_at {
                p.attempt += 1;
                p.since = now;
                p.signals = 0;
                p.stats.attempts += 1;
                p.link = Some(self::opened(&open, &self.direct, me, slot, p.attempt, true));
            }
            let Some(link) = p.link.as_mut() else {
                continue;
            };
            link.poll(now);
            while let Some(signal) = link.take_signal() {
                p.signals += 1;
                if p.signals > MAX_SIGNALS {
                    break;
                }
                self.signals.push(ToRelay::Signal {
                    to: slot,
                    attempt: p.attempt,
                    signal,
                });
            }
            if p.signals > MAX_SIGNALS {
                p.end(now, "too many signals", slot, &t);
                continue;
            }
            match link.state() {
                PairState::Connecting => {
                    if now.saturating_sub(p.since) > t.connect_ms {
                        p.end(now, "it took too long", slot, &t);
                    }
                    continue;
                }
                PairState::Closed(why) => {
                    p.end(now, &why, slot, &t);
                    continue;
                }
                PairState::Open => {}
            }
            if p.open_since.is_none() {
                p.open_since = Some(now);
                p.heard = now;
                p.next_ping = now;
                p.fails = 0;
                p.stats.opened += 1;
                let took = now.saturating_sub(p.since);
                p.stats.connect_ms.get_or_insert(took);
                log::info!("direct link to slot {slot} open after {took} ms");
            }
            while let Some(bytes) = link.recv() {
                p.stats.packets.1 += 1;
                p.heard = now;
                match read_control(&bytes) {
                    Some((PING, stamp)) => {
                        link.send(control(PONG, stamp));
                        p.stats.packets.0 += 1;
                    }
                    Some((PONG, stamp)) => {
                        let rtt = (now as u32).wrapping_sub(stamp);
                        if u64::from(rtt) <= t.stale_ms {
                            let s = &mut p.stats;
                            s.rtt_ms = Some(s.rtt_ms.map_or(rtt, |old| {
                                (old * (8 - RTT_NEW_EIGHTHS) + rtt * RTT_NEW_EIGHTHS) / 8 // nocite: our own netcode (eighths)
                            }));
                            s.rtt_range = Some(
                                s.rtt_range
                                    .map_or((rtt, rtt), |(lo, hi)| (lo.min(rtt), hi.max(rtt))),
                            );
                        }
                    }
                    Some(_) => p.stats.refused += 1,
                    None if wire::is_frames_from(&bytes, slot) => out.push((slot, bytes)),
                    None => p.stats.refused += 1,
                }
            }
            if now.saturating_sub(p.heard) > t.stale_ms {
                p.end(now, "it went silent", slot, &t);
                continue;
            }
            if now >= p.next_ping {
                link.send(control(PING, now as u32));
                p.stats.packets.0 += 1;
                p.next_ping = now + t.ping_ms;
            }
        }
        out
    }

    /// Copies this peer's own frames message onto every open link whose
    /// slot plays; anything else stays on the relay.
    pub fn send_frames(&mut self, ls: &Session, bytes: &[u8]) {
        if !wire::is_frames_from(bytes, self.me) {
            return;
        }
        for (&slot, p) in &mut self.peers {
            if p.open()
                && ls.direct_ok(slot)
                && let Some(link) = p.link.as_mut()
            {
                link.send(bytes.to_vec());
                p.stats.packets.0 += 1;
            }
        }
    }

    /// The signals for the relay to pass on.
    pub fn take_signals(&mut self) -> Vec<ToRelay> {
        std::mem::take(&mut self.signals)
    }

    /// Ends every link (the game is over or left).
    pub fn close(&mut self, now: u64) {
        let t = self.direct.timeouts;
        for (&slot, p) in &mut self.peers {
            p.end(now, "the game ended", slot, &t);
        }
    }

    /// Each pair's numbers, with whether its link is open now.
    pub fn stats(&self, now: u64) -> Vec<(u8, bool, PairStats)> {
        self.peers
            .iter()
            .map(|(&slot, p)| {
                let mut s = p.stats.clone();
                if let Some(t) = p.open_since {
                    s.open_ms += now.saturating_sub(t);
                }
                (slot, p.open(), s)
            })
            .collect()
    }
}

/// Opens an attempt, behind the shape if there is one.
fn opened(
    open: &OpenPair,
    direct: &Direct,
    me: u8,
    peer: u8,
    attempt: u32,
    offerer: bool,
) -> Box<dyn PairLink> {
    let info = PairInfo {
        me,
        peer,
        offerer,
        attempt,
        ice: direct.ice.clone(),
    };
    let link = open(&info);
    match direct.shape {
        Some(shape) => {
            // Each link draws its own: the seed, both slots and the attempt.
            let seed =
                shape.seed ^ (u64::from(me) << 8) ^ u64::from(peer) ^ (u64::from(attempt) << 16); // nocite: our own harness (a distinct stream per link)
            Box::new(Shaped {
                inner: link,
                shape,
                draw: Draw(seed),
                up: Lane::new(),
                down: Lane::new(),
                now: 0,
            })
        }
        None => link,
    }
}

/// In-process links for tests: every client of a test opens its attempts
/// on one [`fake::FakeNet`], and the attempts find each other through the
/// signals they send, as real ones do. A pair can be kept from ever
/// connecting ([`fake::FakeNet::block`]) or made to go silent mid-game
/// ([`fake::FakeNet::darken`]).
pub mod fake {
    use std::cell::RefCell;
    use std::collections::{BTreeSet, VecDeque};
    use std::rc::Rc;

    use glasnost_net::lobby::Signal;

    use super::{OpenPair, PairInfo, PairLink, PairState};

    #[derive(Debug, Default)]
    struct End {
        me: u8,
        pair: (u8, u8),
        offerer: bool,
        peer: Option<usize>,
        answered: bool,
        open_at: Option<u64>,
        closed: bool,
        inbox: VecDeque<Vec<u8>>,
        signals: VecDeque<Signal>,
    }

    #[derive(Debug, Default)]
    struct State {
        ends: Vec<End>,
        blocked: BTreeSet<(u8, u8)>,
        dark: BTreeSet<(u8, u8)>,
        open_ms: u64,
        /// Packets passed, and lost to a dark pair.
        passed: u64,
        lost: u64,
    }

    /// The test's network of fake links.
    #[derive(Clone, Debug, Default)]
    pub struct FakeNet(Rc<RefCell<State>>);

    fn key(a: u8, b: u8) -> (u8, u8) {
        (a.min(b), a.max(b))
    }

    impl FakeNet {
        /// A network whose links open `open_ms` after the answer.
        pub fn new(open_ms: u64) -> Self {
            let n = Self::default();
            n.0.borrow_mut().open_ms = open_ms;
            n
        }

        /// The way to open attempts on this network.
        pub fn opener(&self) -> OpenPair {
            let net = self.clone();
            Rc::new(move |info: &PairInfo| Box::new(net.open(info)) as Box<dyn PairLink>)
        }

        /// The pair of slots `a` and `b` never connects (its offers are
        /// never answered) until [`FakeNet::unblock`].
        pub fn block(&self, a: u8, b: u8) {
            self.0.borrow_mut().blocked.insert(key(a, b));
        }

        /// The pair may connect again.
        pub fn unblock(&self, a: u8, b: u8) {
            self.0.borrow_mut().blocked.remove(&key(a, b));
        }

        /// The pair's links stay open but carry nothing, until
        /// [`FakeNet::light`].
        pub fn darken(&self, a: u8, b: u8) {
            self.0.borrow_mut().dark.insert(key(a, b));
        }

        /// The pair's links carry packets again.
        pub fn light(&self, a: u8, b: u8) {
            self.0.borrow_mut().dark.remove(&key(a, b));
        }

        /// Puts `bytes` on the newest link of slot `to` with slot `from`,
        /// as if `from` had sent them; whether there was one.
        pub fn inject(&self, to: u8, from: u8, bytes: Vec<u8>) -> bool {
            let mut s = self.0.borrow_mut();
            let pair = key(to, from);
            match s
                .ends
                .iter_mut()
                .rev()
                .find(|e| e.me == to && e.pair == pair && e.peer.is_some() && !e.closed)
            {
                Some(e) => {
                    e.inbox.push_back(bytes);
                    true
                }
                None => false,
            }
        }

        /// Packets passed and packets lost to a dark pair.
        pub fn counts(&self) -> (u64, u64) {
            let s = self.0.borrow();
            (s.passed, s.lost)
        }

        fn open(&self, info: &PairInfo) -> FakeLink {
            let mut s = self.0.borrow_mut();
            let id = s.ends.len();
            let mut end = End {
                me: info.me,
                pair: key(info.me, info.peer),
                offerer: info.offerer,
                ..End::default()
            };
            if info.offerer {
                end.signals
                    .push_back(Signal::Offer(format!("v=0\r\no=fake {id}\r\n")));
                end.signals
                    .push_back(Signal::Candidate(format!("{{\"fake\":{id}}}")));
            }
            s.ends.push(end);
            FakeLink {
                net: self.clone(),
                id,
                now: 0,
            }
        }
    }

    /// One fake attempt.
    pub struct FakeLink {
        net: FakeNet,
        id: usize,
        now: u64,
    }

    impl PairLink for FakeLink {
        fn poll(&mut self, now_ms: u64) {
            self.now = now_ms;
            let mut s = self.net.0.borrow_mut();
            let open_ms = s.open_ms;
            let e = &mut s.ends[self.id];
            if e.peer.is_some() && e.answered && e.open_at.is_none() {
                e.open_at = Some(now_ms + open_ms);
            }
        }

        fn state(&self) -> PairState {
            let s = self.net.0.borrow();
            let e = &s.ends[self.id];
            if e.closed || e.peer.is_some_and(|p| s.ends[p].closed) {
                return PairState::Closed("closed".into());
            }
            match e.open_at {
                Some(t) if t <= self.now => PairState::Open,
                _ => PairState::Connecting,
            }
        }

        fn send(&mut self, bytes: Vec<u8>) {
            if self.state() != PairState::Open {
                return;
            }
            let mut s = self.net.0.borrow_mut();
            let (pair, peer) = (s.ends[self.id].pair, s.ends[self.id].peer);
            let Some(peer) = peer else {
                return;
            };
            if s.dark.contains(&pair) {
                s.lost += 1;
                return;
            }
            s.passed += 1;
            s.ends[peer].inbox.push_back(bytes);
        }

        fn recv(&mut self) -> Option<Vec<u8>> {
            self.net.0.borrow_mut().ends[self.id].inbox.pop_front()
        }

        fn signal(&mut self, sig: Signal) {
            let mut s = self.net.0.borrow_mut();
            let me = self.id;
            match sig {
                Signal::Offer(text) if !s.ends[me].offerer => {
                    let Some(other) = text
                        .split_whitespace()
                        .last()
                        .and_then(|t| t.parse::<usize>().ok())
                        .filter(|&o| o < s.ends.len() && s.ends[o].offerer)
                    else {
                        return;
                    };
                    if s.ends[other].pair != s.ends[me].pair || s.blocked.contains(&s.ends[me].pair)
                    {
                        return;
                    }
                    s.ends[me].peer = Some(other);
                    s.ends[me].answered = true;
                    s.ends[other].peer = Some(me);
                    s.ends[me]
                        .signals
                        .push_back(Signal::Answer(format!("v=0\r\no=fake-answer {me}\r\n")));
                }
                Signal::Answer(_) if s.ends[me].offerer && s.ends[me].peer.is_some() => {
                    s.ends[me].answered = true;
                }
                _ => {}
            }
        }

        fn take_signal(&mut self) -> Option<Signal> {
            self.net.0.borrow_mut().ends[self.id].signals.pop_front()
        }

        fn close(&mut self) {
            self.net.0.borrow_mut().ends[self.id].closed = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_packets_read_back_and_nothing_else_reads_as_one() {
        let p = control(PING, 0xDEAD_BEEF);
        assert_eq!(read_control(&p), Some((PING, 0xDEAD_BEEF)));
        assert_eq!(read_control(&p[..7]), None);
        let mut q = p.clone();
        q[2] = PING_VERSION + 1;
        assert_eq!(read_control(&q), None);
        assert_eq!(read_control(b"GN\x06\x00\x01\x00\x00\x00"), None);
    }

    #[test]
    fn a_shaped_lane_delays_loses_and_stalls_by_its_seed() {
        let shape = Shape {
            latency_ms: 40,
            jitter_ms: 30,
            loss_permille: 20,
            stall_every_ms: (1_000, 2_000),
            stall_ms: (100, 300),
            seed: 7,
        };
        let run = || {
            let mut d = Draw(shape.seed);
            let mut lane = Lane::new();
            let mut got = Vec::new();
            for t in 0..10_000u64 {
                lane.push(&shape, &mut d, t, vec![(t % 251) as u8]);
                while let Some(b) = lane.due(t) {
                    got.push((t, b[0]));
                }
            }
            (got, lane.lost, lane.queue.len() as u64)
        };
        let (a, lost, queued) = run();
        assert_eq!(
            run(),
            (a.clone(), lost, queued),
            "the same seed, the same run"
        );
        // Stalls of 100 to 300 ms every 1 to 2 s, and 2 % loss besides.
        assert!((1_000..3_000).contains(&lost), "{lost}");
        assert!(queued <= 70, "only the last 70 ms still queued: {queued}");
        assert_eq!(a.len() as u64 + lost + queued, 10_000);
        // Every packet is late by 40 to 70 ms.
        assert!(a.iter().all(|&(t, b)| {
            (t.saturating_sub(70)..=t.saturating_sub(40)).any(|s| (s % 251) as u8 == b)
        }));
    }
}
