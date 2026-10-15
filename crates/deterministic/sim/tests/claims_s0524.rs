//! Claim tests of S-0524 (multiplayer trigger completion) on synthetic
//! rules (`tests/sw_common`) and synthetic maps with triggers and
//! waypoints. Every name and number is invented; no game data.

mod sw_common;

use std::sync::Arc;

use glasnost_map::triggers::{Action, Event, Owner, TagDef, TriggerDef, Triggers};
use glasnost_map::objects::Placed;
use glasnost_map::{CellXY, Structure};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, LandType, WarheadType};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::mp_triggers::{self, Coverage};
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::superweapons::StormState;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, World};
use sw_common::{ALPHA, BETA, DEFER, SIZE, at, run, step};

/// The demolition warhead's verses: one tenth against every armour.
const DEMO_VERSES: &str = "0.1";
/// The test tower's strength (the testkit's TOWER).
const TOWER_HP: i32 = 800;

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

/// The superweapon ruleset with a demolition warhead DEMO: verses 0.1,
/// a two-cell spread and no fall-off, so each 100-point blast takes 10.
fn ruleset() -> Ruleset {
    let mut r = sw_common::ruleset();
    r.sim.warheads.insert(
        "DEMO".into(),
        WarheadType {
            cell_spread: Some(d("2")),
            percent_at_max: Some(d("1")),
            verses: Some(vec![d(DEMO_VERSES); 11]),
            ..Default::default()
        },
    );
    r.sim.general.c4_warhead = Some("DEMO".into());
    // Clear ground is buildable, so that a restored building can be placed.
    r.sim.land_types.get_mut(&LandType::Clear).unwrap().buildable = Some(true);
    r
}

fn demo(r: &mut Ruleset) -> &mut WarheadType {
    r.sim.warheads.get_mut("DEMO").unwrap()
}

fn action(code: i32) -> Action {
    Action {
        code,
        ..Default::default()
    }
}

fn event(code: i32, value: i32) -> Event {
    Event {
        code,
        kind: 0,
        value,
        type_name: None,
    }
}

/// A trigger of owner `owner` with one event and `actions`.
fn trigger(id: &str, owner: Option<&str>, ev: Event, actions: Vec<Action>) -> TriggerDef {
    TriggerDef {
        owner: owner.map_or(Owner::None, |o| Owner::Name(o.as_bytes().to_vec())),
        events: vec![ev],
        actions,
        ..TriggerDef::created(id.as_bytes().to_vec())
    }
}

/// One tag per trigger, of repeat type `repeat`, named like the trigger.
fn triggers(defs: Vec<TriggerDef>, repeat: i32) -> Triggers {
    let tags = defs
        .iter()
        .enumerate()
        .map(|(i, t)| TagDef {
            id: t.id.clone(),
            repeat,
            name: t.id.clone(),
            first: Some(i as u32),
            chain: vec![i as u32],
        })
        .collect();
    Triggers {
        triggers: defs,
        tags,
        ..Default::default()
    }
}

/// A game on an empty map with `tr`, waypoints `wps` (number, cell) and
/// pre-placed buildings `structures` (owner, type, cell).
fn game(r: &Ruleset, tr: Triggers, wps: &[(u16, CellXY)], structures: &[(&str, &str, CellXY)]) -> World {
    let mut m = testkit::empty_map(SIZE);
    m.triggers = tr;
    for &(n, c) in wps {
        m.waypoints.insert(n, (c.x as i16, c.y as i16));
    }
    for (k, &(owner, ty, cell)) in structures.iter().enumerate() {
        m.objects.structures.push(Structure {
            placed: Placed {
                key: format!("{k}").into_bytes(),
                owner: owner.as_bytes().to_vec(),
                type_name: ty.as_bytes().to_vec(),
                health: 256,
                cell,
                facing: 0,
                raw: Vec::new(),
            },
        });
    }
    let rules = Arc::new(SimRules::new(r, "synthetic-test"));
    glasnost_sim::load::new_game(rules, &m, testkit::catalogue(), &testkit::setup(7))
        .expect("synthetic game loads")
}

/// A game whose one-time trigger fires `actions` at once (elapsed 0),
/// owned by `owner`; runs until it has fired.
fn fired_game(r: &Ruleset, owner: Option<&str>, actions: Vec<Action>, wps: &[(u16, CellXY)]) -> World {
    let t = trigger("T", owner, event(13, 0), actions);
    let mut w = game(r, triggers(vec![t], 0), wps, &[]);
    fire(&mut w);
    w
}

