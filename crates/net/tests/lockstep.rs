//! The lockstep session on the simulated network, with a toy game whose
//! state hash chains every order and step (tests/common): the tick rule, the
//! slot order, stalls, the adaptive delay, the start refusal, drops and the
//! wire format. Our own netcode (docs/development.md, *Multiplayer (M9)*);
//! no spec applies.

mod common;

use std::collections::BTreeMap;

use common::{Harness, Toy, bad_link, human_slots, random_orders};
use glasnost_net::simnet::{Link, SplitMix};
use glasnost_net::wire::{self, Body, Envelope, Frame, Frames};
use glasnost_net::{
    Config, Event, Field, Params, Phase, Session, Simulation, Slot, SlotKind, StartInfo,
};
use glasnost_sim::orders::Order;
use glasnost_sim::world::{HouseId, ObjectId};

const TICK_MS: u32 = 33;

fn start(slots: &[SlotKind], params: Params) -> StartInfo {
    StartInfo {
        build: "test-build".into(),
        rules_hash: "rules".into(),
        map_hash: "map-0".into(),
        setup_hash: "setup".into(),
        seed: 7,
        slots: slots
            .iter()
            .enumerate()
            .map(|(i, &kind)| Slot {
                house: i as u8,
                kind,
            })
            .collect(),
        params: params.sane(),
    }
}

fn fixed(d: u8) -> Params {
    Params {
        initial_delay: d,
        min_delay: d,
        max_delay: d,
        hash_every: 5,
        ..Params::default()
    }
}

fn adaptive() -> Params {
    Params {
        initial_delay: 2,
        min_delay: 1,
        max_delay: 20,
        hash_every: 5,
        ..Params::default()
    }
}

/// Input per slot from its own seeded stream, remembered per (slot, tick)
/// so that the expected schedule can be rebuilt.
struct Script {
    rngs: BTreeMap<u8, SplitMix>,
    given: BTreeMap<(u8, u32), Vec<Order>>,
}

impl Script {
    fn new(slots: &[u8], seed: u64) -> Self {
        Self {
            rngs: slots
                .iter()
                .map(|&s| (s, SplitMix::new(seed * 131 + u64::from(s))))
                .collect(),
            given: BTreeMap::new(),
        }
    }

    fn input(&mut self, slot: u8, tick: u32) -> Vec<Order> {
        let o = random_orders(self.rngs.get_mut(&slot).expect("slot"));
        self.given.insert((slot, tick), o.clone());
        o
    }
}

/// The single-process run: the same orders on the same ticks, in one game.
fn reference(h: &Harness<Toy>) -> Vec<glasnost_net::wire::Hash> {
    let mut g = Toy::new(0);
    let mut out = Vec::new();
    let longest = h.peers.iter().max_by_key(|p| p.ticks.len()).expect("peers");
    for t in &longest.ticks {
        for a in &t.orders {
            g.issue(a.house, a.order.clone());
        }
        g.step();
        out.push(g.state_hash());
    }
    out
}

fn check_against_reference(h: &Harness<Toy>) {
    h.assert_same_schedule();
    h.assert_no_trouble();
    let r = reference(h);
    for p in &h.peers {
        let n = p.hashes.len();
        assert_eq!(
            p.hashes[..],
            r[..n],
            "slot {} differs from the single-process run",
            p.slot
        );
    }
}

