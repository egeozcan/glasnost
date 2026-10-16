//! Claim tests of S-0524 (multiplayer trigger completion) on synthetic
//! rules (`tests/sw_common`) and synthetic maps with triggers and
//! waypoints. Every name and number is invented; no game data.

mod sw_common;

use std::sync::Arc;

use glasnost_map::objects::Placed;
use glasnost_map::triggers::{Action, Event, Owner, TagDef, TriggerDef, Triggers};
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
    r.sim
        .land_types
        .get_mut(&LandType::Clear)
        .unwrap()
        .buildable = Some(true);
    // A 2 × 2 building type for the covering-cell and displacement tests.
    r.sim.buildings.insert(
        "WIDE".into(),
        glasnost_rules::sim::BuildingType {
            strength: Some(500),
            foundation: Some([2, 2]),
            ..Default::default()
        },
    );
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("WIDE".into());
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
fn game(
    r: &Ruleset,
    tr: Triggers,
    wps: &[(u16, CellXY)],
    structures: &[(&str, &str, CellXY)],
) -> World {
    game_on(SIZE, r, tr, wps, structures)
}

/// [`game`] on a map of `size` (a small map has legal cells near the
/// illegal cell (0, 0)).
fn game_on(
    size: i32,
    r: &Ruleset,
    tr: Triggers,
    wps: &[(u16, CellXY)],
    structures: &[(&str, &str, CellXY)],
) -> World {
    let mut m = testkit::empty_map(size);
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
fn fired_game(
    r: &Ruleset,
    owner: Option<&str>,
    actions: Vec<Action>,
    wps: &[(u16, CellXY)],
) -> World {
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
    w.rules()
        .country_names
        .iter()
        .position(|c| c == name)
        .unwrap() as i32
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
    assert_eq!(
        w.sw.storm.centre,
        at(3, 0),
        "an active storm blocks the action"
    );
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
    let w = fired_game(
        &r,
        None,
        vec![a(0), a(0), a(1)],
        &[(0, at(0, 0)), (1, at(4, 0))],
    );
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

// spec: S-0524 claim 4 (code 108 creates a crate at the waypoint's cell through S-0520's trigger placement, with crates off too)
#[test]
fn claim4_crate_action_places_a_crate() {
    let r = ruleset();
    let a = Action {
        waypoint: 0,
        number: 20,
        ..action(108)
    };
    let w = fired_game(&r, None, vec![a], &[(0, at(0, 0))]);
    assert_eq!(mp_triggers::action_coverage(108), Coverage::Implemented);
    // The test rules name no crate overlay: the placement is unsafe data and has no effect (S-0524 Edge cases).
    assert!(w.cells.cell(at(0, 0)).unwrap().overlay == glasnost_map::NO_OVERLAY);
    assert!(w.crates.places.is_empty());
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
    let mut w = game(
        &r,
        triggers(vec![t], 0),
        &[(3, at(0, 0))],
        &[("Beta", "TOWER", at(0, 0))],
    );
    let tower = buildings(&w, "TOWER")[0];
    fire(&mut w);
    assert_eq!(health(&w, tower), TOWER_HP - 5 * 10);
}

// spec: S-0524 claim 5 (an invalid waypoint number follows S-0011 claim 33's departure: no effect and a warning)
#[test]
fn claim5_invalid_number_has_no_effect() {
    let r = ruleset();
    for n in [702, -1] {
        let a = Action {
            number: n,
            ..action(63)
        };
        let t = trigger("T", None, event(13, 0), vec![a]);
        let mut w = game(
            &r,
            triggers(vec![t], 0),
            &[(3, at(0, 0))],
            &[("Beta", "TOWER", at(0, 0))],
        );
        let tower = buildings(&w, "TOWER")[0];
        fire(&mut w);
        assert_eq!(health(&w, tower), TOWER_HP);
        assert!(w.notes.iter().any(|n| n.starts_with("S-0524 claim 5")));
    }
}

/// The side of the small map of the stand-in tests, and its legal cell
/// nearest to the illegal cell (0, 0).
const SMALL: i32 = 6;
const NEAR: (i32, i32) = (3, 4);

fn detonation_points(w: &World) -> Vec<(i32, i32, i32)> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Detonation { pos, .. } => Some((pos.x, pos.y, pos.z)),
            _ => None,
        })
        .collect()
}

// spec: S-0524 claim 5 (an undefined marker still requests all five damages, centred at (128, 128, 0) with the blank stand-in, not at a legal cell; area damage reaches legal targets within the spread; no early rejection; S-0009 claim 44, Q-0760)
#[test]
fn claim5_undefined_marker_acts_at_the_stand_in_point() {
    let mut r = ruleset();
    // A spread of 10 cells, no fall-off: the legal tower near the corner takes every blast.
    demo(&mut r).cell_spread = Some(d("10"));
    let a = Action {
        number: 4,
        ..action(63)
    };
    let t = trigger("T", None, event(13, 0), vec![a]);
    let near = CellXY::new(NEAR.0, NEAR.1);
    let mut w = game_on(
        SMALL,
        &r,
        triggers(vec![t], 0),
        &[(3, CellXY::new(NEAR.0, NEAR.1))],
        &[("Beta", "TOWER", near)],
    );
    assert!(
        w.cells.cell(CellXY::new(0, 0)).is_none(),
        "cell (0, 0) is never legal"
    );
    let tower = buildings(&w, "TOWER")[0];
    w.effects.clear();
    fire(&mut w);
    assert_eq!(
        detonation_points(&w),
        vec![
            (128, 128, 0),
            (213, 213, 0),
            (43, 213, 0),
            (213, 43, 0),
            (43, 43, 0)
        ],
        "the five points around the stand-in centre, in precedence order"
    );
    assert_eq!(health(&w, tower), TOWER_HP - 5 * 10);
    assert!(w.notes.iter().any(|n| n.starts_with("S-0524 claim 5")));
}

