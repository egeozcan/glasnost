//! The margins of every duel gate over seeds and quirk switches (M8 step
//! B4): local diagnostics, run by hand in release.
//!
//! - `margins_sweep` plays each duel over a range of seeds, with the
//!   default quirk switches or with one switch flipped, and prints per
//!   counter the minimum, median and maximum and the seeds at zero.
//! - `quirk_reach` tells which quirk switches a duel reaches: a switch is
//!   reached when flipping it changes the state hash after some tick of a
//!   full run at one of the given seeds.
//! - `watch`, `hits` and `losses` follow one game: the brain's diagnosis,
//!   object counts, credits, power and lines every few ticks
//!   (`GLASNOST_WATCH`); who damages the objects of one type
//!   (`GLASNOST_HITS`); and the hits on one house by attacker and victim
//!   type (`GLASNOST_LOSSES`, with `GLASNOST_LOSSES_NONE` and
//!   `GLASNOST_LOSSES_SHOTS=<type>` for the hits with no attacker and the
//!   shots of one type; both play with the switch `GLASNOST_QUIRK` flipped
//!   when it is set).
//!
//! ```sh
//! GLASNOST_SWEEP=1-32 GLASNOST_DUELS=duel-m6c,duel-m7a GLASNOST_QUIRKS=default,table_root_distance \
//!   cargo test --release -p glasnost-sim-gate --test margins margins_sweep -- --ignored --nocapture
//! GLASNOST_REACH_SEEDS=gate GLASNOST_DUELS=all \
//!   cargo test --release -p glasnost-sim-gate --test margins quirk_reach -- --ignored --nocapture
//! ```
//!
//! Both take `GLASNOST_THREADS` (default 8) and write a CSV to
//! `GLASNOST_SWEEP_OUT` when it is set (a local-lane path; never committed).
//! Each test does nothing unless its variable (`GLASNOST_SWEEP`,
//! `GLASNOST_REACH_SEEDS`, `GLASNOST_WATCH`, `GLASNOST_HITS`,
//! `GLASNOST_LOSSES`) is set, so that the goldens' `--include-ignored` run
//! plays none of them.

#![allow(clippy::disallowed_methods, clippy::disallowed_macros)] // local diagnostics: read the host environment, print, write the lane CSV; no simulation

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use glasnost_ai::{fixture, fixture_m6c, fixture_m7a, fixture_m7b};
use glasnost_rules::quirks::Quirks;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::rules::SimRules;
use glasnost_sim_gate::plain::Plain;
use glasnost_sim_gate::{
    M6C_SEED, M6C_TICKS, M7A_SEED, M7A_TICKS, M7B_SEED, M7B_TICKS, SKIRMISH_TICKS,
};
use glasnost_sim_gate::{m6c, m7a, m7b, plain};

/// A duel gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Duel {
    Plain(PlainKey),
    M6c,
    M7a,
    M7b,
}

/// [`Plain`] with an order, for the result maps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum PlainKey {
    Straight,
    Diagonal,
    Naval,
}

impl PlainKey {
    fn plain(self) -> Plain {
        match self {
            PlainKey::Straight => Plain::Straight,
            PlainKey::Diagonal => Plain::Diagonal,
            PlainKey::Naval => Plain::Naval,
        }
    }
}

const ALL: [Duel; 6] = [
    Duel::Plain(PlainKey::Straight),
    Duel::Plain(PlainKey::Diagonal),
    Duel::Plain(PlainKey::Naval),
    Duel::M6c,
    Duel::M7a,
    Duel::M7b,
];

