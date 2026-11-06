//! The headless simulation runner: runs games without a window.
//!
//! - **Synthetic** (`run_synthetic`, `replay`): the testkit scenario or a
//!   replay file over it, printing per-tick state hashes. Usable anywhere.
//! - **Local lane** (`smoke`): loads every map of the install named by
//!   `GLASNOST_GAME_DIR` in our viewer's sandbox setup, runs it for a number
//!   of ticks and reports counts and timings only (no names, no values, no
//!   paths). Never in CI.

use std::fmt::Write as _;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use glasnost_sim::combat::effects::Effect;
use glasnost_sim::load::{new_game, sandbox_setup};
use glasnost_sim::rules::SimRules;
use glasnost_sim::save::{self, Replay};
use glasnost_sim::world::GameSetup;
use glasnost_sim::{testkit, tick};

/// Per-tick state hashes of the synthetic scenario's scripted replay.
pub fn run_synthetic(seed: u32, ticks: i32) -> Vec<String> {
    let rep = testkit::scripted_replay(seed, ticks);
    replay(&rep).unwrap_or_default()
}

/// Per-tick state hashes of a replay over the synthetic scenario.
pub fn replay(rep: &Replay) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    rep.run(
        testkit::rules(),
        &testkit::map(),
        testkit::catalogue(),
        |w| out.push(save::state_hash(w)),
    )
    .map_err(|e| e.to_string())?;
    Ok(out)
}

/// One map's smoke result.
#[derive(Clone, Debug, Default)]
pub struct MapRun {
    /// Objects in play after loading.
    pub objects: usize,
    /// Objects in play at the end.
    pub objects_end: usize,
    /// Houses of the sandbox setup.
    pub houses: usize,
    /// Owner names whose country was substituted.
    pub substituted: usize,
    /// Distinct notes (provisional rules met).
    pub notes: usize,
    /// Import and catalogue problems.
    pub problems: usize,
    /// Milliseconds to load the inputs (import, catalogue, decode).
    pub load_inputs_ms: u128,
    /// Milliseconds for `new_game`.
    pub new_game_ms: u128,
    /// Milliseconds for the ticks.
    pub ticks_ms: u128,
    /// Ticks run.
    pub ticks: i32,
    /// Mobile objects given a move order.
    pub ordered: usize,
    /// Of those, the ones without a destination at the end.
    pub settled: usize,
    /// Of those, the ones that moved at all.
    pub moved: usize,
    /// Shots made (combat events, S-0301).
    pub shots: usize,
    /// Detonations (S-0303).
    pub detonations: usize,
    /// Objects killed (S-0312).
    pub killed: usize,
    /// Most projectiles in flight at once (S-0306).
    pub max_in_flight: usize,
    /// Why the map did not run to the end, if it did not.
    pub failure: Option<String>,
    /// The skirmish pass: players of the skirmish setup (0: not tried).
    pub skirmish_players: usize,
    /// The skirmish pass: why `new_game` refused, if it did.
    pub skirmish_refused: Option<String>,
    /// The skirmish pass: whether the start list needs a random start cell.
    pub skirmish_random_start: bool,
    /// The skirmish pass: objects in play after loading.
    pub skirmish_objects: usize,
    /// The skirmish pass: milliseconds for `new_game`.
    pub skirmish_ms: u128,
    /// The skirmish pass: starting units created.
    pub skirmish_starting_units: usize,
}

/// Runs every map of the install for `ticks` ticks.
/// `only`: run just the map with this index.
pub fn smoke(dir: &Path, ticks: i32, seed: u32, only: Option<usize>) -> std::io::Result<String> {
    pollster::block_on(smoke_async(dir, ticks, seed, only))
}

