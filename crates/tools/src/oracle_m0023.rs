//! M-0023 on our engine (local lane only): S-0417 scenario U10 part g run
//! 1, a released infantry unit given a move order into a walled pen before
//! its first turn, with the record's controls and baseline. The ground is
//! M-0001's stock map from the install with the record's ring of wall
//! sections added; the types, countries and cells are the record's local
//! values, read at run time from `measurements/local/M-0023.json`, never
//! from a committed number. Each Result row of the record is judged on the
//! recordings. The report names the local values, so it goes only to the
//! git-ignored local lane.

use std::fmt::Write as _;
use std::path::Path;
use std::sync::Arc;

use glasnost_map::CellXY;
use glasnost_sim::econ::release::{self, Outcome};
use glasnost_sim::econ::setup::{self, HouseValues, Player, Settings};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{HouseId, Layer, ObjectId, Pos, Target, World};

use crate::oracle::{self, Ground};

/// The record's ID.
pub const ID: &str = "M-0023";
/// The record whose map the test map is made from (measure: M-0023, Setup:
/// the stock map `Arena.mmx`, M-0001's).
pub const MAP_RECORD: &str = "M-0001";
/// The ticks recorded after the release (measure: M-0023, Procedure step
/// 3: from *t* to *t* + 300).
pub const SPAN: i32 = 300;
/// The seed number whose game seed is the record's (measure: M-0023,
/// Setup: seed 1000 in every run; S-0216 claim 8: seed number s gives
/// 1000 + s).
pub const SEED: u32 = 0;
/// The tick of the release in our games: any tick after the game start
/// (the record's own is local and its value does not matter to a row).
/// design: nocite (a harness choice; every row is relative to *t*)
pub const RELEASE_AT: i32 = 30;
/// The human player's credits before tick 0 (measure: M-0023, Setup,
/// Houses).
pub const CREDITS: i32 = 10000;

/// The record's local values that the set-up needs.
#[derive(Clone, Debug)]
pub struct Local {
    /// The released infantry type.
    pub infantry: String,
    /// K's type.
    pub factory: String,
    /// The wall type.
    pub wall: String,
    /// The human country and the two computer countries.
    pub countries: [String; 3],
    /// The power plant types: the human's and the first computer
    /// house's, then the second's.
    pub plants: [String; 2],
    /// K's top-left cell.
    pub k: CellXY,
    /// The pen's centre (the order's destination).
    pub centre: CellXY,
    /// The power plants' cells: the human's, then the two computer houses'.
    pub plant_cells: [CellXY; 3],
    /// Run 1's new destination and the baseline's (report only).
    pub dest_now: CellXY,
    /// The baseline's and the controls' destination (report only).
    pub dest_base: CellXY,
}

/// The string value of `"key": "value"` in `text`.
fn field<'a>(text: &'a str, key: &str) -> Result<&'a str, String> {
    let pat = format!("\"{key}\": \"");
    let at = text
        .find(&pat)
        .ok_or_else(|| format!("{ID} local values: no {key}"))?;
    let rest = &text[at + pat.len()..];
    let end = rest
        .find('"')
        .ok_or_else(|| format!("{ID} local values: {key} unterminated"))?;
    Ok(&rest[..end])
}

/// A cell written `x,y`.
fn cell(s: &str) -> Result<CellXY, String> {
    let (x, y) = s
        .trim()
        .split_once(',')
        .ok_or_else(|| format!("{ID} local values: {s} is not a cell"))?;
    let n = |v: &str| {
        v.trim()
            .parse::<i32>()
            .map_err(|_| format!("{ID} local values: {s} is not a cell"))
    };
    Ok(CellXY::new(n(x)?, n(y)?))
}

/// The record's local values file: `GLASNOST_M0023_LOCAL` if set (for a
/// worktree, whose git-ignored `measurements/local` is empty), otherwise
/// `<measurements>/local/M-0023.json`.
pub fn local_path(measurements: &Path) -> std::path::PathBuf {
    std::env::var_os("GLASNOST_M0023_LOCAL").map_or_else(
        || measurements.join("local").join(format!("{ID}.json")),
        std::path::PathBuf::from,
    )
}

