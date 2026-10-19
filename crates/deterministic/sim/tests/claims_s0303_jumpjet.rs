//! Claim tests for jumpjets in flight in combat: a jumpjet between take-off
//! and touchdown or crash end is a tracked flying object that the
//! above-terrain detonations of S-0303 claim 6 hit (S-0502 claim 29), a low
//! one can be found twice (claim 31), its in-the-air answer is a height test
//! (S-0502 claim 30; S-0308 table P row 19), and a range search for aircraft
//! finds it among the airborne objects (S-0309 claim 9). Synthetic data only.

mod jump_common;

use glasnost_rules::sim::{ProjectileType, WeaponType};
use glasnost_sim::combat::acquire::{self, SearchContext, SearchKind};
use glasnost_sim::combat::area::{self, Detonation};
use glasnost_sim::combat::{flying, weapon};
use glasnost_sim::jump::{self, Phase};
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, ObjectId, World};
use jump_common::*;

fn he(w: &World) -> u32 {
    w.rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == "HE")
        .expect("the synthetic HE warhead") as u32
}

/// A detonation of the synthetic HE warhead at the unit's position.
fn blast_at(w: &mut World, u: ObjectId) {
    let point = pos(w, u);
    let warhead = Some(he(w));
    area::detonate(
        w,
        Detonation {
            point,
            damage: 50,
            warhead,
            source: None,
            house: Some(HouseId(1)),
            resources: false,
        },
    );
}

// spec: S-0303 Terms and claim 6 (a jumpjet between lift-off and landing is a tracked flying object; a detonation above the terrain considers it); S-0502 claim 4 (above 208 leptons it is not listed in a cell)
#[test]
fn a_jumpjet_in_flight_is_hit_through_the_flying_object_search() {
    let mut w = game();
    let u = put(&mut w, "JV", Kind::Vehicle, 0, 0, 0);
    assert!(!flying::tracked(&w, u), "a grounded jumpjet is not tracked");
    jump::start_in_flight(&mut w, u, Phase::Hovering, 500);
    assert!(flying::tracked(&w, u), "a jumpjet in the air is tracked");
    let c = pos(&w, u).cell();
    assert!(
        !w.cells
            .list(c, glasnost_sim::world::Layer::Ground)
            .contains(&u),
        "at 500 leptons it is not listed in its cell (S-0502 claim 4)"
    );
    let before = w.obj(u).unwrap().health;
    blast_at(&mut w, u);
    assert!(
        w.obj(u).unwrap().health < before,
        "the detonation at its position hits it"
    );
}

// spec: S-0502 claim 29 (revision 18, Q-1500 answered: registered in the update it takes off, at height 0, with no wait for a first rise; registration ends at touchdown)
#[test]
fn a_jumpjet_is_tracked_from_take_off_until_touchdown() {
    let mut w = game();
    let u = put(&mut w, "JV", Kind::Vehicle, 0, 0, 0);
    go(&mut w, u, at(4, 0));
    assert!(
        until(&mut w, 60, |w| phase(w, u) != Phase::Grounded),
        "it takes off"
    );
    assert_eq!(phase(&w, u), Phase::Ascending);
    assert_eq!(height(&w, u), 0, "the take-off update leaves it at height 0");
    assert!(flying::tracked(&w, u), "tracked from the take-off update");
    assert!(
        until(&mut w, 2000, |w| phase(w, u) == Phase::Grounded),
        "it lands"
    );
    assert!(!flying::tracked(&w, u), "it is not tracked after touchdown");
}

// spec: S-0502 claim 30 (revision 18, Q-1501 answered: in the air exactly when on the map and r ≥ 208 leptons, an exact comparison)
#[test]
fn a_jumpjet_is_in_the_air_by_the_height_test() {
    let mut w = game();
    let u = put(&mut w, "JV", Kind::Vehicle, 0, 0, 0);
    assert!(!weapon::in_air(&w, u), "on the ground it is not in the air");
    jump::start_in_flight(&mut w, u, Phase::Hovering, 500);
    assert!(weapon::in_air(&w, u), "at 500 leptons it is in the air");
    let surface = pos(&w, u).z - height(&w, u);
    w.obj_mut(u).unwrap().pos.z = surface + 207;
    assert!(!weapon::in_air(&w, u), "at 207 leptons it is not");
    w.obj_mut(u).unwrap().pos.z = surface + 208;
    assert!(weapon::in_air(&w, u), "at 208 leptons it is");
}

/// A game whose CART type fires only weapon `Flak` (anti-air, not
/// anti-ground) with a guard range of 6 cells (invented values).
fn flak_game() -> World {
    flak_game_with(None)
}

