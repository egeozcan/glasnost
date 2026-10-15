//! Claim tests of S-0513 claims 33 to 41 (the teleport: source pick, warp
//! group, landing and timing), on synthetic rules (`tests/sw_common`).

mod sw_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::superweapons::warp::Stage;
use glasnost_sim::superweapons::{self, ProtectKind, SwEvent, protect};
use glasnost_sim::world::{ObjectId, Pos, Target, World};
use glasnost_sim::{save, teleport, testkit};
use sw_common::*;

/// The rules' teleport delay in these tests.
const DELAY: i32 = 7;

fn rules() -> Ruleset {
    let mut r = ruleset();
    {
        let g = &mut r.sim.general.superweapons;
        g.chrono_placement = Some("MARK".into());
        g.chrono_blast = Some("BLASTS".into());
        g.chrono_blast_dest = Some("BLASTD".into());
        g.chrono_delay = Some(DELAY);
    }
    r.sim.general.warp_out = Some("WARPA".into());
    for n in ["MARK", "BLASTS", "BLASTD", "WARPA"] {
        r.sim
            .animations
            .insert(n.into(), glasnost_rules::sim::AnimationType::default());
        r.registries
            .entry("animations".into())
            .or_default()
            .push(n.into());
    }
    r
}

fn stage(w: &World, id: ObjectId) -> Option<Stage> {
    w.obj(id).and_then(|o| o.sw.warp.as_ref()).map(|m| m.stage)
}

fn dead(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_none_or(|o| o.health <= 0)
}

/// Makes ALPHA's teleport source ready (a CHRONOB far away).
fn chrono(w: &mut World) {
    spawn(w, "CHRONOB", ALPHA, -9, -9);
    ready_tick(w, ALPHA, SOURCE, 300);
}

/// Picks `src` and orders the warp to `dst`; returns t_L, the tick that
/// applies the warp order (S-0203 claim 7). Runs that tick.
fn warp(w: &mut World, src: CellXY, dst: CellXY) -> i32 {
    if !rec(w, ALPHA, SOURCE).ready {
        ready_tick(w, ALPHA, SOURCE, 300);
    }
    fire_now(w, ALPHA, SOURCE, src);
    fire(w, ALPHA, WARP, dst);
    let tl = w.tick;
    step(w);
    tl
}

/// Runs until tick `t` has run (the next tick to run is `t + 1`).
fn run_through(w: &mut World, t: i32) {
    while w.tick <= t {
        step(w);
    }
}

// --- Claim 33 ------------------------------------------------------------------

#[test]
fn claim33_source_pick_places_the_marker_for_its_player() {
    // spec: S-0513 claims 5 and 33 (a source pick: the marker over the source for the launching player only; no announcement, no radar event)
    let mut w = game(&rules());
    chrono(&mut w);
    w.effects.clear();
    fire_now(&mut w, ALPHA, SOURCE, at(2, 2));
    let ev = events(&w);
    assert!(ev.contains(&(SwEvent::SourceMarker, Some(ALPHA), Some(SOURCE))));
    assert!(
        !ev.iter()
            .any(|e| matches!(e.0, SwEvent::Launch | SwEvent::Radar))
    );
    assert_eq!(rec(&w, ALPHA, SOURCE).source, Some(at(2, 2)));
}

#[test]
fn claim33_the_marker_takes_its_creation_draws_whichever_house_picks() {
    // spec: S-0513 claim 33 (Q-0741 part 1: the marker is created whichever house is local and takes its ordinary animation creation draws; hiding it for another viewer does not undo them)
    let mut r = rules();
    // A random rate pair whose delays differ gives the creation one ranged draw (S-0109 claim 3).
    r.sim.animations.get_mut("MARK").unwrap().random_rate = Some([450, 225]);
    for house in [ALPHA, BETA] {
        let mut w = game(&r);
        spawn(&mut w, "CHRONOB", house, -9, -9);
        ready_tick(&mut w, house, SOURCE, 300);
        let before = w.rng.clone();
        w.effects.clear();
        fire_now(&mut w, house, SOURCE, at(2, 2));
        assert_ne!(w.rng, before, "house {house:?}: the marker's creation drew");
        assert!(events(&w).contains(&(SwEvent::SourceMarker, Some(house), Some(SOURCE))));
    }
    // A marker type with a constant delay takes no draw.
    let mut w = game(&rules());
    chrono(&mut w);
    let before = w.rng.clone();
    fire_now(&mut w, ALPHA, SOURCE, at(2, 2));
    assert_eq!(w.rng, before);
}

