//! The lobby's room state machine and messages, without a relay: creating,
//! joining, refusals that name the field, who may change what, the choice
//! rules of the setup, readiness, leaving and the start values. The
//! protocol and the room rules are our own design; the choice rules cite
//! the claims they follow.

use glasnost_net::lobby::{
    self, Begin, Edit, FromRelay, MapChoice, Offered, Refusal, Room, RoomView, ToRelay, Value,
};
use glasnost_net::{Field, Params, SlotKind as NetKind};
use glasnost_ui::setup::{Problem, Setup, Slot, SlotKind};

const HOST: u64 = 10;
const B: u64 = 11;
const C: u64 = 12;

fn map(hash: &str, starts: u32) -> MapChoice {
    MapChoice {
        hash: hash.into(),
        label: format!("map {hash}"),
        starts,
    }
}

fn offered() -> Offered {
    Offered {
        countries: 3,
        credits: (1000, 5000),
        units: (1, 9),
    }
}

/// The host's first setup: its own row, two open persons' rows, a computer.
fn first_setup() -> Setup {
    let mut slots = [Slot::closed(); 8];
    slots[0].kind = SlotKind::Human;
    slots[1].kind = SlotKind::Human;
    slots[2].kind = SlotKind::Ai;
    slots[3].kind = SlotKind::Human;
    Setup {
        map: 0,
        slots,
        credits: 2000,
        units: 3,
        flags: [false, true, true, true, true],
        speed: 2,
    }
}

fn create() -> ToRelay {
    ToRelay::Create {
        name: "Host".into(),
        build: "build-1".into(),
        rules_hash: "rules-1".into(),
        offered: offered(),
        map: map("m4", 4),
        setup: first_setup(),
    }
}

fn join(name: &str) -> ToRelay {
    ToRelay::Join {
        code: "CODE0000".into(),
        name: name.into(),
        build: "build-1".into(),
        rules_hash: "rules-1".into(),
        map_hash: "m4".into(),
    }
}

fn room() -> Room {
    Room::create("CODE0000".into(), HOST, &create()).expect("a room")
}

fn choices(row: u8, colour: Option<u8>, start: Option<u32>) -> Edit {
    Edit::Choices {
        row,
        country: None,
        colour,
        team: None,
        start,
    }
}

#[test]
fn create_and_join_take_the_persons_rows_in_order() {
    let mut r = room();
    assert_eq!(r.row_of(HOST), Some(0));
    assert_eq!(r.join(B, &join("Bea")), Ok(1));
    assert_eq!(r.join(C, &join("Cy")), Ok(3));
    assert_eq!(r.join(13, &join("Dee")), Err(Refusal::RoomFull));
    assert_eq!(r.join(B, &join("Bea")), Err(Refusal::InRoom));
    let info = r.info();
    assert_eq!((info.persons, info.open, info.playing), (3, 0, false));
    let v: RoomView = r.view(B);
    assert_eq!(v.you, B);
    assert_eq!(v.host, HOST);
    let rows: Vec<(u8, &str)> = v.members.iter().map(|m| (m.row, m.name.as_str())).collect();
    assert_eq!(rows, [(0, "Host"), (1, "Bea"), (3, "Cy")]);
}

#[test]
fn a_join_with_another_build_rules_or_map_is_refused_naming_the_field() {
    let mut r = room();
    for (field, msg) in [
        (
            Field::Build,
            ToRelay::Join {
                code: "CODE0000".into(),
                name: "B".into(),
                build: "build-2".into(),
                rules_hash: "rules-1".into(),
                map_hash: "m4".into(),
            },
        ),
        (
            Field::RulesHash,
            ToRelay::Join {
                code: "CODE0000".into(),
                name: "B".into(),
                build: "build-1".into(),
                rules_hash: "rules-2".into(),
                map_hash: "m4".into(),
            },
        ),
        (
            Field::MapHash,
            ToRelay::Join {
                code: "CODE0000".into(),
                name: "B".into(),
                build: "build-1".into(),
                rules_hash: "rules-1".into(),
                map_hash: "other".into(),
            },
        ),
    ] {
        let refusal = r.join(B, &msg).unwrap_err();
        assert_eq!(refusal, Refusal::Mismatch(field));
        assert!(refusal.to_string().contains(field.name()), "{refusal}");
    }
    assert!(!r.has(B));
    // A name with nothing to show is refused; a long one is cut.
    assert_eq!(r.join(B, &join(" \u{7} ")), Err(Refusal::Bad(Value::Name)));
    r.join(B, &join(&"x".repeat(100))).expect("joins");
    assert_eq!(r.view(B).members[1].name.len(), lobby::room::NAME_CHARS);
}

