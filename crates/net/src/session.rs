//! The lockstep session of one peer.
//!
//! The session is a state machine with no I/O: the caller hands it the
//! bytes that arrived ([`Session::receive`]), asks it for the bytes to send
//! ([`Session::poll`]), gives it the local person's orders
//! ([`Session::submit`]) and asks it to run the next tick when the caller's
//! pacer says one is due ([`Session::step`]). Every call takes the caller's
//! clock in milliseconds; the session reads no clock of its own.
//!
//! **Frames.** Frame *n* of a slot is that slot's input for tick *n*
//! (ticks counted from 0, the session's first tick). A peer that has run
//! *k* ticks with delay *D* has made its frames 0 to *k* + *D* − 1: at the
//! start it makes frames 0 to *D* − 1, empty, and after each tick it makes
//! the frames up to the new horizon, the person's orders since the last
//! frame going into the first of them (the others are empty). A frame is
//! made once and never changes; it is sent again, byte for byte, until
//! every other peer has acknowledged it, and a receiver keeps the first copy
//! of each (slot, frame) and drops the rest. When the delay falls, a peer
//! makes no frame until its horizon is behind it again, and the orders wait
//! for that frame.
//!
//! **The tick rule.** Tick *n* runs only when frame *n* of every live
//! person's slot is in. Its orders are issued in slot order, and within a
//! slot in the order the person gave them; then the game steps (our AI
//! thinks inside the step, for the computer slots, on every peer). So a peer
//! that has run *k* ticks can be at most the delay ahead of the slowest
//! live peer: no peer runs tick *k* + *D* before the slowest has run tick
//! *k*.
//!
//! **The delay.** From tick *n* + 1 on, the delay is the largest proposal in
//! the frames of tick *n* (see [`crate::delay`]). Every peer holds exactly
//! those frames when it runs *n*, so all switch on the same tick.
//!
//! **State hashes.** After every `hash_every` ticks a peer takes the game's
//! state hash (checkpoint *c* after tick (*c* + 1) × `hash_every` − 1) and
//! sends it, again until acknowledged. Each peer compares every other
//! peer's hash with its own and reports a difference as
//! [`Event::Desync`].

use std::collections::{BTreeMap, VecDeque};

use glasnost_sim::orders::Order;
use glasnost_sim::world::HouseId;

use crate::delay::{Control, Rtt};
use crate::sim::Simulation;
use crate::start::{Field, Params, SlotKind, StartInfo};
use crate::wire::{self, Body, Echo, Envelope, Frame, Frames, Hash, WireError};

/// A wait counts as a late frame only while this peer is at most this many
/// ticks ahead of the slowest other peer (estimates are off by a tick).
// nocite: our own netcode
pub const AHEAD_SLACK: i64 = 1;

/// The peer's own settings, which need not agree between peers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    /// The caller's paced tick period, in milliseconds (from the game
    /// speed); the delay is worked out in ticks of this length.
    pub tick_ms: u32,
    /// How long to wait before sending unacknowledged frames, a start or a
    /// keep-alive again, in milliseconds.
    pub resend_ms: u32,
    /// How long a peer may stay unheard before [`Event::Silent`], in
    /// milliseconds.
    pub silent_ms: u32,
    /// Ticks with no wait and no delay change before a lower proposal.
    pub calm_ticks: u32,
}

impl Config {
    /// Settings for a tick period of `tick_ms`, the others at our defaults.
    pub fn new(tick_ms: u32) -> Self {
        Self {
            tick_ms: tick_ms.max(1),
            resend_ms: tick_ms.max(1),
            silent_ms: 5_000, // nocite: our own netcode
            calm_ticks: 90,   // nocite: our own netcode
        }
    }
}

/// Where the session is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Waiting for the start values of every other person's peer.
    Handshake,
    /// Running ticks.
    Running,
    /// A start was refused; the session never runs.
    Refused {
        /// The slot whose start differs (or claims a wrong slot).
        slot: u8,
        /// The first differing value.
        field: Field,
    },
}

