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
use glasnost_sim::rules::{Kind, Locomotor, SimRules};
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
    /// Cells of land type water once the game is set up (a navy needs them).
    water: usize,
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
    /// Passengers that boarded a transport and that were let out beside one
    /// still standing (S-0507), and superweapon launches (S-0513), by the
    /// own AI's games.
    loads: usize,
    /// The loads onto a transport with the hovering locomotor.
    hover_loads: usize,
    unloads: usize,
    launches: usize,
    /// Per player house: the special-ability orders the own AI gave (M6c:
    /// garrisons, capture, infiltration), and the garrison occupants inside
    /// buildings at the end.
    special: [glasnost_ai::special::Uses; PLAYERS],
    occupants_inside: usize,
    /// Per player house: the occasions for charges and disarms (S-0518,
    /// S-0519) a person in its seat would have had (M7a step 2c).
    occasions: [Occasions; PLAYERS],
    /// The launches by superweapon kind (the kind names of the simulation).
    launches_by_kind: BTreeMap<String, usize>,
    /// House-ticks a superweapon record stood ready, by kind name.
    ready_ticks: BTreeMap<String, usize>,
    /// Units that went berserk (S-0610 claim 4), and house-ticks in which a
    /// player house's psychic sensors showed a hostile-intent line
    /// (claim 10).
    berserked: usize,
    intent_ticks: usize,
    /// Per player house: the most psychic sensors it had at one tick, and
    /// its distinct units with a berserk weapon (S-0610).
    sensors_max: [usize; PLAYERS],
    berserk_armed: [usize; PLAYERS],
    /// The distinct objects that showed each M6 feature (see [`features`]).
    features: BTreeMap<&'static str, BTreeSet<u32>>,
    /// Per player house: the unit types (infantry, vehicles, aircraft) it
    /// could build at some sampled tick and the ones it had a new object
    /// of (not a starting unit), by [`unit_class`] (PLAN 4.7: the own AI
    /// builds every unit class its side can build).
    offered: [BTreeMap<(&'static str, u8, u32), (i32, bool, Option<Role>)>; PLAYERS],
    made: [BTreeSet<(&'static str, u8, u32)>; PLAYERS],
    /// Counts-only state of a game that is not over at the end (the own
    /// AI's games): per house its objects by kind, the resource left on the
    /// map and the brain's own diagnostic.
    diagnosis: Vec<String>,
    replay_ms: u128,
    replay_matches: bool,
    save_matches: bool,
    /// `GLASNOST_AI_QUICK` skipped the replay and save checks.
    checks_skipped: bool,
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
            // `GLASNOST_AI_SIDES=a,b` plays only that pair of sides; `a,a`
            // plays the first two open countries of side a.
            let want: Option<(i32, i32)> = std::env::var("GLASNOST_AI_SIDES").ok().and_then(|v| {
                let (a, b) = v.split_once(',')?;
                Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
            });
            match want {
                Some((a, b)) if a == b => vec![Some((a, a))],
                _ => side_pairs(&rules)
                    .into_iter()
                    .filter(|p| want.is_none_or(|w| w == *p))
                    .map(Some)
                    .collect(),
            }
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
pub(crate) fn countries(rules: &SimRules) -> Vec<(u32, String)> {
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
    g.water = probe
        .cells
        .grid
        .iter()
        .filter(|(_, c)| c.land == glasnost_map::terrain::land::WATER)
        .count();
    for t in all_types(&probe) {
        if let Some(r) = role_of(&probe, t) {
            *g.types.entry(r).or_default() += 1;
        }
    }
    let chosen = match g.sides {
        Some((a, b)) if a == b => countries(rules)
            .into_iter()
            .filter(|(i, _)| rules.econ.tech.country_side.get(*i as usize) == Some(&Some(a)))
            .map(|(_, n)| n)
            .take(PLAYERS)
            .collect(),
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
    let mut tally = Tally::default();
    // `GLASNOST_AI_TRACE=n`: a timeline line per house every n ticks.
    let trace: i32 = std::env::var("GLASNOST_AI_TRACE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    // `GLASNOST_AI_DUMP=t1,t2,...`: every object of the player houses at those ticks.
    let dump_at: Vec<i32> = std::env::var("GLASNOST_AI_DUMP")
        .map(|v| v.split(',').filter_map(|x| x.trim().parse().ok()).collect())
        .unwrap_or_default();
    // `GLASNOST_AI_WATCH=id,from,to`: one object's state every tick in a range.
    let watch_obj: Option<(u32, i32, i32)> =
        std::env::var("GLASNOST_AI_WATCH").ok().and_then(|v| {
            let p: Vec<&str> = v.split(',').collect();
            Some((
                p.first()?.parse().ok()?,
                p.get(1)?.parse().ok()?,
                p.get(2)?.parse().ok()?,
            ))
        });
    // `GLASNOST_AI_LINES=h`: every item house h starts or loses on its
    // production lines (tick, line kind, unit class or building, cost,
    // credits), and every 300 ticks a still item's progress.
    let lines_of: Option<u8> = std::env::var("GLASNOST_AI_LINES")
        .ok()
        .and_then(|v| v.trim().parse().ok());
    let mut line_items: BTreeMap<String, Option<(glasnost_sim::econ::TypeRef, i32)>> =
        BTreeMap::new();
    let t = Instant::now();
    let mut game = AiGame::new(rules.clone(), map, "install", tiles.clone(), setup.clone())
        .map_err(|e| format!("{e:?}"))?;
    let starting: BTreeSet<u32> = game.world.objects.keys().map(|id| id.0).collect();
    while game.world.tick < max_ticks {
        game.step();
        let w = &game.world;
        if let Some(h) = lines_of {
            print!("{}", line_changes(w, HouseId(h), &mut line_items));
        }
        classes(w, &starting, &mut g.offered, &mut g.made);
        // The effects pile up when nobody drains them: count the new ones.
        for e in w.effects.iter().skip(drained) {
            match e {
                Effect::Shot { .. } => g.shots += 1,
                Effect::Killed { .. } => g.killed += 1,
                _ => {}
            }
        }
        drained = w.effects.len();
        tally.observe(w);
        occasions(w, &mut g.occasions);
        features(w, &mut g.features);
        if trace > 0 && w.tick % trace == 0 {
            print!("{}", timeline(w));
        }
        if let Some((id, from, to)) = watch_obj
            && (from..=to).contains(&w.tick)
            && let Some(o) = w.obj(glasnost_sim::world::ObjectId(id))
        {
            println!(
                "watch tick {} obj {id}: at {:?} beh {:?} dest {:?} target {:?} contact {:?} path {:?}",
                w.tick,
                o.pos.cell(),
                o.beh.current,
                o.dest,
                o.target,
                o.contact,
                o.path
            );
        }
        if dump_at.contains(&w.tick) {
            print!("{}", dump(w));
        }
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
    g.loads = tally.loads;
    g.hover_loads = tally.hover_loads;
    g.unloads = tally.unloads;
    g.launches = tally.launches;
    g.launches_by_kind = tally.launches_by_kind;
    g.berserked = tally.berserked;
    g.intent_ticks = tally.intent_ticks;
    g.sensors_max = tally.sensors_max;
    g.berserk_armed = tally.berserk_armed.map(|s| s.len());
    g.ready_ticks = tally.ready_ticks;
    g.ticks = glasnost_sim::tick::ticks_run(&game.world);
    g.orders = game.replay().orders.len();
    for (h, u) in game.special_uses() {
        if let Some(s) = g.special.get_mut(usize::from(h.0)) {
            *s = u;
        }
    }
    let world = &game.world;
    g.occupants_inside = world
        .objects
        .values()
        .filter(|o| o.kind == glasnost_sim::rules::Kind::Building)
        .map(|o| o.structure.occupants.len())
        .sum();
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
    if !g.over {
        g.diagnosis = diagnosis(&game);
    }
    let end = game.state_hash();
    if std::env::var_os("GLASNOST_AI_QUICK").is_some() {
        // A diagnostic run: no replay, no save check.
        g.checks_skipped = true;
        return Ok(());
    }

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

/// What the lane counts of the transports and superweapons while a game
/// runs (one [`Tally::observe`] after every tick).
#[derive(Debug, Default)]
pub struct Tally {
    /// Passengers that boarded a transport.
    pub loads: usize,
    /// Of those, the loads onto a transport with the hovering locomotor.
    pub hover_loads: usize,
    /// Passengers let out beside a transport still standing.
    pub unloads: usize,
    /// Superweapon launches in all.
    pub launches: usize,
    /// Superweapon launches by kind name.
    pub launches_by_kind: BTreeMap<String, usize>,
    /// House-ticks a superweapon record stood ready, by kind name.
    pub ready_ticks: BTreeMap<String, usize>,
    /// Units that went berserk (S-0610 claim 4: a unit that was not berserk
    /// after the last tick and is now).
    pub berserked: usize,
    /// House-ticks in which a player house's psychic sensors showed at least
    /// one hostile-intent line (S-0610 claim 10).
    pub intent_ticks: usize,
    /// Per player house: the most psychic sensors it had at one tick.
    pub sensors_max: [usize; PLAYERS],
    /// Per player house: its units with a berserk weapon seen so far.
    pub berserk_armed: [BTreeSet<u32>; PLAYERS],
    /// The units berserk after the last tick.
    berserk_before: BTreeSet<u32>,
    /// The (house, superweapon type) pairs ready after the last tick.
    ready_before: BTreeSet<(usize, usize)>,
    /// The (passenger, transport) pairs of the last tick.
    riding: BTreeSet<(u32, u32)>,
}

impl Tally {
    /// Counts what changed in the world's last tick.
    pub fn observe(&mut self, w: &World) {
        let now: BTreeSet<(u32, u32)> = w
            .objects
            .keys()
            .filter_map(|&p| Some((p.0, glasnost_sim::transport::riding(w, p)?.0)))
            .collect();
        self.loads += now.difference(&self.riding).count();
        // The loads onto a hovering transport (it crosses water).
        self.hover_loads += now
            .difference(&self.riding)
            .filter(|&&(_, t)| {
                w.obj(glasnost_sim::world::ObjectId(t))
                    .and_then(|o| w.rules().unit(o.kind, o.type_index))
                    .is_some_and(|u| u.locomotor == Locomotor::Hover)
            })
            .count();
        self.unloads += self
            .riding
            .difference(&now)
            .filter(|&&(p, t)| {
                let alive = |i| {
                    w.obj(glasnost_sim::world::ObjectId(i))
                        .is_some_and(|o| o.on_map && o.health > 0)
                };
                alive(p) && alive(t)
            })
            .count();
        self.riding = now;
        // A launch spends a ready record that is still granted afterwards:
        // count each such fall of the ready mark (a lost giving building
        // also clears the grant).
        let ready_now: BTreeSet<(usize, usize)> = (0..PLAYERS)
            .flat_map(|h| (0..w.rules().superweapons.types.len()).map(move |i| (h, i)))
            .filter(|&(h, i)| {
                glasnost_sim::superweapons::charge::get(w, HouseId(h as u8), i as u32)
                    .is_some_and(|r| r.granted && r.ready && !r.suspended)
            })
            .collect();
        for &(h, i) in self.ready_before.difference(&ready_now) {
            if glasnost_sim::superweapons::charge::get(w, HouseId(h as u8), i as u32)
                .is_some_and(|r| r.granted)
            {
                self.launches += 1;
                let kind = w.rules().superweapons.types[i].kind;
                *self
                    .launches_by_kind
                    .entry(kind.map_or("no kind".to_string(), |k| format!("{k:?}")))
                    .or_default() += 1;
            }
        }
        // The ticks a record stood ready (what the sidebar shows), by kind:
        // many ready ticks with few launches name a kind the AI fails to aim.
        for &(_, i) in &ready_now {
            let kind = w.rules().superweapons.types[i].kind;
            *self
                .ready_ticks
                .entry(kind.map_or("no kind".to_string(), |k| format!("{k:?}")))
                .or_default() += 1;
        }
        self.ready_before = ready_now;
        let berserk: BTreeSet<u32> = w
            .objects
            .keys()
            .filter(|&&id| glasnost_sim::psychic::berserk(w, id))
            .map(|id| id.0)
            .collect();
        self.berserked += berserk.difference(&self.berserk_before).count();
        self.berserk_before = berserk;
        for h in 0..PLAYERS {
            let n = glasnost_sim::psychic::sensors(w, HouseId(h as u8)).len();
            self.sensors_max[h] = self.sensors_max[h].max(n);
        }
        for o in w.objects.values() {
            if let Some(h) = o.owner
                && usize::from(h.0) < PLAYERS
                && o.kind != glasnost_sim::rules::Kind::Building
                && glasnost_ai::psychic::berserk_weapon(w, o.id)
            {
                self.berserk_armed[usize::from(h.0)].insert(o.id.0);
            }
        }
        self.intent_ticks += (0..PLAYERS)
            .filter(|&h| !glasnost_sim::psychic::intent_lines(w, HouseId(h as u8)).is_empty())
            .count();
    }
}

/// The state of a game that did not end, counts only: per player house its
/// buildings and units by kind (and the units by movement zone), the
/// resource cells and bales left on the whole map, and the brain's own
/// diagnostic.
fn diagnosis(game: &AiGame) -> Vec<String> {
    let w = &game.world;
    let mut out = Vec::new();
    let mut cells = 0usize;
    let mut worth = 0i64;
    for (c, _) in w.cells.grid.iter() {
        let v = glasnost_sim::econ::resource::worth(w, c);
        if v > 0 {
            cells += 1;
            worth += v;
        }
    }
    out.push(format!(
        "resource left on the map: {cells} cells, worth {worth}"
    ));
    for (h, line) in game.diagnose() {
        let me = Some(h);
        let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
        for o in w
            .objects
            .values()
            .filter(|o| o.on_map && o.health > 0 && o.owner == me)
        {
            let zone = w
                .rules()
                .unit(o.kind, o.type_index)
                .map_or(String::new(), |u| format!("/{:?}", u.zone));
            *kinds.entry(format!("{:?}{zone}", o.kind)).or_default() += 1;
        }
        out.push(format!(
            "house {} credits {}; objects {kinds:?}; {line}",
            h.0,
            w.houses[usize::from(h.0)].credits
        ));
    }
    out
}

/// A diagnostic dump of every object of the player houses at this tick: its
/// type, cell, behaviour, destination and target (counts and states only).
fn dump(w: &World) -> String {
    let mut s = String::new();
    let cell_of = |t: &Option<glasnost_sim::world::Target>| match t {
        None => "-".to_string(),
        Some(glasnost_sim::world::Target::Cell(c)) => format!("cell({},{})", c.x, c.y),
        Some(t) => format!("{t:?}"),
    };
    for h in 0..PLAYERS {
        let _ = writeln!(
            s,
            "dump tick {} house {h} credits {} low power {}",
            w.tick,
            w.houses[h].credits,
            glasnost_sim::econ::power::low_power(w, HouseId(h as u8))
        );
        for l in w
            .econ
            .lines
            .lines
            .iter()
            .filter(|l| l.house == HouseId(h as u8))
        {
            let _ = writeln!(s, "  line {:?}: {:?}", l.kind, l.item);
        }
        for o in w
            .objects
            .values()
            .filter(|o| o.owner == Some(HouseId(h as u8)))
        {
            let name = match o.kind {
                Kind::Building => w
                    .rules()
                    .buildings
                    .get(o.type_index as usize)
                    .map(|b| b.name.clone()),
                _ => w.rules().unit(o.kind, o.type_index).map(|u| u.name.clone()),
            }
            .unwrap_or_default();
            let c = o.pos.cell();
            let _ = writeln!(
                s,
                "  obj {} {:?} {name} at ({},{}) hp {} map {} beh {:?} dest {} target {}",
                o.id.0,
                o.kind,
                c.x,
                c.y,
                o.health,
                o.on_map,
                o.beh.current,
                cell_of(&o.dest),
                cell_of(&o.target)
            );
        }
    }
    s
}

/// A diagnostic timeline of the player houses at this tick: credits, army
/// units on the map by kind and current behaviour, and each production
/// line's item (kind, progress, done, hold, waiting for money) and queue
/// length. Counts and states only.
fn timeline(w: &World) -> String {
    let mut s = String::new();
    for h in 0..PLAYERS {
        let house = HouseId(h as u8);
        let mut army: BTreeMap<String, usize> = BTreeMap::new();
        for o in w
            .objects
            .values()
            .filter(|o| o.on_map && o.health > 0 && o.owner == Some(house))
        {
            if role_of(w, tech::type_of(o)) == Some(Role::Army) {
                let loco = w.rules().unit(o.kind, o.type_index).map(|u| u.locomotor);
                *army
                    .entry(format!(
                        "{:?}/{:?} {:?}{}",
                        o.kind,
                        loco,
                        o.beh.current,
                        if o.contact.is_some() {
                            " in contact"
                        } else {
                            ""
                        }
                    ))
                    .or_default() += 1;
            }
            if o.kind == Kind::Building
                && let Some(f) = w
                    .rules()
                    .buildings
                    .get(o.type_index as usize)
                    .and_then(|b| b.factory)
            {
                let contacts: Vec<String> = w
                    .objects
                    .values()
                    .filter(|x| x.contact == Some(o.id))
                    .map(|x| {
                        let loco = w.rules().unit(x.kind, x.type_index).map(|u| u.locomotor);
                        format!(
                            "{:?}/{:?} {:?} on map {}",
                            x.kind, loco, x.beh.current, x.on_map
                        )
                    })
                    .collect();
                *army
                    .entry(format!(
                        "factory {f:?} {:?} contacts [{}]",
                        o.beh.current,
                        contacts.join(", ")
                    ))
                    .or_default() += 1;
            }
        }
        let lines: Vec<String> = w
            .econ
            .lines
            .lines
            .iter()
            .filter(|l| l.house == house)
            .map(|l| match &l.item {
                Some(i) => format!(
                    "{:?}: {:?} progress {} done {} hold {:?} waiting {} queue {}",
                    l.kind,
                    i.t.kind,
                    i.progress,
                    i.done_tick.is_some(),
                    i.hold,
                    i.waiting,
                    l.queue.len()
                ),
                None => format!("{:?}: empty, queue {}", l.kind, l.queue.len()),
            })
            .collect();
        let army: Vec<String> = army.iter().map(|(k, n)| format!("{k} ×{n}")).collect();
        let _ = writeln!(
            s,
            "trace tick {} house {h}: credits {}; army {}; lines [{}]",
            w.tick,
            w.houses[h].credits,
            army.join(", "),
            lines.join("; ")
        );
    }
    s
}

/// The report keys of the player houses' units by kind: infantry,
/// vehicles (not ships or hovers), ships, hovers, aircraft.
const UNIT_KEYS: [[&str; 5]; PLAYERS] = [
    [
        "house 0 infantry",
        "house 0 vehicles",
        "house 0 ships",
        "house 0 hovers",
        "house 0 aircraft",
    ],
    [
        "house 1 infantry",
        "house 1 vehicles",
        "house 1 ships",
        "house 1 hovers",
        "house 1 aircraft",
    ],
];

/// Notes the objects that show an M6 feature now: a spawner, a rocket in
/// flight (S-0506), a parasite inside its host (S-0614), an aircraft on the
/// map that is not a rocket (S-0501), a naval yard, a ship, a sinking ship
/// (S-0504), a hover (S-0503), a teleporting unit (S-0207), a repair depot,
/// a gate, a vehicle docked for repair at a depot or yard (S-0314, S-0504),
/// each house's units by kind, and a building the script builds for no role
/// (the tech climb); the owned ones count for the player houses only.
fn features(w: &World, out: &mut BTreeMap<&'static str, BTreeSet<u32>>) {
    let r = w.rules();
    for h in 0..PLAYERS {
        for i in 0..r.superweapons.types.len() {
            let ready = glasnost_sim::superweapons::charge::get(w, HouseId(h as u8), i as u32)
                .is_some_and(|c| c.granted && c.ready && !c.suspended);
            if ready {
                // A ready superweapon: keyed by house and type position, by kind.
                use glasnost_sim::superweapons::SwKind as K;
                let key = match r.superweapons.types[i].kind {
                    Some(K::Missile) => "ready: missile",
                    Some(K::Storm) => "ready: storm",
                    Some(K::IronCurtain) => "ready: iron curtain",
                    Some(K::Chrono | K::ChronoWarp) => "ready: teleport",
                    Some(K::Paradrop | K::AmerParadrop) => "ready: paradrop",
                    Some(K::SpyPlane) => "ready: spy plane",
                    Some(K::ForceShield) => "ready: force shield",
                    Some(_) => "ready: other kind",
                    None => "ready: no kind",
                };
                out.entry(key)
                    .or_default()
                    .insert((h * r.superweapons.types.len() + i) as u32);
            }
        }
    }
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
        // The player houses only: a map's civilian buildings are not the climb.
        if !o.on_map || o.owner.is_none_or(|h| usize::from(h.0) >= PLAYERS) {
            continue;
        }
        let house = o.owner.map_or(0, |h| usize::from(h.0));
        let loco = r.unit(o.kind, o.type_index).map(|u| u.locomotor);
        // Units by kind and house; ships and hovers apart from the other vehicles.
        let unit_kind = match (o.kind, loco) {
            (Kind::Infantry, _) => Some(0),
            (Kind::Vehicle, Some(Locomotor::Ship)) => Some(2),
            (Kind::Vehicle, Some(Locomotor::Hover)) => Some(3),
            (Kind::Vehicle, _) => Some(1),
            (Kind::Aircraft, _) if o.rocket.is_none() => Some(4),
            _ => None,
        };
        if let Some(k) = unit_kind {
            note(UNIT_KEYS[house][k]);
        }
        // A type with no locomotor resolves to the teleporting one (S-0207
        // claim 14): count only the types that name it.
        let names_teleport = r
            .unit(o.kind, o.type_index)
            .is_some_and(|u| u.raw.locomotor.is_some() && u.locomotor == Locomotor::Teleport);
        if names_teleport {
            note("teleporters");
        }
        match o.kind {
            Kind::Aircraft if o.rocket.is_none() => note("aircraft"),
            Kind::Building => {
                let b = r.buildings.get(o.type_index as usize);
                if b.is_some_and(|b| b.naval && b.factory.is_some()) {
                    note("naval yards");
                }
                if b.is_some_and(|b| b.building.unit_repair && !b.naval) {
                    note("repair depots");
                }
                if b.is_some_and(|b| b.gate) {
                    note("gates");
                }
                if r.superweapons
                    .building_sw
                    .get(o.type_index as usize)
                    .is_some_and(|f| f.iter().flatten().next().is_some())
                {
                    note("superweapon buildings");
                }
                // The building by the kind of each superweapon it gives.
                for &i in r
                    .superweapons
                    .building_sw
                    .get(o.type_index as usize)
                    .into_iter()
                    .flatten()
                    .flatten()
                {
                    use glasnost_sim::superweapons::SwKind as K;
                    let key = match r.superweapons.types.get(i as usize).and_then(|t| t.kind) {
                        Some(K::Missile) => "building gives: missile",
                        Some(K::Storm) => "building gives: storm",
                        Some(K::IronCurtain) => "building gives: iron curtain",
                        Some(K::Chrono | K::ChronoWarp) => "building gives: teleport",
                        Some(K::Paradrop) => "building gives: paradrop",
                        Some(K::AmerParadrop) => "building gives: american paradrop",
                        Some(K::SpyPlane) => "building gives: spy plane",
                        Some(K::ForceShield) => "building gives: force shield",
                        Some(_) => "building gives: other kind",
                        None => "building gives: no kind",
                    };
                    note(key);
                }
                if role_of(w, tech::type_of(o)).is_none() {
                    note("climb buildings");
                }
            }
            Kind::Vehicle => {
                if loco == Some(Locomotor::Ship) {
                    note("ships");
                }
                if loco == Some(Locomotor::Hover) {
                    note("hovers");
                }
                // A hovering or amphibious-zone unit standing on water (it
                // crossed the sea, S-0503).
                if (loco == Some(Locomotor::Hover)
                    || r.unit(o.kind, o.type_index)
                        .is_some_and(|u| u.zone == glasnost_rules::sim::MovementZone::Amphibious))
                    && w.cells
                        .cell(o.pos.cell())
                        .is_some_and(|c| c.land == glasnost_map::terrain::land::WATER)
                {
                    note("hover or amphibious on water");
                }
                if glasnost_sim::naval::sinking(w, *id) {
                    note("ships sinking");
                }
                if let Some(l) = glasnost_sim::depot::link(w, *id).filter(|l| l.tethered) {
                    let at_yard = w
                        .obj(l.depot)
                        .and_then(|d| r.buildings.get(d.type_index as usize))
                        .is_some_and(|b| b.naval);
                    note(if at_yard {
                        "docked for repair at a yard"
                    } else {
                        "docked for repair at a depot"
                    });
                }
            }
            _ => {}
        }
    }
}

/// The class of a unit type for the lane's coverage line: the first of
/// harvester, engineer, transport, jumpjet, ship, hover, teleporter,
/// aircraft, deployer, armed, unarmed that fits.
pub(crate) fn unit_class(w: &World, kind: Kind, index: u32) -> &'static str {
    let Some(u) = w.rules().unit(kind, index) else {
        return "unknown";
    };
    if u.harvester {
        "harvester"
    } else if u.engineer {
        "engineer"
    } else if u.passengers > 0 {
        "transport"
    } else if u.jumpjet_locomotor {
        "jumpjet"
    } else if u.locomotor == Locomotor::Ship {
        "ship"
    } else if u.locomotor == Locomotor::Hover {
        "hover"
    } else if u.raw.locomotor.is_some() && u.locomotor == Locomotor::Teleport {
        "teleporter"
    } else if kind == Kind::Aircraft {
        "aircraft"
    } else if u.deployer || glasnost_sim::econ::deploy::deploys_into(w, kind, index).is_some() {
        "deployer"
    } else if u.primary.is_some() {
        "armed"
    } else {
        "unarmed"
    }
}

/// The lines of `GLASNOST_AI_LINES`: what changed on house `h`'s production
/// lines since the last call (`items`: per line index its last item type
/// and progress). Classes and counts only.
fn line_changes(
    w: &World,
    h: HouseId,
    items: &mut BTreeMap<String, Option<(glasnost_sim::econ::TypeRef, i32)>>,
) -> String {
    let mut s = String::new();
    let what = |t: glasnost_sim::econ::TypeRef| {
        if t.kind == Kind::Building {
            format!(
                "building {:?}",
                role_of(w, t).map_or("other".into(), role_name)
            )
        } else {
            format!("{:?} {}", t.kind, unit_class(w, t.kind, t.index))
        }
    };
    let credits = w.houses[usize::from(h.0)].credits;
    for l in &w.econ.lines.lines {
        if l.house != h {
            continue;
        }
        let i = format!("{:?}", l.kind);
        let now = l.item.as_ref().map(|it| (it.t, it.progress));
        let before = items.get(&i).copied().flatten();
        match (before, now) {
            (Some((a, _)), Some((b, _))) if a == b => {
                if w.tick % 300 == 0 {
                    let it = l.item.as_ref().expect("an item");
                    let _ = writeln!(
                        s,
                        "lines tick {} line {i}: {} progress {} waiting {} credits {credits} queue {}",
                        w.tick,
                        what(b),
                        it.progress,
                        it.waiting,
                        l.queue.len()
                    );
                }
            }
            (b, n) if b.map(|x| x.0) != n.map(|x| x.0) => {
                if let Some((a, p)) = b {
                    let _ = writeln!(
                        s,
                        "lines tick {} line {i}: ended {} at progress {p}",
                        w.tick,
                        what(a)
                    );
                }
                if let Some((t, _)) = n {
                    let _ = writeln!(
                        s,
                        "lines tick {} line {i}: started {} credits {credits}",
                        w.tick,
                        what(t)
                    );
                }
            }
            _ => {}
        }
        items.insert(i, now);
    }
    s
}

/// Counts, per player house, the occasions for charges (S-0518) and
/// disarms (S-0519) over a game: the types it could build at some sampled
/// tick, and the house-ticks in which a person in its seat could have
/// charged or disarmed. Counts only.
#[derive(Debug, Default)]
struct Occasions {
    /// Distinct buildable types: chargers (slot-1 electric-charge warhead),
    /// overpowerable buildings, of them the prism type, disarmers (slot-0
    /// bomb-disarm warhead) and bomb detectors (bomb-sight above 0).
    chargers: BTreeSet<(u8, u32)>,
    overpowerable: BTreeSet<u32>,
    overpowerable_prism: BTreeSet<u32>,
    disarmers: BTreeSet<(u8, u32)>,
    detectors: BTreeSet<(u8, u32)>,
    /// House-ticks with an own charger; with an own overpowerable building;
    /// with both; with an idle charger within 12 cells of one.
    charger_ticks: u32,
    overpowerable_ticks: u32,
    both_ticks: u32,
    idle_near_ticks: u32,
    /// Distinct own objects that carried a bomb; house-ticks with an own
    /// object carrying a bomb; with one the house sees; with a seen one and
    /// an own disarmer; and an idle one; and an idle one in the same reach
    /// region that could cover the straight-line distance before the fuse
    /// ends.
    bombed: BTreeSet<u32>,
    bomb_ticks: u32,
    seen_ticks: u32,
    seen_disarmer_ticks: u32,
    seen_idle_ticks: u32,
    seen_reachable_ticks: u32,
}

fn cell_dist2(a: glasnost_map::CellXY, b: glasnost_map::CellXY) -> i64 {
    let (dx, dy) = (i64::from(a.x - b.x), i64::from(a.y - b.y));
    dx * dx + dy * dy
}

fn kind_code(k: Kind) -> u8 {
    match k {
        Kind::Infantry => 0,
        Kind::Vehicle => 1,
        Kind::Aircraft => 2,
        _ => 3,
    }
}

/// One tick of [`Occasions`] for every player house.
fn occasions(w: &World, occ: &mut [Occasions; PLAYERS]) {
    use glasnost_sim::world::ObjectId;
    let r = w.rules();
    let slot_warhead = |c: &glasnost_sim::combat::data::TypeCombat, i: usize| {
        c.slots[i].and_then(|wp| r.combat.warhead_of(wp))
    };
    if w.tick % 300 == 0 {
        for t in all_types(w) {
            for (h, o) in occ.iter_mut().enumerate() {
                let house = HouseId(h as u8);
                if !tech::buildable(w, house, t) {
                    continue;
                }
                if t.kind == Kind::Building {
                    let b = &r.buildings[t.index as usize];
                    if b.building.overpowerable {
                        o.overpowerable.insert(t.index);
                        if r.combat.general.prism.building == Some(t.index) {
                            o.overpowerable_prism.insert(t.index);
                        }
                    }
                    if b.combat.bomb_sight > 0 {
                        o.detectors.insert((3, t.index));
                    }
                } else if let Some(u) = r.unit(t.kind, t.index) {
                    let k = (kind_code(t.kind), t.index);
                    if matches!(t.kind, Kind::Infantry | Kind::Vehicle)
                        && slot_warhead(&u.combat, 1).is_some_and(|x| x.electric_assault)
                    {
                        o.chargers.insert(k);
                    }
                    if slot_warhead(&u.combat, 0).is_some_and(|x| x.bomb_disarm) {
                        o.disarmers.insert(k);
                    }
                    if u.combat.bomb_sight > 0 {
                        o.detectors.insert(k);
                    }
                }
            }
        }
    }
    let delay = r.combat.general.ivan.delay;
    for (h, o) in occ.iter_mut().enumerate() {
        let house = HouseId(h as u8);
        let own: Vec<ObjectId> = w
            .objects
            .values()
            .filter(|x| x.owner == Some(house) && x.on_map)
            .map(|x| x.id)
            .collect();
        let chargers: Vec<ObjectId> = own
            .iter()
            .copied()
            .filter(|&id| glasnost_sim::linked::charger_type(w, id))
            .collect();
        let over: Vec<ObjectId> = own
            .iter()
            .copied()
            .filter(|&id| glasnost_sim::linked::overpowerable(w, id))
            .collect();
        o.charger_ticks += u32::from(!chargers.is_empty());
        o.overpowerable_ticks += u32::from(!over.is_empty());
        o.both_ticks += u32::from(!chargers.is_empty() && !over.is_empty());
        let near = chargers.iter().any(|&c| {
            glasnost_ai::scripted::idle(w, c)
                && over.iter().any(|&b| {
                    let (Some(a), Some(z)) = (w.obj(c), w.obj(b)) else {
                        return false;
                    };
                    cell_dist2(a.pos.cell(), z.pos.cell()) <= 144
                })
        });
        o.idle_near_ticks += u32::from(near);
        let carriers: Vec<ObjectId> = own
            .iter()
            .copied()
            .filter(|&id| glasnost_sim::bomb::carries(w, id))
            .collect();
        if carriers.is_empty() {
            continue;
        }
        o.bombed.extend(carriers.iter().map(|id| id.0));
        o.bomb_ticks += 1;
        let seen: Vec<ObjectId> = carriers
            .into_iter()
            .filter(|&c| glasnost_sim::bomb::visible(w, house, c))
            .collect();
        if seen.is_empty() {
            continue;
        }
        o.seen_ticks += 1;
        let disarmers: Vec<ObjectId> = own
            .iter()
            .copied()
            .filter(|&id| {
                glasnost_sim::combat::weapon::slot_weapon(w, id, 0)
                    .and_then(|wp| r.combat.warhead_of(wp))
                    .is_some_and(|x| x.bomb_disarm)
            })
            .collect();
        if disarmers.is_empty() {
            continue;
        }
        o.seen_disarmer_ticks += 1;
        let idle: Vec<ObjectId> = disarmers
            .into_iter()
            .filter(|&d| glasnost_ai::scripted::idle(w, d))
            .collect();
        if idle.is_empty() {
            continue;
        }
        o.seen_idle_ticks += 1;
        let reachable = idle.iter().any(|&d| {
            let Some(dob) = w.obj(d) else { return false };
            let Some(u) = r.unit(dob.kind, dob.type_index) else {
                return false;
            };
            let speed = if dob.kind == Kind::Infantry {
                glasnost_sim::walk::walking_speed(w, d)
            } else {
                glasnost_sim::rules::type_speed(u.speed)
            };
            if speed <= 0 {
                return false;
            }
            let here = glasnost_sim::regions::reach_in(w, dob.pos.cell(), u.zone);
            seen.iter().any(|&c| {
                let Some(cob) = w.obj(c) else { return false };
                let Some(b) = cob.combat.bomb else {
                    return false;
                };
                let left = delay - (w.tick - b.planted);
                let same = glasnost_sim::regions::reach_in(w, cob.pos.cell(), u.zone) == here;
                let d2 = cell_dist2(cob.pos.cell(), dob.pos.cell());
                // Straight-line leptons (256 per cell) over the speed: a lower bound of the trip.
                let reach = i64::from(left.max(0)) * i64::from(speed);
                same && d2 * 256 * 256 <= reach * reach
            })
        });
        o.seen_reachable_ticks += u32::from(reachable);
    }
}

/// Notes, per player house, the unit types it can build (every 300th tick)
/// and the types of its new objects (every tick; `starting` holds the ids
/// that were there at the start).
fn classes(
    w: &World,
    starting: &BTreeSet<u32>,
    offered: &mut [BTreeMap<(&'static str, u8, u32), (i32, bool, Option<Role>)>; PLAYERS],
    made: &mut [BTreeSet<(&'static str, u8, u32)>; PLAYERS],
) {
    let units = |t: &glasnost_sim::econ::TypeRef| {
        matches!(t.kind, Kind::Infantry | Kind::Vehicle | Kind::Aircraft)
    };
    if w.tick % 300 == 1 {
        for t in all_types(w).into_iter().filter(units) {
            for (h, set) in offered.iter_mut().enumerate() {
                if tech::buildable(w, HouseId(h as u8), t) {
                    set.entry((unit_class(w, t.kind, t.index), t.kind as u8, t.index))
                        .or_insert((
                            w.tick,
                            w.rules()
                                .unit(t.kind, t.index)
                                .is_some_and(|u| u.primary.is_some()),
                            role_of(w, t),
                        ));
                }
            }
        }
    }
    for (id, o) in &w.objects {
        if starting.contains(&id.0) || !o.on_map || o.rocket.is_some() {
            continue;
        }
        let Some(h) = o.owner.map(|h| usize::from(h.0)).filter(|&h| h < PLAYERS) else {
            continue;
        };
        let t = tech::type_of(o);
        if units(&t) {
            made[h].insert((unit_class(w, t.kind, t.index), t.kind as u8, t.index));
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
            "map index {}: {} cells ({} water), inputs {} ms",
            g.index, g.cells, g.water, g.load_ms
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
        if players != Players::Scripted {
            let _ = writeln!(
                s,
                "  transports: {} loads ({} onto a hovering transport), {} unloads; superweapon launches: {}{}",
                g.loads,
                g.hover_loads,
                g.unloads,
                g.launches,
                if g.launches_by_kind.is_empty() && g.ready_ticks.is_empty() {
                    String::new()
                } else {
                    format!(
                        " (by kind: {:?}; house-ticks ready by kind: {:?})",
                        g.launches_by_kind, g.ready_ticks
                    )
                }
            );
            let uses: Vec<String> = g
                .special
                .iter()
                .enumerate()
                .map(|(h, u)| {
                    format!(
                        "house {h} garrison {} capture {} (tech buildings {}) infiltrate {} jumpjets {} deploys {} radiation exits {} erasures {} prisms {} overpowerable buildings {} charges {} bombs {} disarms {} suicides {} sabotages {} strikes {} crates {} berserk attacks {} psychic sensors {} psychic reveals {}",
                        u.garrisons, u.engineers, u.tech, u.spies, u.jumps, u.deploys, u.radiation, u.erasures, u.prisms, u.overpowerables, u.charges, u.bombs, u.disarms, u.suicides, u.sabotages, u.strikes, u.crates, u.berserks, u.sensors, u.reveals
                    )
                })
                .collect();
            let _ = writeln!(
                s,
                "  special abilities (orders): {}; garrison occupants inside at the end: {}",
                uses.join("; "),
                g.occupants_inside
            );
            let occ: Vec<String> = g
                .occasions
                .iter()
                .enumerate()
                .map(|(h, o)| {
                    format!(
                        "house {h} types: chargers {} overpowerable {} (prism type {}) disarmers {} detectors {}; house-ticks: charger {} overpowerable {} both {} idle charger near {}; bombed objects {} bomb {} seen {} seen+disarmer {} seen+idle {} seen+idle reachable {}",
                        o.chargers.len(),
                        o.overpowerable.len(),
                        o.overpowerable_prism.len(),
                        o.disarmers.len(),
                        o.detectors.len(),
                        o.charger_ticks,
                        o.overpowerable_ticks,
                        o.both_ticks,
                        o.idle_near_ticks,
                        o.bombed.len(),
                        o.bomb_ticks,
                        o.seen_ticks,
                        o.seen_disarmer_ticks,
                        o.seen_idle_ticks,
                        o.seen_reachable_ticks
                    )
                })
                .collect();
            let _ = writeln!(s, "  occasions (S-0518, S-0519): {}", occ.join("; "));
            let _ = writeln!(
                s,
                "  psychic weapons: units gone berserk {}, house-ticks with a hostile-intent line {}; most psychic sensors at once per house {:?}; units with a berserk weapon per house {:?}",
                g.berserked, g.intent_ticks, g.sensors_max, g.berserk_armed
            );
        }
        let f: Vec<String> = g
            .features
            .iter()
            .map(|(k, v)| format!("{k} {}", v.len()))
            .collect();
        let _ = writeln!(s, "  M6 features (distinct objects): {}", f.join(", "));
        if players != Players::Scripted {
            for h in 0..PLAYERS {
                let mut by: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
                for &(c, k, i) in g.offered[h].keys() {
                    let e = by.entry(c).or_default();
                    e.0 += 1;
                    if g.made[h].contains(&(c, k, i)) {
                        e.1 += 1;
                    }
                }
                let never: usize = by.values().map(|(o, m)| o - m).sum();
                let v: Vec<String> = by
                    .iter()
                    .map(|(c, (o, m))| format!("{c} {m}/{o}"))
                    .collect();
                let _ = writeln!(
                    s,
                    "  house {h} unit types by class (built/buildable): {}; buildable but never built: {never}",
                    v.join(", ")
                );
                // The never-built types by kind, class, weapon and the first tick they were offered (no names).
                let kinds = [Kind::Infantry, Kind::Vehicle, Kind::Aircraft];
                let left: Vec<String> = g.offered[h]
                    .iter()
                    .filter(|(t, _)| !g.made[h].contains(t))
                    .map(|(&(c, k, _), &(first, armed, role))| {
                        let kind = kinds.into_iter().find(|&x| x as u8 == k);
                        format!(
                            "{:?} {c} {} (AI role {role:?}) from tick {first}",
                            kind.unwrap_or(Kind::Infantry),
                            if armed { "armed" } else { "unarmed" }
                        )
                    })
                    .collect();
                if !left.is_empty() {
                    let _ = writeln!(s, "  house {h} never built: {}", left.join(", "));
                }
            }
        }
        for h in 0..PLAYERS {
            if !g.army_end[h].is_empty() {
                let v: Vec<String> = g.army_end[h]
                    .iter()
                    .map(|(k, n)| format!("{k} ×{n}"))
                    .collect();
                let _ = writeln!(s, "  house {h} army at the end: {}", v.join(", "));
            }
        }
        for line in &g.diagnosis {
            let _ = writeln!(s, "  diagnosis: {line}");
        }
        if g.checks_skipped {
            let _ = writeln!(s, "  replay and save checks skipped (GLASNOST_AI_QUICK)");
        } else {
            let _ = writeln!(
                s,
                "  replay reproduces the final state: {}; save in mid-game, loaded and continued, matches: {}",
                g.replay_matches, g.save_matches
            );
        }
    }
    s
}
