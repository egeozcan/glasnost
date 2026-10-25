//! Arena mode (M7b step 4c; PLAN.md §5 M7 exit and §6 layer 2B): two
//! fixed groups of named types meet on a small flat map of our own, over
//! many seeds, and the outcomes (win rates, times to kill, survivors) are
//! compared with the stock oracle's arena measurement records.
//!
//! # The scenario
//!
//! - **Map**: [`testkit::empty_map`] of `map_size` (clear, flat cells of
//!   [`testkit::catalogue`]'s tile 0, level 0; no EA bytes), with no
//!   objects.
//! - **Houses**: two player houses, A's and B's, not allied, so hostile; sandbox setup (no start positions, no starting
//!   units, no bases), not AI houses. Their countries come from the
//!   scenario, else both take the rules' first country that is not passive
//!   (and not the neutral or special one).
//! - **Placement**: each group stands in rows of [`ROW`] cells across the
//!   map's x axis, one unit per cell, in the order the group lists its
//!   types; the middle cells of the two front rows are `start_distance`
//!   cells apart on the x axis, centred on the map's middle cell, and
//!   further rows stand one cell further back. Every unit faces the enemy
//!   front cell, as the nearest facing value (a 256th of a turn; the
//!   direction is along the x axis, so exact). Units are put on the map
//!   by S-0216's play-time placement (`objects::place_in_play`), as a
//!   harness that adds units to a running game: an infantry unit stands at
//!   the standing position its draw gives, and a refused placement is an
//!   error of the scenario.
//! - **Alternation**: a trial with an odd seed has group A as house 0, on
//!   the lower-x side, created first (so first in the update order, S-0203
//!   claim 3) and ordered first; a trial with an even seed is
//!   [`swapped`]: group B takes all four. Over consecutive seeds the
//!   trials alternate, starting unswapped with the default first seed 1;
//!   a mirror matchup is then balanced over an even number of trials.
//! - **Orders** (orders a person could give, through `orders::issue`): in
//!   the first tick in which orders are taken, every unit is ordered to
//!   attack-move to the enemy group's front cell. Then every
//!   [`REISSUE_PERIOD`] ticks every idle unit of a group (guarding or area
//!   guarding with no target and no destination) is ordered to attack-move
//!   to the cell of the nearest enemy unit of the other group (lowest ID on
//!   a tie).
//! - **End**: a unit *of a group* is one of the objects created for the
//!   group that still exists with health above 0 and is still owned by its
//!   house (a unit taken over by mind control no longer counts for its
//!   group, and counts for no group; a unit off the map, lifted or carried,
//!   still counts). The trial ends after the first tick at whose end a group
//!   has no units: the other group wins, or it is a draw when neither has
//!   any. A trial still running `tick_cap` ticks after the order tick is a
//!   timeout.
//! - **Time to kill**: the ticks from the order tick (counted as the first)
//!   to the end tick, both included.
//! - **Survivors**: per group, the units of the group at the end, and their
//!   summed health in per-mille of the group's summed health at creation
//!   (floored).
//!
//! The scenario's choices (row width, the re-order period, the default map
//! size, distance and tick cap) are rules of this tool, not of the game.
//!
//! # Arena measurement records (alternative format)
//!
//! The lead's arena records (M-0001 to M-0003) use another form, front
//! matter plus a sibling CSV, and another setup; they are read and played
//! by [`crate::oracle`]. The fenced-block form below is kept as a
//! documented alternative for small matchups on this module's own flat
//! map; no record uses it today.
//!
//! A record in this form (`measurements/<topic>/M-xxxx-*.md`) holds one or more fenced blocks whose info string is `arena`;
//! each block is one matchup, one `key: value` per line (`#` starts a
//! comment):
//!
//! ```text
//! group-a: TYPE1*5, TYPE2*2    # type ID and count; count 1 when "*n" is left out
//! group-b: TYPE3*3
//! country-a: Name              # optional (default: see Houses above)
//! country-b: Name              # optional
//! map-size: 32                 # optional; the map is map-size × map-size
//! start-distance: 10           # optional; cells between the two front cells
//! tick-cap: 6000               # optional; ticks from the order tick
//! game-speed: 4                # optional; the lobby game speed, 0 to 6 (default: the rules' lobby speed)
//! n: 32                        # number of trials
//! wins-a: 20                   # optional cross-checks of the trial lines
//! wins-b: 10
//! draws: 1
//! timeouts: 1
//! trial: A 812 3 0 640 0       # one line per trial, in trial order
//! ```
//!
//! A trial line is `<winner> <ticks> <survivors-a> <survivors-b>
//! [<health-a> <health-b>]`: the winner `A`, `B`, `draw` or `timeout`; the
//! time to kill in ticks (as defined above; for a timeout, the cap); the
//! units of each group left at the end; optionally their health in
//! per-mille of the group's full health. All numbers are integers. `n`
//! must equal the number of trial lines, and given win counts must match
//! them.
//!
//! When the record's front matter says `value-visibility: local`, the
//! outcome keys (`n`, the win counts, the trial lines) may instead come from
//! the git-ignored `measurements/local/<id>.json`, a JSON object
//! `{"arena": [ {…}, … ]}` with one object per `arena` block of the record,
//! in order, whose members are the same keys (integers or strings; `trial`
//! an array of trial-line strings). A key may be given in the record or in
//! the local file, not in both.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use glasnost_map::CellXY;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{GameSetup, HouseId, HouseRole, HouseSetup, ObjectId, Target, World};

use crate::stats;

/// Units per row of a group (our tool's choice).
pub const ROW: usize = 5;

/// Ticks between re-orders of idle units (our tool's choice).
pub const REISSUE_PERIOD: i32 = 60;

/// Default map size (our tool's choice).
pub const DEFAULT_MAP_SIZE: i32 = 32;

/// Default distance between the two front cells (our tool's choice).
pub const DEFAULT_DISTANCE: i32 = 10;

/// Default tick cap (our tool's choice).
pub const DEFAULT_TICK_CAP: i32 = 6000;

/// The significance level of the arena comparison: each test (the win rate
/// of either group, the times to kill) fails below it. Our test design
/// choice, not a game value: 1 % keeps false alarms rare over many
/// matchups while 32 trials still catch large differences.
pub const ALPHA: f64 = 0.01;

/// One matchup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scenario {
    /// Group A's and group B's members: type ID and count.
    pub groups: [Vec<(String, u32)>; 2],
    /// Country of house A and house B (`None`: the default).
    pub countries: [Option<String>; 2],
    /// Map size in cells.
    pub map_size: i32,
    /// Cells between the two front cells.
    pub start_distance: i32,
    /// Ticks from the order tick before a trial is a timeout.
    pub tick_cap: i32,
    /// Lobby game speed (`None`: the rules' lobby speed).
    pub game_speed: Option<u8>,
}