/// Reads the record's local values from `path` ([`local_path`]).
pub fn read_local(path: &Path) -> Result<Local, String> {
    let text =
        std::fs::read_to_string(path).map_err(|e| format!("{}: {:?}", path.display(), e.kind()))?;
    let two = |s: &str, sep: &str| -> Result<(String, String), String> {
        s.split_once(sep)
            .map(|(a, b)| (a.trim().to_owned(), b.trim().to_owned()))
            .ok_or_else(|| format!("{ID} local values: {s}"))
    };
    let (c1, c2) = two(field(&text, "computer countries")?, ",")?;
    let plants = field(&text, "power plants")?;
    let first_word = |s: &str| s.split_whitespace().next().unwrap_or("").to_owned();
    let (p1, p2) = two(plants, "),")?;
    let (q1, q2) = two(field(&text, "computer power plants")?, " and ")?;
    Ok(Local {
        infantry: field(&text, "released infantry")?.to_owned(),
        factory: field(&text, "factory K")?.to_owned(),
        wall: field(&text, "wall")?.to_owned(),
        countries: [field(&text, "human country")?.to_owned(), c1, c2],
        plants: [first_word(&p1), first_word(&p2)],
        k: cell(field(&text, "K (top-left foundation cell)")?)?,
        centre: cell(field(&text, "pen centre (order destination)")?)?,
        plant_cells: [
            cell(field(&text, "human power plant")?)?,
            cell(&q1)?,
            cell(&q2)?,
        ],
        dest_now: cell(field(&text, "new destination, run 1 and run 2")?)?,
        dest_base: cell(field(&text, "baseline's and controls' destination")?)?,
    })
}

/// The 16 cells of the 5 × 5 ring round `centre` (measure: M-0023, Setup,
/// Map: 16 wall sections as a 5 × 5 ring closing a 3 × 3 pen).
pub fn ring(centre: CellXY) -> Vec<CellXY> {
    let mut v = Vec::new();
    for dy in -2..=2 {
        for dx in -2..=2 {
            if dx == -2 || dx == 2 || dy == -2 || dy == 2 {
                v.push(CellXY::new(centre.x + dx, centre.y + dy));
            }
        }
    }
    v
}

/// Whether `c` is in the pen or next to it (within 1 cell of its 3 × 3).
fn in_or_next_to_pen(centre: CellXY, c: CellXY) -> bool {
    (c.x - centre.x).abs() <= 2 && (c.y - centre.y).abs() <= 2
}

/// The arms of the record (measure: M-0023, the table of runs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arm {
    /// g1-now: the order in the release's tick, after the release.
    Now,
    /// g1-after: the order two ticks after the release, before that
    /// tick's update.
    After,
    /// g0-none: no order (part b on this map).
    None,
}

/// One row of a recording: the state after a tick's update.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// The tick.
    pub tick: i32,
    /// Current behaviour.
    pub current: Option<Behaviour>,
    /// Queued behaviour.
    pub queued: Option<Behaviour>,
    /// Destination cell, if a cell or an object's cell.
    pub dest: Option<Target>,
    /// Position.
    pub pos: Pos,
    /// Whether its cell holds K.
    pub on_k: bool,
    /// Health.
    pub health: i32,
}

/// One arm's recording.
#[derive(Clone, Debug)]
pub struct Recording {
    /// The arm.
    pub arm: Arm,
    /// *t*, the release tick.
    pub t: i32,
    /// The release position.
    pub release_pos: Pos,
    /// Whether every ring cell held the wall overlay and the centre was clear.
    pub walls_ok: bool,
    /// Whether the order was taken.
    pub order_ok: bool,
    /// The rows, from *t* + 1 to *t* + [`SPAN`].
    pub rows: Vec<Row>,
}

