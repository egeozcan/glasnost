//! M-0024 on our engine (local lane only, M8 step A3 intake 112): M-0021
//! part B's two-group fight of test vehicles with the record's *hunt*
//! section, the scatter property on (part f) or off (part fc) and the
//! retaliate property off, each run recorded and judged by the record's
//! Procedure, one verdict per Result row.
//!
//! The ground, houses, groups and test types are M-0021 part B's
//! ([`crate::oracle_m0021`]); the *hunt* section keeps the install's other
//! fields of the behaviour (measure: M-0024, Setup, Test rules files: its
//! other fields left as stock), read at run time and never written down.
//! The record does not report the counts (Notes, *Not reported: counts and
//! ticks*), so the full report goes only to the git-ignored local lane.
//!
//! Tooling only: the recorder reads state and effects after each tick and
//! touches no simulation state.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::sync::Arc;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::rules::{Behaviour, SimRules};
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

use crate::oracle::{self, Ground, SIDE_HOUSES, SetupOptions, Side};
use crate::oracle_m0021::{self as m21, Part, TestBase};

/// The record's ID.
pub const ID: &str = "M-0024";
/// The record whose map the runs play on (measure: M-0024, Setup: as
/// M-0021's, which plays on M-0001's map).
pub const MAP_RECORD: &str = "M-0001";

/// A part of the record (measure: M-0024, Setup, Test rules files).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Arm {
    /// The scatter property on, the retaliate property off.
    F,
    /// The control: both properties off.
    Fc,
}

impl Arm {
    /// Both parts, in the record's order.
    pub const ALL: [Arm; 2] = [Arm::F, Arm::Fc];

    /// The part's name in the report.
    pub fn text(self) -> &'static str {
        match self {
            Arm::F => "f",
            Arm::Fc => "fc",
        }
    }

    /// The seed numbers it is played with (measure: M-0024, Setup, Runs:
    /// part f seeds 1000 to 1002, part fc seed 1000; seed number *s* is game
    /// seed 1000 + *s*, S-0216 claim 8).
    pub fn seeds(self) -> &'static [u32] {
        match self {
            Arm::F => &m21::SEEDS,
            Arm::Fc => &m21::SEEDS[..1],
        }
    }

    /// Whether *hunt* has the scatter property (measure: M-0024, Setup,
    /// Test rules files: part f on, part fc off).
    pub fn scatter(self) -> bool {
        self == Arm::F
    }
}

/// `base` with M-0021 part B's test types and the record's *hunt* section:
/// the scatter property as `arm` gives, the retaliate property off, the
/// behaviour's other fields as the install has them.
pub fn hunt_ruleset(base: &Ruleset, from: &TestBase, arm: Arm) -> Result<Ruleset, String> {
    // measure: M-0024 (Setup: as M-0021's Setup for part B, strength 400, fragment count 3)
    let (mut r, _) = m21::test_ruleset(base, from, Part::B)?;
    set_hunt(&mut r, arm.scatter(), false);
    Ok(r)
}

/// Sets two of *hunt*'s properties in `r`, keeping its other fields
/// (measure: M-0024 and M-0025, Setup, Test rules files: a *hunt* section
/// that sets two of the behaviour's properties, S-0215 Terms, its other
/// fields left as stock).
pub fn set_hunt(r: &mut Ruleset, scatter: bool, retaliate: bool) {
    let key = Behaviour::Hunt.key();
    let mut p = r.sim.behaviours.get(key).cloned().unwrap_or_default();
    p.scatter = Some(scatter);
    p.retaliate = Some(retaliate);
    r.sim.behaviours.insert(key.into(), p);
}

/// The ground of one part.
pub fn arm_ground(
    ground: &Ground,
    base: &Ruleset,
    hash: &str,
    from: &TestBase,
    arm: Arm,
) -> Result<Ground, String> {
    let r = hunt_ruleset(base, from, arm)?;
    let mut g = ground.clone();
    // measure: M-0024 (Setup: both houses computer-controlled, at the hard computer level)
    g.computer = true;
    g.rules = Arc::new(SimRules::new(&r, &format!("{hash}+{ID}-{}", arm.text())));
    Ok(g)
}

