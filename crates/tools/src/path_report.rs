//! The pathing-report tool (M8 A4, `glasnost-headless path-report FILE`):
//! reads a pathing report the client saved
//! ([`glasnost_app::game::capture::Bundle`]), checks its build and hashes,
//! replays the window from its snapshot with its orders and the movement
//! trace on (`glasnost_sim::trace`), confirms the state hash at the capture
//! and explains what the units of interest did:
//!
//! - `report.txt`: the setup, the capture and the marks, the selection and
//!   view, then per unit of interest (the selected, those ordered in the
//!   window, and any `--unit`) a timeline: orders, path computations
//!   (start, goal, the cells found or why none, the cell searches' sizes
//!   and costs, group spreading), re-paths and their likely reason, refused
//!   steps and what stood in the way, give-ups and give-way requests,
//!   locomotor state changes, and the cell it was in tick by tick (as runs).
//! - `paths.png`: a top-down picture of the cells around them, drawn from
//!   the simulation's state only (no game art): passability colours,
//!   buildings, units, the paths searched as lines, each unit's actual
//!   track, the refused steps' cells, with a legend.
//! - with `--tick T`, `state-T.txt`: every field of each unit of interest
//!   when the world reached tick T.
//!
//! The text names units by id, kind, type index and house, never by type
//! name unless `--names` asks (so a lane run's text carries no game data).
//! Our own tool; no spec applies.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use glasnost_ai::game::Game;
use glasnost_app::game::capture::{self, Bundle};
use glasnost_map::CellXY;
use glasnost_sim::occupancy::Outcome;
use glasnost_sim::orders::Order;
use glasnost_sim::passability::category;
use glasnost_sim::path::Search;
use glasnost_sim::rules::Kind;
use glasnost_sim::trace::{self, Event, What};
use glasnost_sim::world::{Object, ObjectId, Target, World};

use crate::desync::Inputs;
use crate::inspect;

/// The command line.
pub const USAGE: &str =
    "path-report FILE [--out DIR] [--unit ID]... [--tick T] [--names] [--any-build]";

/// The most units of interest followed.
// nocite: our own tool (a readable report)
const MOST_UNITS: usize = 32;

/// The most cells of a found path listed in the text.
// nocite: our own tool (a readable report)
const CELLS_LISTED: usize = 24;

/// What the tool was asked.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Options {
    /// The report file.
    pub file: PathBuf,
    /// The output folder (default: next to the file, named after it).
    pub out: Option<PathBuf>,
    /// Units to follow besides the selected and the ordered.
    pub units: Vec<u32>,
    /// The tick whose full unit state to dump.
    pub tick: Option<i32>,
    /// Name types (off: kinds and type indices only).
    pub names: bool,
    /// Replay a report written by another build.
    pub any_build: bool,
}

/// Reads the command line after `path-report`.
pub fn parse(args: &[String]) -> Result<Options, String> {
    let mut o = Options::default();
    let mut file = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut value = |name: &str| {
            it.next()
                .cloned()
                .ok_or_else(|| format!("{name} needs a value"))
        };
        match a.as_str() {
            "--out" => o.out = Some(PathBuf::from(value("--out")?)),
            "--unit" => {
                let v = value("--unit")?;
                o.units.push(
                    v.trim_start_matches('#')
                        .parse()
                        .map_err(|_| format!("--unit {v}: not an object id"))?,
                );
            }
            "--tick" => {
                let v = value("--tick")?;
                o.tick = Some(v.parse().map_err(|_| format!("--tick {v}: not a tick"))?);
            }
            "--names" => o.names = true,
            "--any-build" => o.any_build = true,
            f if !f.starts_with("--") && file.is_none() => file = Some(PathBuf::from(f)),
            other => return Err(format!("unknown argument {other}")),
        }
    }
    o.file = file.ok_or("no report file given")?;
    Ok(o)
}

/// Why a report could not be explained: the report is unreadable or does
/// not match this build or the inputs (exit code 2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refused(pub String);

impl std::fmt::Display for Refused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// One tick of a unit: where it was and its state.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Sample {
    tick: i32,
    on_map: bool,
    cell: CellXY,
    deck: bool,
    state: String,
}

/// A trace event with what the world said about its objects then.
#[derive(Clone, Debug)]
struct Noted {
    event: Event,
    /// The blockers of a step check, described then.
    blockers: Vec<String>,
    /// For a moving-blocker outcome with nothing listed in the cell: the
    /// vehicles that held the cell's vehicle-present flag at the end of the
    /// tick, described then.
    flag: Vec<String>,
}

/// What a replay found.
pub struct Analysis {
    /// The report's text.
    pub text: String,
    /// The picture (PNG).
    pub png: Vec<u8>,
    /// The `--tick` dump, when asked and reached.
    pub dump: Option<String>,
    /// Whether the replay gave the capture's state hash.
    pub reproduced: bool,
    /// The trace events recorded, for checks.
    pub events: usize,
    /// The units followed.
    pub units: Vec<u32>,
}

/// The inputs for a report: the synthetic fixture's when its rules are the
/// report's, else the install's in `game_dir` (local only).
pub fn inputs_for(b: &Bundle, game_dir: Option<&Path>) -> Result<Inputs, Refused> {
    if let Some(i) = Inputs::synthetic(&b.rules_hash) {
        return Ok(i);
    }
    match game_dir {
        Some(dir) => Inputs::install(dir, &b.map_hash, &b.rules_hash).map_err(Refused),
        None => Err(Refused(format!(
            "the report's rules ({}) are not the synthetic fixture's: set {} to the install it was played with",
            b.rules_hash,
            crate::local_lane::GAME_DIR_VAR
        ))),
    }
}

/// Reads a report file.
pub fn read(path: &Path) -> Result<Bundle, Refused> {
    let bytes = std::fs::read(path).map_err(|e| Refused(format!("{}: {e}", path.display())))?;
    Bundle::from_bytes(&bytes).map_err(|e| Refused(format!("{}: {e}", path.display())))
}

/// The output folder for a report file: `--out`, else `<file stem>/` next
/// to it.
pub fn out_dir(o: &Options) -> PathBuf {
    o.out.clone().unwrap_or_else(|| {
        let stem = o.file.file_stem().map_or_else(
            || "path-report".into(),
            |s| s.to_string_lossy().into_owned(),
        );
        o.file.with_file_name(stem)
    })
}