/// The ground with the record's ring of wall sections added to the map
/// (measure: M-0023, Setup, Map), the wall type's overlay with data 0.
pub fn walled_ground(ground: &Ground, local: &Local) -> Result<Ground, String> {
    // measure: M-0023 (Setup, Map: one stock wall type with the wall flag, an overlay type, S-0211 claim 11 rule 2)
    let i = ground
        .rules
        .overlays
        .iter()
        .position(|o| o.name.eq_ignore_ascii_case(&local.wall))
        .ok_or_else(|| format!("{ID}: no overlay type {}", local.wall))?;
    if !ground.rules.overlays[i].wall {
        return Err(format!("{ID}: the wall type has no wall flag"));
    }
    let ov = u8::try_from(i).map_err(|_| format!("{ID}: overlay index out of range"))?;
    let mut g = ground.clone();
    for c in ring(local.centre) {
        let cell = g
            .map
            .cells
            .get_mut(c)
            .ok_or_else(|| format!("{ID}: ring cell off the map"))?;
        cell.overlay = ov;
        cell.overlay_data = 0;
    }
    Ok(g)
}

/// Whether `c` lists `k` on the ground layer.
fn holds(w: &World, c: CellXY, k: ObjectId) -> bool {
    w.cells.list(c, Layer::Ground).contains(&k)
}

fn row(w: &World, u: ObjectId, k: ObjectId) -> Option<Row> {
    let o = w.obj(u)?;
    Some(Row {
        tick: w.tick - 1,
        current: o.beh.current,
        queued: o.beh.queued,
        dest: o.dest,
        pos: o.pos,
        on_k: holds(w, o.pos.cell(), k),
        health: o.health,
    })
}

/// A move order to the pen's centre, taken and applied at once, outside a
/// tick's update (measure: M-0023, Setup: the harness's ordinary move
/// order given directly to the unit).
fn order(w: &mut World, u: ObjectId, centre: CellXY) -> bool {
    let o = Order::Behaviour {
        actor: u,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(centre)),
    };
    let ok = orders::issue(w, oracle::HUMAN, o);
    orders::apply_pending(w);
    ok
}