/// Runs ticks until a trigger has fired (at most 100).
fn fire(w: &mut World) {
    let f = w.triggers.fired;
    for _ in 0..100 {
        if w.triggers.fired > f {
            return;
        }
        step(w);
    }
    panic!("the trigger did not fire");
}

fn clouds(w: &World) -> usize {
    w.sw.clouds.len()
}

fn health(w: &World, id: ObjectId) -> i32 {
    w.obj(id).unwrap().health
}

fn buildings(w: &World, ty: &str) -> Vec<ObjectId> {
    let ti = w.rules().index_of(Kind::Building, ty).unwrap();
    w.objects
        .values()
        .filter(|o| o.kind == Kind::Building && o.on_map && o.type_index == ti)
        .map(|o| o.id)
        .collect()
}

fn hit(w: &mut World, id: ObjectId, amount: i32) {
    damage::apply(
        w,
        id,
        Hit {
            amount,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: true,
        },
    );
}

fn country(w: &World, name: &str) -> i32 {
    w.rules().country_names.iter().position(|c| c == name).unwrap() as i32
}

fn flashes(w: &World) -> Vec<(i32, Option<glasnost_sim::world::Pos>)> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Trigger {
                code: 66,
                number,
                pos,
                ..
            } => Some((*number, *pos)),
            _ => None,
        })
        .collect()
}

// --- Claim 1 -------------------------------------------------------------------

// spec: S-0524 claim 1 (code 102 requests a storm at the waypoint-slot cell owned by the trigger's owner; a later request during the deferment replaces centre and owner without lengthening the countdown; an active storm blocks it)
#[test]
fn claim1_storm_request_at_a_waypoint() {
    let r = ruleset();
    let a = |wp| Action {
        waypoint: wp,
        ..action(102)
    };
    // First at once, the second after one time unit (15 ticks), the third after 4 (60 ticks).
    let t = triggers(
        vec![
            trigger("A", Some("Alphaland"), event(13, 0), vec![a(0)]),
            trigger("B", Some("Betaland"), event(13, 1), vec![a(1)]),
            trigger("C", Some("Alphaland"), event(13, 4), vec![a(2)]),
        ],
        0,
    );
    let wps = [(0, at(0, 0)), (1, at(3, 0)), (2, at(-3, 0))];
    let mut w = game(&r, t, &wps, &[]);
    fire(&mut w);
    assert_eq!(w.sw.storm.state, StormState::Counting);
    assert_eq!(w.sw.storm.centre, at(0, 0));
    assert_eq!(w.sw.storm.owner, Some(ALPHA));
    let first = w.tick;
    fire(&mut w);
    assert_eq!(w.sw.storm.centre, at(3, 0));
    assert_eq!(w.sw.storm.owner, Some(BETA));
    // Not lengthened: it still ends within the first request's deferment.
    assert!(w.sw.storm.countdown <= DEFER - (w.tick - first - 1));
    fire(&mut w);
    assert_eq!(w.sw.storm.state, StormState::Raging);
    assert_eq!(w.sw.storm.centre, at(3, 0), "an active storm blocks the action");
}

// --- Claim 2 -------------------------------------------------------------------

// spec: S-0524 claim 2 (two code-90 actions at one waypoint make two clouds, with no storm active: no previous-strike exclusion)
#[test]
fn claim2_repeated_strikes_make_two_clouds() {
    let r = ruleset();
    let a = || Action {
        waypoint: 0,
        ..action(90)
    };
    let w = fired_game(&r, None, vec![a(), a()], &[(0, at(0, 0))]);
    assert_eq!(w.sw.storm.state, StormState::Idle);
    assert_eq!(clouds(&w), 2);
}

// spec: S-0524 Edge cases (quirks.repeated_lightning_strike_suppressed on: after one cloud at a waypoint, further code-90 actions there make none; other waypoints still do)
#[test]
fn claim2_quirk_suppresses_repeated_strikes() {
    let mut r = ruleset();
    r.globals.quirks.repeated_lightning_strike_suppressed = true;
    let a = |wp| Action {
        waypoint: wp,
        ..action(90)
    };
    let w = fired_game(&r, None, vec![a(0), a(0), a(1)], &[(0, at(0, 0)), (1, at(4, 0))]);
    assert_eq!(clouds(&w), 2);
}

