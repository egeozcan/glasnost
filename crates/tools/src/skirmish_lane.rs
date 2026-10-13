//! The local lane's scripted skirmish (M5a): two scripted houses play a
//! game on stock maps of the install named by `GLASNOST_GAME_DIR`, through
//! orders only, to the last side (S-0414). The scripts pick their types by
//! role from the install's rules, so nothing here names a stock type. The
//! report holds counts and timings only (no names, no values, no paths) and
//! checks that the recorded replay reproduces the game and that a save in
//! mid-game, loaded and continued, ends in the same state. Never in CI.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use glasnost_ai::scripted::{Plan, Role, all_types, role_of};
use glasnost_ai::skirmish;
use glasnost_map::Map;
use glasnost_map::terrain::TileCatalogue;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::econ::setup::{self, Player, Settings};
use glasnost_sim::econ::tech;
use glasnost_sim::load::new_game;
use glasnost_sim::rules::SimRules;
use glasnost_sim::save;
use glasnost_sim::world::{HouseId, World};

/// Players of the lane's games.
const PLAYERS: usize = 2;

/// The lane's plan: types by role, two harvesters, waves of six (our own
/// opponent's tuning, as in the synthetic skirmish).
fn plan() -> Plan {
    Plan {
        names: Default::default(),
        harvesters: 2,
        wave: 6,
    }
}

/// One map's game.
#[derive(Debug, Default)]
struct Game {
    index: usize,
    load_ms: u128,
    cells: usize,
    run_ms: u128,
    ticks: i32,
    over: bool,
    winner: Option<u8>,
    orders: usize,
    /// Per player house: distinct objects of each role it ever had.
    built: [BTreeMap<Role, usize>; PLAYERS],
    /// The players' country registration indices.
    countries: Vec<usize>,
    /// Types of the rules by role (a role with none blocks the plan).
    types: BTreeMap<Role, usize>,
    /// Per player house: ticks in which its credits rose.
    paid: [u32; PLAYERS],
    /// Per player house: credits at the end.
    credits: [i32; PLAYERS],
    /// Per player house: types it could build at the end, by role.
    buildable: [BTreeMap<Role, usize>; PLAYERS],
    /// Shots and kills of the game (S-0301, S-0312).
    shots: usize,
    killed: usize,
    replay_ms: u128,
    replay_matches: bool,
    save_matches: bool,
    failure: Option<String>,
}

/// Runs the skirmish on `maps` stock maps (the first ones with exactly two
/// start positions, or the listed map indices) for at most `max_ticks`
/// ticks each; returns the report.
pub fn run(
    dir: &Path,
    maps: usize,
    only: &[usize],
    max_ticks: i32,
    seed: u32,
) -> std::io::Result<String> {
    pollster::block_on(run_async(dir, maps, only, max_ticks, seed))
}

async fn run_async(
    dir: &Path,
    maps: usize,
    only: &[usize],
    max_ticks: i32,
    seed: u32,
) -> std::io::Result<String> {
    let t0 = Instant::now();
    let mut vfs = crate::local_lane::mount(dir).await?;
    let mount_ms = t0.elapsed().as_millis();
    let listings = glasnost_convert::find::find_maps(&vfs).await;
    let mut games = Vec::new();
    let mut looked = 0usize;
    for (index, listing) in listings.iter().enumerate() {
        if games.len() >= maps && only.is_empty() {
            break;
        }
        if !only.is_empty() && !only.contains(&index) {
            continue;
        }
        let Ok(bytes) = vfs.read(&listing.file).await else {
            continue;
        };
        looked += 1;
        let t1 = Instant::now();
        let inputs = glasnost_convert::game::load(&mut vfs, bytes, "install".into()).await;
        let load_ms = t1.elapsed().as_millis();
        let (Ok(resolved), Some(tiles)) = (inputs.resolved, inputs.tiles) else {
            continue;
        };
        let map = inputs.map;
        if only.is_empty() && skirmish::start_cells(&map).len() != PLAYERS {
            continue;
        }
        let rules = Arc::new(SimRules::new(&resolved.ruleset, &resolved.sim_rules_hash));
        let mut g = Game {
            index,
            load_ms,
            cells: map.cells.len(),
            ..Default::default()
        };
        let result = catch_unwind(AssertUnwindSafe(|| {
            play(&mut g, &rules, &map, Arc::new(tiles), max_ticks, seed)
        }));
        match result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => g.failure = Some(e),
            Err(_) => g.failure = Some("panic".into()),
        }
        games.push(g);
    }
    Ok(report(mount_ms, looked, &games, max_ticks, seed))
}