// ---------------------------------------------------------------------------
// The recording (measure: M-0024, Procedure: M-0021's recording, and for
// every vehicle each tick its current behaviour, its target or none and its
// destination cell or none)

/// One vehicle's row, stamped *t*: the state at the end of tick *t*.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Row {
    /// Its position.
    pub pos: Pos,
    /// Whether it is on the map with health left.
    pub in_play: bool,
    /// Its current behaviour.
    pub current: Option<Behaviour>,
    /// Whether it holds a target.
    pub target: bool,
    /// Its destination's cell (an object destination: that object's cell).
    pub dest: Option<CellXY>,
}

/// One hit: its tick, the object hit and its attacker's house (`None`: the
/// hit names no attacker).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hit {
    /// The tick.
    pub tick: i32,
    /// The object hit.
    pub id: ObjectId,
    /// Whether the hit names an attacker.
    pub attacker: Option<ObjectId>,
    /// The attacker's owner after the tick, when it is still an object.
    pub attacker_house: Option<HouseId>,
}

/// The row of `id` in `w`, or `None` when it is no object of the world.
pub fn row_of(w: &World, id: ObjectId) -> Option<Row> {
    let o = w.obj(id)?;
    let dest = match o.dest {
        Some(Target::Cell(c)) => Some(c),
        Some(Target::Object(x)) => w.obj(x).map(|d| d.pos.cell()),
        None => None,
    };
    Some(Row {
        pos: o.pos,
        in_play: o.on_map && o.health > 0,
        current: o.beh.current,
        target: o.target.is_some(),
        dest,
    })
}

/// The hits of the effects of the tick just run on any of `ids`.
pub fn hits_of(w: &World, effects: &[Effect], ids: &[ObjectId]) -> Vec<Hit> {
    let tick = w.tick;
    effects
        .iter()
        .filter_map(|e| match e {
            Effect::Hit { id, attacker, .. } if ids.contains(id) => Some(Hit {
                tick,
                id: *id,
                attacker: *attacker,
                attacker_house: attacker
                    .and_then(|a| w.obj(a).or_else(|| glasnost_sim::leftover::object(w, a)))
                    .and_then(|o| o.owner),
            }),
            _ => None,
        })
        .collect()
}

/// One run's recording.
#[derive(Clone, Debug, Default)]
pub struct Recording {
    /// F's vehicles and G's.
    pub vehicles: Vec<ObjectId>,
    /// Each tick's rows, from tick 0 (the state before the first tick).
    pub rows: BTreeMap<i32, BTreeMap<ObjectId, Row>>,
    /// Every hit on a vehicle.
    pub hits: Vec<Hit>,
    /// The last tick played.
    pub end: i32,
    /// Whether one side had no vehicle left at the end.
    pub decided: bool,
}

/// Sets up and plays one run of `arm` with F's group on `f_side` and seed
/// number `seed`, as M-0021 part B's ([`m21::play_to`]'s set-up), to the
/// end of the run.
pub fn play(ground: &Ground, f_side: Side, seed: u32) -> Result<Recording, String> {
    let rules = ground.rules.clone();
    // measure: M-0021 (Setup, Houses: F is the Americans' house and G the Russians', whichever side F's group stands on)
    let g_side = match f_side {
        Side::A => Side::B,
        Side::B => Side::A,
    };
    let mut houses = SIDE_HOUSES;
    houses[f_side.index()] = SIDE_HOUSES[0];
    houses[g_side.index()] = SIDE_HOUSES[1];
    let mut types = [m21::names::F; 2];
    types[g_side.index()] = m21::names::G;
    let first = if m21::F_FIRST { f_side } else { g_side };
    let opts = SetupOptions {
        count: Some(m21::GROUP),
        houses,
        clear_neutral: true,
        // measure: M-0021 (Setup, Houses: the British observer owns one building, its type local)
        observer_building: m21::observer_building(&rules),
    };
    let a = oracle::setup_with(ground, types, first, seed, &opts)?;
    let mut w = a.world;
    let f = a.units[f_side.index()].clone();
    let g = a.units[g_side.index()].clone();
    let all: Vec<ObjectId> = f.iter().chain(&g).copied().collect();
    let mut rec = Recording {
        vehicles: all.clone(),
        ..Default::default()
    };
    let snap = |w: &World, rec: &mut Recording| {
        let rows = all
            .iter()
            .filter_map(|&id| row_of(w, id).map(|r| (id, r)))
            .collect();
        rec.rows.insert(w.tick, rows);
    };
    snap(&w, &mut rec);
    let alive = |w: &World, ids: &[ObjectId]| {
        ids.iter()
            .any(|&id| w.obj(id).is_some_and(|o| o.on_map && o.health > 0))
    };
    // measure: M-0024 (Setup, Runs: in every run one side had no vehicle left before the run ended, and the tick limit of 3000 was not reached; M-0021 part B's limit)
    while w.tick < m21::LIMIT_B {
        glasnost_sim::tick::step(&mut w);
        let effects = std::mem::take(&mut w.effects);
        rec.hits.extend(hits_of(&w, &effects, &all));
        snap(&w, &mut rec);
        rec.end = w.tick;
        // measure: M-0021 (part B: each run until one side has no vehicle left)
        if !alive(&w, &f) || !alive(&w, &g) {
            rec.decided = true;
            break;
        }
    }
    Ok(rec)
}

