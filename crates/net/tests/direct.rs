//! The session's direct links (M9 N7): what it takes from a WebRTC data
//! channel and what it refuses, the first copy of a frame winning across
//! the two paths, a differing copy reported, the relay's drop round
//! answered with the frames run beyond what the relay forwarded, and the
//! wire's check of what a peer copies onto a link. Our own netcode
//! (docs/development.md, *WebRTC (N7)*); no spec applies.

use glasnost_net::session::{HELD_PART, held_parts, lead};
use glasnost_net::simnet::SplitMix;
use glasnost_net::wire::{self, Body, Envelope, Frame, Frames, RELAY};
use glasnost_net::{
    Config, Event, Params, Phase, Session, Simulation, Slot, SlotKind, StartInfo, Via,
};
use glasnost_sim::orders::Order;
use glasnost_sim::world::{HouseId, ObjectId};

const TICK_MS: u32 = 33;

fn start(persons: usize, d: u8) -> StartInfo {
    StartInfo {
        build: "test-build".into(),
        rules_hash: "rules".into(),
        map_hash: "map-0".into(),
        setup_hash: "setup".into(),
        seed: 7,
        slots: (0..persons)
            .map(|i| Slot {
                house: i as u8,
                kind: SlotKind::Human,
            })
            .collect(),
        params: Params {
            initial_delay: d,
            min_delay: d,
            max_delay: d,
            hash_every: 5,
            ..Params::default()
        }
        .sane(),
    }
}

fn encode(from: u8, body: Body) -> Vec<u8> {
    wire::encode(&Envelope { from, body })
}

fn frames_msg(from: u8, first: u32, frames: Vec<Frame>) -> Vec<u8> {
    encode(
        from,
        Body::Frames(Frames {
            first,
            frames,
            ..Frames::default()
        }),
    )
}

fn started(info: &StartInfo, me: u8) -> Session {
    let mut s = Session::new(Config::new(TICK_MS), info.clone(), me, 0).expect("slot");
    for i in 0..info.slots.len() as u8 {
        if i != me {
            s.receive(0, &encode(i, Body::Start(info.clone())));
        }
    }
    assert_eq!(s.phase(), Phase::Running);
    s.take_events();
    s
}

/// A frame with one order naming `actor`.
fn frame(d: u8, actor: u32) -> Frame {
    Frame {
        delay: d,
        orders: vec![Order::Stop {
            actor: ObjectId(actor),
        }],
        resume: None,
    }
}

#[derive(Default)]
struct Recorder(Vec<(HouseId, Order)>);

impl Simulation for Recorder {
    fn issue(&mut self, house: HouseId, order: Order) -> bool {
        self.0.push((house, order));
        true
    }
    fn step(&mut self) {}
    fn state_hash(&self) -> wire::Hash {
        [0; 32]
    }
}

fn bad(s: &mut Session) -> usize {
    s.take_events()
        .iter()
        .filter(|e| matches!(e, Event::BadMessage { .. }))
        .count()
}