/// A slot's state hash at a checkpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotHash {
    /// The slot.
    pub slot: u8,
    /// Its hash.
    pub hash: Hash,
}

/// What happened, for the caller (the client's notices, N4's desync
/// report, N5's drop handling).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// Every other peer's start values match ours; ticks may run.
    Started,
    /// A start was refused: `slot`'s `field` differs from ours.
    Refused {
        /// The slot.
        slot: u8,
        /// The first differing value.
        field: Field,
    },
    /// The frames of `tick` changed the delay, from tick `tick` + 1 on.
    DelayChanged {
        /// The tick whose frames changed it.
        tick: u32,
        /// The delay before.
        from: u8,
        /// The delay after.
        to: u8,
    },
    /// Two peers' state hashes differ after `tick`.
    Desync {
        /// The last tick the hashes cover.
        tick: u32,
        /// Ours.
        mine: SlotHash,
        /// The other peer's.
        theirs: SlotHash,
    },
    /// Nothing heard from `slot` for `ms` milliseconds.
    Silent {
        /// The slot.
        slot: u8,
        /// Milliseconds since it was last heard.
        ms: u64,
    },
    /// `slot` is heard again after [`Event::Silent`].
    Heard {
        /// The slot.
        slot: u8,
    },
    /// `slot` left the session: its frames from `kept` on are empty.
    Dropped {
        /// The slot.
        slot: u8,
        /// How many of its frames count.
        kept: u32,
    },
    /// A message could not be used.
    BadMessage {
        /// Why.
        reason: String,
    },
}

/// One order a tick applied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Applied {
    /// The slot whose frame held it.
    pub slot: u8,
    /// The slot's house.
    pub house: HouseId,
    /// The order.
    pub order: Order,
    /// Whether the simulation took it.
    pub taken: bool,
}

/// One tick the session ran.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tick {
    /// The tick's number (0 for the first).
    pub tick: u32,
    /// The delay from the next tick on.
    pub delay: u8,
    /// The orders applied, in the order they were issued.
    pub orders: Vec<Applied>,
    /// The state hash, when this tick ends a checkpoint.
    pub hash: Option<Hash>,
}

/// What we know of another person's peer.
#[derive(Clone, Debug, Default)]
struct Remote {
    /// Its start values matched ours.
    started: bool,
    /// It has sent frames (so it has accepted every start, ours too).
    running_seen: bool,
    /// Its frames not yet run, by number.
    frames: BTreeMap<u32, Frame>,
    /// How many of its frames we hold or have run, from 0 without a gap.
    have: u32,
    /// How many of our frames it holds, from 0 without a gap.
    acked: u32,
    /// Its checkpoint hashes not yet compared, by checkpoint.
    hashes: BTreeMap<u32, Hash>,
    /// How many of its checkpoint hashes we have, from 0 without a gap.
    hash_have: u32,
    /// How many of its checkpoint hashes we have compared with ours.
    compared: u32,
    /// How many of our checkpoint hashes it holds.
    hash_acked: u32,
    /// The ticks it says it has run.
    ran: u32,
    /// When we last heard from it.
    last_heard: Option<u64>,
    /// Reported silent and not heard since.
    silent: bool,
    /// Its latest stamp and when we received it.
    echo: Option<(u32, u64)>,
    /// The latest of our stamps it echoed that we took a round trip from.
    sampled: Option<u32>,
    /// Round trip to it.
    rtt: Rtt,
    /// It left: its frames from this number on are empty.
    kept: Option<u32>,
}

/// Whether stamp `a` is later than stamp `b` (stamps are milliseconds that
/// wrap).
fn newer(a: u32, b: u32) -> bool {
    (a.wrapping_sub(b) as i32) > 0
}

impl Remote {
    fn live(&self) -> bool {
        self.kept.is_none()
    }

    /// Whether its frame for `tick` is needed.
    fn needs(&self, tick: u32) -> bool {
        self.kept.is_none_or(|k| tick < k)
    }
}