// ---------------------------------------------------------------------------
// The judging (measure: M-0024, Procedure steps 1 to 5)

/// How a vehicle's row stamped *t* reads (measure: M-0024, Procedure step
/// 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Class {
    /// It holds a target.
    Target,
    /// No target, a destination at Chebyshev distance 1 from its cell at
    /// the end of *t* − 1.
    Neighbouring,
    /// No target, a destination farther away.
    Far,
    /// No target, a destination in its own cell (no class of the record's;
    /// counted against every row that lists the classes it allows).
    Own,
    /// Neither.
    Free,
}

/// The Chebyshev distance between two cells.
pub fn chebyshev(a: CellXY, b: CellXY) -> i32 {
    (a.x - b.x).abs().max((a.y - b.y).abs())
}

/// The class of `row` for a vehicle whose cell at the end of *t* − 1 was
/// `from`.
pub fn classify(row: &Row, from: CellXY) -> Class {
    if row.target {
        return Class::Target;
    }
    match row.dest {
        None => Class::Free,
        Some(c) => match chebyshev(c, from) {
            0 => Class::Own,
            1 => Class::Neighbouring,
            _ => Class::Far,
        },
    }
}

/// One run's judged rows.
#[derive(Clone, Debug, Default)]
pub struct Findings {
    /// The judged hits' classes, with whether every hit at the tick named
    /// no attacker (step 2).
    pub judged: Vec<(i32, ObjectId, Class, bool)>,
    /// Judged hits whose vehicle's position changed from the end of *t* − 2
    /// to the end of *t* − 1 (step 3).
    pub moved: usize,
    /// Judged hits with no row at *t* − 2.
    pub no_earlier: usize,
    /// The un-hit free vehicles' classes (step 4), counted.
    pub unhit: BTreeMap<Class, usize>,
    /// Rows in which a vehicle in play is in a behaviour other than *hunt*
    /// (step 5), by behaviour.
    pub other: BTreeMap<String, usize>,
    /// Hits on a free vehicle in play in another behaviour, not judged.
    pub other_hit: usize,
}