impl Scenario {
    /// A scenario of two groups with the defaults.
    pub fn new(a: Vec<(String, u32)>, b: Vec<(String, u32)>) -> Self {
        Self {
            groups: [a, b],
            countries: [None, None],
            map_size: DEFAULT_MAP_SIZE,
            start_distance: DEFAULT_DISTANCE,
            tick_cap: DEFAULT_TICK_CAP,
            game_speed: None,
        }
    }

    /// The groups as written in records: `TYPE*n, TYPE*n` and so on.
    pub fn group_text(&self, side: usize) -> String {
        self.groups[side]
            .iter()
            .map(|(t, n)| format!("{t}*{n}"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Parses a group: `TYPE*n, TYPE` (comma or space separated; count 1 when
/// left out).
pub fn parse_group(s: &str) -> Result<Vec<(String, u32)>, String> {
    let mut out = Vec::new();
    for part in s.split([',', ' ']).filter(|p| !p.trim().is_empty()) {
        let part = part.trim();
        let (ty, n) = match part.split_once('*') {
            Some((t, n)) => (
                t.trim(),
                n.trim()
                    .parse::<u32>()
                    .map_err(|_| format!("bad count in {part:?}"))?,
            ),
            None => (part, 1),
        };
        if ty.is_empty() || n == 0 {
            return Err(format!("bad group member {part:?}"));
        }
        out.push((ty.to_owned(), n));
    }
    if out.is_empty() {
        return Err("empty group".into());
    }
    Ok(out)
}

/// The lobby game speeds a scenario may name (spec: S-0201).
pub const GAME_SPEEDS: std::ops::RangeInclusive<u8> = 0..=6;

/// Parses a game speed: a whole number in [`GAME_SPEEDS`], the game-speed
/// index range of `GameSetup::game_speed` (spec: S-0201).
pub fn parse_speed(v: &str) -> Result<u8, String> {
    v.trim()
        .parse::<u8>()
        .ok()
        .filter(|x| GAME_SPEEDS.contains(x))
        .ok_or_else(|| format!("game speed {v:?}: expected a whole number from 0 to 6"))
}

/// How a trial ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Group A won.
    A,
    /// Group B won.
    B,
    /// Both groups were gone at the end of the same tick.
    Draw,
    /// The tick cap was reached.
    Timeout,
}

impl Outcome {
    fn text(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::Draw => "draw",
            Self::Timeout => "timeout",
        }
    }

    /// Whether the trial was decided (not a timeout): only decided trials
    /// give a time to kill.
    pub fn decided(self) -> bool {
        self != Self::Timeout
    }
}

/// One trial's outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Trial {
    /// How it ended.
    pub outcome: Outcome,
    /// Time to kill in ticks (the cap for a timeout).
    pub ticks: i64,
    /// Units of group A and of group B left.
    pub survivors: [u32; 2],
    /// Their health in per-mille of each group's full health, if known.
    pub health: Option<[u32; 2]>,
}

impl Trial {
    /// The trial as a record's trial line (without `trial: `).
    pub fn line(&self) -> String {
        let mut s = format!(
            "{} {} {} {}",
            self.outcome.text(),
            self.ticks,
            self.survivors[0],
            self.survivors[1]
        );
        if let Some([a, b]) = self.health {
            let _ = write!(s, " {a} {b}");
        }
        s
    }
}

/// Parses a trial line: `<winner> <ticks> <survivors-a> <survivors-b>
/// [<health-a> <health-b>]`.
pub fn parse_trial(s: &str) -> Result<Trial, String> {
    let f: Vec<&str> = s.split_whitespace().collect();
    let outcome = match f.first().copied() {
        Some("A" | "a") => Outcome::A,
        Some("B" | "b") => Outcome::B,
        Some("draw") => Outcome::Draw,
        Some("timeout") => Outcome::Timeout,
        _ => return Err(format!("trial {s:?}: winner must be A, B, draw or timeout")),
    };
    if f.len() != 4 && f.len() != 6 {
        return Err(format!("trial {s:?}: expected 4 or 6 fields"));
    }
    let num = |i: usize| {
        f[i].parse::<u32>()
            .map_err(|_| format!("trial {s:?}: field {} is not a whole number", i + 1))
    };
    Ok(Trial {
        outcome,
        ticks: i64::from(num(1)?),
        survivors: [num(2)?, num(3)?],
        health: if f.len() == 6 {
            Some([num(4)?, num(5)?])
        } else {
            None
        },
    })
}

/// The rules an arena runs on.
pub enum ArenaRules {
    /// The synthetic combat ruleset ([`testkit::combat_ruleset`]).
    Synthetic,
    /// The reference ruleset imported from an install.
    Reference(Arc<SimRules>),
}

impl ArenaRules {
    /// The rules.
    pub fn rules(&self) -> Arc<SimRules> {
        match self {
            Self::Synthetic => testkit::combat_rules(),
            Self::Reference(r) => r.clone(),
        }
    }

    /// A name for reports.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Synthetic => "synthetic",
            Self::Reference(_) => "reference",
        }
    }
}

/// Imports the reference ruleset of the install in `dir` (the local lane's
/// import: the game's search order, the rules and art without any map
/// layer, temperate theater) and resolves it. The outer error is the
/// install's; the inner one says why the rules did not resolve.
pub fn reference_rules(dir: &Path) -> std::io::Result<Result<Arc<SimRules>, String>> {
    pollster::block_on(async {
        let mut vfs = crate::local_lane::mount(dir).await?;
        let options = glasnost_convert::ImportOptions {
            theater: Some(glasnost_vfs::theater::Theater::Temperate),
            ..Default::default()
        };
        let import = glasnost_convert::import(&mut vfs, &options).await;
        Ok(
            glasnost_rules::resolve_ruleset(Some((import.tree, import.provenance)), &[])
                .map(|r| Arc::new(SimRules::new(&r.ruleset, &r.sim_rules_hash)))
                .map_err(|e| e.to_string()),
        )
    })
}

/// The default country: the rules' first country that is not passive, nor
/// the neutral or special one.
fn default_country(rules: &SimRules) -> Option<String> {
    rules
        .country_names
        .iter()
        .zip(&rules.countries)
        .find(|(n, c)| {
            !c.passive
                && n.as_str() != glasnost_sim::load::NEUTRAL
                && n.as_str() != glasnost_sim::load::SPECIAL
        })
        .map(|(n, _)| n.clone())
}

/// The kind of a type ID: infantry, else vehicle.
fn kind_of(rules: &SimRules, name: &str) -> Option<Kind> {
    [Kind::Infantry, Kind::Vehicle]
        .into_iter()
        .find(|&k| rules.index_of(k, name).is_some())
}

/// Whether the trial of `seed` is swapped: an even seed. A swapped trial
/// relabels the groups: group B is house 0, stands on the lower-x side, is
/// created first and is ordered first. Over an even number of consecutive
/// seeds this balances the two things that otherwise favour one group of a
/// mirror matchup: the side of the map, whose geometry differs (with an
/// odd start distance the fronts are not equally far from the middle;
/// with an even one the middle cell lies on lepton boundaries that the
/// two sides cross differently), and the update order, which is the order
/// of entering play (S-0203 claim 3), so the group created first takes its
/// turns first.
pub fn swapped(seed: u32) -> bool {
    seed.is_multiple_of(2)
}

