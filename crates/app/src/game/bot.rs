//! A person at the client played by our AI (M9 N6, the soak harness): it
//! drives a [`Client`] through the same screens and the same order path a
//! person uses, and keeps the numbers the soak reports.
//!
//! The bot presses the client's own targets (laid out by a frame, clicked
//! by pointer), as the lanes do: **Multiplayer**, then **Host** or **Join**
//! with a room code; in the room the host opens the rows for the persons
//! and closes the others, every bot picks its own row's team and says
//! ready, and the host starts once every person is ready. In the game a
//! brain of our AI ([`glasnost_ai::brain::Brain`], the skirmish AI) thinks
//! once per tick the client ran, on the client's own world, for the house
//! of the bot's slot, and its orders go through [`Play::issue`]
//! (super::play::Play::issue): into the outbox, then the lockstep session,
//! so they reach every peer as a person's orders do. The AI sees only its
//! house's shroud and gives only the orders a person could give, as in a
//! skirmish (PLAN §4.7). When the client loses its game (its link closed),
//! the bot presses **Reconnect** after a short wait, and plays on with a
//! new brain once the client is readmitted. Its numbers: ticks run, stalls,
//! the agreed delay over time, rejoins, the evidence's memory and every
//! checkpoint hash the client ran.
//!
//! [`synthetic`] is the synthetic install (invented rules and map,
//! `glasnost_ai::fixture`) a bot can play without game data, for tests and
//! the CI soak. Our own harness; no spec applies.

use std::fmt::Write as _;

use glasnost_ai::brain::Brain;
use glasnost_net::Phase;
use glasnost_sim::econ::victory::Verdict;
use glasnost_sim::targeting::Modifiers;
use glasnost_sim::world::HouseRole;
use glasnost_ui::geom::{Point, Rect};
use glasnost_ui::menu::MenuHit;
use glasnost_ui::netmenu::NetHit;
use glasnost_ui::setup::{MAX_SLOTS, SetupHit, SlotKind, net_period_ms};
use glasnost_ui::shell::Screen;

use super::client::{Client, PairReport};
use super::direct::{Ice, Shape};
use super::play::Button;

/// How long a tick may come after the paced period before the bot counts a
/// stall, in milliseconds.
// nocite: our own harness (well above the pacer's jitter, well below the client's 1 s notice)
pub const STALL_MS: i64 = 100;

/// How many times the host steps through the maps looking for one with a
/// start for every person before it gives up.
// nocite: our own harness
pub const MAX_MAP_PRESSES: usize = 1000;

/// What the bot does in the room.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Role {
    /// Makes the room: `persons` person rows (its own included), every
    /// other row closed, on the install's `map`-th offered map.
    Host {
        /// The persons the game waits for.
        persons: usize,
        /// The map, by its place in the install's list.
        map: usize,
    },
    /// Joins the room of this code.
    Join {
        /// The room code.
        code: String,
    },
}

/// How the bot plays.
#[derive(Clone, Debug)]
pub struct BotConfig {
    /// The person's name in the room.
    pub name: String,
    /// The relay's address (`ws://HOST:PORT/`).
    pub relay: String,
    /// Host or join.
    pub role: Role,
    /// The team the bot picks for its own row (1 to 4), or none.
    pub team: Option<u8>,
    /// The row the bot expects to be given (the room gives the first open
    /// row, so a joiner's row follows the order of the joins); `None`: any.
    pub row: Option<usize>,
    /// Whether our AI plays the bot's house (otherwise it gives no order).
    pub ai: bool,
    /// How long after its game was lost the bot presses **Reconnect**, in
    /// milliseconds.
    pub reconnect_after_ms: i64,
    /// How long the bot waits between two presses in the room or on the
    /// multiplayer screen, in milliseconds.
    pub press_every_ms: i64,
    /// Stop after this many ticks; 0: when the game ends.
    pub max_ticks: u32,
    /// Stop after this much game time (ticks times the room's paced
    /// period), in milliseconds; 0: when the game ends.
    pub max_game_ms: i64,
    /// Whether an order equal to one the brain gave at one of the last
    /// delay + 2 ticks is dropped: in a network game an order runs a delay
    /// after it is given, and until then our AI (made for orders that run
    /// at once) sees no effect and gives it again.
    pub dedupe: bool,
    /// Whether a game to rejoin found in the client's settings at the start
    /// is rejoined (a restarted client process), rather than a room made or
    /// joined.
    pub resume: bool,
}

impl BotConfig {
    /// A bot with our harness's defaults.
    pub fn new(name: &str, relay: &str, role: Role) -> Self {
        Self {
            name: name.into(),
            relay: relay.into(),
            role,
            team: None,
            row: None,
            ai: true,
            reconnect_after_ms: 2_000, // nocite: our own harness
            press_every_ms: 150,       // nocite: our own harness
            max_ticks: 0,
            max_game_ms: 0,
            dedupe: true,
            resume: false,
        }
    }
}

/// Where the bot is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BotPhase {
    /// Not yet in a room.
    Lobby,
    /// In the room.
    Room,
    /// In the game (loading, catching up or playing).
    Game,
    /// The game was lost to the client (its link closed); the bot will
    /// reconnect.
    Lost,
    /// Finished: the game ended, the tick limit was reached, or the bot
    /// gave up with this reason.
    Done(Option<String>),
}

