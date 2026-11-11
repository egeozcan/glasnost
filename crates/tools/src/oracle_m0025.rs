//! M-0025 on our engine (local lane only, M8 step A3 intake 112): a test
//! vehicle *G* of a computer house in *hunt*, with nothing on the map that
//! it can attack, hit by one firer's area shots: the firer of *G*'s own
//! house (part g1), of an allied house (g2) or of an enemy house that *G*'s
//! weapon cannot harm (g3). Each run is recorded and judged by the record's
//! Procedure, one verdict per Result row.
//!
//! # The setup (measure: M-0025, Setup)
//!
//! - **Ground**: M-0001's arena map with M-0021's game settings, three hard
//!   computer houses *A* (the Americans, owning *G*), *B* (the French, on
//!   *A*'s lobby team) and *C* (the Russians, on another team), and
//!   M-0021's British observer with its one building ([`m21::observer_building`]).
//!   *A* and *B* take M-0001's two computer start positions, *C* the map's
//!   start position 2 (the record does not report its own: our choice), and
//!   every house's starting construction vehicle and the neutral house's
//!   objects are taken off before the units, as in M-0021.
//! - **Types** ([`test_ruleset`]): built on M-0021's test chassis. The
//!   special armour class *G*'s warhead cannot harm is special 1, our
//!   choice of the two.
//! - **Units**: *G* of *A* on the arena's centre cell W, *Q* of *C* on the
//!   cell east of it, the firer 7 cells south of *Q* (cells and facings
//!   our choice; the record does not report its own), then the observer's
//!   building. Each unit is placed as the arena places its groups and
//!   queued in *hunt* ([`oracle::place_unit`]); no order is given.
//!
//! The record does not report the hit counts, ticks and cells (Notes, *Not
//! reported*), so the full report goes only to the git-ignored local lane.
//!
//! Tooling only: the recorder reads state and effects after each tick and
//! touches no simulation state.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::Arc;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{ArmourClass, Decimal};
use glasnost_sim::econ::setup::{self, HouseValues, Player, Row as Difficulty, Settings};
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::world::{HouseId, ObjectId, World};

use crate::oracle::{self, Ground};
use crate::oracle_m0021::{self as m21, TestBase};
use crate::oracle_m0024::{self as m24, Hit, Row};

/// The record's ID.
pub const ID: &str = "M-0025";
/// The record whose map the runs play on (measure: M-0025, Setup: as
/// M-0021's, which plays on M-0001's map).
pub const MAP_RECORD: &str = "M-0001";
/// The tick limit of each run (measure: M-0025, Setup, Runs).
pub const LIMIT: i32 = 600;
/// The seed numbers (measure: M-0025, Setup, Runs: seeds 1000 to 1002;
/// seed number *s* is game seed 1000 + *s*, S-0216 claim 8).
pub const SEEDS: [u32; 3] = [0, 1, 2];
/// *G*'s strength and sight in cells (measure: M-0025, Setup, Test types).
pub const G_TYPE: (i32, i32) = (1000, 3);
/// *G*'s weapon: damage, rate of fire, range in cells (measure: M-0025).
pub const G_WEAPON: (i32, i32, i32) = (5, 20, 3);
/// *Q*'s strength (measure: M-0025).
pub const Q_STRENGTH: i32 = 60000;
/// The firer's strength (measure: M-0025).
pub const FIRER_STRENGTH: i32 = 1000;
/// The firer's weapon: damage, rate of fire, range in cells (measure:
/// M-0025).
pub const FIRER_WEAPON: (i32, i32, i32) = (50, 150, 8);
/// The firer's warhead's cell spread in cells (measure: M-0025: 1.5 cells,
/// full damage across the spread).
pub const FIRER_SPREAD: &str = "1.5";
/// The firer's distance from *Q* in cells (measure: M-0025, Setup, Units:
/// 7 cells from *Q*).
pub const FIRER_DISTANCE: i32 = 7;
/// *C*'s start position: another of the map's (measure: M-0025, Setup,
/// Houses; the record does not report its own, so this is our choice).
/// design: nocite (a harness choice)
pub const C_START: u32 = 2;
/// The special armour class *G* cannot harm (measure: M-0025: one of the
/// two special classes; which one is our choice).
pub const SPECIAL: ArmourClass = ArmourClass::Special1;

