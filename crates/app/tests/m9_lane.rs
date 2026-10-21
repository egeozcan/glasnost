//! Local lane for the M9 network client (never in CI): with
//! `GLASNOST_GAME_DIR` set, starts the relay binary (`cargo run --release
//! -p glasnost-relay`) on a free loopback port, mounts the install in two
//! windowless clients on the native link, and has them meet in a room,
//! chat, start a game on the first offered map and play it in lockstep with
//! scripted move orders, comparing their state hashes at every tick. Writes
//! each client's room and game screens to `local-lane/m9/n3/` (git-ignored;
//! `GLASNOST_LANE_OUT` overrides the folder). Prints counts and timings
//! only, never map names, type names or rules values.
//!
//! Run with `cargo test --release -p glasnost-app --test m9_lane --
//! --ignored --nocapture`. `GLASNOST_LANE_TICKS=n` plays n ticks (default
//! 400); `GLASNOST_UI_MAP=n` picks the n-th offered map. Our own netcode.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use std::process::{Child, Command, Stdio};
use std::rc::Rc;
use std::time::{Duration, Instant};

use glasnost_app::game::client::{Client, run_job};
use glasnost_app::game::play::Button;
use glasnost_app::mapview::png;
use glasnost_map::CellXY;
use glasnost_net::Phase;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::Modifiers;
use glasnost_sim::world::Target;
use glasnost_ui::geom::{Point, Rect};
use glasnost_ui::menu::MenuHit;
use glasnost_ui::netmenu::NetHit;
use glasnost_ui::setup::{SetupHit, SlotKind};
use glasnost_ui::shell::Screen;
use glasnost_vfs::source::{FileSource, Unopened};
use glasnost_vfs::{ByteSource, MountOptions, PickedFile};

fn files(dir: &std::path::Path) -> Vec<PickedFile> {
    let ext = MountOptions::default().archive_extensions;
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let Ok(meta) = e.metadata() else { continue };
        if !meta.is_file() {
            continue;
        }
        let name = e.file_name().to_string_lossy().into_owned();
        let x = name
            .rsplit_once('.')
            .map(|(_, x)| x.to_ascii_lowercase())
            .unwrap_or_default();
        let source: Rc<dyn ByteSource> = if ext.contains(&x) {
            match FileSource::open(&e.path()) {
                Ok(f) => Rc::new(f),
                Err(_) => continue,
            }
        } else {
            Rc::new(Unopened(meta.len()))
        };
        out.push(PickedFile { path: name, source });
    }
    out
}

