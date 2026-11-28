//! The AI tournament (#449): our own tooling, no game behaviour, so no
//! spec applies and the numbers are ours.
//!
//! Entrants are (script, difficulty) pairs of the AI's registry
//! (`glasnost_ai::scripts`). Every pair of entrants meets in a round robin
//! on every ground (map), seed, start assignment (the first entrant on
//! start 0, then on start 1) and side assignment (the first entrant playing
//! the first country, then the second): four games per pair, ground and
//! seed. Each game is a two-house skirmish of AI houses, played until it is
//! over (S-0414) or the tick cap.
//!
//! **The cap and its tiebreak.** A game still running at the cap is decided
//! by worth: the cost field (S-0403 claim 2) summed over each house's live
//! objects on the map. A house whose worth is at least 3/2 of the other's
//! wins; otherwise the game is a draw. A house with worth and an enemy with
//! none wins.
//!
//! **Elo.** Every entrant starts at [`START_RATING`]; the games are taken in
//! their fixed order (ground, seed, pair, start, side; the order they are
//! listed in), and after each one both entrants move by
//! `K · (score − expected)`, with [`K`] fixed, the score 1 for a win, ½ for
//! a draw and 0 for a loss, and the expected score `1 / (1 + 10^((Rb −
//! Ra) / 400))`. The games run on worker threads, each in its own world;
//! the ratings are computed after all of them, in the fixed order, so the
//! same options give the same report however many threads ran.
//!
//! Output: the report (`report.txt`) and each game's replay
//! (`replays/game-NNN.replay`, `glasnost_sim::save::Replay`: the setup with
//! the houses' AI seats, not their orders, so the AI has to play again).
//! [`REPLAY_COMMAND`] (`glasnost-headless ai-replay FILE`, [`replay`])
//! plays one again: it takes the ground from the replay's map name (a
//! fixture ground's name, or `install map N` for the install's map of
//! listing index N, read from `GLASNOST_GAME_DIR`), checks the rules hash,
//! and reruns the saved seats through `Game::run_replay` (#466).

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use glasnost_ai::fixture;
use glasnost_ai::game::Game;
use glasnost_ai::scripts;
use glasnost_map::Map;
use glasnost_map::terrain::TileCatalogue;
use glasnost_sim::econ::setup::{self, Player, Settings};
use glasnost_sim::econ::{TypeRef, production, victory};
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::world::{AiDifficulty, AiSeat, HouseId, World};

/// The usage line.
pub const USAGE: &str = "ai-tournament [game=fixture|install] [entrants=default:easy,default:normal,default:hard] [seeds=1-2] [maps=0] [sides=A,B] [ticks=30000] [threads=2] [out=DIR]";

/// The command that plays a tournament replay again ([`replay`]).
pub const REPLAY_COMMAND: &str = "glasnost-headless ai-replay";

/// The replay command's usage line.
pub const REPLAY_USAGE: &str =
    "ai-replay FILE (a tournament replay; GLASNOST_GAME_DIR set for an install ground)";

/// The rating every entrant starts at.
// nocite: our own tooling (the usual Elo start)
pub const START_RATING: f64 = 1500.0;

/// The Elo step factor.
// nocite: our own tooling (the usual Elo factor)
pub const K: f64 = 32.0;

/// The worth ratio, as a fraction, that wins a game at the cap.
// nocite: our own tooling (a capped game is a win only for a house clearly ahead)
const CAP_WIN: (i64, i64) = (3, 2);

