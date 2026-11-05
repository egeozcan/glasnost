//! The arena's harness for M-0020 (M8 A3 intake 101): the unit arena's
//! YURI against GGI and SHK and CAOS against E1, read from our engine.
//!
//! **Part 1** plays the record's six games at seed 0 (M-0002's YURI
//! against GGI and SHK, M-0001's CAOS against E1, each with the row type on
//! side A and on side B) through [`oracle::play_with`], the arena's own
//! game, with a recorder that reads the world and each tick's effects after
//! every step and changes nothing. It reads where the infantry stand at
//! tick 0, the draws before tick 0's update (from our placement log,
//! [`oracle::Placement`]: our stream keeps raw-value counts, not a log of
//! draws), and the record's local values of Procedures 6 and 7.
//!
//! **Part 2** plays YURI against GGI and SHK, YURI on side A, at seeds 0 to
//! 11, once as set up (arm S) and once with every side-A YURI moved before
//! the first step to one standing position of its cell (arms C, T, R, L,
//! B), and two same-spot controls at seed 0 ([`move_to_position`]).
//!
//! *Tick n* is the record's: our step run while the tick number is *n*
//! (S-0201 claim 1: the tick number is 0 at game start and grows by 1 at the
//! end of every tick), so *after tick n's update* is our state after that
//! step, and an effect seen after it belongs to tick *n*.
//!
//! The verdicts go to the arena's output; the local values (ticks, counts
//! of the stock map's content) only to the git-ignored `oracle-M-0020.txt`,
//! beside the record's from `measurements/local/M-0020.json`.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::sync::atomic::{AtomicUsize, Ordering};

use glasnost_map::CellXY;
use glasnost_sim::cells::position;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::rng::Rng;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, HouseRole, ObjectId, Pos, Target, World};

use crate::oracle::{self, Arena, Game, Ground, Placement, Side, Winner};
use crate::oracle_m0021::Verdict;

/// The record.
pub const ID: &str = "M-0020";

/// The record whose map and ground the games play on (measure: M-0020,
/// Setup: map, houses and settings as in M-0001 to M-0003).
pub const MAP_RECORD: &str = "M-0001";

/// Part 1's games: row type, column type, the row type's side, and the
/// record whose archived game it replays.
pub const PART1: [(&str, &str, Side, &str); 6] = [
    ("YURI", "GGI", Side::A, "M-0002"), // measure: M-0020 (Setup, Part 1 games)
    ("YURI", "GGI", Side::B, "M-0002"), // measure: M-0020 (Setup, Part 1 games)
    ("YURI", "SHK", Side::A, "M-0002"), // measure: M-0020 (Setup, Part 1 games)
    ("YURI", "SHK", Side::B, "M-0002"), // measure: M-0020 (Setup, Part 1 games)
    ("CAOS", "E1", Side::A, "M-0001"),  // measure: M-0020 (Setup, Part 1 games)
    ("CAOS", "E1", Side::B, "M-0001"),  // measure: M-0020 (Setup, Part 1 games)
];

/// Part 1's seed number (measure: M-0020, Setup: seed 0 of M-0001's naming).
pub const PART1_SEED: u32 = 0;

/// Part 2's pairings, YURI on side A (measure: M-0020, Setup, Part 2 games).
pub const PART2: [(&str, &str); 2] = [("YURI", "GGI"), ("YURI", "SHK")];

/// Part 2's seed numbers (measure: M-0020, Setup: seeds 0 to 11).
pub const PART2_SEEDS: std::ops::Range<u32> = 0..12;

/// Part 2's seeds that are the archived games' (measure: M-0020, Setup:
/// seeds 0 to 5; Procedure 10).
pub const ARCHIVED_SEEDS: std::ops::Range<u32> = 0..6;

/// The raw values from the seeding to the start of the run (measure:
/// M-0020, Result, *Part 1: draws before tick 0*: 706 in each game).
pub const FIRST_RAW: u64 = 706;

/// An arm of part 2: where every side-A YURI stands before the first step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Arm {
    /// The set-up as is.
    S,
    /// Each at the position it already stands at (Procedure 10's
    /// same-spot run).
    Same,
    /// Each at this standing position of its cell (S-0208 table P).
    At(u8),
}

/// Part 2's six arms by the record's letters (measure: M-0020, Setup, Part
/// 2's arms; spec: S-0208 table P, the five standing positions).
pub const ARMS: [(char, Arm); 6] = [
    ('S', Arm::S),
    ('C', Arm::At(position::CENTRE)),
    ('T', Arm::At(position::TOP)),
    ('R', Arm::At(position::RIGHT)),
    ('L', Arm::At(position::LEFT)),
    ('B', Arm::At(position::BOTTOM)),
];

/// The record's part 2 table: pairing, arm, games, won by side A, won by
/// side B, no winner (measure: M-0020, Result, *Part 2*).
pub const WINNERS: [(&str, &str, char, u32, u32, u32, u32); 12] = [
    ("YURI", "GGI", 'S', 12, 12, 0, 0), // measure: M-0020 (Result, Part 2 table)
    ("YURI", "GGI", 'C', 12, 12, 0, 0), // measure: M-0020 (Result, Part 2 table)
    ("YURI", "GGI", 'T', 12, 12, 0, 0), // measure: M-0020 (Result, Part 2 table)
    ("YURI", "GGI", 'R', 12, 12, 0, 0), // measure: M-0020 (Result, Part 2 table)
    ("YURI", "GGI", 'L', 12, 12, 0, 0), // measure: M-0020 (Result, Part 2 table)
    ("YURI", "GGI", 'B', 12, 12, 0, 0), // measure: M-0020 (Result, Part 2 table)
    ("YURI", "SHK", 'S', 12, 12, 0, 0), // measure: M-0020 (Result, Part 2 table)
    ("YURI", "SHK", 'C', 12, 12, 0, 0), // measure: M-0020 (Result, Part 2 table)
    ("YURI", "SHK", 'T', 12, 12, 0, 0), // measure: M-0020 (Result, Part 2 table)
    ("YURI", "SHK", 'R', 12, 12, 0, 0), // measure: M-0020 (Result, Part 2 table)
    ("YURI", "SHK", 'L', 12, 12, 0, 0), // measure: M-0020 (Result, Part 2 table)
    ("YURI", "SHK", 'B', 12, 12, 0, 0), // measure: M-0020 (Result, Part 2 table)
];

/// Row R*k* of S-0208 table O for a desired point of class centre: the
/// positions tried, left to right (spec: S-0208 table O, R0 and R1 right,
/// left, bottom; R2 left, bottom, right; R3 bottom, right, left). Restated
/// here so that the harness does not read our engine's own table.
pub fn table_o_row(k: i32) -> [u8; 3] {
    use position::{BOTTOM, LEFT, RIGHT};
    match k {
        0 | 1 => [RIGHT, LEFT, BOTTOM], // spec: S-0208 table O (R0, R1)
        2 => [LEFT, BOTTOM, RIGHT],     // spec: S-0208 table O (R2)
        _ => [BOTTOM, RIGHT, LEFT],     // spec: S-0208 table O (R3)
    }
}

/// The standing position whose table P point is exactly `(u, v)`, if any
/// (spec: S-0208 table P: centre 128,128; top 64,64; right 192,64; left
/// 64,192; bottom 192,192).
pub fn point_position(u: i32, v: i32) -> Option<u8> {
    [
        position::CENTRE,
        position::TOP,
        position::RIGHT,
        position::LEFT,
        position::BOTTOM,
    ]
    .into_iter()
    .find(|&p| glasnost_sim::walk::standing_point(p) == (u, v))
}

/// The record's letter of a standing position (S-0208 table P).
pub fn position_letter(p: u8) -> char {
    match p {
        position::CENTRE => 'C',
        position::TOP => 'T',
        position::RIGHT => 'R',
        position::LEFT => 'L',
        position::BOTTOM => 'B',
        _ => '?',
    }
}