/// The bot's numbers.
#[derive(Clone, Debug, Default)]
pub struct BotStats {
    /// The room code.
    pub code: Option<String>,
    /// The bot's slot and row.
    pub slot: Option<u8>,
    /// The bot's row.
    pub row: Option<usize>,
    /// Whether the room is set up as the bot wants it (for the host: the
    /// map, the rows and its team, so that guests may join).
    pub configured: bool,
    /// The paced tick period of the room's speed, in milliseconds.
    pub period_ms: i64,
    /// The player houses the bot's house is allied with, itself aside (the lobby's
    /// teams make the alliances).
    pub allies: usize,
    /// The ticks the client has run.
    pub ticks: u32,
    /// When the game first ran a tick, and when the bot finished (the
    /// caller's clock).
    pub started_ms: Option<i64>,
    /// When the bot finished.
    pub ended_ms: Option<i64>,
    /// Ticks that came more than [`STALL_MS`] after their paced period.
    pub stalls: u32,
    /// The longest gap between two ticks, in milliseconds (catch-up and
    /// the time the client was out of its game aside).
    pub longest_gap_ms: i64,
    /// Times the client showed "Waiting for ...".
    pub waits_shown: u32,
    /// The agreed delay at every change: (tick, delay in ticks).
    pub delay: Vec<(u32, u8)>,
    /// Times the client lost its game.
    pub lost: u32,
    /// Presses of **Reconnect**.
    pub reconnects: u32,
    /// Rejoins that ended in the readmission, with how long each took from
    /// the loss, in milliseconds.
    pub rejoined_ms: Vec<i64>,
    /// The bytes of game this client sent rejoining peers, over all its
    /// connections.
    pub sent_game_bytes: u64,
    /// The bytes of game this client was sent for its own rejoins, all of
    /// them together.
    pub received_game_bytes: usize,
    /// The most the desync evidence held, in bytes.
    pub evidence_max: usize,
    /// The orders our AI gave through the client.
    pub orders: u64,
    /// The orders dropped as repeats of one still in flight
    /// ([`BotConfig::dedupe`]).
    pub repeats: u64,
    /// The orders of the bot's house the game took (from its replay).
    pub taken: usize,
    /// The first desync the client noticed.
    pub desync: Option<u32>,
    /// Every checkpoint the client ran: (tick, state hash).
    pub checkpoints: Vec<(u32, glasnost_net::wire::Hash)>,
    /// The houses that won (the game's outcome), once it ended.
    pub winners: Option<Vec<u8>>,
    /// The tick in which the game was over, once it was.
    pub over: Option<i32>,
    /// The bot's own house.
    pub house: Option<u8>,
    /// The direct links to the other persons, as last seen in the game
    /// (M9 N7).
    pub pairs: Vec<PairReport>,
    /// The relay's round trip, smoothed, and its range, in milliseconds.
    pub relay_rtt: (Option<u32>, Option<(u32, u32)>),
}

impl BotStats {
    /// The game time played, in milliseconds of the room's speed.
    pub fn game_ms(&self) -> i64 {
        i64::from(self.ticks) * self.period_ms
    }

    /// The checkpoints as text, one `TICK HASH` line each.
    pub fn checkpoint_lines(&self) -> String {
        let mut s = String::new();
        for (t, h) in &self.checkpoints {
            let _ = writeln!(s, "{t} {}", glasnost_net::sim::hex(h));
        }
        s
    }

    /// A summary as `key value` lines (counts and timings only).
    pub fn summary(&self) -> String {
        let mut s = String::new();
        let opt = |v: Option<String>| v.unwrap_or_else(|| "-".into());
        let _ = writeln!(s, "code {}", opt(self.code.clone()));
        let _ = writeln!(s, "slot {}", opt(self.slot.map(|v| v.to_string())));
        let _ = writeln!(s, "row {}", opt(self.row.map(|v| v.to_string())));
        let _ = writeln!(s, "house {}", opt(self.house.map(|v| v.to_string())));
        let _ = writeln!(s, "period_ms {}", self.period_ms);
        let _ = writeln!(s, "allies {}", self.allies);
        let _ = writeln!(s, "ticks {}", self.ticks);
        let _ = writeln!(s, "game_ms {}", self.game_ms());
        let wall = match (self.started_ms, self.ended_ms) {
            (Some(a), Some(b)) => b - a,
            _ => 0,
        };
        let _ = writeln!(s, "wall_ms {wall}");
        let _ = writeln!(s, "stalls {}", self.stalls);
        let _ = writeln!(s, "longest_gap_ms {}", self.longest_gap_ms);
        let _ = writeln!(s, "waits_shown {}", self.waits_shown);
        let delays: Vec<String> = self.delay.iter().map(|(t, d)| format!("{t}:{d}")).collect();
        let _ = writeln!(s, "delay {}", delays.join(","));
        let _ = writeln!(s, "lost {}", self.lost);
        let _ = writeln!(s, "reconnects {}", self.reconnects);
        let rj: Vec<String> = self.rejoined_ms.iter().map(i64::to_string).collect();
        let _ = writeln!(s, "rejoined_ms {}", rj.join(","));
        let _ = writeln!(s, "sent_game_bytes {}", self.sent_game_bytes);
        let _ = writeln!(s, "received_game_bytes {}", self.received_game_bytes);
        let _ = writeln!(s, "evidence_max {}", self.evidence_max);
        let _ = writeln!(s, "orders {}", self.orders);
        let _ = writeln!(s, "repeats {}", self.repeats);
        let _ = writeln!(s, "taken {}", self.taken);
        let _ = writeln!(s, "checkpoints {}", self.checkpoints.len());
        let _ = writeln!(s, "desync {}", opt(self.desync.map(|v| v.to_string())));
        let _ = writeln!(s, "over {}", opt(self.over.map(|v| v.to_string())));
        let _ = writeln!(
            s,
            "winners {}",
            opt(self
                .winners
                .as_ref()
                .map(|w| { w.iter().map(u8::to_string).collect::<Vec<_>>().join(",") }))
        );
        let range = |r: Option<(u32, u32)>| r.map_or("-".into(), |(a, b)| format!("{a}-{b}"));
        let _ = writeln!(
            s,
            "relay_rtt_ms {} {}",
            opt(self.relay_rtt.0.map(|v| v.to_string())),
            range(self.relay_rtt.1)
        );
        for p in &self.pairs {
            let paths = p.paths.unwrap_or_default();
            let st = &p.stats;
            let _ = writeln!(
                s,
                "pair{} enabled={} open={} direct_first={} relay_first={} gated={} conflicts={} \
attempts={} opened={} failed={} fallbacks={} connect_ms={} rtt_ms={} rtt_range={} session_rtt_ms={} \
sent={} taken={} refused={} open_ms={}",
                p.slot,
                u8::from(p.enabled),
                u8::from(p.open),
                paths.direct_first,
                paths.relay_first,
                paths.gated,
                paths.conflicts,
                st.attempts,
                st.opened,
                st.failed,
                st.fallbacks,
                opt(st.connect_ms.map(|v| v.to_string())),
                opt(st.rtt_ms.map(|v| v.to_string())),
                range(st.rtt_range),
                opt(p.rtt_ms.map(|v| v.to_string())),
                st.packets.0,
                st.packets.1,
                st.refused,
                st.open_ms,
            );
        }
        s
    }
}

