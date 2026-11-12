//! The AI think benchmark (own AI issues, batch 2: #52, #53, #54, #57 to
//! #60): our own tooling, no game behaviour, so no spec applies and the
//! numbers are ours.
//!
//! It plays one AI game with every tick timed on the host clock through the
//! tick's probe (`glasnost_sim::probe`): each AI house's part of the tick,
//! from its `Think` mark to the `Issue` mark that follows (the brain's
//! `ai:…` parts on the way), and the rest of the tick (`tick::step`). The
//! probe reads nothing back into the game, so the state hashes are the same
//! as without it; they are written every `hash` ticks, a file to compare
//! between builds.
//!
//! The games: the synthetic duels of `glasnost_ai`'s fixtures (`fixture`,
//! its `naval`, `cramped`, `strait` and `islands` maps, `m6c`, `m7a`,
//! `m7b`), or the frame benchmark's game on an install (`install`, with
//! `map=` and `players=`, as `tick-bench` plays it).
//!
//! The summary gives, per house and for all houses, the AI time on the
//! house's think ticks (`scripted::thinks_at`) and between them, the
//! `tick::step` time, their ratio and the AI's share of the step, the
//! `ai:…` parts' shares, and what the think stagger (`THINK_PHASES`) is
//! worth: the largest single tick's AI time against the largest sum of
//! every house's think in one think period (what one tick would cost if
//! every house thought in it).

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::time::Instant;

use glasnost_ai::game::Game;
use glasnost_ai::scripted::{THINK_PERIOD, thinks_at};
use glasnost_ai::{fixture, fixture_m6c, fixture_m7a, fixture_m7b};
use glasnost_sim::probe::{Phase, Probe};
use glasnost_sim::world::HouseId;

/// The usage line.
pub const USAGE: &str = "ai-bench [game=fixture|naval|cramped|strait|islands|m6c|m7a|m7b|install] [seed=N] [ticks=12000] [hash=500] [map=9] [players=8] [out=DIR] [label=NAME]";

/// The run's options (`key=value` words).
#[derive(Clone, Debug)]
pub struct Options {
    /// The game.
    pub game: String,
    /// The game's seed (none: the gate's seed of that duel).
    pub seed: Option<u32>,
    /// Ticks to run at most (the game may end sooner).
    pub ticks: u32,
    /// The state hash every this many ticks (0: only at the end).
    pub hash: u32,
    /// The install game's offered map.
    pub map: usize,
    /// The install game's AI houses.
    pub players: usize,
    /// Output folder.
    pub out: String,
    /// Output base name (none: the game's name).
    pub label: Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            game: "fixture".into(),
            seed: None,
            // nocite: our own tooling (a game's first 12000 ticks; the frame benchmark's map 9 and 8 houses)
            ticks: 12_000,
            hash: 500,
            map: 9,
            players: 8,
            out: "local-lane/ai-issues/b2/bench".into(),
            label: None,
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
                "game" => o.game = v.into(),
                "seed" => o.seed = Some(v.parse().map_err(bad)?),
                "ticks" => o.ticks = v.parse().map_err(bad)?,
                "hash" => o.hash = v.parse().map_err(bad)?,
                "map" => o.map = v.parse().map_err(bad)?,
                "players" => o.players = v.parse().map_err(bad)?,
                "out" => o.out = v.into(),
                "label" => o.label = Some(v.into()),
                _ => return Err(format!("unknown option {k}")),
            }
        }
        Ok(o)
    }
}

/// The probe: the host clock between marks, split into each AI house's
/// part and the step's, and the AI's parts by name.
#[derive(Default)]
struct Clock {
    last: Option<Instant>,
    /// The house whose part of the tick runs, if any.
    house: Option<u8>,
    /// The AI part that runs (`ai:begin` before the brain's first mark).
    part: &'static str,
    /// This tick's AI microseconds per house.
    ai: BTreeMap<u8, f64>,
    /// This tick's AI microseconds per house and part.
    parts: BTreeMap<(u8, &'static str), f64>,
    /// This tick's step microseconds (the rest of the tick).
    step: f64,
}

impl Clock {
    fn begin(&mut self) {
        self.ai.clear();
        self.parts.clear();
        self.step = 0.0;
        self.house = None;
        self.last = Some(Instant::now());
    }