/// Moves infantry unit `id` within its cell to standing position `p`
/// (M-0020's moved arms: before tick 0's update every side-A YURI stands
/// at one named standing position of the cell it stands on). Its position
/// becomes the point of table P in the same cell (spec: S-0208 table P),
/// on the ground there (`World::surface_at`, as a placement's position,
/// S-0216 claim 2), and its mark in the cell's ground record moves to the
/// slot of `p` with its sequence number kept, so that moving a unit to the
/// position it stands at changes nothing. When another unit's mark holds
/// that slot the unit is still moved and its own mark stays where it was:
/// returns `false` then (the tool counts these collisions), `true`
/// otherwise. The tool's set-up, not a placement of our engine; no draw.
pub fn move_to_position(w: &mut World, id: ObjectId, p: u8) -> Result<bool, String> {
    let o = w.obj(id).ok_or("no such object")?;
    if o.kind != Kind::Infantry || !o.on_map {
        return Err(format!("#{} is not an infantry unit on the map", id.0));
    }
    let (cell, layer) = (o.pos.cell(), o.layer);
    let (u, v) = glasnost_sim::walk::standing_point(p);
    let (x, y) = (256 * cell.x + u, 256 * cell.y + v); // spec: S-0204 claim 2 (a position within the cell, 0 … 255 per axis)
    let z = w.surface_at(x, y);
    let occ = w
        .cells
        .occ_mut(cell, layer)
        .ok_or("the unit's cell has no record")?;
    let mine = |m: &Option<glasnost_sim::cells::Mark>| matches!(m, Some(glasnost_sim::cells::Mark { by: glasnost_sim::cells::Marker::Infantry(i, _), .. }) if *i == id);
    let from = occ.marks.iter().position(mine);
    let to = usize::from(p);
    let free = occ.marks.get(to).is_some_and(|m| m.is_none() || mine(m));
    if free && let Some(f) = from {
        let m = occ.marks[f].take();
        occ.marks[to] = m;
    }
    if let Some(ob) = w.obj_mut(id) {
        ob.pos = Pos::new(x, y, z);
    }
    Ok(free)
}

/// An infantry unit of the two players' houses on the map at tick 0:
/// its identifier, cell, and position in the cell (Procedure 2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stand {
    /// The unit.
    pub id: ObjectId,
    /// Its cell (position ÷ 256).
    pub cell: CellXY,
    /// Its position in the cell (position mod 256).
    pub point: (i32, i32),
}

/// The draws before the first step as our placement log gives them
/// (Procedures 3 and 4): S-0216 claim 3's pattern for each try.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Draws {
    /// The raw values the stream had taken at the first creation.
    pub first_raw: u64,
    /// The raw values it had taken before the first step.
    pub before_step: u64,
    /// The raw values of the pattern (every try's creation draws and its
    /// placement or refused-cell draw).
    pub pattern_raw: u64,
    /// Whether the tries follow one another with no raw value between.
    pub contiguous: bool,
    /// The tries whose creation took other than one raw value per
    /// creation number, or whose after-creation draws were not the pattern's.
    pub off_pattern: usize,
    /// The tries (refused or taking).
    pub tries: usize,
    /// The refused tries (the record's objects off the map at tick 0).
    pub refused: usize,
}

/// Checks the placement log against S-0216 claim 3's pattern: each
/// creation takes one raw value (S-0202 claim 10, one creation draw; S-0216
/// claim 8: a try's creation set, its own then its objects'), then an
/// infantry try takes one R(0, 3), one raw value (S-0216 claims 3 and 8: on
/// a usable cell, and the refused cell's unused draw), a vehicle try one
/// R(0, 29) only for a type with a visceroid flag on the cell that takes it
/// (S-0216 claim 3), none otherwise. `visceroid` gives each placement's
/// type's flag.
pub fn draw_check(
    placements: &[Placement],
    visceroid: &[bool],
    first_raw: u64,
    before_step: u64,
) -> Draws {
    let mut d = Draws {
        first_raw,
        before_step,
        contiguous: true,
        ..Default::default()
    };
    let mut at = first_raw;
    for (p, &visc) in placements.iter().zip(visceroid) {
        for t in &p.tries {
            d.tries += 1;
            d.refused += usize::from(!t.took);
            if t.start != at {
                d.contiguous = false;
            }
            // spec: S-0202 claim 10 (one creation draw, one raw value, per object created); S-0216 claim 8 (the try's creation set)
            let after = match p.kind {
                Kind::Infantry => 1, // spec: S-0216 claims 3 and 8 (an infantry try's one R(0, 3), on the taking cell or the refused one)
                _ if t.took && visc => 1, // spec: S-0216 claim 3 (one R(0, 29) for a visceroid type, once accepted)
                _ => 0,                   // spec: S-0216 claim 3 (any other vehicle: no draw)
            };
            if t.creation_raw != u64::from(t.created) || t.after_raw != after {
                d.off_pattern += 1;
            }
            d.pattern_raw += t.creation_raw + t.after_raw;
            at = t.start + t.creation_raw + t.after_raw;
        }
    }
    if at != before_step {
        d.contiguous = false;
    }
    d
}

/// Procedure 5's expected position for each placed infantry unit on the
/// map at tick 0: the result *k* of its taking try's R(0, 3), read from a
/// replay of the scenario stream seeded as the game's (`stream_seed`,
/// spec: S-0216 claim 8, 1000 + s), and the first position of row R*k* of
/// table O ([`table_o_row`]) that no lower-identifier infantry unit holds on
/// the same cell at tick 0 (`None`: none free). Our log pairs each try with
/// its own R(0, 3), which is the record's *n*-th R(0, 3) for the *n*-th
/// infantry object, refused tries counted as its objects off the map.
/// An error when the log is not contiguous or a try is off the pattern.
pub fn expected_positions(
    placements: &[Placement],
    stream_seed: u32,
    first_raw: u64,
    half: bool,
    stands: &[Stand],
) -> Result<BTreeMap<ObjectId, Option<u8>>, String> {
    let mut r = Rng::new(stream_seed);
    for _ in 0..first_raw {
        r.raw();
    }
    let mut k_of: BTreeMap<ObjectId, i32> = BTreeMap::new();
    for p in placements {
        for t in &p.tries {
            if r.taken() != t.start {
                return Err(format!("unit #{}: the log is not contiguous", p.id.0));
            }
            for _ in 0..t.creation_raw {
                r.raw();
            }
            if p.kind == Kind::Infantry {
                if t.after_raw != 1 {
                    return Err(format!("unit #{}: no single R(0, 3)", p.id.0));
                }
                let k = r.ranged(0, 3, half); // spec: S-0216 claim 2 (one draw R(0, 3)); S-0202 claim 4 (a ranged draw)
                if t.took {
                    k_of.insert(p.id, k);
                }
            } else {
                for _ in 0..t.after_raw {
                    r.raw();
                }
            }
        }
    }
    let mut sorted: Vec<&Stand> = stands.iter().collect();
    sorted.sort_by_key(|s| s.id);
    let mut out = BTreeMap::new();
    for (i, s) in sorted.iter().enumerate() {
        let Some(&k) = k_of.get(&s.id) else {
            continue;
        };
        let held: BTreeSet<u8> = sorted[..i]
            .iter()
            .filter(|o| o.cell == s.cell)
            .filter_map(|o| point_position(o.point.0, o.point.1))
            .collect();
        out.insert(s.id, table_o_row(k).into_iter().find(|p| !held.contains(p)));
    }
    Ok(out)
}

/// The end of the drones in a CAOS game (Procedure 7).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CaosEnd {
    /// The first tick after tick 0 with no CAOS on the map with health
    /// above 0.
    pub tick: i32,
    /// The E1 placed; at that tick on the map, off the map, berserk among
    /// those on the map, berserk at any tick up to then.
    pub e1: (usize, usize, usize, usize, usize),
    /// The E1 killed up to then, and of those by an E1.
    pub e1_killed: (usize, usize),
    /// The CAOS kills by the attacker's kind.
    pub caos_kills: BTreeMap<String, usize>,
}

/// A unit under mind control: its group's side at tick 0, its owner.
pub type Captured = (Option<usize>, Option<u8>);

/// A bunker's change of owner: tick, bunker, new owner.
pub type OwnerChange = (i32, u32, String);

/// The running counts of Procedure 7 up to the drones' end.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CaosCount {
    /// The E1 berserk at any tick so far.
    pub berserk_ever: BTreeSet<ObjectId>,
    /// Each bunker's owner at the last tick read.
    pub bunker_owners: BTreeMap<ObjectId, Option<HouseId>>,
    /// The CAOS kills by the attacker's kind.
    pub kills: BTreeMap<String, usize>,
    /// The E1 killed, and of those by an E1.
    pub e1_killed: (usize, usize),
}