#[test]
fn only_the_host_edits_the_room_and_each_person_their_own_row() {
    let mut r = room();
    r.join(B, &join("Bea")).unwrap();
    // The game-wide values and the map are the host's.
    let game = Edit::Game {
        credits: 3000,
        units: 5,
        flags: [true; 5],
        speed: 6,
    };
    assert_eq!(r.edit(B, &game), Err(Refusal::NotHost));
    assert_eq!(r.edit(HOST, &game), Ok(()));
    assert_eq!(r.setup().credits, 3000);
    assert_eq!(r.edit(B, &Edit::Map(map("m8", 8))), Err(Refusal::NotHost));
    // A person sets their own row, not another's; the host sets the computer's.
    assert_eq!(r.edit(B, &choices(1, Some(4), None)), Ok(()));
    assert_eq!(r.setup().slots[1].colour, Some(4));
    assert_eq!(
        r.edit(B, &choices(0, Some(5), None)),
        Err(Refusal::NotYours(0))
    );
    assert_eq!(
        r.edit(HOST, &choices(1, Some(5), None)),
        Err(Refusal::NotYours(1))
    );
    assert_eq!(
        r.edit(B, &choices(2, Some(5), None)),
        Err(Refusal::NotYours(2))
    );
    assert_eq!(r.edit(HOST, &choices(2, Some(5), None)), Ok(()));
    // The host opens, closes and gives rows to computers, not a taken row.
    let row = |row, kind| Edit::Row { row, kind };
    assert_eq!(
        r.edit(HOST, &row(1, SlotKind::Closed)),
        Err(Refusal::Seated(1))
    );
    assert_eq!(r.edit(B, &row(3, SlotKind::Ai)), Err(Refusal::NotHost));
    assert_eq!(r.edit(HOST, &row(3, SlotKind::Ai)), Ok(()));
    assert_eq!(r.setup().slots[3].kind, SlotKind::Ai);
    // spec: S-0412 claim 15 (only the map's rows are offered)
    assert_eq!(r.edit(HOST, &row(4, SlotKind::Ai)), Err(Refusal::NoRow(4)));
    // Values outside the offered ranges.
    let bad = |credits, units, speed| Edit::Game {
        credits,
        units,
        flags: [false; 5],
        speed,
    };
    assert_eq!(
        r.edit(HOST, &bad(999, 3, 0)),
        Err(Refusal::Bad(Value::Credits))
    );
    assert_eq!(
        r.edit(HOST, &bad(1000, 10, 0)),
        Err(Refusal::Bad(Value::Units))
    );
    // spec: S-0201 claim 7 (seven speed positions)
    assert_eq!(
        r.edit(HOST, &bad(1000, 1, 7)),
        Err(Refusal::Bad(Value::Speed))
    );
}

// spec: S-0412 claims 7 and 15 (colour slots 0 … 7, a colour fixed by one player is not free for another; the offered countries; no team or four teams); S-0009 claim 17 (start positions of the map's start list)
#[test]
fn the_choice_rules_of_the_setup_are_enforced() {
    let mut r = room();
    r.join(B, &join("Bea")).unwrap();
    assert_eq!(r.edit(HOST, &choices(0, Some(2), Some(1))), Ok(()));
    assert_eq!(
        r.edit(B, &choices(1, Some(2), None)),
        Err(Refusal::Setup(Problem::SameColour))
    );
    assert_eq!(
        r.edit(B, &choices(1, None, Some(1))),
        Err(Refusal::Setup(Problem::SameStart))
    );
    assert_eq!(
        r.edit(B, &choices(1, Some(8), None)),
        Err(Refusal::Bad(Value::Colour))
    );
    assert_eq!(
        r.edit(B, &choices(1, None, Some(4))),
        Err(Refusal::Bad(Value::Start))
    );
    let with = |country, team| Edit::Choices {
        row: 1,
        country,
        colour: None,
        team,
        start: None,
    };
    assert_eq!(
        r.edit(B, &with(Some(3), None)),
        Err(Refusal::Bad(Value::Country))
    );
    assert_eq!(
        r.edit(B, &with(None, Some(0))),
        Err(Refusal::Bad(Value::Team))
    );
    assert_eq!(
        r.edit(B, &with(None, Some(5))),
        Err(Refusal::Bad(Value::Team))
    );
    assert_eq!(r.edit(B, &with(Some(2), Some(4))), Ok(()));
    // Two players on random are not in each other's way.
    assert_eq!(r.edit(B, &choices(1, None, None)), Ok(()));
    assert_eq!(r.edit(HOST, &choices(2, None, None)), Ok(()));
}

