//! One room's state machine: its members, its setup, readiness and the
//! start.
//!
//! **Rows.** The room's setup is the client's [`Setup`]: up to eight rows
//! ([`MAX_SLOTS`]), each a person's row ([`SlotKind::Human`]), a computer
//! row ([`SlotKind::Ai`]) or closed. A person's row is either taken by one
//! member or open. Only the map's rows are used (S-0412 claim 15); the
//! others stay closed. The host takes the first person's row of the setup
//! it creates the room with; a member who joins takes the first open
//! person's row.
//!
//! **Who changes what.** The host chooses the map, opens, closes or gives a
//! computer player any row nobody has taken, sets the choices of the
//! computer rows and the game-wide values. Each person sets the choices of
//! their own row (country, colour, team, start). Any change of the setup,
//! and any join or leave, clears every member's ready mark, so a start
//! always has the agreement of every person to the setup as it is.
//!
//! **The choice rules** the room enforces are those of the setup screen:
//! the offered countries (S-0412 claim 15), colour slots 0 … 7 of which a
//! slot fixed by one player is not free for another (S-0412 claim 7), no
//! team or teams 1 … 4 (S-0412 claim 15), start positions from the map's
//! start list (S-0009 claim 17), two players never on the same start
//! position, as many players as the map has start positions (S-0009 claim
//! 23) and at least two. The values the screen's sliders offer come from
//! the host's rules ([`Offered`]).
//!
//! **The start.** The host starts when every person's row is taken, every
//! other member is ready and the setup has no problem. The colours left on
//! "random" are drawn then, row by row ([`Setup::resolve_colours`]), and
//! [`Begin::setup`] carries them. The slot table lists the occupied rows in
//! house order ([`Setup::house_order`], S-0009 claim 19: a person on
//! "random" by the slot it was given), so it is the slot table N1's session
//! builds from the game setup. Every member gets the same [`Begin`], with
//! its own slot.
//!
//! **Leaving.** A member who leaves before the start frees their row (open,
//! its choices back on random). When the host leaves, the member on the
//! lowest row becomes host; a room with no member is gone. Once the game
//! has begun nobody joins; a member who dropped comes back to its own slot
//! by its ticket ([`Room::rejoin`], N5).
//! These rules are our own; S-0412 does not cover network games.

use std::collections::BTreeMap;

use glasnost_ui::setup::{self, MAX_SLOTS, MapEntry, Problem, Setup, Slot as Row, SlotKind, TEAMS};

use super::MemberId;
use super::wire::{
    Begin, Edit, MapChoice, MemberView, Offered, Refusal, RoomInfo, RoomView, ToRelay, Value,
};
use crate::start::{Field, Params, Slot, SlotKind as NetKind};

/// The longest name, in characters.
// nocite: our own netcode
pub const NAME_CHARS: usize = 24;

/// The longest build, hash or map label, in bytes.
// nocite: our own netcode
pub const TEXT_BYTES: usize = 128;

/// The colour slots a player may choose (0 … 7).
pub const COLOUR_SLOTS: u8 = 8; // spec: S-0412 claim 7 (colour slots 0 … 7)

/// The positions of the speed control (0 … 6).
pub const SPEEDS: u8 = 7; // spec: S-0201 claim 7 (seven positions)

/// One member.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Member {
    name: String,
    row: u8,
    ready: bool,
}

/// One room.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Room {
    code: String,
    build: String,
    rules_hash: String,
    offered: Offered,
    map: MapChoice,
    setup: Setup,
    seats: [Option<MemberId>; MAX_SLOTS],
    members: BTreeMap<MemberId, Member>,
    host: MemberId,
    playing: bool,
}

/// What [`Room::start`] gives: the start values, `you` still 0, and each
/// member's slot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Started {
    /// The start values.
    pub begin: Begin,
    /// Each member and its slot, in slot order.
    pub persons: Vec<(MemberId, u8)>,
}

impl Started {
    /// The start values for `member`.
    pub fn begin_for(&self, member: MemberId) -> Option<Begin> {
        self.persons
            .iter()
            .find(|(m, _)| *m == member)
            .map(|&(_, slot)| {
                let mut b = self.begin.clone();
                b.you = slot;
                b
            })
    }
}

