//! Pathing reports end to end on the synthetic skirmish (M8 A4): the
//! client model keeps its ring while the person plays, saves a report on
//! the key's call, and the tool replays it, reproduces the capture's hash
//! and names a unit that was refused a step and what refused it; a damaged
//! or mismatched report is refused with a clear reason. Invented data, no
//! game files. Our own diagnostics; no spec applies.

use std::sync::Mutex;

use glasnost_ai::fixture;
use glasnost_ai::game::Game;
use glasnost_app::game::capture::{self, Bundle, Look};
use glasnost_app::game::play::{Play, local_house};
use glasnost_sim::econ::setup::{self, Player, Settings};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target};
use glasnost_tools::desync::Inputs;
use glasnost_tools::path_report::{self, Options};

/// The reports the test store keeps.
static SAVED: Mutex<Vec<(String, Vec<u8>)>> = Mutex::new(Vec::new());

fn save_test(name: &str, bytes: &[u8]) -> Result<String, String> {
    SAVED
        .lock()
        .expect("the store")
        .push((name.into(), bytes.to_vec()));
    Ok(format!("the test store as {name}"))
}

fn refuse(_: &str, _: &[u8]) -> Result<String, String> {
    Err("the disk is full".into())
}

/// The person's game on the cramped map: a clearing whose only way out is
/// a land bridge three cells wide, with eight of the person's vehicles in
/// it.
fn start() -> Play {
    let rules = fixture::rules();
    let players = [
        Player {
            name: "Person".into(),
            country: "Alphaland".into(),
            ..Default::default()
        },
        Player {
            name: "Machine".into(),
            country: "Betaland".into(),
            ai: true,
            ..Default::default()
        },
    ];
    let s = setup::skirmish(&rules, &players, Settings::default(), 0, 7);
    let local = local_house(&s).expect("one person's house");
    let mut game = Game::new(
        rules,
        &fixture::map_cramped(4),
        "synthetic-cramped",
        fixture::catalogue(),
        s,
    )
    .expect("the synthetic skirmish loads");
    let pocket = fixture::at(-fixture::START_DX, 0);
    for dx in -2..=2 {
        for dy in [-2, 2] {
            if dx % 2 == 0 || dy < 0 {
                let cell = glasnost_map::CellXY::new(pocket.x + dx, pocket.y + dy);
                testkit::spawn(&mut game.world, Kind::Vehicle, "GUNR", local.0, cell, 0);
            }
        }
    }
    Play::new(game, Some(local))
}

fn step(p: &mut Play, n: i32) {
    for _ in 0..n {
        assert!(p.step_by(|g| {
            g.step();
            true
        }));
    }
}

fn mobile(p: &Play) -> Vec<ObjectId> {
    p.game
        .world
        .objects
        .values()
        .filter(|o| {
            o.owner == p.local && o.on_map && matches!(o.kind, Kind::Vehicle | Kind::Infantry)
        })
        .map(|o| o.id)
        .collect()
}

/// Plays a while, sends every unit of the person's through the bridge at
/// once, marks, plays on and saves a report.
fn played() -> (Play, Bundle) {
    let mut p = start();
    step(&mut p, 30);
    let units = mobile(&p);
    assert!(units.len() >= 2, "the person has units");
    let pocket = fixture::at(-fixture::START_DX, 0);
    let goal = glasnost_map::CellXY::new(pocket.x + 14, pocket.y);
    p.selection = units.clone();
    let orders: Vec<Order> = units
        .iter()
        .map(|&actor| Order::Behaviour {
            actor,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(goal)),
        })
        .collect();
    assert!(p.issue(orders) > 0, "the moves are taken");
    step(&mut p, 40);
    let mark = p.game.world.tick;
    p.capture.mark(mark);
    step(&mut p, 120);
    let look = Look {
        local: p.local.map(|h| h.0),
        selection: p.selection.iter().map(|o| o.0).collect(),
        speed: p.capture.speed,
        ..Default::default()
    };
    let (text, b) = capture::save_report(&mut p, false, look, Some(save_test));
    let b = b.unwrap_or_else(|| panic!("no report: {text}"));
    assert!(text.contains("saved to the test store"), "{text}");
    assert_eq!(b.marks, vec![mark]);
    (p, b)
}