/// What the recorder reads from one game (Procedures 2 to 7, 9 and 10).
#[derive(Clone, Debug, Default)]
pub struct Run {
    /// The game.
    pub game: Option<Game>,
    /// The houses of side A and side B (two, once set up).
    pub houses: Vec<HouseId>,
    /// Each side's group, in creation order.
    pub units: [Vec<ObjectId>; 2],
    /// The type of each side's group.
    pub types: [String; 2],
    /// The placement log.
    pub placements: Vec<Placement>,
    /// Each placement's type's visceroid flag.
    pub visceroid: Vec<bool>,
    /// The raw values at the first creation.
    pub first_raw: u64,
    /// The stream before the first step (after a moved arm's moves).
    pub rng_before: Option<Rng>,
    /// The ranged-draw quirk of the game's rules.
    pub half: bool,
    /// The side-A YURI moved, and the moves whose slot another unit held.
    pub moved: (usize, usize),
    /// The two players' infantry on the map at tick 0.
    pub stands: Vec<Stand>,
    /// Side A's group at tick 0: on the map, and at the arm's point.
    pub at_arm: (usize, usize),
    /// Per house, the first tick after whose update any of its objects
    /// holds a target, and the targets then by kind.
    pub first_targets: [Option<(i32, BTreeMap<String, usize>)>; 2],
    /// Per house, its first shot: tick, firer's type, target's kind.
    pub first_shots: [Option<(i32, String, String)>; 2],
    /// The first shot of each firer kind.
    pub first_shot_by_firer: BTreeMap<String, i32>,
    /// The first shot of the row type (the YURI games' first YURI shot).
    pub first_row_shot: Option<i32>,
    /// The first tick after whose update any object is under mind
    /// control, and each then: its group's side at tick 0, its owner.
    pub first_capture: Option<(i32, Vec<Captured>)>,
    /// The drones' end (CAOS games).
    pub caos: Option<CaosEnd>,
    /// The first E1 berserk, the first E1 off the map, and each change of
    /// a bunker's owner (tick, bunker, new owner) up to the drones' end.
    pub caos_events: (Option<i32>, Option<i32>, Vec<OwnerChange>),
    /// The CAOS games' running counts up to the drones' end.
    pub caos_count: CaosCount,
    /// Every hit: tick, object, attacker, health before and after.
    pub hits: Vec<(i32, u32, Option<u32>, i32, i32)>,
    /// Every shot (recorded for the same-spot comparison): tick, firer,
    /// target.
    pub shots: Vec<(i32, u32, Target)>,
    /// The state hash and the stream's raw values after each step
    /// (same-spot comparison only).
    pub states: Vec<(String, u64)>,
}

/// Which side's group at tick 0 holds `id`.
fn group_of(run: &Run, id: ObjectId) -> Option<usize> {
    (0..2).find(|&s| run.units[s].contains(&id))
}

/// A house in the report's words: side A, side B, neutral, or its number.
fn house_text(w: &World, run: &Run, h: Option<HouseId>) -> String {
    match h {
        None => "no house".into(),
        Some(h) if h == run.houses[0] => "side A".into(),
        Some(h) if h == run.houses[1] => "side B".into(),
        Some(h) => match w.houses.get(usize::from(h.0)).map(|x| x.role) {
            Some(HouseRole::Neutral) => "neutral".into(),
            _ => format!("house {}", h.0),
        },
    }
}

/// An object's type name, or its kind.
fn type_name(w: &World, id: ObjectId) -> String {
    let Some(o) = w.obj(id) else {
        return "gone".into();
    };
    w.rules()
        .unit(o.kind, o.type_index)
        .map(|u| u.name.clone())
        .unwrap_or_else(|| format!("{:?}", o.kind))
}

/// An object as the record names targets and attackers: its group's type
/// and side, with its owner when another house has it, or a bunker and its
/// holder.
fn label(w: &World, run: &Run, id: ObjectId) -> String {
    if let Some(s) = group_of(run, id) {
        let side = ["A", "B"][s];
        let owner = w.obj(id).and_then(|o| o.owner);
        return if owner == Some(run.houses[s]) {
            format!("{} of side {side}", run.types[s])
        } else {
            format!(
                "{} placed for side {side}, owned by {}",
                run.types[s],
                house_text(w, run, owner)
            )
        };
    }
    if glasnost_sim::structures::bunker::is_bunker(w, id) {
        let owner = w.obj(id).and_then(|o| o.owner);
        return format!("bunker held by {}", house_text(w, run, owner));
    }
    let owner = w.obj(id).and_then(|o| o.owner);
    format!("{} of {}", type_name(w, id), house_text(w, run, owner))
}

/// A target in the record's words.
fn target_label(w: &World, run: &Run, t: Target) -> String {
    match t {
        Target::Object(id) => label(w, run, id),
        Target::Cell(_) => "a cell".into(),
    }
}

/// The recorder's reading after one step (read only).
fn observe(w: &World, effects: &[Effect], run: &mut Run, keep_states: bool) {
    let t = w.tick - 1; // spec: S-0201 claim 1 (the step just run was the tick numbered w.tick − 1)
    if t == 0 {
        // measure: M-0020 (Procedure 2: at tick 0, after its update, every infantry unit of the two players' houses on the map; cell = position ÷ 256, position in the cell = position mod 256)
        run.stands = w
            .objects
            .values()
            .filter(|o| {
                o.kind == Kind::Infantry
                    && o.on_map
                    && o.owner.is_some_and(|h| run.houses.contains(&h))
            })
            .map(|o| Stand {
                id: o.id,
                cell: o.pos.cell(),
                point: (o.pos.x.rem_euclid(256), o.pos.y.rem_euclid(256)),
            })
            .collect();
    }
    // measure: M-0020 (Procedure 6: per house, the first tick after whose update any of its units holds a target, and the kind of each target then)
    for s in 0..2 {
        if run.first_targets[s].is_some() {
            continue;
        }
        let mut m: BTreeMap<String, usize> = BTreeMap::new();
        for o in w.objects.values() {
            if o.owner == Some(run.houses[s])
                && let Some(tg) = o.target
            {
                *m.entry(target_label(w, run, tg)).or_default() += 1;
            }
        }
        if !m.is_empty() {
            run.first_targets[s] = Some((t, m));
        }
    }
    let caos_side = (run.types[0] == "CAOS")
        .then_some(0)
        .or((run.types[1] == "CAOS").then_some(1));
    let caos_open = caos_side.is_some() && run.caos.is_none();
    let mut kills: Vec<(ObjectId, Option<ObjectId>)> = Vec::new();
    for e in effects {
        match e {
            Effect::Shot { firer, target, .. } => {
                let owner = w.obj(*firer).and_then(|o| o.owner);
                let name = type_name(w, *firer);
                let firer_label = if glasnost_sim::structures::bunker::is_bunker(w, *firer) {
                    label(w, run, *firer)
                } else {
                    name.clone()
                };
                // measure: M-0020 (Procedure 6: each house's first shot; the first YURI shot)
                if let Some(s) = (0..2).find(|&s| owner == Some(run.houses[s]))
                    && run.first_shots[s].is_none()
                {
                    run.first_shots[s] = Some((t, name.clone(), target_label(w, run, *target)));
                }
                run.first_shot_by_firer.entry(firer_label).or_insert(t);
                let row = &run.types[usize::from(run.types[1] == "YURI")];
                if run.first_row_shot.is_none() && name == *row {
                    run.first_row_shot = Some(t);
                }
                if keep_states {
                    run.shots.push((t, firer.0, *target));
                }
            }
            Effect::Hit {
                id,
                attacker,
                health,
                ..
            } => {
                run.hits
                    .push((t, id.0, attacker.map(|a| a.0), health.0, health.1));
                // measure: M-0020 (Procedure 7: a kill is a damage event that leaves its victim's health at 0)
                if health.0 > 0 && health.1 <= 0 {
                    kills.push((*id, *attacker));
                }
            }
            _ => {}
        }
    }
    // measure: M-0020 (Procedure 6: the first tick after whose update any unit is under mind control, each unit then with its group at tick 0 and its owner)
    if run.first_capture.is_none() {
        let caps: Vec<Captured> = w
            .objects
            .values()
            .filter(|o| glasnost_sim::mind::mind_controlled(w, o.id))
            .map(|o| (group_of(run, o.id), o.owner.map(|h| h.0)))
            .collect();
        if !caps.is_empty() {
            run.first_capture = Some((t, caps));
        }
    }
    if caos_open && let Some(c) = caos_side {
        caos_tick(w, run, t, c, &kills);
    }
    if keep_states {
        run.states
            .push((glasnost_sim::save::state_hash(w), w.rng.taken()));
    }
}

