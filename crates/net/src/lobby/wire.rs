//! The lobby's messages between a client and the relay.
//!
//! A lobby message is a three-byte header, then the body in postcard:
//!
//! | Bytes | Value |
//! |---|---|
//! | 0, 1 | the magic `G`, `L` |
//! | 2 | [`WIRE_VERSION`], shared with the lockstep messages |
//! | 3 … | a [`ToRelay`] (client to relay) or a [`FromRelay`] (relay to client) |
//!
//! On the same connection the client also sends and receives the lockstep
//! messages of [`crate::wire`] (magic `GN`) once the game has begun; the
//! relay tells the two apart by the magic. A member who dropped from a game
//! comes back with [`ToRelay::Rejoin`] and its [`Begin::ticket`]; a live
//! peer sends it the game in [`ToRelay::Snapshot`] parts, which the relay
//! passes on as [`FromRelay::Snapshot`] (N5). During a game a peer may
//! open WebRTC data channels to the others: the relay passes the signals
//! that set one up between the game's live peers ([`ToRelay::Signal`],
//! [`FromRelay::Signal`]) and answers pings ([`ToRelay::Ping`]) so a
//! client can tell its path through the relay from a direct one (N7). This
//! is our own protocol.

use glasnost_ui::setup::{Problem, Setup, SlotKind};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use super::MemberId;
use crate::start::{Field, Params, Slot, StartInfo};
use crate::wire::WIRE_VERSION;
use glasnost_sim::world::GameSetup;

/// The first two bytes of every lobby message.
// nocite: our own netcode
pub const MAGIC: [u8; 2] = *b"GL";

/// Length of the header before the body.
// nocite: our own netcode (magic, version)
pub const HEADER_LEN: usize = 3;

/// The map of a room: the content hash every member must hold the same map
/// under (`game::session::content_hash` in the client), and what the room
/// shows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapChoice {
    /// The map's content hash.
    pub hash: String,
    /// The map's name as the host's client shows it.
    pub label: String,
    /// Its start positions (S-0009 claim 23), as the host's client counts
    /// them.
    pub starts: u32,
}

/// What the room's rules offer, as the host's client reads them from its
/// rules (every member has the same rules hash, so the same values).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offered {
    /// How many countries the setup offers (S-0412 claim 15).
    pub countries: u8,
    /// The credits slider's smallest and largest value (S-0412 Import
    /// mapping).
    pub credits: (i32, i32),
    /// The unit-count slider's smallest and largest value.
    pub units: (i32, i32),
}

/// A change to the room's setup.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Edit {
    /// The host chooses the map.
    Map(MapChoice),
    /// The host opens a row for a person ([`SlotKind::Human`]), gives it to
    /// a computer player or closes it. A row a person has taken is not
    /// changed this way.
    Row {
        /// The row.
        row: u8,
        /// Its new kind.
        kind: SlotKind,
    },
    /// A person sets the choices of their own row, or the host those of a
    /// computer row. `None` is "random" (start, country, colour) or "no
    /// team".
    Choices {
        /// The row.
        row: u8,
        /// An index into the offered countries.
        country: Option<u8>,
        /// A colour slot 0 … 7.
        colour: Option<u8>,
        /// A team 1 … 4.
        team: Option<u8>,
        /// A start position, 0-based.
        start: Option<u32>,
    },
    /// The host sets the game-wide values.
    Game {
        /// The credits setting.
        credits: i32,
        /// The unit-count setting.
        units: i32,
        /// Short game, superweapons, crates, build off ally, redeploy.
        flags: [bool; 5],
        /// The speed control's position 0 … 6.
        speed: u8,
    },
}