// spec: S-0412 claim 15 (on a map with fewer rows, the rows beyond it are emptied and hidden)
#[test]
fn a_map_change_empties_the_rows_beyond_and_clears_far_starts() {
    let mut r = room();
    r.join(B, &join("Bea")).unwrap();
    r.edit(HOST, &choices(0, None, Some(3))).unwrap();
    r.edit(HOST, &Edit::Map(map("m2", 2))).unwrap();
    assert_eq!(r.map().hash, "m2");
    assert_eq!(r.setup().slots[0].start, None, "start 4 of 2 is random");
    assert_eq!(
        r.setup().slots[2].kind,
        SlotKind::Closed,
        "the computer row is emptied"
    );
    assert_eq!(
        r.setup().slots[3].kind,
        SlotKind::Closed,
        "the open row is closed"
    );
    // A map with fewer rows than a taken row is refused.
    let mut r = room();
    r.join(B, &join("Bea")).unwrap();
    r.join(C, &join("Cy")).unwrap();
    assert_eq!(
        r.edit(HOST, &Edit::Map(map("m2", 2))),
        Err(Refusal::Seated(3))
    );
    assert_eq!(
        r.edit(HOST, &Edit::Map(map("m0", 0))),
        Err(Refusal::Bad(Value::Starts))
    );
}

#[test]
fn ready_needs_the_rooms_map_and_every_change_clears_it() {
    let mut r = room();
    r.join(B, &join("Bea")).unwrap();
    assert_eq!(
        r.set_ready(B, true, "other"),
        Err(Refusal::Mismatch(Field::MapHash))
    );
    r.set_ready(B, true, "m4").unwrap();
    assert!(r.ready(B));
    r.edit(HOST, &choices(2, Some(1), None)).unwrap();
    assert!(!r.ready(B), "an edit clears it");
    r.set_ready(B, true, "m4").unwrap();
    r.join(C, &join("Cy")).unwrap();
    assert!(!r.ready(B), "a join clears it");
    r.set_ready(B, true, "m4").unwrap();
    // After the host changes the map, ready needs the new map's hash.
    r.leave(C);
    r.edit(HOST, &Edit::Map(map("m8", 8))).unwrap();
    assert!(!r.ready(B));
    assert_eq!(
        r.set_ready(B, true, "m4"),
        Err(Refusal::Mismatch(Field::MapHash))
    );
    r.set_ready(B, true, "m8").unwrap();
}

#[test]
fn leaving_opens_the_row_and_passes_the_host_on() {
    let mut r = room();
    r.join(B, &join("Bea")).unwrap();
    r.join(C, &join("Cy")).unwrap();
    r.edit(B, &choices(1, Some(6), Some(2))).unwrap();
    assert!(r.leave(B));
    assert!(!r.leave(B));
    assert_eq!(
        r.setup().slots[1],
        Slot {
            kind: SlotKind::Human,
            ..Slot::closed()
        }
    );
    assert!(r.leave(HOST));
    assert_eq!(r.host(), C);
    assert_eq!(r.join(B, &join("Bea")), Ok(0));
    assert!(r.leave(C) && r.leave(B));
    assert!(r.is_empty());
}

#[test]
fn the_start_needs_every_row_taken_every_person_ready_and_a_sound_setup() {
    let mut r = room();
    r.join(B, &join("Bea")).unwrap();
    let params = Params::default();
    assert_eq!(r.start(B, 1, params), Err(Refusal::NotHost));
    assert_eq!(r.start(HOST, 1, params), Err(Refusal::Open(3)));
    r.edit(
        HOST,
        &Edit::Row {
            row: 3,
            kind: SlotKind::Closed,
        },
    )
    .unwrap();
    assert_eq!(r.start(HOST, 1, params), Err(Refusal::NotReady(1)));
    r.set_ready(B, true, "m4").unwrap();
    let started = r.start(HOST, 77, params).expect("starts");
    assert!(r.playing());
    assert_eq!(r.start(HOST, 1, params), Err(Refusal::Playing));
    assert_eq!(r.join(C, &join("Cy")), Err(Refusal::Playing));
    assert_eq!(r.edit(HOST, &choices(0, None, None)), Err(Refusal::Playing));
    let b = &started.begin;
    assert_eq!(
        (b.build.as_str(), b.rules_hash.as_str(), b.map.hash.as_str()),
        ("build-1", "rules-1", "m4")
    );
    assert_eq!(b.seed, 77);
    assert_eq!(b.params, params.sane());
    assert_eq!(b.rows, [0, 1, 2]);
    assert_eq!(
        b.slots
            .iter()
            .map(|s| (s.house, s.kind))
            .collect::<Vec<_>>(),
        [
            (0, NetKind::Human),
            (1, NetKind::Human),
            (2, NetKind::Computer)
        ]
    );
    assert_eq!(b.names, ["Host", "Bea", "Computer"]);
    assert_eq!(started.persons, [(HOST, 0), (B, 1)]);
    assert_eq!(started.begin_for(B).map(|b| b.you), Some(1));
    assert_eq!(started.begin_for(C), None);
}