/// The CAOS games' reading of one tick (Procedure 7), up to the drones'
/// end.
fn caos_tick(w: &World, run: &mut Run, t: i32, c: usize, kills: &[(ObjectId, Option<ObjectId>)]) {
    let e = 1 - c;
    for &(victim, attacker) in kills {
        let by = attacker.map(|a| {
            if glasnost_sim::structures::bunker::is_bunker(w, a) {
                label(w, run, a)
            } else {
                type_name(w, a)
            }
        });
        if run.units[c].contains(&victim) {
            *run.caos_count
                .kills
                .entry(by.unwrap_or_else(|| "no attacker".into()))
                .or_default() += 1;
        } else if run.units[e].contains(&victim) {
            let by_e1 = attacker.is_some_and(|a| run.units[e].contains(&a));
            run.caos_count.e1_killed.0 += 1;
            run.caos_count.e1_killed.1 += usize::from(by_e1);
        }
    }
    let mut on = 0;
    let mut off = 0;
    let mut berserk_now = 0;
    for &id in &run.units[e] {
        let Some(o) = w.obj(id) else { continue };
        if o.health <= 0 {
            continue;
        }
        if o.on_map {
            on += 1;
            if glasnost_sim::psychic::berserk(w, id) {
                berserk_now += 1;
            }
        } else {
            off += 1;
            if t >= 1 && run.caos_events.1.is_none() {
                run.caos_events.1 = Some(t);
            }
        }
        if glasnost_sim::psychic::berserk(w, id) {
            run.caos_count.berserk_ever.insert(id);
            if run.caos_events.0.is_none() {
                run.caos_events.0 = Some(t);
            }
        }
    }
    for o in w.objects.values() {
        if !glasnost_sim::structures::bunker::is_bunker(w, o.id) {
            continue;
        }
        let prev = run.caos_count.bunker_owners.insert(o.id, o.owner);
        if prev.is_some_and(|p| p != o.owner) {
            let to = house_text(w, run, o.owner);
            run.caos_events.2.push((t, o.id.0, to));
        }
    }
    // measure: M-0020 (Procedure 7: the first tick after tick 0 at which no CAOS is on the map with health above 0)
    let drones = run.units[c]
        .iter()
        .any(|&id| w.obj(id).is_some_and(|o| o.on_map && o.health > 0));
    if t >= 1 && !drones {
        run.caos = Some(CaosEnd {
            tick: t,
            e1: (
                run.units[e].len(),
                on,
                off,
                berserk_now,
                run.caos_count.berserk_ever.len(),
            ),
            e1_killed: run.caos_count.e1_killed,
            caos_kills: run.caos_count.kills.clone(),
        });
    }
}

/// Plays one game with the recorder: `arm` moves side A's group before the
/// first step (part 2), `keep_states` keeps each step's state hash and every
/// shot (the same-spot comparison).
pub fn record(
    ground: &Ground,
    row: &str,
    col: &str,
    row_side: Side,
    seed: u32,
    arm: Arm,
    keep_states: bool,
) -> Result<Run, String> {
    let run = RefCell::new(Run::default());
    let mut prepare = |a: &mut Arena| -> Result<(), String> {
        let mut r = run.borrow_mut();
        let w = &mut a.world;
        r.houses = a.houses.to_vec();
        r.units = a.units.clone();
        r.types = match row_side {
            Side::A => [row.to_owned(), col.to_owned()],
            Side::B => [col.to_owned(), row.to_owned()],
        };
        r.placements = a.placements.clone();
        r.visceroid = a
            .placements
            .iter()
            .map(|p| {
                w.obj(p.id)
                    .and_then(|o| w.rules().unit(o.kind, o.type_index))
                    .is_some_and(|u| u.visceroid)
            })
            .collect();
        r.first_raw = a.first_raw;
        r.half = w.rules().quirks.ranged_draw_half_span;
        let top_quirk = w.rules().quirks.top_position_counts_as_centre;
        if arm != Arm::S {
            // measure: M-0020 (Setup, Part 2's moved arms: before tick 0's update every side-A YURI stands at the arm's position of the cell it stands on)
            for &id in &a.units[0] {
                let Some(o) = w.obj(id) else { continue };
                if o.kind != Kind::Infantry || !o.on_map {
                    continue;
                }
                let p = match arm {
                    Arm::At(p) => p,
                    _ => glasnost_sim::walk::position_class(
                        o.pos.x.rem_euclid(256),
                        o.pos.y.rem_euclid(256),
                        top_quirk,
                    ),
                };
                r.moved.0 += 1;
                if !move_to_position(w, id, p)? {
                    r.moved.1 += 1;
                }
            }
        }
        r.rng_before = Some(w.rng.clone());
        Ok(())
    };
    let mut seen = |w: &World, effects: &[Effect]| {
        let mut r = run.borrow_mut();
        observe(w, effects, &mut r, keep_states);
        if w.tick == 1
            && let Arm::At(p) = arm
        {
            let want = glasnost_sim::walk::standing_point(p);
            let a: Vec<ObjectId> = r.units[0].clone();
            r.at_arm = (0, 0);
            for id in a {
                if let Some(o) = w.obj(id).filter(|o| o.on_map) {
                    r.at_arm.0 += 1;
                    if (o.pos.x.rem_euclid(256), o.pos.y.rem_euclid(256)) == want {
                        r.at_arm.1 += 1;
                    }
                }
            }
        }
    };
    let (game, _, _) = oracle::play_with(
        ground,
        row,
        col,
        row_side,
        seed,
        false,
        &mut prepare,
        &mut seen,
    )?;
    let mut r = run.into_inner();
    r.game = Some(game);
    Ok(r)
}

/// One game of the record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Job {
    /// Part 1's game of this index ([`PART1`]); the YURI games on side A
    /// keep their states for the same-spot comparison.
    Part1(usize),
    /// Part 2: pairing index ([`PART2`]), seed, arm.
    Part2(usize, u32, Arm),
}

/// Every game of the record: part 1's six, part 2's 144, the two same-spot
/// runs (measure: M-0020, Procedures 1, 8 and 10).
pub fn jobs() -> Vec<Job> {
    let mut out: Vec<Job> = (0..PART1.len()).map(Job::Part1).collect();
    for p in 0..PART2.len() {
        for seed in PART2_SEEDS {
            for (_, arm) in ARMS {
                out.push(Job::Part2(p, seed, arm));
            }
        }
        out.push(Job::Part2(p, PART1_SEED, Arm::Same));
    }
    out
}

/// Plays a job.
fn play_job(ground: &Ground, j: Job) -> Result<Run, String> {
    match j {
        Job::Part1(i) => {
            let (row, col, side, _) = PART1[i];
            let same_spot = side == Side::A && row == "YURI";
            record(ground, row, col, side, PART1_SEED, Arm::S, same_spot)
        }
        Job::Part2(p, seed, arm) => {
            let (row, col) = PART2[p];
            record(ground, row, col, Side::A, seed, arm, arm == Arm::Same)
        }
    }
}

/// Plays `jobs` on `threads` worker threads, results in job order.
pub fn play_all(ground: &Ground, jobs: &[Job], threads: usize) -> Vec<Result<Run, String>> {
    let next = AtomicUsize::new(0);
    let slots: Vec<std::sync::Mutex<Option<Result<Run, String>>>> =
        jobs.iter().map(|_| std::sync::Mutex::new(None)).collect();
    std::thread::scope(|s| {
        for _ in 0..threads.max(1) {
            s.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(&j) = jobs.get(i) else { break };
                    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        play_job(ground, j)
                    }))
                    .unwrap_or_else(|_| Err(format!("{j:?}: panic")));
                    if let Ok(mut slot) = slots[i].lock() {
                        *slot = Some(r);
                    }
                }
            });
        }
    });
    slots
        .into_iter()
        .map(|m| {
            m.into_inner()
                .ok()
                .flatten()
                .unwrap_or_else(|| Err("not run".into()))
        })
        .collect()
}

/// A part 1 game's name, the record's (e.g. `YURI-GGI-A`).
pub fn part1_name(i: usize) -> String {
    let (row, col, side, _) = PART1[i];
    format!("{row}-{col}-{}", side.text())
}