/// The house, and the place (0: the lower-x side), of group `side`.
fn house(side: usize, swapped: bool) -> usize {
    side ^ usize::from(swapped)
}

/// The front cell of place `place` (0: the lower-x side).
fn front(sc: &Scenario, place: usize) -> CellXY {
    let half = sc.start_distance / 2;
    if place == 0 {
        testkit::mid(sc.map_size, -half, 0)
    } else {
        testkit::mid(sc.map_size, sc.start_distance - half, 0)
    }
}

/// The cell of the `i`-th unit of the group at place `place`.
fn slot(sc: &Scenario, place: usize, i: usize) -> CellXY {
    const ACROSS: [i32; ROW] = [0, 1, -1, 2, -2];
    let f = front(sc, place);
    let back = (i / ROW) as i32;
    let dx = if place == 0 { -back } else { back };
    CellXY::new(f.x + dx, f.y + ACROSS[i % ROW])
}

/// The game of one trial with its groups placed.
pub struct Arena {
    /// The world.
    pub world: World,
    /// Each group's units, in creation order.
    pub units: [Vec<ObjectId>; 2],
    /// Each group's summed health at creation.
    pub full: [i64; 2],
    /// The trial is swapped ([`swapped`]).
    pub swapped: bool,
}

/// Sets up the game of one trial and places its groups.
pub fn setup_trial(rules: Arc<SimRules>, sc: &Scenario, seed: u32) -> Result<Arena, String> {
    let mut countries = [String::new(), String::new()];
    for (side, c) in countries.iter_mut().enumerate() {
        *c = match &sc.countries[side] {
            Some(c) => c.clone(),
            None => default_country(&rules).ok_or("the rules have no player country")?,
        };
    }
    let new_house = |name: &str, country: &str| HouseSetup {
        name: name.into(),
        country: country.into(),
        human: true,
        start: None,
        role: HouseRole::Player,
        ai: false,
    };
    let swap = swapped(seed);
    // House 0 is group A's, or group B's in a swapped trial.
    let (first, second) = if swap { (1, 0) } else { (0, 1) };
    let setup = GameSetup {
        seed,
        game_speed: sc.game_speed.unwrap_or(rules.general.lobby_game_speed),
        houses: vec![
            new_house(["A", "B"][first], &countries[first]),
            new_house(["A", "B"][second], &countries[second]),
        ],
        alliances: Vec::new(),
        sandbox: true,
        unit_count: 0,
        bases: false,
        skirmish: Default::default(),
    };
    let map = testkit::empty_map(sc.map_size);
    let mut w = glasnost_sim::load::new_game(rules.clone(), &map, testkit::catalogue(), &setup)
        .map_err(|e| e.to_string())?;
    let table = w.rules().quirks.table_direction;
    let mut units: [Vec<ObjectId>; 2] = [Vec::new(), Vec::new()];
    let mut full = [0i64; 2];
    // Created in house order: house 0's group first (S-0203 claim 3).
    for side in [first, second] {
        let h = house(side, swap);
        let (me, them) = (front(sc, h), front(sc, 1 - h));
        let facing = glasnost_num::geom::direction(
            i64::from(them.x - me.x) * 256,
            i64::from(them.y - me.y) * 256,
            table,
        );
        let mut i = 0;
        for (ty, n) in &sc.groups[side] {
            let kind = kind_of(&rules, ty)
                .ok_or_else(|| format!("{ty}: no infantry or vehicle type of that ID"))?;
            for _ in 0..*n {
                let cell = slot(sc, h, i);
                if map.cells.get(cell).is_none() {
                    return Err(format!(
                        "group {} does not fit on a map of {}: make it larger",
                        ["A", "B"][side],
                        sc.map_size
                    ));
                }
                // The nearest facing value (a tool rule; exact along the x axis).
                let v = (u32::from(facing).wrapping_add(128) >> 8) as u8;
                let id = testkit::spawn_in_play(&mut w, kind, ty, h as u8, cell, v, None)
                    .ok_or_else(|| format!("{ty}: could not be placed"))?;
                let o = w.obj(id).ok_or_else(|| format!("{ty}: not created"))?;
                full[side] += i64::from(o.health);
                units[side].push(id);
                i += 1;
            }
        }
    }
    Ok(Arena {
        world: w,
        units,
        full,
        swapped: swap,
    })
}

/// The units of a group: still there, health above 0, owned by its house
/// `h`.
fn alive(w: &World, units: &[ObjectId], h: usize) -> Vec<ObjectId> {
    units
        .iter()
        .copied()
        .filter(|&id| {
            w.obj(id)
                .is_some_and(|o| o.health > 0 && o.owner == Some(HouseId(h as u8)))
        })
        .collect()
}

fn attack_move(actor: ObjectId, c: CellXY) -> Order {
    Order::Behaviour {
        actor,
        behaviour: Behaviour::Move,
        attack_move: true,
        target: None,
        dest: Some(Target::Cell(c)),
    }
}

/// Idle: guarding or area guarding (or no behaviour) with no target and no
/// destination.
fn idle(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        matches!(
            o.beh.current,
            None | Some(Behaviour::Guard) | Some(Behaviour::AreaGuard)
        ) && o.target.is_none()
            && o.dest.is_none()
    })
}

/// Orders every idle unit of group `side` to attack-move to the nearest
/// unit of the other group.
fn reorder_idle(w: &mut World, units: &[Vec<ObjectId>; 2], side: usize, swapped: bool) {
    let (h, them) = (house(side, swapped), house(1 - side, swapped));
    let enemies: Vec<(ObjectId, glasnost_sim::world::Pos)> = alive(w, &units[1 - side], them)
        .into_iter()
        .filter_map(|id| w.obj(id).map(|o| (id, o.pos)))
        .collect();
    for id in alive(w, &units[side], h) {
        if !idle(w, id) {
            continue;
        }
        let Some(p) = w.obj(id).map(|o| o.pos) else {
            continue;
        };
        let nearest = enemies.iter().min_by_key(|(eid, q)| {
            let (dx, dy) = (i64::from(q.x - p.x), i64::from(q.y - p.y));
            (dx * dx + dy * dy, *eid)
        });
        if let Some((_, q)) = nearest {
            orders::issue(w, HouseId(h as u8), attack_move(id, q.cell()));
        }
    }
}

/// Plays one trial.
pub fn run_trial(rules: Arc<SimRules>, sc: &Scenario, seed: u32) -> Result<Trial, String> {
    run_trial_watched(rules, sc, seed, false).map(|(t, _)| t)
}