/// The run's options (`key=value` words).
#[derive(Clone, Debug)]
pub struct Options {
    /// `fixture` (the synthetic skirmish, CI-safe) or `install`.
    pub game: String,
    /// The entrants, in the report's order.
    pub entrants: Vec<AiSeat>,
    /// The seeds.
    pub seeds: Vec<u32>,
    /// The grounds: indices of the fixture maps ([`fixture_grounds`]) or of
    /// the install's maps.
    pub maps: Vec<usize>,
    /// The install's pair of sides (their first open countries); none: the
    /// first two sides.
    pub sides: Option<(i32, i32)>,
    /// The tick cap.
    pub ticks: i32,
    /// Worker threads.
    pub threads: usize,
    /// Output folder.
    pub out: String,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            game: "fixture".into(),
            entrants: AiDifficulty::ALL
                .into_iter()
                .map(|difficulty| AiSeat {
                    script: AiSeat::DEFAULT_SCRIPT,
                    difficulty,
                })
                .collect(),
            seeds: vec![1],
            maps: vec![0],
            sides: None,
            // nocite: our own tooling (a cap long enough for most AI duels to end)
            ticks: 27_000,
            threads: 2,
            out: "local-lane/ai-issues/b9/tournament".into(),
        }
    }
}

/// An entrant's name: `key:difficulty`.
pub fn entrant_name(s: AiSeat) -> String {
    let key = scripts::entry(s.script).map_or("?", |e| e.key);
    format!("{key}:{}", s.difficulty.name().to_ascii_lowercase())
}

/// Parses an entrant: `key:difficulty`, or `key` for normal.
pub fn parse_entrant(w: &str) -> Result<AiSeat, String> {
    let (key, level) = w.split_once(':').unwrap_or((w, "normal"));
    let e = scripts::by_key(key).ok_or_else(|| format!("unknown script {key}"))?;
    let difficulty =
        AiDifficulty::parse(level).ok_or_else(|| format!("unknown difficulty {level}"))?;
    Ok(AiSeat {
        script: e.id,
        difficulty,
    })
}

fn parse_list<T: std::str::FromStr>(v: &str) -> Result<Vec<T>, String> {
    v.split(',')
        .map(|x| x.trim().parse().map_err(|_| format!("bad number {x}")))
        .collect()
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
                "entrants" => {
                    o.entrants = v.split(',').map(parse_entrant).collect::<Result<_, _>>()?;
                }
                "seeds" => {
                    o.seeds = match v.split_once('-') {
                        Some((a, b)) => {
                            let (a, b): (u32, u32) =
                                (a.parse().map_err(bad)?, b.parse().map_err(bad)?);
                            (a..=b).collect()
                        }
                        None => parse_list(v)?,
                    };
                }
                "maps" => o.maps = parse_list(v)?,
                "sides" => {
                    let s: Vec<i32> = parse_list(v)?;
                    let [a, b] = s[..] else {
                        return Err(format!("sides takes two: {v}"));
                    };
                    o.sides = Some((a, b));
                }
                "ticks" => o.ticks = v.parse().map_err(bad)?,
                "threads" => o.threads = v.parse().map_err(bad)?,
                "out" => o.out = v.into(),
                _ => return Err(format!("unknown option {k}")),
            }
        }
        if o.entrants.len() < 2 {
            return Err("a tournament needs two entrants".into());
        }
        if o.seeds.is_empty() || o.maps.is_empty() {
            return Err("no seed or no map".into());
        }
        Ok(o)
    }
}

/// One ground: the rules, the map and the two countries.
pub struct Ground {
    /// The name the report shows.
    pub name: String,
    /// The rules.
    pub rules: Arc<SimRules>,
    /// The map.
    pub map: Map,
    /// Its tiles.
    pub tiles: Arc<TileCatalogue>,
    /// The two countries the entrants play.
    pub countries: [String; 2],
}

/// The fixture's grounds: the synthetic skirmish's map and its diagonal
/// one (test data of our own).
pub fn fixture_grounds() -> Vec<Ground> {
    let ground = |name: &str, map: Map| Ground {
        name: name.into(),
        rules: fixture::rules(),
        map,
        tiles: fixture::catalogue(),
        countries: ["Alphaland".into(), "Betaland".into()],
    };
    vec![
        ground("synthetic", fixture::map()),
        ground("synthetic-diagonal", fixture::map_diagonal()),
    ]
}

/// One game of the round robin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Job {
    /// The ground's place in the options' maps.
    pub ground: usize,
    /// The seed.
    pub seed: u32,
    /// House 0's entrant.
    pub a: usize,
    /// House 1's entrant.
    pub b: usize,
    /// House 0 starts on start 1 (else 0).
    pub swap_start: bool,
    /// House 0 plays the second country (else the first).
    pub swap_side: bool,
}