/// Our names of the test types, weapons, projectile and warheads.
pub mod names {
    /// *G*.
    pub const G: &str = "M25G";
    /// *Q*.
    pub const Q: &str = "M25Q";
    /// The firer of *A* and *B*.
    pub const FIRER: &str = "M25FIRER";
    /// *C*'s firer, of *Q*'s special armour class.
    pub const FIRER_C: &str = "M25FIRERC";
    /// *G*'s weapon.
    pub const G_WEAPON: &str = "M25GW";
    /// The firer's weapon.
    pub const FIRER_WEAPON: &str = "M25FW";
    /// The projectile.
    pub const PROJECTILE: &str = "M25PROJ";
    /// *G*'s warhead.
    pub const G_WARHEAD: &str = "M25GWH";
    /// The firer's warhead.
    pub const FIRER_WARHEAD: &str = "M25FWH";
}

/// A part of the record (measure: M-0025, Setup, Units).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Part {
    /// The firer is of *A*, *G*'s own house.
    G1,
    /// The firer is of *B*, allied with *A*.
    G2,
    /// The firer is of *C*, an enemy *G* cannot harm.
    G3,
}

impl Part {
    /// Every part, in the record's order.
    pub const ALL: [Part; 3] = [Part::G1, Part::G2, Part::G3];

    /// The part's name.
    pub fn text(self) -> &'static str {
        match self {
            Part::G1 => "g1",
            Part::G2 => "g2",
            Part::G3 => "g3",
        }
    }

    /// The firer's house.
    pub fn firer_house(self) -> HouseId {
        match self {
            Part::G1 => HOUSES[0],
            Part::G2 => HOUSES[1],
            Part::G3 => HOUSES[2],
        }
    }

    /// The firer's type.
    pub fn firer_type(self) -> &'static str {
        match self {
            // measure: M-0025 (the firers of A and B keep the chassis's class)
            Part::G1 | Part::G2 => names::FIRER,
            // measure: M-0025 (C's firer has Q's special armour class)
            Part::G3 => names::FIRER_C,
        }
    }
}

/// The house numbers of *A*, *B* and *C*: after the human player house, in
/// S-0009 claim 19's order.
pub const HOUSES: [HouseId; 3] = [HouseId(1), HouseId(2), HouseId(3)];

fn d(s: impl Into<String>) -> Decimal {
    Decimal(s.into())
}

