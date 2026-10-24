//! The relay's core without sockets: rooms, forwarding, drops, forged
//! messages and the limits on hostile input. The protocol, the room rules
//! and the limits are our own netcode (docs/development.md, *The relay and
//! lobby (N2)*); the room's choice rules are tested in glasnost-net.

use glasnost_net::lobby::{
    self, CHAT_CHARS, CODE_ALPHABET, CODE_LEN, ChatLine, FromRelay, MapChoice, Offered, Refusal,
    Signal, ToRelay, Value,
};
use glasnost_net::wire::{self, Body, Envelope, Frame, Frames, RELAY};
use glasnost_net::{Field, StartInfo};
use glasnost_relay::{ConnId, Entropy, Hub, Limits, Out};
use glasnost_ui::setup::{Setup, Slot, SlotKind};

/// A seeded stand-in for the operating system's randomness.
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

/// Entropy that always gives the same bytes (every code collides).
struct Stuck;

impl Entropy for Stuck {
    fn fill(&mut self, buf: &mut [u8]) {
        buf.fill(7);
    }
}

fn hub_with(limits: Limits) -> Hub {
    Hub::new(limits, Box::new(Fake(0x9e37_79b9_7f4a_7c15)))
}

fn hub() -> Hub {
    hub_with(Limits::default())
}

/// A setup with these rows and the rest closed. Each person's row has a
/// fixed colour rising with the row, so that the persons' slots are their
/// rows.
// spec: S-0009 claim 19 (persons by increasing colour slot)
fn setup(rows: &[SlotKind]) -> Setup {
    let mut slots = [Slot::closed(); 8];
    for (i, (s, &k)) in slots.iter_mut().zip(rows).enumerate() {
        s.kind = k;
        if k == SlotKind::Human {
            s.colour = Some(i as u8);
        }
    }
    Setup {
        map: 0,
        slots,
        credits: 2000,
        units: 3,
        flags: [false, true, true, true, true],
        speed: 2,
    }
}

fn create(rows: &[SlotKind]) -> Vec<u8> {
    lobby::wire::encode(&ToRelay::Create {
        name: "Host".into(),
        build: "build-1".into(),
        rules_hash: "rules-1".into(),
        offered: Offered {
            countries: 2,
            credits: (1000, 5000),
            units: (1, 9),
        },
        map: MapChoice {
            hash: "m4".into(),
            label: "map four".into(),
            starts: 4,
        },
        setup: setup(rows),
    })
}

fn join(code: &str, name: &str, build: &str) -> Vec<u8> {
    lobby::wire::encode(&ToRelay::Join {
        code: code.into(),
        name: name.into(),
        build: build.into(),
        rules_hash: "rules-1".into(),
        map_hash: "m4".into(),
    })
}

fn msg(m: &ToRelay) -> Vec<u8> {
    lobby::wire::encode(m)
}

fn ready() -> Vec<u8> {
    msg(&ToRelay::Ready {
        ready: true,
        map_hash: "m4".into(),
    })
}

/// The lobby messages the hub sent, by receiver.
fn lobby_out(outs: &[Out]) -> Vec<(ConnId, FromRelay)> {
    outs.iter()
        .filter_map(|o| match o {
            Out::Send { to, bytes } if lobby::wire::is_lobby(bytes) => {
                Some((*to, lobby::wire::decode(bytes).expect("a lobby message")))
            }
            _ => None,
        })
        .collect()
}

/// The game messages the hub sent, by receiver.
fn game_out(outs: &[Out]) -> Vec<(ConnId, Envelope)> {
    outs.iter()
        .filter_map(|o| match o {
            Out::Send { to, bytes } if !lobby::wire::is_lobby(bytes) => {
                Some((*to, wire::decode(bytes).expect("a game message")))
            }
            _ => None,
        })
        .collect()
}

