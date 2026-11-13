//! The client's network side against a real relay in the test process
//! (M9 N3): two windowless clients on the native link meet in a room,
//! chat, start, and play a synthetic skirmish in lockstep (invented rules
//! and map, `glasnost_ai::fixture`); a client of another build is refused
//! with the field named; a stalled peer is named, a dropped one noticed.
//! Our own netcode; no spec applies beyond the house order the setup cites.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use super::*;
use crate::game::pump::{Pump, STALE_MS};
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
use glasnost_ui::hud::PauseHit;
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
            // The fastest speed: a network game's 16 ms ticks (S-0201
            // claim 17 and Formulas F5, index 0).
            game_speed: 0,
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
        map_info: Vec::new(),
        rules_hash: fixture::rules().hash.clone(),
        names: Default::default(),
        sounds: None,
        speech: None,
        bank: Default::default(),
        cursor_palette: Default::default(),
        strings: None,
        music: None,
        summary: "synthetic".into(),
    }
}

fn client(relay: &Relay, name: &str) -> Client {
    let mut c = Client::new();
    // The relay only, unless a test gives fake direct links (N7).
    c.direct = None;
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

/// The test's host clock. It also notes the longest gap between two
/// readings: every loop here reads it every millisecond or so, so a long
/// gap means the host did not run the test (a loaded machine), see
/// [`under_load`].
struct Clock {
    start: Instant,
    last: std::cell::Cell<i64>,
    gap: std::cell::Cell<i64>,
}

impl Clock {
    fn new() -> Self {
        Clock {
            start: Instant::now(),
            last: std::cell::Cell::new(0),
            gap: std::cell::Cell::new(0),
        }
    }

    fn now(&self) -> i64 {
        let t = self.start.elapsed().as_millis() as i64;
        self.gap.set(self.gap.get().max(t - self.last.get()));
        self.last.set(t);
        t
    }
}

/// Updates every client until `done` holds of them.
fn pump(
    clock: &Clock,
    clients: &mut [&mut Client],
    what: &str,
    done: impl FnMut(&[&mut Client]) -> bool,
) {
    pump_seen(clock, clients, what, |_, _| {}, done);
}

/// `pump`, calling `seen(k, client)` after each update of client `k`, so a
/// caller sees every tick that runs while it waits (the ticks go on while
/// a room is still starting up, and under load a peer runs a few ahead).
fn pump_seen(
    clock: &Clock,
    clients: &mut [&mut Client],
    what: &str,
    mut seen: impl FnMut(usize, &Client),
    mut done: impl FnMut(&[&mut Client]) -> bool,
) {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        for (k, c) in clients.iter_mut().enumerate() {
            c.update(clock.now());
            run_start(c);
            seen(k, c);
        }
        if done(clients) {
            return;
        }
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// Keeps the client's state hash for its current tick when it is a new
/// one (tick 0 is not a played tick); says whether it was new.
fn keep_hash(hashes: &mut BTreeMap<i32, String>, c: &Client) -> bool {
    let Some(s) = c.session.as_ref() else {
        return false;
    };
    let w = &s.play.game.world;
    if w.tick == 0 || hashes.contains_key(&w.tick) {
        return false;
    }
    hashes.insert(w.tick, glasnost_sim::save::state_hash(w));
    true
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
    let clock = Clock::new();
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
    // The ticks start running as soon as a peer's phase is Running, and under
    // load that peer can run several before the other is: every tick's
    // hash is kept from the first update on, not from the loop below.
    let mut hashes = [BTreeMap::new(), BTreeMap::new()];
    pump_seen(
        &clock,
        &mut [&mut a, &mut b],
        "the start",
        |k, c| {
            keep_hash(&mut hashes[k], c);
        },
        |c| {
            c.iter()
                .all(|c| c.lockstep().is_some_and(|ls| ls.phase() == Phase::Running))
        },
    );
    for c in [&a, &b] {
        assert_eq!(c.shell.screen, Screen::Game);
        let s = c.session.as_ref().unwrap();
        assert!(s.network);
        assert!(s.play.outbox.is_some());
    }
    assert_ne!(
        a.session.as_ref().unwrap().play.me(),
        b.session.as_ref().unwrap().play.me(),
        "each plays its own house"
    );

    // Both play; every tick's state hash is kept.
    let mut issued = [0, 0];
    let mut rng = [1u64, 2u64];
    let deadline = Instant::now() + Duration::from_secs(60);
    while hashes.iter().any(|h| h.len() < TICKS as usize) {
        for (k, c) in [&mut a, &mut b].into_iter().enumerate() {
            c.update(clock.now());
            let w = &c.session.as_ref().unwrap().play.game.world;
            let t = w.tick;
            if keep_hash(&mut hashes[k], c) {
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

    // Ann changes the speed through the options menu: the ticks go on
    // while it is open; at its close the speed order is carried out on
    // both machines in one tick, Bea's machine names the house whose player
    // changed it and Ann's does not, and both then pace the new index's
    // period.
    // spec: S-0201 claim 15 (the menu stops no tick in a network game) and claim 17 (sent when the menu closes; carried out in one tick on every machine; the message on every machine but the sender's; the cycles follow the new index's cap, Formulas F5)
    let from = a.session.as_ref().unwrap().play.game.world.game_speed;
    assert_eq!(from, 0);
    assert!(a.session.as_mut().unwrap().open_options());
    let opened = a.session.as_ref().unwrap().play.game.world.tick;
    pump(
        &clock,
        &mut [&mut a, &mut b],
        "ticks with the menu open",
        |c| c[0].session.as_ref().unwrap().play.game.world.tick > opened + 5,
    );
    for hit in [PauseHit::Speed(-1), PauseHit::Resume] {
        let _ = a.frame();
        let r = a.session.as_ref().unwrap().pause_targets().rect_of(&hit);
        click(&mut a, r.expect("an options menu target"));
    }
    assert!(!a.session.as_ref().unwrap().paused);
    let speed = |c: &Client| c.session.as_ref().unwrap().play.game.world.game_speed;
    pump(&clock, &mut [&mut a, &mut b], "the speed order", |c| {
        c.iter().all(|c| speed(c) == 1) && notice(c[1], |t| t == "Ann changed the game speed.")
    });
    assert!(!notice(&a, |t| t.contains("changed the game speed")));
    let at = [&a, &b].map(|c| c.session.as_ref().unwrap().play.game.world.tick);
    pump(
        &clock,
        &mut [&mut a, &mut b],
        "a tick at the new speed",
        |c| {
            c.iter()
                .zip(at)
                .all(|(c, t)| c.session.as_ref().unwrap().play.game.world.tick > t)
        },
    );
    for c in [&a, &b] {
        assert_eq!(c.session.as_ref().unwrap().speed, 1);
        assert_eq!(c.lockstep().unwrap().tick_ms(), 22, "⌊1000/45⌋ ms");
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
        c[0].chat_lines().iter().any(|l| {
            l.notice
                && l.text.starts_with("Bea left the game; they have ")
                && l.text.ends_with(" seconds to reconnect.")
        })
    });
    let at = a.session.as_ref().unwrap().play.game.world.tick;
    pump(&clock, &mut [&mut a], "ticks after the drop", |c| {
        c[0].session.as_ref().unwrap().play.game.world.tick > at + 10
    });
    assert_eq!(a.net_banner(), None);
    relay.stop();
}

/// Our own *Pathing and other fixes* (M8 step C7) in a room: on when the
/// host creates it, the host's box turns it off for every member, a guest
/// has no box to press, and every member's start carries the room's choice
/// into the lobby values its game loads with.
#[test]
fn the_host_sets_the_fixes_layer_for_the_room() {
    let (_rt, relay) = relay(83);
    let clock = Clock::new();
    let mut a = client(&relay, "Ann");
    let mut b = client(&relay, "Bea");
    press_menu(&mut a, MenuHit::Multiplayer);
    press_net(&mut a, NetHit::Host);
    pump(&clock, &mut [&mut a], "the room", |c| {
        c[0].room_code().is_some()
    });
    assert!(a.room().unwrap().setup.fixes, "on by default");
    press_room(&mut a, SetupHit::Kind(1, -1));
    pump(&clock, &mut [&mut a], "an open row", |c| {
        c[0].room().unwrap().setup.slots[1].kind == SlotKind::Human
    });
    press_menu(&mut b, MenuHit::Multiplayer);
    b.netform.code = a.room_code().unwrap().to_owned();
    press_net(&mut b, NetHit::Join);
    pump(&clock, &mut [&mut a, &mut b], "the join", |c| {
        c.iter()
            .all(|c| c.room().is_some_and(|v| v.members.len() == 2))
    });
    let _ = b.frame();
    assert!(
        b.setup_targets.rect_of(&SetupHit::Fixes).is_none(),
        "a guest does not set the layer"
    );
    press_room(&mut a, SetupHit::Fixes);
    pump(&clock, &mut [&mut a, &mut b], "the layer off", |c| {
        c.iter().all(|c| c.room().is_some_and(|v| !v.setup.fixes))
    });
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
        c.iter().all(|c| c.session.is_some())
    });
    for c in [&a, &b] {
        let begin = c.net.as_ref().unwrap().begin.as_ref().expect("the start");
        let install = c.install.as_ref().unwrap();
        let maps: Vec<MapEntry> = install.maps.iter().map(|(_, m)| m.clone()).collect();
        let lobby = begin
            .local_setup(0)
            .lobby(&install.setup, &maps)
            .expect("the room's setup starts");
        assert_eq!(lobby.layers, glasnost_rules::layers::Layers::REFERENCE);
    }
    relay.stop();
}

/// Two peers whose played rules differ only in our own fixes layer (M8 step
/// C7) are refused at the start, on the rules hash.
#[test]
fn a_peer_with_other_layers_is_refused_on_the_rules_hash() {
    use glasnost_rules::layers::Layers;
    use glasnost_rules::resolve::Source;
    let base = Source {
        layer: "base",
        file: "base.jsonc",
        text: r#"{"schema": 1}"#,
    };
    let hash = |layers: Layers| {
        let mut sources = vec![base];
        sources.extend(layers.sources());
        glasnost_rules::resolve_ruleset(None, &sources)
            .unwrap()
            .sim_rules_hash
    };
    let setup = fixture::setup(1);
    let info = |layers| {
        glasnost_net::start::StartInfo::new(
            "build",
            &hash(layers),
            MAP_HASH,
            &setup,
            Default::default(),
        )
    };
    let (plain, fixed) = (info(Layers::REFERENCE), info(Layers::PLAY_FIXES));
    assert_eq!(plain.mismatch(&info(Layers::REFERENCE)), None);
    assert_eq!(
        plain.mismatch(&fixed),
        Some(glasnost_net::start::Field::RulesHash)
    );
}

/// A client of another build is refused when it joins, and the refusal
/// names the build; it stays on the multiplayer screen.
#[test]
fn a_join_of_another_build_is_refused_on_that_field() {
    let (_rt, relay) = relay(5);
    let clock = Clock::new();
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
    // M9 N8b: the client names both builds from the room's peek, before
    // the relay has to refuse the join with only the field's name.
    assert_eq!(
        text,
        &format!(
            "That room runs {BUILD}; this is glasnost another. \
             Everybody must run the same build."
        )
    );
    // The screen shows it.
    let (d, _) = b.frame();
    assert!(d.texts().iter().any(|t| t.contains("That room runs")));
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
    let clock = Clock::new();
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

/// The desync reports the test store keeps: file name and bytes.
static SAVED: std::sync::Mutex<Vec<(String, Vec<u8>)>> = std::sync::Mutex::new(Vec::new());

/// The test's way to store a report (a `SaveReport`): the store.
fn save_test(name: &str, bytes: &[u8]) -> Result<String, String> {
    SAVED
        .lock()
        .expect("the store")
        .push((name.into(), bytes.to_vec()));
    Ok(format!("the test store as {name}"))
}

/// Whether a client shows a notice that `has` holds of.
fn notice(c: &Client, has: impl Fn(&str) -> bool) -> bool {
    c.chat_lines().iter().any(|l| l.notice && has(&l.text))
}

/// Two test clients, one of which changes its house's credits before
/// tick 100, play on through the desync (M9 N4): each notices it once,
/// saves a report through its store, tells the other the report's digest
/// and shows the other's. The two reports name the same mismatched
/// checkpoint, carry the start, the perturbation of the one that made it,
/// and the ticks to a few past the mismatch.
#[test]
fn a_desync_is_reported_by_both_with_their_digests() {
    use glasnost_net::desync::{Bundle, PAST_TICKS, Perturb};
    let (_rt, relay) = relay(77);
    let clock = Clock::new();
    let mut a = client(&relay, "Ann");
    let mut b = client(&relay, "Bea");
    let perturb = Perturb::Credits {
        tick: 100,
        house: 0,
        by: 1,
    };
    a.set_test(Some(perturb));
    b.set_test(None);
    assert_eq!(a.build, test_build());
    assert_ne!(a.build, BUILD, "a test client names another build");
    for c in [&mut a, &mut b] {
        c.save_report = Some(save_test);
    }

    press_menu(&mut a, MenuHit::Multiplayer);
    press_net(&mut a, NetHit::Host);
    pump(&clock, &mut [&mut a], "the room", |c| {
        c[0].room_code().is_some()
    });
    let code = a.room_code().unwrap().to_owned();
    press_room(&mut a, SetupHit::Kind(1, -1));
    pump(&clock, &mut [&mut a], "an open row", |c| {
        c[0].room().unwrap().setup.slots[1].kind == SlotKind::Human
    });
    press_room(&mut a, SetupHit::Kind(2, -1));
    pump(&clock, &mut [&mut a], "a computer row", |c| {
        c[0].room().unwrap().setup.slots[2].kind == SlotKind::Ai
    });
    press_menu(&mut b, MenuHit::Multiplayer);
    b.netform.code = code.clone();
    press_net(&mut b, NetHit::Join);
    pump(&clock, &mut [&mut a, &mut b], "the join", |c| {
        c.iter()
            .all(|c| c.room().is_some_and(|v| v.members.len() == 2))
    });
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

    // Both play on, with orders, until each has its report and the other's
    // digest.
    let mut rng = [3u64, 4u64];
    let mut last = [0, 0];
    let saved = |c: &Client| notice(c, |t| t.contains("report saved to the test store"));
    let heard = |c: &Client| notice(c, |t| t.contains("saved a desync report of tick"));
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        for (k, c) in [&mut a, &mut b].into_iter().enumerate() {
            c.update(clock.now());
            let t = c.session.as_ref().unwrap().play.game.world.tick;
            if t > last[k] {
                last[k] = t;
                script(c, t, &mut rng[k]);
            }
        }
        if [&a, &b].iter().all(|c| saved(c) && heard(c)) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "no reports: {:?} / {:?}",
            a.chat_lines(),
            b.chat_lines()
        );
        std::thread::sleep(Duration::from_millis(1));
    }

    // The two reports, by their names.
    let prefix = format!("desync-{code}-");
    let saved: Vec<(String, Vec<u8>)> = SAVED
        .lock()
        .unwrap()
        .iter()
        .filter(|(n, _)| n.starts_with(&prefix))
        .cloned()
        .collect();
    assert_eq!(saved.len(), 2, "one report each");
    let bundles: Vec<Bundle> = saved
        .iter()
        .map(|(_, bytes)| Bundle::from_bytes(bytes).expect("a report reads back"))
        .collect();
    let m = bundles[0].mismatch.expect("a mismatch");
    let every = bundles[0].start.params.sane().hash_every;
    assert!(m.tick >= 100 && (m.tick + 1).is_multiple_of(every), "{m:?}");
    assert!(
        m.tick < 100 + every,
        "the first checkpoint after the change"
    );
    for (r, (name, _)) in bundles.iter().zip(&saved) {
        assert_eq!(r.mismatch.map(|m| m.tick), Some(m.tick));
        assert_eq!(r.build, test_build());
        assert_eq!(r.start, bundles[0].start);
        assert_eq!(*name, r.file_name(&prefix));
        assert!(r.end() > m.tick + PAST_TICKS, "ticks past the mismatch");
        assert!(r.ticks.first().is_some_and(|t| t.tick < 100));
    }
    let of = |c: &Client| {
        let slot = c.lockstep().unwrap().me();
        bundles
            .iter()
            .zip(&saved)
            .find(|(r, _)| r.writer == slot)
            .map(|(r, (_, bytes))| (r, bytes))
            .expect("a report of the slot")
    };
    let (ra, bytes_a) = of(&a);
    let (rb, bytes_b) = of(&b);
    assert_eq!(
        ra.perturb,
        Some(perturb),
        "the one that changed its game says so"
    );
    assert_eq!(rb.perturb, None);
    assert_eq!(ra.mismatch.unwrap().mine, rb.mismatch.unwrap().theirs);
    assert_eq!(ra.mismatch.unwrap().theirs, rb.mismatch.unwrap().mine);

    // Each noticed the desync once, and shows the other's digest.
    let short = |bytes: &[u8]| glasnost_net::sim::hex(&Bundle::digest(bytes))[..12].to_owned();
    let went = format!("The games went apart at tick {} (a desync).", m.tick);
    for (c, other, theirs) in [(&a, "Bea", bytes_b), (&b, "Ann", bytes_a)] {
        let n = c
            .chat_lines()
            .iter()
            .filter(|l| l.notice && l.text == went)
            .count();
        assert_eq!(n, 1, "{:?}", c.chat_lines());
        let said = format!(
            "{other} saved a desync report of tick {} (digest {}).",
            m.tick,
            short(theirs)
        );
        assert!(notice(c, |t| t == said), "{:?}", c.chat_lines());
    }
    let mine = format!(
        "Desync at tick {}; report saved to the test store as {prefix}",
        m.tick
    );
    assert!(notice(&a, |t| t.starts_with(&mine)));
    // The evidence keeps nothing more once the report is made.
    assert!(
        a.evidence()
            .is_some_and(|e| e.done() && e.bytes_held() == 0)
    );
    relay.stop();
}

/// A settings store shared by a client and the one that replaces it.
#[derive(Clone, Default)]
struct Kept(std::sync::Arc<std::sync::Mutex<Option<String>>>);

impl Store for Kept {
    fn read(&self) -> Option<String> {
        self.0.lock().expect("the store").clone()
    }

    fn write(&self, text: &str) -> Result<(), String> {
        *self.0.lock().expect("the store") = Some(text.into());
        Ok(())
    }
}

/// Runs the clients' games with orders until each has run `until` ticks,
/// keeping every tick's state hash an update shows.
fn play_to(
    clock: &Clock,
    cs: &mut [&mut Client],
    hashes: &mut [BTreeMap<i32, String>],
    rng: &mut [u64],
    until: i32,
) {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let mut done = true;
        for (k, c) in cs.iter_mut().enumerate() {
            c.update(clock.now());
            run_start(c);
            let Some(s) = c.session.as_ref() else {
                done = false;
                continue;
            };
            let w = &s.play.game.world;
            let t = w.tick;
            if t > 0 && !hashes[k].contains_key(&t) {
                hashes[k].insert(t, glasnost_sim::save::state_hash(w));
                if c.lockstep().is_some_and(|ls| !ls.catching_up()) {
                    script(c, t, &mut rng[k]);
                }
            }
            done &= t >= until;
        }
        if done {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "the games stalled at {:?}: {:?}",
            hashes
                .iter()
                .map(|h| h.last_key_value().map(|(t, _)| *t))
                .collect::<Vec<_>>(),
            cs.iter().map(|c| c.net_banner()).collect::<Vec<_>>(),
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// Three persons and a computer play; one person's client goes mid-game
/// and a new one, with the same settings store, offers **Reconnect** (M9
/// N5): it loads the map, takes the game the peer of the lowest slot sends,
/// catches up and is readmitted; all three games then agree tick for tick, with orders from
/// each. (Dropping a client in the test process closes its link as a
/// killed process's would, but it is not a process kill: the local lane
/// does that.)
#[test]
fn a_client_that_went_reconnects_and_plays_on_in_lockstep() {
    let (_rt, relay) = relay(41);
    let clock = Clock::new();
    let mut a = client(&relay, "Ann");
    let store = Kept::default();
    let mut b = client(&relay, "Bea");
    b.attach_store(Box::new(store.clone()));
    b.netform.relay = format!("ws://{}/", relay.addr);
    let mut c = client(&relay, "Cy");

    press_menu(&mut a, MenuHit::Multiplayer);
    press_net(&mut a, NetHit::Host);
    pump(&clock, &mut [&mut a], "the room", |c| {
        c[0].room_code().is_some()
    });
    let code = a.room_code().unwrap().to_owned();
    // Rows 2 and 3 for persons, row 4 for a computer.
    for (row, clicks, kind) in [
        (1, 1, SlotKind::Human),
        (2, 2, SlotKind::Human),
        (3, 1, SlotKind::Ai),
    ] {
        for _ in 0..clicks {
            press_room(&mut a, SetupHit::Kind(row, -1));
            let at = a.room().unwrap().setup.slots[row].kind;
            pump(&clock, &mut [&mut a], "the row's change", |c| {
                c[0].room().unwrap().setup.slots[row].kind != at
            });
        }
        assert_eq!(a.room().unwrap().setup.slots[row].kind, kind);
    }
    for x in [&mut b, &mut c] {
        press_menu(x, MenuHit::Multiplayer);
        x.netform.code = code.clone();
        press_net(x, NetHit::Join);
    }
    pump(&clock, &mut [&mut a, &mut b, &mut c], "the joins", |c| {
        c.iter()
            .all(|c| c.room().is_some_and(|v| v.members.len() == 3))
    });
    press_room(&mut b, SetupHit::Start_);
    press_room(&mut c, SetupHit::Start_);
    pump(&clock, &mut [&mut a, &mut b, &mut c], "ready", |c| {
        c[0].room()
            .unwrap()
            .members
            .iter()
            .all(|m| m.ready || m.row == 0)
    });
    press_room(&mut a, SetupHit::Start_);
    pump(&clock, &mut [&mut a, &mut b, &mut c], "the start", |c| {
        c.iter()
            .all(|c| c.lockstep().is_some_and(|ls| ls.phase() == Phase::Running))
    });
    let slot = b.lockstep().unwrap().me();
    // The peer that sends the game back is the live person of the lowest
    // slot; the slots follow the colours drawn for the rows on random.
    // spec: S-0009 claim 19 (house order by colour slot)
    let host_serves = a.lockstep().unwrap().me() < c.lockstep().unwrap().me();
    let kept = b
        .settings
        .rejoin
        .clone()
        .expect("the game to rejoin is kept");
    assert_eq!((kept.code.as_str(), kept.slot), (code.as_str(), slot));
    assert!(
        store
            .read()
            .is_some_and(|t| t.contains(&format!("rejoin={code} {slot} ")))
    );

    let mut hashes = vec![BTreeMap::new(); 3];
    let mut rng = [5u64, 6, 7];
    play_to(
        &clock,
        &mut [&mut a, &mut b, &mut c],
        &mut hashes,
        &mut rng,
        120,
    );

    // The client goes without a word to the relay.
    drop(b);
    pump(&clock, &mut [&mut a, &mut c], "the drop notices", |c| {
        c.iter()
            .all(|c| notice(c, |t| t.starts_with("Bea left the game; they have ")))
    });

    // A new one, on the same settings store, offers to reconnect.
    let mut b = client(&relay, "Bea");
    b.attach_store(Box::new(store.clone()));
    assert_eq!(b.settings.rejoin.as_ref(), Some(&kept));
    press_menu(&mut b, MenuHit::Multiplayer);
    let (d, _) = b.frame();
    assert!(
        d.texts()
            .iter()
            .any(|t| t.contains(&format!("Reconnect to game {code}")))
    );
    let started = Instant::now();
    press_net(&mut b, NetHit::Reconnect);
    pump(
        &clock,
        &mut [&mut a, &mut b, &mut c],
        "the readmission",
        |c| notice(c[1], |t| t == "You are back in the game."),
    );
    let took = started.elapsed();
    let serving = |x: &Client| notice(x, |t| t == "Bea is reconnecting; sending them the game.");
    assert_eq!(serving(&a), host_serves, "the lower slot sends the game");
    assert_eq!(serving(&c), !host_serves, "one peer sends the game");
    pump(
        &clock,
        &mut [&mut a, &mut b, &mut c],
        "the notices of the return",
        |c| {
            [0, 2]
                .iter()
                .all(|&k| notice(c[k], |t| t == "Bea reconnected."))
        },
    );
    assert_eq!(b.shell.screen, Screen::Game);
    assert_eq!(b.lockstep().unwrap().me(), slot);
    assert_eq!(b.net_banner(), None);
    assert!(b.session.as_ref().unwrap().network);
    let back = b.session.as_ref().unwrap().play.game.world.tick;
    eprintln!("rejoined at tick {back} after {took:?}");

    // All play on with orders; their games agree tick for tick.
    let mut after = vec![BTreeMap::new(); 3];
    play_to(
        &clock,
        &mut [&mut a, &mut b, &mut c],
        &mut after,
        &mut rng,
        back + 200,
    );
    for k in [0, 2] {
        let both = after[1].keys().filter(|t| after[k].contains_key(t)).count();
        assert!(both > 150, "{both} ticks seen by both");
        for (t, h) in &after[1] {
            if let Some(theirs) = after[k].get(t) {
                assert_eq!(h, theirs, "tick {t}");
            }
        }
    }

    // Ann surrenders and leaves through the options menu: three human
    // houses stand, so the menu offers the two surrenders; her client
    // leaves once her surrender order has been carried out, and the others
    // hold her house marked to die from that tick, not from a drop's grace.
    // spec: S-0201 claim 15 (while at least two human houses, the player's own among them, are undefeated: two choices that send the surrender order, one then leaving the game) and S-0414 claim 11 (the surrender marks the house to die)
    let ann = usize::from(a.session.as_ref().unwrap().play.me().0);
    assert!(a.session.as_mut().unwrap().open_options());
    let _ = a.frame();
    let r = a
        .session
        .as_ref()
        .unwrap()
        .pause_targets()
        .rect_of(&PauseHit::SurrenderLeave);
    click(&mut a, r.expect("the surrender that leaves"));
    let doomed = |x: &Client| {
        x.session
            .as_ref()
            .is_some_and(|s| s.play.game.world.houses[ann].econ.result.doomed)
    };
    let tick = |x: &Client| x.session.as_ref().map(|s| s.play.game.world.tick);
    let (mut last, mut seen) = (0, [None, None]);
    pump(
        &clock,
        &mut [&mut a, &mut b, &mut c],
        "the surrender and the leave",
        |c| {
            last = tick(c[0]).unwrap_or(last);
            for (k, x) in [&c[1], &c[2]].into_iter().enumerate() {
                if seen[k].is_none() && doomed(x) {
                    seen[k] = tick(x);
                }
            }
            c[0].shell.screen == Screen::Menu && seen.iter().all(Option::is_some)
        },
    );
    assert!(a.session.is_none(), "Ann left the game");
    for t in seen.into_iter().flatten() {
        assert!(t <= last + 60, "doomed at {t}, Ann left at {last}");
    }
    relay.stop();
}

/// The relay's silence limit in the pump tests, in milliseconds: short, so
/// that a test outlasts it several times in a few seconds. The relay owns
/// this timeout (it runs on the relay's clock, not ours), and the game's
/// start is one blocking update of some 250 ms in a debug build, which a
/// loaded machine stretches several times over; the limit is set well
/// above that, so that only a stall far beyond it can cross it.
const SHORT_SILENCE_MS: u64 = 2_500;

/// How many silence limits the pump tests run for.
const SILENCES: u64 = 3;

/// Two persons and a computer in a room on a relay that closes a
/// connection silent for [`SHORT_SILENCE_MS`]; their games run.
fn short_silence_game(
    seed: u64,
    clock: &Clock,
) -> (tokio::runtime::Runtime, Relay, Client, Client) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("a runtime");
    let limits = Limits {
        silent_ms: SHORT_SILENCE_MS,
        ..Limits::default()
    };
    let relay = rt
        .block_on(glasnost_relay::bind_with(
            "127.0.0.1:0",
            limits,
            Box::new(Fake(seed)),
        ))
        .expect("the relay binds");
    let mut a = client(&relay, "Ann");
    let mut b = client(&relay, "Bea");
    press_menu(&mut a, MenuHit::Multiplayer);
    press_net(&mut a, NetHit::Host);
    pump(clock, &mut [&mut a], "the room", |c| {
        c[0].room_code().is_some()
    });
    let code = a.room_code().unwrap().to_owned();
    press_room(&mut a, SetupHit::Kind(1, -1));
    pump(clock, &mut [&mut a], "an open row", |c| {
        c[0].room().unwrap().setup.slots[1].kind == SlotKind::Human
    });
    press_menu(&mut b, MenuHit::Multiplayer);
    b.netform.code = code;
    press_net(&mut b, NetHit::Join);
    pump(clock, &mut [&mut a, &mut b], "the join", |c| {
        c.iter()
            .all(|c| c.room().is_some_and(|v| v.members.len() == 2))
    });
    press_room(&mut b, SetupHit::Start_);
    pump(clock, &mut [&mut a, &mut b], "ready", |c| {
        c[0].room()
            .unwrap()
            .members
            .iter()
            .all(|m| m.ready || m.row == 0)
    });
    press_room(&mut a, SetupHit::Start_);
    pump(clock, &mut [&mut a, &mut b], "ticks", |c| {
        c.iter().all(|c| tick_of(c) > 20)
    });
    (rt, relay, a, b)
}

/// Runs `attempt` on a fresh clock; when it panics and the host stalled
/// the test for [`STALL_MS`] or more meanwhile, runs it again (up to
/// [`ATTEMPTS`] times). These two tests set the relay's real silence limit
/// ([`SHORT_SILENCE_MS`], a timeout the relay owns, on its own threads)
/// against the test's own loop: a machine so loaded that the loop does not
/// run for a good part of that limit can fail them whatever the code under
/// test does. A stall of that size is a property of the host, not of the
/// pump, so such an attempt counts for nothing and is repeated; a failure
/// with no stall in it, or one that repeats every time, still fails the
/// test with its own message, and both assertions stand unweakened.
fn under_load(attempt: impl Fn(&Clock)) {
    const STALL_MS: i64 = SHORT_SILENCE_MS as i64 / 4;
    const ATTEMPTS: usize = 6;
    for n in 1.. {
        let clock = Clock::new();
        let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| attempt(&clock)));
        let Err(panic) = run else { return };
        if n == ATTEMPTS || clock.gap.get() < STALL_MS {
            std::panic::resume_unwind(panic);
        }
        eprintln!(
            "attempt {n} failed after the host stalled the test for {} ms; trying again",
            clock.gap.get()
        );
    }
}

