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

use glasnost_net::lobby::wire::{self as lobby_wire};
use glasnost_net::lobby::{
    Begin, CHAT_CHARS, Edit, FromRelay, MapChoice, Offered, Refusal, RoomView, ToRelay,
    normalise_code,
};
use glasnost_net::{Config, Event as NetEvent, Field, Phase, Session as Lockstep, StartInfo};
use glasnost_sim::world::HouseId;
use glasnost_ui::chat::Line;
use glasnost_ui::draw::{DrawList, colour};
use glasnost_ui::geom::{Point, Rect};
use glasnost_ui::netmenu::{NetField, NetHit};
use glasnost_ui::setup::{MAX_SLOTS, MapEntry, RoomFrame, Setup, SetupHit, SlotKind};
use glasnost_ui::shell::Screen;

use super::{Client, Job};
use crate::game::link::{Link, LinkState};
use crate::mapview::live::period_ms;

/// The build a client names in a room and in the lockstep handshake.
pub const BUILD: &str = concat!("glasnost ", env!("CARGO_PKG_VERSION"));

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

/// The lockstep bytes kept while the map loads.
// nocite: our own netcode
const EARLY_KEPT: usize = 4_096;

/// How many ticks ahead of the slowest peer this one may run before its
/// pacer stretches the tick period by half.
// nocite: our own netcode (Session::ahead's slack)
const AHEAD_STRETCH: i64 = 1;