/// A person's name made fit to show: control characters left out, spaces
/// trimmed, at most [`NAME_CHARS`] characters. `None` when nothing is left.
pub fn clean_name(name: &str) -> Option<String> {
    let s: String = name
        .chars()
        .filter(|c| !c.is_control())
        .take(NAME_CHARS * 4) // nocite: our own netcode (enough to trim spaces from)
        .collect();
    let s: String = s.trim().chars().take(NAME_CHARS).collect();
    (!s.is_empty()).then_some(s)
}

/// The longest chat line, in characters.
// nocite: our own netcode
pub const CHAT_CHARS: usize = 200;

/// A chat line made fit to pass on: control characters left out, spaces
/// trimmed, at most [`CHAT_CHARS`] characters. `None` when nothing is left.
pub fn clean_chat(text: &str) -> Option<String> {
    let s: String = text
        .chars()
        .filter(|c| !c.is_control())
        .take(CHAT_CHARS * 4) // nocite: our own netcode (enough to trim spaces from)
        .collect();
    let s: String = s.trim().chars().take(CHAT_CHARS).collect();
    (!s.is_empty()).then_some(s)
}

fn text_ok(s: &str) -> bool {
    !s.is_empty() && s.len() <= TEXT_BYTES && !s.chars().any(char::is_control)
}

fn check_map(map: &MapChoice) -> Result<(), Refusal> {
    if !text_ok(&map.hash) || !text_ok(&map.label) {
        return Err(Refusal::Bad(Value::Text));
    }
    // spec: S-0009 claim 23 (the lobby allows as many players as the map has start positions); at least one
    if map.starts == 0 {
        return Err(Refusal::Bad(Value::Starts));
    }
    Ok(())
}

impl Room {
    /// A room made by `host`'s [`ToRelay::Create`].
    pub fn create(code: String, host: MemberId, create: &ToRelay) -> Result<Room, Refusal> {
        let ToRelay::Create {
            name,
            build,
            rules_hash,
            offered,
            map,
            setup,
        } = create
        else {
            return Err(Refusal::NotInRoom);
        };
        let name = clean_name(name).ok_or(Refusal::Bad(Value::Name))?;
        if !text_ok(build) || !text_ok(rules_hash) {
            return Err(Refusal::Bad(Value::Text));
        }
        check_map(map)?;
        let (lo, hi) = offered.credits;
        let (ulo, uhi) = offered.units;
        if lo > hi || ulo > uhi {
            return Err(Refusal::Bad(Value::Offered));
        }
        let mut room = Room {
            code,
            build: build.clone(),
            rules_hash: rules_hash.clone(),
            offered: *offered,
            map: map.clone(),
            setup: Setup {
                map: 0,
                ..setup.clone()
            },
            seats: [None; MAX_SLOTS],
            members: BTreeMap::new(),
            host,
            playing: false,
        };
        let rows = room.rows();
        // spec: S-0412 claim 15 (only the map's rows are used; the others are emptied)
        for r in room.setup.slots.iter_mut().skip(rows) {
            *r = Row::closed();
        }
        room.check_game(room.setup.credits, room.setup.units, room.setup.speed)?;
        for i in 0..rows {
            let r = room.setup.slots[i];
            if r.kind != SlotKind::Closed {
                room.check_choices(i, &r)?;
            }
        }
        let row = room.open_row().ok_or(Refusal::Bad(Value::Row))?;
        room.seats[row] = Some(host);
        room.members.insert(
            host,
            Member {
                name,
                row: row as u8,
                ready: false,
            },
        );
        Ok(room)
    }

    /// The room code.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// The host.
    pub fn host(&self) -> MemberId {
        self.host
    }

    /// The setup.
    pub fn setup(&self) -> &Setup {
        &self.setup
    }

    /// The map.
    pub fn map(&self) -> &MapChoice {
        &self.map
    }

    /// Whether the game has begun.
    pub fn playing(&self) -> bool {
        self.playing
    }