#[test]
fn claim48_a_warp_member_drops_its_target_and_destination() {
    // spec: S-0513 claim 48 (a vehicle still in the warp drops its target and destination, and its former driving does not continue)
    let mut w = game(&rules());
    chrono(&mut w);
    let a = spawn(&mut w, "CART", ALPHA, 0, 0);
    let e = spawn(&mut w, "BUGGY", BETA, 4, 4);
    {
        let o = w.obj_mut(a).unwrap();
        o.target = Some(Target::Object(e));
        o.dest = Some(Target::Cell(at(2, 2)));
    }
    warp(&mut w, at(0, 0), at(5, -5));
    let o = w.obj(a).unwrap();
    assert!(o.sw.warp.is_some());
    assert!(o.target.is_none() && o.dest.is_none());
    assert!(o.path.steps.is_empty());
}

// --- Claim 34 ------------------------------------------------------------------

#[test]
fn claim34_launch_effects_and_group() {
    // spec: S-0513 claims 5 and 34 (activation announcement, radar events at both cells, the blast animations at the source and destination, the marker ends; the block's mobile objects of any house join)
    let mut w = game(&rules());
    chrono(&mut w);
    let a = spawn(&mut w, "CART", ALPHA, 0, 0);
    let b = spawn(&mut w, "BUGGY", BETA, 1, -1);
    let tower = spawn(&mut w, "TOWER", BETA, -1, 0);
    let outside = spawn(&mut w, "CART", BETA, 2, 0);
    fire_now(&mut w, ALPHA, SOURCE, at(0, 0));
    w.effects.clear();
    fire_now(&mut w, ALPHA, WARP, at(5, 5));
    let ev = events(&w);
    assert!(ev.iter().any(|e| e.0 == SwEvent::Launch));
    assert_eq!(ev.iter().filter(|e| e.0 == SwEvent::Radar).count(), 2);
    assert!(ev.iter().any(|e| e.0 == SwEvent::MarkerEnd));
    let an = anims(&w);
    let bs = an.iter().position(|a| a == "BLASTS").unwrap();
    let bd = an.iter().position(|a| a == "BLASTD").unwrap();
    assert!(bs < bd, "the source blast first");
    assert_eq!(stage(&w, a), Some(Stage::Pending));
    assert_eq!(stage(&w, b), Some(Stage::Pending));
    // spec: S-0513 claim 34 (buildings are never moved; objects outside the block are not reached)
    assert_eq!(stage(&w, tower), None);
    assert_eq!(stage(&w, outside), None);
    // spec: S-0513 claim 36 (the group keeps its 3 × 3 shape: a vehicle lands at the centre of the matching cell)
    let m = w.obj(b).unwrap().sw.warp.clone().unwrap();
    let c = at(6, 4);
    assert_eq!(
        (m.landing.x, m.landing.y),
        (c.x * 256 + 128, c.y * 256 + 128)
    );
}

#[test]
fn claim34_vehicle_stops_occupying_its_cell() {
    // spec: S-0513 claim 34 (a vehicle ceases to occupy its cell at launch)
    let mut w = game(&rules());
    chrono(&mut w);
    let a = spawn(&mut w, "CART", ALPHA, 0, 0);
    let c = w.obj(a).unwrap().pos.cell();
    assert!(
        w.cells
            .occ(c, glasnost_sim::world::Layer::Ground)
            .unwrap()
            .vehicles
            > 0
    );
    warp(&mut w, at(0, 0), at(5, 5));
    assert_eq!(
        w.cells
            .occ(c, glasnost_sim::world::Layer::Ground)
            .unwrap()
            .vehicles,
        0
    );
}