    fn close(&mut self) {
        self.mark(Phase::Done);
        self.last = None;
    }
}

impl Probe for Clock {
    fn mark(&mut self, phase: Phase) {
        let now = Instant::now();
        if let Some(last) = self.last {
            let us = now.duration_since(last).as_secs_f64() * 1e6;
            match self.house {
                Some(h) => {
                    *self.ai.entry(h).or_default() += us;
                    *self.parts.entry((h, self.part)).or_default() += us;
                }
                None => self.step += us,
            }
        }
        match phase {
            Phase::Think(h) => {
                self.house = Some(h);
                self.part = "ai:begin";
            }
            Phase::Part(n) if self.house.is_some() => self.part = n,
            Phase::Part(_) => {}
            _ => self.house = None,
        }
        self.last = Some(now);
    }
}

/// One tick's numbers.
struct Row {
    tick: i32,
    ai: BTreeMap<u8, f64>,
    parts: BTreeMap<(u8, &'static str), f64>,
    step: f64,
}

/// Mean and largest of a sample.
fn mean_max(v: &[f64]) -> (f64, f64) {
    if v.is_empty() {
        return (0.0, 0.0);
    }
    (
        v.iter().sum::<f64>() / v.len() as f64,
        v.iter().copied().fold(0.0, f64::max),
    )
}

/// The seed of `o`'s game: its own, else the one the gate (or the frame
/// benchmark) plays.
fn seed_of(o: &Options) -> u32 {
    // nocite: the sim gate's seeds of these duels (sim-gate DUEL_SEED, M6C_SEED, M7A_SEED, M7B_SEED) and the frame benchmark's seed 7
    let d = match o.game.as_str() {
        "m6c" => 2,
        "m7a" => 17,
        "m7b" => 11,
        "install" => 7,
        _ => 20_261_018,
    };
    o.seed.unwrap_or(d)
}

/// The synthetic game of `o`.
fn synthetic(o: &Options) -> Result<Game, String> {
    let seed = seed_of(o);
    let plain = |map: glasnost_map::Map, name: &str| {
        Game::new(
            fixture::rules(),
            &map,
            name,
            fixture::catalogue(),
            fixture::setup(seed),
        )
        .map_err(|e| format!("{e:?}"))
    };
    match o.game.as_str() {
        "fixture" => plain(fixture::map(), "synthetic"),
        "naval" => plain(fixture::map_naval(), "synthetic-naval"),
        // nocite: our own tooling (the placement tests' pocket of 5 cells)
        "cramped" => plain(fixture::map_cramped(5), "synthetic-cramped"),
        "strait" => plain(fixture::map_strait(), "synthetic-strait"),
        "islands" => plain(fixture::map_islands(), "synthetic-islands"),
        "m6c" => Game::new(
            fixture_m6c::rules(),
            &fixture_m6c::map(),
            fixture_m6c::MAP_NAME,
            fixture::catalogue(),
            fixture_m6c::setup(seed),
        )
        .map_err(|e| format!("{e:?}")),
        "m7a" => Game::new(
            fixture_m7a::rules(),
            &fixture_m7a::map(),
            fixture_m7a::MAP_NAME,
            fixture::catalogue(),
            fixture_m7a::setup(seed),
        )
        .map_err(|e| format!("{e:?}")),
        "m7b" => Game::new(
            fixture_m7b::rules(),
            &fixture_m7b::map(),
            fixture_m7b::MAP_NAME,
            fixture::catalogue(),
            fixture_m7b::setup(seed),
        )
        .map_err(|e| format!("{e:?}")),
        g => Err(format!("unknown game {g}")),
    }
}

/// Runs the benchmark; writes `<label>.txt` and `<label>-hashes.txt` to
/// `out` and returns the summary. `dir` is the install (`game=install`).
pub fn run(o: &Options, dir: Option<&str>) -> Result<String, String> {
    let seed = seed_of(o);
    if o.game == "install" {
        let Some(dir) = dir else {
            return Err("game=install needs GLASNOST_GAME_DIR".into());
        };
        let spec = format!("map={} seed={seed} players={} late=0", o.map, o.players);
        let options = glasnost_app::game::bench::BenchOptions::parse(&spec)?;
        let (mut client, _) = glasnost_app::game::bench::start_headless(options, dir)?;
        let Some(session) = client.session.as_mut() else {
            return Err("the game did not start".into());
        };
        let name = format!("install map {} players {}, seed {seed}", o.map, o.players);
        return bench(o, &name, &mut session.play.game);
    }
    let mut game = synthetic(o)?;
    bench(o, &format!("{}, seed {seed}", o.game), &mut game)
}

/// Plays and times `game`, then writes and returns the summary.
fn bench(o: &Options, name: &str, game: &mut Game) -> Result<String, String> {
    let houses = game.ai_houses();
    let mut rows: Vec<Row> = Vec::with_capacity(o.ticks as usize);
    let mut hashes = Vec::new();
    let mut clock = Clock::default();
    for i in 0..o.ticks {
        let tick = game.world.tick;
        clock.begin();
        game.step_with(&mut clock);
        clock.close();
        game.world.effects.clear();
        rows.push(Row {
            tick,
            ai: std::mem::take(&mut clock.ai),
            parts: std::mem::take(&mut clock.parts),
            step: clock.step,
        });
        if o.hash > 0 && (i + 1).is_multiple_of(o.hash) {
            hashes.push(format!("{} {}", game.world.tick, game.state_hash()));
        }
        if game.outcome().is_some() {
            break;
        }
    }
    let end = format!("{} {}", game.world.tick, game.state_hash());
    if hashes.last() != Some(&end) {
        hashes.push(end);
    }
    let summary = report(name, &houses, &rows, game.world.objects.len(), &hashes);
    let label = o.label.clone().unwrap_or_else(|| o.game.clone());
    let out = std::path::Path::new(&o.out);
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    std::fs::write(out.join(format!("{label}.txt")), &summary).map_err(|e| e.to_string())?;
    std::fs::write(
        out.join(format!("{label}-hashes.txt")),
        hashes.join("\n") + "\n",
    )
    .map_err(|e| e.to_string())?;
    Ok(summary)
}

/// The summary of a run.
fn report(
    name: &str,
    houses: &[HouseId],
    rows: &[Row],
    objects: usize,
    hashes: &[String],
) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "Glasnost AI think benchmark: {name}");
    let _ = writeln!(
        s,
        "ticks: {} (from {} to {}); objects at the end {objects}",
        rows.len(),
        rows.first().map_or(0, |r| r.tick),
        rows.last().map_or(0, |r| r.tick)
    );
    let steps: Vec<f64> = rows.iter().map(|r| r.step).collect();
    let (step_mean, step_max) = mean_max(&steps);
    let ai_total: f64 = rows.iter().flat_map(|r| r.ai.values()).sum();
    let step_total: f64 = steps.iter().sum();
    let _ = writeln!(
        s,
        "tick::step (µs): mean {step_mean:.1}, max {step_max:.0}; AI share of all step time {:.1} %",
        100.0 * ai_total / (ai_total + step_total).max(1e-9)
    );
    let _ = writeln!(
        s,
        "\nper house (µs): think-tick AI mean / max, think ÷ step mean, ticks between thinks mean / max"
    );
    let mut all_think = Vec::new();
    let mut all_rest = Vec::new();
    for &h in houses {
        let (mut think, mut rest) = (Vec::new(), Vec::new());
        for r in rows {
            let us = r.ai.get(&h.0).copied().unwrap_or(0.0);
            if thinks_at(h, r.tick) {
                think.push(us);
            } else {
                rest.push(us);
            }
        }
        let (tm, tx) = mean_max(&think);
        let (rm, rx) = mean_max(&rest);
        let _ = writeln!(
            s,
            "  house {:>2}: {tm:>9.1} / {tx:>9.0}   {:>7.1}×   {rm:>8.1} / {rx:>8.0}",
            h.0,
            tm / step_mean.max(1e-9)
        );
        all_think.extend(think);
        all_rest.extend(rest);
    }
    let (tm, tx) = mean_max(&all_think);
    let (rm, rx) = mean_max(&all_rest);
    let _ = writeln!(
        s,
        "  all     : {tm:>9.1} / {tx:>9.0}   {:>7.1}×   {rm:>8.1} / {rx:>8.0}",
        tm / step_mean.max(1e-9)
    );
    // The stagger: the costliest tick's AI time as played, against the
    // costliest think period's summed thinks (each house's think in it).
    let tick_ai: Vec<f64> = rows.iter().map(|r| r.ai.values().sum()).collect();
    let (tick_mean, tick_max) = mean_max(&tick_ai);
    let mut periods: BTreeMap<i32, f64> = BTreeMap::new();
    for r in rows {
        for &h in houses {
            if thinks_at(h, r.tick) {
                *periods.entry(r.tick.div_euclid(THINK_PERIOD)).or_default() +=
                    r.ai.get(&h.0).copied().unwrap_or(0.0);
            }
        }
    }
    let sums: Vec<f64> = periods.values().copied().collect();
    let (sum_mean, sum_max) = mean_max(&sums);
    let _ = writeln!(
        s,
        "\nAI time a tick (µs, every house): mean {tick_mean:.1}, max {tick_max:.0}\nthinks of one period summed (µs, the tick cost if every house thought in one tick): mean {sum_mean:.1}, max {sum_max:.0}"
    );
    // The AI's parts, on think ticks and between them.
    let mut think_parts: BTreeMap<&'static str, f64> = BTreeMap::new();
    let mut rest_parts: BTreeMap<&'static str, f64> = BTreeMap::new();
    for r in rows {
        for (&(h, p), &us) in &r.parts {
            let thinks = houses.iter().any(|x| x.0 == h && thinks_at(*x, r.tick));
            let m = if thinks {
                &mut think_parts
            } else {
                &mut rest_parts
            };
            *m.entry(p).or_default() += us;
        }
    }
    for (title, m) in [
        ("think ticks", &think_parts),
        ("ticks between thinks", &rest_parts),
    ] {
        let sum: f64 = m.values().sum();
        let mut by: Vec<(&str, f64)> = m.iter().map(|(p, v)| (*p, *v)).collect();
        by.sort_by(|a, b| b.1.total_cmp(&a.1));
        let _ = writeln!(
            s,
            "\nAI parts on {title} (share of that AI time, total ms):"
        );
        for (p, v) in by {
            let _ = writeln!(
                s,
                "  {p:<28} {:>6.2} %  {:>10.1}",
                100.0 * v / sum.max(1e-9),
                v / 1000.0
            );
        }
    }
    if let Some(last) = hashes.last() {
        let _ = writeln!(s, "\nstate hash at the end: {last}");
    }
    s
}