#[test]
fn two_to_eight_peers_under_bad_links_agree_every_tick() {
    // A seeded sweep: the property test of the tick rule.
    for seed in 1..=24u64 {
        let mut rng = SplitMix::new(seed);
        let humans = 2 + rng.below(7) as usize;
        let computers = rng.below(3) as usize;
        let mut kinds = vec![SlotKind::Human; humans];
        kinds.extend(vec![SlotKind::Computer; computers]);
        // Shuffle the computer slots in among the persons'.
        for i in (1..kinds.len()).rev() {
            kinds.swap(i, rng.below(i as u64 + 1) as usize);
        }
        let info = start(&kinds, adaptive());
        let mut h = Harness::new(
            &info,
            Config::new(TICK_MS),
            seed,
            bad_link(&mut rng),
            || Toy::new(0),
        );
        for &a in &human_slots(&info) {
            for &b in &human_slots(&info) {
                if a != b {
                    h.net.set_link(a, b, bad_link(&mut rng));
                }
            }
        }
        h.hash_every_tick = true;
        let mut script = Script::new(&human_slots(&info), seed);
        h.run_ticks(150, 120_000, &mut |slot, _, tick| script.input(slot, tick));
        check_against_reference(&h);
        // Every order given was applied once, by its slot's house, and none
        // came from a computer slot.
        let applied: usize = h.peers[0].ticks.iter().map(|t| t.orders.len()).sum();
        let late: usize = script
            .given
            .iter()
            .filter(|((_, t), _)| *t + u32::from(h.max_delay) + 1 >= 150)
            .map(|(_, o)| o.len())
            .sum();
        let given: usize = script.given.values().map(Vec::len).sum();
        assert!(
            applied <= given && applied + late >= given,
            "seed {seed}: {applied} of {given}"
        );
        for t in &h.peers[0].ticks {
            for a in &t.orders {
                assert_eq!(info.slots[usize::from(a.slot)].kind, SlotKind::Human);
                assert_eq!(a.house, HouseId(info.slots[usize::from(a.slot)].house));
            }
        }
        // The network was hostile: messages were lost, duplicated and
        // reordered.
        let s = h.net.stats();
        assert!(
            s.lost > 0 && s.duplicated > 0 && s.reordered > 0,
            "seed {seed}: {s:?}"
        );
    }
}

#[test]
fn a_fixed_delay_runs_every_order_that_many_ticks_later() {
    for d in [1u8, 3, 6] {
        let info = start(
            &[
                SlotKind::Human,
                SlotKind::Computer,
                SlotKind::Human,
                SlotKind::Human,
            ],
            fixed(d),
        );
        let mut rng = SplitMix::new(u64::from(d));
        let mut h = Harness::new(&info, Config::new(TICK_MS), 3, bad_link(&mut rng), || {
            Toy::new(0)
        });
        let mut script = Script::new(&human_slots(&info), 9);
        h.run_ticks(120, 120_000, &mut |slot, _, tick| script.input(slot, tick));
        h.assert_same_schedule();
        h.assert_no_trouble();
        // Not built from the run: the input given before tick t runs at
        // tick t + d, slot by slot in slot order.
        for t in &h.peers[0].ticks {
            let mut want = Vec::new();
            if let Some(from) = t.tick.checked_sub(u32::from(d)) {
                for s in human_slots(&info) {
                    for o in script.given.get(&(s, from)).into_iter().flatten() {
                        want.push((s, o.clone()));
                    }
                }
            }
            let got: Vec<(u8, Order)> =
                t.orders.iter().map(|a| (a.slot, a.order.clone())).collect();
            assert_eq!(got, want, "delay {d}, tick {}", t.tick);
        }
        assert!(h.max_lead <= u32::from(d));
    }
}

/// A recording game for the slot-order test.
#[derive(Default)]
struct Recorder(Vec<(HouseId, Order)>, u32);

impl Simulation for Recorder {
    fn issue(&mut self, house: HouseId, order: Order) -> bool {
        self.0.push((house, order));
        true
    }
    fn step(&mut self) {
        self.1 += 1;
    }
    fn state_hash(&self) -> glasnost_net::wire::Hash {
        [0; 32]
    }
}

fn frames_msg(from: u8, first: u32, frames: Vec<Frame>) -> Vec<u8> {
    wire::encode(&Envelope {
        from,
        body: Body::Frames(Frames {
            first,
            frames,
            ..Frames::default()
        }),
    })
}

fn started(info: &StartInfo, me: u8) -> Session {
    let mut s = Session::new(Config::new(TICK_MS), info.clone(), me, 0).expect("slot");
    for (i, slot) in info.slots.iter().enumerate() {
        if slot.kind == SlotKind::Human && i as u8 != me {
            s.receive(
                0,
                &wire::encode(&Envelope {
                    from: i as u8,
                    body: Body::Start(info.clone()),
                }),
            );
        }
    }
    assert_eq!(s.phase(), Phase::Running);
    s
}

