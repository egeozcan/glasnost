//! The client's network side (M9 N3): the multiplayer screen, a room on a
//! relay, the lockstep-paced game and chat. Our own netcode; no spec
//! applies beyond the setup rules the room shares with a skirmish.
//!
//! One [`Link`] carries both protocols: the lobby's messages (magic `GL`,
//! `glasnost_net::lobby::wire`) are this module's, the lockstep's (magic
//! `GN`) go to the game's [`Lockstep`] session, which has no I/O of its
//! own. Lockstep bytes that arrive while the map still loads are kept and
//! handed over once the session exists.
//!
//! In a network game the person's orders go into `Play::outbox`; the
//! client hands them to the session, and the world changes only when the
//! session runs a tick (`Session::step_by`). The pacer says when a tick is
//! due, the session whether it can run.
//!
//! Every network game keeps desync evidence (`glasnost_net::desync`, M9
//! N4): at a mismatch the client makes a report, stores it through its
//! [`SaveReport`](super::report::SaveReport) (a file natively, a download
//! in the browser), tells the other peers its digest and says where it
//! went.
//!
//! A person whose link goes mid-game can come back (M9 N5): the client
//! keeps the game's relay, room, slot and ticket in the settings store
//! while the game runs, so that the multiplayer screen offers
//! **Reconnect**, also after a restart. A rejoin loads the map as at the
//! start, takes the game a live peer sends ([`glasnost_net::rejoin`]),
//! puts it in place of the fresh one, runs the ticks it missed as fast as
//! it can and is readmitted. A peer the relay asks sends the game in
//! parts, one every 25 ms, while it plays on.

use std::collections::{BTreeSet, VecDeque};

use glasnost_net::desync::{Bundle, Evidence, Perturb};
use glasnost_net::lobby::wire::{self as lobby_wire};
use glasnost_net::lobby::{
    Begin, CHAT_CHARS, Edit, FromRelay, MapChoice, Offered, Part, Refusal, RoomView, ToRelay,
    normalise_code,
};
use glasnost_net::rejoin::{self, Assembler, Transfer};
use glasnost_net::start::SlotKind as StartKind;
use glasnost_net::{
    Config, Event as NetEvent, Field, Paths, Phase, Session as Lockstep, StartInfo, Tick, Via,
};
use glasnost_sim::world::HouseId;
use glasnost_ui::chat::Line;
use glasnost_ui::draw::{DrawList, colour};
use glasnost_ui::geom::{Point, Rect};
use glasnost_ui::netmenu::{NetField, NetHit};
use glasnost_ui::setup::{MAX_SLOTS, MapEntry, RoomFrame, Setup, SetupHit, SlotKind};
use glasnost_ui::shell::Screen;

use super::{Client, Job};
use crate::game::direct::{Direct, Ice, Mesh, PairStats};
use crate::game::link::{Link, LinkState};
use crate::mapview::live::net_period_ms;
use crate::settings::Rejoin;

/// One other person's pair, as [`Client::direct_stats`] gives it (N7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PairReport {
    /// The other person's slot.
    pub slot: u8,
    /// Whether this client makes direct links at all.
    pub enabled: bool,
    /// Whether the pair's link is open now.
    pub open: bool,
    /// The link's numbers.
    pub stats: PairStats,
    /// How the slot's frames came.
    pub paths: Option<Paths>,
    /// The session's round-trip estimate to the slot, in milliseconds.
    pub rtt_ms: Option<u32>,
}

/// The build a client names in a room and in the lockstep handshake.
pub const BUILD: &str = glasnost_net::BUILD;

/// The build a test client names ([`Client::set_test`]): not [`BUILD`], so
/// a room or a game of ordinary clients refuses it.
pub fn test_build() -> String {
    format!("{BUILD} (test)")
}

/// The hex digits of a report's digest a notice shows.
// nocite: our own netcode
const DIGEST_SHOWN: usize = 12;

/// How long the lockstep waits for a peer's frame before the game names
/// whom it waits for.
// nocite: our own netcode
const STALL_MS: u64 = 1_000;

/// The chat lines and notices kept.
// nocite: our own layout
const CHAT_KEPT: usize = 100;

/// How long a chat line stays over the battlefield.
// nocite: our own layout
const CHAT_SHOWN_MS: i64 = 10_000;

/// The lockstep bytes kept while the map loads (and, for a rejoin, until
/// the game sent has come): a rejoin that needs more fails.
// nocite: our own netcode (minutes of a full game's frames)
const EARLY_BYTES: usize = 16 * 1024 * 1024;

/// The most ticks a rejoining peer runs in one update while it catches up.
// nocite: our own netcode (far faster than any game speed at 60 updates a second)
const CATCH_UP_TICKS: u32 = 10;

/// How often a serving peer sends the next part of the game, in
/// milliseconds: some 40 parts a second, well inside the relay's rate.
// nocite: our own netcode
const PART_EVERY_MS: u64 = 25;

/// How many ticks ahead of the slowest peer this one may run before its
/// pacer stretches the tick period by half.
// nocite: our own netcode (Session::ahead's slack)
const AHEAD_STRETCH: i64 = 1;

/// The checkpoint hashes kept for [`Client::take_checkpoints`] until taken;
/// older ones go first.
// nocite: our own netcode (a few minutes of checkpoints at any game speed)
const CHECKPOINTS_KEPT: usize = 4096;

/// How often a client in a game pings the relay, in milliseconds (M9 N7:
/// the relay path's round trip, beside the direct links').
// nocite: our own netcode (once a second, well inside the relay's message rate)
const RELAY_PING_MS: u64 = 1_000;

/// The shortest wait between two updates of a network game by the pump
/// (fixed after the M9 exit run), in milliseconds: while the game waits
/// for frames, or catches up after a rejoin, the pump does not spin.
// nocite: our own netcode (a fraction of the fastest network period)
const WAKE_MIN_MS: u64 = 4;

/// The wait between two updates by the pump on the network screens and
/// while a game loads, in milliseconds.
// nocite: our own netcode (the room and the start answer within a few frames' time)
const WAKE_LINK_MS: u64 = 50;

/// What the person asked the relay for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Intent {
    /// Create a room.
    Host,
    /// Join the room of this code.
    Join(String),
    /// Rejoin this game (N5).
    Rejoin(Rejoin),
}

/// A connection to a relay and what came of it.
pub(super) struct Net {
    link: Box<dyn Link>,
    pub(super) intent: Intent,
    /// The room, once in one.
    pub(super) room: Option<RoomView>,
    /// Chat lines and notices with their host time of arrival.
    pub(super) chat: Vec<(i64, Line)>,
    /// The chat line being typed.
    pub(super) input: String,
    /// Typing goes to the chat (the room's chat field has the focus; in
    /// the game, the chat line is open).
    pub(super) typing: bool,
    /// A line for the room screen (the last refusal).
    pub(super) note: Option<(String, bool)>,
    /// The room's start, once it began.
    pub(super) begin: Option<Begin>,
    /// The game's lockstep session.
    pub(super) lockstep: Option<Lockstep>,
    /// The game's desync evidence, kept from the session's start.
    pub(super) evidence: Option<Evidence>,
    /// The first desync's tick, once one is noticed.
    desynced: Option<u32>,
    early: Vec<Vec<u8>>,
    /// The bytes in `early`.
    early_bytes: usize,
    /// Lockstep bytes were not kept for want of room.
    early_lost: bool,
    /// The game sent for a rejoin, while its parts come (N5).
    rejoin: Option<Assembler>,
    /// The game sent for a rejoin, once all of it came and until the
    /// session starts from it.
    transfer: Option<Vec<u8>>,
    /// The rejoining peer has told the relay it caught up.
    caught_up_sent: bool,
    /// The parts of the game this peer sends rejoining peers, encoded,
    /// with their slots.
    serving: VecDeque<(u8, Vec<u8>)>,
    /// When the last part went.
    last_part: Option<u64>,
    /// The rejoining peers' slots whose game is still to be made.
    to_serve: BTreeSet<u8>,
    /// Bytes of game this peer sent rejoining peers, and the bytes of the
    /// game it was sent (counts for the local lane).
    sent_bytes: u64,
    received_bytes: Option<usize>,
    stalled_since: Option<u64>,
    last_tick: Option<u64>,
    /// The game's checkpoints as this peer ran them (tick, its state hash),
    /// until taken (M9 N6, the soak's per-checkpoint log).
    checkpoints: VecDeque<(u32, glasnost_net::wire::Hash)>,
    /// Whom the game waits for, once it has waited [`STALL_MS`].
    pub(super) waiting: Option<String>,
    /// Why the game cannot start (a handshake refusal).
    pub(super) refused: Option<String>,
    /// Why the link closed, once it did.
    pub(super) closed: Option<String>,
    /// How this client makes direct links in its game (M9 N7).
    direct: Option<Direct>,
    /// The direct links, once the game's session runs.
    mesh: Option<Mesh>,
    /// The relay's round trip, smoothed, and its range, in milliseconds.
    relay_rtt: Option<u32>,
    relay_rtt_range: Option<(u32, u32)>,
    next_relay_ping: u64,
    /// Slots whose two copies of a frame differed (each told once).
    conflicted: BTreeSet<u8>,
}

