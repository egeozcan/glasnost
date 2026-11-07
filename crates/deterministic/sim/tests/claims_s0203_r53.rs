//! Claim tests for S-0203 revision 53 (claim 13 (b), (c), (f), (g) and (h);
//! Q-2691 to Q-2693): the defeat sweep reaches every leftover vehicle of
//! the house, whatever removed it and with or without health left. Scenario
//! O18's parts f to j in our engine's form (claim 13 (e)), on synthetic
//! data; part h's place rule with the cancellation skip is in
//! `claims_s0414.rs`.

#[path = "crate_common/mod.rs"]
mod crate_common;

use std::sync::Arc;

use crate_common::*;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    Country, LocomotionClass, Locomotor, MovementZone, UnitType, WarheadType, WeaponType,
};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::econ::{stats, victory};
use glasnost_sim::leftover;
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, World};

const A: HouseId = HouseId(0);
const B: HouseId = HouseId(1);

fn add_vehicle(r: &mut Ruleset, n: &str, u: UnitType) {
    let l = r.registries.entry("vehicles".into()).or_default();
    if !l.iter().any(|x| x == n) {
        l.push(n.into());
    }
    r.sim.vehicles.insert(n.into(), u);
}

/// The crate fixtures' rules with the switch `quirks.defeat_sweep_leftovers`
/// at `on`, UNARMED carrying a crate, the death weapon detonating at the
/// object's position, and the test types:
/// - BOOMER, O18 part a's type: exploding, the death weapon Blast (which
///   destroys a test vehicle next to it), debris, an explosion and a
///   destruction animation;
/// - BOAT, part h's vessel: BOOMER on the ship locomotor, naval, heavier
///   than the sinking weight; CBOAT, the same of a type that can crash;
/// - CRASHER, part i's type: BOOMER of a type that can crash;
/// - TUMBLER: UNARMED of a type that can crash;
/// - CARRIER: an exploding transport for two.
fn rules(on: bool) -> Ruleset {
    let mut r: Ruleset = ruleset();
    r.globals.quirks.defeat_sweep_leftovers = on;
    r.globals.quirks.death_weapon_unplaced_projectile = false;
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
    r.sim.warheads.insert(
        "BLAST".into(),
        WarheadType {
            verses: Some(vec![d("1"); 11]),
            cell_spread: Some(d("2")),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "Blast".into(),
        WeaponType {
            damage: Some(1000),
            range: Some(d("1")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("BLAST".into()),
            ..Default::default()
        },
    );
    let mut plain = r.sim.vehicles["UNARMED"].clone();
    plain.carries_crate = Some(false);
    let mut boom = plain.clone();
    boom.combat.explodes = Some(true);
    boom.combat.death_weapon = Some("Blast".into());
    boom.combat.min_debris = Some(1);
    boom.combat.max_debris = Some(3);
    boom.combat.debris_anims = Some(vec!["DEBRIS".into()]);
    boom.combat.explosion = Some(vec!["EXPLO".into()]);
    boom.combat.destroy_anim = Some(vec!["WRECK".into()]);
    let mut boat = boom.clone();
    boat.locomotion_class = Some(LocomotionClass::Float);
    boat.locomotor = Some(Locomotor::Ship);
    boat.movement_zone = Some(MovementZone::Water);
    boat.weight = Some(d("4"));
    boat.combat.naval = Some(true);
    let mut cboat = boat.clone();
    cboat.crashable = Some(true);
    let mut crasher = boom.clone();
    crasher.crashable = Some(true);
    let mut tumbler = r.sim.vehicles["UNARMED"].clone();
    tumbler.crashable = Some(true);
    let mut carrier = plain.clone();
    carrier.combat.explodes = Some(true);
    carrier.passengers = Some(2);
    carrier.size_limit = Some(d("10"));
    for (n, u) in [
        ("BOOMER", boom),
        ("BOAT", boat),
        ("CBOAT", cboat),
        ("CRASHER", crasher),
        ("TUMBLER", tumbler),
        ("CARRIER", carrier),
    ] {
        add_vehicle(&mut r, n, u);
    }
    r.sim.general.ship_sinking_weight = Some(d("3"));
    r
}

/// A two-house game that is no sandbox (so the defeat test runs, S-0414
/// claim 1), crates on, orders open; A owns a gun vehicle far away.
fn world(on: bool) -> World {
    world_with(on, |_| ()).0
}

/// [`world`], with `f` placing more objects before the first tick (B, with
/// no object, is defeated in the first house update).
fn world_with<T>(on: bool, f: impl FnOnce(&mut World) -> T) -> (World, T) {
    let rules = Arc::new(SimRules::new(&rules(on), "synthetic-test"));
    let map = testkit::empty_map(SIZE);
    let mut setup = testkit::setup(7);
    setup.sandbox = false;
    let mut w = glasnost_sim::load::new_game(rules, &map, testkit::catalogue(), &setup)
        .expect("synthetic test game loads");
    w.econ.settings.crates = Some(true);
    w.econ.start.bases = true;
    spawn(&mut w, Kind::Vehicle, "ARMED", 0, at(-14, -14));
    let t = f(&mut w);
    while w.tick <= w.orders_blocked_through {
        glasnost_sim::tick::step(&mut w);
    }
    w.effects.clear();
    (w, t)
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

fn c4(w: &World) -> Option<u32> {
    w.rules().combat.general.c4_warhead
}

fn leftover_hits(w: &World, id: ObjectId) -> Vec<(Option<u32>, (i32, i32))> {
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

fn anims(w: &World, name: &str) -> usize {
    w.effects
        .iter()
        .filter(|e| matches!(e, Effect::Anim { name: n, .. } if n == name))
        .count()
}

fn detonations(w: &World) -> usize {
    w.effects
        .iter()
        .filter(|e| matches!(e, Effect::Detonation { .. }))
        .count()
}

fn first_index(w: &World, f: impl Fn(&Effect) -> bool) -> Option<usize> {
    w.effects.iter().position(f)
}

fn last_index(w: &World, f: impl Fn(&Effect) -> bool) -> Option<usize> {
    w.effects.iter().rposition(f)
}

// spec: S-0203 claim 13 (f) (Q-2691, O18 part f and part j): within one sweep, an object at an earlier place whose death weapon destroys a vehicle of the house at a later place leaves a leftover that the sweep reaches at its place, so the crate it carries comes twice; with the later vehicle first in creation order the death weapon finds it gone and its crate comes once; quirks.defeat_sweep_leftovers off: once in both orders
#[test]
fn o18_f_a_death_weapon_destroys_a_later_place() {
    for on in [true, false] {
        for boomer_first in [true, false] {
            let mut w = world(on);
            let (v1, v2) = if boomer_first {
                let v1 = spawn(&mut w, Kind::Vehicle, "BOOMER", 1, at(0, 0));
                (v1, spawn(&mut w, Kind::Vehicle, "UNARMED", 1, at(1, 0)))
            } else {
                let v2 = spawn(&mut w, Kind::Vehicle, "UNARMED", 1, at(1, 0));
                (spawn(&mut w, Kind::Vehicle, "BOOMER", 1, at(0, 0)), v2)
            };
            let crates = w.crates.places.len();
            victory::defeat(&mut w, B);
            assert!(w.obj(v1).is_none() && w.obj(v2).is_none());
            let twice = on && boomer_first;
            assert_eq!(
                leftover_hits(&w, v2),
                if twice {
                    vec![(c4(&w), (0, 0))]
                } else {
                    vec![]
                },
                "on {on}, boomer first {boomer_first}"
            );
            assert!(
                leftover_hits(&w, v1).is_empty(),
                "the sweep's own place is behind it"
            );
            assert_eq!(
                w.crates.places.len(),
                crates + if twice { 2 } else { 1 },
                "on {on}, boomer first {boomer_first}"
            );
        }
    }
}

// spec: S-0203 claim 13 (f) (Q-2691, O18 part g and part j): a surrender's sweep destroys the house's two crate carriers, the defeat test of the same house update fails and the defeat sweeps it again, reaching both leftovers: both destroying hits before either hit of no amount, four crates in all; off, no second hit and two crates
#[test]
fn o18_g_two_sweeps_in_a_surrender() {
    for on in [true, false] {
        let (mut w, (v1, v2)) = world_with(on, |w| {
            (
                spawn(w, Kind::Vehicle, "UNARMED", 1, at(6, 0)),
                spawn(w, Kind::Vehicle, "UNARMED", 1, at(-6, 8)),
            )
        });
        assert!(!victory::defeated(&w, B), "B's vehicles count for it");
        let crates = w.crates.places.len();
        victory::surrender(&mut w, B);
        victory::house_update(&mut w, B);
        assert!(victory::defeated(&w, B));
        let killed =
            |id: ObjectId| move |e: &Effect| matches!(e, Effect::Killed { id: x, .. } if *x == id);
        let k1 = first_index(&w, killed(v1)).expect("the surrender's hit destroys V1");
        let k2 = first_index(&w, killed(v2)).expect("the surrender's hit destroys V2");
        let left = |e: &Effect| matches!(e, Effect::LeftoverHit { .. });
        if on {
            assert_eq!(leftover_hits(&w, v1), vec![(c4(&w), (0, 0))]);
            assert_eq!(leftover_hits(&w, v2), vec![(c4(&w), (0, 0))]);
            let first = first_index(&w, left).unwrap();
            assert!(k1 < first && k2 < first, "both destroying hits first");
            assert_eq!(w.crates.places.len(), crates + 4);
        } else {
            assert!(last_index(&w, left).is_none());
            assert_eq!(w.crates.places.len(), crates + 2);
        }
    }
}

// spec: S-0203 claim 13 (f) (Q-2691: a vehicle of the house removed earlier in the same tick is reached by both sweeps of a surrender's house update, so its death effects come three times in all; the spec leaves it `unmeasured`, following from O18 parts a and g together) and (a) (it stays a leftover to the end of its tick); quirks.defeat_sweep_leftovers off: once, at its own death (seam sweep 6, intake 107's row)
#[test]
fn o18_a_and_g_a_vehicle_removed_earlier_in_the_tick_has_three_sets_of_effects() {
    for on in [true, false] {
        let (mut w, (v0, v1, v2)) = world_with(on, |w| {
            (
                spawn(w, Kind::Vehicle, "UNARMED", 1, at(0, 0)),
                spawn(w, Kind::Vehicle, "UNARMED", 1, at(6, 0)),
                spawn(w, Kind::Vehicle, "UNARMED", 1, at(-6, 8)),
            )
        });
        let crates = w.crates.places.len();
        let tick = w.tick;
        // V0 is destroyed by an ordinary hit earlier in the tick; V1 and V2
        // still count for B.
        damage::apply(&mut w, v0, lethal());
        assert!(w.obj(v0).is_none(), "set-up: V0 has left play");
        assert!(leftover::object(&w, v0).is_some(), "set-up: a leftover");
        assert_eq!(w.crates.places.len(), crates + 1, "set-up: V0's own crate");
        assert!(!victory::defeated(&w, B), "B's other vehicles count for it");
        victory::surrender(&mut w, B);
        victory::house_update(&mut w, B);
        assert_eq!(w.tick, tick, "all in one tick");
        assert!(victory::defeated(&w, B));
        if on {
            assert_eq!(
                leftover_hits(&w, v0),
                vec![(c4(&w), (0, 0)), (c4(&w), (0, 0))],
                "both sweeps reach V0"
            );
            assert_eq!(leftover_hits(&w, v1), vec![(c4(&w), (0, 0))]);
            assert_eq!(leftover_hits(&w, v2), vec![(c4(&w), (0, 0))]);
            // V0: its death and two sweeps; V1 and V2: their destroying hit
            // and the second sweep. Each death asks for a crate near its
            // cell; spec: S-0520 claim 3 (the cell the nearby search selects
            // must hold no overlay), so V0's third, near two crates already
            // there, may be refused.
            let n = w.crates.places.len() - crates;
            assert!(
                (6..=7).contains(&n),
                "crates {n}: {:?}",
                w.crates.places.values().map(|p| p.cell).collect::<Vec<_>>()
            );
        } else {
            assert!(leftover_hits(&w, v0).is_empty());
            assert_eq!(w.crates.places.len(), crates + 1 + 1 + 1);
        }
    }
}

/// D's vessel W of `name` on open water, destroyed with a lethal hit; with
/// `descend`, its sinking descent is run until it leaves play.
fn sunk(w: &mut World, name: &str, descend: bool) -> ObjectId {
    let id = spawn(w, Kind::Vehicle, name, 1, at(0, 0));
    // The water after the placing, whose land recompute reads the synthetic
    // map's tiles, which are clear ground.
    make_water(w, at(0, 0), 3);
    damage::apply(w, id, lethal());
    if descend {
        assert!(glasnost_sim::naval::sinking(w, id), "W sinks");
        assert_eq!(w.obj(id).unwrap().health, 1);
        let mut n = 0;
        while w.obj(id).is_some() {
            glasnost_sim::naval::sinking_turn(w, id);
            n += 1;
            assert!(n < 400, "the descent never ended");
        }
    }
    // Again after the removal, whose land recompute reads the tiles too.
    make_water(w, at(0, 0), 3);
    id
}

// spec: S-0203 claim 13 (b) and (g) (Q-2692, O18 part h): a vessel whose descent ends leaves play with the 1 health point its sinking start restored; the sweep gives it a killing hit of 1 health point with the demolition warhead and no attacker, its destruction is registered once more (a loss, no kill), its death weapon and debris come again and claim 10's branch shows no explosion or destruction animation and restores 1 health point; it is marked sinking and, of a type that cannot crash, kept off the map with its place to the end of the game; S-0312 claim 10
#[test]
fn o18_h_a_vessel_whose_descent_ends() {
    let mut w = world(true);
    let ship = sunk(&mut w, "BOAT", true);
    let left = leftover::object(&w, ship).expect("a leftover");
    assert_eq!(left.health, 1, "claim 13 (b): the 1 health point kept");
    let losses = stats::of(&w, B).losses;
    let kills = stats::of(&w, A).kills;
    w.effects.clear();
    victory::defeat(&mut w, B);
    assert_eq!(leftover_hits(&w, ship), vec![(c4(&w), (1, 1))]);
    assert_eq!(detonations(&w), 1, "the death weapon once more");
    assert!(anims(&w, "DEBRIS") >= 1, "the debris once more");
    assert_eq!(
        anims(&w, "EXPLO") + anims(&w, "WRECK"),
        0,
        "claim 10's branch"
    );
    assert_eq!(stats::of(&w, B).losses, losses + 1, "a loss");
    assert_eq!(stats::of(&w, A).kills, kills, "no kill");
    let left = leftover::object(&w, ship).unwrap();
    assert_eq!(left.health, 1);
    assert!(
        left.naval.as_ref().is_some_and(|n| n.sinking),
        "marked sinking"
    );
    for _ in 0..3 {
        w.effects.clear();
        glasnost_sim::tick::step(&mut w);
        let left = leftover::object(&w, ship).expect("kept to the end of the game");
        assert!(!left.on_map);
        assert_eq!(left.health, 1);
        assert_eq!(
            leftover::next(&w, ObjectId(0)),
            Some(ship),
            "its place kept"
        );
        assert!(leftover_hits(&w, ship).is_empty());
    }
}

// spec: S-0203 claim 13 (g) (Q-2692): a crashable vessel restored to 1 health point that left play takes the sweep's killing hit through claim 10's branch (no animation, 1 health point), fails its crash start, is removed again and ends with its tick; S-0312 claims 10 and 15
#[test]
fn o18_h_a_crashable_vessel_ends_with_its_tick() {
    let mut w = world(true);
    let ship = sunk(&mut w, "CBOAT", false);
    assert!(
        w.obj(ship).is_none(),
        "removed at the end of its death effects"
    );
    assert_eq!(leftover::object(&w, ship).unwrap().health, 1);
    w.effects.clear();
    victory::defeat(&mut w, B);
    assert_eq!(leftover_hits(&w, ship), vec![(c4(&w), (1, 1))]);
    assert_eq!(anims(&w, "EXPLO") + anims(&w, "WRECK"), 0);
    glasnost_sim::tick::step(&mut w);
    assert!(leftover::object(&w, ship).is_none(), "ends with its tick");
}

// spec: S-0203 claim 13 (g) (Q-2692, O18 part i): a vehicle of a type that can crash that left play on the ground with no health takes a hit of no amount at the sweep, which shows claim 9's animations again and starts no crash; it ends with its tick
#[test]
fn o18_i_a_vehicle_whose_crash_ended() {
    let mut w = world(true);
    let j = spawn(&mut w, Kind::Vehicle, "CRASHER", 1, at(0, 0));
    damage::apply(&mut w, j, lethal());
    assert!(w.obj(j).is_none());
    assert_eq!(leftover::object(&w, j).unwrap().health, 0);
    w.effects.clear();
    victory::defeat(&mut w, B);
    assert_eq!(leftover_hits(&w, j), vec![(c4(&w), (0, 0))]);
    assert_eq!(detonations(&w), 1);
    assert_eq!((anims(&w, "EXPLO"), anims(&w, "WRECK")), (1, 1));
    glasnost_sim::tick::step(&mut w);
    assert!(leftover::object(&w, j).is_none());
}

// spec: S-0203 claim 13 (g), Crashing (Q-2692): a leftover of a type that can crash above the terrain at the sweep's killing hit starts a crash after its other death effects: the three tumbling draws of S-0312 claim 16 (unit interval, ranged 0 to 1, unit interval) and no fall, so no removal and no crate; on the ground the same leftover leaves its crate; both end with their tick
#[test]
fn a_leftover_above_the_terrain_starts_a_crash() {
    for above in [true, false] {
        let mut w = world(true);
        let id = spawn(&mut w, Kind::Vehicle, "TUMBLER", 1, at(0, 0));
        if above {
            w.obj_mut(id).unwrap().pos.z += 300;
        }
        let health = w.obj(id).unwrap().health;
        glasnost_sim::objects::remove(&mut w, id);
        let crates = w.crates.places.len();
        let mut rng = w.rng.clone();
        victory::defeat(&mut w, B);
        assert_eq!(leftover_hits(&w, id), vec![(c4(&w), (health, 0))]);
        if above {
            let mode = w.rules().mode;
            let half = w.rules().quirks.ranged_draw_half_span;
            let _ = rng.unit(mode);
            let _ = rng.ranged(0, 1, half);
            let _ = rng.unit(mode);
            assert_eq!(w.rng, rng, "the three tumbling draws, nothing else");
            assert_eq!(w.crates.places.len(), crates, "no removal, no crate");
        } else {
            assert_eq!(w.crates.places.len(), crates + 1);
        }
        glasnost_sim::tick::step(&mut w);
        assert!(leftover::object(&w, id).is_none(), "ends with its tick");
    }
}

// spec: S-0203 claim 13 (b) and (g) (a vehicle passenger killed with its transport keeps its health, so the sweep's hit on it is a killing hit: a loss, its crate; the transport's own leftover with no health takes a hit of no amount); S-0312 claim 5
#[test]
fn a_passenger_killed_with_its_transport() {
    let mut w = world(true);
    let t = spawn(&mut w, Kind::Vehicle, "CARRIER", 1, at(0, 0));
    let p = spawn(&mut w, Kind::Vehicle, "UNARMED", 1, at(1, 0));
    glasnost_sim::transport::board(&mut w, p, t);
    let health = w.obj(p).unwrap().health;
    assert!(health > 0);
    damage::apply(&mut w, t, lethal());
    assert!(w.obj(p).is_none(), "died with its transport");
    assert_eq!(leftover::object(&w, p).unwrap().health, health);
    let losses = stats::of(&w, B).losses;
    let crates = w.crates.places.len();
    w.effects.clear();
    victory::defeat(&mut w, B);
    assert_eq!(leftover_hits(&w, t), vec![(c4(&w), (0, 0))]);
    assert_eq!(leftover_hits(&w, p), vec![(c4(&w), (health, 0))]);
    assert_eq!(stats::of(&w, B).losses, losses + 1);
    assert_eq!(w.crates.places.len(), crates + 1, "the passenger's crate");
}

// spec: S-0203 claim 13 (g) and Edge cases (O18 part j: quirks.defeat_sweep_leftovers off: no leftover takes the hit, so none is set sinking again; the vessel ends with its tick)
#[test]
fn o18_j_switch_off_keeps_no_vessel() {
    let mut w = world(false);
    let ship = sunk(&mut w, "BOAT", true);
    let losses = stats::of(&w, B).losses;
    w.effects.clear();
    victory::defeat(&mut w, B);
    assert!(leftover_hits(&w, ship).is_empty());
    assert_eq!(detonations(&w), 0);
    assert_eq!(stats::of(&w, B).losses, losses);
    glasnost_sim::tick::step(&mut w);
    assert!(leftover::object(&w, ship).is_none());
}

// spec: S-0203 claim 13 (e) and (g) (a vessel kept to the end of the game is simulation state: it survives a save and load with the state hash)
#[test]
fn a_kept_vessel_survives_a_save_and_load() {
    let mut w = world(true);
    let ship = sunk(&mut w, "BOAT", true);
    victory::defeat(&mut w, B);
    glasnost_sim::tick::step(&mut w);
    let h = glasnost_sim::save::state_hash(&w);
    let bytes = glasnost_sim::save::save(&w);
    let w2 = glasnost_sim::save::load(&bytes, w.rules_arc(), w.catalogue.clone().unwrap())
        .expect("the save loads");
    assert_eq!(glasnost_sim::save::state_hash(&w2), h);
    assert!(leftover::object(&w2, ship).is_some());
}