/// Part 1's position facts of one game: placed infantry on the map at
/// tick 0, at a table P point, at C or T, at R, L or B, at Procedure 5's
/// expectation (and those Procedure 5 gave no position).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PositionFacts {
    /// Placed infantry on the map at tick 0 (those of the groups).
    pub placed: usize,
    /// At one of table P's five points.
    pub at_point: usize,
    /// At the centre or top point.
    pub at_c_or_t: usize,
    /// At the right, left or bottom point.
    pub at_rlb: usize,
    /// At Procedure 5's expected position.
    pub as_expected: usize,
    /// Why Procedure 5 could not be followed, if so.
    pub expectation: Option<String>,
}

/// Reads [`PositionFacts`] from a run.
pub fn position_facts(run: &Run, stream_seed: u32) -> PositionFacts {
    let group: BTreeSet<ObjectId> = run.units.iter().flatten().copied().collect();
    let placed: Vec<&Stand> = run
        .stands
        .iter()
        .filter(|s| group.contains(&s.id))
        .collect();
    let mut f = PositionFacts {
        placed: placed.len(),
        ..Default::default()
    };
    for s in &placed {
        match point_position(s.point.0, s.point.1) {
            Some(position::CENTRE | position::TOP) => {
                f.at_point += 1;
                f.at_c_or_t += 1;
            }
            Some(_) => {
                f.at_point += 1;
                f.at_rlb += 1;
            }
            None => {}
        }
    }
    match expected_positions(
        &run.placements,
        stream_seed,
        run.first_raw,
        run.half,
        &run.stands,
    ) {
        Ok(exp) => {
            for s in &placed {
                if let Some(Some(p)) = exp.get(&s.id)
                    && glasnost_sim::walk::standing_point(*p) == s.point
                {
                    f.as_expected += 1;
                }
            }
        }
        Err(e) => f.expectation = Some(e),
    }
    f
}