/// `glasnost-headless path-report`: reads, replays, writes the outputs.
/// Returns the text, the folder written and whether the capture was
/// reproduced.
pub fn run(o: &Options, game_dir: Option<&Path>) -> Result<(Analysis, PathBuf), Refused> {
    let b = read(&o.file)?;
    let inputs = inputs_for(&b, game_dir)?;
    let a = analyse(&b, &inputs, o)?;
    let out = out_dir(o);
    std::fs::create_dir_all(&out)
        .map_err(|e| Refused(format!("cannot make {}: {e}", out.display())))?;
    let write = |name: &str, bytes: &[u8]| {
        std::fs::write(out.join(name), bytes)
            .map_err(|e| Refused(format!("cannot write {}: {e}", out.join(name).display())))
    };
    write("report.txt", a.text.as_bytes())?;
    write("paths.png", &a.png)?;
    if let (Some(t), Some(d)) = (o.tick, &a.dump) {
        write(&format!("state-{t}.txt"), d.as_bytes())?;
    }
    Ok((a, out))
}

fn kind_name(k: Kind) -> &'static str {
    match k {
        Kind::Infantry => "infantry",
        Kind::Vehicle => "vehicle",
        Kind::Aircraft => "aircraft",
        Kind::Building => "building",
        Kind::Terrain => "terrain",
    }
}

fn cell_text(c: CellXY) -> String {
    format!("({},{})", c.x, c.y)
}

fn target_text(t: Option<Target>) -> String {
    match t {
        None => "none".into(),
        Some(Target::Cell(c)) => format!("cell {}", cell_text(c)),
        Some(Target::Object(id)) => format!("#{}", id.0),
    }
}

struct Namer {
    names: Option<BTreeMap<(Kind, u32), String>>,
}

impl Namer {
    fn object(&self, o: &Object) -> String {
        let owner = o
            .owner
            .map_or_else(|| "no house".into(), |h| format!("house {}", h.0));
        let name = self
            .names
            .as_ref()
            .and_then(|n| n.get(&(o.kind, o.type_index)))
            .map(|n| format!(" {n}"))
            .unwrap_or_default();
        format!(
            "#{} ({} type {}{name}, {owner})",
            o.id.0,
            kind_name(o.kind),
            o.type_index
        )
    }

    fn id(&self, w: &World, id: ObjectId) -> String {
        w.obj(id)
            .map_or_else(|| format!("#{} (gone)", id.0), |o| self.object(o))
    }
}

/// A short text of the unit's movement state: behaviour, destination and
/// its locomotor's own state.
fn state_of(o: &Object) -> String {
    let mut s = match o.beh.current {
        Some(b) => format!("behaviour {b:?}, dest {}", target_text(o.dest)),
        None => format!("no behaviour, dest {}", target_text(o.dest)),
    };
    if let Some(d) = &o.drive {
        let _ = write!(
            s,
            ", drive: {}{}{}{}",
            if d.track.is_some() {
                "on a track"
            } else {
                "no track"
            },
            if d.turning_in_place {
                ", turning in place"
            } else {
                ""
            },
            if d.between_steps {
                ", between steps"
            } else {
                ""
            },
            if d.goal.is_some() { ", has a goal" } else { "" },
        );
    }
    if let Some(wk) = &o.walk {
        let _ = write!(
            s,
            ", walk: {}{}",
            if wk.target.is_some() {
                "walking to a point"
            } else {
                "standing"
            },
            if wk.arrived.is_some() {
                ", arrived"
            } else {
                ""
            },
        );
    }
    if let Some(h) = &o.hover {
        let _ = write!(
            s,
            ", hover: {}{}",
            if h.point.is_some() {
                "to a point"
            } else {
                "no point"
            },
            if h.between { ", between cells" } else { "" },
        );
    }
    if !o.path.steps.is_empty() {
        let _ = write!(s, ", {} path steps", o.path.steps.len());
    }
    if let Some(since) = o.path.blocked_since {
        let _ = write!(s, ", blocked since {since}");
    }
    s
}

fn sample(w: &World, id: ObjectId) -> Option<Sample> {
    let o = w.obj(id)?;
    Some(Sample {
        tick: w.tick,
        on_map: o.on_map,
        cell: o.pos.cell(),
        deck: o.layer == glasnost_sim::world::Layer::Deck,
        state: state_of(o),
    })
}

/// The cells a stored path visits from `from` (direction `d` steps by
/// `glasnost_sim::regions::STEPS[d]`).
fn path_cells(from: CellXY, steps: &[u8]) -> Vec<CellXY> {
    let mut c = from;
    let mut out = vec![c];
    for &d in steps {
        let (dx, dy) = glasnost_sim::regions::STEPS[usize::from(d % 8)];
        c = CellXY::new(c.x + dx, c.y + dy);
        out.push(c);
    }
    out
}

fn outcome_text(o: Outcome) -> &'static str {
    match o {
        Outcome::Clear => "clear",
        Outcome::CloakedBlocker => "a cloaked blocker",
        Outcome::MovingBlocker => "a moving blocker",
        Outcome::ClosedFriendlyGate => "a closed friendly gate",
        Outcome::FriendlyObstacle => "a friendly obstacle",
        Outcome::EnemyObstacle => "an enemy obstacle",
        Outcome::StationaryFriendly => "a stationary friendly unit",
        Outcome::Blocked => "blocked (terrain or a building)",
    }
}

fn search_text(s: Search) -> &'static str {
    match s {
        Search::Found => "found a path",
        Search::GateFailed => "failed: the destination is not reachable (the gate refused it)",
        Search::SearchFailed => "failed: the cell search found no path",
    }
}

fn order_text(o: &Order) -> String {
    match o {
        Order::Behaviour {
            behaviour,
            attack_move,
            target,
            dest,
            ..
        } => format!(
            "{behaviour:?}{} target {} dest {}",
            if *attack_move { " (attack-move)" } else { "" },
            target_text(*target),
            target_text(*dest)
        ),
        Order::GuardSpot { spot, .. } => format!("guard spot {}", target_text(Some(*spot))),
        Order::Stop { .. } => "stop".into(),
        Order::Scatter { .. } => "scatter".into(),
        Order::Deploy { .. } => "deploy".into(),
        other => format!("{other:?}"),
    }
}