impl Duel {
    fn name(self) -> &'static str {
        match self {
            Duel::Plain(p) => p.plain().name(),
            Duel::M6c => "duel-m6c",
            Duel::M7a => "duel-m7a",
            Duel::M7b => "duel-m7b",
        }
    }

    fn by_name(s: &str) -> Option<Duel> {
        ALL.into_iter().find(|d| d.name() == s)
    }

    fn gate_seed(self) -> u32 {
        match self {
            Duel::Plain(p) => p.plain().seed(),
            Duel::M6c => M6C_SEED,
            Duel::M7a => M7A_SEED,
            Duel::M7b => M7B_SEED,
        }
    }

    fn ticks(self) -> i32 {
        match self {
            Duel::Plain(_) => SKIRMISH_TICKS,
            Duel::M6c => M6C_TICKS,
            Duel::M7a => M7A_TICKS,
            Duel::M7b => M7B_TICKS,
        }
    }

    /// The duel's ruleset and the map name its rules are made for.
    fn ruleset(self) -> (Ruleset, &'static str) {
        match self {
            Duel::Plain(_) => (fixture::ruleset(), "synthetic-skirmish"),
            Duel::M6c => (fixture_m6c::ruleset(), fixture_m6c::MAP_NAME),
            Duel::M7a => (fixture_m7a::ruleset(), fixture_m7a::MAP_NAME),
            Duel::M7b => (fixture_m7b::ruleset(), fixture_m7b::MAP_NAME),
        }
    }

    /// The duel's rules with `quirks` (the switches are set on the ruleset,
    /// before the rules are resolved, since some resolved values follow
    /// them).
    fn rules(self, quirks: Quirks) -> Arc<SimRules> {
        let (mut r, name) = self.ruleset();
        r.globals.quirks = quirks;
        Arc::new(SimRules::new(&r, name))
    }

    /// Plays one full run.
    fn play(self, rules: Arc<SimRules>, seed: u32) -> Run {
        let ticks = self.ticks();
        let (hashes, outcome, counters, required) = match self {
            Duel::Plain(p) => {
                let (h, g, ex) = plain::play_with(rules, p.plain(), seed, ticks);
                let req = ex.required(p.plain()).iter().map(|&(n, _)| n).collect();
                (h, g.outcome(), ex.counters().to_vec(), req)
            }
            Duel::M6c => {
                let (h, g, ex) = m6c::play_with(rules, seed, ticks);
                let c = ex.counters().to_vec();
                let req = c.iter().map(|&(n, _)| n).collect();
                (h, g.outcome(), c, req)
            }
            Duel::M7a => {
                let (h, g, ex) = m7a::play_with(rules, seed, ticks);
                let c = ex.counters().to_vec();
                let req = c.iter().map(|&(n, _)| n).collect();
                (h, g.outcome(), c, req)
            }
            Duel::M7b => {
                let (h, g, ex) = m7b::play_with(rules, seed, ticks);
                let mut c = ex.counters().to_vec();
                let req = c.iter().map(|&(n, _)| n).collect();
                // The M7a counters are reported, not asserted (duel-m7a
                // asserts them).
                c.extend(ex.m7a.counters());
                (h, g.outcome(), c, req)
            }
        };
        Run {
            ticks: hashes.len(),
            winner: outcome.map_or("none".into(), |o| format!("{:?}", o.results)),
            counters,
            required,
            hashes,
        }
    }
}

/// One full run's results.
struct Run {
    ticks: usize,
    winner: String,
    counters: Vec<(&'static str, u32)>,
    required: Vec<&'static str>,
    hashes: Vec<String>,
}

/// The default switches with `name` flipped (`default`: none flipped).
fn flipped(name: &str) -> Quirks {
    let q = Quirks::default();
    if name == "default" {
        return q;
    }
    let (_, v) = q
        .named()
        .into_iter()
        .find(|&(n, _)| n == name)
        .unwrap_or_else(|| panic!("no quirk switch {name}"));
    let src = format!("{{\"{name}\": {}}}", !v);
    let node = glasnost_rules::jsonc::parse(&src).expect("the switch parses");
    let out: Quirks = glasnost_rules::de::from_node(&node).expect("the switch deserializes");
    assert_ne!(out, q, "{name} flipped");
    out
}

/// Whether `var` is set; when it is not, says that the diagnostic is
/// skipped (so that `--include-ignored`, which regenerates the goldens,
/// plays none of them).
fn wanted(var: &str) -> bool {
    let set = std::env::var_os(var).is_some();
    if !set {
        println!("skipped: {var} is not set");
    }
    set
}

/// A type's name: a unit's or a building's.
fn type_name(
    w: &glasnost_sim::world::World,
    kind: glasnost_sim::rules::Kind,
    index: u32,
) -> String {
    if kind == glasnost_sim::rules::Kind::Building {
        if let Some(b) = usize::try_from(index)
            .ok()
            .and_then(|i| w.rules().buildings.get(i))
        {
            return b.name.clone();
        }
    }
    w.rules()
        .unit(kind, index)
        .map_or_else(|| format!("{kind:?}#{index}"), |u| u.name.clone())
}

fn env(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.into())
}

