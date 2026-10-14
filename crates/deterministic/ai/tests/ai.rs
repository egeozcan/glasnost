//! The own AI of M5b (PLAN §4.7) on the synthetic skirmish: it deploys,
//! builds an economy, produces, attacks, defends its base, rebuilds what it
//! lost, plays the game to a winner (S-0414, play to the last side), and
//! decides only on what its own house sees.
//!
//! The quick tests run in debug builds; the whole games are ignored there
//! (`cargo test --release -p glasnost-ai -- --include-ignored`).

use std::collections::BTreeMap;

use glasnost_ai::brain::Brain;
use glasnost_ai::{fixture, game::Game};
use glasnost_sim::econ::{shroud, tech, victory};
use glasnost_sim::objects;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

/// Ticks a whole synthetic game may take at most.
const LIMIT: i32 = 40_000;

fn start(seed: u32) -> Game {
    Game::new(
        fixture::rules(),
        &fixture::map(),
        "synthetic-skirmish",
        testkit::catalogue(),
        fixture::setup(seed),
    )
    .expect("the synthetic skirmish loads")
}

fn name(w: &World, id: ObjectId) -> Option<String> {
    let o = w.obj(id)?;
    let r = w.rules();
    let t = tech::type_of(o);
    match t.kind {
        Kind::Building => r.buildings.get(t.index as usize).map(|b| b.name.clone()),
        Kind::Vehicle => r.vehicles.get(t.index as usize).map(|u| u.name.clone()),
        Kind::Infantry => r.infantry.get(t.index as usize).map(|u| u.name.clone()),
        _ => None,
    }
}

/// The house's live objects on the map named `n`.
fn of(w: &World, h: u8, n: &str) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| o.owner == Some(HouseId(h)) && o.on_map && o.health > 0)
        .map(|o| o.id)
        .filter(|&id| name(w, id).as_deref() == Some(n))
        .collect()
}

/// Steps until `done` holds or `limit` ticks have run; returns whether it
/// held.
fn step_until(g: &mut Game, limit: i32, mut done: impl FnMut(&World) -> bool) -> bool {
    // The final tick leaves the tick number at itself (S-0201 claim 1): stop once the game is over.
    while g.world.tick < limit && g.outcome().is_none() {
        if done(&g.world) {
            return true;
        }
        g.step();
    }
    done(&g.world)
}

// The build-up: each AI house deploys its MCV and builds power, a refinery
// and a barracks by orders.
#[test]
fn deploys_and_builds_up() {
    let mut g = start(5);
    let built = step_until(&mut g, 3000, |w| {
        (0..2).all(|h| {
            ["YARD", "POWR", "PROC", "PILL"]
                .iter()
                .all(|n| !of(w, h, n).is_empty())
        })
    });
    assert!(built, "both houses built up by tick {}", g.world.tick);
    for h in 0..2 {
        assert!(
            of(&g.world, h, "MCV").is_empty(),
            "house {h} deployed its MCV"
        );
    }
}

// A lost power plant is rebuilt.
#[test]
fn rebuilds_a_lost_power_plant() {
    let mut g = start(5);
    assert!(step_until(&mut g, 1500, |w| !of(w, 0, "POWR").is_empty()));
    for id in of(&g.world, 0, "POWR") {
        objects::remove(&mut g.world, id);
    }
    let lost = g.world.tick;
    assert!(
        step_until(&mut g, lost + 1500, |w| !of(w, 0, "POWR").is_empty()),
        "the power plant lost at tick {lost} was rebuilt"
    );
}

/// The game when both houses have their first gun vehicle.
fn armed() -> Game {
    let mut g = start(5);
    assert!(step_until(&mut g, 9000, |w| (0..2)
        .all(|h| !of(w, h, "GUNR").is_empty())));
    g
}

