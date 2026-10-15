//! The local lane's scripted skirmish (M5a): two scripted houses play a
//! game on stock maps of the install named by `GLASNOST_GAME_DIR`, through
//! orders only, to the last side (S-0414). The scripts pick their types by
//! role from the install's rules, so nothing here names a stock type. The
//! report holds counts and timings only (no names, no values, no paths) and
//! checks that the recorded replay reproduces the game and that a save in
//! mid-game, loaded and continued, ends in the same state. Never in CI.
//!
//! With [`Players::Ai`] (M5b) the houses are played by our own AI through
//! `glasnost_ai::game::Game` instead: the replay holds no orders then (no
//! person plays), and replaying it runs the AI again.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use glasnost_ai::game::Game as AiGame;
use glasnost_ai::scripted::{Plan, Role, all_types, role_of};
use glasnost_ai::skirmish;
use glasnost_map::Map;
use glasnost_map::terrain::TileCatalogue;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::econ::setup::{self, Player, Settings};
use glasnost_sim::econ::tech;
use glasnost_sim::load::new_game;
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::save;
use glasnost_sim::world::{HouseId, World};

/// Players of the lane's games.
const PLAYERS: usize = 2;

/// The tick of the early offer in the report.
const EARLY: i32 = 3000;

/// Who plays the lane's houses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Players {
    /// M5a's script, every `THINK_PERIOD` ticks, its orders recorded.
    Scripted,
    /// The own AI of M5b (`glasnost_ai::game::Game`).
    Ai,
    /// The own AI, one game per pair of sides: each player the first open
    /// country of its side, so that every side plays.
    AiSides,
}

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
    /// Per player house: the first tick it had an object of each role.
    first: [BTreeMap<Role, i32>; PLAYERS],
    /// The players' country registration indices.
    countries: Vec<usize>,
    /// The players' side indices, for [`Players::AiSides`].
    sides: Option<(i32, i32)>,
    /// Types of the rules by role (a role with none blocks the plan).
    types: BTreeMap<Role, usize>,
    /// Per player house: ticks in which its credits rose.
    paid: [u32; PLAYERS],
    /// Per player house: credits at the end.
    credits: [i32; PLAYERS],
    /// Per player house: types it could build at the end, by role.
    buildable: [BTreeMap<Role, usize>; PLAYERS],
    /// Per player house: its army units at the end by current behaviour,
    /// destination kind and target kind.
    army_end: [BTreeMap<String, usize>; PLAYERS],
    /// Per player house: types it could build at tick [`EARLY`], by role.
    early: [BTreeMap<Role, usize>; PLAYERS],
    /// Per player house: credits gained over the game.
    income: [i64; PLAYERS],
    /// Per player house: harvester ticks by current behaviour and harvest
    /// phase.
    harvest_ticks: [BTreeMap<String, u32>; PLAYERS],
    /// Shots and kills of the game (S-0301, S-0312).
    shots: usize,
    killed: usize,
    /// The distinct objects that showed each M6 feature (see [`features`]).
    features: BTreeMap<&'static str, BTreeSet<u32>>,
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
    players: Players,
) -> std::io::Result<String> {
    pollster::block_on(run_async(dir, maps, only, max_ticks, seed, players))
}

async fn run_async(
    dir: &Path,
    maps: usize,
    only: &[usize],
    max_ticks: i32,
    seed: u32,
    players: Players,
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
        let tiles = Arc::new(tiles);
        let pairs: Vec<Option<(i32, i32)>> = if players == Players::AiSides {
            side_pairs(&rules).into_iter().map(Some).collect()
        } else {
            vec![None]
        };
        for pair in pairs {
            let mut g = Game {
                index,
                load_ms,
                cells: map.cells.len(),
                sides: pair,
                ..Default::default()
            };
            let result = catch_unwind(AssertUnwindSafe(|| {
                play(
                    &mut g,
                    &rules,
                    &map,
                    tiles.clone(),
                    max_ticks,
                    seed,
                    players,
                )
            }));
            match result {
                Ok(Ok(())) => {}
                Ok(Err(e)) => g.failure = Some(e),
                Err(_) => g.failure = Some("panic".into()),
            }
            games.push(g);
        }
    }
    Ok(report(mount_ms, looked, &games, max_ticks, seed, players))
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

