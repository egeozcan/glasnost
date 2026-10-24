//! The tick benchmark (M8 step C4 part 2): our own tooling, no game
//! behaviour, so no spec applies and the numbers are ours.
//!
//! - **Local lane** ([`run`]): the frame benchmark's game
//!   (`glasnost_app::game::bench`: an AI skirmish on the install's `map`-th
//!   offered map) set up the same way, then `ticks` ticks of
//!   `glasnost_ai::game::Game::step` with nothing drawn, each timed on the
//!   host clock. With `probe=on` the tick also tells a [`Clock`] where it
//!   is (`glasnost_sim::probe`), which splits each tick's time by the
//!   pipeline's parts (S-0203's rows), the AI's houses and the kinds of
//!   turn in the update order. The probe reads nothing back into the game,
//!   so the state hashes are the same with it on or off. Writes a summary
//!   (mean, median, p95, p99, worst, the worst ticks and what ran in them)
//!   and a per-tick CSV, plus the state hash every `hash` ticks (a file to
//!   compare between builds).
//! - **Synthetic** ([`run_synthetic`]): the testkit's scripted replay
//!   timed the same way; usable in CI, with no timing asserted there.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::time::Instant;

use glasnost_ai::game::Game;
use glasnost_sim::probe::{Phase, Probe, TurnClass};
use glasnost_sim::world::World;
use glasnost_sim::{load, save, testkit, tick};

/// The usage line.
pub const USAGE: &str = "tick-bench [map=9] [seed=7] [players=8] [ticks=9000] [probe=off|on] [hash=500] [save=TICK] [worst=20] [out=DIR] [label=NAME]";

/// The run's options (`key=value` words).
#[derive(Clone, Debug)]
pub struct Options {
    /// Offered-map index.
    pub map: usize,
    /// The game's seed.
    pub seed: u32,
    /// AI houses.
    pub players: usize,
    /// Ticks to run.
    pub ticks: u32,
    /// Whether the phase probe is on.
    pub probe: bool,
    /// The state hash every this many ticks (0: none).
    pub hash: u32,
    /// Save the game after this tick, load it and play on with the loaded
    /// game (the derived-data rebuild check).
    pub save: Option<u32>,
    /// How many worst ticks the summary lists.
    pub worst: usize,
    /// Output folder.
    pub out: String,
    /// Output base name.
    pub label: String,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            // nocite: our own tooling (the frame benchmark's game: map index 9, seed 7, 8 houses, 9000 ticks)
            map: 9,
            seed: 7,
            players: 8,
            ticks: 9000,
            probe: false,
            hash: 500,
            save: None,
            worst: 20,
            out: "local-lane/m8/c4-2".into(),
            label: "ticks".into(),
        }
    }
}

impl Options {
    /// Parses `key=value` words.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let mut o = Self::default();
        for a in args {
            let Some((k, v)) = a.split_once('=') else {
                return Err(format!("not key=value: {a}"));
            };
            let bad = |_| format!("bad value for {k}: {v}");
            match k {
                "map" => o.map = v.parse().map_err(bad)?,
                "seed" => o.seed = v.parse().map_err(bad)?,
                "players" => o.players = v.parse().map_err(bad)?,
                "ticks" => o.ticks = v.parse().map_err(bad)?,
                "probe" => o.probe = v == "on",
                "hash" => o.hash = v.parse().map_err(bad)?,
                "save" => o.save = Some(v.parse().map_err(bad)?),
                "worst" => o.worst = v.parse().map_err(bad)?,
                "out" => o.out = v.into(),
                "label" => o.label = v.into(),
                _ => return Err(format!("unknown option {k}")),
            }
        }
        Ok(o)
    }
}

/// The probe of the benchmark: the host clock between marks, summed per
/// part for the current tick.
#[derive(Default)]
pub struct Clock {
    last: Option<Instant>,
    cur: Option<Phase>,
    /// This tick's microseconds per part.
    pub tick: BTreeMap<Phase, f64>,
    /// This tick's turns per kind.
    pub turns: BTreeMap<TurnClass, u32>,
}

impl Clock {
    /// Closes the open part (call after the step).
    fn close(&mut self) {
        self.mark(Phase::Done);
        self.last = None;
        self.cur = None;
    }
}