async fn smoke_async(
    dir: &Path,
    ticks: i32,
    seed: u32,
    only: Option<usize>,
) -> std::io::Result<String> {
    let t0 = Instant::now();
    let mut vfs = crate::local_lane::mount(dir).await?;
    let mount_ms = t0.elapsed().as_millis();
    let maps = glasnost_convert::find::find_maps(&vfs).await;
    let mut runs = Vec::new();
    for (index, listing) in maps.iter().enumerate() {
        let mut run = MapRun::default();
        if only.is_some_and(|o| o != index) {
            run.failure = Some("skipped".into());
            runs.push(run);
            continue;
        }
        let Ok(bytes) = vfs.read(&listing.file).await else {
            run.failure = Some("map file unreadable".into());
            runs.push(run);
            continue;
        };
        let t1 = Instant::now();
        let inputs = glasnost_convert::game::load(
            &mut vfs,
            bytes,
            "install".into(),
            glasnost_rules::layers::Layers::REFERENCE,
        )
        .await;
        run.load_inputs_ms = t1.elapsed().as_millis();
        run.problems = inputs.problems.len();
        let resolved = match inputs.resolved {
            Ok(r) => r,
            Err(_) => {
                run.failure = Some("rules did not resolve".into());
                runs.push(run);
                continue;
            }
        };
        let Some(tiles) = inputs.tiles else {
            run.failure = Some("no tile catalogue".into());
            runs.push(run);
            continue;
        };
        let rules = Arc::new(SimRules::new(&resolved.ruleset, &resolved.sim_rules_hash));
        let (setup, substituted) = sandbox_setup(&inputs.map, &rules, seed);
        run.houses = setup.houses.len();
        run.substituted = substituted.len();
        let map = inputs.map;
        skirmish_pass(&mut run, &rules, &map, &tiles, seed);
        let result = catch_unwind(AssertUnwindSafe(|| {
            let t2 = Instant::now();
            let mut w = new_game(rules.clone(), &map, Arc::new(tiles.clone()), &setup)
                .map_err(|e| e.to_string())?;
            let new_game_ms = t2.elapsed().as_millis();
            let objects = w.objects.len();
            let t3 = Instant::now();
            let mut ordered = Vec::new();
            let mut fight = [0usize; 4];
            for _ in 0..ticks {
                if w.tick == ORDER_TICK {
                    ordered = order_moves(&mut w);
                }
                tick::step(&mut w);
                // The tick's combat events, counted and then dropped (nothing draws them here).
                for e in w.effects.drain(..) {
                    match e {
                        Effect::Shot { .. }
                        | Effect::Disc {
                            event: glasnost_sim::combat::effects::DiscEvent::Shot,
                            ..
                        } => fight[0] += 1,
                        Effect::Detonation { .. } => fight[1] += 1,
                        Effect::Killed { .. } => fight[2] += 1,
                        Effect::Anim { .. }
                        | Effect::Sound { .. }
                        | Effect::Mind { .. }
                        | Effect::Consumed { .. }
                        | Effect::Spin { .. }
                        | Effect::Trigger { .. }
                        | Effect::Hit { .. }
                        | Effect::Rock { .. }
                        | Effect::DroneRock { .. }
                        | Effect::GrapplePush { .. }
                        | Effect::OverloadRock { .. }
                        | Effect::CrashTumble { .. }
                        | Effect::OverlayCrushed { .. }
                        | Effect::PieceSound { .. }
                        | Effect::Superweapon { .. }
                        | Effect::SupportBeam { .. }
                        | Effect::Captured { .. }
                        | Effect::Infiltrated { .. }
                        | Effect::Cloak { .. }
                        | Effect::GasHit { .. }
                        | Effect::LeftoverHit { .. }
                        | Effect::SpecialAnims { .. }
                        | Effect::DepotSlot { .. }
                        | Effect::Bunkered { .. }
                        | Effect::Converted { .. }
                        | Effect::Seized { .. }
                        | Effect::Route { .. }
                        | Effect::Deactivated { .. }
                        | Effect::Dropped { .. }
                        | Effect::Disc { .. }
                        | Effect::Beam { .. }
                        | Effect::Line { .. }
                        | Effect::FragmentLine { .. }
                        | Effect::Bolt { .. }
                        | Effect::Wave { .. }
                        | Effect::RailLine { .. }
                        | Effect::Drain { .. }
                        | Effect::Maker { .. }
                        | Effect::Domination { .. }
                        | Effect::Walker { .. }
                        | Effect::ShotAnim { .. }
                        | Effect::TargetLine { .. }
                        | Effect::LayerMoment { .. } => {}
                    }
                }
                fight[3] = fight[3].max(w.projectiles.len());
            }
            let ms = t3.elapsed().as_millis();
            let settled = ordered
                .iter()
                .filter(|(id, _)| w.obj(*id).is_some_and(|o| o.dest.is_none()))
                .count();
            let moved = ordered
                .iter()
                .filter(|(id, p)| w.obj(*id).is_some_and(|o| o.pos != *p))
                .count();
            Ok::<_, String>((
                objects,
                w.objects.len(),
                w.notes.len(),
                new_game_ms,
                ms,
                ordered.len(),
                settled,
                moved,
                fight,
            ))
        }));
        match result {
            Ok(Ok((objects, end, notes, ng, tk, ordered, settled, moved, fight))) => {
                [run.shots, run.detonations, run.killed, run.max_in_flight] = fight;
                run.ordered = ordered;
                run.settled = settled;
                run.moved = moved;
                run.objects = objects;
                run.objects_end = end;
                run.notes = notes;
                run.new_game_ms = ng;
                run.ticks_ms = tk;
                run.ticks = ticks;
            }
            Ok(Err(e)) => run.failure = Some(format!("new game: {e}")),
            Err(p) => {
                let msg = p
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| p.downcast_ref::<&str>().map(|s| (*s).to_owned()))
                    .unwrap_or_default();
                run.failure = Some(format!("panic: {msg}"));
            }
        }
        runs.push(run);
    }
    Ok(report(mount_ms, &runs, ticks))
}