/// `base` with the record's test types and *hunt* section added (measure:
/// M-0025, Setup, Test types and Test rules file).
pub fn test_ruleset(base: &Ruleset, from: &TestBase) -> Result<Ruleset, String> {
    let mut r = base.clone();
    let tank = r
        .sim
        .vehicles
        .get(&from.tank)
        .cloned()
        .ok_or_else(|| format!("no vehicle type {}", from.tank))?;
    let wname = tank
        .primary
        .clone()
        .filter(|n| !n.is_empty())
        .ok_or_else(|| format!("{} has no primary weapon", from.tank))?;
    let weapon = r
        .sim
        .weapons
        .get(&wname)
        .cloned()
        .ok_or_else(|| format!("no weapon {wname}"))?;
    let whname = weapon
        .warhead
        .clone()
        .ok_or_else(|| format!("{wname} has no warhead"))?;
    let warhead = r
        .sim
        .warheads
        .get(&whname)
        .cloned()
        .ok_or_else(|| format!("no warhead {whname}"))?;
    let projectile = m21::invisible_projectile(&r)
        .map(|(_, p)| p.clone())
        .ok_or("no invisible projectile type")?;
    let s = &mut r.sim;
    s.projectiles.insert(
        names::PROJECTILE.into(),
        m21::test_projectile(&projectile, None),
    );
    // measure: M-0025 (G's warhead: no cell spread, 100 % against every armour class except one of the two special classes, against which it has 0 %)
    let mut gwh = m21::test_warhead(&warhead);
    let mut verses = vec![d("1"); ArmourClass::ALL.len()];
    verses[SPECIAL.index()] = d("0");
    gwh.verses = Some(verses);
    s.warheads.insert(names::G_WARHEAD.into(), gwh);
    // measure: M-0025 (the firer's warhead: a cell spread of 1.5 cells, full damage across the spread, 100 % against every armour class)
    let mut fwh = m21::test_warhead(&warhead);
    fwh.cell_spread = Some(d(FIRER_SPREAD));
    fwh.percent_at_max = Some(d("1"));
    s.warheads.insert(names::FIRER_WARHEAD.into(), fwh);
    // measure: M-0025 (G's weapon: damage 5, rate of fire 20 ticks, range 3 cells)
    s.weapons.insert(
        names::G_WEAPON.into(),
        m21::test_weapon(
            &weapon,
            (G_WEAPON.0, Some(G_WEAPON.1), G_WEAPON.2),
            names::PROJECTILE,
            names::G_WARHEAD,
        ),
    );
    // measure: M-0025 (the firer's weapon: damage 50, rate of fire 150 ticks, range 8 cells)
    s.weapons.insert(
        names::FIRER_WEAPON.into(),
        m21::test_weapon(
            &weapon,
            (FIRER_WEAPON.0, Some(FIRER_WEAPON.1), FIRER_WEAPON.2),
            names::PROJECTILE,
            names::FIRER_WARHEAD,
        ),
    );
    let armed = |strength: i32, w: Option<&str>| {
        let mut u = m21::test_vehicle(&tank, strength);
        let w = w.map(str::to_owned);
        u.primary = w.clone();
        u.elite_primary = w;
        u.secondary = None;
        u.elite_secondary = None;
        u.combat.weapon_slots = None;
        u.combat.elite_weapon_slots = None;
        u.combat.is_gattling = Some(false);
        u
    };
    // measure: M-0025 (G: strength 1000, sight 3 cells)
    let mut g = armed(G_TYPE.0, Some(names::G_WEAPON));
    g.sight = Some(G_TYPE.1);
    // measure: M-0025 (Q: strength 60000, no weapon, the special armour class)
    let mut q = armed(Q_STRENGTH, None);
    q.combat.armor = Some(SPECIAL);
    // measure: M-0025 (the firer: strength 1000)
    let firer = armed(FIRER_STRENGTH, Some(names::FIRER_WEAPON));
    let mut firer_c = firer.clone();
    firer_c.combat.armor = Some(SPECIAL);
    for (n, u) in [
        (names::G, g),
        (names::Q, q),
        (names::FIRER, firer),
        (names::FIRER_C, firer_c),
    ] {
        r.sim.vehicles.insert(n.into(), u);
        r.registries
            .entry("vehicles".into())
            .or_default()
            .push(n.into());
    }
    // measure: M-0025 (Test rules file: a hunt section with the scatter property on and the retaliate property on, its other fields left as stock)
    m24::set_hunt(&mut r, true, true);
    // measure: M-0025 (Test rules file: the player-scatter setting off)
    r.sim.general.player_scatter = Some(false);
    Ok(r)
}

/// The ground of the record: `ground` with the test types in its rules.
pub fn test_ground(
    ground: &Ground,
    base: &Ruleset,
    hash: &str,
    from: &TestBase,
) -> Result<Ground, String> {
    let r = test_ruleset(base, from)?;
    let mut g = ground.clone();
    g.computer = true;
    g.rules = Arc::new(SimRules::new(&r, &format!("{hash}+{ID}")));
    Ok(g)
}

/// One run's recording.
#[derive(Clone, Debug)]
pub struct Recording {
    /// *G*, *Q* and the firer.
    pub units: [ObjectId; 3],
    /// *G*'s rows by tick, from tick 0 (the state before the first tick).
    pub rows: BTreeMap<i32, Row>,
    /// Every hit on *G*.
    pub hits: Vec<Hit>,
    /// Whether *A* and *B* are allied, and *C* allied with neither.
    pub alliances_ok: bool,
    /// Whether *G*, *Q* and the firer stood on their stated cells.
    pub placed_as_stated: bool,
}

