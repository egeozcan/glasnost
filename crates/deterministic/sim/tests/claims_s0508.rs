//! Claim tests of S-0508 (garrisons) on synthetic rules: invented types and
//! numbers, no game data. Entry, ownership of civilian garrison buildings,
//! eviction, assault, erasure, defeat and the occupants' firing.

mod structures_common;

use glasnost_sim::combat::effects::{Effect, Sound};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::garrison::{self, Evict};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};
use structures_common::*;

fn civ_hut(w: &mut World, name: &str, dx: i32, dy: i32) -> ObjectId {
    put(w, Kind::Building, name, 2, dx, dy)
}

/// Puts a unit of `house` inside `b` by the entry attempt (claims 1 to 5).
fn enter(w: &mut World, b: ObjectId, name: &str, house: u8) -> ObjectId {
    let u = put(w, Kind::Infantry, name, house, -6, -6);
    assert!(garrison::attempt(w, u, b), "{name} went inside");
    u
}

fn sounds(w: &World, which: Sound) -> Vec<(Option<glasnost_sim::world::HouseId>, bool)> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Sound {
                sound, house, pos, ..
            } if *sound == which => Some((*house, pos.is_some())),
            _ => None,
        })
        .collect()
}

// spec: S-0508 claim 1 (own house or passive house, type flags, capacity: arrival refusals) and claim 2 (an occupier in capture whose destination is the building tries to enter on arrival)
#[test]
fn claim_1_and_2_an_ordered_occupier_goes_inside() {
    let mut w = game();
    let b = civ_hut(&mut w, "HUT", 0, 0);
    let u = put(&mut w, Kind::Infantry, "GARRI", 0, -6, 0);
    send(&mut w, u, b);
    let t = until(&mut w, 600, |w| !on_map(w, u)).expect("arrives");
    assert!(t > 4);
    assert_eq!(occ(&w, b), vec![u]);
    assert!(
        w.obj(u).is_some(),
        "the occupant is not destroyed or removed"
    );
    assert_eq!(w.obj(u).unwrap().structure.inside, Some(b));
}

// spec: S-0508 claim 1 (the building belongs to the unit's own house or to a passive house: an enemy's building is refused at arrival; alliance plays no part)
#[test]
fn claim_1_enemy_and_allied_buildings_refuse_the_arrival() {
    for allied in [false, true] {
        let mut w = game();
        if allied {
            w.houses[0].allies = vec![A, B];
            w.houses[1].allies = vec![A, B];
        }
        let b = put(&mut w, Kind::Building, "MIL", 1, 0, 0);
        let u = put(&mut w, Kind::Infantry, "GARRI", 0, -6, 0);
        send(&mut w, u, b);
        until(&mut w, 600, |w| {
            w.obj(u).unwrap().dest.is_none() && w.tick > 20
        })
        .expect("refused");
        assert!(on_map(&w, u), "allied {allied}");
        assert!(occ(&w, b).is_empty());
    }
}

// spec: S-0508 claim 1 (a type that is not an occupier type is refused on arrival, and a building whose type is not garrisonable refuses)
#[test]
fn claim_1_type_flags() {
    let mut w = game();
    let h = civ_hut(&mut w, "HUT", 0, 0);
    let plain = civ_hut(&mut w, "PLAIN", 4, 0);
    let walker = put(&mut w, Kind::Infantry, "WALKER", 0, -6, 0);
    let g = put(&mut w, Kind::Infantry, "GARRI", 0, -6, 3);
    assert!(!garrison::may_enter(&w, walker, h));
    assert!(!garrison::may_enter(&w, g, plain));
    assert!(garrison::may_enter(&w, g, h));
}