/// Replays `b` with the trace on and explains it.
pub fn analyse(b: &Bundle, inputs: &Inputs, o: &Options) -> Result<Analysis, Refused> {
    let this = capture::build_id();
    if b.build != this && !o.any_build {
        return Err(Refused(format!(
            "the report was written by {}; this tool is {this}. Replay it with the build that wrote it, or pass --any-build to try anyway",
            b.build
        )));
    }
    if inputs.layers != b.layers {
        return Err(Refused(format!(
            "the report's game was played with {}, the rules found have {}",
            b.layers.describe(),
            inputs.layers.describe()
        )));
    }
    if inputs.rules.hash != b.rules_hash {
        return Err(Refused(format!(
            "the rules given hash to {}, the report's to {}",
            inputs.rules.hash, b.rules_hash
        )));
    }
    let mut g = Game::load(&b.snapshot, inputs.rules.clone(), inputs.catalogue.clone())
        .map_err(|e| Refused(format!("the report's snapshot does not load: {e}")))?;
    if g.replay().map != b.map_hash {
        return Err(Refused(format!(
            "the snapshot's map is {}, the report names {}",
            g.replay().map,
            b.map_hash
        )));
    }
    let setup = glasnost_net::start::setup_hash(&g.replay().setup);
    if setup != b.setup_hash {
        return Err(Refused(format!(
            "the snapshot's setup hashes to {setup}, the report's to {}",
            b.setup_hash
        )));
    }
    let ran = glasnost_sim::tick::ticks_run(&g.world);
    if ran != b.snapshot_ticks_run || g.world.tick != b.snapshot_tick {
        return Err(Refused(format!(
            "the snapshot is at tick {} ({ran} run), the report says tick {} ({} run)",
            g.world.tick, b.snapshot_tick, b.snapshot_ticks_run
        )));
    }
    let namer = Namer {
        names: o.names.then(|| {
            inputs
                .rules
                .names
                .iter()
                .map(|((k, name), &i)| ((*k, i), name.clone()))
                .collect()
        }),
    };

    // The units of interest: the selected, the ordered, the asked for.
    let mut why: BTreeMap<u32, Vec<&str>> = BTreeMap::new();
    for &id in &b.look.selection {
        why.entry(id).or_default().push("selected");
    }
    for (_, _, order) in &b.orders {
        if let Some(a) = order.actor() {
            let w = why.entry(a.0).or_default();
            if !w.contains(&"ordered") {
                w.push("ordered");
            }
        }
    }
    for &id in &o.units {
        why.entry(id).or_default().push("asked for");
    }
    let mut units: Vec<u32> = why.keys().copied().collect();
    let dropped = units.len().saturating_sub(MOST_UNITS);
    units.truncate(MOST_UNITS);
    let ids: Vec<ObjectId> = units.iter().map(|&u| ObjectId(u)).collect();

    let mut text = String::new();
    let start_world = g.world.clone();
    let mut samples: BTreeMap<u32, Vec<Sample>> = BTreeMap::new();
    let mut noted: Vec<Noted> = Vec::new();
    let mut refused_orders = 0usize;
    let mut dump = None;
    let dump_at = |g: &Game, dump: &mut Option<String>| {
        if o.tick == Some(g.world.tick) && dump.is_none() {
            *dump = Some(dump_units(&g.world, &ids, &namer));
        }
    };
    for &id in &ids {
        if let Some(s) = sample(&g.world, id) {
            samples.entry(id.0).or_default().push(s);
        }
    }
    trace::switch_on(&mut g.world);
    let mut next = 0;
    loop {
        dump_at(&g, &mut dump);
        let t = g.world.tick;
        while next < b.orders.len() && b.orders[next].0 <= t {
            let (ot, house, order) = &b.orders[next];
            if *ot == t && !g.issue(*house, order.clone()) {
                refused_orders += 1;
            }
            next += 1;
        }
        note_events(&g.world, &namer, &mut noted);
        if glasnost_sim::tick::ticks_run(&g.world) >= b.capture_ticks_run {
            break;
        }
        g.step();
        note_events(&g.world, &namer, &mut noted);
        for &id in &ids {
            if let Some(s) = sample(&g.world, id) {
                samples.entry(id.0).or_default().push(s);
            }
        }
    }
    let hash = g.state_hash();
    let reproduced = hash == b.capture_hash && g.world.tick == b.capture_tick;
    let events = noted.len();

    // The heading.
    let _ = writeln!(text, "Pathing report");
    let _ = writeln!(text, "Written by: {}", b.build);
    if b.build != this {
        let _ = writeln!(text, "Replayed by: {this} (another build: --any-build)");
    }
    let _ = writeln!(
        text,
        "Game: {}, game speed index {}, the person's house {}",
        if b.network {
            "network game"
        } else {
            "skirmish"
        },
        b.look.speed,
        b.look
            .local
            .map_or_else(|| "none (a watch game)".to_string(), |h| h.to_string())
    );
    let _ = writeln!(text, "Rules: {}", b.layers.describe());
    let _ = writeln!(
        text,
        "Window: the snapshot at tick {} to the capture at tick {}: {} ticks, {} orders of persons",
        b.snapshot_tick,
        b.capture_tick,
        b.capture_ticks_run - b.snapshot_ticks_run,
        b.orders.len()
    );
    if reproduced {
        let _ = writeln!(
            text,
            "Replay: reproduced; the state hash at tick {} is the capture's ({}).",
            b.capture_tick,
            &b.capture_hash[..b.capture_hash.len().min(16)]
        );
    } else {
        let _ = writeln!(
            text,
            "Replay: NOT reproduced; at tick {} this build computes {}, the capture recorded {} at tick {}. What follows is this build's replay, not necessarily what the person saw.",
            g.world.tick,
            &hash[..16],
            &b.capture_hash[..b.capture_hash.len().min(16)],
            b.capture_tick
        );
    }
    if refused_orders > 0 {
        let _ = writeln!(
            text,
            "Orders the replay refused: {refused_orders} (the game had taken them)"
        );
    }
    let marks: Vec<String> = b.marks.iter().map(|m| m.to_string()).collect();
    let _ = writeln!(
        text,
        "Marks: {}",
        if marks.is_empty() {
            "none".into()
        } else {
            format!("ticks {}", marks.join(", "))
        }
    );
    let sel: Vec<String> = b.look.selection.iter().map(|i| format!("#{i}")).collect();
    let _ = writeln!(
        text,
        "Selection: {}",
        if sel.is_empty() {
            "none".into()
        } else {
            sel.join(" ")
        }
    );
    let v = &b.look.view;
    let corners: Vec<String> = v
        .corners
        .iter()
        .map(|c| c.map_or_else(|| "-".into(), cell_text))
        .collect();
    let _ = writeln!(
        text,
        "View: origin ({},{}) size {}x{}, corner cells {}; pointer cell {}",
        v.origin.0,
        v.origin.1,
        v.size.0,
        v.size.1,
        corners.join(" "),
        b.look
            .cursor
            .map_or_else(|| "off the battlefield".into(), cell_text)
    );
    for c in &b.look.clicks {
        let _ = writeln!(
            text,
            "Click: tick {} {} at {}{}",
            c.tick,
            if c.right { "right" } else { "left" },
            cell_text(c.cell),
            c.object.map(|i| format!(" on #{i}")).unwrap_or_default()
        );
    }
    let _ = writeln!(text, "Trace events recorded: {events}");
    let listed: Vec<String> = units
        .iter()
        .map(|u| format!("#{u} ({})", why[u].join(", ")))
        .collect();
    let _ = writeln!(
        text,
        "Units of interest: {}{}",
        if listed.is_empty() {
            "none (nothing selected or ordered; pass --unit ID)".into()
        } else {
            listed.join(", ")
        },
        if dropped > 0 {
            format!(" and {dropped} more not followed")
        } else {
            String::new()
        }
    );

    // The summary of blockages.
    let mut summary = String::new();
    for &u in &units {
        let mut by: BTreeMap<String, (usize, i32, i32, &'static str)> = BTreeMap::new();
        for n in noted.iter().filter(|n| n.event.unit.0 == u) {
            if let What::Step { outcome, .. } = n.event.what {
                for bl in &n.blockers {
                    let e = by.entry(bl.clone()).or_insert((
                        0,
                        n.event.tick,
                        n.event.tick,
                        outcome_text(outcome),
                    ));
                    e.0 += 1;
                    e.2 = n.event.tick;
                }
                for holder in &n.flag {
                    let e = by
                        .entry(format!(
                            "nothing listed in the cell; its vehicle-present flag held by {holder}"
                        ))
                        .or_insert((0, n.event.tick, n.event.tick, outcome_text(outcome)));
                    e.0 += 1;
                    e.2 = n.event.tick;
                }
                if n.blockers.is_empty() && n.flag.is_empty() {
                    let e = by.entry("nothing listed in the cell".into()).or_insert((
                        0,
                        n.event.tick,
                        n.event.tick,
                        outcome_text(outcome),
                    ));
                    e.0 += 1;
                    e.2 = n.event.tick;
                }
            }
        }
        for (blocker, (n, a, z, why)) in by {
            let _ = if n == 1 {
                writeln!(
                    summary,
                    "  #{u} was refused a step once, at tick {a}, by {why}: {blocker}"
                )
            } else {
                writeln!(
                    summary,
                    "  #{u} was refused a step {n} times, ticks {a} to {z}, by {why}: {blocker}"
                )
            };
        }
    }
    let _ = writeln!(text);
    if summary.is_empty() {
        let _ = writeln!(
            text,
            "Blockages: none of the units of interest was refused a step."
        );
    } else {
        let _ = writeln!(text, "Blockages:");
        text.push_str(&summary);
    }

    // Each unit.
    for &u in &units {
        let id = ObjectId(u);
        let _ = writeln!(text);
        let heading = start_world
            .obj(id)
            .or_else(|| g.world.obj(id))
            .map_or_else(|| format!("#{u} (not in the game)"), |ob| namer.object(ob));
        let _ = writeln!(text, "== {heading}: {}", why[&u].join(", "));
        let ss = samples.get(&u).map(Vec::as_slice).unwrap_or(&[]);
        if let Some(first) = ss.first() {
            let _ = writeln!(
                text,
                "At the snapshot (tick {}): cell {}{}, {}",
                first.tick,
                cell_text(first.cell),
                if first.deck { " on a bridge deck" } else { "" },
                first.state
            );
        }
        if let Some(last) = ss.last() {
            let _ = writeln!(
                text,
                "At the capture (tick {}): cell {}{}, {}",
                last.tick,
                cell_text(last.cell),
                if last.deck { " on a bridge deck" } else { "" },
                last.state
            );
        }
        // The track as runs.
        let mut runs: Vec<String> = Vec::new();
        let mut i = 0;
        while i < ss.len() {
            let mut j = i;
            while j + 1 < ss.len()
                && ss[j + 1].cell == ss[i].cell
                && ss[j + 1].on_map == ss[i].on_map
            {
                j += 1;
            }
            runs.push(format!(
                "{}{} ticks {}-{}",
                cell_text(ss[i].cell),
                if ss[i].on_map { "" } else { " (off the map)" },
                ss[i].tick,
                ss[j].tick
            ));
            i = j + 1;
        }
        let _ = writeln!(text, "Track: {}", runs.join("; "));

        // The timeline.
        let _ = writeln!(text, "Timeline:");
        let mut lines: Vec<(i32, usize, String)> = Vec::new();
        let mut seq = 0usize;
        let mut push = |t: i32, s: String, lines: &mut Vec<(i32, usize, String)>| {
            lines.push((t, seq, s));
            seq += 1;
        };
        for (t, house, order) in &b.orders {
            if order.actor() == Some(id) {
                push(
                    *t,
                    format!("order of house {}: {}", house.0, order_text(order)),
                    &mut lines,
                );
            }
        }
        for w in ss.windows(2) {
            if w[0].state != w[1].state {
                push(w[1].tick, format!("state: {}", w[1].state), &mut lines);
            }
        }
        let mine: Vec<&Noted> = noted.iter().filter(|n| n.event.unit == id).collect();
        let mut k = 0;
        while k < mine.len() {
            let n = mine[k];
            // A run of identical refused steps on consecutive ticks is one
            // line, with the same blocked hold after each when there is one.
            if let What::Step { .. } = n.event.what {
                let hold = |m: &Noted| match m.event.what {
                    What::Blocked {
                        since,
                        waiting_until: Some(u),
                    } => Some((since, u)),
                    _ => None,
                };
                let held = mine.get(k + 1).and_then(|m| hold(m));
                let mut z = k;
                let mut end = k;
                let mut checks = 1;
                loop {
                    let mut j = end + 1;
                    if held.is_some() {
                        if mine.get(j).and_then(|m| hold(m)) != held {
                            break;
                        }
                        j += 1;
                    }
                    if let Some(next) = mine.get(j)
                        && same_step(&next.event.what, &n.event.what)
                        && next.blockers == n.blockers
                        && next.event.tick <= mine[z].event.tick + 1
                    {
                        z = j;
                        end = j;
                        checks += 1;
                    } else {
                        break;
                    }
                }
                let mut s = event_text(n, ss);
                if checks > 1 {
                    let _ = write!(
                        s,
                        " (the same {checks} checks, to tick {})",
                        mine[z].event.tick
                    );
                }
                if let Some((since, u)) = held {
                    let _ = write!(
                        s,
                        "; blocked since tick {since}, the re-path waits until tick {u}"
                    );
                    // The last check's hold.
                    if mine.get(z + 1).and_then(|m| hold(m)) == held {
                        end = z + 1;
                    }
                }
                push(n.event.tick, s, &mut lines);
                k = end + 1;
                continue;
            }
            let mut s = event_text(n, ss);
            if let What::Search { .. } = n.event.what {
                let _ = write!(s, "; {}", search_reason(n, &mine, b, id));
            }
            push(n.event.tick, s, &mut lines);
            k += 1;
        }
        lines.sort_by_key(|l| (l.0, l.1));
        if lines.is_empty() {
            let _ = writeln!(text, "  nothing happened to its movement in the window");
        }
        for (t, _, s) in &lines {
            let _ = writeln!(text, "  tick {t}: {s}");
        }
        // Its totals.
        let count = |f: &dyn Fn(&What) -> bool| mine.iter().filter(|n| f(&n.event.what)).count();
        let _ = writeln!(
            text,
            "Totals: {} path computations ({} found), {} refused step checks, {} blocked re-path holds, {} give-ups, {} give-way requests made, {} answered",
            count(&|w| matches!(w, What::Search { .. })),
            count(&|w| matches!(
                w,
                What::Search {
                    result: Search::Found,
                    ..
                }
            )),
            count(&|w| matches!(w, What::Step { .. })),
            count(&|w| matches!(
                w,
                What::Blocked {
                    waiting_until: Some(_),
                    ..
                }
            )),
            // A give-up of a move and one after a failed computation.
            count(&|w| matches!(w, What::GaveUp | What::Failed { gave_up: true, .. })),
            count(&|w| matches!(w, What::AskedRoom { .. })),
            count(&|w| matches!(w, What::Asked { .. })),
        );
    }
    if let Some(t) = o.tick
        && dump.is_none()
    {
        let _ = writeln!(
            text,
            "\n--tick {t}: the window runs from tick {} to {}; no state dumped",
            b.snapshot_tick, b.capture_tick
        );
    } else if let Some(t) = o.tick {
        let _ = writeln!(text, "\nThe full state at tick {t} is in state-{t}.txt.");
    }
    let png = picture(&g.world, &units, &samples, &noted);
    Ok(Analysis {
        text,
        png,
        dump,
        reproduced,
        events,
        units,
    })
}

fn same_step(a: &What, b: &What) -> bool {
    match (a, b) {
        (
            What::Step {
                from: fa,
                to: ta,
                outcome: oa,
                ..
            },
            What::Step {
                from: fb,
                to: tb,
                outcome: ob,
                ..
            },
        ) => fa == fb && ta == tb && oa == ob,
        _ => false,
    }
}

/// The vehicles other than `mover` whose vehicle-present flag (S-0209
/// claim 1, driving vehicles) is set on cell `c`: a driving vehicle's
/// claimed cells (both cells of a two-cell curve) and a hovering vehicle's.
fn flag_holders(w: &World, c: CellXY, mover: ObjectId) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| o.id != mover)
        .filter(|o| {
            o.drive.as_ref().is_some_and(|d| d.claims.contains(&c))
                || o.hover
                    .as_ref()
                    .is_some_and(|h| h.claims.iter().any(|&(x, _)| x == c))
        })
        .map(|o| o.id)
        .collect()
}

