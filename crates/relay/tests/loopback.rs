//! The relay over real sockets on 127.0.0.1: creating, peeking, joining,
//! leaving and starting a room, forwarding, drops, refusals that name the
//! field, forged drops, and the limits on hostile input. Our own netcode;
//! no spec applies.

mod common;

use std::time::Duration;

use common::{Client, Fake, Incoming, create, join, ready};
use futures_util::{SinkExt, StreamExt};
use glasnost_net::Field;
use glasnost_net::lobby::{FromRelay, Refusal, ToRelay};
use glasnost_net::wire::{self, Body, Envelope, Frame, Frames, RELAY};
use glasnost_relay::{Limits, Relay, bind, bind_with};
use glasnost_ui::setup::SlotKind;
use tokio_tungstenite::tungstenite::{Bytes, Message};

async fn relay(limits: Limits) -> Relay {
    bind_with("127.0.0.1:0", limits, Box::new(Fake(0x5eed)))
        .await
        .expect("binds")
}

fn frames(from: u8, first: u32, n: usize) -> Vec<u8> {
    wire::encode(&Envelope {
        from,
        body: Body::Frames(Frames {
            first,
            frames: vec![Frame::default(); n],
            ..Frames::default()
        }),
    })
}

fn dropped(from: u8, slot: u8, kept: u32) -> Vec<u8> {
    wire::encode(&Envelope {
        from,
        body: Body::Dropped { slot, kept },
    })
}

async fn room_code(c: &mut Client) -> String {
    match c.next_lobby().await {
        FromRelay::Room(v) => v.code,
        m => panic!("a room view: {m:?}"),
    }
}

/// A room of `persons` persons and a computer whose game has begun: the
/// clients by slot.
async fn game(addr: std::net::SocketAddr, persons: usize) -> Vec<Client> {
    let mut rows = vec![SlotKind::Human; persons];
    rows.push(SlotKind::Ai);
    let mut host = Client::connect(addr).await;
    host.lobby(&create(&rows, "m4", 4));
    let code = room_code(&mut host).await;
    let mut all = vec![host];
    for i in 1..persons {
        let mut c = Client::connect(addr).await;
        c.lobby(&join(&code, &format!("P{i}"), "build-1", "m4"));
        let _ = room_code(&mut c).await;
        all.push(c);
    }
    for c in &all[1..] {
        c.lobby(&ready("m4"));
    }
    // Wait until the host sees everyone ready.
    loop {
        if let FromRelay::Room(v) = all[0].next_lobby().await
            && v.members.iter().filter(|m| m.ready).count() == persons - 1
        {
            break;
        }
    }
    all[0].lobby(&ToRelay::Start);
    let mut by_slot: Vec<Option<Client>> = (0..persons).map(|_| None).collect();
    for mut c in all {
        let you = loop {
            if let FromRelay::Begin(b) = c.next_lobby().await {
                break b.you;
            }
        };
        by_slot[usize::from(you)] = Some(c);
    }
    by_slot
        .into_iter()
        .map(|c| c.expect("a slot each"))
        .collect()
}