fn env_num(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Runs the jobs a client handed out (a mount, a map load), timing them.
fn run_jobs(c: &mut Client) -> Duration {
    let t = Instant::now();
    while let Some(job) = c.job.take() {
        let done = pollster::block_on(run_job(job));
        c.done(done);
    }
    t.elapsed()
}

fn click(c: &mut Client, r: Rect) {
    let p = Point::new(r.x + r.w / 2, r.y + r.h / 2);
    c.pointer_moved(p);
    c.pointer_down(Button::Left, p);
    c.pointer_up(Button::Left, p, Modifiers::default());
}

fn press_menu(c: &mut Client, hit: MenuHit) {
    let _ = c.frame();
    let r = c.menu_targets.rect_of(&hit).expect("a menu target");
    click(c, r);
}

fn press_net(c: &mut Client, hit: NetHit) {
    let _ = c.frame();
    let r = c.net_targets.rect_of(&hit).expect("a multiplayer target");
    click(c, r);
}

fn press_room(c: &mut Client, hit: SetupHit) {
    let _ = c.frame();
    let r = c.setup_targets.rect_of(&hit).expect("a room target");
    click(c, r);
}

/// The relay binary, killed when dropped.
struct RelayProcess(Child);

impl Drop for RelayProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Starts `cargo run --release -p glasnost-relay` on a free loopback port
/// and waits until it accepts connections.
fn start_relay() -> (RelayProcess, String) {
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .and_then(|l| l.local_addr())
        .expect("a free port")
        .port();
    let addr = format!("127.0.0.1:{port}");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let child = Command::new(cargo)
        .args(["run", "--release", "-q", "-p", "glasnost-relay", "--"])
        .args(["--listen", &addr])
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .stdin(Stdio::null())
        .spawn()
        .expect("cargo runs the relay");
    let relay = RelayProcess(child);
    let deadline = Instant::now() + Duration::from_secs(600);
    while std::net::TcpStream::connect(&addr).is_err() {
        assert!(Instant::now() < deadline, "the relay never listened");
        std::thread::sleep(Duration::from_millis(50));
    }
    (relay, format!("ws://{addr}/"))
}

struct Clock(Instant);

impl Clock {
    fn now(&self) -> i64 {
        self.0.elapsed().as_millis() as i64
    }
}

/// Updates both clients (running their jobs) until `done` holds; returns
/// how long it took.
fn pump(
    clock: &Clock,
    clients: &mut [&mut Client],
    what: &str,
    mut done: impl FnMut(&[&mut Client]) -> bool,
) -> Duration {
    let t = Instant::now();
    let deadline = t + Duration::from_secs(120);
    loop {
        for c in clients.iter_mut() {
            c.update(clock.now());
            run_jobs(c);
        }
        if done(clients) {
            return t.elapsed();
        }
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn save(c: &mut Client, dir: &std::path::Path, name: &str) {
    let rgba = c.frame_rgba();
    let (w, h) = c.size;
    std::fs::write(
        dir.join(format!("{name}.png")),
        png::png(w as u32, h as u32, &rgba),
    )
    .unwrap();
    println!("wrote {name}.png");
}

/// A person's scripted orders at `tick`: now and then one of the house's
/// units is sent a few cells away, through the client's order path.
fn script(c: &mut Client, tick: i32, n: &mut u64) -> usize {
    if tick % 9 != 4 {
        return 0;
    }
    let Some(s) = c.session.as_mut() else {
        return 0;
    };
    let house = s.play.local;
    let units: Vec<_> = s
        .play
        .game
        .world
        .objects
        .iter()
        .filter(|(_, o)| {
            o.owner == Some(house)
                && o.on_map
                && o.health > 0
                && matches!(o.kind, Kind::Vehicle | Kind::Infantry)
        })
        .map(|(&id, o)| (id, o.pos.cell()))
        .collect();
    if units.is_empty() {
        return 0;
    }
    *n = n
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let (id, at) = units[(*n >> 33) as usize % units.len()];
    let dx = ((*n >> 20) % 13) as i32 - 6;
    let dy = ((*n >> 40) % 13) as i32 - 6;
    s.play.issue(vec![Order::Behaviour {
        actor: id,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(CellXY::new(
            (at.x + dx).max(0),
            (at.y + dy).max(0),
        ))),
    }])
}

#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR and builds the relay"]
fn two_native_clients_play_through_the_relay() {
    let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
        println!("GLASNOST_GAME_DIR not set; skipped");
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    let out = png::lane_out("m9/n3");
    std::fs::create_dir_all(&out).unwrap();
    let ticks = env_num("GLASNOST_LANE_TICKS", 400) as i32;

    let t = Instant::now();
    let (_relay, url) = start_relay();
    println!("relay listening after {} ms", t.elapsed().as_millis());

    let mut clients = Vec::new();
    for name in ["Ann", "Bea"] {
        let mut c = Client::new();
        c.resize((1280, 800));
        c.mount(files(&dir));
        let took = run_jobs(&mut c);
        assert!(c.install.is_some(), "the install reads: {}", c.status);
        println!("{name}: install mounted in {} ms", took.as_millis());
        c.netform.relay = url.clone();
        c.netform.name = name.into();
        clients.push(c);
    }
    let mut b = clients.pop().unwrap();
    let mut a = clients.pop().unwrap();
    let maps = a.install.as_ref().unwrap().maps.len();
    println!("maps offered: {maps}");
    let clock = Clock(Instant::now());

    // The host creates a room, opens row 2 for a person and picks a map.
    press_menu(&mut a, MenuHit::Multiplayer);
    press_net(&mut a, NetHit::Host);
    let took = pump(&clock, &mut [&mut a], "the room", |c| {
        c[0].room_code().is_some()
    });
    println!("room created in {} ms", took.as_millis());
    for _ in 0..4 {
        if a.room().unwrap().setup.slots[1].kind == SlotKind::Human {
            break;
        }
        let before = a.room().unwrap().setup.slots[1].kind;
        press_room(&mut a, SetupHit::Kind(1, -1));
        pump(&clock, &mut [&mut a], "a row change", |c| {
            c[0].room().unwrap().setup.slots[1].kind != before
        });
    }
    assert_eq!(a.room().unwrap().setup.slots[1].kind, SlotKind::Human);
    for _ in 0..env_num("GLASNOST_UI_MAP", 0).min(maps - 1) {
        press_room(&mut a, SetupHit::Map(1));
        pump(&clock, &mut [&mut a], "a map change", |_| true);
    }
    let code = a.room_code().unwrap().to_owned();

    // The other joins by the code.
    press_menu(&mut b, MenuHit::Multiplayer);
    b.netform.code = code.clone();
    press_net(&mut b, NetHit::Join);
    let took = pump(&clock, &mut [&mut a, &mut b], "the join", |c| {
        c.iter()
            .all(|c| c.room().is_some_and(|v| v.members.len() == 2))
    });
    println!("joined in {} ms", took.as_millis());

    // A chat line each way.
    a.say("hello from the host");
    b.say("hello from the guest");
    let took = pump(&clock, &mut [&mut a, &mut b], "the chat", |c| {
        c.iter().all(|c| {
            let lines = c.chat_lines();
            lines.iter().filter(|l| !l.notice).count() >= 2
        })
    });
    println!("chat lines arrived in {} ms", took.as_millis());
    save(&mut a, &out, "room-host");
    save(&mut b, &out, "room-guest");

    // Ready, then the host starts; both load the map.
    press_room(&mut b, SetupHit::Start_);
    pump(&clock, &mut [&mut a, &mut b], "ready", |c| {
        c[0].room()
            .unwrap()
            .members
            .iter()
            .all(|m| m.ready || m.row == 0)
    });
    press_room(&mut a, SetupHit::Start_);
    let took = pump(&clock, &mut [&mut a, &mut b], "the start", |c| {
        c.iter()
            .all(|c| c.lockstep().is_some_and(|ls| ls.phase() == Phase::Running))
    });
    println!("both games loaded and running in {} ms", took.as_millis());
    for c in [&a, &b] {
        assert_eq!(c.shell.screen, Screen::Game, "{}", c.status);
    }

    // Both play; every tick's state hash is kept.
    let mut hashes = [BTreeMap::new(), BTreeMap::new()];
    let mut issued = [0, 0];
    let mut rng = [1u64, 2u64];
    let mut frames = [0u32, 0];
    let t = Instant::now();
    let deadline = t + Duration::from_secs(600);
    while hashes.iter().any(|h| h.len() < ticks as usize) {
        for (k, c) in [&mut a, &mut b].into_iter().enumerate() {
            c.update(clock.now());
            let w = &c.session.as_ref().unwrap().play.game.world;
            let tick = w.tick;
            if !hashes[k].contains_key(&tick) && tick > 0 {
                hashes[k].insert(tick, glasnost_sim::save::state_hash(w));
                issued[k] += script(c, tick, &mut rng[k]);
            }
            // A frame now and then, as a window would draw.
            if tick % 30 == 0 && frames[k] < tick as u32 / 30 + 1 {
                frames[k] += 1;
                let _ = c.frame();
            }
        }
        assert!(Instant::now() < deadline, "the games stalled");
        std::thread::sleep(Duration::from_millis(1));
    }
    let played = t.elapsed();
    let agree = (1..=ticks)
        .filter(|t| hashes[0].get(t) == hashes[1].get(t))
        .count();
    println!(
        "played {ticks} ticks in {} ms ({} ticks/s); orders issued {issued:?}; \
         hashes agree on {agree} of {ticks} ticks",
        played.as_millis(),
        u128::from(ticks as u32) * 1000 / played.as_millis().max(1),
    );
    save(&mut a, &out, "game-host");
    save(&mut b, &out, "game-guest");
    assert_eq!(agree, ticks as usize, "the games went apart");
    assert!(issued.iter().all(|&n| n > 0), "orders were given");

    // The guest leaves; the host is told.
    let notices = |c: &Client| c.chat_lines().iter().filter(|l| l.notice).count();
    let before = notices(&a);
    drop(b);
    let took = pump(&clock, &mut [&mut a], "the drop notice", |c| {
        notices(c[0]) > before
    });
    println!("the leave was noticed in {} ms", took.as_millis());
    save(&mut a, &out, "game-host-after-leave");
}
