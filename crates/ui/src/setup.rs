//! The skirmish setup screen: its model, its mapping to a [`GameSetup`]
//! (S-0412) and its layout.
//!
//! Which values the screen offers and their ranges are interface matters
//! (S-0412 claim 1); the defaults and slider ranges come from the rules
//! ([`SetupRules`], S-0412 Import mapping and Constants).
//!
//! The network lobby (`glasnost_net::lobby`, M9) holds the same [`Setup`]
//! for a room, so the model serializes; there a [`SlotKind::Human`] row is a
//! person's row, taken by one member of the room or open.

use glasnost_rules::layers::{Layers, PLAY_FIXES_NAME};
use glasnost_sim::econ::setup::{self, HouseValues, LobbyChoice, Player, Settings};
use glasnost_sim::rules::SimRules;
use glasnost_sim::world::GameSetup;
use serde::{Deserialize, Serialize};

use crate::draw::{DrawList, Rgba, colour};
use crate::geom::{Point, Rect};
use crate::widget::{self, Targets};

/// The most player rows the screen shows: one human row and up to seven
/// computer rows (S-0412 claim 15).
pub const MAX_SLOTS: usize = 8; // spec: S-0412 claim 15 (at most eight rows)
/// Team numbers offered besides no team (S-0412 claim 15 and Terms).
pub const TEAMS: u8 = 4; // spec: S-0412 claim 15 (no team and four distinct teams)

/// The paced period of game-speed index `speed` in milliseconds: 16 × s for
/// s from 1 to 6, 8 at index 0 (S-0201 claims 4 and 5). The client paces
/// a single-player game's ticks by it; a network game is paced by
/// [`net_period_ms`] (S-0201 claim 17).
pub fn period_ms(speed: u8) -> i64 {
    match speed {
        // spec: S-0201 claim 5 (our engine paces index 0 at a fixed 8 ms)
        0 => 8,
        // spec: S-0201 claims 2 and 4 (s pacing units of 16 ms)
        s => 16 * i64::from(s.min(6)),
    }
}

/// The paced period of a network game's cycle at game-speed index `speed`,
/// in milliseconds: ⌊1000 / cap(s)⌋ (S-0201 claim 17 and Formulas F5, our
/// engine's pacing). The client paces a network game's ticks by it from the
/// start at the room's index and from the cycle after a speed order at the
/// new one, and the relay works out the drop grace period in ticks from it
/// (M9 N5).
pub fn net_period_ms(speed: u8) -> i64 {
    // spec: S-0201 claim 17 and Constants (cap of the agreed tick rate for indices 0 to 6: 60, 45, 30, 20, 15, 12 and 10 ticks per second)
    const CAP: [i64; 7] = [60, 45, 30, 20, 15, 12, 10];
    // spec: S-0201 Formulas F5 (P_net = ⌊1000 / A⌋ ms, in whole milliseconds) and claim 17, Our engine (A = cap(s) for the index in force: no start at 30, no 256-tick schedule, no slowest-machine rate C)
    1000 / CAP[usize::from(speed.min(6))]
}

/// The values and ranges the screen offers, read from the rules by the
/// client.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetupRules {
    /// The default credits setting offered (S-0412 Import mapping).
    pub money: i32,
    /// Credits slider minimum, maximum and step (S-0412 Import mapping).
    pub money_range: (i32, i32, i32),
    /// The default unit-count setting offered.
    pub units: i32,
    /// Unit-count minimum and maximum.
    pub units_range: (i32, i32),
    /// The bases setting (S-0412 claim 1: no lobby control).
    pub bases: bool,
    /// Defaults offered for short game, superweapons, crates, build off
    /// ally and redeploy (S-0412 Import mapping).
    pub flags: [bool; 5],
    /// The game-speed index the control starts at (S-0201 claim 7).
    pub game_speed: u8,
    /// The countries offered (S-0412 claim 15), in registration order.
    pub countries: Vec<String>,
    /// The swatch of each colour slot 0 … 7 (S-0103 claim 10).
    pub colours: Vec<Rgba>,
}

/// A map the screen offers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapEntry {
    /// What the screen shows (never a stock map's name in tests or docs).
    pub label: String,
    /// The number of start positions (S-0009 claim 23).
    pub starts: u32,
}

/// Who plays a slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotKind {
    /// A person: in a skirmish the person at this machine, in a network
    /// room the member who took the row.
    Human,
    /// An AI house (S-0412 claim 12).
    Ai,
    /// Nobody.
    Closed,
}

/// One row of the player list.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Slot {
    /// Who plays it.
    pub kind: SlotKind,
    /// An index into [`SetupRules::countries`], `None` for random
    /// (S-0412 claim 6).
    pub country: Option<usize>,
    /// A colour slot 0 … 7, `None` for random (S-0412 claim 7).
    pub colour: Option<u8>,
    /// Team 1 … [`TEAMS`], `None` for no team (S-0412 claim 8).
    pub team: Option<u8>,
    /// A start position (0-based entry of the start list, S-0009 claim
    /// 17), `None` for random (S-0009 claim 19).
    pub start: Option<u32>,
}

impl Slot {
    /// A closed row, every choice on random or none.
    pub const fn closed() -> Self {
        Self {
            kind: SlotKind::Closed,
            country: None,
            colour: None,
            team: None,
            start: None,
        }
    }
}

/// The yes/no settings of the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flag {
    /// Short game (S-0414 claim 2).
    ShortGame = 0,
    /// Superweapons (S-0412 claim 9).
    Superweapons = 1,
    /// Crates (S-0412 claim 10).
    Crates = 2,
    /// Build off ally (S-0406 claim 4).
    BuildOffAlly = 3,
    /// Redeploy (S-0415 claim 9).
    Redeploy = 4,
}

const FLAGS: [(Flag, &str); 5] = [
    (Flag::ShortGame, "Short game"),
    (Flag::Superweapons, "Superweapons"),
    (Flag::Crates, "Crates"),
    (Flag::BuildOffAlly, "Build off ally"),
    (Flag::Redeploy, "Redeploy"),
];

/// The screen's state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Setup {
    /// The chosen map, an index into the maps offered.
    pub map: usize,
    /// The player rows.
    pub slots: [Slot; MAX_SLOTS],
    /// The credits setting.
    pub credits: i32,
    /// The unit-count setting.
    pub units: i32,
    /// The yes/no settings, by [`Flag`].
    pub flags: [bool; 5],
    /// The speed control's position 0 (slowest) … 6 (fastest).
    pub speed: u8,
    /// Our own rules patch layer *Pathing and other fixes* (M8 step C7,
    /// `glasnost_rules::layers`): not a setting of the original's lobby.
    /// In a network room the host sets it, like the game-wide values.
    pub fixes: bool,
}

/// Whether a new setup starts with *Pathing and other fixes* on: the main
/// session's decision of 2026-10-25 (docs/m8-plan.md). The measurement
/// tools build their setups with it off.
// nocite: our own client setting (the main session's decision, docs/m8-plan.md)
pub const FIXES_BY_DEFAULT: bool = true;