fn duels() -> Vec<Duel> {
    let v = env("GLASNOST_DUELS", "all");
    if v == "all" {
        return ALL.to_vec();
    }
    v.split(',')
        .map(|s| Duel::by_name(s.trim()).unwrap_or_else(|| panic!("no duel {s}")))
        .collect()
}

fn seeds(var: &str, default: &str) -> Vec<u32> {
    let v = env(var, default);
    let mut out = Vec::new();
    for part in v.split(',') {
        if let Some((a, b)) = part.split_once('-') {
            let (a, b): (u32, u32) = (a.parse().expect("from"), b.parse().expect("to"));
            out.extend(a..=b);
        } else if part != "gate" {
            out.push(part.parse().expect("a seed"));
        }
    }
    out
}

fn threads() -> usize {
    env("GLASNOST_THREADS", "8")
        .parse()
        .expect("GLASNOST_THREADS")
}

/// Runs `jobs` on [`threads`] workers; a job that panics gives `Err` with
/// the panic's message.
fn run_all<J, R>(
    jobs: Vec<J>,
    f: impl Fn(&J) -> R + Send + Sync + 'static,
) -> Vec<(J, Result<R, String>)>
where
    J: Clone + Send + 'static,
    R: Send + 'static,
{
    let queue = Arc::new(Mutex::new(
        jobs.into_iter().enumerate().rev().collect::<Vec<_>>(),
    ));
    let done = Arc::new(Mutex::new(Vec::new()));
    let f = Arc::new(f);
    let hs: Vec<_> = (0..threads())
        .map(|_| {
            let (queue, done, f) = (queue.clone(), done.clone(), f.clone());
            std::thread::spawn(move || {
                loop {
                    let Some((i, job)) = queue.lock().expect("queue").pop() else {
                        break;
                    };
                    let j = job.clone();
                    let f2 = f.clone();
                    let r = std::thread::Builder::new()
                        .stack_size(64 << 20)
                        .spawn(move || f2(&j))
                        .expect("spawn")
                        .join()
                        .map_err(|e| {
                            e.downcast_ref::<String>()
                                .cloned()
                                .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                                .unwrap_or_else(|| "panic".into())
                        });
                    done.lock().expect("done").push((i, job, r));
                }
            })
        })
        .collect();
    for h in hs {
        h.join().expect("a worker");
    }
    let mut v = std::mem::take(&mut *done.lock().expect("done"));
    v.sort_by_key(|(i, _, _)| *i);
    v.into_iter().map(|(_, j, r)| (j, r)).collect()
}

fn csv_out(lines: &[String]) {
    if let Ok(p) = std::env::var("GLASNOST_SWEEP_OUT") {
        std::fs::write(&p, lines.join("\n") + "\n").expect("the CSV is written");
        println!("wrote {p}");
    }
}

