//! Claim tests for S-0203 revision 50 (claim 13, Q-2636: a removed object
//! is kept to the end of its tick, and the defeat sweep reaches the house's
//! leftover vehicles) on synthetic data. Scenario O18's parts a, b and e in
//! our engine's form (claim 13 (e)).

#[path = "crate_common/mod.rs"]
mod crate_common;

use std::sync::Arc;

use crate_common::*;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::Country;
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::econ::{stats, victory};
use glasnost_sim::leftover;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const A: HouseId = HouseId(0);
const B: HouseId = HouseId(1);

/// The crate fixtures' rules with UNARMED carrying a crate and the switch
/// `quirks.defeat_sweep_leftovers` at `on`, in a two-house game that is no
/// sandbox (so the defeat test runs, S-0414 claim 1), crates on, the
/// destroyed-by-anything tags of the testkit on the map.
fn world(on: bool) -> World {
    let mut r: Ruleset = ruleset();
    r.globals.quirks.defeat_sweep_leftovers = on;
    r.sim.vehicles.get_mut("UNARMED").unwrap().carries_crate = Some(true);
    for n in ["Alphaland", "Betaland"] {
        r.sim.countries.insert(
            n.into(),
            Country {
                multiplay_passive: Some(false),
                ..Default::default()
            },
        );
    }
    let rules = Arc::new(SimRules::new(&r, "synthetic-test"));
    let mut map = testkit::empty_map(SIZE);
    map.triggers = testkit::entry_triggers();
    let mut setup = testkit::setup(7);
    setup.sandbox = false;
    let mut w = glasnost_sim::load::new_game(rules, &map, testkit::catalogue(), &setup)
        .expect("synthetic test game loads");
    w.econ.settings.crates = Some(true);
    w.econ.start.bases = true;
    w
}

fn leftover_hits(w: &World, id: ObjectId) -> Vec<(Option<u32>, i32)> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::LeftoverHit {
                id: x,
                warhead,
                health,
                ..
            } if *x == id => Some((*warhead, *health)),
            _ => None,
        })
        .collect()
}

fn c4(w: &World) -> Option<u32> {
    w.rules().combat.general.c4_warhead
}

fn lethal() -> Hit {
    Hit {
        amount: 100_000,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: false,
    }
}

fn attack(w: &mut World, actor: ObjectId, target: ObjectId) {
    let owner = w.obj(actor).and_then(|o| o.owner).unwrap();
    assert!(orders::issue(
        w,
        owner,
        Order::Behaviour {
            actor,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(target)),
            dest: None,
        },
    ));
}

/// A's gun vehicle and B's last vehicle, a crate carrier with 1 health
/// point left, two cells apart; orders open.
fn duel(on: bool) -> (World, ObjectId, ObjectId) {
    let mut w = world(on);
    let gun = spawn(&mut w, Kind::Vehicle, "ARMED", 0, at(0, 0));
    let victim = spawn(&mut w, Kind::Vehicle, "UNARMED", 1, at(2, 0));
    w.obj_mut(victim).unwrap().health = 1;
    while w.tick <= w.orders_blocked_through {
        glasnost_sim::tick::step(&mut w);
    }
    assert!(!victory::defeated(&w, B));
    (w, gun, victim)
}

// spec: S-0203 claim 13 (a), (b), (c) and Edge cases, quirks.defeat_sweep_leftovers (O18 part a): the last vehicle of a house, destroyed by a hit in the turn pass of tick t, is a leftover after t, off the map with no health; its house is defeated in t; with the switch on the sweep gives it one more hit (the demolition warhead, no health taken) and its death effects come again (a second crate); off, no hit and one crate; gone after t + 1
#[test]
fn o18_a_the_last_vehicle_destroyed_in_the_turn_pass() {
    for on in [true, false] {
        let (mut w, gun, victim) = duel(on);
        attack(&mut w, gun, victim);
        let mut ticks = 0;
        while w.obj(victim).is_some() {
            glasnost_sim::tick::step(&mut w);
            ticks += 1;
            assert!(ticks < 600, "the victim was never destroyed");
        }
        // claim 13 (b): defeated in the house update of the tick of the destruction.
        assert!(victory::defeated(&w, B), "on {on}");
        assert!(!victory::defeated(&w, A), "on {on}");
        // claim 13 (a) and (e): listed after the tick, off the map, with no health.
        let left = leftover::object(&w, victim).expect("a leftover after its tick");
        assert!(!left.on_map);
        assert_eq!(left.health, 0);
        assert_eq!(left.owner, Some(B));
        let hits = leftover_hits(&w, victim);
        if on {
            assert_eq!(
                hits,
                vec![(c4(&w), 0)],
                "one sweep hit, the demolition warhead, no health"
            );
        } else {
            assert!(hits.is_empty(), "off: no hit on the leftover");
        }
        // claim 13 (c): the death effects once more (the crate it carries).
        assert_eq!(w.crates.places.len(), if on { 2 } else { 1 }, "on {on}");
        // claim 13 (a): gone after tick t + 1, and no further hit.
        glasnost_sim::tick::step(&mut w);
        assert!(leftover::object(&w, victim).is_none());
        assert_eq!(leftover_hits(&w, victim).len(), usize::from(on));
    }
}

