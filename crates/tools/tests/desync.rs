//! Forced desyncs (M9 N4): peers play the synthetic skirmish (invented rules
//! and map, `glasnost_ai::fixture`) in lockstep over the seeded in-memory
//! network, each keeping its desync evidence. One peer is perturbed before
//! a tick (an object's health, a house's credits); every peer that sees the
//! mismatch writes a bundle, tells the others its digest, and the desync
//! tool names the first differing tick, the object or house and the field.
//! With no perturbation there is no desync, and two bundles made on demand
//! compare as the same, exit code 0. Our own netcode; no spec applies.

use std::path::{Path, PathBuf};
use std::process::Command;

use glasnost_ai::fixture;
use glasnost_ai::game::Game;
use glasnost_map::Map;
use glasnost_net::desync::{Bundle, Evidence, Perturb};
use glasnost_net::simnet::{Link, SimNet, SplitMix};
use glasnost_net::wire::Hash;
use glasnost_net::{Config, Event, Params, Phase, Session, SlotKind, StartInfo, pump};
use glasnost_sim::econ::setup::{self, Player, Settings};
use glasnost_sim::orders::Order;
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

/// A person's scripted orders: now and then one of its units goes
/// somewhere.
fn scripted(g: &Game, house: HouseId, tick: u32, rng: &mut SplitMix) -> Vec<Order> {
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
        dest: Some(Target::Cell(fixture::at(dx, dy))),
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
}

struct Played {
    peers: Vec<Peer>,
    start: StartInfo,
}

/// Plays `persons` persons and a computer for [`TICKS`] ticks, `perturb`
/// made on the peer of its slot, then lets the messages settle.
fn play(persons: usize, seed: u64, perturb: Option<(u8, Perturb)>) -> Played {
    let map = map4();
    let setup = setup_of(persons, seed as u32);
    let rules = fixture::rules();
    let info = StartInfo::new(
        "test-build",
        &rules.hash,
        "synthetic-4",
        &setup,
        Params::default(),
    );
    let slots: Vec<u8> = info
        .slots
        .iter()
        .enumerate()
        .filter(|(_, s)| s.kind == SlotKind::Human)
        .map(|(i, _)| i as u8)
        .collect();
    assert_eq!(slots.len(), persons);
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
            let g = game(&map, &setup);
            let p = perturb.filter(|(s, _)| *s == slot).map(|(_, p)| p);
            Peer {
                slot,
                house: HouseId(info.slots[usize::from(slot)].house),
                session: Session::new(Config::new(TICK_MS), info.clone(), slot, 0)
                    .expect("a person's slot"),
                evidence: Evidence::new("test-build", &info, slot, &g).with_perturb(p),
                game: g,
                rng: SplitMix::new(seed * 31 + u64::from(slot)),
                next_due: 0,
                input_for: None,
                bundle: None,
                desyncs: Vec::new(),
                reports: Vec::new(),
                most_snapshots: 0,
                most_ticks: 0,
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
            if now >= p.next_due && p.session.phase() == Phase::Running && p.session.ran() < TICKS {
                let tick = p.session.ran();
                if p.input_for != Some(tick) {
                    p.input_for = Some(tick);
                    for o in scripted(&p.game, p.house, tick, &mut p.rng) {
                        p.session.submit(o);
                    }
                }
                p.evidence.before_tick(tick, &mut p.game);
                if let Some(t) = p.session.step(now, &mut p.game) {
                    p.evidence.after_tick(&t, &p.game, p.session.agreed());
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
            if p.bundle.is_none()
                && let Some(b) = p.evidence.take(p.session.ran())
            {
                let bytes = b.to_bytes();
                let tick = b.mismatch.expect("a mismatch").tick;
                p.session.set_report(tick, Bundle::digest(&bytes));
                p.bundle = Some((b, bytes));
            }
            p.most_snapshots = p.most_snapshots.max(p.evidence.snapshots().count());
            p.most_ticks = p.most_ticks.max(p.evidence.ticks_held());
        }
        if peers.iter().all(|p| p.session.ran() >= TICKS) {
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
    let played = play(3, 5, Some((1, perturb)));
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
    let a = played.peers[0].evidence.bundle_now().expect("a bundle");
    let b = played.peers[1].evidence.bundle_now().expect("a bundle");
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