#[test]
fn merged_orders_apply_in_slot_order_whatever_the_arrival_order() {
    // Slots 0, 2 and 3 are persons (3 is ours), slot 1 a computer; houses
    // in reverse so that slot order and house order differ.
    let mut info = start(
        &[
            SlotKind::Human,
            SlotKind::Computer,
            SlotKind::Human,
            SlotKind::Human,
        ],
        fixed(1),
    );
    for (i, s) in info.slots.iter_mut().enumerate() {
        s.house = 3 - i as u8;
    }
    let mut s = started(&info, 3);
    // Our own frame 0 is made empty at the start; our order goes into frame 1.
    s.submit(Order::Stop {
        actor: ObjectId(30),
    });
    let mut rec = Recorder::default();
    let f = |a: u32| Frame {
        delay: 1,
        orders: vec![
            Order::Stop { actor: ObjectId(a) },
            Order::Deploy {
                actor: ObjectId(a + 1),
            },
        ],
        resume: None,
    };
    // Tick 0: slot 2's frame arrives before slot 0's.
    s.receive(1, &frames_msg(2, 0, vec![f(20), f(22)]));
    assert!(s.step(3, &mut rec).is_none(), "slot 0's frame is missing");
    assert_eq!(s.waiting_for(), vec![0]);
    s.receive(2, &frames_msg(0, 0, vec![f(0), f(2)]));
    let t0 = s.step(3, &mut rec).expect("tick 0 runs");
    let t1 = s.step(3, &mut rec).expect("tick 1 runs");
    assert!(s.step(3, &mut rec).is_none(), "tick 2 waits");
    let order = |a: u32| Order::Stop { actor: ObjectId(a) };
    let deploy = |a: u32| Order::Deploy { actor: ObjectId(a) };
    assert_eq!(
        rec.0,
        vec![
            (HouseId(3), order(0)),
            (HouseId(3), deploy(1)),
            (HouseId(1), order(20)),
            (HouseId(1), deploy(21)),
            (HouseId(3), order(2)),
            (HouseId(3), deploy(3)),
            (HouseId(1), order(22)),
            (HouseId(1), deploy(23)),
            (HouseId(0), order(30)),
        ]
    );
    assert_eq!((t0.tick, t1.tick), (0, 1));
    assert_eq!(
        t1.orders.iter().map(|a| a.slot).collect::<Vec<_>>(),
        vec![0, 0, 2, 2, 3]
    );
}

#[test]
fn a_stalled_peer_stalls_all_and_none_runs_ahead() {
    let d = 4u8;
    let info = start(&[SlotKind::Human; 4], fixed(d));
    let mut h = Harness::new(&info, Config::new(TICK_MS), 5, Link::latency(15), || {
        Toy::new(0)
    });
    h.hash_every_tick = true;
    let mut script = Script::new(&human_slots(&info), 5);
    let mut input = |slot, _: &Toy, tick| script.input(slot, tick);
    h.run_ticks(40, 60_000, &mut input);
    // Freeze slot 2 (it neither steps nor sends): the others run at most
    // the delay ahead of it, then wait.
    h.peer(2).frozen = true;
    let at = h.peer(2).session.ran();
    h.run(3_000, &mut input);
    for p in &h.peers {
        if p.slot != 2 {
            assert_eq!(
                p.session.ran(),
                at + u32::from(d),
                "slot {} stops one delay ahead",
                p.slot
            );
            assert_eq!(p.session.waiting_for(), vec![2]);
        }
    }
    h.peer(2).frozen = false;
    h.run_ticks(100, 60_000, &mut input);
    // Stall slot 1's network instead (its frames and acks are held).
    h.net.stall(1, true, h.now);
    let at = h.peer(1).session.ran();
    h.run(3_000, &mut input);
    for p in &h.peers {
        assert!(
            p.session.ran() <= at + 2 * u32::from(d),
            "slot {}: {} after {at}",
            p.slot,
            p.session.ran()
        );
    }
    let others: Vec<u32> = h
        .peers
        .iter()
        .filter(|p| p.slot != 1)
        .map(|p| p.session.ran())
        .collect();
    assert!(
        others.iter().all(|&r| r == others[0]),
        "the others wait at the same tick: {others:?}"
    );
    let now = h.now;
    h.net.stall(1, false, now);
    h.run_ticks(160, 60_000, &mut input);
    assert!(h.max_lead <= u32::from(d), "lead {}", h.max_lead);
    check_against_reference(&h);
}