/// A client's message to the relay.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum ToRelay {
    /// Create a room and become its host, on the first person's row of
    /// `setup`.
    Create {
        /// The person's name, as the room shows it.
        name: String,
        /// The client's build.
        build: String,
        /// Its `sim_rules_hash`.
        rules_hash: String,
        /// What its rules offer.
        offered: Offered,
        /// The map.
        map: MapChoice,
        /// The first setup (the client's defaults).
        setup: Setup,
    },
    /// Ask what a room is, before joining it.
    Peek {
        /// The room code.
        code: String,
    },
    /// Join a room on its first open person's row.
    Join {
        /// The room code.
        code: String,
        /// The person's name.
        name: String,
        /// The client's build.
        build: String,
        /// Its `sim_rules_hash`.
        rules_hash: String,
        /// The content hash of its copy of the room's map.
        map_hash: String,
    },
    /// Leave the room (before the start; after it, the slot is dropped).
    Leave,
    /// Change the setup.
    Edit(Edit),
    /// Say ready, or not, for the current setup, with the content hash of
    /// the client's copy of the room's map.
    Ready {
        /// Ready or not.
        ready: bool,
        /// The client's map hash.
        map_hash: String,
    },
    /// The host starts the game.
    Start,
    /// A chat line to everyone in the room, before or during its game. It
    /// never reaches a simulation.
    Chat {
        /// What was typed.
        text: String,
    },
    /// Come back to a game after a drop, on the slot of `ticket` (N5). The
    /// relay answers [`FromRelay::Rejoined`] and has a live peer send the
    /// game.
    Rejoin {
        /// The room code.
        code: String,
        /// The slot.
        slot: u8,
        /// The slot's ticket, from the member's [`Begin`].
        ticket: Ticket,
        /// The client's build.
        build: String,
        /// Its `sim_rules_hash`.
        rules_hash: String,
    },
    /// One part of the game for a rejoining peer, from the peer the relay
    /// asked ([`crate::wire::Body::Serve`]); parts go in order, from 0.
    Snapshot(Part),
    /// The rejoining peer has run every tick it has the frames for: it
    /// may be readmitted.
    CaughtUp,
    /// A WebRTC signal for the peer of slot `to` in the client's game (N7).
    /// The relay passes it on, from the sender's slot, only between the
    /// game's live peers and only at a limited rate; one it does not pass
    /// on is dropped without an answer.
    Signal {
        /// The receiving peer's slot.
        to: u8,
        /// The sender's attempt at a link to that peer: a signal of another
        /// attempt starts the link afresh.
        attempt: u32,
        /// The signal.
        signal: Signal,
    },
    /// The relay answers with [`FromRelay::Pong`] carrying the same stamp
    /// (N7: the round trip through the relay).
    Ping {
        /// The client's clock, in milliseconds (wrapping).
        stamp: u32,
    },
}

/// A WebRTC signal (N7): the session description of an offer or an answer,
/// or an ICE candidate as the browsers write it in JSON
/// (`RTCIceCandidateInit`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Signal {
    /// An offer's SDP.
    Offer(String),
    /// An answer's SDP.
    Answer(String),
    /// An ICE candidate, in JSON.
    Candidate(String),
    /// This end makes no direct links (relay-only): its answer to an offer,
    /// so that the offering end stops offering to it until its slot plays
    /// again (a rejoined process may have the links on). It carries no
    /// text (M9 N8c).
    Decline,
}

/// The longest signal, in bytes: an SDP of one data channel and its ICE
/// lines is a few kilobytes.
// nocite: our own netcode
pub const MAX_SIGNAL: usize = 16 * 1024;

impl Signal {
    /// The signal's text.
    pub fn text(&self) -> &str {
        match self {
            Signal::Offer(t) | Signal::Answer(t) | Signal::Candidate(t) => t,
            Signal::Decline => "",
        }
    }

    /// Whether the signal looks like one: at most [`MAX_SIGNAL`] bytes of
    /// printable ASCII and line ends, a description starting with the SDP
    /// version line, a candidate a JSON object; a decline has nothing to
    /// check. Both the relay and the receiving client check it; the WebRTC
    /// stack reads the rest.
    pub fn plausible(&self) -> bool {
        if *self == Signal::Decline {
            return true;
        }
        let t = self.text();
        let ascii = t
            .bytes()
            .all(|b| (0x20..0x7f).contains(&b) || b == b'\r' || b == b'\n');
        let shape = match self {
            Signal::Offer(t) | Signal::Answer(t) => t.starts_with("v=0"), // nocite: SDP's version line (RFC 8866)
            Signal::Candidate(t) => t.starts_with('{') && t.ends_with('}'),
            Signal::Decline => true,
        };
        !t.is_empty() && t.len() <= MAX_SIGNAL && ascii && shape
    }
}

/// A slot's secret, given only to its member in [`Begin`]: the proof that
/// a rejoin is that member's (N5).
pub type Ticket = [u8; 16];

/// One part of a rejoining peer's game ([`crate::rejoin`]).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Part {
    /// Its number, from 0.
    pub part: u32,
    /// How many parts there are.
    pub parts: u32,
    /// Its bytes.
    pub bytes: Vec<u8>,
}

/// A value that is out of range or malformed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Value {
    /// A person's name.
    Name,
    /// A build, hash or label.
    Text,
    /// A row number.
    Row,
    /// A country index.
    Country,
    /// A colour slot.
    Colour,
    /// A team.
    Team,
    /// A start position.
    Start,
    /// The credits setting.
    Credits,
    /// The unit-count setting.
    Units,
    /// The speed control.
    Speed,
    /// The map's start count.
    Starts,
    /// The offered values.
    Offered,
}

impl Value {
    /// The value's name, for messages.
    pub fn name(self) -> &'static str {
        match self {
            Value::Name => "name",
            Value::Text => "text",
            Value::Row => "row",
            Value::Country => "country",
            Value::Colour => "colour",
            Value::Team => "team",
            Value::Start => "start position",
            Value::Credits => "credits",
            Value::Units => "unit count",
            Value::Speed => "game speed",
            Value::Starts => "start count",
            Value::Offered => "offered values",
        }
    }
}