#[test]
fn a_direct_link_carries_only_its_slots_frames() {
    let info = start(3, 2);
    let mut s = started(&info, 0);
    // Another slot's frames, a start, and every message only the relay
    // sends: each refused, none applied.
    let refused = [
        frames_msg(2, 0, vec![frame(2, 1); 4]),
        encode(1, Body::Start(info.clone())),
        encode(1, Body::Dropped { slot: 2, kept: 0 }),
        encode(RELAY, Body::Dropped { slot: 2, kept: 0 }),
        encode(1, Body::Serve { slot: 2, on: true }),
        encode(RELAY, Body::Resume { slot: 2 }),
        encode(
            RELAY,
            Body::Closing {
                slot: 1,
                forwarded: 0,
            },
        ),
        encode(
            1,
            Body::Held {
                slot: 2,
                first: 0,
                frames: vec![frame(2, 1); 4],
                last: true,
            },
        ),
        b"GN\x06".to_vec(),
        b"not ours".to_vec(),
    ];
    for b in &refused {
        s.receive_via(1, b, Via::Direct(1));
        assert_eq!(bad(&mut s), 1, "{b:?}");
    }
    assert_eq!(s.waiting_for(), vec![1, 2], "nothing applied");
    assert_eq!(s.life(2).map(|l| l.live()), Some(true));
    // A frame with a readmission marker, or past a lead beyond the relay's
    // frames, is not taken from a link.
    let marker = Frame {
        resume: Some(2),
        ..frame(2, 1)
    };
    s.receive_via(1, &frames_msg(1, 0, vec![marker]), Via::Direct(1));
    assert_eq!(bad(&mut s), 0);
    let p = s.paths(1).expect("slot 1");
    assert_eq!((p.gated, p.direct_first), (1, 0), "{p:?}");
    assert_eq!(s.waiting_for(), vec![1, 2]);
    // Its own frames near the relay's are taken.
    s.receive_via(
        1,
        &frames_msg(1, 0, vec![frame(2, 10), frame(2, 11)]),
        Via::Direct(1),
    );
    assert_eq!(s.waiting_for(), vec![2]);
    let p = s.paths(1).expect("slot 1");
    assert_eq!((p.direct_first, p.relay_first, p.relay_top), (2, 0, 0));
    // The relay's copy of the same frames counts as passed on, not again.
    s.receive(2, &frames_msg(1, 0, vec![frame(2, 10), frame(2, 11)]));
    let p = s.paths(1).expect("slot 1");
    assert_eq!((p.direct_first, p.relay_first, p.relay_top), (2, 0, 2));
    assert_eq!(s.take_events(), vec![]);
}

#[test]
fn a_relay_path_a_lead_behind_holds_the_game_back() {
    let d = 2u8;
    let info = start(2, d);
    let mut s = started(&info, 0);
    let mut rec = Recorder::default();
    let lead = lead(s.params());
    // Every frame of slot 1 comes on the link, none through the relay: the
    // game runs on them up to a lead past the relay's frames, then waits.
    let mut at = 1;
    while s.ran() < lead + 5 {
        let t = s.ran();
        s.receive_via(at, &frames_msg(1, t, vec![frame(d, t); 3]), Via::Direct(1));
        if s.step(at, &mut rec).is_none() {
            break;
        }
        at += 1;
    }
    assert_eq!(s.ran(), lead);
    assert_eq!(s.waiting_for(), vec![1]);
    let p = s.paths(1).expect("slot 1");
    assert!(p.gated > 0 && p.relay_top == 0, "{p:?}");
    // The relay's copies come: the game goes on.
    s.receive(
        at,
        &frames_msg(1, 0, (0..lead + 3).map(|t| frame(d, t)).collect()),
    );
    assert!(s.step(at, &mut rec).is_some());
    assert_eq!(s.paths(1).map(|p| p.relay_top), Some(lead + 3));
    assert_eq!(bad(&mut s), 0);
}

#[test]
fn hostile_bytes_on_a_direct_link_never_panic() {
    let info = start(2, 3);
    let mut s = started(&info, 0);
    let good = frames_msg(1, 0, vec![frame(3, 1)]);
    let mut rng = SplitMix::new(9);
    for n in 0..5_000u32 {
        let mut b = good.clone();
        match n % 3 {
            0 => b.truncate(rng.below(b.len() as u64) as usize),
            1 => {
                let i = rng.below(b.len() as u64) as usize;
                b[i] = rng.next_u64() as u8;
            }
            _ => b = (0..rng.below(64)).map(|_| rng.next_u64() as u8).collect(),
        }
        s.receive_via(u64::from(n), &b, Via::Direct(1));
        s.receive_via(u64::from(n), &b, Via::Direct(200));
        let _ = s.poll(u64::from(n));
    }
    // A link for a slot that is not another person's takes nothing.
    let mut s = started(&info, 0);
    s.receive_via(1, &frames_msg(0, 0, vec![frame(3, 1)]), Via::Direct(0));
    s.receive_via(1, &frames_msg(7, 0, vec![frame(3, 1)]), Via::Direct(7));
    assert_eq!(s.waiting_for(), vec![1]);
}

