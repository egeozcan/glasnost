//! The unit arena of the oracle records (M7b step 4c part 2; PLAN.md §5
//! M7 exit and §6 layer 2B): the stock oracle's arena games, played again
//! in our engine with the records' setup, and compared with the records'
//! per-game data.
//!
//! # The records
//!
//! An *oracle arena record* is a measurement record
//! `measurements/<topic>/M-xxxx-*.md` with a CSV file of the same stem
//! beside it whose header is exactly [`CSV_HEADER`]: one line per game,
//! `row,col,row_side,seed,winner,end_frame` (the row and column type IDs,
//! the side the row type played (`A` or `B`), the seed, the winner `row`,
//! `col` or `none`, and the sample frame at which the game was over). The
//! record's Setup names the map as a back-quoted file name followed by
//! `(sha256 `…`)`; the rest of the setup is the arena protocol below, which
//! the records M-0001 to M-0003 share. Since their revision of 2026-10-29
//! the records give no army shares (measure: M-0001, M-0002, M-0003,
//! Procedure step 3 and *Tolerance for tests*).
//!
//! # The protocol (measure: M-0001, M-0002, M-0003, Setup and Procedure)
//!
//! - **Seeds** (S-0216 r69 claim 8, *Seeds and the human player house's
//!   country*): the game with the records' seed number *s* seeds the
//!   scenario stream with 1000 + *s* ([`game_seed`]); every output and
//!   argument keeps the seed number.
//! - **Map**: the record's stock map, read from the install
//!   (`GLASNOST_GAME_DIR`) and checked against the record's sha256; never
//!   committed. Its rules layer applies (the inputs of a skirmish on that
//!   map), and its own objects are loaded as in any game on it, the
//!   neutral house's buildings and infantry in play from tick 0, its
//!   vehicles taken off the map before the groups (below). Which of them are the
//!   records' "four bunkers, owned by a neutral house" is Q-2098: none has
//!   the tank-bunker flag of S-0606 (`--groups` lists them). W is the cell
//!   of start waypoint [`W_WAYPOINT`].
//! - **Houses** (S-0216 r67 claim 8, *Game start*): in S-0009 claim 19's
//!   order, a human player house at start [`HUMAN_START`] (M-0001's idle
//!   house, a player, not an observer, of side A's country, r69), side A
//!   playing country
//!   [`COUNTRIES`]`[0]` at start 0, side B [`COUNTRIES`]`[1]` at start 1,
//!   then the neutral and special houses, each with S-0202 claim 14's two
//!   house-creation draws. Skirmish setup with 0 credits, the bases setting
//!   on and the unit-count setting 0 (each player house gets its base unit
//!   and no budget unit), short game, superweapons and crates off, game
//!   speed fastest (index 0, glossary claim G2). After the game start every
//!   vehicle the human player house does not own is taken off the map with
//!   no draw ([`take_off_before_the_groups`]); the human player house's
//!   base unit stays on its base cell, and no house treats that house as an
//!   enemy (`House::idle_test_house`). The record's houses
//!   are hard computer players: per S-0412 claim 4 a hard computer gets
//!   difficulty index 0 and the easy row, which our engine applies through
//!   the handicap (S-0412 claim 5). With `computer` set (the default)
//!   both are test-only computer-controlled player houses (glossary;
//!   S-0215 and S-0309 Terms): the original's computer-owner rules, the
//!   oracle house's skill level (the rules' maximum, S-0412 claim 3) and
//!   difficulty index 0, no teams, and no brain (our own AI takes no part).
//!   Like the original's computer players they stop the unused-house
//!   cleanup (S-0009 claim 28). See *Differences* in docs/development.md for what we cannot
//!   reproduce (the original's skirmish AI, the observer, the tech level).
//! - **Groups**: per side one type, round(9000 / cost) units (an exact half
//!   to the even number, S-0216 claim 8's Formulas), at least 1, infantry at
//!   most 60. Unit *i* of side A on cell (W.x − 6 − ⌊i/8⌋, W.y − 4 + (i mod
//!   8)) facing map east, side B on (W.x + 6 + ⌊i/8⌋, W.y − 4 + (i mod 8))
//!   facing map west. Our arena's set-up is S-0216 r59 claim 8 (design):
//!   the map's objects as any game loads them; the row type's group first,
//!   on whichever side it plays, then the other group, each by index, each
//!   unit created, tested and placed before the next; each cell that
//!   refuses a unit ([`takes`]) costs the draws of one try of the unit's
//!   type (its creation set's draws, then an unused R(0, 3) for infantry),
//!   whose objects are discarded, and the next of its eight neighbours
//!   ([`NEIGHBOURS`]) is tried with a new try; set-up stops with an error
//!   when none takes it ([`place_unit`]); the placement is a play-time
//!   placement (S-0216 claims 1 to 4: the facings, an infantry unit's
//!   standing position with its one draw R(0, 3)), and the group's mission,
//!   hunt, is then queued as a placing step's behaviour. M-0006's moves and
//!   M-0019's tick-0 findings are the acceptance check, not an input. No
//!   order is given.
//! - **Samples**: every [`SAMPLE_PERIOD`] frames (ticks from game start);
//!   the game is over at the first sample at which only one side owns
//!   anything (that side wins), or at which neither does (`none`), or at
//!   the first sample at or after [`FRAME_LIMIT`] (`none` when both own
//!   something).
//!
//! # The comparison (each record's *Tolerance for tests*, M-0017)
//!
//! Only the pairings that stand as stock are scored: those with neither
//! type one of [`UNCONFIRMED_TYPES`] (M-0017; M-0001 and M-0003, *Stock
//! rows*). The others are played and shown, with no verdict. A stock game
//! that crashed ([`crashed`]) has no result: it and our game of the same
//! side and seed are left out, so MIND against E2 runs on M-0017's
//! restated line (n = 5). Per scored pairing, pooled over both sides: the
//! row's win proportion with a two-proportion z test, failing at
//! p < [`ALPHA`]; and, where the record shows the same winner for every
//! seed on a side, whether ours shows that same winner for every seed on
//! that side. A pairing passes when both hold.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use glasnost_map::terrain::TileCatalogue;
use glasnost_map::{CellXY, Map};
use glasnost_sim::econ::TypeRef;
use glasnost_sim::econ::setup::{self, HouseValues, Player, Row, Settings};
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::world::{HouseId, ObjectId, World};

use crate::stats;

/// The CSV header of an oracle arena record (measure: M-0001, Procedure,
/// per-game data).
pub const CSV_HEADER: &str = "row,col,row_side,seed,winner,end_frame";

/// Each side's budget in credits (measure: M-0001, Setup, Groups).
pub const BUDGET: i64 = 9000;

/// The most units of an infantry group (measure: M-0001, Setup, Groups).
pub const INFANTRY_CAP: u32 = 60;

/// Frames between samples (measure: M-0001, Procedure step 2).
pub const SAMPLE_PERIOD: i32 = 304;

/// The game is over at the first sample at or after this frame (measure:
/// M-0001, Procedure step 2).
pub const FRAME_LIMIT: i32 = 6000;

/// The start waypoint whose cell is W (measure: M-0001, Setup, Map).
pub const W_WAYPOINT: u16 = 2;

/// Cells from W to the front column of each side, along x (measure:
/// M-0001, Setup, Groups: W.x − 6 and W.x + 6).
pub const FRONT: i32 = 6;

/// Rows of a group's columns: unit *i* stands in row i mod 8, starting
/// 4 cells before W.y (measure: M-0001, Setup, Groups).
pub const COLUMN: i32 = 8;

/// The first row's offset from W.y (measure: M-0001, Setup, Groups).
pub const FIRST_ROW: i32 = -4;

/// The countries of side A and side B (measure: M-0001, Setup, Players).
pub const COUNTRIES: [&str; 2] = ["Americans", "Russians"];

/// The start positions of side A and side B (measure: M-0001, Setup,
/// Players).
pub const STARTS: [u32; 2] = [0, 1];

/// The start position of the human player house (S-0216 claim 8, *Game
/// start*: M-0001's idle house at start 3; measure: M-0001, Setup, Players).
pub const HUMAN_START: u32 = 3;

/// The country of the human player house: British (S-0216 claim 8, *Seeds
/// and the human player house's country*, r70; measure: M-0001, M-0002 and
/// M-0003, Setup, Players). The country picks the house's base unit (S-0009
/// claim 45); the house-creation draws do not depend on it (S-0202 claim
/// 14). [`human_country`] looks it up in the loaded rules.
// spec: S-0216 claim 8 (Seeds and the human player house's country, r70: the human player house is British)
// measure: M-0001 (Setup, Players: an observer human of the British country)
pub const HUMAN_COUNTRY: &str = "British";

/// The human player house's country as the loaded rules name it: the
/// rules' country of the name [`HUMAN_COUNTRY`] (compared without regard to
/// case). An error, never a fall back to another country, when the rules
/// have none.
// spec: S-0216 claim 8 (Seeds and the human player house's country, r70)
pub fn human_country(rules: &SimRules) -> Result<String, String> {
    rules
        .country_names
        .iter()
        .find(|n| n.eq_ignore_ascii_case(HUMAN_COUNTRY))
        .cloned()
        .ok_or_else(|| {
            format!(
                "the rules have no country {HUMAN_COUNTRY}, the country of the records' human player house (S-0216 claim 8, M-0001 Setup)"
            )
        })
}

/// What the arena adds to a seed number to seed the scenario stream.
// spec: S-0216 claim 8 (Seeds and the human player house's country, r69, Q-2447 answered: the arena's game for seed number s seeds S-0202's generator with 1000 + s)
pub const GAME_SEED_OFFSET: u32 = 1000;

/// The seed of the scenario stream for the arena's game with seed number
/// `seed` (S-0216 claim 8, *Seeds and the human player house's country*):
/// 1000 + `seed`. Seed numbers are the records' naming (M-0001: seeds 0
/// to 5) and stay so everywhere else: the CSV's `seed` column, [`SEEDS`]
/// and the command line's seed arguments. `None` when the sum does not fit.
pub fn game_seed(seed: u32) -> Option<u32> {
    seed.checked_add(GAME_SEED_OFFSET)
}

/// The human player house's house number: first in S-0009 claim 19's house
/// order (human players before computer players; S-0216 claim 8, *Game
/// start*).
pub const HUMAN: HouseId = HouseId(0);

/// The house numbers of side A and side B: the two computer players, after
/// the human player house in S-0009 claim 19's order (S-0216 claim 8,
/// *Game start*).
pub const SIDE_HOUSES: [HouseId; 2] = [HouseId(1), HouseId(2)];

/// The facing values *v* of side A and side B for a play-time placement
/// (S-0216 claim 1: body and turret facings 256 × *v*): 256 × 64 = 16384
/// is map east and 256 × 192 = 49152 map west (glossary *facing*); side A
/// faces east, side B west (measure: M-0001, Setup, Groups).
pub const FACINGS: [u8; 2] = [64, 192];

/// The offsets of the eight neighbouring cells a unit its stated cell does
/// not take is tried on, in order (S-0216 claim 8, *Moving on*).
// spec: S-0216 claim 8 (the order (x − 1, y − 1), (x, y − 1), (x + 1, y − 1), (x − 1, y), (x + 1, y), (x − 1, y + 1), (x, y + 1), (x + 1, y + 1))
pub const NEIGHBOURS: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

/// The eight neighbouring cells of `c` in [`NEIGHBOURS`] order.
pub fn neighbours(c: CellXY) -> [CellXY; 8] {
    NEIGHBOURS.map(|(dx, dy)| CellXY::new(c.x + dx, c.y + dy))
}

/// The fastest game speed: index 0 (glossary claim G2; measure: M-0001,
/// Setup, Players: game speed fastest).
pub const FASTEST: u8 = 0;

/// Seeds per side of a pairing (measure: M-0001, Setup: seeds 0 to 5).
pub const SEEDS: std::ops::Range<u32> = 0..6;

/// The significance level of each test (the records' *Tolerance for
/// tests*: p < 0.01).
pub const ALPHA: f64 = 0.01;

/// A side of the arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Side {
    /// Side A: start 0, the lower-x group.
    A,
    /// Side B: start 1, the higher-x group.
    B,
}

impl Side {
    /// 0 for A, 1 for B.
    pub fn index(self) -> usize {
        match self {
            Self::A => 0,
            Self::B => 1,
        }
    }

    /// `A` or `B`.
    pub fn text(self) -> &'static str {
        ["A", "B"][self.index()]
    }
}

/// Who won a game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Winner {
    /// The row type's side.
    Row,
    /// The column type's side.
    Col,
    /// Neither.
    None,
}

impl Winner {
    /// The winner as the CSV writes it.
    pub fn text(self) -> &'static str {
        match self {
            Self::Row => "row",
            Self::Col => "col",
            Self::None => "none",
        }
    }
}

/// One game: a CSV line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Game {
    /// The row type ID.
    pub row: String,
    /// The column type ID.
    pub col: String,
    /// The side the row type played.
    pub row_side: Side,
    /// The seed.
    pub seed: u32,
    /// The winner.
    pub winner: Winner,
    /// The sample frame at which the game was over.
    pub end_frame: i32,
}

impl Game {
    /// The game as a CSV line (without the line end).
    pub fn line(&self) -> String {
        format!(
            "{},{},{},{},{},{}",
            self.row,
            self.col,
            self.row_side.text(),
            self.seed,
            self.winner.text(),
            self.end_frame
        )
    }
}

/// Parses a CSV of games (the header first).
pub fn parse_csv(text: &str) -> Result<Vec<Game>, String> {
    let mut lines = text.lines().enumerate();
    match lines.next() {
        Some((_, h)) if h.trim() == CSV_HEADER => {}
        _ => return Err(format!("the CSV header is not {CSV_HEADER:?}")),
    }
    let mut out = Vec::new();
    for (n, line) in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split(',').map(str::trim).collect();
        let at = |e: String| format!("CSV line {}: {e}", n + 1);
        if f.len() != 6 {
            return Err(at("expected 6 fields".into()));
        }
        let row_side = match f[2] {
            "A" => Side::A,
            "B" => Side::B,
            s => return Err(at(format!("row_side {s:?} is not A or B"))),
        };
        let winner = match f[4] {
            "row" => Winner::Row,
            "col" => Winner::Col,
            "none" => Winner::None,
            s => return Err(at(format!("winner {s:?} is not row, col or none"))),
        };
        if f[0].is_empty() || f[1].is_empty() {
            return Err(at("empty type ID".into()));
        }
        out.push(Game {
            row: f[0].to_owned(),
            col: f[1].to_owned(),
            row_side,
            seed: f[3].parse().map_err(|_| at(format!("seed {:?}", f[3])))?,
            winner,
            end_frame: f[5]
                .parse()
                .map_err(|_| at(format!("end_frame {:?}", f[5])))?,
        });
    }
    Ok(out)
}

/// Writes games as a CSV with the header.
pub fn write_csv(games: &[Game]) -> String {
    let mut s = format!("{CSV_HEADER}\n");
    for g in games {
        s.push_str(&g.line());
        s.push('\n');
    }
    s
}

/// An oracle arena record.
#[derive(Clone, Debug)]
pub struct OracleRecord {
    /// The record's ID (front matter `id`).
    pub id: String,
    /// The record's markdown file.
    pub path: PathBuf,
    /// The map's file name.
    pub map_file: String,
    /// The map file's sha256, lower-case hex.
    pub map_sha256: String,
    /// The stock games, in CSV order, with the rows of a stock replay
    /// record ([`StockReplay`], M-0016) in place of the archived ones.
    pub games: Vec<Game>,
    /// The types whose pairings a stock replay record confirms as stock
    /// for this record (M-0016: SREF and V3), so that they are scored.
    pub covered: Vec<String>,
    /// How many of [`OracleRecord::games`] a stock replay record restated.
    pub restated: usize,
}

impl OracleRecord {
    /// Whether a pairing of this record stands as stock and so is scored:
    /// neither type is one of [`UNCONFIRMED_TYPES`] (M-0017), or a stock
    /// replay record covers it (M-0016).
    pub fn scores(&self, pairing: &(String, String)) -> bool {
        // measure: M-0017 (the pairings without SREF or V3 stand as stock), M-0016 (every row of the SREF and V3 pairings stands as stock, its 48 rows in place)
        UNCONFIRMED_TYPES
            .iter()
            .all(|t| (*t != pairing.0 && *t != pairing.1) || self.covered.iter().any(|c| c == t))
    }
}

/// The front matter's `key: value` lines.
fn front_matter(text: &str) -> Vec<(String, String)> {
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some("---") {
        return Vec::new();
    }
    lines
        .take_while(|l| l.trim() != "---")
        .filter_map(|l| {
            let (k, v) = l.split_once(':')?;
            Some((k.trim().to_owned(), v.trim().to_owned()))
        })
        .collect()
}

/// The map file name and sha256 of a record's Setup: the first line
/// starting `- Map:` holds a back-quoted file name, then `sha256` and a
/// back-quoted hex digest.
fn map_of(text: &str) -> Result<(String, String), String> {
    let line = text
        .lines()
        .find(|l| l.trim_start().starts_with("- Map:"))
        .ok_or("the Setup has no `- Map:` line")?;
    let quoted: Vec<&str> = line.split('`').skip(1).step_by(2).collect();
    let file = quoted.first().ok_or("the Map line names no file")?;
    let sha = line
        .split("sha256")
        .nth(1)
        .and_then(|rest| rest.split('`').nth(1))
        .ok_or("the Map line gives no sha256")?;
    if sha.len() != 64 || !sha.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!(
            "the Map line's sha256 {sha:?} is not 64 hex digits"
        ));
    }
    Ok(((*file).to_owned(), sha.to_ascii_lowercase()))
}

/// Reads one record from its markdown text and its CSV text.
pub fn parse_record(path: &Path, md: &str, csv: &str) -> Result<OracleRecord, String> {
    let fm = front_matter(md);
    let id = fm
        .iter()
        .find(|(k, _)| k == "id")
        .map(|(_, v)| v.clone())
        .ok_or("the front matter has no id")?;
    let (map_file, map_sha256) = map_of(md).map_err(|e| format!("{id}: {e}"))?;
    let games = parse_csv(csv).map_err(|e| format!("{id}: {e}"))?;
    Ok(OracleRecord {
        id,
        path: path.to_owned(),
        map_file,
        map_sha256,
        games,
        covered: Vec::new(),
        restated: 0,
    })
}