/// All of the record's rows over our runs: (row, verdict), in the
/// record's order. `part1` holds part 1's runs by index, `part2` part 2's
/// by (pairing, seed, arm), `archived` the archived games of M-0001 and
/// M-0002.
pub fn verdicts(
    part1: &BTreeMap<usize, Run>,
    part2: &BTreeMap<(usize, u32, Arm), Run>,
    archived: &[Game],
) -> Vec<(String, Verdict)> {
    let mut out: Vec<(String, Verdict)> = Vec::new();
    let mut push = |row: &str, v: Verdict| out.push((format!("M-0020 {row}"), v));
    let all1 = part1.len() == PART1.len();
    let missing1 = || Verdict::NotCheckable(format!("{} of 6 part 1 games ran", part1.len()));
    let archived_of = |row: &str, col: &str, side: Side, seed: u32| {
        archived
            .iter()
            .find(|g| g.row == row && g.col == col && g.row_side == side && g.seed == seed)
    };
    // measure: M-0020 (Result, Part 1: replays)
    {
        let mut same = 0;
        let mut differ = Vec::new();
        for (&i, r) in part1 {
            let (row, col, side, _) = PART1[i];
            let ours = r.game.as_ref();
            match (ours, archived_of(row, col, side, PART1_SEED)) {
                (Some(o), Some(a)) if o.winner == a.winner && o.end_frame == a.end_frame => {
                    same += 1
                }
                (Some(o), Some(a)) => differ.push(format!(
                    "{} (winner {} end {} against {} end {})",
                    part1_name(i),
                    o.winner.text(),
                    o.end_frame,
                    a.winner.text(),
                    a.end_frame
                )),
                _ => differ.push(format!("{} (no archived game)", part1_name(i))),
            }
        }
        push(
            "part 1: result equal to the archived game's (6 of 6)",
            Verdict::Shown(format!(
                "ours {same} of {}; the arena's per-game comparison, judged by its statistics, not here{}",
                part1.len(),
                if differ.is_empty() {
                    String::new()
                } else {
                    format!("; differing: {}", differ.join(", "))
                }
            )),
        );
        push(
            "part 1: shot and damage events identical to those recorded during M-0017's replay (6 of 6)",
            Verdict::NotCheckable(
                "M-0017 does not publish its recordings (M-0020 Evidence); ours have no recording to compare with"
                    .into(),
            ),
        );
    }
    // measure: M-0020 (Result, Part 1: standing positions at tick 0)
    let facts: Vec<(usize, PositionFacts)> = part1
        .iter()
        .map(|(&i, r)| {
            let (_, _, _, _) = PART1[i];
            (
                i,
                position_facts(r, oracle::game_seed(PART1_SEED).unwrap_or(0)),
            )
        })
        .collect();
    let pos_row = |name: &str,
                   ok: &dyn Fn(&PositionFacts) -> bool,
                   what: &dyn Fn(&PositionFacts) -> String| {
        if !all1 {
            return (name.to_owned(), missing1());
        }
        let bad: Vec<String> = facts
            .iter()
            .filter(|(_, f)| !ok(f))
            .map(|(i, f)| format!("{}: {}", part1_name(*i), what(f)))
            .collect();
        (
            name.to_owned(),
            if bad.is_empty() {
                Verdict::Pass
            } else {
                Verdict::Fail(bad.join("; "))
            },
        )
    };
    let rows = [
        pos_row(
            "part 1: placed infantry exactly at one of the five standing-position points of its cell (all, 6 games)",
            &|f| f.at_point == f.placed,
            &|f| {
                format!(
                    "{} of the placed infantry are off every point",
                    f.placed - f.at_point
                )
            },
        ),
        pos_row(
            "part 1: placed infantry at the centre or top position (none, 6 games)",
            &|f| f.at_c_or_t == 0,
            &|f| format!("{} at the centre or top", f.at_c_or_t),
        ),
        pos_row(
            "part 1: placed infantry at the right, left or bottom position (all, 6 games)",
            &|f| f.at_rlb == f.placed,
            &|f| format!("{} not at the right, left or bottom", f.placed - f.at_rlb),
        ),
        pos_row(
            "part 1: placed infantry at the position Procedure 5 expects (all, 6 games; k replayed from our stream, so partly by construction)",
            &|f| f.expectation.is_none() && f.as_expected == f.placed,
            &|f| match &f.expectation {
                Some(e) => format!("Procedure 5 not followed: {e}"),
                None => format!("{} not at the expected position", f.placed - f.as_expected),
            },
        ),
    ];
    for (n, v) in rows {
        push(&n, v);
    }
    // measure: M-0020 (Result, Part 1: draws before tick 0)
    let draws: Vec<(usize, Draws)> = part1
        .iter()
        .map(|(&i, r)| {
            let before = r.rng_before.as_ref().map_or(0, Rng::taken);
            (
                i,
                draw_check(&r.placements, &r.visceroid, r.first_raw, before),
            )
        })
        .collect();
    let draw_row = |name: &str, ok: &dyn Fn(&Draws) -> bool, what: &dyn Fn(&Draws) -> String| {
        if !all1 {
            return (name.to_owned(), missing1());
        }
        let bad: Vec<String> = draws
            .iter()
            .filter(|(_, d)| !ok(d))
            .map(|(i, d)| format!("{}: {}", part1_name(*i), what(d)))
            .collect();
        (
            name.to_owned(),
            if bad.is_empty() {
                Verdict::Pass
            } else {
                Verdict::Fail(bad.join("; "))
            },
        )
    };
    let rows = [
        draw_row(
            "part 1: Procedure 4's pattern equal to the last draws before tick 0's update (6 of 6; from our placement log's raw-value counts)",
            &|d| d.contiguous && d.off_pattern == 0,
            &|d| {
                format!(
                    "{} of {} tries off the pattern{}",
                    d.off_pattern,
                    d.tries,
                    if d.contiguous {
                        ""
                    } else {
                        ", raw values between tries or after the last"
                    }
                )
            },
        ),
        draw_row(
            "part 1: draws of that run each taking exactly one raw value (all; read as one raw value per creation number and per placement draw, our stream keeps no draw log)",
            &|d| d.off_pattern == 0,
            &|d| format!("{} tries take other raw values", d.off_pattern),
        ),
        draw_row(
            "part 1: draws after that run and before tick 0's update (none)",
            &|d| d.first_raw + d.pattern_raw == d.before_step,
            &|d| {
                format!(
                    "the run ends {} raw values from the first step",
                    d.before_step.abs_diff(d.first_raw + d.pattern_raw)
                )
            },
        ),
        draw_row(
            "part 1: raw values taken from the seeding to the start of that run (706 in each)",
            &|d| d.first_raw == FIRST_RAW,
            &|d| format!("ours {}", d.first_raw),
        ),
    ];
    for (n, v) in rows {
        push(&n, v);
    }
    // measure: M-0020 (Result, Part 1: in all four YURI games every unit under mind control at the first such tick is of the group opposing the YURI, and is then owned by the YURI's house)
    {
        let yuri: Vec<(&usize, &Run)> = part1
            .iter()
            .filter(|(i, _)| PART1[**i].0 == "YURI")
            .collect();
        let mut bad = Vec::new();
        for &(&i, r) in &yuri {
            let ys = usize::from(PART1[i].2 == Side::B);
            match &r.first_capture {
                None => bad.push(format!("{}: no unit under mind control", part1_name(i))),
                Some((_, caps)) => {
                    let off = caps
                        .iter()
                        .filter(|(g, h)| *g != Some(1 - ys) || *h != Some(r.houses[ys].0))
                        .count();
                    if off > 0 {
                        bad.push(format!(
                            "{}: {off} of {} not of the opposing group owned by the YURI's house",
                            part1_name(i),
                            caps.len()
                        ));
                    }
                }
            }
        }
        push(
            "part 1: in all four YURI games every unit under mind control at the first such tick is of the group opposing the YURI and owned by the YURI's house",
            if yuri.len() != 4 {
                Verdict::NotCheckable(format!("{} of 4 YURI games ran", yuri.len()))
            } else if bad.is_empty() {
                Verdict::Pass
            } else {
                Verdict::Fail(bad.join("; "))
            },
        );
    }
    // measure: M-0020 (Result, Part 2: standing positions and the winner)
    for (row, col, arm_c, games, a, b, none) in WINNERS {
        let Some(p) = PART2.iter().position(|&(r, c)| r == row && c == col) else {
            continue;
        };
        let arm = ARMS.iter().find(|(c, _)| *c == arm_c).map(|&(_, a)| a);
        let runs: Vec<&Run> = PART2_SEEDS
            .filter_map(|s| arm.and_then(|arm| part2.get(&(p, s, arm))))
            .collect();
        let count = |w: Winner| {
            runs.iter()
                .filter(|r| r.game.as_ref().is_some_and(|g| g.winner == w))
                .count() as u32
        };
        // Side A is the row side: Row is side A, Col side B.
        let ours = (count(Winner::Row), count(Winner::Col), count(Winner::None));
        let name = format!(
            "part 2: {row} against {col}, arm {arm_c}: {games} games, won by side A {a}, by side B {b}, no winner {none}"
        );
        push(
            &name,
            if runs.len() as u32 != games {
                Verdict::NotCheckable(format!("{} of {games} games ran", runs.len()))
            } else if ours == (a, b, none) {
                Verdict::Pass
            } else {
                Verdict::Fail(format!(
                    "ours: side A {}, side B {}, no winner {}",
                    ours.0, ours.1, ours.2
                ))
            },
        );
    }
    let moved: Vec<(&(usize, u32, Arm), &Run)> = part2
        .iter()
        .filter(|((_, _, a), _)| matches!(a, Arm::At(_)))
        .collect();
    let all2 = moved.len() == 120;
    {
        let bad: Vec<String> = moved
            .iter()
            .filter(|(_, r)| r.at_arm.1 != r.at_arm.0 || r.at_arm.0 == 0)
            .map(|((p, s, a), r)| {
                format!(
                    "{}-{} seed {s} arm {}: {} of {} on the map at the point",
                    PART2[*p].0,
                    PART2[*p].1,
                    arm_letter(*a),
                    r.at_arm.1,
                    r.at_arm.0
                )
            })
            .collect();
        push(
            "part 2: moved games with every side-A YURI at the arm's position at tick 0 (120 of 120)",
            if !all2 {
                Verdict::NotCheckable(format!("{} of 120 moved games ran", moved.len()))
            } else if bad.is_empty() {
                Verdict::Pass
            } else {
                Verdict::Fail(format!("{} games: {}", bad.len(), bad.join("; ")))
            },
        );
        let bad: Vec<String> = moved
            .iter()
            .filter(|((p, s, _), r)| {
                part2
                    .get(&(*p, *s, Arm::S))
                    .is_none_or(|sr| sr.rng_before.is_none() || sr.rng_before != r.rng_before)
            })
            .map(|((p, s, a), _)| {
                format!(
                    "{}-{} seed {s} arm {}",
                    PART2[*p].0,
                    PART2[*p].1,
                    arm_letter(*a)
                )
            })
            .collect();
        push(
            "part 2: moved games whose draws before tick 0's update equal arm S's at the same seed (120 of 120; the streams compared whole)",
            if !all2 {
                Verdict::NotCheckable(format!("{} of 120 moved games ran", moved.len()))
            } else if bad.is_empty() {
                Verdict::Pass
            } else {
                Verdict::Fail(bad.join("; "))
            },
        );
    }
    {
        let mut same = 0;
        let mut n = 0;
        let mut differ = Vec::new();
        for (p, &(row, col)) in PART2.iter().enumerate() {
            for s in ARCHIVED_SEEDS {
                let Some(g) = part2.get(&(p, s, Arm::S)).and_then(|r| r.game.as_ref()) else {
                    continue;
                };
                n += 1;
                match archived_of(row, col, Side::A, s) {
                    Some(a) if a.winner == g.winner && a.end_frame == g.end_frame => same += 1,
                    _ => differ.push(format!("{row}-{col} seed {s}")),
                }
            }
        }
        push(
            "part 2: arm S games at seeds 0 to 5 with the archived game's result (12 of 12)",
            Verdict::Shown(format!(
                "ours {same} of {n}; the arena's per-game comparison, judged by its statistics, not here{}",
                if differ.is_empty() {
                    String::new()
                } else {
                    format!("; differing: {}", differ.join(", "))
                }
            )),
        );
    }
    {
        let mut ok = 0;
        let mut bad = Vec::new();
        let mut n = 0;
        for (p, &pairing) in PART2.iter().enumerate() {
            let i = PART1
                .iter()
                .position(|&(r, c, s, _)| (r, c) == pairing && s == Side::A);
            let (Some(ctl), Some(base)) = (
                part2.get(&(p, PART1_SEED, Arm::Same)),
                i.and_then(|i| part1.get(&i)),
            ) else {
                continue;
            };
            n += 1;
            let mut what = Vec::new();
            if ctl.states != base.states {
                let first = ctl
                    .states
                    .iter()
                    .zip(&base.states)
                    .position(|(a, b)| a != b)
                    .map_or("in length".to_owned(), |k| format!("from step {}", k + 1));
                what.push(format!("object state and draws differ {first}"));
            }
            if ctl.shots != base.shots {
                what.push("shots differ".into());
            }
            if ctl.hits != base.hits {
                what.push("damage differs".into());
            }
            if ctl.rng_before != base.rng_before {
                what.push("draws before the first step differ".into());
            }
            if what.is_empty() {
                ok += 1;
            } else {
                bad.push(format!("{}-{}: {}", pairing.0, pairing.1, what.join(", ")));
            }
        }
        push(
            "part 2: same-spot runs identical to part 1's replay of the same game (object state by state hash per step, shots, damage, draws after the seeding; 2 of 2)",
            if n != 2 {
                Verdict::NotCheckable(format!("{n} of 2 same-spot runs ran with their replay"))
            } else if ok == 2 {
                Verdict::Pass
            } else {
                Verdict::Fail(bad.join("; "))
            },
        );
    }
    {
        let bad: Vec<String> = moved
            .iter()
            .filter(|((p, s, _), r)| {
                part2
                    .get(&(*p, *s, Arm::S))
                    .is_none_or(|sr| sr.hits == r.hits)
            })
            .map(|((p, s, a), _)| {
                format!(
                    "{}-{} seed {s} arm {}",
                    PART2[*p].0,
                    PART2[*p].1,
                    arm_letter(*a)
                )
            })
            .collect();
        push(
            "part 2: moved games whose damage events differ from arm S's game at the same seed (120 of 120)",
            if !all2 {
                Verdict::NotCheckable(format!("{} of 120 moved games ran", moved.len()))
            } else if bad.is_empty() {
                Verdict::Pass
            } else {
                Verdict::Fail(format!(
                    "{} games with the same damage: {}",
                    bad.len(),
                    bad.join("; ")
                ))
            },
        );
    }
    out
}