// spec: S-0524 claim 5 (an explicit non-legal waypoint cell is a defined marker: its own point, unlike the undefined marker's stand-in point)
#[test]
fn claim5_defined_marker_uses_its_own_point() {
    let r = ruleset();
    let a = Action {
        number: 3,
        ..action(63)
    };
    let t = trigger("T", None, event(13, 0), vec![a]);
    let mut w = game_on(
        SMALL,
        &r,
        triggers(vec![t], 0),
        &[(3, CellXY::new(0, 1))],
        &[],
    );
    w.effects.clear();
    fire(&mut w);
    let pts = detonation_points(&w);
    assert_eq!(pts.len(), 5);
    assert_eq!((pts[0].0, pts[0].1), (128, 256 + 128));
}

// --- Claim 6 -------------------------------------------------------------------

fn flash_game(r: &Ruleset) -> World {
    let a = Action {
        number: 2,
        ..action(66)
    };
    let t = trigger("T", None, event(13, 0), vec![a]);
    let mut w = game(
        r,
        triggers(vec![t], 0),
        &[(2, at(1, 1))],
        &[("Beta", "TOWER", at(1, 1))],
    );
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

// spec: S-0524 claim 6 (an undefined marker requests the flash at claim 5's stand-in point (128, 128, 0), same gates, no damage, no draw; a request is not a promise of visible pixels)
#[test]
fn claim6_undefined_marker_flashes_at_the_stand_in_point() {
    let mut r = ruleset();
    demo(&mut r).bright = Some(true);
    let a = Action {
        number: 9,
        ..action(66)
    };
    let t = trigger("T", None, event(13, 0), vec![a]);
    let mut w = game(
        &r,
        triggers(vec![t], 0),
        &[(2, at(1, 1))],
        &[("Beta", "TOWER", at(1, 1))],
    );
    w.effects.clear();
    fire(&mut w);
    assert_eq!(
        flashes(&w),
        vec![(63, Some(glasnost_sim::world::Pos::new(128, 128, 0)))]
    );
    let tower = buildings(&w, "TOWER")[0];
    assert_eq!(health(&w, tower), TOWER_HP);
    // The same gates: no flash without the bright flag.
    let mut quiet = r.clone();
    demo(&mut quiet).bright = Some(false);
    let t = trigger(
        "T",
        None,
        event(13, 0),
        vec![Action {
            number: 9,
            ..action(66)
        }],
    );
    let mut v = game(&quiet, triggers(vec![t], 0), &[], &[]);
    fire(&mut v);
    assert!(flashes(&v).is_empty());
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

// spec: S-0524 claim 8 (shrinking removes, relocates, stops or switches off nothing: buildings and units outside keep health, owner, order, power and firing; an excluded power source still contributes and an excluded building still counts for defeat; Q-0761)
#[test]
fn claim8_shrink_leaves_excluded_objects_alone() {
    let r = ruleset();
    let a = Action {
        items: [5, 3, 10, 8],
        ..action(40)
    };
    let edge = at(0, 6);
    let t = trigger("T", None, event(13, 1), vec![a]);
    // An armed tower and a power plant in the excluded strip, an enemy vehicle in the tower's reach.
    let mut w = game(
        &r,
        triggers(vec![t], 0),
        &[],
        &[("Alpha", "TOWER", edge), ("Alpha", "PLANT", at(1, 6))],
    );
    let tower = buildings(&w, "TOWER")[0];
    let plant = buildings(&w, "PLANT")[0];
    let target = testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, at(0, 2), 0);
    let walker = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, at(0, 8), 0);
    assert!(w.cells.usable(edge) && w.cells.usable(at(1, 6)) && w.cells.usable(at(0, 8)));
    run(&mut w, 2);
    let power_before = glasnost_sim::econ::power::totals(&w, ALPHA);
    assert_eq!(power_before.0, 100);
    fire(&mut w);
    assert!(!w.cells.usable(edge) && !w.cells.usable(at(1, 6)) && !w.cells.usable(at(0, 8)));
    assert!(w.cells.usable(at(0, 2)), "the target stays in the new area");
    // Nothing removed, moved or stopped; ownership and health unchanged.
    for (id, cell) in [(tower, edge), (plant, at(1, 6)), (walker, at(0, 8))] {
        let o = w.obj(id).expect("kept");
        assert!(o.on_map);
        assert_eq!(o.pos.cell(), cell);
        assert_eq!(o.owner, Some(ALPHA));
    }
    assert_eq!(health(&w, tower), TOWER_HP);
    // The excluded power source still contributes; the house is not defeated by exclusion alone.
    assert_eq!(glasnost_sim::econ::power::totals(&w, ALPHA), power_before);
    assert!(!glasnost_sim::econ::victory::defeated(&w, ALPHA));
    // The excluded armed tower keeps firing at the target in reach (ordinary gates).
    let before = health(&w, target);
    run(&mut w, 200);
    assert!(
        w.obj(target).is_none_or(|o| o.health < before),
        "the excluded tower still attacks"
    );
}