/// Why the setup cannot start yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Problem {
    /// No map to play.
    NoMap,
    /// Fewer than two players.
    TooFewPlayers,
    /// More players than the map has start positions.
    TooManyPlayers,
    /// Two players chose the same start position.
    SameStart,
    /// Two players chose the same colour.
    SameColour,
    /// A country index outside the list.
    NoCountry,
    /// A watch game whose computers are all on one team.
    AllAllied,
}

impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NoMap => "No multiplayer map found",
            Self::TooFewPlayers => "At least two players are needed",
            Self::TooManyPlayers => "More players than start positions",
            Self::SameStart => "Two players share a start position",
            Self::SameColour => "Two players share a colour",
            Self::NoCountry => "Unknown country",
            Self::AllAllied => "Computers on one team cannot end the game",
        })
    }
}

/// The values the client turns into a game: the lobby side of
/// [`setup::skirmish`] and [`setup::resolve_lobby`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lobby {
    /// The players in house order.
    pub players: Vec<Player>,
    /// Their random choices, in the same order.
    pub choices: Vec<LobbyChoice>,
    /// The game-wide settings.
    pub settings: Settings,
    /// The unit-count setting.
    pub unit_count: i32,
    /// The bases setting.
    pub bases: bool,
    /// The game-speed index.
    pub game_speed: u8,
    /// The map index.
    pub map: usize,
    /// Our own rules patch layers the game's rules are resolved with (M8
    /// step C7).
    pub layers: Layers,
}

/// The name of the house of the person (the client renames it as it
/// likes).
pub const HUMAN_NAME: &str = "Player";

/// The name of the n-th person's house (0-based) in a game of several
/// persons: [`HUMAN_NAME`], "Player 2", …
pub fn person_name(n: usize) -> String {
    if n == 0 {
        HUMAN_NAME.into()
    } else {
        format!("{HUMAN_NAME} {}", n + 1)
    }
}

/// The name of the n-th AI house (0-based): "Computer", "Computer 2", …
pub fn ai_name(n: usize) -> String {
    if n == 0 {
        "Computer".into()
    } else {
        format!("Computer {}", n + 1)
    }
}

/// Steps `v` by `d` through `0 .. n` with an extra "random" state (`None`)
/// before 0.
fn cycle_opt(v: Option<usize>, d: i32, n: usize) -> Option<usize> {
    let states = n as i32 + 1;
    let i = v.map_or(0, |x| x as i32 + 1);
    let j = (i + d).rem_euclid(states);
    if j == 0 { None } else { Some(j as usize - 1) }
}

/// The player rows a map offers (S-0412 claim 15): its start-waypoint
/// count (`MapEntry::starts`, S-0009 claim 23), at most [`MAX_SLOTS`].
/// Listed maps have at least two starts, so the random-map player-count
/// fallback for a count of zero never applies; with no map, every row.
pub fn rows(map: Option<&MapEntry>) -> usize {
    map.map_or(MAX_SLOTS, |m| (m.starts as usize).min(MAX_SLOTS))
}

impl Setup {
    /// The screen's first state: the person in slot 1, one AI in slot 2,
    /// every value at the rules' default.
    pub fn new(rules: &SetupRules) -> Self {
        let mut slots = [Slot::closed(); MAX_SLOTS];
        slots[0].kind = SlotKind::Human;
        slots[1].kind = SlotKind::Ai;
        Self {
            map: 0,
            slots,
            credits: rules.money,
            units: rules.units,
            flags: rules.flags,
            // spec: S-0201 claim 7 (position p sets index 6 − p)
            speed: 6 - rules.game_speed.min(6),
            fixes: FIXES_BY_DEFAULT,
        }
    }

    /// The game-speed index of the speed control (S-0201 claim 7).
    pub fn game_speed(&self) -> u8 {
        // spec: S-0201 claim 7 (seven positions, slowest to fastest, set indices 6 … 0)
        6 - self.speed.min(6)
    }

    /// The occupied slots.
    pub fn players(&self) -> impl Iterator<Item = (usize, &Slot)> {
        self.slots
            .iter()
            .enumerate()
            .filter(|(_, s)| s.kind != SlotKind::Closed)
    }

    /// Why the setup cannot start, if it cannot.
    pub fn problem(&self, rules: &SetupRules, maps: &[MapEntry]) -> Option<Problem> {
        self.check(rules.countries.len(), maps.get(self.map))
    }

    /// Why the setup cannot start on `map` with `countries` countries
    /// offered, if it cannot ([`Setup::problem`] without the rest of the
    /// rules, for the network lobby, which knows only these two).
    pub fn check(&self, countries: usize, map: Option<&MapEntry>) -> Option<Problem> {
        let Some(map) = map else {
            return Some(Problem::NoMap);
        };
        let n = self.players().count();
        if n < 2 {
            return Some(Problem::TooFewPlayers);
        }
        if n as u32 > map.starts {
            return Some(Problem::TooManyPlayers);
        }
        let ps: Vec<&Slot> = self.players().map(|(_, s)| s).collect();
        for (i, a) in ps.iter().enumerate() {
            if a.country.is_some_and(|c| c >= countries) {
                return Some(Problem::NoCountry);
            }
            for b in &ps[i + 1..] {
                if a.start.is_some() && a.start == b.start {
                    return Some(Problem::SameStart);
                }
                // spec: S-0412 claim 7 (a slot fixed by one player is not free for another)
                if a.colour.is_some() && a.colour == b.colour {
                    return Some(Problem::SameColour);
                }
            }
        }
        if self.watching() {
            // spec: S-0412 claim 8 (players with the same lobby team number start allied; no team is no team) and S-0414 claim 6 (the end test runs only after a defeat, and the game ends when no hostile pair remains)
            // design: watch mode (lead request 2026-11-02, an extra not in the original): computers all on one team start with no hostile pair, so no one would be watched to win; such a watch game is refused
            let first = ps.first().and_then(|s| s.team);
            if first.is_some() && ps.iter().all(|s| s.team == first) {
                return Some(Problem::AllAllied);
            }
        }
        None
    }

    /// Whether this setup starts a watch game: players, none of them a
    /// person's (S-0412 Terms, *human participant*).
    ///
    /// design: watch mode (lead request 2026-11-02, an extra not in the
    /// original): the game is played by computers only and watched.
    pub fn watching(&self) -> bool {
        self.players().next().is_some() && self.players().all(|(_, s)| s.kind == SlotKind::Ai)
    }