/// The open countries: those that are not passive and not the neutral or
/// special house's, with their registration indices.
fn countries(rules: &SimRules) -> Vec<(u32, String)> {
    rules
        .country_names
        .iter()
        .zip(&rules.countries)
        .enumerate()
        .filter(|(_, (n, c))| {
            !c.passive
                && n.as_str() != glasnost_sim::load::NEUTRAL
                && n.as_str() != glasnost_sim::load::SPECIAL
        })
        .map(|(i, (n, _))| (i as u32, n.clone()))
        .collect()
}

/// The players' countries: the first open countries that own a harvester
/// type the scripts can use (one the simulation moves, with a tech level),
/// else the first open countries.
fn player_countries(w: &World) -> Vec<String> {
    let open = countries(w.rules());
    let harvests = |c: u32| {
        all_types(w).into_iter().any(|t| {
            role_of(w, t) == Some(Role::Harvester)
                && w.rules()
                    .econ
                    .tech
                    .of(t)
                    .is_some_and(|f| f.tech_level >= 0 && f.owners.contains(&c))
        })
    };
    let able: Vec<String> = open
        .iter()
        .filter(|(c, _)| harvests(*c))
        .map(|(_, n)| n.clone())
        .take(PLAYERS)
        .collect();
    if able.len() == PLAYERS {
        able
    } else {
        open.into_iter().map(|(_, n)| n).take(PLAYERS).collect()
    }
}

fn play(
    g: &mut Game,
    rules: &Arc<SimRules>,
    map: &Map,
    tiles: Arc<TileCatalogue>,
    max_ticks: i32,
    seed: u32,
) -> Result<(), String> {
    let players = |names: Vec<String>| -> Vec<Player> {
        names
            .into_iter()
            .enumerate()
            .map(|(i, country)| Player {
                name: format!("Player {i}"),
                country,
                start: None,
                ai: true,
                values: Default::default(),
            })
            .collect()
    };
    let first = players(
        countries(rules)
            .into_iter()
            .map(|(_, n)| n)
            .take(PLAYERS)
            .collect(),
    );
    if first.len() < PLAYERS {
        return Err("too few countries".into());
    }
    let probe = new_game(
        rules.clone(),
        map,
        tiles.clone(),
        &setup::skirmish(rules, &first, Settings::default(), 0, seed),
    )
    .map_err(|e| format!("{e:?}"))?;
    for t in all_types(&probe) {
        if let Some(r) = role_of(&probe, t) {
            *g.types.entry(r).or_default() += 1;
        }
    }
    let chosen = player_countries(&probe);
    g.countries = chosen
        .iter()
        .filter_map(|n| rules.country_names.iter().position(|x| x == n))
        .collect();
    drop(probe);
    let setup = setup::skirmish(rules, &players(chosen), Settings::default(), 0, seed);
    let t = Instant::now();
    let run = skirmish::run(
        rules.clone(),
        map,
        "install",
        tiles.clone(),
        setup,
        &plan(),
        max_ticks,
    )
    .map_err(|e| format!("{e:?}"))?;
    g.run_ms = t.elapsed().as_millis();
    g.ticks = run.replay.ticks;
    g.over = run.over;
    g.winner = run.winner.map(|h| h.0);
    g.orders = run.replay.orders.len();

    // The replay without the scripts, watching what each house had.
    let mut seen: [BTreeMap<Role, BTreeSet<u32>>; PLAYERS] = Default::default();
    let mut last = [i32::MAX; PLAYERS];
    let paid = &mut g.paid;
    let (mut shots, mut killed, mut drained) = (0usize, 0usize, 0usize);
    let t = Instant::now();
    let world = run
        .replay
        .run(rules.clone(), map, tiles.clone(), |w| {
            // The effects pile up in a replay: count the new ones.
            for e in w.effects.iter().skip(drained) {
                match e {
                    Effect::Shot { .. } => shots += 1,
                    Effect::Killed { .. } => killed += 1,
                    _ => {}
                }
            }
            drained = w.effects.len();
            for h in 0..PLAYERS {
                watch(w, h, &mut seen[h]);
                let c = w.houses[h].credits;
                if c > last[h] {
                    paid[h] += 1;
                }
                last[h] = c;
            }
        })
        .map_err(|e| format!("replay: {e:?}"))?;
    g.replay_ms = t.elapsed().as_millis();
    (g.shots, g.killed) = (shots, killed);
    for h in 0..PLAYERS {
        g.credits[h] = world.houses[h].credits;
        for t in all_types(&world) {
            if let Some(r) = role_of(&world, t)
                && tech::buildable(&world, HouseId(h as u8), t)
            {
                *g.buildable[h].entry(r).or_default() += 1;
            }
        }
    }
    for (built, seen) in g.built.iter_mut().zip(&seen) {
        *built = seen.iter().map(|(r, s)| (*r, s.len())).collect();
    }
    g.replay_matches = save::state_hash(&world) == save::state_hash(&run.world);

    // A save in mid-game, loaded and continued.
    let rep = &run.replay;
    let mut w =
        new_game(rules.clone(), map, tiles.clone(), &rep.setup).map_err(|e| format!("{e:?}"))?;
    rep.run_on(&mut w, rep.ticks / 2, |_| {});
    let mut loaded =
        save::load(&save::save(&w), rules.clone(), tiles).map_err(|e| format!("load: {e:?}"))?;
    let same_at_save = save::state_hash(&loaded) == save::state_hash(&w);
    rep.run_on(&mut loaded, rep.ticks, |_| {});
    g.save_matches = same_at_save && save::state_hash(&loaded) == save::state_hash(&run.world);
    Ok(())
}