/// The lockstep session of one peer.
#[derive(Clone, Debug)]
pub struct Session {
    cfg: Config,
    info: StartInfo,
    params: Params,
    me: u8,
    phase: Phase,
    remotes: BTreeMap<u8, Remote>,
    /// Ticks run.
    ran: u32,
    /// The delay in effect.
    delay: u8,
    /// Our frames still needed (to run or to resend), by number.
    mine: BTreeMap<u32, Frame>,
    /// How many frames we have made.
    next_frame: u32,
    /// The person's orders for the next frame we make.
    queued: Vec<Order>,
    /// Our checkpoint hashes still needed, by checkpoint.
    my_hashes: BTreeMap<u32, Hash>,
    /// How many checkpoint hashes we have taken.
    hashes_made: u32,
    ctrl: Control,
    events: VecDeque<Event>,
    dirty: bool,
    last_frames_sent: Option<u64>,
    last_start_sent: Option<u64>,
    started_at: Option<u64>,
}

impl Session {
    /// A session for the peer of slot `me` (a person's slot of the table),
    /// starting from `info`, with the caller's clock at `now_ms`. A table
    /// that reaches the relay's sender id ([`wire::RELAY`]) is refused on
    /// the field *slot table*.
    pub fn new(cfg: Config, info: StartInfo, me: u8, now_ms: u64) -> Result<Self, Field> {
        if info.slots.len() > usize::from(wire::RELAY) {
            return Err(Field::Slots);
        }
        if !info.is_human(me) {
            return Err(Field::Slot);
        }
        let params = info.params.sane();
        let remotes = info
            .slots
            .iter()
            .enumerate()
            .filter(|(i, s)| s.kind == SlotKind::Human && *i != usize::from(me))
            .map(|(i, _)| (i as u8, Remote::default()))
            .collect();
        let mut s = Self {
            cfg,
            info,
            params,
            me,
            phase: Phase::Handshake,
            remotes,
            ran: 0,
            delay: params.initial_delay,
            mine: BTreeMap::new(),
            next_frame: 0,
            queued: Vec::new(),
            my_hashes: BTreeMap::new(),
            hashes_made: 0,
            ctrl: Control::default(),
            events: VecDeque::new(),
            dirty: false,
            last_frames_sent: None,
            last_start_sent: None,
            started_at: None,
        };
        s.maybe_start(now_ms);
        Ok(s)
    }

    /// Our slot.
    pub fn me(&self) -> u8 {
        self.me
    }

    /// Our start values.
    pub fn info(&self) -> &StartInfo {
        &self.info
    }

    /// Where the session is.
    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// Ticks run.
    pub fn ran(&self) -> u32 {
        self.ran
    }

    /// The delay in effect.
    pub fn delay(&self) -> u8 {
        self.delay
    }

    /// How many frames this peer has made (its horizon).
    pub fn frames_made(&self) -> u32 {
        self.next_frame
    }

    /// The smoothed round trip to `slot`, in milliseconds.
    pub fn rtt(&self, slot: u8) -> Option<u32> {
        self.remotes.get(&slot).and_then(|r| r.rtt.smoothed())
    }

    /// How many ticks this peer is ahead of the slowest live other peer, by
    /// estimate: the other's ticks run as it last reported them, plus the
    /// ticks it ran since by our clock and half a round trip. A pacer may
    /// stretch its tick period while this is above 1 (N3); the session
    /// itself uses it to tell a late frame from a wait of a peer that is
    /// ahead.
    pub fn ahead(&self, now_ms: u64) -> i64 {
        let tick = u64::from(self.cfg.tick_ms.max(1));
        self.remotes
            .values()
            .filter(|r| r.live() && r.running_seen)
            .map(|r| {
                let at = r.echo.map_or(now_ms, |e| e.1);
                let half = u64::from(r.rtt.smoothed().unwrap_or(0) / 2); // nocite: our own netcode (one way is half the round trip)
                let est = u64::from(r.ran) + (now_ms.saturating_sub(at) + half) / tick;
                i64::from(self.ran) - est.min(i64::MAX as u64) as i64
            })
            .max()
            .unwrap_or(0)
    }