/// The margin sweep: per duel and quirk setting, each counter's minimum,
/// median and maximum over the seeds, and the seeds where a required
/// counter is zero.
#[test]
#[ignore = "a sweep of whole games; set GLASNOST_SWEEP, GLASNOST_DUELS, GLASNOST_QUIRKS"]
fn margins_sweep() {
    if !wanted("GLASNOST_SWEEP") {
        return;
    }
    let seeds = seeds("GLASNOST_SWEEP", "1-32");
    let settings: Vec<String> = env("GLASNOST_QUIRKS", "default")
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    let mut jobs = Vec::new();
    for d in duels() {
        for s in &settings {
            for &seed in &seeds {
                jobs.push((d, s.clone(), seed));
            }
        }
    }
    let results = run_all(jobs, |(d, s, seed)| {
        let r = d.play(d.rules(flipped(s)), *seed);
        (r.ticks, r.winner, r.counters, r.required)
    });
    let mut csv = vec!["duel,setting,seed,ticks,winner,counter,required,value".to_string()];
    type Key = (Duel, String);
    let mut by: BTreeMap<Key, Vec<(u32, Result<_, String>)>> = BTreeMap::new();
    for ((d, s, seed), r) in results {
        by.entry((d, s)).or_default().push((seed, r));
    }
    let mut summary = Vec::new();
    for ((d, s), runs) in &by {
        println!("== {} / {s} ==", d.name());
        let ok: Vec<_> = runs
            .iter()
            .filter_map(|(seed, r)| match r {
                Ok(v) => Some((*seed, v)),
                Err(e) => {
                    println!("seed {seed}: PANICKED: {e}");
                    summary.push(format!("{} / {s}: seed {seed} panicked: {e}", d.name()));
                    None
                }
            })
            .collect();
        for (seed, (ticks, winner, counters, required)) in &ok {
            let zeros: Vec<&str> = counters
                .iter()
                .filter(|&&(n, v)| v == 0 && required.contains(&n))
                .map(|&(n, _)| n)
                .collect();
            println!("seed {seed}: {ticks} ticks, results {winner}, required at zero {zeros:?}");
            for &(n, v) in counters {
                csv.push(format!(
                    "{},{s},{seed},{ticks},\"{winner}\",\"{n}\",{},{v}",
                    d.name(),
                    required.contains(&n)
                ));
            }
        }
        let Some((_, (_, _, first, required))) = ok.first() else {
            continue;
        };
        let mut least: Option<(u32, &str)> = None;
        for (i, &(name, _)) in first.iter().enumerate() {
            let mut v: Vec<(u32, u32)> = ok.iter().map(|(seed, r)| (r.2[i].1, *seed)).collect();
            v.sort_unstable();
            let zero: Vec<u32> = v.iter().filter(|p| p.0 == 0).map(|p| p.1).collect();
            let req = required.contains(&name);
            if req && least.is_none_or(|(m, _)| v[0].0 < m) {
                least = Some((v[0].0, name));
            }
            println!(
                "{}{name}: min {} median {} max {}{}",
                if req { "" } else { "(reported) " },
                v[0].0,
                v[v.len() / 2].0,
                v[v.len() - 1].0,
                if zero.is_empty() {
                    String::new()
                } else {
                    format!(", zero at seeds {zero:?}")
                }
            );
        }
        let early = ok
            .iter()
            .filter(|(_, r)| r.0 < usize::try_from(d.ticks()).unwrap_or(usize::MAX))
            .count();
        if let Some((m, n)) = least {
            summary.push(format!(
                "{} / {s}: {} seeds, smallest required minimum {m} ({n}), ended early {early}",
                d.name(),
                ok.len()
            ));
        }
    }
    println!("== summary ==");
    for l in &summary {
        println!("{l}");
    }
    csv_out(&csv);
}

/// Which quirk switches each duel reaches: per duel and candidate switch
/// (`GLASNOST_QUIRKS`, default every switch of `Quirks::named`), whether
/// flipping it changes the state hash after some tick of a full run at a
/// seed of `GLASNOST_REACH_SEEDS` (`gate` is the duel's gate seed; more
/// seeds may be listed), and the first such tick.
#[test]
#[ignore = "plays every candidate switch flipped; set GLASNOST_DUELS, GLASNOST_QUIRKS, GLASNOST_REACH_SEEDS"]
fn quirk_reach() {
    if !wanted("GLASNOST_REACH_SEEDS") {
        return;
    }
    let candidates: Vec<String> = match std::env::var("GLASNOST_QUIRKS") {
        Ok(v) => v.split(',').map(|s| s.trim().to_string()).collect(),
        Err(_) => Quirks::default()
            .named()
            .into_iter()
            .map(|(n, _)| n.to_string())
            .collect(),
    };
    let spec = env("GLASNOST_REACH_SEEDS", "gate");
    let with_gate = spec.split(',').any(|p| p == "gate");
    let extra = seeds("GLASNOST_REACH_SEEDS", "gate");
    let mut runs = Vec::new();
    for d in duels() {
        let mut ss: Vec<u32> = extra.clone();
        if with_gate {
            ss.insert(0, d.gate_seed());
        }
        ss.dedup();
        for seed in ss {
            runs.push((d, seed));
        }
    }
    // The default runs first; each flipped run is compared with its default
    // run in its worker, so only the first differing tick is kept.
    let base: BTreeMap<(Duel, u32), Arc<Vec<String>>> = run_all(runs.clone(), |(d, seed)| {
        d.play(d.rules(Quirks::default()), *seed).hashes
    })
    .into_iter()
    .map(|(k, r)| (k, Arc::new(r.expect("the default run finishes"))))
    .collect();
    let base = Arc::new(base);
    let mut jobs = Vec::new();
    for &(d, seed) in &runs {
        for c in &candidates {
            jobs.push((d, c.clone(), seed));
        }
    }
    let b2 = base.clone();
    let results = run_all(jobs, move |(d, s, seed)| {
        let r = d.play(d.rules(flipped(s)), *seed);
        let b = &b2[&(*d, *seed)];
        let first = r.hashes.iter().zip(b.iter()).position(|(x, y)| x != y);
        match first {
            Some(i) => Some((i, r.ticks)),
            None if r.hashes.len() != b.len() => Some((r.hashes.len().min(b.len()), r.ticks)),
            None => None,
        }
    });
    let mut csv = vec!["duel,switch,seed,reached,first_tick,note".to_string()];
    let mut reached: BTreeMap<Duel, Vec<String>> = BTreeMap::new();
    for ((d, s, seed), r) in &results {
        let (hit, tick, note) = match r {
            Err(e) => (true, String::new(), format!("panicked: {e}")),
            Ok(Some((i, n))) => (true, i.to_string(), format!("{n} ticks played")),
            Ok(None) => (false, String::new(), String::new()),
        };
        csv.push(format!("{},{s},{seed},{hit},{tick},\"{note}\"", d.name()));
        if hit {
            println!(
                "{} {s} seed {seed}: reached at tick {tick} {note}",
                d.name()
            );
            let v = reached.entry(*d).or_default();
            if !v.contains(s) {
                v.push(s.clone());
            }
        }
    }
    println!("== reached ==");
    for (d, v) in &reached {
        println!("{} ({}): {}", d.name(), v.len(), v.join(","));
    }
    csv_out(&csv);
}

