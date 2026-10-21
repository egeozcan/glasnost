//! Forced desyncs (M9 N4): peers play the synthetic skirmish (invented rules
//! and map, `glasnost_ai::fixture`) in lockstep over the seeded in-memory
//! network, each keeping its desync evidence. One peer is perturbed before
//! a tick (an object's health, a house's credits); every peer that sees the
//! mismatch writes a bundle, tells the others its digest, and the desync
//! tool names the first differing tick, the object or house and the field.
//! With no perturbation there is no desync, and two bundles made on demand
//! compare as the same, exit code 0. Our own netcode; no spec applies.
//!
//! The local lane (`GLASNOST_GAME_DIR` set, never in CI) forces a desync on
//! a few of the install's maps and prints counts and timings only: the
//! bundle and snapshot sizes, the evidence's cost per tick, and what the
//! tool names (how many objects and fields, never their values).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::Instant;

use glasnost_ai::fixture;
use glasnost_ai::game::Game;
use glasnost_map::Map;
use glasnost_map::geometry::CellXY;
use glasnost_map::terrain::TileCatalogue;
use glasnost_net::desync::{Bundle, Evidence, Perturb};
use glasnost_net::simnet::{Link, SimNet, SplitMix};
use glasnost_net::wire::Hash;
use glasnost_net::{Config, Event, Params, Phase, Session, SlotKind, StartInfo, pump};
use glasnost_sim::econ::setup::{self, Player, Settings};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::SimRules;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{GameSetup, HouseId, Target};
use glasnost_tools::desync::{self, Inputs, Report};

const TICK_MS: u32 = 33;

/// The tick a perturbation is made before.
const T: u32 = 100;

/// The ticks every peer runs.
const TICKS: u32 = 200;

/// The synthetic skirmish map with two more start waypoints.
fn map4() -> Map {
    let mut m = fixture::map();
    for (k, dx, dy) in [(2u16, -14, -18), (3, 14, 18)] {
        let c = fixture::at(dx, dy);
        m.waypoints.insert(k, (c.x as i16, c.y as i16));
    }
    m
}

/// `persons` persons and one computer player.
fn setup_of(persons: usize, seed: u32) -> GameSetup {
    let players: Vec<Player> = (0..=persons)
        .map(|i| Player {
            name: format!("P{i}"),
            country: if i % 2 == 0 { "Alphaland" } else { "Betaland" }.into(),
            start: Some(i as u32),
            ai: i == persons,
            values: Default::default(),
        })
        .collect();
    setup::skirmish(&fixture::rules(), &players, Settings::default(), 0, seed)
}

fn game(map: &Map, setup: &GameSetup) -> Game {
    Game::new(
        fixture::rules(),
        map,
        "synthetic-4",
        fixture::catalogue(),
        setup.clone(),
    )
    .expect("the synthetic skirmish loads")
}

/// What the peers play: the map, its rules and setup, and how long.
struct Case {
    map: Map,
    map_id: String,
    rules: Arc<SimRules>,
    catalogue: Arc<TileCatalogue>,
    setup: GameSetup,
    /// The cell the scripted moves go around.
    centre: CellXY,
    /// The ticks every peer runs.
    ticks: u32,
    /// Whether the evidence takes every tick's hash as the game runs.
    tick_hashes: bool,
}

impl Case {
    /// The synthetic skirmish of `persons` persons and a computer.
    fn synthetic(persons: usize, seed: u32) -> Self {
        Self {
            map: map4(),
            map_id: "synthetic-4".into(),
            rules: fixture::rules(),
            catalogue: fixture::catalogue(),
            setup: setup_of(persons, seed),
            centre: fixture::at(0, 0),
            ticks: TICKS,
            tick_hashes: false,
        }
    }

    fn game(&self) -> Game {
        Game::new(
            self.rules.clone(),
            &self.map,
            &self.map_id,
            self.catalogue.clone(),
            self.setup.clone(),
        )
        .expect("the case's game loads")
    }
}