/// A person played by our AI.
pub struct Bot {
    /// How it plays.
    pub cfg: BotConfig,
    /// Where it is.
    pub phase: BotPhase,
    /// Its numbers.
    pub stats: BotStats,
    brain: Option<Brain>,
    last_press: Option<i64>,
    /// The tick the brain last thought at.
    thought: Option<i32>,
    /// The last tick seen run, and when.
    last_tick: Option<(u32, i64)>,
    lost_at: Option<i64>,
    /// The game was lost and the client is not yet readmitted.
    rejoining: bool,
    waiting: bool,
    maps_pressed: usize,
    started_at: Option<i64>,
    /// The orders given in the last ticks, not yet seen to run.
    in_flight: std::collections::VecDeque<(i32, glasnost_sim::orders::Order)>,
    /// The client's transfer counters as last seen; they start again with
    /// every connection.
    seen_transfer: (u64, Option<usize>),
}

fn click(c: &mut Client, r: Rect) {
    let p = Point::new(r.x + r.w / 2, r.y + r.h / 2);
    c.pointer_moved(p);
    c.pointer_down(Button::Left, p);
    c.pointer_up(Button::Left, p, Modifiers::default());
}

impl Bot {
    /// A bot that will play as `cfg` says.
    pub fn new(cfg: BotConfig) -> Self {
        Self {
            cfg,
            phase: BotPhase::Lobby,
            stats: BotStats::default(),
            brain: None,
            last_press: None,
            thought: None,
            last_tick: None,
            lost_at: None,
            rejoining: false,
            waiting: false,
            maps_pressed: 0,
            started_at: None,
            in_flight: Default::default(),
            seen_transfer: (0, None),
        }
    }

    /// Whether the bot has finished.
    pub fn done(&self) -> bool {
        matches!(self.phase, BotPhase::Done(_))
    }

    /// Why the bot gave up, if it did.
    pub fn failure(&self) -> Option<&str> {
        match &self.phase {
            BotPhase::Done(Some(why)) => Some(why),
            _ => None,
        }
    }

    fn finish(&mut self, now: i64, why: Option<String>) {
        self.stats.ended_ms = Some(now);
        self.phase = BotPhase::Done(why);
    }

    /// Whether a press is due (presses are spaced so that the relay's
    /// answer to one is seen before the next).
    fn due(&mut self, now: i64) -> bool {
        if self
            .last_press
            .is_some_and(|t| now - t < self.cfg.press_every_ms)
        {
            return false;
        }
        self.last_press = Some(now);
        true
    }

    fn press_menu(c: &mut Client, hit: MenuHit) -> bool {
        let _ = c.frame();
        let Some(r) = c.menu_targets.rect_of(&hit) else {
            return false;
        };
        click(c, r);
        true
    }

    fn press_net(c: &mut Client, hit: NetHit) -> bool {
        let _ = c.frame();
        let Some(r) = c.net_targets.rect_of(&hit) else {
            return false;
        };
        click(c, r);
        true
    }

    fn press_room(c: &mut Client, hit: SetupHit) -> bool {
        let _ = c.frame();
        let Some(r) = c.setup_targets.rect_of(&hit) else {
            return false;
        };
        click(c, r);
        true
    }

    /// One step: call after every [`Client::update`] (and after running
    /// the jobs the client handed out). `now` is the clock given to the
    /// client.
    pub fn step(&mut self, c: &mut Client, now: i64) {
        if self.done() {
            return;
        }
        match c.shell.screen.clone() {
            Screen::Menu => {
                if self.due(now) {
                    Self::press_menu(c, MenuHit::Multiplayer);
                }
            }
            Screen::Multiplayer => self.on_multiplayer(c, now),
            Screen::Room => self.on_room(c, now),
            Screen::Game => self.on_game(c, now),
            Screen::End(_) => {
                // The client shows the end screen on the tick the game is
                // over, before the bot saw it in the game.
                self.collect(c);
                self.outcome(c);
                self.finish(now, None);
            }
            Screen::Setup | Screen::Settings => {
                self.finish(now, Some("the client left the network screens".into()));
            }
        }
    }