// spec: S-0009 claim 19 (persons by increasing colour index, then computers in slot order); Q-1805 (interim: persons on random after them)
#[test]
fn the_slot_table_follows_the_house_order() {
    let mut r = room();
    r.join(B, &join("Bea")).unwrap();
    r.join(C, &join("Cy")).unwrap();
    r.edit(HOST, &choices(0, Some(5), None)).unwrap();
    r.edit(C, &choices(3, Some(1), None)).unwrap();
    r.set_ready(B, true, "m4").unwrap();
    r.set_ready(C, true, "m4").unwrap();
    let s = r.start(HOST, 3, Params::default()).unwrap();
    assert_eq!(s.begin.rows, [3, 0, 1, 2]);
    assert_eq!(s.begin.names, ["Cy", "Host", "Bea", "Computer"]);
    assert_eq!(s.persons, [(C, 0), (HOST, 1), (B, 2)]);
    assert_eq!(s.begin.slots[3].kind, NetKind::Computer);
}

// spec: S-0412 claim 7 (a colour fixed by one player is not free for another); S-0412 claim 15 (the offered countries and rows)
#[test]
fn a_create_with_a_bad_setup_is_refused() {
    let with = |f: &dyn Fn(&mut Setup)| {
        let mut s = first_setup();
        f(&mut s);
        Room::create(
            "CODE0000".into(),
            HOST,
            &ToRelay::Create {
                name: "Host".into(),
                build: "build-1".into(),
                rules_hash: "rules-1".into(),
                offered: offered(),
                map: map("m4", 4),
                setup: s,
            },
        )
        .map(|_| ())
    };
    assert_eq!(with(&|_| {}), Ok(()));
    assert_eq!(
        with(&|s| {
            s.slots[0].colour = Some(1);
            s.slots[2].colour = Some(1);
        }),
        Err(Refusal::Setup(Problem::SameColour))
    );
    assert_eq!(
        with(&|s| s.slots[2].country = Some(9)),
        Err(Refusal::Bad(Value::Country))
    );
    assert_eq!(with(&|s| s.credits = 1), Err(Refusal::Bad(Value::Credits)));
    assert_eq!(
        with(&|s| {
            for r in &mut s.slots {
                if r.kind == SlotKind::Human {
                    r.kind = SlotKind::Ai;
                }
            }
        }),
        Err(Refusal::Bad(Value::Row))
    );
    // Rows beyond the map's are emptied.
    let mut s = first_setup();
    s.slots[6].kind = SlotKind::Ai;
    let r = Room::create(
        "CODE0000".into(),
        HOST,
        &ToRelay::Create {
            name: "Host".into(),
            build: "build-1".into(),
            rules_hash: "rules-1".into(),
            offered: offered(),
            map: map("m4", 4),
            setup: s,
        },
    )
    .unwrap();
    assert_eq!(r.setup().slots[6].kind, SlotKind::Closed);
}