/// A person's scripted orders: now and then one of its units goes
/// somewhere around `centre`.
fn scripted(g: &Game, house: HouseId, tick: u32, rng: &mut SplitMix, centre: CellXY) -> Vec<Order> {
    if tick % 13 != 5 {
        return Vec::new();
    }
    let units: Vec<_> = g
        .world
        .objects
        .iter()
        .filter(|(_, o)| {
            o.owner == Some(house)
                && o.on_map
                && o.health > 0
                && matches!(o.kind, Kind::Vehicle | Kind::Infantry)
        })
        .map(|(&id, _)| id)
        .collect();
    if units.is_empty() {
        return Vec::new();
    }
    let id = units[rng.below(units.len() as u64) as usize];
    let dx = rng.below(31) as i32 - 15;
    let dy = rng.below(31) as i32 - 15;
    vec![Order::Behaviour {
        actor: id,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(CellXY {
            x: centre.x + dx,
            y: centre.y + dy,
        })),
    }]
}

struct Peer {
    slot: u8,
    house: HouseId,
    session: Session,
    game: Game,
    evidence: Evidence,
    rng: SplitMix,
    next_due: u64,
    input_for: Option<u32>,
    bundle: Option<(Bundle, Vec<u8>)>,
    desyncs: Vec<u32>,
    reports: Vec<(u8, u32, Hash)>,
    most_snapshots: usize,
    most_ticks: usize,
    most_bytes: usize,
    /// Ticks run, and the time spent in them and in the evidence after
    /// them (nanoseconds; the lane's timings).
    steps: u32,
    step_ns: u128,
    evidence_ns: u128,
    evidence_most_ns: u128,
    /// How long making the report took (it replays the ticks).
    take_ns: u128,
}

struct Played {
    peers: Vec<Peer>,
    start: StartInfo,
}

/// Plays `persons` persons and a computer for [`TICKS`] ticks, `perturb`
/// made on the peer of its slot, then lets the messages settle.
fn play(persons: usize, seed: u64, perturb: Option<(u8, Perturb)>) -> Played {
    play_with(persons, seed, perturb, false)
}

/// [`play`], with every tick's hash taken as the game runs or not.
fn play_with(
    persons: usize,
    seed: u64,
    perturb: Option<(u8, Perturb)>,
    tick_hashes: bool,
) -> Played {
    let case = Case {
        tick_hashes,
        ..Case::synthetic(persons, seed as u32)
    };
    let played = play_case(&case, seed, perturb);
    assert_eq!(played.peers.len(), persons);
    played
}