#[test]
fn the_delay_adapts_both_ways_and_switches_on_the_same_tick_everywhere() {
    let info = start(
        &[
            SlotKind::Human,
            SlotKind::Human,
            SlotKind::Computer,
            SlotKind::Human,
        ],
        adaptive(),
    );
    let mut cfg = Config::new(TICK_MS);
    cfg.calm_ticks = 30;
    let mut h = Harness::new(&info, cfg, 11, Link::latency(5), || Toy::new(0));
    h.hash_every_tick = true;
    let mut script = Script::new(&human_slots(&info), 11);
    let mut input = |slot, _: &Toy, tick| script.input(slot, tick);
    h.run_ticks(150, 60_000, &mut input);
    let low = h.peers[0].session.delay();
    // A slow link: 250 ms each way with jitter.
    h.net.set_all(Link {
        latency_ms: 250,
        jitter_ms: 40,
        ..Link::default()
    });
    h.run_ticks(450, 200_000, &mut input);
    let high = h.peers[0].session.delay();
    assert!(high >= low + 5, "the delay rose: {low} -> {high}");
    // A fast link again.
    h.net.set_all(Link::latency(5));
    h.run_ticks(1100, 200_000, &mut input);
    let back = h.peers[0].session.delay();
    assert!(back + 4 <= high, "the delay fell: {high} -> {back}");
    // The same changes on the same ticks on every peer.
    let first = h.delay_changes(0);
    assert!(
        first.iter().any(|c| c.2 > c.1) && first.iter().any(|c| c.2 < c.1),
        "{first:?}"
    );
    for p in human_slots(&info) {
        let n = h.delay_changes(p).len().min(first.len());
        assert_eq!(h.delay_changes(p)[..n], first[..n], "slot {p}");
    }
    check_against_reference(&h);
}

#[test]
fn late_frames_raise_the_delay() {
    // The same short round trip with and without loss: the waits for
    // resent frames raise the delay above what the round trip asks for.
    let run = |loss_permille| {
        let info = start(&[SlotKind::Human; 3], adaptive());
        let link = Link {
            latency_ms: 20,
            loss_permille,
            ..Link::default()
        };
        let mut h = Harness::new(&info, Config::new(TICK_MS), 13, link, || Toy::new(0));
        h.hash_every_tick = true;
        let mut script = Script::new(&human_slots(&info), 13);
        h.run_ticks(400, 200_000, &mut |slot, _, tick| script.input(slot, tick));
        check_against_reference(&h);
        h.max_delay
    };
    let clean = run(0);
    let lossy = run(400);
    assert!(lossy > clean, "lossy {lossy}, clean {clean}");
}

