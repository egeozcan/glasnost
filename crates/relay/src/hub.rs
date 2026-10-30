//! The relay's core, with no I/O: connections, rooms, forwarding, drops and
//! the limits on hostile input.
//!
//! The server ([`crate::server`]) hands the hub what happens on its sockets
//! ([`Hub::connect`], [`Hub::receive`], [`Hub::heard`], [`Hub::closed`])
//! and its clock ([`Hub::tick`]), and carries out what the hub asks for
//! ([`Hub::take_out`]: send bytes, send a keepalive, close a connection).
//! Every call takes the caller's clock in milliseconds; host randomness
//! (room codes and seeds) comes from the [`Entropy`] the hub is given.
//!
//! **Messages.** A connection sends lobby messages (`GL`,
//! [`glasnost_net::lobby::wire`]) and, once its room's game has begun,
//! lockstep messages (`GN`, [`glasnost_net::wire`]). The hub answers lobby
//! messages itself and forwards lockstep messages, byte for byte, to every
//! other live person of the room, in the order they came.
//!
//! **What the hub never forwards.** A lockstep message whose sender byte is
//! not the connection's slot (a client speaking for another), a
//! [`Body::Dropped`] (only the relay reports drops) and a lockstep message
//! from a connection outside a game are discarded and counted
//! ([`Stats`]). Bytes that are not a message of ours close the connection,
//! with the reason.
//!
//! **Drops.** When a person's connection closes, goes unheard for
//! [`Limits::silent_ms`], breaks a limit or leaves during the game, the hub
//! sends `Dropped { slot, kept }` as [`RELAY`] to every remaining person of
//! the room, after everything it forwarded from that slot. `kept` is the
//! number of that slot's frames, counted from frame 0 without a gap, that
//! the hub forwarded to every other person: each of them holds those
//! frames, since each connection's messages leave in the order the hub
//! queued them, so every peer treats the same frames as the slot's last
//! (N1, `Session::drop_slot`). While a person has no other live person in
//! the game, nobody lacks any of its frames: each of its frames messages
//! then raises `kept` to its end even past a gap (its session sends no
//! frame nobody needs, and a peer rejoining meanwhile is sent the frames
//! below the serving peer's first in the game itself). Unless the slot
//! rejoins, every peer surrenders its house on tick `kept + grace_ticks`;
//! the hub works out `grace_ticks` at the start from [`Limits::grace_ms`]
//! and the room's game speed (N5).
//!
//! **Rejoins (N5).** Every person is sent a secret ticket for its slot in
//! its [`lobby::Begin`]. A connection that brings a slot's ticket back
//! ([`ToRelay::Rejoin`]) while the slot is away and its house has not
//! surrendered becomes that slot's *rejoining* connection (one at a time
//! per game; a ticket for a slot still live replaces its connection). The
//! hub picks the live person of the lowest slot to serve it
//! ([`Body::Serve`]) and from then on also sends the rejoining connection
//! every lockstep message it forwards and every drop; the serving peer's
//! snapshot parts ([`ToRelay::Snapshot`]) go to it, and its messages go to
//! the serving peer only. When it says it has caught up
//! ([`ToRelay::CaughtUp`]), the hub asks the serving peer to readmit it
//! ([`Body::Resume`]); that peer's next frame carries the marker. The hub
//! forwards a marker only from the peer it asked, in one frame, and closes
//! any other sender of one. Once it has forwarded the marker's frame, of
//! tick *n*, the slot is live again from frame `max(n + max_delay + 1,
//! kept)` on, which the hub counts as forwarded, unless *n* is past the
//! surrender's tick. A rejoining connection that goes before that is
//! dropped again at that frame.
//!
//! **Direct links (N7).** The hub passes WebRTC signals
//! ([`ToRelay::Signal`]) between the live persons of a game, from the
//! sender's slot, at most [`Limits::signal_burst`] at once and one per
//! [`Limits::signal_every_ms`] on average; it never reads them beyond a
//! sanity check ([`Signal::plausible`]), and drops the rest. It answers a
//! [`ToRelay::Ping`] at once. It remembers which pairs of slots signalled:
//! their peers may hold frames of each other that came on a data channel
//! and that the hub never saw. So when a slot that signalled drops, the hub
//! does not report the drop at once: it asks each live peer that slot
//! signalled with which of its frames that peer holds beyond `kept`
//! ([`Body::Closing`]), takes the answers ([`Body::Held`], each without a
//! gap from `kept`, the first copy of a frame winning) for at most
//! [`Limits::answer_ms`], closes a peer that has not answered by then, and
//! sends everyone the frames put together, then the `Dropped` whose `kept`
//! counts them. Every live peer then holds every frame of the slot that any
//! live peer ran. The hub stays the only authority on drops: a data channel
//! carries frames, and nothing else.
//!
//! The limits and the protocol are our own design (docs/development.md,
//! *The relay and lobby (N2)*, *Reconnect and surrender (N5)*, *WebRTC
//! (N7)*).

use std::collections::{BTreeMap, BTreeSet};

use glasnost_net::lobby::{
    self, CODE_LEN, ChatLine, FromRelay, MemberId, Part, Refusal, Room, Signal, Started, Ticket,
    ToRelay, Value, normalise_code,
};
use glasnost_net::wire::{
    self, Body, Envelope, Frame, Frames, MAGIC as GAME_MAGIC, RELAY, WireError,
};
use glasnost_net::{Field, Params};
use glasnost_ui::setup::net_period_ms;

/// A connection, by the hub's number (also its member id in a room).
pub type ConnId = MemberId;

/// Host randomness for room codes and game seeds. The server uses the
/// operating system's; tests use a seeded stand-in.
pub trait Entropy: Send {
    /// Fills `buf` with random bytes.
    fn fill(&mut self, buf: &mut [u8]);
}