/// Judges one recording (measure: M-0024, Procedure).
pub fn judge(rec: &Recording) -> Findings {
    let mut out = Findings::default();
    // measure: M-0024 (Procedure step 2: several hits on one vehicle in one tick count as one; the hit names no attacker when every hit on it at t names none)
    let mut hit_at: BTreeMap<(i32, ObjectId), bool> = BTreeMap::new();
    for h in &rec.hits {
        let none = hit_at.entry((h.tick, h.id)).or_insert(true);
        *none &= h.attacker.is_none();
    }
    for (&t, rows) in &rec.rows {
        let Some(prev_rows) = rec.rows.get(&(t - 1)) else {
            continue;
        };
        for (&id, cur) in rows {
            if cur.in_play && cur.current != Some(Behaviour::Hunt) {
                let name = cur
                    .current
                    .map_or_else(|| "none".to_owned(), |b| format!("{b:?}"));
                *out.other.entry(name).or_default() += 1;
            }
            let Some(prev) = prev_rows.get(&id) else {
                continue;
            };
            // measure: M-0024 (Procedure step 1: free at the end of a tick when its row shows neither a target nor a destination)
            let free = prev.in_play && !prev.target && prev.dest.is_none();
            let hit = hit_at.get(&(t, id)).copied();
            // measure: M-0024 (Procedure step 5: a vehicle in another behaviour at the end of t − 1 is not judged at t)
            if prev.current != Some(Behaviour::Hunt) {
                if free && hit.is_some() && cur.in_play {
                    out.other_hit += 1;
                }
                continue;
            }
            // measure: M-0024 (Procedure steps 2 and 4: on the map with health left at the end of t)
            if !free || !cur.in_play {
                continue;
            }
            let class = classify(cur, prev.pos.cell());
            match hit {
                Some(no_attacker) => {
                    // measure: M-0024 (Procedure step 3: compare the position at the end of t − 2 with the one at the end of t − 1)
                    match rec.rows.get(&(t - 2)).and_then(|r| r.get(&id)) {
                        Some(p2) if p2.pos != prev.pos => out.moved += 1,
                        Some(_) => {}
                        None => out.no_earlier += 1,
                    }
                    out.judged.push((t, id, class, no_attacker));
                }
                None => *out.unhit.entry(class).or_default() += 1,
            }
        }
    }
    out
}

fn yes(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}

/// The runs of the record: each part with F on side A and on side B, over
/// its seeds (8 runs).
pub fn runs() -> Vec<(Arm, Side, u32)> {
    let mut out = Vec::new();
    for arm in Arm::ALL {
        for side in [Side::A, Side::B] {
            for &s in arm.seeds() {
                out.push((arm, side, s));
            }
        }
    }
    out
}