#[test]
fn a_saved_report_replays_to_its_capture_and_names_the_blocked() {
    let (p, b) = played();
    let saved = SAVED.lock().unwrap().clone();
    let (name, bytes) = saved
        .iter()
        .find(|(_, bytes)| Bundle::from_bytes(bytes).is_ok_and(|x| x == b))
        .expect("the store has the report");
    assert!(
        name.starts_with("path-") && name.ends_with(".gpath"),
        "{name}"
    );
    assert_eq!(&Bundle::from_bytes(bytes).unwrap(), &b);
    assert_eq!(b.capture_hash, p.game.state_hash());
    assert!(b.snapshot_ticks_run < b.capture_ticks_run);
    let inputs = Inputs::synthetic(&b.rules_hash).expect("the synthetic rules");
    let a = path_report::analyse(&b, &inputs, &Options::default()).expect("the report replays");
    assert!(a.reproduced, "{}", a.text);
    assert!(a.events > 0, "the trace recorded events");
    assert!(a.text.contains("Replay: reproduced"), "{}", a.text);
    assert!(a.png.starts_with(b"\x89PNG"));
    // A unit refused a step by another unit, named with its blocker.
    let line = a
        .text
        .lines()
        .find(|l| l.contains("was refused a step") && l.contains(": #"))
        .unwrap_or_else(|| panic!("no unit was refused a step by another:\n{}", a.text));
    let blocked: u32 = line
        .trim_start()
        .trim_start_matches('#')
        .split(' ')
        .next()
        .and_then(|s| s.parse().ok())
        .expect("the blocked unit's id");
    assert!(a.units.contains(&blocked));
    assert!(a.text.contains(&format!("== #{blocked} (")), "{}", a.text);
    assert!(a.text.contains("path computation"), "{}", a.text);
}

