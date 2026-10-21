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
//! (N1, `Session::drop_slot`).
//!
//! The limits and the protocol are our own design (docs/development.md,
//! *The relay and lobby (N2)*).

use std::collections::BTreeMap;

use glasnost_net::lobby::{
    self, CODE_LEN, FromRelay, MemberId, Refusal, Room, Started, ToRelay, normalise_code,
};
use glasnost_net::wire::{self, Body, Envelope, MAGIC as GAME_MAGIC, RELAY, WireError};
use glasnost_net::{Field, Params};

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
    /// The lockstep parameters of every game.
    pub params: Params,
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
            params: Params::default(),
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
}

/// A room's game: who plays each slot and what was forwarded.
#[derive(Clone, Debug)]
struct Game {
    /// Per slot: the connection of its person while it is live.
    conns: Vec<Option<ConnId>>,
    /// Per slot: its frames forwarded to every other live person, from
    /// frame 0 without a gap.
    kept: Vec<u32>,
}

impl Game {
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
            self.out.push(Out::Send { to, bytes });
        }
    }

    fn send_lobby(&mut self, now_ms: u64, to: ConnId, msg: &FromRelay) {
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
            if let Some(slot) = g.slot_of(conn) {
                g.conns[usize::from(slot)] = None;
                let kept = g.kept[usize::from(slot)];
                let to: Vec<ConnId> = g.live().map(|(_, c)| c).collect();
                let bytes = wire::encode(&Envelope {
                    from: RELAY,
                    body: Body::Dropped { slot, kept },
                });
                e.room.leave(conn);
                self.stats.drops += 1;
                for t in to {
                    self.send(now_ms, t, bytes.clone());
                }
            }
            let empty = self.rooms[code]
                .game
                .as_ref()
                .is_none_or(|g| g.live().next().is_none());
            if empty {
                self.rooms.remove(code);
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

    /// Bytes from `conn` at `now_ms`: one WebSocket message.
    pub fn receive(&mut self, now_ms: u64, conn: ConnId, bytes: &[u8]) {
        if !self.conns.contains_key(&conn) {
            return;
        }
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
            Body::Frames(f) => {
                let k = &mut game.kept[usize::from(slot)];
                if f.first <= *k {
                    *k = (*k).max(f.first.saturating_add(f.frames.len() as u32));
                }
            }
            Body::Start(_) => {}
        }
        let to: Vec<ConnId> = game
            .live()
            .filter(|&(s, _)| s != slot)
            .map(|(_, c)| c)
            .collect();
        self.stats.forwarded += 1;
        for t in to {
            self.send(now_ms, t, bytes.to_vec());
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
            ToRelay::Start => {
                let Some(code) = in_room else {
                    return self.refuse(now_ms, conn, Refusal::NotInRoom);
                };
                let mut seed = [0u8; 4];
                self.entropy.fill(&mut seed);
                let params = self.limits.params;
                let e = self.rooms.get_mut(&code).expect("the room exists");
                match e.room.start(conn, u32::from_le_bytes(seed), params) {
                    Ok(started) => self.begin(now_ms, &code, started),
                    Err(r) => self.refuse(now_ms, conn, r),
                }
            }
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
        let mut game = Game {
            conns: vec![None; slots],
            kept: vec![0; slots],
        };
        for &(m, slot) in &started.persons {
            game.conns[usize::from(slot)] = Some(m);
        }
        if let Some(e) = self.rooms.get_mut(code) {
            e.game = Some(game);
        }
        for &(m, _) in &started.persons {
            if let Some(b) = started.begin_for(m) {
                self.send_lobby(now_ms, m, &FromRelay::Begin(b));
            }
        }
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
    }
}