/// Plays one trial; with `watch`, also describes it: each group's shots,
/// and every unit's state at the end (owner, health, cell, behaviour,
/// target, destination). A diagnostic of our tool.
pub fn run_trial_watched(
    rules: Arc<SimRules>,
    sc: &Scenario,
    seed: u32,
    watch: bool,
) -> Result<(Trial, String), String> {
    let Arena {
        world: mut w,
        units,
        full,
        swapped,
    } = setup_trial(rules, sc, seed)?;
    // The groups in house order.
    let order = if swapped { [1, 0] } else { [0, 1] };
    let mut shots = [0u32; 2];
    let mut count_shots = |w: &mut World| {
        for e in w.effects.drain(..) {
            if let glasnost_sim::combat::effects::Effect::Shot { firer, .. } = e {
                for side in 0..2 {
                    if units[side].contains(&firer) {
                        shots[side] += 1;
                    }
                }
            }
        }
    };
    // The first tick in which orders are taken.
    let mut guard = 0;
    while orders::blocked(&w) {
        glasnost_sim::tick::step(&mut w);
        w.effects.clear();
        guard += 1;
        if guard > 10_000 {
            return Err("orders never open".into());
        }
    }
    for side in order {
        let h = house(side, swapped);
        let target = front(sc, 1 - h);
        for id in alive(&w, &units[side], h) {
            orders::issue(&mut w, HouseId(h as u8), attack_move(id, target));
        }
    }
    let mut ticks: i64 = 0;
    let mut checks: BTreeMap<String, u32> = BTreeMap::new();
    let outcome = loop {
        if ticks > 0 && ticks % i64::from(REISSUE_PERIOD) == 0 {
            for side in order {
                reorder_idle(&mut w, &units, side, swapped);
            }
        }
        glasnost_sim::tick::step(&mut w);
        count_shots(&mut w);
        if watch {
            tally_checks(&w, &units, swapped, &mut checks);
        }
        ticks += 1;
        let a = alive(&w, &units[0], house(0, swapped)).len();
        let b = alive(&w, &units[1], house(1, swapped)).len();
        match (a, b) {
            (0, 0) => break Outcome::Draw,
            (_, 0) => break Outcome::A,
            (0, _) => break Outcome::B,
            _ if ticks >= i64::from(sc.tick_cap) => break Outcome::Timeout,
            _ => {}
        }
    };
    let mut survivors = [0u32; 2];
    let mut health = [0u32; 2];
    for (side, ids) in units.iter().enumerate() {
        let left = alive(&w, ids, house(side, swapped));
        survivors[side] = left.len() as u32;
        let hp: i64 = left
            .iter()
            .filter_map(|&id| w.obj(id).map(|o| i64::from(o.health)))
            .sum();
        health[side] = if full[side] > 0 {
            (hp * 1000 / full[side]) as u32
        } else {
            0
        };
    }
    let mut seen = String::new();
    if watch {
        let _ = writeln!(
            seen,
            "  seed {seed}{}: shots A {}, B {}",
            if swapped { " (swapped)" } else { "" },
            shots[0],
            shots[1]
        );
        for (k, n) in &checks {
            let _ = writeln!(seen, "    firing check {k}: {n} unit-ticks");
        }
        for (side, ids) in units.iter().enumerate() {
            for &id in ids {
                let Some(o) = w.obj(id) else {
                    let _ = writeln!(seen, "    {} #{}: gone", ["A", "B"][side], id.0);
                    continue;
                };
                let _ = writeln!(
                    seen,
                    "    {} #{}: owner {:?}, health {}, on map {}, cell ({}, {}), behaviour {:?}, target {:?}, destination {:?}",
                    ["A", "B"][side],
                    id.0,
                    o.owner.map(|h| h.0),
                    o.health,
                    o.on_map,
                    o.pos.cell().x,
                    o.pos.cell().y,
                    o.beh.current,
                    o.target,
                    o.dest
                );
            }
        }
    }
    Ok((
        Trial {
            outcome,
            ticks,
            survivors,
            health: Some(health),
        },
        seen,
    ))
}

/// Counts, per group and weapon slot, the firing check's answers (S-0301
/// claim 9) of every unit of the group that has a target, for `--watch`.
fn tally_checks(
    w: &World,
    units: &[Vec<ObjectId>; 2],
    swapped: bool,
    checks: &mut BTreeMap<String, u32>,
) {
    for (side, ids) in units.iter().enumerate() {
        for id in alive(w, ids, house(side, swapped)) {
            let Some(t) = w.obj(id).and_then(|o| o.target) else {
                continue;
            };
            for slot in 0..2 {
                let a = glasnost_sim::combat::check::check(w, id, Some(t), slot, true);
                *checks
                    .entry(format!("{} slot {slot} {a:?}", ["A", "B"][side]))
                    .or_insert(0) += 1;
            }
        }
    }
}

/// Plays `n` trials with seeds `first_seed`, `first_seed + 1`, ….
pub fn run_matchup(
    rules: Arc<SimRules>,
    sc: &Scenario,
    first_seed: u32,
    n: u32,
) -> Result<Vec<Trial>, String> {
    (0..n)
        .map(|i| run_trial(rules.clone(), sc, first_seed.wrapping_add(i)))
        .collect()
}

/// Counts of a set of trials: wins of A, wins of B, draws, timeouts.
pub fn counts(trials: &[Trial]) -> [u32; 4] {
    let mut c = [0u32; 4];
    for t in trials {
        c[match t.outcome {
            Outcome::A => 0,
            Outcome::B => 1,
            Outcome::Draw => 2,
            Outcome::Timeout => 3,
        }] += 1;
    }
    c
}

/// The times to kill of the decided trials.
pub fn times_to_kill(trials: &[Trial]) -> Vec<i64> {
    trials
        .iter()
        .filter(|t| t.outcome.decided())
        .map(|t| t.ticks)
        .collect()
}

fn pct(x: u32, n: usize) -> f64 {
    if n == 0 {
        0.0
    } else {
        100.0 * f64::from(x) / n as f64
    }
}

