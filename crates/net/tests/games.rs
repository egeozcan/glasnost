//! Real games over the simulated network: every peer runs its own
//! `glasnost_ai::game::Game` on the synthetic skirmish (invented rules and
//! map, `glasnost_ai::fixture`), some persons' slots played by scripted
//! orders, some by a brain of our AI on the person's own peer (its orders
//! cross the network like a person's), and the computer slots by our AI
//! inside the tick on every peer. Under bad links every peer's state hash
//! after every tick equals the others' and a single-process run of the same
//! orders. Our own netcode; no spec applies.

mod common;

use std::collections::BTreeMap;

use common::{Harness, bad_link, human_slots};
use glasnost_ai::brain::Brain;
use glasnost_ai::fixture;
use glasnost_ai::game::Game;
use glasnost_map::Map;
use glasnost_net::simnet::SplitMix;
use glasnost_net::{Config, Params, Simulation, SlotKind, StartInfo};
use glasnost_sim::econ::setup::{self, Player, Settings};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{GameSetup, HouseId, Target};

const TICK_MS: u32 = 33;

/// Who plays a seat.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Seat {
    /// A person, by scripted orders.
    Scripted,
    /// A person, by a brain of our AI on the person's peer.
    Brained,
    /// A computer player.
    Computer,
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

fn setup_of(seats: &[Seat], seed: u32) -> GameSetup {
    let players: Vec<Player> = seats
        .iter()
        .enumerate()
        .map(|(i, &s)| Player {
            name: format!("P{i}"),
            country: if i % 2 == 0 { "Alphaland" } else { "Betaland" }.into(),
            start: Some(i as u32),
            ai: s == Seat::Computer,
            values: Default::default(),
        })
        .collect();
    setup::skirmish(&fixture::rules(), &players, Settings::default(), 0, seed)
}

fn game(map: &Map, setup: &GameSetup) -> Game {
    Game::new(
        fixture::rules(),
        map,
        "synthetic-8",
        fixture::catalogue(),
        setup.clone(),
    )
    .expect("the synthetic skirmish loads")
}

/// A person's scripted orders: deploy the first vehicle early, then now and
/// then send one of the house's units somewhere.
fn scripted(g: &Game, house: HouseId, tick: u32, rng: &mut SplitMix) -> Vec<Order> {
    let units: Vec<_> = g
        .world
        .objects
        .iter()
        .filter(|(_, o)| {
            o.owner == Some(house)
                && o.on_map
                && o.health > 0
                && matches!(o.kind, Kind::Vehicle | Kind::Infantry)
        })
        .map(|(&id, o)| (id, o.kind))
        .collect();
    let mut out = Vec::new();
    if tick == 3
        && let Some(&(id, _)) = units.iter().find(|(_, k)| *k == Kind::Vehicle)
    {
        out.push(Order::Deploy { actor: id });
    }
    if tick.is_multiple_of(17) && !units.is_empty() {
        let (id, _) = units[rng.below(units.len() as u64) as usize];
        let dx = rng.below(41) as i32 - 20;
        let dy = rng.below(41) as i32 - 20;
        out.push(Order::Behaviour {
            actor: id,
            behaviour: Behaviour::Move,
            attack_move: rng.below(2) == 0,
            target: None,
            dest: Some(Target::Cell(fixture::at(dx, dy))),
        });
    }
    out
}

