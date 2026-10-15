//! Claim tests of S-0504 (naval units: the ship locomotor, wakes, sinking,
//! underwater vessels and docking at yards) on a synthetic water map with
//! invented types (every name and number here is made up; no game data).

mod common;

use common::{Scene, cell, code_tile};
use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BuildingType, Decimal, LandRow, LandType, LocomotionClass, Locomotor, MovementZone, UnitType,
    WeaponType,
};
use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::combat::damage::{self, Hit, HitResult};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::econ::TypeRef;
use glasnost_sim::naval;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, ClickOn, Modifiers};
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};
use glasnost_sim::{save, testkit, tick};

const ALPHA: HouseId = HouseId(0);
/// The terrain code of a water tile in the scene's catalogue (S-0211 Terms).
const WATER_CODE: u8 = 9;
const WAKE: &str = "Ripple";

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

fn owners() -> Option<Vec<String>> {
    Some(vec!["Alphaland".into(), "Betaland".into()])
}

/// A land row with one multiplier for every class but the float classes.
fn land_row(ground: &str, float: &str) -> LandRow {
    LandRow {
        foot: dec(ground),
        track: dec(ground),
        wheel: dec(ground),
        hover: dec("1"),
        float: dec(float),
        amphibious: dec("1"),
        float_beach: dec(float),
        buildable: None,
    }
}

/// A ship type: the ship locomotor, the float class, the water zone, the
/// naval flag, the gun, and a weight above the default sinking weight.
fn ship() -> UnitType {
    let mut t = UnitType {
        strength: Some(300),
        speed: Some(6),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Float),
        locomotor: Some(Locomotor::Ship),
        movement_zone: Some(MovementZone::Water),
        primary: Some("Gun".into()),
        weight: dec("4"),
        tech_level: Some(1),
        owner: owners(),
        cost: Some(100),
        ..Default::default()
    };
    t.combat.naval = Some(true);
    t.combat.armor = Some(glasnost_rules::sim::ArmourClass::Light);
    t.combat.primary_fire_flh = Some([90, 0, 60]);
    t
}

/// The naval ruleset: the testkit's combat ruleset (CART, BUGGY, Gun, HE and
/// the rest) plus
/// - a water land row passable only to the float and amphibious classes,
///   ground rows impassable to the float class;
/// - ships SKIFF (surface), LIGHT (weight below the sinking weight),
///   DIVER (underwater, cloakable), SCOW (no weight);
/// - HOVER (the hovering locomotor), AMPHI (amphibious zone);
/// - buildings NAVYARD (naval vehicle factory with the repair flag) and
///   DEPOT (land, repair flag);
/// - the game-wide wake animation `Ripple`.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let push = |r: &mut Ruleset, reg: &str, n: &str| {
        r.registries.entry(reg.into()).or_default().push(n.into());
    };
    let s = &mut r.sim;
    s.land_types.insert(LandType::Clear, land_row("1", "0"));
    s.land_types.insert(LandType::Road, land_row("1", "0"));
    s.land_types.insert(LandType::Water, land_row("0", "1"));
    s.general.wake = Some(WAKE.into());
    let mut light = ship();
    light.weight = dec("2");
    let mut diver = ship();
    diver.combat.underwater = Some(true);
    diver.combat.cloakable = Some(true);
    let mut scow = ship();
    scow.weight = None;
    let hover = UnitType {
        strength: Some(100),
        speed: Some(6),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Hover),
        locomotor: Some(Locomotor::Hover),
        ..Default::default()
    };
    let amphi = UnitType {
        movement_zone: Some(MovementZone::Amphibious),
        locomotion_class: Some(LocomotionClass::Amphibious),
        ..hover.clone()
    };
    for (n, u) in [
        ("SKIFF", ship()),
        ("LIGHT", light),
        ("DIVER", diver),
        ("SCOW", scow),
        ("HOVER", hover),
        ("AMPHI", amphi),
    ] {
        r.sim.vehicles.insert(n.into(), u);
        push(&mut r, "vehicles", n);
    }
    let yard = BuildingType {
        strength: Some(500),
        foundation: Some([2, 2]),
        factory: Some("vehicle".into()),
        naval: Some(true),
        unit_repair: Some(true),
        tech_level: Some(1),
        owner: owners(),
        cost: Some(1000),
        ..Default::default()
    };
    let depot = BuildingType {
        naval: None,
        factory: None,
        ..yard.clone()
    };
    r.sim.buildings.insert("NAVYARD".into(), yard);
    r.sim.buildings.insert("DEPOT".into(), depot);
    push(&mut r, "buildings", "NAVYARD");
    push(&mut r, "buildings", "DEPOT");
    r
}

/// Whether a cell offset lies in the lake: dx in −8 … 10, dy in −6 … 6,
/// less a land peninsula at dx = 3 for dy −6 … 2.
fn lake(dx: i32, dy: i32) -> bool {
    let inside = (-8..=10).contains(&dx) && (-6..=6).contains(&dy);
    let peninsula = dx == 3 && dy <= 2;
    inside && !peninsula
}