impl Net {
    fn new(link: Box<dyn Link>, intent: Intent) -> Self {
        Self {
            link,
            intent,
            room: None,
            chat: Vec::new(),
            input: String::new(),
            typing: false,
            note: None,
            begin: None,
            lockstep: None,
            evidence: None,
            desynced: None,
            early: Vec::new(),
            early_bytes: 0,
            early_lost: false,
            rejoin: None,
            transfer: None,
            caught_up_sent: false,
            serving: VecDeque::new(),
            last_part: None,
            to_serve: BTreeSet::new(),
            sent_bytes: 0,
            received_bytes: None,
            stalled_since: None,
            last_tick: None,
            checkpoints: VecDeque::new(),
            waiting: None,
            refused: None,
            closed: None,
            direct: None,
            mesh: None,
            relay_rtt: None,
            relay_rtt_range: None,
            next_relay_ping: 0,
            conflicted: BTreeSet::new(),
        }
    }

    fn send(&mut self, m: &ToRelay) {
        self.link.send(lobby_wire::encode(m));
    }

    /// Takes what arrived: the lobby's messages are returned, the
    /// lockstep's go to the session (or are kept until it exists).
    fn receive(&mut self, now: u64) -> Vec<FromRelay> {
        let mut out = Vec::new();
        while let Some(bytes) = self.link.recv() {
            if lobby_wire::is_lobby(&bytes) {
                match lobby_wire::decode::<FromRelay>(&bytes) {
                    Ok(m) => out.push(m),
                    Err(e) => log::warn!("a lobby message the client cannot read: {e:?}"),
                }
            } else if let Some(ls) = self.lockstep.as_mut() {
                ls.receive(now, &bytes);
            } else if self.begin.is_some() {
                if self.early_bytes + bytes.len() <= EARLY_BYTES {
                    self.early_bytes += bytes.len();
                    self.early.push(bytes);
                } else {
                    self.early_lost = true;
                }
            }
        }
        out
    }

    /// What a rejoin is doing, while this client is not yet readmitted to
    /// its game (N5).
    fn rejoin_banner(&self) -> Option<String> {
        if !matches!(self.intent, Intent::Rejoin(_)) || self.begin.is_none() {
            return None;
        }
        match (&self.lockstep, &self.rejoin) {
            (Some(ls), _) if ls.catching_up() => {
                Some(format!("Catching up with the game (tick {})...", ls.ran()))
            }
            (Some(_), _) => None,
            (None, Some(asm)) => Some(match asm.progress() {
                (k, Some(n)) => format!("Receiving the game ({k} of {n} parts)..."),
                (_, None) => "Waiting for the game...".into(),
            }),
            (None, None) => Some("Loading the game...".into()),
        }
    }

    /// Sends the next part of the game to a rejoining peer when one is due.
    fn send_parts(&mut self, now: u64) {
        let serving = self.lockstep.as_ref().map(Lockstep::serving);
        // A rejoin the relay called off: its parts go no more.
        self.serving
            .retain(|(s, _)| serving.as_ref().is_some_and(|v| v.contains(s)));
        if self.serving.is_empty()
            || self
                .last_part
                .is_some_and(|t| now.saturating_sub(t) < PART_EVERY_MS)
        {
            return;
        }
        if let Some((_, bytes)) = self.serving.pop_front() {
            self.link.send(bytes);
            self.last_part = Some(now);
        }
    }

    /// Sends what the lockstep session has to send: everything to the
    /// relay, and this peer's frames also on its open direct links (N7).
    fn flush(&mut self, now: u64) {
        if let Some(ls) = self.lockstep.as_mut() {
            for b in ls.poll(now) {
                if let Some(m) = self.mesh.as_mut() {
                    m.send_frames(ls, &b);
                }
                self.link.send(b);
            }
        }
    }

    /// The direct links (N7): set up once the session exists, then their
    /// frames go to the session (which takes only the link's own slot's
    /// frames) and their signals to the relay. A game pings the relay too.
    fn pump_direct(&mut self, now: u64) {
        let Some(ls) = self.lockstep.as_mut() else {
            return;
        };
        let mesh = self.mesh.get_or_insert_with(|| {
            let persons: Vec<u8> = ls
                .info()
                .slots
                .iter()
                .enumerate()
                .filter(|(_, s)| s.kind == StartKind::Human)
                .map(|(i, _)| i as u8)
                .collect();
            let direct = self.direct.clone().unwrap_or_else(Direct::off);
            if direct.open.is_none() {
                log::info!("direct links: off; the relay carries every message");
            }
            Mesh::new(ls.me(), &persons, direct)
        });
        for (slot, bytes) in mesh.update(now, ls) {
            ls.receive_via(now, &bytes, Via::Direct(slot));
        }
        for m in mesh.take_signals() {
            self.link.send(lobby_wire::encode(&m));
        }
        if now >= self.next_relay_ping {
            self.next_relay_ping = now + RELAY_PING_MS;
            self.send(&ToRelay::Ping { stamp: now as u32 });
        }
    }

    /// The relay's answer to a ping.
    fn pong(&mut self, now: u64, stamp: u32) {
        let rtt = (now as u32).wrapping_sub(stamp);
        // An answer older than a minute is not a round trip of this game.
        if rtt > 60_000 {
            return; // nocite: our own netcode (a minute)
        }
        self.relay_rtt = Some(self.relay_rtt.map_or(rtt, |old| (old * 7 + rtt) / 8)); // nocite: our own netcode (the usual one-eighth smoothing)
        self.relay_rtt_range = Some(
            self.relay_rtt_range
                .map_or((rtt, rtt), |(lo, hi)| (lo.min(rtt), hi.max(rtt))),
        );
    }

    /// Keeps a tick's checkpoint hash, if it has one.
    fn checkpoint(checkpoints: &mut VecDeque<(u32, glasnost_net::wire::Hash)>, t: &Tick) {
        if let Some(h) = t.hash {
            if checkpoints.len() >= CHECKPOINTS_KEPT {
                checkpoints.pop_front();
            }
            checkpoints.push_back((t.tick, h));
        }
    }

    fn line(&mut self, now: i64, who: &str, text: &str, notice: bool) {
        self.chat.push((
            now,
            Line {
                who: who.into(),
                text: text.into(),
                notice,
            },
        ));
        if self.chat.len() > CHAT_KEPT {
            let extra = self.chat.len() - CHAT_KEPT;
            self.chat.drain(..extra);
        }
    }

    /// The name of a game slot's player.
    fn slot_name(&self, slot: u8) -> String {
        self.begin
            .as_ref()
            .and_then(|b| b.names.get(usize::from(slot)))
            .cloned()
            .unwrap_or_else(|| format!("slot {}", slot + 1))
    }

    /// The room's code: the room's, or the rejoined game's.
    fn code(&self) -> Option<String> {
        match (&self.room, &self.intent) {
            (Some(v), _) => Some(v.code.clone()),
            (None, Intent::Rejoin(r)) => Some(r.code.clone()),
            _ => None,
        }
    }