// An enemy seen near the base draws the house's army to it in the next
// tick (the reactions run every tick).
#[test]
fn defends_the_base() {
    let mut g = armed();
    let yard = of(&g.world, 0, "YARD")[0];
    let at = g.world.obj(yard).unwrap().pos.cell();
    // On the far side of the yard from the army, out of its reach.
    let gun = g.world.obj(of(&g.world, 0, "GUNR")[0]).unwrap().pos.cell();
    let far = glasnost_map::CellXY::new(
        at.x + 1 + 5 * (at.x - gun.x).signum(),
        at.y + 1 + 5 * (at.y - gun.y).signum(),
    );
    let intruder = testkit::spawn(&mut g.world, Kind::Vehicle, "GUNR", 1, far, 0);
    // The intruder is seen once the next tick's sight update ran.
    g.step();
    assert!(shroud::seen(
        &g.world,
        HouseId(0),
        g.world.obj(intruder).unwrap().pos.cell()
    ));
    // The house's own reactions aside, the brain sends its army: with
    // every army unit's own target dropped, it orders them at the intruder.
    let army = of(&g.world, 0, "GUNR");
    let mut w = g.world.clone();
    for &id in &army {
        w.obj_mut(id).unwrap().target = None;
    }
    let map = fixture::map();
    let orders = Brain::new(&w, &map, HouseId(0)).think(&w);
    for &id in &army {
        assert!(
            orders.iter().any(|o| matches!(o,
                Order::Behaviour { actor, target: Some(Target::Object(t)), .. }
                    if *t == intruder && *actor == id)),
            "army unit {id:?} is sent at the intruder: {orders:?}"
        );
    }
}

// The AI sees only its own house's shroud: two worlds that differ only by
// an enemy unit on a cell the house has not explored give the same orders.
#[test]
fn decides_only_on_what_it_sees() {
    let mut g = armed();
    let map = fixture::map();
    let hidden = (0..=2 * fixture::SIZE)
        .flat_map(|y| (0..=2 * fixture::SIZE).map(move |x| glasnost_map::CellXY::new(x, y)))
        .find(|&c| {
            g.world.cells.usable(c)
                && !shroud::explored(&g.world, HouseId(0), c)
                && g.world.objects.values().all(|o| o.pos.cell() != c)
        })
        .expect("a cell house 0 has not explored");
    let base = Brain::new(&g.world, &map, HouseId(0));
    let mut other = g.world.clone();
    testkit::spawn(&mut other, Kind::Vehicle, "GUNR", 1, hidden, 0);
    assert!(!shroud::explored(&other, HouseId(0), hidden));
    // Over a whole think period, from the same brain state.
    for _ in 0..glasnost_ai::scripted::THINK_PERIOD {
        let (mut a, mut b) = (base.clone(), base.clone());
        assert_eq!(
            format!("{:?}", a.think(&g.world)),
            format!("{:?}", b.think(&other)),
            "same orders at tick {}",
            g.world.tick
        );
        g.step();
        other = {
            let mut o = g.world.clone();
            testkit::spawn(&mut o, Kind::Vehicle, "GUNR", 1, hidden, 0);
            o
        };
    }
}

// A lost harvester is rebuilt.
#[test]
#[ignore = "long in debug builds; run in release (`cargo test --release -p glasnost-ai -- --include-ignored`)"]
fn rebuilds_lost_harvesters() {
    let mut g = start(5);
    assert!(step_until(&mut g, 9000, |w| !of(w, 0, "MINR").is_empty()));
    for id in of(&g.world, 0, "MINR") {
        objects::remove(&mut g.world, id);
    }
    let lost = g.world.tick;
    assert!(
        step_until(&mut g, lost + 4000, |w| !of(w, 0, "MINR").is_empty()),
        "the harvesters lost at tick {lost} were rebuilt"
    );
}

// A lost construction yard is rebuilt: the house builds a unit that
// deploys into one and deploys it.
#[test]
#[ignore = "long in debug builds; run in release (`cargo test --release -p glasnost-ai -- --include-ignored`)"]
fn rebuilds_a_lost_yard() {
    let mut g = start(5);
    assert!(step_until(&mut g, 9000, |w| !of(w, 0, "FACT").is_empty()));
    for id in of(&g.world, 0, "YARD") {
        objects::remove(&mut g.world, id);
    }
    let lost = g.world.tick;
    let mut built = false;
    assert!(
        step_until(&mut g, lost + 6000, |w| {
            built |= !of(w, 0, "MCV").is_empty();
            !of(w, 0, "YARD").is_empty()
        }),
        "the yard lost at tick {lost} was rebuilt (a deploying unit built: {built})"
    );
    assert!(built, "by a unit built for it");
}