/// As [`flak_game`], with CART's secondary weapon set.
fn flak_game_with(secondary: Option<&str>) -> World {
    let mut r = ruleset();
    r.sim.projectiles.insert(
        "AAOnly".into(),
        ProjectileType {
            aa: Some(true),
            ag: Some(false),
            inviso: Some(true),
            ..Default::default()
        },
    );
    let gun = r.sim.weapons["Gun"].clone();
    r.sim.weapons.insert(
        "Flak".into(),
        WeaponType {
            projectile: Some("AAOnly".into()),
            ..gun
        },
    );
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.primary = Some("Flak".into());
    cart.secondary = secondary.map(String::from);
    cart.guard_range = Some(d("6"));
    glasnost_sim::testkit::game_with(&r, SIZE, 7)
}

// spec: S-0309 claim 9 (a range search that looks for aircraft scans the tracked flying objects first) and table C row "class" (that scan also accepts vehicles); S-0502 claim 4 (a jumpjet above 208 leptons is in no cell, so only that scan finds it)
#[test]
fn a_range_search_for_aircraft_finds_a_jumpjet_in_flight() {
    let mut w = flak_game();
    let a = put(&mut w, "CART", Kind::Vehicle, 0, 0, 0);
    let u = put(&mut w, "JV", Kind::Vehicle, 1, 3, 0);
    // On the ground the jumpjet is in the ground layer: not an airborne object, and a vehicle the anti-air-only search rejects.
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::Range, SearchContext::PerTick),
        None
    );
    jump::start_in_flight(&mut w, u, Phase::Hovering, 500);
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::Range, SearchContext::PerTick),
        Some(u)
    );
}

// spec: S-0309 claim 9 (allied airborne objects are looked at only with a friendly-row exception)
#[test]
fn a_range_search_passes_over_an_allied_jumpjet_in_flight() {
    let mut w = flak_game();
    let a = put(&mut w, "CART", Kind::Vehicle, 0, 0, 0);
    let u = put(&mut w, "JV", Kind::Vehicle, 0, 3, 0);
    jump::start_in_flight(&mut w, u, Phase::Hovering, 500);
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::Range, SearchContext::PerTick),
        None
    );
}

// spec: S-0309 claim 7 (the map-wide search's second pass takes only objects of the ground render layer, so a hovering jumpjet, not an aircraft, is never found by it)
#[test]
fn a_map_wide_search_never_finds_a_hovering_jumpjet() {
    let mut w = flak_game_with(Some("Gun"));
    let a = put(&mut w, "CART", Kind::Vehicle, 0, 0, 0);
    let u = put(&mut w, "JV", Kind::Vehicle, 1, 3, 0);
    jump::start_in_flight(&mut w, u, Phase::Hovering, 500);
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::MapWide, SearchContext::PerTick),
        None
    );
    // The range search finds it among the airborne objects (claim 9).
    assert_eq!(
        acquire::search(&mut w, a, SearchKind::Range, SearchContext::PerTick),
        Some(u)
    );
}

/// The health a blast slightly above the unit takes from it.
fn loss_from_raised_blast(w: &mut World, u: ObjectId) -> i32 {
    let before = w.obj(u).unwrap().health;
    let mut point = pos(w, u);
    point.z += 8;
    let warhead = Some(he(w));
    area::detonate(
        w,
        Detonation {
            point,
            damage: 50,
            warhead,
            source: None,
            house: Some(HouseId(1)),
            resources: false,
        },
    );
    before - w.obj(u).unwrap().health
}

// spec: S-0502 claim 31 (a low jumpjet in its take-off update is registered and listed in its cell: a detonation above the terrain finds it once through the flying-object search and once in its cell, and the two entries are not merged)
#[test]
fn a_low_jumpjet_is_found_twice() {
    let mut w = game();
    let u = put(&mut w, "JV", Kind::Vehicle, 0, 0, 0);
    let once = loss_from_raised_blast(&mut w, u);
    assert!(once > 0, "the grounded unit is hit once, in its cell");
    go(&mut w, u, at(4, 0));
    assert!(until(&mut w, 60, |w| phase(w, u) != Phase::Grounded));
    assert!(flying::tracked(&w, u));
    let c = pos(&w, u).cell();
    assert!(w.cells.list(c, glasnost_sim::world::Layer::Ground).contains(&u));
    let twice = loss_from_raised_blast(&mut w, u);
    assert_eq!(twice, 2 * once, "two entries at the same distance");
}