/// Plays one arm on the walled ground.
pub fn play(g: &Ground, local: &Local, arm: Arm) -> Result<Recording, String> {
    let rules = g.rules.clone();
    let country = |want: &str| {
        rules
            .country_names
            .iter()
            .find(|n| n.eq_ignore_ascii_case(want))
            .cloned()
            .ok_or_else(|| format!("{ID}: no country {want}"))
    };
    // measure: M-0023 (Setup, Houses: the human player and two computer houses, one of each side)
    let players = vec![
        Player {
            name: "Human".into(),
            country: country(&local.countries[0])?,
            start: Some(oracle::HUMAN_START),
            ai: false,
            values: HouseValues::default(),
        },
        Player {
            name: "Computer 1".into(),
            country: country(&local.countries[1])?,
            start: Some(oracle::STARTS[0]),
            ai: true,
            values: HouseValues::default(),
        },
        Player {
            name: "Computer 2".into(),
            country: country(&local.countries[2])?,
            start: Some(oracle::STARTS[1]),
            ai: true,
            values: HouseValues::default(),
        },
    ];
    // measure: M-0023 (Setup, Game: superweapons, crates and the short game off; bases on)
    let settings = Settings {
        credits: Some(CREDITS),
        short_game: Some(false),
        superweapons: Some(false),
        crates: Some(false),
        ..Default::default()
    };
    let seed = oracle::game_seed(SEED).ok_or("seed")?;
    let mut gs = setup::skirmish(&rules, &players, settings, 0, seed);
    gs.bases = true;
    gs.game_speed = oracle::FASTEST;
    for h in [1, 2] {
        gs.houses[h].human = false;
    }
    let mut w = glasnost_sim::load::new_game(rules.clone(), &g.map, Arc::clone(&g.tiles), &gs)
        .map_err(|e| format!("{e:?}"))?;
    // measure: M-0023 (Setup: every starting object and every object of a house that is not a player removed)
    let all: Vec<ObjectId> = w.objects.keys().copied().collect();
    for id in all {
        glasnost_sim::objects::take_out_quietly(&mut w, id);
    }
    let walls_ok = ring(local.centre).iter().all(|&c| {
        w.cells
            .cell(c)
            .is_some_and(|x| x.overlay != glasnost_map::NO_OVERLAY)
    }) && w
        .cells
        .cell(local.centre)
        .is_some_and(|x| x.overlay == glasnost_map::NO_OVERLAY)
        && w.cells.list(local.centre, Layer::Ground).is_empty();
    // measure: M-0023 (Setup, Houses: K and the human's power plant, one power plant for each computer house; the buildings created on their stated cells before tick 0)
    let k = glasnost_sim::testkit::spawn(&mut w, Kind::Building, &local.factory, 0, local.k, 0);
    for (i, (h, ty)) in [
        (0, &local.plants[0]),
        (1, &local.plants[0]),
        (2, &local.plants[1]),
    ]
    .into_iter()
    .enumerate()
    {
        glasnost_sim::testkit::spawn(&mut w, Kind::Building, ty, h, local.plant_cells[i], 0);
    }
    while w.tick < RELEASE_AT {
        glasnost_sim::tick::step(&mut w);
    }
    let ti = rules
        .index_of(Kind::Infantry, &local.infantry)
        .ok_or_else(|| format!("{ID}: no infantry type {}", local.infantry))?;
    let t = w.tick - 1;
    let u = glasnost_sim::objects::create(&mut w, Kind::Infantry, ti, Some(HouseId(0)));
    if release::release(&mut w, k, u) != Outcome::Released {
        return Err(format!("{ID}: the release failed"));
    }
    let release_pos = w.obj(u).ok_or("no unit")?.pos;
    let mut order_ok = arm == Arm::None;
    if arm == Arm::Now {
        order_ok = order(&mut w, u, local.centre);
    }
    let mut rows = Vec::new();
    for _ in 0..SPAN {
        if arm == Arm::After && w.tick == t + 2 {
            order_ok = order(&mut w, u, local.centre);
        }
        glasnost_sim::tick::step(&mut w);
        rows.push(row(&w, u, k).ok_or("the unit is gone")?);
    }
    Ok(Recording {
        arm,
        t,
        release_pos,
        walls_ok,
        order_ok,
        rows,
    })
}

fn dest_cell(d: Option<Target>, w_cells: impl Fn(ObjectId) -> Option<CellXY>) -> Option<CellXY> {
    match d? {
        Target::Cell(c) => Some(c),
        Target::Object(o) => w_cells(o),
    }
}

/// The judged quantities of one arm (measure: M-0023, Procedure step 4).
#[derive(Clone, Debug, Default)]
pub struct Derived {
    /// *F* − *t*.
    pub f: Option<i32>,
    /// *G* − *F* (the first later tick at whose end the unit holds a
    /// destination: our analogue of the baseline's request-turn draws,
    /// which our recording does not tie to the record's sources).
    pub g: Option<i32>,
    /// *W* − *t*.
    pub w: Option<i32>,
    /// The baseline's request turn − *t*.
    pub request: Option<i32>,
}

fn derive(r: &Recording) -> Derived {
    let rel = |row: &Row| row.tick - r.t;
    let mut d = Derived::default();
    if r.arm == Arm::Now {
        let f = r.rows.iter().position(|x| x.dest.is_none());
        if let Some(fi) = f {
            d.f = Some(rel(&r.rows[fi]));
            d.g = r.rows[fi + 1..]
                .iter()
                .find(|x| x.dest.is_some())
                .map(|x| rel(x) - rel(&r.rows[fi]));
        }
    } else {
        d.request = r.rows.iter().find(|x| x.dest.is_some()).map(rel);
    }
    d.w = r.rows.iter().find(|x| x.pos != r.release_pos).map(rel);
    d
}