/// Notes the house's objects on the map by role.
fn watch(w: &World, h: usize, seen: &mut BTreeMap<Role, BTreeSet<u32>>) {
    for (id, o) in &w.objects {
        if o.owner != Some(HouseId(h as u8)) || !o.on_map || o.health <= 0 {
            continue;
        }
        if let Some(r) = role_of(w, tech::type_of(o)) {
            seen.entry(r).or_default().insert(id.0);
        }
    }
}

fn role_name(r: Role) -> String {
    match r {
        Role::Factory(k) => format!("factory({k:?})").to_lowercase(),
        r => format!("{r:?}").to_lowercase(),
    }
}

fn report(mount_ms: u128, looked: usize, games: &[Game], max_ticks: i32, seed: u32) -> String {
    let mut s = String::new();
    let _ = writeln!(
        s,
        "glasnost-headless skirmish: {} games of {PLAYERS} scripted houses, at most {max_ticks} ticks, seed {seed}; install mounted in {mount_ms} ms; {looked} maps read",
        games.len()
    );
    for g in games {
        let _ = writeln!(
            s,
            "map index {}: {} cells, inputs {} ms",
            g.index, g.cells, g.load_ms
        );
        let types: Vec<String> = g
            .types
            .iter()
            .map(|(r, n)| format!("{} {n}", role_name(*r)))
            .collect();
        let _ = writeln!(
            s,
            "  types by role: {}; player countries (registration index): {:?}",
            types.join(", "),
            g.countries
        );
        if !g.types.contains_key(&Role::Yard)
            || !g.types.keys().any(|r| matches!(r, Role::Factory(_)))
        {
            let _ = writeln!(
                s,
                "  no construction yard or factory types: the scripts cannot build a base"
            );
        }
        if let Some(e) = &g.failure {
            let _ = writeln!(s, "  failed: {e}");
            continue;
        }
        let per_tick = g.run_ms * 1000 / (g.ticks.max(1) as u128);
        let _ = writeln!(
            s,
            "  {} after {} ticks ({}), {} orders; run {} ms ({per_tick} µs per tick), replay {} ms",
            if g.over { "over" } else { "not over" },
            g.ticks,
            g.winner
                .map_or("no winner".to_string(), |h| format!("house {h} won")),
            g.orders,
            g.run_ms,
            g.replay_ms
        );
        for h in 0..PLAYERS {
            let roles: Vec<String> = g.built[h]
                .iter()
                .map(|(r, n)| format!("{} {n}", role_name(*r)))
                .collect();
            let roles = if roles.is_empty() {
                "nothing by role".to_string()
            } else {
                roles.join(", ")
            };
            let can: Vec<String> = g.buildable[h]
                .iter()
                .map(|(r, n)| format!("{} {n}", role_name(*r)))
                .collect();
            let _ = writeln!(
                s,
                "  house {h}: {roles}; credits rose in {} ticks, {} at the end; could build at the end: {}",
                g.paid[h],
                g.credits[h],
                can.join(", ")
            );
        }
        let _ = writeln!(s, "  combat: {} shots, {} killed", g.shots, g.killed);
        let _ = writeln!(
            s,
            "  replay reproduces the final state: {}; save in mid-game, loaded and continued, matches: {}",
            g.replay_matches, g.save_matches
        );
    }
    s
}