// spec: S-0524 claim 8 (an object outside the new area is not made untargetable by the action: an ordered attack on it from inside is still an attack and damages it; it is not moved in or removed to stay alive)
#[test]
fn claim8_unit_outside_is_still_a_target() {
    let r = ruleset();
    let a = Action {
        items: [5, 3, 10, 8],
        ..action(40)
    };
    let t = trigger("T", None, event(13, 0), vec![a]);
    let mut w = game(
        &r,
        triggers(vec![t], 0),
        &[],
        &[("Alpha", "TOWER", at(0, 2))],
    );
    let tower = buildings(&w, "TOWER")[0];
    let walker = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 1, at(0, 7), 0);
    fire(&mut w);
    assert!(!w.cells.usable(at(0, 7)));
    // Not moved in automatically, still alive.
    assert_eq!(w.obj(walker).unwrap().pos.cell(), at(0, 7));
    // An attack order from inside on the object outside.
    w.tick = w.tick.max(w.orders_blocked_through + 1);
    let issued = glasnost_sim::orders::issue(
        &mut w,
        ALPHA,
        glasnost_sim::orders::Order::Behaviour {
            actor: tower,
            behaviour: glasnost_sim::rules::Behaviour::Attack,
            attack_move: false,
            target: Some(glasnost_sim::world::Target::Object(walker)),
            dest: None,
        },
    );
    assert!(issued);
    let before = health(&w, walker);
    run(&mut w, 300);
    assert!(
        w.obj(walker).is_none_or(|o| o.health < before),
        "the object outside was attacked"
    );
}

// --- Claim 9 -------------------------------------------------------------------