/// The skirmish pass of the smoke run: a game set up by [`skirmish_setup`]
/// (start positions, starting units), loaded but not run.
fn skirmish_pass(
    run: &mut MapRun,
    rules: &Arc<SimRules>,
    map: &glasnost_map::Map,
    tiles: &glasnost_map::terrain::TileCatalogue,
    seed: u32,
) {
    let setup = skirmish_setup(map, rules, seed);
    let players = setup.houses.iter().filter(|h| h.human).count();
    run.skirmish_players = players;
    let (list, n) = map.start_list(players as u16);
    run.skirmish_random_start = list.len() < usize::from(n);
    let t = Instant::now();
    let result = catch_unwind(AssertUnwindSafe(|| {
        new_game(rules.clone(), map, Arc::new(tiles.clone()), &setup).map(|w| w.objects.len())
    }));
    run.skirmish_ms = t.elapsed().as_millis();
    // The same game without starting units, to count them.
    let bare = GameSetup {
        unit_count: 0,
        bases: false,
        ..setup.clone()
    };
    let without = catch_unwind(AssertUnwindSafe(|| {
        new_game(rules.clone(), map, Arc::new(tiles.clone()), &bare).map(|w| w.objects.len())
    }));
    if let (Ok(Ok(a)), Ok(Ok(b))) = (&result, &without) {
        run.skirmish_starting_units = a.saturating_sub(*b);
    }
    match result {
        Ok(Ok(objects)) => run.skirmish_objects = objects,
        Ok(Err(e)) => run.skirmish_refused = Some(format!("{e:?}")),
        Err(_) => run.skirmish_refused = Some("panic".into()),
    }
}