impl Probe for Clock {
    fn mark(&mut self, phase: Phase) {
        let now = Instant::now();
        if let (Some(last), Some(cur)) = (self.last, self.cur) {
            *self.tick.entry(cur).or_default() += now.duration_since(last).as_secs_f64() * 1e6;
        }
        if let Phase::Turn(c) = phase {
            *self.turns.entry(c).or_default() += 1;
        }
        self.last = Some(now);
        self.cur = Some(phase);
    }
}

/// The name of a part in the outputs. The AI's houses fold into one
/// `think` column; the per-house split is in the summary.
fn name(p: Phase) -> String {
    match p {
        Phase::Turn(c) => format!("turn:{c:?}"),
        Phase::Think(h) => format!("think:{h}"),
        Phase::Part(n) => n.to_owned(),
        _ => format!("{p:?}"),
    }
}

/// One tick's numbers.
struct Row {
    tick: i32,
    us: f64,
    objects: usize,
    order: usize,
    parts: BTreeMap<Phase, f64>,
    turns: BTreeMap<TurnClass, u32>,
}

/// Nearest-rank statistics of a sample.
fn stats(v: &[f64]) -> (f64, f64, f64, f64, f64) {
    if v.is_empty() {
        return (0.0, 0.0, 0.0, 0.0, 0.0);
    }
    let mut s = v.to_vec();
    s.sort_by(f64::total_cmp);
    let rank = |q: f64| s[((q * s.len() as f64).ceil() as usize).clamp(1, s.len()) - 1];
    let mean = s.iter().sum::<f64>() / s.len() as f64;
    (mean, rank(0.5), rank(0.95), rank(0.99), s[s.len() - 1])
}

/// Runs the ticks on `game`, returning the rows and the hash lines.
fn play(
    game: &mut Game,
    o: &Options,
    mut each: impl FnMut(&mut Game) -> Result<(), String>,
) -> Result<(Vec<Row>, Vec<String>), String> {
    let mut rows = Vec::with_capacity(o.ticks as usize);
    let mut hashes = Vec::new();
    let mut clock = Clock::default();
    for i in 0..o.ticks {
        let tick = game.world.tick;
        let t0 = Instant::now();
        if o.probe {
            clock.tick.clear();
            clock.turns.clear();
            game.step_with(&mut clock);
            clock.close();
        } else {
            game.step();
        }
        let us = t0.elapsed().as_secs_f64() * 1e6;
        // Nothing draws these ticks: their effects go, as in the frame benchmark.
        game.world.effects.clear();
        rows.push(Row {
            tick,
            us,
            objects: game.world.objects.len(),
            order: game.world.update.order.len(),
            parts: std::mem::take(&mut clock.tick),
            turns: std::mem::take(&mut clock.turns),
        });
        let n = i + 1;
        if o.hash > 0 && n.is_multiple_of(o.hash) {
            hashes.push(format!("{} {}", game.world.tick, game.state_hash()));
        }
        each(game)?;
        if game.outcome().is_some() {
            break;
        }
    }
    let end = format!("{} {}", game.world.tick, game.state_hash());
    if hashes.last() != Some(&end) {
        hashes.push(end);
    }
    Ok((rows, hashes))
}