    /// The member of this client in the room.
    fn me(&self) -> Option<&glasnost_net::lobby::MemberView> {
        let v = self.room.as_ref()?;
        v.members.iter().find(|m| m.id == v.you)
    }

    /// Closes the link; the relay sees the connection go.
    pub(super) fn close(&mut self) {
        self.link.close();
    }

    /// Whether the link is still opening.
    pub(super) fn connecting(&self) -> bool {
        self.link.state() == LinkState::Connecting
    }

    /// The lines of the chat log, oldest first.
    pub(super) fn lines(&self) -> Vec<Line> {
        self.chat.iter().map(|(_, l)| l.clone()).collect()
    }
}

/// The room's map as the room screen lists it: the one map of the room.
fn room_maps(v: &RoomView, have: bool) -> Vec<MapEntry> {
    let label = if have {
        v.map.label.clone()
    } else {
        format!("{} (not in your install)", v.map.label)
    };
    vec![MapEntry {
        label,
        starts: v.map.starts,
    }]
}

impl Client {
    /// The local install's index of a map by its content hash.
    fn map_index(&self, hash: &str) -> Option<usize> {
        self.install
            .as_ref()?
            .map_hashes
            .iter()
            .position(|h| h == hash)
    }

    /// A click on a target of the multiplayer screen.
    pub(super) fn net_menu(&mut self, hit: NetHit) {
        match hit {
            NetHit::Field(f) => self.netform.focus = Some(f),
            NetHit::Back => {
                self.net_leave();
                self.netform.focus = None;
                self.shell.screen = Screen::Menu;
            }
            NetHit::Host => self.net_open(Intent::Host),
            NetHit::Join => match normalise_code(&self.netform.code) {
                Some(code) => self.net_open(Intent::Join(code)),
                None => {
                    self.net_status = (
                        "That is not a room code (8 letters and digits).".into(),
                        true,
                    );
                }
            },
            NetHit::Reconnect => {
                if let Some(r) = self.settings.rejoin.clone() {
                    self.net_open(Intent::Rejoin(r));
                }
            }
        }
    }

    /// Keeps the game this client can rejoin (N5), or forgets it.
    pub(super) fn keep_rejoin(&mut self, r: Option<Rejoin>) {
        if self.settings.rejoin != r {
            self.settings.rejoin = r;
            self.settings_changed();
        }
    }

    /// The game this client plays began or was rejoined: it can rejoin it
    /// from now on, also after a restart.
    fn begun(&mut self) {
        let Some(n) = self.net.as_ref() else {
            return;
        };
        let (Some(b), Some(code)) = (n.begin.as_ref(), n.code()) else {
            return;
        };
        let relay = match &n.intent {
            Intent::Rejoin(r) => r.relay.clone(),
            _ => self.netform.relay.trim().to_owned(),
        };
        let r = Rejoin {
            relay,
            code,
            slot: b.you,
            ticket: b.ticket,
        };
        self.keep_rejoin(Some(r));
    }

    /// Connects to the relay and asks for a room (host) or about one (join).
    fn net_open(&mut self, intent: Intent) {
        self.net_leave();
        let Some(install) = self.install.as_ref() else {
            return;
        };
        let Some(connect) = self.connect else {
            self.net_status = ("This client cannot open a network connection.".into(), true);
            return;
        };
        let first = match &intent {
            Intent::Host => {
                let Some(((_, entry), hash)) = install.maps.first().zip(install.map_hashes.first())
                else {
                    return;
                };
                let rules = &install.setup;
                ToRelay::Create {
                    name: self.netform.name.trim().to_owned(),
                    build: self.build.clone(),
                    rules_hash: install.rules_hash.clone(),
                    offered: Offered {
                        countries: rules.countries.len().min(255) as u8,
                        credits: (rules.money_range.0, rules.money_range.1),
                        units: rules.units_range,
                    },
                    map: MapChoice {
                        hash: hash.clone(),
                        label: entry.label.clone(),
                        starts: entry.starts,
                    },
                    setup: Setup::new(rules),
                }
            }
            Intent::Join(code) => ToRelay::Peek { code: code.clone() },
            Intent::Rejoin(r) => ToRelay::Rejoin {
                code: r.code.clone(),
                slot: r.slot,
                ticket: r.ticket,
                build: self.build.clone(),
                rules_hash: install.rules_hash.clone(),
            },
        };
        let relay = match &intent {
            Intent::Rejoin(r) => r.relay.clone(),
            _ => {
                self.remember_relay();
                self.netform.relay.trim().to_owned()
            }
        };
        let link = connect(&relay);
        let mut net = Net::new(link, intent);
        net.direct = Some(self.direct_setup());
        net.send(&first);
        self.net = Some(net);
        self.netform.focus = None;
        self.net_status = ("Connecting to the relay...".into(), false);
    }

    /// How this client's next game makes direct links (N7): the settings'
    /// switch and servers, unless a command-line or URL option set them.
    fn direct_setup(&self) -> Direct {
        let open = if self.settings.direct {
            self.direct.clone()
        } else {
            None
        };
        let ice = if self.ice.urls.is_empty() {
            Ice::from_urls(&self.settings.ice)
        } else {
            self.ice.clone()
        };
        Direct {
            open,
            ice,
            shape: self.direct_shape,
            timeouts: self.direct_timeouts,
        }
    }

    /// Leaves the room or the game and closes the link.
    pub fn net_leave(&mut self) {
        if let Some(mut n) = self.net.take() {
            if (n.room.is_some() || n.begin.is_some()) && n.closed.is_none() {
                n.send(&ToRelay::Leave);
            }
            n.close();
        }
    }

    /// Moves the link's messages: the lobby's to the screens, the
    /// lockstep's to the game; the game's events become notices. Runs every
    /// frame on every screen.
    pub(super) fn net_pump(&mut self, now_ms: i64) {
        let now = now_ms.max(0) as u64;
        let Some(net) = self.net.as_mut() else {
            return;
        };
        let msgs = net.receive(now);
        for m in msgs {
            self.net_message(m, now_ms);
        }
        let lost = self.lost_game();
        let Some(net) = self.net.as_mut() else {
            return;
        };
        if let LinkState::Closed(why) = net.link.state()
            && net.closed.is_none()
        {
            net.closed = Some(why.clone());
            match self.shell.screen {
                Screen::Multiplayer | Screen::Room => {
                    self.net_status = (format!("The connection closed: {why}"), true);
                    self.net = None;
                    self.shell.screen = Screen::Multiplayer;
                    return;
                }
                _ if lost => {
                    return self.net_lost(&format!("The connection to the relay closed: {why}"));
                }
                _ => net.line(
                    now_ms,
                    "",
                    &format!("The connection to the relay closed: {why}"),
                    true,
                ),
            }
        }
        if let Some(net) = self.net.as_mut() {
            net.pump_direct(now);
        }
        self.net_events(now_ms);
        self.net_serve(now_ms);
        if let Some(net) = self.net.as_mut() {
            net.send_parts(now);
            net.flush(now);
        }
    }

    /// Sends the game to the rejoining peers the relay asked this peer to
    /// serve (N5), once its session runs: the session's catch-up and the
    /// game's save after the same ticks, in parts that go one at a time.
    fn net_serve(&mut self, now_ms: i64) {
        let (Some(net), Some(s)) = (self.net.as_mut(), self.session.as_ref()) else {
            return;
        };
        let Some(ls) = net.lockstep.as_ref() else {
            return;
        };
        let asked = ls.serving();
        net.to_serve.retain(|slot| asked.contains(slot));
        let Some(slot) = net.to_serve.first().copied() else {
            return;
        };
        let Some(t) = Transfer::of(ls, &s.play.game) else {
            return;
        };
        net.to_serve.remove(&slot);
        let who = net.slot_name(slot);
        let bytes = t.to_bytes();
        log::info!("rejoin: sending {who} the game ({} bytes)", bytes.len());
        net.sent_bytes += bytes.len() as u64;
        let ps = rejoin::parts(&bytes);
        let n = ps.len() as u32;
        for (i, b) in ps.into_iter().enumerate() {
            let m = ToRelay::Snapshot(Part {
                part: i as u32,
                parts: n,
                bytes: b,
            });
            net.serving.push_back((slot, lobby_wire::encode(&m)));
        }
        net.line(
            now_ms,
            "",
            &format!("{who} is reconnecting; sending them the game."),
            true,
        );
    }