/// The report of one matchup: the scenario, N, win rates, draws and
/// timeouts, mean and standard deviation of the time to kill, survivors,
/// and every trial as a record trial line.
pub fn report(
    rules_name: &str,
    sc: &Scenario,
    speed: u8,
    first_seed: u32,
    trials: &[Trial],
) -> String {
    let mut s = String::new();
    let n = trials.len();
    let [a, b, d, t] = counts(trials);
    let _ = writeln!(
        s,
        "matchup: A {} vs B {} ({rules_name} ruleset)",
        sc.group_text(0),
        sc.group_text(1)
    );
    let _ = writeln!(
        s,
        "  map {0}x{0}, start distance {1}, tick cap {2}, game speed {speed}, countries {3} / {4}, seeds {first_seed} to {5}",
        sc.map_size,
        sc.start_distance,
        sc.tick_cap,
        sc.countries[0].as_deref().unwrap_or("(default)"),
        sc.countries[1].as_deref().unwrap_or("(default)"),
        (u64::from(first_seed) + n as u64).saturating_sub(1),
    );
    let _ = writeln!(
        s,
        "  N {n}: A wins {a} ({:.1} %), B wins {b} ({:.1} %), draws {d}, timeouts {t}",
        pct(a, n),
        pct(b, n)
    );
    let ttk: Vec<f64> = times_to_kill(trials).iter().map(|&x| x as f64).collect();
    match stats::mean_sd(&ttk) {
        Some((m, sd)) => {
            let min = ttk.iter().copied().fold(f64::INFINITY, f64::min);
            let max = ttk.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let _ = writeln!(
                s,
                "  time to kill (ticks, {} decided trials): mean {m:.1}, sd {sd:.1}, min {min}, max {max}",
                ttk.len()
            );
        }
        None => {
            let _ = writeln!(s, "  time to kill: no decided trial");
        }
    }
    for (side, name) in ["A", "B"].iter().enumerate() {
        let units: Vec<f64> = trials
            .iter()
            .map(|t| f64::from(t.survivors[side]))
            .collect();
        let hp: Vec<f64> = trials
            .iter()
            .filter_map(|t| t.health.map(|h| f64::from(h[side])))
            .collect();
        let (mu, su) = stats::mean_sd(&units).unwrap_or((0.0, 0.0));
        let (mh, sh) = stats::mean_sd(&hp).unwrap_or((0.0, 0.0));
        let _ = writeln!(
            s,
            "  survivors {name}: mean {mu:.2} units (sd {su:.2}), mean health {mh:.0} ‰ of full (sd {sh:.0})"
        );
    }
    let _ = writeln!(
        s,
        "  trials (record lines: winner ticks survivors-a survivors-b health-a health-b):"
    );
    for (i, tr) in trials.iter().enumerate() {
        let _ = writeln!(
            s,
            "    trial: {}    # seed {}",
            tr.line(),
            u64::from(first_seed) + i as u64
        );
    }
    s
}

/// The infantry and vehicle type IDs of `rules` that a matchup can name,
/// in registration order, each with its kind, its tech level and the sides
/// (by side index) of the countries that own it. For picking matchups from
/// the loaded rules; on the reference ruleset the list is local output
/// only.
pub fn type_list(rules: &SimRules) -> String {
    use glasnost_sim::econ::TypeRef;
    let mut s = String::from("kind      index  tech  sides  type ID\n");
    let mut rows: Vec<(Kind, u32, &str)> = rules
        .names
        .iter()
        .filter(|((k, _), _)| matches!(k, Kind::Infantry | Kind::Vehicle))
        .map(|((k, n), &i)| (*k, i, n.as_str()))
        .collect();
    rows.sort_by_key(|&(k, i, _)| (k as u8, i));
    for (kind, index, name) in rows {
        let tech = rules.econ.tech.of(TypeRef { kind, index });
        let mut sides: Vec<i32> = tech
            .map(|t| {
                t.owners
                    .iter()
                    .filter_map(|&c| {
                        rules
                            .econ
                            .tech
                            .country_side
                            .get(c as usize)
                            .copied()
                            .flatten()
                    })
                    .collect()
            })
            .unwrap_or_default();
        sides.sort_unstable();
        sides.dedup();
        let sides = if sides.is_empty() {
            "-".to_owned()
        } else {
            sides
                .iter()
                .map(i32::to_string)
                .collect::<Vec<_>>()
                .join(",")
        };
        let _ = writeln!(
            s,
            "{:<9} {index:5}  {:4}  {sides:5}  {name}",
            format!("{kind:?}"),
            tech.map_or(-1, |t| t.tech_level)
        );
    }
    s
}

// ---------------------------------------------------------------------------
// Command line

/// The `arena` command line of `glasnost-headless`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cli {
    /// The matchup given on the command line (`None` with `--records`).
    pub scenario: Option<Scenario>,
    /// Trials per matchup (`None`: 32, or the record's N with `--records`).
    pub seeds: Option<u32>,
    /// The first seed.
    pub first_seed: u32,
    /// Run every arena record's matchup and compare.
    pub records: bool,
    /// Use the synthetic ruleset even when an install is named.
    pub synthetic: bool,
    /// List the type IDs a matchup can name, and run nothing.
    pub types: bool,
    /// Describe the first trial (shots, the units' state at the end).
    pub watch: bool,
}

/// Trials per command-line matchup when `--seeds` is not given.
pub const DEFAULT_SEEDS: u32 = 32;

/// The usage line of the `arena` command.
pub const USAGE: &str = "arena [--seeds N] [--first-seed K] [--ticks CAP] [--size S] [--distance D] [--speed G] [--countries X,Y] [--synthetic] [--watch] \"A:TYPE*n,TYPE*n\" \"B:TYPE*n\" | arena --records [--seeds N] [--first-seed K] | arena --types";

/// Parses the arguments after `arena`.
pub fn parse_cli(args: &[String]) -> Result<Cli, String> {
    let mut sc = Scenario::new(Vec::new(), Vec::new());
    let mut cli = Cli {
        scenario: None,
        seeds: None,
        first_seed: 1,
        records: false,
        synthetic: false,
        types: false,
        watch: false,
    };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut value = |name: &str| {
            it.next()
                .cloned()
                .ok_or_else(|| format!("{name} needs a value"))
        };
        let int = |name: &str, v: String| {
            v.parse::<i64>()
                .ok()
                .filter(|x| *x > 0)
                .ok_or_else(|| format!("{name}: {v:?} is not a positive whole number"))
        };
        match a.as_str() {
            "--seeds" => cli.seeds = Some(int(a, value(a)?)? as u32),
            "--first-seed" => cli.first_seed = int(a, value(a)?)? as u32,
            "--ticks" => sc.tick_cap = int(a, value(a)?)? as i32,
            "--size" => sc.map_size = int(a, value(a)?)? as i32,
            "--distance" => sc.start_distance = int(a, value(a)?)? as i32,
            "--speed" => sc.game_speed = Some(parse_speed(&value(a)?)?),
            "--countries" => {
                let v = value(a)?;
                let (x, y) = v.split_once(',').ok_or("--countries needs X,Y")?;
                sc.countries = [Some(x.trim().to_owned()), Some(y.trim().to_owned())];
            }
            "--records" => cli.records = true,
            "--synthetic" => cli.synthetic = true,
            "--types" => cli.types = true,
            "--watch" => cli.watch = true,
            g if g.len() > 2 && (g.starts_with("A:") || g.starts_with("a:")) => {
                sc.groups[0] = parse_group(&g[2..])?
            }
            g if g.len() > 2 && (g.starts_with("B:") || g.starts_with("b:")) => {
                sc.groups[1] = parse_group(&g[2..])?
            }
            other => return Err(format!("unknown argument {other:?}")),
        }
    }
    if cli.types {
        return Ok(cli);
    }
    match (cli.records, sc.groups.iter().all(|g| !g.is_empty())) {
        (true, false) => {}
        (true, true) => return Err("give either --records or two groups, not both".into()),
        (false, true) => cli.scenario = Some(sc),
        (false, false) => return Err("give the two groups as \"A:TYPE*n\" \"B:TYPE*n\"".into()),
    }
    Ok(cli)
}