// --- Claim 3 -------------------------------------------------------------------

// spec: S-0524 claim 3 (code 95 launches the fixed-name payload at the waypoint-slot cell with no firer and no house; the number slot is unused)
#[test]
fn claim3_nuclear_strike_ignores_the_number_slot() {
    let r = ruleset();
    let payload = |w: &World| {
        let wp = *w.rules().combat.weapon_index.get("NukePayload").unwrap();
        w.projectiles
            .values()
            .filter(|p| p.weapon == wp)
            .map(|p| format!("{:?} {:?} {:?} {:?}", p.target, p.attacker, p.house, p.pos))
            .collect::<Vec<_>>()
    };
    let launch = |n: i32| {
        let a = Action {
            waypoint: 0,
            number: n,
            ..action(95)
        };
        let w = fired_game(&r, Some("Alphaland"), vec![a], &[(0, at(2, 1))]);
        payload(&w)
    };
    let zero = launch(0);
    assert_eq!(zero.len(), 1);
    assert!(zero[0].contains(" None None "), "no firer and no house");
    assert_eq!(zero, launch(7));
}

// --- Claim 4 -------------------------------------------------------------------

// spec: S-0524 claim 4 (code 108 is recognised; crates, S-0520, are a seam: no effect and a note)
#[test]
fn claim4_crate_action_is_a_seam() {
    let r = ruleset();
    let a = Action {
        waypoint: 0,
        number: 20,
        ..action(108)
    };
    let w = fired_game(&r, None, vec![a], &[(0, at(0, 0))]);
    assert_eq!(mp_triggers::action_coverage(108), Coverage::Seam);
    assert!(w.notes.iter().any(|n| n.starts_with("S-0524 claim 4")));
    assert!(w.cells.cell(at(0, 0)).unwrap().overlay == glasnost_map::NO_OVERLAY);
}

// --- Claim 5 -------------------------------------------------------------------

// spec: S-0524 claim 5 (code 63: the number slot names a waypoint; five area damages of 100 with the demolition warhead, no attacker: 5 × 100 × 0.1 off a tower in reach of all five points)
#[test]
fn claim5_five_demolition_blasts() {
    let r = ruleset();
    let a = Action {
        number: 3,
        ..action(63)
    };
    let t = trigger("T", None, event(13, 0), vec![a]);
    let mut w = game(&r, triggers(vec![t], 0), &[(3, at(0, 0))], &[("Beta", "TOWER", at(0, 0))]);
    let tower = buildings(&w, "TOWER")[0];
    fire(&mut w);
    assert_eq!(health(&w, tower), TOWER_HP - 5 * 10);
}

// spec: S-0524 claim 5 (an absent waypoint marker or an invalid number: no effect and a warning, a safe departure)
#[test]
fn claim5_absent_marker_has_no_effect() {
    let r = ruleset();
    for n in [4, 702, -1] {
        let a = Action {
            number: n,
            ..action(63)
        };
        let t = trigger("T", None, event(13, 0), vec![a]);
        let mut w = game(&r, triggers(vec![t], 0), &[(3, at(0, 0))], &[("Beta", "TOWER", at(0, 0))]);
        let tower = buildings(&w, "TOWER")[0];
        fire(&mut w);
        assert_eq!(health(&w, tower), TOWER_HP);
        assert!(w.notes.iter().any(|n| n.starts_with("S-0524 claim 5")));
    }
}

// --- Claim 6 -------------------------------------------------------------------

fn flash_game(r: &Ruleset) -> World {
    let a = Action {
        number: 2,
        ..action(66)
    };
    let t = trigger("T", None, event(13, 0), vec![a]);
    let mut w = game(r, triggers(vec![t], 0), &[(2, at(1, 1))], &[("Beta", "TOWER", at(1, 1))]);
    w.effects.clear();
    fire(&mut w);
    w
}

// spec: S-0524 claim 6 and Formulas (a bright demolition warhead without a positive size: a flash at the waypoint point at level 63; no damage, no draw)
#[test]
fn claim6_fallback_flash_level() {
    let mut r = ruleset();
    demo(&mut r).bright = Some(true);
    let w = flash_game(&r);
    let p = mp_triggers::waypoint_point(&w, at(1, 1));
    assert_eq!(flashes(&w), vec![(63, Some(p))]);
    let tower = buildings(&w, "TOWER")[0];
    assert_eq!(health(&w, tower), TOWER_HP);
    // No draw: the same game without the action keeps the same random state.
    let mut quiet = r.clone();
    quiet.sim.general.c4_warhead = None;
    let v = flash_game(&quiet);
    assert!(flashes(&v).is_empty());
    assert_eq!(w.rng, v.rng);
}