/// The water scene: a flat clear map with the lake of [`lake`] in water tiles.
fn scene(r: Ruleset) -> Scene {
    let mut s = Scene::new();
    s.ruleset = r;
    for dx in -10..=12 {
        for dy in -8..=8 {
            if lake(dx, dy) {
                s.set_cell(cell(dx, dy), code_tile(WATER_CODE), 0);
            }
        }
    }
    s
}

fn world(r: Ruleset) -> World {
    scene(r).build()
}

fn at(w: &mut World, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    let kind = if w.rules().index_of(Kind::Vehicle, name).is_some() {
        Kind::Vehicle
    } else {
        Kind::Building
    };
    testkit::spawn(w, kind, name, house, cell(dx, dy), 0)
}

fn open(w: &mut World) {
    while w.tick < 4 {
        tick::step(w);
    }
}

fn land_of(w: &World, id: ObjectId) -> u8 {
    let c = w.obj(id).unwrap().pos.cell();
    w.cells.cell(c).unwrap().land
}

fn wakes(w: &World) -> Vec<Pos> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Anim { name, pos } if name == WAKE => Some(*pos),
            _ => None,
        })
        .collect()
}

fn lethal() -> Hit {
    Hit {
        amount: 100_000,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: true,
    }
}

fn click_attack(w: &mut World, actor: ObjectId, target: ObjectId) {
    let r = targeting::click(
        w,
        ALPHA,
        &[actor],
        ClickOn::Object(target),
        Modifiers::default(),
    );
    assert!(!r.orders.is_empty(), "the click gives an attack order");
    for o in r.orders {
        assert!(orders::issue(w, ALPHA, o));
    }
}

fn click_move(w: &mut World, actor: ObjectId, c: CellXY) {
    let r = targeting::click(w, ALPHA, &[actor], ClickOn::Cell(c), Modifiers::default());
    assert!(!r.orders.is_empty(), "the click gives a move order");
    for o in r.orders {
        assert!(orders::issue(w, ALPHA, o));
    }
}

fn stopped(w: &World, id: ObjectId) -> bool {
    let o = w.obj(id).unwrap();
    o.dest.is_none() && !glasnost_sim::occupancy::in_progress(w, id)
}

// spec: S-0504 claim 1 (a ship moves by the driving rules: it carries the driving state, travels along water around land, stops at its destination, and never enters land)
#[test]
fn claim_1_ship_moves_on_water_around_land() {
    let mut w = world(rules());
    let skiff = at(&mut w, "SKIFF", 0, 0, 0);
    assert!(
        w.obj(skiff).unwrap().drive.is_some(),
        "a ship carries the driving state"
    );
    open(&mut w);
    let goal = cell(6, 0);
    click_move(&mut w, skiff, goal);
    let start = w.obj(skiff).unwrap().pos;
    let mut moved = false;
    for _ in 0..1500 {
        tick::step(&mut w);
        assert_eq!(land_of(&w, skiff), land::WATER, "a ship never enters land");
        moved |= w.obj(skiff).unwrap().pos != start;
        if stopped(&w, skiff) {
            break;
        }
    }
    assert!(moved);
    assert!(stopped(&w, skiff), "it stops");
    let c = w.obj(skiff).unwrap().pos.cell();
    assert!(
        (c.x - goal.x).abs() <= 1 && (c.y - goal.y).abs() <= 1,
        "it arrives at its destination: {c:?}"
    );
}

// spec: S-0504 claim 1 (a destination on land: the ship does not leave the water)
#[test]
fn claim_1_ship_never_enters_land() {
    let mut w = world(rules());
    let skiff = at(&mut w, "SKIFF", 0, 1, 0);
    open(&mut w);
    // The peninsula, then the shore beyond the lake.
    for goal in [cell(3, 0), cell(14, 0)] {
        let _ = orders::issue(&mut w, ALPHA, common::move_to(skiff, goal));
        for _ in 0..400 {
            tick::step(&mut w);
            assert_eq!(land_of(&w, skiff), land::WATER, "goal {goal:?}");
        }
    }
}

// spec: S-0504 claim 2 (a moving surface ship makes one wake at its post-movement position on tick numbers divisible by 8; none while standing; an underwater type makes none)
#[test]
fn claim_2_ship_wakes() {
    let run = |name: &str| {
        let mut w = world(rules());
        let id = at(&mut w, name, 0, -6, 4);
        open(&mut w);
        let mut ticks = Vec::new();
        let mut idle = 0;
        for _ in 0..40 {
            w.effects.clear();
            tick::step(&mut w);
            for p in wakes(&w) {
                assert_eq!(p, w.obj(id).unwrap().pos, "at its own position");
                ticks.push(w.tick - 1);
            }
        }
        idle += ticks.len();
        click_move(&mut w, id, cell(8, 4));
        let mut moving = Vec::new();
        for _ in 0..80 {
            w.effects.clear();
            let t = w.tick;
            tick::step(&mut w);
            for p in wakes(&w) {
                assert_eq!(p, w.obj(id).unwrap().pos, "at its post-movement position");
                moving.push(t);
            }
        }
        (idle, moving)
    };
    let (idle, moving) = run("SKIFF");
    assert_eq!(idle, 0, "a standing ship makes no wake");
    assert!(moving.len() >= 5, "{moving:?}");
    assert!(moving.iter().all(|t| t % 8 == 0), "{moving:?}");
    let (_, under) = run("DIVER");
    assert!(
        under.is_empty(),
        "an underwater type makes no wakes: {under:?}"
    );
}

