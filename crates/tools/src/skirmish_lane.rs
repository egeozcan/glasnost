//! The local lane's scripted skirmish (M5a): two scripted houses play a
//! game on stock maps of the install named by `GLASNOST_GAME_DIR`, through
//! orders only, to the last side (S-0414). The scripts pick their types by
//! role from the install's rules, so nothing here names a stock type. The
//! report holds counts and timings only (no names, no values, no paths) and
//! checks that the recorded replay reproduces the game and that a save in
//! mid-game, loaded and continued, ends in the same state. Never in CI.
//!
//! `GLASNOST_PLAY_FIXES=1` plays with our own rules patch layer *Pathing
//! and other fixes* (M8 step C7, `glasnost_rules::layers`); unset, the lane
//! plays the reference ruleset, and the report says so only when it is on.
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
use glasnost_rules::layers::Layers;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::econ::setup::{self, Player, Settings};
use glasnost_sim::econ::tech;
use glasnost_sim::load::new_game;
use glasnost_sim::rules::{Kind, Locomotor, SimRules};
use glasnost_sim::save;
use glasnost_sim::world::{HouseId, World};

/// A unit type as the unit-class counts key it: class, kind, type index.
type UnitKey = (&'static str, u8, u32);

/// What the lane notes of an offered unit type: the first tick it was
/// buildable, whether it is armed, and its AI role.
type Offer = (i32, bool, Option<Role>);

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

/// The tick at which [`R8Watch`] counts the cash tech buildings each house
/// holds.
const R8_SAMPLE: i32 = 9000; // nocite: our own lane's sampling tick

/// Army units beyond half the way to the enemy's yard that make a push for
/// [`R8Watch`].
const R8_PUSH: usize = 4; // nocite: our own lane's threshold

/// M8 step A4 round 8 (the lead's notes 48, 51 and 52), counts only, per
/// player house: the first tick it saw (explored) and held a cash-producing
/// tech building and how many it held at [`R8_SAMPLE`]; its engineers that
/// went into its own buildings, and of those the ones into a building above
/// half strength or of no economy or production role; its first push (the
/// tick at which [`R8_PUSH`] of its army units first stood beyond half the
/// way from its yard to the enemy's, its army units then beyond that and
/// within a quarter of the way home) and its largest number of army units
/// beyond half the way at once.
#[derive(Debug, Default)]
struct R8Watch {
    tech_seen: [Option<i32>; PLAYERS],
    tech_held: [Option<i32>; PLAYERS],
    tech_at: [usize; PLAYERS],
    repairs: [usize; PLAYERS],
    wasteful: [usize; PLAYERS],
    /// Engineers bound for an own building: house, building, wasteful.
    bound: BTreeMap<glasnost_sim::world::ObjectId, (usize, glasnost_sim::world::ObjectId, bool)>,
    push: [Option<(i32, usize, usize)>; PLAYERS],
    out_max: [usize; PLAYERS],
    /// Every engineer seen on the map, with its house.
    engineers: BTreeMap<glasnost_sim::world::ObjectId, usize>,
    /// Engineers killed.
    eng_lost: [usize; PLAYERS],
    /// The first tick an engineer of the house was bound for a cash tech
    /// building it does not hold.
    tech_bound: [Option<i32>; PLAYERS],
}

impl R8Watch {
    fn observe(&mut self, w: &World, killed: &BTreeSet<glasnost_sim::world::ObjectId>) {
        use glasnost_sim::world::{ObjectId, Target};
        let cash = |o: &glasnost_sim::world::Object| {
            o.kind == Kind::Building
                && o.on_map
                && o.health > 0
                && w.rules()
                    .buildings
                    .get(o.type_index as usize)
                    .is_some_and(|t| t.raw.needs_engineer.unwrap_or(false))
                && glasnost_sim::structures::tech::cash_producing(w, o.id)
        };
        let yards: Vec<Option<glasnost_map::CellXY>> = (0..PLAYERS)
            .map(|h| {
                w.objects
                    .values()
                    .find(|o| {
                        o.owner == Some(HouseId(h as u8))
                            && o.on_map
                            && glasnost_ai::scripted::is_yard(w, o)
                    })
                    .map(|o| o.pos.cell())
            })
            .collect();
        for h in 0..PLAYERS {
            let house = HouseId(h as u8);
            if self.tech_seen[h].is_none()
                && w.objects.values().any(|o| {
                    cash(o)
                        && o.owner != Some(house)
                        && glasnost_sim::econ::shroud::explored(w, house, o.pos.cell())
                })
            {
                self.tech_seen[h] = Some(w.tick);
            }
            let held = w
                .objects
                .values()
                .filter(|o| cash(o) && o.owner == Some(house))
                .count();
            if held > 0 && self.tech_held[h].is_none() {
                self.tech_held[h] = Some(w.tick);
            }
            if w.tick == R8_SAMPLE {
                self.tech_at[h] = held;
            }
            // The push: army units beyond half the way to the enemy's yard.
            let (Some(me), Some(them)) = (yards[h], yards[1 - h]) else {
                continue;
            };
            let d2 = |a: glasnost_map::CellXY, b: glasnost_map::CellXY| {
                i64::from(a.x - b.x).pow(2) + i64::from(a.y - b.y).pow(2)
            };
            let way = d2(me, them);
            let (mut out, mut home) = (0usize, 0usize);
            for o in w.objects.values().filter(|o| {
                o.owner == Some(house)
                    && o.on_map
                    && o.health > 0
                    && o.kind != Kind::Building
                    && o.kind != Kind::Aircraft
                    && role_of(w, tech::type_of(o)) == Some(Role::Army)
            }) {
                let d = d2(o.pos.cell(), me);
                if d * 4 > way {
                    out += 1;
                } else if d * 16 <= way {
                    home += 1;
                }
            }
            self.out_max[h] = self.out_max[h].max(out);
            if out >= R8_PUSH && self.push[h].is_none() {
                self.push[h] = Some((w.tick, out, home));
            }
        }
        // Engineers bound for an own building, and those that went in.
        for o in w.objects.values().filter(|o| o.on_map && o.health > 0) {
            let Some(h) = o.owner.map(|h| usize::from(h.0)).filter(|&h| h < PLAYERS) else {
                continue;
            };
            if !glasnost_sim::structures::engineer::is_engineer(w, o.id) {
                continue;
            }
            self.engineers.insert(o.id, h);
            let goal = match (o.dest, o.target) {
                (Some(Target::Object(b)), _) | (_, Some(Target::Object(b))) => Some(b),
                _ => None,
            };
            if self.tech_bound[h].is_none()
                && goal
                    .and_then(|b| w.obj(b))
                    .is_some_and(|b| b.owner != o.owner && cash(b))
            {
                self.tech_bound[h] = Some(w.tick);
            }
            match goal
                .and_then(|b| w.obj(b))
                .filter(|b| b.kind == Kind::Building && b.owner == o.owner && b.on_map)
            {
                Some(b) => {
                    let strength = glasnost_sim::combat::damage::strength(w, b.id);
                    let role = role_of(w, tech::type_of(b));
                    let matters =
                        matches!(role, Some(Role::Yard | Role::Refinery | Role::Factory(_)))
                            || cash(b);
                    let wasteful = b.health * 2 > strength || !matters;
                    self.bound.insert(o.id, (h, b.id, wasteful));
                }
                None => {
                    self.bound.remove(&o.id);
                }
            }
        }
        for (e, &h) in &self.engineers {
            if killed.contains(e) {
                self.eng_lost[h] += 1;
            }
        }
        let gone: Vec<ObjectId> = self
            .bound
            .keys()
            .copied()
            .filter(|&e| w.obj(e).is_none_or(|o| !o.on_map))
            .collect();
        for e in gone {
            let (h, b, wasteful) = self.bound.remove(&e).unwrap();
            if killed.contains(&e) || w.obj(b).is_none_or(|o| !o.on_map) {
                continue;
            }
            self.repairs[h] += 1;
            if wasteful {
                self.wasteful[h] += 1;
            }
        }
    }
}

/// One map's game.
#[derive(Debug, Default)]
struct Game {
    index: usize,
    load_ms: u128,
    /// M8 step A4 round 8's counts ([`R8Watch`]).
    r8: R8Watch,
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
    /// Per player house (M8 step B1): the most construction yards it owned
    /// at once, the first tick it owned two, the first sampled tick it
    /// could build a yard vehicle, and over the expansion probe's samples
    /// how often each condition of the yard vehicle's build failed.
    yards_max: [usize; PLAYERS],
    two_yards_at: [Option<i32>; PLAYERS],
    yard_unit_from: [Option<i32>; PLAYERS],
    expansion_blocks: [BTreeMap<String, usize>; PLAYERS],
    /// Per player house (M8 step B2): the first tick each superweapon kind
    /// was granted and stood ready.
    first_granted: [BTreeMap<String, i32>; PLAYERS],
    first_ready: [BTreeMap<String, i32>; PLAYERS],
    occupants_inside: usize,
    /// Per player house: the occasions for charges and disarms (S-0518,
    /// S-0519) a person in its seat would have had (M7a step 2c).
    occasions: [Occasions; PLAYERS],
    /// The launches by superweapon kind (the kind names of the simulation).
    launches_by_kind: BTreeMap<String, usize>,
    /// House-ticks a superweapon record stood ready, by kind name.
    ready_ticks: BTreeMap<String, usize>,
    /// House-ticks a superweapon record stood granted, and granted but
    /// suspended, by kind name.
    granted_ticks: BTreeMap<String, (usize, usize)>,
    /// The suspended house-ticks by cause (see [`Tally::suspended_causes`]).
    suspended_causes: BTreeMap<String, [usize; 3]>,
    /// Units that went berserk (S-0610 claim 4), and house-ticks in which a
    /// player house's psychic sensors showed a hostile-intent line
    /// (claim 10).
    berserked: usize,
    intent_ticks: usize,
    /// Per player house: the most psychic sensors it had at one tick, and
    /// its distinct units with a berserk weapon (S-0610).
    sensors_max: [usize; PLAYERS],
    berserk_armed: [usize; PLAYERS],
    /// Takeovers (S-0601 claim 6), and per player house the most objects
    /// it controlled at once and its distinct controllers.
    taken: usize,
    controlled_max: [usize; PLAYERS],
    controllers: [usize; PLAYERS],
    /// Overload pulses of unlimited controllers that played the overload
    /// sound (S-0601 claim 16), and per player house the most victims one
    /// unlimited controller held at once.
    overloads: usize,
    unlimited_max: [usize; PLAYERS],
    /// Per player house: infantry copies released at a cloning building
    /// (S-0404 claim 14).
    clones: [usize; PLAYERS],
    /// Per player house: the most slave miners (S-0402) and the most slaves
    /// on the map at once.
    masters_max: [usize; PLAYERS],
    slaves_max: [usize; PLAYERS],
    /// Per player house: house-ticks with a gattling at its top stage, and
    /// the distinct gattling types it had (S-0609).
    top_ticks: [usize; PLAYERS],
    gattling_types: [usize; PLAYERS],
    /// Gas (S-0613): hits clouds dealt, clouds that appeared, the most gas
    /// particles at once and per player house its virus units.
    gas_hits: usize,
    clouds: usize,
    gas_max: usize,
    virus_armed: [usize; PLAYERS],
    /// Particle systems (S-0616): systems that appeared, by behaviour kind;
    /// effect-weapon emitters made, by place (fire, spark, railgun); the
    /// most particles of every kind at once.
    systems_by_kind: BTreeMap<String, usize>,
    emitters: [usize; 3],
    particles_max: usize,
    /// Absorbing buildings (S-0605): entries, releases with the building
    /// still standing, and per player house the most passengers its
    /// absorbers held at once and its distinct absorbing buildings.
    absorbed: usize,
    absorber_releases: usize,
    passengers_max: [usize; PLAYERS],
    absorbers: [usize; PLAYERS],
    /// Recycling buildings (S-0603): units consumed and per player house
    /// its distinct recycling buildings.
    ground: usize,
    grinders: [usize; PLAYERS],
    /// Tank bunkers (S-0606): vehicles that became bunkered, vehicles let
    /// out (link ended with both still standing), and per player house its
    /// distinct bunkers.
    bunkered: usize,
    bunker_releases: usize,
    bunkers: [usize; PLAYERS],
    /// Magnetic lift (S-0607): units that became seized and seized units
    /// that became dropped.
    seized: usize,
    dropped: usize,
    /// Special-hit effects (S-0615): burning walkers made, and disc-laser
    /// shots, hits (final steps that detonated) and cancellations.
    walkers: usize,
    disc_shots: usize,
    disc_hits: usize,
    disc_cancelled: usize,
    /// Drain (S-0608): links formed, links ended, credits moved by
    /// payouts, and the sum over ticks of the buildings drained.
    drain_links: usize,
    drain_ends: usize,
    credits_drained: i64,
    drained_ticks: usize,
    /// Genetic mutation (S-0611): launches, infantry-making animations
    /// started and infantry the conversion animations made.
    mutator_launches: usize,
    conversions: usize,
    infantry_mutated: usize,
    /// Psychic domination (S-0612): launches, objects taken over, objects
    /// the sweep's area damage hurt and destroyed.
    dominator_launches: usize,
    dominated: usize,
    dominator_damaged: usize,
    dominator_killed: usize,
    dominator_sweeps: Vec<usize>,
    /// The distinct objects that showed each M6 feature (see [`features`]).
    features: BTreeMap<&'static str, BTreeSet<u32>>,
    /// Per player house: the unit types (infantry, vehicles, aircraft) it
    /// could build at some sampled tick and the ones it had a new object
    /// of (not a starting unit), by [`unit_class`] (PLAN 4.7: the own AI
    /// builds every unit class its side can build).
    offered: [BTreeMap<UnitKey, Offer>; PLAYERS],
    made: [BTreeSet<UnitKey>; PLAYERS],
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
    let layers = Layers::from_flag(std::env::var("GLASNOST_PLAY_FIXES").ok().as_deref());
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
        let inputs = glasnost_convert::game::load(&mut vfs, bytes, "install".into(), layers).await;
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
    Ok(report(
        mount_ms, looked, &games, max_ticks, seed, players, layers,
    ))
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
    // `GLASNOST_AI_STALE=n`: every n ticks, the player houses' objects that
    // hold a removed object as destination or target, and the rockets on the
    // map whose flight has not started.
    let stale_every: i32 = std::env::var("GLASNOST_AI_STALE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
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
    // `GLASNOST_AI_WATCH_VS=id`: with the watch, the firing check's answers
    // (every reason, and the own AI's lasting test, both without range) of
    // the watched object against object `id`, the slot it would use, its
    // infantry sequence and the AI's attack test.
    let watch_vs: Option<u32> = std::env::var("GLASNOST_AI_WATCH_VS")
        .ok()
        .and_then(|v| v.trim().parse().ok());
    // `GLASNOST_AI_HASH=n,from,to`: the world's state hash (the AI's state
    // left out) every n ticks, and every tick from `from` to `to`; to find
    // the first tick at which two builds' games part.
    let hash_at: Option<(i32, i32, i32)> = std::env::var("GLASNOST_AI_HASH").ok().and_then(|v| {
        let p: Vec<i32> = v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
        Some((
            *p.first()?,
            p.get(1).copied().unwrap_or(0),
            p.get(2).copied().unwrap_or(-1),
        ))
    });
    // `GLASNOST_AI_OBJS=from,to`: every object's whole state (`Debug`, one
    // line each), the spawned objects, the credits and the random number
    // generator, every tick from
    // `from` to `to`; to diff two builds' games at the tick they part.
    let objs_at: Option<(i32, i32)> = std::env::var("GLASNOST_AI_OBJS").ok().and_then(|v| {
        let p: Vec<i32> = v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
        Some((*p.first()?, *p.get(1)?))
    });
    // `GLASNOST_AI_LINES=h`: every item house h starts or loses on its
    // production lines (tick, line kind, unit class or building, cost,
    // credits), and every 300 ticks a still item's progress.
    let lines_of: Option<u8> = std::env::var("GLASNOST_AI_LINES")
        .ok()
        .and_then(|v| v.trim().parse().ok());
    let mut line_items: BTreeMap<String, Option<(glasnost_sim::econ::TypeRef, i32)>> =
        BTreeMap::new();
    // `GLASNOST_AI_ECON=n`: an economy line per house every n ticks (income,
    // spending by category, harvesters, slave miners and slaves).
    let econ_every: i32 = std::env::var("GLASNOST_AI_ECON")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let mut econ = EconTrace::default();
    // `GLASNOST_AI_SWPROBE=n`: every n ticks, per house and charged
    // mutator or dominator record, what the AI's aim sees.
    let sw_probe: i32 = std::env::var("GLASNOST_AI_SWPROBE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    // `GLASNOST_AI_GUNPROBE=n`: every n ticks, per house, what the AI's
    // gunner loading sees (`glasnost_ai::gunner::probe`).
    let gun_probe: i32 = std::env::var("GLASNOST_AI_GUNPROBE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    // `GLASNOST_AI_EXPROBE=n`: every n ticks, per house, what the AI's
    // expansion step sees (`glasnost_ai::brain::ExpansionProbe`).
    let ex_probe: i32 = std::env::var("GLASNOST_AI_EXPROBE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    // `GLASNOST_AI_AAPROBE=n`: every n ticks, per house that sees a drainer,
    // what the AI's anti-air step sees (`glasnost_ai::antiair::probe`).
    let aa_probe: i32 = std::env::var("GLASNOST_AI_AAPROBE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    // `GLASNOST_AI_ECOPROBE=n`: every n ticks, per house, what the AI's
    // economy sees (`glasnost_ai::brain::Brain::economy_probe`), and at the
    // end each house's harvester cycles (M8 step B8).
    let eco_probe: i32 = std::env::var("GLASNOST_AI_ECOPROBE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let mut cycles = CycleTrace::default();
    let mut field_spells = FieldSpells::default();
    let mut broke_spells = BrokeSpells::default();
    let t = Instant::now();
    let mut game = AiGame::new(rules.clone(), map, "install", tiles.clone(), setup.clone())
        .map_err(|e| format!("{e:?}"))?;
    let starting: BTreeSet<u32> = game.world.objects.keys().map(|id| id.0).collect();
    // The house of a sweep seen in this tick's effects, and the objects of
    // other houses the last launch's preview would take.
    let mut swept_by: Option<HouseId> = None;
    let mut dom_watch: Vec<glasnost_sim::world::ObjectId> = Vec::new();
    while game.world.tick < max_ticks {
        game.step();
        let w = &game.world;
        if let Some(h) = lines_of {
            print!("{}", line_changes(w, HouseId(h), &mut line_items));
        }
        classes(w, &starting, &mut g.offered, &mut g.made);
        // The effects pile up when nobody drains them: count the new ones.
        let mut killed_now = BTreeSet::new();
        for e in w.effects.iter().skip(drained) {
            if let Effect::Killed { id, .. } = e {
                killed_now.insert(*id);
            }
            match e {
                Effect::Shot { .. } => g.shots += 1,
                Effect::Killed { .. } => g.killed += 1,
                Effect::GasHit { .. } => g.gas_hits += 1,
                Effect::Consumed { .. } => g.ground += 1,
                _ => {}
            }
            tally.effect(e);
            if let Effect::Killed { id, .. } = e
                && dom_watch.contains(id)
            {
                println!(
                    "  object {} killed at tick {} (sweep seen {})",
                    id.0,
                    w.tick,
                    swept_by.is_some()
                );
            }
            if let Effect::Domination {
                event: glasnost_sim::domination::DomEvent::Started,
                ..
            } = e
                && let Some(d) = w.domination.current.as_ref()
            {
                let (line, watch) = dominator_launch_line(w, d);
                println!("{line}");
                dom_watch = watch;
            }
            if let Effect::Domination {
                event: glasnost_sim::domination::DomEvent::Sweep,
                house,
                ..
            } = e
            {
                swept_by = *house;
            }
        }
        dom_watch.retain(|&id| {
            let gone = w.obj(id).is_none_or(|o| o.health <= 0);
            if gone && swept_by.is_none() {
                println!("  object {} gone at tick {} before the sweep", id.0, w.tick);
            }
            !gone || swept_by.is_some()
        });
        if let Some(h) = swept_by.take() {
            println!(
                "dominator sweep tick {} house {}: owner changes {}",
                w.tick,
                h.0,
                tally.dominator_sweeps.last().copied().unwrap_or(0)
            );
            for id in std::mem::take(&mut dom_watch) {
                println!("  {}", dominator_watch_line(w, id, h));
            }
        }
        drained = w.effects.len();
        g.r8.observe(w, &killed_now);
        tally.observe(w);
        occasions(w, &mut g.occasions);
        features(w, &mut g.features);
        if econ_every > 0 {
            econ.observe(w);
            if w.tick % econ_every == 0 {
                print!("{}", econ.report(w));
            }
        }
        if sw_probe > 0 && w.tick % sw_probe == 0 {
            print!("{}", sw_probe_line(w));
        }
        if gun_probe > 0 && w.tick % gun_probe == 0 {
            for h in 0..PLAYERS {
                let glasnost_ai::gunner::GunnerProbe {
                    armed,
                    gunners,
                    carrying,
                    empty_home,
                    healers,
                    free,
                } = glasnost_ai::gunner::probe(w, HouseId(h as u8));
                if gunners > 0 {
                    println!(
                        "gunprobe tick {} house {h} armed {armed} gunners {gunners} carrying {carrying} empty-idle-home {empty_home} healers {healers} free-home {free}",
                        w.tick
                    );
                }
            }
        }
        expansion_and_superweapons(g, w);
        if aa_probe > 0 && w.tick % aa_probe == 0 {
            for (h, p) in game.antiair_probes() {
                if p.drainers.is_empty() && p.draining_on_ground == 0 {
                    continue;
                }
                println!("{}", aa_probe_line(w, h, &p));
            }
        }
        if ex_probe > 0 && w.tick % ex_probe == 0 {
            for (h, p) in game.expansion_probes() {
                let blocked = p.blocked_by();
                if let Some(m) = g.expansion_blocks.get_mut(usize::from(h.0)) {
                    for b in &blocked {
                        *m.entry((*b).to_string()).or_default() += 1;
                    }
                    if blocked.is_empty() {
                        *m.entry("none".to_string()).or_default() += 1;
                    }
                }
                println!(
                    "exprobe tick {} house {}: yards {} units {} (idle {}) lined {} buildable {} (yard vehicle types {}, fewest unmet prerequisites {:?}, unblocking building {:?}) cost {} credits {} explored {} worked {} own-half unworked {} field {} harvesters-kept {} intruders {} selling {} paying {} economy-kept {} outnumbered {} losing {} saving {} superweapon building {} (owed {}, wanted {}) income {:?} intruders by {:?} losing at home {}; blocked by {:?}",
                    w.tick,
                    h.0,
                    p.yards,
                    p.units,
                    p.idle_units,
                    p.lined,
                    p.buildable,
                    p.yard_types,
                    p.unmet,
                    p.unblock,
                    p.cost,
                    p.credits,
                    p.explored,
                    p.worked,
                    p.own_half,
                    p.field,
                    p.harvesters_kept,
                    p.intruders,
                    p.selling,
                    p.paying,
                    p.economy_kept,
                    p.outnumbered,
                    p.losing,
                    p.saving,
                    p.superweapon_building,
                    p.superweapon_owed,
                    p.superweapon_wanted,
                    p.income,
                    p.intruders_by,
                    p.losing_at_home,
                    blocked
                );
            }
        }
        if eco_probe > 0 {
            cycles.observe(w);
            if w.tick % glasnost_ai::scripted::THINK_PERIOD == 0 {
                field_spells.observe(w.tick, game.brains(), w);
            }
            if w.tick % BROKE_SAMPLE == 0 {
                broke_spells.observe(w, &game.broke_probes());
            }
            if w.tick % eco_probe == 0 {
                for line in game.economy_probes() {
                    println!("{line}");
                }
            }
        }
        if trace > 0 && w.tick % trace == 0 {
            print!("{}", timeline(w));
        }
        if let Some((every, from, to)) = hash_at
            && ((every > 0 && w.tick % every == 0) || (from..=to).contains(&w.tick))
        {
            println!("hash tick {} {}", w.tick, save::state_hash(w));
        }
        if let Some((from, to)) = objs_at
            && (from..=to).contains(&w.tick)
        {
            for o in w.objects.values() {
                println!("objs tick {} {o:?}", w.tick);
            }
            println!("objs tick {} spawned {:?}", w.tick, w.spawned);
            let credits: Vec<i32> = w.houses.iter().map(|h| h.credits).collect();
            println!("objs tick {} credits {credits:?}", w.tick);
            println!("objs tick {} rng {:?}", w.tick, w.rng);
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
            if let Some(vs) = watch_vs {
                println!(
                    "{}",
                    watch_vs_line(w, o.id, glasnost_sim::world::ObjectId(vs))
                );
            }
        }
        if dump_at.contains(&w.tick) {
            print!("{}", dump(w));
        }
        if stale_every > 0 && w.tick % stale_every == 0 {
            print!("{}", stale(w));
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
    if eco_probe > 0 {
        print!("{}", cycles.report());
        print!("{}", field_spells.report(game.world.tick));
        print!("{}", broke_spells.report());
    }
    g.loads = tally.loads;
    g.hover_loads = tally.hover_loads;
    g.unloads = tally.unloads;
    g.launches = tally.launches;
    g.launches_by_kind = tally.launches_by_kind;
    g.berserked = tally.berserked;
    g.intent_ticks = tally.intent_ticks;
    g.sensors_max = tally.sensors_max;
    g.berserk_armed = tally.berserk_armed.map(|s| s.len());
    g.taken = tally.taken;
    g.controlled_max = tally.controlled_max;
    g.controllers = tally.controllers.map(|s| s.len());
    g.overloads = tally.overloads;
    g.unlimited_max = tally.unlimited_max;
    g.clones = tally.clones;
    g.masters_max = tally.masters_max;
    g.slaves_max = tally.slaves_max;
    g.top_ticks = tally.top_ticks;
    g.gattling_types = tally.gattling_types.map(|s| s.len());
    g.clouds = tally.clouds;
    g.gas_max = tally.gas_max;
    g.systems_by_kind = tally.systems_by_kind;
    g.emitters = tally.emitters;
    g.particles_max = tally.particles_max;
    g.virus_armed = tally.virus_armed.map(|s| s.len());
    g.absorbed = tally.absorbed;
    g.absorber_releases = tally.absorber_releases;
    g.passengers_max = tally.passengers_max;
    g.absorbers = tally.absorbers.map(|s| s.len());
    g.grinders = tally.grinders.map(|s| s.len());
    g.bunkered = tally.bunkered;
    g.bunker_releases = tally.bunker_releases;
    g.bunkers = tally.bunkers.map(|s| s.len());
    g.seized = tally.seized;
    g.dropped = tally.dropped;
    g.walkers = tally.walkers;
    g.disc_shots = tally.disc_shots;
    g.disc_hits = tally.disc_hits;
    g.disc_cancelled = tally.disc_cancelled;
    g.drain_links = tally.drain_links;
    g.drain_ends = tally.drain_ends;
    g.credits_drained = tally.credits_drained;
    g.drained_ticks = tally.drained_ticks;
    g.mutator_launches = tally.mutator_launches;
    g.conversions = tally.conversions;
    g.infantry_mutated = tally.infantry_mutated;
    g.dominator_launches = tally.dominator_launches;
    g.dominated = tally.dominated;
    g.dominator_damaged = tally.dominator_damaged;
    g.dominator_killed = tally.dominator_killed;
    g.dominator_sweeps = tally.dominator_sweeps.clone();
    g.ready_ticks = tally.ready_ticks;
    g.granted_ticks = tally.granted_ticks;
    g.suspended_causes = tally.suspended_causes;
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

/// Per player house after each tick (M8 steps B1 and B2): its construction
/// yards, from when it can build a yard vehicle (sampled every
/// [`YARD_UNIT_SAMPLE`] ticks), and the first tick each superweapon kind
/// was granted and stood ready (not suspended: what the sidebar shows).
fn expansion_and_superweapons(g: &mut Game, w: &World) {
    for h in 0..PLAYERS {
        let house = Some(HouseId(h as u8));
        let yards = w
            .objects
            .values()
            .filter(|o| o.on_map && o.health > 0 && o.owner == house)
            .filter(|o| glasnost_ai::scripted::is_yard(w, o))
            .count();
        g.yards_max[h] = g.yards_max[h].max(yards);
        if yards >= 2 && g.two_yards_at[h].is_none() {
            g.two_yards_at[h] = Some(w.tick);
        }
        if g.yard_unit_from[h].is_none()
            && w.tick % YARD_UNIT_SAMPLE == 0
            && !glasnost_ai::brain::yard_units(w, HouseId(h as u8)).is_empty()
        {
            g.yard_unit_from[h] = Some(w.tick);
        }
        for (i, t) in w.rules().superweapons.types.iter().enumerate() {
            let Some(r) = glasnost_sim::superweapons::charge::get(w, HouseId(h as u8), i as u32)
            else {
                continue;
            };
            let kind = t.kind.map_or("no kind".to_string(), |k| format!("{k:?}"));
            if r.granted {
                g.first_granted[h].entry(kind.clone()).or_insert(w.tick);
                if r.ready && !r.suspended {
                    g.first_ready[h].entry(kind).or_insert(w.tick);
                }
            }
        }
    }
}

/// Ticks between the lane's samples of whether a house can build a yard
/// vehicle.
const YARD_UNIT_SAMPLE: i32 = 100;

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
    /// House-ticks a superweapon record stood granted, and granted but
    /// suspended, by kind name: a kind granted long and never ready names
    /// a charge the house never finishes.
    pub granted_ticks: BTreeMap<String, (usize, usize)>,
    /// The suspended house-ticks of [`Tally::granted_ticks`] by cause: a
    /// timed blackout running (S-0405 claim 6, a force-shield launch or a
    /// spy), a drain blackout (claim 5), or a true shortage of power.
    pub suspended_causes: BTreeMap<String, [usize; 3]>,
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
    /// Takeovers (S-0601 claim 6: an object controlled now that was not
    /// controlled after the last tick, or by another controller).
    pub taken: usize,
    /// Per player house: the most objects its controllers held at once.
    pub controlled_max: [usize; PLAYERS],
    /// Per player house: its controllers (objects with a control record)
    /// seen so far.
    pub controllers: [BTreeSet<u32>; PLAYERS],
    /// Overload pulses that played the overload sound (S-0601 claim 16).
    pub overloads: usize,
    /// Per player house: the most victims one unlimited controller held.
    pub unlimited_max: [usize; PLAYERS],
    /// Per player house: infantry copies released at a cloning building
    /// (S-0404 claim 14: a new infantry unit in contact with a cloning
    /// building when first seen).
    pub clones: [usize; PLAYERS],
    /// Per player house: the most slave miners and the most slaves on the
    /// map at one tick (S-0402).
    pub masters_max: [usize; PLAYERS],
    /// Per player house: the most slaves on the map at one tick.
    pub slaves_max: [usize; PLAYERS],
    /// The infantry seen so far.
    pub infantry_seen: BTreeSet<u32>,
    /// Per player house: the ticks in which at least one of its objects
    /// stood at its type's top gattling stage (S-0609 claim 2).
    pub top_ticks: [usize; PLAYERS],
    /// Per player house: the gattling types (kind code, type index) it had
    /// an object of (S-0609 Terms).
    pub gattling_types: [BTreeSet<(u8, u32)>; PLAYERS],
    /// Gas clouds that appeared (S-0613 claims 2 to 4): warhead-made
    /// particle systems and virus-death animations.
    pub clouds: usize,
    /// The most gas particles in the world at one tick (S-0613).
    pub gas_max: usize,
    /// Per player house: its units whose first weapon kills with the virus
    /// death (infantry-death kind 8, S-0107 claim 17) seen so far.
    pub virus_armed: [BTreeSet<u32>; PLAYERS],
    /// Entries into absorbing buildings (S-0605 claims 7 and 8: a
    /// (passenger, building) pair now that was not there after the last
    /// tick).
    pub absorbed: usize,
    /// Passengers that came out of an absorbing building still standing
    /// (S-0605 claim 13).
    pub absorber_releases: usize,
    /// Per player house: the most passengers its absorbing buildings held
    /// at once.
    pub passengers_max: [usize; PLAYERS],
    /// Per player house: its absorbing buildings seen so far.
    pub absorbers: [BTreeSet<u32>; PLAYERS],
    /// Per player house: its recycling buildings seen so far (S-0603).
    pub grinders: [BTreeSet<u32>; PLAYERS],
    /// Vehicles that became bunkered (S-0606 claim 7: a (vehicle, bunker)
    /// link now that was not there after the last tick).
    pub bunkered: usize,
    /// Bunker links that ended with the vehicle and the bunker still
    /// standing (S-0606 claims 12 and 13).
    pub bunker_releases: usize,
    /// Per player house: its tank bunkers seen so far (S-0606).
    pub bunkers: [BTreeSet<u32>; PLAYERS],
    /// Units that became seized (S-0607 claim 5).
    pub seized: usize,
    /// Seized units that became dropped (S-0607 claim 13).
    pub dropped: usize,
    /// Burning walkers that appeared (S-0615 claim 4).
    pub walkers: usize,
    /// Disc-laser shots (S-0615 claim 12).
    pub disc_shots: usize,
    /// Disc lasers that reached their final step and detonated (S-0615
    /// claim 16).
    pub disc_hits: usize,
    /// Disc lasers cancelled by the re-check (S-0615 claim 14).
    pub disc_cancelled: usize,
    /// Drain links that formed (S-0608 claim 5).
    pub drain_links: usize,
    /// Drain links that ended (S-0608 claim 14).
    pub drain_ends: usize,
    /// Credits that drain payouts moved (S-0608 claims 7 to 10).
    pub credits_drained: i64,
    /// The sum over ticks of the buildings drained at the tick's end
    /// (S-0608 claim 1).
    pub drained_ticks: usize,
    /// Genetic mutator launches (S-0611 claim 2: the activation sound).
    pub mutator_launches: usize,
    /// Infantry-making animations started (S-0611 claims 8 and 9: the
    /// conversion animations and any other).
    pub conversions: usize,
    /// Infantry made by a conversion animation's end (S-0611 claims 9 to
    /// 12).
    pub infantry_mutated: usize,
    /// Psychic dominator launches (S-0612 claim 1: the activation sound).
    pub dominator_launches: usize,
    /// Objects a sweep took over (S-0612 claim 14).
    pub dominated: usize,
    /// Objects a sweep's area damage hurt and that survived it (S-0612
    /// claim 9).
    pub dominator_damaged: usize,
    /// Objects a sweep's area damage destroyed (S-0612 claim 9).
    pub dominator_killed: usize,
    /// Per sweep (S-0612 claim 8), in order: the objects it took over that
    /// changed owner (S-0612 claim 14: a takeover of the dominating house's
    /// own object changes no owner and is not counted here).
    pub dominator_sweeps: Vec<usize>,
    /// The owner of every object while a domination is before its sweep
    /// (read at the end of the last tick), to tell an owner change.
    dom_owners: BTreeMap<u32, Option<HouseId>>,
    /// The burning walkers after the last tick.
    walkers_before: BTreeSet<u32>,
    /// The seized units after the last tick.
    seized_before: BTreeSet<u32>,
    /// The dropped units after the last tick.
    dropped_before: BTreeSet<u32>,
    /// The (vehicle, bunker) links after the last tick.
    bunkered_before: BTreeSet<(u32, u32)>,
    /// The (passenger, building) pairs after the last tick.
    absorbed_before: BTreeSet<(u32, u32)>,
    /// The gas systems and animations after the last tick.
    clouds_before: BTreeSet<u32>,
    /// Particle systems that appeared, by behaviour kind (S-0616).
    pub systems_by_kind: BTreeMap<String, usize>,
    /// Effect-weapon emitters made, by place: fire, spark, railgun
    /// (S-0616 claim 1; a spark place also counts damage sparks).
    pub emitters: [usize; 3],
    /// The most particles of every kind in the world at one tick.
    pub particles_max: usize,
    /// The particle systems after the last tick.
    systems_before: BTreeSet<u32>,
    /// The (holder, system) emitter pairs after the last tick.
    emitters_before: BTreeSet<(u8, u32, u32)>,
    /// The (victim, controller) pairs after the last tick.
    victims_before: BTreeSet<(u32, u32)>,
    /// The units berserk after the last tick.
    berserk_before: BTreeSet<u32>,
    /// The (house, superweapon type) pairs ready after the last tick.
    ready_before: BTreeSet<(usize, usize)>,
    /// The (passenger, transport) pairs of the last tick.
    riding: BTreeSet<(u32, u32)>,
}

impl Tally {
    /// Counts one new effect of the world's last tick (the disc-laser
    /// events, S-0615 claims 12, 14 and 16).
    pub fn effect(&mut self, e: &Effect) {
        use glasnost_sim::combat::effects::DiscEvent;
        use glasnost_sim::drain::DrainEvent;
        use glasnost_sim::mutation::MakerEvent;
        match e {
            Effect::Mind {
                sound: glasnost_sim::mind::MindSound::Overload,
                ..
            } => self.overloads += 1,
            Effect::Superweapon {
                event: glasnost_sim::superweapons::SwEvent::GeneticSound,
                ..
            } => self.mutator_launches += 1,
            Effect::Maker {
                event: MakerEvent::Made,
                ..
            } => self.conversions += 1,
            Effect::Maker {
                event:
                    MakerEvent::Ended {
                        unit: Some(_),
                        conversion: true,
                    },
                ..
            } => self.infantry_mutated += 1,
            Effect::Superweapon {
                event: glasnost_sim::superweapons::SwEvent::DominatorSound,
                ..
            } => self.dominator_launches += 1,
            Effect::Domination { event, house, .. } => match event {
                glasnost_sim::domination::DomEvent::Sweep => self.dominator_sweeps.push(0),
                glasnost_sim::domination::DomEvent::Taken(id) => {
                    self.dominated += 1;
                    let before = self.dom_owners.get(&id.0).copied().flatten();
                    if before.is_some()
                        && before != *house
                        && let Some(n) = self.dominator_sweeps.last_mut()
                    {
                        *n += 1;
                    }
                }
                glasnost_sim::domination::DomEvent::Damage { damaged, killed } => {
                    self.dominator_damaged += *damaged as usize;
                    self.dominator_killed += *killed as usize;
                }
                _ => {}
            },
            _ => {}
        }
        if let Effect::Drain { event, .. } = e {
            match event {
                DrainEvent::Formed { .. } => self.drain_links += 1,
                DrainEvent::Ended => self.drain_ends += 1,
                DrainEvent::Paid { amount, .. } => self.credits_drained += i64::from(*amount),
            }
        }
        if let Effect::Disc { event, .. } = e {
            match event {
                DiscEvent::Shot => self.disc_shots += 1,
                DiscEvent::Hit { .. } => self.disc_hits += 1,
                DiscEvent::Cancelled => self.disc_cancelled += 1,
                DiscEvent::ChargeUp { .. } => {}
            }
        }
    }

    /// Counts what changed in the world's last tick.
    pub fn observe(&mut self, w: &World) {
        self.drained_ticks += w.drain.links.len();
        self.dom_owners.clear();
        if w.domination
            .current
            .as_ref()
            .is_some_and(|d| matches!(d.phase, glasnost_sim::domination::Phase::LeadIn { .. }))
        {
            self.dom_owners = w.objects.values().map(|o| (o.id.0, o.owner)).collect();
        }
        let mut masters = [0usize; PLAYERS];
        let mut slaves = [0usize; PLAYERS];
        for o in w.objects.values().filter(|o| o.on_map && o.health > 0) {
            let Some(h) = o.owner.map(|h| usize::from(h.0)).filter(|&h| h < PLAYERS) else {
                continue;
            };
            masters[h] += usize::from(glasnost_sim::econ::slaves::is_master(w, o.id));
            slaves[h] += usize::from(o.econ.slave.is_some());
        }
        for h in 0..PLAYERS {
            self.masters_max[h] = self.masters_max[h].max(masters[h]);
            self.slaves_max[h] = self.slaves_max[h].max(slaves[h]);
        }
        for o in w
            .objects
            .values()
            .filter(|o| o.on_map && o.kind == Kind::Infantry)
        {
            if !self.infantry_seen.insert(o.id.0) {
                continue;
            }
            let cloner = o.contact.and_then(|k| w.obj(k)).is_some_and(|b| {
                b.kind == Kind::Building
                    && w.rules()
                        .buildings
                        .get(b.type_index as usize)
                        .is_some_and(|t| t.raw.econ.cloning == Some(true))
            });
            if let Some(h) = o.owner.map(|h| usize::from(h.0)).filter(|&h| h < PLAYERS)
                && cloner
            {
                self.clones[h] += 1;
            }
        }
        let walkers: BTreeSet<u32> = w.walkers.keys().map(|id| id.0).collect();
        self.walkers += walkers.difference(&self.walkers_before).count();
        self.walkers_before = walkers;
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
        // count each such fall of the ready mark to not ready (a lost giving
        // building also clears the grant; low power suspends a record that
        // stays ready, which is no launch).
        let ready_now: BTreeSet<(usize, usize)> = (0..PLAYERS)
            .flat_map(|h| (0..w.rules().superweapons.types.len()).map(move |i| (h, i)))
            .filter(|&(h, i)| {
                glasnost_sim::superweapons::charge::get(w, HouseId(h as u8), i as u32)
                    .is_some_and(|r| r.granted && r.ready && !r.suspended)
            })
            .collect();
        for &(h, i) in self.ready_before.difference(&ready_now) {
            if glasnost_sim::superweapons::charge::get(w, HouseId(h as u8), i as u32)
                .is_some_and(|r| r.granted && !r.ready)
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
        for h in 0..PLAYERS {
            for i in 0..w.rules().superweapons.types.len() {
                let Some(r) =
                    glasnost_sim::superweapons::charge::get(w, HouseId(h as u8), i as u32)
                else {
                    continue;
                };
                if r.granted {
                    let kind = w.rules().superweapons.types[i].kind;
                    let e = self
                        .granted_ticks
                        .entry(kind.map_or("no kind".to_string(), |k| format!("{k:?}")))
                        .or_default();
                    e.0 += 1;
                    e.1 += usize::from(r.suspended);
                    if r.suspended {
                        let house = HouseId(h as u8);
                        let cause = if glasnost_sim::econ::power::blackout_running(w, house) {
                            0
                        } else if w.houses[h].econ.power.drain_blackout {
                            1
                        } else {
                            2
                        };
                        self.suspended_causes
                            .entry(kind.map_or("no kind".to_string(), |k| format!("{k:?}")))
                            .or_default()[cause] += 1;
                    }
                }
            }
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
        let victims: BTreeSet<(u32, u32)> = w
            .mind
            .victims
            .iter()
            .map(|(v, r)| (v.0, r.controller.0))
            .collect();
        self.taken += victims.difference(&self.victims_before).count();
        self.victims_before = victims;
        let mut held = [0usize; PLAYERS];
        for (&c, r) in &w.mind.controllers {
            if let Some(h) = w.obj(c).and_then(|o| o.owner)
                && usize::from(h.0) < PLAYERS
            {
                held[usize::from(h.0)] += r.victims.len();
                self.controllers[usize::from(h.0)].insert(c.0);
                if r.unlimited {
                    let m = &mut self.unlimited_max[usize::from(h.0)];
                    *m = (*m).max(r.victims.len());
                }
            }
        }
        for (m, n) in self.controlled_max.iter_mut().zip(held) {
            *m = (*m).max(n);
        }
        let mut top = [false; PLAYERS];
        for o in w.objects.values() {
            let Some(h) = o.owner.map(|h| usize::from(h.0)).filter(|&h| h < PLAYERS) else {
                continue;
            };
            if !glasnost_sim::gattling::flagged(w, o.id) {
                continue;
            }
            self.gattling_types[h].insert((kind_code(o.kind), o.type_index));
            top[h] |= glasnost_sim::gattling::at_top(w, o.id);
        }
        for (t, at) in self.top_ticks.iter_mut().zip(top) {
            *t += usize::from(at);
        }
        self.intent_ticks += (0..PLAYERS)
            .filter(|&h| !glasnost_sim::psychic::intent_lines(w, HouseId(h as u8)).is_empty())
            .count();
        let clouds: BTreeSet<u32> = w
            .gas
            .systems
            .keys()
            .copied()
            .filter(|&id| glasnost_sim::gas::is_cloud(w, id))
            .chain(w.gas.anims.keys().copied())
            .map(|id| id.0)
            .collect();
        self.clouds += clouds.difference(&self.clouds_before).count();
        self.clouds_before = clouds;
        self.gas_max = self.gas_max.max(glasnost_sim::gas::particles(w).count());
        let systems: BTreeSet<u32> = w.gas.systems.keys().map(|id| id.0).collect();
        for &id in systems.difference(&self.systems_before) {
            let kind = w.gas.systems[&glasnost_sim::world::ObjectId(id)].type_index as usize;
            let kind = w.rules().gas.systems[kind]
                .kind
                .map_or("none".to_string(), |k| format!("{k:?}").to_lowercase());
            *self.systems_by_kind.entry(kind).or_default() += 1;
        }
        self.systems_before = systems;
        let emitters: BTreeSet<(u8, u32, u32)> = w
            .objects
            .values()
            .flat_map(|o| {
                let c = &o.combat;
                [
                    c.fire_emitter.map(|e| (0, o.id.0, e.system.0)),
                    c.spark_emitter.map(|s| (1, o.id.0, s.0)),
                    c.railgun_emitter.map(|s| (2, o.id.0, s.0)),
                ]
            })
            .flatten()
            .collect();
        for &(place, _, _) in emitters.difference(&self.emitters_before) {
            self.emitters[usize::from(place)] += 1;
        }
        self.emitters_before = emitters;
        let all: usize = w.gas.systems.values().map(|s| s.particles.len()).sum();
        self.particles_max = self.particles_max.max(all);
        let r = w.rules();
        for o in w.objects.values() {
            let Some(h) = o.owner.map(|h| usize::from(h.0)).filter(|&h| h < PLAYERS) else {
                continue;
            };
            let virus = r
                .unit(o.kind, o.type_index)
                .and_then(|u| u.combat.slots[0])
                .and_then(|p| r.combat.weapon(p).warhead)
                // spec: S-0107 claim 17 (infantry-death kind 8, the virus death)
                .is_some_and(|wh| r.combat.warhead(wh).inf_death == 8);
            if virus && o.kind != glasnost_sim::rules::Kind::Building {
                self.virus_armed[h].insert(o.id.0);
            }
        }
        let absorbed: BTreeSet<(u32, u32)> = w
            .objects
            .values()
            .filter_map(|o| Some((o.id.0, o.structure.absorbed_in?.0)))
            .collect();
        self.absorbed += absorbed.difference(&self.absorbed_before).count();
        self.absorber_releases += self
            .absorbed_before
            .difference(&absorbed)
            .filter(|&&(p, b)| {
                let on = |i| {
                    w.obj(glasnost_sim::world::ObjectId(i))
                        .is_some_and(|o| o.on_map && o.health > 0)
                };
                on(p) && on(b)
            })
            .count();
        self.absorbed_before = absorbed;
        let links: BTreeSet<(u32, u32)> = w
            .objects
            .values()
            .filter_map(|o| Some((o.id.0, o.structure.bunkered_in?.0)))
            .collect();
        self.bunkered += links.difference(&self.bunkered_before).count();
        self.bunker_releases += self
            .bunkered_before
            .difference(&links)
            .filter(|&&(v, b)| {
                let on = |i| {
                    w.obj(glasnost_sim::world::ObjectId(i))
                        .is_some_and(|o| o.on_map && o.health > 0)
                };
                on(v) && on(b)
            })
            .count();
        self.bunkered_before = links;
        let seized: BTreeSet<u32> = w
            .objects
            .values()
            .filter(|o| glasnost_sim::lift::seized(w, o.id))
            .map(|o| o.id.0)
            .collect();
        self.seized += seized.difference(&self.seized_before).count();
        self.seized_before = seized;
        let dropped: BTreeSet<u32> = w
            .objects
            .values()
            .filter(|o| glasnost_sim::lift::dropped(w, o.id))
            .map(|o| o.id.0)
            .collect();
        self.dropped += dropped.difference(&self.dropped_before).count();
        self.dropped_before = dropped;
        let mut held = [0usize; PLAYERS];
        for o in w.objects.values() {
            let Some(h) = o.owner.map(|h| usize::from(h.0)).filter(|&h| h < PLAYERS) else {
                continue;
            };
            if glasnost_sim::structures::absorb::is_absorber(w, o.id) {
                self.absorbers[h].insert(o.id.0);
                held[h] += glasnost_sim::structures::absorb::count(w, o.id) as usize;
            }
            if glasnost_sim::structures::grinder::is_recycler(w, o.id) {
                self.grinders[h].insert(o.id.0);
            }
            if glasnost_sim::structures::bunker::is_bunker(w, o.id) {
                self.bunkers[h].insert(o.id.0);
            }
        }
        for (m, n) in self.passengers_max.iter_mut().zip(held) {
            *m = (*m).max(n);
        }
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
/// The watch's line against a second object (`GLASNOST_AI_WATCH_VS`): the
/// slot and weapon the watched object would use, the firing check with
/// every reason and the own AI's lasting test (both without range), the
/// target's health and owner, and the watched object's infantry sequence.
fn watch_vs_line(
    w: &World,
    id: glasnost_sim::world::ObjectId,
    vs: glasnost_sim::world::ObjectId,
) -> String {
    use glasnost_sim::combat::{check, weapon};
    let t = Some(glasnost_sim::world::Target::Object(vs));
    let (slot, wp) = weapon::would_use(w, id, t);
    let every = check::check(w, id, t, slot, false);
    let lasting = check::check_lasting(w, id, t, slot);
    let seq = w
        .obj(id)
        .and_then(|o| o.infantry.as_ref())
        .and_then(|s| s.seq);
    let (hp, owner) = w.obj(vs).map_or((0, None), |o| (o.health, o.owner));
    format!(
        "  vs {}: slot {slot} weapon {:?} check {every:?} lasting {lasting:?} target hp {hp} owner {owner:?} seq {seq:?}",
        vs.0, wp
    )
}

/// A diagnostic list of the player houses' objects that hold a removed
/// object as destination or target, and of rocket missiles on the map whose
/// flight has not started (`GLASNOST_AI_STALE=n`, every n ticks; M8 step A4
/// round 9): one line each, with the object's state.
fn stale(w: &World) -> String {
    use glasnost_sim::world::Target;
    let mut s = String::new();
    let gone = |t: Option<Target>| matches!(t, Some(Target::Object(k)) if w.obj(k).is_none());
    for o in w
        .objects
        .values()
        .filter(|o| o.owner.is_some_and(|h| usize::from(h.0) < PLAYERS))
    {
        let rocket_waits = o.on_map && o.rocket.as_ref().is_some_and(|r| r.phase.is_none());
        if !(gone(o.dest) || gone(o.target) || rocket_waits) {
            continue;
        }
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
            "stale tick {} obj {} {:?} {name} house {:?} at ({},{}) z {} hp {} map {} beh {:?}/{:?} due {} dest {:?}{} target {:?}{} rocket waits {rocket_waits}",
            w.tick,
            o.id.0,
            o.kind,
            o.owner.map(|h| h.0),
            c.x,
            c.y,
            o.pos.z,
            o.health,
            o.on_map,
            o.beh.current,
            o.beh.queued,
            o.beh.due(w.tick),
            o.dest,
            if gone(o.dest) { " (gone)" } else { "" },
            o.target,
            if gone(o.target) { " (gone)" } else { "" },
        );
    }
    s
}

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
                    Some(K::Genetic) => "ready: genetic mutator",
                    Some(K::Dominator) => "ready: psychic dominator",
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
                        Some(K::Genetic) => "building gives: genetic mutator",
                        Some(K::Dominator) => "building gives: psychic dominator",
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

/// The spells of `GLASNOST_AI_ECOPROBE` (M8 step B9) in which a house's
/// plan wants the refinery by a new field next
/// (`glasnost_ai::brain::Brain::wants_field_refinery`), sampled every
/// think period: per house, the first and last tick of each spell, and
/// the stalls within them: samples at which the want held while the
/// house had no item on a building line, so that nothing was being built
/// for it (or for anything else).
#[derive(Default)]
struct FieldSpells {
    /// Per house: the open spell's first and last tick.
    open: BTreeMap<u8, (i32, i32)>,
    /// Per house: the closed spells.
    done: BTreeMap<u8, Vec<(i32, i32)>>,
    /// Per house: the open stall's first and last tick.
    stall: BTreeMap<u8, (i32, i32)>,
    /// Per house: the stalled samples and the longest stall in ticks.
    stalls: BTreeMap<u8, (u32, i32)>,
}

/// A spell longer than this many ticks is listed (M8 step B9's count).
const LONG_SPELL: i32 = 3000;

impl FieldSpells {
    fn observe(&mut self, tick: i32, brains: &[glasnost_ai::brain::Brain], w: &World) {
        for b in brains {
            let h = b.house().0;
            let wants = b.wants_field_refinery(w);
            if wants {
                self.open.entry(h).or_insert((tick, tick)).1 = tick;
            } else if let Some(s) = self.open.remove(&h) {
                self.done.entry(h).or_default().push(s);
            }
            // An empty line ceases to exist (S-0404 claim 1): the building
            // line is idle when the house has none with an item.
            let idle = !w.econ.lines.lines.iter().any(|l| {
                l.house == b.house()
                    && l.kind == glasnost_sim::econ::production::LineKind::Building
                    && l.item.is_some()
            });
            if wants && idle {
                let s = *self.stall.entry(h).or_insert((tick, tick));
                self.stall.insert(h, (s.0, tick));
                let e = self.stalls.entry(h).or_default();
                e.0 += 1;
                e.1 = e.1.max(tick - s.0);
            } else {
                self.stall.remove(&h);
            }
        }
    }

    fn report(&mut self, end: i32) -> String {
        for (h, s) in std::mem::take(&mut self.open) {
            self.done.entry(h).or_default().push((s.0, s.1.max(end)));
        }
        let mut out = String::new();
        for (h, spells) in &self.done {
            let long: Vec<&(i32, i32)> = spells.iter().filter(|s| s.1 - s.0 > LONG_SPELL).collect();
            let _ = writeln!(
                out,
                "fieldspell house {h}: {} spells, {} ticks wanted, longest {}; stalled samples {}, longest stall {}; over {LONG_SPELL} ticks: {} {long:?}",
                spells.len(),
                spells.iter().map(|s| s.1 - s.0 + 1).sum::<i32>(),
                spells.iter().map(|s| s.1 - s.0).max().unwrap_or(0),
                self.stalls.get(h).map_or(0, |s| s.0),
                self.stalls.get(h).map_or(0, |s| s.1),
                long.len()
            );
        }
        out
    }
}

/// How often the broke probe samples (M8 step B10), in ticks.
const BROKE_SAMPLE: i32 = 60;

/// The spells of `GLASNOST_AI_ECOPROBE` (M8 step B10) in which a house
/// waits broke for an economy item (`glasnost_ai::brain::BrokeProbe`:
/// a lost refinery, a refinery by a new field or a harvester, with less
/// money in hand than it still owes), with what runs beside it on the
/// lines; the spells in which a house owns only its construction yard
/// with nothing on its lines; and the game's bridge links (S-0212 claim
/// 3) as they are enabled and disabled.
#[derive(Default)]
struct BrokeSpells {
    /// Per house: the open broke spell.
    open: BTreeMap<u8, BrokeSpell>,
    /// Per house: the closed broke spells.
    done: BTreeMap<u8, Vec<BrokeSpell>>,
    /// Per house: the open yard-only spell.
    yard_open: BTreeMap<u8, YardSpell>,
    /// Per house: the closed yard-only spells.
    yard_done: BTreeMap<u8, Vec<YardSpell>>,
    /// The enabled links at the first sample and each change after it.
    links: Vec<(i32, usize, usize)>,
}

#[derive(Clone, Debug)]
struct BrokeSpell {
    from: i32,
    to: i32,
    wait: glasnost_ai::brain::EconomyWait,
    /// Samples by what runs beside the wait: plant, superweapon, both, neither.
    behind: [u32; 4],
    paying: u32,
    saving: u32,
    selling: u32,
    samples: u32,
    credits: i64,
    income: Option<i64>,
}

#[derive(Clone, Debug)]
struct YardSpell {
    from: i32,
    to: i32,
    credits: i64,
    most: i64,
    selling: bool,
    harvesters: usize,
    army: usize,
    refinery: i64,
    plant: i64,
    wanted: Option<Role>,
}

impl BrokeSpells {
    fn observe(&mut self, w: &World, probes: &[(HouseId, glasnost_ai::brain::BrokeProbe)]) {
        let tick = w.tick;
        for (h, p) in probes {
            let h = h.0;
            match p.wait.filter(|_| p.broke) {
                Some(wait) => {
                    if self.open.get(&h).is_some_and(|s| s.wait != wait)
                        && let Some(s) = self.open.remove(&h)
                    {
                        self.done.entry(h).or_default().push(s);
                    }
                    let s = self.open.entry(h).or_insert(BrokeSpell {
                        from: tick,
                        to: tick,
                        wait,
                        behind: [0; 4],
                        paying: 0,
                        saving: 0,
                        selling: 0,
                        samples: 0,
                        credits: p.credits,
                        income: p.income,
                    });
                    s.to = tick;
                    s.samples += 1;
                    let plant = p.plant_lined && !p.lined;
                    s.behind[match (plant, p.superweapon_lined) {
                        (true, false) => 0,
                        (false, true) => 1,
                        (true, true) => 2,
                        (false, false) => 3,
                    }] += 1;
                    s.paying += u32::from(p.paying);
                    s.saving += u32::from(p.saving);
                    s.selling += u32::from(p.selling);
                }
                None => {
                    if let Some(s) = self.open.remove(&h) {
                        self.done.entry(h).or_default().push(s);
                    }
                }
            }
            if p.yard_only && p.idle {
                let s = self.yard_open.entry(h).or_insert(YardSpell {
                    from: tick,
                    to: tick,
                    credits: p.credits,
                    most: p.credits,
                    selling: p.selling,
                    harvesters: p.harvesters,
                    army: p.army,
                    refinery: p.refinery_price,
                    plant: p.plant_price,
                    wanted: p.wanted,
                });
                s.to = tick;
                s.most = s.most.max(p.credits);
            } else if let Some(s) = self.yard_open.remove(&h) {
                self.yard_done.entry(h).or_default().push(s);
            }
        }
        let enabled = w.cells.links.iter().filter(|l| l.enabled).count();
        if self.links.last().is_none_or(|&(_, _, e)| e != enabled) {
            self.links.push((tick, w.cells.links.len(), enabled));
        }
    }

    fn report(&mut self) -> String {
        for (h, s) in std::mem::take(&mut self.open) {
            self.done.entry(h).or_default().push(s);
        }
        for (h, s) in std::mem::take(&mut self.yard_open) {
            self.yard_done.entry(h).or_default().push(s);
        }
        let mut out = String::new();
        for (h, spells) in &self.done {
            let long: Vec<String> = spells
                .iter()
                .filter(|s| s.to - s.from > LONG_SPELL)
                .map(|s| {
                    format!(
                        "{}-{} {:?} plant/sw/both/neither {:?} paying {} saving {} selling {} of {} credits {} income {:?}",
                        s.from,
                        s.to,
                        s.wait,
                        s.behind,
                        s.paying,
                        s.saving,
                        s.selling,
                        s.samples,
                        s.credits,
                        s.income
                    )
                })
                .collect();
            let _ = writeln!(
                out,
                "brokespell house {h}: {} spells, {} ticks, longest {}; over {LONG_SPELL} ticks: {} {long:?}",
                spells.len(),
                spells
                    .iter()
                    .map(|s| s.to - s.from + BROKE_SAMPLE)
                    .sum::<i32>(),
                spells.iter().map(|s| s.to - s.from).max().unwrap_or(0),
                long.len()
            );
        }
        for (h, spells) in &self.yard_done {
            let _ = writeln!(
                out,
                "yardonly house {h}: {} spells, {} ticks; {:?}",
                spells.len(),
                spells.iter().map(|s| s.to - s.from + BROKE_SAMPLE).sum::<i32>(),
                spells
                    .iter()
                    .map(|s| format!(
                        "{}-{} credits {} most {} selling {} harvesters {} army {} refinery {} plant {} wants {:?}",
                        s.from,
                        s.to,
                        s.credits,
                        s.most,
                        s.selling,
                        s.harvesters,
                        s.army,
                        s.refinery,
                        s.plant,
                        s.wanted
                    ))
                    .collect::<Vec<_>>()
            );
        }
        let _ = writeln!(out, "bridgelinks (tick, links, enabled): {:?}", self.links);
        out
    }
}

/// The harvester cycles of `GLASNOST_AI_ECOPROBE` (M8 step B8): per
/// harvester of a player house (not a slave miner), the ticks at which it
/// starts and ends loading and unloading, and per finished cycle (from one
/// unloading start to the next) the ticks out to the field, loading, back
/// to the refinery and unloading, with the distance in cells from the
/// cell where loading started to the nearest refinery of its house.
#[derive(Default)]
struct CycleTrace {
    /// Per harvester: whether it was loading and unloading at the last
    /// tick, and the ticks of the current cycle's marks.
    now: BTreeMap<glasnost_sim::world::ObjectId, CycleMarks>,
    /// Per house: the finished cycles.
    done: [Vec<Cycle>; PLAYERS],
}

#[derive(Default, Clone, Copy)]
struct CycleMarks {
    loading: bool,
    unloading: bool,
    unload_start: Option<i32>,
    unload_end: Option<i32>,
    load_start: Option<i32>,
    load_end: Option<i32>,
    distance: Option<i64>,
}

#[derive(Clone, Copy)]
struct Cycle {
    out: i32,
    loading: i32,
    back: i32,
    unloading: i32,
    distance: i64,
}

impl CycleTrace {
    fn observe(&mut self, w: &World) {
        use glasnost_sim::econ::harvest::Phase;
        let t = w.tick;
        for o in w.objects.values().filter(|o| o.on_map && o.health > 0) {
            let Some(h) = o.owner.map(|h| usize::from(h.0)).filter(|&h| h < PLAYERS) else {
                continue;
            };
            let Some(hs) = &o.econ.harvest else { continue };
            if glasnost_sim::econ::slaves::is_master(w, o.id) {
                continue;
            }
            let m = self.now.entry(o.id).or_default();
            let loading = hs.phase == Phase::Loading
                && o.beh.current == Some(glasnost_sim::rules::Behaviour::Harvest);
            let unloading = hs.unloading;
            if loading && !m.loading && m.load_start.is_none() {
                m.load_start = Some(t);
                let c = o.pos.cell();
                m.distance = w
                    .objects
                    .values()
                    .filter(|r| {
                        r.on_map
                            && r.owner == o.owner
                            && r.kind == Kind::Building
                            && role_of(w, tech::type_of(r)) == Some(Role::Refinery)
                    })
                    .map(|r| {
                        let (dx, dy) = (
                            i64::from(r.pos.cell().x - c.x),
                            i64::from(r.pos.cell().y - c.y),
                        );
                        (dx * dx + dy * dy).isqrt()
                    })
                    .min();
            }
            if !loading && m.loading {
                m.load_end = Some(t);
            }
            if unloading && !m.unloading {
                if let (Some(us), Some(ue), Some(ls), Some(le), Some(d)) = (
                    m.unload_start,
                    m.unload_end,
                    m.load_start,
                    m.load_end,
                    m.distance,
                ) && us <= ue
                    && ue <= ls
                    && ls <= le
                {
                    self.done[h].push(Cycle {
                        out: ls - ue,
                        loading: le - ls,
                        back: t - le,
                        unloading: ue - us,
                        distance: d,
                    });
                }
                m.unload_start = Some(t);
                m.unload_end = None;
                m.load_start = None;
                m.load_end = None;
            }
            if !unloading && m.unloading {
                m.unload_end = Some(t);
            }
            m.loading = loading;
            m.unloading = unloading;
        }
    }

    fn report(&self) -> String {
        fn median<T: Copy + Ord>(mut v: Vec<T>) -> Option<T> {
            v.sort_unstable();
            v.get(v.len() / 2).copied()
        }
        let mut s = String::new();
        for (h, cs) in self.done.iter().enumerate() {
            let _ = writeln!(
                s,
                "ecoprobe cycles house {h}: {} cycles; median ticks out {:?} loading {:?} back {:?} unloading {:?}; median distance from the refinery {:?}, most {:?}",
                cs.len(),
                median(cs.iter().map(|c| c.out).collect()),
                median(cs.iter().map(|c| c.loading).collect()),
                median(cs.iter().map(|c| c.back).collect()),
                median(cs.iter().map(|c| c.unloading).collect()),
                median(cs.iter().map(|c| c.distance).collect()),
                cs.iter().map(|c| c.distance).max(),
            );
        }
        s
    }
}

/// The lines of `GLASNOST_AI_LINES`: what changed on house `h`'s production
/// lines since the last call (`items`: per line index its last item type
/// and progress). Classes and counts only.
/// The economy trace of `GLASNOST_AI_ECON`: credits earned and spent by
/// category since the last report, per house.
#[derive(Default)]
struct EconTrace {
    last_credits: [i32; PLAYERS],
    earned: [i64; PLAYERS],
    spent: [BTreeMap<&'static str, i64>; PLAYERS],
    /// Per line: its item's object and the balance owed at the last tick.
    owed: BTreeMap<usize, (glasnost_sim::world::ObjectId, i32)>,
    /// Objects lost since the last report, by category.
    lost: [BTreeMap<&'static str, usize>; PLAYERS],
    /// The player houses' objects on the map at the last tick.
    alive: BTreeMap<glasnost_sim::world::ObjectId, (usize, &'static str)>,
    /// Per object, the kinds (and locomotors) of the objects that had it
    /// as their target at the last tick.
    hunted: BTreeMap<glasnost_sim::world::ObjectId, Vec<String>>,
    /// Objects lost since the last report, by category and the kind of an
    /// object that targeted it at the tick before ("none" for no one).
    lost_to: [BTreeMap<String, usize>; PLAYERS],
}

impl EconTrace {
    fn category(w: &World, t: glasnost_sim::econ::TypeRef) -> &'static str {
        if t.kind == Kind::Building {
            let gives = w
                .rules()
                .superweapons
                .building_sw
                .get(t.index as usize)
                .is_some_and(|f| f.iter().any(Option::is_some));
            return match role_of(w, t) {
                Some(Role::Power) => "power",
                Some(Role::Refinery) => "refinery",
                Some(Role::Yard) | Some(Role::Factory(_)) => "factory",
                _ if gives => "superweapon",
                _ => match glasnost_sim::econ::production::line_kind(w, t) {
                    Some(glasnost_sim::econ::production::LineKind::Defence) => "defence",
                    _ => "other building",
                },
            };
        }
        match (role_of(w, t), t.kind) {
            (Some(Role::Harvester), _) => "harvester",
            (Some(Role::Army), Kind::Infantry) => "army infantry",
            (Some(Role::Army), Kind::Aircraft) => "army aircraft",
            (Some(Role::Army), _) => "army vehicle",
            _ => "other unit",
        }
    }

    fn observe(&mut self, w: &World) {
        let mut alive = BTreeMap::new();
        for o in w.objects.values().filter(|o| o.on_map && o.health > 0) {
            let Some(h) = o.owner.map(|h| usize::from(h.0)).filter(|&h| h < PLAYERS) else {
                continue;
            };
            let c = if o.econ.slave.is_some() {
                "slave"
            } else {
                Self::category(w, tech::type_of(o))
            };
            alive.insert(o.id, (h, c));
        }
        for (id, &(h, c)) in &self.alive {
            // Gone from the map: dead, or inside something (a slave home,
            // a passenger); count only the dead.
            if !alive.contains_key(id) && w.obj(*id).is_none_or(|o| o.health <= 0) {
                *self.lost[h].entry(c).or_default() += 1;
                let by = self
                    .hunted
                    .get(id)
                    .and_then(|v| v.first().cloned())
                    .unwrap_or_else(|| "none".into());
                *self.lost_to[h].entry(format!("{c}<-{by}")).or_default() += 1;
            }
        }
        self.alive = alive;
        self.hunted.clear();
        for o in w.objects.values().filter(|o| o.on_map && o.health > 0) {
            if let Some(glasnost_sim::world::Target::Object(t)) = o.target {
                let loco = w.rules().unit(o.kind, o.type_index).map(|u| u.locomotor);
                self.hunted
                    .entry(t)
                    .or_default()
                    .push(format!("{:?}/{:?}", o.kind, loco));
            }
        }
        // Paid this tick per house, so that a delivery in a tick that also
        // pays for production still counts as earned.
        let mut paid_now = [0i64; PLAYERS];
        let mut now = BTreeMap::new();
        for (i, l) in w.econ.lines.lines.iter().enumerate() {
            let h = usize::from(l.house.0);
            let Some(item) = &l.item else { continue };
            if h >= PLAYERS {
                continue;
            }
            let paid = match self.owed.get(&i) {
                Some(&(o, owed)) if o == item.object => owed - item.owed,
                _ => item.start_price - item.owed,
            };
            if paid > 0 {
                *self.spent[h].entry(Self::category(w, item.t)).or_default() += i64::from(paid);
                paid_now[h] += i64::from(paid);
            }
            now.insert(i, (item.object, item.owed));
        }
        self.owed = now;
        for (h, paid) in paid_now.iter().enumerate() {
            let c = w.houses[h].credits;
            let gained = i64::from(c - self.last_credits[h]) + paid;
            if gained > 0 {
                self.earned[h] += gained;
            }
            self.last_credits[h] = c;
        }
    }

    fn report(&mut self, w: &World) -> String {
        use glasnost_sim::econ::slaves;
        let mut s = String::new();
        for h in 0..PLAYERS {
            let house = HouseId(h as u8);
            let own: Vec<&glasnost_sim::world::Object> = w
                .objects
                .values()
                .filter(|o| o.on_map && o.health > 0 && o.owner == Some(house))
                .collect();
            let harvesters = own
                .iter()
                .filter(|o| o.econ.harvest.is_some() && !slaves::is_master(w, o.id))
                .count();
            let mut masters: BTreeMap<String, usize> = BTreeMap::new();
            let mut members = 0usize;
            let mut inside = 0usize;
            for o in own.iter().filter(|o| slaves::is_master(w, o.id)) {
                if let Some(g) = slaves::group_of(w, o.id) {
                    *masters
                        .entry(format!("{:?} {:?}", o.kind, g.state))
                        .or_default() += 1;
                    members += g.members.iter().filter(|m| m.id.is_some()).count();
                    inside += g
                        .members
                        .iter()
                        .filter(|m| m.id.is_some() && m.inside)
                        .count();
                }
            }
            let mut phases: BTreeMap<String, usize> = BTreeMap::new();
            for o in &own {
                if let Some(ss) = &o.econ.slave {
                    *phases
                        .entry(format!("{:?}/{:?}", o.beh.current, ss.phase))
                        .or_default() += 1;
                }
            }
            let items: Vec<String> = w
                .econ
                .lines
                .lines
                .iter()
                .filter(|l| l.house == house)
                .filter_map(|l| {
                    let i = l.item.as_ref()?;
                    Some(format!(
                        "{:?}:{} {}/54{}",
                        l.kind,
                        Self::category(w, i.t),
                        i.progress,
                        if i.waiting { " waiting" } else { "" }
                    ))
                })
                .collect();
            let spent: i64 = self.spent[h].values().sum();
            // The army on the map by kind.
            let mut army: BTreeMap<String, usize> = BTreeMap::new();
            for o in own
                .iter()
                .filter(|o| role_of(w, tech::type_of(o)) == Some(Role::Army))
            {
                *army.entry(format!("{:?}", o.kind)).or_default() += 1;
            }
            // Explored resource cells and their worth, and the share within
            // 12 cells of a harvester or slave master of the house.
            let workers: Vec<glasnost_map::CellXY> = own
                .iter()
                .filter(|o| o.econ.harvest.is_some() || slaves::is_master(w, o.id))
                .map(|o| o.pos.cell())
                .collect();
            let (mut cells, mut worth, mut near) = (0usize, 0i64, 0i64);
            for (c, _) in w.cells.grid.iter() {
                if glasnost_sim::econ::resource::resource_at(w, c).is_some()
                    && glasnost_sim::econ::shroud::explored(w, house, c)
                {
                    cells += 1;
                    let v = glasnost_sim::econ::resource::worth(w, c);
                    worth += v;
                    if workers.iter().any(|&p| {
                        let (dx, dy) = (i64::from(p.x - c.x), i64::from(p.y - c.y));
                        dx * dx + dy * dy <= 144
                    }) {
                        near += v;
                    }
                }
            }
            // Who drains the house's buildings, and who shoots at each
            // drainer (kind/locomotor), for the drain blackouts.
            let drainers: Vec<String> = w
                .drain
                .links
                .iter()
                .filter(|&(&b, _)| w.obj(b).is_some_and(|o| o.owner == Some(house)))
                .filter_map(|(_, &d)| {
                    let o = w.obj(d)?;
                    let loco = w.rules().unit(o.kind, o.type_index).map(|u| u.locomotor);
                    let shooters = self.hunted.get(&d).cloned().unwrap_or_default();
                    Some(format!(
                        "{:?}/{:?} {} hp {} targeted by {shooters:?}",
                        o.kind, loco, d.0, o.health
                    ))
                })
                .collect();
            let _ = writeln!(
                s,
                "econ tick {} house {h}: credits {} earned {} spent {} {:?}; harvesters {harvesters}; slave masters {:?} slaves {members} ({inside} inside) {:?}; explored resource cells {cells} worth {worth} near workers {near}; power {}/{} plants {} blackout {} drained {} {drainers:?}; army {:?}; buildings {}; lost {:?}; lost to {:?}; lines [{}]",
                w.tick,
                w.houses[h].credits,
                self.earned[h],
                spent,
                self.spent[h],
                masters,
                phases,
                w.houses[h].econ.power.output,
                w.houses[h].econ.power.drain,
                own.iter()
                    .filter(|o| role_of(w, tech::type_of(o)) == Some(Role::Power))
                    .count(),
                glasnost_sim::econ::power::blackout_running(w, house),
                w.houses[h].econ.power.drain_blackout,
                army,
                own.iter().filter(|o| o.kind == Kind::Building).count(),
                self.lost[h],
                self.lost_to[h],
                items.join(", ")
            );
            self.lost_to[h].clear();
            self.earned[h] = 0;
            self.spent[h].clear();
            self.lost[h].clear();
        }
        s
    }
}

fn line_changes(
    w: &World,
    h: HouseId,
    items: &mut BTreeMap<String, Option<(glasnost_sim::econ::TypeRef, i32)>>,
) -> String {
    let mut s = String::new();
    let what = |t: glasnost_sim::econ::TypeRef| {
        if t.kind == Kind::Building {
            let gives: Vec<String> = w
                .rules()
                .superweapons
                .building_sw
                .get(t.index as usize)
                .map(|f| {
                    f.iter()
                        .flatten()
                        .filter_map(|&i| w.rules().superweapons.types.get(i as usize))
                        .filter_map(|x| x.kind.map(|k| format!("{k:?}")))
                        .collect()
                })
                .unwrap_or_default();
            let other = if gives.is_empty() {
                format!("other #{}", t.index)
            } else {
                format!("superweapon {}", gives.join("+"))
            };
            format!("building {:?}", role_of(w, t).map_or(other, role_name))
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
    offered: &mut [BTreeMap<UnitKey, Offer>; PLAYERS],
    made: &mut [BTreeSet<UnitKey>; PLAYERS],
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
    layers: Layers,
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
    if layers != Layers::REFERENCE {
        let _ = writeln!(
            s,
            "rules layers: {} (GLASNOST_PLAY_FIXES)",
            layers.describe()
        );
    }
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
            let r8 = &g.r8;
            let _ = writeln!(
                s,
                "  house {h} round 8: cash tech building first seen {:?} first held {:?}, held at tick {R8_SAMPLE} {}; engineers into own buildings {} (above half strength or of no economy or production role {}); first push (tick, beyond half way, near home) {:?}, most beyond half way at once {}; engineers on the map {}, killed {}, first bound for a cash tech building {:?}",
                r8.tech_seen[h],
                r8.tech_held[h],
                r8.tech_at[h],
                r8.repairs[h],
                r8.wasteful[h],
                r8.push[h],
                r8.out_max[h],
                r8.engineers.values().filter(|&&x| x == h).count(),
                r8.eng_lost[h],
                r8.tech_bound[h]
            );
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
                        " (by kind: {:?}; house-ticks ready by kind: {:?}; house-ticks granted (suspended) by kind: {:?}; suspended by cause [timed blackout, drain blackout, shortage]: {:?})",
                        g.launches_by_kind, g.ready_ticks, g.granted_ticks, g.suspended_causes
                    )
                }
            );
            let uses: Vec<String> = g
                .special
                .iter()
                .enumerate()
                .map(|(h, u)| {
                    format!(
                        "house {h} garrison {} capture {} (tech buildings {}) infiltrate {} jumpjets {} deploys {} radiation exits {} erasures {} prisms {} overpowerable buildings {} charges {} bombs {} disarms {} suicides {} sabotages {} strikes {} crates {} berserk attacks {} psychic sensors {} psychic reveals {} mind control {} gattling defences {} spins kept {} absorber entries {} grinder entries {} bunker builds {} bunker entries {} bunker exits {} lifter builds {} lift orders {} lift drop orders {} drainer builds {} drain orders {} mutator launches {} dominator launches {} economy holds {} resumes {} gunner loads {} plants first {} economy defence {} curtained buildings {} superweapon holds {} expansion builds {} expansion moves {} expansion unblocks {} expansion cancels {} anti-air builds {} non-fighter cancels {}",
                        u.garrisons, u.engineers, u.tech, u.spies, u.jumps, u.deploys, u.radiation, u.erasures, u.prisms, u.overpowerables, u.charges, u.bombs, u.disarms, u.suicides, u.sabotages, u.strikes, u.crates, u.berserks, u.sensors, u.reveals, u.takeovers, u.gattling_defences, u.spins_kept, u.absorbs, u.grinds, u.bunker_builds, u.bunker_entries, u.bunker_exits, u.lifter_builds, u.lift_orders, u.lift_drops, u.drainer_builds, u.drain_orders, u.mutator_launches, u.dominator_launches, u.economy_holds, u.resumes, u.gunner_loads, u.plants_first, u.economy_defence, u.curtained_buildings, u.superweapon_holds, u.expansion_builds, u.expansion_moves, u.expansion_unblocks, u.expansion_cancels, u.anti_air_builds, u.non_fighter_cancels
                    )
                })
                .collect();
            let _ = writeln!(
                s,
                "  special abilities (orders): {}; garrison occupants inside at the end: {}",
                uses.join("; "),
                g.occupants_inside
            );
            let _ = writeln!(
                s,
                "  expansion per house: most construction yards at once {:?}, first tick with two {:?}, yard vehicle buildable from {:?}, expansion probe samples by failed condition {:?}; superweapons per house: first tick granted by kind {:?}, first tick ready by kind {:?}",
                g.yards_max,
                g.two_yards_at,
                g.yard_unit_from,
                g.expansion_blocks,
                g.first_granted,
                g.first_ready
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
            let _ = writeln!(
                s,
                "  mind control: takeovers {}; most objects controlled at once per house {:?}; controllers per house {:?}; overload pulses {}; most victims of one unlimited controller per house {:?}; cloned infantry per house {:?}; most slave miners and slaves at once per house {:?} {:?}",
                g.taken,
                g.controlled_max,
                g.controllers,
                g.overloads,
                g.unlimited_max,
                g.clones,
                g.masters_max,
                g.slaves_max
            );
            let _ = writeln!(
                s,
                "  gattling: house-ticks at the top stage per house {:?}; gattling types per house {:?}",
                g.top_ticks, g.gattling_types
            );
            let _ = writeln!(
                s,
                "  gas: hits {}, clouds {}, most particles at once {}; virus units per house {:?}",
                g.gas_hits, g.clouds, g.gas_max, g.virus_armed
            );
            let _ = writeln!(
                s,
                "  particle systems: appeared by kind {:?}; emitters made (fire, spark, railgun) {:?}; most particles at once {}",
                g.systems_by_kind, g.emitters, g.particles_max
            );
            let _ = writeln!(
                s,
                "  absorbers: entries {}, releases beside a standing building {}; most passengers at once per house {:?}; absorbing buildings per house {:?}",
                g.absorbed, g.absorber_releases, g.passengers_max, g.absorbers
            );
            let _ = writeln!(
                s,
                "  grinders: units ground {}; recycling buildings per house {:?}",
                g.ground, g.grinders
            );
            let _ = writeln!(
                s,
                "  bunkers: vehicles bunkered {}, let out with both standing {}; tank bunkers per house {:?}",
                g.bunkered, g.bunker_releases, g.bunkers
            );
            let _ = writeln!(
                s,
                "  magnetic lift: units seized {}, dropped {}",
                g.seized, g.dropped
            );
            let _ = writeln!(
                s,
                "  special hits: burning walkers {}; disc lasers shot {}, hit {}, cancelled {}",
                g.walkers, g.disc_shots, g.disc_hits, g.disc_cancelled
            );
            let _ = writeln!(
                s,
                "  drain: links formed {}, ended {}; credits drained {}; drained-building ticks {}",
                g.drain_links, g.drain_ends, g.credits_drained, g.drained_ticks
            );
            let _ = writeln!(
                s,
                "  mutation: mutator launches {}, infantry-making animations {}, infantry mutated {}",
                g.mutator_launches, g.conversions, g.infantry_mutated
            );
            let _ = writeln!(
                s,
                "  psychic domination: dominator launches {}, objects taken over {}, objects damaged {}, killed {}; sweeps {}, sweeps that changed an owner {}, owner changes {}",
                g.dominator_launches,
                g.dominated,
                g.dominator_damaged,
                g.dominator_killed,
                g.dominator_sweeps.len(),
                g.dominator_sweeps.iter().filter(|&&n| n > 0).count(),
                g.dominator_sweeps.iter().sum::<usize>()
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

/// The lane line of a domination launch: its house and cell, and what its
/// sweep would take over as things stand at the launch
/// (`glasnost_sim::domination::preview`): every object, and those of other
/// houses (an owner change, S-0612 claim 14). Units can still walk in or
/// out before the sweep, so the sweep's own line gives what it took.
fn dominator_launch_line(
    w: &World,
    d: &glasnost_sim::domination::Domination,
) -> (String, Vec<glasnost_sim::world::ObjectId>) {
    let taken = glasnost_sim::domination::preview(w, d.house, d.cell);
    let other: Vec<glasnost_sim::world::ObjectId> = taken
        .iter()
        .copied()
        .filter(|&id| w.obj(id).is_some_and(|o| o.owner != Some(d.house)))
        .collect();
    let line = format!(
        "dominator launch tick {} house {} cell ({}, {}): preview takes {}, of other houses {}{}",
        w.tick,
        d.house.0,
        d.cell.x,
        d.cell.y,
        taken.len(),
        other.len(),
        other
            .iter()
            .map(|&id| format!(
                "; {} estimate {}",
                dominator_watch_line(w, id, d.house),
                dominator_estimate(w, d.cell, id)
            ))
            .collect::<String>()
    );
    (line, other)
}

/// The area damage the sweep at `c` is expected to deal object `id`
/// (`damage::hit_estimate` at each hit's distance), or -1 out of reach.
fn dominator_estimate(
    w: &World,
    c: glasnost_map::CellXY,
    id: glasnost_sim::world::ObjectId,
) -> i32 {
    let d = &w.rules().superweapons.domination;
    let Some(h) = d.warhead else { return -1 };
    let p = glasnost_sim::superweapons::cell_point(w, c, false);
    let r = glasnost_sim::combat::area::radii(w, h).0;
    let hits: Vec<i32> = glasnost_sim::combat::area::object_distances(w, p, id)
        .into_iter()
        .filter(|&x| x <= r)
        .collect();
    if hits.is_empty() {
        return -1;
    }
    hits.into_iter()
        .map(|x| glasnost_sim::combat::damage::hit_estimate(w, id, d.damage, h, x))
        .sum()
}

/// One object of a domination's preview, as the lane watches it from the
/// launch to the sweep: kind, owner, cell, health, whether it has a
/// movement destination and its behaviour.
fn dominator_watch_line(w: &World, id: glasnost_sim::world::ObjectId, h: HouseId) -> String {
    match w.obj(id) {
        None => format!("object {} gone", id.0),
        Some(o) => format!(
            "object {} {:?} owner {:?}{} cell ({}, {}) health {} dest {} beh {:?}",
            id.0,
            o.kind,
            o.owner.map(|x| x.0),
            if o.owner == Some(h) {
                " (dominating)"
            } else {
                ""
            },
            o.pos.cell().x,
            o.pos.cell().y,
            o.health,
            o.dest.is_some(),
            o.beh.current
        ),
    }
}

/// The line of `GLASNOST_AI_AAPROBE` for house `h`: each seen drainer in
/// the air with its cell and what answers it (kind, type index, cell,
/// distance in cells, behaviour), the drainers draining on the ground,
/// what the lines hold, the type ordered now and the candidate counts.
fn aa_probe_line(w: &World, h: HouseId, p: &glasnost_ai::antiair::Probe) -> String {
    let mut parts = Vec::new();
    for &(d, a) in &p.drainers {
        let Some(o) = w.obj(d) else { continue };
        let dc = o.pos.cell();
        let what = match a.and_then(|a| w.obj(a)) {
            Some(x) => {
                let c = x.pos.cell();
                let dist =
                    ((i64::from(c.x - dc.x)).pow(2) + (i64::from(c.y - dc.y)).pow(2)).isqrt();
                let t = Some(glasnost_sim::world::Target::Object(d));
                let (slot, wp) = glasnost_sim::combat::weapon::would_use(w, x.id, t);
                let full = glasnost_sim::combat::check::check(w, x.id, t, slot, true);
                let reach = wp.map(|wp| w.rules().combat.weapon(wp).range);
                format!(
                    "answered by {:?} #{} at ({}, {}) {} cells {:?} dest {:?} check {:?} range {:?} heights {} {} target {}",
                    x.kind,
                    x.type_index,
                    c.x,
                    c.y,
                    dist,
                    x.beh.current,
                    x.dest,
                    full,
                    reach,
                    x.pos.z,
                    o.pos.z,
                    matches!(x.target, Some(glasnost_sim::world::Target::Object(t)) if t == d)
                )
            }
            None => "unanswered".to_string(),
        };
        let link = glasnost_sim::drain::partner(w, d).is_some();
        parts.push(format!(
            "drainer at ({}, {}) linked {link} {what}",
            dc.x, dc.y
        ));
    }
    format!(
        "aaprobe tick {} house {}: credits {} drainers {} on ground {} lined {} build {:?} candidates hit/lit/startable/free {:?} units hit/near/unberserk/check/there {:?} defences hit/operating/weapon/check/range {:?} refused {:?}; [{}]",
        w.tick,
        h.0,
        w.houses[usize::from(h.0)].credits,
        p.drainers.len(),
        p.draining_on_ground,
        p.lined,
        p.build.map(|t| (t.kind, t.index)),
        p.candidates,
        p.units,
        p.defences,
        p.refused.map(|(t, slot, a)| (t.kind, t.index, slot, a)),
        parts.join("; ")
    )
}

/// The lines of `GLASNOST_AI_SWPROBE`: per house and mutator or dominator
/// record (granted, ready or not, suspended), what the AI's aim sees.
fn sw_probe_line(w: &World) -> String {
    use glasnost_sim::superweapons::{SwKind, charge};
    let mut s = String::new();
    for h in 0..PLAYERS {
        let house = HouseId(h as u8);
        for (i, t) in w.rules().superweapons.types.iter().enumerate() {
            let Some(r) = charge::get(w, house, i as u32) else {
                continue;
            };
            if !r.granted {
                continue;
            }
            let p = &w.houses[h].econ.power;
            match t.kind {
                Some(SwKind::Genetic) => {
                    let (seen, any, best) = glasnost_ai::mutator::probe(w, house);
                    let best = best.map_or("none".into(), |(g, l, n)| {
                        format!("gain {g} loss {l} infantry {n}")
                    });
                    s += &format!(
                        "swprobe tick {} house {h} Genetic ready {} suspended {} power {}/{}: seen infantry {seen} most in a reach {any} best {best}\n",
                        w.tick, r.ready, r.suspended, p.output, p.drain
                    );
                }
                Some(SwKind::Dominator) => {
                    let glasnost_ai::dominator::Probe {
                        seen,
                        units,
                        dominable,
                        moving,
                        refused,
                        not_examined,
                        best,
                    } = glasnost_ai::dominator::probe(w, house);
                    let best = best.map_or("none".into(), |(g, l, n)| {
                        format!("gain {g} loss {l} units {n}")
                    });
                    s += &format!(
                        "swprobe tick {} house {h} Dominator ready {} suspended {} power {}/{}: seen objects {seen} units {units} dominable {dominable} moving {moving} not examined {not_examined} best {best} refused {refused:?}\n",
                        w.tick, r.ready, r.suspended, p.output, p.drain
                    );
                }
                _ => {}
            }
        }
    }
    s
}