/// The CSV beside a record file.
fn csv_beside(md: &Path) -> PathBuf {
    md.with_extension("csv")
}

/// Every oracle arena record under `dir` (recursively, outside `local/`),
/// sorted by path: a `M-*.md` file with a CSV of the same stem whose first
/// line is [`CSV_HEADER`].
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
                && std::fs::read_to_string(csv_beside(&p))
                    .is_ok_and(|t| t.lines().next().map(str::trim) == Some(CSV_HEADER))
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

/// Every `M-*.md` record under `dir` (recursively, outside `local/`) with a
/// CSV of the same stem beside it whose first line is not [`CSV_HEADER`]:
/// a per-game file that the arena would not read. A changed header must
/// not make the arena skip a record silently (M8 step A3 intake 47: the
/// records' revision of 2026-10-29 dropped two columns, and the arena found
/// no record).
pub fn unrecognised_csvs(dir: &Path) -> Vec<PathBuf> {
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
                && let Ok(t) = std::fs::read_to_string(csv_beside(&p))
                && t.lines().next().map(str::trim) != Some(CSV_HEADER)
            {
                out.push(csv_beside(&p));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, &mut out);
    out.sort();
    out
}

/// Reads every oracle arena record under `dir` (the `measurements/`
/// folder), with the rows of every stock replay record there in place
/// ([`apply_stock_replay`]).
pub fn load_records(dir: &Path) -> Result<Vec<OracleRecord>, String> {
    let mut out = Vec::new();
    for path in record_files(dir) {
        let md = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let csv = std::fs::read_to_string(csv_beside(&path))
            .map_err(|e| format!("{}: {e}", path.display()))?;
        out.push(parse_record(&path, &md, &csv)?);
    }
    if !out.is_empty() {
        for r in load_stock_replays(dir)? {
            apply_stock_replay(&mut out, &r)?;
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Stock replay records (M-0016)

/// The header of a stock replay record's table of restated rows (measure:
/// M-0016, Result, *The 48 stock rows*): one line per game whose stock
/// replay changed its recorded row, naming the game as the oracle arena
/// records' per-game data do, with its archived and its stock winner and
/// end frame.
pub const STOCK_ROWS_HEADER: &str = "| Record | Row | Column | Row side | Seed | Archived winner | Archived end frame | Stock winner | Stock end frame |";

/// The start of the header of a stock replay record's per-pairing table
/// (measure: M-0016, Result, *Pairings with changed rows, stock*): the
/// row's wins, the column's wins and `none` with the stock rows in place.
pub const STOCK_PAIRINGS_HEADER: &str =
    "| Record | Row | Column | n | Row wins | Column wins | None |";

/// The types whose games a stock replay record replays, by record: every
/// archived game of M-0001 and M-0003 with one of them on either side
/// (measure: M-0016, Setup, *Games*). Its restated rows replace the
/// archived ones and every other row of those pairings stands as stock
/// (M-0016, Result, *What replaces what*).
pub const STOCK_REPLAY_TYPES: [(&str, [&str; 2]); 1] = [("M-0016", ["SREF", "V3"])]; // measure: M-0016

/// One restated game of a stock replay record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StockRow {
    /// The oracle arena record whose row it replaces (`M-0001`).
    pub record: String,
    /// The game, with the archived winner and end frame.
    pub archived: Game,
    /// The stock winner and end frame.
    pub stock: (Winner, i32),
}

/// One line of a stock replay record's per-pairing table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StockPairing {
    /// The oracle arena record (`M-0001`).
    pub record: String,
    /// Row and column type.
    pub pairing: (String, String),
    /// Its games.
    pub n: u32,
    /// Row wins, column wins and `none`, with the stock rows in place.
    pub counts: [u32; 3],
}

/// A stock replay record (M-0016): rows that replace an oracle arena
/// record's archived rows, and the per-pairing lines they give.
#[derive(Clone, Debug)]
pub struct StockReplay {
    /// The record's ID.
    pub id: String,
    /// The types it covers ([`STOCK_REPLAY_TYPES`]).
    pub covers: Vec<String>,
    /// The restated rows, in table order.
    pub rows: Vec<StockRow>,
    /// The per-pairing lines, in table order.
    pub pairings: Vec<StockPairing>,
}

/// The cells of each line of the markdown table whose header line starts
/// with `header`, after its separator line, trimmed; `None` when there is
/// no such table.
fn table_rows(md: &str, header: &str) -> Option<Vec<Vec<String>>> {
    let mut lines = md.lines().skip_while(|l| !l.trim().starts_with(header));
    lines.next()?;
    lines.next()?; // the separator line
    Some(
        lines
            .take_while(|l| l.trim_start().starts_with('|'))
            .map(|l| {
                l.trim()
                    .trim_matches('|')
                    .split('|')
                    .map(|c| c.trim().to_owned())
                    .collect()
            })
            .collect(),
    )
}

fn winner_of(s: &str) -> Result<Winner, String> {
    match s {
        "row" => Ok(Winner::Row),
        "col" => Ok(Winner::Col),
        "none" => Ok(Winner::None),
        s => Err(format!("winner {s:?} is not row, col or none")),
    }
}

/// Reads a stock replay record from its markdown text: `None` when it has
/// no table headed [`STOCK_ROWS_HEADER`].
pub fn parse_stock_replay(md: &str) -> Result<Option<StockReplay>, String> {
    let Some(rows) = table_rows(md, STOCK_ROWS_HEADER) else {
        return Ok(None);
    };
    let id = front_matter(md)
        .into_iter()
        .find(|(k, _)| k == "id")
        .map(|(_, v)| v)
        .ok_or("a stock replay record without an id")?;
    let covers = STOCK_REPLAY_TYPES
        .iter()
        .find(|(r, _)| *r == id)
        .map(|(_, t)| t.iter().map(|x| (*x).to_owned()).collect())
        .unwrap_or_default();
    let mut out = StockReplay {
        id: id.clone(),
        covers,
        rows: Vec::new(),
        pairings: Vec::new(),
    };
    for (n, f) in rows.iter().enumerate() {
        let at = |e: String| format!("{id}: stock row {}: {e}", n + 1);
        if f.len() < 9 {
            return Err(at("expected at least 9 cells".into()));
        }
        let side = match f[3].as_str() {
            "A" => Side::A,
            "B" => Side::B,
            s => return Err(at(format!("row side {s:?} is not A or B"))),
        };
        let num = |s: &str| s.parse::<i32>().map_err(|_| at(format!("number {s:?}")));
        out.rows.push(StockRow {
            record: f[0].clone(),
            archived: Game {
                row: f[1].clone(),
                col: f[2].clone(),
                row_side: side,
                seed: f[4].parse().map_err(|_| at(format!("seed {:?}", f[4])))?,
                winner: winner_of(&f[5]).map_err(at)?,
                end_frame: num(&f[6])?,
            },
            stock: (winner_of(&f[7]).map_err(at)?, num(&f[8])?),
        });
    }
    for (n, f) in table_rows(md, STOCK_PAIRINGS_HEADER)
        .unwrap_or_default()
        .iter()
        .enumerate()
    {
        let at = |e: String| format!("{id}: pairing line {}: {e}", n + 1);
        if f.len() < 7 {
            return Err(at("expected at least 7 cells".into()));
        }
        let num = |s: &str| s.parse::<u32>().map_err(|_| at(format!("number {s:?}")));
        out.pairings.push(StockPairing {
            record: f[0].clone(),
            pairing: (f[1].clone(), f[2].clone()),
            n: num(&f[3])?,
            counts: [num(&f[4])?, num(&f[5])?, num(&f[6])?],
        });
    }
    Ok(Some(out))
}

/// The `M-*.md` files under `dir` (recursively, outside `local/`), in path
/// order.
fn record_paths(dir: &Path) -> Vec<PathBuf> {
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
            } else if name.starts_with("M-") && name.ends_with(".md") {
                out.push(p);
            }
        }
    }
    let mut paths = Vec::new();
    walk(dir, &mut paths);
    paths.sort();
    paths
}

/// Every stock replay record under `dir` (recursively, outside `local/`),
/// in path order: a `M-*.md` file with a table headed
/// [`STOCK_ROWS_HEADER`].
pub fn load_stock_replays(dir: &Path) -> Result<Vec<StockReplay>, String> {
    let paths = record_paths(dir);
    let mut out = Vec::new();
    for p in paths {
        let md = std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        if let Some(r) = parse_stock_replay(&md).map_err(|e| format!("{}: {e}", p.display()))? {
            out.push(r);
        }
    }
    Ok(out)
}

/// Puts a stock replay record's rows in place of the archived ones
/// (measure: M-0016, Result, *What replaces what*: each restated row
/// replaces the row with the same record, row, column, row side and seed,
/// its stock winner and end frame in place of the archived ones) and marks
/// the types it covers as scored in the records it names. A restated row
/// whose archived winner and end frame are not the record's row is an
/// error (M-0016, Evidence: they equal M-0001's or M-0003's per-game row).
pub fn apply_stock_replay(records: &mut [OracleRecord], r: &StockReplay) -> Result<(), String> {
    for row in &r.rows {
        let Some(rec) = records.iter_mut().find(|x| x.id == row.record) else {
            return Err(format!("{}: no oracle arena record {}", r.id, row.record));
        };
        let a = &row.archived;
        let Some(g) = rec.games.iter_mut().find(|g| key(g) == key(a)) else {
            return Err(format!(
                "{}: {} has no game {} against {}, side {:?}, seed {}",
                r.id, row.record, a.row, a.col, a.row_side, a.seed
            ));
        };
        if (g.winner, g.end_frame) != (a.winner, a.end_frame) {
            return Err(format!(
                "{}: {} {} against {}, side {:?}, seed {}: the record's row is {} {}, the archived row {} {}",
                r.id,
                row.record,
                a.row,
                a.col,
                a.row_side,
                a.seed,
                g.winner.text(),
                g.end_frame,
                a.winner.text(),
                a.end_frame
            ));
        }
        // measure: M-0016 (Result, What replaces what: the stock winner and end frame replace the archived ones)
        (g.winner, g.end_frame) = row.stock;
        rec.restated += 1;
    }
    // measure: M-0016 (Setup: every archived game of M-0001 and M-0003 with SREF or V3 on either side; Result: the other rows of those pairings stand as stock; M-0002 has no game with SREF or V3)
    let named: Vec<String> = r.rows.iter().map(|x| x.record.clone()).collect();
    for rec in records.iter_mut().filter(|x| named.contains(&x.id)) {
        for t in &r.covers {
            if !rec.covered.contains(t) {
                rec.covered.push(t.clone());
            }
        }
    }
    Ok(())
}

/// Finds a record by ID (`M-0001`) or by the start of its file name.
pub fn find_record<'a>(records: &'a [OracleRecord], name: &str) -> Option<&'a OracleRecord> {
    records.iter().find(|r| {
        r.id.eq_ignore_ascii_case(name)
            || r.path
                .file_name()
                .is_some_and(|f| f.to_string_lossy().starts_with(name))
    })
}

/// The pairings of a list of games, in order of first appearance.
pub fn pairings(games: &[Game]) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for g in games {
        let p = (g.row.clone(), g.col.clone());
        if !out.contains(&p) {
            out.push(p);
        }
    }
    out
}