    /// Gives every occupied row whose colour is on "random", person or
    /// computer, its colour slot (S-0412 claim 7), in increasing row order
    /// (S-0009 claim 19, our engine's rule for a network game), from a
    /// stream of its own seeded from `seed`. The network room does this at
    /// the start, before [`Setup::house_order`], because a person's place in
    /// the house order follows the slot it was given (S-0009 claim 19). Our
    /// lobby has no observer rows, so every occupied row draws. With at
    /// most eight rows and the fixed colours distinct ([`Setup::check`]),
    /// every row finds a free slot.
    pub fn resolve_colours(&mut self, seed: u32) {
        // nocite: our own netcode (the room's colour draws take a stream of their own, seeded one above the game seed, so that they do not repeat the lobby stream's country draws; S-0412 claim 7 leaves how the setup draws to our engine)
        let mut rng = glasnost_sim::rng::Rng::new(seed.wrapping_add(1));
        // spec: S-0412 claim 7 (a slot is free when no player has it fixed and no player resolved earlier was given it)
        let mut taken: Vec<u8> = self.players().filter_map(|(_, s)| s.colour).collect();
        let rows: Vec<usize> = self
            .players()
            .filter(|(_, s)| s.colour.is_none())
            .map(|(i, _)| i)
            .collect();
        // spec: S-0009 claim 19 (every player on "random", person or computer, draws in increasing lobby-slot order, design)
        for i in rows {
            // spec: S-0412 claim 7 (the first free draw of R(0, 7)); the half-span switch cannot change a draw over 0 … 7 (S-0202 claim 5)
            if let Some(c) = setup::draw_colour(&mut rng, &taken, true) {
                taken.push(c);
                self.slots[i].colour = Some(c);
            }
        }
    }

    /// The occupied rows in house order (S-0009 claim 19): the persons in
    /// increasing order of their colour slot, the earlier row first on a
    /// tie (design), then the computer rows in row order. A person whose
    /// colour was on "random" is placed by the slot it was given, as if it
    /// had chosen it, so a setup of several persons is resolved first
    /// ([`Setup::resolve_colours`], as the network room does). A person
    /// still on "random" comes after those with a slot; that arises only
    /// with one person, as in a skirmish, whose order is the row order.
    /// Observers would come between the persons and the computers (S-0009
    /// claim 19, design); our lobby has none.
    pub fn house_order(&self) -> Vec<usize> {
        let mut persons: Vec<(Option<u8>, usize)> = self
            .players()
            .filter(|(_, s)| s.kind == SlotKind::Human)
            .map(|(i, s)| (s.colour, i))
            .collect();
        // spec: S-0009 claim 19 (persons by increasing colour slot, a given slot as if chosen; ties to the earlier lobby slot, design; every person's house before every computer player's)
        persons.sort_by_key(|&(c, i)| (c.is_none(), c, i));
        persons
            .into_iter()
            .map(|(_, i)| i)
            // spec: S-0009 claim 19 (then the computer players in lobby-slot order)
            .chain(
                self.players()
                    .filter(|(_, s)| s.kind == SlotKind::Ai)
                    .map(|(i, _)| i),
            )
            .collect()
    }

    /// The lobby values of this setup (S-0412 claim 1), the players in
    /// house order ([`Setup::house_order`]).
    pub fn lobby(&self, rules: &SetupRules, maps: &[MapEntry]) -> Result<Lobby, Problem> {
        if let Some(p) = self.problem(rules, maps) {
            return Err(p);
        }
        let mut players = Vec::new();
        let mut choices = Vec::new();
        let mut ais = 0;
        let mut persons = 0;
        for i in self.house_order() {
            let s = &self.slots[i];
            let ai = s.kind == SlotKind::Ai;
            let name = if ai {
                ais += 1;
                ai_name(ais - 1)
            } else {
                persons += 1;
                person_name(persons - 1)
            };
            players.push(Player {
                name,
                // A random country is resolved by resolve_lobby; the
                // placeholder is overwritten there (S-0412 claim 6).
                country: s
                    .country
                    .and_then(|c| rules.countries.get(c))
                    .or(rules.countries.first())
                    .cloned()
                    .unwrap_or_default(),
                // spec: S-0009 claim 19 (no start position chosen: random)
                start: s.start,
                // spec: S-0412 claim 12 (AI houses take the computer slots)
                ai,
                values: HouseValues {
                    // spec: S-0412 claim 8 (lobby team)
                    team: s.team,
                    handicap: None,
                    colour: s.colour,
                    observer: false,
                },
            });
            choices.push(LobbyChoice {
                // spec: S-0412 claims 6 and 7 (random country, random colour)
                country: s.country.is_none(),
                colour: s.colour.is_none(),
            });
        }
        let f = |x: Flag| Some(self.flags[x as usize]);
        Ok(Lobby {
            players,
            choices,
            // spec: S-0412 claim 1 (the lobby's setup values)
            settings: Settings {
                credits: Some(self.credits),
                short_game: f(Flag::ShortGame),
                superweapons: f(Flag::Superweapons),
                crates: f(Flag::Crates),
                build_off_ally: f(Flag::BuildOffAlly),
                redeploy: f(Flag::Redeploy),
                // spec: S-0412 claim 1 (bridge destruction is always on in a skirmish; no lobby control)
                bridge_destruction: Some(true),
                all_base_units: false,
                houses: Vec::new(),
            },
            unit_count: self.units,
            // spec: S-0412 claim 1 (the bases setting has no lobby control)
            bases: rules.bases,
            game_speed: self.game_speed(),
            map: self.map,
            layers: Layers {
                play_fixes: self.fixes,
            },
        })
    }

    /// Applies one step of a control.
    pub fn apply(&mut self, hit: &SetupHit, rules: &SetupRules, maps: &[MapEntry]) {
        let step =
            |v: i32, d: i32, lo: i32, hi: i32, s: i32| (v + d * s.max(1)).clamp(lo, hi.max(lo));
        match *hit {
            SetupHit::Map(d) if !maps.is_empty() => {
                self.map = (self.map as i32 + d).rem_euclid(maps.len() as i32) as usize;
                // Start positions past the new map's count become random.
                let starts = maps[self.map].starts;
                for s in &mut self.slots {
                    if s.start.is_some_and(|x| x >= starts) {
                        s.start = None;
                    }
                }
                // spec: S-0412 claim 15 (computer rows beyond the new map's row count are emptied and hidden)
                let rows = rows(maps.get(self.map));
                for s in self.slots.iter_mut().skip(rows) {
                    if s.kind == SlotKind::Ai {
                        *s = Slot::closed();
                    }
                }
            }
            SetupHit::Kind(0, d) if d != 0 => {
                // design: watch mode (lead request 2026-11-02, an extra not in the original): the first row is the person's or a computer's; with no person's row the game is watched
                self.slots[0].kind = match self.slots[0].kind {
                    SlotKind::Human => SlotKind::Ai,
                    _ => SlotKind::Human,
                };
            }
            SetupHit::Kind(i, d) if i > 0 => {
                // Every other row is a computer's or closed (interim UI rule).
                let order = [SlotKind::Ai, SlotKind::Closed];
                let k = order
                    .iter()
                    .position(|k| *k == self.slots[i].kind)
                    .unwrap_or(0) as i32;
                self.slots[i].kind = order[(k + d).rem_euclid(2) as usize];
            }
            SetupHit::Country(i, d) => {
                let s = &mut self.slots[i];
                s.country = cycle_opt(s.country, d, rules.countries.len());
            }
            SetupHit::Colour(i, d) => {
                let s = &mut self.slots[i];
                s.colour = cycle_opt(s.colour.map(usize::from), d, 8).map(|c| c as u8);
            }
            SetupHit::Team(i, d) => {
                let s = &mut self.slots[i];
                s.team = cycle_opt(s.team.map(|t| t as usize - 1), d, TEAMS as usize)
                    .map(|t| t as u8 + 1);
            }
            SetupHit::Start(i, d) => {
                let n = maps.get(self.map).map_or(0, |m| m.starts) as usize;
                let s = &mut self.slots[i];
                s.start = cycle_opt(s.start.map(|x| x as usize), d, n).map(|x| x as u32);
            }
            SetupHit::Credits(d) => {
                let (lo, hi, st) = rules.money_range;
                self.credits = step(self.credits, d, lo, hi, st);
            }
            SetupHit::Units(d) => {
                let (lo, hi) = rules.units_range;
                self.units = step(self.units, d, lo, hi, 1);
            }
            SetupHit::Toggle(f) => self.flags[f as usize] ^= true,
            SetupHit::Fixes => self.fixes ^= true,
            SetupHit::Speed(d) => self.speed = (self.speed as i32 + d).clamp(0, 6) as u8,
            _ => {}
        }
    }

