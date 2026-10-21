//! The client's network side against a real relay in the test process
//! (M9 N3): two windowless clients on the native link meet in a room,
//! chat, start, and play a synthetic skirmish in lockstep (invented rules
//! and map, `glasnost_ai::fixture`); a client of another build is refused
//! with the field named; a stalled peer is named, a dropped one noticed.
//! Our own netcode; no spec applies beyond the house order the setup cites.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use super::*;
use crate::mapview::load::Loaded;
use glasnost_ai::fixture;
use glasnost_convert::find::Listing;
use glasnost_fmt::mix::MixEntry;
use glasnost_map::Map;
use glasnost_net::Phase;
use glasnost_relay::{Entropy, Limits, Relay};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::Target;
use glasnost_ui::setup::{MapEntry, SetupRules, SlotKind};
use glasnost_vfs::ArchiveId;

/// The ticks both games run and compare.
const TICKS: i32 = 300;

/// The synthetic map's content hash as both clients name it.
const MAP_HASH: &str = "map-synthetic-8";

struct Fake(u64);

impl Entropy for Fake {
    fn fill(&mut self, buf: &mut [u8]) {
        for b in buf {
            // xorshift64
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            *b = (self.0 >> 24) as u8;
        }
    }
}

/// A relay on 127.0.0.1 in its own runtime, which serves it until dropped.
fn relay(seed: u64) -> (tokio::runtime::Runtime, Relay) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("a runtime");
    let r = rt
        .block_on(glasnost_relay::bind_with(
            "127.0.0.1:0",
            Limits::default(),
            Box::new(Fake(seed)),
        ))
        .expect("the relay binds");
    (rt, r)
}

/// The synthetic skirmish map with six more start waypoints.
fn map8() -> Map {
    let mut m = fixture::map();
    for (k, dx, dy) in [
        (2u16, -14, -18),
        (3, 14, 18),
        (4, -14, 18),
        (5, 14, -18),
        (6, 0, -24),
        (7, 0, 24),
    ] {
        let c = fixture::at(dx, dy);
        m.waypoints.insert(k, (c.x as i16, c.y as i16));
    }
    m
}

fn install() -> Install {
    let file = FileRef {
        archive: ArchiveId(0),
        entry: MixEntry {
            id: 8,
            offset: 0,
            size: 0,
        },
    };
    Install {
        setup: SetupRules {
            money: 2000,
            money_range: (1000, 5000, 100),
            units: 3,
            units_range: (1, 9),
            bases: true,
            flags: [false, true, true, true, true],
            // The fast speeds: 16 ms ticks.
            game_speed: 1,
            countries: vec!["Alphaland".into(), "Betaland".into()],
            colours: vec![[200, 0, 0, 255]; 8],
        },
        maps: vec![(
            Listing {
                file,
                archive: "synthetic".into(),
                name: "synthetic-8".into(),
                theater: "x".into(),
                multiplayer: true,
            },
            MapEntry {
                label: "synthetic-8".into(),
                starts: 8,
            },
        )],
        map_hashes: vec![MAP_HASH.into()],
        rules_hash: fixture::rules().hash.clone(),
        names: Default::default(),
        sounds: None,
        speech: None,
        bank: Default::default(),
        summary: "synthetic".into(),
    }
}

fn client(relay: &Relay, name: &str) -> Client {
    let mut c = Client::new();
    c.cpu_map = false;
    c.done(Done::Prepared(Vfs::default(), Ok(Box::new(install()))));
    c.netform.relay = format!("ws://{}/", relay.addr);
    c.netform.name = name.into();
    c
}

fn click(c: &mut Client, r: Rect) {
    let p = r.centre();
    c.pointer_down(Button::Left, p);
    c.pointer_up(Button::Left, p, Modifiers::default());
}

/// Draws a frame (laying out the targets) and clicks a menu, multiplayer
/// or room target.
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

/// The start job a client hands out, run on the synthetic data: the game
/// of the room's setup on the synthetic map, played from the slot.
fn run_start(c: &mut Client) {
    let Some(Job::Start {
        vfs,
        lobby,
        offered,
        seed,
        local,
        ..
    }) = c.job.take()
    else {
        return;
    };
    let gs = ui_setup::game_setup(&lobby, &fixture::rules(), &offered, seed, seed)
        .expect("the room's setup makes a game setup");
    let s = Loaded::bare(map8(), fixture::rules(), fixture::catalogue(), &gs)
        .and_then(|l| Session::bare(l, MAP_HASH.into(), local));
    c.done(Done::Started(vfs, Box::new(s), SoundBank::new()));
}

struct Clock(Instant);