/// Sets up and plays one run of `part` with seed number `seed`.
pub fn play(g: &Ground, part: Part, seed: u32) -> Result<Recording, String> {
    let rules = g.rules.clone();
    let country = |want: &str| {
        rules
            .country_names
            .iter()
            .find(|n| n.eq_ignore_ascii_case(want))
            .cloned()
            .ok_or_else(|| format!("{ID}: no country {want}"))
    };
    // measure: M-0025 (Setup, Houses: three computer houses at the hard level, A and B on one team, C on another; M-0021's observer)
    let computer = |name: &str, c: &str, start: u32, team: u8| -> Result<Player, String> {
        Ok(Player {
            name: name.into(),
            country: country(c)?,
            start: Some(start),
            ai: true,
            values: HouseValues {
                team: Some(team),
                // spec: S-0412 claim 4 (a hard computer: index 0, the easy row), applied through claim 5's handicap
                handicap: Some(Difficulty::Easy),
                ..Default::default()
            },
        })
    };
    let players = vec![
        Player {
            name: "Human".into(),
            country: oracle::human_country(&rules)?,
            start: Some(oracle::HUMAN_START),
            ai: false,
            values: HouseValues::default(),
        },
        computer("A", "Americans", oracle::STARTS[0], 0)?,
        computer("B", "French", oracle::STARTS[1], 0)?,
        computer("C", "Russians", C_START, 1)?,
    ];
    // measure: M-0021 (Setup: 0 credits, short game, superweapons and crates off; bases on; the fastest speed)
    let settings = Settings {
        credits: Some(0),
        short_game: Some(false),
        superweapons: Some(false),
        crates: Some(false),
        ..Default::default()
    };
    let stream = oracle::game_seed(seed).ok_or("seed")?;
    let mut gs = setup::skirmish(&rules, &players, settings, 0, stream);
    gs.bases = true;
    gs.game_speed = oracle::FASTEST;
    for h in HOUSES {
        gs.houses[usize::from(h.0)].human = false;
    }
    let mut w = glasnost_sim::load::new_game(rules.clone(), &g.map, Arc::clone(&g.tiles), &gs)
        .map_err(|e| format!("{e:?}"))?;
    // spec: S-0216 claim 8 (Game start: no house treats the human player house as an enemy; design, copied from M-0001's Setup)
    w.houses[usize::from(oracle::HUMAN.0)].idle_test_house = true;
    // measure: M-0021 (Setup, Houses: each house's starting construction vehicle off the map at tick 0; every object of the neutral house removed)
    let vehicles: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| o.kind == Kind::Vehicle)
        .map(|o| o.id)
        .collect();
    for id in vehicles {
        glasnost_sim::objects::take_out_quietly(&mut w, id);
    }
    oracle::take_off_neutral(&mut w);
    for h in HOUSES {
        let house = &mut w.houses[usize::from(h.0)];
        // spec: S-0412 claim 3 (a computer player house: the rules' maximum house skill level)
        house.skill = rules.general.iq_max;
        // spec: S-0412 claim 4 (a hard computer: difficulty index 0)
        house.econ.setup.difficulty = 0;
    }
    let alliances_ok = w.allied(Some(HOUSES[0]), Some(HOUSES[1]))
        && !w.allied(Some(HOUSES[0]), Some(HOUSES[2]))
        && !w.allied(Some(HOUSES[1]), Some(HOUSES[2]));
    let ti = |n: &str| {
        rules
            .index_of(Kind::Vehicle, n)
            .ok_or_else(|| format!("{ID}: no vehicle type {n}"))
    };
    let at = g.w;
    let q_at = CellXY::new(at.x + 1, at.y);
    let f_at = CellXY::new(q_at.x, q_at.y + FIRER_DISTANCE);
    let facing = oracle::FACINGS[0];
    // measure: M-0025 (Setup, Units, in creation order: G of A; Q of C, on the cell next to G's; one firer, 7 cells from Q; the observer's building)
    let (gid, gc, _) =
        oracle::place_unit(&mut w, Kind::Vehicle, ti(names::G)?, HOUSES[0], at, facing)?;
    let (qid, qc, _) = oracle::place_unit(
        &mut w,
        Kind::Vehicle,
        ti(names::Q)?,
        HOUSES[2],
        q_at,
        facing,
    )?;
    let (fid, fc, _) = oracle::place_unit(
        &mut w,
        Kind::Vehicle,
        ti(part.firer_type())?,
        part.firer_house(),
        f_at,
        facing,
    )?;
    if let Some(name) = m21::observer_building(&rules) {
        let &(x, y) = g
            .map
            .waypoints
            .get(&(oracle::HUMAN_START as u16))
            .ok_or("the map has no start position for the observer")?;
        glasnost_sim::testkit::spawn(
            &mut w,
            Kind::Building,
            &name,
            oracle::HUMAN.0,
            CellXY::new(i32::from(x), i32::from(y)),
            0,
        );
    }
    let mut rec = Recording {
        units: [gid, qid, fid],
        alliances_ok,
        placed_as_stated: gc == at && qc == q_at && fc == f_at,
        rows: BTreeMap::new(),
        hits: Vec::new(),
    };
    let snap = |w: &World, rec: &mut Recording| {
        if let Some(r) = m24::row_of(w, gid) {
            rec.rows.insert(w.tick, r);
        }
    };
    snap(&w, &mut rec);
    while w.tick < LIMIT {
        glasnost_sim::tick::step(&mut w);
        let effects = std::mem::take(&mut w.effects);
        rec.hits.extend(m24::hits_of(&w, &effects, &[gid]));
        snap(&w, &mut rec);
    }
    Ok(rec)
}