    fn on_multiplayer(&mut self, c: &mut Client, now: i64) {
        if self.phase == BotPhase::Game {
            // The client lost its game: its link closed.
            self.collect(c);
            self.phase = BotPhase::Lost;
            self.stats.lost += 1;
            self.lost_at = Some(now);
            self.rejoining = true;
            self.brain = None;
            self.waiting = false;
            log::info!(
                "bot {}: the game was lost: {}",
                self.cfg.name,
                c.net_status().0
            );
        }
        if c.networked() {
            return;
        }
        if self.phase == BotPhase::Lobby && self.cfg.resume && c.settings.rejoin.is_some() {
            // A restarted client: its last game waits for it.
            self.phase = BotPhase::Lost;
            self.lost_at = Some(now - self.cfg.reconnect_after_ms);
            self.rejoining = true;
            log::info!("bot {}: a game to rejoin was saved", self.cfg.name);
        }
        if self.phase == BotPhase::Lost {
            if c.settings.rejoin.is_none() {
                let why = format!(
                    "the game cannot be rejoined: {}",
                    c.net_status().0.trim_end_matches('.')
                );
                return self.finish(now, Some(why));
            }
            let lost = self.lost_at.unwrap_or(now);
            // A refused rejoin (another one in progress) is tried again
            // after the same wait.
            let since = self.last_press.map_or(lost, |p| p.max(lost));
            if now - since >= self.cfg.reconnect_after_ms {
                self.last_press = Some(now);
                if Self::press_net(c, NetHit::Reconnect) {
                    self.stats.reconnects += 1;
                    log::info!("bot {}: reconnecting", self.cfg.name);
                }
            }
            return;
        }
        // Not yet in a room (or a refusal brought the client back here).
        if !self.due(now) {
            return;
        }
        if self.last_press.is_some() && c.net_status().1 {
            // A failed attempt waits a little longer.
            self.last_press = Some(now + self.cfg.press_every_ms * 5);
            log::info!("bot {}: {}", self.cfg.name, c.net_status().0);
        }
        c.netform.relay = self.cfg.relay.clone();
        c.netform.name = self.cfg.name.clone();
        match &self.cfg.role {
            Role::Host { .. } => {
                Self::press_net(c, NetHit::Host);
            }
            Role::Join { code } => {
                c.netform.code = code.clone();
                Self::press_net(c, NetHit::Join);
            }
        }
    }

    fn on_room(&mut self, c: &mut Client, now: i64) {
        self.phase = BotPhase::Room;
        let Some(v) = c.room().cloned() else {
            return;
        };
        self.stats.code = Some(v.code.clone());
        let Some(me) = v.members.iter().find(|m| m.id == v.you).cloned() else {
            return;
        };
        let row = usize::from(me.row);
        self.stats.row = Some(row);
        if let Some(want) = self.cfg.row
            && want != row
        {
            let why = format!("the room gave row {row}, not row {want}");
            c.net_leave();
            return self.finish(now, Some(why));
        }
        if !self.due(now) {
            return;
        }
        let host = v.you == v.host;
        if host && let Role::Host { persons, map } = self.cfg.role.clone() {
            // The map first: every other edit is made on it. The bot goes
            // on from the `map`-th to the first with a start for every
            // person.
            if self.maps_pressed < map || (v.map.starts as usize) < persons {
                if self.maps_pressed > MAX_MAP_PRESSES {
                    let why = format!("no offered map has {persons} start positions");
                    c.net_leave();
                    return self.finish(now, Some(why));
                }
                self.maps_pressed += 1;
                Self::press_room(c, SetupHit::Map(1));
                return;
            }
            let rows = glasnost_ui::setup::rows(Some(&glasnost_ui::setup::MapEntry {
                label: String::new(),
                starts: v.map.starts,
            }))
            .min(MAX_SLOTS);
            for r in 0..rows {
                let taken = v.members.iter().any(|m| usize::from(m.row) == r);
                let want = if r < persons {
                    SlotKind::Human
                } else {
                    SlotKind::Closed
                };
                let kind = v.setup.slots[r].kind;
                if kind != want && !taken {
                    // Human, computer, closed in turn.
                    let d = if want == SlotKind::Human { -1 } else { 1 };
                    Self::press_room(c, SetupHit::Kind(r, d));
                    return;
                }
            }
        }
        if v.setup.slots[row].team != self.cfg.team {
            Self::press_room(c, SetupHit::Team(row, 1));
            return;
        }
        // The room is as the bot wants it: the others may come.
        self.stats.configured = true;
        if host {
            let Role::Host { persons, .. } = self.cfg.role else {
                return;
            };
            let all_in = v.members.len() >= persons;
            let ready = v.members.iter().all(|m| m.id == v.host || m.ready);
            if all_in && ready {
                // A refused start (a mark cleared meanwhile) is tried again
                // after a longer wait.
                self.last_press = Some(now + self.cfg.press_every_ms * 10);
                Self::press_room(c, SetupHit::Start_);
            }
        } else if !me.ready {
            Self::press_room(c, SetupHit::Start_);
        }
    }