/// Every game, in the fixed order.
pub fn jobs(o: &Options, grounds: usize) -> Vec<Job> {
    let mut out = Vec::new();
    for ground in 0..grounds {
        for &seed in &o.seeds {
            for a in 0..o.entrants.len() {
                for b in a + 1..o.entrants.len() {
                    for swap_start in [false, true] {
                        for swap_side in [false, true] {
                            out.push(Job {
                                ground,
                                seed,
                                a,
                                b,
                                swap_start,
                                swap_side,
                            });
                        }
                    }
                }
            }
        }
    }
    out
}

/// How a game ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameResult {
    /// House 0's entrant won.
    A,
    /// House 1's entrant won.
    B,
    /// A draw.
    Draw,
}

/// A game's record.
#[derive(Clone, Debug)]
pub struct Played {
    /// The game.
    pub job: Job,
    /// The result.
    pub result: GameResult,
    /// Whether the game ended before the cap.
    pub over: bool,
    /// The ticks run.
    pub ticks: i32,
    /// Each house's worth at the end.
    pub worth: [i64; 2],
    /// The state hash at the end, its first 16 digits.
    pub hash: String,
    /// The replay's bytes.
    pub replay: Vec<u8>,
}

/// A house's worth: the cost field summed over its live objects on the map.
pub fn worth(w: &World, h: HouseId) -> i64 {
    w.objects
        .values()
        .filter(|o| o.owner == Some(h) && o.on_map && o.health > 0)
        .filter(|o| {
            matches!(
                o.kind,
                Kind::Building | Kind::Vehicle | Kind::Infantry | Kind::Aircraft
            )
        })
        .map(|o| {
            i64::from(production::cost_field(
                w,
                TypeRef {
                    kind: o.kind,
                    index: o.type_index,
                },
            ))
        })
        .sum()
}

/// The capped game's result by worth (see the module's tiebreak).
pub fn by_worth(worth: [i64; 2]) -> GameResult {
    let (n, d) = CAP_WIN;
    if worth[0] * d >= worth[1] * n && worth[0] > worth[1] {
        GameResult::A
    } else if worth[1] * d >= worth[0] * n && worth[1] > worth[0] {
        GameResult::B
    } else {
        GameResult::Draw
    }
}

/// Plays one game.
pub fn play(o: &Options, g: &Ground, job: Job) -> Result<Played, String> {
    let (start_a, start_b) = if job.swap_start { (1, 0) } else { (0, 1) };
    let (side_a, side_b) = if job.swap_side { (1, 0) } else { (0, 1) };
    let player = |name: &str, side: usize, start: u32| Player {
        name: name.into(),
        country: g.countries[side].clone(),
        start: Some(start),
        ai: true,
        values: Default::default(),
    };
    let mut s = setup::skirmish(
        &g.rules,
        &[player("A", side_a, start_a), player("B", side_b, start_b)],
        Settings::default(),
        0,
        job.seed,
    );
    s.ai = vec![o.entrants[job.a], o.entrants[job.b]];
    let mut game = Game::new(g.rules.clone(), &g.map, &g.name, g.tiles.clone(), s)
        .map_err(|e| format!("{e:?}"))?;
    while glasnost_sim::tick::ticks_run(&game.world) < o.ticks && game.outcome().is_none() {
        game.step();
        game.world.effects.clear();
    }
    let end = end_of(&game);
    Ok(Played {
        job,
        result: end.result,
        over: end.over,
        ticks: end.ticks,
        worth: end.worth,
        hash: end.hash[..16].to_owned(),
        replay: game.replay().to_bytes(),
    })
}

/// How a played game stands at its end.
struct End {
    result: GameResult,
    over: bool,
    ticks: i32,
    worth: [i64; 2],
    /// The full state hash.
    hash: String,
}

