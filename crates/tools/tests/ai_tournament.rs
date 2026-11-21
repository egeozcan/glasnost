//! The AI tournament (#449) on the synthetic fixture: the same options give
//! the same report however many threads ran, and each game's replay plays
//! the game again.

use glasnost_ai::game::Game;
use glasnost_sim::save::Replay;
use glasnost_sim::world::{AiDifficulty, AiSeat};
use glasnost_tools::ai_tournament::{self, GameResult, Options};

fn options(threads: usize, out: &std::path::Path) -> Options {
    Options {
        game: "fixture".into(),
        entrants: [AiDifficulty::Easy, AiDifficulty::Hard]
            .into_iter()
            .map(|difficulty| AiSeat {
                script: AiSeat::DEFAULT_SCRIPT,
                difficulty,
            })
            .collect(),
        seeds: vec![3],
        maps: vec![0],
        sides: None,
        ticks: 600,
        threads,
        out: out.to_string_lossy().into_owned(),
    }
}

#[test]
fn the_same_options_give_the_same_report() {
    let dir = std::env::temp_dir().join(format!("glasnost-ai-tournament-{}", std::process::id()));
    let one = ai_tournament::run(&options(1, &dir.join("one")), None).expect("runs");
    let two = ai_tournament::run(&options(2, &dir.join("two")), None).expect("runs");
    let again = ai_tournament::run(&options(2, &dir.join("again")), None).expect("runs");
    assert_eq!(one, two);
    assert_eq!(two, again);
    assert!(one.contains("easy") && one.contains("hard"), "{one}");
    assert!(!one.contains("failed:"), "{one}");
    // Two entrants: one pair, two starts, two sides.
    let games = ai_tournament::jobs(&options(1, &dir), 1);
    assert_eq!(games.len(), 4);
    let written = std::fs::read_to_string(dir.join("one").join("report.txt")).expect("report");
    assert_eq!(written, one);
    // Each replay plays its game again to the same hash, with the choices.
    let grounds = ai_tournament::fixture_grounds();
    for (n, job) in games.iter().enumerate() {
        let played = ai_tournament::play(&options(1, &dir), &grounds[0], *job).expect("plays");
        let bytes = std::fs::read(dir.join("one/replays").join(format!("game-{n:03}.replay")))
            .expect("replay");
        assert_eq!(bytes, played.replay);
        let replay = Replay::from_bytes(&bytes).expect("reads");
        let o = options(1, &dir);
        assert_eq!(replay.setup.ai, vec![o.entrants[job.a], o.entrants[job.b]]);
        let g = &grounds[0];
        let again = Game::run_replay(&replay, g.rules.clone(), &g.map, g.tiles.clone(), |_| {})
            .expect("replays");
        assert_eq!(&again.state_hash()[..16], played.hash);
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_ratings_follow_the_games_in_order() {
    // Nothing played: everyone at the start.
    assert_eq!(
        ai_tournament::ratings(3, &[]),
        vec![ai_tournament::START_RATING; 3]
    );
    assert_eq!(ai_tournament::by_worth([300, 200]), GameResult::A);
    assert_eq!(ai_tournament::by_worth([299, 200]), GameResult::Draw);
    assert_eq!(ai_tournament::by_worth([200, 300]), GameResult::B);
    assert_eq!(ai_tournament::by_worth([0, 0]), GameResult::Draw);
    assert_eq!(ai_tournament::by_worth([1, 0]), GameResult::A);
}

#[test]
fn the_options_parse() {
    let words: Vec<String> = [
        "entrants=default:easy,default",
        "seeds=2-4",
        "maps=0,1",
        "ticks=90",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    let o = Options::parse(&words).expect("parses");
    assert_eq!(o.entrants.len(), 2);
    assert_eq!(o.entrants[1], AiSeat::DEFAULT);
    assert_eq!(o.seeds, vec![2, 3, 4]);
    assert_eq!(o.maps, vec![0, 1]);
    assert!(Options::parse(&["entrants=default".to_owned()]).is_err());
    assert!(Options::parse(&["entrants=nobody:easy,default".to_owned()]).is_err());
}

/// A game of house 0's entrant `a` against house 1's `b` on seed `seed`.
fn played(a: usize, b: usize, seed: u32, result: GameResult) -> ai_tournament::Played {
    ai_tournament::Played {
        job: ai_tournament::Job {
            ground: 0,
            seed,
            a,
            b,
            swap_start: false,
            swap_side: false,
        },
        result,
        over: true,
        // nocite: test data of our own
        ticks: 1,
        worth: [0, 0],
        hash: String::new(),
        replay: Vec::new(),
    }
}

/// Games whose table Elo depends on their order.
fn some_games() -> Vec<ai_tournament::Played> {
    // nocite: test data of our own
    vec![
        played(0, 1, 1, GameResult::A),
        played(0, 1, 2, GameResult::A),
        played(1, 2, 3, GameResult::B),
        played(0, 2, 4, GameResult::Draw),
        played(0, 1, 5, GameResult::B),
        played(1, 2, 6, GameResult::A),
        played(0, 2, 7, GameResult::B),
    ]
}

// design: own AI (#449): the order-free Elo is the mean final Elo over seeded shuffles of the games, so the same games give the same ratings in whatever order they come; one game has one order and gives the table's Elo; the ratings still sum to the start times the entrants
#[test]
fn the_order_free_ratings_do_not_depend_on_the_games_order() {
    let games = some_games();
    let mut reversed = games.clone();
    reversed.reverse();
    assert_ne!(
        ai_tournament::ratings(3, &games),
        ai_tournament::ratings(3, &reversed),
        "the table's Elo depends on the order of these games"
    );
    let free = ai_tournament::order_free_ratings(3, &games);
    assert_eq!(free, ai_tournament::order_free_ratings(3, &games));
    assert_eq!(free, ai_tournament::order_free_ratings(3, &reversed));
    let sum: f64 = free.iter().sum();
    // nocite: test data of our own (a rounding margin)
    assert!((sum - 3.0 * ai_tournament::START_RATING).abs() < 1e-6, "{free:?}");
    assert_eq!(
        ai_tournament::order_free_ratings(2, &games[..1]),
        ai_tournament::ratings(2, &games[..1])
    );
    assert_eq!(
        ai_tournament::order_free_ratings(3, &[]),
        vec![ai_tournament::START_RATING; 3]
    );
}

// design: own AI (#449): the report gives the order-free Elo on a line of its own after the table, each entrant in the table's order
#[test]
fn the_report_gives_the_order_free_ratings() {
    let dir = std::env::temp_dir();
    let mut o = options(1, &dir);
    o.entrants.push(AiSeat::DEFAULT);
    let games = some_games();
    let results: Vec<Result<ai_tournament::Played, String>> =
        games.iter().cloned().map(Ok).collect();
    let report = ai_tournament::report(&o, &ai_tournament::fixture_grounds(), &results);
    let free = ai_tournament::order_free_ratings(3, &games);
    let line = report
        .lines()
        .find(|l| l.starts_with("order-free Elo"))
        .unwrap_or_else(|| panic!("no order-free line in\n{report}"));
    for (i, r) in free.iter().enumerate() {
        let name = ai_tournament::entrant_name(o.entrants[i]);
        assert!(line.contains(&format!("{name} {r:.1}")), "{line}");
    }
}