/// The skirmish setup of the smoke run (a rule of our tool, not of the
/// game): as many players as the lobby allows on the map, every one on a
/// random start position, the first a human player's house and the others
/// computer players' houses (AI houses count as human-controlled, glossary
/// *AI house*), with the rules' countries that are not passive in turn; then
/// the neutral and special houses, not human-controlled (S-0009 claims 19
/// and 27).
pub fn skirmish_setup(map: &glasnost_map::Map, rules: &SimRules, seed: u32) -> GameSetup {
    use glasnost_sim::world::{HouseRole, HouseSetup};
    let defined = (0..8u16).filter(|k| map.waypoints.contains_key(k)).count();
    let players = if defined == 0 { 8 } else { defined };
    let countries: Vec<&String> = rules
        .country_names
        .iter()
        .zip(&rules.countries)
        .filter(|(n, c)| !c.passive && *n != "Neutral" && *n != "Special")
        .map(|(n, _)| n)
        .collect();
    let mut houses = Vec::new();
    for i in 0..players {
        let country = countries
            .get(i % countries.len().max(1))
            .map(|s| (*s).clone())
            .unwrap_or_default();
        houses.push(HouseSetup {
            name: if i == 0 {
                "<human player>".into()
            } else {
                "Computer".into()
            },
            country,
            human: true,
            start: None,
            role: HouseRole::Player,
            // The tool's computer players are AI houses (S-0412 claim 12).
            ai: i != 0,
        });
    }
    for (name, role) in [
        (glasnost_sim::load::NEUTRAL, HouseRole::Neutral),
        (glasnost_sim::load::SPECIAL, HouseRole::Special),
    ] {
        if rules.country_names.iter().any(|n| n == name) {
            houses.push(HouseSetup {
                name: name.into(),
                country: name.into(),
                human: false,
                start: None,
                role,
                ai: false,
            });
        }
    }
    GameSetup {
        seed,
        game_speed: rules.general.lobby_game_speed,
        houses,
        alliances: Vec::new(),
        sandbox: false,
        unit_count: SMOKE_UNIT_COUNT,
        bases: true,
        skirmish: Default::default(),
    }
}

/// The unit-count setting of the smoke run's skirmish (our tool's choice).
const SMOKE_UNIT_COUNT: i32 = 10;

/// The tick in which the smoke run orders its moves (the first tick after
/// the order block of S-0201 claim 14).
const ORDER_TICK: i32 = 4;

/// Gives every mobile object a move order to the cell of the mobile object
/// that follows it in ID order (the last one goes to the first one's cell),
/// through each owner's pending list. Returns the ordered objects and their
/// positions.
fn order_moves(
    w: &mut glasnost_sim::world::World,
) -> Vec<(glasnost_sim::world::ObjectId, glasnost_sim::world::Pos)> {
    use glasnost_sim::orders::{self, Order};
    use glasnost_sim::rules::Behaviour;
    use glasnost_sim::world::Target;
    let mobile: Vec<_> = w
        .objects
        .values()
        .filter(|o| o.kind.is_mobile() && o.owner.is_some())
        .map(|o| (o.id, o.owner, o.pos))
        .collect();
    let mut out = Vec::new();
    for (i, &(id, owner, pos)) in mobile.iter().enumerate() {
        let (_, _, to) = mobile[(i + 1) % mobile.len()];
        let Some(h) = owner else { continue };
        let order = Order::Behaviour {
            actor: id,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(to.cell())),
        };
        if orders::issue(w, h, order) {
            out.push((id, pos));
        }
    }
    out
}