/// An object, where it stands, and whether it is on the move.
fn placed(w: &World, namer: &Namer, b: ObjectId) -> String {
    let mut s = namer.id(w, b);
    if let Some(o) = w.obj(b) {
        let _ = write!(
            s,
            " at {}, {}",
            cell_text(o.pos.cell()),
            if o.dest.is_some() || !o.path.steps.is_empty() {
                "moving"
            } else {
                "standing"
            }
        );
    }
    s
}

/// Takes the trace's new events, describing their blockers from the world
/// as it is now.
fn note_events(w: &World, namer: &Namer, out: &mut Vec<Noted>) {
    for e in trace::take(w) {
        let (blockers, flag) = match &e.what {
            What::Step {
                blockers,
                to,
                outcome,
                ..
            } => {
                let listed: Vec<String> = blockers
                    .iter()
                    .filter(|&&b| b != e.unit)
                    .map(|&b| placed(w, namer, b))
                    .collect();
                // A moving blocker with nothing listed comes from the
                // cell's vehicle-present flag (S-0209 claim 5 row VR5):
                // name the vehicles that hold it.
                let flag = if listed.is_empty() && *outcome == Outcome::MovingBlocker {
                    flag_holders(w, *to, e.unit)
                        .into_iter()
                        .map(|b| placed(w, namer, b))
                        .collect()
                } else {
                    Vec::new()
                };
                (listed, flag)
            }
            _ => (Vec::new(), Vec::new()),
        };
        out.push(Noted {
            event: e,
            blockers,
            flag,
        });
    }
}