/// The next game message, as an envelope.
async fn next_game(c: &mut Client) -> Envelope {
    loop {
        match c.next().await {
            Incoming::Game(b) => return wire::decode(&b).expect("a game message"),
            Incoming::Lobby(_) => continue,
            Incoming::Closed(r) => panic!("closed: {r:?}"),
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn create_peek_join_leave_and_start_over_sockets() {
    let r = bind("127.0.0.1:0", Limits::default()).await.expect("binds");
    let mut host = Client::connect(r.addr).await;
    host.lobby(&create(
        &[SlotKind::Human, SlotKind::Human, SlotKind::Ai],
        "m4",
        4,
    ));
    let code = room_code(&mut host).await;

    let mut b = Client::connect(r.addr).await;
    b.lobby(&ToRelay::Peek {
        code: code.to_lowercase(),
    });
    let FromRelay::Info(info) = b.next_lobby().await else {
        panic!("room info");
    };
    assert_eq!((info.persons, info.open), (1, 1));
    b.lobby(&join(&code, "Bea", "build-1", "m4"));
    let FromRelay::Room(v) = b.next_lobby().await else {
        panic!("a room view");
    };
    assert_eq!(v.members.len(), 2);
    let FromRelay::Room(_) = host.next_lobby().await else {
        panic!("the host sees the join");
    };
    // Bea leaves and comes back.
    b.lobby(&ToRelay::Leave);
    let FromRelay::Room(v) = host.next_lobby().await else {
        panic!("the host sees the leave");
    };
    assert_eq!(v.members.len(), 1);
    b.lobby(&join(&code, "Bea", "build-1", "m4"));
    let _ = b.next_lobby().await;
    let _ = host.next_lobby().await;
    b.lobby(&ready("m4"));
    let _ = b.next_lobby().await;
    let _ = host.next_lobby().await;
    host.lobby(&ToRelay::Start);
    let (FromRelay::Begin(hb), FromRelay::Begin(bb)) =
        (host.next_lobby().await, b.next_lobby().await)
    else {
        panic!("both begin");
    };
    assert_eq!((hb.you, bb.you), (0, 1));
    assert_eq!(hb.seed, bb.seed);
    assert_eq!(hb.slots, bb.slots);
    // Forwarding, both ways.
    host.send(frames(0, 0, 2));
    b.send(frames(1, 0, 1));
    assert_eq!(next_game(&mut b).await.from, 0);
    assert_eq!(next_game(&mut host).await.from, 1);
    r.stop();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_join_with_another_build_or_map_is_refused_naming_the_field() {
    let r = relay(Limits::default()).await;
    let mut host = Client::connect(r.addr).await;
    host.lobby(&create(&[SlotKind::Human, SlotKind::Human], "m4", 4));
    let code = room_code(&mut host).await;
    let mut b = Client::connect(r.addr).await;
    for (msg, field, word) in [
        (join(&code, "B", "build-2", "m4"), Field::Build, "build"),
        (join(&code, "B", "build-1", "m5"), Field::MapHash, "map"),
    ] {
        b.lobby(&msg);
        let m = b.next_lobby().await;
        assert_eq!(m, FromRelay::Refused(Refusal::Mismatch(field)));
        let FromRelay::Refused(why) = m else {
            unreachable!()
        };
        assert!(why.to_string().contains(word), "{why}");
    }
    r.stop();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_forged_drop_is_never_delivered_and_a_real_one_is() {
    let r = relay(Limits::default()).await;
    let mut c = game(r.addr, 3).await;
    // Slot 0 reports slot 2 dropped, as itself and as the relay, and speaks
    // for slot 1; then sends a frame of its own.
    c[0].send(dropped(0, 2, 0));
    c[0].send(dropped(RELAY, 2, 0));
    c[0].send(frames(1, 0, 1));
    c[0].send(frames(0, 0, 3));
    for s in [1, 2] {
        let e = next_game(&mut c[s]).await;
        assert_eq!(
            e.from, 0,
            "the first game message slot {s} gets is slot 0's frames"
        );
        assert!(matches!(e.body, Body::Frames(_)));
    }
    // Slot 0 is cut off: the others hear, from the relay, that it kept 3.
    let gone = c.remove(0);
    gone.kill();
    for p in &mut c {
        assert_eq!(
            next_game(p).await,
            Envelope {
                from: RELAY,
                body: Body::Dropped { slot: 0, kept: 3 },
            }
        );
    }
    r.stop();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn hostile_bytes_close_the_connection_with_a_reason_and_the_relay_goes_on() {
    let r = relay(Limits {
        max_message: 1024,
        ..Limits::default()
    })
    .await;
    let mut a = Client::connect(r.addr).await;
    a.text("hello");
    assert_eq!(a.closed().await, "not a Glasnost message");
    let mut b = Client::connect(r.addr).await;
    b.send(b"GL\x02\xff\xff\xff\xff".to_vec());
    assert!(b.closed().await.starts_with("unreadable lobby message"));
    let mut c = Client::connect(r.addr).await;
    c.send(vec![b'G'; 4096]);
    assert_eq!(c.closed().await, "message too large");
    let mut d = Client::connect(r.addr).await;
    d.send(b"GL\x63".to_vec());
    assert_eq!(
        d.next_lobby().await,
        FromRelay::Refused(Refusal::Mismatch(Field::Protocol))
    );
    assert_eq!(d.closed().await, "another protocol version");
    // Still serving.
    let mut e = Client::connect(r.addr).await;
    e.lobby(&create(&[SlotKind::Human], "m4", 4));
    let _ = room_code(&mut e).await;
    r.stop();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rate_rooms_and_connections_are_limited() {
    let r = relay(Limits {
        max_rooms: 1,
        max_connections: 3,
        rate: 1,
        burst: 4,
        ..Limits::default()
    })
    .await;
    let mut a = Client::connect(r.addr).await;
    a.lobby(&create(&[SlotKind::Human], "m4", 4));
    let code = room_code(&mut a).await;
    let mut b = Client::connect(r.addr).await;
    b.lobby(&create(&[SlotKind::Human], "m4", 4));
    assert_eq!(
        b.next_lobby().await,
        FromRelay::Refused(Refusal::TooManyRooms)
    );
    let mut c = Client::connect(r.addr).await;
    for _ in 0..10 {
        c.lobby(&ToRelay::Peek { code: code.clone() });
    }
    assert_eq!(c.closed().await, "too many messages");
    // a and b are open; c's place frees once the relay has forgotten it.
    let mut d = Client::connect(r.addr).await;
    d.lobby(&ToRelay::Peek { code: code.clone() });
    assert!(matches!(d.next_lobby().await, FromRelay::Info(_)));
    let mut e = Client::connect(r.addr).await;
    let full = match e.next().await {
        Incoming::Closed(r) => r,
        other => panic!("the fourth connection closes: {other:?}"),
    };
    assert_eq!(full.as_deref(), Some("the relay is full"));
    r.stop();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_silent_connection_is_closed_and_its_slot_dropped() {
    let r = relay(Limits {
        silent_ms: 600,
        keepalive_ms: 100,
        ..Limits::default()
    })
    .await;
    let mut c = game(r.addr, 2).await;
    // Slot 0 stops reading, so answers no keepalive; its socket stays open.
    c[0].pause();
    let e = next_game(&mut c[1]).await;
    assert_eq!(
        e,
        Envelope {
            from: RELAY,
            body: Body::Dropped { slot: 0, kept: 0 },
        },
        "slot 0 never sent a frame"
    );
    r.stop();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_client_that_never_reads_is_closed_as_silent() {
    let r = relay(Limits {
        silent_ms: 500,
        keepalive_ms: 100,
        ..Limits::default()
    })
    .await;
    let (mut ws, _) = tokio_tungstenite::connect_async(format!("ws://{}/", r.addr))
        .await
        .expect("connects");
    tokio::time::sleep(Duration::from_millis(900)).await;
    // Now read: keepalives, then the relay's reason and its close.
    let mut reason = None;
    while let Ok(Some(m)) = tokio::time::timeout(Duration::from_secs(5), ws.next()).await {
        let Ok(m) = m else { break };
        if let Message::Close(f) = m {
            reason = f.map(|f| f.reason.to_string());
            break;
        }
    }
    assert_eq!(reason.as_deref(), Some("silent for too long"));
    let _ = ws.send(Message::Binary(Bytes::new())).await;
    r.stop();
}