/// Watches one game: `GLASNOST_WATCH=<duel>,<seed>,<every>[,<quirk>]` prints,
/// every `every` ticks, each AI house's diagnostic line and the count of
/// its objects on the map by type, and at the end the special-ability
/// orders.
#[test]
#[ignore = "a diagnostic of one game; set GLASNOST_WATCH=<duel>,<seed>,<every>[,<quirk>]"]
fn watch() {
    if !wanted("GLASNOST_WATCH") {
        return;
    }
    use glasnost_ai::game::Game;
    let v = env("GLASNOST_WATCH", "");
    let parts: Vec<&str> = v.split(',').collect();
    let d = Duel::by_name(parts[0]).expect("a duel");
    let seed: u32 = parts[1].parse().expect("a seed");
    let every: i32 = parts[2].parse().expect("a period");
    let rules = d.rules(flipped(parts.get(3).copied().unwrap_or("default")));
    let (map, name, setup) = match d {
        Duel::Plain(p) => (p.plain().map(), p.plain().map_name(), plain::setup(seed)),
        Duel::M6c => (
            fixture_m6c::map(),
            fixture_m6c::MAP_NAME,
            fixture_m6c::setup(seed),
        ),
        Duel::M7a => (
            fixture_m7a::map(),
            fixture_m7a::MAP_NAME,
            fixture_m7a::setup(seed),
        ),
        Duel::M7b => (
            fixture_m7b::map(),
            fixture_m7b::MAP_NAME,
            fixture_m7b::setup(seed),
        ),
    };
    let mut g = Game::new(rules, &map, name, fixture::catalogue(), setup).expect("loads");
    while g.world.tick < d.ticks() {
        g.step();
        g.world.effects.clear();
        if g.world.tick % every == 0 || g.outcome().is_some() {
            println!("-- tick {}", g.world.tick);
            for (h, line) in g.diagnose() {
                let mut by: BTreeMap<String, u32> = BTreeMap::new();
                for o in g.world.objects.values() {
                    if o.on_map && o.owner == Some(h) {
                        let n = type_name(&g.world, o.kind, o.type_index);
                        *by.entry(n).or_default() += 1;
                    }
                }
                println!("house {}: {line}\n  objects {by:?}", h.0);
                // `GLASNOST_WATCH_TYPE=<type>[,<type>...]`: each object of
                // those types, with its cell, health, behaviours, target and
                // destination.
                let traced = env("GLASNOST_WATCH_TYPE", "");
                let traced: Vec<&str> = traced.split(',').filter(|s| !s.is_empty()).collect();
                for o in g.world.objects.values() {
                    let t = type_name(&g.world, o.kind, o.type_index);
                    if o.owner == Some(h) && traced.contains(&t.as_str()) {
                        println!(
                            "  {t}#{} at {:?} on map {} health {} beh {:?} target {:?} dest {:?} contact {:?}",
                            o.id.0,
                            o.pos.cell(),
                            o.on_map,
                            o.health,
                            o.beh,
                            o.target,
                            o.dest,
                            o.contact
                        );
                    }
                }
                println!(
                    "  credits {} power {:?}",
                    g.world.houses[usize::from(h.0)].credits,
                    glasnost_sim::econ::power::totals(&g.world, h)
                );
                let name = |t: glasnost_sim::econ::TypeRef| type_name(&g.world, t.kind, t.index);
                for l in g.world.econ.lines.lines.iter().filter(|l| l.house == h) {
                    println!(
                        "  line {:?}: item {:?} queue {:?}",
                        l.kind,
                        l.item
                            .as_ref()
                            .map(|i| (name(i.t), i.progress, i.waiting, i.done())),
                        l.queue.iter().map(|&t| name(t)).collect::<Vec<_>>()
                    );
                }
            }
            if g.outcome().is_some() {
                break;
            }
        }
    }
    println!("{:?}", g.special_uses());
}