    fn on_game(&mut self, c: &mut Client, now: i64) {
        self.phase = BotPhase::Game;
        self.collect(c);
        let Some(ls) = c.lockstep() else {
            return;
        };
        let (me, delay, ran, running) = (
            ls.me(),
            ls.delay(),
            ls.ran(),
            ls.phase() == Phase::Running && !ls.catching_up(),
        );
        self.stats.slot = Some(me);
        if !running {
            return;
        }
        if self.rejoining {
            self.rejoining = false;
            let took = now - self.lost_at.unwrap_or(now);
            self.stats.rejoined_ms.push(took);
            self.last_tick = None;
            log::info!(
                "bot {}: readmitted at tick {ran} after {took} ms",
                self.cfg.name
            );
        }
        if self.stats.delay.last().is_none_or(|&(_, d)| d != delay) {
            self.stats.delay.push((ran, delay));
        }
        let banner = c.net_banner().is_some_and(|b| b.starts_with("Waiting for"));
        if banner && !self.waiting {
            self.stats.waits_shown += 1;
        }
        self.waiting = banner;
        let Some(s) = c.session.as_mut() else {
            return;
        };
        let period = net_period_ms(s.speed).max(1);
        self.stats.period_ms = period;
        self.stats.house = Some(s.play.me().0);
        let world = &s.play.game.world;
        self.stats.allies = world.houses[usize::from(s.play.me().0)]
            .allies
            .iter()
            .filter(|h| {
                **h != s.play.me() && world.houses[usize::from(h.0)].role == HouseRole::Player
            })
            .count();
        let started = *self.started_at.get_or_insert(now);
        self.stats.started_ms.get_or_insert(started);
        // The ticks and their gaps.
        match self.last_tick {
            Some((t, at)) if ran > t => {
                let gap = now - at;
                // spec: none (our own harness: a tick a period after the last is on time)
                if ran == t + 1 && gap > period + STALL_MS {
                    self.stats.stalls += 1;
                }
                if ran == t + 1 {
                    self.stats.longest_gap_ms = self.stats.longest_gap_ms.max(gap);
                }
                self.last_tick = Some((ran, now));
            }
            Some(_) => {}
            None => self.last_tick = Some((ran, now)),
        }
        self.stats.ticks = ran;
        // Our AI plays the house: once per tick, at the tick boundary.
        let w = &s.play.game.world;
        if self.cfg.ai && self.thought != Some(w.tick) && s.play.game.outcome().is_none() {
            self.thought = Some(w.tick);
            let house = s.play.me();
            let brain = self
                .brain
                .get_or_insert_with(|| Brain::new(w, &s.loaded.map, house));
            let mut orders = brain.think(w);
            // nocite: our own harness (a diagnostic line every 1500 ticks; it names
            // types, so only at the debug level, `GLASNOST_LOG=debug`)
            if w.tick % 1500 == 0 {
                log::debug!(
                    "bot {}: tick {}: {}",
                    self.cfg.name,
                    w.tick,
                    brain.diagnose(w)
                );
            }
            log::debug!(
                "bot {}: tick {} orders {}",
                self.cfg.name,
                w.tick,
                orders.len()
            );
            if self.cfg.dedupe {
                // An order given at an earlier tick runs only a delay later:
                // until then the brain sees no effect and gives it again. A
                // person does not click twice; the bot drops the repeat.
                let window = i32::from(delay) + 2; // nocite: our own harness (the delay, a tick to run it, a tick to see it)
                let tick = w.tick;
                self.in_flight.retain(|(t, _)| tick - *t <= window);
                let before = orders.len();
                orders.retain(|o| !self.in_flight.iter().any(|(t, f)| *t != tick && f == o));
                self.stats.repeats += (before - orders.len()) as u64;
                self.in_flight
                    .extend(orders.iter().map(|o| (tick, o.clone())));
            }
            if !orders.is_empty() {
                self.stats.orders += s.play.issue(orders) as u64;
            }
        }
        if s.play.game.outcome().is_some() {
            self.collect(c);
            self.outcome(c);
            return self.finish(now, None);
        }
        let game_ms = i64::from(ran) * period;
        if (self.cfg.max_ticks > 0 && ran >= self.cfg.max_ticks)
            || (self.cfg.max_game_ms > 0 && game_ms >= self.cfg.max_game_ms)
        {
            self.collect(c);
            self.finish(now, None);
        }
    }

    /// Takes the game's outcome, if it is over.
    fn outcome(&mut self, c: &Client) {
        let Some(o) = c.session.as_ref().and_then(|s| s.play.game.outcome()) else {
            return;
        };
        self.stats.winners = Some(
            o.results
                .iter()
                .filter(|(_, v)| *v == Verdict::Win)
                .map(|(h, _)| h.0)
                .collect(),
        );
        self.stats.over = Some(o.over);
        if let Some(s) = c.session.as_ref() {
            let w = &s.play.game.world;
            for (h, v) in &o.results {
                let mut n = [0usize; 4];
                for ob in w.objects.values() {
                    if ob.owner == Some(*h) && ob.on_map && ob.health > 0 {
                        let k = match ob.kind {
                            glasnost_sim::rules::Kind::Infantry => 0,
                            glasnost_sim::rules::Kind::Vehicle => 1,
                            glasnost_sim::rules::Kind::Aircraft => 2,
                            _ => 3,
                        };
                        n[k] += 1;
                    }
                }
                log::info!(
                    "bot {}: over at {}: house {} {:?}: infantry {} vehicles {} aircraft {} buildings and other {}",
                    self.cfg.name,
                    o.over,
                    h.0,
                    v,
                    n[0],
                    n[1],
                    n[2],
                    n[3]
                );
            }
        }
    }