/// The relay's limits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// The largest message, in bytes.
    pub max_message: usize,
    /// The most open connections.
    pub max_connections: usize,
    /// The most rooms.
    pub max_rooms: usize,
    /// Messages per second a connection may send on average.
    pub rate: u32,
    /// Messages a connection may send at once above the average.
    pub burst: u32,
    /// Refused requests (a wrong room code, say) before the connection is
    /// closed, against guessing room codes.
    pub max_refusals: u32,
    /// How long a connection may stay in no room, in milliseconds.
    pub idle_ms: u64,
    /// How long a connection may stay unheard (no message and no answer to
    /// a keepalive), in milliseconds.
    pub silent_ms: u64,
    /// A keepalive goes to a connection nothing was sent to for this long,
    /// in milliseconds.
    pub keepalive_ms: u64,
    /// A chat line costs this many milliseconds of a connection's chat
    /// bucket, which refills one millisecond a millisecond.
    pub chat_every_ms: u64,
    /// Chat lines a connection may send at once.
    pub chat_burst: u32,
    /// The lockstep parameters of every game (the grace period in ticks
    /// is worked out from `grace_ms` at each start).
    pub params: Params,
    /// How long a dropped person has to rejoin before their house
    /// surrenders, in milliseconds of game time at the room's speed.
    pub grace_ms: u64,
    /// The most bytes of game a serving peer may send one rejoin.
    pub max_snapshot: usize,
    /// A WebRTC signal costs this many milliseconds of a connection's
    /// signal bucket, which refills one millisecond a millisecond (N7).
    pub signal_every_ms: u64,
    /// Signals a connection may send at once (N7).
    pub signal_burst: u32,
    /// How long the peers of a dropping slot have to say which of its
    /// frames they hold, in milliseconds (N7); one that has not answered by
    /// then is closed.
    pub answer_ms: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_message: 64 * 1024, // nocite: our own netcode
            max_connections: 1024,  // nocite: our own netcode
            max_rooms: 256,         // nocite: our own netcode
            rate: 200, // nocite: our own netcode (a lockstep peer sends some 30 to 60 a second)
            burst: 400, // nocite: our own netcode
            max_refusals: 32, // nocite: our own netcode
            idle_ms: 120_000, // nocite: our own netcode
            silent_ms: 15_000, // nocite: our own netcode
            keepalive_ms: 2_000, // nocite: our own netcode
            chat_every_ms: 1_000, // nocite: our own netcode (a line a second on average)
            chat_burst: 5, // nocite: our own netcode
            params: Params::default(),
            grace_ms: 60_000, // nocite: our own netcode
            max_snapshot: glasnost_net::rejoin::MAX_TRANSFER,
            signal_every_ms: 50, // nocite: our own netcode (twenty a second on average)
            signal_burst: 64, // nocite: our own netcode (an offer, an answer and their candidates, several times over)
            answer_ms: 10_000, // nocite: our own netcode (well past a stalled path's few seconds, below silent_ms)
        }
    }
}

/// What the hub asks the server to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Out {
    /// Send these bytes on this connection.
    Send {
        /// The connection.
        to: ConnId,
        /// One message.
        bytes: Vec<u8>,
    },
    /// Send a keepalive (a WebSocket ping) on this connection.
    Keepalive {
        /// The connection.
        to: ConnId,
    },
    /// Close this connection, after the messages before, with this reason.
    /// The hub has forgotten it.
    Close {
        /// The connection.
        to: ConnId,
        /// Why.
        reason: String,
    },
}

/// What the hub counted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    /// Lockstep messages forwarded (once per message, not per receiver).
    pub forwarded: u64,
    /// `Dropped` messages from clients, discarded.
    pub forged_drops: u64,
    /// Lockstep messages whose sender byte was not the sender's slot,
    /// discarded.
    pub forged_senders: u64,
    /// Lockstep messages from a connection outside a game, discarded.
    pub outside_game: u64,
    /// Drops reported.
    pub drops: u64,
    /// Connections refused at the connection limit.
    pub refused_connections: u64,
    /// Connections closed for bytes that are not a message of ours.
    pub closed_unreadable: u64,
    /// Connections closed for a message above the size limit.
    pub closed_size: u64,
    /// Connections closed for too many messages.
    pub closed_rate: u64,
    /// Connections closed for too many refused requests.
    pub closed_refusals: u64,
    /// Connections closed for silence or for staying in no room.
    pub closed_silent: u64,
    /// Chat lines passed on (once per line, not per receiver).
    pub chats: u64,
    /// Chat lines refused as too fast.
    pub chats_too_fast: u64,
    /// Rejoins begun.
    pub rejoins: u64,
    /// Rejoining peers readmitted.
    pub readmitted: u64,
    /// Readmission markers or relay messages from clients nobody asked
    /// for: the sender is closed (markers) or the message discarded.
    pub forged_rejoins: u64,
    /// Bytes of game passed on to rejoining peers.
    pub snapshot_bytes: u64,
    /// WebRTC signals passed on (N7).
    pub signals: u64,
    /// WebRTC signals dropped: too fast, malformed, outside a game or to a
    /// slot that is not another live person's (N7).
    pub signals_dropped: u64,
    /// Drops that asked the peers for the slot's frames first (N7).
    pub rounds: u64,
    /// Answers' parts taken in those rounds.
    pub held_parts: u64,
    /// Answers' parts not taken: late, unasked, out of order, too long or
    /// with a readmission marker (that answer then ends).
    pub held_refused: u64,
    /// Peers closed for not answering a round in time.
    pub unanswered: u64,
    /// Messages received from connections (N8b).
    pub messages_in: u64,
    /// Bytes of those messages.
    pub bytes_in: u64,
    /// Messages queued for connections, once per receiver (N8b).
    pub messages_out: u64,
    /// Bytes of those messages.
    pub bytes_out: u64,
    /// Requests the relay answered with a refusal (N8b).
    pub refusals: u64,
}

impl Stats {
    /// What was counted since `earlier` (counters only grow; a smaller
    /// number counts as 0).
    pub fn since(&self, earlier: &Stats) -> Stats {
        let d = |a: u64, b: u64| a.saturating_sub(b);
        Stats {
            forwarded: d(self.forwarded, earlier.forwarded),
            forged_drops: d(self.forged_drops, earlier.forged_drops),
            forged_senders: d(self.forged_senders, earlier.forged_senders),
            outside_game: d(self.outside_game, earlier.outside_game),
            drops: d(self.drops, earlier.drops),
            refused_connections: d(self.refused_connections, earlier.refused_connections),
            closed_unreadable: d(self.closed_unreadable, earlier.closed_unreadable),
            closed_size: d(self.closed_size, earlier.closed_size),
            closed_rate: d(self.closed_rate, earlier.closed_rate),
            closed_refusals: d(self.closed_refusals, earlier.closed_refusals),
            closed_silent: d(self.closed_silent, earlier.closed_silent),
            chats: d(self.chats, earlier.chats),
            chats_too_fast: d(self.chats_too_fast, earlier.chats_too_fast),
            rejoins: d(self.rejoins, earlier.rejoins),
            readmitted: d(self.readmitted, earlier.readmitted),
            forged_rejoins: d(self.forged_rejoins, earlier.forged_rejoins),
            snapshot_bytes: d(self.snapshot_bytes, earlier.snapshot_bytes),
            signals: d(self.signals, earlier.signals),
            signals_dropped: d(self.signals_dropped, earlier.signals_dropped),
            rounds: d(self.rounds, earlier.rounds),
            held_parts: d(self.held_parts, earlier.held_parts),
            held_refused: d(self.held_refused, earlier.held_refused),
            unanswered: d(self.unanswered, earlier.unanswered),
            messages_in: d(self.messages_in, earlier.messages_in),
            bytes_in: d(self.bytes_in, earlier.bytes_in),
            messages_out: d(self.messages_out, earlier.messages_out),
            bytes_out: d(self.bytes_out, earlier.bytes_out),
            refusals: d(self.refusals, earlier.refusals),
        }
    }
}

/// The relay's periodic line (`--stats`): what it holds now and what
/// `delta` (the counts since the last line, [`Stats::since`]) says happened
/// in `seconds`. Counts only: no codes, names or addresses.
pub fn stats_line(hub: &Hub, delta: &Stats, seconds: u64) -> String {
    format!(
        "stats rooms={} connections={} games={} in={}msg/{}B out={}msg/{}B \
         refused={} drops={} rejoins={} over={}s",
        hub.rooms(),
        hub.connections(),
        hub.games(),
        delta.messages_in,
        delta.bytes_in,
        delta.messages_out,
        delta.bytes_out,
        delta.refusals + delta.refused_connections,
        delta.drops,
        delta.rejoins,
        seconds
    )
}