fn yes(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}

/// Plays the three arms and judges each Result row. Returns the summary
/// (no local values) and the full report (with them).
pub fn run(ground: &Ground, local: &Local) -> Result<(String, String), String> {
    let g = walled_ground(ground, local)?;
    let now = play(&g, local, Arm::Now)?;
    let after = play(&g, local, Arm::After)?;
    let none = play(&g, local, Arm::None)?;
    let (dn, da, db) = (derive(&now), derive(&after), derive(&none));
    let at = |r: &Recording, k: i32| r.rows.iter().find(|x| x.tick == r.t + k).cloned();
    let rowf = |r: &Recording, rel: Option<i32>| rel.and_then(|k| at(r, k));
    let k_cells = |_o: ObjectId| None;
    let mut rows: Vec<(String, bool, String)> = Vec::new();
    let mut push = |name: &str, ok: bool, ours: String| rows.push((name.to_owned(), ok, ours));
    // Void rules of step 2 (those that apply to our recording).
    let void = |r: &Recording, f: Option<i32>| {
        let first = at(r, 1);
        r.walls_ok
            && r.order_ok
            && first.is_some_and(|x| x.on_k)
            && r.rows.iter().all(|x| x.health == r.rows[0].health)
            && (r.arm != Arm::Now
                || f.is_some_and(|f| {
                    r.rows
                        .iter()
                        .take_while(|x| x.tick <= r.t + f)
                        .all(|x| x.on_k)
                }))
    };
    let v = [void(&now, dn.f), void(&after, None), void(&none, None)];
    // measure: M-0023 (Result: the void rules of step 2 pass in every arm; the P25(a) rule is run 2's and not ours)
    push(
        "Void rules of step 2",
        v.iter().all(|&b| b),
        format!(
            "now {}, controls {}, baseline {}",
            yes(v[0]),
            yes(v[1]),
            yes(v[2])
        ),
    );
    // measure: M-0023 (Result: U on a cell that holds K at t + 1, every arm)
    let on_k = [&now, &after, &none].map(|r| at(r, 1).is_some_and(|x| x.on_k));
    push(
        "U on a cell that holds K at t + 1",
        on_k.iter().all(|&b| b),
        format!("{on_k:?}"),
    );
    // measure: M-0023 (Result: at t + 1, guard current and a new destination: run 1 no; controls and baseline yes)
    let req1 = |r: &Recording| {
        at(r, 1).is_some_and(|x| x.current == Some(Behaviour::Guard) && x.dest.is_some())
    };
    let (r1n, r1a, r1b) = (req1(&now), req1(&after), req1(&none));
    push(
        "At t + 1: guard current, a new destination",
        !r1n && r1a && r1b,
        format!(
            "now {}, controls {}, baseline {}",
            yes(r1n),
            yes(r1a),
            yes(r1b)
        ),
    );
    // measure: M-0023 (Result: F − t = 1)
    push("F − t", dn.f == Some(1), format!("{:?}", dn.f));
    let fr = rowf(&now, dn.f);
    // measure: M-0023 (Result: at the end of F, move current, guard queued, no destination)
    let f_ok = fr.as_ref().is_some_and(|x| {
        x.current == Some(Behaviour::Move) && x.queued == Some(Behaviour::Guard) && x.dest.is_none()
    });
    push(
        "At the end of F: move current, guard queued, no destination",
        f_ok,
        format!("{:?}", fr.as_ref().map(|x| (x.current, x.queued, x.dest))),
    );
    // measure: M-0023 (Result: G − F = 1)
    push("G − F", dn.g == Some(1), format!("{:?}", dn.g));
    let gr = dn.f.zip(dn.g).and_then(|(f, g)| at(&now, f + g));
    // measure: M-0023 (Result: at the end of G, guard current, nothing queued, a new destination)
    let g_ok = gr.as_ref().is_some_and(|x| {
        x.current == Some(Behaviour::Guard) && x.queued.is_none() && x.dest.is_some()
    });
    push(
        "At the end of G: guard current, nothing queued, a new destination",
        g_ok,
        format!("{:?}", gr.as_ref().map(|x| (x.current, x.queued, x.dest))),
    );
    let gd = gr.as_ref().and_then(|x| dest_cell(x.dest, k_cells));
    let base_d = rowf(&none, db.request).and_then(|x| dest_cell(x.dest, k_cells));
    let ctrl_d = rowf(&after, da.request).and_then(|x| dest_cell(x.dest, k_cells));
    // The destination's cell holds K: read on the recording's world is not
    // kept, so it is judged by K's foundation from the release cell's row.
    let k_foot = |c: CellXY| {
        // K's foundation holds the release cell; a destination off K is any
        // cell the unit's rows never show as holding K at that cell.
        [&now, &after, &none]
            .iter()
            .flat_map(|r| r.rows.iter())
            .any(|x| x.pos.cell() == c && x.on_k)
    };
    // measure: M-0023 (Result: the new destination on a cell that holds K: no, in every arm)
    let on = [gd, ctrl_d, base_d].map(|d| d.is_some_and(k_foot));
    push(
        "New destination on a cell that holds K",
        on.iter().all(|&b| !b) && gd.is_some(),
        format!("{on:?}"),
    );
    // measure: M-0023 (Result: the new destination in the pen or next to it: no, in every arm)
    let pen = [gd, ctrl_d, base_d].map(|d| d.is_some_and(|c| in_or_next_to_pen(local.centre, c)));
    push(
        "New destination in the pen or next to it",
        pen.iter().all(|&b| !b),
        format!("{pen:?}"),
    );
    // measure: M-0023 (Result: steps before W: none; W is the first position change, so this holds by its definition, and is judged as no cell change before W)
    let steps = [(&now, dn.w), (&after, da.w), (&none, db.w)].map(|(r, w)| {
        w.is_some_and(|w| {
            r.rows
                .iter()
                .take_while(|x| x.tick < r.t + w)
                .all(|x| x.pos == r.release_pos)
        })
    });
    push(
        "Steps before W: none",
        steps.iter().all(|&b| b),
        format!("{steps:?}"),
    );
    // measure: M-0023 (Result: the first step 1 tick after G; the baseline's 1 tick after its request turn; the controls' as the baseline's)
    let lag_now = dn.w.zip(dn.f.zip(dn.g)).map(|(w, (f, g))| w - f - g);
    let lag_base = db.w.zip(db.request).map(|(w, q)| w - q);
    let lag_ctrl = da.w.zip(da.request).map(|(w, q)| w - q);
    push(
        "First step after G (the baseline: after its request turn)",
        lag_now == Some(1) && lag_base == Some(1) && lag_ctrl == Some(1),
        format!("now {lag_now:?}, controls {lag_ctrl:?}, baseline {lag_base:?}"),
    );
    // measure: M-0023 (Result: U enters the pen within 300 ticks: no, every arm)
    let entered = [&now, &after, &none].map(|r| {
        r.rows.iter().any(|x| {
            (x.pos.cell().x - local.centre.x).abs() <= 1
                && (x.pos.cell().y - local.centre.y).abs() <= 1
        })
    });
    push(
        "U enters the pen within 300 ticks",
        entered.iter().all(|&b| !b),
        format!("{entered:?}"),
    );
    // measure: M-0023 (Result, Controls: positions identical to the baseline's tick for tick; from t + 2 the destination reads the pen's centre and move is current)
    let same_pos = after
        .rows
        .iter()
        .zip(none.rows.iter())
        .all(|(a, b)| a.pos == b.pos);
    let c2 = at(&after, 2).is_some_and(|x| {
        x.dest == Some(Target::Cell(local.centre)) && x.current == Some(Behaviour::Move)
    });
    push(
        "Controls: positions as the baseline's; at t + 2 the pen's centre and move",
        same_pos && c2,
        format!("positions {}, t + 2 {}", yes(same_pos), yes(c2)),
    );

    // measure: M-0023 (Also recorded, outside the verdict: the arrival tick of step 5, the first tick from which the position does not change again within the record)
    let arrival = |r: &Recording| {
        let last = r.rows.last()?.pos;
        let i = r
            .rows
            .iter()
            .rposition(|x| x.pos != last)
            .map_or(0, |i| i + 1);
        Some(i)
    };
    let mut also: Vec<(String, bool)> = Vec::new();
    // measure: M-0023 (Also recorded, Controls: at the end of the arrival tick the destination still reads the pen's centre; cleared in the next tick with move current and guard queued; guard current in the tick after that)
    let ctrl = arrival(&after).map(|i| {
        let at = |k: usize| after.rows.get(i + k);
        let kept = at(0).is_some_and(|x| x.dest == Some(Target::Cell(local.centre)));
        let cleared = at(1).is_some_and(|x| {
            x.dest.is_none()
                && x.current == Some(Behaviour::Move)
                && x.queued == Some(Behaviour::Guard)
        });
        let guard = at(2).is_some_and(|x| x.current == Some(Behaviour::Guard));
        (kept, cleared, guard)
    });
    also.push((
        "Controls: the pen's centre kept through the arrival tick".into(),
        ctrl.is_some_and(|c| c.0),
    ));
    also.push((
        "Controls: cleared the next tick, move current and guard queued".into(),
        ctrl.is_some_and(|c| c.1),
    ));
    also.push((
        "Controls: guard current the tick after that".into(),
        ctrl.is_some_and(|c| c.2),
    ));
    // measure: M-0023 (Also recorded, Baseline: a single step from the release position to a standing position in a cell next to the release cell, changing in every tick with no stop, then no change; the destination cleared in its arrival tick)
    let single = db.w.zip(arrival(&none)).is_some_and(|(w, i)| {
        let first = none.rows.iter().position(|x| x.tick == none.t + w);
        let end = none.rows[i].pos.cell();
        let from = none.release_pos.cell();
        first.is_some_and(|f| (f..=i).all(|k| k == 0 || none.rows[k].pos != none.rows[k - 1].pos))
            && (end.x - from.x).abs() <= 1
            && (end.y - from.y).abs() <= 1
            && end != from
    });
    also.push(("Baseline: a single step".into(), single));
    let base_cleared = arrival(&none).is_some_and(|i| {
        none.rows[i].dest.is_none() && (i == 0 || none.rows[i - 1].dest.is_some())
    });
    also.push((
        "Baseline: the destination cleared in its arrival tick".into(),
        base_cleared,
    ));

    let failing = rows.iter().filter(|r| !r.1).count();
    let mut summary = String::new();
    let _ = writeln!(summary, "| Row | Ours holds |");
    let _ = writeln!(summary, "|---|---|");
    for (name, ok, _) in &rows {
        let _ = writeln!(summary, "| {name} | {} |", yes(*ok));
    }
    let _ = writeln!(summary, "{ID}: rows failing: {failing} of {}", rows.len());
    let _ = writeln!(summary, "Also recorded, outside the verdict:");
    for (name, ok) in &also {
        let _ = writeln!(summary, "- {name}: ours as recorded: {}", yes(*ok));
    }
    let mut full = summary.clone();
    let _ = writeln!(full, "\nOur readings (local values; never commit):");
    for (name, _, ours) in &rows {
        let _ = writeln!(full, "- {name}: {ours}");
    }
    let _ = writeln!(
        full,
        "- run 1's new destination: ours {gd:?}, the record's {:?}",
        local.dest_now
    );
    let _ = writeln!(
        full,
        "- the baseline's destination: ours {base_d:?}, the record's {:?}; the controls' ours {ctrl_d:?}",
        local.dest_base
    );
    let _ = writeln!(
        full,
        "- W − t: now {:?}, controls {:?}, baseline {:?}",
        dn.w, da.w, db.w
    );
    Ok((summary, full))
}
