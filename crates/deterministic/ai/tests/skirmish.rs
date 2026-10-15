//! The scripted skirmish on the synthetic fixture (M5a): each house deploys
//! its MCV, builds power, a refinery and factories by orders, harvests and
//! produces units, and the game is played to the last side (S-0414). The
//! replay of the recorded orders reproduces the game, also across a save
//! and load.

use std::collections::BTreeMap;

use glasnost_ai::{fixture, skirmish};
use glasnost_sim::econ::tech;
use glasnost_sim::rules::Kind;
use glasnost_sim::save;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, World};

/// Ticks the synthetic game may take at most.
const LIMIT: i32 = 40_000;

/// The seed of the synthetic game.
const SEED: u32 = 5;

fn run_for(ticks: i32) -> skirmish::Run {
    skirmish::run(
        fixture::rules(),
        &fixture::map(),
        "synthetic-skirmish",
        testkit::catalogue(),
        fixture::setup(SEED),
        &fixture::plan(),
        ticks,
    )
    .expect("the synthetic skirmish loads")
}

/// The house's objects on the map by type name.
fn counts(w: &World, h: u8) -> BTreeMap<String, usize> {
    let r = w.rules();
    let mut out = BTreeMap::new();
    for o in w
        .objects
        .values()
        .filter(|o| o.owner == Some(HouseId(h)) && o.on_map && o.health > 0)
    {
        let t = tech::type_of(o);
        let name = match t.kind {
            Kind::Building => r.buildings.get(t.index as usize).map(|b| b.name.clone()),
            Kind::Vehicle => r.vehicles.get(t.index as usize).map(|u| u.name.clone()),
            Kind::Infantry => r.infantry.get(t.index as usize).map(|u| u.name.clone()),
            _ => None,
        };
        if let Some(n) = name {
            *out.entry(n).or_default() += 1;
        }
    }
    out
}

// The game is played through orders to the last side, and every house
// went through the M5a loop on the way.
#[test]
#[ignore = "long in debug builds; run in release (`cargo test --release -p glasnost-ai -- --include-ignored`)"]
fn plays_to_the_last_side() {
    let run = run_for(LIMIT);
    assert!(run.over, "the game is over within {LIMIT} ticks");
    assert!(run.winner.is_some(), "one house won");
    // Replaying the orders, watch each house's highest counts and credits.
    let mut most: [BTreeMap<String, usize>; 2] = Default::default();
    let mut credit_rises = [0u32; 2];
    let mut last_credits = [i32::MAX; 2];
    let world = run
        .replay
        .run(
            fixture::rules(),
            &fixture::map(),
            testkit::catalogue(),
            |w| {
                for h in 0..2u8 {
                    for (k, v) in counts(w, h) {
                        let e = most[usize::from(h)].entry(k).or_default();
                        *e = (*e).max(v);
                    }
                    let c = w.houses[usize::from(h)].credits;
                    if c > last_credits[usize::from(h)] {
                        credit_rises[usize::from(h)] += 1;
                    }
                    last_credits[usize::from(h)] = c;
                }
            },
        )
        .expect("the replay runs");
    for h in 0..2 {
        for name in ["YARD", "POWR", "PROC", "PILL", "FACT", "MINR"] {
            assert!(
                most[h].get(name).copied().unwrap_or(0) >= 1,
                "house {h} had a {name}: {:?}",
                most[h]
            );
        }
        let army =
            most[h].get("GUNR").copied().unwrap_or(0) + most[h].get("WALKER").copied().unwrap_or(0);
        assert!(army >= 3, "house {h} produced an army: {:?}", most[h]);
        assert!(credit_rises[h] > 0, "house {h} was paid for ore");
    }
    // The replay reproduces the game without the scripts.
    assert_eq!(save::state_hash(&world), save::state_hash(&run.world));
}

// A save in mid-game, loaded and continued, ends in the same state.
#[test]
#[ignore = "long in debug builds; run in release (`cargo test --release -p glasnost-ai -- --include-ignored`)"]
fn save_and_load_mid_game() {
    let run = run_for(LIMIT);
    let rep = &run.replay;
    let mut w = glasnost_sim::load::new_game(
        fixture::rules(),
        &fixture::map(),
        testkit::catalogue(),
        &rep.setup,
    )
    .expect("loads");
    rep.run_on(&mut w, rep.ticks / 2, |_| {});
    let bytes = save::save(&w);
    let mut loaded =
        save::load(&bytes, fixture::rules(), testkit::catalogue()).expect("the save loads");
    assert_eq!(save::state_hash(&loaded), save::state_hash(&w));
    rep.run_on(&mut loaded, rep.ticks, |_| {});
    assert_eq!(save::state_hash(&loaded), save::state_hash(&run.world));
}

// The opening: both MCVs deploy by order and a build order follows.
#[test]
fn opening_by_orders() {
    let run = run_for(300);
    for h in 0..2u8 {
        let c = counts(&run.world, h);
        assert_eq!(c.get("YARD"), Some(&1), "house {h}: {c:?}");
        assert_eq!(c.get("MCV"), None, "house {h}: {c:?}");
    }
    assert!(
        run.replay
            .orders
            .iter()
            .any(|o| matches!(o.2, glasnost_sim::orders::Order::Build { .. })),
        "a build order was issued"
    );
}

// The tech climb: with the base standing, each house builds the chain
// RADR then LAB that unlocks the V3-like launcher LNCH, builds launchers
// and fires their rockets (S-0506) in play.
#[test]
#[ignore = "long in debug builds; run in release (`cargo test --release -p glasnost-ai -- --include-ignored`)"]
fn climbs_to_the_launcher_and_fires() {
    let run = run_for(LIMIT);
    let mut most: [BTreeMap<String, usize>; 2] = Default::default();
    let mut first: [BTreeMap<String, i32>; 2] = Default::default();
    let mut rockets = 0usize;
    run.replay
        .run(
            fixture::rules(),
            &fixture::map(),
            testkit::catalogue(),
            |w| {
                for h in 0..2u8 {
                    for (k, v) in counts(w, h) {
                        let e = most[usize::from(h)].entry(k.clone()).or_default();
                        *e = (*e).max(v);
                        first[usize::from(h)].entry(k).or_insert(w.tick);
                    }
                }
                rockets += w
                    .objects
                    .values()
                    .filter(|o| o.on_map && o.rocket.as_ref().is_some_and(|r| r.phase.is_some()))
                    .count();
            },
        )
        .expect("the replay runs");
    let mut launchers = 0;
    for h in 0..2 {
        let f = &first[h];
        if let (Some(r), Some(t)) = (f.get("RADR"), f.get("LAB")) {
            assert!(r < t, "house {h}: RADR before LAB: {f:?}");
        }
        if let (Some(t), Some(l)) = (f.get("LAB"), f.get("LNCH")) {
            assert!(t < l, "house {h}: LAB before LNCH: {f:?}");
        }
        launchers += most[h].get("LNCH").copied().unwrap_or(0);
    }
    assert!(launchers > 0, "a house built a launcher: {most:?}");
    assert!(rockets > 0, "a rocket flew");
}