    /// Draws the screen and lays out its targets.
    pub fn view(
        &self,
        size: (i32, i32),
        rules: &SetupRules,
        maps: &[MapEntry],
        hover: Option<Point>,
    ) -> (DrawList, Targets<SetupHit>) {
        self.view_framed(size, rules, maps, hover, None)
    }

    /// The panel of the screen in a window of `size` (our own layout).
    pub fn panel_rect(size: (i32, i32)) -> Rect {
        let (w, h) = size;
        Rect::new(
            (w - 620).max(0) / 2,
            (h - 440).max(0) / 2,
            620.min(w),
            440.min(h),
        )
    }

    /// Draws the screen as a skirmish (`room` none) or as a network room,
    /// where the rows carry the members' names and ready marks and only
    /// what this member may change has targets.
    pub fn view_framed(
        &self,
        size: (i32, i32),
        rules: &SetupRules,
        maps: &[MapEntry],
        hover: Option<Point>,
        room: Option<&RoomFrame>,
    ) -> (DrawList, Targets<SetupHit>) {
        let mut d = DrawList::new();
        let mut t = Targets::new();
        let (w, h) = size;
        d.fill(Rect::new(0, 0, w, h), colour::BACKDROP);
        let outer = Self::panel_rect(size);
        let title = room.map_or("SKIRMISH SETUP", |r| r.title.as_str());
        widget::panel(&mut d, outer, Some(title));
        let hot = |r: Rect| hover.is_some_and(|p| r.contains(p));
        // In a room only the host changes the map and the game-wide values.
        let game_wide = room.is_none_or(|r| r.host);
        let choice = |d: &mut DrawList,
                      t: &mut Targets<SetupHit>,
                      r: Rect,
                      label: &str,
                      value: &str,
                      mk: &dyn Fn(i32) -> SetupHit| {
            let (a, b) = widget::choice(d, r, label, value, game_wide);
            if game_wide {
                t.push_choice(a, mk(-1), b, mk(1));
            }
        };
        let x0 = outer.x + 10;
        let mut y = outer.y + 24;
        let map_label = maps.get(self.map).map_or("(none)".to_owned(), |m| {
            format!("{} ({} starts)", m.label, m.starts)
        });
        choice(
            &mut d,
            &mut t,
            Rect::new(x0, y, 600, 16),
            "Map",
            &map_label,
            &SetupHit::Map,
        );
        y += 24;
        // Player table header.
        let cols = [
            (0, 70, "Player"),
            (74, 140, "Country"),
            (218, 90, "Colour"),
            (312, 70, "Team"),
            (386, 90, "Start"),
        ];
        for (cx, cw, name) in cols {
            d.text_left(Rect::new(x0 + cx, y, cw, 12), name, colour::DIM);
        }
        y += 14;
        // spec: S-0412 claim 15 (only the map's player rows are shown)
        for (i, s) in self.slots.iter().enumerate().take(rows(maps.get(self.map))) {
            let row = Rect::new(x0, y, 600, 16);
            d.fill(
                row,
                if i % 2 == 0 {
                    colour::PANEL
                } else {
                    colour::RAISED
                },
            );
            let name = room.and_then(|r| r.names[i].as_deref());
            let kind = match s.kind {
                SlotKind::Human if room.is_some() => name.unwrap_or("Open"),
                SlotKind::Human => "You",
                SlotKind::Ai => "AI",
                SlotKind::Closed => "Closed",
            };
            let r = Rect::new(x0, y, 70, 16);
            // In a skirmish every row's kind is set (the first row's to the
            // person or a computer, for a watch game); in a room the host
            // sets the kind of every row nobody has taken.
            let kind_editable = match room {
                None => true,
                Some(f) => f.host && name.is_none(),
            };
            if !kind_editable {
                let mine = room.is_none_or(|f| f.mine == Some(i));
                let ink = if mine { colour::ACCENT } else { colour::TEXT };
                d.text_left(r, &crate::font::fit(kind, r.w), ink);
            } else {
                let (a, b) = widget::choice(&mut d, r, "", kind, true);
                t.push_choice(a, SetupHit::Kind(i, -1), b, SetupHit::Kind(i, 1));
            }
            if let Some(f) = room {
                let (mark, ink) = if f.host_row == Some(i) {
                    ("Host", colour::ACCENT)
                } else if f.ready[i] {
                    ("Ready", colour::GOOD)
                } else if name.is_some() {
                    ("Not ready", colour::DIM)
                } else {
                    ("", colour::DIM)
                };
                d.text_left(Rect::new(x0 + 500, y, 98, 16), mark, ink);
            }
            // In a room a member sets its own row, the host the computers'.
            let editable =
                room.is_none_or(|f| f.mine == Some(i) || (f.host && s.kind == SlotKind::Ai));
            if s.kind != SlotKind::Closed {
                let country = s
                    .country
                    .and_then(|c| rules.countries.get(c))
                    .map_or("Random", String::as_str);
                let colour_name = s
                    .colour
                    .map_or("Random".to_owned(), |c| format!("Slot {}", c + 1));
                let team = s.team.map_or("None".to_owned(), |c| format!("Team {c}"));
                let start = s
                    .start
                    .map_or("Random".to_owned(), |c| format!("Start {}", c + 1));
                let cells: [Cell; 4] = [
                    (74, 140, country.to_owned(), SetupHit::Country),
                    (218, 90, colour_name, SetupHit::Colour),
                    (312, 70, team, SetupHit::Team),
                    (386, 90, start, SetupHit::Start),
                ];
                for (cx, cw, value, mk) in cells {
                    let r = Rect::new(x0 + cx, y, cw, 16);
                    let (a, b) = widget::choice(&mut d, r, "", &value, editable);
                    if editable {
                        t.push_choice(a, mk(i, -1), b, mk(i, 1));
                    }
                }
                if let Some(c) = s.colour.and_then(|c| rules.colours.get(c as usize)) {
                    d.fill(Rect::new(x0 + 480, y + 3, 10, 10), *c);
                    d.frame(Rect::new(x0 + 480, y + 3, 10, 10), colour::EDGE);
                }
            }
            y += 18;
        }
        y += 8;
        let half = 295;
        choice(
            &mut d,
            &mut t,
            Rect::new(x0, y, half, 16),
            "Credits",
            &self.credits.to_string(),
            &SetupHit::Credits,
        );
        choice(
            &mut d,
            &mut t,
            Rect::new(x0 + 305, y, half, 16),
            "Units",
            &self.units.to_string(),
            &SetupHit::Units,
        );
        y += 20;
        let speed = format!("{} of 7", self.speed + 1);
        choice(
            &mut d,
            &mut t,
            Rect::new(x0, y, half, 16),
            "Game speed",
            &speed,
            &SetupHit::Speed,
        );
        y += 24;
        // The original's yes/no settings, then our own fixes layer in the
        // grid's last place.
        let boxes = FLAGS
            .iter()
            .map(|(f, name)| (self.flags[*f as usize], *name, SetupHit::Toggle(*f)))
            .chain([(self.fixes, PLAY_FIXES_NAME, SetupHit::Fixes)]);
        for (k, (on, name, hit)) in boxes.enumerate() {
            let r = Rect::new(x0 + (k as i32 % 3) * 200, y + (k as i32 / 3) * 20, 190, 16);
            let box_r = Rect::new(r.x, r.y + 2, 12, 12);
            d.fill(box_r, colour::RAISED);
            d.frame(box_r, if hot(r) { colour::ACCENT } else { colour::EDGE });
            if on {
                d.fill(box_r.inset(3), colour::ACCENT);
            }
            let ink = if game_wide { colour::TEXT } else { colour::DIM };
            d.text_left(
                Rect::new(r.x + 14, r.y, r.w - 14, 16),
                &crate::font::fit(name, r.w - 14),
                ink,
            );
            if game_wide {
                t.push(r, hit);
            }
        }
        let problem = self.problem(rules, maps);
        let by = outer.bottom() - 30;
        let note = match (room.and_then(|f| f.note.as_ref()), &problem) {
            (Some((text, bad)), _) => Some((text.clone(), *bad)),
            (None, Some(p)) => Some((p.to_string(), true)),
            (None, None) => None,
        };
        if let Some((text, bad)) = note {
            let ink = if bad { colour::BAD } else { colour::TEXT };
            let text = crate::font::fit(&text, 360);
            d.text_left(Rect::new(x0, by, 360, 20), &text, ink);
        }
        let back = Rect::new(outer.right() - 230, by, 100, 20);
        let go = Rect::new(outer.right() - 120, by, 110, 20);
        let (back_label, go_label, go_on) = match room {
            None => ("Back", "Start game", problem.is_none()),
            // The host starts a setup without a problem; the others say
            // ready or not.
            Some(f) => (
                "Leave",
                f.go.as_str(),
                f.go_enabled && (!f.host || problem.is_none()),
            ),
        };
        widget::button(&mut d, back, back_label, hot(back), true);
        widget::button(&mut d, go, go_label, hot(go), go_on);
        t.push(back, SetupHit::Back);
        if go_on {
            t.push(go, SetupHit::Start_);
        }
        (d, t)
    }
}