// spec: S-0524 Formulas (a positive fraction f: level = trunc(63 × min(f, 1)), no lower bound of 21)
#[test]
fn claim6_fraction_flash_level() {
    for (f, level) in [("0.5", 31), ("0.1", 6), ("2", 63), ("1", 63)] {
        let mut r = ruleset();
        demo(&mut r).bright = Some(true);
        demo(&mut r).combat_light_size = Some(d(f));
        let w = flash_game(&r);
        assert_eq!(flashes(&w)[0].0, level, "fraction {f}");
    }
}

// spec: S-0524 claim 6 (no flash without the bright flag)
#[test]
fn claim6_no_flash_without_bright() {
    let w = flash_game(&ruleset());
    assert!(flashes(&w).is_empty());
}

// --- Claim 7 -------------------------------------------------------------------

// spec: S-0524 claim 7 (code 73: the number slot becomes base ambient in hundredths, and target ambient unless a weather storm is active)
#[test]
fn claim7_base_and_target_ambient() {
    let r = ruleset();
    let a = Action {
        number: 57,
        ..action(73)
    };
    let w = fired_game(&r, None, vec![a], &[]);
    assert_eq!(w.triggers.mp.ambient.base, Some(57));
    assert_eq!(w.triggers.mp.ambient.target, Some(57));
    // With a storm in force (raging or clearing), only the base changes.
    for s in [StormState::Raging, StormState::Clearing] {
        let mut w = game(&r, Triggers::default(), &[], &[]);
        w.sw.storm.state = s;
        mp_triggers::set_ambient(&mut w, 41);
        assert_eq!(w.triggers.mp.ambient.base, Some(41));
        assert_eq!(w.triggers.mp.ambient.target, None);
    }
}

// --- Claim 8 -------------------------------------------------------------------

// spec: S-0524 claim 8 (code 40: items four to seven become the local area's left, top, width and height; usable-area tests use the new area at once)
#[test]
fn claim8_resize_local_area() {
    let r = ruleset();
    let a = Action {
        items: [5, 3, 10, 8],
        ..action(40)
    };
    let edge = at(0, 15);
    let t = trigger("T", None, event(13, 0), vec![a]);
    let mut w = game(&r, triggers(vec![t], 0), &[], &[]);
    assert!(w.cells.usable(edge));
    assert!(w.cells.usable(at(0, 0)));
    fire(&mut w);
    assert_eq!(w.cells.footprint().local, [5, 3, 10, 8]);
    assert!(!w.cells.usable(edge));
    assert!(w.cells.usable(at(0, 0)));
    // The cells' categories follow: the excluded cell is no longer passable ground.
    let i = w.cells.idx(edge).unwrap();
    assert_eq!(
        w.cells.category[i],
        glasnost_sim::passability::compute_category(&w, edge)
    );
}

// --- Claim 9 -------------------------------------------------------------------

fn restore_game(r: &Ruleset) -> World {
    let t = triggers(vec![], 0);
    game(
        r,
        t,
        &[],
        &[("Alpha", "TOWER", at(0, 0)), ("Alpha", "TOWER", at(4, 0)), ("Beta", "TOWER", at(-4, 0))],
    )
}

// spec: S-0524 claim 9 (the record holds each house's pre-placed buildings by type and cell; buildings created during play are not added)
#[test]
fn claim9_starting_record() {
    let r = ruleset();
    let mut w = restore_game(&r);
    assert_eq!(w.triggers.mp.starting.len(), 3);
    let alpha: Vec<CellXY> = w
        .triggers
        .mp
        .starting
        .iter()
        .filter(|b| b.house == ALPHA)
        .map(|b| b.cell)
        .collect();
    assert_eq!(alpha, vec![at(0, 0), at(4, 0)]);
    testkit::spawn(&mut w, Kind::Building, "TOWER", 0, at(0, 4), 0);
    run(&mut w, 2);
    assert_eq!(w.triggers.mp.starting.len(), 3);
    // A later building is not restored: removing it and restoring changes nothing there.
    let later = *buildings(&w, "TOWER").iter().find(|&&b| w.obj(b).unwrap().pos.cell() == at(0, 4)).unwrap();
    glasnost_sim::objects::remove(&mut w, later);
    let v = country(&w, "Alphaland");
    mp_triggers::restore_starting_buildings(&mut w, v, None);
    assert!(buildings(&w, "TOWER").iter().all(|&b| w.obj(b).unwrap().pos.cell() != at(0, 4)));
}