#[test]
fn a_mismatched_start_is_refused_with_the_field_named() {
    let base = start(&[SlotKind::Human, SlotKind::Human], fixed(2));
    type Change = fn(&mut StartInfo);
    let cases: [(Change, Field); 7] = [
        (|i| i.build = "other-build".into(), Field::Build),
        (|i| i.rules_hash = "other-rules".into(), Field::RulesHash),
        (|i| i.map_hash = "map-1".into(), Field::MapHash),
        (|i| i.seed += 1, Field::Seed),
        (|i| i.slots[1].house = 5, Field::Slots),
        (|i| i.setup_hash = "x".into(), Field::Setup),
        (|i| i.params.hash_every += 1, Field::Params),
    ];
    for (change, field) in cases {
        let mut other = base.clone();
        change(&mut other);
        let mut h = Harness::new(&base, Config::new(TICK_MS), 1, Link::latency(10), || {
            Toy::new(0)
        });
        // Slot 1 runs on the other values.
        h.peers[1].session = Session::new(Config::new(TICK_MS), other, 1, 0).expect("slot");
        h.run(500, &mut |_, _, _| Vec::new());
        assert_eq!(
            h.peers[0].session.phase(),
            Phase::Refused { slot: 1, field }
        );
        assert_eq!(
            h.peers[1].session.phase(),
            Phase::Refused { slot: 0, field }
        );
        assert!(
            h.peers[0]
                .events
                .contains(&Event::Refused { slot: 1, field })
        );
        assert_eq!(h.rans(), vec![0, 0], "nothing runs");
        assert!(!field.name().is_empty());
    }
    // Another wire version is refused before its body is read.
    let mut s = Session::new(Config::new(TICK_MS), base.clone(), 0, 0).expect("slot");
    let mut bytes = wire::encode(&Envelope {
        from: 1,
        body: Body::Start(base.clone()),
    });
    bytes[2] = wire::WIRE_VERSION + 1;
    s.receive(1, &bytes);
    assert_eq!(
        s.phase(),
        Phase::Refused {
            slot: 1,
            field: Field::Protocol
        }
    );
    // A start from a computer slot, or claiming our own slot, is refused.
    let info = start(
        &[SlotKind::Human, SlotKind::Computer, SlotKind::Human],
        fixed(2),
    );
    for from in [0u8, 1, 9] {
        let mut s = Session::new(Config::new(TICK_MS), info.clone(), 0, 0).expect("slot");
        s.receive(
            1,
            &wire::encode(&Envelope {
                from,
                body: Body::Start(info.clone()),
            }),
        );
        assert_eq!(
            s.phase(),
            Phase::Refused {
                slot: from,
                field: Field::Slot
            }
        );
    }
    assert_eq!(
        Session::new(Config::new(TICK_MS), info, 1, 0).err(),
        Some(Field::Slot)
    );
}

#[test]
fn a_dropped_peer_no_longer_holds_the_others() {
    let info = start(&[SlotKind::Human; 3], fixed(3));
    let mut h = Harness::new(&info, Config::new(TICK_MS), 21, Link::latency(10), || {
        Toy::new(0)
    });
    h.hash_every_tick = true;
    let mut script = Script::new(&human_slots(&info), 21);
    let mut input = |slot, _: &Toy, tick| script.input(slot, tick);
    h.run_ticks(60, 60_000, &mut input);
    h.net.cut(2);
    h.peer(2).gone = true;
    h.run(6_000, &mut input);
    // The silent slot is reported, then dropped (as the relay will) at the
    // frames everyone holds.
    for s in [0u8, 1] {
        assert!(
            h.peer(s)
                .events
                .iter()
                .any(|e| matches!(e, Event::Silent { slot: 2, .. }))
        );
        assert_eq!(h.peer(s).session.waiting_for(), vec![2]);
    }
    let kept = h.peers[..2]
        .iter()
        .map(|p| p.session.ran())
        .max()
        .expect("peers");
    assert_eq!(h.peers[0].session.ran(), h.peers[1].session.ran());
    // A drop report from a peer's slot is a forgery: it is reported as a bad
    // message and changes nothing (only the relay reports drops, N2).
    let now = h.now;
    for (s, forger) in [(0u8, 1u8), (1, 0), (0, 2)] {
        let forged = wire::encode(&Envelope {
            from: forger,
            body: Body::Dropped { slot: 2, kept },
        });
        h.peer(s).session.receive(now, &forged);
        let p = h.peer(s);
        assert!(p.session.take_events().iter().any(
            |e| matches!(e, Event::BadMessage { reason } if reason.contains("not from the relay"))
        ));
        assert_eq!(
            p.session.waiting_for(),
            vec![2],
            "the forged drop is ignored"
        );
    }
    // Nor does the relay speak for a slot in any other message.
    let from_relay = wire::encode(&Envelope {
        from: wire::RELAY,
        body: Body::Frames(Frames::default()),
    });
    h.peer(0).session.receive(now, &from_relay);
    assert!(
        h.peer(0)
            .session
            .take_events()
            .iter()
            .any(|e| matches!(e, Event::BadMessage { .. }))
    );
    let bytes = wire::encode(&Envelope {
        from: wire::RELAY,
        body: Body::Dropped { slot: 2, kept },
    });
    for s in [0u8, 1] {
        h.peer(s).session.receive(now, &bytes);
    }
    h.run_ticks(200, 60_000, &mut input);
    for s in [0u8, 1] {
        assert!(h.peer(s).events.contains(&Event::Dropped { slot: 2, kept }));
        assert!(
            h.peer(s)
                .ticks
                .iter()
                .all(|t| t.tick < kept || t.orders.iter().all(|a| a.slot != 2))
        );
    }
    check_against_reference(&h);
}