#[derive(Clone, Debug)]
struct Conn {
    last_heard: u64,
    last_sent: u64,
    /// Since when it is in no room.
    idle_since: Option<u64>,
    /// The rate bucket, in thousandths of a message.
    tokens: u64,
    refilled: u64,
    room: Option<String>,
    refusals: u32,
    /// The chat bucket, in milliseconds.
    chat_tokens: u64,
    chat_refilled: u64,
    /// The signal bucket, in milliseconds (N7).
    signal_tokens: u64,
    signal_refilled: u64,
}

/// A drop that waits for the peers' answers (N7).
#[derive(Clone, Debug)]
struct Round {
    /// The dropping slot.
    slot: u8,
    /// Its frames forwarded to everyone.
    forwarded: u32,
    /// The peers whose answer is due, by slot, with the frames of theirs
    /// taken so far.
    waiting: BTreeMap<u8, u32>,
    /// The frames from `forwarded` on, first copy first.
    merged: Vec<Frame>,
    /// When the peers that have not answered are closed.
    deadline: u64,
}

/// A rejoin in progress.
#[derive(Clone, Debug)]
struct Rejoin {
    /// The slot.
    slot: u8,
    /// The rejoining connection, until it goes.
    conn: Option<ConnId>,
    /// The serving peer's slot and connection.
    donor: u8,
    donor_conn: ConnId,
    /// The serving peer was asked to readmit it.
    asked: bool,
    /// The frame of the serving peer that carries the marker.
    marker: Option<u32>,
    /// Bytes of game passed on.
    sent: usize,
}

/// A marker the hub let through: it may come again in resent frames.
#[derive(Clone, Copy, Debug)]
struct Marker {
    from: u8,
    frame: u32,
    slot: u8,
}

/// A room's game: who plays each slot and what was forwarded.
#[derive(Clone, Debug)]
struct Game {
    /// Per slot: the connection of its person while it is live.
    conns: Vec<Option<ConnId>>,
    /// Per slot: its frames forwarded to every other live person, from
    /// frame 0 without a gap; while the slot's person is the only live one,
    /// to the furthest end of its frames messages.
    kept: Vec<u32>,
    /// The start values, for a rejoin.
    started: Started,
    /// Per slot: its ticket (zero for a computer's slot).
    tickets: Vec<Ticket>,
    /// Per slot: its house surrendered after a drop.
    surrendered: Vec<bool>,
    /// The most ticks any live person said it had run.
    max_ran: u32,
    /// The rejoin in progress.
    rejoin: Option<Rejoin>,
    /// Markers let through.
    markers: Vec<Marker>,
    /// Pairs of slots (lower first) whose peers signalled (N7).
    signalled: BTreeSet<(u8, u8)>,
    /// Drops waiting for answers (N7).
    rounds: Vec<Round>,
}

impl Game {
    fn human(&self, slot: u8) -> bool {
        self.started.persons.iter().any(|&(_, s)| s == slot)
    }

    /// The tick a dropped slot's house surrenders on.
    fn surrender_at(&self, slot: u8) -> u32 {
        self.kept[usize::from(slot)].saturating_add(self.started.begin.params.grace_ticks)
    }

    /// Whether a dropped slot's house has surrendered, as far as the hub
    /// can tell: some live person has run the surrender's tick.
    fn surrendered(&self, slot: u8) -> bool {
        self.surrendered[usize::from(slot)]
            || (self.conns[usize::from(slot)].is_none() && self.max_ran > self.surrender_at(slot))
    }

    /// The rejoining connection, while there is one.
    fn catching(&self) -> Option<ConnId> {
        self.rejoin.as_ref().and_then(|r| r.conn)
    }
    fn slot_of(&self, conn: ConnId) -> Option<u8> {
        self.conns
            .iter()
            .position(|c| *c == Some(conn))
            .map(|s| s as u8)
    }

    fn live(&self) -> impl Iterator<Item = (u8, ConnId)> + '_ {
        self.conns
            .iter()
            .enumerate()
            .filter_map(|(s, c)| c.map(|c| (s as u8, c)))
    }
}

#[derive(Clone, Debug)]
struct Entry {
    room: Room,
    game: Option<Game>,
}

/// The relay's state.
pub struct Hub {
    limits: Limits,
    entropy: Box<dyn Entropy>,
    conns: BTreeMap<ConnId, Conn>,
    rooms: BTreeMap<String, Entry>,
    next: ConnId,
    stats: Stats,
    out: Vec<Out>,
}

/// Tries at a fresh room code before the hub gives up (a collision of 40
/// random bits among a few hundred rooms is rare already).
// nocite: our own netcode
const CODE_TRIES: usize = 16;

/// One message in the rate bucket, which counts thousandths of a message.
// nocite: our own netcode
const ONE: u64 = 1000;

/// A WebSocket close reason holds at most 123 bytes; reasons are kept
/// shorter, cut at a character boundary.
// nocite: the WebSocket protocol (RFC 6455, 125-byte control frames)
pub const MAX_REASON: usize = 123;

/// A lockstep message of the relay's own.
fn relay_message(body: Body) -> Vec<u8> {
    wire::encode(&Envelope { from: RELAY, body })
}

/// Whether every readmission marker in `slot`'s frames is one the hub
/// asked that peer for, in one frame (N5); a new one is recorded as the
/// rejoin's.
fn markers_allowed(g: &mut Game, slot: u8, f: &Frames) -> bool {
    for (i, fr) in f.frames.iter().enumerate() {
        let Some(target) = fr.resume else {
            continue;
        };
        let n = f.first.saturating_add(i as u32);
        if g.markers
            .iter()
            .any(|m| m.from == slot && m.frame == n && m.slot == target)
        {
            continue;
        }
        match g.rejoin.as_mut() {
            Some(r) if r.asked && r.donor == slot && r.slot == target && r.marker.is_none() => {
                r.marker = Some(n);
                g.markers.push(Marker {
                    from: slot,
                    frame: n,
                    slot: target,
                });
            }
            _ => return false,
        }
    }
    true
}

fn short(reason: &str) -> String {
    let mut end = reason.len().min(MAX_REASON);
    while !reason.is_char_boundary(end) {
        end -= 1;
    }
    reason[..end].to_owned()
}

impl Hub {
    /// A hub with these limits and this source of host randomness.
    pub fn new(limits: Limits, entropy: Box<dyn Entropy>) -> Self {
        Self {
            limits,
            entropy,
            conns: BTreeMap::new(),
            rooms: BTreeMap::new(),
            next: 1,
            stats: Stats::default(),
            out: Vec::new(),
        }
    }

    /// The limits.
    pub fn limits(&self) -> &Limits {
        &self.limits
    }

    /// What the hub counted.
    pub fn stats(&self) -> Stats {
        self.stats
    }

    /// Open connections.
    pub fn connections(&self) -> usize {
        self.conns.len()
    }

    /// Rooms whose game has begun.
    pub fn games(&self) -> usize {
        self.rooms.values().filter(|e| e.room.playing()).count()
    }

    /// Rooms.
    pub fn rooms(&self) -> usize {
        self.rooms.len()
    }