// spec: S-0524 claim 9 (code 130 resolves the number slot as a house value; a present recorded building receives a healing hit of minus its strength and returns to full health; the other house's is untouched)
#[test]
fn claim9_heals_present_buildings() {
    let r = ruleset();
    let t = trigger("T", None, event(13, 1), vec![]);
    let mut w = game(
        &r,
        triggers(vec![t], 0),
        &[],
        &[("Alpha", "TOWER", at(0, 0)), ("Beta", "TOWER", at(-4, 0))],
    );
    let v = country(&w, "Alphaland");
    w.triggers.map.triggers[0].actions = vec![Action {
        number: v,
        ..action(130)
    }];
    let towers = buildings(&w, "TOWER");
    for &b in &towers {
        hit(&mut w, b, 300);
    }
    fire(&mut w);
    let mine = towers.iter().find(|&&b| w.obj(b).unwrap().owner == Some(ALPHA)).unwrap();
    let theirs = towers.iter().find(|&&b| w.obj(b).unwrap().owner == Some(BETA)).unwrap();
    assert_eq!(health(&w, *mine), TOWER_HP);
    assert_eq!(health(&w, *theirs), TOWER_HP - 300);
}

// spec: S-0524 claim 9 (an unresolved house value gives no effect; S-0011 claim 23's position values resolve through the start positions)
#[test]
fn claim9_house_values() {
    let r = ruleset();
    let mut w = restore_game(&r);
    let towers = buildings(&w, "TOWER");
    for &b in &towers {
        hit(&mut w, b, 300);
    }
    mp_triggers::restore_starting_buildings(&mut w, -1, None);
    assert!(towers.iter().all(|&b| health(&w, b) == TOWER_HP - 300));
    // Position B (4476) is the house at start position 1.
    w.houses[1].start = Some(1);
    assert_eq!(mp_triggers::resolve_house(&w, 4476, None), Some(BETA));
    assert_eq!(mp_triggers::resolve_house(&w, 4475, None), None);
    assert_eq!(mp_triggers::resolve_house(&w, 9013, Some(ALPHA)), Some(ALPHA));
    mp_triggers::restore_starting_buildings(&mut w, 4476, None);
    let theirs = towers.iter().find(|&&b| w.obj(b).unwrap().owner == Some(BETA)).unwrap();
    assert_eq!(health(&w, *theirs), TOWER_HP);
}

// spec: S-0524 claim 9 (a missing building is created at its cell for its house, with no recorded tag; a vehicle in its foundation is displaced to a nearby cell)
#[test]
fn claim9_replaces_missing_buildings_and_displaces() {
    let r = ruleset();
    let mut w = restore_game(&r);
    let gone = *buildings(&w, "TOWER").iter().find(|&&b| w.obj(b).unwrap().pos.cell() == at(4, 0)).unwrap();
    glasnost_sim::objects::remove(&mut w, gone);
    let cart = testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, at(4, 0), 0);
    let v = country(&w, "Alphaland");
    mp_triggers::restore_starting_buildings(&mut w, v, None);
    let new = *buildings(&w, "TOWER")
        .iter()
        .find(|&&b| w.obj(b).unwrap().pos.cell() == at(4, 0))
        .unwrap_or_else(|| panic!("a replacement: {:?}", w.notes));
    assert_eq!(w.obj(new).unwrap().owner, Some(ALPHA));
    assert!(!w.triggers.objects.contains_key(&new), "no recorded tag");
    let c = w.obj(cart).expect("the vehicle stays").pos.cell();
    assert_ne!(c, at(4, 0));
    assert!(w.obj(cart).unwrap().on_map);
}