fn report(mount_ms: u128, runs: &[MapRun], ticks: i32) -> String {
    let mut s = String::new();
    let _ = writeln!(
        s,
        "glasnost-headless smoke: {} maps, {ticks} ticks each, install mounted in {mount_ms} ms",
        runs.len()
    );
    let ok = runs.iter().filter(|r| r.failure.is_none()).count();
    let _ = writeln!(s, "ran to the end: {ok}; failed: {}", runs.len() - ok);
    let total_ticks: i64 = runs.iter().map(|r| i64::from(r.ticks)).sum();
    let total_ms: u128 = runs.iter().map(|r| r.ticks_ms).sum();
    if total_ms > 0 {
        let _ = writeln!(
            s,
            "ticks per second over all maps: {}",
            i128::from(total_ticks) * 1000 / total_ms as i128
        );
    }
    if let Some(slowest) = runs
        .iter()
        .filter(|r| r.ticks > 0)
        .max_by_key(|r| r.ticks_ms * 1000 / (r.ticks as u128).max(1))
    {
        let _ = writeln!(
            s,
            "slowest map: {} objects, {} µs per tick",
            slowest.objects,
            slowest.ticks_ms * 1000 / (slowest.ticks as u128).max(1)
        );
    }
    let ordered: usize = runs.iter().map(|r| r.ordered).sum();
    let settled: usize = runs.iter().map(|r| r.settled).sum();
    let moved: usize = runs.iter().map(|r| r.moved).sum();
    let _ = writeln!(
        s,
        "move orders: {ordered}; moved: {moved}; without a destination at the end: {settled}"
    );
    let shots: usize = runs.iter().map(|r| r.shots).sum();
    let detonations: usize = runs.iter().map(|r| r.detonations).sum();
    let killed: usize = runs.iter().map(|r| r.killed).sum();
    let fought = runs.iter().filter(|r| r.shots > 0).count();
    let in_flight = runs.iter().map(|r| r.max_in_flight).max().unwrap_or(0);
    let _ = writeln!(
        s,
        "combat: {shots} shots, {detonations} detonations, {killed} killed; shots on {fought} maps; at most {in_flight} projectiles in flight"
    );
    let tried: Vec<&MapRun> = runs.iter().filter(|r| r.skirmish_players > 0).collect();
    let refused = tried
        .iter()
        .filter(|r| r.skirmish_refused.is_some())
        .count();
    let random = tried.iter().filter(|r| r.skirmish_random_start).count();
    let random_refused = tried
        .iter()
        .filter(|r| r.skirmish_random_start && r.skirmish_refused.is_some())
        .count();
    let mut reasons: std::collections::BTreeMap<String, usize> = Default::default();
    for r in &tried {
        if let Some(e) = &r.skirmish_refused {
            *reasons.entry(e.clone()).or_default() += 1;
        }
    }
    let skirmish_ms: u128 = tried.iter().map(|r| r.skirmish_ms).sum();
    let units: usize = tried.iter().map(|r| r.skirmish_starting_units).sum();
    let with_units = tried
        .iter()
        .filter(|r| r.skirmish_starting_units > 0)
        .count();
    let _ = writeln!(
        s,
        "skirmish pass: {} maps set up, {} loaded, {refused} refused ({random} need a random start cell, {random_refused} of them refused); new_game {skirmish_ms} ms in all",
        tried.len(),
        tried.len() - refused
    );
    for (e, n) in reasons {
        let _ = writeln!(s, "  refused {n}: {e}");
    }
    let _ = writeln!(
        s,
        "skirmish starting units: {units} on {with_units} maps (unit count {SMOKE_UNIT_COUNT}, bases on)"
    );
    let _ = writeln!(
        s,
        "index  objects  end  houses  subst  notes  problems  inputs_ms  new_game_ms  ticks_ms  ordered  moved  settled  shots  killed  result"
    );
    for (i, r) in runs.iter().enumerate() {
        let _ = writeln!(
            s,
            "{i:5}  {:7}  {:4}  {:6}  {:5}  {:5}  {:8}  {:9}  {:11}  {:8}  {:7}  {:5}  {:7}  {:5}  {:6}  {}",
            r.objects,
            r.objects_end,
            r.houses,
            r.substituted,
            r.notes,
            r.problems,
            r.load_inputs_ms,
            r.new_game_ms,
            r.ticks_ms,
            r.ordered,
            r.moved,
            r.settled,
            r.shots,
            r.killed,
            r.failure.as_deref().unwrap_or("ok")
        );
    }
    s
}