/// Plays the case's persons for its ticks, `perturb` made on the peer of
/// its slot, then lets the messages settle.
fn play_case(case: &Case, seed: u64, perturb: Option<(u8, Perturb)>) -> Played {
    let ticks = case.ticks;
    let info = StartInfo::new(
        "test-build",
        &case.rules.hash,
        &case.map_id,
        &case.setup,
        Params::default(),
    );
    let slots: Vec<u8> = info
        .slots
        .iter()
        .enumerate()
        .filter(|(_, s)| s.kind == SlotKind::Human)
        .map(|(i, _)| i as u8)
        .collect();
    let link = Link {
        latency_ms: 25,
        jitter_ms: 15,
        loss_permille: 60,
        dup_permille: 30,
        reorder_permille: 50,
    };
    let mut net = SimNet::new(seed, &slots, link);
    let mut peers: Vec<Peer> = slots
        .iter()
        .map(|&slot| {
            let g = case.game();
            let p = perturb.filter(|(s, _)| *s == slot).map(|(_, p)| p);
            Peer {
                slot,
                house: HouseId(info.slots[usize::from(slot)].house),
                session: Session::new(Config::new(TICK_MS), info.clone(), slot, 0)
                    .expect("a person's slot"),
                evidence: Evidence::new("test-build", &info, slot, &g)
                    .with_perturb(p)
                    .with_tick_hashes(case.tick_hashes),
                game: g,
                rng: SplitMix::new(seed * 31 + u64::from(slot)),
                next_due: 0,
                input_for: None,
                bundle: None,
                desyncs: Vec::new(),
                reports: Vec::new(),
                most_snapshots: 0,
                most_ticks: 0,
                most_bytes: 0,
                steps: 0,
                step_ns: 0,
                evidence_ns: 0,
                evidence_most_ns: 0,
                take_ns: 0,
            }
        })
        .collect();
    let mut now = 0u64;
    let mut settle = 0u64;
    loop {
        now += 1;
        assert!(now < 600_000, "the peers stalled");
        for p in peers.iter_mut() {
            pump(&mut p.session, &mut net.endpoint(p.slot), now);
            if now >= p.next_due && p.session.phase() == Phase::Running && p.session.ran() < ticks {
                let tick = p.session.ran();
                if p.input_for != Some(tick) {
                    p.input_for = Some(tick);
                    for o in scripted(&p.game, p.house, tick, &mut p.rng, case.centre) {
                        p.session.submit(o);
                    }
                }
                p.evidence.before_tick(tick, &mut p.game);
                let t0 = Instant::now();
                if let Some(t) = p.session.step(now, &mut p.game) {
                    let t1 = Instant::now();
                    p.evidence.after_tick(&t, &p.game, p.session.agreed());
                    let ns = t1.elapsed().as_nanos();
                    p.steps += 1;
                    p.step_ns += (t1 - t0).as_nanos();
                    p.evidence_ns += ns;
                    p.evidence_most_ns = p.evidence_most_ns.max(ns);
                    p.next_due = now + u64::from(TICK_MS);
                    pump(&mut p.session, &mut net.endpoint(p.slot), now);
                }
            }
            for e in p.session.take_events() {
                match e {
                    Event::Desync { tick, mine, theirs } => {
                        p.desyncs.push(tick);
                        p.evidence.desync(tick, theirs.slot, mine.hash, theirs.hash);
                    }
                    Event::Report { slot, tick, digest } => p.reports.push((slot, tick, digest)),
                    Event::BadMessage { reason } => panic!("a bad message: {reason}"),
                    _ => {}
                }
            }
            let t0 = Instant::now();
            if p.bundle.is_none()
                && let Some(b) = p.evidence.take(p.session.ran(), &p.game)
            {
                p.take_ns = t0.elapsed().as_nanos();
                let bytes = b.to_bytes();
                let tick = b.mismatch.expect("a mismatch").tick;
                p.session.set_report(tick, Bundle::digest(&bytes));
                p.bundle = Some((b, bytes));
            }
            p.most_snapshots = p.most_snapshots.max(p.evidence.snapshots().count());
            p.most_ticks = p.most_ticks.max(p.evidence.ticks_held());
            p.most_bytes = p.most_bytes.max(p.evidence.bytes_held());
        }
        if peers.iter().all(|p| p.session.ran() >= ticks) {
            settle += 1;
            if settle > 3_000 {
                break;
            }
        }
    }
    Played { peers, start: info }
}

fn inputs() -> Inputs {
    Inputs::synthetic(&fixture::rules().hash).expect("the fixture's rules")
}

fn bundle_of(p: &Peer) -> &Bundle {
    &p.bundle.as_ref().expect("a bundle").0
}

/// Every peer that saw the mismatch wrote a bundle from the agreed
/// checkpoint before it, heard the others' digests, and kept its evidence
/// bounded.
fn check_reports(played: &Played, perturbed: u8) {
    let every = played.start.params.hash_every;
    // The checkpoint whose ticks hold tick T.
    let checkpoint = (T / every + 1) * every - 1;
    for p in &played.peers {
        assert_eq!(p.desyncs.first(), Some(&checkpoint), "slot {}", p.slot);
        let (b, bytes) = p.bundle.as_ref().expect("a bundle");
        let m = b.mismatch.expect("a mismatch");
        assert_eq!(m.tick, checkpoint);
        if p.slot != perturbed {
            assert_eq!(m.slot, perturbed);
        }
        assert_eq!(b.base.ran, checkpoint + 1 - every, "the agreed snapshot");
        assert_eq!(b.after.as_ref().map(|s| s.ran), Some(checkpoint + 1));
        assert!(b.end() > checkpoint + glasnost_net::desync::PAST_TICKS);
        assert_eq!(Bundle::from_bytes(bytes).as_ref(), Ok(b));
        assert!(
            p.most_snapshots <= p.evidence.max_snapshots() + 1,
            "slot {} held {} snapshots",
            p.slot,
            p.most_snapshots
        );
        for q in &played.peers {
            if q.slot == p.slot || (p.slot != perturbed && q.slot != perturbed) {
                continue;
            }
            let (_, theirs) = q.bundle.as_ref().expect("a bundle");
            assert!(
                p.reports
                    .contains(&(q.slot, checkpoint, Bundle::digest(theirs))),
                "slot {} heard slot {}'s digest",
                p.slot,
                q.slot
            );
        }
    }
}