/// An arm's letter.
pub fn arm_letter(a: Arm) -> char {
    match a {
        Arm::S => 'S',
        Arm::Same => '=',
        Arm::At(p) => position_letter(p),
    }
}

/// The record's local value of `key` in `json` (the text of
/// `measurements/local/M-0020.json`), its whitespace collapsed, read with no
/// JSON library: the bracketed or plain text after the key's `"value":`.
pub fn local_value(json: &str, key: &str) -> Option<String> {
    let at = json.find(&format!("\"{key}\":"))?;
    let rest = &json[at..];
    let v = rest.find("\"value\":")? + "\"value\":".len();
    let body = rest[v..].trim_start();
    let end = match body.chars().next()? {
        '{' | '[' => {
            let mut depth = 0i32;
            let mut quoted = false;
            let mut end = None;
            for (k, ch) in body.char_indices() {
                match ch {
                    '"' => quoted = !quoted,
                    '{' | '[' if !quoted => depth += 1,
                    '}' | ']' if !quoted => {
                        depth -= 1;
                        if depth == 0 {
                            end = Some(k + 1);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            end?
        }
        _ => body.find([',', '\n']).unwrap_or(body.len()),
    };
    Some(body[..end].split_whitespace().collect::<Vec<_>>().join(" "))
}

/// The local report: ours beside the record's local values (written only
/// to the git-ignored `oracle-M-0020.txt`).
pub fn local_report(
    part1: &BTreeMap<usize, Run>,
    part2: &BTreeMap<(usize, u32, Arm), Run>,
    json: Option<&str>,
) -> String {
    let mut out = String::new();
    let rec = |key: &str| {
        json.and_then(|j| local_value(j, key))
            .unwrap_or_else(|| "(not in the local file)".into())
    };
    let _ = writeln!(
        out,
        "Local values (ours, then the record's from measurements/local/M-0020.json)"
    );
    let _ = writeln!(out, "  record tick0-positions: {}", rec("tick0-positions"));
    let _ = writeln!(out, "  record shared-cells: {}", rec("shared-cells"));
    let _ = writeln!(out, "  record placement-draws: {}", rec("placement-draws"));
    let _ = writeln!(
        out,
        "  record total-raw-values-to-tick0: {}",
        rec("total-raw-values-to-tick0")
    );
    for (&i, r) in part1 {
        let n = part1_name(i);
        let _ = writeln!(out, "\n{n}:");
        if let Some(g) = &r.game {
            let _ = writeln!(
                out,
                "  result: winner {}, end {}",
                g.winner.text(),
                g.end_frame
            );
        }
        // Positions per house.
        for s in 0..2 {
            let mut m: BTreeMap<char, usize> = BTreeMap::new();
            for st in r.stands.iter().filter(|st| r.units[s].contains(&st.id)) {
                let c = point_position(st.point.0, st.point.1).map_or('?', position_letter);
                *m.entry(c).or_default() += 1;
            }
            let _ = writeln!(
                out,
                "  tick-0 positions, side {} ({}): {m:?}",
                ["A", "B"][s],
                r.types[s]
            );
        }
        let d = draw_check(
            &r.placements,
            &r.visceroid,
            r.first_raw,
            r.rng_before.as_ref().map_or(0, Rng::taken),
        );
        let _ = writeln!(
            out,
            "  refused tries (the record's objects off the map): {}; pattern raw values {} over {} tries; first raw {}; raw values before the first step {}",
            d.refused, d.pattern_raw, d.tries, d.first_raw, d.before_step
        );
        let mut cells: BTreeMap<CellXY, Vec<char>> = BTreeMap::new();
        for st in &r.stands {
            cells
                .entry(st.cell)
                .or_default()
                .push(point_position(st.point.0, st.point.1).map_or('?', position_letter));
        }
        let shared: Vec<String> = cells
            .iter()
            .filter(|(_, v)| v.len() > 1)
            .map(|(c, v)| format!("({}, {}) {v:?}", c.x, c.y))
            .collect();
        let _ = writeln!(
            out,
            "  shared cells: {}",
            if shared.is_empty() {
                "none".into()
            } else {
                shared.join(", ")
            }
        );
        let f = position_facts(r, oracle::game_seed(PART1_SEED).unwrap_or(0));
        let _ = writeln!(out, "  position facts: {f:?}");
        let _ = writeln!(out, "  first targets (h1 = side A): {:?}", r.first_targets);
        let _ = writeln!(out, "    record: {}", rec(&format!("{n}-first-targets")));
        let _ = writeln!(out, "  first shots per house: {:?}", r.first_shots);
        let _ = writeln!(
            out,
            "  first shot per firer kind: {:?}",
            r.first_shot_by_firer
        );
        let _ = writeln!(out, "    record: {}", rec(&format!("{n}-first-shots")));
        if PART1[i].0 == "YURI" {
            let _ = writeln!(
                out,
                "  first YURI shot: {:?}; record: {}",
                r.first_row_shot,
                rec(&format!("{n}-first-yuri-shot"))
            );
            let _ = writeln!(
                out,
                "  first capture: {:?}; record: {}",
                r.first_capture.as_ref().map(|(t, c)| (t, c.len())),
                rec(&format!("{n}-first-capture"))
            );
        } else {
            let _ = writeln!(out, "  drones' end: {:?}", r.caos);
            let _ = writeln!(
                out,
                "    record no-caos-tick: {}",
                rec(&format!("{n}-no-caos-tick"))
            );
            let _ = writeln!(
                out,
                "    record e1-at-that-tick: {}",
                rec(&format!("{n}-e1-at-that-tick"))
            );
            let _ = writeln!(
                out,
                "    record caos-kills: {}",
                rec(&format!("{n}-caos-kills"))
            );
            let _ = writeln!(
                out,
                "  first berserk E1: {:?}; record: {}",
                r.caos_events.0,
                rec(&format!("{n}-first-berserk"))
            );
            let _ = writeln!(
                out,
                "  first E1 off the map: {:?}; bunker owner changes (tick, bunker, to): {:?}",
                r.caos_events.1, r.caos_events.2
            );
            let _ = writeln!(
                out,
                "    record bunker-events: {}",
                rec(&format!("{n}-bunker-events"))
            );
        }
    }
    for (p, (row, col)) in PART2.iter().enumerate() {
        let _ = writeln!(
            out,
            "\npart 2 {row}-{col} (YURI on side A), seeds 0..11 in order:"
        );
        for (c, arm) in ARMS {
            let line = |f: &dyn Fn(&Run) -> String| {
                PART2_SEEDS
                    .map(|s| part2.get(&(p, s, arm)).map_or("-".into(), f))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let _ = writeln!(
                out,
                "  arm {c}: winners [{}]; end ticks [{}]; first capture [{}]; moved {}",
                line(&|r| r
                    .game
                    .as_ref()
                    .map_or("-".into(), |g| g.winner.text().to_owned())),
                line(&|r| r
                    .game
                    .as_ref()
                    .map_or("-".into(), |g| g.end_frame.to_string())),
                line(&|r| r
                    .first_capture
                    .as_ref()
                    .map_or("-".into(), |(t, _)| t.to_string())),
                line(&|r| format!("{}/{}", r.moved.0, r.moved.1)),
            );
        }
        let _ = writeln!(
            out,
            "  record end ticks: {}",
            rec(&format!("q3-{row}-{col}-end-ticks"))
        );
        let _ = writeln!(
            out,
            "  record first capture: {}",
            rec(&format!("q3-{row}-{col}-first-capture-ticks"))
        );
        if let Some(r) = part2.get(&(p, PART1_SEED, Arm::Same)) {
            let _ = writeln!(
                out,
                "  same-spot run: moved {}, slot collisions {}, steps {}",
                r.moved.0,
                r.moved.1,
                r.states.len()
            );
        }
    }
    out
}

/// Plays the record and judges it: (the verdicts' summary for the arena's
/// output, the whole report with its local values). `archived` are the
/// archived games of M-0001 and M-0002; `json` the local file's text.
pub fn run(
    ground: &Ground,
    archived: &[Game],
    json: Option<&str>,
    threads: usize,
) -> Result<(String, String), String> {
    let jobs = jobs();
    let results = play_all(ground, &jobs, threads);
    let mut part1 = BTreeMap::new();
    let mut part2 = BTreeMap::new();
    let mut errors = String::new();
    for (j, r) in jobs.iter().zip(results) {
        match (j, r) {
            (Job::Part1(i), Ok(run)) => {
                part1.insert(*i, run);
            }
            (Job::Part2(p, s, a), Ok(run)) => {
                part2.insert((*p, *s, *a), run);
            }
            (j, Err(e)) => {
                let _ = writeln!(errors, "{ID}: {j:?}: {e}");
            }
        }
    }
    let v = verdicts(&part1, &part2, archived);
    let mut summary = errors.clone();
    let failing = v
        .iter()
        .filter(|(_, v)| matches!(v, Verdict::Fail(_)))
        .count();
    let checked = v
        .iter()
        .filter(|(_, v)| matches!(v, Verdict::Pass | Verdict::Fail(_)))
        .count();
    for (row, verdict) in &v {
        let _ = writeln!(summary, "  {row}: {}", verdict.text());
    }
    let _ = writeln!(
        summary,
        "{ID}: rows failing: {failing} of {checked} checked ({} rows)",
        v.len()
    );
    let mut full = summary.clone();
    full.push('\n');
    full.push_str(&local_report(&part1, &part2, json));
    Ok((summary, full))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oracle::Try;

    fn t(start: u64, creation_raw: u64, created: u32, took: bool, after_raw: u64) -> Try {
        Try {
            cell: CellXY::new(0, 0),
            start,
            creation_raw,
            created,
            took,
            after_raw,
        }
    }

    fn inf(id: u32, tries: Vec<Try>) -> Placement {
        Placement {
            side: Side::A,
            index: 0,
            id: ObjectId(id),
            kind: Kind::Infantry,
            tries,
        }
    }

    // measure: M-0020 (Procedure 4: each object's creation draw, then an infantry object's R(0, 3); a refused try counted as an object off the map); spec: S-0216 claims 3 and 8
    #[test]
    fn the_draw_pattern_counts_refused_tries_and_finds_gaps() {
        let ps = vec![
            inf(1, vec![t(10, 1, 1, true, 1)]),
            inf(3, vec![t(12, 1, 1, false, 1), t(14, 1, 1, true, 1)]),
        ];
        let d = draw_check(&ps, &[false, false], 10, 16);
        assert!(d.contiguous);
        assert_eq!(
            (d.tries, d.refused, d.pattern_raw, d.off_pattern),
            (3, 1, 6, 0)
        );
        // A raw value between the run and the first step.
        assert!(!draw_check(&ps, &[false, false], 10, 17).contiguous);
        // A creation that takes two raw values for one creation number.
        let odd = vec![inf(1, vec![t(10, 2, 1, true, 1)])];
        assert_eq!(draw_check(&odd, &[false], 10, 13).off_pattern, 1);
        // A vehicle of a visceroid type takes one raw value once placed.
        let v = vec![Placement {
            kind: Kind::Vehicle,
            ..inf(5, vec![t(10, 1, 1, true, 1)])
        }];
        assert_eq!(draw_check(&v, &[true], 10, 12).off_pattern, 0);
        assert_eq!(draw_check(&v, &[false], 10, 12).off_pattern, 1);
    }

    // measure: M-0020 (Procedure 5: the k-th R(0, 3) for the k-th infantry object, row Rk of table O, the first position no lower-identifier unit holds on the cell); spec: S-0208 table O
    #[test]
    fn procedure_5_reads_k_from_the_stream_and_skips_held_positions() {
        let seed = 7;
        let ps = vec![
            inf(1, vec![t(0, 1, 1, true, 1)]),
            inf(2, vec![t(2, 1, 1, false, 1), t(4, 1, 1, true, 1)]),
        ];
        // The draws the replay will read: raw 1 and raw 5 (0-based).
        let mut r = Rng::new(seed);
        r.raw();
        let k1 = r.ranged(0, 3, false);
        for _ in 0..3 {
            r.raw();
        }
        let k2 = r.ranged(0, 3, false);
        let first = table_o_row(k1)[0];
        let (u, v) = glasnost_sim::walk::standing_point(first);
        let c = CellXY::new(3, 3);
        let stands = vec![
            Stand {
                id: ObjectId(1),
                cell: c,
                point: (u, v),
            },
            Stand {
                id: ObjectId(2),
                cell: c,
                point: (0, 0),
            },
        ];
        let exp = expected_positions(&ps, seed, 0, false, &stands).expect("contiguous");
        assert_eq!(exp[&ObjectId(1)], Some(first));
        let want2 = table_o_row(k2).into_iter().find(|&p| p != first);
        assert_eq!(exp[&ObjectId(2)], want2);
        // A gap in the log is an error.
        assert!(expected_positions(&ps, seed, 1, false, &stands).is_err());
    }

    // spec: S-0208 table O (R0 and R1 right, left, bottom; R2 left, bottom, right; R3 bottom, right, left) and table P
    #[test]
    fn table_o_and_table_p_are_the_specs() {
        use position::{BOTTOM, CENTRE, LEFT, RIGHT, TOP};
        assert_eq!(table_o_row(0), [RIGHT, LEFT, BOTTOM]);
        assert_eq!(table_o_row(1), [RIGHT, LEFT, BOTTOM]);
        assert_eq!(table_o_row(2), [LEFT, BOTTOM, RIGHT]);
        assert_eq!(table_o_row(3), [BOTTOM, RIGHT, LEFT]);
        assert_eq!(point_position(128, 128), Some(CENTRE));
        assert_eq!(point_position(64, 64), Some(TOP));
        assert_eq!(point_position(192, 64), Some(RIGHT));
        assert_eq!(point_position(64, 192), Some(LEFT));
        assert_eq!(point_position(192, 192), Some(BOTTOM));
        assert_eq!(point_position(100, 100), None);
    }

    // measure: M-0020 (Procedure 10: a same-spot run moves every side-A YURI to the position it already stands at, and is identical to the unmoved game)
    #[test]
    fn moving_a_unit_to_its_own_position_changes_nothing() {
        let g = oracle::synthetic_ground(32);
        let mut a = oracle::setup(&g, ["WALKER", "WALKER"], Side::A, 0).expect("set-up");
        let w = &mut a.world;
        let id = a.units[0][0];
        let before = glasnost_sim::save::state_hash(w);
        let o = w.obj(id).expect("placed");
        let top = w.rules().quirks.top_position_counts_as_centre;
        let p = glasnost_sim::walk::position_class(
            o.pos.x.rem_euclid(256),
            o.pos.y.rem_euclid(256),
            top,
        );
        assert!(move_to_position(w, id, p).expect("moved"));
        assert_eq!(glasnost_sim::save::state_hash(w), before);
        // Another position moves it and its mark.
        let other = if p == position::CENTRE {
            position::TOP
        } else {
            position::CENTRE
        };
        assert!(move_to_position(w, id, other).expect("moved"));
        let o = w.obj(id).expect("placed");
        assert_eq!(
            (o.pos.x.rem_euclid(256), o.pos.y.rem_euclid(256)),
            glasnost_sim::walk::standing_point(other)
        );
        let occ = w.cells.occ(o.pos.cell(), o.layer).expect("record");
        assert!(matches!(
            occ.marks[usize::from(other)],
            Some(glasnost_sim::cells::Mark { by: glasnost_sim::cells::Marker::Infantry(i, _), .. }) if i == id
        ));
        assert!(occ.marks[usize::from(p)].is_none());
        assert_ne!(glasnost_sim::save::state_hash(w), before);
    }

    #[test]
    fn local_values_are_read_without_a_json_library() {
        let j = "{\n \"values\": {\n  \"a-b\": {\n   \"value\": {\n    \"x\": 1,\n    \"s\": \"q { r\"\n   },\n   \"unit\": \"t\"\n  },\n  \"c\": {\n   \"value\": 42,\n   \"unit\": \"t\"\n  }\n }\n}";
        assert_eq!(
            local_value(j, "a-b").as_deref(),
            Some("{ \"x\": 1, \"s\": \"q { r\" }")
        );
        assert_eq!(local_value(j, "c").as_deref(), Some("42"));
        assert_eq!(local_value(j, "d"), None);
    }

    // measure: M-0020 (Setup: part 1's six games, part 2's 144 and two same-spot runs)
    #[test]
    fn the_jobs_are_the_records_games() {
        let j = jobs();
        assert_eq!(j.len(), 6 + 2 * (12 * 6 + 1));
        assert_eq!(WINNERS.len(), PART2.len() * ARMS.len());
    }
}