#[test]
fn lobby_messages_round_trip_and_hostile_bytes_never_panic() {
    let msgs = [
        create(),
        join("Bea"),
        ToRelay::Peek {
            code: "CODE0000".into(),
        },
        ToRelay::Leave,
        ToRelay::Edit(choices(1, Some(3), None)),
        ToRelay::Ready {
            ready: true,
            map_hash: "m4".into(),
        },
        ToRelay::Start,
    ];
    for m in &msgs {
        let bytes = lobby::wire::encode(m);
        assert!(lobby::wire::is_lobby(&bytes));
        assert_eq!(lobby::wire::decode::<ToRelay>(&bytes).as_ref(), Ok(m));
    }
    let mut r = room();
    r.join(B, &join("Bea")).unwrap();
    let back = [
        FromRelay::Room(r.view(B)),
        FromRelay::Info(r.info()),
        FromRelay::Refused(Refusal::Mismatch(Field::MapHash)),
        FromRelay::Closing("bye".into()),
    ];
    for m in &back {
        let bytes = lobby::wire::encode(m);
        assert_eq!(lobby::wire::decode::<FromRelay>(&bytes).as_ref(), Ok(m));
    }
    // Another version, short, foreign or trailing bytes are errors.
    let mut bytes = lobby::wire::encode(&ToRelay::Start);
    bytes[2] += 1;
    assert_eq!(
        lobby::wire::decode::<ToRelay>(&bytes),
        Err(lobby::wire::Error::Version(bytes[2]))
    );
    assert_eq!(
        lobby::wire::decode::<ToRelay>(b"GL"),
        Err(lobby::wire::Error::Short)
    );
    assert_eq!(
        lobby::wire::decode::<ToRelay>(b"GN\x02\x00"),
        Err(lobby::wire::Error::Magic)
    );
    let mut bytes = lobby::wire::encode(&ToRelay::Start);
    bytes.push(0);
    assert!(lobby::wire::decode::<ToRelay>(&bytes).is_err());
    // Every truncation and random bytes after a valid header.
    let full = lobby::wire::encode(&create());
    for n in 0..full.len() {
        let _ = lobby::wire::decode::<ToRelay>(&full[..n]);
    }
    let mut x: u64 = 0x9e37_79b9_7f4a_7c15;
    for _ in 0..2000 {
        let mut b = vec![b'G', b'L', glasnost_net::wire::WIRE_VERSION];
        for _ in 0..(x % 64) {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            b.push(x as u8);
        }
        let _ = lobby::wire::decode::<ToRelay>(&b);
        let _ = lobby::wire::decode::<FromRelay>(&b);
    }
}

#[test]
fn begin_checks_the_game_setup_built_from_it() {
    let mut r = room();
    r.join(B, &join("Bea")).unwrap();
    r.edit(
        HOST,
        &Edit::Row {
            row: 3,
            kind: SlotKind::Closed,
        },
    )
    .unwrap();
    r.set_ready(B, true, "m4").unwrap();
    let begin: Begin = r
        .start(HOST, 5, Params::default())
        .unwrap()
        .begin_for(B)
        .unwrap();
    assert_eq!(begin.local_setup(7).map, 7);
    // A game setup whose slots differ is refused on the slot table.
    let sim = glasnost_ai::fixture::rules();
    let players: Vec<glasnost_sim::econ::setup::Player> = (0..3)
        .map(|i| glasnost_sim::econ::setup::Player {
            name: format!("P{i}"),
            country: "Alphaland".into(),
            start: None,
            ai: i == 0,
            values: Default::default(),
        })
        .collect();
    let g = glasnost_sim::econ::setup::skirmish(&sim, &players, Default::default(), 0, 5);
    assert_eq!(begin.start_info(&g), Err(Field::Slots));
    let players: Vec<_> = players
        .into_iter()
        .enumerate()
        .map(|(i, mut p)| {
            p.ai = i == 2;
            p
        })
        .collect();
    let g = glasnost_sim::econ::setup::skirmish(&sim, &players, Default::default(), 0, 6);
    assert_eq!(begin.start_info(&g), Err(Field::Seed));
    let g = glasnost_sim::econ::setup::skirmish(&sim, &players, Default::default(), 0, 5);
    let info = begin.start_info(&g).expect("the room's slots and seed");
    assert_eq!(info.map_hash, "m4");
    assert_eq!(info.slots, begin.slots);
}

#[test]
fn chat_lines_are_cleaned_and_round_trip() {
    assert_eq!(
        lobby::clean_chat("  hi\tthere\u{0} "),
        Some("hithere".into())
    );
    assert_eq!(lobby::clean_chat(" \n\r "), None);
    let long = "\u{e9}".repeat(lobby::CHAT_CHARS * 2);
    assert_eq!(
        lobby::clean_chat(&long).map(|s| s.chars().count()),
        Some(lobby::CHAT_CHARS)
    );
    let to = ToRelay::Chat {
        text: "gl hf".into(),
    };
    assert_eq!(
        lobby::wire::decode::<ToRelay>(&lobby::wire::encode(&to)),
        Ok(to)
    );
    let from = FromRelay::Chat(lobby::ChatLine {
        from: 7,
        row: 2,
        name: "P".into(),
        text: "gl hf".into(),
    });
    assert_eq!(
        lobby::wire::decode::<FromRelay>(&lobby::wire::encode(&from)),
        Ok(from)
    );
    assert_eq!(
        Refusal::ChatTooFast.to_string(),
        "chat lines too fast; that one was not sent"
    );
}