    /// Takes the client's checkpoints, desync and transfer numbers.
    fn collect(&mut self, c: &mut Client) {
        self.stats.checkpoints.extend(c.take_checkpoints());
        if self.stats.desync.is_none() {
            self.stats.desync = c.net_desync();
        }
        if let Some(e) = c.evidence() {
            self.stats.evidence_max = self.stats.evidence_max.max(e.bytes_held());
        }
        let (sent, got) = c.transfer_bytes();
        let (seen_sent, seen_got) = self.seen_transfer;
        // A new connection counts from 0 again.
        self.stats.sent_game_bytes += sent - if sent < seen_sent { 0 } else { seen_sent };
        if let (Some(g), None) = (got, seen_got) {
            self.stats.received_game_bytes += g;
        }
        self.seen_transfer = (sent, got);
        if let Some(s) = c.session.as_ref() {
            let house = s.play.me();
            self.stats.taken = s
                .play
                .game
                .replay()
                .orders
                .iter()
                .filter(|(_, h, _)| *h == house)
                .count();
        }
    }
}

/// The usage line of a net-client (`glasnost-headless net-client` and the
/// wasm client take the same options).
pub const USAGE: &str = "net-client --relay ws://HOST:PORT/ (--host N [--map M] | --join CODE) \
[--name NAME] [--team T] [--slot ROW] [--seed S] [--ticks N | --minutes M] [--hash-log FILE] \
[--report FILE] [--code-file FILE] [--settings FILE] [--resume] [--no-ai] [--no-dedupe] \
[--relay-only] [--ice URLS] [--direct-shape LAT,JIT,LOSS,EVERY_MIN,EVERY_MAX,STALL_MIN,STALL_MAX] \
[--game-dir DIR | --synthetic]";

/// The options of a net-client.
#[derive(Clone, Debug)]
pub struct Options {
    /// How the bot plays.
    pub bot: BotConfig,
    /// The seed of the reconnect wait's jitter.
    pub seed: u64,
    /// Where the checkpoint hashes go, one `TICK HASH` line each.
    pub hash_log: Option<String>,
    /// Where the numbers go.
    pub report: Option<String>,
    /// Where the room code goes once the room is made.
    pub code_file: Option<String>,
    /// The client's settings file (its saved rejoin).
    pub settings: Option<String>,
    /// The install's folder, or none for the synthetic install.
    pub game_dir: Option<String>,
    /// No direct links: the relay carries everything (M9 N7).
    pub relay_only: bool,
    /// The direct links' STUN or TURN servers ([`Ice::from_urls`]).
    pub ice: Option<String>,
    /// A bad network on the direct path, seeded by the seed.
    pub direct_shape: Option<Shape>,
}

impl Options {
    /// Sets the client's direct links as the options say.
    pub fn apply(&self, c: &mut Client) {
        if self.relay_only {
            c.direct = None;
        }
        if let Some(ice) = self.ice.as_deref() {
            c.ice = Ice::from_urls(ice);
        }
        c.direct_shape = self.direct_shape;
    }
}

/// Reads `--direct-shape`: latency, jitter, loss in a thousand, the
/// shortest and longest wait between stalls and the shortest and longest
/// stall, in milliseconds, comma-separated.
fn shape(text: &str, seed: u64) -> Result<Shape, String> {
    let v: Vec<u64> = text
        .split(',')
        .map(|p| p.trim().parse::<u64>())
        .collect::<Result<_, _>>()
        .map_err(|_| format!("--direct-shape needs seven numbers: {text}"))?;
    let [latency_ms, jitter_ms, loss, e0, e1, s0, s1] = v[..] else {
        return Err(format!("--direct-shape needs seven numbers: {text}"));
    };
    Ok(Shape {
        latency_ms,
        jitter_ms,
        loss_permille: loss.min(1000) as u32,
        stall_every_ms: (e0, e1),
        stall_ms: (s0, s1),
        seed,
    })
}