fn restore_game(r: &Ruleset) -> World {
    let t = triggers(vec![], 0);
    game(
        r,
        t,
        &[],
        &[
            ("Alpha", "TOWER", at(0, 0)),
            ("Alpha", "TOWER", at(4, 0)),
            ("Beta", "TOWER", at(-4, 0)),
        ],
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
    let later = *buildings(&w, "TOWER")
        .iter()
        .find(|&&b| w.obj(b).unwrap().pos.cell() == at(0, 4))
        .unwrap();
    glasnost_sim::objects::remove(&mut w, later);
    let v = country(&w, "Alphaland");
    mp_triggers::restore_starting_buildings(&mut w, v, None);
    assert!(
        buildings(&w, "TOWER")
            .iter()
            .all(|&b| w.obj(b).unwrap().pos.cell() != at(0, 4))
    );
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
    let mine = towers
        .iter()
        .find(|&&b| w.obj(b).unwrap().owner == Some(ALPHA))
        .unwrap();
    let theirs = towers
        .iter()
        .find(|&&b| w.obj(b).unwrap().owner == Some(BETA))
        .unwrap();
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
    assert_eq!(
        mp_triggers::resolve_house(&w, 9013, Some(ALPHA)),
        Some(ALPHA)
    );
    mp_triggers::restore_starting_buildings(&mut w, 4476, None);
    let theirs = towers
        .iter()
        .find(|&&b| w.obj(b).unwrap().owner == Some(BETA))
        .unwrap();
    assert_eq!(health(&w, *theirs), TOWER_HP);
}

// spec: S-0524 claim 9 (a missing building is created at its cell for its house, with no recorded tag; a vehicle in its foundation is displaced to a nearby cell)
#[test]
fn claim9_replaces_missing_buildings_and_displaces() {
    let r = ruleset();
    let mut w = restore_game(&r);
    let gone = *buildings(&w, "TOWER")
        .iter()
        .find(|&&b| w.obj(b).unwrap().pos.cell() == at(4, 0))
        .unwrap();
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
    let b = *buildings(&w, "TOWER")
        .iter()
        .find(|&&b| w.obj(b).unwrap().pos.cell() == at(0, 0))
        .unwrap();
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
    let gone = *buildings(&w, "TOWER")
        .iter()
        .find(|&&b| w.obj(b).unwrap().pos.cell() == at(4, 0))
        .unwrap();
    glasnost_sim::objects::remove(&mut w, gone);
    testkit::spawn(&mut w, Kind::Building, "PLANT", 1, at(4, 0), 0);
    let cart = testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, at(4, 0), 0);
    let v = country(&w, "Alphaland");
    mp_triggers::restore_starting_buildings(&mut w, v, None);
    assert!(
        buildings(&w, "TOWER")
            .iter()
            .all(|&b| w.obj(b).unwrap().pos.cell() != at(4, 0))
    );
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

fn wide(w: &World) -> Vec<ObjectId> {
    buildings(w, "WIDE")
}

fn facing_of(w: &World, id: ObjectId) -> u16 {
    w.obj(id).unwrap().body.value(w.tick)
}

// spec: S-0524 claim 9 (the per-house restoration order is the surviving object creation order, oldest first, not type order or cell order; the record holds top-left cells; Q-0763)
#[test]
fn claim9_restoration_follows_creation_order() {
    let r = ruleset();
    // Creation order: PLANT, then TOWER; cell order: TOWER first; type order: TOWER first.
    let mut w = game(
        &r,
        triggers(vec![], 0),
        &[],
        &[("Alpha", "PLANT", at(5, 0)), ("Alpha", "TOWER", at(0, 0))],
    );
    let plant_ti = w.rules().index_of(Kind::Building, "PLANT").unwrap();
    let tower_ti = w.rules().index_of(Kind::Building, "TOWER").unwrap();
    assert!(tower_ti < plant_ti);
    let kinds: Vec<u32> = w
        .triggers
        .mp
        .starting
        .iter()
        .map(|b| b.type_index)
        .collect();
    assert_eq!(
        kinds,
        vec![plant_ti, tower_ti],
        "the record is in creation order"
    );
    for b in w.triggers.mp.starting.iter() {
        assert!(b.cell == at(5, 0) || b.cell == at(0, 0));
    }
    for id in buildings(&w, "PLANT")
        .into_iter()
        .chain(buildings(&w, "TOWER"))
    {
        glasnost_sim::objects::remove(&mut w, id);
    }
    let v = country(&w, "Alphaland");
    mp_triggers::restore_starting_buildings(&mut w, v, None);
    let (p, t) = (buildings(&w, "PLANT")[0], buildings(&w, "TOWER")[0]);
    assert!(p < t, "the older record is restored first");
}

// spec: S-0524 claim 9 (an action restores only its resolved house; records of different houses are not interleaved and a later action sees earlier results)
#[test]
fn claim9_each_action_restores_only_its_house() {
    let r = ruleset();
    let mut w = restore_game(&r);
    for id in buildings(&w, "TOWER") {
        glasnost_sim::objects::remove(&mut w, id);
    }
    let b = country(&w, "Betaland");
    mp_triggers::restore_starting_buildings(&mut w, b, None);
    let owners: Vec<_> = buildings(&w, "TOWER")
        .iter()
        .map(|&i| w.obj(i).unwrap().owner)
        .collect();
    assert_eq!(owners, vec![Some(BETA)], "only the selected house");
    let a = country(&w, "Alphaland");
    mp_triggers::restore_starting_buildings(&mut w, a, None);
    assert_eq!(buildings(&w, "TOWER").len(), 3);
}

// spec: S-0524 claim 9 (a recorded cell covered by any ground-layer foundation cell of a building of the recorded type and house is healed, though that building's own top-left cell differs; no second building is attempted; a wrong owner does not qualify; the replacement goes to the recorded top-left cell; Q-0763)
#[test]
fn claim9_covering_cell_heals_a_shifted_building() {
    let r = ruleset();
    let record = at(0, 0);
    let shifted = at(-1, -1); // its bottom-right foundation cell is the recorded top-left cell
    // Healed: same type and house, shifted by one cell.
    let mut w = game(&r, triggers(vec![], 0), &[], &[("Alpha", "WIDE", record)]);
    for id in wide(&w) {
        glasnost_sim::objects::remove(&mut w, id);
    }
    let sh = testkit::spawn(&mut w, Kind::Building, "WIDE", 0, shifted, 0);
    hit(&mut w, sh, 300);
    let a = country(&w, "Alphaland");
    mp_triggers::restore_starting_buildings(&mut w, a, None);
    assert_eq!(wide(&w), vec![sh], "no second building");
    assert_eq!(health(&w, sh), 500);
    // Control: a wrong owner does not qualify; the replacement is blocked by the existing foundation.
    let mut v = game(&r, triggers(vec![], 0), &[], &[("Alpha", "WIDE", record)]);
    for id in wide(&v) {
        glasnost_sim::objects::remove(&mut v, id);
    }
    let other = testkit::spawn(&mut v, Kind::Building, "WIDE", 1, shifted, 0);
    hit(&mut v, other, 300);
    mp_triggers::restore_starting_buildings(&mut v, a, None);
    assert_eq!(wide(&v), vec![other]);
    assert_eq!(health(&v, other), 200);
    // An empty-cell control: the replacement stands at the recorded top-left cell.
    let mut u = game(&r, triggers(vec![], 0), &[], &[("Alpha", "WIDE", record)]);
    for id in wide(&u) {
        glasnost_sim::objects::remove(&mut u, id);
    }
    mp_triggers::restore_starting_buildings(&mut u, a, None);
    let new = wide(&u);
    assert_eq!(new.len(), 1);
    assert_eq!(u.obj(new[0]).unwrap().pos.cell(), record);
}

// --- Claim 13 -------------------------------------------------------------------

// spec: S-0524 claim 13 (a missing starting building is a fresh object at full type strength, placed by ordinary during-play placement: construction queued, not the map-loading state; the power switch on; no recorded tag, production job or charge; Q-0764)
#[test]
fn claim13_replacement_is_a_fresh_building_under_construction() {
    let r = ruleset();
    let mut w = game(
        &r,
        triggers(vec![], 0),
        &[],
        &[("Alpha", "TOWER", at(0, 0))],
    );
    let old = buildings(&w, "TOWER")[0];
    // Map-loading placement: idle body, guard queued.
    assert_eq!(
        w.obj(old).unwrap().beh.queued,
        Some(glasnost_sim::rules::Behaviour::Guard)
    );
    hit(&mut w, old, 300);
    glasnost_sim::objects::remove(&mut w, old);
    let a = country(&w, "Alphaland");
    mp_triggers::restore_starting_buildings(&mut w, a, None);
    let new = buildings(&w, "TOWER")[0];
    assert_ne!(new, old);
    let o = w.obj(new).unwrap();
    assert_eq!(
        o.health, TOWER_HP,
        "full type strength, not the saved health"
    );
    assert_eq!(
        o.beh.queued,
        Some(glasnost_sim::rules::Behaviour::Construction)
    );
    assert!(o.econ.construction.building_up, "its build-up is under way");
    assert!(!o.econ.building.switched_off, "the power switch begins on");
    assert!(!w.triggers.objects.contains_key(&new), "no recorded tag");
    // The ordinary one-stage rules: with no build-up sprite it hands over to guard.
    run(&mut w, 10);
    let o = w.obj(new).unwrap();
    assert!(
        !o.econ.construction.building_up,
        "handed over: {:?} {:?}",
        o.beh.current, o.beh.queued
    );
}

// spec: S-0524 claim 13 (only ground-layer mobile occupants of the attempted foundation cells are lifted and re-placed, each by S-0212 claim 25's search from its old cell, facing 0; infantry are not packed into a shared cell; a unit in an adjacent non-foundation cell is untouched; Q-0764)
#[test]
fn claim13_displaced_occupants_are_placed_by_the_nearby_search() {
    let r = ruleset();
    let mut w = game(&r, triggers(vec![], 0), &[], &[("Alpha", "WIDE", at(0, 0))]);
    for id in wide(&w) {
        glasnost_sim::objects::remove(&mut w, id);
    }
    let foundation = [at(0, 0), at(1, 0), at(0, 1), at(1, 1)];
    let carts = [
        testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, at(0, 0), 64),
        testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, at(1, 1), 192),
    ];
    let men = [
        testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, at(1, 0), 64),
        testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, at(0, 1), 64),
    ];
    let outside = testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, at(2, 0), 128);
    let outside_before = (w.obj(outside).unwrap().pos, facing_of(&w, outside));
    let a = country(&w, "Alphaland");
    mp_triggers::restore_starting_buildings(&mut w, a, None);
    let new = wide(&w);
    assert_eq!(new.len(), 1);
    assert_eq!(w.obj(new[0]).unwrap().pos.cell(), at(0, 0));
    for id in carts.iter().chain(men.iter()) {
        let o = w.obj(*id).expect("re-placed");
        assert!(o.on_map);
        assert!(
            !foundation.contains(&o.pos.cell()),
            "not left under the new building"
        );
        assert_eq!(facing_of(&w, *id), 0, "facing 0");
        assert_eq!(o.layer, glasnost_sim::world::Layer::Ground);
    }
    // Infantry are not deliberately packed into one cell.
    assert_ne!(
        w.obj(men[0]).unwrap().pos.cell(),
        w.obj(men[1]).unwrap().pos.cell()
    );
    // The unit outside the foundation was not lifted.
    assert_eq!(
        (w.obj(outside).unwrap().pos, facing_of(&w, outside)),
        outside_before
    );
}

