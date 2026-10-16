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
        fixture::catalogue(),
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
            fixture::catalogue(),
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
        fixture::catalogue(),
        &rep.setup,
    )
    .expect("loads");
    rep.run_on(&mut w, rep.ticks / 2, |_| {});
    let bytes = save::save(&w);
    let mut loaded =
        save::load(&bytes, fixture::rules(), fixture::catalogue()).expect("the save loads");
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
            fixture::catalogue(),
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

// On a map with water the houses build a naval yard on the sea, then
// ships, and send the ships to the water off the enemy base; ships fight
// and sink. The whole game's course shifts with every spec revision (a house
// can lose its construction yard before its naval yard stands), so a few
// seeds are tried and one game must show all of it.
#[test]
#[ignore = "long in debug builds; run in release (`cargo test --release -p glasnost-ai -- --include-ignored`)"]
fn builds_a_navy_on_a_water_map() {
    let mut failures = Vec::new();
    for seed in [SEED, 6, 7, 8] {
        match navy_game(seed) {
            Ok(()) => return,
            Err(e) => failures.push((seed, e)),
        }
    }
    panic!("no seed showed a navy: {failures:?}");
}

/// One seed's naval game, checked; the error names the first claim that fails.
fn navy_game(seed: u32) -> Result<(), String> {
    let map = fixture::map_naval();
    let run = skirmish::run(
        fixture::rules(),
        &map,
        "synthetic-naval",
        fixture::catalogue(),
        fixture::setup(seed),
        &fixture::plan(),
        LIMIT,
    )
    .expect("the naval skirmish loads");
    let mut most: [BTreeMap<String, usize>; 2] = Default::default();
    let mut yard_on_water = true;
    let mut furthest = [0i32; 2];
    let mut ship_fired = false;
    let mut sunk = 0usize;
    let mut sinking_seen = std::collections::BTreeSet::new();
    run.replay
        .run(fixture::rules(), &map, fixture::catalogue(), |w| {
            for h in 0..2u8 {
                for (k, v) in counts(w, h) {
                    let e = most[usize::from(h)].entry(k).or_default();
                    *e = (*e).max(v);
                }
            }
            let r = w.rules();
            for o in w.objects.values().filter(|o| o.on_map) {
                let t = tech::type_of(o);
                let name = match t.kind {
                    Kind::Building => r.buildings.get(t.index as usize).map(|b| b.name.as_str()),
                    Kind::Vehicle => r.vehicles.get(t.index as usize).map(|u| u.name.as_str()),
                    _ => None,
                };
                let Some(h) = o.owner else { continue };
                match name {
                    Some("NAVY") => {
                        let c = o.pos.cell();
                        let (fw, fh) = o.foundation;
                        for dx in 0..fw {
                            for dy in 0..fh {
                                let k = glasnost_map::CellXY::new(c.x + dx, c.y + dy);
                                yard_on_water &= map
                                    .cells
                                    .get(k)
                                    .is_some_and(|x| x.tile == fixture::WATER_TILE);
                            }
                        }
                    }
                    Some("BOAT") => {
                        let i = usize::from(h.0);
                        // How far east (house 0) or west (house 1) of its start the ship got.
                        let x = o.pos.cell().x - fixture::at(0, 0).x;
                        let gone = if i == 0 { x } else { -x };
                        furthest[i] = furthest[i].max(gone);
                        ship_fired |= o.target.is_some();
                        if glasnost_sim::naval::sinking(w, o.id) && sinking_seen.insert(o.id) {
                            sunk += 1;
                        }
                    }
                    _ => {}
                }
            }
        })
        .expect("the replay runs");
    if !(yard_on_water) {
        return Err(format!("every naval yard stands on water"));
    }
    // Both houses used to build a naval yard and ships before the fighting reached
    // either base; since the M6b2 revisions (S-0207 claim 26 and the others) one house can
    // lose its construction yard first, so one navy is required here (see docs/development.md).
    for name in ["NAVY", "BOAT"] {
        if !most.iter().any(|m| m.get(name).copied().unwrap_or(0) > 0) {
            return Err(format!("a house built {name}: {most:?}"));
        }
    }
    // S-0503: the hovercraft is army of the land vehicle factory once a
    // naval yard stands.
    if !(most.iter().any(|m| m.get("HOVR").copied().unwrap_or(0) > 0)) {
        return Err(format!("a house built a hovercraft: {most:?}"));
    }
    if !(furthest.iter().any(|&d| d > 0)) {
        return Err(format!("a ship crossed towards the enemy: {furthest:?}"));
    }
    if !(ship_fired) {
        return Err(format!("a ship took a target"));
    }
    // spec: S-0504 claim 4 (a killed ship on water sinks)
    if !(sunk > 0) {
        return Err(format!("a ship sank"));
    }
    Ok(())
}

// The houses climb to an aircraft pad, build planes on it, send idle
// planes with ammunition against seen enemies, and the planes fly back to
// their pads and rearm (S-0501 claims 21 to 30).
#[test]
#[ignore = "long in debug builds; run in release (`cargo test --release -p glasnost-ai -- --include-ignored`)"]
fn flies_aircraft_from_pads() {
    let run = run_for(LIMIT);
    let mut most: [BTreeMap<String, usize>; 2] = Default::default();
    // Per plane: seen in the air with a target, spent a round, back on the
    // ground after that, rearmed there.
    #[derive(Default)]
    struct Seen {
        attacked: bool,
        spent: bool,
        landed: bool,
        rearmed: bool,
    }
    let mut planes: BTreeMap<u32, Seen> = BTreeMap::new();
    run.replay
        .run(
            fixture::rules(),
            &fixture::map(),
            fixture::catalogue(),
            |w| {
                for h in 0..2u8 {
                    for (k, v) in counts(w, h) {
                        let e = most[usize::from(h)].entry(k).or_default();
                        *e = (*e).max(v);
                    }
                }
                let jet = w.rules().index_of(Kind::Aircraft, "JET");
                for o in w
                    .objects
                    .values()
                    .filter(|o| o.on_map && o.kind == Kind::Aircraft && Some(o.type_index) == jet)
                {
                    let s = planes.entry(o.id.0).or_default();
                    let up = glasnost_sim::air::in_the_air(w, o.id);
                    if up && o.target.is_some() {
                        s.attacked = true;
                    }
                    if o.combat.ammo < 2 {
                        s.spent = true;
                    }
                    if s.spent && !up && glasnost_sim::air::on_ground(w, o.id) {
                        s.landed = true;
                        if o.combat.ammo == 2 {
                            s.rearmed = true;
                        }
                    }
                }
            },
        )
        .expect("the replay runs");
    let pads: usize = most
        .iter()
        .map(|m| m.get("HPAD").copied().unwrap_or(0))
        .sum();
    assert!(pads > 0, "a house built a pad: {most:?}");
    assert!(!planes.is_empty(), "a house built planes: {most:?}");
    assert!(planes.values().any(|s| s.attacked), "a plane attacked");
    assert!(planes.values().any(|s| s.spent), "a plane fired");
    assert!(planes.values().any(|s| s.landed), "a plane came back down");
    assert!(planes.values().any(|s| s.rearmed), "a plane rearmed");
}