#[test]
fn claim34_protected_and_busy_objects_stay() {
    // spec: S-0513 claim 34 (a protected object, or one already in a warp, is left in place)
    let mut w = game(&rules());
    chrono(&mut w);
    let a = spawn(&mut w, "CART", ALPHA, 0, 0);
    let b = spawn(&mut w, "BUGGY", BETA, 1, 0);
    protect::start(&mut w, b, ProtectKind::Curtain, 500);
    warp(&mut w, at(0, 0), at(5, 5));
    assert_eq!(stage(&w, a), Some(Stage::Pending));
    assert_eq!(stage(&w, b), None);
    // A second warp in the leaving stage leaves the member in its first warp.
    step(&mut w);
    let before = w.obj(a).unwrap().sw.warp.clone();
    // The second-stage order is unchecked (S-0512 claim 10) and reuses the source.
    fire_now(&mut w, ALPHA, WARP, at(-5, -5));
    let after = w.obj(a).unwrap().sw.warp.clone().unwrap();
    assert_eq!(after.landing, before.unwrap().landing);
}

#[test]
fn claim34_aircraft_in_flight_is_left_alone() {
    // spec: S-0513 claim 34 (an object in the air is left alone)
    let mut w = game(&rules());
    chrono(&mut w);
    let p = spawn(&mut w, "PDPLANE", BETA, 0, 0);
    let surface = w.surface_for(p);
    w.obj_mut(p).unwrap().pos.z = surface + 1000;
    w.obj_mut(p).unwrap().air.as_mut().unwrap().ground = false;
    warp(&mut w, at(0, 0), at(5, 5));
    assert_eq!(stage(&w, p), None);
}

// --- Claim 35 ------------------------------------------------------------------

#[test]
fn claim35_organic_objects_die_crediting_the_launching_house() {
    // spec: S-0513 claims 35 and 41 (organic without the teleporter flag: a whole-strength demolition hit, the launching house credited; the teleporter flag joins)
    let mut w = game(&rules());
    chrono(&mut w);
    let blob = spawn(&mut w, "BLOB", BETA, 0, 0);
    let walker = spawn(&mut w, "WALKER", BETA, 1, 0);
    let jumper = spawn(&mut w, "JUMPER", BETA, 0, 1);
    let kills = w.houses[0].combat.kills;
    warp(&mut w, at(0, 0), at(5, 5));
    assert!(dead(&w, blob));
    assert!(dead(&w, walker));
    assert_eq!(w.houses[0].combat.kills, kills + 2);
    assert_eq!(stage(&w, jumper), Some(Stage::Pending));
}

#[test]
fn claim35_protected_organic_quirk() {
    // spec: S-0513 claim 35 (quirks.warp_kill_before_protection: protection does not prevent the death; off: left alone like any protected object)
    for quirk in [true, false] {
        let mut r = rules();
        r.globals.quirks.warp_kill_before_protection = quirk;
        let mut w = game(&r);
        chrono(&mut w);
        let blob = spawn(&mut w, "BLOB", BETA, 0, 0);
        protect::start(&mut w, blob, ProtectKind::ForceShield, 500);
        warp(&mut w, at(0, 0), at(5, 5));
        assert_eq!(dead(&w, blob), quirk, "quirk {quirk}");
        assert_eq!(stage(&w, blob), None);
    }
}

// --- Claims 36, 39 and 40: a clean warp ----------------------------------------