// spec: S-0524 claim 13 (a displaced unit whose search finds no cell is not left off the map: the placement at the stand-in cell fails and the unit is removed; Q-0764)
#[test]
fn claim13_no_nearby_cell_removes_the_unit() {
    let r = ruleset();
    // A map with 15 legal cells: the 2 × 2 record, and a blocker on every other legal cell.
    let size = 3;
    let fp = glasnost_map::Footprint::new(size, size, [0, 0, size, size]);
    let record = CellXY::new(2, 2);
    let inside: Vec<CellXY> = vec![
        record,
        CellXY::new(3, 2),
        CellXY::new(2, 3),
        CellXY::new(3, 3),
    ];
    assert!(inside.iter().all(|&c| fp.is_legal(c)));
    let mut blockers: Vec<(&str, &str, CellXY)> = Vec::new();
    for x in 0..8 {
        for y in 0..8 {
            let c = CellXY::new(x, y);
            if fp.is_legal(c) && !inside.contains(&c) {
                blockers.push(("Beta", "TOWER", c));
            }
        }
    }
    let mut structures = vec![("Alpha", "WIDE", record)];
    structures.extend(blockers);
    let mut w = game_on(size, &r, triggers(vec![], 0), &[], &structures);
    for id in wide(&w) {
        glasnost_sim::objects::remove(&mut w, id);
    }
    let cart = testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, record, 64);
    let a = country(&w, "Alphaland");
    mp_triggers::restore_starting_buildings(&mut w, a, None);
    assert_eq!(wide(&w).len(), 1, "the building was placed");
    assert!(
        w.obj(cart).is_none_or(|o| !o.on_map),
        "removed, not retained off the map or under the building"
    );
}

// spec: S-0524 claim 13 (when the building cannot be placed, earlier lifted occupants are offered their original points, facing 0, and stay; overlays removed are not restored; Q-0764)
#[test]
fn claim13_failed_placement_returns_occupants_facing_zero() {
    let r = ruleset();
    let mut w = game(&r, triggers(vec![], 0), &[], &[("Alpha", "WIDE", at(0, 0))]);
    for id in wide(&w) {
        glasnost_sim::objects::remove(&mut w, id);
    }
    // A building blocks one foundation cell; a vehicle stands in another.
    testkit::spawn(&mut w, Kind::Building, "PLANT", 1, at(1, 0), 0);
    let cart = testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, at(0, 1), 64);
    let before = w.obj(cart).unwrap().pos;
    let a = country(&w, "Alphaland");
    mp_triggers::restore_starting_buildings(&mut w, a, None);
    assert!(wide(&w).is_empty(), "no replacement remains");
    let o = w.obj(cart).expect("kept");
    assert!(o.on_map);
    assert_eq!(o.pos, before, "offered its original point");
    assert_eq!(facing_of(&w, cart), 0);
}