/// How the setup screen shows a network room (our own layout): the
/// client fills it from the relay's view of the room.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RoomFrame {
    /// The panel's title (the room code).
    pub title: String,
    /// The name of the member on each person's row; `None` while open.
    pub names: [Option<String>; MAX_SLOTS],
    /// Each row's member's ready mark.
    pub ready: [bool; MAX_SLOTS],
    /// The host's row.
    pub host_row: Option<usize>,
    /// This member's row.
    pub mine: Option<usize>,
    /// Whether this member is the host.
    pub host: bool,
    /// The go button's label ("Start game", "Ready", "Not ready").
    pub go: String,
    /// Whether the go button may be pressed.
    pub go_enabled: bool,
    /// A line in place of the setup's problem: a refusal, a missing map,
    /// what the room waits for; and whether it is bad.
    pub note: Option<(String, bool)>,
}

/// A column of a player row: offset, width, value and target maker.
type Cell = (i32, i32, String, fn(usize, i32) -> SetupHit);

/// A target of the setup screen; the `i32` is the step (−1 or +1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetupHit {
    /// The map choice.
    Map(i32),
    /// Who plays slot `.0`.
    Kind(usize, i32),
    /// Slot `.0`'s country.
    Country(usize, i32),
    /// Slot `.0`'s colour.
    Colour(usize, i32),
    /// Slot `.0`'s team.
    Team(usize, i32),
    /// Slot `.0`'s start position.
    Start(usize, i32),
    /// The credits setting.
    Credits(i32),
    /// The unit-count setting.
    Units(i32),
    /// A yes/no setting.
    Toggle(Flag),
    /// Our own *Pathing and other fixes* (M8 step C7).
    Fixes,
    /// The speed control.
    Speed(i32),
    /// Back to the main menu.
    Back,
    /// Start the game.
    Start_,
}