/// The summary and CSV of a run.
fn report(o: &Options, head: &str, rows: &[Row], hashes: &[String]) -> (String, String) {
    let mut s = String::new();
    let _ = writeln!(s, "Glasnost tick benchmark (M8 C4 part 2): {}", o.label);
    s.push_str(head);
    let totals: Vec<f64> = rows.iter().map(|r| r.us).collect();
    let (mean, median, p95, p99, worst) = stats(&totals);
    let _ = writeln!(
        s,
        "ticks: {} (from tick {} to {}); probe {}",
        rows.len(),
        rows.first().map_or(0, |r| r.tick),
        rows.last().map_or(0, |r| r.tick),
        if o.probe { "on" } else { "off" }
    );
    let _ = writeln!(
        s,
        "tick time (µs): mean {mean:.0}, median {median:.0}, p95 {p95:.0}, p99 {p99:.0}, worst {worst:.0}; over 16.7 ms: {}",
        totals.iter().filter(|&&t| t > 16_667.0).count()
    );
    // By thousand ticks: the game's course.
    let _ = writeln!(
        s,
        "\nby 1000 ticks (µs): from  mean  median  p95  worst  objects  order"
    );
    for chunk in rows.chunks(1000) {
        let v: Vec<f64> = chunk.iter().map(|r| r.us).collect();
        let (m, md, p, _, w) = stats(&v);
        let last = chunk.last().map_or((0, 0), |r| (r.objects, r.order));
        let _ = writeln!(
            s,
            "  {:>6}  {m:>7.0} {md:>7.0} {p:>7.0} {w:>8.0}  {:>6}  {:>6}",
            chunk[0].tick, last.0, last.1
        );
    }
    let mut csv = String::from("tick,us,objects,order");
    let mut names: Vec<Phase> = Vec::new();
    let mut classes: Vec<TurnClass> = Vec::new();
    if o.probe {
        let mut all: BTreeMap<Phase, f64> = BTreeMap::new();
        let mut turns: BTreeMap<TurnClass, u64> = BTreeMap::new();
        for r in rows {
            for (p, v) in &r.parts {
                *all.entry(*p).or_default() += v;
            }
            for (c, n) in &r.turns {
                *turns.entry(*c).or_default() += u64::from(*n);
            }
        }
        names = all.keys().copied().collect();
        classes = turns.keys().copied().collect();
        let sum: f64 = all.values().sum();
        let ticks = rows.len().max(1) as f64;
        let _ = writeln!(
            s,
            "\nattribution (probe on; share of the probed time, mean µs a tick, worst tick's µs):"
        );
        let mut by: Vec<(Phase, f64)> = all.iter().map(|(p, v)| (*p, *v)).collect();
        by.sort_by(|a, b| b.1.total_cmp(&a.1));
        for (p, v) in &by {
            let w = rows
                .iter()
                .map(|r| r.parts.get(p).copied().unwrap_or(0.0))
                .fold(0.0, f64::max);
            let _ = writeln!(
                s,
                "  {:<24} {:>6.2} %  {:>9.1}  {:>9.0}",
                name(*p),
                100.0 * v / sum.max(1e-9),
                v / ticks,
                w
            );
        }
        let _ = writeln!(s, "\nturns a tick by kind (mean):");
        for (c, n) in &turns {
            let _ = writeln!(s, "  {:<16} {:>9.1}", format!("{c:?}"), *n as f64 / ticks);
        }
        for p in &names {
            let _ = write!(csv, ",{}", name(*p));
        }
        for c in &classes {
            let _ = write!(csv, ",n:{c:?}");
        }
    }
    csv.push('\n');
    for r in rows {
        let _ = write!(csv, "{},{:.1},{},{}", r.tick, r.us, r.objects, r.order);
        for p in &names {
            let _ = write!(csv, ",{:.1}", r.parts.get(p).copied().unwrap_or(0.0));
        }
        for c in &classes {
            let _ = write!(csv, ",{}", r.turns.get(c).copied().unwrap_or(0));
        }
        csv.push('\n');
    }
    let mut idx: Vec<usize> = (0..rows.len()).collect();
    idx.sort_by(|&a, &b| rows[b].us.total_cmp(&rows[a].us));
    let _ = writeln!(s, "\nworst ticks (µs; with the probe, the largest parts):");
    for &i in idx.iter().take(o.worst) {
        let r = &rows[i];
        let mut parts: Vec<(Phase, f64)> = r.parts.iter().map(|(p, v)| (*p, *v)).collect();
        parts.sort_by(|a, b| b.1.total_cmp(&a.1));
        let top: Vec<String> = parts
            .iter()
            .take(5)
            .map(|(p, v)| format!("{} {v:.0}", name(*p)))
            .collect();
        let _ = writeln!(
            s,
            "  tick {:>6}: {:>8.0}  objects {} order {}  {}",
            r.tick,
            r.us,
            r.objects,
            r.order,
            top.join(", ")
        );
    }
    if let Some(last) = hashes.last() {
        let _ = writeln!(s, "\nstate hash at the end: {last}");
    }
    (s, csv)
}