#[test]
fn claim39_40_clean_timeline() {
    // spec: S-0513 claim 39 and T2 (leaving stage from t_L + 1, wait expiry in t_L + 61, departure in t_L + 62, placed in t_L + 63, arrival effects in t_L + 64) and claim 40 (held delay 0: phasing in ends in t_A + 1)
    let mut w = game(&rules());
    chrono(&mut w);
    let a = spawn(&mut w, "CART", ALPHA, 0, 0);
    let old = w.obj(a).unwrap().pos;
    let tl = warp(&mut w, at(0, 0), at(5, 5));
    assert_eq!(stage(&w, a), Some(Stage::Pending));
    assert!(!superweapons::being_erased(&w, a));
    run_through(&mut w, tl + 1);
    assert_eq!(stage(&w, a), Some(Stage::Leaving(tl + 1)));
    assert!(superweapons::being_erased(&w, a));
    run_through(&mut w, tl + 60);
    assert_eq!(stage(&w, a), Some(Stage::Leaving(tl + 1)));
    run_through(&mut w, tl + 61);
    assert_eq!(stage(&w, a), Some(Stage::Expired));
    assert!(superweapons::being_erased(&w, a));
    assert_eq!(w.obj(a).unwrap().pos, old, "still at its old position");
    w.effects.clear();
    run_through(&mut w, tl + 62);
    assert_eq!(stage(&w, a), Some(Stage::Away));
    assert!(!superweapons::being_erased(&w, a));
    assert!(teleport::phasing_in(&w, a));
    assert!(!w.obj(a).unwrap().on_map);
    assert!(
        !w.cells
            .list(old.cell(), glasnost_sim::world::Layer::Ground)
            .contains(&a)
    );
    assert_eq!(anims(&w), vec!["WARPA".to_string()]);
    run_through(&mut w, tl + 63);
    assert_eq!(stage(&w, a), Some(Stage::Placed));
    let c = at(5, 5);
    let p = w.obj(a).unwrap().pos;
    assert_eq!((p.x, p.y), (c.x * 256 + 128, c.y * 256 + 128));
    assert!(w.obj(a).unwrap().on_map);
    run_through(&mut w, tl + 64);
    assert_eq!(stage(&w, a), Some(Stage::Hold(tl + 64)));
    assert!(teleport::phasing_in(&w, a));
    run_through(&mut w, tl + 65);
    assert_eq!(stage(&w, a), None);
    assert!(!teleport::phasing_in(&w, a));
    assert!(!dead(&w, a));
}

#[test]
fn claim36_infantry_keeps_its_position_within_the_cell() {
    // spec: S-0513 claim 36 (an infantry unit keeps its position within the cell)
    let mut w = game(&rules());
    chrono(&mut w);
    let j = spawn(&mut w, "JUMPER", ALPHA, 1, 1);
    let p = w.obj(j).unwrap().pos;
    warp(&mut w, at(0, 0), at(-5, 4));
    let m = w.obj(j).unwrap().sw.warp.clone().unwrap();
    let c = at(-4, 5);
    assert_eq!(
        (m.landing.x, m.landing.y),
        (
            c.x * 256 + p.x.rem_euclid(256),
            c.y * 256 + p.y.rem_euclid(256)
        )
    );
}

#[test]
fn claim39_targeters_drop_the_member_at_departure() {
    // spec: S-0513 claim 39 (at departure, every object targeting it drops it, S-0207 claim 20 e)
    let mut w = game(&rules());
    chrono(&mut w);
    let a = spawn(&mut w, "CART", ALPHA, 0, 0);
    let x = spawn(&mut w, "CART", BETA, 0, 6);
    let tl = warp(&mut w, at(0, 0), at(5, 5));
    run_through(&mut w, tl + 61);
    w.obj_mut(x).unwrap().target = Some(Target::Object(a));
    run_through(&mut w, tl + 62);
    assert_ne!(w.obj(x).unwrap().target, Some(Target::Object(a)));
}

// --- Claim 37 ------------------------------------------------------------------

