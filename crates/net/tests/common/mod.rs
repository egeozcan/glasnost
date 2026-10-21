//! A test harness: several peers, each with its own game and lockstep
//! session, on the simulated network, driven by a simulated clock in steps
//! of one millisecond. Each peer runs a tick when its pacer is due and the
//! session lets it; a person's input for a tick is asked for once, before
//! the peer's first try at that tick.

#![allow(dead_code)]

use glasnost_net::simnet::{Link, SimNet, SplitMix};
use glasnost_net::wire::Hash;
use glasnost_net::{Config, Event, Phase, Session, Simulation, StartInfo, Tick, pump};
use glasnost_sim::orders::Order;
use glasnost_sim::world::{HouseId, ObjectId};
use sha2::{Digest, Sha256};

/// One peer.
pub struct Peer<S> {
    /// The slot it plays.
    pub slot: u8,
    /// Its session.
    pub session: Session,
    /// Its game.
    pub sim: S,
    /// When its next tick is due.
    pub next_due: u64,
    /// The tick its input was last asked for.
    pub input_for: Option<u32>,
    /// A frozen peer neither steps nor touches the network.
    pub frozen: bool,
    /// Cut off for good (dropped).
    pub gone: bool,
    /// The ticks it ran.
    pub ticks: Vec<Tick>,
    /// Its state hash after every tick, when asked for.
    pub hashes: Vec<Hash>,
    /// Its events.
    pub events: Vec<Event>,
}

/// The peers and their network.
pub struct Harness<S> {
    /// The peers, one per person's slot, in slot order.
    pub peers: Vec<Peer<S>>,
    /// The network.
    pub net: SimNet,
    /// The clock.
    pub now: u64,
    /// The tick period.
    pub tick_ms: u32,
    /// Record each peer's state hash after every tick.
    pub hash_every_tick: bool,
    /// The largest lead of one peer over another seen.
    pub max_lead: u32,
    /// The largest delay any peer had in effect.
    pub max_delay: u8,
}

/// The persons' slots of a start.
pub fn human_slots(info: &StartInfo) -> Vec<u8> {
    info.slots
        .iter()
        .enumerate()
        .filter(|(_, s)| s.kind == glasnost_net::SlotKind::Human)
        .map(|(i, _)| i as u8)
        .collect()
}

impl<S: Simulation> Harness<S> {
    /// Peers for every person's slot of `info`, each with the game `make`
    /// returns, on a network of `link` seeded with `seed`.
    pub fn new(
        info: &StartInfo,
        cfg: Config,
        seed: u64,
        link: Link,
        mut make: impl FnMut() -> S,
    ) -> Self {
        let slots = human_slots(info);
        let net = SimNet::new(seed, &slots, link);
        let peers = slots
            .iter()
            .map(|&slot| Peer {
                slot,
                session: Session::new(cfg, info.clone(), slot, 0).expect("a person's slot"),
                sim: make(),
                next_due: 0,
                input_for: None,
                frozen: false,
                gone: false,
                ticks: Vec::new(),
                hashes: Vec::new(),
                events: Vec::new(),
            })
            .collect();
        Self {
            peers,
            net,
            now: 0,
            tick_ms: cfg.tick_ms,
            hash_every_tick: false,
            max_lead: 0,
            max_delay: info.params.initial_delay,
        }
    }

    /// The peer of slot `slot`.
    pub fn peer(&mut self, slot: u8) -> &mut Peer<S> {
        self.peers
            .iter_mut()
            .find(|p| p.slot == slot)
            .expect("a peer of that slot")
    }

    /// Runs `ms` milliseconds; `input(slot, game, tick)` gives a person's
    /// orders for a tick.
    pub fn run(&mut self, ms: u64, input: &mut impl FnMut(u8, &S, u32) -> Vec<Order>) {
        for _ in 0..ms {
            self.now += 1;
            let now = self.now;
            for p in self.peers.iter_mut() {
                if p.frozen || p.gone {
                    continue;
                }
                pump(&mut p.session, &mut self.net.endpoint(p.slot), now);
                if now >= p.next_due && p.session.phase() == Phase::Running {
                    let tick = p.session.ran();
                    if p.input_for != Some(tick) {
                        p.input_for = Some(tick);
                        for o in input(p.slot, &p.sim, tick) {
                            p.session.submit(o);
                        }
                    }
                    if let Some(t) = p.session.step(now, &mut p.sim) {
                        p.next_due = now + u64::from(self.tick_ms);
                        self.max_delay = self.max_delay.max(t.delay);
                        if self.hash_every_tick {
                            p.hashes.push(p.sim.state_hash());
                        }
                        p.ticks.push(t);
                        // Send the new frames at once.
                        pump(&mut p.session, &mut self.net.endpoint(p.slot), now);
                    }
                }
                p.events.extend(p.session.take_events());
            }
            let live = self.peers.iter().filter(|p| !p.gone);
            let most = live.clone().map(|p| p.session.ran()).max().unwrap_or(0);
            let least = live.map(|p| p.session.ran()).min().unwrap_or(0);
            self.max_lead = self.max_lead.max(most - least);
            assert!(
                most - least <= u32::from(self.max_delay),
                "a peer runs {} ticks ahead with the largest delay {}",
                most - least,
                self.max_delay
            );
        }
    }

