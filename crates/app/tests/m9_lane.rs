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
//! The N5 lane (`a_killed_client_process_restarts_and_rejoins`) runs the
//! guest as a child process of the test, kills it mid-game, starts it again
//! on the same settings file and has it rejoin; it writes to
//! `local-lane/m9/n5/`.
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
    let house = s.play.me();
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

/// Whether a client shows a notice that `has` holds of.
fn notice(c: &Client, has: impl Fn(&str) -> bool) -> bool {
    c.chat_lines().iter().any(|l| l.notice && has(&l.text))
}

/// A variable a guest process is given.
fn var(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} is set"))
}

/// A mounted client on the install of `dir`.
fn mounted(dir: &std::path::Path, name: &str, url: &str) -> Client {
    let mut c = Client::new();
    c.resize((1280, 800));
    c.mount(files(dir));
    let took = run_jobs(&mut c);
    assert!(c.install.is_some(), "the install reads: {}", c.status);
    println!("{name}: install mounted in {} ms", took.as_millis());
    c.netform.relay = url.into();
    c.netform.name = name.into();
    c
}

/// The guest of [`a_killed_client_process_restarts_and_rejoins`] in a
/// process of its own (M9 N5): this test binary run again with
/// `GLASNOST_LANE_ROLE=guest`. It joins the room of `GLASNOST_LANE_CODE`,
/// or, when its settings file names a game to rejoin, presses
/// **Reconnect**; then it plays with orders until `GLASNOST_LANE_PLAY`
/// ticks after its readmission and writes every tick's state hash to
/// `GLASNOST_LANE_HASHES`. Its parent kills the first such process.
#[test]
#[ignore = "local lane: the guest process of the rejoin lane"]
fn lane_guest_process() {
    if std::env::var_os("GLASNOST_LANE_ROLE").is_none() {
        println!("not a guest process; skipped");
        return;
    }
    let dir = std::path::PathBuf::from(var("GLASNOST_GAME_DIR"));
    let mut c = mounted(&dir, "Bea", &var("GLASNOST_LANE_URL"));
    let store = glasnost_app::settings::FileStore(var("GLASNOST_LANE_SETTINGS").into());
    c.attach_store(Box::new(store));
    let play: i32 = var("GLASNOST_LANE_PLAY").parse().expect("a tick count");
    let clock = Clock(Instant::now());
    let t = Instant::now();
    press_menu(&mut c, MenuHit::Multiplayer);
    let rejoining = c.settings.rejoin.is_some();
    if rejoining {
        println!("guest: reconnecting");
        press_net(&mut c, NetHit::Reconnect);
    } else {
        c.netform.code = var("GLASNOST_LANE_CODE");
        press_net(&mut c, NetHit::Join);
        pump(&clock, &mut [&mut c], "the join", |c| c[0].room().is_some());
        press_room(&mut c, SetupHit::Start_);
    }
    let mut hashes = BTreeMap::new();
    let mut rng = 9u64;
    let mut loaded = None;
    let mut readmitted = None;
    let deadline = Instant::now() + Duration::from_secs(900);
    loop {
        c.update(clock.now());
        run_jobs(&mut c);
        if let (Some(s), Some(ls)) = (c.session.as_ref(), c.lockstep()) {
            let w = &s.play.game.world;
            let tick = w.tick;
            if rejoining && loaded.is_none() {
                let (_, bytes) = c.transfer_bytes();
                loaded = Some(t.elapsed());
                println!(
                    "guest: the game sent loaded after {} ms ({} bytes, tick {})",
                    t.elapsed().as_millis(),
                    bytes.unwrap_or(0),
                    ls.ran()
                );
            }
            let caught = !ls.catching_up() && ls.phase() == Phase::Running;
            if caught && readmitted.is_none() {
                readmitted = Some((t.elapsed(), tick));
                if let Some(l) = loaded {
                    println!(
                        "guest: readmitted at tick {tick} after {} ms; caught up in {} ms",
                        t.elapsed().as_millis(),
                        (t.elapsed() - l).as_millis()
                    );
                }
            }
            if tick > 0 && !hashes.contains_key(&tick) {
                hashes.insert(tick, glasnost_sim::save::state_hash(w));
                if caught {
                    script(&mut c, tick, &mut rng);
                }
            }
            if readmitted.is_some_and(|(_, at)| tick >= at + play) {
                break;
            }
        }
        assert!(Instant::now() < deadline, "the guest's game stalled");
        std::thread::sleep(Duration::from_millis(1));
    }
    if rejoining {
        save(&mut c, &png::lane_out("m9/n5"), "game-rejoined");
    }
    let text: String = hashes.iter().map(|(t, h)| format!("{t} {h}\n")).collect();
    std::fs::write(var("GLASNOST_LANE_HASHES"), text).expect("the hashes are written");
}