    /// Whether the link of a network game that still runs (or is being
    /// rejoined) went: the person can rejoin it (N5).
    fn lost_game(&self) -> bool {
        let in_game = match self.shell.screen {
            Screen::Game => true,
            Screen::Settings => self.settings_from_game,
            _ => false,
        };
        in_game
            && self.settings.rejoin.is_some()
            && self.net.as_ref().is_some_and(|n| n.begin.is_some())
            && self.session.as_ref().is_none_or(|s| s.play.end().is_none())
    }

    /// The network game is lost to this client for now (its link went, or
    /// a rejoin failed): back to the multiplayer screen, which offers
    /// **Reconnect** within the grace period (N5).
    fn net_lost(&mut self, why: &str) {
        // A failed rejoin has used some of the grace period already.
        let grace_s = self.net.as_ref().and_then(|n| {
            if matches!(n.intent, Intent::Rejoin(_)) {
                return None;
            }
            let b = n.begin.as_ref()?;
            let speed = self.session.as_ref().map(|s| s.speed)?;
            Some(
                u64::from(b.params.sane().grace_ticks) * net_period_ms(speed).max(1) as u64 / 1_000,
            )
        });
        let text = match grace_s {
            Some(s) => format!(
                "{why}. Reconnect within about {s} seconds to play on, or your house surrenders."
            ),
            None => format!("{why}. Reconnect to play on."),
        };
        log::warn!("{text}");
        self.net_leave();
        self.session = None;
        self.captions.clear();
        self.settings_from_game = false;
        self.rebinding = None;
        self.net_status = (text, true);
        self.shell.screen = Screen::Multiplayer;
    }

    /// The lockstep session's events as notices.
    fn net_events(&mut self, now_ms: i64) {
        let Some(net) = self.net.as_mut() else {
            return;
        };
        let Some(events) = net.lockstep.as_mut().map(Lockstep::take_events) else {
            return;
        };
        let session = self.session.as_ref();
        let period = session.map_or(1, |s| net_period_ms(s.speed).max(1) as u64);
        for e in events {
            let Some(ls) = net.lockstep.as_ref() else {
                return;
            };
            let me = ls.me();
            match e {
                NetEvent::Refused { slot, field } => {
                    let who = net.slot_name(slot);
                    let text =
                        format!("The game cannot start: {who}'s {field} differs from yours.");
                    net.refused = Some(text.clone());
                    net.line(now_ms, "", &text, true);
                }
                // The report is made a few ticks later (`net_report`).
                NetEvent::Desync { tick, mine, theirs } => {
                    if let Some(ev) = net.evidence.as_mut() {
                        ev.desync(tick, theirs.slot, mine.hash, theirs.hash);
                    }
                    if net.desynced.is_none() {
                        net.desynced = Some(tick);
                        net.line(
                            now_ms,
                            "",
                            &format!("The games went apart at tick {tick} (a desync)."),
                            true,
                        );
                    }
                }
                NetEvent::Report { slot, tick, digest } => {
                    let who = net.slot_name(slot);
                    let hex = glasnost_net::sim::hex(&digest);
                    net.line(
                        now_ms,
                        "",
                        &format!(
                            "{who} saved a desync report of tick {tick} (digest {}).",
                            &hex[..DIGEST_SHOWN]
                        ),
                        true,
                    );
                }
                // The game goes on without the slot; its house surrenders
                // once the grace period is over, unless it rejoins (N5).
                NetEvent::Dropped { slot, .. } => {
                    let who = net.slot_name(slot);
                    let left = ls.surrender_at(slot).map(|at| at.saturating_sub(ls.ran()));
                    let text = match left {
                        Some(left) => {
                            let ms = u64::from(left) * period;
                            format!(
                                "{who} left the game; they have {} seconds to reconnect.",
                                ms.div_ceil(1_000)
                            )
                        }
                        None => format!("{who} left the game."),
                    };
                    net.line(now_ms, "", &text, true);
                }
                // The game goes once this peer's session runs
                // (`net_serve`); a new ask drops what an earlier one left.
                NetEvent::Serve { slot } => {
                    net.serving.retain(|(s, _)| *s != slot);
                    net.to_serve.insert(slot);
                }
                NetEvent::Resumed { slot, .. } if slot == me => {
                    net.line(now_ms, "", "You are back in the game.", true);
                }
                NetEvent::Resumed { slot, .. } => {
                    let who = net.slot_name(slot);
                    net.line(now_ms, "", &format!("{who} reconnected."), true);
                }
                NetEvent::Surrendered { slot, .. } => {
                    let who = net.slot_name(slot);
                    net.line(
                        now_ms,
                        "",
                        &format!("{who} did not come back in time: their house surrendered."),
                        true,
                    );
                }
                NetEvent::BadMessage { reason } => {
                    log::warn!("a lockstep message was refused: {reason}");
                }
                // A peer sent two different copies of a frame, one the
                // short way: the first ran; the hashes tell whether the
                // others ran the same (N7).
                NetEvent::Conflicting { slot, frame } => {
                    log::warn!("two copies of slot {slot}'s frame {frame} differ");
                    if net.conflicted.insert(slot) {
                        let who = net.slot_name(slot);
                        net.line(
                            now_ms,
                            "",
                            &format!("{who}'s orders came twice and differed (frame {frame})."),
                            true,
                        );
                    }
                }
                NetEvent::Started
                | NetEvent::Closing { .. }
                | NetEvent::DelayChanged { .. }
                | NetEvent::Silent { .. }
                | NetEvent::Heard { .. } => {}
            }
        }
        self.net_report(now_ms);
    }

    /// Makes the desync report once the evidence has the ticks past the
    /// mismatch, stores it, tells the other peers its digest and says where
    /// it went. Making it replays the ticks since the base snapshot for
    /// their hashes (a moment's work, once).
    fn net_report(&mut self, now_ms: i64) {
        let save = self.save_report;
        let (Some(net), Some(s)) = (self.net.as_mut(), self.session.as_ref()) else {
            return;
        };
        let (Some(ls), Some(ev)) = (net.lockstep.as_mut(), net.evidence.as_mut()) else {
            return;
        };
        let Some(b) = ev.take(ls.ran(), &s.play.game) else {
            return;
        };
        let tick = b.mismatch.map_or(0, |m| m.tick);
        let bytes = b.to_bytes();
        let digest = Bundle::digest(&bytes);
        let hex = glasnost_net::sim::hex(&digest);
        let code = net.room.as_ref().map_or("game", |v| v.code.as_str());
        let name = b.file_name(&format!("desync-{code}-"));
        let text = match save.map(|f| f(&name, &bytes)) {
            Some(Ok(place)) => {
                ls.set_report(tick, digest);
                log::info!("desync report of tick {tick} saved to {place} (digest {hex})");
                format!(
                    "Desync at tick {tick}; report saved to {place} (digest {}).",
                    &hex[..DIGEST_SHOWN]
                )
            }
            Some(Err(why)) => {
                log::warn!("the desync report of tick {tick} was not saved: {why}");
                format!("Desync at tick {tick}; the report was not saved: {why}.")
            }
            None => format!("Desync at tick {tick}; this client cannot save a report."),
        };
        net.line(now_ms, "", &text, true);
    }