// spec: S-0504 claim 2 (no wake animation defined: a ship makes no wakes)
#[test]
fn claim_2_no_wake_animation() {
    let mut r = rules();
    r.sim.general.wake = None;
    let mut w = world(r);
    let id = at(&mut w, "SKIFF", 0, -6, 4);
    open(&mut w);
    click_move(&mut w, id, cell(8, 4));
    for _ in 0..80 {
        w.effects.clear();
        tick::step(&mut w);
        assert!(!w.effects.iter().any(|e| matches!(e, Effect::Anim { .. })));
    }
}

/// Kills `id` with a lethal hit between ticks.
fn kill(w: &mut World, id: ObjectId) -> HitResult {
    damage::apply(w, id, lethal())
}

// spec: S-0504 claim 4 and F1, F2; S-0312 claim 10 (a killed heavy surface vessel on water sinks: 1 health point, in play; from height 0 it descends 5 leptons per own update, leaves play at its 81st descent, and makes 20 wakes at the water surface within 170 leptons per axis)
#[test]
fn claim_4_sinking_descent_and_removal() {
    let mut w = world(rules());
    let skiff = at(&mut w, "SKIFF", 0, 0, 0);
    open(&mut w);
    let p0 = w.obj(skiff).unwrap().pos;
    assert_eq!(w.height_above_surface(skiff), 0);
    assert_eq!(kill(&mut w, skiff), HitResult::NowDead);
    let o = w.obj(skiff).expect("a sinking vessel stays in play");
    assert_eq!(o.health, 1);
    assert!(naval::sinking(&w, skiff));
    assert!(!damage::dead(o));
    let mut descents = 0;
    let mut wake_count = 0;
    loop {
        w.effects.clear();
        let t = w.tick;
        tick::step(&mut w);
        descents += 1;
        let ws = wakes(&w);
        let Some(o) = w.obj(skiff) else {
            assert!(ws.is_empty(), "no draws or wake on the removal update");
            break;
        };
        assert_eq!(o.pos.z, p0.z - 5 * descents, "5 leptons per update");
        assert_eq!((o.pos.x, o.pos.y), (p0.x, p0.y), "horizontal position kept");
        if t % 4 == 0 {
            assert_eq!(ws.len(), 1, "tick {t}");
            let p = ws[0];
            assert!((p.x - p0.x).abs() <= 170 && (p.y - p0.y).abs() <= 170);
            assert_eq!(p.z, w.surface_at(p0.x, p0.y), "at the water surface");
        } else {
            assert!(ws.is_empty(), "tick {t}");
        }
        wake_count += ws.len();
        assert!(descents < 200);
    }
    assert_eq!(descents, 81, "F1: n = ⌊(0 + 400) / 5⌋ + 1");
    assert_eq!(wake_count, 20);
}

// spec: S-0504 claim 4 and F2 (two ranged draws per wake tick, none on other ticks or on the removal update; with no wake animation the draws are still taken)
#[test]
fn claim_4_sinking_draws() {
    let mut w = world(rules());
    let skiff = at(&mut w, "SKIFF", 0, 0, 0);
    open(&mut w);
    kill(&mut w, skiff);
    for tick_no in [8, 9, 10, 11, 12] {
        let mut a = w.clone();
        a.tick = tick_no;
        let before = a.rng.taken();
        assert!(!naval::sinking_turn(&mut a, skiff));
        let n = a.rng.taken() - before;
        assert_eq!(n, if tick_no % 4 == 0 { 2 } else { 0 }, "tick {tick_no}");
    }
    // Below −400 leptons: removal, no draws.
    let mut a = w.clone();
    a.tick = 8;
    a.obj_mut(skiff).unwrap().pos.z = -396;
    let before = a.rng.taken();
    assert!(naval::sinking_turn(&mut a, skiff));
    assert!(a.obj(skiff).is_none());
    assert_eq!(a.rng.taken(), before);
    // Exactly −400 after the descent stays in play (strictly below).
    let mut a = w.clone();
    a.tick = 9;
    a.obj_mut(skiff).unwrap().pos.z = -395;
    assert!(!naval::sinking_turn(&mut a, skiff));
    assert_eq!(a.obj(skiff).unwrap().pos.z, -400);
    // No wake animation: the same draws, no wake.
    let mut r = rules();
    r.sim.general.wake = None;
    let mut b = world(r);
    let s2 = at(&mut b, "SKIFF", 0, 0, 0);
    open(&mut b);
    kill(&mut b, s2);
    b.tick = 8;
    b.effects.clear();
    let before = b.rng.taken();
    naval::sinking_turn(&mut b, s2);
    assert_eq!(b.rng.taken() - before, 2);
    assert!(!b.effects.iter().any(|e| matches!(e, Effect::Anim { .. })));
}