// --- Claims 10 to 12 -------------------------------------------------------------

/// Tags: 0 ENT (entered-by any house), 1 DIE (destroyed by anything,
/// carry-over as `carry`), 2 DIE2 (destroyed by anything, no carry-over);
/// all repeating, each setting a variable.
fn entry_game(carry: bool) -> World {
    let mut die = trigger(
        "DIE",
        None,
        event(48, 0),
        vec![Action {
            number: 1,
            ..action(56)
        }],
    );
    die.carry_over = carry;
    let t = triggers(
        vec![
            trigger(
                "ENT",
                None,
                event(1, -1),
                vec![Action {
                    number: 0,
                    ..action(56)
                }],
            ),
            die,
            trigger(
                "DIE2",
                None,
                event(48, 0),
                vec![Action {
                    number: 2,
                    ..action(56)
                }],
            ),
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
    assert_eq!(
        w.triggers.fired - f,
        3,
        "two entry raises and one destruction raise"
    );
    assert_eq!(w.triggers.objects.get(&b), Some(&tag_index(&w, "DIE")));
    assert_eq!(w.obj(b).unwrap().owner, Some(ALPHA));
    assert!(w.obj(eng).is_none_or(|o| !o.on_map));
    let old = w
        .triggers
        .tags
        .get(&tag_index(&w, "ENT"))
        .expect("the old tag is not deleted");
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
    assert!(
        !w.triggers.variables[1],
        "no destruction raise for the thief"
    );
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
        during = Some((
            w.triggers.fired,
            w.triggers.variables[0],
            w.triggers.variables[2],
        ));
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

/// Two repeating tags that fire on every raise (the any-event code): OLD
/// (variable 3) and NEW (variable 4, with the carry-over flag when `carry`).
/// Within one synchronous seam call nothing polls, so `fired` counts raises.
fn any_raise_game(carry: bool) -> World {
    let mut new = trigger(
        "NEW",
        None,
        event(8, 0),
        vec![Action {
            number: 4,
            ..action(56)
        }],
    );
    new.carry_over = carry;
    let t = triggers(
        vec![
            trigger(
                "OLD",
                None,
                event(8, 0),
                vec![Action {
                    number: 3,
                    ..action(56)
                }],
            ),
            new,
        ],
        2,
    );
    game(&ruleset(), t, &[], &[])
}

// spec: S-0524 claim 10 (capture's ownership change raises neither the replaced old tag nor the carried new tag, with or without carry-over; no relative order of carry-over and ownership change is required; Q-0765, C15)
#[test]
fn claim10_ownership_change_raises_no_tag() {
    for carry in [true, false] {
        let mut w = any_raise_game(carry);
        let b = testkit::spawn(&mut w, Kind::Building, "TOWER", 1, at(0, 0), 0);
        let eng = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, at(1, 0), 0);
        tag(&mut w, b, "OLD");
        tag(&mut w, eng, "NEW");
        let f = w.triggers.fired;
        let mut during_takeover = None;
        mp_triggers::capture_seam(&mut w, b, eng, |w| {
            let before = w.triggers.fired;
            let old = w.obj(b).unwrap().owner;
            w.obj_mut(b).unwrap().owner = Some(ALPHA);
            glasnost_sim::econ::events::owner_changed(w, b, old);
            during_takeover = Some(w.triggers.fired - before);
        });
        assert_eq!(
            during_takeover,
            Some(0),
            "carry {carry}: the ownership change raises nothing"
        );
        // Two entry raises on the old tag, then the engineer's own destruction raise.
        assert_eq!(w.triggers.fired - f, 3, "carry {carry}");
        assert_eq!(w.obj(b).unwrap().owner, Some(ALPHA));
    }
}

// spec: S-0524 claim 11 (a vehicle-like building's entry: its entry raise precedes the takeover effects; its entering unit gets a destroyed-by-anything raise on the resulting shared tag when carry-over applies; no relative order of carry-over and the ownership change is required)
#[test]
fn claim11_vehicle_like_building_entry() {
    let mut w = entry_game(true);
    let b = testkit::spawn(&mut w, Kind::Building, "TOWER", 1, at(0, 0), 0);
    let unit = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, at(1, 0), 0);
    tag(&mut w, b, "ENT");
    tag(&mut w, unit, "DIE");
    let f = w.triggers.fired;
    let mut entry_before_takeover = false;
    mp_triggers::vehicle_like_entry_seam(&mut w, b, unit, |w| {
        entry_before_takeover = w.triggers.variables[0];
        w.obj_mut(b).unwrap().owner = Some(ALPHA);
    });
    assert!(
        entry_before_takeover,
        "the entry raise precedes the takeover effects"
    );
    assert_eq!(
        w.triggers.fired - f,
        2,
        "one entry raise and the unit's destruction raise"
    );
    assert!(
        w.triggers.variables[1],
        "the unit's destruction raise acts on the shared tag"
    );
    assert_eq!(w.triggers.objects.get(&b), Some(&tag_index(&w, "DIE")));
    assert!(w.obj(unit).is_none_or(|o| !o.on_map));
}

// spec: S-0524 claim 12 (a recycled or otherwise consumed entrant gets one own-tag destroyed-by-anything raise after the effect and before removal; its passengers get none merely for being removed with it; S-0603 claims 6 and 10)
#[test]
fn claim12_consumed_entrants_passengers_get_no_raise() {
    let mut w = entry_game(false);
    let building = testkit::spawn(&mut w, Kind::Building, "TOWER", 1, at(0, 0), 0);
    let carrier = testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, at(1, 0), 0);
    let rider = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, at(2, 0), 0);
    tag(&mut w, building, "ENT");
    tag(&mut w, carrier, "DIE2");
    tag(&mut w, rider, "DIE");
    // The rider rides the carrier (a passenger order of one).
    glasnost_sim::transport::board(&mut w, rider, carrier);
    let f = w.triggers.fired;
    mp_triggers::consuming_entry_seam(&mut w, building, carrier, |_| {});
    assert_eq!(
        w.triggers.fired - f,
        2,
        "the entry raise and the entrant's own raise"
    );
    assert!(w.triggers.variables[2], "the entrant's own raise");
    assert!(
        !w.triggers.variables[1],
        "no raise for the removed passenger"
    );
}