/// One run's judged hits (measure: M-0025, Procedure).
#[derive(Clone, Debug, Default)]
pub struct Findings {
    /// The judged hits: tick, whether every hit at the tick came from the
    /// part's firer's house, whether *G* moved from *t* − 2 to *t* − 1, and
    /// whether its row at *t* shows a neighbouring destination, no target
    /// and *hunt*.
    pub judged: Vec<(i32, bool, bool, bool)>,
    /// Rows in which *G* holds a target.
    pub target_rows: usize,
    /// Hits on *G* in all.
    pub hits: usize,
}

/// Judges one recording.
pub fn judge(rec: &Recording, part: Part) -> Findings {
    let mut out = Findings {
        hits: rec.hits.len(),
        target_rows: rec.rows.values().filter(|r| r.target).count(),
        ..Default::default()
    };
    // measure: M-0025 (Procedure step 1: several hits in one tick count as one)
    let mut by_tick: BTreeMap<i32, Vec<&Hit>> = BTreeMap::new();
    for h in &rec.hits {
        by_tick.entry(h.tick).or_default().push(h);
    }
    for (&t, hits) in &by_tick {
        // measure: M-0025 (Procedure step 1: a hit with no row at t − 1 or t − 2 is not judged)
        let (Some(p2), Some(p1), Some(cur)) = (
            rec.rows.get(&(t - 2)),
            rec.rows.get(&(t - 1)),
            rec.rows.get(&t),
        ) else {
            continue;
        };
        // measure: M-0025 (Procedure step 1: G's row stamped t − 1 shows hunt, no target and no destination, and G is on the map with health left at the end of t)
        if p1.current != Some(Behaviour::Hunt) || p1.target || p1.dest.is_some() || !cur.in_play {
            continue;
        }
        // measure: M-0025 (Procedure step 2: the house of the attacker)
        let house_ok = hits
            .iter()
            .all(|h| h.attacker.is_some() && h.attacker_house == Some(part.firer_house()));
        // measure: M-0025 (Procedure step 3: its target, its behaviour, and its destination's Chebyshev distance from its cell at the end of t − 1)
        let outcome = !cur.target
            && cur.current == Some(Behaviour::Hunt)
            && cur
                .dest
                .is_some_and(|c| m24::chebyshev(c, p1.pos.cell()) == 1);
        out.judged.push((t, house_ok, p2.pos != p1.pos, outcome));
    }
    out
}

fn yes(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}