#[test]
fn the_first_copy_wins_and_a_different_one_is_reported_once() {
    let info = start(2, 1);
    let mut s = started(&info, 0);
    let mut rec = Recorder::default();
    s.receive_via(1, &frames_msg(1, 0, vec![frame(1, 5)]), Via::Direct(1));
    let t = s.step(2, &mut rec).expect("tick 0 runs on the direct copy");
    assert_eq!(t.orders[0].order, Order::Stop { actor: ObjectId(5) });
    // The relay's copy says otherwise: the frame ran already, the
    // difference is reported, once.
    for at in [3, 4] {
        s.receive(at, &frames_msg(1, 0, vec![frame(1, 6)]));
    }
    assert_eq!(
        s.take_events(),
        vec![Event::Conflicting { slot: 1, frame: 0 }]
    );
    // Once the relay passed it on, the direct copy is forgotten.
    assert_eq!(s.paths(1).map(|p| p.conflicts), Some(1));
    // A relay copy first, then a different direct one: the relay's runs.
    s.receive(5, &frames_msg(1, 1, vec![frame(1, 7)]));
    s.receive_via(5, &frames_msg(1, 1, vec![frame(1, 8)]), Via::Direct(1));
    let t = s.step(6, &mut rec).expect("tick 1 runs");
    assert_eq!(t.orders[0].order, Order::Stop { actor: ObjectId(7) });
    assert_eq!(
        s.take_events(),
        vec![Event::Conflicting { slot: 1, frame: 1 }]
    );
}

/// The `Held` answers in `out` for `slot`, put together.
fn held_in(out: &[Vec<u8>], from: u8, slot: u8) -> Option<(u32, Vec<Frame>, bool)> {
    let mut got: Option<(u32, Vec<Frame>, bool)> = None;
    for b in out {
        let env = wire::decode(b).expect("a message");
        if let Body::Held {
            slot: s,
            first,
            frames,
            last,
        } = env.body
        {
            assert_eq!((env.from, s), (from, slot));
            match got.as_mut() {
                None => got = Some((first, frames, last)),
                Some((f0, fs, l)) => {
                    assert_eq!(*f0 + fs.len() as u32, first, "parts without a gap");
                    fs.extend(frames);
                    *l = last;
                }
            }
        }
    }
    got
}

#[test]
fn a_drop_round_is_answered_with_every_frame_run_or_held_past_the_relays() {
    let d = 2u8;
    let info = start(2, d);
    let mut s = started(&info, 0);
    let mut rec = Recorder::default();
    // Frames 0 to 6 of slot 1 come only on the link; ticks 0 to 3 run.
    let fs: Vec<Frame> = (0..7).map(|i| frame(d, 100 + i)).collect();
    s.receive_via(1, &frames_msg(1, 0, fs[..4].to_vec()), Via::Direct(1));
    for i in 0..4 {
        assert!(s.step(2 + i, &mut rec).is_some(), "tick {i}");
    }
    s.receive_via(6, &frames_msg(1, 4, fs[4..].to_vec()), Via::Direct(1));
    let _ = s.poll(10);
    // A peer cannot start a round, nor answer one.
    for forged in [
        encode(
            1,
            Body::Closing {
                slot: 1,
                forwarded: 0,
            },
        ),
        encode(
            1,
            Body::Held {
                slot: 1,
                first: 0,
                frames: vec![],
                last: true,
            },
        ),
    ] {
        s.receive(11, &forged);
        assert_eq!(bad(&mut s), 1);
    }
    // The relay forwarded none of them and asks: the answer holds the
    // four run and the three held, without a gap, through the relay only.
    s.receive(
        12,
        &encode(
            RELAY,
            Body::Closing {
                slot: 1,
                forwarded: 0,
            },
        ),
    );
    assert_eq!(s.take_events(), vec![Event::Closing { slot: 1 }]);
    assert!(!s.direct_ok(1), "the link is no longer used");
    let out = s.poll(13);
    assert_eq!(held_in(&out, 0, 1), Some((0, fs.clone(), true)));
    assert!(out.iter().all(|b| !wire::is_frames_from(b, 0) || {
        let e = wire::decode(b).expect("a message");
        matches!(e.body, Body::Frames(_))
    }));
    // Frames on the link after the ask are not taken (nor refused).
    s.receive_via(14, &frames_msg(1, 7, vec![frame(d, 107)]), Via::Direct(1));
    assert_eq!(s.take_events(), vec![]);
    // The relay sends what the peers held, a frame more from another
    // peer's answer, then the drop that counts them: nothing ran past it.
    let more: Vec<Frame> = (0..8).map(|i| frame(d, 100 + i)).collect();
    for b in held_parts(RELAY, 1, 0, more) {
        s.receive(15, &b);
    }
    s.receive(15, &encode(RELAY, Body::Dropped { slot: 1, kept: 8 }));
    assert_eq!(s.take_events(), vec![Event::Dropped { slot: 1, kept: 8 }]);
    for i in 4..12 {
        assert!(s.step(16 + i, &mut rec).is_some(), "tick {i}");
    }
    let of_1 = rec.0.iter().filter(|(h, _)| *h == HouseId(1)).count();
    assert_eq!(of_1, 8, "its frames 0 to 7 ran, and no later one");
    assert_eq!(bad(&mut s), 0);
}