// spec: S-0508 claim 1 (the occupant count must not equal the capacity: equality, so capacity 0 admits none and a negative capacity admits indefinitely; quirks.garrison_capacity_equality off: below the capacity)
#[test]
fn claim_1_capacity_equality_and_its_quirk() {
    for quirk in [true, false] {
        let mut r = ruleset();
        r.sim.general.condition_red = Some(d("0.5"));
        let mut w = game_on(&r, Default::default());
        let mut q = w.rules().quirks;
        q.garrison_capacity_equality = quirk;
        let mut rules = (*w.rules_arc()).clone();
        rules.quirks = q;
        w.rules = Some(std::sync::Arc::new(rules));
        let zero = civ_hut(&mut w, "ZERO", 0, 0);
        let neg = civ_hut(&mut w, "NEG", 4, 0);
        let g = put(&mut w, Kind::Infantry, "GARRI", 0, -6, 0);
        assert!(!garrison::may_enter(&w, g, zero), "capacity 0 admits none");
        assert_eq!(garrison::may_enter(&w, g, neg), quirk, "quirk {quirk}");
        if quirk {
            for _ in 0..3 {
                enter(&mut w, neg, "GARRI", 0);
            }
            assert_eq!(garrison::count(&w, neg), 3, "above 1: not stopped");
        }
    }
}

// spec: S-0508 claim 1 (at the capacity the last unit is refused on arrival; it stays in play)
#[test]
fn claim_1_full_building_refuses() {
    let mut w = game();
    let b = civ_hut(&mut w, "HUT", 0, 0);
    for _ in 0..3 {
        enter(&mut w, b, "GARRI", 0);
    }
    let late = put(&mut w, Kind::Infantry, "GARRI", 0, -6, 0);
    assert!(!garrison::may_enter(&w, late, b));
    assert!(!garrison::attempt(&mut w, late, b));
    assert!(on_map(&w, late));
    assert_eq!(garrison::count(&w, b), 3);
}

// spec: S-0508 claim 1 (not both positive health and badly damaged; a building being sold is refused)
#[test]
fn claim_1_damage_and_sale_gates() {
    let mut w = game();
    let b = civ_hut(&mut w, "HUT", 0, 0);
    let g = put(&mut w, Kind::Infantry, "GARRI", 0, -6, 0);
    assert!(garrison::may_enter(&w, g, b));
    w.obj_mut(b).unwrap().health = 200; // exactly the red ratio of 0.5
    assert!(!garrison::may_enter(&w, g, b), "at the red ratio");
    w.obj_mut(b).unwrap().health = 201;
    assert!(garrison::may_enter(&w, g, b), "just above");
    let t = w.tick;
    w.obj_mut(b).unwrap().beh.force(Behaviour::Selling, t, true);
    assert!(!garrison::may_enter(&w, g, b), "being sold");
}

// spec: S-0508 claim 2 (only the capture behaviour with the building as destination counts: a target, area guard or patrol does not)
#[test]
fn claim_2_other_behaviours_do_not_enter() {
    for beh in [Behaviour::AreaGuard, Behaviour::Patrol] {
        let mut w = game();
        let b = civ_hut(&mut w, "HUT", 0, 0);
        let u = put(&mut w, Kind::Infantry, "GARRI", 0, -6, 0);
        let t = w.tick;
        {
            let o = w.obj_mut(u).unwrap();
            o.dest = Some(Target::Object(b));
            o.beh.force(beh, t, true);
        }
        until(&mut w, 400, |w| {
            w.obj(u).unwrap().pos.cell() == at(0, 0) || w.tick > 300
        });
        assert!(on_map(&w, u), "{beh:?} does not garrison");
        assert!(occ(&w, b).is_empty());
    }
}

fn counted() -> World {
    let t = triggers();
    game_on(&ruleset(), t)
}

// spec: S-0508 claim 3 (a passing attempt gives the building's tag exactly one entry raise with the entrant as causing object; a refused attempt raises nothing)
#[test]
fn claim_3_one_entry_raise_on_success_none_on_refusal() {
    let mut w = counted();
    let b = civ_hut(&mut w, "HUT", 0, 0);
    tag(&mut w, b, "ENT");
    let before = w.triggers.fired;
    let g = put(&mut w, Kind::Infantry, "GARRI", 0, -6, 0);
    assert!(garrison::attempt(&mut w, g, b));
    assert_eq!(w.triggers.fired - before, 1, "one raise");
    assert!(w.triggers.variables[0], "the tag's action ran");
    // A refused attempt (an enemy's building) raises nothing.
    let enemy = put(&mut w, Kind::Building, "HUT", 1, 6, 0);
    tag(&mut w, enemy, "ENT");
    let f = w.triggers.fired;
    let g2 = put(&mut w, Kind::Infantry, "GARRI", 0, -6, 2);
    assert!(!garrison::attempt(&mut w, g2, enemy));
    assert_eq!(w.triggers.fired, f);
}