/// Past the grace period a dropped slot's house surrenders on every peer,
/// in the orders of the same tick (N5; the order is S-0414 claim 11's
/// surrender). No peer can ask for that, nor for a rejoin's serve or
/// readmission: those come from the relay only.
#[test]
fn a_dropped_house_surrenders_after_the_grace_period_everywhere_on_one_tick() {
    let grace = 20; // nocite: a test grace period
    let info = start(
        &[SlotKind::Human; 3],
        Params {
            grace_ticks: grace,
            ..fixed(3)
        },
    );
    let mut h = Harness::new(&info, Config::new(TICK_MS), 23, Link::latency(10), || {
        Toy::new(0)
    });
    let mut script = Script::new(&human_slots(&info), 23);
    let mut input = |slot, _: &Toy, tick| script.input(slot, tick);
    h.run_ticks(40, 60_000, &mut input);
    h.net.cut(2);
    h.peer(2).gone = true;
    h.run(3_000, &mut input);
    let kept = h.peers[..2]
        .iter()
        .map(|p| p.session.ran())
        .max()
        .expect("peers");
    let now = h.now;
    for (s, forger) in [(0u8, 1u8), (1, 0), (0, 2)] {
        for body in [Body::Serve { slot: 2, on: true }, Body::Resume { slot: 2 }] {
            let forged = wire::encode(&Envelope { from: forger, body });
            h.peer(s).session.receive(now, &forged);
            assert!(h.peer(s).session.take_events().iter().any(
                |e| matches!(e, Event::BadMessage { reason } if reason.contains("not from the relay"))
            ));
        }
        assert!(h.peer(s).session.serving().is_empty());
    }
    let bytes = wire::encode(&Envelope {
        from: wire::RELAY,
        body: Body::Dropped { slot: 2, kept },
    });
    for s in [0u8, 1] {
        h.peer(s).session.receive(now, &bytes);
        assert_eq!(h.peer(s).session.surrender_at(2), Some(kept + grace));
    }
    h.run_ticks(kept + grace + 30, 60_000, &mut input);
    let x = kept + grace;
    for s in [0u8, 1] {
        let p = h.peer(s);
        assert!(p.events.contains(&Event::Surrendered { slot: 2, tick: x }));
        let surrenders: Vec<u32> = p
            .ticks
            .iter()
            .filter(|t| {
                t.orders
                    .iter()
                    .any(|a| a.slot == 2 && a.house == HouseId(2) && a.order == Order::Surrender)
            })
            .map(|t| t.tick)
            .collect();
        assert_eq!(surrenders, [x], "slot {s}: once, on tick {x}");
        assert_eq!(p.session.surrender_at(2), None);
        assert!(p.session.life(2).is_some_and(|l| l.surrendered));
    }
    check_against_reference(&h);
}

#[test]
fn a_desync_is_reported_with_tick_slots_and_hashes() {
    let info = start(&[SlotKind::Human; 3], fixed(2));
    let mut h = Harness::new(&info, Config::new(TICK_MS), 4, Link::latency(10), || {
        Toy::new(0)
    });
    let mut input = |_, _: &Toy, _| Vec::new();
    h.run_ticks(12, 60_000, &mut input);
    // Perturb slot 1's game between checkpoints.
    h.peer(1).sim.issue(HouseId(9), Order::Cheer);
    h.run_ticks(40, 60_000, &mut input);
    for p in &h.peers {
        let d: Vec<&Event> = p
            .events
            .iter()
            .filter(|e| matches!(e, Event::Desync { .. }))
            .collect();
        assert!(!d.is_empty(), "slot {} reports it", p.slot);
        let Event::Desync { tick, mine, theirs } = d[0] else {
            unreachable!()
        };
        assert_eq!(
            *tick, 14,
            "the first checkpoint after the change (every 5 ticks)"
        );
        assert_eq!(mine.slot, p.slot);
        assert_ne!(mine.hash, theirs.hash);
        if p.slot != 1 {
            assert_eq!(theirs.slot, 1);
        }
    }
}