/// Plays `seats` over a bad network for `ticks` ticks and checks every
/// peer, tick by tick, against the others and a single-process run.
fn play(seats: &[Seat], seed: u64, ticks: u32, params: Params) -> Harness<Game> {
    let map = map8();
    let setup = setup_of(seats, seed as u32);
    let rules = fixture::rules();
    let info = StartInfo::new("test-build", &rules.hash, "synthetic-8", &setup, params);
    for (slot, seat) in info.slots.iter().zip(seats) {
        assert_eq!(slot.kind == SlotKind::Computer, *seat == Seat::Computer);
    }
    let mut rng = SplitMix::new(seed);
    let mut h = Harness::new(
        &info,
        Config::new(TICK_MS),
        seed,
        bad_link(&mut rng),
        || game(&map, &setup),
    );
    let humans = human_slots(&info);
    for &a in &humans {
        for &b in &humans {
            if a != b {
                h.net.set_link(a, b, bad_link(&mut rng));
            }
        }
    }
    h.hash_every_tick = true;
    let start = game(&map, &setup);
    let mut brains: BTreeMap<u8, Brain> = BTreeMap::new();
    let mut rngs: BTreeMap<u8, SplitMix> = BTreeMap::new();
    for &s in &humans {
        let house = HouseId(info.slots[usize::from(s)].house);
        match seats[usize::from(s)] {
            Seat::Brained => {
                brains.insert(s, Brain::new(&start.world, &map, house));
            }
            _ => {
                rngs.insert(s, SplitMix::new(seed * 7 + u64::from(s)));
            }
        }
    }
    let slots = info.slots.clone();
    h.run_ticks(ticks, 600_000, &mut |slot, g: &Game, tick| {
        let house = HouseId(slots[usize::from(slot)].house);
        if let Some(b) = brains.get_mut(&slot) {
            b.think(&g.world)
        } else {
            scripted(
                g,
                house,
                tick,
                rngs.get_mut(&slot).expect("a scripted slot"),
            )
        }
    });
    h.assert_same_schedule();
    h.assert_no_trouble();
    // The single-process run: one game, the same orders on the same ticks.
    let mut one = game(&map, &setup);
    let n = h.peers.iter().map(|p| p.hashes.len()).max().unwrap_or(0);
    let longest = h
        .peers
        .iter()
        .find(|p| p.hashes.len() == n)
        .expect("a peer");
    let mut reference = Vec::new();
    for t in &longest.ticks {
        for a in &t.orders {
            one.issue(a.house, a.order.clone());
        }
        Simulation::step(&mut one);
        reference.push(Simulation::state_hash(&one));
    }
    for p in &h.peers {
        assert_eq!(
            p.hashes[..],
            reference[..p.hashes.len()],
            "slot {} differs from the single-process run",
            p.slot
        );
        assert_eq!(
            p.sim.replay().orders,
            one.replay().orders[..p.sim.replay().orders.len()]
        );
    }
    // The persons' orders crossed the network and were taken by the games.
    let taken = longest
        .ticks
        .iter()
        .flat_map(|t| &t.orders)
        .filter(|a| a.taken)
        .count();
    assert!(taken > 0, "some orders were taken");
    // Computer slots gave no frame orders: each game plays their houses.
    for t in &longest.ticks {
        for a in &t.orders {
            assert_ne!(seats[usize::from(a.slot)], Seat::Computer);
        }
    }
    h
}

fn params() -> Params {
    Params {
        initial_delay: 2,
        min_delay: 1,
        max_delay: 20,
        hash_every: 10,
        ..Params::default()
    }
}

#[test]
fn two_peers_with_a_computer_agree_with_one_game() {
    let h = play(
        &[Seat::Brained, Seat::Computer, Seat::Scripted],
        1,
        240,
        params(),
    );
    assert_eq!(h.peers.len(), 2);
}

#[test]
fn four_peers_agree_with_one_game() {
    play(
        &[
            Seat::Scripted,
            Seat::Brained,
            Seat::Computer,
            Seat::Scripted,
            Seat::Brained,
        ],
        2,
        200,
        params(),
    );
}

#[test]
fn eight_peers_agree_with_one_game() {
    play(
        &[
            Seat::Brained,
            Seat::Scripted,
            Seat::Scripted,
            Seat::Brained,
            Seat::Scripted,
            Seat::Brained,
            Seat::Scripted,
            Seat::Scripted,
        ],
        3,
        120,
        params(),
    );
}

#[test]
fn seeded_sweep_of_small_games() {
    for seed in 4..=7u64 {
        let mut rng = SplitMix::new(seed);
        let n = 2 + rng.below(3) as usize;
        let mut seats: Vec<Seat> = (0..n)
            .map(|_| match rng.below(3) {
                0 => Seat::Scripted,
                1 => Seat::Brained,
                _ => Seat::Computer,
            })
            .collect();
        if seats.iter().filter(|s| **s != Seat::Computer).count() < 2 {
            seats.push(Seat::Scripted);
            seats.push(Seat::Brained);
        }
        play(&seats, seed, 90, params());
    }
}