/// Why the relay did not do what a client asked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Refusal {
    /// A value differs from the room's: the protocol version, the build,
    /// the rules hash or the map hash.
    Mismatch(Field),
    /// No room has that code.
    NoRoom,
    /// The room has no open person's row.
    RoomFull,
    /// The relay holds as many rooms as it may.
    TooManyRooms,
    /// The room's game has begun.
    Playing,
    /// The client is in no room.
    NotInRoom,
    /// The client is in a room already.
    InRoom,
    /// Only the host may do that.
    NotHost,
    /// That row is not the client's to change.
    NotYours(u8),
    /// A person has taken that row.
    Seated(u8),
    /// That row is not offered on this map (S-0412 claim 15).
    NoRow(u8),
    /// A value is out of range or malformed.
    Bad(Value),
    /// The setup has a problem (two players share a colour or a start, too
    /// many or too few players, an unknown country, no map).
    Setup(Problem),
    /// A person's row has nobody on it.
    Open(u8),
    /// The person on that row is not ready.
    NotReady(u8),
    /// Chat lines came faster than the relay passes them on; this one was
    /// not sent.
    ChatTooFast,
    /// A rejoin's ticket is not that slot's (N5).
    Ticket,
    /// A rejoin came after the slot's grace period: its house surrendered.
    Surrendered,
    /// Another person is rejoining the game; try again in a moment.
    Rejoining,
    /// A rejoin's room has no game, or nobody left in it to send the game.
    NoGame,
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refusal::Mismatch(field) => write!(f, "the {field} differs from the room's"),
            Refusal::NoRoom => f.write_str("no room has that code"),
            Refusal::RoomFull => f.write_str("the room is full"),
            Refusal::TooManyRooms => f.write_str("the relay has no room for another room"),
            Refusal::Playing => f.write_str("the room's game has begun"),
            Refusal::NotInRoom => f.write_str("not in a room"),
            Refusal::InRoom => f.write_str("already in a room"),
            Refusal::NotHost => f.write_str("only the host may do that"),
            Refusal::NotYours(r) => write!(f, "row {} is not yours", r + 1),
            Refusal::Seated(r) => write!(f, "a person is on row {}", r + 1),
            Refusal::NoRow(r) => write!(f, "this map has no row {}", r + 1),
            Refusal::Bad(v) => write!(f, "bad {}", v.name()),
            Refusal::Setup(p) => write!(f, "{p}"),
            Refusal::Open(r) => write!(f, "nobody is on row {}", r + 1),
            Refusal::NotReady(r) => write!(f, "the person on row {} is not ready", r + 1),
            Refusal::ChatTooFast => f.write_str("chat lines too fast; that one was not sent"),
            Refusal::Ticket => f.write_str("that slot is not yours to rejoin"),
            Refusal::Surrendered => f.write_str(
                "too late to rejoin: the grace period ran out and the house surrendered",
            ),
            Refusal::Rejoining => {
                f.write_str("another player is rejoining the game; try again in a moment")
            }
            Refusal::NoGame => f.write_str("that room has no game to rejoin"),
        }
    }
}

/// A member as the room shows it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberView {
    /// The member.
    pub id: MemberId,
    /// The row it has taken.
    pub row: u8,
    /// Its name.
    pub name: String,
    /// Whether it is ready for the current setup.
    pub ready: bool,
}

/// The room as a member sees it, sent after every change.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomView {
    /// The room code.
    pub code: String,
    /// The member this view is for.
    pub you: MemberId,
    /// The host.
    pub host: MemberId,
    /// The room's build.
    pub build: String,
    /// The room's rules hash.
    pub rules_hash: String,
    /// What the room's rules offer.
    pub offered: Offered,
    /// The map.
    pub map: MapChoice,
    /// The setup; its `map` index is 0 (the client puts in its own index of
    /// `map`).
    pub setup: Setup,
    /// The members, by row.
    pub members: Vec<MemberView>,
}

/// What a room is, for a client deciding to join it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomInfo {
    /// The room code.
    pub code: String,
    /// The room's build.
    pub build: String,
    /// The room's rules hash.
    pub rules_hash: String,
    /// The map.
    pub map: MapChoice,
    /// The persons in the room.
    pub persons: u8,
    /// Open persons' rows.
    pub open: u8,
    /// Whether its game has begun.
    pub playing: bool,
}