#[test]
fn claim37_landing_kills_an_unprotected_vehicle() {
    // spec: S-0513 claim 37 (an unprotected vehicle in the landing cell is killed, no attacking house)
    let mut w = game(&rules());
    chrono(&mut w);
    let a = spawn(&mut w, "CART", ALPHA, 0, 0);
    let v = spawn(&mut w, "BUGGY", BETA, 5, 5);
    let kills = w.houses[0].combat.kills;
    let tl = warp(&mut w, at(0, 0), at(5, 5));
    run_through(&mut w, tl + 62);
    assert!(dead(&w, v));
    assert_eq!(w.houses[0].combat.kills, kills, "no credit");
    assert_eq!(stage(&w, a), Some(Stage::Away));
}

#[test]
fn claim37_protected_occupant_kills_the_member() {
    // spec: S-0513 claim 37 (any protected object kills the member)
    let mut w = game(&rules());
    chrono(&mut w);
    let a = spawn(&mut w, "CART", ALPHA, 0, 0);
    let v = spawn(&mut w, "BUGGY", BETA, 5, 5);
    let tl = warp(&mut w, at(0, 0), at(5, 5));
    run_through(&mut w, tl + 61);
    protect::start(&mut w, v, ProtectKind::Curtain, 500);
    run_through(&mut w, tl + 62);
    assert!(dead(&w, a));
    assert!(!dead(&w, v));
}

#[test]
fn claim37_infantry_on_infantry() {
    // spec: S-0513 claim 37 (an infantry member kills an infantry unit only on its exact landing point; otherwise both share the cell)
    for exact in [true, false] {
        let mut w = game(&rules());
        chrono(&mut w);
        let j = spawn(&mut w, "JUMPER", ALPHA, 0, 0);
        let v = spawn(&mut w, "WALKER", BETA, 5, 5);
        let tl = warp(&mut w, at(0, 0), at(5, 5));
        let landing = w.obj(j).unwrap().sw.warp.clone().unwrap().landing;
        run_through(&mut w, tl + 61);
        let mut p = landing;
        if !exact {
            p.x += 40;
        }
        p.z = w.obj(v).unwrap().pos.z;
        w.obj_mut(v).unwrap().pos = p;
        run_through(&mut w, tl + 62);
        assert_eq!(dead(&w, v), exact, "exact {exact}");
        assert_eq!(stage(&w, j), Some(Stage::Away));
    }
}

#[test]
fn claim37_40_blocked_landing_is_redirected_and_holds() {
    // spec: S-0513 claim 37 (a building blocks the landing: the member's landing point moves to a nearby cell) and claim 40 (the held delay becomes the teleport delay; phasing in ends in t_A + H)
    let mut w = game(&rules());
    chrono(&mut w);
    let a = spawn(&mut w, "CART", ALPHA, 0, 0);
    spawn(&mut w, "TOWER", BETA, 5, 5);
    let tl = warp(&mut w, at(0, 0), at(5, 5));
    run_through(&mut w, tl + 62);
    assert_eq!(stage(&w, a), Some(Stage::Blocked));
    let m = w.obj(a).unwrap().sw.warp.clone().unwrap();
    assert_ne!(m.landing.cell(), at(5, 5));
    assert!(m.redirected);
    assert_eq!(w.obj(a).unwrap().sw.held_delay, DELAY);
    let t = until(&mut w, 20, |w| {
        stage(w, a).is_some_and(|s| matches!(s, Stage::Hold(_)))
    });
    let ta = t - 1;
    assert_eq!(stage(&w, a), Some(Stage::Hold(ta)));
    run_through(&mut w, ta + DELAY - 1);
    assert!(teleport::phasing_in(&w, a));
    run_through(&mut w, ta + DELAY);
    assert!(!teleport::phasing_in(&w, a));
    assert_eq!(w.obj(a).unwrap().pos.cell(), m.landing.cell());
}