    /// The codes of the rooms, in order.
    pub fn codes(&self) -> Vec<String> {
        self.rooms.keys().cloned().collect()
    }

    /// The room `code`.
    pub fn room(&self, code: &str) -> Option<&Room> {
        self.rooms.get(code).map(|e| &e.room)
    }

    /// The number of `slot`'s frames forwarded to everyone in room `code`'s
    /// game (what a drop of that slot reports as `kept`).
    pub fn kept(&self, code: &str, slot: u8) -> Option<u32> {
        let g = self.rooms.get(code)?.game.as_ref()?;
        g.kept.get(usize::from(slot)).copied()
    }

    /// Takes what the server should do, oldest first.
    pub fn take_out(&mut self) -> Vec<Out> {
        std::mem::take(&mut self.out)
    }

    /// A new connection at `now_ms`; `None` at the connection limit (the
    /// server then closes the socket).
    pub fn connect(&mut self, now_ms: u64) -> Option<ConnId> {
        if self.conns.len() >= self.limits.max_connections {
            self.stats.refused_connections += 1;
            return None;
        }
        let id = self.next;
        self.next += 1;
        self.conns.insert(
            id,
            Conn {
                last_heard: now_ms,
                last_sent: now_ms,
                idle_since: Some(now_ms),
                tokens: u64::from(self.limits.burst) * ONE,
                refilled: now_ms,
                room: None,
                refusals: 0,
                chat_tokens: u64::from(self.limits.chat_burst) * self.limits.chat_every_ms,
                chat_refilled: now_ms,
                signal_tokens: u64::from(self.limits.signal_burst) * self.limits.signal_every_ms,
                signal_refilled: now_ms,
            },
        );
        Some(id)
    }

    /// Something came from `conn` that is not a message (a keepalive's
    /// answer).
    pub fn heard(&mut self, now_ms: u64, conn: ConnId) {
        if let Some(c) = self.conns.get_mut(&conn) {
            c.last_heard = c.last_heard.max(now_ms);
        }
    }

    /// `conn`'s socket closed (or failed): it leaves its room, or is
    /// dropped from its game.
    pub fn closed(&mut self, now_ms: u64, conn: ConnId) {
        self.forget(now_ms, conn);
    }

    /// Closes `conn` with `reason`, as for a broken limit (the server's own
    /// reasons, such as a queue that does not drain).
    pub fn kick(&mut self, now_ms: u64, conn: ConnId, reason: &str) {
        self.close(now_ms, conn, reason);
    }

    /// `conn` sent a message above [`Limits::max_message`] that the
    /// server did not read whole: it is closed.
    pub fn too_large(&mut self, now_ms: u64, conn: ConnId) {
        if self.conns.contains_key(&conn) {
            self.stats.closed_size += 1;
            self.close(now_ms, conn, "message too large");
        }
    }

    fn send(&mut self, now_ms: u64, to: ConnId, bytes: Vec<u8>) {
        if let Some(c) = self.conns.get_mut(&to) {
            c.last_sent = now_ms;
            self.stats.messages_out += 1;
            self.stats.bytes_out += bytes.len() as u64;
            self.out.push(Out::Send { to, bytes });
        }
    }

    fn send_lobby(&mut self, now_ms: u64, to: ConnId, msg: &FromRelay) {
        if matches!(msg, FromRelay::Refused(_)) {
            self.stats.refusals += 1;
        }
        self.send(now_ms, to, lobby::wire::encode(msg));
    }

    fn close(&mut self, now_ms: u64, conn: ConnId, reason: &str) {
        if !self.conns.contains_key(&conn) {
            return;
        }
        let reason = short(reason);
        self.send_lobby(now_ms, conn, &FromRelay::Closing(reason.clone()));
        self.forget(now_ms, conn);
        self.out.push(Out::Close { to: conn, reason });
    }

    /// Removes `conn` and takes it out of its room or game.
    fn forget(&mut self, now_ms: u64, conn: ConnId) {
        let Some(c) = self.conns.remove(&conn) else {
            return;
        };
        if let Some(code) = c.room {
            self.leave_room(now_ms, conn, &code);
        }
    }

    /// `conn` leaves room `code`: a drop when its game has begun.
    fn leave_room(&mut self, now_ms: u64, conn: ConnId, code: &str) {
        let Some(e) = self.rooms.get_mut(code) else {
            return;
        };
        if let Some(g) = e.game.as_mut() {
            let mut sends = Vec::new();
            let mut closes = Vec::new();
            if let Some(r) = g.rejoin.as_mut().filter(|r| r.conn == Some(conn)) {
                // The rejoining connection goes. Once the serving peer was
                // asked to readmit it, its marker may be on the way: it is
                // let through, and the slot drops again at its frame.
                e.room.leave(conn);
                if r.asked {
                    r.conn = None;
                } else {
                    sends.push((
                        r.donor_conn,
                        relay_message(Body::Serve {
                            slot: r.slot,
                            on: false,
                        }),
                    ));
                    g.rejoin = None;
                }
            } else if let Some(slot) = g.slot_of(conn) {
                g.conns[usize::from(slot)] = None;
                let kept = g.kept[usize::from(slot)];
                e.room.leave(conn);
                // It answers no round now (N7).
                for r in &mut g.rounds {
                    r.waiting.remove(&slot);
                }
                let peers: BTreeMap<u8, u32> = g
                    .live()
                    .filter(|&(s, _)| g.signalled.contains(&(s.min(slot), s.max(slot))))
                    .map(|(s, _)| (s, 0))
                    .collect();
                g.signalled.retain(|&(a, b)| a != slot && b != slot);
                if peers.is_empty() {
                    let bytes = relay_message(Body::Dropped { slot, kept });
                    self.stats.drops += 1;
                    for t in g.live().map(|(_, c)| c).chain(g.catching()) {
                        sends.push((t, bytes.clone()));
                    }
                } else {
                    // Its direct peers may hold frames the hub never saw:
                    // they are asked first (N7).
                    let ask = relay_message(Body::Closing {
                        slot,
                        forwarded: kept,
                    });
                    for &s in peers.keys() {
                        if let Some(c) = g.conns[usize::from(s)] {
                            sends.push((c, ask.clone()));
                        }
                    }
                    self.stats.rounds += 1;
                    g.rounds.push(Round {
                        slot,
                        forwarded: kept,
                        waiting: peers,
                        merged: Vec::new(),
                        deadline: now_ms.saturating_add(self.limits.answer_ms),
                    });
                }
                if g.rejoin.as_ref().is_some_and(|r| r.donor == slot) {
                    // The serving peer went before the readmission: that
                    // rejoin cannot finish.
                    let r = g.rejoin.take().expect("a rejoin");
                    closes.extend(r.conn.map(|c| {
                        (
                            c,
                            "the player sending you the game left; reconnect to try again",
                        )
                    }));
                }
            }
            let empty = g.live().next().is_none();
            if empty {
                closes.extend(g.catching().map(|c| (c, "everyone else left the game")));
                self.rooms.remove(code);
            }
            for (t, b) in sends {
                self.send(now_ms, t, b);
            }
            for (c, why) in closes {
                self.close(now_ms, c, why);
            }
            if !empty {
                self.finish_rounds(now_ms, code, false);
            }
            return;
        }
        e.room.leave(conn);
        if e.room.is_empty() {
            self.rooms.remove(code);
        } else {
            self.broadcast(now_ms, code);
        }
    }

