//! Claim tests of S-0504 (naval units: the ship locomotor, wakes, sinking,
//! underwater vessels and docking at yards) on a synthetic water map with
//! invented types (every name and number here is made up; no game data).

mod common;

use common::{Scene, cell, code_tile};
use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_num::geom::eight_direction;
use glasnost_num::{F32, F64, Rounding};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BuildingType, Decimal, LandRow, LandType, LocomotionClass, Locomotor, MovementZone, UnitType,
    WeaponType,
};
use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::combat::damage::{self, Hit, HitResult};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::econ::TypeRef;
use glasnost_sim::naval::{self, Lean};
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
///   DIVER (underwater, cloakable), DEEP (underwater, not cloakable),
///   SCOW (no weight);
/// - HOVER (the hovering locomotor), AMPHI (amphibious zone);
/// - buildings NAVYARD (naval vehicle factory with the repair flag, its
///   sole docking displacement to the cell (2, 1) from its top-left) and
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
    let mut deep = ship();
    deep.combat.underwater = Some(true);
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
        ("DEEP", deep),
        ("SCOW", scow),
        ("HOVER", hover),
        ("AMPHI", amphi),
    ] {
        r.sim.vehicles.insert(n.into(), u);
        push(&mut r, "vehicles", n);
    }
    r.sim.aircraft.insert(
        "FLYER".into(),
        UnitType {
            strength: Some(100),
            ..Default::default()
        },
    );
    push(&mut r, "aircraft", "FLYER");
    let mut yard = BuildingType {
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
    // The sole docking displacement from the foundation centre to the centre
    // of the cell (2, 1) cells from the top-left cell (S-0314 claim 18).
    yard.econ.docking_offsets = Some(vec![[384, 128, 0]]);
    let mut depot = BuildingType {
        naval: None,
        factory: None,
        ..yard.clone()
    };
    depot.econ.docking_offsets = None;
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
            Effect::Anim { name, pos, .. } if name == WAKE => Some(*pos),
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

// spec: S-0504 claim 1 and S-0207 claim 26 (order-level cleanup of a ship: an accepted same-cell move is removed at the first locomotion update without a track, with no search; final completion in tick t clears only the locomotor point and the unchanged cell goal goes at tick t + 1)
#[test]
fn claim_1_ship_goal_cleanup() {
    let mut w = world(rules());
    let skiff = at(&mut w, "SKIFF", 0, 0, 0);
    open(&mut w);
    let here = w.obj(skiff).unwrap().pos.cell();
    assert!(orders::issue(&mut w, ALPHA, common::move_to(skiff, here)));
    orders::apply_pending(&mut w);
    assert_eq!(w.obj(skiff).unwrap().dest, Some(Target::Cell(here)));
    let taken = w.rng.taken();
    glasnost_sim::drive::tick(&mut w, skiff);
    assert_eq!(
        w.obj(skiff).unwrap().dest,
        None,
        "removed by the first update"
    );
    assert_eq!(w.rng.taken(), taken, "no search draw");
    // A move to the next cell: the goal outlives the completion tick by one.
    let goal = CellXY::new(here.x + 1, here.y);
    assert!(orders::issue(&mut w, ALPHA, common::move_to(skiff, goal)));
    let mut saw_track = false;
    let mut done = false;
    for _ in 0..400 {
        tick::step(&mut w);
        let o = w.obj(skiff).unwrap();
        let on_track = o.drive.as_ref().unwrap().track.is_some();
        saw_track |= on_track;
        if saw_track && !on_track {
            done = true;
            break;
        }
    }
    assert!(done, "the track completes");
    assert_eq!(w.obj(skiff).unwrap().pos.cell(), goal);
    assert_eq!(
        w.obj(skiff).unwrap().dest,
        Some(Target::Cell(goal)),
        "kept through the completion tick"
    );
    assert!(!glasnost_sim::drive::goal_present(&w, skiff));
    tick::step(&mut w);
    assert_eq!(
        w.obj(skiff).unwrap().dest,
        None,
        "removed at the next update"
    );
}

/// A ship whose destination names a unit of `kind` (`name`) that stands on
/// the ship's own point.
fn naming(kind: Kind, name: &str) -> (World, ObjectId, ObjectId) {
    let mut w = world(rules());
    let ship = at(&mut w, "SKIFF", 0, 0, 0);
    open(&mut w);
    let n = testkit::spawn(&mut w, kind, name, 0, cell(5, 5), 0);
    let here = w.obj(ship).unwrap().pos;
    w.obj_mut(n).unwrap().pos = here;
    let o = w.obj_mut(ship).unwrap();
    o.dest = Some(Target::Object(n));
    // Not at rest in guard: the cell-equality rows are tested here.
    o.beh.current = Some(Behaviour::Move);
    (w, ship, n)
}

const UNIT_GOALS: [(Kind, &str); 3] = [
    (Kind::Vehicle, "CART"),
    (Kind::Infantry, "WALKER"),
    (Kind::Aircraft, "FLYER"),
];

// spec: S-0504 claim 1 (S-0207 claim 26, D31: a destination naming a vehicle, infantry unit or aircraft is not a cell destination; with no track, supplied-point cell equality alone clears neither the destination nor the point, whether the named unit is stationary or moving)
#[test]
fn claim_1_ship_keeps_a_destination_naming_a_unit_under_cell_equality() {
    for (kind, name) in UNIT_GOALS {
        let (mut w, ship, n) = naming(kind, name);
        for step in 0..4 {
            // The named unit moving, with no replacement instruction, makes no difference.
            if step >= 2 {
                w.obj_mut(n).unwrap().pos.x += 20;
            }
            glasnost_sim::drive::tick(&mut w, ship);
            assert_eq!(
                w.obj(ship).unwrap().dest,
                Some(Target::Object(n)),
                "{name}, update {step}"
            );
            assert!(glasnost_sim::drive::goal_present(&w, ship), "{name}");
        }
    }
}

// spec: S-0504 claim 1 (S-0207 claim 26, D31: at rest in guard with a locomotor point present, exact equality of the ship's world point and that point in x, y and z removes the destination, for a unit goal too; the comparison is with the retained point, not the named unit's position; same-cell equality alone does not qualify)
#[test]
fn claim_1_ship_guard_row_compares_the_retained_point_for_a_unit_goal() {
    // (the retained point's offset from the ship, the named unit's offset) -> removed?
    for (kind, name) in UNIT_GOALS {
        for (point_dx, named_dx, removed) in [(0, 0, true), (0, 90, true), (30, 0, false)] {
            let (mut w, ship, n) = naming(kind, name);
            let p = w.obj(ship).unwrap().pos;
            {
                let o = w.obj_mut(ship).unwrap();
                o.beh.current = Some(Behaviour::Guard);
                let d = o.drive.as_mut().unwrap();
                d.for_dest = Some(Target::Object(n));
                d.goal = Some(Pos::new(p.x - point_dx, p.y, p.z));
            }
            w.obj_mut(n).unwrap().pos.x += named_dx;
            glasnost_sim::drive::tick(&mut w, ship);
            let kept = w.obj(ship).unwrap().dest == Some(Target::Object(n));
            assert_eq!(kept, !removed, "{name}: point {point_dx}, named {named_dx}");
        }
    }
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

// spec: S-0312 claim 10 and S-0504 claim 6 (what does not sink: an underwater type, a type lighter than the sinking weight, a naval type off water); S-0504 claim 11 (a type without a weight weighs exactly 1.0)
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
    assert!(
        !sinks("SCOW", 0, 0),
        "no weight: 1.0 below the sinking weight 3"
    );
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
    // S-0504 claims 10 and 13: no immunity, the hit is taken and the vessel stays sinking at 1 health.
    assert_eq!(kill(&mut w, skiff), HitResult::NowDead);
    assert!(naval::sinking(&w, skiff), "still sinking");
    assert_eq!(w.obj(skiff).unwrap().health, 1);
}

// spec: S-0504 claim 11 (an absent weight is exactly 1.0: it sinks when the sinking weight is at most 1.0 and not above)
#[test]
fn claim_11_absent_weight_is_one() {
    for (threshold, sinks) in [("1", true), ("1.0001", false), ("0.5", true)] {
        let mut r = rules();
        r.sim.general.ship_sinking_weight = dec(threshold);
        let mut w = world(r);
        let id = at(&mut w, "SCOW", 0, 0, 0);
        kill(&mut w, id);
        assert_eq!(naval::sinking(&w, id), sinks, "threshold {threshold}");
    }
}

// spec: S-0504 claim 10 and S-0314 claims 9 and 19 (a sinking vessel still self-heals on the ordinary schedule; healing does not end sinking or its descent)
#[test]
fn claim_10_sinking_vessel_self_heals() {
    let mut r = rules();
    r.sim.vehicles.get_mut("SKIFF").unwrap().combat.self_healing = Some(true);
    let mut w = world(r);
    let skiff = at(&mut w, "SKIFF", 0, 0, 0);
    open(&mut w);
    assert_eq!(kill(&mut w, skiff), HitResult::NowDead);
    assert_eq!(w.obj(skiff).unwrap().health, 1);
    for _ in 0..30 {
        tick::step(&mut w);
    }
    let o = w.obj(skiff).unwrap();
    assert!(o.health > 1, "healed: {}", o.health);
    assert!(naval::sinking(&w, skiff), "still sinking");
    assert!(w.height_above_surface(skiff) < 0, "still descending");
    let gone = (0..200).any(|_| {
        tick::step(&mut w);
        w.obj(skiff).is_none()
    });
    assert!(gone, "leaves play as usual");
}

// spec: S-0504 claim 10 (a sinking vessel takes ordinary damage: a nonlethal hit after healing lowers its health)
#[test]
fn claim_10_sinking_vessel_takes_damage() {
    let mut r = rules();
    r.sim.vehicles.get_mut("SKIFF").unwrap().combat.self_healing = Some(true);
    let mut w = world(r);
    let skiff = at(&mut w, "SKIFF", 0, 0, 0);
    open(&mut w);
    kill(&mut w, skiff);
    for _ in 0..30 {
        tick::step(&mut w);
    }
    let h = w.obj(skiff).unwrap().health;
    assert!(h > 1);
    let mut hit = lethal();
    hit.amount = 1;
    let _ = damage::apply(&mut w, skiff, hit);
    assert!(w.obj(skiff).unwrap().health < h, "damaged while sinking");
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

// spec: S-0504 claims 4, 5 and 15 (a sinking vessel's state, its lean included, survives a save and load: the state hash is kept and both copies remove it on the same tick)
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
    // spec: S-0504 claim 15 (the pitch and the kept direction are saved with the vessel)
    assert_eq!(l.obj(skiff).unwrap().naval, w.obj(skiff).unwrap().naval);
    assert_ne!(naval::pitch(&l, skiff), glasnost_num::F32::ZERO);
    let a = common::run_until(&mut w, 200, |w| w.obj(skiff).is_none());
    let b = common::run_until(&mut l, 200, |w| w.obj(skiff).is_none());
    assert_eq!(a, b);
    assert_eq!(a, 81 - 13);
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
}

// spec: S-0504 claim 6 (an underwater type moves like a surface ship and is never submerged without cloaking); S-0516 claims 16 and 41 (a self-cloaking underwater vessel enters play submerged)
#[test]
fn claim_6_underwater_vessel() {
    let mut w = world(rules());
    let diver = at(&mut w, "DIVER", 0, -6, 4);
    let skiff = at(&mut w, "SKIFF", 0, -6, -4);
    let deep = at(&mut w, "DEEP", 0, -6, 0);
    open(&mut w);
    assert!(naval::submerged(&w, diver), "placed already cloaked");
    assert!(!naval::submerged(&w, skiff));
    assert!(
        !naval::submerged(&w, deep),
        "an underwater type that cannot cloak"
    );
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

// spec: S-0504 claim 7 (only the plain amphibious zone bypasses the naval-flag comparison; amphibious-crusher and amphibious-destroyer must match the building's naval flag)
#[test]
fn claim_7_other_amphibious_zones_match_the_flag() {
    for z in [
        MovementZone::AmphibiousCrusher,
        MovementZone::AmphibiousDestroyer,
    ] {
        let mut r = rules();
        r.sim.vehicles.get_mut("AMPHI").unwrap().movement_zone = Some(z);
        let mut w = world(r);
        let yard = at(&mut w, "NAVYARD", 0, -8, -6);
        let depot = at(&mut w, "DEPOT", 0, 13, 0);
        let amphi = at(&mut w, "AMPHI", 0, 0, 2);
        assert!(
            naval::naval_match_refuses(&w, yard, amphi),
            "{z:?} at a naval yard"
        );
        assert!(
            !naval::naval_match_refuses(&w, depot, amphi),
            "{z:?} at a land depot"
        );
    }
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
    let wasp = UnitType {
        strength: Some(60),
        speed: Some(12),
        rot: Some(10),
        spawned: Some(true),
        locomotor: Some(Locomotor::Fly),
        primary: Some("Gun".into()),
        ..Default::default()
    };
    // spec: S-0501 claim 1 and Terms (a slot type that is not landable is marked visiting by its launch placement's type test, after its placement fall-back gave it guard; the launch pass's move and attack reach it, claim 35; Q-1838)
    r.sim.aircraft.insert("WASP".into(), wasp);
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

// spec: S-0504 claim 12 (the obsolete-unloading flag, not the vehicle-factory flag, selects the centre of the cell (2, 1) cells from the top-left foundation cell, at the building's height; an ordinary naval repair yard uses S-0314 claim 18's foundation centre plus its sole docking displacement)
#[test]
fn claim_12_yard_docking_point() {
    let mut r = rules();
    r.sim.buildings.get_mut("DEPOT").unwrap().econ.weeder = Some(true);
    let mut w = world(r);
    let yard = at(&mut w, "NAVYARD", 0, -4, -4);
    let depot = at(&mut w, "DEPOT", 0, 5, 7);
    let skiff = at(&mut w, "SKIFF", 0, -8, 0);
    assert_eq!(naval::yard_docking_point(&w, yard), None);
    let c = w.reference_point(yard);
    let p = glasnost_sim::depot::docking_point(&w, yard, skiff);
    assert_eq!((p.x - c.x, p.y - c.y, p.z - c.z), (384, 128, 0));
    let o = w.obj(depot).unwrap();
    let tl = o.pos.cell();
    let want = Pos::centre(CellXY::new(tl.x + 2, tl.y + 1), o.pos.z);
    assert_eq!(naval::yard_docking_point(&w, depot), Some(want));
    assert_eq!(glasnost_sim::depot::docking_point(&w, depot, skiff), want);
}

/// A damaged SKIFF clicked onto a repairing NAVYARD on the lake, run until
/// it is docked asleep at the yard's docking point.
fn ship_docked_at_yard() -> (World, ObjectId, ObjectId) {
    let mut w = world(rules());
    for h in &mut w.houses {
        h.credits = 10_000;
    }
    let yard = at(&mut w, "NAVYARD", 0, 5, 3);
    let skiff = at(&mut w, "SKIFF", 0, -4, 0);
    w.obj_mut(skiff).unwrap().health = 200;
    open(&mut w);
    let r = targeting::click(
        &w,
        ALPHA,
        &[skiff],
        ClickOn::Object(yard),
        Modifiers::default(),
    );
    assert!(
        r.orders.iter().any(|o| matches!(
            o,
            Order::Behaviour {
                behaviour: Behaviour::Enter,
                ..
            }
        )),
        "the click over the yard is enter: {:?}",
        r.orders
    );
    for o in r.orders {
        assert!(orders::issue(&mut w, ALPHA, o));
    }
    let mut docked = false;
    for _ in 0..1500 {
        tick::step(&mut w);
        // spec: S-0314 claim 17 (a yard without the pad flag services the vessel untethered)
        if w.obj(skiff).unwrap().contact == Some(yard)
            && w.obj(skiff).unwrap().beh.current == Some(Behaviour::Sleep)
        {
            docked = true;
            break;
        }
    }
    assert!(docked, "the ship arrives at its yard");
    assert!(!glasnost_sim::depot::tethered(&w, skiff));
    (w, yard, skiff)
}

// spec: S-0504 claims 8, 12 and 14 and S-0314 claims 6 and 16 to 18 (a damaged ship clicked onto its repairing yard docks at the yard's docking point, passes the 200-lepton waiting gate, is repaired step by step at its owner's cost and is sent away)
#[test]
fn claim_14_ship_repaired_at_yard() {
    let (mut w, yard, skiff) = ship_docked_at_yard();
    let o = w.obj(yard).unwrap();
    let c = o.pos.cell();
    assert_eq!(
        w.obj(skiff).unwrap().pos.cell(),
        CellXY::new(c.x + 2, c.y + 1)
    );
    let credits = w.houses[0].credits;
    let mut done = false;
    for _ in 0..6000 {
        tick::step(&mut w);
        if w.obj(skiff).unwrap().contact.is_none() {
            done = true;
            break;
        }
    }
    assert!(done, "sent away");
    assert_eq!(w.obj(skiff).unwrap().health, 300);
    assert!(w.houses[0].credits < credits, "its owner paid");
}

// spec: S-0504 claim 14 and S-0314 claim 20 (a docked vessel keeps its contact and tether under a move order; an untethered one, as at a yard without the pad flag, S-0314 claim 17, loses it)
#[test]
fn claim_14_docked_ship_keeps_contact_under_a_move() {
    let (mut w, _yard, skiff) = ship_docked_at_yard();
    click_move(&mut w, skiff, cell(-4, 4));
    tick::step(&mut w);
    assert_eq!(w.obj(skiff).unwrap().contact, None, "untethered: released");
    let (mut w, yard, skiff) = ship_docked_at_yard();
    // The tether a pad exchange would give, set directly.
    w.obj_mut(skiff)
        .unwrap()
        .depot
        .link
        .as_mut()
        .unwrap()
        .tethered = true;
    click_move(&mut w, skiff, cell(-4, 4));
    tick::step(&mut w);
    assert_eq!(w.obj(skiff).unwrap().contact, Some(yard));
    assert!(glasnost_sim::depot::tethered(&w, skiff));
}

// spec: S-0314 claim 22 and S-0504 claim 14 (a vehicle destroyed while in contact frees its place; the yard meets no contact at its next processing); S-0312 claim 10 (a killed vessel that sinks stays in play at 1 health point) (#179)
#[test]
fn claim_14_killed_docked_vessel_frees_its_place_at_the_killing_hit() {
    let (mut w, yard, skiff) = ship_docked_at_yard();
    // Run until the yard has taken a first paid step (it is then working).
    let c0 = w.houses[0].credits;
    for _ in 0..600 {
        tick::step(&mut w);
        if w.houses[0].credits < c0 {
            break;
        }
    }
    assert!(w.houses[0].credits < c0, "a first step was paid");
    for _ in 0..3 {
        tick::step(&mut w);
    }
    kill(&mut w, skiff);
    assert!(naval::sinking(&w, skiff), "the killed vessel sinks, in play");
    assert_eq!(w.obj(skiff).unwrap().health, 1);
    let contact_after_kill = w.obj(skiff).unwrap().contact;
    let other = at(&mut w, "SKIFF", 0, -4, 0);
    w.obj_mut(other).unwrap().health = 200;
    let accepts_other = glasnost_sim::depot::accepts(&w, yard, other, true);
    let credits = w.houses[0].credits;
    let mut max_health = 1;
    let mut ticks = 0;
    while w.obj(skiff).is_some() && ticks < 200 {
        tick::step(&mut w);
        ticks += 1;
        if let Some(o) = w.obj(skiff) {
            max_health = max_health.max(o.health);
        }
    }
    let paid = credits - w.houses[0].credits;
    assert_eq!(contact_after_kill, None, "the killing hit frees the place");
    assert!(accepts_other, "the place is free for another vessel");
    assert_eq!(paid, 0, "no repair step is paid for the wreck");
    assert_eq!(max_health, 1, "the wreck's health is not raised by the yard");
}

/// Kills a fresh SKIFF, runs 20 ticks of its descent, then hits it again
/// lethally when `again`; returns the height and health just after, and the
/// tick at which it leaves play.
fn descent_with_second_kill(again: bool) -> (i32, i32, i32) {
    let mut w = world(rules());
    let skiff = at(&mut w, "SKIFF", 0, 0, 0);
    open(&mut w);
    kill(&mut w, skiff);
    for _ in 0..20 {
        tick::step(&mut w);
    }
    let before = w.obj(skiff).unwrap().pos.z;
    let losses = |w: &World| w.houses[0].econ.result.stats.losses;
    let l0 = losses(&w);
    if again {
        assert_eq!(kill(&mut w, skiff), HitResult::NowDead);
        // No exception to the destruction credit for a vessel already descending (S-0312 owns it).
        assert_eq!(losses(&w), l0 + 1, "credited again");
    }
    let o = w.obj(skiff).unwrap();
    assert_eq!(o.pos.z, before, "the hit itself changes no height");
    let (z, health) = (o.pos.z, o.health);
    common::run_until(&mut w, 400, |w| w.obj(skiff).is_none());
    (z, health, w.tick)
}

// spec: S-0504 claim 13 (a further killing hit on a sinking vessel that meets S-0312 claim 10's conditions leaves it at 1 health, still sinking; its descent continues from its current height and its removal tick does not move)
#[test]
fn claim_13_further_killing_hit_continues_descent() {
    let (z0, _, gone0) = descent_with_second_kill(false);
    let (z1, health, gone1) = descent_with_second_kill(true);
    assert_eq!(health, 1);
    assert_eq!(z1, z0, "not put back at the surface");
    assert!(z1 < 0, "already descending");
    assert_eq!(gone1, gone0, "removal not extended");
}

/// The naval ruleset with explosion and destruction lists on SCOW, DIVER
/// and SKIFF, and a crashable copy of SCOW, CRASHER.
fn rules_with_death_anims() -> Ruleset {
    let mut r = rules();
    let names = |v: &[&str]| Some(v.iter().map(|s| s.to_string()).collect::<Vec<_>>());
    for n in ["SCOW", "DIVER", "SKIFF"] {
        let c = &mut r.sim.vehicles.get_mut(n).unwrap().combat;
        c.explosion = names(&["BOOM1", "BOOM2"]);
        c.destroy_anim = names(&["WRECK1", "WRECK2", "WRECK3"]);
    }
    let mut crasher = r.sim.vehicles["SCOW"].clone();
    crasher.crashable = Some(true);
    r.sim.vehicles.insert("CRASHER".into(), crasher);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("CRASHER".into());
    r
}

fn anim_names(w: &World) -> Vec<String> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Anim { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect()
}

/// Starts `name`'s descent by a cull (S-0614 claim 11), runs 5 ticks, then
/// kills it when `hit`; returns the world, the vessel, and the tick at
/// which it leaves play (within 400 ticks).
fn culled_then_hit(name: &str, hit: bool) -> (World, ObjectId, Option<i32>) {
    let mut w = world(rules_with_death_anims());
    let v = at(&mut w, name, 0, 0, 0);
    open(&mut w);
    naval::start_sinking(&mut w, v);
    for _ in 0..5 {
        tick::step(&mut w);
    }
    if hit {
        assert_eq!(kill(&mut w, v), HitResult::NowDead);
    }
    let mut gone = None;
    let mut w2 = w.clone();
    for _ in 0..400 {
        tick::step(&mut w2);
        if w2.obj(v).is_none() {
            gone = Some(w2.tick);
            break;
        }
    }
    (w, v, gone)
}

// spec: S-0312 claim 32 table row 2 and claim 15; S-0504 claims 4 and 13 (a cull-started vessel failing claim 10's conditions, no weight or underwater, ends a killing hit at 0 health with claim 9's explosion and destruction animations and their two list picks, and stays in play because it is sinking; it keeps its descent and leaves play below −400 leptons at the same tick as without the hit)
#[test]
fn claim_13_cull_started_type_failing_the_condition_dies() {
    for name in ["SCOW", "DIVER"] {
        let (_, _, gone0) = culled_then_hit(name, false);
        let mut w = world(rules_with_death_anims());
        let v = at(&mut w, name, 0, 0, 0);
        open(&mut w);
        naval::start_sinking(&mut w, v);
        for _ in 0..5 {
            tick::step(&mut w);
        }
        let z = w.obj(v).unwrap().pos.z;
        let losses = |w: &World| w.houses[0].econ.result.stats.losses;
        let l0 = losses(&w);
        w.effects.clear();
        let mut probe = w.rng.clone();
        assert_eq!(kill(&mut w, v), HitResult::NowDead);
        let o = w.obj(v).expect("stays in play");
        assert_eq!(
            (o.health, o.pos.z),
            (0, z),
            "{name}: 0 health, no height change"
        );
        assert!(naval::sinking(&w, v), "{name}: still sinking");
        assert_eq!(losses(&w), l0 + 1, "{name}: destruction registered");
        // spec: S-0312 claim 9 (one list pick from each list, explosion first)
        let a = anim_names(&w);
        let e = probe.list_pick(2) as usize;
        let d = probe.list_pick(3) as usize;
        assert_eq!(
            a,
            vec![
                ["BOOM1", "BOOM2"][e].to_string(),
                ["WRECK1", "WRECK2", "WRECK3"][d].to_string()
            ],
            "{name}"
        );
        assert_eq!(w.rng, probe, "{name}: the two list picks only");
        // The descent goes on at 0 health: 5 leptons per own update.
        tick::step(&mut w);
        let o = w.obj(v).expect("still in play");
        assert_eq!((o.health, o.pos.z), (0, z - 5), "{name}");
        let (_, _, gone1) = culled_then_hit(name, true);
        assert!(gone0.is_some(), "{name}: the culled descent ends");
        assert_eq!(
            gone1, gone0,
            "{name}: leaves play at the end of the same descent"
        );
    }
    // The eligible type, cull-started, is restored to 1 health and keeps sinking.
    let mut w = world(rules());
    let skiff = at(&mut w, "SKIFF", 0, 0, 0);
    open(&mut w);
    naval::start_sinking(&mut w, skiff);
    for _ in 0..5 {
        tick::step(&mut w);
    }
    let z = w.obj(skiff).unwrap().pos.z;
    assert_eq!(kill(&mut w, skiff), HitResult::NowDead);
    let o = w.obj(skiff).unwrap();
    assert_eq!((o.health, o.pos.z), (1, z));
    assert!(naval::sinking(&w, skiff));
}

// spec: S-0312 claim 32 table row 1 (an eligible vessel already sinking: no claim 9 explosion or destruction animation and no list picks at a further killing hit)
#[test]
fn claim_32_eligible_further_hit_shows_no_vessel_animations() {
    let mut w = world(rules_with_death_anims());
    let skiff = at(&mut w, "SKIFF", 0, 0, 0);
    open(&mut w);
    kill(&mut w, skiff);
    for _ in 0..10 {
        tick::step(&mut w);
    }
    w.effects.clear();
    let probe = w.rng.clone();
    assert_eq!(kill(&mut w, skiff), HitResult::NowDead);
    assert!(anim_names(&w).is_empty(), "{:?}", anim_names(&w));
    assert_eq!(w.rng, probe, "no draws for a type without debris or crew");
    assert_eq!(w.obj(skiff).unwrap().health, 1);
}

// spec: S-0312 claims 15 and 32 (a crashable type that was sinking cannot start a crash below the surface: a killing hit removes it at the end of its effects, after claim 9's animations)
#[test]
fn claim_32_crashable_sinking_type_is_removed() {
    let mut w = world(rules_with_death_anims());
    let v = at(&mut w, "CRASHER", 0, 0, 0);
    open(&mut w);
    naval::start_sinking(&mut w, v);
    for _ in 0..5 {
        tick::step(&mut w);
    }
    w.effects.clear();
    assert_eq!(kill(&mut w, v), HitResult::NowDead);
    assert!(w.obj(v).is_none(), "removed at once");
    assert_eq!(
        anim_names(&w).len(),
        2,
        "explosion and destruction animations"
    );
}

/// The ruleset with death animations, a crashable copy of SKIFF (SKIFFC),
/// and, when `debris`, one or two debris animations on both.
fn first_hit_rules(debris: bool) -> Ruleset {
    let mut r = rules_with_death_anims();
    if debris {
        let c = &mut r.sim.vehicles.get_mut("SKIFF").unwrap().combat;
        c.min_debris = Some(1);
        c.max_debris = Some(3);
        c.debris_anims = Some(vec!["CHUNK".to_string()]);
    }
    let mut copy = r.sim.vehicles["SKIFF"].clone();
    copy.crashable = Some(true);
    r.sim.vehicles.insert("SKIFFC".into(), copy);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("SKIFFC".into());
    r
}

/// The first killing hit on `name` at height 0 on open water: the world
/// after the hit, the vessel, and the draws the hit took.
fn first_killing_hit(name: &str, debris: bool) -> (World, ObjectId, u64) {
    let mut w = world(first_hit_rules(debris));
    let v = at(&mut w, name, 0, 0, 0);
    open(&mut w);
    assert_eq!(w.height_above_surface(v), 0);
    assert!(
        naval::sinks_at_kill(&w, v),
        "{name} meets the sinking conditions"
    );
    assert!(!naval::sinking(&w, v));
    w.effects.clear();
    let before = w.rng.taken();
    assert_eq!(kill(&mut w, v), HitResult::NowDead);
    let n = w.rng.taken() - before;
    (w, v, n)
}

// spec: S-0312 claims 10 and 15 (Q-0652: a crashable surface vessel meeting the sinking conditions at its first killing hit cannot start a crash and leaves play at the end of that hit, with neither of claim 9's animations nor their list picks and no crash-start draws); S-0504 claim 4 (no descent, sinking wakes or wake draws afterwards). The non-crashable control stays in play at 1 health and descends
#[test]
fn claim_4_crashable_vessel_first_killing_hit_is_removed() {
    let (mut c, cv, cn) = first_killing_hit("SKIFFC", false);
    let (mut k, kv, kn) = first_killing_hit("SKIFF", false);
    assert!(c.obj(cv).is_none(), "removed in the killing hit");
    assert!(anim_names(&c).is_empty(), "{:?}", anim_names(&c));
    assert_eq!(cn, 0, "no list picks, no crash-start draws");
    assert_eq!(cn, kn, "the same draws as the non-crashable control");
    let o = k.obj(kv).expect("the control stays in play");
    assert_eq!(o.health, 1);
    assert!(naval::sinking(&k, kv));
    assert!(anim_names(&k).is_empty());
    let z0 = o.pos.z;
    for _ in 0..8 {
        c.effects.clear();
        let before = c.rng.taken();
        tick::step(&mut c);
        assert!(c.obj(cv).is_none());
        assert!(wakes(&c).is_empty(), "no sinking wakes");
        assert_eq!(c.rng.taken(), before, "no wake draws");
        tick::step(&mut k);
    }
    assert_eq!(k.obj(kv).unwrap().pos.z, z0 - 5 * 8, "the control descends");
}

// spec: S-0312 claims 4, 10 and 15 (Q-0652: the shared death effects that apply, here the debris and its draws, still come before the crashable vessel's removal)
#[test]
fn claim_15_crashable_vessel_first_hit_keeps_debris() {
    let (c, cv, cn) = first_killing_hit("SKIFFC", true);
    let (k, _, kn) = first_killing_hit("SKIFF", true);
    assert!(c.obj(cv).is_none());
    assert!(cn > 0, "the debris draws are taken");
    assert_eq!(cn, kn, "the same draws as the non-crashable control");
    let a = anim_names(&c);
    assert!(!a.is_empty() && a.iter().all(|n| n == "CHUNK"), "{a:?}");
    assert_eq!(anim_names(&k), a);
}

// spec: S-0312 claim 32 and S-0302 claim 8 (at 0 health a sinking vessel is out of ordinary damage: a later lethal hit registers no further destruction)
#[test]
fn claim_32_zero_health_aftermath_takes_no_further_kill() {
    let mut w = world(rules_with_death_anims());
    let v = at(&mut w, "SCOW", 0, 0, 0);
    open(&mut w);
    naval::start_sinking(&mut w, v);
    tick::step(&mut w);
    kill(&mut w, v);
    let losses = |w: &World| w.houses[0].econ.result.stats.losses;
    let l0 = losses(&w);
    w.effects.clear();
    let _ = kill(&mut w, v);
    assert_eq!(losses(&w), l0, "no second destruction at 0 health");
    assert!(anim_names(&w).is_empty());
    assert_eq!(w.obj(v).unwrap().health, 0);
}

// spec: S-0504 claim 13; S-0314 claims 9 and 10 (a sinking vessel at 0 health is not revived by self-healing)
#[test]
fn claim_13_zero_health_sinking_vessel_not_self_healed() {
    let mut r = rules();
    r.sim.vehicles.get_mut("SKIFF").unwrap().combat.self_healing = Some(true);
    let mut w = world(r);
    let skiff = at(&mut w, "SKIFF", 0, 0, 0);
    open(&mut w);
    naval::start_sinking(&mut w, skiff);
    w.obj_mut(skiff).unwrap().health = 0;
    for _ in 0..30 {
        tick::step(&mut w);
        if let Some(o) = w.obj(skiff) {
            assert_eq!(o.health, 0, "not revived at tick {}", w.tick);
        }
    }
}

// ---------------------------------------------------------------------------
// The lean of a sinking vessel (claims 15 and 16, F4; Q-1871).

/// A single-precision value from its exact decimal (S-0504 F4's values).
fn single(text: &str) -> F32 {
    glasnost_num::decimal::nearest_double(text).to_f32(Rounding::TowardZero)
}

/// `p` negated (exact).
fn neg(p: F32) -> F32 {
    p.to_f64().negated().to_f32(Rounding::TowardZero)
}

/// The full lean *L* (S-0504 F4).
fn full_lean() -> F32 {
    single("0.785398185253143310546875")
}

/// The pitch held in a vessel's naval state.
fn pitch_of(w: &World, id: ObjectId) -> F32 {
    w.obj(id).unwrap().naval.as_ref().unwrap().pitch
}

// spec: S-0504 F4 and claim 15 (Course of the lean: from level the 78th step gives |p| = 0.779999315738677978515625 under round-toward-zero and 0.779999554157257080078125 with quirks.chop_rounding off; the 79th stops at L and later steps keep it; stern down is the mirror)
#[test]
fn f4_the_lean_reaches_the_full_lean_at_the_79th_step() {
    for (mode, at_78) in [
        (Rounding::TowardZero, "0.779999315738677978515625"),
        (Rounding::Nearest, "0.779999554157257080078125"),
    ] {
        for lean in [Lean::BowDown, Lean::SternDown] {
            let sign = |p: F32| if lean == Lean::BowDown { p } else { neg(p) };
            let mut p = F32::ZERO;
            for n in 1..=78 {
                p = naval::pitch_step(p, lean, mode);
                assert_ne!(p, sign(full_lean()), "{mode:?} {lean:?} step {n}");
            }
            assert_eq!(p, sign(single(at_78)), "{mode:?} {lean:?}: the 78th step");
            p = naval::pitch_step(p, lean, mode);
            assert_eq!(
                p,
                sign(full_lean()),
                "{mode:?} {lean:?}: the 79th step stops at L"
            );
            for _ in 0..5 {
                p = naval::pitch_step(p, lean, mode);
                assert_eq!(
                    p,
                    sign(full_lean()),
                    "{mode:?} {lean:?}: the full lean holds"
                );
            }
        }
    }
}

// spec: S-0504 claim 15 (Starting value: steps start from the pitch the body had, and a pitch in the opposite sense passes through level first)
#[test]
fn claim_15_a_pitch_in_the_opposite_sense_passes_through_level() {
    let mut p = single("-0.05");
    let mut steps = 0;
    while p.to_f64().lt(F64::ZERO) {
        p = naval::pitch_step(p, Lean::BowDown, Rounding::TowardZero);
        steps += 1;
        assert!(steps < 10);
    }
    assert!(
        (5..=6).contains(&steps),
        "about five steps of 0.01 from −0.05: {steps}"
    );
    // A pitch already at the opposite full lean is not held there (the original's oddity is not copied).
    let q = naval::pitch_step(neg(full_lean()), Lean::BowDown, Rounding::TowardZero);
    assert!(q.to_f64().gt(neg(full_lean()).to_f64()));
}

// spec: S-0504 claim 15 and Constants (Direction: bow down at 8-directions 1 to 5, stern down at 0, 6 and 7, the 8-direction of S-0204 F4 rounding to the nearest step)
#[test]
fn claim_15_the_end_that_goes_down_by_eight_direction() {
    for g in 0..8u16 {
        let want = if (1..=5).contains(&g) {
            Lean::BowDown
        } else {
            Lean::SternDown
        };
        assert_eq!(naval::lean_of(g * 8192), want, "8-direction {g}");
    }
    // The boundaries of S-0204 F4: half a step either side.
    assert_eq!(naval::lean_of(4095), Lean::SternDown, "still north");
    assert_eq!(naval::lean_of(4096), Lean::BowDown, "north-east");
    assert_eq!(naval::lean_of(45055), Lean::BowDown, "still south-west");
    assert_eq!(naval::lean_of(45056), Lean::SternDown, "west");
    assert_eq!(naval::lean_of(61439), Lean::SternDown, "north-west");
    assert_eq!(naval::lean_of(61440), Lean::SternDown, "north again");
}

/// Sinks a SKIFF at body facing `facing` from height 0 with a killing hit;
/// its pitch after each descent while it stays in play, and whether every
/// own update took exactly claim 4's two ranged wake draws (on a tick number
/// divisible by 4, none otherwise), run update by update on a copy. No
/// wake animation is defined, so a wake's own creation takes no draws
/// (the two wake draws are still taken, claim 4).
fn sink_and_record(facing: u16) -> (Vec<F32>, bool) {
    let mut r = rules();
    r.sim.general.wake = None;
    let mut w = world(r);
    let skiff = testkit::spawn(&mut w, Kind::Vehicle, "SKIFF", 0, cell(0, 0), facing);
    open(&mut w);
    assert_eq!(kill(&mut w, skiff), HitResult::NowDead);
    assert_eq!(pitch_of(&w, skiff), F32::ZERO, "level at the hit");
    let mut a = w.clone();
    let mut wake_draws_only = true;
    for t in 0..80 {
        a.tick = 8 + t;
        let mut expected = a.rng.clone();
        if a.tick % 4 == 0 {
            let half = a.rules().quirks.ranged_draw_half_span;
            expected.ranged(-170, 170, half);
            expected.ranged(-170, 170, half);
        }
        assert!(!naval::sinking_turn(&mut a, skiff));
        wake_draws_only &= a.rng == expected;
    }
    let mut pitches = Vec::new();
    loop {
        tick::step(&mut w);
        if w.obj(skiff).is_none() {
            break;
        }
        pitches.push(pitch_of(&w, skiff));
        assert!(pitches.len() < 200);
    }
    assert_eq!(
        pitch_of(&a, skiff),
        pitches[79],
        "the same lean update by update"
    );
    (pitches, wake_draws_only)
}

// spec: S-0504 claim 15 (Timing: one step per descent in the same own update; from height 0 the lean is complete at the 79th descent, held at the 80th, and the vessel leaves play at the 81st; no random draw beyond claim 4's wake draws; the direction from the body facing at the first step); claim 4 and F1
#[test]
fn claim_15_the_lean_grows_with_each_descent_and_takes_no_draw() {
    for (facing, lean) in [(0u16, Lean::SternDown), (16384, Lean::BowDown)] {
        let (pitches, wake_draws_only) = sink_and_record(facing);
        assert_eq!(pitches.len(), 80, "the 81st descent removes it");
        let mut p = F32::ZERO;
        for (n, got) in pitches.iter().enumerate() {
            p = naval::pitch_step(p, lean, Rounding::TowardZero);
            assert_eq!(*got, p, "facing {facing}: after descent {}", n + 1);
        }
        let full = if lean == Lean::BowDown {
            full_lean()
        } else {
            neg(full_lean())
        };
        assert_ne!(
            pitches[77], full,
            "the 78th descent is short of the full lean"
        );
        assert_eq!(pitches[78], full, "complete at the 79th descent");
        assert_eq!(pitches[79], full, "held at the 80th");
        assert!(wake_draws_only, "facing {facing}: only the wake draws");
    }
}

// spec: S-0504 claim 15 (Turning: a body turn that continues during descent, claim 10, keeps the direction set at the first step even when it carries the facing into the other group; N16 part a in our engine)
#[test]
fn claim_15_a_continuing_turn_keeps_the_lean() {
    let mut w = world(rules());
    let skiff = testkit::spawn(&mut w, Kind::Vehicle, "SKIFF", 0, cell(0, 0), 40960);
    open(&mut w);
    // A slow turn from south-west (5) towards north (0) through west.
    let t = w.tick;
    assert!(w.obj_mut(skiff).unwrap().body.turn_to(0, t, 1));
    kill(&mut w, skiff);
    let dir = |w: &World| eight_direction(w.obj(skiff).unwrap().body.value(w.tick));
    assert_eq!(dir(&w), 5);
    let mut last = F32::ZERO;
    let mut crossed = false;
    for _ in 0..60 {
        tick::step(&mut w);
        crossed |= dir(&w) == 6 || dir(&w) == 7;
        let n = w.obj(skiff).unwrap().naval.clone().unwrap();
        assert_eq!(n.lean, Some(Lean::BowDown));
        assert!(
            n.pitch.to_f64().gt(last.to_f64()),
            "the bow keeps going down at tick {}",
            w.tick
        );
        last = n.pitch;
    }
    assert!(
        crossed,
        "the turn carried the facing into the stern-down group"
    );
}

// spec: S-0504 claims 13 and 15 (a vessel set sinking again continues with its lean and pitch kept); claim 15 (Causes and types: a start without a hit, as a cull's, leans in the same way)
#[test]
fn claim_15_a_second_start_keeps_the_lean() {
    let mut w = world(rules());
    let skiff = testkit::spawn(&mut w, Kind::Vehicle, "SKIFF", 0, cell(0, 0), 16384);
    open(&mut w);
    naval::start_sinking(&mut w, skiff);
    for _ in 0..10 {
        tick::step(&mut w);
    }
    let p = pitch_of(&w, skiff);
    let mut want = F32::ZERO;
    for _ in 0..10 {
        want = naval::pitch_step(want, Lean::BowDown, Rounding::TowardZero);
    }
    assert_eq!(p, want, "bow down, ten steps, as after a hit");
    w.obj_mut(skiff).unwrap().body = glasnost_sim::turning::Turning::at(0);
    naval::start_sinking(&mut w, skiff);
    assert_eq!(pitch_of(&w, skiff), p);
    tick::step(&mut w);
    assert_eq!(
        pitch_of(&w, skiff),
        naval::pitch_step(p, Lean::BowDown, Rounding::TowardZero),
        "still bow down"
    );
}

// spec: S-0504 claims 15 and 16 (only a sinking vehicle's pitch is drawn: one afloat reads level)
#[test]
fn claim_15_a_vessel_afloat_is_level() {
    let mut w = world(rules());
    let skiff = testkit::spawn(&mut w, Kind::Vehicle, "SKIFF", 0, cell(0, 0), 16384);
    open(&mut w);
    assert_eq!(naval::pitch(&w, skiff), F32::ZERO);
    kill(&mut w, skiff);
    tick::step(&mut w);
    assert_ne!(naval::pitch(&w, skiff), F32::ZERO);
}