    /// Runs until every live peer has run `ticks` ticks (or `limit_ms`
    /// passes: a test failure).
    pub fn run_ticks(
        &mut self,
        ticks: u32,
        limit_ms: u64,
        input: &mut impl FnMut(u8, &S, u32) -> Vec<Order>,
    ) {
        let end = self.now + limit_ms;
        while self
            .peers
            .iter()
            .any(|p| !p.gone && p.session.ran() < ticks)
        {
            assert!(
                self.now < end,
                "the peers did not reach tick {ticks} in {limit_ms} ms: {:?}",
                self.rans()
            );
            self.run(1, input);
        }
    }

    /// Ticks run per peer.
    pub fn rans(&self) -> Vec<u32> {
        self.peers.iter().map(|p| p.session.ran()).collect()
    }

    /// Every live peer applied the same orders on the same ticks, with the
    /// same delays, as far as the shortest run goes.
    pub fn assert_same_schedule(&self) {
        let live: Vec<&Peer<S>> = self.peers.iter().filter(|p| !p.gone).collect();
        let n = live.iter().map(|p| p.ticks.len()).min().unwrap_or(0);
        for p in &live[1..] {
            assert_eq!(
                p.ticks[..n],
                live[0].ticks[..n],
                "slot {} applied another schedule than slot {}",
                p.slot,
                live[0].slot
            );
        }
    }

    /// No peer reported a desync, a refusal or a bad message.
    pub fn assert_no_trouble(&self) {
        for p in &self.peers {
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
    }

    /// The delay history of a peer: (tick, delay from the next tick on).
    pub fn delay_changes(&self, slot: u8) -> Vec<(u32, u8, u8)> {
        let p = self.peers.iter().find(|p| p.slot == slot).expect("peer");
        p.events
            .iter()
            .filter_map(|e| match *e {
                Event::DelayChanged { tick, from, to } => Some((tick, from, to)),
                _ => None,
            })
            .collect()
    }
}

/// A toy game: its state is a hash chain of every order issued (with its
/// house) and every step, so any difference in what was applied, or in its
/// order, shows in the hash.
#[derive(Clone, Debug)]
pub struct Toy {
    /// The chain.
    pub state: Hash,
    /// Ticks run.
    pub ticks: u32,
    /// The orders issued, with the tick.
    pub issued: Vec<(u32, HouseId, Order)>,
}

impl Toy {
    /// A toy game from a seed.
    pub fn new(seed: u32) -> Self {
        Self {
            state: Sha256::digest(seed.to_le_bytes()).into(),
            ticks: 0,
            issued: Vec::new(),
        }
    }
}

impl Simulation for Toy {
    fn issue(&mut self, house: HouseId, order: Order) -> bool {
        let mut h = Sha256::new();
        h.update(self.state);
        h.update([house.0]);
        h.update(postcard::to_allocvec(&order).expect("serializes"));
        self.state = h.finalize().into();
        self.issued.push((self.ticks, house, order));
        true
    }

    fn step(&mut self) {
        let mut h = Sha256::new();
        h.update(self.state);
        h.update(b"step");
        self.state = h.finalize().into();
        self.ticks += 1;
    }

    fn state_hash(&self) -> Hash {
        self.state
    }
}

/// Seeded random orders: about one tick in three a person gives one to three
/// orders.
pub fn random_orders(rng: &mut SplitMix) -> Vec<Order> {
    if rng.below(3) != 0 {
        return Vec::new();
    }
    (0..1 + rng.below(3))
        .map(|_| match rng.below(3) {
            0 => Order::Cheer,
            1 => Order::Stop {
                actor: ObjectId(rng.below(1000) as u32),
            },
            _ => Order::Deploy {
                actor: ObjectId(rng.below(1000) as u32),
            },
        })
        .collect()
}

/// A bad link: latency, jitter, loss, duplicates and reordering.
pub fn bad_link(rng: &mut SplitMix) -> Link {
    Link {
        latency_ms: 10 + rng.below(120) as u32,
        jitter_ms: rng.below(60) as u32,
        loss_permille: rng.below(150) as u32,
        dup_permille: rng.below(100) as u32,
        reorder_permille: rng.below(100) as u32,
    }
}
