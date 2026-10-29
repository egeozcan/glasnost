//! Allied AI houses (M8 step B9): two AI houses on the same lobby team
//! start allied (S-0412 claim 8) and never go for each other. No object of
//! either targets one of the other's, nothing of either is hit, and no wave
//! is sent to the ally's base: with no enemy left to find, the waves scout
//! instead (S-0309 claim 18: an allied house's objects are not acquired;
//! the AI reads the house's alliances as the sim does).

use glasnost_ai::{fixture, game::Game, skirmish};
use glasnost_map::CellXY;
use glasnost_sim::econ::setup::{self, HouseValues, Player, Settings};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{GameSetup, HouseId, ObjectId, Target};
use std::collections::BTreeMap;

/// Ticks the allied game is watched: past the first waves (the plan's
/// waves of six leave from about tick 6 000 on this fixture).
const WATCH: i32 = 14_000;

/// The fixture's two AI houses, both on lobby team 1.
fn allied_setup(seed: u32) -> GameSetup {
    let player = |name: &str, country: &str| Player {
        name: name.into(),
        country: country.into(),
        start: None,
        ai: true,
        values: HouseValues {
            team: Some(1),
            ..Default::default()
        },
    };
    setup::skirmish(
        &fixture::rules(),
        &[player("Alpha", "Alphaland"), player("Beta", "Betaland")],
        Settings::default(),
        0,
        seed,
    )
}

fn dist2(a: CellXY, b: CellXY) -> i64 {
    let (dx, dy) = (i64::from(a.x - b.x), i64::from(a.y - b.y));
    dx * dx + dy * dy
}

// spec: S-0412 claim 8 (two houses with the same lobby team start allied, both ways)
// spec: S-0309 claim 18 (an object of an allied house is not acquired)
#[test]
#[ignore = "long in debug builds; run in release (`cargo test --release -p glasnost-ai -- --include-ignored`)"]
fn allied_ai_houses_never_go_for_each_other() {
    let map = fixture::map();
    let mut g = Game::new(
        fixture::rules(),
        &map,
        "synthetic-skirmish",
        fixture::catalogue(),
        allied_setup(5),
    )
    .expect("the synthetic skirmish loads");
    let (a, b) = (HouseId(0), HouseId(1));
    assert!(
        g.world.allied(Some(a), Some(b)) && g.world.allied(Some(b), Some(a)),
        "the lobby team allies the houses"
    );
    // Each house's start: its construction yard's cell once deployed.
    let starts = skirmish::start_cells(&map);
    let mut hp: BTreeMap<ObjectId, i32> = BTreeMap::new();
    let mut hits = 0usize;
    let mut at_ally = 0usize;
    let mut sent_to_ally = [0usize; 2];
    let mut moved = [0usize; 2];
    while g.world.tick < WATCH {
        g.step();
        let w = &g.world;
        assert!(g.outcome().is_none(), "the allied game goes on");
        // The start cell each house took: the one nearest its first
        // building.
        let start_of = |h: HouseId| {
            let at = w
                .objects
                .values()
                .find(|o| o.owner == Some(h) && o.kind == Kind::Building && o.on_map)
                .map(|o| o.pos.cell())?;
            starts.iter().copied().min_by_key(|&c| dist2(c, at))
        };
        let ally_start = [start_of(b), start_of(a)];
        for o in w.objects.values().filter(|o| o.on_map) {
            if let Some(before) = hp.insert(o.id, o.health)
                && o.health < before
            {
                hits += 1;
            }
            let Some(owner) = o.owner else { continue };
            if let Some(Target::Object(t)) = o.target
                && let Some(to) = w.obj(t).and_then(|t| t.owner)
                && to != owner
            {
                at_ally += 1;
            }
            // An army unit moving to the ally's start cell.
            let i = usize::from(owner.0);
            if i < 2
                && matches!(o.kind, Kind::Infantry | Kind::Vehicle)
                && o.beh.current == Some(Behaviour::Move)
            {
                moved[i] += 1;
                if let (Some(Target::Cell(d)), Some(s)) = (o.dest, ally_start[i])
                    && dist2(d, s) <= 25
                {
                    sent_to_ally[i] += 1;
                }
            }
        }
    }
    assert_eq!(hits, 0, "nothing of either house is hit");
    assert_eq!(at_ally, 0, "no object targets the ally's");
    assert!(
        moved.iter().all(|&m| m > 0),
        "both houses' units moved: {moved:?}"
    );
    assert_eq!(sent_to_ally, [0, 0], "no wave goes to the ally's base");
}