// spec: S-0303 claim 6 and Edge cases (revision 19: quirks.flying_listed_double_hit on, an object found by the flying-object search and a scanned cell takes two hits, flying first; off, only its flying entry, one hit)
#[test]
fn flying_listed_double_hit_both_positions() {
    for on in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.flying_listed_double_hit = on;
        let mut w = glasnost_sim::testkit::game_with(&r, SIZE, 7);
        glasnost_sim::testkit::run(&mut w, 4);
        let u = put(&mut w, "JV", Kind::Vehicle, 0, 0, 0);
        let once = loss_from_raised_blast(&mut w, u);
        go(&mut w, u, at(4, 0));
        assert!(until(&mut w, 60, |w| phase(w, u) != Phase::Grounded));
        assert!(flying::tracked(&w, u));
        let c = pos(&w, u).cell();
        assert!(w.cells.list(c, glasnost_sim::world::Layer::Ground).contains(&u));
        let loss = loss_from_raised_blast(&mut w, u);
        assert_eq!(loss, if on { 2 * once } else { once }, "switch {on}");
    }
}

// spec: S-0502 claim 29 (crash end: registration ends when the unit enters crashed, before the impact; the start of a crash does not end it)
#[test]
fn a_crashing_jumpjet_stays_tracked_until_the_crash_ends() {
    let mut w = game();
    let u = put(&mut w, "JV", Kind::Vehicle, 0, 0, 0);
    jump::start_in_flight(&mut w, u, Phase::Hovering, 500);
    assert!(jump::start_crash(&mut w, u, None));
    step(&mut w);
    assert_eq!(phase(&w, u), Phase::Crashing);
    assert!(flying::tracked(&w, u), "a crashing unit stays registered");
    let mut ended = false;
    for _ in 0..400 {
        step(&mut w);
        if w.obj(u).is_none_or(|_| phase(&w, u) == Phase::Crashed) {
            ended = true;
            break;
        }
        assert!(flying::tracked(&w, u));
    }
    assert!(ended, "the crash ends");
    assert!(!flying::tracked(&w, u), "not registered from the crash end");
}

/// A JV jumpjet descending to land under `quirk`, with its health set to 0
/// (no longer alive) just before touchdown: whether it is still tracked
/// once grounded.
fn dead_touchdown_tracked(quirk: bool) -> bool {
    let mut r = ruleset();
    r.globals.quirks.jumpjet_falling_touchdown_tracked = quirk;
    let mut w = glasnost_sim::testkit::game_with(&r, SIZE, 7);
    glasnost_sim::testkit::run(&mut w, 4);
    let u = put(&mut w, "JV", Kind::Vehicle, 0, 0, 0);
    go(&mut w, u, at(2, 0));
    assert!(until(&mut w, 2000, |w| phase(w, u) == Phase::Descending));
    assert!(flying::tracked(&w, u));
    w.obj_mut(u).unwrap().health = 0;
    assert!(until(&mut w, 2000, |w| phase(w, u) == Phase::Grounded));
    flying::tracked(&w, u)
}

// spec: S-0502 claim 29 and Edge cases (quirks.jumpjet_falling_touchdown_tracked: on, a touchdown of a unit no longer alive leaves it registered; off, every touchdown ends registration)
#[test]
fn falling_touchdown_quirk_both_positions() {
    assert!(dead_touchdown_tracked(true));
    assert!(!dead_touchdown_tracked(false));
}

// spec: S-0502 claim 29 (registration moves to the sector of a new cell in ascending and cruising updates, with S-0303 claim 11's arrival order)
#[test]
fn a_cruising_jumpjet_changes_sectors() {
    let mut w = game();
    let u = put(&mut w, "JV", Kind::Vehicle, 0, -12, 0);
    let start = flying::sector(&w, pos(&w, u).cell());
    go(&mut w, u, at(12, 0));
    let goal = flying::sector(&w, at(12, 0));
    assert_ne!(start, goal, "the flight crosses a sector boundary");
    assert!(until(&mut w, 3000, |w| phase(w, u) == Phase::Descending));
    assert_eq!(w.flying.sector_of.get(&u), Some(&goal));
}

// spec: S-0502 claim 29 (placed on the map in the air: registered only when airborne and its type has the counts-as-aircraft flag; without the flag not until it next takes off)
#[test]
fn placement_in_the_air_needs_the_counts_as_aircraft_flag() {
    for flag in [false, true] {
        let mut r = ruleset();
        r.sim.vehicles.get_mut("JV").unwrap().econ.considered_aircraft = Some(flag);
        let mut w = glasnost_sim::testkit::game_with(&r, SIZE, 7);
        let u = put(&mut w, "JV", Kind::Vehicle, 0, 0, 0);
        assert!(!flying::tracked(&w, u), "placed on the ground: not registered");
        let mut p = pos(&w, u);
        let facing = w.body_facing(u);
        glasnost_sim::objects::take_inside(&mut w, u);
        p.z += 300;
        glasnost_sim::objects::place(&mut w, u, p, facing);
        assert_eq!(flying::tracked(&w, u), flag);
    }
}