/// Checks the shape of a record's games: per pairing 6 seeds (0 to 5) on
/// each side, each once.
pub fn check_shape(games: &[Game]) -> Result<(), String> {
    for (row, col) in pairings(games) {
        for side in [Side::A, Side::B] {
            let mut seeds: Vec<u32> = games
                .iter()
                .filter(|g| g.row == row && g.col == col && g.row_side == side)
                .map(|g| g.seed)
                .collect();
            seeds.sort_unstable();
            if seeds != SEEDS.collect::<Vec<_>>() {
                return Err(format!(
                    "{row} against {col}, row on side {}: seeds {seeds:?}, expected 0 to 5 once each",
                    side.text()
                ));
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Placement records (M-0006)

/// The header of a placement record's table of measured placements
/// (measure: M-0006, Result, *Units off their stated cells*): a stated cell
/// and the cell its unit was placed on, as `x,y`.
pub const PLACED_HEADER: &str = "| Stated cell | Placed on |";

/// The header of a placement record's table of the neutral structures in
/// play at tick 0 (measure: M-0006, Result, *Map content*): a cell as
/// `x,y`.
pub const NEUTRAL_HEADER: &str = "| Cell | Relative to W | Note |";

/// The oracle arena records whose games a placement record's table covers,
/// by placement record (measure: M-0006, Result, *Units off their stated
/// cells*: the archived games of M-0001 to M-0003).
pub const PLACEMENT_RECORDS: [(&str, [&str; 3]); 1] = [("M-0006", ["M-0001", "M-0002", "M-0003"])]; // measure: M-0006

/// A placement record (M-0006): its W, the measured placements and the
/// cells of the neutral structures, all absolute cells of the record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placements {
    /// The record's ID.
    pub id: String,
    /// The oracle arena records it covers ([`PLACEMENT_RECORDS`]).
    pub covers: Vec<String>,
    /// The record's W.
    pub w: CellXY,
    /// Stated cell and placed cell, in table order.
    pub moves: Vec<(CellXY, CellXY)>,
    /// The cells of the neutral structures, in table order.
    pub neutral: Vec<CellXY>,
}

fn cell_text(s: &str) -> Option<CellXY> {
    let (x, y) = s.trim().split_once(',')?;
    Some(CellXY::new(x.trim().parse().ok()?, y.trim().parse().ok()?))
}

/// Reads a placement record from its markdown text: `None` when it has no
/// table headed [`PLACED_HEADER`]. Its W is the `W = (x, y)` of its text.
pub fn parse_placements(md: &str) -> Result<Option<Placements>, String> {
    let Some(rows) = table_rows(md, PLACED_HEADER) else {
        return Ok(None);
    };
    let id = front_matter(md)
        .into_iter()
        .find(|(k, _)| k == "id")
        .map(|(_, v)| v)
        .ok_or("a placement record without an id")?;
    let w = md
        .split_once("W = (")
        .and_then(|(_, rest)| rest.split_once(')'))
        .and_then(|(c, _)| cell_text(c))
        .ok_or_else(|| format!("{id}: no W = (x, y) in its text"))?;
    let covers = PLACEMENT_RECORDS
        .iter()
        .find(|(r, _)| *r == id)
        .map(|(_, t)| t.iter().map(|x| (*x).to_owned()).collect())
        .unwrap_or_default();
    let mut moves = Vec::new();
    for (n, f) in rows.iter().enumerate() {
        let c = |i: usize| {
            f.get(i)
                .and_then(|s| cell_text(s))
                .ok_or_else(|| format!("{id}: placement line {}: cell {:?}", n + 1, f.get(i)))
        };
        moves.push((c(0)?, c(1)?));
    }
    let mut neutral = Vec::new();
    for (n, f) in table_rows(md, NEUTRAL_HEADER)
        .unwrap_or_default()
        .iter()
        .enumerate()
    {
        neutral.push(
            f.first()
                .and_then(|s| cell_text(s))
                .ok_or_else(|| format!("{id}: neutral line {}: cell {:?}", n + 1, f.first()))?,
        );
    }
    Ok(Some(Placements {
        id,
        covers,
        w,
        moves,
        neutral,
    }))
}

/// Every placement record under `dir` (recursively, outside `local/`), in
/// path order.
pub fn load_placements(dir: &Path) -> Result<Vec<Placements>, String> {
    let mut out = Vec::new();
    for p in record_paths(dir) {
        let md = std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        if let Some(r) = parse_placements(&md).map_err(|e| format!("{}: {e}", p.display()))? {
            out.push(r);
        }
    }
    Ok(out)
}

/// The measured placements for the games of oracle arena record `record`,
/// relative to W: the acceptance check of our set-up (S-0216 claim 8,
/// *Acceptance*), never an input to it.
pub fn placed_for(placements: &[Placements], record: &str) -> Vec<(CellXY, CellXY)> {
    let rel = |w: CellXY, c: CellXY| CellXY::new(c.x - w.x, c.y - w.y);
    placements
        .iter()
        .filter(|p| p.covers.iter().any(|c| c == record))
        .flat_map(|p| {
            p.moves
                .iter()
                .map(move |&(a, b)| (rel(p.w, a), rel(p.w, b)))
        })
        .collect()
}

// ---------------------------------------------------------------------------
// The ground and one game

/// What every game of a record shares: the rules, the map, the tiles and
/// W.
#[derive(Clone)]
pub struct Ground {
    /// The resolved rules (with the map's rules layer).
    pub rules: Arc<SimRules>,
    /// The map.
    pub map: Map,
    /// The tile catalogue.
    pub tiles: Arc<TileCatalogue>,
    /// The cell W.
    pub w: CellXY,
    /// The countries of side A and side B.
    pub countries: [String; 2],
    /// Whether the two houses are computer-controlled (not human-controlled),
    /// as the record's houses are; `false` plays them as AI houses of our
    /// engine, which follow the human-controlled rules (glossary *AI house*).
    pub computer: bool,
}

/// The sha256 of `bytes`, lower-case hex.
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest;
    sha2::Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Loads the ground of a record from the install in `dir`: the top-level
/// file named `map_file` (letter case ignored) must have the record's
/// sha256; the map inside it is decoded with the install's rules and the
/// map's rules layer.
pub fn stock_ground(dir: &Path, map_file: &str, sha256: &str) -> Result<Ground, String> {
    stock_inputs(dir, map_file, sha256).map(|(g, _, _)| g)
}

/// [`stock_ground`], with the resolved ruleset it was built from and that
/// ruleset's hash, for a record that adds test types of its own to the
/// install's rules (M-0021).
pub fn stock_inputs(
    dir: &Path,
    map_file: &str,
    sha256: &str,
) -> Result<(Ground, glasnost_rules::schema::Ruleset, String), String> {
    let entry = std::fs::read_dir(dir)
        .map_err(|e| format!("could not read the install ({:?})", e.kind()))?
        .flatten()
        .find(|e| {
            e.file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case(map_file)
        })
        .ok_or_else(|| format!("the install has no {map_file}"))?;
    let bytes = std::fs::read(entry.path()).map_err(|e| format!("{map_file}: {:?}", e.kind()))?;
    let got = sha256_hex(&bytes);
    if got != sha256 {
        return Err(format!(
            "{map_file}: sha256 {got} is not the record's {sha256}"
        ));
    }
    pollster::block_on(async {
        let mut vfs = crate::local_lane::mount(dir)
            .await
            .map_err(|e| format!("could not mount the install ({:?})", e.kind()))?;
        let listings = glasnost_convert::find::find_maps(&vfs).await;
        let listing = listings
            .iter()
            .find(|l| l.archive.eq_ignore_ascii_case(map_file))
            .ok_or_else(|| format!("no map found inside {map_file}"))?;
        let map_bytes = vfs
            .read(&listing.file)
            .await
            .map_err(|e| format!("{map_file}: {e:?}"))?;
        let inputs = glasnost_convert::game::load(
            &mut vfs,
            map_bytes,
            map_file.to_owned(),
            glasnost_rules::layers::Layers::REFERENCE,
        )
        .await;
        let resolved = inputs
            .resolved
            .map_err(|e| format!("the rules did not resolve: {e}"))?;
        let tiles = inputs
            .tiles
            .ok_or("the theater's tile data was not found")?;
        let map = inputs.map;
        let &(x, y) = map
            .waypoints
            .get(&W_WAYPOINT)
            .ok_or_else(|| format!("{map_file} has no waypoint {W_WAYPOINT}"))?;
        let ground = Ground {
            rules: Arc::new(SimRules::new(&resolved.ruleset, &resolved.sim_rules_hash)),
            map,
            tiles: Arc::new(tiles),
            w: CellXY::new(i32::from(x), i32::from(y)),
            countries: COUNTRIES.map(str::to_owned),
            computer: true,
        };
        Ok((ground, resolved.ruleset, resolved.sim_rules_hash))
    })
}

/// The synthetic base-unit type of [`synthetic_ruleset`], an unarmed
/// vehicle of our own.
pub const SYNTHETIC_BASE_UNIT: &str = "BASE";

/// The synthetic combat ruleset with what our arena's game start reads
/// (S-0216 claim 8, *Game start*), all invented: the neutral and special
/// houses' countries, passive (so they take no start position and no
/// starting units, S-0009 claims 19 and 45), and an unarmed vehicle type
/// [`SYNTHETIC_BASE_UNIT`] of both player countries as the only base-unit
/// type (S-0009 claim 45).
pub fn synthetic_ruleset() -> glasnost_rules::schema::Ruleset {
    let mut r = glasnost_sim::testkit::combat_ruleset();
    // The human player house's country, invented, a player country like the first two.
    r.registries
        .entry("countries".into())
        .or_default()
        .push(HUMAN_COUNTRY.into());
    r.sim.countries.insert(
        HUMAN_COUNTRY.into(),
        glasnost_rules::sim::Country::default(),
    );
    let players: Vec<String> = r.registries["countries"].clone();
    for name in [glasnost_sim::load::NEUTRAL, glasnost_sim::load::SPECIAL] {
        r.registries
            .entry("countries".into())
            .or_default()
            .push(name.into());
        r.sim.countries.insert(
            name.into(),
            glasnost_rules::sim::Country {
                multiplay_passive: Some(true),
                multiplay_passive_base: Some(true),
                ..Default::default()
            },
        );
    }
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push(SYNTHETIC_BASE_UNIT.into());
    r.sim.vehicles.insert(
        SYNTHETIC_BASE_UNIT.into(),
        glasnost_rules::sim::UnitType {
            strength: Some(1000),
            speed: Some(4),
            rot: Some(4),
            locomotion_class: Some(glasnost_rules::sim::LocomotionClass::Track),
            locomotor: Some(glasnost_rules::sim::Locomotor::Drive),
            owner: Some(players),
            ..Default::default()
        },
    );
    r.sim.general.base_unit = Some(vec![SYNTHETIC_BASE_UNIT.into()]);
    r
}

/// A synthetic ground for tests and CI: [`synthetic_ruleset`] on our own
/// flat map of `size` cells, with start waypoints 0 and 1, W in the middle
/// and start waypoint [`HUMAN_START`] off to one side, the countries the
/// synthetic rules' first two.
pub fn synthetic_ground(size: i32) -> Ground {
    let rules = Arc::new(SimRules::new(&synthetic_ruleset(), "synthetic-arena"));
    let mut map = glasnost_sim::testkit::empty_map(size);
    let w = glasnost_sim::testkit::mid(size, 0, 0);
    let wp = |c: CellXY| (c.x as i16, c.y as i16);
    map.waypoints
        .insert(0, wp(CellXY::new(w.x - size / 2, w.y)));
    map.waypoints
        .insert(1, wp(CellXY::new(w.x + size / 2, w.y)));
    map.waypoints.insert(W_WAYPOINT, wp(w));
    map.waypoints
        .insert(HUMAN_START as u16, wp(CellXY::new(w.x, w.y - size / 2)));
    let countries = [0, 1].map(|i| rules.country_names[i].clone());
    Ground {
        rules,
        map,
        tiles: glasnost_sim::testkit::catalogue(),
        w,
        countries,
        computer: true,
    }
}

/// The kind and group size of type `name` for the records' budget
/// [`BUDGET`] ([`count`]).
pub fn group(rules: &SimRules, name: &str) -> Result<(Kind, u32, i64), String> {
    let (kind, index) = [Kind::Infantry, Kind::Vehicle, Kind::Aircraft]
        .into_iter()
        .find_map(|k| rules.index_of(k, name).map(|i| (k, i)))
        .ok_or_else(|| format!("{name}: no infantry, vehicle or aircraft type of that ID"))?;
    let cost = i64::from(
        rules
            .econ
            .production
            .of(TypeRef { kind, index })
            .map_or(0, |t| t.cost),
    );
    Ok((kind, count(kind, BUDGET, cost) as u32, cost))
}

/// A group's count for `budget` and its type's `cost` (S-0216 claim 8,
/// *Group counts*): budget / cost rounded to the nearest integer, an exact
/// half to the even one ([`round_half_even`]), then held to the records'
/// minimum 1 and, for infantry, their cap [`INFANTRY_CAP`] (measure:
/// M-0001, Setup, Groups). The rule is the same for infantry and vehicle
/// groups. A cost of 0 or less gives the minimum (our guard: the formula
/// takes a positive cost).
pub fn count(kind: Kind, budget: i64, cost: i64) -> i64 {
    // spec: S-0216 claim 8 (the count rounded, then the records' minimum and infantry cap)
    let mut n = if cost > 0 {
        round_half_even(budget, cost)
    } else {
        1
    };
    n = n.max(1); // measure: M-0001 (Setup, Groups: at least 1)
    if kind == Kind::Infantry {
        n = n.min(i64::from(INFANTRY_CAP));
    }
    n
}

/// a / b rounded to the nearest whole number, an exact half to the even
/// one (a ≥ 0, b > 0): S-0216 claim 8's Formulas, *q* = ⌊*B* / *c*⌋,
/// *r* = *B* − *q* × *c*; *q* when 2*r* < *c*, *q* + 1 when 2*r* > *c*, the
/// even one of the two when 2*r* = *c*, in exact integer arithmetic. Our
/// arena's own tie rule; what the records' harness gave is open (Q-2099
/// question 1).
// spec: S-0216 claim 8 (Formulas: the arena group count before the minimum and cap)
pub fn round_half_even(a: i64, b: i64) -> i64 {
    let (q, r) = (a / b, a % b);
    match (2 * r).cmp(&b) {
        std::cmp::Ordering::Greater => q + 1,
        std::cmp::Ordering::Equal => q + (q & 1),
        std::cmp::Ordering::Less => q,
    }
}

/// The cell of unit `i` of `side` (measure: M-0001, Setup, Groups).
pub fn cell_of(w: CellXY, side: Side, i: u32) -> CellXY {
    let (back, row) = ((i / COLUMN as u32) as i32, (i % COLUMN as u32) as i32);
    let x = match side {
        Side::A => w.x - FRONT - back,
        Side::B => w.x + FRONT + back,
    };
    CellXY::new(x, w.y + FIRST_ROW + row)
}

/// A game set up and ready to run.
pub struct Arena {
    /// The world.
    pub world: World,
    /// The houses of side A and side B.
    pub houses: [HouseId; 2],
    /// Each side's units, in creation order.
    pub units: [Vec<ObjectId>; 2],
    /// The units that stand off their stated cell ([`placed_cell`]), in
    /// creation order.
    pub moved: Vec<Moved>,
    /// The raw values the scenario stream had taken since its seeding when
    /// the set-up's first object was created: the place of the first
    /// creation draw (S-0216 claim 8, *Acceptance*: M-0019's *N*₀).
    pub first_raw: u64,
    /// The vehicles taken off the map before the groups (S-0216 claim 8,
    /// *Removal before the groups*), in creation order. None of them is
    /// in `world`.
    pub removed: Vec<ObjectId>,
    /// Every unit of the two groups with its tries ([`place_unit_logged`]),
    /// in creation order (the tool's log, read only).
    pub placements: Vec<Placement>,
}

/// The units off their stated cells, one line each (shown by `--watch` and
/// `--groups`).
pub fn moved_text(moved: &[Moved]) -> String {
    let mut out = String::new();
    for m in moved {
        let _ = writeln!(
            out,
            "  side {:?} unit {}: stated cell ({}, {}) does not take it ({}); placed on ({}, {})",
            m.side, m.index, m.stated.x, m.stated.y, m.why, m.placed.x, m.placed.y
        );
    }
    out
}

/// What stands on the map at tick 0 that neither side owns, by owner
/// house, kind and type ID, with counts, and the tank bunkers and
/// garrisonable buildings among them (shown by `--groups`: where the
/// records' four neutral bunkers of M-0001 and M-0017 Setup are to be
/// found, Q-2098).
pub fn others_text(a: &Arena) -> String {
    let w = &a.world;
    let names: BTreeMap<(Kind, u32), &str> = w
        .rules()
        .names
        .iter()
        .map(|((k, name), &i)| ((*k, i), name.as_str()))
        .collect();
    let mut seen: BTreeMap<(String, String, String), u32> = BTreeMap::new();
    for o in w.objects.values() {
        if !o.on_map || o.owner.is_some_and(|h| a.houses.contains(&h)) {
            continue;
        }
        let owner = o.owner.map_or("none".to_owned(), |h| {
            w.houses
                .get(usize::from(h.0))
                .map_or(format!("house {}", h.0), |x| {
                    format!("house {} ({})", h.0, x.name)
                })
        });
        let name = names
            .get(&(o.kind, o.type_index))
            .map_or(format!("#{}", o.type_index), |n| (*n).to_owned());
        *seen
            .entry((owner, format!("{:?}", o.kind), name))
            .or_default() += 1;
    }
    let mut out = String::new();
    for ((owner, kind, name), n) in seen {
        let _ = writeln!(out, "  {owner}: {kind} {name} x{n}");
    }
    // The tank bunkers among them, by the bunker flag (S-0606 Terms).
    let bunkers: Vec<String> = w
        .objects
        .values()
        .filter(|o| {
            o.on_map
                && !o.owner.is_some_and(|h| a.houses.contains(&h))
                && glasnost_sim::structures::bunker::is_bunker(w, o.id)
        })
        .map(|o| {
            format!(
                "owner {:?} at ({}, {}) health {}",
                o.owner.map(|h| h.0),
                o.pos.cell().x,
                o.pos.cell().y,
                o.health
            )
        })
        .collect();
    let _ = writeln!(
        out,
        "  tank bunkers (S-0606 Terms) among them: {}: {}",
        bunkers.len(),
        bunkers.join("; ")
    );
    // The garrisonable buildings among them (S-0508 Terms).
    let garrisons = w
        .objects
        .values()
        .filter(|o| {
            o.on_map
                && !o.owner.is_some_and(|h| a.houses.contains(&h))
                && glasnost_sim::structures::garrison::garrisonable(w, o.id)
        })
        .count();
    let _ = writeln!(
        out,
        "  garrisonable buildings (S-0508 Terms) among them: {garrisons}"
    );
    out
}

/// A unit that its stated cell did not take, and the cell it stands on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Moved {
    /// Its side.
    pub side: Side,
    /// Its index *i* in its group.
    pub index: u32,
    /// Its stated cell ([`cell_of`]).
    pub stated: CellXY,
    /// The cell it stands on.
    pub placed: CellXY,
    /// Why its stated cell did not take it ([`refusal`]).
    pub why: String,
}

/// Whether `cell` takes the created unit `id` of `kind`, not yet on the map
/// (S-0216 claim 8, *The take test*, design): the cell is inside the map's
/// usable area by S-0009 claim 4's height-aware test, the unit's entry
/// outcome there is *clear*, asked as S-0216 claim 5 asks it, and, for a
/// vehicle, no unit stands on the cell; for an infantry unit, the cell's
/// ground record has neither of S-0216 claim 2's first-row refusals and at
/// least one of its right, left and bottom standing positions is free. The
/// test takes no draw. An aircraft-kind group is not covered (S-0216 Open
/// questions, *Aircraft-kind groups*): no record has one, and such a group
/// is refused by [`setup`].
pub fn takes(w: &World, id: ObjectId, kind: Kind, cell: CellXY) -> bool {
    refusal(w, id, kind, cell).is_none()
}

/// Why `cell` does not take the unit ([`takes`]): `None` when it does.
pub fn refusal(w: &World, id: ObjectId, kind: Kind, cell: CellXY) -> Option<String> {
    use glasnost_sim::world::Layer;
    if !matches!(kind, Kind::Infantry | Kind::Vehicle) {
        return Some(format!("a {kind:?} group is not covered (S-0216 claim 8)"));
    }
    let Some(level) = w.cells.cell(cell).map(|c| i32::from(c.level)) else {
        return Some("not a map cell".into());
    };
    // spec: S-0216 claim 8 (the cell inside the map's usable area by S-0009 claim 4's height-aware test, as S-0009 claim 5a applies it to object placement)
    if !w.cells.usable(cell) {
        return Some("outside the usable area".into());
    }
    // spec: S-0216 claim 8 and claim 5 (the unit's entry outcome for the cell, asked without an approach direction or source elevation, against the ground occupants; only clear takes)
    let outcome = if kind == Kind::Infantry {
        glasnost_sim::occupancy::infantry_outcome(w, id, cell, level)
    } else {
        glasnost_sim::occupancy::vehicle_outcome(w, id, cell, level)
    };
    if outcome != glasnost_sim::occupancy::Outcome::Clear {
        return Some(format!("entry outcome {outcome:?}"));
    }
    if kind == Kind::Vehicle {
        // spec: S-0216 claim 8 (for a vehicle, no unit stands on the cell)
        let unit_there = w.objects.values().any(|o| {
            o.on_map
                && o.id != id
                && matches!(o.kind, Kind::Infantry | Kind::Vehicle | Kind::Aircraft)
                && o.pos.cell() == cell
        });
        return unit_there.then(|| "a unit stands there".into());
    }
    // spec: S-0216 claim 8 and claim 2 (for an infantry unit, neither the vehicle-present flag nor the object-present flag without an open gate in the cell: claim 2's first row)
    if glasnost_sim::occupancy::no_position_by_flags(w, cell, Layer::Ground) {
        return Some("a first-row refusal (vehicle-present or object-present)".into());
    }
    // spec: S-0216 claim 8 (at least one of the right, left and bottom standing positions, S-0208 table O's centre rows, is free)
    if w.cells.occ(cell, Layer::Ground).is_some_and(|o| o.full()) {
        return Some("the right, left and bottom positions are taken".into());
    }
    None
}

/// The cell the created unit `id` would stand on, given its stated cell
/// (S-0216 claim 8, *Moving on*, design), asked without any draw: the
/// stated cell when it takes the unit ([`takes`]), else the first of its
/// eight neighbouring cells, in [`NEIGHBOURS`] order, that does; `None`
/// when none does. The take test does not depend on which object of the
/// type asks it, so this is the cell [`place_unit`] reaches, though
/// [`place_unit`] asks each cell with a new try and its draws. First come,
/// first served: no cell is kept for a later unit, which is tested on its
/// stated cell in turn, so an infantry unit can still be taken by a cell
/// that holds infantry, and a vehicle is moved on.
pub fn placed_cell(w: &World, id: ObjectId, kind: Kind, stated: CellXY) -> Option<CellXY> {
    if takes(w, id, kind, stated) {
        return Some(stated);
    }
    neighbours(stated)
        .into_iter()
        .find(|&c| takes(w, id, kind, c))
}

/// The cost field of an object's type (S-0403 claim 2).
fn cost_of(w: &World, id: ObjectId) -> i64 {
    w.obj(id).map_or(0, |o| {
        i64::from(glasnost_sim::econ::production::cost_field(
            w,
            TypeRef {
                kind: o.kind,
                index: o.type_index,
            },
        ))
    })
}

/// Takes off the map every vehicle that `human` does not own (S-0216 r67
/// claim 8, *Removal before the groups*, design): the two computer houses'
/// base units and every vehicle the map gave the neutral house. Each is
/// taken out of the game with no draw, no animation and no recalculation
/// of its cell, which it frees ([`glasnost_sim::objects::take_out_quietly`]);
/// none of them exists at the next tick, and the creation numbers they took
/// stay taken. Returns the vehicles taken, in creation order.
pub fn take_off_before_the_groups(w: &mut World, human: HouseId) -> Vec<ObjectId> {
    take_off_vehicles(w, Some(human))
}

/// [`take_off_before_the_groups`] with no house kept (M-0021, Setup: the
/// observer's starting construction vehicle is off the map at tick 0 as
/// each computer house's is).
fn take_off_vehicles(w: &mut World, keep: Option<HouseId>) -> Vec<ObjectId> {
    // spec: S-0216 claim 8 (Removal before the groups: every vehicle that the human player house does not own)
    let gone: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| o.kind == Kind::Vehicle && (keep.is_none() || o.owner != keep))
        .map(|o| o.id)
        .collect();
    for &id in &gone {
        // spec: S-0216 claim 8 (Removal before the groups: no random draw, no animation, the cells freed; the creation numbers stay taken)
        glasnost_sim::objects::take_out_quietly(w, id);
    }
    gone
}

/// Sets up the game of `types` (side A's and side B's type) with seed
/// number `seed`, whose scenario stream is seeded with [`game_seed`], the
/// group of `row_side` (the side the pairing's row type plays) created
/// and placed first (S-0216 claim 8, *Order*). A game of a type against
/// itself has no row side of its own: its callers pass [`Side::A`]. The
/// game start before the groups is claim 8's *Game start* and *Removal
/// before the groups* (r67): the houses [`HUMAN`], then side A and side B
/// ([`SIDE_HOUSES`]), then the neutral and special houses.
pub fn setup(
    ground: &Ground,
    types: [&str; 2],
    row_side: Side,
    seed: u32,
) -> Result<Arena, String> {
    setup_with(ground, types, row_side, seed, &SetupOptions::default())
}

/// What a record's set-up changes in [`setup`]'s (M-0021, Setup).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetupOptions {
    /// Each group's count, in place of the budget's ([`group`]).
    pub count: Option<u32>,
    /// The house whose group stands on side A's cells, and side B's. The
    /// default is [`SIDE_HOUSES`]; swapped, side A's house's group stands
    /// on side B's cells, facing as side B's, and the houses keep their
    /// countries and start positions.
    pub houses: [HouseId; 2],
    /// Every object of the neutral house is taken off before the groups,
    /// as the vehicles of *Removal before the groups* are.
    pub clear_neutral: bool,
    /// The human player house is an observer (M-0021, Setup, Houses): its
    /// starting construction vehicle is taken off with the others, and
    /// after both groups it owns one building (this type, if the rules
    /// have it) on its start position's cell, nothing else.
    pub observer_building: Option<String>,
}