    fn net_message(&mut self, m: FromRelay, now_ms: i64) {
        let lost = self.lost_game();
        let Some(net) = self.net.as_mut() else {
            return;
        };
        match m {
            FromRelay::Room(v) => {
                if net.room.is_none() {
                    self.net_status = (String::new(), false);
                    if self.shell.screen == Screen::Multiplayer {
                        self.shell.screen = Screen::Room;
                    }
                }
                net.note = None;
                net.room = Some(v);
            }
            FromRelay::Info(info) => {
                if net.room.is_some() || net.intent == Intent::Host {
                    return;
                }
                if info.playing {
                    self.net_status = ("That room's game has begun.".into(), true);
                    self.net_leave();
                    return;
                }
                // Say which two builds differ before the relay refuses the
                // join with only the field's name (M9 N8b).
                if info.build != self.build {
                    self.net_status = (
                        format!(
                            "That room runs {}; this is {}. Everybody must run the same build.",
                            info.build, self.build
                        ),
                        true,
                    );
                    self.net_leave();
                    return;
                }
                let map_hash = self
                    .map_index(&info.map.hash)
                    .and_then(|i| self.install.as_ref().map(|x| x.map_hashes[i].clone()))
                    .unwrap_or_else(|| "-".into());
                let rules_hash = self
                    .install
                    .as_ref()
                    .map(|i| i.rules_hash.clone())
                    .unwrap_or_default();
                let join = ToRelay::Join {
                    code: info.code,
                    name: self.netform.name.trim().to_owned(),
                    build: self.build.clone(),
                    rules_hash,
                    map_hash,
                };
                if let Some(net) = self.net.as_mut() {
                    net.send(&join);
                }
            }
            FromRelay::Refused(r) => {
                let text = r.to_string();
                if net.room.is_none() && net.begin.is_none() {
                    // A rejoin that can never work forgets its game; one
                    // that may work in a moment keeps it.
                    let gone = matches!(
                        r,
                        Refusal::Ticket
                            | Refusal::Surrendered
                            | Refusal::NoGame
                            | Refusal::NoRoom
                            | Refusal::Mismatch(_)
                    );
                    if matches!(net.intent, Intent::Rejoin(_)) && gone {
                        self.keep_rejoin(None);
                    }
                    self.net_status = (format!("The relay refused: {text}."), true);
                    self.net_leave();
                } else if self.shell.screen == Screen::Room {
                    net.note = Some((format!("Refused: {text}"), true));
                } else if r == Refusal::ChatTooFast || self.shell.screen == Screen::Game {
                    net.line(now_ms, "", &format!("Refused: {text}."), true);
                }
            }
            FromRelay::Begin(b) | FromRelay::Rejoined(b) => {
                if matches!(net.intent, Intent::Rejoin(_)) {
                    net.rejoin = Some(Assembler::default());
                }
                net.begin = Some(b);
                net.typing = false;
                self.begun();
                self.net_begin();
            }
            FromRelay::Snapshot(part) => self.net_part(&part),
            FromRelay::Closing(why) => {
                net.closed = Some(why.clone());
                if matches!(self.shell.screen, Screen::Multiplayer | Screen::Room) {
                    self.net_status = (format!("The relay closed the connection: {why}"), true);
                    self.net_leave();
                    self.shell.screen = Screen::Multiplayer;
                } else if lost {
                    self.net_lost(&format!("The relay closed the connection: {why}"));
                } else {
                    net.line(
                        now_ms,
                        "",
                        &format!("The relay closed the connection: {why}"),
                        true,
                    );
                }
            }
            FromRelay::Chat(c) => net.line(now_ms, &c.name, &c.text, false),
            // Direct links' signalling and the relay's ping (N7).
            FromRelay::Signal {
                from,
                attempt,
                signal,
            } => {
                let now = now_ms.max(0) as u64;
                if let (Some(m), Some(ls)) = (net.mesh.as_mut(), net.lockstep.as_ref()) {
                    m.signal_in(now, ls, from, attempt, signal);
                }
            }
            FromRelay::Pong { stamp } => net.pong(now_ms.max(0) as u64, stamp),
        }
    }

    /// The room began: load the map and start the game of the room's setup
    /// with the room's seed, played from this member's slot.
    fn net_begin(&mut self) {
        let Some(b) = self.net.as_ref().and_then(|n| n.begin.clone()) else {
            return;
        };
        let fail = |c: &mut Client, why: String| {
            c.net_status = (why, true);
            c.net_leave();
            c.shell.screen = Screen::Multiplayer;
        };
        let Some(index) = self.map_index(&b.map.hash) else {
            return fail(self, "The room's map is not in your install.".into());
        };
        let (Some(install), Some(vfs)) = (self.install.as_ref(), self.vfs.take()) else {
            return fail(self, "The install is busy.".into());
        };
        let maps: Vec<MapEntry> = install.maps.iter().map(|(_, m)| m.clone()).collect();
        let lobby = match b.local_setup(index).lobby(&install.setup, &maps) {
            // spec: S-0213 claim 15 (Network games: every machine checks an order's actor's owner immediately before applying it)
            Ok(mut l) => {
                l.settings.network = true;
                l
            }
            Err(p) => {
                self.vfs = Some(vfs);
                return fail(self, format!("The room's setup cannot start here: {p}"));
            }
        };
        let Some(slot) = b.slots.get(usize::from(b.you)) else {
            self.vfs = Some(vfs);
            return fail(self, "The room gave this client no slot.".into());
        };
        let job = Job::Start {
            vfs,
            file: install.maps[index].0.file,
            lobby,
            offered: install.setup.countries.clone(),
            seed: b.seed,
            speech: install
                .speech
                .as_ref()
                .map(glasnost_audio::speech::SpeechCatalog::from_ini)
                .unwrap_or_default(),
            local: Some(HouseId(slot.house)),
            cursor: install.cursor_palette.clone(),
            colour_blind: self.settings.colour_blind,
            watch: false,
        };
        self.session = None;
        self.status = "Loading the map...".into();
        self.busy = true;
        self.job = Some(job);
        self.net_loading = true;
        self.shell.screen = Screen::Game;
        self.loading_music();
    }

    /// A part of the game sent for a rejoin (N5): once all of it came, the
    /// session starts from it (now, or when the map has loaded).
    fn net_part(&mut self, part: &Part) {
        let Some(net) = self.net.as_mut() else {
            return;
        };
        let Some(asm) = net.rejoin.as_mut() else {
            log::warn!("a part of a game this client did not ask for");
            return;
        };
        match asm.add(part.part, part.parts, &part.bytes) {
            Ok(None) => {}
            Ok(Some(bytes)) => {
                log::info!("rejoin: the game came ({} bytes)", bytes.len());
                net.rejoin = None;
                net.received_bytes = Some(bytes.len());
                net.transfer = Some(bytes);
                self.net_resume();
            }
            Err(e) => self.net_lost(&format!("The game sent for the rejoin is not usable: {e}")),
        }
    }

    /// The network game's map has loaded: the lockstep session starts from
    /// this client's own values (its build, the game's rules hash and the
    /// map's content hash), which the handshake checks against the others'.
    pub(super) fn net_started(&mut self) {
        let now = self.clock.max(0) as u64;
        let (Some(net), Some(s)) = (self.net.as_mut(), self.session.as_mut()) else {
            return;
        };
        let Some(b) = net.begin.as_ref() else {
            return;
        };
        if matches!(net.intent, Intent::Rejoin(_)) {
            s.network = true;
            return self.net_resume();
        }
        let info = StartInfo::new(
            &self.build,
            &s.play.game.world.rules().hash,
            &s.map_id,
            &s.play.game.replay().setup,
            b.params,
        );
        let cfg = Config::new(net_period_ms(s.speed) as u32);
        let evidence = Evidence::new(&self.build, &info, b.you, &s.play.game);
        match Lockstep::new(cfg, info, b.you, now) {
            Ok(mut ls) => {
                if net.early_lost {
                    log::warn!("lockstep messages were lost while the map loaded");
                }
                net.early_bytes = 0;
                for bytes in net.early.drain(..) {
                    ls.receive(now, &bytes);
                }
                net.lockstep = Some(ls);
                net.evidence = Some(
                    evidence
                        .with_perturb(self.perturb)
                        .with_layers(s.play.capture.layers.describe()),
                );
                net.desynced = None;
                s.network = true;
                s.play.outbox = Some(Vec::new());
            }
            Err(f) => {
                let text = format!("The game cannot start: its {f} is not the room's.");
                net.refused = Some(text.clone());
                net.line(self.clock, "", &text, true);
            }
        }
    }