    /// Sends every member of room `code` its view.
    fn broadcast(&mut self, now_ms: u64, code: &str) {
        let Some(e) = self.rooms.get(code) else {
            return;
        };
        let views: Vec<(ConnId, FromRelay)> = e
            .room
            .members()
            .map(|m| (m, FromRelay::Room(e.room.view(m))))
            .collect();
        for (m, v) in views {
            self.send_lobby(now_ms, m, &v);
        }
    }

    fn refuse(&mut self, now_ms: u64, conn: ConnId, why: Refusal) {
        self.send_lobby(now_ms, conn, &FromRelay::Refused(why));
        let Some(c) = self.conns.get_mut(&conn) else {
            return;
        };
        c.refusals += 1;
        if c.refusals > self.limits.max_refusals {
            self.stats.closed_refusals += 1;
            self.close(now_ms, conn, "too many refused requests");
        }
    }

    /// Takes one message's worth from `conn`'s rate bucket; false when it
    /// is empty.
    fn spend(&mut self, now_ms: u64, conn: ConnId) -> bool {
        let rate = u64::from(self.limits.rate);
        let cap = u64::from(self.limits.burst) * ONE;
        let Some(c) = self.conns.get_mut(&conn) else {
            return false;
        };
        let ms = now_ms.saturating_sub(c.refilled);
        c.refilled = c.refilled.max(now_ms);
        // `rate` a second is `rate` thousandths a millisecond.
        c.tokens = c.tokens.saturating_add(ms.saturating_mul(rate)).min(cap);
        if c.tokens < ONE {
            return false;
        }
        c.tokens -= ONE;
        true
    }

    /// Takes one chat line's worth from `conn`'s chat bucket; false when it
    /// is too low.
    fn spend_chat(&mut self, now_ms: u64, conn: ConnId) -> bool {
        let cost = self.limits.chat_every_ms;
        let cap = u64::from(self.limits.chat_burst) * cost;
        let Some(c) = self.conns.get_mut(&conn) else {
            return false;
        };
        let ms = now_ms.saturating_sub(c.chat_refilled);
        c.chat_refilled = c.chat_refilled.max(now_ms);
        c.chat_tokens = c.chat_tokens.saturating_add(ms).min(cap);
        if c.chat_tokens < cost {
            return false;
        }
        c.chat_tokens -= cost;
        true
    }

    /// A chat line from `conn`: to every member of its room, the sender
    /// too, before or during the game. Lines too fast are refused without
    /// counting against the refusal limit.
    fn on_chat(&mut self, now_ms: u64, conn: ConnId, text: &str) {
        let Some(code) = self.room_of(conn) else {
            return self.refuse(now_ms, conn, Refusal::NotInRoom);
        };
        let Some(text) = lobby::clean_chat(text) else {
            return self.refuse(now_ms, conn, Refusal::Bad(Value::Text));
        };
        if !self.spend_chat(now_ms, conn) {
            self.stats.chats_too_fast += 1;
            return self.send_lobby(now_ms, conn, &FromRelay::Refused(Refusal::ChatTooFast));
        }
        let Some(e) = self.rooms.get(&code) else {
            return;
        };
        let line = FromRelay::Chat(ChatLine {
            from: conn,
            row: e.room.row_of(conn).unwrap_or(0),
            name: e.room.name_of(conn).unwrap_or_default().to_owned(),
            text,
        });
        let to: Vec<ConnId> = e.room.members().collect();
        let bytes = lobby::wire::encode(&line);
        self.stats.chats += 1;
        for t in to {
            self.send(now_ms, t, bytes.clone());
        }
    }

    /// Bytes from `conn` at `now_ms`: one WebSocket message.
    pub fn receive(&mut self, now_ms: u64, conn: ConnId, bytes: &[u8]) {
        if !self.conns.contains_key(&conn) {
            return;
        }
        self.stats.messages_in += 1;
        self.stats.bytes_in += bytes.len() as u64;
        self.heard(now_ms, conn);
        if !self.spend(now_ms, conn) {
            self.stats.closed_rate += 1;
            self.close(now_ms, conn, "too many messages");
            return;
        }
        if bytes.len() > self.limits.max_message {
            self.stats.closed_size += 1;
            self.close(now_ms, conn, "message too large");
            return;
        }
        if lobby::wire::is_lobby(bytes) {
            match lobby::wire::decode::<ToRelay>(bytes) {
                Ok(msg) => self.on_lobby(now_ms, conn, msg),
                Err(lobby::wire::Error::Version(_)) => {
                    self.send_lobby(
                        now_ms,
                        conn,
                        &FromRelay::Refused(Refusal::Mismatch(Field::Protocol)),
                    );
                    self.stats.closed_unreadable += 1;
                    self.close(now_ms, conn, "another protocol version");
                }
                Err(e) => {
                    self.stats.closed_unreadable += 1;
                    self.close(now_ms, conn, &format!("unreadable lobby message: {e}"));
                }
            }
        } else if bytes.get(..2) == Some(&GAME_MAGIC[..]) {
            self.on_game(now_ms, conn, bytes);
        } else {
            self.stats.closed_unreadable += 1;
            self.close(now_ms, conn, "not a Glasnost message");
        }
    }