// --- S-0513 claim 22: the waypoint curtain (code 109; Q-0734) ------------------

// spec: S-0513 claim 22 and Import mapping (code 109 applies invulnerability at its waypoint-slot cell: the ground layer of the effect block, no house, the imported curtain duration, the number slot unused; infantry die; the invoke animation at the terrain surface plus 5 leptons, no radar event, no announcement)
#[test]
fn s0513_claim22_waypoint_curtain_action() {
    let r = ruleset();
    let run_with = |number: i32| {
        let a = Action {
            waypoint: 0,
            number,
            ..action(109)
        };
        let t = trigger("T", None, event(13, 0), vec![a]);
        let mut w = game(&r, triggers(vec![t], 0), &[(0, at(0, 0))], &[]);
        let cart = testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, at(0, 1), 0);
        let walker = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 1, at(1, 0), 0);
        let far = testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, at(5, 5), 0);
        w.effects.clear();
        fire(&mut w);
        (w, cart, walker, far)
    };
    let (w, cart, walker, far) = run_with(0);
    assert!(glasnost_sim::superweapons::protected(&w, cart));
    assert!(!glasnost_sim::superweapons::protected(&w, far));
    assert!(
        w.obj(walker).is_none_or(|o| o.health <= 0 || !o.on_map),
        "infantry die"
    );
    // Protected for the imported curtain duration.
    let p = w.obj(cart).unwrap().sw.protection.unwrap();
    assert_eq!(p.duration, sw_common::CURTAIN_D);
    // No radar event, no announcement.
    assert!(!w.effects.iter().any(|e| matches!(
        e,
        Effect::Superweapon {
            event: glasnost_sim::superweapons::SwEvent::Radar
                | glasnost_sim::superweapons::SwEvent::Launch,
            ..
        }
    )));
    // The invoke animation at the terrain surface plus 5 leptons (no bridge lift on this flat map).
    let surface = w.surface_at(at(0, 0).x * 256 + 128, at(0, 0).y * 256 + 128);
    assert!(w.effects.iter().any(|e| matches!(
        e,
        Effect::Anim { name, pos, .. } if name == "CURTA" && pos.z == surface + 5
    )));
    // The number slot changes nothing.
    let (w2, cart2, _, _) = run_with(77);
    assert_eq!(w.rng, w2.rng);
    assert!(glasnost_sim::superweapons::protected(&w2, cart2));
    assert_eq!(mp_triggers::action_coverage(109), Coverage::Implemented);
}

// --- Import mapping ------------------------------------------------------------

// spec: S-0524 I1 (the codes this spec adds are recognised and take effect; 108 places a crate, S-0520)
#[test]
fn i1_action_coverage() {
    for c in [40, 63, 66, 73, 90, 95, 102, 108, 130] {
        assert_eq!(
            mp_triggers::action_coverage(c),
            Coverage::Implemented,
            "{c}"
        );
    }
    assert_eq!(mp_triggers::event_coverage(1), Coverage::Implemented);
}

// --- Claim 15 (Q-0840 answered): a restored replacement's construction readiness -------------

/// What a restored replacement's construction does, measured on the way.
#[derive(Debug)]
struct Restored {
    /// The tick of the restoration (before the tick's turn pass).
    entry: i32,
    /// The ready mark at entry.
    mark_at_entry: bool,
    /// The first tick at the end of which *construction* is current.
    current: Option<i32>,
    /// The first construction processing.
    first: Option<i32>,
    /// The first tick at the end of which the construction state is over.
    handover: Option<i32>,
}

/// Restores a removed starting TOWER of Alphaland on a game whose TOWER type
/// has `frames` build-up image frames (`None`: no build-up sprite), then
/// runs 300 ticks recording the ready mark, the start and the handover.
fn restored_construction(mut r: Ruleset, frames: Option<i32>) -> Restored {
    {
        let t = r.sim.buildings.get_mut("TOWER").unwrap();
        t.econ.buildup = frames.map(|_| true);
        t.econ.buildup_frames = frames;
    }
    let mut w = game(
        &r,
        triggers(vec![], 0),
        &[],
        &[("Alpha", "TOWER", at(0, 0))],
    );
    // Past the first ticks, so that no start-up pass is mixed in.
    run(&mut w, 6);
    let old = buildings(&w, "TOWER")[0];
    glasnost_sim::objects::remove(&mut w, old);
    let a = country(&w, "Alphaland");
    mp_triggers::restore_starting_buildings(&mut w, a, None);
    let new = buildings(&w, "TOWER")[0];
    let mut out = Restored {
        entry: w.tick,
        mark_at_entry: w.obj(new).unwrap().beh.ready_mark,
        current: None,
        first: None,
        handover: None,
    };
    for _ in 0..300 {
        let t = w.tick;
        step(&mut w);
        let o = w.obj(new).unwrap();
        if out.current.is_none()
            && o.beh.current == Some(glasnost_sim::rules::Behaviour::Construction)
        {
            out.current = Some(t);
        }
        out.first = out.first.or(o.econ.construction.first);
        if out.handover.is_none() && !o.econ.construction.building_up {
            out.handover = Some(t);
            break;
        }
    }
    out
}