    /// The live slots whose frame for the next tick has not arrived (the
    /// client's "waiting for player" notice).
    pub fn waiting_for(&self) -> Vec<u8> {
        if self.phase != Phase::Running {
            return self
                .remotes
                .iter()
                .filter(|(_, r)| !r.started)
                .map(|(&s, _)| s)
                .collect();
        }
        let tick = self.ran;
        self.remotes
            .iter()
            .filter(|(_, r)| r.needs(tick) && !r.frames.contains_key(&tick))
            .map(|(&s, _)| s)
            .collect()
    }

    /// Takes the events since the last call, oldest first.
    pub fn take_events(&mut self) -> Vec<Event> {
        self.events.drain(..).collect()
    }

    /// Queues a person's order; it goes into the next frame this peer
    /// makes, so it runs one delay or so after the current tick.
    pub fn submit(&mut self, order: Order) {
        self.queued.push(order);
    }

    fn maybe_start(&mut self, now_ms: u64) {
        if self.phase == Phase::Handshake && self.remotes.values().all(|r| r.started) {
            self.phase = Phase::Running;
            self.started_at = Some(now_ms);
            self.events.push_back(Event::Started);
            self.extend();
        }
    }

    fn refuse(&mut self, slot: u8, field: Field) {
        if matches!(self.phase, Phase::Refused { .. }) {
            return;
        }
        self.phase = Phase::Refused { slot, field };
        self.events.push_back(Event::Refused { slot, field });
    }

    fn bad(&mut self, reason: String) {
        self.events.push_back(Event::BadMessage { reason });
    }

    /// Hands the session a message that arrived at `now_ms`.
    pub fn receive(&mut self, now_ms: u64, bytes: &[u8]) {
        let env = match wire::decode(bytes) {
            Ok(e) => e,
            Err(WireError::Version { from, .. }) => {
                self.refuse(from, Field::Protocol);
                return;
            }
            Err(e) => {
                self.bad(e.to_string());
                return;
            }
        };
        let from = env.from;
        // Only the relay reports a drop, and the relay sends nothing else
        // of this protocol (N2): a drop from a peer's slot is a forgery, and
        // is never applied.
        match (from, env.body) {
            (wire::RELAY, Body::Dropped { slot, kept }) => self.drop_slot(slot, kept),
            (wire::RELAY, _) => self.bad("the relay sends only drop reports".into()),
            (_, Body::Dropped { slot, .. }) => self.bad(format!(
                "a drop report for slot {slot} from slot {from}, not from the relay"
            )),
            (_, body) => self.on_message(now_ms, from, body),
        }
    }

    fn on_message(&mut self, now_ms: u64, from: u8, body: Body) {
        let Some(r) = self.remotes.get_mut(&from) else {
            match body {
                Body::Start(_) => self.refuse(from, Field::Slot),
                _ => self.bad(format!(
                    "a message from slot {from}, not another person's slot"
                )),
            }
            return;
        };
        r.last_heard = Some(now_ms);
        if r.silent {
            r.silent = false;
            self.events.push_back(Event::Heard { slot: from });
        }
        match body {
            Body::Start(info) => {
                if matches!(self.phase, Phase::Refused { .. }) || r.started {
                    return;
                }
                match self.info.mismatch(&info) {
                    Some(field) => self.refuse(from, field),
                    None => {
                        r.started = true;
                        self.maybe_start(now_ms);
                    }
                }
            }
            Body::Frames(f) => self.on_frames(now_ms, from, f),
            Body::Dropped { .. } => {}
        }
    }

    /// The highest frame number we accept from a peer: a peer can be at most
    /// one largest delay ahead of us and has made frames one largest delay
    /// beyond that.
    fn frame_limit(&self) -> u32 {
        self.ran
            .saturating_add(2 * u32::from(self.params.max_delay)) // nocite: our own netcode (two largest delays ahead)
            .saturating_add(1)
    }