/// What the report says was in a refused step's cell.
fn in_the_cell(n: &Noted) -> String {
    match (n.blockers.is_empty(), n.flag.is_empty()) {
        (false, _) => n.blockers.join("; "),
        (true, true) => "nothing listed".into(),
        (true, false) => format!(
            "nothing listed; its vehicle-present flag held by {}",
            n.flag.join("; ")
        ),
    }
}

fn event_text(n: &Noted, _samples: &[Sample]) -> String {
    match &n.event.what {
        What::Search {
            asked,
            mode,
            from,
            goal,
            to,
            congested,
            corridor,
            searches,
            result,
            steps,
        } => {
            let mut s = format!("path computation ({asked:?}");
            if let Some(m) = mode
                && m != asked
            {
                let _ = write!(s, ", run as {m:?}");
            }
            // The trace has the cell searched for only when a path was
            // stored; a failed computation names the destination asked for,
            // and nothing is said about substitution.
            let _ = write!(
                s,
                ") from {} to {}",
                cell_text(*from),
                to.or(*goal).map_or_else(|| "no cell".into(), cell_text)
            );
            if to.is_some() && goal.is_some() && goal != to {
                let _ = write!(
                    s,
                    " (the destination {} was substituted)",
                    goal.map_or_else(String::new, cell_text)
                );
            }
            let _ = write!(s, ": {}", search_text(*result));
            if !steps.is_empty() {
                let cells = path_cells(*from, steps);
                let shown: Vec<String> = cells
                    .iter()
                    .skip(1)
                    .take(CELLS_LISTED)
                    .map(|c| cell_text(*c))
                    .collect();
                let _ = write!(
                    s,
                    ", {} steps: {}{}",
                    steps.len(),
                    shown.join(" "),
                    if cells.len() > CELLS_LISTED + 1 {
                        " …"
                    } else {
                        ""
                    }
                );
            }
            if let Some(c) = corridor {
                let _ = write!(s, "; corridor {}", if *c { "used" } else { "not used" });
            }
            for (i, cs) in searches.iter().enumerate() {
                let _ = write!(
                    s,
                    "; cell search {}{}: {} expanded, {} recorded",
                    i + 1,
                    if cs.corridor { " in the corridor" } else { "" },
                    cs.expansions,
                    cs.recorded
                );
                match (cs.found, cs.cost_bits) {
                    (Some(f), Some(c)) => {
                        let _ = write!(s, ", found {f} steps at cost {}", f32::from_bits(c));
                    }
                    _ => {
                        let _ = write!(s, ", found none");
                    }
                }
            }
            // spec: S-0212 claim 13 (the 5th failed cell search inside a corridor fails the path computation)
            if *result == Search::SearchFailed
                && searches.len() == 5
                && searches
                    .last()
                    .is_some_and(|c| c.corridor && c.found.is_none())
            {
                let _ = write!(
                    s,
                    "; the 5th cell search inside a corridor failed, which ends the computation (the cap of 5)"
                );
            }
            if !congested.is_empty() {
                let shown: Vec<String> = congested.iter().take(8).map(|c| cell_text(*c)).collect();
                let _ = write!(
                    s,
                    "; {} congested cells {}{}",
                    congested.len(),
                    shown.join(" "),
                    if congested.len() > 8 { " …" } else { "" }
                );
            }
            s
        }
        What::Failed {
            result,
            gave_up,
            dest_cleared,
            retry_at,
        } => format!(
            "after the failure ({}): {}{}, retry at tick {retry_at}",
            search_text(*result),
            if *gave_up {
                "gave up"
            } else {
                "kept its order"
            },
            if *dest_cleared {
                ", destination cleared"
            } else {
                ""
            }
        ),
        What::Step {
            mover,
            from,
            to,
            outcome,
            repeated,
            ..
        } => format!(
            "{} step {} -> {} refused by {}{}; in the cell: {}",
            match mover {
                trace::Mover::Drive => "drive",
                trace::Mover::Walk => "walk",
                trace::Mover::Hover => "hover",
            },
            cell_text(*from),
            cell_text(*to),
            outcome_text(*outcome),
            if *repeated { " (a repeated check)" } else { "" },
            in_the_cell(n)
        ),
        What::Blocked {
            since,
            waiting_until,
        } => match waiting_until {
            Some(u) => format!("blocked since tick {since}; the re-path waits until tick {u}"),
            None => format!("blocked since tick {since}; re-paths now"),
        },
        What::Waited { why } => format!("waited: {why}"),
        What::GaveUp => "gave up its move".into(),
        What::AskedRoom { cell } => format!("asked the units in {} to make room", cell_text(*cell)),
        What::Asked { accepted, dest } => format!(
            "was asked to make room: {}{}",
            if *accepted { "moves aside" } else { "stays" },
            dest.map(|c| format!(", destination {}", cell_text(c)))
                .unwrap_or_default()
        ),
        What::GaveWay { refuge, to } => format!(
            "gives way in a deadlock{}: goes to {}",
            to.map(|x| format!(" to unit {}", x.0)).unwrap_or_default(),
            cell_text(*refuge)
        ),
        What::Resumed => "resumes the order it gave way from".into(),
        What::Spread { common, cell } => format!(
            "group order: its destination {} spread to {}",
            cell_text(*common),
            cell_text(*cell)
        ),
    }
}