/// The agreed checkpoints (M9 N4) grow while the games agree and stop at
/// the first mismatched one; a desync report's digest reaches every other
/// peer once, in the frames that follow.
#[test]
fn agreed_checkpoints_stop_at_a_desync_and_a_report_reaches_the_others() {
    let info = start(&[SlotKind::Human; 3], fixed(2));
    let mut h = Harness::new(&info, Config::new(TICK_MS), 6, Link::latency(10), || {
        Toy::new(0)
    });
    let mut input = |_, _: &Toy, _| Vec::new();
    h.run_ticks(12, 60_000, &mut input);
    for p in &h.peers {
        let a = p.session.agreed();
        assert!((1..=2).contains(&a), "slot {}: {a}", p.slot);
    }
    // Slot 1's game changes before tick 12: checkpoint 2 (ticks 10 to 14)
    // differs.
    h.peer(1).sim.issue(HouseId(9), Order::Cheer);
    h.run_ticks(40, 60_000, &mut input);
    for p in &h.peers {
        assert_eq!(p.session.agreed(), 2, "slot {}", p.slot);
    }
    let digest = [7u8; 32];
    h.peer(1).session.set_report(14, digest);
    h.run_ticks(60, 60_000, &mut input);
    for p in &h.peers {
        let reports: Vec<&Event> = p
            .events
            .iter()
            .filter(|e| matches!(e, Event::Report { .. }))
            .collect();
        if p.slot == 1 {
            assert!(reports.is_empty(), "no report of its own");
        } else {
            assert_eq!(
                reports,
                [&Event::Report {
                    slot: 1,
                    tick: 14,
                    digest
                }],
                "slot {}",
                p.slot
            );
        }
    }
}

#[test]
fn the_same_seed_gives_the_same_run() {
    let run = || {
        let info = start(&[SlotKind::Human; 5], adaptive());
        let mut rng = SplitMix::new(77);
        let mut h = Harness::new(&info, Config::new(TICK_MS), 77, bad_link(&mut rng), || {
            Toy::new(0)
        });
        let mut script = Script::new(&human_slots(&info), 77);
        h.run(8_000, &mut |slot, _, tick| script.input(slot, tick));
        (
            h.rans(),
            h.peers.iter().map(|p| p.events.clone()).collect::<Vec<_>>(),
            h.net.stats(),
        )
    };
    assert_eq!(run(), run());
}

#[test]
fn hostile_bytes_never_panic() {
    let info = start(&[SlotKind::Human, SlotKind::Human], adaptive());
    let mut s = Session::new(Config::new(TICK_MS), info.clone(), 0, 0).expect("slot");
    let good = wire::encode(&Envelope {
        from: 1,
        body: Body::Frames(Frames {
            stamp: 5,
            ran: 3,
            acks: vec![1, 2],
            hash_acks: vec![0, 0],
            first: 0,
            frames: vec![Frame {
                delay: 3,
                orders: vec![Order::Cheer],
                resume: None,
            }],
            ..Frames::default()
        }),
    });
    assert!(wire::decode(&good).is_ok());
    let mut rng = SplitMix::new(1);
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
        let _ = wire::decode(&b);
        s.receive(u64::from(n), &b);
        let _ = s.poll(u64::from(n));
    }
    // Frames far beyond the window are not kept.
    let mut s = started(&info, 0);
    s.receive(1, &frames_msg(1, 1_000_000, vec![Frame::default(); 3]));
    assert_eq!(s.waiting_for(), vec![1]);
}

#[test]
fn a_lone_person_with_computers_runs_without_a_network() {
    let info = start(
        &[SlotKind::Computer, SlotKind::Human, SlotKind::Computer],
        fixed(2),
    );
    let mut s = Session::new(Config::new(TICK_MS), info, 1, 0).expect("slot");
    assert_eq!(s.phase(), Phase::Running);
    let mut g = Toy::new(0);
    s.submit(Order::Cheer);
    for _ in 0..10 {
        assert!(s.step(0, &mut g).is_some());
        assert!(s.poll(0).is_empty());
    }
    assert_eq!(g.issued, vec![(2, HouseId(1), Order::Cheer)]);
}