    fn on_frames(&mut self, now_ms: u64, from: u8, f: Frames) {
        let limit = self.frame_limit();
        let hash_limit = self
            .hashes_made
            .saturating_add(limit / self.params.hash_every + 1); // nocite: our own netcode (checkpoints within the frame window)
        let me = usize::from(self.me);
        let next_frame = self.next_frame;
        let hashes_made = self.hashes_made;
        let Some(r) = self.remotes.get_mut(&from) else {
            return;
        };
        r.running_seen = true;
        r.ran = r.ran.max(f.ran);
        // Only a newer stamp is echoed, and only a newer echo is a sample:
        // a late duplicate or a message overtaken on the way would echo, or
        // be, an old stamp, and its round trip would be a stale one.
        if r.echo.is_none_or(|(s, _)| newer(f.stamp, s)) {
            r.echo = Some((f.stamp, now_ms));
        }
        for e in f.echoes.iter().filter(|e| usize::from(e.slot) == me) {
            if r.sampled.is_none_or(|s| newer(e.stamp, s)) {
                r.sampled = Some(e.stamp);
                let rtt = (now_ms as u32).wrapping_sub(e.stamp).wrapping_sub(e.hold);
                r.rtt.sample(rtt);
            }
        }
        if let Some(&a) = f.acks.get(me) {
            r.acked = r.acked.max(a.min(next_frame));
        }
        if let Some(&a) = f.hash_acks.get(me) {
            r.hash_acked = r.hash_acked.max(a.min(hashes_made));
        }
        for (i, fr) in f.frames.into_iter().enumerate() {
            let t = f.first.saturating_add(i as u32);
            if t < r.have || t >= limit || !r.needs(t) {
                continue;
            }
            r.frames.entry(t).or_insert(fr);
        }
        while r.frames.contains_key(&r.have) {
            r.have += 1;
        }
        for (i, h) in f.hashes.into_iter().enumerate() {
            let c = f.first_hash.saturating_add(i as u32);
            if c < r.hash_have || c >= hash_limit {
                continue;
            }
            r.hashes.entry(c).or_insert(h);
        }
        while r.hashes.contains_key(&r.hash_have) {
            r.hash_have += 1;
        }
        self.compare(from);
    }

    /// Compares `slot`'s checkpoint hashes with ours as far as both are in.
    fn compare(&mut self, slot: u8) {
        let k = self.params.hash_every;
        let me = self.me;
        let Some(r) = self.remotes.get_mut(&slot) else {
            return;
        };
        while r.compared < r.hash_have && r.compared < self.hashes_made {
            let c = r.compared;
            r.compared += 1;
            let (Some(theirs), Some(&mine)) = (r.hashes.remove(&c), self.my_hashes.get(&c)) else {
                continue;
            };
            if theirs != mine {
                self.events.push_back(Event::Desync {
                    tick: (c + 1) * k - 1,
                    mine: SlotHash {
                        slot: me,
                        hash: mine,
                    },
                    theirs: SlotHash { slot, hash: theirs },
                });
            }
        }
    }

    /// Marks `slot` as gone: its frames from number `kept` on are empty and
    /// not awaited (the relay's report, N2; a test's). `kept` must be the
    /// same on every peer: the number of that slot's frames the relay
    /// forwarded to everyone.
    pub fn drop_slot(&mut self, slot: u8, kept: u32) {
        let ran = self.ran;
        let Some(r) = self.remotes.get_mut(&slot) else {
            return;
        };
        if r.kept.is_some() {
            return;
        }
        r.kept = Some(kept);
        r.frames.retain(|&t, _| t < kept);
        self.events.push_back(Event::Dropped { slot, kept });
        if ran > kept {
            self.bad(format!(
                "slot {slot} dropped after frame {kept}, but tick {} already ran with its frames",
                ran - 1
            ));
        }
    }

    /// Frames are due from every live other peer for `tick`.
    fn missing(&self, tick: u32) -> bool {
        self.remotes
            .values()
            .any(|r| r.needs(tick) && !r.frames.contains_key(&tick))
    }