impl Default for SetupOptions {
    fn default() -> Self {
        Self {
            count: None,
            houses: SIDE_HOUSES,
            clear_neutral: false,
            observer_building: None,
        }
    }
}

/// Takes off the map every object of the neutral house (M-0021, Setup:
/// every object of the map's neutral house removed), as
/// [`take_off_before_the_groups`] takes its vehicles: with no draw and no
/// recalculation of its cells. Returns the objects taken, in creation order.
pub fn take_off_neutral(w: &mut World) -> Vec<ObjectId> {
    let neutral: Vec<HouseId> = w
        .houses
        .iter()
        .enumerate()
        .filter(|(_, h)| h.role == glasnost_sim::world::HouseRole::Neutral)
        .filter_map(|(i, _)| u8::try_from(i).ok().map(HouseId))
        .collect();
    let gone: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| o.owner.is_some_and(|h| neutral.contains(&h)))
        .map(|o| o.id)
        .collect();
    for &id in &gone {
        // measure: M-0021 (Setup, What is not stock: every object of the map's neutral house removed; taken out as claim 8's vehicles are, our choice)
        glasnost_sim::objects::take_out_quietly(w, id);
    }
    gone
}

/// [`setup`] with a record's changes ([`SetupOptions`]).
pub fn setup_with(
    ground: &Ground,
    types: [&str; 2],
    row_side: Side,
    seed: u32,
    opts: &SetupOptions,
) -> Result<Arena, String> {
    let rules = &ground.rules;
    // spec: S-0216 claim 8 (Seeds and the human player house's country, r69: the game for seed number s seeds the generator with 1000 + s)
    let stream_seed = game_seed(seed).ok_or_else(|| format!("seed number {seed} is too large"))?;
    // spec: S-0216 claim 8 (Game start, r67: a human player house at start 3, M-0001's idle house; it holds a start position, so it is a player, not an observer; r70: it is British, M-0001 to M-0003 Setup)
    let human = Player {
        name: "Human".into(),
        country: human_country(rules)?,
        start: Some(HUMAN_START),
        ai: false,
        values: HouseValues::default(),
    };
    let sides: [Player; 2] = [0, 1].map(|i| Player {
        name: format!("Side {}", ["A", "B"][i]),
        country: ground.countries[i].clone(),
        start: Some(STARTS[i]),
        // An AI house with no brain: the record's houses are computer
        // players (measure: M-0001, Setup, Players), which stop the
        // unused-house cleanup (S-0009 claim 28); our own AI takes no part.
        ai: true,
        values: HouseValues {
            // spec: S-0412 claim 4 (a hard computer: index 0, the easy row), applied through claim 5's handicap
            handicap: Some(Row::Easy),
            ..Default::default()
        },
    });
    // spec: S-0216 claim 8 (Game start: the houses in S-0009 claim 19's order, the human player house, then the two computer houses; setup::skirmish adds the neutral and special houses after them)
    let players: Vec<Player> = std::iter::once(human).chain(sides).collect();
    let settings = Settings {
        // measure: M-0001 (0 credits, short game off, superweapons off, crates off)
        credits: Some(0),
        short_game: Some(false),
        superweapons: Some(false),
        // spec: S-0216 claim 8 (Game start: crates off, M-0001's setting)
        crates: Some(false),
        ..Default::default()
    };
    // spec: S-0216 claim 8 (Game start, r67: the unit-count setting 0, so no house buys a budget unit)
    let mut gs = setup::skirmish(rules, &players, settings, 0, stream_seed);
    // spec: S-0216 claim 8 (Game start, r67: the bases setting on, so each player house gets its base unit, S-0009 claims 45 and 46)
    gs.bases = true;
    gs.game_speed = FASTEST;
    let houses = opts.houses;
    if ground.computer {
        // measure: M-0001 (Setup, Players: two computer houses). Both
        // houses are test-only computer-controlled player houses (glossary;
        // S-0215 Terms): they follow the original's computer-owner rules.
        for h in houses {
            gs.houses[usize::from(h.0)].human = false;
        }
    }
    let mut w = glasnost_sim::load::new_game(rules.clone(), &ground.map, ground.tiles.clone(), &gs)
        .map_err(|e| format!("{e:?}"))?;
    // spec: S-0216 claim 8 (Game start, r67: no house treats the human player house as an enemy; no object targets or damages its objects; design, copied from M-0001's Setup)
    w.houses[usize::from(HUMAN.0)].idle_test_house = true;
    // spec: S-0216 claim 8 (Removal before the groups, r67: after resource-glint setup, the last of new_game's draws, and before the first group's first creation)
    let mut removed = if opts.observer_building.is_some() {
        // measure: M-0021 (Setup, Houses: the observer's starting construction vehicle, like each computer house's, is off the map at tick 0 and gone from tick 1 on)
        take_off_vehicles(&mut w, None)
    } else {
        take_off_before_the_groups(&mut w, HUMAN)
    };
    if opts.clear_neutral {
        removed.extend(take_off_neutral(&mut w));
    }
    if ground.computer {
        // spec: S-0215 Terms and S-0309 Terms (Test-only exception: a marked test house takes its oracle house's skill level and difficulty index, and has no teams)
        // spec: S-0412 claim 3 (a computer player house: the rules' maximum house skill level)
        let skill = rules.general.iq_max;
        for h in houses {
            let house = &mut w.houses[usize::from(h.0)];
            house.skill = skill;
            // spec: S-0412 claim 4 (a hard computer: difficulty index 0)
            house.econ.setup.difficulty = 0;
        }
    }
    // Nothing of the two houses may exist before the groups are placed.
    if w.objects
        .values()
        .any(|o| o.owner.is_some_and(|h| houses.contains(&h)))
    {
        return Err("a side owns objects before its group is placed".into());
    }
    let mut units: [Vec<ObjectId>; 2] = [Vec::new(), Vec::new()];
    let mut moved = Vec::new();
    let mut placements = Vec::new();
    let first_raw = w.rng.taken();
    // spec: S-0216 claim 8 (Order, r59: the group of the pairing's row type first, on whichever side it plays, then the other group, each in increasing unit index; each unit created, then tested and placed, before the next is created; measure: M-0019, creation order puts the row type's units first)
    let order = match row_side {
        Side::A => [Side::A, Side::B],
        Side::B => [Side::B, Side::A],
    };
    for side in order {
        let s = side.index();
        let (kind, budget_n, _) = group(rules, types[s])?;
        let n = opts.count.unwrap_or(budget_n);
        let ti = rules
            .index_of(kind, types[s])
            .ok_or_else(|| format!("{}: no type of that ID", types[s]))?;
        for i in 0..n {
            let stated = cell_of(ground.w, side, i);
            if ground.map.cells.get(stated).is_none() {
                return Err(format!(
                    "{}: cell ({}, {}) is off the map",
                    types[s], stated.x, stated.y
                ));
            }
            let (id, cell, why, tries) =
                place_unit_logged(&mut w, kind, ti, houses[s], stated, FACINGS[s])
                    .map_err(|e| format!("{}: unit {i}: {e}", types[s]))?;
            placements.push(Placement {
                side,
                index: i,
                id,
                kind,
                tries,
            });
            if let Some(why) = why {
                moved.push(Moved {
                    side,
                    index: i,
                    stated,
                    placed: cell,
                    why,
                });
            }
            units[s].push(id);
        }
    }
    // measure: M-0021 (Setup, Houses, creation order: the observer's building after both groups; its type, cell, facing and mission are kept local, so the type is the caller's choice, at the human start position's cell, facing 0, guard)
    if let Some(name) = &opts.observer_building {
        if rules.index_of(Kind::Building, name).is_none() {
            return Err(format!(
                "the observer's building type {name} does not exist"
            ));
        }
        let &(x, y) = ground
            .map
            .waypoints
            .get(&(HUMAN_START as u16))
            .ok_or("the map has no start position for the observer")?;
        glasnost_sim::testkit::spawn(
            &mut w,
            Kind::Building,
            name,
            HUMAN.0,
            CellXY::new(i32::from(x), i32::from(y)),
            0,
        );
    }
    Ok(Arena {
        world: w,
        houses,
        units,
        moved,
        first_raw,
        removed,
        placements,
    })
}

/// Creates one unit of `kind` and type index `ti` for `house` and places it
/// as S-0216 r59 claim 8 gives (design). Each cell of the order (its
/// `stated` cell, then its eight neighbours, [`NEIGHBOURS`]) is tried with
/// a new try: a unit of the type created with its whole creation set (its
/// own creation draw and creation number, then those of the objects its
/// creation makes, S-0611 claim 14), then the take test ([`refusal`]),
/// which takes no draw. A cell that refuses keeps the try's draws, adds an
/// unused draw R(0, 3) for an infantry unit, and keeps no object: the try's
/// set is discarded, its creation numbers stay reserved
/// ([`glasnost_sim::objects::discard_unplaced`]). The cell that takes the
/// unit gets a play-time placement of that try with facing value `v`
/// (S-0216 claims 1 to 4: an infantry unit's one draw R(0, 3)); then the
/// group's mission, hunt, is queued. Returns the unit, its cell, and why its
/// stated cell refused it when it was moved on. An error when no cell takes
/// it (claim 8's error stop) or the placement fails.
pub fn place_unit(
    w: &mut World,
    kind: Kind,
    ti: u32,
    house: HouseId,
    stated: CellXY,
    v: u8,
) -> Result<(ObjectId, CellXY, Option<String>), String> {
    place_unit_logged(w, kind, ti, house, stated, v).map(|(id, cell, why, _)| (id, cell, why))
}

/// One try of a unit's placement ([`place_unit`]), as our tool reads it
/// from the scenario stream's count of raw values and the world's creation
/// numbers around each step (read only; the tool's own log, M-0020
/// Procedure 4 and M-0019's model's count *c*).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Try {
    /// The cell tried.
    pub cell: CellXY,
    /// The raw values the stream had taken when the try's creation began.
    pub start: u64,
    /// The raw values the try's creation took (its creation set's draws).
    pub creation_raw: u64,
    /// The objects the creation made (creation numbers taken), the unit's
    /// own included.
    pub created: u32,
    /// Whether the cell took the unit.
    pub took: bool,
    /// The raw values taken after the creation: a refused cell's unused
    /// R(0, 3), or the placement's draws on the cell that took it.
    pub after_raw: u64,
}

/// One unit of the set-up with its tries, in creation order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placement {
    /// Its side.
    pub side: Side,
    /// Its index *i* in its group.
    pub index: u32,
    /// The unit.
    pub id: ObjectId,
    /// Its kind.
    pub kind: Kind,
    /// Its tries, the stated cell's first; the last took it.
    pub tries: Vec<Try>,
}

/// [`place_unit`] with its tries ([`Try`]) logged.
pub fn place_unit_logged(
    w: &mut World,
    kind: Kind,
    ti: u32,
    house: HouseId,
    stated: CellXY,
    v: u8,
) -> Result<(ObjectId, CellXY, Option<String>, Vec<Try>), String> {
    let mut refused: Vec<String> = Vec::new();
    let mut why = None;
    let mut tries = Vec::new();
    // spec: S-0216 claim 8 (Moving on: the stated cell, then its eight neighbours in order; if none takes the unit, set-up stops and reports an error)
    for cell in std::iter::once(stated).chain(neighbours(stated)) {
        let (start, numbers) = (w.rng.taken(), w.next_id);
        // spec: S-0216 claim 8 (Order and Draws of a refused cell, r59: each cell is tried with a new try, a unit of the unit's own type created with its creation draws, its own then those of the objects its creation makes, each with its creation number, before the take test)
        let id = glasnost_sim::objects::create(w, kind, ti, Some(house));
        let mut t = Try {
            cell,
            start,
            creation_raw: w.rng.taken() - start,
            created: w.next_id - numbers,
            took: false,
            after_raw: 0,
        };
        let after = w.rng.taken();
        let Some(r) = refusal(w, id, kind, cell) else {
            // spec: S-0216 claim 8 (Placement: a play-time placement on the cell that takes the unit, as claims 1 to 4 give it: the facings of claim 1, an infantry unit's standing position of claim 2 with its one draw R(0, 3), claim 3's draws, claim 4's fall-back)
            if !glasnost_sim::objects::place_in_play(w, id, cell, v) {
                return Err(format!(
                    "could not be placed on cell ({}, {})",
                    cell.x, cell.y
                ));
            }
            t.took = true;
            t.after_raw = w.rng.taken() - after;
            tries.push(t);
            let o = w.obj_mut(id).ok_or("not created")?;
            // spec: S-0216 claim 8 and claim 4 (the group's mission is then queued as a placing step's behaviour, S-0215 claim 7, without being started); S-0009 claim 41 (hunt, measure: M-0001)
            o.beh.queue(Some(Behaviour::Hunt));
            return Ok((id, cell, why, tries));
        };
        if kind == Kind::Infantry {
            let half = w.rules().quirks.ranged_draw_half_span;
            // spec: S-0216 claim 8 (Draws of a refused cell, r59: for an infantry unit, one draw R(0, 3) after the try's creation draws, taken whether or not claim 2's first row would refuse the cell, its result not used)
            let _ = w.rng.ranged(0, 3, half);
        }
        t.after_raw = w.rng.taken() - after;
        tries.push(t);
        // spec: S-0216 claim 8 (Draws of a refused cell, r59: the refused try never enters play and our engine keeps no object of it, a documented departure with no switch; its creation numbers stay reserved, as S-0611 claim 11's never-placed units)
        glasnost_sim::objects::discard_unplaced(w, id);
        refused.push(format!("({}, {}) {r}", cell.x, cell.y));
        why.get_or_insert(r);
    }
    Err(format!(
        "neither ({}, {}) nor a neighbouring cell takes it: {}",
        stated.x,
        stated.y,
        refused.join("; ")
    ))
}

/// What a house owns: the number of its objects with health above 0 (any
/// kind, on or off the map), and their summed cost fields (shown by
/// `--watch`). A rocket missile in flight (S-0506) is not counted: it is a
/// weapon's shot rather than a unit of the army (our reading of M-0001's
/// "owns anything").
pub fn owned(w: &World, h: HouseId) -> (usize, i64) {
    let ids: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| {
            o.owner == Some(h) && o.health > 0 && o.kind != Kind::Terrain && o.rocket.is_none()
        })
        .map(|o| o.id)
        .collect();
    let value = ids.iter().map(|&id| cost_of(w, id)).sum();
    (ids.len(), value)
}

/// Plays the game of `row` against `col` with the row type on `row_side`
/// and `seed`. With `watch`, also describes it: each sample's owned counts
/// and values, the shots of each side, and every unit's state at the end.
pub fn play(
    ground: &Ground,
    row: &str,
    col: &str,
    row_side: Side,
    seed: u32,
    watch: bool,
) -> Result<(Game, String), String> {
    play_checked(ground, row, col, row_side, seed, watch).map(|(g, s, _)| (g, s))
}

/// What one game shows of the houses other than the two players' (M-0019
/// Procedure 8, revised 2026-11-05), read from our state only. *Tick 0* is
/// the state after tick 0's update (measure: M-0019, Setup, *Tick 0*: after
/// tick 0's update), which is ours after the game's first step, the tick
/// numbered 0 (spec: S-0201 claim 1, the tick number 0 at game start, grown
/// by 1 at the end of every tick); *the next tick* is ours after the second.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OtherHouses {
    /// At tick 0 and at the next tick: the houses, other than the two
    /// players' and the observer's, that own an object, with their role.
    pub owners: [Vec<(u8, glasnost_sim::world::HouseRole)>; 2],
    /// The neutral house's vehicles at tick 0: on the map, off the map.
    pub neutral_vehicles: (usize, usize),
    /// Of the neutral house's vehicles at tick 0, those whose identifier
    /// the next tick still has.
    pub neutral_vehicles_next: usize,
    /// The neutral house's infantry and buildings at tick 0: on the map,
    /// off the map.
    pub neutral_rest: (usize, usize),
    /// The owners of the tank bunkers (S-0606 Terms) at tick 0.
    pub bunker_owners: Vec<Option<u8>>,
    /// The observer house's objects at tick 0 and at the next tick: kind
    /// and whether on the map.
    pub observer: [Vec<(Kind, bool)>; 2],
    /// The samples (every [`SAMPLE_PERIOD`] ticks, M-0017 Procedure 2) to
    /// the game's end, and those at which the observer house does not own
    /// exactly one object by [`owned`]'s count, or is defeated: (tick,
    /// count, defeated).
    pub samples: (usize, Vec<(i32, usize, bool)>),
}