/// Runs the command line on `rules` and returns the report. With
/// `--records`, `measurements` is the folder the records are read from.
pub fn run_cli(cli: &Cli, rules: &ArenaRules, measurements: &Path) -> Result<String, String> {
    let r = rules.rules();
    let mut out = String::new();
    let speed = |sc: &Scenario| sc.game_speed.unwrap_or(r.general.lobby_game_speed);
    if cli.types {
        out.push_str(&type_list(&r));
        return Ok(out);
    }
    if let Some(sc) = &cli.scenario {
        let n = cli.seeds.unwrap_or(DEFAULT_SEEDS);
        let trials = run_matchup(r.clone(), sc, cli.first_seed, n)?;
        out.push_str(&report(
            rules.name(),
            sc,
            speed(sc),
            cli.first_seed,
            &trials,
        ));
        if cli.watch {
            let (_, seen) = run_trial_watched(r.clone(), sc, cli.first_seed, true)?;
            out.push_str("watch (first trial):\n");
            out.push_str(&seen);
        }
        return Ok(out);
    }
    let records = load_records(measurements, true)?;
    if records.is_empty() {
        out.push_str("skipped: no arena measurement record in measurements/\n");
        return Ok(out);
    }
    let mut failed = 0;
    for rec in &records {
        let n = cli.seeds.unwrap_or(rec.trials.len() as u32);
        let trials = run_matchup(r.clone(), &rec.scenario, cli.first_seed, n)?;
        let _ = writeln!(out, "{} (arena block {}):", rec.id, rec.block + 1);
        out.push_str(&report(
            rules.name(),
            &rec.scenario,
            speed(&rec.scenario),
            cli.first_seed,
            &trials,
        ));
        let c = compare(&rec.trials, &trials, ALPHA);
        out.push_str(&c.report(ALPHA));
        if !c.pass {
            failed += 1;
        }
    }
    let _ = writeln!(
        out,
        "arena comparison: {} matchups, {failed} failed (α = {ALPHA} per test)",
        records.len()
    );
    Ok(out)
}

// ---------------------------------------------------------------------------
// Records

/// One matchup of an arena measurement record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArenaRecord {
    /// The record's ID (front matter `id`).
    pub id: String,
    /// Which `arena` block of the record (0 for the first).
    pub block: usize,
    /// The scenario.
    pub scenario: Scenario,
    /// The stock game's trials (empty when the outcome keys were not
    /// read: see [`parse_record`]).
    pub trials: Vec<Trial>,
}

/// The front matter's `key: value` lines (comments after `#` dropped).
fn front_matter(text: &str) -> Vec<(String, String)> {
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some("---") {
        return Vec::new();
    }
    lines
        .take_while(|l| l.trim() != "---")
        .filter_map(|l| {
            let l = l.split('#').next().unwrap_or("");
            let (k, v) = l.split_once(':')?;
            Some((k.trim().to_owned(), v.trim().to_owned()))
        })
        .collect()
}

/// The `arena` blocks of a record, each as its `key: value` lines.
fn arena_blocks(text: &str) -> Result<Vec<Vec<(String, String)>>, String> {
    let mut blocks = Vec::new();
    let mut current: Option<Vec<(String, String)>> = None;
    for (n, line) in text.lines().enumerate() {
        let t = line.trim();
        match &mut current {
            None if t.starts_with("```") && t.trim_start_matches('`').trim() == "arena" => {
                current = Some(Vec::new());
            }
            None => {}
            Some(_) if t.starts_with("```") => blocks.extend(current.take()),
            Some(lines) => {
                let content = t.split('#').next().unwrap_or("").trim();
                if content.is_empty() {
                    continue;
                }
                let (k, v) = content
                    .split_once(':')
                    .ok_or_else(|| format!("line {}: expected `key: value`", n + 1))?;
                lines.push((k.trim().to_owned(), v.trim().to_owned()));
            }
        }
    }
    if current.is_some() {
        return Err("an arena block is not closed".into());
    }
    Ok(blocks)
}

/// The outcome keys of block `i` from a local values file.
fn local_block(json: &str, i: usize) -> Result<Vec<(String, String)>, String> {
    use glasnost_rules::jsonc::{self, Value};
    let root = jsonc::parse(json).map_err(|e| format!("local values file: {e}"))?;
    let blocks = match root.get("arena").map(|n| &n.value) {
        Some(Value::Array(a)) => a,
        _ => return Err("local values file: no \"arena\" array".into()),
    };
    let block = blocks
        .get(i)
        .ok_or_else(|| format!("local values file: no entry for arena block {}", i + 1))?;
    let members = block
        .members()
        .ok_or("local values file: an arena entry is not an object")?;
    let scalar = |v: &Value| match v {
        Value::Integer(n) => Some(n.text().to_owned()),
        Value::String(s) => Some(s.clone()),
        _ => None,
    };
    let mut out = Vec::new();
    for m in members {
        match &m.value.value {
            Value::Array(items) => {
                for it in items {
                    let v = scalar(&it.value)
                        .ok_or_else(|| format!("local values file: bad item in {:?}", m.key))?;
                    out.push((m.key.clone(), v));
                }
            }
            v => {
                let v = scalar(v)
                    .ok_or_else(|| format!("local values file: bad value of {:?}", m.key))?;
                out.push((m.key.clone(), v));
            }
        }
    }
    Ok(out)
}

const OUTCOME_KEYS: [&str; 6] = ["n", "wins-a", "wins-b", "draws", "timeouts", "trial"];