// spec: S-0508 claim 4 (a refused attempt: the destination is cleared, a forced move-anywhere scatter request with the building's centre as threat point, the unit stays on the map)
#[test]
fn claim_4_refusal_clears_the_destination_and_scatters() {
    let mut w = game();
    let b = put(&mut w, Kind::Building, "HUT", 1, 0, 0);
    let g = put(&mut w, Kind::Infantry, "GARRI", 0, 0, 2);
    let t = w.tick;
    {
        let o = w.obj_mut(g).unwrap();
        o.dest = Some(Target::Object(b));
        o.beh.force(Behaviour::Capture, t, true);
    }
    assert!(!garrison::attempt(&mut w, g, b));
    let o = w.obj(g).unwrap();
    assert!(o.on_map);
    assert_ne!(o.dest, Some(Target::Object(b)), "destination cleared");
    assert!(
        matches!(o.dest, Some(Target::Cell(_))),
        "the scatter request gave it a cell to go to"
    );
}

// spec: S-0508 claim 5 (going inside: the unit leaves the map at once and is added at the end of the list; it keeps its owner, health and experience; the building's owner does not change at this moment)
#[test]
fn claim_5_going_inside() {
    let mut w = game();
    let b = civ_hut(&mut w, "HUT", 0, 0);
    let first = put(&mut w, Kind::Infantry, "GARRI", 0, -6, 0);
    w.obj_mut(first).unwrap().health = 77;
    w.obj_mut(first).unwrap().combat.expected_health = 77;
    assert!(garrison::attempt(&mut w, first, b));
    let second = put(&mut w, Kind::Infantry, "GARRI", 1, -6, 2);
    assert!(garrison::attempt(&mut w, second, b) || !garrison::may_enter(&w, second, b));
    let o = w.obj(first).unwrap();
    assert!(!o.on_map);
    assert_eq!(o.health, 77);
    assert_eq!(o.owner, Some(A));
    assert_eq!(owner(&w, b), Some(CIV), "no change at the entry");
    assert_eq!(occ(&w, b)[0], first);
    assert!(!w.update.contains(first), "an occupant is out of play");
}

// spec: S-0508 claim 6 (the garrisoned sound and announcement when an entry makes the count 1, for the entering unit's owner, at the unit's position; later entries are silent)
#[test]
fn claim_6_announcement_on_the_first_entry_only() {
    let mut w = game();
    let b = civ_hut(&mut w, "HUT", 0, 0);
    w.effects.clear();
    enter(&mut w, b, "GARRI", 0);
    assert_eq!(sounds(&w, Sound::Garrisoned), vec![(Some(A), true)]);
    w.effects.clear();
    enter(&mut w, b, "GARRI", 0);
    assert!(sounds(&w, Sound::Garrisoned).is_empty());
}

// spec: S-0508 claim 7 (a civilian garrison building that belongs to the civilian house changes owner to the first occupant's house at its own turn, not at the entry; a building of another tech level never does)
#[test]
fn claim_7_owner_follows_the_first_occupant_at_the_building_turn() {
    let mut w = game();
    let civ = civ_hut(&mut w, "HUT", 0, 0);
    let mil = civ_hut(&mut w, "MIL", 6, 0);
    enter(&mut w, civ, "GARRI", 0);
    enter(&mut w, mil, "GARRI", 0);
    assert_eq!(owner(&w, civ), Some(CIV));
    testkit::run(&mut w, 1);
    assert_eq!(owner(&w, civ), Some(A));
    assert_eq!(owner(&w, mil), Some(CIV), "another tech level: never");
    // A mixed garrison: the first occupant's house owns it, the other's occupants stay.
    let mut w = game();
    let b = civ_hut(&mut w, "HUT", 0, 0);
    let a1 = enter(&mut w, b, "GARRI", 0);
    let b1 = enter(&mut w, b, "GARRI", 1);
    testkit::run(&mut w, 2);
    assert_eq!(owner(&w, b), Some(A));
    assert_eq!(occ(&w, b), vec![a1, b1]);
    assert_eq!(
        owner(&w, b1),
        Some(B),
        "claim 17: occupants keep their house"
    );
}