    /// A rejoin's game came and its map has loaded (N5): the game sent goes
    /// in place of the fresh one and the session starts from the sender's,
    /// with every lockstep message kept since the relay asked for the game.
    /// It then catches up ([`Client::net_tick`]).
    fn net_resume(&mut self) {
        let now = self.clock.max(0) as u64;
        let (Some(net), Some(s)) = (self.net.as_mut(), self.session.as_mut()) else {
            return;
        };
        if net.lockstep.is_some() || self.net_loading {
            return;
        }
        let (Some(b), Some(bytes)) = (net.begin.as_ref(), net.transfer.take()) else {
            return;
        };
        if net.early_lost {
            return self.net_lost("Too much of the game went on while it was being sent");
        }
        let info = StartInfo::new(
            &self.build,
            &s.play.game.world.rules().hash,
            &s.map_id,
            &s.play.game.replay().setup,
            b.params,
        );
        let me = b.you;
        let cfg = Config::new(net_period_ms(s.speed) as u32);
        let loaded = Transfer::from_bytes(&bytes).and_then(|t| {
            let g = t.load(&s.play.game)?;
            Ok((t, g))
        });
        let (t, game) = match loaded {
            Ok(x) => x,
            Err(e) => return self.net_lost(&format!("The game sent for the rejoin: {e}")),
        };
        let mut ls = match Lockstep::rejoin(cfg, info.clone(), me, &t.catchup, now) {
            Ok(ls) => ls,
            Err(f) => {
                return self.net_lost(&format!(
                    "The game sent for the rejoin does not fit: its {f} is not the room's"
                ));
            }
        };
        net.early_bytes = 0;
        for bytes in net.early.drain(..) {
            ls.receive(now, &bytes);
        }
        let ran = ls.ran();
        log::info!(
            "rejoin: the game of tick {ran} loaded ({} bytes); catching up",
            bytes.len()
        );
        s.replace_game(game);
        net.evidence = Some(
            Evidence::resume(&self.build, &info, me, &s.play.game, ran)
                .with_perturb(self.perturb)
                .with_layers(s.play.capture.layers.describe()),
        );
        net.lockstep = Some(ls);
        net.desynced = None;
        net.caught_up_sent = false;
        s.network = true;
        s.play.outbox = Some(Vec::new());
    }

    /// Runs a network game's tick when one is due and the lockstep session
    /// has every frame for it; returns whether it ran. The person's orders
    /// go to the session first.
    pub(super) fn net_tick(&mut self, now_ms: i64) -> bool {
        let now = now_ms.max(0) as u64;
        let (Some(net), Some(s)) = (self.net.as_mut(), self.session.as_mut()) else {
            return false;
        };
        let Some(ls) = net.lockstep.as_mut() else {
            return false;
        };
        if ls.catching_up() {
            // A rejoining peer runs the ticks it missed unpaced, a few an
            // update, and once it has run every one it has the frames for,
            // says so: the relay then has it readmitted (N5). Its person's
            // orders meanwhile go nowhere, and its ticks make no sound.
            if let Some(out) = s.play.outbox.as_mut() {
                out.clear();
            }
            let mut blocked = false;
            for _ in 0..CATCH_UP_TICKS {
                let ev = &mut net.evidence;
                let kept = &mut net.checkpoints;
                let ran = s.step_by(|g| {
                    if let Some(ev) = ev.as_mut() {
                        ev.before_tick(ls.ran(), g);
                    }
                    let Some(t) = ls.step(now, g) else {
                        return false;
                    };
                    Net::checkpoint(kept, &t);
                    if let Some(ev) = ev.as_mut() {
                        ev.after_tick(&t, g, ls.agreed());
                    }
                    true
                });
                if !ran {
                    blocked = true;
                    break;
                }
            }
            s.play.cues.clear();
            s.play.acks.clear();
            if blocked && !net.caught_up_sent {
                net.caught_up_sent = true;
                log::info!("rejoin: caught up to tick {}", ls.ran());
                net.send(&ToRelay::CaughtUp);
            }
            net.flush(now);
            self.pacer.reset();
            return false;
        }
        if let Some(out) = s.play.outbox.as_mut() {
            for o in out.drain(..) {
                ls.submit(o);
            }
        }
        let period = net_period_ms(s.speed).max(1) as u64;
        // A peer ahead of the slowest other stretches its period by half,
        // so the others catch up without a wait (our own netcode).
        let stretched = ls.ahead(now) > AHEAD_STRETCH
            && net
                .last_tick
                .is_some_and(|t| now.saturating_sub(t) < period + period / 2); // nocite: our own netcode (half a period more)
        if self.pacer.remaining_for(now_ms, period as i64) > 0 || stretched {
            self.pacer.waited(now_ms);
            return false;
        }
        let ev = &mut net.evidence;
        let kept = &mut net.checkpoints;
        let me = ls.me();
        let mut changed_by = Vec::new();
        let mut surrendered = false;
        let ran = s.step_by(|g| {
            if let Some(ev) = ev.as_mut() {
                ev.before_tick(ls.ran(), g);
            }
            let Some(t) = ls.step(now, g) else {
                return false;
            };
            Net::checkpoint(kept, &t);
            if let Some(ev) = ev.as_mut() {
                ev.after_tick(&t, g, ls.agreed());
            }
            // spec: S-0201 claim 17 (every machine but the sender's shows a message naming the house whose player changed the speed, in the tick the order is carried out)
            changed_by.extend(
                t.orders
                    .iter()
                    .filter(|a| a.taken && a.slot != me)
                    .filter(|a| matches!(a.order, glasnost_sim::orders::Order::GameSpeed { .. }))
                    .map(|a| a.slot),
            );
            // design: S-0201 claim 15's choice that surrenders and leaves waits until the lockstep has carried our own surrender order out, every peer then holding it
            surrendered |= t
                .orders
                .iter()
                .any(|a| a.slot == me && matches!(a.order, glasnost_sim::orders::Order::Surrender));
            true
        });
        let leave = surrendered && s.leave_after_surrender;
        // spec: S-0201 claim 17, Our engine (from the cycle after a speed order is carried out, the cycles and the lockstep's delay go by the new index's period)
        let tick_ms = net_period_ms(s.speed).max(1) as u32;
        if ls.tick_ms() != tick_ms {
            ls.set_tick_ms(tick_ms);
        }
        if !ran {
            self.pacer.waited(now_ms);
        }
        if ran {
            self.pacer.due_after(now_ms, period as i64);
            net.last_tick = Some(now);
            net.stalled_since = None;
            net.waiting = None;
        } else if ls.phase() == Phase::Running && s.play.running() {
            let since = *net.stalled_since.get_or_insert(now);
            if now.saturating_sub(since) >= STALL_MS {
                let names: Vec<String> = ls
                    .waiting_for()
                    .into_iter()
                    .map(|slot| net.slot_name(slot))
                    .collect();
                net.waiting = (!names.is_empty()).then(|| names.join(", "));
            }
        }
        for slot in changed_by {
            let who = net.slot_name(slot);
            net.line(now_ms, "", &format!("{who} changed the game speed."), true);
        }
        net.flush(now);
        if leave {
            // spec: S-0201 claim 15 (the choice that surrenders then leaves the game)
            self.session_event(crate::game::session::SessionEvent::Leave);
        }
        ran
    }

    /// Whether the person types into a chat or a field now (key bindings
    /// then do not act).
    pub(super) fn typing(&self) -> bool {
        match self.shell.screen {
            Screen::Multiplayer => self.netform.focus.is_some(),
            Screen::Room | Screen::Game => self.net.as_ref().is_some_and(|n| n.typing),
            Screen::Setup => self.shell.picker.open && self.shell.picker.typing,
            _ => false,
        }
    }