    /// The members, in member order.
    pub fn members(&self) -> impl Iterator<Item = MemberId> + '_ {
        self.members.keys().copied()
    }

    /// Whether `id` is a member.
    pub fn has(&self, id: MemberId) -> bool {
        self.members.contains_key(&id)
    }

    /// The row `id` has taken.
    pub fn row_of(&self, id: MemberId) -> Option<u8> {
        self.members.get(&id).map(|m| m.row)
    }

    /// The name of member `id`.
    pub fn name_of(&self, id: MemberId) -> Option<&str> {
        self.members.get(&id).map(|m| m.name.as_str())
    }

    /// Whether `id` is marked ready.
    pub fn ready(&self, id: MemberId) -> bool {
        self.members.get(&id).is_some_and(|m| m.ready)
    }

    fn entry(&self) -> MapEntry {
        MapEntry {
            label: self.map.label.clone(),
            starts: self.map.starts,
        }
    }

    /// The rows the map offers (S-0412 claim 15).
    pub fn rows(&self) -> usize {
        // spec: S-0412 claim 15 (the map's start-waypoint count gives the rows, at most eight)
        setup::rows(Some(&self.entry()))
    }

    /// The first open person's row.
    fn open_row(&self) -> Option<usize> {
        (0..self.rows())
            .find(|&i| self.setup.slots[i].kind == SlotKind::Human && self.seats[i].is_none())
    }

    fn unready_all(&mut self) {
        for m in self.members.values_mut() {
            m.ready = false;
        }
    }

    fn check_game(&self, credits: i32, units: i32, speed: u8) -> Result<(), Refusal> {
        let (lo, hi) = self.offered.credits;
        // spec: S-0412 claim 1 (the credits and unit-count settings come from the lobby's sliders; their ranges from the rules, Import mapping)
        if !(lo..=hi).contains(&credits) {
            return Err(Refusal::Bad(Value::Credits));
        }
        let (lo, hi) = self.offered.units;
        if !(lo..=hi).contains(&units) {
            return Err(Refusal::Bad(Value::Units));
        }
        if speed >= SPEEDS {
            return Err(Refusal::Bad(Value::Speed));
        }
        Ok(())
    }

    /// Whether row `i` may hold the choices of `r`: values in range and no
    /// colour or start another occupied row has fixed.
    fn check_choices(&self, i: usize, r: &Row) -> Result<(), Refusal> {
        // spec: S-0412 claim 15 (the offered countries)
        if r.country
            .is_some_and(|c| c >= usize::from(self.offered.countries))
        {
            return Err(Refusal::Bad(Value::Country));
        }
        // spec: S-0412 claim 7 (colour slots 0 … 7)
        if r.colour.is_some_and(|c| c >= COLOUR_SLOTS) {
            return Err(Refusal::Bad(Value::Colour));
        }
        // spec: S-0412 claim 15 (no team, or one of four teams)
        if r.team.is_some_and(|t| t == 0 || t > TEAMS) {
            return Err(Refusal::Bad(Value::Team));
        }
        // spec: S-0009 claim 17 (a start position is an entry of the map's start list)
        if r.start.is_some_and(|s| s >= self.map.starts) {
            return Err(Refusal::Bad(Value::Start));
        }
        for (j, o) in self.setup.slots.iter().enumerate() {
            if j == i || o.kind == SlotKind::Closed {
                continue;
            }
            // spec: S-0412 claim 7 (a colour slot fixed by one player is not free for another)
            if r.colour.is_some() && r.colour == o.colour {
                return Err(Refusal::Setup(Problem::SameColour));
            }
            if r.start.is_some() && r.start == o.start {
                return Err(Refusal::Setup(Problem::SameStart));
            }
        }
        Ok(())
    }

    /// What the room is, for [`ToRelay::Peek`].
    pub fn info(&self) -> RoomInfo {
        let rows = self.rows();
        RoomInfo {
            code: self.code.clone(),
            build: self.build.clone(),
            rules_hash: self.rules_hash.clone(),
            map: self.map.clone(),
            persons: self.members.len() as u8,
            open: (0..rows)
                .filter(|&i| self.setup.slots[i].kind == SlotKind::Human && self.seats[i].is_none())
                .count() as u8,
            playing: self.playing,
        }
    }

    /// The room as `you` sees it.
    pub fn view(&self, you: MemberId) -> RoomView {
        let mut members: Vec<MemberView> = self
            .members
            .iter()
            .map(|(&id, m)| MemberView {
                id,
                row: m.row,
                name: m.name.clone(),
                ready: m.ready,
            })
            .collect();
        members.sort_by_key(|m| m.row);
        RoomView {
            code: self.code.clone(),
            you,
            host: self.host,
            build: self.build.clone(),
            rules_hash: self.rules_hash.clone(),
            offered: self.offered,
            map: self.map.clone(),
            setup: self.setup.clone(),
            members,
        }
    }

    /// `id` joins with `join` ([`ToRelay::Join`]): the build, the rules hash
    /// and the map hash must be the room's, and an open person's row must be
    /// free. Returns the row taken.
    pub fn join(&mut self, id: MemberId, join: &ToRelay) -> Result<u8, Refusal> {
        let ToRelay::Join {
            name,
            build,
            rules_hash,
            map_hash,
            ..
        } = join
        else {
            return Err(Refusal::NotInRoom);
        };
        if self.playing {
            // A dropped member comes back by its ticket instead
            // (`ToRelay::Rejoin`, N5).
            return Err(Refusal::Playing);
        }
        if self.members.contains_key(&id) {
            return Err(Refusal::InRoom);
        }
        if *build != self.build {
            return Err(Refusal::Mismatch(Field::Build));
        }
        if *rules_hash != self.rules_hash {
            return Err(Refusal::Mismatch(Field::RulesHash));
        }
        if *map_hash != self.map.hash {
            return Err(Refusal::Mismatch(Field::MapHash));
        }
        let name = clean_name(name).ok_or(Refusal::Bad(Value::Name))?;
        let row = self.open_row().ok_or(Refusal::RoomFull)?;
        self.seats[row] = Some(id);
        self.members.insert(
            id,
            Member {
                name,
                row: row as u8,
                ready: false,
            },
        );
        self.unready_all();
        Ok(row as u8)
    }

    /// `id` leaves. Returns false when it was no member. Before the start
    /// its row is opened, with its choices on random; the host's role passes
    /// to the member on the lowest row.
    pub fn leave(&mut self, id: MemberId) -> bool {
        let Some(m) = self.members.remove(&id) else {
            return false;
        };
        let row = usize::from(m.row);
        self.seats[row] = None;
        if !self.playing {
            self.setup.slots[row] = Row {
                kind: SlotKind::Human,
                ..Row::closed()
            };
        }
        if self.host == id
            && let Some((&next, _)) = self.members.iter().min_by_key(|(_, m)| m.row)
        {
            self.host = next;
        }
        self.unready_all();
        true
    }

    /// Whether the room has no member left.
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// `id` changes the setup.
    pub fn edit(&mut self, id: MemberId, edit: &Edit) -> Result<(), Refusal> {
        let me = self.members.get(&id).ok_or(Refusal::NotInRoom)?.row;
        if self.playing {
            return Err(Refusal::Playing);
        }
        let host = id == self.host;
        match edit {
            Edit::Map(map) => {
                if !host {
                    return Err(Refusal::NotHost);
                }
                check_map(map)?;
                let new_rows = setup::rows(Some(&MapEntry {
                    label: map.label.clone(),
                    starts: map.starts,
                }));
                if let Some(r) = (new_rows..MAX_SLOTS).find(|&r| self.seats[r].is_some()) {
                    return Err(Refusal::Seated(r as u8));
                }
                self.map = map.clone();
                for (i, r) in self.setup.slots.iter_mut().enumerate() {
                    // Start positions past the new map's count become random, as on the setup screen.
                    if r.start.is_some_and(|s| s >= map.starts) {
                        r.start = None;
                    }
                    // spec: S-0412 claim 15 (rows beyond the new map's row count are emptied and hidden)
                    if i >= new_rows {
                        *r = Row::closed();
                    }
                }
            }
            Edit::Row { row, kind } => {
                if !host {
                    return Err(Refusal::NotHost);
                }
                let i = usize::from(*row);
                if i >= self.rows() {
                    return Err(Refusal::NoRow(*row));
                }
                if self.seats[i].is_some() {
                    return Err(Refusal::Seated(*row));
                }
                self.setup.slots[i] = Row {
                    kind: *kind,
                    ..Row::closed()
                };
            }
            Edit::Choices {
                row,
                country,
                colour,
                team,
                start,
            } => {
                let i = usize::from(*row);
                if i >= self.rows() {
                    return Err(Refusal::NoRow(*row));
                }
                let target = self.setup.slots[i];
                let allowed = *row == me || (host && target.kind == SlotKind::Ai);
                if !allowed {
                    return Err(Refusal::NotYours(*row));
                }
                let new = Row {
                    kind: target.kind,
                    country: country.map(usize::from),
                    colour: *colour,
                    team: *team,
                    start: *start,
                };
                self.check_choices(i, &new)?;
                self.setup.slots[i] = new;
            }
            Edit::Game {
                credits,
                units,
                flags,
                speed,
            } => {
                if !host {
                    return Err(Refusal::NotHost);
                }
                self.check_game(*credits, *units, *speed)?;
                self.setup.credits = *credits;
                self.setup.units = *units;
                self.setup.flags = *flags;
                self.setup.speed = *speed;
            }
        }
        self.unready_all();
        Ok(())
    }

    /// `id` says ready or not for the current setup, with its map hash.
    pub fn set_ready(&mut self, id: MemberId, ready: bool, map_hash: &str) -> Result<(), Refusal> {
        if self.playing {
            return Err(Refusal::Playing);
        }
        if ready && map_hash != self.map.hash {
            return Err(Refusal::Mismatch(Field::MapHash));
        }
        let m = self.members.get_mut(&id).ok_or(Refusal::NotInRoom)?;
        m.ready = ready;
        Ok(())
    }

    /// The host `id` starts the game with `seed` (the relay's host
    /// randomness) and the lockstep parameters `params`.
    pub fn start(&mut self, id: MemberId, seed: u32, params: Params) -> Result<Started, Refusal> {
        if !self.members.contains_key(&id) {
            return Err(Refusal::NotInRoom);
        }
        if self.playing {
            return Err(Refusal::Playing);
        }
        if id != self.host {
            return Err(Refusal::NotHost);
        }
        if let Some(p) = self
            .setup
            .check(usize::from(self.offered.countries), Some(&self.entry()))
        {
            return Err(Refusal::Setup(p));
        }
        let persons_rows: Vec<usize> = (0..self.rows())
            .filter(|&i| self.setup.slots[i].kind == SlotKind::Human)
            .collect();
        if let Some(&i) = persons_rows.iter().find(|&&i| self.seats[i].is_none()) {
            return Err(Refusal::Open(i as u8));
        }
        if let Some(&i) = persons_rows
            .iter()
            .find(|&&i| self.seats[i].is_some_and(|m| m != self.host && !self.ready(m)))
        {
            return Err(Refusal::NotReady(i as u8));
        }
        // spec: S-0009 claim 19 (the colour draws come before the house order is fixed: every row on "random", person or computer, in increasing lobby-slot order, design)
        let mut setup = self.setup.clone();
        setup.resolve_colours(seed);
        // spec: S-0009 claim 19 (the slot table follows the house order of the game setup)
        let order = setup.house_order();
        let mut slots = Vec::with_capacity(order.len());
        let mut names = Vec::with_capacity(order.len());
        let mut persons = Vec::new();
        let mut ais = 0;
        for (slot, &row) in order.iter().enumerate() {
            let kind = self.setup.slots[row].kind;
            if kind == SlotKind::Human {
                let m = self.seats[row].expect("every person's row is taken");
                persons.push((m, slot as u8));
                names.push(self.members[&m].name.clone());
            } else {
                ais += 1;
                names.push(setup::ai_name(ais - 1));
            }
            slots.push(Slot {
                house: slot as u8,
                kind: if kind == SlotKind::Human {
                    NetKind::Human
                } else {
                    // spec: S-0412 claim 12 (AI houses take the computer slots)
                    NetKind::Computer
                },
            });
        }
        self.playing = true;
        Ok(Started {
            begin: Begin {
                build: self.build.clone(),
                rules_hash: self.rules_hash.clone(),
                map: self.map.clone(),
                seed,
                setup,
                slots,
                rows: order.iter().map(|&r| r as u8).collect(),
                names,
                params: params.sane(),
                you: 0,
                ticket: [0; 16],
            },
            persons,
        })
    }

    /// `id` comes back to its game on row `row` under `name` after a drop
    /// (N5): the relay has checked its ticket. The room only lists it again,
    /// for chat; the game's slots are the relay's.
    pub fn rejoin(&mut self, id: MemberId, row: u8, name: &str) -> Result<(), Refusal> {
        if !self.playing {
            return Err(Refusal::NoGame);
        }
        if self.members.contains_key(&id) {
            return Err(Refusal::InRoom);
        }
        let seat = self
            .seats
            .get_mut(usize::from(row))
            .ok_or(Refusal::Bad(Value::Row))?;
        if seat.is_some() {
            return Err(Refusal::Seated(row));
        }
        *seat = Some(id);
        self.members.insert(
            id,
            Member {
                name: clean_name(name).unwrap_or_else(|| "?".into()),
                row,
                ready: true,
            },
        );
        Ok(())
    }
}
