//! Arena mode (M7b step 4c, PLAN.md §6 layer 2B).
//!
//! - `synthetic_arena_is_deterministic`: a synthetic matchup over a few
//!   seeds gives the identical report twice (in CI).
//! - `arena_records_parse`: every oracle arena record under
//!   `measurements/` (front matter plus the sibling CSV) parses and has the
//!   protocol's shape, no record's CSV goes unread, and the scored pairings
//!   and MIND against E2's line are M-0017's (in CI).
//! - `arena_matches_the_oracle`: the local lane's comparison (never in
//!   CI): with `GLASNOST_GAME_DIR` set, the records' games are played in
//!   our engine with their setup (`glasnost_tools::oracle`) and compared
//!   per pairing that stands as stock (M-0017): a two-proportion test on
//!   the row's win proportion at `oracle::ALPHA`, and the same winner on
//!   every side where every stock seed gave one.
//! - `oracle_records_are_compared`: the oracle comparison on the synthetic
//!   ground with a temporary record (in CI).
//! - The other tests cover the synthetic arena of `glasnost_tools::arena`
//!   (our own flat map and invented groups), including its fenced-block
//!   record form, kept as a documented alternative.

use std::path::{Path, PathBuf};

use glasnost_tools::arena::{self, ArenaRules, Cli, Scenario};
use glasnost_tools::local_lane::GAME_DIR_VAR;
use glasnost_tools::oracle;

fn measurements() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../measurements")
}

/// A small synthetic matchup: two carts against three walkers.
fn small() -> Scenario {
    let mut sc = Scenario::new(
        vec![("CART".into(), 2)],
        vec![("WALKER".into(), 3), ("CART".into(), 1)],
    );
    sc.map_size = 24;
    sc.start_distance = 8;
    sc.tick_cap = 1500;
    sc
}

#[test]
fn synthetic_arena_is_deterministic() {
    let cli = Cli {
        scenario: Some(small()),
        seeds: Some(3),
        first_seed: 5,
        records: false,
        synthetic: true,
        types: false,
        watch: false,
    };
    let once = arena::run_cli(&cli, &ArenaRules::Synthetic, &measurements()).unwrap();
    let twice = arena::run_cli(&cli, &ArenaRules::Synthetic, &measurements()).unwrap();
    assert_eq!(once, twice);
    assert!(once.contains("N 3:"), "{once}");
    assert_eq!(once.matches("    trial: ").count(), 3, "{once}");
    // Every trial ends inside the cap and the counts add up.
    let trials = arena::run_matchup(ArenaRules::Synthetic.rules(), &small(), 5, 3).unwrap();
    assert_eq!(arena::counts(&trials).iter().sum::<u32>(), 3);
    for t in &trials {
        assert!(t.ticks >= 1 && t.ticks <= 1500);
        // The report's trial line reads back as the same trial.
        assert_eq!(arena::parse_trial(&t.line()).unwrap(), *t);
    }
}