/// The objects of `w` that the houses other than `players` and `observer`
/// own, by owner house and role.
fn other_owners(
    w: &World,
    players: [HouseId; 2],
    observer: HouseId,
) -> Vec<(u8, glasnost_sim::world::HouseRole)> {
    let owners: BTreeSet<u8> = w
        .objects
        .values()
        .filter_map(|o| o.owner)
        .filter(|h| !players.contains(h) && *h != observer)
        .map(|h| h.0)
        .collect();
    owners
        .into_iter()
        .map(|h| {
            (
                h,
                w.houses
                    .get(usize::from(h))
                    .map_or(Default::default(), |x| x.role),
            )
        })
        .collect()
}

/// [`play`] with the M-0019 facts of the other houses ([`OtherHouses`]),
/// read after each step without changing the game.
pub fn play_checked(
    ground: &Ground,
    row: &str,
    col: &str,
    row_side: Side,
    seed: u32,
    watch: bool,
) -> Result<(Game, String, OtherHouses), String> {
    play_with(
        ground,
        row,
        col,
        row_side,
        seed,
        watch,
        &mut |_| Ok(()),
        &mut |_, _| {},
    )
}

/// [`play_checked`] with two hooks of a record's harness: `prepare` sees
/// the set-up before the game's first step (and may change it: M-0020's
/// moved arms), `observe` reads the world and the tick's effects after
/// each step, before the game's own reading of them (read only).
#[allow(clippy::too_many_arguments)]
pub fn play_with(
    ground: &Ground,
    row: &str,
    col: &str,
    row_side: Side,
    seed: u32,
    watch: bool,
    prepare: &mut dyn FnMut(&mut Arena) -> Result<(), String>,
    observe: &mut dyn FnMut(&World, &[glasnost_sim::combat::effects::Effect]),
) -> Result<(Game, String, OtherHouses), String> {
    use glasnost_sim::world::HouseRole;
    let types = match row_side {
        Side::A => [row, col],
        Side::B => [col, row],
    };
    let mut arena = setup(ground, types, row_side, seed)?;
    prepare(&mut arena)?;
    let Arena {
        world: mut w,
        houses,
        units,
        moved,
        ..
    } = arena;
    let mut seen = String::new();
    if watch {
        seen.push_str(&moved_text(&moved));
    }
    let mut facts = OtherHouses::default();
    let neutral: Vec<HouseId> = w
        .houses
        .iter()
        .enumerate()
        .filter(|(_, h)| h.role == HouseRole::Neutral)
        .filter_map(|(i, _)| u8::try_from(i).ok().map(HouseId))
        .collect();
    let mut neutral_vehicles_at_0: Vec<ObjectId> = Vec::new();
    let mut shots = [0u32; 2];
    let (winner_side, end_frame) = loop {
        glasnost_sim::tick::step(&mut w);
        // measure: M-0019 (Procedure 8: the objects of the other houses at tick 0, after its update, and at the next tick); spec: S-0201 claim 1 (our step run while the tick number is 0 is tick 0)
        if w.tick == 1 || w.tick == 2 {
            let k = (w.tick - 1) as usize;
            facts.owners[k] = other_owners(&w, houses, HUMAN);
            facts.observer[k] = w
                .objects
                .values()
                .filter(|o| o.owner == Some(HUMAN))
                .map(|o| (o.kind, o.on_map))
                .collect();
            if k == 0 {
                let owners: BTreeSet<Option<u8>> = w
                    .objects
                    .values()
                    .filter(|o| glasnost_sim::structures::bunker::is_bunker(&w, o.id))
                    .map(|o| o.owner.map(|h| h.0))
                    .collect();
                facts.bunker_owners = owners.into_iter().collect();
                for o in w.objects.values() {
                    if !o.owner.is_some_and(|h| neutral.contains(&h)) {
                        continue;
                    }
                    if o.kind == Kind::Vehicle {
                        neutral_vehicles_at_0.push(o.id);
                        if o.on_map {
                            facts.neutral_vehicles.0 += 1;
                        } else {
                            facts.neutral_vehicles.1 += 1;
                        }
                    } else if matches!(o.kind, Kind::Infantry | Kind::Building) {
                        if o.on_map {
                            facts.neutral_rest.0 += 1;
                        } else {
                            facts.neutral_rest.1 += 1;
                        }
                    }
                }
            } else {
                facts.neutral_vehicles_next = neutral_vehicles_at_0
                    .iter()
                    .filter(|&&id| w.obj(id).is_some())
                    .count();
            }
        }
        let effects = std::mem::take(&mut w.effects);
        observe(&w, &effects);
        for e in effects {
            if watch
                && let glasnost_sim::combat::effects::Effect::Shot { firer, .. } = e
                && let Some(owner) = w.obj(firer).and_then(|o| o.owner)
                && let Some(s) = houses.iter().position(|&h| h == owner)
            {
                shots[s] += 1;
            }
        }
        // A game whose final tick has run stays as it is (S-0414 claim 8,
        // S-0201 claim 1): it is read at the next sample.
        let finished = w.econ.end.finished;
        let frame = if finished {
            (w.tick + SAMPLE_PERIOD - 1) / SAMPLE_PERIOD * SAMPLE_PERIOD
        } else {
            w.tick
        };
        if !finished && (frame <= 0 || frame % SAMPLE_PERIOD != 0) {
            continue;
        }
        let o = houses.map(|h| owned(&w, h));
        // measure: M-0019 (Procedure 8: the observer house's row in every sample of the periodic sampler to the run's end: a unit count of 1, not defeated); our count is [`owned`]'s
        let (n, lost) = (
            owned(&w, HUMAN).0,
            glasnost_sim::econ::victory::defeated(&w, HUMAN),
        );
        facts.samples.0 += 1;
        if n != 1 || lost {
            facts.samples.1.push((frame, n, lost));
        }
        if watch {
            let _ = writeln!(
                seen,
                "  frame {frame}: A owns {} (value {}), B owns {} (value {}); shots so far A {}, B {}",
                o[0].0, o[0].1, o[1].0, o[1].1, shots[0], shots[1]
            );
        }
        match (o[0].0 > 0, o[1].0 > 0) {
            (true, false) => break (Some(Side::A), frame),
            (false, true) => break (Some(Side::B), frame),
            (false, false) => break (None, frame),
            (true, true) if frame >= FRAME_LIMIT || finished => break (None, frame),
            _ => {}
        }
    };
    if watch {
        // The ground around W: each cell's height level, `*` where the cell
        // is outside the usable playfield, `A`/`B` where a unit of a side
        // stands at the end.
        let _ = writeln!(
            seen,
            "  ground around W ({}, {}) at the end (rows y, columns x from W.x - 16):",
            ground.w.x, ground.w.y
        );
        for y in ground.w.y - 8..=ground.w.y + 8 {
            let mut line = format!("    y {y:4} ");
            for x in ground.w.x - 16..=ground.w.x + 16 {
                let c = CellXY::new(x, y);
                let here = |s: usize| {
                    units[s]
                        .iter()
                        .any(|&id| w.obj(id).is_some_and(|o| o.health > 0 && o.pos.cell() == c))
                };
                let ch = if here(0) {
                    'A'
                } else if here(1) {
                    'B'
                } else {
                    match w.cells.cell(c) {
                        None => ' ',
                        Some(_) if !w.cells.usable(c) => '*',
                        Some(cl) => char::from_digit(u32::from(cl.level.clamp(0, 9) as u8), 10)
                            .unwrap_or('?'),
                    }
                };
                line.push(ch);
            }
            let _ = writeln!(seen, "{line}");
        }
        // The reach regions of each side's movement zone (S-0212 claim 4),
        // one letter per region in order of appearance, `.` for none.
        for (s, t) in types.iter().enumerate() {
            let Some(zone) = units[s]
                .first()
                .and_then(|&id| w.obj(id).map(|o| (o.kind, o.type_index)))
                .or_else(|| {
                    group(&w.rules_arc(), t)
                        .ok()
                        .and_then(|(k, _, _)| w.rules().index_of(k, t).map(|i| (k, i)))
                })
                .and_then(|(k, i)| w.rules().unit(k, i).map(|u| u.zone))
            else {
                continue;
            };
            let _ = writeln!(
                seen,
                "  reach regions of side {}'s zone {zone:?}:",
                ["A", "B"][s]
            );
            let mut names: Vec<glasnost_sim::regions::Reach> = Vec::new();
            for y in ground.w.y - 8..=ground.w.y + 8 {
                let mut line = format!("    y {y:4} ");
                for x in ground.w.x - 16..=ground.w.x + 16 {
                    let r = glasnost_sim::regions::reach_in(&w, CellXY::new(x, y), zone);
                    let ch = match r {
                        glasnost_sim::regions::Reach::Region(_) => {
                            let k = names.iter().position(|n| *n == r).unwrap_or_else(|| {
                                names.push(r);
                                names.len() - 1
                            });
                            char::from(b'a' + (k % 26) as u8)
                        }
                        _ => '.',
                    };
                    line.push(ch);
                }
                let _ = writeln!(seen, "{line}");
            }
        }
        for (s, ids) in units.iter().enumerate() {
            for &id in ids {
                let Some(o) = w.obj(id) else {
                    let _ = writeln!(seen, "    {} #{}: gone", ["A", "B"][s], id.0);
                    continue;
                };
                let _ = writeln!(
                    seen,
                    "    {} #{}: owner {:?}, health {}, on map {}, cell ({}, {}), behaviour {:?}, target {:?}, destination {:?}",
                    ["A", "B"][s],
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
    let winner = match winner_side {
        None => Winner::None,
        Some(s) if s == row_side => Winner::Row,
        Some(_) => Winner::Col,
    };
    Ok((
        Game {
            row: row.to_owned(),
            col: col.to_owned(),
            row_side,
            seed,
            winner,
            end_frame,
        },
        seen,
        facts,
    ))
}

/// M-0019's rows under *Other houses at tick 0* (revised 2026-11-05) over
/// our games: (game, its facts) to (row, verdict), in the record's order.
pub fn other_houses_verdicts(
    games: &[(String, OtherHouses)],
) -> Vec<(String, crate::oracle_m0021::Verdict)> {
    use crate::oracle_m0021::Verdict;
    use glasnost_sim::world::HouseRole;
    let rows = [
        "M-0019 other houses: every game has objects of exactly one house besides the two players' and the observer's, the map's neutral house, which owns the bunkers",
        "M-0019 other houses: the neutral house: at tick 0 every vehicle it owns is off the map",
        "M-0019 other houses: the neutral house: none of those vehicles' identifiers is among the objects at the next tick",
        "M-0019 other houses: the neutral house: its infantry and buildings are all on the map at tick 0",
        "M-0019 other houses: the observer house: at tick 0 and at the next tick it owns one object, a vehicle on the map, and nothing else",
        "M-0019 other houses: the observer house: every sample to the run's end gives it a unit count of 1, not defeated",
    ];
    if games.is_empty() {
        return rows
            .iter()
            .map(|r| {
                (
                    (*r).to_owned(),
                    Verdict::NotCheckable("no game was played".into()),
                )
            })
            .collect();
    }
    let mut bad: [Vec<String>; 6] = Default::default();
    let mut vacuous = 0;
    for (name, f) in games {
        // measure: M-0019 (Other houses at tick 0: in every game the recorder gives objects of exactly one house besides the two players' and the observer's, the house that owns the map's four bunkers, the map's neutral house)
        let neutral_only =
            (0..2).all(|k| f.owners[k].len() == 1 && f.owners[k][0].1 == HouseRole::Neutral);
        let bunkers_neutral = f
            .bunker_owners
            .iter()
            .all(|b| f.owners[0].first().is_some_and(|o| Some(o.0) == *b));
        if !(neutral_only && bunkers_neutral) {
            bad[0].push(format!(
                "{name}: other owners {:?} / {:?}, bunker owners {:?}",
                f.owners[0], f.owners[1], f.bunker_owners
            ));
        }
        // measure: M-0019 (Other houses at tick 0: the neutral house: at tick 0 every vehicle it owns is off the map)
        // The counts are the stock map's content (M-0019: not given), so only the local file holds them.
        if f.neutral_vehicles.0 > 0 {
            bad[1].push(format!("{name}: some on the map"));
        }
        if f.neutral_vehicles == (0, 0) {
            vacuous += 1;
        }
        // measure: M-0019 (Other houses at tick 0: none of their identifiers is among the objects the recorder gives at the next tick)
        if f.neutral_vehicles_next > 0 {
            bad[2].push(format!("{name}: some still there"));
        }
        // measure: M-0019 (Other houses at tick 0: its infantry and its buildings are all on the map at tick 0)
        if f.neutral_rest.1 > 0 || f.neutral_rest.0 == 0 {
            bad[3].push(format!(
                "{name}: {}",
                if f.neutral_rest.0 == 0 {
                    "none on the map"
                } else {
                    "some off the map"
                }
            ));
        }
        // measure: M-0019 (Other houses at tick 0: the observer house: at tick 0 and at the next tick it owns one object, a vehicle on the map, and nothing else)
        if !(0..2).all(|k| f.observer[k] == [(Kind::Vehicle, true)]) {
            bad[4].push(format!("{name}: {:?} / {:?}", f.observer[0], f.observer[1]));
        }
        // measure: M-0019 (Other houses at tick 0: in every sample to the run's end, the observer house's row gives a unit count of 1, and the house is not defeated)
        if f.samples.0 == 0 || !f.samples.1.is_empty() {
            bad[5].push(format!(
                "{name}: {} samples, off: {:?}",
                f.samples.0,
                f.samples.1.first()
            ));
        }
    }
    let n = games.len();
    rows.iter()
        .zip(bad)
        .enumerate()
        .map(|(i, (r, b))| {
            let v = if b.is_empty() {
                Verdict::Pass
            } else {
                Verdict::Fail(format!(
                    "{} of {n} games: {}",
                    b.len(),
                    b.iter().take(3).cloned().collect::<Vec<_>>().join("; ")
                ))
            };
            let row = if i == 1 && vacuous > 0 {
                format!(
                    "{r} (ours has none at tick 0 in {vacuous} of {n} games: S-0216 claim 8's removal before the groups takes them out of the world, so the row holds there with no vehicle to read)"
                )
            } else {
                (*r).to_owned()
            };
            (row, v)
        })
        .collect()
}

/// One line per object of the two houses (and per object a side's group
/// started with): what a seed can change in play.
fn unit_states(w: &World, houses: [HouseId; 2], units: &[Vec<ObjectId>; 2]) -> Vec<String> {
    let mut ids: Vec<ObjectId> = units.iter().flatten().copied().collect();
    ids.extend(
        w.objects
            .values()
            .filter(|o| o.owner.is_some_and(|h| houses.contains(&h)))
            .map(|o| o.id),
    );
    ids.sort_unstable();
    ids.dedup();
    ids.iter()
        .map(|&id| match w.obj(id) {
            None => format!("#{} gone", id.0),
            Some(o) => format!(
                "#{} owner {:?} health {} on map {} pos ({}, {}, {}) body {:?} turret {:?} behaviour {:?} target {:?} destination {:?}",
                id.0,
                o.owner.map(|h| h.0),
                o.health,
                o.on_map,
                o.pos.x,
                o.pos.y,
                o.pos.z,
                o.body,
                o.turret,
                o.beh.current,
                o.target,
                o.dest
            ),
        })
        .collect()
}

/// The seed-to-seed spread of one pairing and side (an M8 step A1
/// diagnostic): plays seeds 0 to 5 and reports, for each seed, its result,
/// the raw values its scenario random stream gave, the first tick at which
/// a unit of either house differs from the seed-0 game (and the first such
/// unit in both games), and the notes (provisional rules and seams) its
/// game met.
pub fn spread(ground: &Ground, row: &str, col: &str, row_side: Side) -> Result<String, String> {
    let types = match row_side {
        Side::A => [row, col],
        Side::B => [col, row],
    };
    let mut out = String::new();
    let mut base: Vec<Vec<String>> = Vec::new();
    let mut notes = std::collections::BTreeSet::new();
    for seed in SEEDS {
        let (g, _) = play(ground, row, col, row_side, seed, false)?;
        let Arena {
            world: mut w,
            houses,
            units,
            ..
        } = setup(ground, types, row_side, seed)?;
        let mut first: Option<(i32, String, String)> = None;
        let mut states = vec![unit_states(&w, houses, &units)];
        let mut t = 0usize;
        while w.tick < g.end_frame && !w.econ.end.finished {
            glasnost_sim::tick::step(&mut w);
            w.effects.clear();
            t += 1;
            let s = unit_states(&w, houses, &units);
            if seed > 0 && first.is_none() {
                let b = base.get(t).or(base.last());
                if let Some(b) = b
                    && *b != s
                {
                    let k = b.iter().zip(&s).position(|(x, y)| x != y);
                    let (x, y) = k.map_or_else(
                        || (format!("{} lines", b.len()), format!("{} lines", s.len())),
                        |k| (b[k].clone(), s[k].clone()),
                    );
                    first = Some((w.tick, x, y));
                }
            }
            if seed == 0 {
                states.push(s);
            }
        }
        if seed == 0 {
            base = states;
        }
        let _ = writeln!(
            out,
            "seed {seed}: {} ; raw values taken {}; {}",
            g.line(),
            w.rng.taken(),
            match &first {
                None if seed == 0 => "the reference game".to_owned(),
                None => "no unit differs from seed 0 at any tick".to_owned(),
                Some((t, _, _)) => format!("first differs from seed 0 at tick {t}"),
            }
        );
        if let Some((_, x, y)) = first {
            let _ = writeln!(out, "    seed 0: {x}\n    seed {seed}: {y}");
        }
        notes.extend(w.notes.iter().cloned());
    }
    for n in notes {
        let _ = writeln!(out, "  note: {n}");
    }
    Ok(out)
}

/// Plays one pairing's side over `seeds` and reports each game and the
/// winner counts (`--seed-range`). The records give only seeds 0 to 5, so this is
/// a check of how often our games give an outcome, not a comparison.
pub fn seed_range(
    ground: &Ground,
    row: &str,
    col: &str,
    side: Side,
    seeds: std::ops::RangeInclusive<u32>,
    threads: usize,
) -> String {
    let jobs: Vec<Job> = seeds
        .map(|s| (row.to_owned(), col.to_owned(), side, s))
        .collect();
    let games = run_jobs(ground, &jobs, threads);
    let mut out = format!("{row} against {col}, row on side {}:", side.text());
    let mut counts = [0u32; 3];
    let mut failed = 0;
    for (j, g) in jobs.iter().zip(&games) {
        match g {
            Ok(g) => {
                counts[g.winner as usize] += 1;
                let _ = write!(out, " {}:{}@{}", j.3, g.winner.text(), g.end_frame);
            }
            Err(_) => failed += 1,
        }
    }
    let _ = writeln!(
        out,
        "\n  row {} col {} none {} of {} (not played: {failed})",
        counts[0],
        counts[1],
        counts[2],
        jobs.len()
    );
    out
}

/// One game to play: row, column, the row's side, the seed.
pub type Job = (String, String, Side, u32);

/// The jobs of a record's pairings (every pairing when `only` is empty):
/// per pairing the row on side A for seeds 0 to 5, then on side B.
pub fn jobs(record: &OracleRecord, only: &[(String, String)]) -> Vec<Job> {
    let mut out = Vec::new();
    for (row, col) in pairings(&record.games) {
        if !only.is_empty()
            && !only
                .iter()
                .any(|(r, c)| r.eq_ignore_ascii_case(&row) && c.eq_ignore_ascii_case(&col))
        {
            continue;
        }
        for side in [Side::A, Side::B] {
            for seed in SEEDS {
                out.push((row.clone(), col.clone(), side, seed));
            }
        }
    }
    out
}

/// Plays `jobs` on `threads` worker threads (each game is its own world;
/// the results come back in job order).
pub fn run_jobs(ground: &Ground, jobs: &[Job], threads: usize) -> Vec<Result<Game, String>> {
    run_jobs_checked(ground, jobs, threads)
        .into_iter()
        .map(|r| r.map(|(g, _)| g))
        .collect()
}

/// [`run_jobs`] with each game's M-0019 facts of the other houses
/// ([`play_checked`]).
pub fn run_jobs_checked(
    ground: &Ground,
    jobs: &[Job],
    threads: usize,
) -> Vec<Result<(Game, OtherHouses), String>> {
    let next = AtomicUsize::new(0);
    type Slot = std::sync::Mutex<Option<Result<(Game, OtherHouses), String>>>;
    let results: Vec<Slot> = jobs.iter().map(|_| std::sync::Mutex::new(None)).collect();
    std::thread::scope(|s| {
        for _ in 0..threads.max(1) {
            s.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some((row, col, side, seed)) = jobs.get(i) else {
                        break;
                    };
                    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        play_checked(ground, row, col, *side, *seed, false)
                            .map(|(g, _, facts)| (g, facts))
                    }))
                    .unwrap_or_else(|_| Err(format!("{row} against {col}: panic")));
                    if let Ok(mut slot) = results[i].lock() {
                        *slot = Some(r);
                    }
                }
            });
        }
    });
    results
        .into_iter()
        .map(|m| {
            m.into_inner()
                .ok()
                .flatten()
                .unwrap_or_else(|| Err("not run".into()))
        })
        .collect()
}