// spec: S-0312 claim 10 and S-0504 claim 6 (what does not sink: an underwater type, a type lighter than the sinking weight, a naval type off water; Q-0640 interim: a type without a weight)
#[test]
fn claim_10_sinking_condition() {
    let sinks = |name: &str, dx: i32, dy: i32| {
        let mut w = world(rules());
        let id = at(&mut w, name, 0, dx, dy);
        kill(&mut w, id);
        w.obj(id).is_some_and(|_| naval::sinking(&w, id))
    };
    assert!(sinks("SKIFF", 0, 0));
    assert!(!sinks("DIVER", 0, 0), "underwater");
    assert!(!sinks("LIGHT", 0, 0), "weight 2 < 3");
    assert!(!sinks("SCOW", 0, 0), "no weight (Q-0640, interim)");
    assert!(!sinks("SKIFF", 3, 0), "on land");
    assert!(!sinks("CART", 0, 0), "not naval");
    // A weight exactly the sinking weight sinks.
    let mut r = rules();
    r.sim.vehicles.get_mut("LIGHT").unwrap().weight = dec("3");
    let mut w = world(r);
    let id = at(&mut w, "LIGHT", 0, 0, 0);
    kill(&mut w, id);
    assert!(naval::sinking(&w, id));
}

// spec: S-0504 claim 5; S-0301 claim 9 ranks 4 and 9; S-0309 claim 24 (a sinking vessel's firing checks are illegal, a firer's check against it is illegal, it cannot be newly assigned, and its locomotor makes no progress)
#[test]
fn claim_5_sinking_in_play() {
    let mut w = world(rules());
    let skiff = at(&mut w, "SKIFF", 0, 0, 0);
    let foe = at(&mut w, "SKIFF", 1, 2, 2);
    open(&mut w);
    let tf = Some(Target::Object(foe));
    let ts = Some(Target::Object(skiff));
    assert_ne!(check::check(&w, skiff, tf, 0, false), Answer::Illegal);
    assert_ne!(check::check(&w, foe, ts, 0, false), Answer::Illegal);
    // Moving when it starts to sink.
    click_move(&mut w, skiff, cell(-6, 0));
    for _ in 0..10 {
        tick::step(&mut w);
    }
    w.obj_mut(foe).unwrap().target = ts;
    kill(&mut w, skiff);
    assert_eq!(
        w.obj(foe).unwrap().target,
        None,
        "the killing hit clears its targeters"
    );
    assert_eq!(
        check::check(&w, skiff, tf, 0, false),
        Answer::Illegal,
        "sinking firer"
    );
    w.obj_mut(foe).unwrap().combat.rearm_until = 0;
    assert_eq!(
        check::check(&w, foe, ts, 0, false),
        Answer::Illegal,
        "sinking target"
    );
    // A running rearm countdown (rank 5) answers before the late target tests (rank 9).
    w.obj_mut(foe).unwrap().combat.rearm_until = w.tick + 50;
    assert_eq!(check::check(&w, foe, ts, 0, false), Answer::Rearming);
    assert_eq!(
        glasnost_sim::combat::acquire::redirect(&w, foe, Target::Object(skiff)),
        None
    );
    let p = w.obj(skiff).unwrap().pos;
    let track = w.obj(skiff).unwrap().drive.as_ref().unwrap().track.clone();
    for _ in 0..12 {
        tick::step(&mut w);
    }
    let o = w.obj(skiff).unwrap();
    assert_eq!((o.pos.x, o.pos.y), (p.x, p.y), "no locomotor progress");
    assert_eq!(
        o.drive.as_ref().unwrap().track,
        track,
        "no track transition"
    );
    // A body turn already under way is not claimed to stop (claim 5: not every facing change stops).
    // Q-0642 (interim): no further hit.
    assert_eq!(kill(&mut w, skiff), HitResult::Unaffected);
}

// spec: S-0614 claim 11 and S-0504 claims 4 and 5 (a vessel set sinking without a hit keeps its health and descends the same way)
#[test]
fn claim_5_sinking_without_a_hit() {
    let mut w = world(rules());
    let skiff = at(&mut w, "SKIFF", 0, 0, 0);
    open(&mut w);
    let h = w.obj(skiff).unwrap().health;
    naval::start_sinking(&mut w, skiff);
    for _ in 0..10 {
        tick::step(&mut w);
    }
    let o = w.obj(skiff).unwrap();
    assert_eq!(o.health, h);
    assert_eq!(o.pos.z, -50);
    let n = common::run_until(&mut w, 200, |w| w.obj(skiff).is_none());
    assert_eq!(n, 71);
}