fn closes(outs: &[Out]) -> Vec<(ConnId, String)> {
    outs.iter()
        .filter_map(|o| match o {
            Out::Close { to, reason } => Some((*to, reason.clone())),
            _ => None,
        })
        .collect()
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

/// A room of `persons` persons (and a computer) whose game has begun; the
/// connections by slot.
fn game(h: &mut Hub, persons: usize) -> (String, Vec<ConnId>) {
    let mut rows = vec![SlotKind::Human; persons];
    rows.push(SlotKind::Ai);
    let host = h.connect(0).unwrap();
    h.receive(0, host, &create(&rows));
    let code = h.codes()[0].clone();
    let mut conns = vec![host];
    for i in 1..persons {
        let c = h.connect(0).unwrap();
        h.receive(0, c, &join(&code, &format!("P{i}"), "build-1"));
        conns.push(c);
    }
    // Any join clears readiness: everyone readies once all are in.
    for &c in &conns[1..] {
        h.receive(0, c, &ready());
    }
    h.receive(0, host, &msg(&ToRelay::Start));
    let all = h.take_out();
    let begins: Vec<(ConnId, u8)> = lobby_out(&all)
        .into_iter()
        .filter_map(|(to, m)| match m {
            FromRelay::Begin(b) => Some((to, b.you)),
            _ => None,
        })
        .collect();
    assert_eq!(begins.len(), persons, "{:?}", lobby_out(&all));
    let mut by_slot = vec![0; persons];
    for (c, you) in begins {
        by_slot[usize::from(you)] = c;
    }
    (code, by_slot)
}

#[test]
fn create_peek_join_ready_and_start() {
    let mut h = hub();
    let host = h.connect(0).unwrap();
    h.receive(
        0,
        host,
        &create(&[SlotKind::Human, SlotKind::Human, SlotKind::Ai]),
    );
    let out = lobby_out(&h.take_out());
    let [(to, FromRelay::Room(v))] = &out[..] else {
        panic!("a room view: {out:?}");
    };
    assert_eq!((*to, v.you, v.host), (host, host, host));
    let code = v.code.clone();
    assert_eq!(code.len(), CODE_LEN);
    assert!(code.bytes().all(|b| CODE_ALPHABET.contains(&b)));

    // A code is typed loosely: lower case, with a dash, I for 1.
    let b = h.connect(10).unwrap();
    let typed = format!("{}-{}", &code[..4], &code[4..]).to_lowercase();
    h.receive(
        10,
        b,
        &msg(&ToRelay::Peek {
            code: typed.clone(),
        }),
    );
    let out = lobby_out(&h.take_out());
    let [(_, FromRelay::Info(info))] = &out[..] else {
        panic!("room info: {out:?}");
    };
    assert_eq!((info.persons, info.open, info.playing), (1, 1, false));

    h.receive(20, b, &join(&typed, "Bea", "build-1"));
    let out = lobby_out(&h.take_out());
    assert_eq!(out.len(), 2, "both members see the room");
    for (to, m) in &out {
        let FromRelay::Room(v) = m else {
            panic!("a room view: {m:?}");
        };
        assert_eq!(v.you, *to);
        assert_eq!(v.members.len(), 2);
    }

    h.receive(30, host, &msg(&ToRelay::Start));
    assert!(matches!(
        &lobby_out(&h.take_out())[..],
        [(_, FromRelay::Refused(Refusal::NotReady(1)))]
    ));
    h.receive(40, b, &ready());
    h.take_out();
    h.receive(50, b, &msg(&ToRelay::Start));
    assert!(matches!(
        &lobby_out(&h.take_out())[..],
        [(_, FromRelay::Refused(Refusal::NotHost))]
    ));
    h.receive(60, host, &msg(&ToRelay::Start));
    let out = lobby_out(&h.take_out());
    let begins: Vec<_> = out
        .iter()
        .map(|(to, m)| match m {
            FromRelay::Begin(b) => (*to, b.you, b.seed, b.slots.len()),
            m => panic!("a begin: {m:?}"),
        })
        .collect();
    assert_eq!(begins.len(), 2);
    assert_eq!(begins[0].2, begins[1].2, "one seed for everyone");
    assert_eq!(
        begins.iter().map(|b| (b.0, b.1, b.3)).collect::<Vec<_>>(),
        [(host, 0, 3), (b, 1, 3)]
    );
    assert!(h.room(&code).unwrap().playing());
    // Nobody joins a game.
    let c = h.connect(70).unwrap();
    h.receive(70, c, &join(&code, "Cy", "build-1"));
    assert!(matches!(
        &lobby_out(&h.take_out())[..],
        [(_, FromRelay::Refused(Refusal::Playing))]
    ));
}

#[test]
fn leaving_before_the_start_opens_the_row_and_an_empty_room_goes() {
    let mut h = hub();
    let host = h.connect(0).unwrap();
    h.receive(0, host, &create(&[SlotKind::Human, SlotKind::Human]));
    let code = h.codes()[0].clone();
    let b = h.connect(0).unwrap();
    h.receive(0, b, &join(&code, "Bea", "build-1"));
    h.take_out();
    h.receive(1, host, &msg(&ToRelay::Leave));
    let out = lobby_out(&h.take_out());
    let [(to, FromRelay::Room(v))] = &out[..] else {
        panic!("the room to the one left: {out:?}");
    };
    assert_eq!((*to, v.host), (b, b), "the host's role passes on");
    assert_eq!(h.room(&code).unwrap().info().open, 1);
    // The one who left may make a room of its own.
    h.receive(2, host, &create(&[SlotKind::Human]));
    assert_eq!(h.rooms(), 2);
    h.receive(3, b, &msg(&ToRelay::Leave));
    h.closed(4, host);
    assert_eq!(h.rooms(), 0);
    h.receive(5, b, &msg(&ToRelay::Leave));
    assert!(matches!(
        &lobby_out(&h.take_out())[..],
        [.., (_, FromRelay::Refused(Refusal::NotInRoom))]
    ));
}

#[test]
fn a_join_with_another_build_is_refused_naming_the_field() {
    let mut h = hub();
    let host = h.connect(0).unwrap();
    h.receive(0, host, &create(&[SlotKind::Human, SlotKind::Human]));
    let code = h.codes()[0].clone();
    h.take_out();
    let b = h.connect(0).unwrap();
    h.receive(0, b, &join(&code, "Bea", "build-2"));
    let out = lobby_out(&h.take_out());
    assert_eq!(
        out,
        [(b, FromRelay::Refused(Refusal::Mismatch(Field::Build)))]
    );
    assert!(
        Refusal::Mismatch(Field::Build)
            .to_string()
            .contains("build"),
        "the reason names the field"
    );
    assert_eq!(h.room(&code).unwrap().info().persons, 1);
}

#[test]
fn guessing_room_codes_closes_the_connection() {
    let mut h = hub_with(Limits {
        max_refusals: 3,
        ..Limits::default()
    });
    let c = h.connect(0).unwrap();
    for i in 0..3 {
        h.receive(
            i,
            c,
            &msg(&ToRelay::Peek {
                code: "ZZZZZZZZ".into(),
            }),
        );
        assert_eq!(
            lobby_out(&h.take_out()),
            [(c, FromRelay::Refused(Refusal::NoRoom))]
        );
    }
    h.receive(
        4,
        c,
        &msg(&ToRelay::Peek {
            code: "not a code at all".into(),
        }),
    );
    let out = h.take_out();
    assert_eq!(closes(&out), [(c, "too many refused requests".to_owned())]);
    assert_eq!(h.stats().closed_refusals, 1);
    assert_eq!(h.connections(), 0);
}

#[test]
fn a_colliding_room_code_is_drawn_again_and_never_reused() {
    let mut h = Hub::new(Limits::default(), Box::new(Stuck));
    let a = h.connect(0).unwrap();
    h.receive(0, a, &create(&[SlotKind::Human]));
    let b = h.connect(0).unwrap();
    h.receive(0, b, &create(&[SlotKind::Human]));
    assert_eq!(h.rooms(), 1);
    assert!(matches!(
        &lobby_out(&h.take_out())[..],
        [_, (_, FromRelay::Refused(Refusal::TooManyRooms))]
    ));
}

#[test]
fn forwarding_reaches_every_other_person_byte_for_byte_and_in_order() {
    let mut h = hub();
    let (code, c) = game(&mut h, 3);
    let m1 = frames(0, 0, 2);
    let m2 = frames(0, 2, 1);
    h.receive(100, c[0], &m1);
    h.receive(101, c[0], &m2);
    let start = wire::encode(&Envelope {
        from: 1,
        body: Body::Start(StartInfo {
            slots: Vec::new(),
            seed: 1,
            params: glasnost_net::Params::default(),
            build: String::new(),
            rules_hash: String::new(),
            map_hash: String::new(),
            setup_hash: String::new(),
        }),
    });
    h.receive(102, c[1], &start);
    let sent: Vec<(ConnId, Vec<u8>)> = h
        .take_out()
        .into_iter()
        .filter_map(|o| match o {
            Out::Send { to, bytes } => Some((to, bytes)),
            _ => None,
        })
        .collect();
    assert_eq!(
        sent,
        [
            (c[1], m1.clone()),
            (c[2], m1),
            (c[1], m2.clone()),
            (c[2], m2),
            (c[0], start.clone()),
            (c[2], start),
        ]
    );
    assert_eq!(h.stats().forwarded, 3);
    assert_eq!(h.kept(&code, 0), Some(3));
}

#[test]
fn forged_senders_forged_drops_and_early_game_messages_are_never_forwarded() {
    let mut h = hub();
    // A game message before any game.
    let lone = h.connect(0).unwrap();
    h.receive(0, lone, &frames(0, 0, 1));
    let (code, c) = game(&mut h, 3);
    // Slot 0 speaks for slot 1, and for the relay.
    h.receive(100, c[0], &frames(1, 0, 1));
    h.receive(100, c[0], &frames(RELAY, 0, 1));
    // Slot 0 reports slot 2 dropped, as itself and as the relay.
    for from in [0, RELAY] {
        let forged = wire::encode(&Envelope {
            from,
            body: Body::Dropped { slot: 2, kept: 0 },
        });
        h.receive(101, c[0], &forged);
    }
    assert_eq!(h.take_out(), []);
    let s = h.stats();
    assert_eq!(
        (
            s.outside_game,
            s.forged_senders,
            s.forged_drops,
            s.forwarded
        ),
        (1, 3, 1, 0)
    );
    assert_eq!(h.kept(&code, 1), Some(0), "nothing counted for slot 1");
    assert_eq!(h.connections(), 4, "forgeries are discarded, not closed");
}

#[test]
fn a_lost_person_is_reported_dropped_with_the_frames_everyone_holds() {
    let mut h = hub();
    let (code, c) = game(&mut h, 3);
    h.receive(100, c[0], &frames(0, 0, 3));
    // A gap: frames 5 and 6 do not count until 3 and 4 have come.
    h.receive(101, c[0], &frames(0, 5, 2));
    assert_eq!(h.kept(&code, 0), Some(3));
    // A resend of frames already counted, then 2..4.
    h.receive(102, c[0], &frames(0, 0, 2));
    h.receive(103, c[0], &frames(0, 2, 2));
    assert_eq!(h.kept(&code, 0), Some(4));
    h.take_out();
    h.closed(104, c[0]);
    let out = game_out(&h.take_out());
    let drop = Envelope {
        from: RELAY,
        body: Body::Dropped { slot: 0, kept: 4 },
    };
    assert_eq!(out, [(c[1], drop.clone()), (c[2], drop)]);
    assert_eq!(h.stats().drops, 1);

    // Slot 1 leaves by the lobby's Leave: a drop too.
    h.receive(105, c[1], &frames(1, 0, 1));
    h.receive(106, c[1], &msg(&ToRelay::Leave));
    let out = game_out(&h.take_out());
    assert_eq!(
        out.last(),
        Some(&(
            c[2],
            Envelope {
                from: RELAY,
                body: Body::Dropped { slot: 1, kept: 1 },
            }
        ))
    );
    assert_eq!(h.rooms(), 1);
    // The last person goes: the room goes.
    h.closed(107, c[2]);
    assert_eq!(h.take_out(), []);
    assert_eq!(h.rooms(), 0);
}

#[test]
fn the_drop_comes_after_everything_forwarded_from_the_slot() {
    let mut h = hub();
    let (_, c) = game(&mut h, 2);
    h.take_out();
    h.receive(100, c[0], &frames(0, 0, 5));
    h.closed(100, c[0]);
    let out = game_out(&h.take_out());
    assert!(matches!(
        &out[..],
        [
            (
                _,
                Envelope {
                    from: 0,
                    body: Body::Frames(_)
                }
            ),
            (
                _,
                Envelope {
                    from: RELAY,
                    body: Body::Dropped { slot: 0, kept: 5 }
                }
            ),
        ]
    ));
}

#[test]
fn bytes_that_are_no_message_close_the_connection_with_a_reason() {
    let mut h = hub();
    let v = wire::WIRE_VERSION;
    let cases: [(Vec<u8>, &str); 5] = [
        (b"hello".to_vec(), "not a Glasnost message"),
        (Vec::new(), "not a Glasnost message"),
        (
            vec![b'G', b'L', v, 0xff, 0xff, 0xff],
            "unreadable lobby message",
        ),
        (
            vec![b'G', b'N', v, 0, 0xff, 0xff],
            "unreadable game message",
        ),
        (vec![b'G', b'N', v + 6, 0], "another protocol version"),
    ];
    for (bytes, reason) in cases {
        let c = h.connect(0).unwrap();
        h.receive(0, c, &bytes);
        let out = h.take_out();
        let lob = lobby_out(&out);
        assert!(
            matches!(&lob[..], [(_, FromRelay::Closing(r))] if r.starts_with(reason)),
            "{bytes:?}: {lob:?}"
        );
        let cl = closes(&out);
        assert_eq!(cl.len(), 1);
        assert!(cl[0].1.starts_with(reason), "{bytes:?}: {cl:?}");
    }
    assert_eq!(h.stats().closed_unreadable, 5);
    // Another lobby protocol version is refused naming the protocol.
    let c = h.connect(0).unwrap();
    h.receive(0, c, b"GL\x63rest");
    let lob = lobby_out(&h.take_out());
    assert_eq!(
        lob[0],
        (c, FromRelay::Refused(Refusal::Mismatch(Field::Protocol)))
    );
    assert_eq!(h.connections(), 0);
}

#[test]
fn the_limits_on_size_rate_rooms_and_connections_hold() {
    let mut h = hub_with(Limits {
        max_message: 100,
        max_connections: 3,
        max_rooms: 1,
        rate: 10,
        burst: 5,
        ..Limits::default()
    });
    // Size.
    let a = h.connect(0).unwrap();
    let mut big = b"GL".to_vec();
    big.resize(101, 0);
    h.receive(0, a, &big);
    assert_eq!(closes(&h.take_out()), [(a, "message too large".to_owned())]);
    // Rooms.
    let b = h.connect(0).unwrap();
    h.receive(0, b, &create(&[SlotKind::Human]));
    let c = h.connect(0).unwrap();
    h.receive(0, c, &create(&[SlotKind::Human]));
    assert!(matches!(
        &lobby_out(&h.take_out())[..],
        [_, (_, FromRelay::Refused(Refusal::TooManyRooms))]
    ));
    // Connections: b and c are open.
    let d = h.connect(0).unwrap();
    assert_eq!(h.connect(0), None);
    assert_eq!(h.stats().refused_connections, 1);
    // Rate: a burst of 5, then 10 a second.
    let peek = msg(&ToRelay::Peek {
        code: h.codes()[0].clone(),
    });
    for _ in 0..5 {
        h.receive(1000, d, &peek);
    }
    assert!(closes(&h.take_out()).is_empty());
    h.receive(1100, d, &peek);
    assert!(
        closes(&h.take_out()).is_empty(),
        "a tenth of a second earns one"
    );
    h.receive(1100, d, &peek);
    assert_eq!(closes(&h.take_out()), [(d, "too many messages".to_owned())]);
    let s = h.stats();
    assert_eq!((s.closed_size, s.closed_rate), (1, 1));
}

#[test]
fn players_per_room_are_the_rows_of_the_setup() {
    let mut h = hub();
    let host = h.connect(0).unwrap();
    h.receive(0, host, &create(&[SlotKind::Human, SlotKind::Human]));
    let code = h.codes()[0].clone();
    let b = h.connect(0).unwrap();
    h.receive(0, b, &join(&code, "B", "build-1"));
    let c = h.connect(0).unwrap();
    h.take_out();
    h.receive(0, c, &join(&code, "C", "build-1"));
    assert_eq!(
        lobby_out(&h.take_out()),
        [(c, FromRelay::Refused(Refusal::RoomFull))]
    );
}

#[test]
fn silent_and_idle_connections_close_and_keepalives_go_out() {
    let mut h = hub_with(Limits {
        idle_ms: 10_000,
        silent_ms: 3_000,
        keepalive_ms: 1_000,
        ..Limits::default()
    });
    let (_, c) = game(&mut h, 2);
    let lone = h.connect(0).unwrap();
    h.take_out();
    h.tick(1_000);
    let out = h.take_out();
    let mut kept: Vec<ConnId> = out
        .iter()
        .filter_map(|o| match o {
            Out::Keepalive { to } => Some(*to),
            _ => None,
        })
        .collect();
    kept.sort_unstable();
    assert_eq!(kept, [c[0], c[1], lone]);
    // Slot 0 answers; slot 1 and the lone connection stay silent.
    for t in [2_000, 2_900] {
        h.heard(t, c[0]);
        h.heard(t, lone);
        h.tick(t);
    }
    h.take_out();
    h.tick(3_000);
    let out = h.take_out();
    assert_eq!(closes(&out), [(c[1], "silent for too long".to_owned())]);
    assert_eq!(
        game_out(&out),
        [(
            c[0],
            Envelope {
                from: RELAY,
                body: Body::Dropped { slot: 1, kept: 0 },
            }
        )]
    );
    // The lone connection is heard but in no room.
    for t in [5_000, 7_000, 9_000] {
        h.heard(t, lone);
        h.heard(t, c[0]);
        h.tick(t);
    }
    h.heard(10_000, c[0]);
    h.heard(10_000, lone);
    h.take_out();
    h.tick(10_000);
    assert_eq!(
        closes(&h.take_out()),
        [(lone, "in no room for too long".to_owned())]
    );
    assert_eq!(h.stats().closed_silent, 2);
}

#[test]
fn random_bytes_never_panic_the_hub() {
    let mut h = hub_with(Limits {
        rate: 1_000_000,
        burst: 1_000_000,
        max_refusals: u32::MAX,
        ..Limits::default()
    });
    let (_, c) = game(&mut h, 2);
    let mut rng = Fake(12345);
    let v = wire::WIRE_VERSION;
    let prefixes: [&[u8]; 5] = [
        b"",
        b"GL",
        &[b'G', b'L', v],
        &[b'G', b'N', v],
        &[b'G', b'N', v, 0],
    ];
    let mut opened = Vec::new();
    for i in 0..20_000u64 {
        let mut len = [0u8; 1];
        rng.fill(&mut len);
        let mut body = vec![0u8; usize::from(len[0] % 48)];
        rng.fill(&mut body);
        let mut bytes = prefixes[(i % 5) as usize].to_vec();
        bytes.extend(body);
        let to = match i % 3 {
            0 => c[(i % 2) as usize],
            _ => {
                let id = h.connect(i).unwrap_or(1);
                opened.push(id);
                id
            }
        };
        h.receive(i, to, &bytes);
        h.tick(i);
        let _ = h.take_out();
    }
}

fn chat(text: &str) -> Vec<u8> {
    msg(&ToRelay::Chat { text: text.into() })
}

/// The chat lines the hub sent, by receiver.
fn chats(outs: &[Out]) -> Vec<(ConnId, ChatLine)> {
    lobby_out(outs)
        .into_iter()
        .filter_map(|(to, m)| match m {
            FromRelay::Chat(c) => Some((to, c)),
            _ => None,
        })
        .collect()
}

#[test]
fn chat_reaches_every_member_in_the_room_and_in_the_game_cleaned() {
    let mut h = hub();
    let host = h.connect(0).unwrap();
    h.receive(0, host, &create(&[SlotKind::Human, SlotKind::Human]));
    let code = h.codes()[0].clone();
    let guest = h.connect(0).unwrap();
    h.receive(0, guest, &join(&code, "Guest", "build-1"));
    let outsider = h.connect(0).unwrap();
    h.take_out();
    h.receive(10, guest, &chat("  hello\u{7} there  "));
    let got = chats(&h.take_out());
    let to: Vec<ConnId> = got.iter().map(|(t, _)| *t).collect();
    assert_eq!(
        to,
        [host, guest],
        "every member, the sender too, no outsider"
    );
    for (_, c) in &got {
        assert_eq!(
            *c,
            ChatLine {
                from: guest,
                row: 1,
                name: "Guest".into(),
                text: "hello there".into(),
            }
        );
    }
    // A long line is cut, an empty one refused; nobody outside a room chats.
    h.receive(20, host, &chat(&"x".repeat(CHAT_CHARS + 50)));
    let got = chats(&h.take_out());
    assert_eq!(got[0].1.text.chars().count(), CHAT_CHARS);
    h.receive(30, host, &chat(" \n "));
    assert_eq!(
        lobby_out(&h.take_out()),
        [(host, FromRelay::Refused(Refusal::Bad(Value::Text)))]
    );
    h.receive(40, outsider, &chat("hi"));
    assert_eq!(
        lobby_out(&h.take_out()),
        [(outsider, FromRelay::Refused(Refusal::NotInRoom))]
    );
    assert_eq!(h.stats().chats, 2);

    // During the game, too, to every person still in it.
    let mut h = hub();
    let (_, by_slot) = game(&mut h, 3);
    h.receive(100, by_slot[2], &chat("gg"));
    let got = chats(&h.take_out());
    let mut to: Vec<ConnId> = got.iter().map(|(t, _)| *t).collect();
    to.sort_unstable();
    let mut all = by_slot.clone();
    all.sort_unstable();
    assert_eq!(to, all);
    h.closed(200, by_slot[0]);
    h.take_out();
    h.receive(300, by_slot[1], &chat("one left"));
    let got = chats(&h.take_out());
    assert_eq!(got.len(), 2, "the one who left hears nothing");
    assert!(got.iter().all(|(t, _)| *t != by_slot[0]));
}

#[test]
fn chat_too_fast_is_refused_without_closing_the_connection() {
    let limits = Limits::default();
    let mut h = hub_with(limits);
    let host = h.connect(0).unwrap();
    h.receive(0, host, &create(&[SlotKind::Human, SlotKind::Ai]));
    h.take_out();
    let burst = limits.chat_burst as usize;
    // Far more lines than the refusal limit, all at once.
    let n = burst + limits.max_refusals as usize + 10;
    for _ in 0..n {
        h.receive(0, host, &chat("spam"));
    }
    let outs = h.take_out();
    assert_eq!(chats(&outs).len(), burst, "the burst goes through");
    let refused = lobby_out(&outs)
        .into_iter()
        .filter(|(_, m)| *m == FromRelay::Refused(Refusal::ChatTooFast))
        .count();
    assert_eq!(refused, n - burst);
    assert!(closes(&outs).is_empty(), "too fast chat never closes");
    assert_eq!(h.stats().chats_too_fast, (n - burst) as u64);
    // The bucket refills: one line per chat_every_ms.
    let later = limits.chat_every_ms;
    h.receive(later, host, &chat("again"));
    assert_eq!(chats(&h.take_out()).len(), 1);
    h.receive(later, host, &chat("again"));
    assert!(chats(&h.take_out()).is_empty());
}

// ---- Direct links (N7) ----

fn offer() -> Signal {
    Signal::Offer("v=0\r\no=- 1 1 IN IP4 0.0.0.0\r\n".into())
}

fn signal(to: u8, signal: Signal) -> Vec<u8> {
    msg(&ToRelay::Signal {
        to,
        attempt: 1,
        signal,
    })
}

/// The signals the hub passed on, by receiver: (to, from slot).
fn signals(outs: &[Out]) -> Vec<(ConnId, u8)> {
    lobby_out(outs)
        .into_iter()
        .filter_map(|(to, m)| match m {
            FromRelay::Signal { from, .. } => Some((to, from)),
            _ => None,
        })
        .collect()
}

fn held(from: u8, slot: u8, first: u32, n: usize, last: bool) -> Vec<u8> {
    // Frames told apart by their delay: 1, 2, … within each part.
    wire::encode(&Envelope {
        from,
        body: Body::Held {
            slot,
            first,
            frames: (0..n)
                .map(|i| Frame {
                    delay: 1 + (i % 200) as u8,
                    ..Frame::default()
                })
                .collect(),
            last,
        },
    })
}

/// [`game`], with each slot's ticket.
fn game_tickets(h: &mut Hub, persons: usize) -> (String, Vec<ConnId>, Vec<[u8; 16]>) {
    let mut rows = vec![SlotKind::Human; persons];
    rows.push(SlotKind::Ai);
    let host = h.connect(0).unwrap();
    h.receive(0, host, &create(&rows));
    let code = h.codes()[0].clone();
    let mut conns = vec![host];
    for i in 1..persons {
        let c = h.connect(0).unwrap();
        h.receive(0, c, &join(&code, &format!("P{i}"), "build-1"));
        conns.push(c);
    }
    for &c in &conns[1..] {
        h.receive(0, c, &ready());
    }
    h.receive(0, host, &msg(&ToRelay::Start));
    let mut by_slot = vec![0; persons];
    let mut tickets = vec![[0; 16]; persons];
    for (to, m) in lobby_out(&h.take_out()) {
        if let FromRelay::Begin(b) = m {
            by_slot[usize::from(b.you)] = to;
            tickets[usize::from(b.you)] = b.ticket;
        }
    }
    (code, by_slot, tickets)
}

#[test]
fn signals_pass_between_live_persons_of_a_game_only() {
    let mut h = hub();
    // Before a game: nobody to signal.
    let lone = h.connect(0).unwrap();
    h.receive(0, lone, &signal(1, offer()));
    assert!(signals(&h.take_out()).is_empty());
    let (_, c) = game(&mut h, 3);
    h.take_out();
    // To each other person, from the sender's slot, the sender's attempt
    // and signal unchanged.
    h.receive(10, c[0], &signal(2, offer()));
    let out = lobby_out(&h.take_out());
    assert_eq!(
        out,
        [(
            c[2],
            FromRelay::Signal {
                from: 0,
                attempt: 1,
                signal: offer()
            }
        )]
    );
    h.receive(
        10,
        c[2],
        &signal(0, Signal::Candidate("{\"candidate\":\"\"}".into())),
    );
    assert_eq!(signals(&h.take_out()), [(c[0], 2)]);
    // A relay-only peer's decline (M9 N8c) has no text and passes too.
    h.receive(10, c[2], &signal(0, Signal::Decline));
    assert_eq!(
        lobby_out(&h.take_out()),
        [(
            c[0],
            FromRelay::Signal {
                from: 2,
                attempt: 1,
                signal: Signal::Decline
            }
        )]
    );
    // Not to itself, a computer's slot, an empty or a missing one; nothing
    // that does not look like a signal.
    let s0 = h.stats();
    for (to, sig) in [
        (0, offer()),
        (3, offer()),
        (7, offer()),
        (200, offer()),
        (1, Signal::Offer("hello".into())),
        (1, Signal::Answer(String::new())),
        (1, Signal::Candidate("not json".into())),
        (
            1,
            Signal::Offer(format!("v=0{}", "x".repeat(lobby::MAX_SIGNAL))),
        ),
        (1, Signal::Offer("v=0\u{7}bell".into())),
    ] {
        h.receive(11, c[0], &signal(to, sig));
    }
    let outs = h.take_out();
    assert!(signals(&outs).is_empty());
    assert!(closes(&outs).is_empty(), "a bad signal closes nothing");
    let s1 = h.stats();
    assert_eq!(s1.signals_dropped, s0.signals_dropped + 9);
    assert_eq!(s1.signals, 3);
    // Nor to a slot that dropped.
    h.closed(12, c[1]);
    h.take_out();
    h.receive(13, c[0], &signal(1, offer()));
    assert!(signals(&h.take_out()).is_empty());
}

#[test]
fn signals_too_fast_are_dropped_and_a_ping_is_answered() {
    let limits = Limits::default();
    let mut h = hub_with(limits);
    let (_, c) = game(&mut h, 2);
    h.take_out();
    let burst = limits.signal_burst as usize;
    for _ in 0..burst + 20 {
        h.receive(100, c[0], &signal(1, offer()));
    }
    let outs = h.take_out();
    assert_eq!(signals(&outs).len(), burst);
    assert!(closes(&outs).is_empty());
    assert_eq!(h.stats().signals_dropped, 20);
    // The bucket refills.
    h.receive(100 + limits.signal_every_ms, c[0], &signal(1, offer()));
    assert_eq!(signals(&h.take_out()).len(), 1);
    // A ping, in or out of a game, is answered with its stamp.
    h.receive(200, c[1], &msg(&ToRelay::Ping { stamp: 77 }));
    assert_eq!(
        lobby_out(&h.take_out()),
        [(c[1], FromRelay::Pong { stamp: 77 })]
    );
}

#[test]
fn a_drop_of_a_slot_without_direct_links_is_reported_at_once() {
    let mut h = hub();
    let (_, c) = game(&mut h, 3);
    // Slots 1 and 2 signalled; slot 0 did not.
    h.receive(1, c[1], &signal(2, offer()));
    h.receive(2, c[0], &frames(0, 0, 4));
    h.take_out();
    h.closed(3, c[0]);
    let out = game_out(&h.take_out());
    let drop = Envelope {
        from: RELAY,
        body: Body::Dropped { slot: 0, kept: 4 },
    };
    assert_eq!(out, [(c[1], drop.clone()), (c[2], drop)]);
    assert_eq!(h.stats().rounds, 0);
}

#[test]
fn a_drop_of_a_slot_with_direct_links_waits_for_its_peers_frames() {
    let mut h = hub();
    let (code, c, tickets) = game_tickets(&mut h, 3);
    // Slot 2 signalled with 0 and 1; the hub forwarded 5 of its frames.
    h.receive(1, c[2], &signal(0, offer()));
    h.receive(1, c[2], &signal(1, offer()));
    h.receive(2, c[2], &frames(2, 0, 5));
    h.take_out();
    h.closed(3, c[2]);
    let out = game_out(&h.take_out());
    let ask = Envelope {
        from: RELAY,
        body: Body::Closing {
            slot: 2,
            forwarded: 5,
        },
    };
    assert_eq!(
        out,
        [(c[0], ask.clone()), (c[1], ask)],
        "asked, not dropped"
    );
    assert_eq!(h.kept(&code, 2), Some(5));
    // A rejoin of the slot waits for the round.
    let r = h.connect(4).unwrap();
    h.receive(
        4,
        r,
        &msg(&ToRelay::Rejoin {
            code: code.clone(),
            slot: 2,
            ticket: tickets[2],
            build: "build-1".into(),
            rules_hash: "rules-1".into(),
        }),
    );
    assert_eq!(
        lobby_out(&h.take_out()),
        [(r, FromRelay::Refused(Refusal::Rejoining))]
    );
    // Slot 0 held frames 5 to 7 and answers in two parts; slot 1 held 5 to
    // 10 (its first three the same frames) and answers in one. The answers
    // are not forwarded.
    h.receive(5, c[0], &held(0, 2, 5, 2, false));
    h.receive(5, c[0], &held(0, 2, 7, 1, true));
    assert!(
        game_out(&h.take_out()).is_empty(),
        "still waiting for slot 1"
    );
    h.receive(6, c[1], &held(1, 2, 5, 6, true));
    let out = game_out(&h.take_out());
    let to0: Vec<&Body> = out
        .iter()
        .filter(|(t, _)| *t == c[0])
        .map(|(_, e)| &e.body)
        .collect();
    let to1: Vec<&Body> = out
        .iter()
        .filter(|(t, _)| *t == c[1])
        .map(|(_, e)| &e.body)
        .collect();
    assert_eq!(to0, to1, "the same to everyone");
    assert!(out.iter().all(|(_, e)| e.from == RELAY));
    let [
        Body::Held {
            slot: 2,
            first: 5,
            frames,
            last: true,
        },
        Body::Dropped { slot: 2, kept: 11 },
    ] = &to0[..]
    else {
        panic!("{to0:?}");
    };
    let delays: Vec<u8> = frames.iter().map(|f| f.delay).collect();
    // The first copy of each frame wins: slot 0's 5 and 6 (its first
    // part), its 7, then slot 1's 8 to 10.
    assert_eq!(delays, [1, 2, 1, 4, 5, 6]);
    assert_eq!(h.kept(&code, 2), Some(11));
    let s = h.stats();
    assert_eq!(
        (s.rounds, s.held_parts, s.held_refused, s.drops),
        (1, 3, 0, 1)
    );
}

#[test]
fn answers_out_of_turn_are_refused_and_a_silent_peer_is_closed() {
    let limits = Limits::default();
    let mut h = hub_with(limits);
    let (code, c) = game(&mut h, 3);
    h.receive(1, c[0], &signal(1, offer()));
    h.receive(1, c[0], &signal(2, offer()));
    h.receive(2, c[0], &frames(0, 0, 3));
    h.take_out();
    // A peer cannot ask for a round.
    h.receive(
        2,
        c[1],
        &wire::encode(&Envelope {
            from: 1,
            body: Body::Closing {
                slot: 0,
                forwarded: 0,
            },
        }),
    );
    assert!(game_out(&h.take_out()).is_empty());
    assert_eq!(h.stats().forged_drops, 1);
    // An answer nobody asked for is not taken.
    h.receive(2, c[1], &held(1, 0, 3, 1, true));
    assert_eq!(h.stats().held_refused, 1);
    h.closed(3, c[0]);
    h.take_out();
    // Slot 1's answer starts at the wrong frame: that answer ends, empty.
    h.receive(4, c[1], &held(1, 0, 4, 1, true));
    // Slot 2 says nothing; at the deadline it is closed, and the drop
    // goes to slot 1 with nothing more.
    h.tick(3 + limits.answer_ms - 1);
    assert!(game_out(&h.take_out()).is_empty());
    h.tick(3 + limits.answer_ms);
    let outs = h.take_out();
    assert_eq!(
        closes(&outs),
        [(c[2], "did not answer a drop in time".to_owned())]
    );
    let drops: Vec<(ConnId, Body)> = game_out(&outs)
        .into_iter()
        .filter(|(_, e)| matches!(e.body, Body::Dropped { .. }))
        .map(|(t, e)| (t, e.body))
        .collect();
    assert_eq!(drops[0], (c[1], Body::Dropped { slot: 0, kept: 3 }));
    assert_eq!(h.kept(&code, 0), Some(3));
    let s = h.stats();
    assert_eq!((s.held_refused, s.unanswered), (2, 1), "{s:?}");
    // Slot 2's own drop follows (it never signalled with slot 1 itself, so
    // at once).
    assert!(drops.contains(&(c[1], Body::Dropped { slot: 2, kept: 0 })));
}

#[test]
fn an_answer_with_a_marker_or_past_a_lead_ends_and_a_peer_that_leaves_is_not_waited_for() {
    let mut h = hub();
    let (code, c) = game(&mut h, 3);
    h.receive(1, c[2], &signal(0, offer()));
    h.receive(1, c[2], &signal(1, offer()));
    h.take_out();
    h.closed(2, c[2]);
    h.take_out();
    // Slot 0's answer carries a readmission marker: refused, it ends.
    h.receive(
        3,
        c[0],
        &wire::encode(&Envelope {
            from: 0,
            body: Body::Held {
                slot: 2,
                first: 0,
                frames: vec![Frame {
                    resume: Some(1),
                    ..Frame::default()
                }],
                last: true,
            },
        }),
    );
    assert!(game_out(&h.take_out()).is_empty(), "slot 1 still to answer");
    // Slot 1 leaves before it answers: the round ends without it, and its
    // own drop follows.
    h.closed(4, c[1]);
    let out = game_out(&h.take_out());
    let bodies: Vec<&Body> = out.iter().map(|(_, e)| &e.body).collect();
    assert!(
        bodies.contains(&&Body::Dropped { slot: 2, kept: 0 }),
        "{bodies:?}"
    );
    assert!(
        bodies.contains(&&Body::Dropped { slot: 1, kept: 0 }),
        "{bodies:?}"
    );
    assert_eq!(h.kept(&code, 2), Some(0));
    // Past a lead: an answer longer than that ends.
    let mut h = hub();
    let (_, c) = game(&mut h, 2);
    h.receive(1, c[1], &signal(0, offer()));
    h.take_out();
    h.closed(2, c[1]);
    h.take_out();
    let lead = glasnost_net::session::lead(&Limits::default().params) as usize;
    h.receive(3, c[0], &held(0, 1, 0, lead + 1, true));
    let out = game_out(&h.take_out());
    assert_eq!(
        out.last().map(|(_, e)| &e.body),
        Some(&Body::Dropped { slot: 1, kept: 0 })
    );
    assert_eq!(h.stats().held_refused, 1);
}

/// The counts behind the relay's periodic line (M9 N8b): messages and bytes
/// in and out, refusals, rooms, connections and games, as differences since
/// the last line, and no code, name or address in the text.
#[test]
fn the_stats_line_counts_since_the_last_line_and_names_nobody() {
    use glasnost_relay::stats_line;
    let mut h = hub();
    let before = h.stats();
    let (code, conns) = game(&mut h, 2);
    h.take_out();
    // A refused join: another build.
    let c = h.connect(1).unwrap();
    h.receive(1, c, &join(&code, "Cy", "build-9"));
    h.take_out();
    let first = h.stats().since(&before);
    assert!(first.messages_in >= 4, "{first:?}");
    assert!(first.bytes_in > 0 && first.bytes_out > 0);
    assert!(first.messages_out >= 1);
    assert_eq!(first.refusals, 1);
    let line = stats_line(&h, &first, 60);
    assert!(
        line.starts_with("stats rooms=1 connections=3 games=1 in="),
        "{line}"
    );
    assert!(
        line.contains("refused=1 drops=0 rejoins=0 over=60s"),
        "{line}"
    );
    assert!(!line.contains(&code) && !line.contains("Cy"), "{line}");
    // A quiet minute counts nothing new.
    let mark = h.stats();
    let quiet = stats_line(&h, &h.stats().since(&mark), 60);
    assert!(
        quiet.contains("in=0msg/0B out=0msg/0B refused=0"),
        "{quiet}"
    );
    // A drop is counted in the next line only.
    h.closed(2, conns[1]);
    h.take_out();
    assert_eq!(h.stats().since(&mark).drops, 1);
}