// AI against AI is played to a winner, and every house went through the
// whole loop on the way: deploy, economy, production, attack.
#[test]
#[ignore = "long in debug builds; run in release (`cargo test --release -p glasnost-ai -- --include-ignored`)"]
fn plays_to_a_winner() {
    for seed in [5, 6] {
        let mut g = start(seed);
        let mut most: [BTreeMap<String, usize>; 2] = Default::default();
        let mut paid = [false; 2];
        let mut last = [i32::MAX; 2];
        while g.world.tick < LIMIT {
            g.step();
            for h in 0..2u8 {
                let w = &g.world;
                let mut now: BTreeMap<String, usize> = BTreeMap::new();
                for o in w
                    .objects
                    .values()
                    .filter(|o| o.owner == Some(HouseId(h)) && o.on_map)
                {
                    if let Some(n) = name(w, o.id) {
                        *now.entry(n).or_default() += 1;
                    }
                }
                for (k, v) in now {
                    let e = most[usize::from(h)].entry(k).or_default();
                    *e = (*e).max(v);
                }
                let c = w.houses[usize::from(h)].credits;
                paid[usize::from(h)] |= c > last[usize::from(h)];
                last[usize::from(h)] = c;
            }
            if g.outcome().is_some() {
                break;
            }
        }
        let out = g.outcome().expect("the game ended");
        let winners: Vec<_> = out
            .results
            .iter()
            .filter(|r| r.1 == victory::Verdict::Win)
            .collect();
        assert_eq!(winners.len(), 1, "seed {seed}: one winner: {out:?}");
        for h in 0..2 {
            for n in ["YARD", "POWR", "PROC", "PILL", "FACT", "MINR"] {
                assert!(
                    most[h].get(n).copied().unwrap_or(0) >= 1,
                    "seed {seed} house {h} had a {n}: {:?}",
                    most[h]
                );
            }
            let army = most[h].get("GUNR").copied().unwrap_or(0)
                + most[h].get("WALKER").copied().unwrap_or(0);
            assert!(
                army >= 3,
                "seed {seed} house {h} produced an army: {:?}",
                most[h]
            );
            assert!(paid[h], "seed {seed} house {h} was paid for ore");
        }
        // spec: S-0201 claim 1 (the final tick leaves the tick number at itself; a replay of an ended game stops after it)
        let replayed = Game::run_replay(
            g.replay(),
            fixture::rules(),
            &fixture::map(),
            testkit::catalogue(),
            |_| {},
        )
        .expect("replays");
        assert_eq!(
            replayed.state_hash(),
            g.state_hash(),
            "seed {seed}: the replay of the ended game reproduces its final state"
        );
    }
}

// A house without income and without the credits for anything it could
// build sells a sellable building for its refund.
#[test]
fn a_broke_house_sells_a_building() {
    // The power plant has a build-up sprite, so it is sellable (S-0408 claim 3).
    let mut r = fixture::ruleset();
    let p = r
        .sim
        .buildings
        .get_mut("POWR")
        .expect("fixture power plant");
    p.econ.buildup = Some(true);
    p.econ.buildup_frames = Some(8);
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-skirmish"));
    let new = || {
        Game::new(
            rules.clone(),
            &fixture::map(),
            "synthetic-skirmish",
            testkit::catalogue(),
            fixture::setup(5),
        )
        .expect("the synthetic skirmish loads")
    };
    let mut g = new();
    assert!(step_until(&mut g, 3000, |w| ["YARD", "POWR", "PROC"]
        .iter()
        .all(|n| !of(w, 0, n).is_empty())));
    // No income from now on: the harvesters go, and the credits too.
    for id in of(&g.world, 0, "MINR") {
        objects::remove(&mut g.world, id);
    }
    g.world.houses[0].credits = 0;
    let from = g.world.tick;
    let powr = of(&g.world, 0, "POWR").len();
    let sold = step_until(&mut g, from + 4000, |w| of(w, 0, "POWR").len() < powr);
    assert!(
        sold,
        "house 0 sold its power plant by tick {}",
        g.world.tick
    );
    // A house with income does not sell.
    let mut g = new();
    let sold = step_until(&mut g, 6000, |w| {
        w.objects.values().any(|o| o.econ.sale.is_some())
    });
    assert!(!sold, "no sale while the economy runs");
}