/// The start of a room's game: the same values for every member but `you`.
/// Each member builds its game setup from `setup` with its own rules and
/// map, then its [`StartInfo`] with [`Begin::start_info`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Begin {
    /// The build.
    pub build: String,
    /// The rules hash.
    pub rules_hash: String,
    /// The map.
    pub map: MapChoice,
    /// The game seed (the relay's host randomness).
    pub seed: u32,
    /// The setup as it was started, its colours on "random" given their
    /// slots ([`Setup::resolve_colours`]); its `map` index is 0.
    pub setup: Setup,
    /// The slot table, in house order ([`Setup::house_order`]).
    pub slots: Vec<Slot>,
    /// The setup row of each slot.
    pub rows: Vec<u8>,
    /// The name of each slot's player, for the client to show (the houses
    /// themselves are named by [`Setup::lobby`]).
    pub names: Vec<String>,
    /// The lockstep parameters.
    pub params: Params,
    /// The receiving member's slot.
    pub you: u8,
    /// The receiving member's ticket for a rejoin of its slot (N5); only
    /// that member is sent it.
    pub ticket: Ticket,
}

impl Begin {
    /// The setup with the client's own index of the room's map.
    pub fn local_setup(&self, map_index: usize) -> Setup {
        let mut s = self.setup.clone();
        s.map = map_index;
        s
    }

    /// The start values of the lockstep session for `game`, the game setup
    /// the member built from [`Begin::setup`]. A setup whose slot table or
    /// seed is not the room's is refused on that field.
    pub fn start_info(&self, game: &GameSetup) -> Result<StartInfo, Field> {
        let info = StartInfo::new(
            &self.build,
            &self.rules_hash,
            &self.map.hash,
            game,
            self.params,
        );
        if info.slots != self.slots {
            return Err(Field::Slots);
        }
        if info.seed != self.seed {
            return Err(Field::Seed);
        }
        Ok(info)
    }
}

/// The relay's message to a client.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum FromRelay {
    /// The room after a change (to every member).
    Room(RoomView),
    /// The answer to [`ToRelay::Peek`].
    Info(RoomInfo),
    /// A request was refused.
    Refused(Refusal),
    /// The game begins (to every member).
    Begin(Begin),
    /// The relay closes the connection, and why.
    Closing(String),
    /// A member's chat line (to every member, the sender too).
    Chat(ChatLine),
    /// The answer to [`ToRelay::Rejoin`]: the game's start values, as at
    /// its start (N5). The game itself follows in [`FromRelay::Snapshot`].
    Rejoined(Begin),
    /// One part of the game, for a rejoining peer.
    Snapshot(Part),
    /// A WebRTC signal from the peer of slot `from` of the game (N7).
    Signal {
        /// The sending peer's slot.
        from: u8,
        /// Its attempt at the link ([`ToRelay::Signal`]).
        attempt: u32,
        /// The signal.
        signal: Signal,
    },
    /// The answer to [`ToRelay::Ping`].
    Pong {
        /// The stamp the client sent.
        stamp: u32,
    },
}

/// A chat line as the relay passes it on.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatLine {
    /// Who said it.
    pub from: MemberId,
    /// Their row when they said it.
    pub row: u8,
    /// Their name.
    pub name: String,
    /// The line, cleaned ([`super::room::clean_chat`]).
    pub text: String,
}

/// Why bytes are not a lobby message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// Shorter than the header.
    Short,
    /// Not a lobby message.
    Magic,
    /// Another version of the protocol.
    Version(u8),
    /// The body cannot be read.
    Body(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Short => f.write_str("message shorter than its header"),
            Error::Magic => f.write_str("not a Glasnost lobby message"),
            Error::Version(v) => write!(f, "wire version {v}, the relay speaks {WIRE_VERSION}"),
            Error::Body(e) => write!(f, "unreadable message body: {e}"),
        }
    }
}

/// Whether `bytes` start as a lobby message.
pub fn is_lobby(bytes: &[u8]) -> bool {
    bytes.get(..2) == Some(&MAGIC[..])
}

/// Writes a lobby message.
pub fn encode<T: Serialize>(msg: &T) -> Vec<u8> {
    let mut out = Vec::with_capacity(HEADER_LEN + 64); // nocite: our own netcode (a first guess at a message's size)
    out.extend_from_slice(&MAGIC);
    out.push(WIRE_VERSION);
    postcard::to_extend(msg, out).expect("a lobby message serializes")
}

/// Reads a lobby message. Never panics on hostile bytes.
pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, Error> {
    if bytes.len() < HEADER_LEN {
        return Err(Error::Short);
    }
    if !is_lobby(bytes) {
        return Err(Error::Magic);
    }
    if bytes[2] != WIRE_VERSION {
        return Err(Error::Version(bytes[2]));
    }
    let (msg, rest) =
        postcard::take_from_bytes(&bytes[HEADER_LEN..]).map_err(|e| Error::Body(e.to_string()))?;
    if !rest.is_empty() {
        return Err(Error::Body(format!("{} trailing bytes", rest.len())));
    }
    Ok(msg)
}