// spec: S-0524 claim 9 (a same-type building of another house does not count: it is not healed, and the blocked replacement leaves nothing)
#[test]
fn claim9_wrong_owner_is_not_healed() {
    let r = ruleset();
    let mut w = restore_game(&r);
    let b = *buildings(&w, "TOWER").iter().find(|&&b| w.obj(b).unwrap().pos.cell() == at(0, 0)).unwrap();
    w.obj_mut(b).unwrap().owner = Some(BETA);
    hit(&mut w, b, 300);
    let before = buildings(&w, "TOWER").len();
    let v = country(&w, "Alphaland");
    mp_triggers::restore_starting_buildings(&mut w, v, None);
    assert_eq!(health(&w, b), TOWER_HP - 300);
    assert_eq!(buildings(&w, "TOWER").len(), before);
}

/// A restoration whose placement is blocked by another building, with a
/// displaced vehicle in the same cell.
fn failed_return(r: &Ruleset) -> (World, ObjectId) {
    let mut w = restore_game(r);
    let gone = *buildings(&w, "TOWER").iter().find(|&&b| w.obj(b).unwrap().pos.cell() == at(4, 0)).unwrap();
    glasnost_sim::objects::remove(&mut w, gone);
    testkit::spawn(&mut w, Kind::Building, "PLANT", 1, at(4, 0), 0);
    let cart = testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, at(4, 0), 0);
    let v = country(&w, "Alphaland");
    mp_triggers::restore_starting_buildings(&mut w, v, None);
    assert!(buildings(&w, "TOWER").iter().all(|&b| w.obj(b).unwrap().pos.cell() != at(4, 0)));
    (w, cart)
}

// spec: S-0524 claim 9 and Edge cases (quirks.restoration_failed_return_removes_unit on: a failed placement leaves no replacement and a unit that cannot return is removed)
#[test]
fn claim9_failed_return_removes() {
    let (w, cart) = failed_return(&ruleset());
    assert!(w.obj(cart).is_none_or(|o| !o.on_map));
}

// spec: S-0524 Edge cases (quirks.restoration_failed_return_removes_unit off: the unit is kept at a nearby cell)
#[test]
fn claim9_failed_return_kept_with_quirk_off() {
    let mut r = ruleset();
    r.globals.quirks.restoration_failed_return_removes_unit = false;
    let (w, cart) = failed_return(&r);
    let o = w.obj(cart).expect("kept");
    assert!(o.on_map);
    assert_ne!(o.pos.cell(), at(4, 0));
}

// --- Claims 10 to 12 -------------------------------------------------------------

/// Tags: 0 ENT (entered-by any house), 1 DIE (destroyed by anything,
/// carry-over as `carry`), 2 DIE2 (destroyed by anything, no carry-over);
/// all repeating, each setting a variable.
fn entry_game(carry: bool) -> World {
    let mut die = trigger("DIE", None, event(48, 0), vec![Action { number: 1, ..action(56) }]);
    die.carry_over = carry;
    let t = triggers(
        vec![
            trigger("ENT", None, event(1, -1), vec![Action { number: 0, ..action(56) }]),
            die,
            trigger("DIE2", None, event(48, 0), vec![Action { number: 2, ..action(56) }]),
        ],
        2,
    );
    game(&ruleset(), t, &[], &[])
}

fn tag(w: &mut World, id: ObjectId, name: &str) {
    glasnost_sim::triggers::object_tag(w, id, Some(name.as_bytes()));
}

fn tag_index(w: &World, name: &str) -> u32 {
    w.triggers.map.object_tag(name.as_bytes()).unwrap()
}

// spec: S-0524 claim 10 (capture: two entry raises, then carry-over of a flagged tag replacing the building's tag, ownership change, then one destroyed-by-anything raise on the engineer's tag before its removal; the replaced tag survives without carriers)
#[test]
fn claim10_capture_raises_and_carry_over() {
    let mut w = entry_game(true);
    let b = testkit::spawn(&mut w, Kind::Building, "TOWER", 1, at(0, 0), 0);
    let eng = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, at(1, 0), 0);
    tag(&mut w, b, "ENT");
    tag(&mut w, eng, "DIE");
    let f = w.triggers.fired;
    mp_triggers::capture_seam(&mut w, b, eng, |w| {
        w.obj_mut(b).unwrap().owner = Some(ALPHA);
    });
    assert_eq!(w.triggers.fired - f, 3, "two entry raises and one destruction raise");
    assert_eq!(w.triggers.objects.get(&b), Some(&tag_index(&w, "DIE")));
    assert_eq!(w.obj(b).unwrap().owner, Some(ALPHA));
    assert!(w.obj(eng).is_none_or(|o| !o.on_map));
    let old = w.triggers.tags.get(&tag_index(&w, "ENT")).expect("the old tag is not deleted");
    assert_eq!(old.carriers, 0);
    assert!(w.triggers.variables[0] && w.triggers.variables[1]);
}