// spec: S-0508 claim 8 (an empty civilian garrison building that does not belong to the civilian house returns to it at its turn; the owner losing it, when local, gets the abandoned feedback; a building of another tech level does not)
#[test]
fn claim_8_abandoned_buildings_return() {
    let mut w = game();
    let b = civ_hut(&mut w, "HUT", 0, 0);
    let mil = civ_hut(&mut w, "MIL", 6, 0);
    for x in [b, mil] {
        let old = owner(&w, x);
        w.obj_mut(x).unwrap().owner = Some(A);
        glasnost_sim::econ::events::owner_changed(&mut w, x, old);
    }
    w.effects.clear();
    testkit::run(&mut w, 1);
    assert_eq!(owner(&w, b), Some(CIV));
    assert_eq!(owner(&w, mil), Some(A), "not a civilian garrison building");
    let ab = sounds(&w, Sound::Abandoned);
    assert_eq!(ab, vec![(Some(A), true)], "feedback for the owner it lost");
}

// spec: S-0508 claim 8 and Q-0971 (with no civilian house the owner is left unchanged and nothing is announced)
#[test]
fn claim_8_no_civilian_house_leaves_the_owner() {
    let mut w = testkit::game_with(&ruleset(), SIZE, 7);
    let b = testkit::spawn(&mut w, Kind::Building, "HUT", 0, at(0, 0), 0);
    assert!(garrison::civilian_house(&w).is_none());
    w.effects.clear();
    testkit::run(&mut w, 3);
    assert_eq!(owner(&w, b), Some(A));
    assert!(sounds(&w, Sound::Abandoned).is_empty());
}

// spec: S-0508 claim 9 (a badly damaged civilian garrison building evicts its occupants at its turn without hunting; emptied, it returns to the civilian house in that same turn, with feedback for the owner it had; a building of another tech level is never evacuated)
#[test]
fn claim_9_forced_evacuation() {
    let mut w = game();
    let civ = civ_hut(&mut w, "HUT", 0, 0);
    let mil = civ_hut(&mut w, "MIL", 8, 0);
    let c1 = enter(&mut w, civ, "GARRI", 0);
    let m1 = enter(&mut w, mil, "GARRI", 0);
    testkit::run(&mut w, 2);
    assert_eq!(owner(&w, civ), Some(A));
    w.obj_mut(civ).unwrap().health = 150;
    w.obj_mut(mil).unwrap().health = 150;
    w.effects.clear();
    testkit::run(&mut w, 1);
    assert!(on_map(&w, c1), "put out");
    assert!(!on_map(&w, m1), "another tech level: stays inside");
    assert_eq!(owner(&w, civ), Some(CIV), "back in the same turn");
    assert_eq!(sounds(&w, Sound::Abandoned), vec![(Some(A), true)]);
    assert!(
        !w.obj(c1)
            .unwrap()
            .beh
            .queued
            .is_some_and(|b| b == Behaviour::Hunt),
        "no hunting"
    );
}

/// A game with a 2 × 2 hut of the civilian house at (0, 0), `n` occupiers of
/// Alpha inside, the ownership settled.
fn garrisoned(n: usize) -> (World, ObjectId, Vec<ObjectId>) {
    let mut w = game();
    let b = civ_hut(&mut w, "HUT", 0, 0);
    let us: Vec<ObjectId> = (0..n).map(|_| enter(&mut w, b, "GARRI", 0)).collect();
    testkit::run(&mut w, 2);
    (w, b, us)
}

/// Blocks the cell `c` with a building of Beta's.
fn block(w: &mut World, c: glasnost_map::CellXY) {
    testkit::spawn(w, Kind::Building, "PLAIN", 1, c, 0);
}