// spec: S-0504 claims 4 and 5 (a sinking vessel's state survives a save and load: the state hash is kept and both copies remove it on the same tick)
#[test]
fn sinking_save_load_round_trip() {
    let mut w = world(rules());
    let skiff = at(&mut w, "SKIFF", 0, 0, 0);
    open(&mut w);
    kill(&mut w, skiff);
    for _ in 0..13 {
        tick::step(&mut w);
    }
    let bytes = save::save(&w);
    let mut l = save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
    assert!(naval::sinking(&l, skiff));
    let a = common::run_until(&mut w, 200, |w| w.obj(skiff).is_none());
    let b = common::run_until(&mut l, 200, |w| w.obj(skiff).is_none());
    assert_eq!(a, b);
    assert_eq!(a, 81 - 13);
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
}

// spec: S-0504 claim 6 (an underwater type moves like a surface ship and is never submerged without cloaking; S-0516's cloak states are not modelled yet)
#[test]
fn claim_6_underwater_vessel() {
    let mut w = world(rules());
    let diver = at(&mut w, "DIVER", 0, -6, 4);
    let skiff = at(&mut w, "SKIFF", 0, -6, -4);
    open(&mut w);
    assert!(!naval::submerged(&w, diver));
    assert!(!naval::submerged(&w, skiff));
    click_move(&mut w, diver, cell(-1, 4));
    let p = w.obj(diver).unwrap().pos;
    for _ in 0..60 {
        tick::step(&mut w);
        assert_eq!(land_of(&w, diver), land::WATER);
        assert_eq!(w.height_above_surface(diver), 0, "no depth of its own");
    }
    assert_ne!(w.obj(diver).unwrap().pos, p);
}

// spec: S-0504 claim 7 (the naval match: exactly one naval of building and unit refuses, unless the unit is amphibious)
#[test]
fn claim_7_naval_match() {
    let mut w = world(rules());
    let yard = at(&mut w, "NAVYARD", 0, -8, -6);
    let depot = at(&mut w, "DEPOT", 0, 13, 0);
    let skiff = at(&mut w, "SKIFF", 0, 0, 0);
    let cart = at(&mut w, "CART", 0, 13, 4);
    let amphi = at(&mut w, "AMPHI", 0, 0, 2);
    assert!(!naval::naval_match_refuses(&w, yard, skiff));
    assert!(
        naval::naval_match_refuses(&w, yard, cart),
        "a land vehicle at a naval yard"
    );
    assert!(
        naval::naval_match_refuses(&w, depot, skiff),
        "a vessel at a land depot"
    );
    assert!(!naval::naval_match_refuses(&w, depot, cart));
    assert!(!naval::naval_match_refuses(&w, yard, amphi), "amphibious");
    assert!(!naval::naval_match_refuses(&w, depot, amphi), "amphibious");
}

// spec: S-0504 claim 8 and F3 (the starting distance, 200 for ship and hovering locomotors and 100 for others, strictly; the spatial distance with height, truncated)
#[test]
fn claim_8_waiting_gate() {
    assert_eq!(
        naval::depot_distance(Pos::new(0, 0, 0), Pos::new(3, 4, 0)),
        5
    );
    assert_eq!(
        naval::depot_distance(Pos::new(1, 1, 1), Pos::new(0, 0, 0)),
        1
    );
    assert_eq!(
        naval::depot_distance(Pos::new(0, 0, 200), Pos::new(0, 0, 0)),
        200,
        "height counts"
    );
    assert_eq!(
        naval::depot_distance(Pos::new(10_000, 10_000, 0), Pos::new(0, 0, 0)),
        14_142
    );
    assert_eq!(
        naval::depot_distance(Pos::new(-140, 0, 0), Pos::new(0, 0, 0)),
        140
    );
    let mut w = world(rules());
    let skiff = at(&mut w, "SKIFF", 0, 0, 0);
    let hover = at(&mut w, "HOVER", 0, 0, 3);
    let cart = at(&mut w, "CART", 0, 13, 0);
    assert_eq!(naval::starting_distance(&w, skiff), 200);
    assert_eq!(naval::starting_distance(&w, hover), 200);
    assert_eq!(naval::starting_distance(&w, cart), 100);
    let near = |w: &World, id: ObjectId, dx: i32, dz: i32| {
        let p = w.obj(id).unwrap().pos;
        naval::waiting_gate(w, id, Pos::new(p.x + dx, p.y, p.z + dz))
    };
    assert!(near(&w, skiff, 199, 0));
    assert!(!near(&w, skiff, 200, 0), "strictly closer");
    assert!(
        !near(&w, skiff, 150, 150),
        "height counts: trunc(√45000) = 212"
    );
    assert!(near(&w, cart, 99, 0));
    assert!(!near(&w, cart, 100, 0));
    // A ship that does not stand still is not waited for.
    open(&mut w);
    click_move(&mut w, skiff, cell(-6, 0));
    tick::step(&mut w);
    tick::step(&mut w);
    assert!(!near(&w, skiff, 0, 0));
    assert_eq!(naval::SETTLING_DELAY, 3);
}