/// Builds a block's scenario and trials from its lines. With `outcomes`
/// false the outcome keys are not required.
fn parse_block(
    lines: &[(String, String)],
    outcomes: bool,
) -> Result<(Scenario, Vec<Trial>), String> {
    let mut sc = Scenario::new(Vec::new(), Vec::new());
    let mut n = None;
    let mut cross: [Option<u32>; 4] = [None; 4];
    let mut trials = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    for (k, v) in lines {
        if k != "trial" {
            if seen.contains(&k.as_str()) {
                return Err(format!("{k} is given twice"));
            }
            seen.push(k);
        }
        let int = || {
            v.parse::<i64>()
                .map_err(|_| format!("{k}: {v:?} is not a whole number"))
        };
        let pos = || -> Result<i32, String> {
            let x = int()?;
            i32::try_from(x)
                .ok()
                .filter(|x| *x > 0)
                .ok_or_else(|| format!("{k}: {v} out of range"))
        };
        let count = || -> Result<u32, String> {
            u32::try_from(int()?).map_err(|_| format!("{k}: {v} out of range"))
        };
        match k.as_str() {
            "group-a" => sc.groups[0] = parse_group(v)?,
            "group-b" => sc.groups[1] = parse_group(v)?,
            "country-a" => sc.countries[0] = Some(v.clone()),
            "country-b" => sc.countries[1] = Some(v.clone()),
            "map-size" => sc.map_size = pos()?,
            "start-distance" => sc.start_distance = pos()?,
            "tick-cap" => sc.tick_cap = pos()?,
            "game-speed" => sc.game_speed = Some(parse_speed(v)?),
            "n" => n = Some(count()?),
            "wins-a" => cross[0] = Some(count()?),
            "wins-b" => cross[1] = Some(count()?),
            "draws" => cross[2] = Some(count()?),
            "timeouts" => cross[3] = Some(count()?),
            "trial" => trials.push(parse_trial(v)?),
            other => return Err(format!("unknown key {other:?}")),
        }
    }
    if sc.groups.iter().any(Vec::is_empty) {
        return Err("group-a and group-b are required".into());
    }
    if !outcomes {
        return Ok((sc, Vec::new()));
    }
    let n = n.ok_or("n is required")?;
    if trials.len() != n as usize {
        return Err(format!(
            "n is {n} but there are {} trial lines",
            trials.len()
        ));
    }
    let c = counts(&trials);
    for (i, name) in ["wins-a", "wins-b", "draws", "timeouts"].iter().enumerate() {
        if let Some(x) = cross[i]
            && x != c[i]
        {
            return Err(format!("{name} is {x} but the trial lines give {}", c[i]));
        }
    }
    Ok((sc, trials))
}

/// Reads the arena matchups of one record. `local` is the text of the
/// record's local values file (`measurements/local/<id>.json`), if any;
/// it is read when the front matter says `value-visibility: local`. With
/// `outcomes` false only the scenarios are read (CI can check that every
/// record parses without the local files).
pub fn parse_record(
    text: &str,
    local: Option<&str>,
    outcomes: bool,
) -> Result<Vec<ArenaRecord>, String> {
    let fm = front_matter(text);
    let get = |k: &str| fm.iter().find(|(key, _)| key == k).map(|(_, v)| v.clone());
    let id = get("id").ok_or("the front matter has no id")?;
    let is_local = get("value-visibility").as_deref() == Some("local");
    let mut out = Vec::new();
    for (i, mut lines) in arena_blocks(text)?.into_iter().enumerate() {
        if is_local && outcomes {
            let json = local.ok_or_else(|| {
                format!(
                    "{id} is value-visibility: local but measurements/local/{id}.json is missing"
                )
            })?;
            let extra = local_block(json, i)?;
            for (k, _) in &extra {
                if !OUTCOME_KEYS.contains(&k.as_str()) {
                    return Err(format!(
                        "{id}: local values file key {k:?} is not an outcome key"
                    ));
                }
                if lines.iter().any(|(lk, _)| lk == k) {
                    return Err(format!(
                        "{id}: {k} is in both the record and the local file"
                    ));
                }
            }
            lines.extend(extra);
        }
        let (scenario, trials) = parse_block(&lines, outcomes)
            .map_err(|e| format!("{id}, arena block {}: {e}", i + 1))?;
        out.push(ArenaRecord {
            id: id.clone(),
            block: i,
            scenario,
            trials,
        });
    }
    Ok(out)
}