/// The first open country of each side, by side index.
fn side_countries(rules: &SimRules) -> BTreeMap<i32, String> {
    let mut out = BTreeMap::new();
    for (i, n) in countries(rules) {
        if let Some(Some(side)) = rules.econ.tech.country_side.get(i as usize) {
            out.entry(*side).or_insert(n);
        }
    }
    out
}

/// Every pair of sides that have an open country.
fn side_pairs(rules: &SimRules) -> Vec<(i32, i32)> {
    let sides: Vec<i32> = side_countries(rules).into_keys().collect();
    let mut out = Vec::new();
    for (k, &a) in sides.iter().enumerate() {
        for &b in &sides[k + 1..] {
            out.push((a, b));
        }
    }
    out
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
    who: Players,
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
    let chosen = match g.sides {
        Some((a, b)) => {
            let by_side = side_countries(rules);
            vec![by_side[&a].clone(), by_side[&b].clone()]
        }
        None => player_countries(&probe),
    };
    g.countries = chosen
        .iter()
        .filter_map(|n| rules.country_names.iter().position(|x| x == n))
        .collect();
    drop(probe);
    let setup = setup::skirmish(rules, &players(chosen), Settings::default(), 0, seed);
    if who != Players::Scripted {
        return play_ai(g, rules, map, tiles, max_ticks, setup);
    }
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
    let income = &mut g.income;
    let harvest_ticks = &mut g.harvest_ticks;
    let feats = &mut g.features;
    let (mut shots, mut killed, mut drained) = (0usize, 0usize, 0usize);
    let first = &mut g.first;
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
            features(w, feats);
            for h in 0..PLAYERS {
                watch(w, h, &mut seen[h], &mut first[h]);
                let c = w.houses[h].credits;
                if c > last[h] {
                    paid[h] += 1;
                    income[h] += i64::from(c - last[h]);
                }
                for o in w
                    .objects
                    .values()
                    .filter(|o| o.on_map && o.health > 0 && o.owner == Some(HouseId(h as u8)))
                {
                    if let Some(hs) = &o.econ.harvest {
                        let key = format!("{:?}/{:?}", o.beh.current, hs.phase);
                        *harvest_ticks[h].entry(key).or_default() += 1;
                    } else if let Some(ss) = &o.econ.slave {
                        // A slave on the map (S-0402 claim 14), by its own phase.
                        let key = format!("slave {:?}/{:?}", o.beh.current, ss.phase);
                        *harvest_ticks[h].entry(key).or_default() += 1;
                    }
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

/// The AI's game (M5b): the own AI plays both houses through
/// `glasnost_ai::game::Game`; the replay (no person's orders) runs the AI
/// again, and a save in mid-game, loaded and continued, must end alike.
fn play_ai(
    g: &mut Game,
    rules: &Arc<SimRules>,
    map: &Map,
    tiles: Arc<TileCatalogue>,
    max_ticks: i32,
    setup: glasnost_sim::world::GameSetup,
) -> Result<(), String> {
    let mut seen: [BTreeMap<Role, BTreeSet<u32>>; PLAYERS] = Default::default();
    let mut last = [i32::MAX; PLAYERS];
    let mut drained = 0usize;
    let t = Instant::now();
    let mut game = AiGame::new(rules.clone(), map, "install", tiles.clone(), setup.clone())
        .map_err(|e| format!("{e:?}"))?;
    while game.world.tick < max_ticks {
        game.step();
        let w = &game.world;
        // The effects pile up when nobody drains them: count the new ones.
        for e in w.effects.iter().skip(drained) {
            match e {
                Effect::Shot { .. } => g.shots += 1,
                Effect::Killed { .. } => g.killed += 1,
                _ => {}
            }
        }
        drained = w.effects.len();
        features(w, &mut g.features);
        if w.tick == EARLY {
            for h in 0..PLAYERS {
                g.early[h] = offer(w, h);
            }
        }
        for h in 0..PLAYERS {
            watch(w, h, &mut seen[h], &mut g.first[h]);
            let c = w.houses[h].credits;
            if c > last[h] {
                g.paid[h] += 1;
                g.income[h] += i64::from(c - last[h]);
            }
            for o in w
                .objects
                .values()
                .filter(|o| o.on_map && o.health > 0 && o.owner == Some(HouseId(h as u8)))
            {
                if let Some(hs) = &o.econ.harvest {
                    let key = format!("{:?}/{:?}", o.beh.current, hs.phase);
                    *g.harvest_ticks[h].entry(key).or_default() += 1;
                } else if let Some(ss) = &o.econ.slave {
                    // A slave on the map (S-0402 claim 14), by its own phase.
                    let key = format!("slave {:?}/{:?}", o.beh.current, ss.phase);
                    *g.harvest_ticks[h].entry(key).or_default() += 1;
                }
            }
            last[h] = c;
        }
        if let Some(o) = game.outcome() {
            g.over = true;
            g.winner = o
                .results
                .iter()
                .find(|r| r.1 == glasnost_sim::econ::victory::Verdict::Win)
                .map(|r| r.0.0);
            break;
        }
    }
    g.run_ms = t.elapsed().as_millis();
    g.ticks = glasnost_sim::tick::ticks_run(&game.world);
    g.orders = game.replay().orders.len();
    let world = &game.world;
    for h in 0..PLAYERS {
        g.credits[h] = world.houses[h].credits;
        for t in all_types(world) {
            if let Some(r) = role_of(world, t)
                && tech::buildable(world, HouseId(h as u8), t)
            {
                *g.buildable[h].entry(r).or_default() += 1;
            }
        }
    }
    for (built, seen) in g.built.iter_mut().zip(&seen) {
        *built = seen.iter().map(|(r, s)| (*r, s.len())).collect();
    }
    for h in 0..PLAYERS {
        for o in world
            .objects
            .values()
            .filter(|o| o.on_map && o.health > 0 && o.owner == Some(HouseId(h as u8)))
        {
            if role_of(world, tech::type_of(o)) == Some(Role::Army) {
                let kind = |t: &Option<glasnost_sim::world::Target>| match t {
                    None => "none",
                    Some(glasnost_sim::world::Target::Cell(_)) => "cell",
                    Some(_) => "object",
                };
                let key = format!(
                    "{:?} dest {} target {}",
                    o.beh.current,
                    kind(&o.dest),
                    kind(&o.target)
                );
                *g.army_end[h].entry(key).or_default() += 1;
            }
        }
    }
    let end = game.state_hash();

    // The replay runs the AI again.
    let t = Instant::now();
    let replayed = AiGame::run_replay(game.replay(), rules.clone(), map, tiles.clone(), |_| {})
        .map_err(|e| format!("replay: {e:?}"))?;
    g.replay_ms = t.elapsed().as_millis();
    g.replay_matches = replayed.state_hash() == end;

    // A save in mid-game, loaded and continued.
    let mut half = AiGame::new(rules.clone(), map, "install", tiles.clone(), setup)
        .map_err(|e| format!("{e:?}"))?;
    while half.world.tick < g.ticks / 2 {
        half.step();
    }
    let mut loaded =
        AiGame::load(&half.save(), rules.clone(), tiles).map_err(|e| format!("load: {e:?}"))?;
    let same_at_save = loaded.state_hash() == half.state_hash();
    // The final tick leaves the tick number at itself (S-0201 claim 1): count the ticks run.
    while glasnost_sim::tick::ticks_run(&loaded.world) < g.ticks {
        loaded.step();
    }
    g.save_matches = same_at_save && loaded.state_hash() == end;
    Ok(())
}

/// Notes the objects that show an M6 feature now: a spawner, a rocket in
/// flight (S-0506), a parasite inside its host (S-0614), an aircraft on the
/// map that is not a rocket (S-0501), a naval yard, a ship, a sinking ship
/// (S-0504), and a building the script builds for no role (the tech climb).
fn features(w: &World, out: &mut BTreeMap<&'static str, BTreeSet<u32>>) {
    let r = w.rules();
    for (id, o) in &w.objects {
        let mut note = |k: &'static str| {
            out.entry(k).or_default().insert(id.0);
        };
        if o.spawn.is_some() {
            note("spawners");
        }
        if o.rocket.as_ref().is_some_and(|x| x.phase.is_some()) {
            note("rocket flights");
        }
        if glasnost_sim::parasite::host_of(w, *id).is_some() {
            note("parasites latched");
        }
        if !o.on_map || o.owner.is_none() {
            continue;
        }
        match o.kind {
            Kind::Aircraft if o.rocket.is_none() => note("aircraft"),
            Kind::Building => {
                let b = r.buildings.get(o.type_index as usize);
                if b.is_some_and(|b| b.naval && b.factory.is_some()) {
                    note("naval yards");
                }
                if role_of(w, tech::type_of(o)).is_none() {
                    note("climb buildings");
                }
            }
            Kind::Vehicle => {
                if r.unit(o.kind, o.type_index)
                    .is_some_and(|u| u.locomotor == glasnost_sim::rules::Locomotor::Ship)
                {
                    note("ships");
                }
                if glasnost_sim::naval::sinking(w, *id) {
                    note("ships sinking");
                }
            }
            _ => {}
        }
    }
}

/// The types house `h` can build now, by role.
fn offer(w: &World, h: usize) -> BTreeMap<Role, usize> {
    let mut out = BTreeMap::new();
    for t in all_types(w) {
        if let Some(r) = role_of(w, t)
            && tech::buildable(w, HouseId(h as u8), t)
        {
            *out.entry(r).or_default() += 1;
        }
    }
    out
}

/// Notes the house's objects on the map by role, and the first tick of
/// each role.
fn watch(
    w: &World,
    h: usize,
    seen: &mut BTreeMap<Role, BTreeSet<u32>>,
    first: &mut BTreeMap<Role, i32>,
) {
    for (id, o) in &w.objects {
        if o.owner != Some(HouseId(h as u8)) || !o.on_map || o.health <= 0 {
            continue;
        }
        if let Some(r) = role_of(w, tech::type_of(o)) {
            seen.entry(r).or_default().insert(id.0);
            first.entry(r).or_insert(w.tick);
        }
    }
}

fn role_name(r: Role) -> String {
    match r {
        Role::Factory(k) => format!("factory({k:?})").to_lowercase(),
        r => format!("{r:?}").to_lowercase(),
    }
}

fn report(
    mount_ms: u128,
    looked: usize,
    games: &[Game],
    max_ticks: i32,
    seed: u32,
    players: Players,
) -> String {
    let mut s = String::new();
    let (cmd, who) = match players {
        Players::Scripted => ("skirmish", "scripted houses"),
        Players::Ai => ("ai", "AI houses"),
        Players::AiSides => ("ai-sides", "AI houses, one game per pair of sides"),
    };
    let _ = writeln!(
        s,
        "glasnost-headless {cmd}: {} games of {PLAYERS} {who}, at most {max_ticks} ticks, seed {seed}; install mounted in {mount_ms} ms; {looked} maps read",
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
            "  types by role: {}; player countries (registration index): {:?}{}",
            types.join(", "),
            g.countries,
            g.sides
                .map_or(String::new(), |(a, b)| format!("; sides {a} and {b}"))
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
            let first: Vec<String> = g.first[h]
                .iter()
                .map(|(r, t)| format!("{} {t}", role_name(*r)))
                .collect();
            let _ = writeln!(s, "  house {h} first tick by role: {}", first.join(", "));
            if !g.early[h].is_empty() || g.first[h].len() < 3 {
                let early: Vec<String> = g.early[h]
                    .iter()
                    .map(|(r, n)| format!("{} {n}", role_name(*r)))
                    .collect();
                let _ = writeln!(
                    s,
                    "  house {h} could build at tick {EARLY}: {}",
                    early.join(", ")
                );
            }
        }
        for h in 0..PLAYERS {
            let mut ticks: Vec<_> = g.harvest_ticks[h].iter().collect();
            ticks.sort_by_key(|(_, n)| std::cmp::Reverse(**n));
            let ticks: Vec<String> = ticks.iter().map(|(k, n)| format!("{k} {n}")).collect();
            let _ = writeln!(
                s,
                "  house {h} income {}; harvester ticks by behaviour/phase: {}",
                g.income[h],
                ticks.join(", ")
            );
        }
        let _ = writeln!(s, "  combat: {} shots, {} killed", g.shots, g.killed);
        let f: Vec<String> = g
            .features
            .iter()
            .map(|(k, v)| format!("{k} {}", v.len()))
            .collect();
        let _ = writeln!(s, "  M6 features (distinct objects): {}", f.join(", "));
        for h in 0..PLAYERS {
            if !g.army_end[h].is_empty() {
                let v: Vec<String> = g.army_end[h]
                    .iter()
                    .map(|(k, n)| format!("{k} ×{n}"))
                    .collect();
                let _ = writeln!(s, "  house {h} army at the end: {}", v.join(", "));
            }
        }
        let _ = writeln!(
            s,
            "  replay reproduces the final state: {}; save in mid-game, loaded and continued, matches: {}",
            g.replay_matches, g.save_matches
        );
    }
    s
}