// spec: S-0524 claim 10 (an engineer whose tag has no carry-over: still two entry raises and one destruction raise, no transfer)
#[test]
fn claim10_capture_without_carry_over() {
    let mut w = entry_game(false);
    let b = testkit::spawn(&mut w, Kind::Building, "TOWER", 1, at(0, 0), 0);
    let eng = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, at(1, 0), 0);
    tag(&mut w, b, "ENT");
    tag(&mut w, eng, "DIE2");
    let f = w.triggers.fired;
    mp_triggers::capture_seam(&mut w, b, eng, |_| {});
    assert_eq!(w.triggers.fired - f, 3);
    assert_eq!(w.triggers.objects.get(&b), Some(&tag_index(&w, "ENT")));
}

// spec: S-0524 claim 11 (hijack: one entry raise on the vehicle's tag, carry-over when flagged, takeover, and no destroyed-by-anything raise for the thief)
#[test]
fn claim11_hijack_raises_and_carry_over() {
    let mut w = entry_game(true);
    let v = testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, at(0, 0), 0);
    let thief = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, at(1, 0), 0);
    tag(&mut w, v, "ENT");
    tag(&mut w, thief, "DIE");
    let f = w.triggers.fired;
    mp_triggers::hijack_seam(&mut w, v, thief, |w| {
        w.obj_mut(v).unwrap().owner = Some(ALPHA);
    });
    assert_eq!(w.triggers.fired - f, 1, "the entry raise only");
    assert!(!w.triggers.variables[1], "no destruction raise for the thief");
    assert_eq!(w.triggers.objects.get(&v), Some(&tag_index(&w, "DIE")));
    assert_eq!(w.obj(v).unwrap().owner, Some(ALPHA));
    assert!(w.obj(thief).is_none_or(|o| !o.on_map));
}

// spec: S-0524 claim 12 (a consuming entry: the building's entry raise precedes the effect; the entrant's destroyed-by-anything raise follows it and precedes removal)
#[test]
fn claim12_consuming_entry_order() {
    let mut w = entry_game(false);
    let b = testkit::spawn(&mut w, Kind::Building, "TOWER", 1, at(0, 0), 0);
    let spy = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, at(1, 0), 0);
    tag(&mut w, b, "ENT");
    tag(&mut w, spy, "DIE2");
    let f = w.triggers.fired;
    let mut during = None;
    mp_triggers::consuming_entry_seam(&mut w, b, spy, |w| {
        during = Some((w.triggers.fired, w.triggers.variables[0], w.triggers.variables[2]));
    });
    assert_eq!(during, Some((f + 1, true, false)));
    assert_eq!(w.triggers.fired - f, 2);
    assert!(w.triggers.variables[2]);
    assert!(w.obj(spy).is_none_or(|o| !o.on_map));
}

// spec: S-0524 claim 12 (a retaining entry such as a garrison: the entry raise precedes the entry's effect; no raise for the retained entrant)
#[test]
fn claim12_retaining_entry() {
    let mut w = entry_game(false);
    let b = testkit::spawn(&mut w, Kind::Building, "TOWER", 1, at(0, 0), 0);
    let inf = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, at(1, 0), 0);
    tag(&mut w, b, "ENT");
    tag(&mut w, inf, "DIE2");
    let f = w.triggers.fired;
    let mut seen = false;
    mp_triggers::retaining_entry_seam(&mut w, b, inf, |w| seen = w.triggers.variables[0]);
    assert!(seen);
    assert_eq!(w.triggers.fired - f, 1);
    assert!(w.obj(inf).is_some_and(|o| o.on_map));
}

// --- Import mapping ------------------------------------------------------------

// spec: S-0524 I1 (the codes this spec adds are recognised; 108 is a seam)
#[test]
fn i1_action_coverage() {
    for c in [40, 63, 66, 73, 90, 95, 102, 130] {
        assert_eq!(mp_triggers::action_coverage(c), Coverage::Implemented, "{c}");
    }
    assert_eq!(mp_triggers::action_coverage(108), Coverage::Seam);
    assert_eq!(mp_triggers::event_coverage(1), Coverage::Implemented);
}