/// Runs the naval yard production of one SKIFF for house 0 and returns the
/// world, the yard and the released ship.
fn produce_ship() -> (World, ObjectId, ObjectId) {
    let mut w = world(rules());
    // The yard on the lake's west water, its rally point in open water.
    let yard = at(&mut w, "NAVYARD", 0, -8, -6);
    w.obj_mut(yard).unwrap().guard_spot = Some(Target::Cell(cell(-2, -4)));
    open(&mut w);
    w.houses[0].credits = 10_000;
    let what = TypeRef {
        kind: Kind::Vehicle,
        index: w.rules().index_of(Kind::Vehicle, "SKIFF").unwrap(),
    };
    assert!(orders::issue(&mut w, ALPHA, Order::Build { what }));
    let mut ship = None;
    for _ in 0..2000 {
        tick::step(&mut w);
        ship = w
            .objects
            .values()
            .find(|o| o.kind == Kind::Vehicle && o.on_map && o.owner == Some(ALPHA))
            .map(|o| o.id);
        if ship.is_some() {
            break;
        }
    }
    let ship = ship.expect("the yard releases the ship");
    (w, yard, ship)
}

// spec: S-0504 claim 1, S-0404 claim 13 (naval production in play: a yard on water builds a ship, releases it onto water toward its rally point, and the ship is then ordered to move and to attack)
#[test]
fn naval_production_in_play() {
    let (mut w, _yard, ship) = produce_ship();
    assert_eq!(land_of(&w, ship), land::WATER, "released onto water");
    assert_eq!(w.obj(ship).unwrap().dest, Some(Target::Cell(cell(-2, -4))));
    for _ in 0..600 {
        tick::step(&mut w);
        assert_eq!(land_of(&w, ship), land::WATER);
        if stopped(&w, ship) {
            break;
        }
    }
    assert!(stopped(&w, ship));
    // Then ordered across the lake and to attack a vehicle on the shore.
    click_move(&mut w, ship, cell(7, 0));
    for _ in 0..1500 {
        tick::step(&mut w);
        assert_eq!(land_of(&w, ship), land::WATER);
        if stopped(&w, ship) {
            break;
        }
    }
    let cart = at(&mut w, "CART", 1, 12, 0);
    let t = w.tick;
    w.obj_mut(cart)
        .unwrap()
        .beh
        .force(Behaviour::Sleep, t, false);
    let h0 = w.obj(cart).unwrap().health;
    click_attack(&mut w, ship, cart);
    for _ in 0..800 {
        tick::step(&mut w);
        assert_eq!(land_of(&w, ship), land::WATER);
        if w.obj(cart).is_none_or(|o| o.health < h0) {
            break;
        }
    }
    assert!(
        w.obj(cart).is_none_or(|o| o.health < h0),
        "the ship damages its target"
    );
}

// spec: S-0504 claims 1, 2 and 4 (end to end: a ship moving by a click makes wakes; a ship killed by an enemy's fire sinks, descends and leaves play)
#[test]
fn killed_ship_sinks_in_play() {
    let mut r = rules();
    // A frail ship, so that the enemy gun kills it quickly.
    r.sim.vehicles.get_mut("SKIFF").unwrap().strength = Some(20);
    let mut w = world(r);
    let skiff = at(&mut w, "SKIFF", 0, -6, 4);
    let cart = at(&mut w, "CART", 1, 12, 4);
    open(&mut w);
    click_move(&mut w, skiff, cell(8, 4));
    let mut moving_wakes = 0;
    let mut sank_at = None;
    for _ in 0..2000 {
        w.effects.clear();
        tick::step(&mut w);
        if sank_at.is_none() {
            moving_wakes += wakes(&w).len();
        }
        if sank_at.is_none() && naval::sinking(&w, skiff) {
            sank_at = Some((w.tick, w.obj(skiff).unwrap().pos));
        }
        if w.obj(skiff).is_none() {
            break;
        }
    }
    assert!(moving_wakes > 0, "wakes while moving");
    let (t, p) = sank_at.expect("the enemy's fire sinks the ship");
    assert!(w.obj(skiff).is_none(), "it leaves play");
    assert!(w.tick - t >= 79, "after its descent: {} ticks", w.tick - t);
    assert!(p.z <= 0);
    assert!(w.obj(cart).is_some());
}

/// A rocket type of S-0506 set B.
fn rocket(speed: i32) -> UnitType {
    UnitType {
        strength: Some(40),
        speed: Some(speed),
        rot: Some(20),
        missile_spawn: Some(true),
        spawned: Some(true),
        locomotor: Some(Locomotor::Rocket),
        ..Default::default()
    }
}