/// The likely reason of a path computation, from what came before it on
/// its tick.
fn search_reason(n: &Noted, mine: &[&Noted], b: &Bundle, id: ObjectId) -> &'static str {
    let t = n.event.tick;
    if b.orders
        .iter()
        .any(|(ot, _, o)| *ot == t && o.actor() == Some(id))
    {
        return "after an order this tick";
    }
    let before: Vec<&What> = mine
        .iter()
        .take_while(|m| !std::ptr::eq(**m, n))
        .filter(|m| m.event.tick == t)
        .map(|m| &m.event.what)
        .collect();
    if before.iter().any(|w| matches!(w, What::Blocked { .. })) {
        return "a re-path after a blocked step";
    }
    if before.iter().any(|w| matches!(w, What::Step { .. })) {
        return "after a refused step";
    }
    if let What::Search { asked, .. } = n.event.what
        && asked != glasnost_sim::path::Mode::Fresh
    {
        return "a re-path after the blockage delay";
    }
    let at = mine
        .iter()
        .position(|m| std::ptr::eq(*m, n))
        .unwrap_or(mine.len());
    let earlier = mine[..at]
        .iter()
        .rev()
        .find(|m| matches!(m.event.what, What::Failed { .. }));
    if let Some(Noted {
        event: Event {
            what: What::Failed { retry_at, .. },
            ..
        },
        ..
    }) = earlier
        && *retry_at <= t
    {
        return "the retry after a failed computation";
    }
    "the path ran out or the move began"
}

/// Every field of each unit, as `path = value` lines.
fn dump_units(w: &World, ids: &[ObjectId], namer: &Namer) -> String {
    let mut out = format!(
        "The units of interest when the world reached tick {}\n",
        w.tick
    );
    for &id in ids {
        let _ = writeln!(out, "\n== {}", namer.id(w, id));
        let Some(o) = w.obj(id) else {
            continue;
        };
        match inspect::view(o) {
            Ok(v) => flatten("", &v, &mut out),
            Err(e) => {
                let _ = writeln!(out, "(not viewable: {e})");
            }
        }
    }
    out
}

fn flatten(at: &str, v: &inspect::Value, out: &mut String) {
    match v {
        inspect::Value::Struct(fields) => {
            for (k, x) in fields {
                let p = if at.is_empty() {
                    (*k).to_owned()
                } else {
                    format!("{at}.{k}")
                };
                flatten(&p, x, out);
            }
        }
        inspect::Value::Just(x) if matches!(**x, inspect::Value::Struct(_)) => flatten(at, x, out),
        other => {
            let _ = writeln!(out, "{at} = {}", other.show());
        }
    }
}

// The picture.

/// Pixels per cell.
// nocite: our own tool (a readable picture)
const CELL_PX: i32 = 12;

/// Cells of margin around what the picture shows.
// nocite: our own tool (a readable picture)
const MARGIN: i32 = 5;

/// The most cells across the picture.
// nocite: our own tool (a bounded picture)
const MOST_CELLS: i32 = 120;

/// The units' colours, in turn.
const UNIT_COLOURS: [[u8; 3]; 8] = [
    [230, 40, 40],
    [40, 110, 240],
    [250, 170, 0],
    [180, 40, 220],
    [0, 190, 190],
    [240, 90, 170],
    [120, 200, 40],
    [140, 90, 40],
];

const BACKGROUND: [u8; 3] = [24, 24, 24];
const TEXT: [u8; 3] = [235, 235, 235];
const BUILDING: [u8; 3] = [120, 70, 60];
const OTHER_UNIT: [u8; 3] = [200, 200, 200];
const REFUSED: [u8; 3] = [255, 255, 0];

fn category_colour(c: u8) -> ([u8; 3], &'static str) {
    match c {
        category::OPEN => ([96, 132, 72], "open"),
        category::CRUSHABLE => ([150, 150, 90], "crushable"),
        category::WALL_LIKE => ([70, 70, 80], "wall-like"),
        category::BEACH => ([196, 180, 120], "beach"),
        category::WATER => ([50, 80, 150], "water"),
        category::PARTIAL => ([40, 85, 40], "partial obstacle"),
        category::BLOCKED => ([60, 50, 45], "blocked"),
        _ => ([0, 0, 0], "off the map"),
    }
}

struct Canvas {
    w: i32,
    h: i32,
    rgba: Vec<u8>,
}