    /// Runs the next tick on `sim` if its frames are all in: issues the
    /// merged orders in slot order, steps, agrees the delay, takes the
    /// checkpoint hash when one is due and makes our next frames. Returns
    /// `None` when a frame is missing, and notes the wait as a late frame
    /// unless this peer is ahead of the others (see [`Session::ahead`]).
    pub fn step<S: Simulation + ?Sized>(&mut self, now_ms: u64, sim: &mut S) -> Option<Tick> {
        if self.phase != Phase::Running {
            return None;
        }
        let tick = self.ran;
        if self.missing(tick) {
            // The first tick waits for the others' start; that is no late
            // frame. Nor is a wait of a peer ahead of the others in time: it
            // waits whatever the delay, and counting it would raise the
            // delay without end.
            if tick > 0 && self.ahead(now_ms) <= AHEAD_SLACK {
                self.ctrl.waited(tick);
            }
            return None;
        }
        let mut proposals = Vec::new();
        let mut applied = Vec::new();
        for (i, slot) in self.info.slots.iter().enumerate() {
            let s = i as u8;
            if slot.kind != SlotKind::Human {
                continue;
            }
            let frame = if s == self.me {
                self.mine.get(&tick).cloned()
            } else {
                self.remotes
                    .get_mut(&s)
                    .filter(|r| r.needs(tick))
                    .and_then(|r| r.frames.remove(&tick))
            };
            let Some(frame) = frame else {
                continue;
            };
            proposals.push(frame.delay);
            let house = HouseId(slot.house);
            for order in frame.orders {
                let taken = sim.issue(house, order.clone());
                applied.push(Applied {
                    slot: s,
                    house,
                    order,
                    taken,
                });
            }
        }
        sim.step();
        self.ran += 1;
        let agreed = proposals
            .iter()
            .copied()
            .max()
            .unwrap_or(self.delay)
            .clamp(self.params.min_delay, self.params.max_delay);
        if agreed != self.delay {
            self.events.push_back(Event::DelayChanged {
                tick,
                from: self.delay,
                to: agreed,
            });
            self.ctrl.changed(tick);
            self.delay = agreed;
        }
        let hash = if self.ran.is_multiple_of(self.params.hash_every) {
            let h = sim.state_hash();
            self.my_hashes.insert(self.hashes_made, h);
            self.hashes_made += 1;
            let slots: Vec<u8> = self.remotes.keys().copied().collect();
            for s in slots {
                self.compare(s);
            }
            Some(h)
        } else {
            None
        };
        self.extend();
        self.prune();
        Some(Tick {
            tick,
            delay: self.delay,
            orders: applied,
            hash,
        })
    }

    /// The largest round-trip timeout to the live other peers.
    fn worst_rto(&self) -> Option<u32> {
        self.remotes
            .values()
            .filter(|r| r.live())
            .filter_map(|r| r.rtt.rto())
            .max()
    }

    /// Makes our frames up to the horizon: ticks run + delay.
    fn extend(&mut self) {
        let target = self.ran + u32::from(self.delay);
        if self.next_frame >= target {
            return;
        }
        let proposal = self.ctrl.propose(
            self.ran,
            self.delay,
            self.worst_rto(),
            self.cfg.tick_ms,
            self.cfg.calm_ticks,
            &self.params,
        );
        let mut orders = std::mem::take(&mut self.queued);
        while self.next_frame < target {
            self.mine.insert(
                self.next_frame,
                Frame {
                    delay: proposal,
                    orders: std::mem::take(&mut orders),
                },
            );
            self.next_frame += 1;
        }
        self.dirty = true;
    }

    /// The first of our frames some live peer still lacks.
    fn first_unacked(&self) -> u32 {
        self.remotes
            .values()
            .filter(|r| r.live())
            .map(|r| r.acked)
            .min()
            .unwrap_or(self.next_frame)
    }

    /// The first of our checkpoints some live peer still lacks or has not
    /// had compared.
    fn first_hash_needed(&self) -> u32 {
        self.remotes
            .values()
            .filter(|r| r.live())
            .map(|r| r.hash_acked.min(r.compared))
            .min()
            .unwrap_or(self.hashes_made)
    }