#[test]
fn claim40_held_delay_persists_quirk() {
    // spec: S-0513 claim 40 (quirks.warp_hold_persists: a member once redirected keeps the hold for a later clean warp; off: 0 for that warp)
    for quirk in [true, false] {
        let mut r = rules();
        r.globals.quirks.warp_hold_persists = quirk;
        let mut w = game(&r);
        chrono(&mut w);
        let a = spawn(&mut w, "CART", ALPHA, 0, 0);
        w.obj_mut(a).unwrap().sw.held_delay = DELAY;
        let tl = warp(&mut w, at(0, 0), at(5, 5));
        run_through(&mut w, tl + 65);
        assert_eq!(stage(&w, a).is_some(), quirk, "quirk {quirk}");
        run_through(&mut w, tl + 64 + DELAY);
        assert_eq!(stage(&w, a), None);
    }
}

// --- Claims 38 and 41 ------------------------------------------------------------

fn water_at(w: &mut World, cells: &[CellXY]) {
    let mut cat = (*testkit::catalogue()).clone();
    let first = cat.tiles.len() as i32;
    let mut water = cat.tiles[0].clone();
    if let Some(Some(c)) = water.cells.first_mut() {
        c.terrain_code = 9; // the catalogue's water terrain code (as in claims_s0614)
    }
    cat.tiles.push(water);
    cat.allow_tiberium.push(false);
    w.catalogue = Some(std::sync::Arc::new(cat));
    for &c in cells {
        w.cells.grid.get_mut(c).unwrap().tile = first;
    }
    glasnost_sim::load::recalc_cells(w, cells);
}

#[test]
fn claim38_41_vehicle_on_water_sinks_crediting_the_launching_house() {
    // spec: S-0513 claims 38 and 41 (a non-infantry, non-naval, non-hovering member arriving on water off a deck sinks; its kill is credited to the launching house)
    let mut w = game(&rules());
    chrono(&mut w);
    water_at(&mut w, &[at(5, 5)]);
    assert_eq!(
        w.cells.cell(at(5, 5)).unwrap().land,
        glasnost_map::terrain::land::WATER
    );
    let a = spawn(&mut w, "CART", BETA, 0, 0);
    let kills = w.houses[0].combat.kills;
    let tl = warp(&mut w, at(0, 0), at(5, 5));
    run_through(&mut w, tl + 64);
    assert!(glasnost_sim::naval::sinking(&w, a));
    assert_eq!(w.houses[0].combat.kills, kills + 1);
    assert_eq!(stage(&w, a), None);
}

#[test]
fn claim38_protected_ground_object_kills_on_arrival() {
    // spec: S-0513 claim 38 (a protected object listed in the landing cell's ground layer kills the member on arrival)
    let mut w = game(&rules());
    chrono(&mut w);
    let j = spawn(&mut w, "JUMPER", ALPHA, 0, 0);
    let v = spawn(&mut w, "WALKER", BETA, 5, 5);
    let tl = warp(&mut w, at(0, 0), at(5, 5));
    // Shares the cell at the check, then protected before the arrival.
    let landing = w.obj(j).unwrap().sw.warp.clone().unwrap().landing;
    run_through(&mut w, tl + 61);
    let z = w.obj(v).unwrap().pos.z;
    w.obj_mut(v).unwrap().pos = Pos::new(landing.x + 40, landing.y, z);
    run_through(&mut w, tl + 63);
    assert!(!dead(&w, j));
    protect::start(&mut w, v, ProtectKind::Curtain, 500);
    run_through(&mut w, tl + 64);
    assert!(dead(&w, j));
}

// --- Determinism ---------------------------------------------------------------------

#[test]
fn warp_state_survives_save_and_load() {
    // spec: S-0513 T2 (a member's warp state is game state: the state hash survives a save and load mid-warp)
    let mut w = game(&rules());
    chrono(&mut w);
    let a = spawn(&mut w, "CART", ALPHA, 0, 0);
    let tl = warp(&mut w, at(0, 0), at(5, 5));
    run_through(&mut w, tl + 62);
    let bytes = save::save(&w);
    let mut l = save::load(&bytes, w.rules_arc(), testkit::catalogue()).expect("loads");
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
    run_through(&mut w, tl + 70);
    run_through(&mut l, tl + 70);
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
    assert_eq!(stage(&l, a), None);
}