impl Canvas {
    fn new(w: i32, h: i32) -> Self {
        let mut rgba = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..w * h {
            rgba.extend_from_slice(&[BACKGROUND[0], BACKGROUND[1], BACKGROUND[2], 255]);
        }
        Self { w, h, rgba }
    }

    fn set(&mut self, x: i32, y: i32, c: [u8; 3]) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            let i = ((y * self.w + x) * 4) as usize;
            self.rgba[i..i + 3].copy_from_slice(&c);
        }
    }

    fn fill(&mut self, x: i32, y: i32, w: i32, h: i32, c: [u8; 3]) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, c);
            }
        }
    }

    fn line(&mut self, a: (i32, i32), b: (i32, i32), c: [u8; 3], thick: i32) {
        let (mut x, mut y) = a;
        let dx = (b.0 - a.0).abs();
        let dy = -(b.1 - a.1).abs();
        let sx = if a.0 < b.0 { 1 } else { -1 };
        let sy = if a.1 < b.1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.fill(x - thick / 2, y - thick / 2, thick, thick, c);
            if (x, y) == b {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    fn text(&mut self, x: i32, y: i32, s: &str, c: [u8; 3]) {
        let mut pen = x;
        for ch in s.chars() {
            if let Some(g) = glasnost_ui::font::glyph(ch) {
                for gy in 0..glasnost_ui::font::GLYPH_ROWS {
                    for gx in 0..g.width {
                        if g.mask[(gy * g.width + gx) as usize] {
                            self.set(pen + gx, y + gy, c);
                        }
                    }
                }
            }
            pen += glasnost_ui::font::advance(ch);
        }
    }
}

fn picture(
    w: &World,
    units: &[u32],
    samples: &BTreeMap<u32, Vec<Sample>>,
    noted: &[Noted],
) -> Vec<u8> {
    // What to show: the tracks, the paths and the refused cells.
    let mut cells: BTreeSet<CellXY> = BTreeSet::new();
    let mut paths: Vec<(u32, Vec<CellXY>)> = Vec::new();
    let mut refused: BTreeSet<CellXY> = BTreeSet::new();
    for (&u, ss) in samples {
        cells.extend(ss.iter().filter(|s| s.on_map).map(|s| s.cell));
        for n in noted.iter().filter(|n| n.event.unit.0 == u) {
            match &n.event.what {
                What::Search { from, steps, .. } if !steps.is_empty() => {
                    let p = path_cells(*from, steps);
                    cells.extend(p.iter().copied());
                    paths.push((u, p));
                }
                What::Step { to, .. } => {
                    refused.insert(*to);
                    cells.insert(*to);
                }
                _ => {}
            }
        }
    }
    let (x0, y0, x1, y1) = if cells.is_empty() {
        let f = w.cells.footprint();
        let side = i32::from(f.coordinate_side());
        (0, 0, side.min(MOST_CELLS) - 1, side.min(MOST_CELLS) - 1)
    } else {
        let xs = cells.iter().map(|c| c.x);
        let ys = cells.iter().map(|c| c.y);
        let (ax, zx) = (xs.clone().min().unwrap_or(0), xs.max().unwrap_or(0));
        let (ay, zy) = (ys.clone().min().unwrap_or(0), ys.max().unwrap_or(0));
        let cx = (ax + zx) / 2;
        let cy = (ay + zy) / 2;
        let half = MOST_CELLS / 2;
        (
            (ax - MARGIN).max(cx - half),
            (ay - MARGIN).max(cy - half),
            (zx + MARGIN).min(cx + half),
            (zy + MARGIN).min(cy + half),
        )
    };
    let cols = x1 - x0 + 1;
    let rows = y1 - y0 + 1;
    let colour_of = |u: u32| {
        let i = units.iter().position(|x| *x == u).unwrap_or(0);
        UNIT_COLOURS[i % UNIT_COLOURS.len()]
    };
    // The legend's lines: a swatch (or none) and its text.
    let mut legend: Vec<(Option<[u8; 3]>, String)> = vec![
        (
            None,
            format!("Cells ({x0},{y0}) to ({x1},{y1}); x to the right, y down"),
        ),
        (
            None,
            "(the game's view turns this 45 degrees; higher ground is lighter)".into(),
        ),
    ];
    for cat in [
        category::OPEN,
        category::CRUSHABLE,
        category::WALL_LIKE,
        category::BEACH,
        category::WATER,
        category::PARTIAL,
        category::BLOCKED,
    ] {
        let (c, name) = category_colour(cat);
        legend.push((Some(c), name.into()));
    }
    legend.push((Some(BUILDING), "a building".into()));
    legend.push((Some(OTHER_UNIT), "another unit at the capture".into()));
    legend.push((
        Some(REFUSED),
        "a cross: a cell a unit of interest was refused a step into".into(),
    ));
    for &u in units {
        legend.push((
            Some(colour_of(u)),
            format!("#{u}: track (thick), searched paths (thin), place at the capture"),
        ));
    }
    let text_w = |s: &str| s.chars().map(glasnost_ui::font::advance).sum::<i32>();
    let legend_w = legend
        .iter()
        .map(|(_, s)| CELL_PX + 12 + text_w(s))
        .max()
        .unwrap_or(0);
    let line_h = glasnost_ui::font::LINE_HEIGHT + 2;
    let width = (cols * CELL_PX).max(legend_w);
    let height = rows * CELL_PX + legend.len() as i32 * line_h + 8;
    let mut cv = Canvas::new(width, height);
    let centre = |c: CellXY| {
        (
            (c.x - x0) * CELL_PX + CELL_PX / 2,
            (c.y - y0) * CELL_PX + CELL_PX / 2,
        )
    };
    // The cells.
    for y in y0..=y1 {
        for x in x0..=x1 {
            let c = CellXY::new(x, y);
            let cat = w
                .cells
                .idx(c)
                .map_or(category::OFF_MAP, |i| w.cells.category[i]);
            let (mut col, _) = category_colour(cat);
            if let Some(cell) = w.cells.cell(c)
                && cat != category::OFF_MAP
            {
                // Higher ground is lighter.
                let lift = (i32::from(cell.level) * 6).clamp(0, 60) as u8;
                for ch in &mut col {
                    *ch = ch.saturating_add(lift);
                }
            }
            let building = w
                .cells
                .list(c, glasnost_sim::world::Layer::Ground)
                .iter()
                .any(|id| w.obj(*id).is_some_and(|o| o.kind == Kind::Building));
            if building {
                col = BUILDING;
            }
            let px = (x - x0) * CELL_PX;
            let py = (y - y0) * CELL_PX;
            cv.fill(px, py, CELL_PX - 1, CELL_PX - 1, col);
        }
    }
    // The refused steps' cells.
    for &c in &refused {
        if c.x < x0 || c.x > x1 || c.y < y0 || c.y > y1 {
            continue;
        }
        let (cx, cy) = centre(c);
        let r = CELL_PX / 2 - 1;
        cv.line((cx - r, cy - r), (cx + r, cy + r), REFUSED, 2);
        cv.line((cx - r, cy + r), (cx + r, cy - r), REFUSED, 2);
    }
    let inside = |c: CellXY| c.x >= x0 && c.x <= x1 && c.y >= y0 && c.y <= y1;
    // The searched paths: thin lines in a lighter shade.
    for (u, p) in &paths {
        let c = colour_of(*u);
        let light = [
            c[0].saturating_add(70),
            c[1].saturating_add(70),
            c[2].saturating_add(70),
        ];
        for s in p.windows(2) {
            if inside(s[0]) && inside(s[1]) {
                cv.line(centre(s[0]), centre(s[1]), light, 1);
            }
        }
    }
    // The tracks: thick lines.
    for (&u, ss) in samples {
        let c = colour_of(u);
        let on: Vec<CellXY> = ss.iter().filter(|s| s.on_map).map(|s| s.cell).collect();
        for s in on.windows(2) {
            if s[0] != s[1] && inside(s[0]) && inside(s[1]) {
                cv.line(centre(s[0]), centre(s[1]), c, 3);
            }
        }
    }
    // The units at the capture: the followed in their colour, others grey.
    for o in w.objects.values() {
        if !o.on_map || matches!(o.kind, Kind::Building | Kind::Terrain) {
            continue;
        }
        let c = o.pos.cell();
        if !inside(c) {
            continue;
        }
        let (cx, cy) = centre(c);
        let followed = units.contains(&o.id.0);
        let col = if followed {
            colour_of(o.id.0)
        } else {
            OTHER_UNIT
        };
        let r = if followed { CELL_PX / 3 } else { CELL_PX / 4 };
        cv.fill(cx - r, cy - r, 2 * r, 2 * r, col);
        if followed {
            cv.fill(cx - 1, cy - 1, 2, 2, BACKGROUND);
        }
    }
    // The legend.
    let mut y = rows * CELL_PX + 4;
    for (swatch, line) in &legend {
        match swatch {
            Some(c) => {
                cv.fill(4, y, CELL_PX - 1, CELL_PX - 1, *c);
                cv.text(4 + CELL_PX + 4, y + 1, line, TEXT);
            }
            None => cv.text(4, y, line, TEXT),
        }
        y += line_h;
    }
    glasnost_app::mapview::png::png(width as u32, height as u32, &cv.rgba)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glasnost_sim::path::Mode;

    fn search_event(goal: CellXY, to: Option<CellXY>, result: Search) -> Noted {
        Noted {
            event: Event {
                tick: 1,
                unit: ObjectId(1),
                what: What::Search {
                    asked: Mode::Fresh,
                    mode: Some(Mode::Fresh),
                    from: CellXY::new(1, 1),
                    goal: Some(goal),
                    to,
                    congested: Vec::new(),
                    corridor: Some(true),
                    searches: Vec::new(),
                    result,
                    steps: if to.is_some() { vec![2] } else { Vec::new() },
                },
            },
            blockers: Vec::new(),
            flag: Vec::new(),
        }
    }

    // A failed computation names the destination asked for and says nothing
    // of substitution (pathing report 1 read "to no cell (the destination
    // ... was substituted)" for every failure).
    #[test]
    fn a_failed_computation_names_its_destination() {
        let t = event_text(
            &search_event(CellXY::new(5, 6), None, Search::SearchFailed),
            &[],
        );
        assert!(t.contains("from (1,1) to (5,6): "), "{t}");
        assert!(!t.contains("substituted"), "{t}");
        let t = event_text(
            &search_event(CellXY::new(5, 6), Some(CellXY::new(4, 6)), Search::Found),
            &[],
        );
        assert!(
            t.contains("to (4,6) (the destination (5,6) was substituted)"),
            "{t}"
        );
    }

    // Pathing report 2: a computation that ends at the cap says so.
    #[test]
    fn a_computation_ended_by_the_cap_says_so() {
        let failed = trace::CellSearch {
            corridor: true,
            expansions: 40,
            recorded: 40,
            found: None,
            cost_bits: None,
        };
        let mut n = search_event(CellXY::new(5, 6), None, Search::SearchFailed);
        let What::Search { searches, .. } = &mut n.event.what else {
            unreachable!()
        };
        *searches = vec![failed.clone(); 5];
        let t = event_text(&n, &[]);
        assert!(t.contains("the cap of 5"), "{t}");
        let What::Search { searches, .. } = &mut n.event.what else {
            unreachable!()
        };
        searches.pop();
        let t = event_text(&n, &[]);
        assert!(!t.contains("the cap"), "{t}");
    }

    // Pathing report 2: a moving blocker with nothing listed in the cell
    // names the vehicle whose vehicle-present flag is on the cell (the A4
    // capture's seam "a refused step whose cell lists no object names no
    // blocker").
    #[test]
    fn the_holder_of_a_vehicle_present_flag_is_named() {
        use glasnost_sim::testkit;
        let size = 20;
        let mut w = testkit::game_with(&testkit::ruleset(), size, 7);
        testkit::run(&mut w, 2);
        let a = testkit::spawn(
            &mut w,
            Kind::Vehicle,
            "BUGGY",
            0,
            testkit::mid(size, 0, 0),
            0,
        );
        let b = testkit::spawn(
            &mut w,
            Kind::Vehicle,
            "BUGGY",
            0,
            testkit::mid(size, 4, 0),
            0,
        );
        let ahead = testkit::mid(size, 1, 0);
        w.obj_mut(a)
            .unwrap()
            .drive
            .as_mut()
            .expect("a driving vehicle")
            .claims
            .push(ahead);
        assert_eq!(flag_holders(&w, ahead, b), vec![a]);
        assert!(
            flag_holders(&w, ahead, a).is_empty(),
            "the mover is left out"
        );
        assert!(flag_holders(&w, testkit::mid(size, 2, 0), b).is_empty());
        let namer = Namer { names: None };
        let n = Noted {
            event: Event {
                tick: 1,
                unit: b,
                what: What::Step {
                    mover: trace::Mover::Drive,
                    from: testkit::mid(size, 2, 0),
                    to: ahead,
                    outcome: Outcome::MovingBlocker,
                    repeated: false,
                    blockers: Vec::new(),
                },
            },
            blockers: Vec::new(),
            flag: flag_holders(&w, ahead, b)
                .into_iter()
                .map(|x| placed(&w, &namer, x))
                .collect(),
        };
        let t = event_text(&n, &[]);
        assert!(
            t.contains(&format!(
                "in the cell: nothing listed; its vehicle-present flag held by #{} (vehicle",
                a.0
            )),
            "{t}"
        );
    }
}