/// The naval ruleset with a ship-borne spawner VESSEL: `spawns` slots of
/// `n`, the spawner weapon Launch (range 30 cells); `missile` adds the set-B
/// rocket type RKB and its game-wide settings.
fn spawner_rules(spawns: &str, n: i32, missile: bool) -> Ruleset {
    let mut r = rules();
    let mut v = ship();
    v.strength = Some(800);
    v.primary = Some("Launch".into());
    v.missile_spawn = Some(missile);
    v.combat.spawns = Some(spawns.into());
    v.combat.spawns_number = Some(n);
    v.combat.spawn_regen_rate = Some(50);
    v.combat.spawn_reload_rate = Some(0);
    r.sim.vehicles.insert("VESSEL".into(), v);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("VESSEL".into());
    r.sim.vehicles.get_mut("CART").unwrap().strength = Some(5000);
    r.sim.weapons.insert(
        "Launch".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(20),
            range: dec("30"),
            projectile: Some("Bullet".into()),
            warhead: Some("HE".into()),
            spawner: Some(true),
            ..Default::default()
        },
    );
    if missile {
        r.sim.aircraft.insert("RKB".into(), rocket(40));
        r.registries
            .entry("aircraft".into())
            .or_default()
            .push("RKB".into());
        let g = &mut r.sim.general;
        g.d_misl_type = Some("RKB".into());
        g.d_misl_pitch_final = dec("0.5");
        g.d_misl_turn_rate = dec("0.2");
        g.d_misl_pause_frames = Some(3);
        g.d_misl_tilt_frames = Some(8);
        g.d_misl_altitude = Some(500);
        g.d_misl_acceleration = dec("2.5");
        g.d_misl_damage = Some(200);
        g.d_misl_elite_damage = Some(250);
        g.d_misl_warhead = Some("RKW".into());
        r.sim.warheads.insert(
            "RKW".into(),
            glasnost_rules::sim::WarheadType {
                cell_spread: dec("2"),
                percent_at_max: dec("0.5"),
                verses: Some((0..11).map(|_| Decimal("1".into())).collect()),
                ..Default::default()
            },
        );
    } else {
        r.sim.aircraft.insert(
            "WASP".into(),
            UnitType {
                strength: Some(60),
                speed: Some(12),
                rot: Some(10),
                spawned: Some(true),
                locomotor: Some(Locomotor::Fly),
                primary: Some("Gun".into()),
                ..Default::default()
            },
        );
        r.registries
            .entry("aircraft".into())
            .or_default()
            .push("WASP".into());
    }
    r
}

// spec: S-0504 claims 1 and 3, S-0506 claims 1 to 9 and 15 to 19 (end to end, set B on the ship locomotor: a ship-borne launcher ordered by a click launches its two missiles, which fly onto and damage the target; the ship stays on water)
#[test]
fn ship_borne_set_b_launcher_attacks() {
    let mut w = world(spawner_rules("RKB", 2, true));
    let vessel = at(&mut w, "VESSEL", 0, 0, 0);
    let cart = at(&mut w, "CART", 1, 12, 2);
    open(&mut w);
    let h0 = w.obj(cart).unwrap().health;
    let slots: Vec<ObjectId> = w
        .obj(vessel)
        .unwrap()
        .spawn
        .as_ref()
        .unwrap()
        .slots
        .iter()
        .map(|s| s.object.unwrap())
        .collect();
    assert_eq!(slots.len(), 2);
    click_attack(&mut w, vessel, cart);
    let mut detonations = 0;
    for _ in 0..900 {
        w.effects.clear();
        tick::step(&mut w);
        assert_eq!(land_of(&w, vessel), land::WATER);
        detonations += w
            .effects
            .iter()
            .filter(|e| matches!(e, Effect::Detonation { damage: 200, .. }))
            .count();
        if slots.iter().all(|&x| w.obj(x).is_none()) {
            break;
        }
    }
    assert!(detonations >= 2, "{detonations}");
    assert!(w.obj(cart).unwrap().health < h0);
}

/// Runs a ship (crusher or not) along the lake's southern row past a
/// crushable overlay of registration index 0; returns whether it ever
/// entered the crush-slowdown state and whether it arrived.
fn ship_past_weed(crusher: bool) -> (bool, bool) {
    let mut r = rules();
    r.registries.insert("overlays".into(), vec!["WEED".into()]);
    r.sim.overlays.insert(
        "WEED".into(),
        glasnost_rules::sim::OverlayType {
            has_art: Some(true),
            crushable: Some(true),
            ..Default::default()
        },
    );
    r.sim.vehicles.get_mut("SKIFF").unwrap().crusher = Some(crusher);
    let mut s = scene(r);
    s.cell_mut(cell(-3, 5)).overlay = 0;
    let mut w = s.build();
    let id = at(&mut w, "SKIFF", 0, -7, 5);
    open(&mut w);
    click_move(&mut w, id, cell(1, 5));
    let mut entered = false;
    for _ in 0..800 {
        tick::step(&mut w);
        entered |= w.obj(id).unwrap().crush_slowdown;
        if stopped(&w, id) {
            break;
        }
    }
    (entered, w.obj(id).unwrap().pos.cell() == cell(1, 5))
}