#[test]
fn the_tool_writes_its_outputs_and_dumps_a_tick() {
    let (_, b) = played();
    let dir = std::env::temp_dir().join(format!("glasnost-path-report-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(b.file_name(1));
    std::fs::write(&file, b.to_bytes()).unwrap();
    let tick = b.snapshot_tick + 5;
    let o = path_report::parse(&[
        file.display().to_string(),
        "--tick".into(),
        tick.to_string(),
    ])
    .unwrap();
    let (a, out) = path_report::run(&o, None).expect("the tool runs");
    assert!(a.reproduced);
    assert_eq!(out, dir.join(file.file_stem().unwrap()));
    assert!(out.join("report.txt").exists());
    assert!(
        std::fs::read(out.join("paths.png"))
            .unwrap()
            .starts_with(b"\x89PNG")
    );
    let dump = std::fs::read_to_string(out.join(format!("state-{tick}.txt"))).unwrap();
    assert!(dump.contains("pos.x = "), "{dump}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn damaged_and_mismatched_reports_are_refused() {
    let (_, b) = played();
    let bytes = b.to_bytes();
    let mut flipped = bytes.clone();
    let last = flipped.len() - 1;
    flipped[last] ^= 0x55;
    let e = Bundle::from_bytes(&flipped).unwrap_err();
    assert!(e.contains("damaged"), "{e}");
    let e = Bundle::from_bytes(&bytes[..6]).unwrap_err();
    assert!(e.contains("too short"), "{e}");
    let e = Bundle::from_bytes(b"GDSYwhatever, not ours").unwrap_err();
    assert!(e.contains("not a pathing report"), "{e}");
    let inputs = Inputs::synthetic(&b.rules_hash).unwrap();
    // Another build.
    let mut other = b.clone();
    other.build = "glasnost 0.0.0 (save format 1, game format 1)".into();
    let e = path_report::analyse(&other, &inputs, &Options::default())
        .err()
        .expect("another build is refused");
    assert!(e.0.contains("--any-build"), "{e}");
    // Other rules: no inputs for them, and the given ones are refused.
    let mut other = b.clone();
    other.rules_hash = "not-these-rules".into();
    assert!(path_report::inputs_for(&other, None).is_err());
    assert!(path_report::analyse(&other, &inputs, &Options::default()).is_err());
    // Another setup.
    let mut other = b.clone();
    other.setup_hash = "0".repeat(64);
    let e = path_report::analyse(&other, &inputs, &Options::default())
        .err()
        .expect("another setup is refused");
    assert!(e.0.contains("setup"), "{e}");
    // A capture hash the replay does not give: explained, not reproduced.
    let mut other = b.clone();
    other.capture_hash = "f".repeat(64);
    let a = path_report::analyse(&other, &inputs, &Options::default()).unwrap();
    assert!(!a.reproduced);
    assert!(a.text.contains("NOT reproduced"), "{}", a.text);
}

#[test]
fn the_ring_covers_the_window_and_a_failed_save_says_why() {
    let mut p = start();
    let window = capture::window_ticks(p.capture.speed);
    let gap = capture::spacing(p.capture.speed);
    step(&mut p, window + gap + 3);
    assert!(p.capture.covered(&p.game) >= window);
    assert!(p.capture.snapshots().count() <= capture::MAX_SNAPSHOTS);
    let (text, b) = capture::save_report(&mut p, false, Look::default(), Some(refuse));
    assert!(b.is_none());
    assert!(text.contains("the disk is full"), "{text}");
    let (text, b) = capture::save_report(&mut p, false, Look::default(), None);
    assert!(b.is_none());
    assert!(text.contains("cannot save"), "{text}");
}

/// The local lane (never in CI; `GLASNOST_GAME_DIR` names the install):
/// a skirmish on the install's first two-player map, every unit of the
/// person's sent to one cell at once (a crowd), a mark, a report saved,
/// and the tool run on it. Writes `report.txt`, `paths.png` and the
/// ring's cost (`ring-cost.txt`) to `local-lane/m8/a4-capture/`
/// (git-ignored): counts, timings, ids and cells only. The report file
/// itself (a save of the install's game) is deleted.
#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn lane_capture_on_the_install() {
    use std::time::Instant;
    let Some(dir) = std::env::var_os(glasnost_tools::local_lane::GAME_DIR_VAR) else {
        eprintln!("skipped: GLASNOST_GAME_DIR is not set (local-only lane)");
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let out = root.join("local-lane/m8/a4-capture");
    std::fs::create_dir_all(&out).unwrap();
    let (game, map_index) = pollster::block_on(async {
        let mut vfs = glasnost_tools::local_lane::mount(&dir)
            .await
            .expect("the install");
        let listings = glasnost_convert::find::find_maps(&vfs).await;
        for (index, listing) in listings.iter().enumerate() {
            let Ok(bytes) = vfs.read(&listing.file).await else {
                continue;
            };
            let hash = glasnost_net::start::content_hash(&bytes);
            let inputs = glasnost_convert::game::load(
                &mut vfs,
                bytes,
                "install".into(),
                glasnost_rules::layers::Layers::REFERENCE,
            )
            .await;
            let (Ok(resolved), Some(tiles)) = (inputs.resolved, inputs.tiles) else {
                continue;
            };
            if glasnost_ai::skirmish::start_cells(&inputs.map).len() != 2 {
                continue;
            }
            let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(
                &resolved.ruleset,
                &resolved.sim_rules_hash,
            ));
            let countries: Vec<String> = rules
                .country_names
                .iter()
                .zip(&rules.countries)
                .filter(|(n, c)| !c.passive && *n != "Neutral" && *n != "Special")
                .map(|(n, _)| n.clone())
                .collect();
            let players = [
                Player {
                    name: "Person".into(),
                    country: countries[0].clone(),
                    ..Default::default()
                },
                Player {
                    name: "Machine".into(),
                    country: countries[1 % countries.len()].clone(),
                    ai: true,
                    ..Default::default()
                },
            ];
            let s = setup::skirmish(&rules, &players, Settings::default(), 10, 3);
            let g = Game::new(rules, &inputs.map, &hash, std::sync::Arc::new(tiles), s)
                .expect("the skirmish loads");
            return (g, index);
        }
        panic!("no two-player map in the install");
    });
    let local = glasnost_sim::world::HouseId(0);
    let mut p = Play::new(game, Some(local));
    let speed = p.capture.speed;
    // Play, timing every tick's ring work, with and without a snapshot.
    let mut ring_ns: Vec<u128> = Vec::new();
    let mut snap_ns: Vec<u128> = Vec::new();
    let mut tick_ns: u128 = 0;
    let mut ticks = 0u32;
    let mut play = |p: &mut Play, n: i32| {
        for _ in 0..n {
            let t = Instant::now();
            p.game.step();
            tick_ns += t.elapsed().as_nanos();
            ticks += 1;
            let t = Instant::now();
            let took = p.capture.after_tick(&p.game);
            let e = t.elapsed().as_nanos();
            ring_ns.push(e);
            if took {
                snap_ns.push(e);
            }
        }
    };
    // Long enough for the ring to take and drop snapshots: a window and two
    // spacings.
    play(
        &mut p,
        capture::window_ticks(speed) + 2 * capture::spacing(speed),
    );
    let units = mobile(&p);
    assert!(!units.is_empty(), "the person has units");
    let first = p.game.world.objects[&units[0]].pos.cell();
    let goal = glasnost_map::CellXY::new(first.x + 12, first.y + 4);
    p.selection = units.clone();
    let orders: Vec<Order> = units
        .iter()
        .map(|&actor| Order::Behaviour {
            actor,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(goal)),
        })
        .collect();
    let taken = p.issue(orders);
    play(&mut p, 150);
    let mark = p.game.world.tick;
    p.capture.mark(mark);
    play(&mut p, 150);
    let look = Look {
        local: Some(0),
        selection: p.selection.iter().map(|o| o.0).collect(),
        speed,
        ..Default::default()
    };
    let t = Instant::now();
    let (_, b) = capture::save_report(&mut p, false, look, Some(save_test));
    let b = b.expect("a report");
    let bytes = b.to_bytes();
    let save_ms = t.elapsed().as_millis();
    let file = out.join(b.file_name(1));
    std::fs::write(&file, &bytes).unwrap();
    let t = Instant::now();
    let o = Options {
        file: file.clone(),
        out: Some(out.clone()),
        tick: Some(mark),
        ..Default::default()
    };
    let (a, _) = path_report::run(&o, Some(&dir)).expect("the tool runs");
    let tool_ms = t.elapsed().as_millis();
    let ring_total: u128 = ring_ns.iter().sum();
    let snap_max = snap_ns.iter().max().copied().unwrap_or(0);
    let snap_mean = snap_ns.iter().sum::<u128>() / (snap_ns.len().max(1) as u128);
    let ms = |ns: u128| ns as f64 / 1e6;
    let cost = format!(
        "map index {map_index}; speed index {speed}; window {} ticks, a snapshot every {} ticks\n\
         ticks played {ticks}; mean tick {:.3} ms\n\
         snapshots taken {}; a snapshot {:.2} ms mean, {:.2} ms most; the ring {:.4} ms a tick on average\n\
         ring held {} snapshots, {} bytes; covered {} ticks\n\
         orders taken {taken} for {} units; report {} bytes, made in {save_ms} ms; tool {tool_ms} ms; reproduced {}; trace events {}\n",
        capture::window_ticks(speed),
        capture::spacing(speed),
        ms(tick_ns) / f64::from(ticks),
        snap_ns.len(),
        ms(snap_mean),
        ms(snap_max),
        ms(ring_total) / f64::from(ticks),
        p.capture.snapshots().count(),
        p.capture.bytes_held(),
        p.capture.covered(&p.game),
        units.len(),
        bytes.len(),
        a.reproduced,
        a.events,
    );
    std::fs::write(out.join("ring-cost.txt"), &cost).unwrap();
    let _ = std::fs::remove_file(&file);
    eprint!("{cost}");
    assert!(a.reproduced, "the lane's report did not reproduce");
}