/// The end of a game: its result (the victory rules', else by worth), the
/// ticks run, the worths and the state hash.
fn end_of(game: &Game) -> End {
    let worth = [
        worth(&game.world, HouseId(0)),
        worth(&game.world, HouseId(1)),
    ];
    let (result, over) = match game.outcome() {
        Some(out) => {
            let winner = out
                .results
                .iter()
                .find(|r| r.1 == victory::Verdict::Win)
                .map(|r| r.0);
            let r = match winner {
                Some(HouseId(0)) => GameResult::A,
                Some(HouseId(1)) => GameResult::B,
                _ => GameResult::Draw,
            };
            (r, true)
        }
        None => (by_worth(worth), false),
    };
    End {
        result,
        over,
        ticks: glasnost_sim::tick::ticks_run(&game.world),
        worth,
        hash: game.state_hash(),
    }
}

/// The ground a tournament replay was played on, from its map name: a
/// fixture ground by its name ([`fixture_grounds`]), or `install map N`,
/// the install's map of listing index N (`dir`, the game dir, needed).
pub fn replay_ground(map: &str, dir: Option<&str>) -> Result<Ground, String> {
    if let Some(m) = map.strip_prefix("install map ") {
        let m: usize = m
            .parse()
            .map_err(|_| format!("not a tournament ground: {map:?}"))?;
        let dir = dir.ok_or_else(|| {
            format!("the replay was played on {map:?}: set GLASNOST_GAME_DIR to the install")
        })?;
        return install_grounds(std::path::Path::new(dir), &[m], None)?
            .pop()
            .ok_or_else(|| format!("no map {m}"));
    }
    fixture_grounds()
        .into_iter()
        .find(|g| g.name == map)
        .ok_or_else(|| format!("not a tournament ground: {map:?}"))
}

/// Plays a tournament replay (its bytes) again (#466): the ground from its
/// map name ([`replay_ground`]), the saved AI seats rerun through
/// `Game::run_replay`. Returns a line in the report's form, ending on the
/// first 16 digits of the state hash, and the full hash on a second line.
pub fn replay(bytes: &[u8], dir: Option<&str>) -> Result<String, String> {
    let rep = glasnost_sim::save::Replay::from_bytes(bytes).map_err(|e| e.to_string())?;
    if !rep.setup.houses.iter().any(|h| h.ai) {
        return Err("the replay has no AI house: play it with glasnost-headless replay".into());
    }
    let g = replay_ground(&rep.map, dir)?;
    let game = Game::run_replay(&rep, g.rules.clone(), &g.map, g.tiles.clone(), |_| {}).map_err(
        |e| match e {
            glasnost_sim::save::SaveError::RulesMismatch { .. } if dir.is_some() => format!(
                "{e} (an install ground's rules depend on GLASNOST_PLAY_FIXES: set it as the tournament had it)"
            ),
            e => e.to_string(),
        },
    )?;
    let end = end_of(&game);
    let res = match end.result {
        GameResult::A => "house 0",
        GameResult::B => "house 1",
        GameResult::Draw => "draw",
    };
    Ok(format!(
        "{} seed {} {} v {}: {res} {} at tick {}, worth {} v {}, hash {}\nstate hash {}\n",
        rep.map,
        rep.setup.seed,
        entrant_name(rep.setup.ai_seat(0)),
        entrant_name(rep.setup.ai_seat(1)),
        if end.over { "(over)" } else { "(cap)" },
        end.ticks,
        end.worth[0],
        end.worth[1],
        &end.hash[..16],
        end.hash
    ))
}