/// Replaces in `stored` every game of the pairings of `fresh` by `fresh`'s
/// games, keeping the record's pairing order (`order`).
pub fn merge(stored: &[Game], fresh: &[Game], order: &[(String, String)]) -> Vec<Game> {
    let redone = pairings(fresh);
    let mut out = Vec::new();
    for p in order {
        let src = if redone.contains(p) { fresh } else { stored };
        out.extend(src.iter().filter(|g| g.row == p.0 && g.col == p.1).cloned());
    }
    out
}

// ---------------------------------------------------------------------------
// Command line

/// The usage of `arena --record`.
pub const USAGE: &str = "arena --record <ID|all> [--pairing ROW,COL]… [--threads N] [--out DIR] [--measurements DIR] [--compare-only] [--groups] [--ai-houses] [--watch ROW,COL,A|B,SEED] [--trace|--trace-summary ROW,COL,A|B,SEED] [--spread ROW,COL,A|B] [--seed-range ROW,COL,A|B,FIRST-LAST]";

/// The `arena --record` command line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cli {
    /// The record's ID, or `all`.
    pub record: String,
    /// Only these pairings (all when empty).
    pub pairings: Vec<(String, String)>,
    /// Worker threads.
    pub threads: usize,
    /// Where our CSV and the comparison go.
    pub out: PathBuf,
    /// Where the records are.
    pub measurements: PathBuf,
    /// Compare the stored CSV of ours without playing.
    pub compare_only: bool,
    /// Play this one game and describe it.
    pub watch: Option<Job>,
    /// Play this one game and print its per-unit trace
    /// ([`crate::oracle_trace::trace`]); the flag asks for every change,
    /// not only the summary.
    pub trace: Option<(Job, bool)>,
    /// List each type's kind, group size and cost, and play nothing.
    pub groups: bool,
    /// Report the seed-to-seed spread of one pairing and side ([`spread`]).
    pub spread: Option<(String, String, Side)>,
    /// Play the houses as AI houses of our engine (human-controlled rules)
    /// instead of computer-controlled ones.
    pub ai_houses: bool,
    /// Play one pairing's side over a range of seeds beyond the records'
    /// 0 to 5 and count the winners ([`seed_range`]; a noise check of ours,
    /// the records have no games for those seeds).
    pub seed_range: Option<(String, String, Side, std::ops::RangeInclusive<u32>)>,
}

/// Default worker threads (our tool's choice).
pub const DEFAULT_THREADS: usize = 6;

/// Parses the arguments after `arena` when they hold `--record`.
pub fn parse_cli(args: &[String]) -> Result<Cli, String> {
    let mut cli = Cli {
        record: String::new(),
        pairings: Vec::new(),
        threads: DEFAULT_THREADS,
        out: PathBuf::from("local-lane/m7b/arena"),
        measurements: PathBuf::from("measurements"),
        compare_only: false,
        watch: None,
        trace: None,
        groups: false,
        spread: None,
        ai_houses: false,
        seed_range: None,
    };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut value = |name: &str| {
            it.next()
                .cloned()
                .ok_or_else(|| format!("{name} needs a value"))
        };
        match a.as_str() {
            "--record" => cli.record = value(a)?,
            "--pairing" => {
                let v = value(a)?;
                let (r, c) = v.split_once(',').ok_or("--pairing needs ROW,COL")?;
                cli.pairings
                    .push((r.trim().to_owned(), c.trim().to_owned()));
            }
            "--threads" => {
                cli.threads = value(a)?
                    .parse()
                    .ok()
                    .filter(|&n| n > 0)
                    .ok_or("--threads needs a positive whole number")?
            }
            "--out" => cli.out = PathBuf::from(value(a)?),
            "--measurements" => cli.measurements = PathBuf::from(value(a)?),
            "--compare-only" => cli.compare_only = true,
            "--groups" => cli.groups = true,
            "--ai-houses" => cli.ai_houses = true,
            "--spread" => {
                let v = value(a)?;
                let f: Vec<&str> = v.split(',').map(str::trim).collect();
                let side = match f.get(2).copied() {
                    Some("A" | "a") => Side::A,
                    Some("B" | "b") => Side::B,
                    _ => return Err("--spread needs ROW,COL,A|B".into()),
                };
                cli.spread = Some((f[0].to_owned(), f[1].to_owned(), side));
            }
            "--seed-range" => {
                let v = value(a)?;
                let f: Vec<&str> = v.split(',').map(str::trim).collect();
                let bad = || "--seed-range needs ROW,COL,A|B,FIRST-LAST".to_owned();
                let side = match f.get(2).copied() {
                    Some("A" | "a") => Side::A,
                    Some("B" | "b") => Side::B,
                    _ => return Err(bad()),
                };
                let (lo, hi) = f.get(3).and_then(|r| r.split_once('-')).ok_or_else(bad)?;
                let lo: u32 = lo.trim().parse().map_err(|_| bad())?;
                let hi: u32 = hi.trim().parse().map_err(|_| bad())?;
                if lo > hi {
                    return Err(bad());
                }
                cli.seed_range = Some((f[0].to_owned(), f[1].to_owned(), side, lo..=hi));
            }
            "--watch" => {
                let v = value(a)?;
                let f: Vec<&str> = v.split(',').map(str::trim).collect();
                let side = match f.get(2).copied() {
                    Some("A" | "a") => Side::A,
                    Some("B" | "b") => Side::B,
                    _ => return Err("--watch needs ROW,COL,A|B,SEED".into()),
                };
                let seed = f
                    .get(3)
                    .and_then(|s| s.parse().ok())
                    .ok_or("--watch needs ROW,COL,A|B,SEED")?;
                cli.watch = Some((f[0].to_owned(), f[1].to_owned(), side, seed));
            }
            "--trace" | "--trace-summary" => {
                let detail = a == "--trace";
                let v = value(a)?;
                let f: Vec<&str> = v.split(',').map(str::trim).collect();
                let side = match f.get(2).copied() {
                    Some("A" | "a") => Side::A,
                    Some("B" | "b") => Side::B,
                    _ => return Err(format!("{a} needs ROW,COL,A|B,SEED")),
                };
                let seed = f
                    .get(3)
                    .and_then(|s| s.parse().ok())
                    .ok_or_else(|| format!("{a} needs ROW,COL,A|B,SEED"))?;
                cli.trace = Some(((f[0].to_owned(), f[1].to_owned(), side, seed), detail));
            }
            other => return Err(format!("unknown argument {other:?}")),
        }
    }
    if cli.record.is_empty() {
        return Err("--record needs a record ID or all".into());
    }
    Ok(cli)
}

/// Runs the command line: plays (or with `--compare-only` reads) our games
/// of the chosen records, stores them as `<out>/ours-<ID>.csv` (merged
/// with the stored ones when only some pairings were played), compares
/// them with the record and writes `<out>/oracle-<ID>.txt`. `game_dir` is
/// the install (`None`: only `--compare-only` works). Returns the report.
pub fn run_cli(cli: &Cli, game_dir: Option<&Path>) -> Result<String, String> {
    let records = load_records(&cli.measurements)?;
    let only_m0021 = cli.record.eq_ignore_ascii_case(crate::oracle_m0021::ID);
    if only_m0021 {
        return run_m0021(cli, game_dir, &records);
    }
    if cli.record.eq_ignore_ascii_case(crate::oracle_m0020::ID) {
        return run_m0020(cli, game_dir, &records);
    }
    let mut out = run_records(cli, game_dir, &records)?;
    let plain = cli.pairings.is_empty()
        && !cli.compare_only
        && !cli.groups
        && cli.spread.is_none()
        && cli.trace.is_none()
        && cli.seed_range.is_none()
        && cli.watch.is_none();
    if cli.record.eq_ignore_ascii_case("all") && plain {
        out.push_str(&run_m0021(cli, game_dir, &records)?);
        out.push_str(&run_m0020(cli, game_dir, &records)?);
    }
    Ok(out)
}

/// M-0020's games ([`crate::oracle_m0020`]) on M-0001's ground: the
/// verdicts to the output, the whole report (with its local values, beside
/// the record's from `measurements/local/M-0020.json` when present) to
/// `oracle-M-0020.txt` in the output directory. Skipped when M-0020's
/// record file is not in the measurements directory.
pub fn run_m0020(
    cli: &Cli,
    game_dir: Option<&Path>,
    records: &[OracleRecord],
) -> Result<String, String> {
    use crate::oracle_m0020 as m20;
    let present = record_paths(&cli.measurements).iter().any(|p| {
        p.file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with(m20::ID))
    });
    if !present {
        return Ok(format!(
            "skipped: no {} record in {}\n",
            m20::ID,
            cli.measurements.display()
        ));
    }
    let Some(map) = find_record(records, m20::MAP_RECORD) else {
        return Ok(format!(
            "skipped: {} plays on {}'s map, and {} is not in {}\n",
            m20::ID,
            m20::MAP_RECORD,
            m20::MAP_RECORD,
            cli.measurements.display()
        ));
    };
    let dir = game_dir.ok_or_else(|| {
        format!(
            "{} is not set: the oracle arena needs the install (local lane only)",
            crate::local_lane::GAME_DIR_VAR
        )
    })?;
    let mut ground = stock_ground(dir, &map.map_file, &map.map_sha256)?;
    // measure: M-0020 (Setup: two computer houses, as in M-0001 to M-0003)
    ground.computer = true;
    // measure: M-0020 (Setup: part 1 replays M-0001's and M-0002's archived games; part 2's arm S at seeds 0 to 5 against them)
    let archived: Vec<Game> = records
        .iter()
        .filter(|r| r.id == "M-0001" || r.id == "M-0002")
        .flat_map(|r| r.games.iter().cloned())
        .collect();
    let json = std::fs::read_to_string(
        cli.measurements
            .join("local")
            .join(format!("{}.json", m20::ID)),
    )
    .ok();
    let t0 = std::time::Instant::now();
    let (summary, full) = m20::run(&ground, &archived, json.as_deref(), cli.threads)?;
    std::fs::create_dir_all(&cli.out).map_err(|e| format!("{}: {e}", cli.out.display()))?;
    let txt = cli.out.join(format!("oracle-{}.txt", m20::ID));
    std::fs::write(&txt, &full).map_err(|e| format!("{}: {e}", txt.display()))?;
    Ok(format!(
        "{}: played on {} threads in {} s\n{summary}wrote {} (git-ignored, with local values; never commit it)\n",
        m20::ID,
        cli.threads,
        t0.elapsed().as_secs(),
        txt.display()
    ))
}

/// M-0021's test pairing ([`crate::oracle_m0021`]) on M-0001's ground: the
/// verdicts to the output, the whole report (with its local values) to
/// `oracle-M-0021.txt` in the output directory. Skipped when M-0021's
/// record file is not in the measurements directory.
pub fn run_m0021(
    cli: &Cli,
    game_dir: Option<&Path>,
    records: &[OracleRecord],
) -> Result<String, String> {
    use crate::oracle_m0021 as m21;
    let present = record_paths(&cli.measurements).iter().any(|p| {
        p.file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with(m21::ID))
    });
    if !present {
        return Ok(format!(
            "skipped: no {} record in {}\n",
            m21::ID,
            cli.measurements.display()
        ));
    }
    let Some(map) = find_record(records, m21::MAP_RECORD) else {
        return Ok(format!(
            "skipped: {} plays on {}'s map, and {} is not in {}\n",
            m21::ID,
            m21::MAP_RECORD,
            m21::MAP_RECORD,
            cli.measurements.display()
        ));
    };
    let dir = game_dir.ok_or_else(|| {
        format!(
            "{} is not set: the oracle arena needs the install (local lane only)",
            crate::local_lane::GAME_DIR_VAR
        )
    })?;
    let (ground, base, hash) = stock_inputs(dir, &map.map_file, &map.map_sha256)?;
    let from = m21::base_from_env(m21::DEFAULT_BASE);
    let t0 = std::time::Instant::now();
    let (summary, full) = m21::run(&ground, &base, &hash, &from, cli.threads)?;
    std::fs::create_dir_all(&cli.out).map_err(|e| format!("{}: {e}", cli.out.display()))?;
    let txt = cli.out.join(format!("oracle-{}.txt", m21::ID));
    std::fs::write(&txt, &full).map_err(|e| format!("{}: {e}", txt.display()))?;
    Ok(format!(
        "{}: played on {} threads in {} s\n{summary}wrote {} (git-ignored, with local values; never commit it)\n",
        m21::ID,
        cli.threads,
        t0.elapsed().as_secs(),
        txt.display()
    ))
}