/// Builds both parts' grounds on `ground` (whose rules are `base`, of hash
/// `hash`), plays the record's 8 runs one after another and judges each
/// Result row. Returns the summary (no local values) and the full report
/// (with the counts, local values; never commit it).
pub fn run(
    ground: &Ground,
    base: &Ruleset,
    hash: &str,
    from: &TestBase,
) -> Result<(String, String), String> {
    let mut grounds = BTreeMap::new();
    for arm in Arm::ALL {
        grounds.insert(arm, arm_ground(ground, base, hash, from, arm)?);
    }
    let mut found: Vec<((Arm, Side, u32), Recording, Findings)> = Vec::new();
    for (arm, side, seed) in runs() {
        let g = grounds.get(&arm).ok_or("no ground")?;
        let rec = play(g, side, seed).map_err(|e| {
            format!(
                "{ID}: part {} F on {} seed {seed}: {e}",
                arm.text(),
                side.text()
            )
        })?;
        let f = judge(&rec);
        found.push(((arm, side, seed), rec, f));
    }
    let of = |arm: Arm| found.iter().filter(move |((a, _, _), _, _)| *a == arm);
    let judged = |arm: Arm| {
        of(arm)
            .flat_map(|(_, _, f)| f.judged.iter().copied())
            .collect::<Vec<_>>()
    };
    let (jf, jc) = (judged(Arm::F), judged(Arm::Fc));
    let count = |v: &[(i32, ObjectId, Class, bool)], c: &[Class]| {
        v.iter().filter(|x| c.contains(&x.2)).count()
    };
    let mut rows: Vec<(String, bool, String)> = Vec::new();
    // measure: M-0024 (Result: part f, judged hits classed neighbouring destination or target: all, exact)
    let f_ok = count(&jf, &[Class::Neighbouring, Class::Target]);
    rows.push((
        "Part f: judged hits classed neighbouring destination or target: all".into(),
        !jf.is_empty() && f_ok == jf.len(),
        format!("{f_ok} of {}", jf.len()),
    ));
    // measure: M-0024 (Result: part f, judged hits classed far destination or free: 0, exact)
    let f_bad = count(&jf, &[Class::Far, Class::Free, Class::Own]);
    rows.push((
        "Part f: judged hits classed far destination or free: 0".into(),
        f_bad == 0,
        format!(
            "{f_bad} (far {}, free {}, own cell {})",
            count(&jf, &[Class::Far]),
            count(&jf, &[Class::Free]),
            count(&jf, &[Class::Own])
        ),
    ));
    // measure: M-0024 (Result: part f, every judged hit that names no attacker classed neighbouring destination, exact)
    let none: Vec<_> = jf.iter().filter(|x| x.3).copied().collect();
    let none_ok = count(&none, &[Class::Neighbouring]);
    rows.push((
        "Part f: every judged hit naming no attacker classed neighbouring destination".into(),
        none_ok == none.len(),
        format!(
            "{none_ok} of {}; all judged hits neighbouring: {} of {}",
            none.len(),
            count(&jf, &[Class::Neighbouring]),
            jf.len()
        ),
    ));
    // measure: M-0024 (Result: part fc, judged hits classed neighbouring or far destination: 0, exact)
    let c_bad = count(&jc, &[Class::Neighbouring, Class::Far, Class::Own]);
    rows.push((
        "Part fc: judged hits classed neighbouring or far destination: 0".into(),
        c_bad == 0,
        format!("{c_bad} of {}", jc.len()),
    ));
    // measure: M-0024 (Result: part fc, judged hits classed free or target: all, exact)
    let c_ok = count(&jc, &[Class::Free, Class::Target]);
    rows.push((
        "Part fc: judged hits classed free or target: all".into(),
        !jc.is_empty() && c_ok == jc.len(),
        format!("{c_ok} of {}", jc.len()),
    ));
    // measure: M-0024 (Result: parts f and fc, judged hits whose vehicle's position changed from the end of t − 2 to the end of t − 1: 0, exact)
    let moved: usize = found.iter().map(|(_, _, f)| f.moved).sum();
    rows.push((
        "Parts f and fc: judged hits whose position changed from t − 2 to t − 1: 0".into(),
        moved == 0,
        format!("{moved}"),
    ));
    // measure: M-0024 (Result: parts f and fc, un-hit free vehicles classed neighbouring destination: 0, exact)
    let unhit = |c: Class| -> usize {
        found
            .iter()
            .map(|(_, _, f)| f.unhit.get(&c).copied().unwrap_or(0))
            .sum()
    };
    rows.push((
        "Parts f and fc: un-hit free vehicles classed neighbouring destination: 0".into(),
        unhit(Class::Neighbouring) == 0,
        format!("{}", unhit(Class::Neighbouring)),
    ));

    let failing = rows.iter().filter(|r| !r.1).count();
    let mut summary = String::new();
    let _ = writeln!(summary, "| Row | Ours holds |");
    let _ = writeln!(summary, "|---|---|");
    for (name, ok, _) in &rows {
        let _ = writeln!(summary, "| {name} | {} |", yes(*ok));
    }
    let _ = writeln!(summary, "{ID}: rows failing: {failing} of {}", rows.len());
    let decided = found.iter().filter(|(_, r, _)| r.decided).count();
    let _ = writeln!(
        summary,
        "Not for tests: runs decided before the tick limit {decided} of {}",
        found.len()
    );
    let mut full = summary.clone();
    let _ = writeln!(full, "\nOur readings (local values; never commit):");
    for (name, _, ours) in &rows {
        let _ = writeln!(full, "- {name}: {ours}");
    }
    let _ = writeln!(
        full,
        "- un-hit free vehicles classed far destination: {}; own cell {}",
        unhit(Class::Far),
        unhit(Class::Own)
    );
    for ((arm, side, seed), rec, f) in &found {
        let none = f.judged.iter().filter(|x| x.3).count();
        let _ = writeln!(
            full,
            "- part {} F on {} seed {seed}: end {} (decided {}), judged hits {} (naming no attacker {none}, no row at t − 2 {}), un-hit {:?}, other behaviours {:?}, other-behaviour free hits {}",
            arm.text(),
            side.text(),
            rec.end,
            yes(rec.decided),
            f.judged.len(),
            f.no_earlier,
            f.unhit,
            f.other,
            f.other_hit
        );
        let firsts: BTreeSet<ObjectId> = f.judged.iter().map(|x| x.1).collect();
        let _ = writeln!(full, "  vehicles with a judged hit: {}", firsts.len());
        for &(t, id, class, _) in f.judged.iter().filter(|x| x.3) {
            let _ = writeln!(
                full,
                "  hit naming no attacker: tick {t}, {id:?}, classed {class:?}, row {:?}",
                rec.rows.get(&t).and_then(|r| r.get(&id))
            );
        }
    }
    Ok((summary, full))
}