fn load_average() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|l| {
            let f: Vec<&str> = l.split_whitespace().take(3).collect();
            (f.len() == 3).then(|| f.join(" "))
        })
        .unwrap_or_else(|| "unknown".into())
}

/// Runs the local lane's tick benchmark on the install in `dir`; writes
/// `<label>.txt`, `<label>.csv` and `<label>-hashes.txt` to `out` and
/// returns the summary.
pub fn run(o: &Options, dir: &str) -> Result<String, String> {
    let spec = format!("map={} seed={} players={} late=0", o.map, o.seed, o.players);
    let bench = glasnost_app::game::bench::BenchOptions::parse(&spec)?;
    let load_before = load_average();
    let (mut client, map_hash) = glasnost_app::game::bench::start_headless(bench, dir)?;
    let Some(session) = client.session.as_mut() else {
        return Err("the game did not start".into());
    };
    let game = &mut session.play.game;
    let save_at = o.save;
    let mut saved = String::new();
    let (rows, hashes) = play(game, o, |g| {
        if save_at == Some(g.world.tick as u32) {
            let bytes = g.save();
            let rules = g.world.rules_arc();
            let cat = g
                .world
                .catalogue
                .clone()
                .ok_or_else(|| "no tile catalogue".to_owned())?;
            let before = g.state_hash();
            *g = Game::load(&bytes, rules, cat).map_err(|e| e.to_string())?;
            let after = g.state_hash();
            saved = format!(
                "saved and loaded after tick {} ({} bytes); hash {} before, {} after\n",
                g.world.tick,
                bytes.len(),
                &before[..16],
                &after[..16]
            );
        }
        Ok(())
    })?;
    let mut head = String::new();
    let _ = writeln!(head, "build: {}", glasnost_app::game::client::BUILD);
    let _ = writeln!(
        head,
        "map: index {} (content hash {map_hash}); seed {}; {} AI houses asked",
        o.map, o.seed, o.players
    );
    let _ = writeln!(
        head,
        "load average: before {load_before}; after {}",
        load_average()
    );
    head.push_str(&saved);
    let (summary, csv) = report(o, &head, &rows, &hashes);
    let out = std::path::Path::new(&o.out);
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let write =
        |name: String, text: &str| std::fs::write(out.join(name), text).map_err(|e| e.to_string());
    write(format!("{}.txt", o.label), &summary)?;
    write(format!("{}.csv", o.label), &csv)?;
    write(
        format!("{}-hashes.txt", o.label),
        &(hashes.join("\n") + "\n"),
    )?;
    Ok(summary)
}

/// The synthetic tick benchmark: the testkit's scripted replay of `ticks`
/// ticks, timed per tick (with the probe when asked). Returns the summary
/// and the world's state hash at the end. No timing is asserted anywhere.
pub fn run_synthetic(seed: u32, ticks: i32, probe: bool) -> (String, String) {
    let rep = testkit::scripted_replay(seed, ticks);
    let mut w: World = load::new_game(
        testkit::rules(),
        &testkit::map(),
        testkit::catalogue(),
        &rep.setup,
    )
    .expect("the synthetic scenario loads");
    let mut rows = Vec::new();
    let mut clock = Clock::default();
    while tick::ticks_run(&w) < rep.ticks && !w.econ.end.finished {
        rep.issue_due(&mut w);
        let t = w.tick;
        let t0 = Instant::now();
        if probe {
            clock.tick.clear();
            clock.turns.clear();
            tick::step_with(&mut w, &mut clock);
            clock.close();
        } else {
            tick::step(&mut w);
        }
        let us = t0.elapsed().as_secs_f64() * 1e6;
        w.effects.clear();
        rows.push(Row {
            tick: t,
            us,
            objects: w.objects.len(),
            order: w.update.order.len(),
            parts: std::mem::take(&mut clock.tick),
            turns: std::mem::take(&mut clock.turns),
        });
    }
    let o = Options {
        ticks: ticks.max(0) as u32,
        probe,
        label: "synthetic".into(),
        ..Options::default()
    };
    let hash = save::state_hash(&w);
    let (summary, _) = report(&o, "", &rows, std::slice::from_ref(&hash));
    (summary, hash)
}