/// Builds the game setup of a lobby (S-0412): [`setup::skirmish`], the
/// lobby's bases and speed values, the random choices resolved from the
/// lobby's own stream (claims 6 and 7) and the checks of claims 11 and 13.
pub fn game_setup(
    lobby: &Lobby,
    rules: &SimRules,
    offered: &[String],
    seed: u32,
    lobby_seed: u32,
) -> Result<GameSetup, setup::SetupError> {
    let mut g = setup::skirmish(
        rules,
        &lobby.players,
        lobby.settings.clone(),
        lobby.unit_count,
        seed,
    );
    // spec: S-0412 claim 1 (the bases setting and game speed are setup values)
    g.bases = lobby.bases;
    // spec: S-0201 claim 7 (the lobby's index replaces the game speed at skirmish start)
    g.game_speed = lobby.game_speed;
    // spec: S-0412 claims 6 and 7 (random country and colour, from the lobby's own stream)
    setup::resolve_lobby(&mut g, rules, &lobby.choices, offered, lobby_seed)?;
    setup::check(&g, rules)?;
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: S-0201 Formulas F5 table and claim 17 (a network game's period ⌊1000 / cap(s)⌋: 16, 22, 33, 50, 66, 83 and 100 ms for indices 0 to 6, against the single-player 8 and 16 × s)
    #[test]
    fn network_pacing_follows_the_cap_table() {
        let net: Vec<i64> = (0..=6).map(net_period_ms).collect();
        assert_eq!(net, [16, 22, 33, 50, 66, 83, 100]);
        let single: Vec<i64> = (0..=6).map(period_ms).collect();
        assert_eq!(single, [8, 16, 32, 48, 64, 80, 96]);
        assert_eq!(net_period_ms(9), 100, "out of range: the slowest");
    }

    fn rules() -> SetupRules {
        SetupRules {
            money: 1200,
            money_range: (1000, 3000, 100),
            units: 3,
            units_range: (1, 5),
            bases: true,
            flags: [false, true, true, true, true],
            game_speed: 4,
            countries: vec!["Alphaland".into(), "Betaland".into()],
            colours: vec![[200, 0, 0, 255]; 8],
        }
    }

    fn maps() -> Vec<MapEntry> {
        vec![
            MapEntry {
                label: "map A".into(),
                starts: 2,
            },
            MapEntry {
                label: "map B".into(),
                starts: 4,
            },
        ]
    }

    #[test]
    fn first_state_takes_the_rules_defaults() {
        // spec: S-0412 claim 1 (defaults offered come from the rules)
        let s = Setup::new(&rules());
        assert_eq!(s.credits, 1200);
        assert_eq!(s.units, 3);
        assert_eq!(s.flags, [false, true, true, true, true]);
        // spec: S-0201 claim 7 (the control starts at the lobby default index)
        assert_eq!(s.game_speed(), 4);
        assert_eq!(s.speed, 2);
        assert_eq!(s.players().count(), 2);
        assert_eq!(s.problem(&rules(), &maps()), None);
    }

    /// Our own fixes layer (M8 step C7): on in a new setup, toggled by its
    /// box, and carried into the lobby values as the game's layers.
    #[test]
    fn the_fixes_start_on_toggle_and_reach_the_lobby() {
        let r = rules();
        let mut s = Setup::new(&r);
        assert!(s.fixes, "on by default (docs/m8-plan.md)");
        assert_eq!(s.lobby(&r, &maps()).unwrap().layers, Layers::PLAY_FIXES);
        s.apply(&SetupHit::Fixes, &r, &maps());
        assert!(!s.fixes);
        assert_eq!(s.lobby(&r, &maps()).unwrap().layers, Layers::REFERENCE);
        assert_eq!(s.flags, r.flags, "the original's settings are untouched");
    }

    #[test]
    fn speed_positions_map_to_indices_six_to_zero() {
        // spec: S-0201 claim 7 (positions slowest to fastest set indices 6, 5, …, 0)
        let mut s = Setup::new(&rules());
        for p in 0..=6u8 {
            s.speed = p;
            assert_eq!(s.game_speed(), 6 - p);
        }
        s.apply(&SetupHit::Speed(5), &rules(), &maps());
        assert_eq!(s.speed, 6);
    }

    #[test]
    fn sliders_step_and_clamp() {
        let r = rules();
        let mut s = Setup::new(&r);
        s.apply(&SetupHit::Credits(1), &r, &maps());
        assert_eq!(s.credits, 1300);
        for _ in 0..100 {
            s.apply(&SetupHit::Credits(1), &r, &maps());
        }
        assert_eq!(s.credits, 3000);
        s.apply(&SetupHit::Units(-1), &r, &maps());
        s.apply(&SetupHit::Units(-1), &r, &maps());
        s.apply(&SetupHit::Units(-1), &r, &maps());
        assert_eq!(s.units, 1);
    }

    #[test]
    fn random_states_cycle_with_the_values() {
        let r = rules();
        let mut s = Setup::new(&r);
        s.apply(&SetupHit::Country(1, 1), &r, &maps());
        assert_eq!(s.slots[1].country, Some(0));
        s.apply(&SetupHit::Country(1, 1), &r, &maps());
        s.apply(&SetupHit::Country(1, 1), &r, &maps());
        assert_eq!(s.slots[1].country, None);
        s.apply(&SetupHit::Colour(0, -1), &r, &maps());
        assert_eq!(s.slots[0].colour, Some(7));
        s.apply(&SetupHit::Team(0, 1), &r, &maps());
        assert_eq!(s.slots[0].team, Some(1));
        s.apply(&SetupHit::Start(0, -1), &r, &maps());
        assert_eq!(s.slots[0].start, Some(1));
        // The first row is the person's or a computer's, never closed.
        s.apply(&SetupHit::Kind(0, 1), &r, &maps());
        assert_eq!(s.slots[0].kind, SlotKind::Ai);
        s.apply(&SetupHit::Kind(0, -1), &r, &maps());
        assert_eq!(s.slots[0].kind, SlotKind::Human);
        s.apply(&SetupHit::Kind(2, 1), &r, &maps());
        assert_eq!(s.slots[2].kind, SlotKind::Ai);
    }

    /// The setup of a watch game: the first row a computer's too.
    fn watch_setup(r: &SetupRules) -> Setup {
        let mut s = Setup::new(r);
        s.apply(&SetupHit::Kind(0, 1), r, &maps());
        s
    }

    // design: watch mode (lead request 2026-11-02, an extra not in the original): with the first row a computer's and no person's row, the setup starts a watch game
    #[test]
    fn the_first_row_can_be_a_computer_for_a_watch_game() {
        let r = rules();
        let s = Setup::new(&r);
        assert!(!s.watching());
        let s = watch_setup(&r);
        assert_eq!(s.slots[0].kind, SlotKind::Ai);
        assert!(s.watching());
        assert_eq!(s.problem(&r, &maps()), None);
        // The skirmish view offers the first row's kind control.
        let (_, t) = s.view((800, 600), &r, &maps(), None);
        assert!(t.rect_of(&SetupHit::Kind(0, 1)).is_some());
        // Closing every row leaves no player and no watch game.
        let mut none = s.clone();
        for slot in &mut none.slots {
            slot.kind = SlotKind::Closed;
        }
        assert!(!none.watching());
        assert_eq!(none.problem(&r, &maps()), Some(Problem::TooFewPlayers));
    }

    // spec: S-0412 claim 8 (the same lobby team number starts allied; no team is not a team) and S-0414 claim 6 (the end test runs only after a defeat)
    // design: watch mode (lead request 2026-11-02, an extra not in the original): computers all on one team are refused
    #[test]
    fn a_watch_game_of_one_team_is_refused() {
        let r = rules();
        let mut s = watch_setup(&r);
        s.slots[0].team = Some(2);
        s.slots[1].team = Some(2);
        assert_eq!(s.problem(&r, &maps()), Some(Problem::AllAllied));
        assert!(s.lobby(&r, &maps()).is_err());
        // Another team, or no team, gives a hostile pair.
        s.slots[1].team = Some(3);
        assert_eq!(s.problem(&r, &maps()), None);
        s.slots[1].team = None;
        s.slots[0].team = None;
        assert_eq!(
            s.problem(&r, &maps()),
            None,
            "two on no team are not allied"
        );
        // With a person's row the rule does not apply.
        let mut p = Setup::new(&r);
        p.slots[0].team = Some(1);
        p.slots[1].team = Some(1);
        assert_eq!(p.problem(&r, &maps()), None);
    }

    // spec: S-0009 claim 19 (the persons first, then the computer players in lobby-slot order)
    #[test]
    fn with_no_person_the_house_order_is_the_computer_rows() {
        let r = rules();
        let mut s = watch_setup(&r);
        s.apply(&SetupHit::Map(1), &r, &maps());
        s.slots[3].kind = SlotKind::Ai;
        s.slots[0].colour = Some(5);
        s.slots[3].colour = Some(1);
        assert_eq!(s.house_order(), vec![0, 1, 3]);
        let lobby = s.lobby(&r, &maps()).expect("a watch game starts");
        assert!(lobby.players.iter().all(|p| p.ai));
        let names: Vec<String> = lobby.players.iter().map(|p| p.name.clone()).collect();
        assert_eq!(names, vec![ai_name(0), ai_name(1), ai_name(2)]);
    }

    #[test]
    fn problems_block_the_start() {
        let r = rules();
        let mut s = Setup::new(&r);
        s.slots[2].kind = SlotKind::Ai;
        assert_eq!(s.problem(&r, &maps()), Some(Problem::TooManyPlayers));
        s.apply(&SetupHit::Map(1), &r, &maps());
        assert_eq!(s.problem(&r, &maps()), None);
        s.slots[0].start = Some(3);
        s.slots[1].start = Some(3);
        assert_eq!(s.problem(&r, &maps()), Some(Problem::SameStart));
        s.slots[1].start = None;
        // spec: S-0412 claim 7 (a colour fixed by one player is not free for another)
        s.slots[0].colour = Some(2);
        s.slots[2].colour = Some(2);
        assert_eq!(s.problem(&r, &maps()), Some(Problem::SameColour));
        s.slots[2].colour = None;
        // Changing to a map with fewer starts makes a far start random.
        s.slots[2].kind = SlotKind::Closed;
        s.apply(&SetupHit::Map(1), &r, &maps());
        assert_eq!(s.slots[0].start, None);
        s.slots[1].kind = SlotKind::Closed;
        assert_eq!(s.problem(&r, &maps()), Some(Problem::TooFewPlayers));
        assert_eq!(s.problem(&r, &[]), Some(Problem::NoMap));
    }

    #[test]
    fn lobby_maps_every_value() {
        let r = rules();
        let mut s = Setup::new(&r);
        s.slots[0].country = Some(1);
        s.slots[0].colour = Some(5);
        s.slots[0].team = Some(2);
        s.slots[0].start = Some(1);
        s.flags[Flag::ShortGame as usize] = true;
        let l = s.lobby(&r, &maps()).unwrap();
        assert_eq!(l.players.len(), 2);
        let (p, a) = (&l.players[0], &l.players[1]);
        assert_eq!(p.name, HUMAN_NAME);
        assert!(!p.ai);
        assert_eq!(p.country, "Betaland");
        assert_eq!(p.values.colour, Some(5));
        // spec: S-0412 claim 8 (lobby team)
        assert_eq!(p.values.team, Some(2));
        // spec: S-0009 claim 17 (start position i is the i-th start entry)
        assert_eq!(p.start, Some(1));
        // spec: S-0412 claim 12 (AI houses take the computer slots)
        assert!(a.ai);
        assert_eq!(a.name, "Computer");
        // spec: S-0412 claims 6 and 7 (random choices are marked for resolve_lobby)
        assert_eq!(
            l.choices[0],
            LobbyChoice {
                country: false,
                colour: false
            }
        );
        assert_eq!(
            l.choices[1],
            LobbyChoice {
                country: true,
                colour: true
            }
        );
        // spec: S-0412 claim 1 (the lobby's setup values)
        assert_eq!(l.settings.credits, Some(1200));
        assert_eq!(l.settings.short_game, Some(true));
        assert_eq!(l.settings.superweapons, Some(true));
        assert_eq!(l.unit_count, 3);
        assert!(l.bases);
        assert_eq!(l.game_speed, 4);
    }

    // spec: S-0009 claim 19 (persons by increasing colour slot, ties to the earlier lobby slot, design, then computers in lobby-slot order; Q-1805: a person on "random" placed by the slot it was given; the draws in increasing lobby-slot order, persons and computers alike, design)
    #[test]
    fn house_order_puts_persons_by_colour_then_computers() {
        let r = rules();
        let mut s = Setup::new(&r);
        // The skirmish: the person, then the computer, in row order.
        assert_eq!(s.house_order(), vec![0, 1]);
        s.slots[2].kind = SlotKind::Human;
        s.slots[3].kind = SlotKind::Ai;
        s.slots[4].kind = SlotKind::Human;
        s.slots[0].colour = Some(5);
        s.slots[2].colour = Some(1);
        // Row 4's person on "random": placed by the slot it is given.
        let mut seen = [false; 3];
        for seed in 0..64 {
            let mut t = s.clone();
            t.resolve_colours(seed);
            let given = t.slots[4].colour.expect("a slot");
            // spec: S-0412 claim 7 (every row on "random" gets a free slot, distinct from the others)
            let mut all: Vec<u8> = t.players().filter_map(|(_, x)| x.colour).collect();
            assert_eq!(all.len(), 5);
            all.sort();
            all.dedup();
            assert_eq!(all.len(), 5);
            assert_eq!((t.slots[0].colour, t.slots[2].colour), (Some(5), Some(1)));
            let (case, order) = if given < 1 {
                (0, vec![4, 2, 0, 1, 3])
            } else if given < 5 {
                (1, vec![2, 4, 0, 1, 3])
            } else {
                (2, vec![2, 0, 4, 1, 3])
            };
            seen[case] = true;
            assert_eq!(t.house_order(), order, "given {given}");
            // The same seed resolves the same way.
            let mut u = s.clone();
            u.resolve_colours(seed);
            assert_eq!(u, t);
        }
        assert_eq!(seen, [true; 3]);
        // spec: S-0009 claim 19 (a tie between two persons goes to the earlier lobby slot, design)
        let mut tie = s.clone();
        tie.slots[4].colour = Some(1);
        assert_eq!(tie.house_order(), vec![2, 4, 0, 1, 3]);
        s.slots[4].colour = Some(3);
        assert_eq!(s.house_order(), vec![2, 4, 0, 1, 3]);
        let maps = [MapEntry {
            label: "map C".into(),
            starts: 8,
        }];
        let l = s.lobby(&r, &maps).unwrap();
        let names: Vec<&str> = l.players.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            names,
            ["Player", "Player 2", "Player 3", "Computer", "Computer 2"]
        );
        let colours: Vec<_> = l.players.iter().map(|p| p.values.colour).collect();
        assert_eq!(colours, [Some(1), Some(3), Some(5), None, None]);
        assert_eq!(
            s.check(r.countries.len(), maps.first()),
            s.problem(&r, &maps)
        );
    }

    #[test]
    fn game_setup_resolves_random_choices() {
        let sim = glasnost_ai::fixture::rules();
        let r = rules();
        let s = Setup::new(&r);
        let l = s.lobby(&r, &maps()).unwrap();
        let g = game_setup(&l, &sim, &r.countries, 7, 99).unwrap();
        let players: Vec<_> = g
            .houses
            .iter()
            .filter(|h| h.role == glasnost_sim::world::HouseRole::Player)
            .collect();
        assert_eq!(players.len(), 2);
        // spec: S-0412 claim 12
        assert!(!players[0].ai && players[1].ai);
        // spec: S-0412 claim 7 (random colours resolve to distinct slots 0 … 7)
        let c0 = g.skirmish.houses[0].colour.unwrap();
        let c1 = g.skirmish.houses[1].colour.unwrap();
        assert!(c0 <= 7 && c1 <= 7 && c0 != c1);
        // spec: S-0412 claim 6 (a random country is one the setup offers)
        for p in &players {
            assert!(r.countries.contains(&p.country), "{}", p.country);
        }
        assert_eq!(g.game_speed, 4);
        assert_eq!(g.seed, 7);
        // The same lobby seed resolves the same way.
        assert_eq!(game_setup(&l, &sim, &r.countries, 7, 99).unwrap(), g);
    }

    // spec: S-0412 claim 15 (the map's start-waypoint count gives the rows shown; switching to a map with fewer rows empties and hides the computer slots beyond them; no team and four teams)
    #[test]
    fn claim_15_rows_follow_the_map() {
        let r = rules();
        let mut s = Setup::new(&r);
        // Map B: four rows, a computer in the fourth.
        s.apply(&SetupHit::Map(1), &r, &maps());
        s.slots[3].kind = SlotKind::Ai;
        let (_, t) = s.view((800, 600), &r, &maps(), None);
        assert!(t.rect_of(&SetupHit::Kind(3, 1)).is_some());
        assert!(t.rect_of(&SetupHit::Kind(4, 1)).is_none());
        // Back to map A: two rows.
        s.apply(&SetupHit::Map(1), &r, &maps());
        assert_eq!(s.slots[3].kind, SlotKind::Closed);
        assert_eq!(s.slots[1].kind, SlotKind::Ai);
        let (_, t) = s.view((800, 600), &r, &maps(), None);
        assert!(t.rect_of(&SetupHit::Kind(1, 1)).is_some());
        assert!(t.rect_of(&SetupHit::Kind(2, 1)).is_none());
        // No team, then teams 1 … 4, then no team again.
        let mut teams = Vec::new();
        for _ in 0..5 {
            s.apply(&SetupHit::Team(0, 1), &r, &maps());
            teams.push(s.slots[0].team);
        }
        assert_eq!(teams, [Some(1), Some(2), Some(3), Some(4), None]);
    }

    #[test]
    fn layout_offers_every_control() {
        let r = rules();
        let s = Setup::new(&r);
        let (d, t) = s.view((800, 600), &r, &maps(), None);
        assert!(d.texts().contains(&"SKIRMISH SETUP"));
        for want in [
            SetupHit::Map(1),
            SetupHit::Kind(1, 1),
            SetupHit::Country(0, -1),
            SetupHit::Colour(1, 1),
            SetupHit::Team(0, 1),
            SetupHit::Start(1, 1),
            SetupHit::Credits(1),
            SetupHit::Units(-1),
            SetupHit::Speed(1),
            SetupHit::Toggle(Flag::Crates),
            SetupHit::Fixes,
            SetupHit::Back,
            SetupHit::Start_,
        ] {
            let rect = t
                .rect_of(&want)
                .unwrap_or_else(|| panic!("{want:?} missing"));
            assert_eq!(t.hit(rect.centre()), Some(want), "{want:?} not on top");
        }
        // A closed slot offers no country.
        assert!(t.rect_of(&SetupHit::Country(3, 1)).is_none());
    }

    /// A network room: each member's targets are what it may change (our
    /// own netcode; the room enforces the same rules).
    #[test]
    fn a_room_offers_each_member_only_its_own_controls() {
        let r = rules();
        let mut s = Setup::new(&r);
        s.map = 1;
        s.slots[1].kind = SlotKind::Human;
        s.slots[2].kind = SlotKind::Ai;
        s.slots[3].kind = SlotKind::Human;
        let mut f = RoomFrame {
            title: "ROOM AB12CD34".into(),
            names: [const { None }; MAX_SLOTS],
            ready: [false; MAX_SLOTS],
            host_row: Some(0),
            mine: Some(1),
            host: false,
            go: "Ready".into(),
            go_enabled: true,
            note: None,
        };
        f.names[0] = Some("Ann".into());
        f.names[1] = Some("Bob".into());
        f.ready[1] = true;
        let (d, t) = s.view_framed((800, 600), &r, &maps(), None, Some(&f));
        let texts = d.texts();
        for want in [
            "ROOM AB12CD34",
            "Ann",
            "Bob",
            "Open",
            "Host",
            "Ready",
            "Leave",
        ] {
            assert!(texts.contains(&want), "{want} missing: {texts:?}");
        }
        // A guest: its own row's choices, ready, leave; nothing else.
        for want in [
            SetupHit::Country(1, 1),
            SetupHit::Start(1, -1),
            SetupHit::Back,
            SetupHit::Start_,
        ] {
            assert!(t.rect_of(&want).is_some(), "{want:?} missing");
        }
        for not in [
            SetupHit::Map(1),
            SetupHit::Kind(3, 1),
            SetupHit::Kind(2, 1),
            SetupHit::Country(0, 1),
            SetupHit::Country(2, 1),
            SetupHit::Credits(1),
            SetupHit::Speed(1),
            SetupHit::Toggle(Flag::Crates),
            SetupHit::Fixes,
        ] {
            assert!(t.rect_of(&not).is_none(), "{not:?} offered to a guest");
        }
        // The host: the map, the game, the untaken and computer rows.
        f.host = true;
        f.mine = Some(0);
        f.go = "Start game".into();
        let (_, t) = s.view_framed((800, 600), &r, &maps(), None, Some(&f));
        for want in [
            SetupHit::Map(1),
            SetupHit::Kind(3, 1),
            SetupHit::Kind(2, 1),
            SetupHit::Country(0, 1),
            SetupHit::Country(2, 1),
            SetupHit::Credits(1),
            SetupHit::Toggle(Flag::Crates),
            SetupHit::Fixes,
            SetupHit::Start_,
        ] {
            assert!(t.rect_of(&want).is_some(), "{want:?} missing for the host");
        }
        for not in [SetupHit::Kind(1, 1), SetupHit::Country(1, 1)] {
            assert!(t.rect_of(&not).is_none(), "{not:?} offered to the host");
        }
        // A note replaces the problem line; a host's problem blocks the start.
        f.note = Some(("the map hash differs from the room's".into(), true));
        s.slots[1].start = Some(0);
        s.slots[0].start = Some(0);
        let (d, t) = s.view_framed((800, 600), &r, &maps(), None, Some(&f));
        assert!(d.texts().contains(&"the map hash differs from the room's"));
        assert!(t.rect_of(&SetupHit::Start_).is_none());
    }
}