// spec: S-0524 claim 15 (a restored replacement enters with its ready mark clear; a positive-delay range of at least 2 stages sets the mark when an eligible building turn first lands on the last stage; construction becomes current at that turn's second per-tick start, its first processing is on the next tick, and the handover follows F6; the build-up runs twice, as for a placed building) and S-0406 claim 17 (restored replacement, positive delay: C = R + 1, H follows F6)
#[test]
fn claim15_restored_multi_stage_range_runs_twice_and_hands_over_by_f6() {
    // 10 image frames: n = 5 stages, d_b = 9 ticks (S-0108 claim 5), the range of 36 ticks.
    let k = restored_construction(ruleset(), Some(10));
    assert!(!k.mark_at_entry, "the mark is clear at entry");
    // The mark is set, and consumed by the start, in the turn that first lands on the last stage: construction is current at the end of it.
    let r = k.current.expect("construction becomes current");
    assert_eq!(r, k.entry + 36, "R follows entry by the range: {k:?}");
    assert_eq!(k.first, Some(r + 1), "C = R + 1");
    // The second run restarts the range at stage 0: the handover is a range later.
    assert_eq!(k.handover, Some(r + 1 + 36), "H = C + (n - 1) x d_b: {k:?}");
}

// spec: S-0524 claim 15 (a zero-delay range, including the ordinary no-build-up-sprite range: the mark is still clear at entry; the first eligible building turn sets it without advancing the stage and starts queued construction at its second per-tick start; construction is first processed on the next tick and hands over on the following tick; missing art does not make restoration an immediately completed building or a deployed entry) and S-0406 claim 17 (U the actual first eligible turn: C = U + 1, H = U + 2)
#[test]
fn claim15_restored_zero_delay_range_sets_the_mark_at_its_first_eligible_turn() {
    let k = restored_construction(ruleset(), None);
    assert!(
        !k.mark_at_entry,
        "the mark is clear at entry, missing art included"
    );
    let u = k
        .current
        .expect("the first eligible building turn starts construction");
    assert!(u >= k.entry, "{k:?}");
    assert_eq!(k.first, Some(u + 1), "C = U + 1");
    assert_eq!(k.handover, Some(u + 2), "H = U + 2");
}

// spec: S-0524 claim 15 (the shared quirks.placed_buildup_twice applies to restored replacements too: off sets the mark at entry and uses the deployed-style start) and S-0406 claim 17 and Edge cases (off: C = U + 1 and the handover follows F6; U the first eligible building turn)
#[test]
fn claim15_restored_replacement_follows_the_placed_buildup_twice_switch() {
    let mut r = ruleset();
    r.globals.quirks.placed_buildup_twice = false;
    let k = restored_construction(r, Some(10));
    assert!(k.mark_at_entry, "off: the mark is set at entry");
    let u = k
        .current
        .expect("construction becomes current at the first turn");
    assert_eq!(k.first, Some(u + 1), "C = U + 1");
    assert_eq!(k.handover, Some(u + 1 + 36), "H follows F6");
}

// spec: S-0524 claim 15 (a positive delay with 1 stage or fewer: the range does not report completion and the placed-building non-completion rule of S-0406 claim 18 applies) and S-0406 claim 18 (applies to restored replacements as to placed buildings)
#[test]
fn claim15_restored_single_stage_range_never_completes() {
    for frames in [3, 1] {
        let k = restored_construction(ruleset(), Some(frames));
        assert!(!k.mark_at_entry);
        assert_eq!(
            k.current, None,
            "frames {frames}: the range reports no ready mark"
        );
        assert_eq!(
            k.handover, None,
            "frames {frames}: construction never completes"
        );
    }
}

// spec: S-0524 claim 15 (healing an existing building under claim 9 does not restart its construction or change its ready mark)
#[test]
fn claim15_healing_an_existing_building_does_not_restart_construction() {
    let r = ruleset();
    let mut w = game(
        &r,
        triggers(vec![], 0),
        &[],
        &[("Alpha", "TOWER", at(0, 0))],
    );
    run(&mut w, 10);
    let b = buildings(&w, "TOWER")[0];
    hit(&mut w, b, 300);
    let (mark, entry) = {
        let o = w.obj(b).unwrap();
        (o.beh.ready_mark, o.econ.construction.entry)
    };
    let a = country(&w, "Alphaland");
    mp_triggers::restore_starting_buildings(&mut w, a, None);
    assert_eq!(buildings(&w, "TOWER"), vec![b], "the same building");
    let o = w.obj(b).unwrap();
    assert_eq!(o.health, TOWER_HP, "healed");
    assert!(!o.econ.construction.building_up, "no construction cycle");
    assert_eq!(o.beh.ready_mark, mark, "its ready mark is unchanged");
    assert_eq!(o.econ.construction.entry, entry, "no fresh entry");
}