// spec: S-0508 claim 10 (the exit cell is the qualifying ring cell of the highest rank; the ranking of the four sides; the far corner has the highest priority)
#[test]
fn claim_10_exit_ranking() {
    let (w0, b0, _) = garrisoned(1);
    let top_left = w0.obj(b0).unwrap().pos.cell();
    let ring = garrison::exit_ring(top_left, (2, 2), true);
    assert_eq!(ring.len(), 2 * (2 + 2) + 3, "all but one corner");
    let (x0, y0) = (top_left.x, top_left.y);
    assert_eq!(
        ring[0],
        glasnost_map::CellXY::new(x0 + 2, y0 + 2),
        "far corner first"
    );
    assert_eq!(ring[1], glasnost_map::CellXY::new(x0 + 2, y0 + 1));
    assert!(!ring.contains(&glasnost_map::CellXY::new(x0 - 1, y0 - 1)));
    for k in 0..ring.len() {
        let (mut w, b, us) = garrisoned(1);
        for &c in &ring[..k] {
            block(&mut w, c);
        }
        assert_eq!(garrison::exit_cell(&w, b), Some(ring[k]), "rank {k}");
        assert!(garrison::evict(&mut w, b, Evict::PLAIN));
        let p = w.obj(us[0]).unwrap().pos;
        assert_eq!(p.cell(), ring[k], "placed in the exit cell");
    }
}

// spec: S-0508 claim 10 and Edge cases (the corner (x₀ − 1, y₀ − 1) is never tried; quirks.garrison_exit_skips_corner off: tried last)
#[test]
fn claim_10_the_corner_is_skipped_or_last() {
    let top_left = at(0, 0);
    let on = garrison::exit_ring(top_left, (2, 2), true);
    let off = garrison::exit_ring(top_left, (2, 2), false);
    let corner = glasnost_map::CellXY::new(top_left.x - 1, top_left.y - 1);
    assert!(!on.contains(&corner));
    assert_eq!(off.last(), Some(&corner));
    assert_eq!(&off[..off.len() - 1], &on[..]);
    // Every other cell blocked, the corner free: lost with the switch on, exits there with it off.
    for skips in [true, false] {
        let (mut w, b, us) = garrisoned(2);
        let mut q = w.rules().quirks;
        q.garrison_exit_skips_corner = skips;
        let mut rules = (*w.rules_arc()).clone();
        rules.quirks = q;
        w.rules = Some(std::sync::Arc::new(rules));
        for &c in &on {
            block(&mut w, c);
        }
        let done = garrison::evict(&mut w, b, Evict::PLAIN);
        assert!(done);
        if skips {
            assert!(us.iter().all(|&u| w.obj(u).is_none()), "lost");
        } else {
            assert!(us.iter().all(|&u| w.obj(u).unwrap().pos.cell() == corner));
        }
    }
}

// spec: S-0508 claim 11 (all occupants are put on the map in that one cell, from the last to the first, at its centre, facing 0; each has its target cleared and a scatter request before the next is placed; a hunting eviction gives hunt)
#[test]
fn claim_11_putting_occupants_out() {
    for hunting in [false, true] {
        let (mut w, b, us) = garrisoned(3);
        let opts = if hunting { Evict::HUNT } else { Evict::PLAIN };
        assert!(garrison::evict(&mut w, b, opts));
        let c = garrison::exit_ring(at(0, 0), (2, 2), true)[0];
        for &u in &us {
            let o = w.obj(u).unwrap();
            assert!(o.on_map);
            assert_eq!((o.pos.x, o.pos.y), (256 * c.x + 128, 256 * c.y + 128));
            assert_eq!(o.body.value(w.tick), 0, "facing 0");
            assert!(o.target.is_none());
            assert!(matches!(o.dest, Some(Target::Cell(_))), "scatter requested");
            assert_eq!(o.structure.inside, None);
            assert_eq!(
                o.beh.queued == Some(Behaviour::Hunt) || o.beh.current == Some(Behaviour::Hunt),
                hunting
            );
        }
        let bo = w.obj(b).unwrap();
        assert!(bo.structure.occupants.is_empty());
        assert_eq!(bo.structure.index, 0);
    }
}