/// Every measurement record file under `dir` (recursively, outside
/// `local/`) with an `arena` block, sorted by path.
pub fn record_files(dir: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for e in entries.flatten() {
            let p = e.path();
            let name = e.file_name().to_string_lossy().into_owned();
            if p.is_dir() {
                if name != "local" {
                    walk(&p, out);
                }
            } else if name.starts_with("M-")
                && name.ends_with(".md")
                && std::fs::read_to_string(&p).is_ok_and(|t| {
                    t.lines().any(|l| {
                        l.trim().trim_start_matches('`').trim() == "arena"
                            && l.trim().starts_with("```")
                    })
                })
            {
                out.push(p);
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, &mut out);
    out.sort();
    out
}

/// Reads every arena record under `dir` (the `measurements/` folder), with
/// local values from `dir/local/`.
pub fn load_records(dir: &Path, outcomes: bool) -> Result<Vec<ArenaRecord>, String> {
    let mut out = Vec::new();
    for path in record_files(dir) {
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let id = front_matter(&text)
            .into_iter()
            .find(|(k, _)| k == "id")
            .map(|(_, v)| v)
            .unwrap_or_default();
        let local = std::fs::read_to_string(dir.join("local").join(format!("{id}.json"))).ok();
        out.extend(parse_record(&text, local.as_deref(), outcomes)?);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Comparison

/// Our matchup against the stock record.
#[derive(Clone, Debug)]
pub struct Comparison {
    /// Win rates of A and B, stock.
    pub stock_rates: [f64; 2],
    /// Win rates of A and B, ours.
    pub our_rates: [f64; 2],
    /// Two-proportion tests of A's and B's win rates.
    pub rate_tests: [Option<stats::ProportionTest>; 2],
    /// The KS test of the times to kill of the decided trials.
    pub ks: Option<stats::KsTest>,
    /// Why a test was not run, if one was not.
    pub notes: Vec<String>,
    /// Every test at or above [`ALPHA`].
    pub pass: bool,
}

/// Compares our trials with the stock ones at significance `alpha`.
pub fn compare(stock: &[Trial], ours: &[Trial], alpha: f64) -> Comparison {
    let (cs, co) = (counts(stock), counts(ours));
    let (ns, no) = (stock.len() as u32, ours.len() as u32);
    let rate = |x: u32, n: u32| {
        if n == 0 {
            0.0
        } else {
            f64::from(x) / f64::from(n)
        }
    };
    let mut notes = Vec::new();
    let mut pass = true;
    let rate_tests = [0, 1].map(|i| {
        let t = stats::two_proportion(cs[i], ns, co[i], no);
        match t {
            Some(t) if t.p < alpha => pass = false,
            Some(_) => {}
            None => {
                pass = false;
                notes.push("a sample is empty".into());
            }
        }
        t
    });
    let (ts, to) = (times_to_kill(stock), times_to_kill(ours));
    let ks = match (ts.is_empty(), to.is_empty()) {
        (true, true) => {
            notes.push("times to kill not compared: no decided trial on either side".into());
            None
        }
        (true, false) | (false, true) => {
            pass = false;
            notes.push("times to kill: only one side has decided trials".into());
            None
        }
        (false, false) => {
            let k = stats::ks_two_sample(&ts, &to);
            if k.is_some_and(|k| k.p < alpha) {
                pass = false;
            }
            k
        }
    };
    Comparison {
        stock_rates: [rate(cs[0], ns), rate(cs[1], ns)],
        our_rates: [rate(co[0], no), rate(co[1], no)],
        rate_tests,
        ks,
        notes,
        pass,
    }
}

impl Comparison {
    /// One report line per test.
    pub fn report(&self, alpha: f64) -> String {
        let mut s = String::new();
        for (i, side) in ["A", "B"].iter().enumerate() {
            let p = self.rate_tests[i].map_or("n/a".to_owned(), |t| format!("{:.4}", t.p));
            let _ = writeln!(
                s,
                "  win rate {side}: stock {:.3}, ours {:.3}, two-proportion p = {p}",
                self.stock_rates[i], self.our_rates[i]
            );
        }
        if let Some(k) = self.ks {
            let _ = writeln!(s, "  time to kill: KS D = {:.3}, p = {:.4}", k.d, k.p);
        }
        for n in &self.notes {
            let _ = writeln!(s, "  note: {n}");
        }
        let _ = writeln!(
            s,
            "  {} (α = {alpha})",
            if self.pass { "pass" } else { "FAIL" }
        );
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_parse() {
        assert_eq!(
            parse_group("CART*2, WALKER").unwrap(),
            vec![("CART".into(), 2), ("WALKER".into(), 1)]
        );
        assert_eq!(parse_group("CART*3 WALKER*1").unwrap().len(), 2);
        assert!(parse_group("").is_err());
        assert!(parse_group("CART*0").is_err());
        assert!(parse_group("CART*x").is_err());
    }

    #[test]
    fn trial_lines_round_trip() {
        let t = parse_trial("A 812 3 0 640 0").unwrap();
        assert_eq!(t.outcome, Outcome::A);
        assert_eq!(
            (t.ticks, t.survivors, t.health),
            (812, [3, 0], Some([640, 0]))
        );
        assert_eq!(t.line(), "A 812 3 0 640 0");
        let d = parse_trial("draw 400 0 0").unwrap();
        assert_eq!((d.outcome, d.health), (Outcome::Draw, None));
        assert!(parse_trial("C 1 0 0").is_err());
        assert!(parse_trial("A 1 0").is_err());
        assert!(parse_trial("A -1 0 0").is_err());
    }

    /// An invented record: synthetic type IDs, made-up numbers.
    const RECORD: &str = "---\nid: M-test\ntopic: combat\nvalue-visibility: committed\n---\n\n# Arena\n\n```arena\ngroup-a: CART*2   # two carts\ngroup-b: WALKER*3\nmap-size: 24\nstart-distance: 8\ntick-cap: 3000\ngame-speed: 4\nn: 3\nwins-a: 2\ntrial: A 500 1 0 300 0\ntrial: A 640 2 0 700 0\ntrial: timeout 3000 1 1\n```\n";

    #[test]
    fn a_committed_record_parses() {
        let r = parse_record(RECORD, None, true).unwrap();
        assert_eq!(r.len(), 1);
        let r = &r[0];
        assert_eq!(r.id, "M-test");
        assert_eq!(r.scenario.groups[0], vec![("CART".into(), 2)]);
        assert_eq!(r.scenario.groups[1], vec![("WALKER".into(), 3)]);
        assert_eq!(
            (
                r.scenario.map_size,
                r.scenario.start_distance,
                r.scenario.tick_cap
            ),
            (24, 8, 3000)
        );
        assert_eq!(r.scenario.game_speed, Some(4));
        assert_eq!(counts(&r.trials), [2, 0, 0, 1]);
        assert_eq!(times_to_kill(&r.trials), vec![500, 640]);
    }

    #[test]
    fn record_checks_catch_mistakes() {
        let wrong_n = RECORD.replace("n: 3", "n: 4");
        assert!(parse_record(&wrong_n, None, true).is_err());
        let wrong_wins = RECORD.replace("wins-a: 2", "wins-a: 1");
        assert!(parse_record(&wrong_wins, None, true).is_err());
        let typo = RECORD.replace("tick-cap", "tick-cpa");
        assert!(parse_record(&typo, None, true).is_err());
        let open = RECORD.trim_end().trim_end_matches("```");
        assert!(parse_record(open, None, true).is_err());
        // Scenario only: the outcome keys are not needed.
        let bare = "---\nid: M-test\n---\n```arena\ngroup-a: CART\ngroup-b: WALKER\n```\n";
        assert!(parse_record(bare, None, true).is_err());
        assert_eq!(parse_record(bare, None, false).unwrap().len(), 1);
    }

    #[test]
    fn a_local_record_takes_its_outcomes_from_the_local_file() {
        let text = "---\nid: M-test\nvalue-visibility: local\n---\n```arena\ngroup-a: CART*2\ngroup-b: WALKER*3\n```\n\n```arena\ngroup-a: WALKER\ngroup-b: WALKER\n```\n";
        let json = r#"{ "arena": [
            { "n": 2, "wins-b": 1, "trial": ["A 10 1 0", "B 12 0 2"] },
            { "n": 1, "trial": ["draw 7 0 0 0 0"] },
        ] }"#;
        let r = parse_record(text, Some(json), true).unwrap();
        assert_eq!(r.len(), 2);
        assert_eq!(counts(&r[0].trials), [1, 1, 0, 0]);
        assert_eq!(r[1].trials[0].outcome, Outcome::Draw);
        assert_eq!(r[1].block, 1);
        // Missing local file, or a key in both places, is an error.
        assert!(parse_record(text, None, true).is_err());
        let both = text.replacen("group-b: WALKER*3", "group-b: WALKER*3\nn: 2", 1);
        assert!(parse_record(&both, Some(json), true).is_err());
        // A scenario key in the local file is refused.
        let bad = r#"{ "arena": [ { "group-a": "CART", "n": 0 }, { "n": 0 } ] }"#;
        assert!(parse_record(text, Some(bad), true).is_err());
    }

    #[test]
    fn comparison_passes_equal_samples_and_fails_different_ones() {
        let t = |o: Outcome, ticks: i64| Trial {
            outcome: o,
            ticks,
            survivors: [0, 0],
            health: None,
        };
        let same: Vec<Trial> = (0..32)
            .map(|i| t(if i % 2 == 0 { Outcome::A } else { Outcome::B }, 500 + i))
            .collect();
        assert!(compare(&same, &same, ALPHA).pass);
        let all_a: Vec<Trial> = (0..32).map(|i| t(Outcome::A, 500 + i)).collect();
        let all_b: Vec<Trial> = (0..32).map(|i| t(Outcome::B, 500 + i)).collect();
        let c = compare(&all_a, &all_b, ALPHA);
        assert!(!c.pass);
        // Same winners, much slower kills: the KS test fails.
        let slow: Vec<Trial> = (0..32).map(|i| t(Outcome::A, 5000 + i)).collect();
        let k = compare(&all_a, &slow, ALPHA);
        assert!(k.rate_tests[0].unwrap().p >= ALPHA);
        assert!(k.ks.unwrap().p < ALPHA && !k.pass);
        // Only timeouts on both sides: the times are not compared.
        let to: Vec<Trial> = (0..4).map(|_| t(Outcome::Timeout, 100)).collect();
        let c = compare(&to, &to, ALPHA);
        assert!(c.ks.is_none() && c.pass);
    }
}