/// [`run_cli`] for the records with a games file (M-0001 to M-0003).
fn run_records(
    cli: &Cli,
    game_dir: Option<&Path>,
    records: &[OracleRecord],
) -> Result<String, String> {
    if records.is_empty() {
        return Ok(format!(
            "skipped: no oracle arena record in {}\n",
            cli.measurements.display()
        ));
    }
    let chosen: Vec<&OracleRecord> = if cli.record.eq_ignore_ascii_case("all") {
        records.iter().collect()
    } else {
        vec![
            find_record(records, &cli.record)
                .ok_or_else(|| format!("no oracle arena record {}", cli.record))?,
        ]
    };
    let mut out = String::new();
    let mut grounds: BTreeMap<(String, String), Ground> = BTreeMap::new();
    let mut ground_of = |r: &OracleRecord| -> Result<Ground, String> {
        let key = (r.map_file.clone(), r.map_sha256.clone());
        if let Some(g) = grounds.get(&key) {
            return Ok(g.clone());
        }
        let dir = game_dir.ok_or_else(|| {
            format!(
                "{} is not set: the oracle arena needs the install (local lane only)",
                crate::local_lane::GAME_DIR_VAR
            )
        })?;
        let t0 = std::time::Instant::now();
        let mut g = stock_ground(dir, &r.map_file, &r.map_sha256)?;
        g.computer = !cli.ai_houses;
        eprintln!("loaded {} in {} ms", r.map_file, t0.elapsed().as_millis());
        grounds.insert(key, g.clone());
        Ok(g)
    };
    if cli.groups {
        for r in &chosen {
            let ground = ground_of(r)?;
            let mut types: Vec<String> = Vec::new();
            for g in &r.games {
                for t in [&g.row, &g.col] {
                    if !types.contains(t) {
                        types.push(t.clone());
                    }
                }
            }
            let _ = writeln!(out, "{} groups (type, kind, units, cost):", r.id);
            for t in &types {
                match group(&ground.rules, t) {
                    Ok((kind, n, cost)) => {
                        let powered = ground.rules.index_of(kind, t).is_some_and(|i| {
                            kind == Kind::Vehicle
                                && ground
                                    .rules
                                    .econ
                                    .power
                                    .powered_unit
                                    .get(i as usize)
                                    .copied()
                                    .unwrap_or(false)
                        });
                        let _ = writeln!(
                            out,
                            "  {t} {kind:?} {n} {cost}{}",
                            if powered {
                                " (powered unit: disabled on entering a cell without a control centre, S-0405 claim 13)"
                            } else {
                                ""
                            }
                        );
                    }
                    Err(e) => {
                        let _ = writeln!(out, "  {t}: {e}");
                    }
                }
            }
            // The placement on each side, with the type against itself.
            let mut others = None;
            for t in &types {
                match setup(&ground, [t, t], Side::A, 0) {
                    Ok(a) => {
                        let n = |side| a.moved.iter().filter(|m| m.side == side).count();
                        let _ = writeln!(
                            out,
                            "  {t}: units off their stated cells: side A {}, side B {}",
                            n(Side::A),
                            n(Side::B)
                        );
                        out.push_str(&moved_text(&a.moved));
                        others.get_or_insert_with(|| others_text(&a));
                    }
                    Err(e) => {
                        let _ = writeln!(out, "  {t}: set-up failed: {e}");
                    }
                }
            }
            if let Some(o) = others {
                let _ = writeln!(
                    out,
                    "{} map objects at tick 0 that neither side owns:",
                    r.id
                );
                out.push_str(&o);
            }
        }
        return Ok(out);
    }
    if let Some((row, col, side)) = &cli.spread {
        let r = chosen[0];
        let ground = ground_of(r)?;
        let _ = writeln!(
            out,
            "{} spread: {row} against {col}, row on side {}",
            r.id,
            side.text()
        );
        out.push_str(&spread(&ground, row, col, *side)?);
        return Ok(out);
    }
    if let Some(((row, col, side, seed), detail)) = &cli.trace {
        let r = chosen[0];
        let ground = ground_of(r)?;
        out.push_str(&crate::oracle_trace::trace(
            &ground, row, col, *side, *seed, *detail,
        )?);
        return Ok(out);
    }
    if let Some((row, col, side, range)) = &cli.seed_range {
        let r = chosen[0];
        let ground = ground_of(r)?;
        out.push_str(&seed_range(
            &ground,
            row,
            col,
            *side,
            range.clone(),
            cli.threads,
        ));
        return Ok(out);
    }
    if let Some((row, col, side, seed)) = &cli.watch {
        let r = chosen[0];
        let ground = ground_of(r)?;
        let (g, seen) = play(&ground, row, col, *side, *seed, true)?;
        let _ = writeln!(out, "{} watch: {}", r.id, g.line());
        for t in [row, col] {
            let (kind, n, cost) = group(&ground.rules, t)?;
            let _ = writeln!(out, "  {t}: {kind:?}, {n} units at cost {cost}");
        }
        out.push_str(&seen);
        return Ok(out);
    }
    std::fs::create_dir_all(&cli.out).map_err(|e| format!("{}: {e}", cli.out.display()))?;
    let mut other_houses: Vec<(String, OtherHouses)> = Vec::new();
    for r in chosen {
        check_shape(&r.games).map_err(|e| format!("{}: {e}", r.id))?;
        let stored_path = cli.out.join(format!("ours-{}.csv", r.id));
        let stored = std::fs::read_to_string(&stored_path)
            .ok()
            .map(|t| parse_csv(&t))
            .transpose()?
            .unwrap_or_default();
        let ours = if cli.compare_only {
            stored
        } else {
            let ground = ground_of(r)?;
            let jobs = jobs(r, &cli.pairings);
            let t0 = std::time::Instant::now();
            let results = run_jobs_checked(&ground, &jobs, cli.threads);
            let secs = t0.elapsed().as_secs();
            let mut fresh = Vec::new();
            for (j, res) in jobs.iter().zip(results) {
                match res {
                    Ok((g, facts)) => {
                        other_houses.push((
                            format!(
                                "{} {} against {} side {} seed {}",
                                r.id,
                                j.0,
                                j.1,
                                j.2.text(),
                                j.3
                            ),
                            facts,
                        ));
                        fresh.push(g);
                    }
                    Err(e) => {
                        let _ = writeln!(
                            out,
                            "{}: {} against {}, side {}, seed {}: {e}",
                            r.id,
                            j.0,
                            j.1,
                            j.2.text(),
                            j.3
                        );
                    }
                }
            }
            let _ = writeln!(
                out,
                "{}: played {} games on {} threads in {secs} s",
                r.id,
                fresh.len(),
                cli.threads
            );
            let merged = merge(&stored, &fresh, &pairings(&r.games));
            std::fs::write(&stored_path, write_csv(&merged))
                .map_err(|e| format!("{}: {e}", stored_path.display()))?;
            merged
        };
        let c = compare_record(r, &ours);
        let text = report(&c);
        let txt = cli.out.join(format!("oracle-{}.txt", r.id));
        let full = format!(
            "{text}\nour games (winner@frame), by pairing and the row's side:\n{}",
            games_text(&ours)
        );
        std::fs::write(&txt, &full).map_err(|e| format!("{}: {e}", txt.display()))?;
        out.push_str(&text);
        let _ = writeln!(
            out,
            "wrote {} and {} (git-ignored; never commit them)",
            stored_path.display(),
            txt.display()
        );
    }
    let m0019 = record_paths(&cli.measurements).iter().any(|p| {
        p.file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with(M0019))
    });
    if m0019 && !other_houses.is_empty() {
        out.push_str(&other_houses_report(cli, &other_houses)?);
    }
    Ok(out)
}

/// M-0019's ID (the record of the placements at tick 0).
pub const M0019: &str = "M-0019";

/// M-0019's rows of the other houses over the games played: the verdicts
/// to the output and to `oracle-M-0019.txt` in the output directory, with
/// every game's facts (counts of the stock map's objects, so local).
fn other_houses_report(cli: &Cli, games: &[(String, OtherHouses)]) -> Result<String, String> {
    let v = other_houses_verdicts(games);
    let failing = v
        .iter()
        .filter(|(_, v)| matches!(v, crate::oracle_m0021::Verdict::Fail(_)))
        .count();
    let mut text = format!(
        "{M0019} rows (other houses at tick 0, {} games):\n",
        games.len()
    );
    for (row, verdict) in &v {
        let _ = writeln!(text, "  {row}: {}", verdict.text());
    }
    let _ = writeln!(text, "{M0019}: rows failing: {failing} of {}", v.len());
    let mut full = text.clone();
    for (name, f) in games {
        let _ = writeln!(full, "  {name}: {f:?}");
    }
    let txt = cli.out.join(format!("oracle-{M0019}.txt"));
    std::fs::write(&txt, &full).map_err(|e| format!("{}: {e}", txt.display()))?;
    let _ = writeln!(
        text,
        "wrote {} (git-ignored, with the stock map's counts; never commit it)",
        txt.display()
    );
    Ok(text)
}

// ---------------------------------------------------------------------------
// Comparison

/// Whether every game of a side has the same winner, and which.
fn unanimous(games: &[&Game], side: Side) -> Option<Winner> {
    let mut w = games
        .iter()
        .filter(|g| g.row_side == side)
        .map(|g| g.winner);
    let first = w.next()?;
    w.all(|x| x == first).then_some(first)
}

/// The types whose stock rows M-0017 does not cover: its replay leaves out
/// every game with one of them on either side, so a pairing with one does
/// not stand as stock, and no test runs against its rows (M-0001 and
/// M-0003, *Tolerance for tests* and Notes, *Stock rows*).
pub const UNCONFIRMED_TYPES: [&str; 2] = ["SREF", "V3"]; // measure: M-0017

/// Whether a pairing stands as stock and so is scored (measure: M-0017;
/// M-0001 and M-0003, *Tolerance for tests*).
pub fn confirmed(pairing: &(String, String)) -> bool {
    // measure: M-0017
    !UNCONFIRMED_TYPES
        .iter()
        .any(|t| *t == pairing.0 || *t == pairing.1)
}

/// Whether a stock game crashed and so has no result: the harness records
/// a crashed game as `none` with an end frame before the limit (measure:
/// M-0001, Procedure step 3). M-0017: the 7 MIND against E2 games of M-0001
/// are the only ones of the records with a `none` and an end frame short of
/// the limit sample.
pub fn crashed(g: &Game) -> bool {
    // measure: M-0001 (Procedure step 3), M-0017
    g.winner == Winner::None && g.end_frame < FRAME_LIMIT
}

/// The key of a game within its record.
/// A game's row, column, row side and seed: what names one game.
type GameKey<'a> = (&'a str, &'a str, Side, u32);

fn key(g: &Game) -> GameKey<'_> {
    (&g.row, &g.col, g.row_side, g.seed)
}

/// One pairing compared.
#[derive(Clone, Debug)]
pub struct PairingComparison {
    /// Row and column type.
    pub pairing: (String, String),
    /// Stock and our counts: row wins, column wins, none.
    pub counts: [[u32; 3]; 2],
    /// The two-proportion test of the row's win proportion.
    pub win: Option<stats::ProportionTest>,
    /// Per side (A, B): the stock winner when every stock game of the side
    /// had the same one, and whether ours reproduces it.
    pub same: [Option<(Winner, bool)>; 2],
    /// Our games of the pairing (n).
    pub n: usize,
    /// Whether the pairing stands as stock and is scored ([`confirmed`],
    /// or [`OracleRecord::scores`] in [`compare_record`]).
    pub confirmed: bool,
    /// Whether it is scored only because a stock replay record covers it
    /// (M-0016: the SREF and V3 pairings, which M-0017 leaves out).
    pub by_replay: bool,
    /// Games left out of the comparison because the stock game crashed
    /// (each counted once; ours of the same game are left out too).
    pub excluded: usize,
}

impl PairingComparison {
    /// Whether the win-proportion test passes at [`ALPHA`].
    pub fn win_pass(&self) -> bool {
        self.win.is_some_and(|t| t.p >= ALPHA)
    }

    /// Whether every side-wise unanimous stock outcome is reproduced.
    pub fn same_reproduced(&self) -> bool {
        self.same.iter().flatten().all(|&(_, ok)| ok)
    }

    /// Whether the pairing is within the record's tolerance: the win test
    /// passes and every side on which the stock games all gave the same
    /// outcome gives it in ours too (measure: M-0001, M-0002, M-0003,
    /// *Tolerance for tests*). A pairing that is not scored has no verdict;
    /// this then tells only what the tests would say.
    pub fn pass(&self) -> bool {
        self.win_pass() && self.same_reproduced()
    }

    /// Whether the pairing is scored and outside the tolerance.
    pub fn fails(&self) -> bool {
        self.confirmed && !self.pass()
    }

    /// How far off it is, for ranking: side-wise mismatches first, then the
    /// win-count difference.
    pub fn badness(&self) -> (usize, i64) {
        let mism = self.same.iter().flatten().filter(|(_, ok)| !ok).count();
        let dw = (i64::from(self.counts[0][0]) - i64::from(self.counts[1][0])).abs();
        (mism, dw)
    }
}

/// Compares the games of one pairing.
pub fn compare_pairing(
    pairing: &(String, String),
    stock: &[Game],
    ours: &[Game],
) -> PairingComparison {
    let of = |gs: &'_ [Game]| -> Vec<Game> {
        gs.iter()
            .filter(|g| g.row == pairing.0 && g.col == pairing.1)
            .cloned()
            .collect()
    };
    let (s, o) = (of(stock), of(ours));
    let count = |gs: &[Game]| {
        let mut c = [0u32; 3];
        for g in gs {
            c[match g.winner {
                Winner::Row => 0,
                Winner::Col => 1,
                Winner::None => 2,
            }] += 1;
        }
        c
    };
    let (cs, co) = (count(&s), count(&o));
    let sr: Vec<&Game> = s.iter().collect();
    let or: Vec<&Game> = o.iter().collect();
    let same = [Side::A, Side::B].map(|side| {
        unanimous(&sr, side).map(|w| {
            let ours_same = unanimous(&or, side) == Some(w);
            (w, ours_same)
        })
    });
    PairingComparison {
        pairing: pairing.clone(),
        counts: [cs, co],
        win: stats::two_proportion(cs[0], s.len() as u32, co[0], o.len() as u32),
        same,
        n: o.len(),
        confirmed: confirmed(pairing),
        by_replay: false,
        excluded: 0,
    }
}

/// A record compared with our games.
#[derive(Clone, Debug)]
pub struct RecordComparison {
    /// The record's ID.
    pub id: String,
    /// Every pairing of the record with games of ours, in record order.
    pub pairings: Vec<PairingComparison>,
    /// Pairings of the record with no game of ours.
    pub missing: Vec<(String, String)>,
    /// The stock rows a stock replay record put in place of archived ones
    /// (M-0016), and the types it covers.
    pub restated: (usize, Vec<String>),
}

impl RecordComparison {
    /// The scored pairings.
    pub fn scored(&self) -> Vec<&PairingComparison> {
        self.pairings.iter().filter(|p| p.confirmed).collect()
    }

    /// The scored pairings outside the tolerance.
    pub fn failing(&self) -> Vec<&PairingComparison> {
        self.pairings.iter().filter(|p| p.fails()).collect()
    }

    /// The scored pairings that do not reproduce a side-wise unanimous
    /// outcome.
    pub fn not_same(&self) -> Vec<&PairingComparison> {
        self.scored()
            .into_iter()
            .filter(|p| !p.same_reproduced())
            .collect()
    }
}

/// The stock games of a record that the comparison uses: all but the
/// crashed ones ([`crashed`]), with the keys of the crashed ones.
fn usable(record: &OracleRecord) -> (Vec<Game>, Vec<GameKey<'_>>) {
    // measure: M-0017 (crashed stock games have no result: they and our
    // games of the same row, column, side and seed are not compared)
    let dead: Vec<GameKey<'_>> = record
        .games
        .iter()
        .filter(|g| crashed(g))
        .map(key)
        .collect();
    let stock = record
        .games
        .iter()
        .filter(|g| !dead.contains(&key(g)))
        .cloned()
        .collect();
    (stock, dead)
}

/// Compares a record's games with ours.
pub fn compare_record(record: &OracleRecord, ours: &[Game]) -> RecordComparison {
    let mut out = RecordComparison {
        id: record.id.clone(),
        pairings: Vec::new(),
        missing: Vec::new(),
        restated: (record.restated, record.covered.clone()),
    };
    let (stock, dead) = usable(record);
    let ours: Vec<Game> = ours
        .iter()
        .filter(|g| !dead.contains(&key(g)))
        .cloned()
        .collect();
    for p in pairings(&record.games) {
        if ours.iter().any(|g| g.row == p.0 && g.col == p.1) {
            let mut c = compare_pairing(&p, &stock, &ours);
            c.excluded = dead.iter().filter(|k| k.0 == p.0 && k.1 == p.1).count();
            c.confirmed = record.scores(&p);
            // measure: M-0016 (SREF and V3 pairings scored through its stock rows)
            c.by_replay = c.confirmed && !confirmed(&p);
            out.pairings.push(c);
        } else {
            out.missing.push(p);
        }
    }
    out
}

/// The stock line of one pairing as the comparison uses it: the counts of
/// row wins, column wins and `none` over the usable games, and the games
/// per side (A, B). For MIND against E2 of M-0001 this is M-0017's restated
/// line.
pub fn stock_line(record: &OracleRecord, pairing: &(String, String)) -> ([u32; 3], [u32; 2]) {
    let (stock, _) = usable(record);
    let mut c = [0u32; 3];
    let mut sides = [0u32; 2];
    for g in stock
        .iter()
        .filter(|g| g.row == pairing.0 && g.col == pairing.1)
    {
        c[match g.winner {
            Winner::Row => 0,
            Winner::Col => 1,
            Winner::None => 2,
        }] += 1;
        sides[g.row_side.index()] += 1;
    }
    (c, sides)
}

fn p_text(p: Option<f64>) -> String {
    p.map_or("  n/a ".to_owned(), |p| format!("{p:.4}"))
}