    fn on_game(&mut self, now_ms: u64, conn: ConnId, bytes: &[u8]) {
        let env = match wire::decode(bytes) {
            Ok(env) => env,
            Err(WireError::Version { .. }) => {
                self.stats.closed_unreadable += 1;
                self.close(now_ms, conn, "another protocol version");
                return;
            }
            Err(e) => {
                self.stats.closed_unreadable += 1;
                self.close(now_ms, conn, &format!("unreadable game message: {e}"));
                return;
            }
        };
        let code = self.room_of(conn);
        let Some(game) = code.and_then(|code| self.rooms.get_mut(&code)?.game.as_mut()) else {
            self.stats.outside_game += 1;
            return;
        };
        if let Some(r) = game.rejoin.as_ref().filter(|r| r.conn == Some(conn)) {
            // The rejoining peer's acknowledgements, for the serving peer
            // only; it makes no frame before it is readmitted.
            if env.from != r.slot {
                self.stats.forged_senders += 1;
                return;
            }
            if !matches!(env.body, Body::Frames(_)) {
                self.stats.forged_rejoins += 1;
                return;
            }
            let to = r.donor_conn;
            self.stats.forwarded += 1;
            self.send(now_ms, to, bytes.to_vec());
            return;
        }
        let Some(slot) = game.slot_of(conn) else {
            self.stats.outside_game += 1;
            return;
        };
        if env.from != slot {
            self.stats.forged_senders += 1;
            return;
        }
        match &env.body {
            Body::Dropped { .. } => {
                self.stats.forged_drops += 1;
                return;
            }
            Body::Serve { .. } | Body::Resume { .. } => {
                self.stats.forged_rejoins += 1;
                return;
            }
            Body::Closing { .. } => {
                self.stats.forged_drops += 1;
                return;
            }
            Body::Held {
                slot: of,
                first,
                frames,
                last,
            } => {
                let cap = glasnost_net::session::lead(&game.started.begin.params);
                let taken = game
                    .rounds
                    .iter_mut()
                    .find(|r| r.slot == *of)
                    .and_then(|r| {
                        let n = r.waiting.get_mut(&slot)?;
                        let at = r.forwarded.saturating_add(*n);
                        let ok = *first == at
                            && n.saturating_add(frames.len() as u32) <= cap
                            && frames.iter().all(|f| f.resume.is_none());
                        if !ok {
                            // That answer ends here.
                            r.waiting.remove(&slot);
                            return Some(false);
                        }
                        for (i, f) in frames.iter().enumerate() {
                            let k = (*n as usize) + i;
                            if k == r.merged.len() {
                                r.merged.push(f.clone());
                            }
                        }
                        *n = n.saturating_add(frames.len() as u32);
                        if *last {
                            r.waiting.remove(&slot);
                        }
                        Some(true)
                    });
                match taken {
                    Some(true) => self.stats.held_parts += 1,
                    _ => self.stats.held_refused += 1,
                }
                if let Some(code) = self.room_of(conn) {
                    self.finish_rounds(now_ms, &code, false);
                }
                return;
            }
            Body::Frames(f) => {
                if !markers_allowed(game, slot, f) {
                    self.stats.forged_rejoins += 1;
                    self.close(now_ms, conn, "a readmission marker nobody asked for");
                    return;
                }
                game.markers
                    .retain(|m| m.from != slot || m.frame >= f.first);
                // With no other live person, every frame of the slot is
                // held by everyone, so a gap counts too: a lone peer sends
                // none of the frames nobody needs (its first is its next
                // frame), and a rejoining peer has the frames below the
                // serving peer's first in the game it is sent. Our own
                // netcode (the M9 exit run).
                let alone = game.live().all(|(s, _)| s == slot);
                let k = &mut game.kept[usize::from(slot)];
                if f.first <= *k || alone {
                    *k = (*k).max(f.first.saturating_add(f.frames.len() as u32));
                }
                game.max_ran = game.max_ran.max(f.ran);
            }
            Body::Start(_) => {}
        }
        let to: Vec<ConnId> = game
            .live()
            .filter(|&(s, _)| s != slot)
            .map(|(_, c)| c)
            .chain(game.catching())
            .collect();
        self.stats.forwarded += 1;
        for t in to {
            self.send(now_ms, t, bytes.to_vec());
        }
        if let Some(code) = self.room_of(conn) {
            self.after_forward(now_ms, &code);
        }
    }

    /// After a forward in room `code`'s game: a readmission marker now
    /// held by everyone takes effect, and a rejoin whose house has
    /// surrendered meanwhile ends.
    fn after_forward(&mut self, now_ms: u64, code: &str) {
        let Some(g) = self.rooms.get_mut(code).and_then(|e| e.game.as_mut()) else {
            return;
        };
        let Some(r) = g.rejoin.clone() else {
            return;
        };
        let s = usize::from(r.slot);
        let mut sends = Vec::new();
        let mut close = None;
        if let Some(n) = r.marker {
            g.rejoin = None;
            if n >= g.surrender_at(r.slot) {
                // Too late: every peer surrenders the house before its tick
                // and lets the marker go.
                g.surrendered[s] = true;
                close = r.conn.map(|c| {
                    (
                        c,
                        "too late: the grace period ran out and your house surrendered",
                    )
                });
            } else {
                let gap = u32::from(g.started.begin.params.max_delay) + 1; // nocite: our own netcode (as `Session`'s readmission)
                let from = n.saturating_add(gap).max(g.kept[s]);
                g.kept[s] = from;
                match r.conn {
                    Some(c) => {
                        g.conns[s] = Some(c);
                        self.stats.readmitted += 1;
                    }
                    None => {
                        let bytes = relay_message(Body::Dropped {
                            slot: r.slot,
                            kept: from,
                        });
                        self.stats.drops += 1;
                        sends.extend(g.live().map(|(_, c)| (c, bytes.clone())));
                    }
                }
            }
        } else if g.surrendered(r.slot) {
            g.rejoin = None;
            g.surrendered[s] = true;
            sends.push((
                r.donor_conn,
                relay_message(Body::Serve {
                    slot: r.slot,
                    on: false,
                }),
            ));
            close = r.conn.map(|c| {
                (
                    c,
                    "too late: the grace period ran out and your house surrendered",
                )
            });
        }
        for (t, b) in sends {
            self.send(now_ms, t, b);
        }
        if let Some((c, why)) = close {
            self.close(now_ms, c, why);
        }
    }

    fn room_of(&self, conn: ConnId) -> Option<String> {
        self.conns.get(&conn).and_then(|c| c.room.clone())
    }