/// Who hits a type: `GLASNOST_HITS=<duel>,<seed>,<type>[,<until tick>]`
/// prints, at each tick an object of the type loses health, the objects
/// whose target it is (their type, owner and squared distance in cells).
#[test]
#[ignore = "a diagnostic of one game; set GLASNOST_HITS=<duel>,<seed>,<type>[,<until>]"]
fn hits() {
    if !wanted("GLASNOST_HITS") {
        return;
    }
    use glasnost_ai::game::Game;
    use glasnost_sim::world::Target;
    let v = env("GLASNOST_HITS", "");
    let parts: Vec<&str> = v.split(',').collect();
    let d = Duel::by_name(parts[0]).expect("a duel");
    let seed: u32 = parts[1].parse().expect("a seed");
    let ty = parts[2];
    let until: i32 = parts
        .get(3)
        .map_or(d.ticks(), |s| s.parse().expect("a tick"));
    let rules = d.rules(flipped(&env("GLASNOST_QUIRK", "default")));
    let (map, name, setup) = match d {
        Duel::Plain(p) => (p.plain().map(), p.plain().map_name(), plain::setup(seed)),
        Duel::M6c => (
            fixture_m6c::map(),
            fixture_m6c::MAP_NAME,
            fixture_m6c::setup(seed),
        ),
        Duel::M7a => (
            fixture_m7a::map(),
            fixture_m7a::MAP_NAME,
            fixture_m7a::setup(seed),
        ),
        Duel::M7b => (
            fixture_m7b::map(),
            fixture_m7b::MAP_NAME,
            fixture_m7b::setup(seed),
        ),
    };
    let mut g = Game::new(rules, &map, name, fixture::catalogue(), setup).expect("loads");
    let tname = |g: &Game, id| {
        g.world
            .obj(id)
            .and_then(|o| {
                g.world
                    .rules()
                    .unit(o.kind, o.type_index)
                    .map(|u| u.name.clone())
            })
            .unwrap_or_else(|| "?".into())
    };
    let mut health: BTreeMap<glasnost_sim::world::ObjectId, i32> = BTreeMap::new();
    while g.world.tick < until {
        g.step();
        g.world.effects.clear();
        let watched: Vec<_> = g
            .world
            .objects
            .values()
            .filter(|o| o.on_map && tname(&g, o.id) == ty)
            .map(|o| (o.id, o.health, o.pos.cell(), o.owner))
            .collect();
        for &(id, h, at, owner) in &watched {
            let before = health.insert(id, h).unwrap_or(h);
            if h < before {
                let by: Vec<String> = g
                    .world
                    .objects
                    .values()
                    .filter(|o| o.target == Some(Target::Object(id)))
                    .map(|o| {
                        let c = o.pos.cell();
                        let dd = (c.x - at.x).pow(2) + (c.y - at.y).pow(2);
                        let mood = if o.combat.berserk.is_some() {
                            " berserk"
                        } else {
                            ""
                        };
                        format!(
                            "{}#{} of {:?} d2={dd}{mood}",
                            tname(&g, o.id),
                            o.id.0,
                            o.owner
                        )
                    })
                    .collect();
                println!(
                    "tick {}: {ty}#{} of {owner:?} at {at:?} health {before} -> {h}, targeted by {by:?}",
                    g.world.tick, id.0
                );
            }
        }
        for (id, _) in health.clone() {
            if !watched.iter().any(|w| w.0 == id) {
                println!("tick {}: {ty}#{} gone", g.world.tick, id.0);
                health.remove(&id);
            }
        }
        if g.outcome().is_some() {
            break;
        }
    }
}

