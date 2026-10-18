//! Arena mode (M7b step 4c, PLAN.md §6 layer 2B).
//!
//! - `synthetic_arena_is_deterministic`: a synthetic matchup over a few
//!   seeds gives the identical report twice (in CI).
//! - `arena_records_parse`: every arena measurement record under
//!   `measurements/` parses (its scenario; the outcomes may be local-only),
//!   and passes with none (in CI).
//! - `arena_matches_the_oracle`: the local lane's statistical comparison
//!   (never in CI): with `GLASNOST_GAME_DIR` set, every arena record's
//!   matchup is played on the reference ruleset over the record's N seeds
//!   (or `GLASNOST_ARENA_N`) and compared with the record: a two-proportion
//!   z test on each group's win rate and a two-sample Kolmogorov–Smirnov
//!   test on the times to kill, at `arena::ALPHA`.

use std::path::{Path, PathBuf};

use glasnost_tools::arena::{self, ALPHA, ArenaRules, Cli, Scenario};
use glasnost_tools::local_lane::GAME_DIR_VAR;

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

#[test]
fn arena_records_parse() {
    let records = arena::load_records(&measurements(), false).unwrap();
    if records.is_empty() {
        println!("skipped: no arena measurement record in measurements/");
    }
    for r in &records {
        println!("{} block {}: {:?}", r.id, r.block + 1, r.scenario);
    }
}

#[test]
fn arena_matches_the_oracle() {
    let records = match arena::load_records(&measurements(), false) {
        Ok(r) => r,
        Err(e) => panic!("{e}"),
    };
    if records.is_empty() {
        println!("skipped: no arena measurement record in measurements/");
        return;
    }
    let Some(dir) = std::env::var_os(GAME_DIR_VAR) else {
        println!(
            "skipped: {GAME_DIR_VAR} is not set (the comparison needs the reference ruleset; local-only lane)"
        );
        return;
    };
    let rules = match arena::reference_rules(Path::new(&dir)) {
        Ok(Ok(r)) => ArenaRules::Reference(r),
        Ok(Err(e)) => panic!("the reference rules did not resolve: {e}"),
        Err(e) => panic!("could not read the install ({:?})", e.kind()),
    };
    let seeds = std::env::var("GLASNOST_ARENA_N")
        .ok()
        .and_then(|v| v.parse().ok());
    let cli = Cli {
        scenario: None,
        seeds,
        first_seed: 1,
        records: true,
        synthetic: false,
        types: false,
        watch: false,
    };
    let report = arena::run_cli(&cli, &rules, &measurements()).unwrap();
    println!("{report}");
    assert!(
        report.contains(" 0 failed"),
        "an arena matchup is outside the oracle's tolerance (α = {ALPHA})"
    );
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