fn tick_of(c: &Client) -> i32 {
    c.session.as_ref().map_or(0, |s| s.play.game.world.tick)
}

/// Bea's window keeps drawing frames (an update every millisecond or so)
/// for `ms` milliseconds; Ann's draws none from the start of it, and is
/// updated only by the network pump when `pumped` (at the times the pump
/// asks for), else not at all. Ann's ticks run meanwhile, and whether Bea
/// was told that Ann left.
fn ann_draws_no_frames(
    clock: &Clock,
    a: &mut Client,
    b: &mut Client,
    pumped: bool,
    ms: u64,
) -> (i32, bool) {
    let mut p = Pump::default();
    let start = clock.now();
    p.drew(start);
    let mut wake = start;
    let ticks = tick_of(a);
    let mut updates = 0;
    while clock.now() - start < ms as i64 {
        let now = clock.now();
        b.update(now);
        if pumped && now >= wake {
            if p.between_frames(a, now) {
                updates += 1;
            }
            wake = now + p.wait_ms(a, now).expect("a network link") as i64;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    if pumped {
        assert!(updates > 50, "the pump updated Ann {updates} times");
    }
    let left = notice(b, |t| t.starts_with("Ann left the game"));
    (tick_of(a) - ticks, left)
}

/// The M9 exit run's first game: a window that draws no frames (covered
/// by a screenshot overlay) lost its game, as the relay closed its silent
/// connection. With the network pump it plays on for several of the
/// relay's silence limits: the relay hears it, its ticks run, and both
/// games keep the same hashes.
#[test]
fn a_network_game_plays_on_while_its_window_draws_no_frames() {
    under_load(plays_on_without_frames);
}

fn plays_on_without_frames(clock: &Clock) {
    let (_rt, relay, mut a, mut b) = short_silence_game(51, clock);
    // While Ann's frames come, the pump leaves Ann's updates to them.
    let mut p = Pump::default();
    for _ in 0..50 {
        let now = clock.now();
        a.update(now);
        p.drew(now);
        b.update(now);
        assert!(!p.between_frames(&mut a, now + 2), "frames are coming");
        let wait = p.wait_ms(&a, now).expect("a network link");
        assert!((1..=STALE_MS as u64).contains(&wait), "{wait}");
        std::thread::sleep(Duration::from_millis(2));
    }
    let ran = tick_of(&b);
    let (ticks, left) =
        ann_draws_no_frames(clock, &mut a, &mut b, true, SILENCES * SHORT_SILENCE_MS);
    assert!(!left, "the relay kept Ann's connection");
    assert!(a.net.as_ref().is_some_and(|n| n.closed.is_none()));
    // Some 250 ticks at the fastest network speed in 4 s; a fifth at
    // least, on a loaded machine.
    assert!(ticks > 50, "Ann ran {ticks} ticks");
    assert!(tick_of(&b) > ran + 50, "Bea's game went on");
    // Both games agree: Ann's frames came, and the state hashes match.
    let mut hashes = [BTreeMap::new(), BTreeMap::new()];
    let deadline = Instant::now() + Duration::from_secs(30);
    let at = tick_of(&a).max(tick_of(&b)) + 5;
    while hashes.iter().any(|h| h.len() < 20) {
        for (k, c) in [&mut a, &mut b].into_iter().enumerate() {
            c.update(clock.now());
            if tick_of(c) >= at {
                keep_hash(&mut hashes[k], c);
            }
        }
        assert!(Instant::now() < deadline, "the games stalled");
        std::thread::sleep(Duration::from_millis(1));
    }
    let both: Vec<_> = hashes[0]
        .iter()
        .filter(|(t, _)| hashes[1].contains_key(t))
        .collect();
    assert!(both.len() > 10, "{} ticks in common", both.len());
    for (t, h) in both {
        assert_eq!(Some(h), hashes[1].get(t), "tick {t}");
    }
    relay.stop();
}

/// The control of the test above: without the pump, the same window loses
/// its game once the relay's silence limit passes.
#[test]
fn a_window_that_draws_no_frames_and_is_not_pumped_loses_its_game() {
    under_load(loses_its_game_unpumped);
}

fn loses_its_game_unpumped(clock: &Clock) {
    let (_rt, relay, mut a, mut b) = short_silence_game(52, clock);
    let (ticks, left) =
        ann_draws_no_frames(clock, &mut a, &mut b, false, SILENCES * SHORT_SILENCE_MS);
    assert_eq!(ticks, 0);
    assert!(left, "the relay closed Ann's silent connection");
    relay.stop();
}

/// Direct links (M9 N7), with this file's helpers.
#[path = "client_direct_tests.rs"]
mod direct;