    fn on_lobby(&mut self, now_ms: u64, conn: ConnId, msg: ToRelay) {
        let in_room = self.room_of(conn);
        match &msg {
            ToRelay::Create { .. } => {
                if in_room.is_some() {
                    return self.refuse(now_ms, conn, Refusal::InRoom);
                }
                if self.rooms.len() >= self.limits.max_rooms {
                    return self.refuse(now_ms, conn, Refusal::TooManyRooms);
                }
                let Some(code) = self.fresh_code() else {
                    return self.refuse(now_ms, conn, Refusal::TooManyRooms);
                };
                match Room::create(code.clone(), conn, &msg) {
                    Ok(room) => {
                        self.rooms.insert(code.clone(), Entry { room, game: None });
                        self.enter(conn, &code);
                        self.broadcast(now_ms, &code);
                    }
                    Err(r) => self.refuse(now_ms, conn, r),
                }
            }
            ToRelay::Peek { code } => match normalise_code(code).and_then(|c| self.rooms.get(&c)) {
                Some(e) => {
                    let info = FromRelay::Info(e.room.info());
                    self.send_lobby(now_ms, conn, &info);
                }
                None => self.refuse(now_ms, conn, Refusal::NoRoom),
            },
            ToRelay::Join { code, .. } => {
                if in_room.is_some() {
                    return self.refuse(now_ms, conn, Refusal::InRoom);
                }
                let Some(code) = normalise_code(code).filter(|c| self.rooms.contains_key(c)) else {
                    return self.refuse(now_ms, conn, Refusal::NoRoom);
                };
                let e = self.rooms.get_mut(&code).expect("the room exists");
                match e.room.join(conn, &msg) {
                    Ok(_) => {
                        self.enter(conn, &code);
                        self.broadcast(now_ms, &code);
                    }
                    Err(r) => self.refuse(now_ms, conn, r),
                }
            }
            ToRelay::Leave => {
                let Some(code) = in_room else {
                    return self.refuse(now_ms, conn, Refusal::NotInRoom);
                };
                if let Some(c) = self.conns.get_mut(&conn) {
                    c.room = None;
                    c.idle_since = Some(now_ms);
                }
                self.leave_room(now_ms, conn, &code);
            }
            ToRelay::Edit(edit) => {
                let Some(code) = in_room else {
                    return self.refuse(now_ms, conn, Refusal::NotInRoom);
                };
                let e = self.rooms.get_mut(&code).expect("the room exists");
                match e.room.edit(conn, edit) {
                    Ok(()) => self.broadcast(now_ms, &code),
                    Err(r) => self.refuse(now_ms, conn, r),
                }
            }
            ToRelay::Ready { ready, map_hash } => {
                let Some(code) = in_room else {
                    return self.refuse(now_ms, conn, Refusal::NotInRoom);
                };
                let e = self.rooms.get_mut(&code).expect("the room exists");
                match e.room.set_ready(conn, *ready, map_hash) {
                    Ok(()) => self.broadcast(now_ms, &code),
                    Err(r) => self.refuse(now_ms, conn, r),
                }
            }
            ToRelay::Chat { text } => self.on_chat(now_ms, conn, text),
            ToRelay::Start => {
                let Some(code) = in_room else {
                    return self.refuse(now_ms, conn, Refusal::NotInRoom);
                };
                let mut seed = [0u8; 4];
                self.entropy.fill(&mut seed);
                let mut params = self.limits.params;
                let e = self.rooms.get_mut(&code).expect("the room exists");
                let period = net_period_ms(e.room.setup().game_speed()).max(1) as u64;
                params.grace_ticks =
                    (self.limits.grace_ms / period).min(u64::from(u32::MAX)) as u32;
                match e.room.start(conn, u32::from_le_bytes(seed), params) {
                    Ok(started) => self.begin(now_ms, &code, started),
                    Err(r) => self.refuse(now_ms, conn, r),
                }
            }
            ToRelay::Rejoin {
                code,
                slot,
                ticket,
                build,
                rules_hash,
            } => {
                if in_room.is_some() {
                    return self.refuse(now_ms, conn, Refusal::InRoom);
                }
                let Some(code) = normalise_code(code).filter(|c| self.rooms.contains_key(c)) else {
                    return self.refuse(now_ms, conn, Refusal::NoRoom);
                };
                self.rejoin(now_ms, conn, &code, *slot, ticket, build, rules_hash);
            }
            ToRelay::Snapshot(part) => self.on_snapshot(now_ms, conn, in_room, part),
            ToRelay::CaughtUp => {
                let Some(g) = in_room
                    .and_then(|c| self.rooms.get_mut(&c))
                    .and_then(|e| e.game.as_mut())
                else {
                    return;
                };
                let Some(r) = g
                    .rejoin
                    .as_mut()
                    .filter(|r| r.conn == Some(conn) && !r.asked)
                else {
                    return;
                };
                r.asked = true;
                let (to, slot) = (r.donor_conn, r.slot);
                self.send(now_ms, to, relay_message(Body::Resume { slot }));
            }
            ToRelay::Signal {
                to,
                attempt,
                signal,
            } => self.on_signal(now_ms, conn, *to, *attempt, signal),
            ToRelay::Ping { stamp } => {
                self.send_lobby(now_ms, conn, &FromRelay::Pong { stamp: *stamp })
            }
        }
    }

    /// A rejoin of `slot` in room `code` by `conn` with `ticket`.
    #[allow(clippy::too_many_arguments)]
    fn rejoin(
        &mut self,
        now_ms: u64,
        conn: ConnId,
        code: &str,
        slot: u8,
        ticket: &Ticket,
        build: &str,
        rules_hash: &str,
    ) {
        let Some(g) = self.rooms.get(code).and_then(|e| e.game.as_ref()) else {
            return self.refuse(now_ms, conn, Refusal::NoGame);
        };
        if !g.human(slot) || g.tickets[usize::from(slot)] != *ticket {
            return self.refuse(now_ms, conn, Refusal::Ticket);
        }
        if build != g.started.begin.build {
            return self.refuse(now_ms, conn, Refusal::Mismatch(Field::Build));
        }
        if rules_hash != g.started.begin.rules_hash {
            return self.refuse(now_ms, conn, Refusal::Mismatch(Field::RulesHash));
        }
        if g.surrendered(slot) {
            return self.refuse(now_ms, conn, Refusal::Surrendered);
        }
        if g.rounds.iter().any(|r| r.slot == slot) {
            // Its drop is still being put together (N7).
            return self.send_lobby(now_ms, conn, &FromRelay::Refused(Refusal::Rejoining));
        }
        let mut replaced = Vec::new();
        match &g.rejoin {
            Some(r) if r.slot != slot => {
                return self.send_lobby(now_ms, conn, &FromRelay::Refused(Refusal::Rejoining));
            }
            Some(r) => replaced.extend(r.conn),
            None => {}
        }
        replaced.extend(g.conns[usize::from(slot)]);
        for old in replaced {
            self.close(now_ms, old, "replaced by a rejoin of the same slot");
        }
        // The room may be gone (nobody else was left) or a readmission may
        // still be on its way for the connection just replaced.
        let Some(e) = self.rooms.get_mut(code) else {
            return self.refuse(now_ms, conn, Refusal::NoGame);
        };
        let Some(g) = e.game.as_mut() else {
            return self.refuse(now_ms, conn, Refusal::NoGame);
        };
        if g.rejoin.is_some() {
            return self.send_lobby(now_ms, conn, &FromRelay::Refused(Refusal::Rejoining));
        }
        let Some((donor, donor_conn)) = g.live().next() else {
            return self.refuse(now_ms, conn, Refusal::NoGame);
        };
        let s = usize::from(slot);
        let mut begin = g.started.begin.clone();
        begin.you = slot;
        begin.ticket = g.tickets[s];
        let row = begin.rows.get(s).copied().unwrap_or(0);
        let name = begin.names.get(s).cloned().unwrap_or_default();
        g.rejoin = Some(Rejoin {
            slot,
            conn: Some(conn),
            donor,
            donor_conn,
            asked: false,
            marker: None,
            sent: 0,
        });
        let _ = e.room.rejoin(conn, row, &name);
        self.stats.rejoins += 1;
        self.enter(conn, code);
        self.send_lobby(now_ms, conn, &FromRelay::Rejoined(begin));
        self.send(
            now_ms,
            donor_conn,
            relay_message(Body::Serve { slot, on: true }),
        );
    }

    /// A part of the game from the serving peer of its room's rejoin: on to
    /// the rejoining connection. A part from anyone else, or after the
    /// rejoin ended, is dropped.
    fn on_snapshot(&mut self, now_ms: u64, conn: ConnId, code: Option<String>, part: &Part) {
        let max = self.limits.max_snapshot;
        let Some(g) = code
            .and_then(|c| self.rooms.get_mut(&c))
            .and_then(|e| e.game.as_mut())
        else {
            return;
        };
        let Some(r) = g.rejoin.as_mut().filter(|r| r.donor_conn == conn) else {
            return;
        };
        let Some(to) = r.conn else {
            return;
        };
        r.sent = r.sent.saturating_add(part.bytes.len());
        if r.sent > max {
            let (slot, donor) = (r.slot, r.donor_conn);
            g.rejoin = None;
            self.send(
                now_ms,
                donor,
                relay_message(Body::Serve { slot, on: false }),
            );
            return self.close(now_ms, to, "the game is too large to send");
        }
        self.stats.snapshot_bytes += part.bytes.len() as u64;
        self.send_lobby(now_ms, to, &FromRelay::Snapshot(part.clone()));
    }