/// Plays every job on `o.threads` worker threads; the results in job
/// order.
pub fn play_all(o: &Options, grounds: &[Ground], jobs: &[Job]) -> Vec<Result<Played, String>> {
    let next = AtomicUsize::new(0);
    type Slot = std::sync::Mutex<Option<Result<Played, String>>>;
    let results: Vec<Slot> = jobs.iter().map(|_| std::sync::Mutex::new(None)).collect();
    std::thread::scope(|s| {
        for _ in 0..o.threads.max(1) {
            s.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(&job) = jobs.get(i) else { break };
                    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        play(o, &grounds[job.ground], job)
                    }))
                    .unwrap_or_else(|_| Err("panic".into()));
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

/// The ratings after `games`, in their order (see the module's Elo).
pub fn ratings(entrants: usize, games: &[Played]) -> Vec<f64> {
    let mut r = vec![START_RATING; entrants];
    for g in games {
        let (a, b) = (g.job.a, g.job.b);
        let expected = 1.0 / (1.0 + 10f64.powf((r[b] - r[a]) / 400.0));
        let score = match g.result {
            GameResult::A => 1.0,
            GameResult::B => 0.0,
            GameResult::Draw => 0.5,
        };
        r[a] += K * (score - expected);
        r[b] -= K * (score - expected);
    }
    r
}

/// The shuffles of the games that the order-free Elo is the mean over.
// nocite: design: own AI (#449): a fixed count, so the same games give the same line, and enough that more shuffles move the mean little
pub const SHUFFLES: u64 = 100;

/// Each entrant's order-free Elo (design: own AI, #449): the mean of its
/// final Elo ([`ratings`]' rule) over [`SHUFFLES`] seeded shuffles of the
/// games. The games are first put in the order of their job (ground, seed,
/// pair, start, side) and result, so the ratings do not depend on the
/// order the games come in; shuffle `n` is a Fisher-Yates shuffle drawn
/// from [`splitmix64`] seeded with `n`.
pub fn order_free_ratings(entrants: usize, games: &[Played]) -> Vec<f64> {
    let score = |r: GameResult| match r {
        GameResult::A => 1.0,
        GameResult::B => 0.0,
        GameResult::Draw => 0.5,
    };
    let rank = |r: GameResult| match r {
        GameResult::A => 0u8,
        GameResult::Draw => 1,
        GameResult::B => 2,
    };
    let mut keyed: Vec<_> = games
        .iter()
        .map(|g| {
            let j = g.job;
            (
                (j.ground, j.seed, j.a, j.b, j.swap_start, j.swap_side),
                rank(g.result),
                score(g.result),
            )
        })
        .collect();
    keyed.sort_by_key(|x| (x.0, x.1));
    let canonical: Vec<(usize, usize, f64)> = keyed
        .iter()
        .map(|&((_, _, a, b, _, _), _, s)| (a, b, s))
        .collect();
    if canonical.len() < 2 {
        return elo_of(entrants, &canonical);
    }
    let mut sum = vec![0.0; entrants];
    for n in 0..SHUFFLES {
        let mut order = canonical.clone();
        let mut state = n;
        for i in (1..order.len()).rev() {
            let j = (splitmix64(&mut state) % (i as u64 + 1)) as usize;
            order.swap(i, j);
        }
        for (s, r) in sum.iter_mut().zip(elo_of(entrants, &order)) {
            *s += r;
        }
    }
    sum.into_iter().map(|s| s / SHUFFLES as f64).collect()
}

/// The final Elo after `games` (house 0's entrant, house 1's, house 0's
/// score), in their order: [`ratings`]' rule.
fn elo_of(entrants: usize, games: &[(usize, usize, f64)]) -> Vec<f64> {
    let mut r = vec![START_RATING; entrants];
    for &(a, b, score) in games {
        // nocite: our own tooling (the usual Elo scale, as in `ratings`)
        let expected = 1.0 / (1.0 + 10f64.powf((r[b] - r[a]) / 400.0));
        r[a] += K * (score - expected);
        r[b] -= K * (score - expected);
    }
    r
}

/// The next number of the splitmix64 generator (our own tooling, #449: a
/// small, fixed generator for the order-free Elo's shuffles).
fn splitmix64(state: &mut u64) -> u64 {
    // nocite: design: own AI (#449): splitmix64's published increment and mixing constants
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    // nocite: design: own AI (#449): splitmix64's published mixing constants and shifts
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    // nocite: design: own AI (#449): splitmix64's published mixing constants and shifts
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    // nocite: design: own AI (#449): splitmix64's published final shift
    z ^ (z >> 31)
}

/// The report of a finished tournament (no timing: the same options give
/// the same text).
pub fn report(o: &Options, grounds: &[Ground], results: &[Result<Played, String>]) -> String {
    let mut s = String::new();
    let games: Vec<Played> = results
        .iter()
        .filter_map(|r| r.as_ref().ok().cloned())
        .collect();
    let _ = writeln!(s, "Glasnost AI tournament (#449)");
    let _ = writeln!(
        s,
        "game {}; grounds {}; seeds {:?}; tick cap {}; games {} ({} failed)",
        o.game,
        grounds
            .iter()
            .map(|g| g.name.as_str())
            .collect::<Vec<_>>()
            .join(", "),
        o.seeds,
        o.ticks,
        results.len(),
        results.len() - games.len()
    );
    let _ = writeln!(
        s,
        "Elo: start {START_RATING}, K {K}, games in their listed order; a game at the cap is won at {}/{} of the other's worth, else drawn",
        CAP_WIN.0, CAP_WIN.1
    );
    let rating = ratings(o.entrants.len(), &games);
    // Per entrant: won, drawn, lost; won by the end of the game, by worth.
    let mut tally: BTreeMap<usize, [usize; 5]> = BTreeMap::new();
    for g in &games {
        for (me, mine) in [(g.job.a, GameResult::A), (g.job.b, GameResult::B)] {
            let t = tally.entry(me).or_default();
            match g.result {
                GameResult::Draw => t[1] += 1,
                r if r == mine => {
                    t[0] += 1;
                    t[if g.over { 3 } else { 4 }] += 1;
                }
                _ => t[2] += 1,
            }
        }
    }
    let _ = writeln!(
        s,
        "\n{:<18} {:>7} {:>5} {:>4} {:>5} {:>4}   {:>9} {:>8}",
        "entrant", "rating", "games", "won", "drawn", "lost", "won: end", "by worth"
    );
    let mut order: Vec<usize> = (0..o.entrants.len()).collect();
    order.sort_by(|&x, &y| rating[y].total_cmp(&rating[x]).then(x.cmp(&y)));
    for &i in &order {
        let t = tally.get(&i).copied().unwrap_or_default();
        let _ = writeln!(
            s,
            "{:<18} {:>7.1} {:>5} {:>4} {:>5} {:>4}   {:>9} {:>8}",
            entrant_name(o.entrants[i]),
            rating[i],
            t[0] + t[1] + t[2],
            t[0],
            t[1],
            t[2],
            t[3],
            t[4]
        );
    }
    // design: own AI (#449): the order-free Elo, a second view beside the table's
    let free = order_free_ratings(o.entrants.len(), &games);
    let _ = writeln!(
        s,
        "\norder-free Elo (the mean over {SHUFFLES} seeded shuffles of the game order): {}",
        order
            .iter()
            .map(|&i| format!("{} {:.1}", entrant_name(o.entrants[i]), free[i]))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let _ = writeln!(
        s,
        "\nreplays: replays/game-NNN.replay for each game below that did not fail; play one again with `{REPLAY_COMMAND} FILE` (with GLASNOST_GAME_DIR set for an install ground)"
    );
    let _ = writeln!(
        s,
        "\ngames (house 0 against house 1; start and country index of house 0):"
    );
    for (n, r) in results.iter().enumerate() {
        match r {
            Ok(g) => {
                let res = match g.result {
                    GameResult::A => "house 0",
                    GameResult::B => "house 1",
                    GameResult::Draw => "draw",
                };
                let _ = writeln!(
                    s,
                    "  {n:>3} {} seed {} {} v {} start {} country {}: {res} {} at tick {}, worth {} v {}, hash {}",
                    grounds[g.job.ground].name,
                    g.job.seed,
                    entrant_name(o.entrants[g.job.a]),
                    entrant_name(o.entrants[g.job.b]),
                    u8::from(g.job.swap_start),
                    u8::from(g.job.swap_side),
                    if g.over { "(over)" } else { "(cap)" },
                    g.ticks,
                    g.worth[0],
                    g.worth[1],
                    g.hash
                );
            }
            Err(e) => {
                let _ = writeln!(s, "  {n:>3} failed: {e}");
            }
        }
    }
    s
}

/// Runs the tournament on `grounds`; writes the report and the replays to
/// `o.out` and returns the report.
pub fn run_on(o: &Options, grounds: &[Ground]) -> Result<String, String> {
    let jobs = jobs(o, grounds.len());
    let results = play_all(o, grounds, &jobs);
    let text = report(o, grounds, &results);
    let out = std::path::Path::new(&o.out);
    let replays = out.join("replays");
    std::fs::create_dir_all(&replays).map_err(|e| e.to_string())?;
    for (n, r) in results.iter().enumerate() {
        if let Ok(g) = r {
            std::fs::write(replays.join(format!("game-{n:03}.replay")), &g.replay)
                .map_err(|e| e.to_string())?;
        }
    }
    std::fs::write(out.join("report.txt"), &text).map_err(|e| e.to_string())?;
    Ok(text)
}

/// Runs the tournament: on the fixture's grounds, or on the install at
/// `dir` (`game=install`).
pub fn run(o: &Options, dir: Option<&str>) -> Result<String, String> {
    let grounds = match o.game.as_str() {
        "fixture" => {
            let all = fixture_grounds();
            let mut picked = Vec::new();
            let mut all: Vec<Option<Ground>> = all.into_iter().map(Some).collect();
            for &m in &o.maps {
                let g = all
                    .get_mut(m)
                    .and_then(Option::take)
                    .ok_or_else(|| format!("no fixture ground {m} (or twice)"))?;
                picked.push(g);
            }
            picked
        }
        "install" => {
            let dir = dir.ok_or("game=install needs GLASNOST_GAME_DIR")?;
            install_grounds(std::path::Path::new(dir), &o.maps, o.sides)?
        }
        g => return Err(format!("unknown game {g}")),
    };
    run_on(o, &grounds)
}

/// The install's grounds: its maps of the given listing indices, with the
/// first open country of each of the two sides.
fn install_grounds(
    dir: &std::path::Path,
    maps: &[usize],
    sides: Option<(i32, i32)>,
) -> Result<Vec<Ground>, String> {
    pollster::block_on(async {
        let mut vfs = crate::local_lane::mount(dir)
            .await
            .map_err(|e| format!("could not read the install ({:?})", e.kind()))?;
        let layers = glasnost_rules::layers::Layers::from_flag(
            std::env::var("GLASNOST_PLAY_FIXES").ok().as_deref(),
        );
        let listings = glasnost_convert::find::find_maps(&vfs).await;
        let mut out = Vec::new();
        for &m in maps {
            let listing = listings.get(m).ok_or_else(|| format!("no map {m}"))?;
            let bytes = vfs
                .read(&listing.file)
                .await
                .map_err(|_| format!("map {m} unreadable"))?;
            let inputs =
                glasnost_convert::game::load(&mut vfs, bytes, "install".into(), layers).await;
            let (Ok(resolved), Some(tiles)) = (inputs.resolved, inputs.tiles) else {
                return Err(format!("map {m} did not load"));
            };
            let rules = Arc::new(SimRules::new(&resolved.ruleset, &resolved.sim_rules_hash));
            let by_side = crate::skirmish_lane::side_countries(&rules);
            let (a, b) = match sides {
                Some(p) => p,
                None => {
                    let mut k = by_side.keys();
                    match (k.next(), k.next()) {
                        (Some(&a), Some(&b)) => (a, b),
                        _ => return Err("fewer than two sides".into()),
                    }
                }
            };
            let country = |s: i32| {
                by_side
                    .get(&s)
                    .cloned()
                    .ok_or_else(|| format!("no open country of side {s}"))
            };
            out.push(Ground {
                name: format!("install map {m}"),
                rules: rules.clone(),
                map: inputs.map,
                tiles: Arc::new(tiles),
                countries: [country(a)?, country(b)?],
            });
        }
        Ok(out)
    })
}