// spec: S-0504 claim 1 and S-0210 claim 9 (the ship locomotor's crush slowdown: a crusher ship entering the cell of the index-0 overlay on a crush-mode track; a ship that is no crusher does not)
#[test]
fn ship_crush_slowdown() {
    let (entered, arrived) = ship_past_weed(true);
    assert!(arrived);
    assert!(entered, "a crusher ship enters the state");
    let (entered, _) = ship_past_weed(false);
    assert!(!entered, "not a crusher");
}

// spec: S-0504 claim 3 and S-0506 claims 1 and 2 (a carrier-like ship with ordinary spawned aircraft: the spawner hooks run in the ship's turns without fault; the ship keeps to the water)
#[test]
fn carrier_like_ship_runs() {
    let mut w = world(spawner_rules("WASP", 2, false));
    let vessel = at(&mut w, "VESSEL", 0, 0, 0);
    let cart = at(&mut w, "CART", 1, 12, 2);
    open(&mut w);
    assert!(w.obj(vessel).unwrap().spawn.is_some());
    let r = targeting::click(
        &w,
        ALPHA,
        &[vessel],
        ClickOn::Object(cart),
        Modifiers::default(),
    );
    for o in r.orders {
        let _ = orders::issue(&mut w, ALPHA, o);
    }
    for _ in 0..400 {
        tick::step(&mut w);
        assert_eq!(land_of(&w, vessel), land::WATER);
    }
    click_move(&mut w, vessel, cell(-6, 0));
    for _ in 0..400 {
        tick::step(&mut w);
        assert_eq!(land_of(&w, vessel), land::WATER);
    }
    assert!(w.obj(vessel).is_some());
}

/// Orders `actor` of `house` by a click on `target`.
fn click_attack_as(w: &mut World, house: HouseId, actor: ObjectId, target: ObjectId) {
    let r = targeting::click(
        w,
        house,
        &[actor],
        ClickOn::Object(target),
        Modifiers::default(),
    );
    assert!(!r.orders.is_empty(), "the click gives an attack order");
    for o in r.orders {
        assert!(orders::issue(w, house, o));
    }
}

// spec: S-0504 claims 1 and 3, S-0506 claims 1 to 9 and 15 to 19, S-0501 claims 35 and 36 (end to end: a missile ship and a carrier ship of two houses fight across the lake; the missiles damage the carrier, the carrier's aircraft fly out and damage the missile ship, and both ships stay on water)
#[test]
fn carrier_and_missile_ship_fight() {
    let mut r = spawner_rules("RKB", 2, true);
    r.sim.aircraft.insert(
        "WASP".into(),
        UnitType {
            strength: Some(60),
            speed: Some(12),
            rot: Some(10),
            spawned: Some(true),
            locomotor: Some(Locomotor::Fly),
            primary: Some("Gun".into()),
            ..Default::default()
        },
    );
    r.registries
        .entry("aircraft".into())
        .or_default()
        .push("WASP".into());
    let mut carrier = r.sim.vehicles["VESSEL"].clone();
    carrier.missile_spawn = Some(false);
    carrier.combat.spawns = Some("WASP".into());
    r.sim.vehicles.insert("CARRIER".into(), carrier);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("CARRIER".into());
    let mut w = world(r);
    let launcher = at(&mut w, "VESSEL", 0, -6, 4);
    let carrier = at(&mut w, "CARRIER", 1, 8, 4);
    open(&mut w);
    let (l0, c0) = (
        w.obj(launcher).unwrap().health,
        w.obj(carrier).unwrap().health,
    );
    click_attack_as(&mut w, ALPHA, launcher, carrier);
    click_attack_as(&mut w, HouseId(1), carrier, launcher);
    let mut aircraft_seen = false;
    for _ in 0..2000 {
        tick::step(&mut w);
        for s in [launcher, carrier] {
            if w.obj(s).is_some_and(|o| o.health > 0) {
                assert_eq!(land_of(&w, s), land::WATER);
            }
        }
        aircraft_seen |= w
            .objects
            .values()
            .any(|o| o.kind == Kind::Aircraft && o.owner == Some(HouseId(1)) && o.on_map);
        let hurt = |id: ObjectId, h0: i32| w.obj(id).is_none_or(|o| o.health < h0);
        if hurt(launcher, l0) && hurt(carrier, c0) {
            break;
        }
    }
    assert!(aircraft_seen, "the carrier launches its aircraft");
    assert!(
        w.obj(carrier).is_none_or(|o| o.health < c0),
        "the missiles hit the carrier"
    );
    assert!(
        w.obj(launcher).is_none_or(|o| o.health < l0),
        "the carrier's aircraft hit the missile ship"
    );
}