impl Clock {
    fn now(&self) -> i64 {
        self.0.elapsed().as_millis() as i64
    }
}

/// Updates every client until `done` holds of them.
fn pump(
    clock: &Clock,
    clients: &mut [&mut Client],
    what: &str,
    mut done: impl FnMut(&[&mut Client]) -> bool,
) {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        for c in clients.iter_mut() {
            c.update(clock.now());
            run_start(c);
        }
        if done(clients) {
            return;
        }
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// A person's scripted orders at `tick`: now and then one of the house's
/// units is sent somewhere, through the client's order path.
fn script(c: &mut Client, tick: i32, n: &mut u64) -> usize {
    if tick % 7 != 3 {
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
        .map(|(&id, _)| id)
        .collect();
    if units.is_empty() {
        return 0;
    }
    *n = n
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let id = units[(*n >> 33) as usize % units.len()];
    let dx = ((*n >> 20) % 41) as i32 - 20;
    let dy = ((*n >> 40) % 41) as i32 - 20;
    s.play.issue(vec![Order::Behaviour {
        actor: id,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(fixture::at(dx, dy))),
    }])
}

/// Two persons and a computer meet in a room, chat, start and play 300
/// ticks with orders; their state hashes agree at every tick. A peer that
/// stops is named in the other's banner; once it goes, the other shows
/// that it left.
#[test]
fn two_clients_meet_chat_and_play_in_lockstep() {
    let (_rt, relay) = relay(31);
    let clock = Clock(Instant::now());
    let mut a = client(&relay, "Ann");
    let mut b = client(&relay, "Bea");

    // The host creates a room.
    press_menu(&mut a, MenuHit::Multiplayer);
    assert_eq!(a.shell.screen, Screen::Multiplayer);
    press_net(&mut a, NetHit::Host);
    pump(&clock, &mut [&mut a], "the room", |c| {
        c[0].room_code().is_some()
    });
    assert_eq!(a.shell.screen, Screen::Room);
    let code = a.room_code().unwrap().to_owned();
    // Row 2 opens for a person, row 3 takes a computer.
    press_room(&mut a, SetupHit::Kind(1, -1));
    pump(&clock, &mut [&mut a], "an open row", |c| {
        c[0].room().unwrap().setup.slots[1].kind == SlotKind::Human
    });
    press_room(&mut a, SetupHit::Kind(2, -1));
    pump(&clock, &mut [&mut a], "a computer row", |c| {
        c[0].room().unwrap().setup.slots[2].kind == SlotKind::Ai
    });

    // The other types the code and joins.
    press_menu(&mut b, MenuHit::Multiplayer);
    press_net(&mut b, NetHit::Field(glasnost_ui::netmenu::NetField::Code));
    b.typed(&code.to_lowercase());
    assert_eq!(b.netform.code, code.to_lowercase());
    press_net(&mut b, NetHit::Join);
    pump(&clock, &mut [&mut a, &mut b], "the join", |c| {
        c.iter()
            .all(|c| c.room().is_some_and(|v| v.members.len() == 2))
    });
    assert_eq!(b.shell.screen, Screen::Room);
    assert_eq!(b.room().unwrap().code, code);

    // A chat line reaches both, the sender too.
    let _ = b.frame();
    let field = b.chat_field.expect("the room's chat field");
    click(&mut b, field);
    b.typed("hello there");
    b.key(Key::Char('s'), true);
    b.key(Key::Enter, true);
    let said = |c: &Client| {
        c.chat_lines()
            .iter()
            .any(|l| l.who == "Bea" && l.text == "hello there" && !l.notice)
    };
    pump(&clock, &mut [&mut a, &mut b], "the chat line", |c| {
        c.iter().all(|c| said(c))
    });
    assert!(b.net.as_ref().unwrap().input.is_empty());

    // Ready, then the host starts.
    press_room(&mut b, SetupHit::Start_);
    pump(&clock, &mut [&mut a, &mut b], "ready", |c| {
        c[0].room()
            .unwrap()
            .members
            .iter()
            .all(|m| m.ready || m.row == 0)
    });
    press_room(&mut a, SetupHit::Start_);
    pump(&clock, &mut [&mut a, &mut b], "the start", |c| {
        c.iter()
            .all(|c| c.lockstep().is_some_and(|ls| ls.phase() == Phase::Running))
    });
    for c in [&a, &b] {
        assert_eq!(c.shell.screen, Screen::Game);
        let s = c.session.as_ref().unwrap();
        assert!(s.network);
        assert!(s.play.outbox.is_some());
    }
    assert_ne!(
        a.session.as_ref().unwrap().play.local,
        b.session.as_ref().unwrap().play.local,
        "each plays its own house"
    );

    // Both play; every tick's state hash is kept.
    let mut hashes = [BTreeMap::new(), BTreeMap::new()];
    let mut issued = [0, 0];
    let mut rng = [1u64, 2u64];
    let deadline = Instant::now() + Duration::from_secs(60);
    while hashes.iter().any(|h| h.len() < TICKS as usize) {
        for (k, c) in [&mut a, &mut b].into_iter().enumerate() {
            c.update(clock.now());
            let w = &c.session.as_ref().unwrap().play.game.world;
            let t = w.tick;
            if !hashes[k].contains_key(&t) && t > 0 {
                hashes[k].insert(t, glasnost_sim::save::state_hash(w));
                issued[k] += script(c, t, &mut rng[k]);
            }
        }
        assert!(Instant::now() < deadline, "the games stalled: {hashes:?}");
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(
        issued.iter().all(|&n| n > 10),
        "orders were given: {issued:?}"
    );
    for t in 1..=TICKS {
        assert_eq!(hashes[0].get(&t), hashes[1].get(&t), "tick {t}");
    }

    // The other stops: after a while the host names whom it waits for.
    let stop = Instant::now();
    while stop.elapsed() < Duration::from_millis(1_600) {
        a.update(clock.now());
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(a.net_banner().as_deref(), Some("Waiting for Bea..."));
    let _ = a.frame();

    // It goes: the host shows that it left, and plays on.
    drop(b);
    pump(&clock, &mut [&mut a], "the drop notice", |c| {
        c[0].chat_lines()
            .iter()
            .any(|l| l.notice && l.text == "Bea left the game.")
    });
    let at = a.session.as_ref().unwrap().play.game.world.tick;
    pump(&clock, &mut [&mut a], "ticks after the drop", |c| {
        c[0].session.as_ref().unwrap().play.game.world.tick > at + 10
    });
    assert_eq!(a.net_banner(), None);
    relay.stop();
}

/// A client of another build is refused when it joins, and the refusal
/// names the build; it stays on the multiplayer screen.
#[test]
fn a_join_of_another_build_is_refused_on_that_field() {
    let (_rt, relay) = relay(5);
    let clock = Clock(Instant::now());
    let mut a = client(&relay, "Ann");
    let mut b = client(&relay, "Bea");
    b.build = "glasnost another".into();
    press_menu(&mut a, MenuHit::Multiplayer);
    press_net(&mut a, NetHit::Host);
    pump(&clock, &mut [&mut a], "the room", |c| {
        c[0].room_code().is_some()
    });
    press_room(&mut a, SetupHit::Kind(1, -1));
    pump(&clock, &mut [&mut a], "an open row", |c| {
        c[0].room().unwrap().setup.slots[1].kind == SlotKind::Human
    });
    press_menu(&mut b, MenuHit::Multiplayer);
    b.netform.code = a.room_code().unwrap().to_owned();
    press_net(&mut b, NetHit::Join);
    pump(&clock, &mut [&mut a, &mut b], "the refusal", |c| {
        !c[1].networked()
    });
    assert_eq!(b.shell.screen, Screen::Multiplayer);
    let (text, bad) = &b.net_status;
    assert!(bad);
    assert_eq!(
        text,
        "The relay refused: the build differs from the room's."
    );
    // The screen shows it.
    let (d, _) = b.frame();
    assert!(d.texts().iter().any(|t| t.contains("build differs")));
    // A code that is no code is refused on the screen.
    b.netform.code = "nope".into();
    press_net(&mut b, NetHit::Join);
    assert!(!b.networked());
    assert!(b.net_status.0.contains("not a room code"));
    relay.stop();
}

/// A relay that is not there: the multiplayer screen says so.
#[test]
fn an_unreachable_relay_is_reported() {
    let clock = Clock(Instant::now());
    let mut c = Client::new();
    c.done(Done::Prepared(Vfs::default(), Ok(Box::new(install()))));
    // A port nothing listens on: bind one and let it go.
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .and_then(|l| l.local_addr())
        .map(|a| a.port())
        .expect("a free port");
    c.netform.relay = format!("ws://127.0.0.1:{port}/");
    press_menu(&mut c, MenuHit::Multiplayer);
    press_net(&mut c, NetHit::Host);
    pump(&clock, &mut [&mut c], "the failure", |c| !c[0].networked());
    assert_eq!(c.shell.screen, Screen::Multiplayer);
    assert!(c.net_status.1);
    assert!(
        c.net_status
            .0
            .starts_with("The connection closed: cannot reach the relay")
    );
}