fn write(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, bytes).expect("a bundle file");
    path
}

fn out_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("glasnost-desync-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a temporary folder");
    dir
}

fn headless(a: &Path, b: &Path) -> (Option<i32>, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_glasnost-headless"))
        .arg("desync")
        .arg(a)
        .arg(b)
        .env_remove("GLASNOST_GAME_DIR")
        .output()
        .expect("glasnost-headless runs");
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

/// One object's health changed by 1 on one of two peers before tick T:
/// both write a bundle; the tool names tick T (the perturbation is made
/// before tick T runs, so the hash after tick T is the first that differs),
/// the object and its health; the binary exits with 1.
#[test]
fn a_perturbed_object_is_named_with_its_field_and_tick() {
    let probe = game(&map4(), &setup_of(2, 3));
    let (&id, obj) = probe
        .world
        .objects
        .iter()
        .find(|(_, o)| o.owner == Some(HouseId(0)) && o.kind == Kind::Vehicle && o.on_map)
        .expect("a vehicle of house 0");
    let kind = format!("{:?}", obj.kind);
    let perturb = Perturb::Health {
        tick: T,
        object: id.0,
        by: -1,
    };
    let played = play(2, 3, Some((0, perturb)));
    check_reports(&played, 0);
    let (a, b) = (bundle_of(&played.peers[0]), bundle_of(&played.peers[1]));
    assert_eq!(a.perturb, Some(perturb));
    assert_eq!(b.perturb, None);
    for x in [a, b] {
        assert!(x.ticks.iter().all(|t| t.hash.is_some()));
        let every = played.start.params.hash_every;
        for t in &x.ticks {
            assert_eq!(
                t.live,
                (t.tick + 1).is_multiple_of(every),
                "tick {}",
                t.tick
            );
        }
    }
    let r: Report = desync::compare(a, b, &inputs()).expect("the bundles compare");
    println!("{}", r.text);
    assert!(r.differs);
    assert_eq!(r.first_tick, Some(T));
    assert_eq!(r.replay_tick, Some(T));
    assert_eq!(r.diff_tick, Some(T));
    assert_eq!(r.orders_tick, None);
    assert_eq!(r.unreproduced, [None, None]);
    let o = r
        .diff
        .objects
        .iter()
        .find(|o| o.id == id.0)
        .expect("the perturbed object is named");
    assert_eq!(o.kind, kind);
    assert_eq!(o.owner, Some(0));
    let f = o
        .fields
        .iter()
        .find(|f| f.path == "health")
        .expect("its health is named");
    let (ha, hb): (i32, i32) = (f.a.parse().unwrap(), f.b.parse().unwrap());
    assert_eq!(ha, hb - 1);
    assert!(r.text.contains("The first differing tick: 100."));
    assert!(r.text.contains(&format!("object {}: {kind}", id.0)));
    assert!(r.text.contains("    health: "));
    // Without its record of the perturbation, A's own hashes are not what
    // its replay computes, from tick T on, and the two replays agree: the
    // tool falls back to each peer's own snapshot at the checkpoint.
    let mut bare = a.clone();
    bare.perturb = None;
    let r2 = desync::compare(&bare, b, &inputs()).expect("the bundles compare");
    assert!(r2.differs);
    assert_eq!(r2.first_tick, Some(T));
    assert_eq!(r2.replay_tick, None);
    assert_eq!(r2.unreproduced, [Some(T), None]);
    let every = played.start.params.hash_every;
    assert_eq!(r2.diff_tick, Some((T / every + 1) * every - 1));
    assert!(
        r2.diff
            .objects
            .iter()
            .any(|o| o.id == id.0 && o.fields.iter().any(|f| f.path == "health"))
    );
    // The binary: exit code 1, the same first tick.
    let dir = out_dir("object");
    let pa = write(
        &dir,
        &a.file_name("a-"),
        &played.peers[0].bundle.as_ref().unwrap().1,
    );
    let pb = write(
        &dir,
        &b.file_name("b-"),
        &played.peers[1].bundle.as_ref().unwrap().1,
    );
    let (code, text) = headless(&pa, &pb);
    assert_eq!(code, Some(1), "{text}");
    assert!(text.contains("The first differing tick: 100."), "{text}");
    // Bounded: two snapshots of the small synthetic game and the ticks.
    for p in &played.peers {
        let n = p.bundle.as_ref().unwrap().1.len();
        println!("slot {}: a bundle of {n} bytes", p.slot);
        assert!(n < 4 * 1024 * 1024);
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// One house's credits changed by 1 on the middle one of three peers: the
/// tool names the house and its credits; the two unperturbed peers' bundles
/// compare as the same.
#[test]
fn a_perturbed_house_is_named_and_the_others_agree() {
    let house = 1u8;
    let perturb = Perturb::Credits {
        tick: T,
        house,
        by: 1,
    };
    // Every tick's hash taken as the games run, not replayed for the report.
    let played = play_with(3, 5, Some((1, perturb)), true);
    assert!(
        played
            .peers
            .iter()
            .all(|p| bundle_of(p).ticks.iter().all(|t| t.live))
    );
    check_reports(&played, 1);
    let mid = bundle_of(&played.peers[1]);
    let first = bundle_of(&played.peers[0]);
    let last = bundle_of(&played.peers[2]);
    let r = desync::compare(mid, first, &inputs()).expect("the bundles compare");
    println!("{}", r.text);
    assert!(r.differs);
    assert_eq!(r.first_tick, Some(T));
    assert_eq!(r.replay_tick, Some(T));
    let h = r
        .diff
        .houses
        .iter()
        .find(|h| h.house == house)
        .expect("the perturbed house is named");
    let f = h
        .fields
        .iter()
        .find(|f| f.path == "credits")
        .expect("its credits are named");
    let (ca, cb): (i64, i64) = (f.a.parse().unwrap(), f.b.parse().unwrap());
    assert_eq!(ca, cb + 1);
    assert!(r.text.contains("    credits: "));
    // The two unperturbed peers held the same game.
    let same = desync::compare(first, last, &inputs()).expect("the bundles compare");
    assert!(!same.differs, "{}", same.text);
    assert_eq!(same.first_tick, None);
}

/// No perturbation: no desync, no bundle; two bundles made on demand
/// compare as the same and the binary exits with 0. The evidence stays
/// bounded all game long.
#[test]
fn without_a_perturbation_nothing_differs() {
    let played = play(2, 9, None);
    for p in &played.peers {
        assert!(p.desyncs.is_empty());
        assert!(p.bundle.is_none());
        assert!(p.evidence.mismatch().is_none());
        assert!(p.most_snapshots <= p.evidence.max_snapshots());
        let every = played.start.params.hash_every;
        assert!(
            p.most_ticks <= p.evidence.max_snapshots() * every as usize,
            "slot {} held {} ticks",
            p.slot,
            p.most_ticks
        );
        assert!(p.session.agreed() * every + 2 * every >= TICKS);
    }
    let a = played.peers[0]
        .evidence
        .bundle_now(&played.peers[0].game)
        .expect("a bundle");
    let b = played.peers[1]
        .evidence
        .bundle_now(&played.peers[1].game)
        .expect("a bundle");
    let r = desync::compare(&a, &b, &inputs()).expect("the bundles compare");
    println!("{}", r.text);
    assert!(!r.differs);
    assert!(r.text.contains("No difference."));
    let dir = out_dir("none");
    let pa = write(&dir, &a.file_name("a-"), &a.to_bytes());
    let pb = write(&dir, &b.file_name("b-"), &b.to_bytes());
    let (code, text) = headless(&pa, &pb);
    assert_eq!(code, Some(0), "{text}");
    // A file that is no bundle cannot be compared: exit code 2.
    let junk = write(&dir, "junk.gdsync", b"not a bundle");
    let (code, _) = headless(&pa, &junk);
    assert_eq!(code, Some(2));
    let _ = std::fs::remove_dir_all(&dir);
}

/// The most of the install's maps the lane plays.
const LANE_MAPS: usize = 3;

/// The tick the lane's perturbation is made before.
const LANE_T: u32 = 300;

/// The ticks the lane's peers run.
const LANE_TICKS: u32 = 400;

/// The lane's output folder: `local-lane/m9/n4/` (git-ignored), or
/// `GLASNOST_LANE_OUT`.
fn lane_out() -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../local-lane");
    match std::env::var_os("GLASNOST_LANE_OUT") {
        Some(p) if Path::new(&p).is_absolute() => PathBuf::from(p),
        Some(p) => root.join(p),
        None => root.join("m9/n4"),
    }
}

/// The first multiplayer maps of the install that load, each with two
/// persons (the first two players' houses) and computers for the others.
async fn lane_cases(dir: &Path, most: usize) -> Vec<(usize, Case)> {
    let mut vfs = glasnost_tools::local_lane::mount(dir)
        .await
        .expect("the install mounts");
    let mut out = Vec::new();
    let maps = glasnost_convert::find::find_maps(&vfs).await;
    for (index, listing) in maps.iter().enumerate() {
        if out.len() >= most {
            break;
        }
        if !listing.multiplayer {
            continue;
        }
        let Ok(bytes) = vfs.read(&listing.file).await else {
            continue;
        };
        let map_id = glasnost_net::start::content_hash(&bytes);
        let inputs = glasnost_convert::game::load(&mut vfs, bytes, "install".into()).await;
        let (Ok(resolved), Some(tiles)) = (inputs.resolved, inputs.tiles) else {
            continue;
        };
        let rules = Arc::new(SimRules::new(&resolved.ruleset, &resolved.sim_rules_hash));
        let map = inputs.map;
        let Some(&(x, y)) = map.waypoints.get(&0) else {
            continue;
        };
        let mut setup = glasnost_tools::headless::skirmish_setup(&map, &rules, 11);
        let players = glasnost_net::start::slots_of(&setup).len();
        if players < 3 {
            continue;
        }
        setup.houses[1].ai = false;
        out.push((
            index,
            Case {
                map,
                map_id,
                rules,
                catalogue: Arc::new(tiles),
                setup,
                centre: CellXY {
                    x: i32::from(x),
                    y: i32::from(y),
                },
                ticks: LANE_TICKS,
                tick_hashes: false,
            },
        ));
    }
    out
}

/// The mean of `n` runs of `f`, in microseconds.
fn mean_us(n: u32, mut f: impl FnMut()) -> u128 {
    let t = Instant::now();
    for _ in 0..n {
        f();
    }
    t.elapsed().as_micros() / u128::from(n.max(1))
}

/// The local lane (never in CI): on the first few multiplayer maps of the
/// install, two persons and computers play over the seeded network; slot
/// 0's game changes before tick 300 (an object's health on the first map,
/// a house's credits on the others). Both peers write a bundle, hear the
/// other's digest, and the tool, on the install's rules, names the first
/// differing tick. Prints and writes counts and timings only, never map
/// names, type names or values.
///
/// Run with `cargo test --release -p glasnost-tools --test desync --
/// --ignored --nocapture lane`.
#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn lane_a_desync_on_the_installs_maps_is_reported() {
    let Some(dir) = std::env::var_os(glasnost_tools::local_lane::GAME_DIR_VAR) else {
        println!("GLASNOST_GAME_DIR not set; skipped");
        return;
    };
    let dir = PathBuf::from(dir);
    let t = Instant::now();
    let cases = pollster::block_on(lane_cases(&dir, LANE_MAPS));
    let mut out = format!(
        "{} maps loaded in {} ms\n",
        cases.len(),
        t.elapsed().as_millis()
    );
    assert!(!cases.is_empty(), "no multiplayer map loads");
    for (k, (index, case)) in cases.iter().enumerate() {
        let g = case.game();
        let perturb = match g
            .world
            .objects
            .iter()
            .find(|(_, o)| o.owner == Some(HouseId(0)) && o.on_map && o.health > 0)
        {
            Some((&id, _)) if k == 0 => Perturb::Health {
                tick: LANE_T,
                object: id.0,
                by: -1,
            },
            _ => Perturb::Credits {
                tick: LANE_T,
                house: 0,
                by: 1,
            },
        };
        let kind = match perturb {
            Perturb::Health { .. } => "health",
            Perturb::Credits { .. } => "credits",
        };
        let objects0 = g.world.objects.len();
        let save0 = g.save().len();
        let t = Instant::now();
        let played = play_case(case, 21 + k as u64, Some((0, perturb)));
        let play_ms = t.elapsed().as_millis();
        let every = played.start.params.hash_every;
        let checkpoint = (LANE_T / every + 1) * every - 1;
        out += &format!(
            "map #{index}: {} persons, {} houses, {objects0} objects at the start, a save of {save0} bytes; {kind} changed before tick {LANE_T}; played in {play_ms} ms\n",
            played.peers.len(),
            case.setup.houses.len(),
        );
        for p in &played.peers {
            let n = u128::from(p.steps.max(1));
            out += &format!(
                "  slot {}: {} ticks, a tick {} us, the evidence after it {} us on average, {} us at most; held at most {} snapshots ({} allowed), {} ticks, {} bytes\n",
                p.slot,
                p.steps,
                p.step_ns / n / 1_000,
                p.evidence_ns / n / 1_000,
                p.evidence_most_ns / 1_000,
                p.most_snapshots,
                p.evidence.max_snapshots(),
                p.most_ticks,
                p.most_bytes,
            );
        }
        let g_end = &played.peers[0].game;
        let objects1 = g_end.world.objects.len();
        let save_us = mean_us(10, || {
            std::hint::black_box(g_end.save());
        });
        let hash_us = mean_us(10, || {
            std::hint::black_box(g_end.state_hash());
        });
        out += &format!(
            "  at the end: {objects1} objects, a save of {} bytes in {save_us} us, a state hash in {hash_us} us\n",
            g_end.save().len()
        );
        for p in &played.peers {
            assert_eq!(p.desyncs.first(), Some(&checkpoint), "slot {}", p.slot);
            let (b, bytes) = p.bundle.as_ref().expect("a bundle");
            assert_eq!(b.mismatch.map(|m| m.tick), Some(checkpoint));
            assert!(p.most_snapshots <= p.evidence.max_snapshots() + 1);
            out += &format!(
                "  slot {}'s bundle: made in {} ms, {} bytes, {} ticks, snapshots of {} and {} bytes; heard {} digest(s)\n",
                p.slot,
                p.take_ns / 1_000_000,
                bytes.len(),
                b.ticks.len(),
                b.base.bytes.len(),
                b.after.as_ref().map_or(0, |s| s.bytes.len()),
                p.reports.len(),
            );
            assert_eq!(p.reports.len(), played.peers.len() - 1);
        }
        let (a, b) = (bundle_of(&played.peers[0]), bundle_of(&played.peers[1]));
        let t = Instant::now();
        let inputs = Inputs::install(&dir, &a.start.map_hash, &a.start.rules_hash)
            .expect("the install's inputs for the report");
        let inputs_ms = t.elapsed().as_millis();
        let t = Instant::now();
        let r = desync::compare(a, b, &inputs).expect("the bundles compare");
        let compare_ms = t.elapsed().as_millis();
        let fields: usize = r.diff.objects.iter().map(|o| o.fields.len()).sum::<usize>()
            + r.diff.houses.iter().map(|h| h.fields.len()).sum::<usize>()
            + r.diff.game.len()
            + r.diff.ai.iter().map(|h| h.fields.len()).sum::<usize>();
        out += &format!(
            "  the tool: inputs in {inputs_ms} ms, compared in {compare_ms} ms; first differing tick {:?}, replays differ at {:?}, diff at {:?}, unreproduced {:?}; {} objects differ, {} houses, {} AI houses, {} game-wide fields, {fields} fields in all\n",
            r.first_tick,
            r.replay_tick,
            r.diff_tick,
            r.unreproduced,
            r.diff.objects_differing,
            r.diff.houses.len(),
            r.diff.ai.len(),
            r.diff.game.len(),
        );
        assert!(r.differs);
        assert_eq!(r.first_tick, Some(LANE_T));
        assert_eq!(r.replay_tick, Some(LANE_T));
        assert_eq!(r.unreproduced, [None, None]);
        match perturb {
            Perturb::Health { object, .. } => assert!(
                r.diff
                    .objects
                    .iter()
                    .any(|o| o.id == object && o.fields.iter().any(|f| f.path == "health"))
            ),
            Perturb::Credits { house, .. } => assert!(
                r.diff
                    .houses
                    .iter()
                    .any(|h| h.house == house && h.fields.iter().any(|f| f.path == "credits"))
            ),
        }
    }
    print!("{out}");
    let dir = lane_out();
    if std::fs::create_dir_all(&dir).is_ok() {
        let _ = std::fs::write(dir.join("lane.txt"), &out);
    }
}