/// What hits a house: `GLASNOST_LOSSES=<duel>,<seed>,<house>[,<until tick>]`
/// tallies the hits on the house's objects by attacker type and victim
/// type, every 500 ticks.
#[test]
#[ignore = "a diagnostic of one game; set GLASNOST_LOSSES=<duel>,<seed>,<house>[,<until>]"]
fn losses() {
    if !wanted("GLASNOST_LOSSES") {
        return;
    }
    use glasnost_ai::game::Game;
    use glasnost_sim::combat::effects::Effect;
    let v = env("GLASNOST_LOSSES", "");
    let parts: Vec<&str> = v.split(',').collect();
    let d = Duel::by_name(parts[0]).expect("a duel");
    let seed: u32 = parts[1].parse().expect("a seed");
    let house: u8 = parts[2].parse().expect("a house");
    let until: i32 = parts
        .get(3)
        .map_or(d.ticks(), |s| s.parse().expect("a tick"));
    let rules = d.rules(flipped(&env("GLASNOST_QUIRK", "default")));
    let (map, name, setup) = match d {
        Duel::Plain(p) => (p.plain().map(), p.plain().map_name(), plain::setup(seed)),
        Duel::M6c => (
            fixture_m6c::map(),
            fixture_m6c::MAP_NAME,
            fixture_m6c::setup(seed),
        ),
        Duel::M7a => (
            fixture_m7a::map(),
            fixture_m7a::MAP_NAME,
            fixture_m7a::setup(seed),
        ),
        Duel::M7b => (
            fixture_m7b::map(),
            fixture_m7b::MAP_NAME,
            fixture_m7b::setup(seed),
        ),
    };
    let mut g = Game::new(rules, &map, name, fixture::catalogue(), setup).expect("loads");
    let mut types: BTreeMap<glasnost_sim::world::ObjectId, String> = BTreeMap::new();
    let mut tally: BTreeMap<(String, String), u32> = BTreeMap::new();
    while g.world.tick < until {
        for o in g.world.objects.values() {
            if let Some(u) = g.world.rules().unit(o.kind, o.type_index) {
                types.entry(o.id).or_insert_with(|| u.name.clone());
            }
        }
        g.step();
        for e in &g.world.effects {
            if let Effect::Shot { firer, .. } = e
                && std::env::var("GLASNOST_LOSSES_SHOTS")
                    .is_ok_and(|t| types.get(firer).is_some_and(|n| *n == t))
            {
                let o = g.world.obj(*firer);
                println!(
                    "tick {}: shot by {:?} of {:?} at {:?} target {:?}",
                    g.world.tick,
                    types.get(firer),
                    o.and_then(|o| o.owner),
                    o.map(|o| o.pos.cell()),
                    o.and_then(|o| o.target).map(|t| (
                        t,
                        match t {
                            glasnost_sim::world::Target::Object(i) => types.get(&i).cloned(),
                            _ => None,
                        }
                    ))
                );
            }
            if let Effect::Hit {
                id,
                owner,
                attacker,
                ..
            } = e
                && *owner == Some(glasnost_sim::world::HouseId(house))
            {
                let a = attacker
                    .and_then(|a| types.get(&a).cloned())
                    .unwrap_or_else(|| "none".into());
                let vt = types.get(id).cloned().unwrap_or_else(|| "?".into());
                if attacker.is_none() && std::env::var("GLASNOST_LOSSES_NONE").is_ok() {
                    println!("tick {}: {vt} hit by none: {e:?}", g.world.tick);
                }
                *tally.entry((a, vt)).or_default() += 1;
            }
        }
        g.world.effects.clear();
        if g.world.tick % 500 == 0 || g.outcome().is_some() {
            println!("-- tick {}: {tally:?}", g.world.tick);
            tally.clear();
        }
        if g.outcome().is_some() {
            break;
        }
    }
}