// spec: S-0508 claim 12 (no ring cell qualifies: with the inside fallback its cell is the exit; without it every occupant is removed, not destroyed; quirks.garrison_no_exit_removes off: the occupants stay inside)
#[test]
fn claim_12_no_exit() {
    let ring = garrison::exit_ring(at(0, 0), (2, 2), false);
    // Inside fallback.
    {
        let (mut w, b, us) = garrisoned(2);
        for &c in &ring {
            block(&mut w, c);
        }
        assert!(garrison::evict(&mut w, b, Evict::SALE));
        let inside = garrison::inside_cell(at(0, 0), (2, 2));
        assert_eq!(
            inside,
            glasnost_map::CellXY::new(at(0, 0).x + 1, at(0, 0).y + 1)
        );
        assert!(us.iter().all(|&u| w.obj(u).unwrap().pos.cell() == inside));
    }
    // Removal.
    {
        let (mut w, b, us) = garrisoned(2);
        for &c in &ring {
            block(&mut w, c);
        }
        let losses = w.houses[0].combat.losses;
        assert!(garrison::evict(&mut w, b, Evict::PLAIN));
        assert!(us.iter().all(|&u| w.obj(u).is_none()), "removed");
        assert_eq!(w.houses[0].combat.losses, losses, "not destroyed: no loss");
        assert_eq!(garrison::count(&w, b), 0);
    }
    // The quirk off.
    {
        let (mut w, b, us) = garrisoned(2);
        let mut rules = (*w.rules_arc()).clone();
        rules.quirks.garrison_no_exit_removes = false;
        w.rules = Some(std::sync::Arc::new(rules));
        for &c in &ring {
            block(&mut w, c);
        }
        assert!(!garrison::evict(&mut w, b, Evict::PLAIN));
        assert_eq!(occ(&w, b), us, "still inside");
        assert!(us.iter().all(|&u| !on_map(&w, u)));
    }
}

// spec: S-0508 claim 13 (a deploy order evicts the occupants: no hunting, no inside fallback)
#[test]
fn claim_13_deploy_order() {
    let (mut w, b, us) = garrisoned(2);
    let house = owner(&w, b).unwrap();
    orders::issue(&mut w, house, Order::Deploy { actor: b });
    until(&mut w, 400, |w| on_map(w, us[0])).expect("evicted");
    assert!(us.iter().all(|&u| on_map(&w, u)));
    assert_eq!(garrison::count(&w, b), 0);
    assert!(
        !w.obj(us[0])
            .unwrap()
            .beh
            .current
            .is_some_and(|b| b == Behaviour::Hunt),
        "no hunting"
    );
}

// spec: S-0508 claim 13 and S-0312 claim 17 (a destroyed building evicts its occupants; with no qualifying exit they are removed, not killed)
#[test]
fn claim_13_destruction() {
    let (mut w, b, us) = garrisoned(2);
    let losses = w.houses[0].combat.losses;
    let hit = glasnost_sim::combat::damage::Hit {
        amount: 10_000,
        warhead: w.rules().combat.general.c4_warhead,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: false,
    };
    glasnost_sim::combat::damage::apply(&mut w, b, hit);
    assert!(us.iter().all(|&u| on_map(&w, u)), "put out");
    assert_eq!(
        w.houses[0].combat.losses,
        losses + 1,
        "only the building was lost: the occupants did not die"
    );
}

// spec: S-0508 claim 13 and S-0408 claim 9 (a sold building puts its occupants out with the inside fallback, without hunting)
#[test]
fn claim_13_sale() {
    let (mut w, b, us) = garrisoned(2);
    let house = owner(&w, b).unwrap();
    orders::issue(&mut w, house, Order::Sell { actor: b });
    until(&mut w, 400, |w| on_map(w, us[0])).expect("put out by the sale");
    assert!(us.iter().all(|&u| on_map(&w, u)));
}

// spec: S-0508 claim 14 (an assaulter type clears an occupied building that does not count its house as allied; one entry raise; the assaulter survives, its destination cleared, and scatters; the building returns to the civilian house at its next turn)
#[test]
fn claim_14_assault() {
    let t = triggers();
    let mut w = game_on(&ruleset(), t);
    let b = civ_hut(&mut w, "HUT", 0, 0);
    let o1 = enter(&mut w, b, "GARRI", 1);
    let o2 = enter(&mut w, b, "GARRI", 1);
    testkit::run(&mut w, 2);
    assert_eq!(owner(&w, b), Some(B));
    tag(&mut w, b, "ENT");
    let a = put(&mut w, Kind::Infantry, "ASSAULT", 0, -6, 0);
    let f = w.triggers.fired;
    assert!(
        !garrison::attempt(&mut w, a, b),
        "the assaulter stays outside"
    );
    assert_eq!(w.triggers.fired - f, 1, "one entry raise");
    assert!(on_map(&w, a));
    assert!(
        w.obj(o1).is_none() && w.obj(o2).is_none(),
        "occupants removed"
    );
    assert_eq!(garrison::count(&w, b), 0);
    assert_ne!(w.obj(a).unwrap().dest, Some(Target::Object(b)));
    testkit::run(&mut w, 2);
    assert_eq!(owner(&w, b), Some(CIV));
}