    /// Text typed on the keyboard (printable characters only).
    pub fn typed(&mut self, text: &str) {
        let text: String = text.chars().filter(|c| !c.is_control()).collect();
        if text.is_empty() {
            return;
        }
        match self.shell.screen {
            Screen::Multiplayer => self.netform.type_text(&text),
            Screen::Setup => self.search_typed(&text),
            Screen::Room | Screen::Game => {
                if let Some(n) = self.net.as_mut().filter(|n| n.typing) {
                    for c in text.chars() {
                        if n.input.chars().count() < CHAT_CHARS {
                            n.input.push(c);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Enter, Backspace and Escape on the network screens and in a network
    /// game; returns whether the key was taken.
    pub(super) fn net_key(&mut self, k: super::Key) -> bool {
        use super::Key;
        match (self.shell.screen.clone(), k) {
            (Screen::Multiplayer, Key::Backspace) => {
                self.netform.backspace();
                true
            }
            // Without a field typing, Enter and Escape are the keyboard's
            // press and back (the client's menu keys).
            (Screen::Multiplayer, Key::Enter | Key::Escape) if self.netform.focus.is_some() => {
                self.netform.focus = match (self.netform.focus, k) {
                    (Some(NetField::Name), Key::Enter) => Some(NetField::Relay),
                    (Some(NetField::Relay), Key::Enter) => Some(NetField::Code),
                    _ => None,
                };
                // The keyboard focus follows the field typed into.
                if let Some(f) = self.netform.focus {
                    self.net_focus.at = Some(NetHit::Field(f));
                }
                true
            }
            (Screen::Room | Screen::Game, _) => {
                let in_game = self.shell.screen == Screen::Game;
                let Some(n) = self.net.as_mut() else {
                    return false;
                };
                match k {
                    Key::Backspace if n.typing => {
                        n.input.pop();
                        true
                    }
                    Key::Enter if n.typing => {
                        let text = std::mem::take(&mut n.input);
                        if !text.trim().is_empty() {
                            n.send(&ToRelay::Chat { text });
                        }
                        // In the room the field keeps the focus; in the game
                        // the chat line closes.
                        n.typing = !in_game;
                        true
                    }
                    Key::Enter if in_game => {
                        n.typing = true;
                        true
                    }
                    Key::Escape if n.typing => {
                        n.typing = false;
                        n.input.clear();
                        true
                    }
                    _ => n.typing && matches!(k, Key::Char(_)),
                }
            }
            _ => false,
        }
    }

    /// A click on the room screen at `p`: the chat field, or a setup target
    /// turned into a request to the relay.
    pub(super) fn room_click(&mut self, p: Point) {
        let on_chat = self.chat_field.is_some_and(|r| r.contains(p));
        if let Some(n) = self.net.as_mut() {
            n.typing = on_chat;
        }
        if on_chat {
            return;
        }
        let Some(hit) = self.setup_targets.hit(p) else {
            return;
        };
        self.setup_focus.at = Some(hit);
        self.room_press(hit);
    }

    /// A room control pressed (a click or the keyboard), turned into a
    /// request to the relay.
    pub(super) fn room_press(&mut self, hit: SetupHit) {
        if hit == SetupHit::Back {
            self.net_leave();
            self.net_status = ("You left the room.".into(), false);
            self.shell.screen = Screen::Multiplayer;
            return;
        }
        if let Some(m) = self.room_request(hit)
            && let Some(n) = self.net.as_mut()
        {
            n.send(&m);
        }
    }

    /// The request a room target makes.
    fn room_request(&self, hit: SetupHit) -> Option<ToRelay> {
        let install = self.install.as_ref()?;
        let net = self.net.as_ref()?;
        let v = net.room.as_ref()?;
        let local = self.map_index(&v.map.hash);
        let map_hash = || {
            local
                .map(|i| install.map_hashes[i].clone())
                .unwrap_or_else(|| "-".into())
        };
        let host = v.you == v.host;
        let mut setup = v.setup.clone();
        setup.map = 0;
        let maps = room_maps(v, local.is_some());
        Some(match hit {
            SetupHit::Start_ if host => ToRelay::Start,
            SetupHit::Start_ => ToRelay::Ready {
                ready: !net.me().is_some_and(|m| m.ready),
                map_hash: map_hash(),
            },
            SetupHit::Map(d) => {
                let n = install.maps.len() as i32;
                let j = (local.map_or(0, |i| i as i32 + d)).rem_euclid(n.max(1)) as usize;
                let (_, entry) = install.maps.get(j)?;
                ToRelay::Edit(Edit::Map(MapChoice {
                    hash: install.map_hashes[j].clone(),
                    label: entry.label.clone(),
                    starts: entry.starts,
                }))
            }
            SetupHit::Kind(i, d) => {
                let order = [SlotKind::Human, SlotKind::Ai, SlotKind::Closed];
                let k = order
                    .iter()
                    .position(|k| *k == setup.slots[i].kind)
                    .unwrap_or(0) as i32;
                ToRelay::Edit(Edit::Row {
                    row: i as u8,
                    kind: order[(k + d).rem_euclid(3) as usize],
                })
            }
            SetupHit::Country(i, _)
            | SetupHit::Colour(i, _)
            | SetupHit::Team(i, _)
            | SetupHit::Start(i, _) => {
                setup.apply(&hit, &install.setup, &maps);
                let s = setup.slots[i];
                ToRelay::Edit(Edit::Choices {
                    row: i as u8,
                    country: s.country.map(|c| c.min(255) as u8),
                    colour: s.colour,
                    team: s.team,
                    start: s.start,
                })
            }
            SetupHit::Credits(_)
            | SetupHit::Units(_)
            | SetupHit::Toggle(_)
            | SetupHit::Fixes
            | SetupHit::Speed(_) => {
                setup.apply(&hit, &install.setup, &maps);
                ToRelay::Edit(Edit::Game {
                    credits: setup.credits,
                    units: setup.units,
                    flags: setup.flags,
                    speed: setup.speed,
                    fixes: setup.fixes,
                })
            }
            SetupHit::Back => return None,
            // The map picker is the skirmish screen's (design: map picker
            // (lead's request 2026-11-13)); the room never offers it.
            SetupHit::OpenMaps
            | SetupHit::PickMap(_)
            | SetupHit::SearchField
            | SetupHit::ClearSearch
            | SetupHit::Filter(..)
            | SetupHit::ScrollMaps(_)
            | SetupHit::MapsDone => return None,
        })
    }

    /// The room screen: the room's setup with its members, and the chat
    /// below it.
    pub(super) fn room_frame(&mut self, hover: Option<Point>) -> DrawList {
        let size = self.ui_size();
        let (Some(install), Some(net)) = (self.install.as_ref(), self.net.as_ref()) else {
            return DrawList::new();
        };
        let Some(v) = net.room.as_ref() else {
            return DrawList::new();
        };
        let local = self.map_index(&v.map.hash);
        let host = v.you == v.host;
        let mut setup = v.setup.clone();
        setup.map = 0;
        let maps = room_maps(v, local.is_some());
        let mut frame = RoomFrame {
            title: format!("ROOM {}", v.code),
            host,
            ..RoomFrame::default()
        };
        for m in &v.members {
            let row = usize::from(m.row);
            if row < MAX_SLOTS {
                frame.names[row] = Some(m.name.clone());
                frame.ready[row] = m.ready;
                if m.id == v.host {
                    frame.host_row = Some(row);
                }
                if m.id == v.you {
                    frame.mine = Some(row);
                }
            }
        }
        let others_ready = v.members.iter().all(|m| m.id == v.host || m.ready);
        let me_ready = net.me().is_some_and(|m| m.ready);
        (frame.go, frame.go_enabled) = if host {
            ("Start game".into(), others_ready)
        } else if me_ready {
            ("Not ready".into(), true)
        } else {
            ("Ready".into(), local.is_some())
        };
        frame.note = net.note.clone().or_else(|| {
            if local.is_none() {
                Some(("You do not have this room's map.".into(), true))
            } else if host && !others_ready {
                Some(("Waiting for everyone to be ready.".into(), false))
            } else if !host && !me_ready {
                Some(("Say ready when the setup suits you.".into(), false))
            } else {
                None
            }
        });
        let rules = self.shown_setup_rules(&install.setup);
        let (mut d, t) = setup.view_framed(size, &rules, &maps, hover, Some(&frame));
        self.setup_targets = t;
        let panel = Setup::panel_rect(size);
        let y = panel.bottom() + 6;
        let h = (size.1 - y - 6).min(160);
        self.chat_field = if h >= 44 {
            let lines = net.lines();
            Some(glasnost_ui::chat::view(
                &mut d,
                Rect::new(panel.x, y, panel.w, h),
                &lines,
                &net.input,
                net.typing,
            ))
        } else {
            None
        };
        d
    }

    /// The network game's notices over the battlefield: chat, whom the game
    /// waits for, why it cannot start.
    pub(super) fn net_overlay(&self, d: &mut DrawList) {
        let Some(net) = self.net.as_ref() else {
            return;
        };
        let (w, h) = self.ui_size();
        let recent: Vec<Line> = net
            .chat
            .iter()
            .filter(|(at, _)| self.clock - at < CHAT_SHOWN_MS)
            .map(|(_, l)| l.clone())
            .collect();
        let r = Rect::new(8, (h - 200).max(0), (w - 220).clamp(100, 520), 150);
        glasnost_ui::chat::overlay(d, r, &recent, net.typing.then_some(net.input.as_str()));
        let banner = net
            .refused
            .clone()
            .or_else(|| {
                net.waiting
                    .as_ref()
                    .map(|names| format!("Waiting for {names}..."))
            })
            .or_else(|| net.rejoin_banner())
            .or_else(|| {
                net.lockstep
                    .as_ref()
                    .filter(|ls| ls.phase() == Phase::Handshake)
                    .map(|_| "Waiting for the other players to load...".into())
            });
        if let Some(text) = banner {
            let r = Rect::new((w - 420).max(0) / 2, 60, 420.min(w), 20);
            d.fill(r, colour::VEIL);
            // design: readable text and display scale (lead's request 2026-11-10)
            d.text_centred_outlined(r, &text, colour::ACCENT);
        }
    }

    /// The multiplayer screen.
    pub(super) fn net_menu_frame(&mut self, hover: Option<Point>) -> DrawList {
        self.netform.rejoin = self.settings.rejoin.as_ref().map(|r| r.code.clone());
        let busy = self.net.as_ref().is_some_and(|n| n.room.is_none());
        let (status, bad) = if busy && self.net.as_ref().is_some_and(Net::connecting) {
            ("Connecting to the relay...".to_owned(), false)
        } else if busy {
            ("Waiting for the relay...".to_owned(), false)
        } else {
            self.net_status.clone()
        };
        let (d, t) = self.netform.view(self.ui_size(), &status, bad, busy, hover);
        self.net_targets = t;
        d
    }

    /// Makes this a test client (M9 N4; the native client in a debug build
    /// with `GLASNOST_PERTURB` set, and tests): it names [`test_build`], so
    /// that only other test clients play with it, and makes `perturb` in
    /// its network games to force a desync.
    pub fn set_test(&mut self, perturb: Option<Perturb>) {
        self.build = test_build();
        self.perturb = perturb;
    }

    /// The direct links' numbers (N7): per other person, whether its link
    /// is open now, the link's numbers, how its frames came and the
    /// session's round-trip estimate. Empty before the game runs.
    pub fn direct_stats(&self) -> Vec<PairReport> {
        let Some(net) = self.net.as_ref() else {
            return Vec::new();
        };
        let (Some(mesh), Some(ls)) = (net.mesh.as_ref(), net.lockstep.as_ref()) else {
            return Vec::new();
        };
        let now = self.clock.max(0) as u64;
        mesh.stats(now)
            .into_iter()
            .map(|(slot, open, stats)| PairReport {
                slot,
                enabled: mesh.enabled(),
                open,
                stats,
                paths: ls.paths(slot),
                rtt_ms: ls.rtt(slot),
            })
            .collect()
    }

    /// The relay's round trip, smoothed, and its range, in milliseconds
    /// (N7).
    pub fn relay_rtt(&self) -> (Option<u32>, Option<(u32, u32)>) {
        self.net
            .as_ref()
            .map_or((None, None), |n| (n.relay_rtt, n.relay_rtt_range))
    }

    /// The network game's desync evidence, once the game runs.
    pub fn evidence(&self) -> Option<&Evidence> {
        self.net.as_ref()?.evidence.as_ref()
    }

    /// Bytes of game this client sent rejoining peers, and the bytes of the
    /// game it was sent for its own rejoin, if any (M9 N5).
    pub fn transfer_bytes(&self) -> (u64, Option<usize>) {
        self.net
            .as_ref()
            .map_or((0, None), |n| (n.sent_bytes, n.received_bytes))
    }

    /// The checkpoints this client's game ran since the last call (tick,
    /// state hash), oldest first: the hashes its lockstep session compares
    /// with the other peers' (M9 N6). At most the last 4096 are kept.
    pub fn take_checkpoints(&mut self) -> Vec<(u32, glasnost_net::wire::Hash)> {
        self.net
            .as_mut()
            .map(|n| n.checkpoints.drain(..).collect())
            .unwrap_or_default()
    }

    /// The multiplayer screen's status line and whether it reports a
    /// failure (a refusal, a lost connection).
    pub fn net_status(&self) -> (&str, bool) {
        (self.net_status.0.as_str(), self.net_status.1)
    }

    /// The tick of the first desync this client's network game noticed, if
    /// any (M9 N6).
    pub fn net_desync(&self) -> Option<u32> {
        self.net.as_ref()?.desynced
    }

    /// The room code, while in a room.
    pub fn room_code(&self) -> Option<&str> {
        self.net.as_ref()?.room.as_ref().map(|v| v.code.as_str())
    }

    /// The room as the relay last showed it.
    pub fn room(&self) -> Option<&RoomView> {
        self.net.as_ref()?.room.as_ref()
    }

    /// The network game's lockstep session, once it exists.
    pub fn lockstep(&self) -> Option<&Lockstep> {
        self.net.as_ref()?.lockstep.as_ref()
    }

    /// The chat log and notices, oldest first.
    pub fn chat_lines(&self) -> Vec<Line> {
        self.net.as_ref().map(Net::lines).unwrap_or_default()
    }

    /// The network game's banner: why it cannot start, or whom it waits for.
    pub fn net_banner(&self) -> Option<String> {
        let n = self.net.as_ref()?;
        n.refused
            .clone()
            .or_else(|| n.waiting.as_ref().map(|w| format!("Waiting for {w}...")))
            .or_else(|| n.rejoin_banner())
    }

    /// Says a chat line in the room or the game.
    pub fn say(&mut self, text: &str) {
        if let Some(n) = self.net.as_mut() {
            n.send(&ToRelay::Chat { text: text.into() });
        }
    }

    /// Whether a network link is open or opening.
    pub fn networked(&self) -> bool {
        self.net.is_some()
    }

    /// How long the client may go without an update at host time `now_ms`
    /// while a network link is open, for [`super::pump::Pump`]: in a
    /// network game until its next tick is due, at least [`WAKE_MIN_MS`]
    /// (also while it waits for frames or catches up); on the network
    /// screens and while a game loads, [`WAKE_LINK_MS`]. `None` without a
    /// link.
    pub fn net_wait_ms(&self, now_ms: i64) -> Option<u64> {
        let net = self.net.as_ref()?;
        let session = self
            .session
            .as_ref()
            .filter(|_| self.shell.screen == Screen::Game && net.begin.is_some());
        let Some(s) = session else {
            return Some(WAKE_LINK_MS);
        };
        let period = net_period_ms(s.speed).max(1);
        let left = self.pacer.remaining_for(now_ms, period).max(0) as u64;
        Some(left.clamp(WAKE_MIN_MS, period as u64))
    }

    /// The field the multiplayer screen types into (tests and the window).
    pub fn net_field(&self) -> Option<NetField> {
        self.netform.focus
    }

    /// Why the game cannot start, as a lockstep field.
    pub fn net_refused_field(&self) -> Option<Field> {
        match self.lockstep()?.phase() {
            Phase::Refused { field, .. } => Some(field),
            _ => None,
        }
    }
}