    /// Forgets our frames that are run and held by every live peer, and our
    /// hashes every live peer holds and has had compared.
    fn prune(&mut self) {
        let keep_from = self.first_unacked().min(self.ran);
        self.mine.retain(|&t, _| t >= keep_from);
        let hash_from = self.first_hash_needed();
        self.my_hashes.retain(|&c, _| c >= hash_from);
    }

    fn envelope(&self, body: Body) -> Vec<u8> {
        wire::encode(&Envelope {
            from: self.me,
            body,
        })
    }

    fn frames_message(&self, now_ms: u64) -> Vec<u8> {
        let first = self.first_unacked();
        let first_hash = self
            .remotes
            .values()
            .filter(|r| r.live())
            .map(|r| r.hash_acked)
            .min()
            .unwrap_or(self.hashes_made);
        let slots = self.info.slots.len();
        let mut acks = vec![0; slots];
        let mut hash_acks = vec![0; slots];
        if let Some(a) = acks.get_mut(usize::from(self.me)) {
            *a = self.next_frame;
        }
        if let Some(a) = hash_acks.get_mut(usize::from(self.me)) {
            *a = self.hashes_made;
        }
        let mut echoes = Vec::new();
        for (&s, r) in &self.remotes {
            if let Some(a) = acks.get_mut(usize::from(s)) {
                *a = r.have;
            }
            if let Some(a) = hash_acks.get_mut(usize::from(s)) {
                *a = r.hash_have;
            }
            if let Some((stamp, at)) = r.echo {
                echoes.push(Echo {
                    slot: s,
                    stamp,
                    hold: now_ms.saturating_sub(at).min(u64::from(u32::MAX)) as u32,
                });
            }
        }
        let frames = (first..self.next_frame)
            .map(|t| self.mine.get(&t).cloned().unwrap_or_default())
            .collect();
        let hashes = (first_hash..self.hashes_made)
            .filter_map(|c| self.my_hashes.get(&c).copied())
            .collect();
        self.envelope(Body::Frames(Frames {
            stamp: now_ms as u32,
            ran: self.ran,
            echoes,
            acks,
            hash_acks,
            first,
            frames,
            first_hash,
            hashes,
        }))
    }

    fn check_silence(&mut self, now_ms: u64) {
        if self.phase != Phase::Running {
            return;
        }
        let since_start = self.started_at.unwrap_or(now_ms);
        let silent_ms = u64::from(self.cfg.silent_ms);
        let mut out = Vec::new();
        for (&s, r) in self.remotes.iter_mut() {
            if !r.live() || r.silent {
                continue;
            }
            let last = r.last_heard.unwrap_or(since_start).max(since_start);
            let ms = now_ms.saturating_sub(last);
            if ms >= silent_ms {
                r.silent = true;
                out.push(Event::Silent { slot: s, ms });
            }
        }
        self.events.extend(out);
    }

    /// The messages to send now, each to every other peer: the start while
    /// some peer may not have it, and the frames when there are new ones or
    /// the resend time has passed.
    pub fn poll(&mut self, now_ms: u64) -> Vec<Vec<u8>> {
        self.check_silence(now_ms);
        let mut out = Vec::new();
        if self.remotes.is_empty() {
            return out;
        }
        let resend = u64::from(self.cfg.resend_ms);
        let start_due = self.last_start_sent.is_none_or(|t| now_ms >= t + resend);
        let start_needed = match self.phase {
            Phase::Handshake | Phase::Refused { .. } => true,
            Phase::Running => self.remotes.values().any(|r| r.live() && !r.running_seen),
        };
        if start_needed && start_due {
            out.push(self.envelope(Body::Start(self.info.clone())));
            self.last_start_sent = Some(now_ms);
        }
        if self.phase == Phase::Running && self.remotes.values().any(Remote::live) {
            let due = self.dirty || self.last_frames_sent.is_none_or(|t| now_ms >= t + resend);
            if due {
                out.push(self.frames_message(now_ms));
                self.dirty = false;
                self.last_frames_sent = Some(now_ms);
            }
        }
        out
    }
}