// spec: S-0508 claim 14 (an allied owner's building, or an empty one, is not assaulted; a type with both marks is an occupier only and never assaults)
#[test]
fn claim_14_eligibility() {
    let mut w = game();
    let b = civ_hut(&mut w, "HUT", 0, 0);
    let a = put(&mut w, Kind::Infantry, "ASSAULT", 0, -6, 0);
    assert!(!garrison::assault_eligible(&w, a, b), "empty");
    enter(&mut w, b, "GARRI", 1);
    assert!(garrison::assault_eligible(&w, a, b));
    w.obj_mut(b).unwrap().owner = Some(B);
    assert!(garrison::assault_eligible(&w, a, b), "a hostile owner");
    w.houses[0].allies = vec![A, B];
    assert!(
        garrison::assault_eligible(&w, a, b),
        "A allies B but B does not ally A: a one-way test from the owner's side"
    );
    w.houses[1].allies = vec![A, B];
    assert!(
        !garrison::assault_eligible(&w, a, b),
        "an owner that counts A as allied is refused"
    );
    w.obj_mut(b).unwrap().owner = Some(A);
    assert!(
        !garrison::assault_eligible(&w, a, b),
        "own-house alliance included"
    );
    w.obj_mut(b).unwrap().owner = Some(B);
    // A type with both marks uses the occupation test alone: it occupies an own-house building.
    let both = put(&mut w, Kind::Infantry, "BOTH", 1, -6, 3);
    assert!(garrison::occupier(&w, both) && garrison::assaulter(&w, both));
    assert!(garrison::attempt(&mut w, both, b));
    assert!(!on_map(&w, both), "it went inside");
}

// spec: S-0508 claim 14 and Edge cases (reversed kill credit: the surviving assaulter is recorded as destroyed by each occupant; quirks.assaulter_reversed_kill_credit off: each occupant is the victim)
#[test]
fn claim_14_reversed_kill_credit() {
    for reversed in [true, false] {
        let mut w = game();
        let mut rules = (*w.rules_arc()).clone();
        rules.quirks.assaulter_reversed_kill_credit = reversed;
        w.rules = Some(std::sync::Arc::new(rules));
        let b = civ_hut(&mut w, "HUT", 0, 0);
        enter(&mut w, b, "GARRI", 1);
        enter(&mut w, b, "GARRI", 1);
        testkit::run(&mut w, 2);
        let a = put(&mut w, Kind::Infantry, "ASSAULT", 0, -6, 0);
        let (la, lb) = (w.houses[0].combat.losses, w.houses[1].combat.losses);
        let (ka, kb) = (w.houses[0].combat.kills, w.houses[1].combat.kills);
        garrison::attempt(&mut w, a, b);
        assert!(on_map(&w, a), "the assaulter survives");
        assert_eq!(w.obj(a).unwrap().health, 100, "no damage");
        if reversed {
            // Two occupants of Beta destroyed the assaulter of Alpha, twice over.
            assert_eq!(w.houses[0].combat.losses - la, 2);
            assert_eq!(w.houses[1].combat.kills - kb, 2);
            assert_eq!(w.houses[1].combat.losses, lb);
        } else {
            assert_eq!(w.houses[1].combat.losses - lb, 2);
            assert_eq!(w.houses[0].combat.kills - ka, 2);
            assert_eq!(w.houses[0].combat.losses, la);
        }
    }
}

// spec: S-0508 claim 15 (a completed erasure removes the occupants with the building, without death and without putting them out)
#[test]
fn claim_15_erasure() {
    let (mut w, b, us) = garrisoned(2);
    let losses = w.houses[0].combat.losses;
    garrison::erased(&mut w, b);
    assert!(us.iter().all(|&u| w.obj(u).is_none()));
    assert_eq!(w.houses[0].combat.losses, losses);
    assert_eq!(garrison::count(&w, b), 0);
}