/// What the person asked the relay for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Intent {
    /// Create a room.
    Host,
    /// Join the room of this code.
    Join(String),
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
    early: Vec<Vec<u8>>,
    stalled_since: Option<u64>,
    last_tick: Option<u64>,
    /// Whom the game waits for, once it has waited [`STALL_MS`].
    pub(super) waiting: Option<String>,
    /// Why the game cannot start (a handshake refusal).
    pub(super) refused: Option<String>,
    /// Why the link closed, once it did.
    pub(super) closed: Option<String>,
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
            early: Vec::new(),
            stalled_since: None,
            last_tick: None,
            waiting: None,
            refused: None,
            closed: None,
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
            } else if self.begin.is_some() && self.early.len() < EARLY_KEPT {
                self.early.push(bytes);
            }
        }
        out
    }

    /// Sends what the lockstep session has to send.
    fn flush(&mut self, now: u64) {
        if let Some(ls) = self.lockstep.as_mut() {
            for b in ls.poll(now) {
                self.link.send(b);
            }
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
        }
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
        };
        let link = connect(self.netform.relay.trim());
        let mut net = Net::new(link, intent);
        net.send(&first);
        self.net = Some(net);
        self.netform.focus = None;
        self.net_status = ("Connecting to the relay...".into(), false);
    }

    /// Leaves the room or the game and closes the link.
    pub(super) fn net_leave(&mut self) {
        if let Some(mut n) = self.net.take() {
            if n.room.is_some() && n.closed.is_none() {
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
                _ => net.line(
                    now_ms,
                    "",
                    &format!("The connection to the relay closed: {why}"),
                    true,
                ),
            }
        }
        self.net_events(now_ms);
        if let Some(net) = self.net.as_mut() {
            net.flush(now);
        }
    }

    /// The lockstep session's events as notices.
    fn net_events(&mut self, now_ms: i64) {
        let Some(net) = self.net.as_mut() else {
            return;
        };
        let Some(ls) = net.lockstep.as_mut() else {
            return;
        };
        for e in ls.take_events() {
            match e {
                NetEvent::Refused { slot, field } => {
                    let who = net.slot_name(slot);
                    let text =
                        format!("The game cannot start: {who}'s {field} differs from yours.");
                    net.refused = Some(text.clone());
                    net.line(now_ms, "", &text, true);
                }
                // N4 seam: the desync report. Now a notice.
                NetEvent::Desync { tick, .. } => net.line(
                    now_ms,
                    "",
                    &format!("The games went apart at tick {tick} (a desync)."),
                    true,
                ),
                // N5 seam: surrender and reconnection. Now a notice; the
                // game goes on without the slot.
                NetEvent::Dropped { slot, .. } => {
                    let who = net.slot_name(slot);
                    net.line(now_ms, "", &format!("{who} left the game."), true);
                }
                NetEvent::BadMessage { reason } => {
                    log::warn!("a lockstep message was refused: {reason}");
                }
                NetEvent::Started
                | NetEvent::DelayChanged { .. }
                | NetEvent::Silent { .. }
                | NetEvent::Heard { .. } => {}
            }
        }
    }

    fn net_message(&mut self, m: FromRelay, now_ms: i64) {
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
                if net.room.is_none() {
                    self.net_status = (format!("The relay refused: {text}."), true);
                    self.net_leave();
                } else if self.shell.screen == Screen::Room {
                    net.note = Some((format!("Refused: {text}"), true));
                } else if r == Refusal::ChatTooFast || self.shell.screen == Screen::Game {
                    net.line(now_ms, "", &format!("Refused: {text}."), true);
                }
            }
            FromRelay::Begin(b) => {
                net.begin = Some(b);
                net.typing = false;
                self.net_begin();
            }
            FromRelay::Closing(why) => {
                net.closed = Some(why.clone());
                if matches!(self.shell.screen, Screen::Multiplayer | Screen::Room) {
                    self.net_status = (format!("The relay closed the connection: {why}"), true);
                    self.net_leave();
                    self.shell.screen = Screen::Multiplayer;
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
            Ok(l) => l,
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
        };
        self.session = None;
        self.status = "Loading the map...".into();
        self.busy = true;
        self.job = Some(job);
        self.shell.screen = Screen::Game;
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
        let info = StartInfo::new(
            &self.build,
            &s.play.game.world.rules().hash,
            &s.map_id,
            &s.play.game.replay().setup,
            b.params,
        );
        let cfg = Config::new(period_ms(s.speed) as u32);
        match Lockstep::new(cfg, info, b.you, now) {
            Ok(mut ls) => {
                for bytes in net.early.drain(..) {
                    ls.receive(now, &bytes);
                }
                net.lockstep = Some(ls);
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
        if let Some(out) = s.play.outbox.as_mut() {
            for o in out.drain(..) {
                ls.submit(o);
            }
        }
        let period = period_ms(s.speed).max(1) as u64;
        // A peer ahead of the slowest other stretches its period by half,
        // so the others catch up without a wait (our own netcode).
        let stretched = ls.ahead(now) > AHEAD_STRETCH
            && net
                .last_tick
                .is_some_and(|t| now.saturating_sub(t) < period + period / 2); // nocite: our own netcode (half a period more)
        if self.pacer.remaining(now_ms, s.speed) > 0 || stretched {
            return false;
        }
        let ran = s.step_by(|g| ls.step(now, g).is_some());
        if ran {
            self.pacer.due(now_ms, s.speed);
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
        net.flush(now);
        ran
    }

    /// Whether the person types into a chat or a field now (key bindings
    /// then do not act).
    pub(super) fn typing(&self) -> bool {
        match self.shell.screen {
            Screen::Multiplayer => self.netform.focus.is_some(),
            Screen::Room | Screen::Game => self.net.as_ref().is_some_and(|n| n.typing),
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
            (Screen::Multiplayer, Key::Enter | Key::Escape) => {
                self.netform.focus = match (self.netform.focus, k) {
                    (Some(NetField::Name), Key::Enter) => Some(NetField::Relay),
                    (Some(NetField::Relay), Key::Enter) => Some(NetField::Code),
                    _ => None,
                };
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
            | SetupHit::Speed(_) => {
                setup.apply(&hit, &install.setup, &maps);
                ToRelay::Edit(Edit::Game {
                    credits: setup.credits,
                    units: setup.units,
                    flags: setup.flags,
                    speed: setup.speed,
                })
            }
            SetupHit::Back => return None,
        })
    }

    /// The room screen: the room's setup with its members, and the chat
    /// below it.
    pub(super) fn room_frame(&mut self, hover: Option<Point>) -> DrawList {
        let size = self.size;
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
        let (mut d, t) = setup.view_framed(size, &install.setup, &maps, hover, Some(&frame));
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
        let (w, h) = self.size;
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
            .or_else(|| {
                net.lockstep
                    .as_ref()
                    .filter(|ls| ls.phase() == Phase::Handshake)
                    .map(|_| "Waiting for the other players to load...".into())
            });
        if let Some(text) = banner {
            let r = Rect::new((w - 420).max(0) / 2, 60, 420.min(w), 20);
            d.fill(r, colour::VEIL);
            d.text_centred(r, &text, colour::ACCENT);
        }
    }

    /// The multiplayer screen.
    pub(super) fn net_menu_frame(&mut self, hover: Option<Point>) -> DrawList {
        let busy = self.net.as_ref().is_some_and(|n| n.room.is_none());
        let (status, bad) = if busy && self.net.as_ref().is_some_and(Net::connecting) {
            ("Connecting to the relay...".to_owned(), false)
        } else if busy {
            ("Waiting for the relay...".to_owned(), false)
        } else {
            self.net_status.clone()
        };
        let (d, t) = self.netform.view(self.size, &status, bad, busy, hover);
        self.net_targets = t;
        d
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
