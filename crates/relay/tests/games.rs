//! Real games over the real relay on 127.0.0.1: three persons and a
//! computer meet in a room, start, and each runs its own lockstep session
//! (N1) and its own `glasnost_ai::game::Game` on the synthetic skirmish
//! (invented rules and map, `glasnost_ai::fixture`), the persons' slots
//! played by scripted orders and the computer's by our AI inside the tick.
//! Every peer's state hash after every tick equals the others' and a
//! single-process run of the same orders; a peer cut off mid-game is
//! reported dropped to the others with the same `kept`. Our own netcode; no
//! spec applies, beyond the house order the setup cites.

mod common;

use std::time::{Duration, Instant};

use common::skirmish::{build, game, map8, scripted};
use common::{Client, Fake, Incoming, create, join, ready};
use glasnost_ai::game::Game;
use glasnost_net::lobby::{Begin, Edit, FromRelay, ToRelay};
use glasnost_net::simnet::SplitMix;
use glasnost_net::wire::Hash;
use glasnost_net::{
    Config, Event, Phase, Session, Simulation, SlotKind as NetKind, Tick, Transport, pump,
};
use glasnost_relay::{Limits, bind_with};
use glasnost_sim::world::HouseId;
use glasnost_ui::setup::SlotKind;

/// The tick period of the test games, in milliseconds.
const TICK_MS: u32 = 20;

/// Ticks every survivor runs.
const TICKS: u32 = 300;

/// The tick after which the cut-off peer goes.
const KILL_AT: u32 = 120;

/// The client as the session's transport.
struct Link<'a>(&'a mut Client);

impl Transport for Link<'_> {
    fn send(&mut self, _now_ms: u64, bytes: &[u8]) {
        self.0.send(bytes.to_vec());
    }

    fn recv(&mut self, _now_ms: u64) -> Option<Vec<u8>> {
        match self.0.try_next()? {
            Incoming::Game(b) => Some(b),
            Incoming::Lobby(m) => panic!("a lobby message in the game: {m:?}"),
            Incoming::Closed(r) => panic!("the relay closed a live peer: {r:?}"),
        }
    }
}

struct Peer {
    slot: u8,
    client: Option<Client>,
    session: Session,
    game: Game,
    house: HouseId,
    rng: SplitMix,
    next_due: u64,
    input_for: Option<u32>,
    ticks: Vec<Tick>,
    hashes: Vec<Hash>,
    events: Vec<Event>,
}