/// Reads a net-client's options; `game_dir` is the install folder when no
/// option names one.
pub fn parse(args: &[String], game_dir: Option<String>) -> Result<Options, String> {
    let mut relay = None;
    let mut join = None;
    let mut host = None;
    let mut name = "Bot".to_string();
    let mut team = None;
    let mut row = None;
    let mut seed = 1u64;
    let mut ticks = 0u32;
    let mut minutes = 0i64;
    let mut map = 0usize;
    let mut hash_log = None;
    let mut report = None;
    let mut code_file = None;
    let mut settings = None;
    let mut resume = false;
    let mut ai = true;
    let mut dedupe = true;
    let mut relay_only = false;
    let mut ice = None;
    let mut direct_shape = None;
    let mut game_dir = game_dir;
    let mut it = args.iter();
    let num = |v: Option<&String>, what: &str| -> Result<u64, String> {
        v.and_then(|s| s.parse().ok())
            .ok_or_else(|| format!("{what} needs a number"))
    };
    let text = |v: Option<&String>, what: &str| -> Result<String, String> {
        v.cloned().ok_or_else(|| format!("{what} needs a value"))
    };
    while let Some(a) = it.next() {
        match a.as_str() {
            "--relay" => relay = Some(text(it.next(), a)?),
            "--host" => host = Some(num(it.next(), a)? as usize),
            "--join" => join = Some(text(it.next(), a)?),
            "--map" => map = num(it.next(), a)? as usize,
            "--name" => name = text(it.next(), a)?,
            "--team" => team = Some(num(it.next(), a)? as u8),
            "--slot" => row = Some(num(it.next(), a)? as usize),
            "--seed" => seed = num(it.next(), a)?,
            "--ticks" => ticks = num(it.next(), a)? as u32,
            "--minutes" => minutes = num(it.next(), a)? as i64,
            "--hash-log" => hash_log = Some(text(it.next(), a)?),
            "--report" => report = Some(text(it.next(), a)?),
            "--code-file" => code_file = Some(text(it.next(), a)?),
            "--settings" => settings = Some(text(it.next(), a)?),
            "--resume" => resume = true,
            "--no-ai" => ai = false,
            "--no-dedupe" => dedupe = false,
            "--relay-only" => relay_only = true,
            "--ice" => ice = Some(text(it.next(), a)?),
            "--direct-shape" => direct_shape = Some(text(it.next(), a)?),
            "--game-dir" => game_dir = Some(text(it.next(), a)?),
            "--synthetic" => game_dir = None,
            other => return Err(format!("unknown option {other}")),
        }
    }
    let role = match (host, join) {
        (Some(persons), None) => Role::Host { persons, map },
        (None, Some(code)) => Role::Join { code },
        _ => return Err("one of --host and --join is needed".into()),
    };
    let relay = relay.ok_or("--relay is needed")?;
    let mut bot = BotConfig::new(&name, &relay, role);
    bot.team = team;
    bot.row = row;
    bot.ai = ai;
    bot.max_ticks = ticks;
    bot.max_game_ms = minutes * 60_000;
    bot.resume = resume;
    bot.dedupe = dedupe;
    // The reconnect wait, jittered by the seed so that peers cut at once do
    // not all come back on the same frame.
    bot.reconnect_after_ms += (seed % 1000) as i64; // nocite: our own harness (up to a second more)
    // The direct path's draws, apart from the relay path's.
    let direct_shape = match direct_shape {
        Some(t) => Some(shape(&t, seed ^ 0x5EED_D1EC)?), // nocite: our own harness (any constant apart from the seed)
        None => None,
    };
    Ok(Options {
        bot,
        seed,
        hash_log,
        report,
        code_file,
        settings,
        game_dir,
        relay_only,
        ice,
        direct_shape,
    })
}

/// How often a running net-client prints a progress line, in milliseconds.
// nocite: our own harness
pub const PROGRESS_MS: i64 = 30_000;

/// A net-client's loop around its bot: the status lines it prints and the
/// checkpoint lines it writes, the same natively and in wasm.
pub struct Driver {
    /// The bot.
    pub bot: Bot,
    said_room: bool,
    said_game: bool,
    lost: u32,
    readmitted: usize,
    written: usize,
    progress: Option<i64>,
}

impl Driver {
    /// A driver of a bot that plays as `cfg` says.
    pub fn new(cfg: BotConfig) -> Self {
        Self {
            bot: Bot::new(cfg),
            said_room: false,
            said_game: false,
            lost: 0,
            readmitted: 0,
            written: 0,
            progress: None,
        }
    }

    /// One step of the bot (after [`Client::update`] and the client's
    /// jobs); returns the status lines to print: `room CODE row R` once the
    /// room is set up, `game slot S house H`, `lost N at tick T`,
    /// `readmitted N at tick T after MS ms`, a progress line now and then.
    pub fn step(&mut self, c: &mut Client, now: i64) -> Vec<String> {
        self.bot.step(c, now);
        let pairs = c.direct_stats();
        if !pairs.is_empty() {
            self.bot.stats.pairs = pairs;
            self.bot.stats.relay_rtt = c.relay_rtt();
        }
        let s = &self.bot.stats;
        let mut out = Vec::new();
        if !self.said_room
            && s.configured
            && let (Some(code), Some(row)) = (&s.code, s.row)
        {
            self.said_room = true;
            out.push(format!("room {code} row {row}"));
        }
        if self.bot.phase == BotPhase::Game
            && !self.said_game
            && let (Some(slot), Some(house)) = (s.slot, s.house)
        {
            self.said_game = true;
            self.progress = Some(now);
            out.push(format!("game slot {slot} house {house}"));
        }
        if s.lost > self.lost {
            self.lost = s.lost;
            out.push(format!("lost {} at tick {}", s.lost, s.ticks));
        }
        if s.rejoined_ms.len() > self.readmitted {
            self.readmitted = s.rejoined_ms.len();
            out.push(format!(
                "readmitted {} at tick {} after {} ms",
                self.readmitted,
                s.ticks,
                s.rejoined_ms[self.readmitted - 1]
            ));
        }
        if let Some(p) = self.progress
            && now - p >= PROGRESS_MS
        {
            self.progress = Some(now);
            out.push(format!(
                "tick {} delay {} stalls {} evidence {}",
                s.ticks,
                s.delay.last().map_or(0, |d| d.1),
                s.stalls,
                s.evidence_max
            ));
        }
        out
    }

    /// Whether the bot reached its game.
    pub fn in_game(&self) -> bool {
        self.said_game
    }

    /// The checkpoint lines not yet taken, `TICK HASH` each.
    pub fn hash_lines(&mut self) -> String {
        let mut s = String::new();
        for (t, h) in &self.bot.stats.checkpoints[self.written..] {
            let _ = writeln!(s, "{t} {}", glasnost_net::sim::hex(h));
        }
        self.written = self.bot.stats.checkpoints.len();
        s
    }

    /// The report: the seed, the numbers and any failure.
    pub fn report(&self, seed: u64) -> String {
        let mut text = format!("seed {seed}\n");
        text.push_str(&self.bot.stats.summary());
        if let Some(why) = self.bot.failure() {
            let _ = writeln!(text, "failure {why}");
        }
        text
    }
}