    /// A WebRTC signal from `conn` for slot `to` of its game (N7): passed
    /// on between live persons only, at the signal rate.
    fn on_signal(&mut self, now_ms: u64, conn: ConnId, to: u8, attempt: u32, signal: &Signal) {
        let target = self
            .room_of(conn)
            .and_then(|c| self.rooms.get(&c))
            .and_then(|e| e.game.as_ref())
            .and_then(|g| {
                let from = g.slot_of(conn)?;
                let c = g.conns.get(usize::from(to)).copied().flatten()?;
                (from != to).then_some((from, c))
            });
        let Some((from, target)) = target.filter(|_| signal.plausible()) else {
            self.stats.signals_dropped += 1;
            return;
        };
        if !self.spend_signal(now_ms, conn) {
            self.stats.signals_dropped += 1;
            return;
        }
        if let Some(g) = self
            .room_of(conn)
            .and_then(|c| self.rooms.get_mut(&c))
            .and_then(|e| e.game.as_mut())
        {
            g.signalled.insert((from.min(to), from.max(to)));
        }
        self.stats.signals += 1;
        self.send_lobby(
            now_ms,
            target,
            &FromRelay::Signal {
                from,
                attempt,
                signal: signal.clone(),
            },
        );
    }

    /// Takes one signal's worth from `conn`'s signal bucket; false when it
    /// is too low.
    fn spend_signal(&mut self, now_ms: u64, conn: ConnId) -> bool {
        let cost = self.limits.signal_every_ms;
        let cap = u64::from(self.limits.signal_burst) * cost;
        let Some(c) = self.conns.get_mut(&conn) else {
            return false;
        };
        let ms = now_ms.saturating_sub(c.signal_refilled);
        c.signal_refilled = c.signal_refilled.max(now_ms);
        c.signal_tokens = c.signal_tokens.saturating_add(ms).min(cap);
        if c.signal_tokens < cost {
            return false;
        }
        c.signal_tokens -= cost;
        true
    }

    /// Ends the rounds of room `code`'s game that every asked peer has
    /// answered, and with `late` those past their deadline (N7): the frames
    /// put together go to every live peer and the rejoining one, then the
    /// drop counting them; a peer that did not answer in time is closed.
    fn finish_rounds(&mut self, now_ms: u64, code: &str, late: bool) {
        let Some(g) = self.rooms.get_mut(code).and_then(|e| e.game.as_mut()) else {
            return;
        };
        let mut sends = Vec::new();
        let mut closes = Vec::new();
        let mut i = 0;
        while i < g.rounds.len() {
            let r = &g.rounds[i];
            let due = r.waiting.is_empty() || (late && now_ms >= r.deadline);
            if !due {
                i += 1;
                continue;
            }
            let r = g.rounds.remove(i);
            for &s in r.waiting.keys() {
                closes.extend(g.conns[usize::from(s)]);
            }
            let kept = r.forwarded.saturating_add(r.merged.len() as u32);
            g.kept[usize::from(r.slot)] = kept;
            let mut out = glasnost_net::session::held_parts(RELAY, r.slot, r.forwarded, r.merged);
            out.push(relay_message(Body::Dropped { slot: r.slot, kept }));
            self.stats.drops += 1;
            let to: Vec<ConnId> = g
                .live()
                .map(|(_, c)| c)
                .filter(|c| !closes.contains(c))
                .chain(g.catching())
                .collect();
            for t in to {
                for b in &out {
                    sends.push((t, b.clone()));
                }
            }
        }
        for (t, b) in sends {
            self.send(now_ms, t, b);
        }
        for c in closes {
            self.stats.unanswered += 1;
            self.close(now_ms, c, "did not answer a drop in time");
        }
    }

    fn enter(&mut self, conn: ConnId, code: &str) {
        if let Some(c) = self.conns.get_mut(&conn) {
            c.room = Some(code.to_owned());
            c.idle_since = None;
        }
    }

    fn begin(&mut self, now_ms: u64, code: &str, started: Started) {
        let slots = started.begin.slots.len();
        let mut tickets = vec![[0u8; 16]; slots];
        for &(_, slot) in &started.persons {
            self.entropy.fill(&mut tickets[usize::from(slot)]);
        }
        let mut game = Game {
            conns: vec![None; slots],
            kept: vec![0; slots],
            started: started.clone(),
            tickets,
            surrendered: vec![false; slots],
            max_ran: 0,
            rejoin: None,
            markers: Vec::new(),
            signalled: BTreeSet::new(),
            rounds: Vec::new(),
        };
        for &(m, slot) in &started.persons {
            game.conns[usize::from(slot)] = Some(m);
        }
        let begins: Vec<(ConnId, lobby::Begin)> = started
            .persons
            .iter()
            .filter_map(|&(m, slot)| {
                let mut b = started.begin_for(m)?;
                b.ticket = game.tickets[usize::from(slot)];
                Some((m, b))
            })
            .collect();
        if let Some(e) = self.rooms.get_mut(code) {
            e.game = Some(game);
        }
        for (m, b) in begins {
            self.send_lobby(now_ms, m, &FromRelay::Begin(b));
        }
    }

    /// The slot whose rejoin room `code`'s game is serving, if any.
    pub fn rejoining(&self, code: &str) -> Option<u8> {
        let g = self.rooms.get(code)?.game.as_ref()?;
        g.rejoin.as_ref().map(|r| r.slot)
    }

    fn fresh_code(&mut self) -> Option<String> {
        for _ in 0..CODE_TRIES {
            let mut b = [0u8; CODE_LEN];
            self.entropy.fill(&mut b);
            let code = lobby::code_from_bytes(&b);
            if !self.rooms.contains_key(&code) {
                return Some(code);
            }
        }
        None
    }

    /// The clock at `now_ms`: closes the silent and idle connections and
    /// sends keepalives. Call it a few times a second.
    pub fn tick(&mut self, now_ms: u64) {
        let mut silent = Vec::new();
        let mut idle = Vec::new();
        let mut keep = Vec::new();
        for (&id, c) in &self.conns {
            if now_ms.saturating_sub(c.last_heard) >= self.limits.silent_ms {
                silent.push(id);
            } else if c
                .idle_since
                .is_some_and(|t| now_ms.saturating_sub(t) >= self.limits.idle_ms)
            {
                idle.push(id);
            } else if now_ms.saturating_sub(c.last_sent) >= self.limits.keepalive_ms {
                keep.push(id);
            }
        }
        for id in silent {
            self.stats.closed_silent += 1;
            self.close(now_ms, id, "silent for too long");
        }
        for id in idle {
            self.stats.closed_silent += 1;
            self.close(now_ms, id, "in no room for too long");
        }
        for id in keep {
            if let Some(c) = self.conns.get_mut(&id) {
                c.last_sent = now_ms;
                self.out.push(Out::Keepalive { to: id });
            }
        }
        let late: Vec<String> = self
            .rooms
            .iter()
            .filter(|(_, e)| {
                e.game
                    .as_ref()
                    .is_some_and(|g| g.rounds.iter().any(|r| now_ms >= r.deadline))
            })
            .map(|(c, _)| c.clone())
            .collect();
        for code in late {
            self.finish_rounds(now_ms, &code, true);
        }
    }
}