/// Three persons (rows 0 to 2, two with a chosen colour) and a computer
/// (row 3) meet in a room and begin: each client's `Begin`, by row.
async fn meet(addr: std::net::SocketAddr) -> Vec<(Client, Begin)> {
    let rows = [
        SlotKind::Human,
        SlotKind::Human,
        SlotKind::Human,
        SlotKind::Ai,
    ];
    let mut host = Client::connect(addr).await;
    host.lobby(&create(&rows, "synthetic-8", 8));
    let FromRelay::Room(v) = host.next_lobby().await else {
        panic!("a room");
    };
    let code = v.code;
    let mut clients = vec![host];
    for i in 1..3 {
        let mut c = Client::connect(addr).await;
        c.lobby(&ToRelay::Peek { code: code.clone() });
        let FromRelay::Info(info) = c.next_lobby().await else {
            panic!("room info");
        };
        assert_eq!(info.map.starts, 8);
        c.lobby(&join(&code, &format!("P{i}"), "build-1", "synthetic-8"));
        clients.push(c);
    }
    // The host takes colour 5 and row 1 colour 1; row 2 stays on random.
    for (row, colour) in [(0u8, 5u8), (1, 1)] {
        clients[usize::from(row)].lobby(&ToRelay::Edit(Edit::Choices {
            row,
            country: None,
            colour: Some(colour),
            team: None,
            start: None,
        }));
    }
    for c in &clients[1..] {
        c.lobby(&ready("synthetic-8"));
    }
    loop {
        if let FromRelay::Room(v) = clients[0].next_lobby().await
            && v.members.len() == 3
            && v.members.iter().filter(|m| m.ready).count() == 2
            && v.setup.slots[1].colour == Some(1)
        {
            break;
        }
    }
    clients[0].lobby(&ToRelay::Start);
    let mut out = Vec::new();
    for mut c in clients {
        let begin = loop {
            if let FromRelay::Begin(b) = c.next_lobby().await {
                break b;
            }
        };
        out.push((c, begin));
    }
    out
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn three_persons_and_a_computer_play_over_the_relay_and_survive_a_drop() {
    let relay = bind_with("127.0.0.1:0", Limits::default(), Box::new(Fake(77)))
        .await
        .expect("binds");
    let map = map8();
    let met = meet(relay.addr).await;
    // spec: S-0009 claim 19 (persons by increasing colour slot, then computers; Q-1805: row 2's person on "random" is placed by the slot it was given)
    let given = met[0].1.setup.slots[2]
        .colour
        .expect("row 2 is given a slot");
    let rows = if given < 1 {
        [2, 1, 0, 3]
    } else if given < 5 {
        [1, 2, 0, 3]
    } else {
        [1, 0, 2, 3]
    };
    assert_eq!(met[0].1.rows, rows, "given slot {given}");
    assert_eq!(
        met[0].1.slots.iter().map(|s| s.kind).collect::<Vec<_>>(),
        [
            NetKind::Human,
            NetKind::Human,
            NetKind::Human,
            NetKind::Computer
        ]
    );
    let start = Instant::now();
    let now = || start.elapsed().as_millis() as u64;
    let mut setups = Vec::new();
    let mut peers: Vec<Peer> = met
        .into_iter()
        .map(|(client, begin)| {
            let (game, info, gs) = build(&begin, &map);
            setups.push(gs);
            let house = HouseId(info.slots[usize::from(begin.you)].house);
            Peer {
                slot: begin.you,
                client: Some(client),
                session: Session::new(Config::new(TICK_MS), info, begin.you, now())
                    .expect("a person's slot"),
                game,
                house,
                rng: SplitMix::new(1000 + u64::from(begin.you)),
                next_due: 0,
                input_for: None,
                ticks: Vec::new(),
                hashes: Vec::new(),
                events: Vec::new(),
            }
        })
        .collect();
    peers.sort_by_key(|p| p.slot);
    assert!(setups.iter().all(|g| *g == setups[0]), "one game setup");
    // The peer of slot 1 is cut off: the host (row 0) when row 2 was given a slot above 5, else a guest.
    let victim = 1u8;
    let deadline = Duration::from_secs(120);
    loop {
        assert!(
            start.elapsed() < deadline,
            "the game did not finish in time"
        );
        let t = now();
        for p in &mut peers {
            let Some(client) = p.client.as_mut() else {
                continue;
            };
            pump(&mut p.session, &mut Link(client), t);
            if t >= p.next_due && p.session.phase() == Phase::Running {
                let tick = p.session.ran();
                if p.input_for != Some(tick) {
                    p.input_for = Some(tick);
                    for o in scripted(&p.game, p.house, tick, &mut p.rng) {
                        p.session.submit(o);
                    }
                }
                if let Some(done) = p.session.step(t, &mut p.game) {
                    p.next_due = t + u64::from(TICK_MS);
                    p.hashes.push(Simulation::state_hash(&p.game));
                    p.ticks.push(done);
                    pump(&mut p.session, &mut Link(client), t);
                }
            }
            p.events.extend(p.session.take_events());
            if p.slot == victim && p.session.ran() >= KILL_AT {
                p.client.take().expect("live").kill();
            }
        }
        if peers
            .iter()
            .filter(|p| p.slot != victim)
            .all(|p| p.session.ran() >= TICKS)
        {
            break;
        }
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
    relay.stop();

    // No peer saw trouble.
    for p in &peers {
        for e in &p.events {
            assert!(
                !matches!(
                    e,
                    Event::Desync { .. } | Event::Refused { .. } | Event::BadMessage { .. }
                ),
                "slot {}: {e:?}",
                p.slot
            );
        }
    }
    // Every survivor heard the drop, with the same kept.
    let survivors: Vec<&Peer> = peers.iter().filter(|p| p.slot != victim).collect();
    let kept: Vec<u32> = survivors
        .iter()
        .map(|p| {
            let drops: Vec<u32> = p
                .events
                .iter()
                .filter_map(|e| match e {
                    Event::Dropped { slot, kept } if *slot == victim => Some(*kept),
                    _ => None,
                })
                .collect();
            assert_eq!(drops.len(), 1, "slot {} hears one drop", p.slot);
            drops[0]
        })
        .collect();
    assert_eq!(kept[0], kept[1], "the same kept on every survivor");
    let kept = kept[0];
    let gone = peers.iter().find(|p| p.slot == victim).expect("the victim");
    assert!(
        kept >= KILL_AT,
        "the victim's frames up to its last tick count"
    );
    assert!(kept <= gone.session.frames_made());
    // The same schedule and hashes on every survivor, and the victim's as
    // far as it ran; nothing of the victim's at or after kept.
    let n = TICKS as usize;
    assert_eq!(survivors[0].ticks[..n], survivors[1].ticks[..n]);
    assert_eq!(survivors[0].hashes[..n], survivors[1].hashes[..n]);
    let g = gone.hashes.len();
    assert_eq!(gone.hashes[..], survivors[0].hashes[..g]);
    assert_eq!(gone.ticks[..], survivors[0].ticks[..g]);
    let victims_orders = |p: &Peer| {
        p.ticks
            .iter()
            .filter(|t| t.orders.iter().any(|a| a.slot == victim))
            .map(|t| t.tick)
            .collect::<Vec<_>>()
    };
    let theirs = victims_orders(survivors[0]);
    assert!(
        !theirs.is_empty(),
        "the victim's orders ran before the drop"
    );
    assert!(
        theirs.iter().all(|&t| t < kept),
        "{theirs:?} against kept {kept}"
    );
    // A single-process run of the same orders gives the same hashes.
    let mut one = game(&map, &setups[0]);
    let mut reference = Vec::new();
    for t in &survivors[0].ticks {
        for a in &t.orders {
            one.issue(a.house, a.order.clone());
        }
        Simulation::step(&mut one);
        reference.push(Simulation::state_hash(&one));
    }
    assert_eq!(reference[..n], survivors[0].hashes[..n]);
}