/// Plays the record's 9 runs one after another and judges each Result
/// row. Returns the summary (no local values) and the full report (with
/// them; never commit it).
pub fn run(
    ground: &Ground,
    base: &Ruleset,
    hash: &str,
    from: &TestBase,
) -> Result<(String, String), String> {
    let g = test_ground(ground, base, hash, from)?;
    let mut found: Vec<((Part, u32), Recording, Findings)> = Vec::new();
    for part in Part::ALL {
        for &seed in &SEEDS {
            let rec = play(&g, part, seed)
                .map_err(|e| format!("{ID}: part {} seed {seed}: {e}", part.text()))?;
            let f = judge(&rec, part);
            found.push(((part, seed), rec, f));
        }
    }
    let judged: Vec<(i32, bool, bool, bool)> = found
        .iter()
        .flat_map(|(_, _, f)| f.judged.iter().copied())
        .collect();
    let mut rows: Vec<(String, bool, String)> = Vec::new();
    // measure: M-0025 (Result: judged hits whose attacker is of A (g1), B (g2), C (g3): all, exact)
    let house = judged.iter().filter(|x| x.1).count();
    rows.push((
        "Judged hits whose attacker is of A (g1), B (g2), C (g3): all".into(),
        house == judged.len(),
        format!("{house} of {}", judged.len()),
    ));
    // measure: M-0025 (Result: judged hits leaving G with a destination in a cell next to its own, no target, in hunt: all, exact)
    let ok = judged.iter().filter(|x| x.3).count();
    rows.push((
        "Judged hits leaving G with a neighbouring destination, no target, in hunt: all".into(),
        ok == judged.len(),
        format!("{ok} of {}", judged.len()),
    ));
    // measure: M-0025 (Result: judged hits whose G position changed from the end of t − 2 to the end of t − 1: 0, exact)
    let moved = judged.iter().filter(|x| x.2).count();
    rows.push((
        "Judged hits whose G position changed from t − 2 to t − 1: 0".into(),
        moved == 0,
        format!("{moved}"),
    ));
    // measure: M-0025 (Result: runs with at least one judged hit: 9 of 9, exact)
    let with = found
        .iter()
        .filter(|(_, _, f)| !f.judged.is_empty())
        .count();
    rows.push((
        "Runs with at least one judged hit: 9".into(),
        with == found.len() && found.len() == 9,
        format!("{with} of {}", found.len()),
    ));
    // measure: M-0025 (Result: part g3, rows in which G holds a target: 0, exact)
    let g3: usize = found
        .iter()
        .filter(|((p, _), _, _)| *p == Part::G3)
        .map(|(_, _, f)| f.target_rows)
        .sum();
    rows.push((
        "Part g3: rows in which G holds a target: 0".into(),
        g3 == 0,
        format!("{g3}"),
    ));
    let setup_ok = found
        .iter()
        .all(|(_, r, _)| r.alliances_ok && r.placed_as_stated);
    rows.push((
        "Set-up: A and B allied, C an enemy of both; units on their stated cells".into(),
        setup_ok,
        format!("{setup_ok}"),
    ));

    let failing = rows.iter().filter(|r| !r.1).count();
    let mut summary = String::new();
    let _ = writeln!(summary, "| Row | Ours holds |");
    let _ = writeln!(summary, "|---|---|");
    for (name, ok, _) in &rows {
        let _ = writeln!(summary, "| {name} | {} |", yes(*ok));
    }
    let _ = writeln!(summary, "{ID}: rows failing: {failing} of {}", rows.len());
    let mut full = summary.clone();
    let _ = writeln!(full, "\nOur readings (local values; never commit):");
    for (name, _, ours) in &rows {
        let _ = writeln!(full, "- {name}: {ours}");
    }
    for ((part, seed), rec, f) in &found {
        let first = rec.hits.first().map(|h| h.tick);
        let _ = writeln!(
            full,
            "- part {} seed {seed}: hits on G {}, the first at {first:?}; judged {:?}; rows with a target {}; alliances {}, placed {}",
            part.text(),
            f.hits,
            f.judged,
            f.target_rows,
            yes(rec.alliances_ok),
            yes(rec.placed_as_stated)
        );
    }
    Ok((summary, full))
}