/// The synthetic install a bot can play without game data: invented rules
/// and map (`glasnost_ai::fixture`) with eight start positions, at the
/// fastest network pacing (index 0).
pub mod synthetic {
    use glasnost_ai::fixture;
    use glasnost_convert::find::Listing;
    use glasnost_fmt::mix::MixEntry;
    use glasnost_map::Map;
    use glasnost_ui::setup::{self as ui_setup, MapEntry, SetupRules};
    use glasnost_vfs::{ArchiveId, FileRef, Vfs};

    use crate::game::client::{Client, Done, Job};
    use crate::game::install::Install;
    use crate::game::session::Session;
    use crate::mapview::load::Loaded;

    /// The synthetic map's content hash as every synthetic client names it.
    pub const MAP_HASH: &str = "map-synthetic-8";

    /// The synthetic skirmish map with six more start waypoints.
    pub fn map8() -> Map {
        let mut m = fixture::map();
        for (k, dx, dy) in [
            (2u16, -14, -18),
            (3, 14, 18),
            (4, -14, 18),
            (5, 14, -18),
            (6, 0, -24),
            (7, 0, 24),
        ] {
            let c = fixture::at(dx, dy);
            m.waypoints.insert(k, (c.x as i16, c.y as i16));
        }
        m
    }

    /// The synthetic install.
    pub fn install() -> Install {
        let file = FileRef {
            archive: ArchiveId(0),
            entry: MixEntry {
                id: 8,
                offset: 0,
                size: 0,
            },
        };
        Install {
            setup: SetupRules {
                money: 2000,
                money_range: (1000, 5000, 100),
                units: 3,
                units_range: (1, 9),
                bases: true,
                flags: [false, true, true, true, true],
                // Index 0: 16 ms ticks in a network game (S-0201 claim 17, F5).
                game_speed: 0,
                countries: vec!["Alphaland".into(), "Betaland".into()],
                colours: vec![[200, 0, 0, 255]; 8],
            },
            maps: vec![(
                Listing {
                    file,
                    archive: "synthetic".into(),
                    name: "synthetic-8".into(),
                    theater: "x".into(),
                    multiplayer: true,
                },
                MapEntry {
                    label: "synthetic-8".into(),
                    starts: 8,
                },
            )],
            map_hashes: vec![MAP_HASH.into()],
            map_info: Vec::new(),
            rules_hash: fixture::rules().hash.clone(),
            names: Default::default(),
            sounds: None,
            speech: None,
            bank: Default::default(),
            cursor_palette: Default::default(),
            strings: None,
            music: None,
            summary: "synthetic".into(),
        }
    }

    /// A client with the synthetic install, drawn on the CPU.
    pub fn client() -> Client {
        let mut c = Client::new();
        c.cpu_map = false;
        c.done(Done::Prepared(Vfs::default(), Ok(Box::new(install()))));
        c
    }

    /// Runs the start job a synthetic client handed out: the game of the
    /// room's setup on the synthetic map, played from the client's slot.
    pub fn run_jobs(c: &mut Client) {
        let Some(Job::Start {
            vfs,
            lobby,
            offered,
            seed,
            local,
            ..
        }) = c.job.take()
        else {
            return;
        };
        let s = ui_setup::game_setup(&lobby, &fixture::rules(), &offered, seed, seed)
            .map_err(|e| e.to_string())
            .and_then(|gs| Loaded::bare(map8(), fixture::rules(), fixture::catalogue(), &gs))
            .and_then(|l| Session::bare(l, MAP_HASH.into(), local));
        c.done(Done::Started(
            vfs,
            Box::new(s),
            glasnost_audio::bank::SoundBank::new(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Vec<String> {
        s.split_whitespace().map(str::to_string).collect()
    }

    /// The options of a host and a guest; the reconnect wait takes the
    /// seed's jitter.
    #[test]
    fn options_parse() {
        let o = parse(
            &args("--relay ws://h:1/ --host 4 --map 2 --team 1 --slot 0 --minutes 30 --seed 7"),
            Some("dir".into()),
        )
        .unwrap();
        assert_eq!(o.bot.role, Role::Host { persons: 4, map: 2 });
        assert_eq!(o.bot.team, Some(1));
        assert_eq!(o.bot.row, Some(0));
        assert_eq!(o.bot.max_game_ms, 30 * 60_000);
        assert_eq!(o.bot.reconnect_after_ms, 2_007);
        assert_eq!(o.game_dir.as_deref(), Some("dir"));
        assert!(o.bot.dedupe && o.bot.ai && !o.bot.resume);
        let o = parse(
            &args(
                "--relay ws://h:1/ --join ABCD --resume --no-dedupe --synthetic --hash-log h.txt",
            ),
            Some("dir".into()),
        )
        .unwrap();
        assert_eq!(
            o.bot.role,
            Role::Join {
                code: "ABCD".into()
            }
        );
        assert!(o.bot.resume && !o.bot.dedupe);
        assert_eq!(o.game_dir, None);
        assert_eq!(o.hash_log.as_deref(), Some("h.txt"));
    }

    /// A missing role, both roles, a missing relay or a bad number are
    /// refused.
    #[test]
    fn bad_options_are_refused() {
        for a in [
            "--relay ws://h:1/",
            "--relay ws://h:1/ --host 4 --join X",
            "--host 4",
            "--relay ws://h:1/ --host four",
            "--relay ws://h:1/ --host 4 --frobnicate",
        ] {
            assert!(parse(&args(a), None).is_err(), "{a}");
        }
    }
}