#[test]
fn the_command_line_parses() {
    let args: Vec<String> = [
        "--seeds",
        "4",
        "--ticks",
        "900",
        "--size",
        "20",
        "--distance",
        "6",
        "--speed",
        "4",
        "A:CART*2,WALKER",
        "B:WALKER*3",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let cli = arena::parse_cli(&args).unwrap();
    let sc = cli.scenario.unwrap();
    assert_eq!(cli.seeds, Some(4));
    assert_eq!(
        (sc.tick_cap, sc.map_size, sc.start_distance, sc.game_speed),
        (900, 20, 6, Some(4))
    );
    assert_eq!(sc.groups[0], vec![("CART".into(), 2), ("WALKER".into(), 1)]);
    assert!(arena::parse_cli(&["A:CART".to_string()]).is_err());
    assert!(
        arena::parse_cli(&["--records".to_string()])
            .unwrap()
            .records
    );
    assert!(arena::parse_cli(&["--bogus".to_string()]).is_err());
    // Game speeds 0 to 6, on the command line as in records.
    let speed = |v: &str| {
        arena::parse_cli(&["--speed".into(), v.into(), "A:CART".into(), "B:CART".into()])
            .map(|c| c.scenario.unwrap().game_speed)
    };
    assert_eq!(speed("0"), Ok(Some(0)));
    assert_eq!(speed("6"), Ok(Some(6)));
    assert!(speed("7").is_err());
    // The synthetic ruleset's type list names its infantry and vehicles.
    let list = arena::type_list(&ArenaRules::Synthetic.rules());
    for t in ["CART", "CRUSHER", "BUGGY", "WALKER"] {
        assert!(list.contains(t), "{list}");
    }
}

/// Every even seed's trial is swapped: group B is house 0, stands on the
/// lower-x side and is created first; odd seeds keep group A there.
#[test]
fn even_seeds_swap_the_groups() {
    let sc = small();
    for seed in [1u32, 2] {
        let swapped = seed.is_multiple_of(2);
        assert_eq!(arena::swapped(seed), swapped);
        let a = arena::setup_trial(ArenaRules::Synthetic.rules(), &sc, seed).unwrap();
        assert_eq!(a.swapped, swapped);
        let first = usize::from(swapped);
        let (lead, other) = (&a.units[first], &a.units[1 - first]);
        let o = |id| a.world.obj(id).unwrap();
        // The group at the lower-x side is house 0 and was created first.
        assert!(lead.iter().all(|&id| o(id).owner.map(|h| h.0) == Some(0)));
        assert!(other.iter().all(|&id| o(id).owner.map(|h| h.0) == Some(1)));
        assert!(lead.iter().max() < other.iter().min());
        let max_x = lead.iter().map(|&id| o(id).pos.x).max().unwrap();
        let min_x = other.iter().map(|&id| o(id).pos.x).min().unwrap();
        assert!(max_x < min_x, "seed {seed}");
    }
}

#[test]
fn an_unknown_type_is_refused() {
    let sc = Scenario::new(vec![("NOSUCH".into(), 1)], vec![("CART".into(), 1)]);
    let e = arena::run_trial(ArenaRules::Synthetic.rules(), &sc, 1).unwrap_err();
    assert!(e.contains("NOSUCH"), "{e}");
}

/// Every oracle arena record under `measurements/` (the lead's form: front
/// matter, a Setup `- Map:` line, and the sibling CSV) parses and has the
/// protocol's shape: per pairing seeds 0 to 5 once on each side (measure:
/// M-0001, M-0002, M-0003, Procedure). No record's CSV has a header the
/// arena does not read, so a changed header cannot make the arena skip a
/// record silently. In CI; no game data is read.
#[test]
fn arena_records_parse() {
    let unread = oracle::unrecognised_csvs(&measurements());
    assert!(
        unread.is_empty(),
        "per-game CSVs beside a record whose header is not {:?}: {unread:?}",
        oracle::CSV_HEADER
    );
    let records = oracle::load_records(&measurements()).unwrap();
    if records.is_empty() {
        println!("skipped: no oracle arena record in measurements/");
    }
    for r in &records {
        oracle::check_shape(&r.games).unwrap_or_else(|e| panic!("{}: {e}", r.id));
        assert!(
            !r.map_file.is_empty() && r.map_sha256.len() == 64,
            "{}",
            r.id
        );
        println!(
            "{}: {} games, {} pairings",
            r.id,
            r.games.len(),
            oracle::pairings(&r.games).len()
        );
    }
    // The fenced-block form of `arena::load_records` is a documented
    // alternative that no record uses; it still parses whatever is there.
    arena::load_records(&measurements(), false).unwrap();
}

/// The pairings the comparison scores are those M-0017 confirms as stock:
/// 84 of M-0001's 96, all 24 of M-0002's, 24 of M-0003's 35 (M-0017's
/// table of games; the pairings with SREF or V3 on either side are not
/// scored, M-0001 and M-0003 *Tolerance for tests*). The 7 crashed MIND
/// against E2 games are the only stock games left out, and that pairing's
/// line is M-0017's restated one: n = 5, 3 row wins, 2 column wins, no
/// `none`, every game on side B. In CI; no game data is read.
#[test]
fn scored_pairings_and_mind_against_e2_are_those_of_m0017() {
    let records = oracle::load_records(&measurements()).unwrap();
    if records.is_empty() {
        println!("skipped: no oracle arena record in measurements/");
        return;
    }
    // measure: M-0017 (Setup: pairings per record; Result: the 7 crashed games and the restated line)
    let want = [
        ("M-0001", 84, 96, 7),
        ("M-0002", 24, 24, 0),
        ("M-0003", 24, 35, 0),
    ];
    for (id, scored, all, crashed) in want {
        let r = oracle::find_record(&records, id).unwrap_or_else(|| panic!("no record {id}"));
        let ps = oracle::pairings(&r.games);
        assert_eq!(ps.len(), all, "{id}");
        assert_eq!(
            ps.iter().filter(|p| oracle::confirmed(p)).count(),
            scored,
            "{id}"
        );
        assert_eq!(
            r.games.iter().filter(|g| oracle::crashed(g)).count(),
            crashed,
            "{id}"
        );
        // Our games equal to the stock ones: the scored pairings all pass,
        // and the report counts only them.
        let c = oracle::compare_record(r, &r.games);
        assert_eq!(c.scored().len(), scored, "{id}");
        assert!(c.failing().is_empty(), "{}", oracle::report(&c));
        assert!(
            oracle::report(&c).contains(&format!("scored pairings failing: 0 of {scored}")),
            "{id}"
        );
    }
    let m1 = oracle::find_record(&records, "M-0001").unwrap();
    // measure: M-0017 (MIND against E2 restated: n 5, 3 / 2 / 0, side B seeds 1 to 5, no side-A game)
    let pair = ("MIND".to_owned(), "E2".to_owned());
    assert_eq!(oracle::stock_line(m1, &pair), ([3, 2, 0], [0, 5]));
    let c = oracle::compare_record(m1, &m1.games);
    let p = c.pairings.iter().find(|p| p.pairing == pair).unwrap();
    assert_eq!((p.n, p.excluded, p.counts[0]), (5, 7, [3, 2, 0]));
    assert!(p.confirmed);
}

/// The local lane's comparison (never in CI): with `GLASNOST_GAME_DIR` set,
/// every oracle arena record (or only `GLASNOST_ARENA_RECORD`, an ID) is
/// played in our engine with the record's setup and compared with its
/// CSV per pairing that stands as stock (M-0017): the two-proportion test
/// on the row's wins at p < [`oracle::ALPHA`], and the same winner on every
/// side where the stock seeds all gave it (each record's *Tolerance for
/// tests*). The SREF and V3 pairings are played and shown, not scored. The
/// tables go to `local-lane/m7b/arena/oracle-<ID>.txt` (git-ignored). Fails
/// while any scored pairing is outside the tolerance; see
/// docs/development.md, *Arena mode*, for the open discrepancies.
#[test]
fn arena_matches_the_oracle() {
    let records = oracle::load_records(&measurements()).unwrap();
    if records.is_empty() {
        println!("skipped: no oracle arena record in measurements/");
        return;
    }
    let Some(dir) = std::env::var_os(GAME_DIR_VAR) else {
        println!(
            "skipped: {GAME_DIR_VAR} is not set (the oracle arena needs the install; local-only lane)"
        );
        return;
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cli = oracle::Cli {
        record: std::env::var("GLASNOST_ARENA_RECORD").unwrap_or_else(|_| "all".into()),
        pairings: Vec::new(),
        threads: std::thread::available_parallelism().map_or(4, |n| n.get()),
        out: root.join("local-lane/m7b/arena"),
        measurements: measurements(),
        compare_only: false,
        watch: None,
        trace: None,
        groups: false,
        spread: None,
        ai_houses: false,
    };
    let report = oracle::run_cli(&cli, Some(Path::new(&dir))).unwrap();
    println!("{report}");
    assert!(
        report
            .lines()
            .filter(|l| l.contains("scored pairings failing:"))
            .all(|l| l.contains("scored pairings failing: 0 of")),
        "an oracle arena pairing is outside the record's tolerance (p < {})",
        oracle::ALPHA
    );
}

/// The oracle comparison end to end on the synthetic ground: a record made
/// in a temporary folder from our own games passes; the same record with
/// the winners swapped fails. Invented test data, not measurements.
#[test]
fn oracle_records_are_compared() {
    let ground = oracle::synthetic_ground(40);
    let dir = std::env::temp_dir().join(format!("glasnost-oracle-{}", std::process::id()));
    let topic = dir.join("combat");
    std::fs::create_dir_all(&topic).unwrap();
    let md = "---\nid: M-test\n---\n\n## Setup\n\n- Map: `Synthetic.map` (sha256 `0000000000000000000000000000000000000000000000000000000000000000`).\n";
    let mut jobs = Vec::new();
    for side in [oracle::Side::A, oracle::Side::B] {
        for seed in oracle::SEEDS {
            jobs.push(("CART".to_string(), "WALKER".to_string(), side, seed));
        }
    }
    let ours: Vec<oracle::Game> = oracle::run_jobs(&ground, &jobs, 4)
        .into_iter()
        .collect::<Result<_, _>>()
        .unwrap();
    std::fs::write(topic.join("M-test-same.md"), md).unwrap();
    std::fs::write(topic.join("M-test-same.csv"), oracle::write_csv(&ours)).unwrap();
    let records = oracle::load_records(&dir).unwrap();
    assert_eq!(records.len(), 1);
    oracle::check_shape(&records[0].games).unwrap();
    let c = oracle::compare_record(&records[0], &ours);
    assert!(
        c.failing().is_empty() && c.not_same().is_empty(),
        "{}",
        oracle::report(&c)
    );
    let reversed: Vec<oracle::Game> = ours
        .iter()
        .map(|g| oracle::Game {
            winner: match g.winner {
                oracle::Winner::Row => oracle::Winner::Col,
                oracle::Winner::Col => oracle::Winner::Row,
                w => w,
            },
            ..g.clone()
        })
        .collect();
    let c = oracle::compare_record(&records[0], &reversed);
    assert_eq!(c.failing().len(), 1, "{}", oracle::report(&c));
    let _ = std::fs::remove_dir_all(&dir);
}

/// The whole `--records` path on the synthetic ruleset, over records in a
/// temporary folder: one made from our own trial lines (the comparison
/// passes) and one whose outcomes are reversed (it fails). The records are
/// invented test data, not measurements.
#[test]
fn records_are_run_and_compared() {
    let sc = small();
    let trials = arena::run_matchup(ArenaRules::Synthetic.rules(), &sc, 1, 8).unwrap();
    let lines: String = trials
        .iter()
        .map(|t| format!("trial: {}\n", t.line()))
        .collect();
    let block = |trials: &str| {
        format!(
            "```arena\ngroup-a: {}\ngroup-b: {}\nmap-size: 24\nstart-distance: 8\ntick-cap: 1500\nn: 8\n{trials}```\n",
            sc.group_text(0),
            sc.group_text(1)
        )
    };
    let reversed: String = trials
        .iter()
        .map(|t| {
            let mut r = *t;
            r.outcome = match t.outcome {
                arena::Outcome::A => arena::Outcome::B,
                arena::Outcome::B => arena::Outcome::A,
                o => o,
            };
            r.ticks = t.ticks * 3;
            format!("trial: {}\n", r.line())
        })
        .collect();
    let dir = std::env::temp_dir().join(format!("glasnost-arena-{}", std::process::id()));
    let topic = dir.join("combat");
    std::fs::create_dir_all(&topic).unwrap();
    let write = |name: &str, id: &str, body: &str| {
        std::fs::write(
            topic.join(name),
            format!("---\nid: {id}\nvalue-visibility: committed\n---\n\n{body}"),
        )
        .unwrap();
    };
    write("M-test-same.md", "same", &block(&lines));
    let cli = Cli {
        scenario: None,
        seeds: None,
        first_seed: 1,
        records: true,
        synthetic: true,
        types: false,
        watch: false,
    };
    let report = arena::run_cli(&cli, &ArenaRules::Synthetic, &dir).unwrap();
    assert!(report.contains("1 matchups, 0 failed"), "{report}");
    assert!(report.contains("pass (α = 0.01)"), "{report}");
    write("M-test-reversed.md", "reversed", &block(&reversed));
    let report = arena::run_cli(&cli, &ArenaRules::Synthetic, &dir).unwrap();
    println!("{report}");
    assert!(report.contains("2 matchups, 1 failed"), "{report}");
    let _ = std::fs::remove_dir_all(&dir);
}