#[test]
fn a_round_for_a_slot_not_live_here_is_answered_empty() {
    let info = start(3, 2);
    let mut s = started(&info, 0);
    s.receive(1, &encode(RELAY, Body::Dropped { slot: 2, kept: 0 }));
    s.take_events();
    s.receive(
        2,
        &encode(
            RELAY,
            Body::Closing {
                slot: 2,
                forwarded: 0,
            },
        ),
    );
    assert_eq!(s.take_events(), vec![], "nothing to close here");
    let out = s.poll(3);
    assert_eq!(held_in(&out, 0, 2), Some((0, vec![], true)));
}

#[test]
fn long_answers_come_in_parts() {
    let n = 2 * HELD_PART + 5;
    let frames: Vec<Frame> = (0..n as u32).map(|i| frame(1, i)).collect();
    let parts = held_parts(RELAY, 3, 40, frames.clone());
    assert_eq!(parts.len(), 3);
    assert_eq!(held_in(&parts, RELAY, 3), Some((40, frames, true)));
    let lasts: Vec<bool> = parts
        .iter()
        .map(|b| match wire::decode(b).expect("held").body {
            Body::Held { last, .. } => last,
            _ => unreachable!(),
        })
        .collect();
    assert_eq!(lasts, [false, false, true]);
    assert_eq!(held_parts(RELAY, 3, 7, vec![]).len(), 1, "an empty answer");
}

#[test]
fn only_a_peers_own_frames_messages_go_on_a_link() {
    let info = start(2, 2);
    let mut s = started(&info, 0);
    let out = s.poll(1);
    assert!(out.iter().any(|b| wire::is_frames_from(b, 0)));
    assert!(out.iter().all(|b| !wire::is_frames_from(b, 1)));
    for (b, why) in [
        (encode(0, Body::Start(info.clone())), "a start"),
        (encode(0, Body::Dropped { slot: 1, kept: 0 }), "a drop"),
        (
            encode(
                0,
                Body::Held {
                    slot: 1,
                    first: 0,
                    frames: vec![],
                    last: true,
                },
            ),
            "an answer",
        ),
        (b"GN".to_vec(), "a header only"),
        (Vec::new(), "nothing"),
    ] {
        assert!(!wire::is_frames_from(&b, 0), "{why}");
    }
    let mut other = frames_msg(0, 0, vec![]);
    other[2] = wire::WIRE_VERSION - 1;
    assert!(!wire::is_frames_from(&other, 0), "another version");
}