/// The comparison as a table, with a summary and the failing pairings
/// (largest discrepancies first).
pub fn report(c: &RecordComparison) -> String {
    let mut s = String::new();
    let scored = c.scored();
    let _ = writeln!(
        s,
        "{}: {} pairings compared ({} scored, {} not scored), {} without games of ours",
        c.id,
        c.pairings.len(),
        scored.len(),
        c.pairings.len() - scored.len(),
        c.missing.len()
    );
    let _ = writeln!(
        s,
        "{:<7} {:<7} {:>2} | stock W/L/N  ours W/L/N | p(win) | same A / same B        | result",
        "row", "col", "n"
    );
    for p in &c.pairings {
        let same = |i: usize| match p.same[i] {
            None => "-".to_owned(),
            Some((w, ok)) => format!("{} {}", w.text(), if ok { "yes" } else { "NO" }),
        };
        let result = match (p.confirmed, p.pass()) {
            (true, true) => "pass".to_owned(),
            (true, false) => format!(
                "FAIL ({})",
                match (p.win_pass(), p.same_reproduced()) {
                    (false, false) => "win test, side outcome",
                    (false, true) => "win test",
                    _ => "side outcome",
                }
            ),
            // measure: M-0017 (SREF and V3 pairings do not stand as stock; no test runs against them)
            (false, ok) => format!(
                "not scored, SREF or V3 (M-0017); the tests would {}",
                if ok { "pass" } else { "fail" }
            ),
        };
        let mut notes = if p.excluded == 0 {
            String::new()
        } else {
            format!(" [{} crashed stock games left out, M-0017]", p.excluded)
        };
        if p.by_replay {
            notes.push_str(" [stock rows of M-0016]");
        }
        let _ = writeln!(
            s,
            "{:<7} {:<7} {:>2} | {:>2}/{:>2}/{:>2}     {:>2}/{:>2}/{:>2}   | {} | {:<9} / {:<9} | {result}{notes}",
            p.pairing.0,
            p.pairing.1,
            p.n,
            p.counts[0][0],
            p.counts[0][1],
            p.counts[0][2],
            p.counts[1][0],
            p.counts[1][1],
            p.counts[1][2],
            p_text(p.win.map(|t| t.p)),
            same(0),
            same(1),
        );
    }
    let count = |part: &[&PairingComparison], f: &dyn Fn(&PairingComparison) -> bool| {
        part.iter().filter(|p| f(p)).count()
    };
    let sides = |part: &[&PairingComparison]| -> (usize, usize) {
        (
            part.iter().map(|p| p.same.iter().flatten().count()).sum(),
            part.iter()
                .map(|p| p.same.iter().flatten().filter(|(_, ok)| *ok).count())
                .sum(),
        )
    };
    let _ = writeln!(s);
    let (all, kept) = sides(&scored);
    let _ = writeln!(
        s,
        "scored pairings (stock rows, M-0017): failing the win test (p < {ALPHA}) {}, failing the side outcome {}; scored pairings failing: {} of {}",
        count(&scored, &|p| !p.win_pass()),
        count(&scored, &|p| !p.same_reproduced()),
        c.failing().len(),
        scored.len()
    );
    let _ = writeln!(
        s,
        "side-wise unanimous stock outcomes of the scored pairings: {all}; reproduced {kept}"
    );
    if c.restated.0 > 0 {
        let replay: Vec<&PairingComparison> =
            scored.iter().copied().filter(|p| p.by_replay).collect();
        let _ = writeln!(
            s,
            "of these, scored through a stock replay ({} rows restated, covering {}, M-0016): {} pairings, failing {}",
            c.restated.0,
            c.restated.1.join(", "),
            replay.len(),
            count(&replay, &|p| !p.pass()),
        );
    }
    let unscored: Vec<&PairingComparison> = c.pairings.iter().filter(|p| !p.confirmed).collect();
    let (all, kept) = sides(&unscored);
    let _ = writeln!(
        s,
        "not scored (SREF or V3, M-0017): {} pairings; the tests would fail {}; side-wise unanimous stock outcomes {all}, reproduced {kept}",
        unscored.len(),
        count(&unscored, &|p| !p.pass()),
    );
    let mut ranked: Vec<&PairingComparison> = c.failing();
    ranked.sort_by_key(|p| std::cmp::Reverse(p.badness()));
    let _ = writeln!(s, "failing scored pairings, largest discrepancy first:");
    for p in ranked {
        let _ = writeln!(
            s,
            "  {} against {}: stock row wins {}/{}, ours {}/{}{}",
            p.pairing.0,
            p.pairing.1,
            p.counts[0][0],
            p.counts[0].iter().sum::<u32>(),
            p.counts[1][0],
            p.n,
            if p.win_pass() {
                " (win test passes; a side outcome differs)"
            } else {
                ""
            }
        );
    }
    for m in &c.missing {
        let _ = writeln!(s, "  not run: {} against {}", m.0, m.1);
    }
    s
}

/// Per pairing and side, our games in short form, for reading by eye.
pub fn games_text(games: &[Game]) -> String {
    let mut by: BTreeMap<(String, String, Side), Vec<&Game>> = BTreeMap::new();
    for g in games {
        by.entry((g.row.clone(), g.col.clone(), g.row_side))
            .or_default()
            .push(g);
    }
    let mut s = String::new();
    for ((r, c, side), gs) in by {
        let _ = write!(s, "{r} {c} {}:", side.text());
        for g in gs {
            let _ = write!(s, " {}@{}", g.winner.text(), g.end_frame);
        }
        s.push('\n');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_sizes_round_an_exact_half_to_even() {
        // spec: S-0216 claim 8 (Formulas: q when 2r < c, q + 1 when 2r > c, the even one when 2r = c)
        assert_eq!(round_half_even(9, 2), 4);
        assert_eq!(round_half_even(15, 2), 8);
        assert_eq!(round_half_even(10, 4), 2);
        assert_eq!(round_half_even(11, 4), 3);
        assert_eq!(round_half_even(13, 4), 3);
        assert_eq!(round_half_even(1, 3), 0);
    }

    #[test]
    fn the_map_line_is_read() {
        let md = "---\nid: M-test\n---\n\n## Setup\n\n- Map: the map `Some.mmx` (sha256 `ABCDEF0123456789abcdef0123456789abcdef0123456789abcdef0123456789`), around W.\n";
        let (f, h) = map_of(md).unwrap();
        assert_eq!(f, "Some.mmx");
        assert_eq!(
            h,
            "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
        );
        assert!(map_of("- Map: `x.mmx` no digest").is_err());
    }

    #[test]
    fn placement_follows_the_protocol() {
        let w = CellXY::new(50, 40);
        assert_eq!(cell_of(w, Side::A, 0), CellXY::new(44, 36));
        assert_eq!(cell_of(w, Side::A, 7), CellXY::new(44, 43));
        assert_eq!(cell_of(w, Side::A, 8), CellXY::new(43, 36));
        assert_eq!(cell_of(w, Side::B, 0), CellXY::new(56, 36));
        assert_eq!(cell_of(w, Side::B, 9), CellXY::new(57, 37));
    }

    /// The neighbours of a refused stated cell come in S-0216 claim 8's
    /// order, the eight neighbours only.
    #[test]
    fn neighbours_follow_claim_8s_order() {
        let c = CellXY::new(10, 20);
        let off: Vec<(i32, i32)> = neighbours(c).iter().map(|p| (p.x - 10, p.y - 20)).collect();
        // spec: S-0216 claim 8 (Moving on: (x − 1, y − 1), (x, y − 1), (x + 1, y − 1), (x − 1, y), (x + 1, y), (x − 1, y + 1), (x, y + 1), (x + 1, y + 1))
        assert_eq!(
            off,
            [
                (-1, -1),
                (0, -1),
                (1, -1),
                (-1, 0),
                (1, 0),
                (-1, 1),
                (0, 1),
                (1, 1)
            ]
        );
    }

    /// A stated cell with a unit on it does not take a vehicle: the vehicle
    /// goes to the first neighbour, in claim 8's order, that takes it
    /// (synthetic scenario).
    #[test]
    fn a_unit_on_the_stated_cell_moves_a_vehicle_to_the_first_neighbour_that_takes_it() {
        let mut w = glasnost_sim::testkit::combat_game(1);
        let (kind, ti, c) = w
            .objects
            .values()
            .find(|o| o.on_map && o.kind == Kind::Vehicle)
            .map(|o| (o.kind, o.type_index, o.pos.cell()))
            .unwrap();
        let id = glasnost_sim::objects::create(&mut w, kind, ti, None);
        // spec: S-0216 claim 8 (the take test: for a vehicle, no unit stands on the cell)
        assert!(refusal(&w, id, kind, c).is_some());
        let placed = placed_cell(&w, id, kind, c).unwrap();
        assert_ne!(placed, c);
        let first = neighbours(c).into_iter().find(|&p| takes(&w, id, kind, p));
        assert_eq!(Some(placed), first);
        assert!((placed.x - c.x).abs() <= 1 && (placed.y - c.y).abs() <= 1);
    }

    /// S-0216 claim 8's count rule (PT7 parts a and b, on test budgets and
    /// costs): an exact half to the even count for both kinds, the minimum
    /// 1 and the infantry cap.
    #[test]
    fn counts_round_an_exact_half_to_even_then_take_the_minimum_and_cap() {
        for kind in [Kind::Infantry, Kind::Vehicle] {
            // spec: S-0216 claim 8 (Formulas and PT7 part a: B / c = n + 1/2 gives n for n even, n + 1 for n odd, for infantry and vehicles alike)
            assert_eq!(count(kind, 9000, 2000), 4, "{kind:?}: 4.5");
            assert_eq!(count(kind, 9000, 1200), 8, "{kind:?}: 7.5");
            assert_eq!(count(kind, 9000, 3600), 2, "{kind:?}: 2.5");
            assert_eq!(count(kind, 9000, 6000), 2, "{kind:?}: 1.5");
            assert_eq!(count(kind, 9000, 1999), 5, "{kind:?}: above the half");
            assert_eq!(count(kind, 9000, 2001), 4, "{kind:?}: below the half");
            // spec: S-0216 claim 8 (PT7 part b: below the records' minimum gives the minimum)
            assert_eq!(count(kind, 9000, 20000), 1, "{kind:?}: 0.45");
        }
        // spec: S-0216 claim 8 (PT7 part b: an infantry count above the cap gives the cap; a vehicle count has none)
        assert_eq!(count(Kind::Infantry, 9000, 100), i64::from(INFANTRY_CAP));
        assert_eq!(count(Kind::Vehicle, 9000, 100), 90);
    }

    fn record_of(id: &str, games: Vec<Game>) -> OracleRecord {
        OracleRecord {
            id: id.into(),
            path: PathBuf::new(),
            map_file: String::new(),
            map_sha256: String::new(),
            games,
            covered: Vec::new(),
            restated: 0,
        }
    }

    fn g(row_side: Side, seed: u32, winner: Winner) -> Game {
        Game {
            row: "R".into(),
            col: "C".into(),
            row_side,
            seed,
            winner,
            end_frame: 304,
        }
    }

    fn twelve(f: impl Fn(Side, u32) -> Winner) -> Vec<Game> {
        let mut out = Vec::new();
        for side in [Side::A, Side::B] {
            for seed in SEEDS {
                out.push(g(side, seed, f(side, seed)));
            }
        }
        out
    }

    #[test]
    fn csv_has_the_six_columns_of_the_restated_records() {
        // measure: M-0001, M-0002, M-0003 (per-game data: row, col, row_side, seed, winner, end_frame)
        assert_eq!(CSV_HEADER, "row,col,row_side,seed,winner,end_frame");
        let games = twelve(|_, s| if s == 3 { Winner::None } else { Winner::Row });
        let text = write_csv(&games);
        assert_eq!(parse_csv(&text).unwrap(), games);
        assert!(check_shape(&games).is_ok());
        assert!(check_shape(&games[1..]).is_err());
        assert!(parse_csv("row,col\n").is_err());
        assert!(parse_csv(&format!("{CSV_HEADER}\nR,C,X,0,row,304\n")).is_err());
        // The old form with the two share columns is refused.
        assert!(
            parse_csv(&format!(
                "{CSV_HEADER},row_left,col_left\nR,C,A,0,row,304,1,0\n"
            ))
            .is_err()
        );
        assert!(parse_csv(&format!("{CSV_HEADER}\nR,C,A,0,row,304,1,0\n")).is_err());
    }

    #[test]
    fn the_win_test_and_the_side_outcome_make_the_verdict() {
        // measure: M-0001 (Tolerance for tests: the win-proportion test, and the same outcome where every seed of a side gave it)
        let stock = twelve(|_, _| Winner::Row);
        let c = compare_pairing(&("R".into(), "C".into()), &stock, &stock);
        assert!(c.pass() && c.win_pass() && c.same_reproduced(), "{c:?}");
        assert_eq!(
            c.same,
            [Some((Winner::Row, true)), Some((Winner::Row, true))]
        );
        let reversed = twelve(|_, _| Winner::Col);
        let c = compare_pairing(&("R".into(), "C".into()), &stock, &reversed);
        assert!(!c.pass() && !c.win_pass() && !c.same_reproduced());
        assert!(c.win.unwrap().p < ALPHA);
        // A side-wise difference that the pooled test does not catch: stock
        // 6/6 on side A, ours 5/6 there. It fails on the side outcome alone.
        let near = twelve(|side, s| {
            if side == Side::A && s == 0 {
                Winner::Col
            } else {
                Winner::Row
            }
        });
        let c = compare_pairing(&("R".into(), "C".into()), &stock, &near);
        assert!(c.win_pass() && !c.same_reproduced() && !c.pass());
        assert!(c.fails());
        assert_eq!(
            c.same,
            [Some((Winner::Row, false)), Some((Winner::Row, true))]
        );
        let r = report(&RecordComparison {
            id: "M-test".into(),
            pairings: vec![c],
            missing: Vec::new(),
            restated: (0, Vec::new()),
        });
        assert!(r.contains("FAIL (side outcome)"), "{r}");
        assert!(r.contains("scored pairings failing: 1 of 1"), "{r}");
        // A stock side with mixed outcomes asks only the win test.
        let mixed = twelve(|_, s| if s < 3 { Winner::Row } else { Winner::Col });
        let ours = twelve(|_, s| if s < 2 { Winner::Row } else { Winner::Col });
        let c = compare_pairing(&("R".into(), "C".into()), &mixed, &ours);
        assert_eq!(c.same, [None, None]);
        assert!(c.pass());
    }

    #[test]
    fn crashed_stock_games_are_left_out_of_both_sides() {
        // measure: M-0001 (Procedure step 3: a crashed game is `none` with an end frame before the limit), M-0017
        // Stock: side A has no result in any game (a crash before the limit
        // sample), side B ordinary. Ours: side A won by the row.
        let mut stock = twelve(|side, _| {
            if side == Side::A {
                Winner::None
            } else {
                Winner::Row
            }
        });
        // A stock `none` at the limit sample is a real draw, not a crash.
        stock[6].winner = Winner::None;
        stock[6].end_frame = FRAME_LIMIT + 80;
        assert_eq!(stock.iter().filter(|g| crashed(g)).count(), 6);
        let ours = twelve(|_, _| Winner::Row);
        let record = record_of("M-test", stock);
        assert_eq!(
            stock_line(&record, &("R".into(), "C".into())),
            ([5, 0, 1], [0, 6])
        );
        let c = compare_record(&record, &ours);
        let p = &c.pairings[0];
        assert_eq!(p.n, 6, "our side-A games are not counted");
        assert_eq!(p.excluded, 6);
        assert_eq!(p.counts[0], [5, 0, 1], "stock: the draw stays");
        assert_eq!(p.counts[1][0], 6);
        assert_eq!(p.same[0], None, "no stock game on side A");
        assert!(report(&c).contains("crashed stock games left out"));
    }

    #[test]
    fn sref_and_v3_pairings_are_shown_but_not_scored() {
        // measure: M-0017; M-0001 and M-0003 (Tolerance for tests: no test runs against the SREF and V3 rows)
        let pair = |a: &str, b: &str| (a.to_owned(), b.to_owned());
        assert!(confirmed(&pair("MIND", "E2")));
        assert!(!confirmed(&pair("SREF", "E2")));
        assert!(!confirmed(&pair("E1", "V3")));
        let mk = |row: &str, w: Winner| {
            let mut v = twelve(|_, _| w);
            for x in &mut v {
                x.row = row.into();
            }
            v
        };
        let mut stock = mk("SREF", Winner::Row);
        stock.extend(mk("E2", Winner::Row));
        stock.extend(mk("V3", Winner::Row));
        // Ours: SREF reversed (would fail, not scored), E2 reversed (fails),
        // V3 the same (would pass, not scored).
        let mut ours = mk("SREF", Winner::Col);
        ours.extend(mk("E2", Winner::Col));
        ours.extend(mk("V3", Winner::Row));
        let record = record_of("M-test", stock);
        let c = compare_record(&record, &ours);
        assert!(!c.pairings[0].confirmed && !c.pairings[0].fails());
        assert!(c.pairings[1].confirmed && c.pairings[1].fails());
        assert_eq!(c.scored().len(), 1);
        assert_eq!(c.failing().len(), 1);
        assert_eq!(c.not_same().len(), 1);
        let r = report(&c);
        assert!(r.contains("scored pairings failing: 1 of 1"), "{r}");
        assert!(
            r.contains("not scored (SREF or V3, M-0017): 2 pairings; the tests would fail 1"),
            "{r}"
        );
        assert!(
            r.contains("not scored, SREF or V3 (M-0017); the tests would fail"),
            "{r}"
        );
        assert!(
            !r.lines()
                .skip_while(|l| !l.starts_with("failing scored pairings"))
                .any(|l| l.contains("SREF against")),
            "{r}"
        );
    }

    #[test]
    fn merging_replaces_only_the_redone_pairings() {
        let mut a = twelve(|_, _| Winner::Row);
        let mut b = twelve(|_, _| Winner::Row);
        for x in &mut b {
            x.col = "D".into();
        }
        a.extend(b.clone());
        let mut fresh = twelve(|_, _| Winner::Col);
        for x in &mut fresh {
            x.col = "D".into();
        }
        let order = pairings(&a);
        let m = merge(&a, &fresh, &order);
        assert_eq!(m.len(), 24);
        assert!(m[..12].iter().all(|x| x.winner == Winner::Row));
        assert!(m[12..].iter().all(|x| x.winner == Winner::Col));
    }

    #[test]
    fn spread_option_names_one_pairing_and_side() {
        let args = |v: &[&str]| v.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        let c = parse_cli(&args(&["--record", "M-0003", "--spread", "MGTK,APOC,B"])).unwrap();
        assert_eq!(c.spread, Some(("MGTK".into(), "APOC".into(), Side::B)));
        assert!(parse_cli(&args(&["--record", "M-0003", "--spread", "MGTK,APOC"])).is_err());
    }

    #[test]
    fn trace_options_name_one_game_with_or_without_detail() {
        let args = |v: &[&str]| v.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        let c = parse_cli(&args(&["--record", "M-0001", "--trace", "CAOS,BFRT,A,1"])).unwrap();
        assert_eq!(
            c.trace,
            Some((("CAOS".into(), "BFRT".into(), Side::A, 1), true))
        );
        let c = parse_cli(&args(&[
            "--record",
            "M-0001",
            "--trace-summary",
            "CAOS,BFRT,b,0",
        ]))
        .unwrap();
        assert_eq!(
            c.trace,
            Some((("CAOS".into(), "BFRT".into(), Side::B, 0), false))
        );
        assert!(parse_cli(&args(&["--record", "M-0001", "--trace", "CAOS,BFRT,A"])).is_err());
    }

    #[test]
    fn seed_range_option_names_one_side_and_a_seed_range() {
        let args = |v: &[&str]| v.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        let c = parse_cli(&args(&[
            "--record",
            "M-0001",
            "--seed-range",
            "DISK,FV,b,0-63",
        ]))
        .unwrap();
        assert_eq!(
            c.seed_range,
            Some(("DISK".into(), "FV".into(), Side::B, 0..=63))
        );
        for bad in ["DISK,FV,B", "DISK,FV,C,0-5", "DISK,FV,A,5-0", "DISK,FV,A,0"] {
            assert!(parse_cli(&args(&["--record", "M-0001", "--seed-range", bad])).is_err());
        }
    }
}