/// Starts a guest process ([`lane_guest_process`]).
fn spawn_guest(dir: &std::path::Path, url: &str, code: &str, out: &std::path::Path) -> Child {
    Command::new(std::env::current_exe().expect("the test binary"))
        .args(["--exact", "lane_guest_process", "--ignored", "--nocapture"])
        .args(["--test-threads", "1"])
        .env("GLASNOST_LANE_ROLE", "guest")
        .env("GLASNOST_GAME_DIR", dir)
        .env("GLASNOST_LANE_URL", url)
        .env("GLASNOST_LANE_CODE", code)
        .env("GLASNOST_LANE_SETTINGS", out.join("guest-settings.txt"))
        .env("GLASNOST_LANE_HASHES", out.join("guest-hashes.txt"))
        .env("GLASNOST_LANE_PLAY", "300")
        .stdin(Stdio::null())
        .spawn()
        .expect("the guest process starts")
}

/// M9 N5 on real data: a host in this process and a guest in a process of
/// its own play with a computer through the relay binary; the guest's
/// process is killed mid-game and restarted, finds the game in its
/// settings file, reconnects, catches up and plays on; the hashes of both
/// games agree on every tick both saw after the readmission. Prints counts,
/// timings and the bytes of the game sent; writes the host's and the
/// rejoined guest's game screens to `local-lane/m9/n5/`.
#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR and builds the relay"]
fn a_killed_client_process_restarts_and_rejoins() {
    let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
        println!("GLASNOST_GAME_DIR not set; skipped");
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    let out = png::lane_out("m9/n5");
    std::fs::create_dir_all(&out).unwrap();
    for f in ["guest-settings.txt", "guest-hashes.txt"] {
        let _ = std::fs::remove_file(out.join(f));
    }
    let kill_at = env_num("GLASNOST_LANE_TICKS", 300) as i32;

    let t = Instant::now();
    let (_relay, url) = start_relay();
    println!("relay listening after {} ms", t.elapsed().as_millis());
    let mut a = mounted(&dir, "Ann", &url);
    let clock = Clock(Instant::now());

    // A room with a row for the guest and one for a computer.
    press_menu(&mut a, MenuHit::Multiplayer);
    press_net(&mut a, NetHit::Host);
    pump(&clock, &mut [&mut a], "the room", |c| {
        c[0].room_code().is_some()
    });
    for (row, kind) in [(1, SlotKind::Human), (2, SlotKind::Ai)] {
        for _ in 0..4 {
            if a.room().unwrap().setup.slots[row].kind == kind {
                break;
            }
            let before = a.room().unwrap().setup.slots[row].kind;
            press_room(&mut a, SetupHit::Kind(row, -1));
            pump(&clock, &mut [&mut a], "a row change", |c| {
                c[0].room().unwrap().setup.slots[row].kind != before
            });
        }
        assert_eq!(a.room().unwrap().setup.slots[row].kind, kind);
    }
    let code = a.room_code().unwrap().to_owned();

    // The guest process joins and gets ready; the host starts.
    let mut guest = spawn_guest(&dir, &url, &code, &out);
    pump(&clock, &mut [&mut a], "the guest, ready", |c| {
        let v = c[0].room().unwrap();
        v.members.len() == 2 && v.members.iter().all(|m| m.ready || m.row == 0)
    });
    press_room(&mut a, SetupHit::Start_);
    let took = pump(&clock, &mut [&mut a], "the start", |c| {
        c[0].lockstep()
            .is_some_and(|ls| ls.phase() == Phase::Running)
    });
    println!("the game runs after {} ms", took.as_millis());

    // Both play; the host gives orders too.
    let mut hashes = BTreeMap::new();
    let mut rng = 1u64;
    let mut step = |a: &mut Client, hashes: &mut BTreeMap<i32, String>| {
        a.update(clock.now());
        run_jobs(a);
        let w = &a.session.as_ref().unwrap().play.game.world;
        let tick = w.tick;
        if tick > 0 && !hashes.contains_key(&tick) {
            hashes.insert(tick, glasnost_sim::save::state_hash(w));
            script(a, tick, &mut rng);
        }
        std::thread::sleep(Duration::from_millis(1));
        tick
    };
    let deadline = Instant::now() + Duration::from_secs(600);
    while step(&mut a, &mut hashes) < kill_at {
        assert!(Instant::now() < deadline, "the game stalled");
    }

    // The guest's process is killed; the host plays on with the computer.
    guest.kill().expect("the guest is killed");
    let _ = guest.wait();
    let killed = Instant::now();
    println!("guest process killed at host tick {kill_at}");
    while !notice(&a, |t| t.starts_with("Bea left the game; they have ")) {
        step(&mut a, &mut hashes);
        assert!(Instant::now() < deadline, "no drop notice");
    }
    println!(
        "the drop was noticed in {} ms",
        killed.elapsed().as_millis()
    );

    // A new guest process finds the game in its settings file and
    // reconnects; the host sends it the game.
    let spawned = Instant::now();
    let mut guest = spawn_guest(&dir, &url, &code, &out);
    let mut asked = None;
    let mut back = None;
    let status = loop {
        let tick = step(&mut a, &mut hashes);
        if asked.is_none()
            && notice(&a, |t| {
                t.ends_with("is reconnecting; sending them the game.")
            })
        {
            asked = Some(tick);
            println!(
                "the rejoin reached the host {} ms after the restart, at host tick {tick}",
                spawned.elapsed().as_millis()
            );
        }
        if back.is_none() && notice(&a, |t| t == "Bea reconnected.") {
            back = Some(tick);
            println!(
                "readmitted {} ms after the restart, at host tick {tick}; game sent: {} bytes",
                spawned.elapsed().as_millis(),
                a.transfer_bytes().0
            );
        }
        if let Some(s) = guest.try_wait().expect("the guest process") {
            break s;
        }
        assert!(Instant::now() < deadline, "the rejoined game stalled");
    };
    assert!(status.success(), "the rejoined guest: {status}");
    let back = back.expect("the guest was readmitted");
    save(&mut a, &out, "game-host-after-rejoin");

    // Every tick both saw after the readmission agrees.
    let theirs: BTreeMap<i32, String> = std::fs::read_to_string(out.join("guest-hashes.txt"))
        .expect("the guest's hashes")
        .lines()
        .filter_map(|l| {
            let (t, h) = l.split_once(' ')?;
            Some((t.parse().ok()?, h.to_owned()))
        })
        .collect();
    let both: Vec<i32> = theirs
        .keys()
        .copied()
        .filter(|t| hashes.contains_key(t))
        .collect();
    let agree = both.iter().filter(|t| hashes[t] == theirs[t]).count();
    let after = both.iter().filter(|&&t| t > back).count();
    println!(
        "hashes: both saw {} ticks ({after} after the readmission); they agree on {agree}",
        both.len()
    );
    assert_eq!(agree, both.len(), "the games went apart");
    assert!(after >= 200, "too few ticks after the readmission");
}