// spec: S-0508 claim 16 (weapon hits do not damage, remove or evict individual occupants: a nonlethal hit above the red ratio leaves the count, the occupants' health and the index alone)
#[test]
fn claim_16_hits_do_not_pass_through() {
    let (mut w, b, us) = garrisoned(3);
    w.obj_mut(b).unwrap().structure.index = 1;
    let health: Vec<i32> = us.iter().map(|&u| w.obj(u).unwrap().health).collect();
    let hit = glasnost_sim::combat::damage::Hit {
        amount: 60,
        warhead: w.rules().combat.general.c4_warhead,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: false,
    };
    glasnost_sim::combat::damage::apply(&mut w, b, hit);
    assert!(w.obj(b).unwrap().health < 400 && w.obj(b).unwrap().health > 200);
    assert_eq!(garrison::count(&w, b), 3);
    assert_eq!(
        us.iter()
            .map(|&u| w.obj(u).unwrap().health)
            .collect::<Vec<_>>(),
        health
    );
    assert_eq!(w.obj(b).unwrap().structure.index, 1);
}

// spec: S-0508 claim 17 (a building ownership change leaves the occupants and their houses alone)
#[test]
fn claim_17_ownership_change_keeps_the_occupants() {
    let (mut w, b, us) = garrisoned(2);
    assert_eq!(owner(&w, b), Some(A));
    glasnost_sim::structures::ownership::change(
        &mut w,
        b,
        Some(B),
        glasnost_sim::structures::ownership::Cause::Trigger,
    );
    assert_eq!(owner(&w, b), Some(B));
    assert_eq!(occ(&w, b), us);
    assert!(
        us.iter()
            .all(|&u| owner(&w, u) == Some(A) && !on_map(&w, u))
    );
}

// spec: S-0508 claim 18 (after a completed eviction the shared index is 0, reoccupation starts at 0, an additional entry keeps the index)
#[test]
fn claim_18_index_reset_and_additional_entry() {
    let (mut w, b, us) = garrisoned(2);
    w.obj_mut(b).unwrap().structure.index = 1;
    let third = enter(&mut w, b, "GARRI", 0);
    assert_eq!(
        w.obj(b).unwrap().structure.index,
        1,
        "an additional entry keeps it"
    );
    assert_eq!(garrison::current_occupant(&w, b), Some(us[1]));
    assert!(garrison::evict(&mut w, b, Evict::PLAIN));
    assert_eq!(w.obj(b).unwrap().structure.index, 0);
    assert!(on_map(&w, third));
    // Reoccupation: the first occupant fires first.
    assert!(garrison::attempt(&mut w, us[1], b));
    assert!(garrison::attempt(&mut w, us[0], b));
    assert_eq!(garrison::current_occupant(&w, b), Some(us[1]));
}

// spec: S-0508 claim 19 (the defeat of an occupant's own house kills its occupants one by one inside a building of a surviving house: no eviction, the numeric index and the list are kept)
#[test]
fn claim_19_defeat_hits_occupants_in_a_foreign_building() {
    let (mut w, b, us) = garrisoned(3);
    w.obj_mut(b).unwrap().structure.index = 1;
    glasnost_sim::structures::ownership::change(
        &mut w,
        b,
        Some(B),
        glasnost_sim::structures::ownership::Cause::Trigger,
    );
    let hp = w.obj(b).unwrap().health;
    glasnost_sim::econ::victory::defeat(&mut w, A);
    assert!(us.iter().all(|&u| w.obj(u).is_none()), "killed inside");
    assert_eq!(
        w.obj(b).unwrap().health,
        hp,
        "the foreign building is not hit"
    );
    assert_eq!(
        w.obj(b).unwrap().structure.index,
        1,
        "numeric index retained"
    );
    assert_eq!(occ(&w, b), us, "places kept (Q-0970)");
    assert_eq!(garrison::count(&w, b), 3);
    assert!(garrison::living(&w, b).is_empty());
    assert_eq!(garrison::current_occupant(&w, b), None);
}