// spec: S-0203 claim 13 (a) and (c) (O18 part b): a vehicle destroyed in the tick before the defeat no longer exists in the defeat tick and takes no hit; its crate comes once
#[test]
fn o18_b_no_hit_on_a_vehicle_destroyed_in_the_tick_before() {
    let (mut w, _, victim) = duel(true);
    damage::apply(&mut w, victim, lethal());
    assert!(leftover::object(&w, victim).is_some());
    glasnost_sim::tick::step(&mut w);
    assert!(victory::defeated(&w, B));
    assert!(leftover::object(&w, victim).is_none());
    assert!(leftover_hits(&w, victim).is_empty());
    assert_eq!(w.crates.places.len(), 1);
}

// spec: S-0203 claim 13 (c) (O18 part e): the sweep's hit on a leftover vehicle raises no tag and registers no destruction (no kill, no loss); our engine offers it to no leftover with health left and to no leftover of another kind
#[test]
fn o18_e_the_sweep_hit_raises_no_tag_and_counts_nothing() {
    let (mut w, _, victim) = duel(true);
    let kept = spawn(&mut w, Kind::Vehicle, "UNARMED", 1, at(4, 4));
    let man = spawn(&mut w, Kind::Infantry, "WALKER", 1, at(-4, 4));
    glasnost_sim::triggers::object_tag(&mut w, victim, Some(b"DIE"));
    damage::apply(&mut w, victim, lethal());
    damage::apply(&mut w, man, lethal());
    // A removal with health left (not a death).
    glasnost_sim::objects::remove(&mut w, kept);
    assert!(w.triggers.variables[1], "the destroying hit raised the tag");
    w.triggers.variables[1] = false;
    let fired = w.triggers.fired;
    let losses = stats::of(&w, B).losses;
    let kills = stats::of(&w, A).kills;
    let crates = w.crates.places.len();
    // The sweep within the same tick as the destructions.
    victory::defeat(&mut w, B);
    assert_eq!(leftover_hits(&w, victim), vec![(c4(&w), 0)]);
    assert!(leftover_hits(&w, kept).is_empty(), "health left: no hit");
    assert!(leftover_hits(&w, man).is_empty(), "another kind: no hit");
    assert_eq!(w.triggers.fired, fired, "no tag raise");
    assert!(!w.triggers.variables[1]);
    assert_eq!(stats::of(&w, B).losses, losses, "no loss");
    assert_eq!(stats::of(&w, A).kills, kills, "no kill");
    assert_eq!(w.crates.places.len(), crates + 1, "the crate once more");
    assert_eq!(
        leftover::object(&w, victim).unwrap().health,
        0,
        "no health changed"
    );
}

// spec: S-0203 claim 13 (e) (the leftovers our engine keeps for the sweep are simulation state: they survive a save and load with the state hash)
#[test]
fn leftovers_survive_a_save_and_load() {
    let (mut w, _, victim) = duel(true);
    damage::apply(&mut w, victim, lethal());
    let h = glasnost_sim::save::state_hash(&w);
    let bytes = glasnost_sim::save::save(&w);
    let w2 = glasnost_sim::save::load(&bytes, w.rules_arc(), w.catalogue.clone().unwrap())
        .expect("the save loads");
    assert_eq!(glasnost_sim::save::state_hash(&w2), h);
    assert!(leftover::object(&w2, victim).is_some());
}
