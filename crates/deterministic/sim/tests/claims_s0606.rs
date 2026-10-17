//! Claim tests of S-0606 (vehicle bunkers) on an empty synthetic map with
//! invented types: every name and number here is made up; no game data.
//! Houses Alpha (0) and Beta (1) of the testkit's sandbox game, not allied.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, LocomotionClass, UnitType, WarheadType};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::{Effect, Sound};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::bunker::{self, BunkerTrack, Stage};
use glasnost_sim::targeting;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};
use glasnost_sim::{save, testkit, tick};

const SIZE: i32 = 24;
const ALPHA: HouseId = HouseId(0);

fn dec(s: &str) -> Decimal {
    Decimal(s.into())
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn building(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut BuildingType)) {
    let mut t = BuildingType {
        strength: Some(1000),
        foundation: Some([2, 2]),
        bunker: Some(true),
        number_impassable_rows: Some(0),
        ..Default::default()
    };
    // A build-up sprite, so that the building can be sold (S-0408 claim 4).
    t.econ.buildup = Some(true);
    t.econ.buildup_frames = Some(4);
    f(&mut t);
    register(r, "buildings", name);
    r.sim.buildings.insert(name.into(), t);
}

fn vehicle(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.vehicles["CRUSHER"].clone();
    t.crusher = None;
    f(&mut t);
    register(r, "vehicles", name);
    r.sim.vehicles.insert(name.into(), t);
}

fn infantry(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.infantry["WALKER"].clone();
    f(&mut t);
    register(r, "infantry", name);
    r.sim.infantry.insert(name.into(), t);
}

/// The combat ruleset with:
/// - PIT: a 2 × 2 tank bunker (no impassable rows), PITS: PIT with a power
///   drain (it can be switched off, S-0405 claim 10);
/// - SHED: a plain 2 × 2 building;
/// - TANK: a turreted vehicle with the weapon Gun (bunker-eligible), not a
///   crusher; NOFLAG: TANK with the bunkerable flag off; NOTURRET: TANK
///   without the turret flag; HOVER: TANK of the hover class; BUGGY (the
///   testkit's): turreted, no weapon;
/// - STAY: a non-crushable walker; FLOAT: a balloon-hovering TANK;
/// - warheads PEN (penetrates bunkers) and FLAT (does not);
/// - bunker rate-of-fire multiplier 2, damage multiplier 0.5, range bonus 2;
/// - the walls-up and walls-down sounds Up and Down.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    building(&mut r, "PIT", |_| {});
    building(&mut r, "PITS", |t| t.econ.power = Some(-10));
    building(&mut r, "SHED", |t| t.bunker = None);
    vehicle(&mut r, "TANK", |_| {});
    vehicle(&mut r, "NOFLAG", |t| t.bunkerable = Some(false));
    vehicle(&mut r, "NOTURRET", |t| t.turret = None);
    vehicle(&mut r, "HOVER", |t| {
        t.locomotion_class = Some(LocomotionClass::Hover);
    });
    vehicle(&mut r, "FLOAT", |t| t.balloon_hover = Some(true));
    infantry(&mut r, "STAY", |t| t.crushable = None);
    for (name, pen) in [("PEN", true), ("FLAT", false)] {
        r.sim.warheads.insert(
            name.into(),
            WarheadType {
                verses: Some(vec![dec("1"); 11]),
                penetrates_bunker: pen.then_some(true),
                ..Default::default()
            },
        );
    }
    let g = &mut r.sim.general;
    g.bunker_rof_multiplier = Some(dec("2"));
    g.bunker_damage_multiplier = Some(dec("0.5"));
    g.bunker_weapon_range_bonus = Some(2);
    g.bunker_walls_up_sound = Some("Up".into());
    g.bunker_walls_down_sound = Some("Down".into());
    r
}

fn quirked(f: impl FnOnce(&mut glasnost_rules::quirks::Quirks)) -> Ruleset {
    let mut r = rules();
    f(&mut r.globals.quirks);
    r
}

fn game_on(r: &Ruleset) -> World {
    let mut w = testkit::game_with(r, SIZE, 7);
    testkit::run(&mut w, 2);
    w
}

fn game() -> World {
    game_on(&rules())
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, at(dx, dy), 0)
}

/// An Alpha bunker of type `name` with its top-left cell at the middle
/// cell, settled for a few ticks.
fn pit(w: &mut World, name: &str) -> ObjectId {
    let b = put(w, Kind::Building, name, 0, 0, 0);
    testkit::run(w, 3);
    b
}

fn tank(w: &mut World, dx: i32, dy: i32) -> ObjectId {
    put(w, Kind::Vehicle, "TANK", 0, dx, dy)
}

fn issue_enter(w: &mut World, u: ObjectId, b: ObjectId) {
    let house = w.obj(u).unwrap().owner.unwrap();
    orders::issue(
        w,
        house,
        Order::Behaviour {
            actor: u,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(b)),
        },
    );
}

/// The enter order on bunker `b`, applied in the next tick.
fn send(w: &mut World, u: ObjectId, b: ObjectId) {
    issue_enter(w, u, b);
    tick::step(w);
}

fn until(w: &mut World, max: i32, done: impl Fn(&World) -> bool) -> bool {
    for _ in 0..max {
        if done(w) {
            return true;
        }
        tick::step(w);
    }
    done(w)
}

fn stage(w: &World, b: ObjectId) -> Stage {
    w.obj(b).unwrap().structure.bunker_stage
}

fn inside(w: &World, u: ObjectId, b: ObjectId) -> bool {
    bunker::bunkered_in(w, u) == Some(b)
}

fn sounds(w: &World, s: Sound) -> usize {
    w.effects
        .iter()
        .filter(|e| matches!(e, Effect::Sound { sound, .. } if *sound == s))
        .count()
}

fn round_trip(w: &World) {
    let bytes = save::save(w);
    let l = save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    assert_eq!(save::state_hash(&l), save::state_hash(w));
}

fn track(w: &World, u: ObjectId) -> Option<glasnost_sim::drive::Track> {
    w.obj(u)
        .and_then(|o| o.drive.as_ref())
        .and_then(|d| d.track.clone())
}

fn cur(w: &World, u: ObjectId) -> Option<Behaviour> {
    w.obj(u).unwrap().beh.current
}

fn queued(w: &World, u: ObjectId) -> Option<Behaviour> {
    w.obj(u).unwrap().beh.queued
}

/// Sends a fresh Alpha TANK from (`dx`, `dy`) into `b` and runs until it
/// is bunkered.
fn fill(w: &mut World, b: ObjectId, dx: i32, dy: i32) -> ObjectId {
    let u = tank(w, dx, dy);
    send(w, u, b);
    assert!(until(w, 1500, |w| inside(w, u, b)), "bunkered");
    u
}

fn foundation(b: CellXY) -> [CellXY; 4] {
    [
        b,
        CellXY::new(b.x + 1, b.y),
        CellXY::new(b.x, b.y + 1),
        CellXY::new(b.x + 1, b.y + 1),
    ]
}

fn warhead(name: &str) -> u32 {
    rules().sim.warheads.keys().position(|k| k == name).unwrap() as u32
}

fn hit(w: &World, id: ObjectId, warhead_name: Option<&str>, amount: i32) -> Hit {
    let _ = w;
    let _ = id;
    Hit {
        amount,
        warhead: warhead_name.map(warhead),
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: warhead_name.is_none(),
        no_escape: false,
    }
}

/// A killing hit: FLAT on a building (S-0302 claim 8: a penetrating
/// warhead does not hurt an occupied bunker), PEN on a unit (a
/// non-penetrating one does not hurt a bunkered vehicle).
fn lethal(w: &World, id: ObjectId) -> Hit {
    let o = w.obj(id).unwrap();
    let wh = if o.kind == Kind::Building { "FLAT" } else { "PEN" };
    hit(w, id, Some(wh), o.health * 10)
}

// ---------------------------------------------------------------------------
// End to end and claims 4 to 7: the entry.

// spec: S-0606 claims 3, 5, 6 and 7 (an eligible vehicle sent into a bunker docks, the bunker runs S1 one row per 15-tick processing, and the vehicle ends bunkered at the reference point facing south, in guard, with the walls-up effects)
#[test]
fn entry_timeline_end_to_end() {
    let mut w = game();
    let b = pit(&mut w, "PIT");
    let u = tank(&mut w, 6, 1);
    send(&mut w, u, b);
    assert_eq!(w.obj(u).unwrap().contact, Some(b), "claim 3: contact taken");
    let mut changes: Vec<(i32, Stage)> = vec![(w.tick, stage(&w, b))];
    let mut effects: Vec<Effect> = Vec::new();
    let mut saved = false;
    for _ in 0..1500 {
        tick::step(&mut w);
        effects.extend(w.effects.iter().cloned());
        let s = stage(&w, b);
        if changes.last().unwrap().1 != s {
            changes.push((w.tick, s));
            if s == Stage::DrivingIn && !saved {
                round_trip(&w);
                saved = true;
            }
        }
        if inside(&w, u, b) {
            break;
        }
    }
    let order: Vec<Stage> = changes.iter().map(|c| c.1).collect();
    assert_eq!(
        order,
        [
            Stage::Clearing,
            Stage::Waiting,
            Stage::TurningIn,
            Stage::DrivingIn,
            Stage::TurningSouth,
            Stage::RaisingWalls,
            Stage::Occupied
        ],
        "S1 order"
    );
    // spec: S-0606 claim 6 (at most one row per processing, 15 ticks apart without interruption)
    for pair in changes[1..].windows(2) {
        let gap = pair[1].0 - pair[0].0;
        assert!(gap >= 15 && gap % 15 == 0, "gap {gap} in {changes:?}");
    }
    assert_eq!(changes[6].0 - changes[5].0, 15, "raising walls: always");
    // spec: S-0606 claim 7 (the link on both sides; at the reference point facing south; guard; still the contact; on the map)
    let o = w.obj(u).unwrap();
    assert_eq!(bunker::occupant(&w, b), Some(u));
    assert_eq!(o.pos.x, w.reference_point(b).x);
    assert_eq!(o.pos.y, w.reference_point(b).y);
    assert_eq!(w.body_facing(u), 32768);
    assert_eq!(o.contact, Some(b));
    assert!(o.on_map);
    assert!(matches!(cur(&w, u), Some(Behaviour::Guard)) || queued(&w, u) == Some(Behaviour::Guard));
    assert_eq!(cur(&w, b), Some(Behaviour::Repair));
    // spec: S-0606 claims 6 and 7 (special-1 and special-2 start, the bunkered notice and the walls-up sound)
    assert!(effects.iter().any(|e| matches!(e,
        Effect::SpecialAnims { building, slots: [1, 2], start: true, damaged: false } if *building == b)));
    assert!(effects.iter().any(|e| matches!(e,
        Effect::Bunkered { vehicle, bunker } if *vehicle == u && *bunker == b)));
    assert_eq!(
        effects
            .iter()
            .filter(|e| matches!(e, Effect::Sound { sound: Sound::BunkerWallsUp, .. }))
            .count(),
        1
    );
    round_trip(&w);
    // spec: S-0606 claim 7 (occupied: nothing further; the vehicle keeps guard)
    testkit::run(&mut w, 120);
    assert!(inside(&w, u, b));
    assert_eq!(cur(&w, u), Some(Behaviour::Guard));
    assert_eq!(stage(&w, b), Stage::Occupied);
}

// spec: S-0606 claim 6 (driving in is held until the entry track ends: the vehicle stands still on a foundation cell)
// spec: S-0606 table T1 and F2 (the track from the rounded body facing toward the reference point)
#[test]
fn claim_6_driving_in_waits_for_the_track() {
    let mut w = game();
    let b = pit(&mut w, "PIT");
    let u = tank(&mut w, 6, 1);
    send(&mut w, u, b);
    assert!(until(&mut w, 1500, |w| stage(w, b) == Stage::DrivingIn));
    assert!(track(&w, u).is_some(), "on the entry track");
    let tr = track(&w, u).unwrap();
    assert!(matches!(tr.bunker, Some(BunkerTrack::Entry(_))));
    assert_eq!((tr.target.0, tr.target.1), (w.reference_point(b).x, w.reference_point(b).y));
    // Held while the track runs.
    while track(&w, u).is_some() {
        assert_eq!(stage(&w, b), Stage::DrivingIn);
        tick::step(&mut w);
    }
    assert!(until(&mut w, 60, |w| stage(w, b) == Stage::TurningSouth));
}

// spec: S-0606 claim 4 and F1 (the docking point: the reference point plus the quadrant's ±128 displacement, from the vehicle's direction; a foundation cell centre on the vehicle's side)
#[test]
fn claim_4_docking_point_from_four_sides() {
    let mut w = game();
    let b = pit(&mut w, "PIT");
    let c = w.reference_point(b);
    let cells = foundation(at(0, 0));
    for (dx, dy, want) in [
        (1, -5, (128, -128)),
        (6, 1, (128, 128)),
        (0, 6, (-128, 128)),
        (-5, 0, (-128, -128)),
    ] {
        let u = tank(&mut w, dx, dy);
        let p = bunker::docking_point(&w, b, u);
        assert_eq!((p.x - c.x, p.y - c.y), want, "from ({dx}, {dy})");
        assert!(cells.contains(&p.cell()));
        assert_eq!(p.x % 256, 128, "a cell centre");
        assert_eq!(p.y % 256, 128, "a cell centre");
        w.objects.remove(&u);
    }
}

// spec: S-0606 claim 5 (arrival at the docking cell: the vehicle stops, stays in enter and queues no sleep; the bunker queues repair)
#[test]
fn claim_5_arrival() {
    let mut w = game();
    let b = pit(&mut w, "PIT");
    let u = tank(&mut w, 6, 1);
    send(&mut w, u, b);
    let dock = bunker::docking_point(&w, b, u).cell();
    assert!(until(&mut w, 1000, |w| w.obj(u).unwrap().pos.cell() == dock
        && w.obj(b).unwrap().beh.current == Some(Behaviour::Repair)));
    assert_eq!(cur(&w, u), Some(Behaviour::Enter));
    assert_ne!(queued(&w, u), Some(Behaviour::Sleep));
    assert_eq!(w.obj(u).unwrap().dest, None);
    assert_eq!(w.obj(u).unwrap().contact, Some(b));
}

// spec: S-0606 claim 7 (every foundation cell of the occupied bunker is a blocking cell)
#[test]
fn claim_7_occupied_foundation_blocks() {
    let mut w = game();
    let b = pit(&mut w, "PIT");
    let cells = foundation(at(0, 0));
    assert!(cells.iter().all(|&c| !glasnost_sim::occupancy::blocking_cell(&w, b, c)));
    fill(&mut w, b, 6, 1);
    assert!(cells.iter().all(|&c| glasnost_sim::occupancy::blocking_cell(&w, b, c)));
}

// ---------------------------------------------------------------------------
// Claims 1 to 3: who may enter.

// spec: S-0606 claim 1 (bunker-eligible: the bunkerable flag, absent on for vehicles and off for other kinds; the turret flag; a current weapon; not hover)
// spec: S-0606 Edge cases (a turreted vehicle without a turret weapon cannot enter)
#[test]
fn claim_1_eligibility() {
    let mut w = game();
    let ok = tank(&mut w, 5, 0);
    let flag = put(&mut w, Kind::Vehicle, "NOFLAG", 0, 5, 2);
    let tur = put(&mut w, Kind::Vehicle, "NOTURRET", 0, 5, 4);
    let hov = put(&mut w, Kind::Vehicle, "HOVER", 0, 5, 6);
    let bug = put(&mut w, Kind::Vehicle, "BUGGY", 0, 5, 8);
    let walker = put(&mut w, Kind::Infantry, "WALKER", 0, -5, 0);
    assert!(bunker::eligible(&w, ok));
    assert!(!bunker::eligible(&w, flag), "bunkerable off");
    assert!(!bunker::eligible(&w, tur), "no turret");
    assert!(!bunker::eligible(&w, hov), "hover");
    assert!(!bunker::eligible(&w, bug), "no turret weapon");
    assert!(!bunker::eligible(&w, walker), "infantry: the flag is off by default");
}

// spec: S-0606 claim 2 (rows P1, P2, P6, P7, P8, P9 and P10; first match wins)
#[test]
fn claim_2_permission_rows() {
    let mut w = game();
    let b = pit(&mut w, "PIT");
    let ok = tank(&mut w, 5, 0);
    let foe = put(&mut w, Kind::Vehicle, "TANK", 1, 5, 4);
    let float = put(&mut w, Kind::Vehicle, "FLOAT", 0, -5, 4);
    let bug = put(&mut w, Kind::Vehicle, "BUGGY", 0, -5, -4);
    assert!(bunker::permits(&w, b, ok), "P10");
    assert!(!bunker::permits(&w, b, foe), "P1");
    assert!(!bunker::permits(&w, b, float), "P6");
    assert!(!bunker::permits(&w, b, bug), "P8");
    // P9: standing on a foundation cell.
    let on = tank(&mut w, 0, 0);
    w.obj_mut(on).unwrap().pos = Pos::centre(at(1, 1), 0);
    assert!(!bunker::permits(&w, b, on), "P9");
    w.objects.remove(&on);
    // P7: switched off refuses; low power alone does not.
    let s = put(&mut w, Kind::Building, "PITS", 0, -8, -8);
    testkit::run(&mut w, 3);
    assert!(bunker::permits(&w, s, ok), "switched on, any power");
    orders::issue(&mut w, ALPHA, Order::PowerOff { actor: s });
    tick::step(&mut w);
    assert!(w.obj(s).unwrap().econ.building.switched_off);
    assert!(!bunker::permits(&w, s, ok), "P7");
    // P2: selling.
    glasnost_sim::econ::sell::sell(&mut w, ALPHA, b);
    tick::step(&mut w);
    assert!(!bunker::permits(&w, b, ok), "P2");
    // Not a bunker: no permission of its own.
    let shed = put(&mut w, Kind::Building, "SHED", 0, 8, -8);
    assert!(!bunker::permits(&w, shed, ok));
}

// spec: S-0606 claim 2 row P4 and claim 3 (one contact place: a second vehicle given the bunker as destination has it cleared and gets no contact; no waiting queue)
#[test]
fn claim_3_second_vehicle_is_turned_away() {
    let mut w = game();
    let b = pit(&mut w, "PIT");
    let a = tank(&mut w, 6, 1);
    let c = tank(&mut w, -5, 1);
    send(&mut w, a, b);
    assert_eq!(w.obj(a).unwrap().contact, Some(b));
    assert!(!bunker::permits(&w, b, c), "P4");
    send(&mut w, c, b);
    assert_eq!(w.obj(c).unwrap().contact, None);
    assert_eq!(w.obj(c).unwrap().dest, None);
    round_trip(&w);
    assert!(until(&mut w, 1500, |w| inside(w, a, b)));
    assert!(!inside(&w, c, b));
    assert_eq!(bunker::occupant(&w, b), Some(a));
}

// spec: S-0606 claim 3 (the bunker answers a switched-off refusal; the entering vehicle's contact ends and its destination is cleared)
#[test]
fn claim_3_switched_off_refuses_the_request() {
    let mut w = game();
    let s = pit(&mut w, "PITS");
    let u = tank(&mut w, 6, 1);
    orders::issue(&mut w, ALPHA, Order::PowerOff { actor: s });
    tick::step(&mut w);
    send(&mut w, u, s);
    assert_eq!(w.obj(u).unwrap().contact, None);
    assert_eq!(w.obj(u).unwrap().dest, None);
}

// ---------------------------------------------------------------------------
// Claim 6: clearing the foundation, both quirk positions.

// spec: S-0606 claim 6 (clearing: a forced scatter request to the object on a foundation cell; waiting holds until it has gone)
#[test]
fn claim_6_clearing_scatters_a_blocker() {
    let mut w = game();
    let b = pit(&mut w, "PIT");
    let s = put(&mut w, Kind::Infantry, "STAY", 0, 0, 1);
    testkit::run(&mut w, 2);
    let u = tank(&mut w, 6, 1);
    send(&mut w, u, b);
    assert!(until(&mut w, 1500, |w| inside(w, u, b)));
    let cells = foundation(at(0, 0));
    assert!(!cells.contains(&w.obj(s).unwrap().pos.cell()), "scattered off the foundation");
}

// spec: S-0606 Edge cases (quirks.bunker_clear_nearest_only on: the vehicle is the nearest object in its docking cell, a second object there is neither asked to scatter nor waited for; off: it is asked to scatter and waited for)
#[test]
fn claim_6_nearest_only_both_positions() {
    for nearest in [true, false] {
        let r = quirked(|q| q.bunker_clear_nearest_only = nearest);
        let mut w = game_on(&r);
        let b = pit(&mut w, "PIT");
        let u = tank(&mut w, 6, 1);
        send(&mut w, u, b);
        // Arrived at the docking cell, before the clearing row has run.
        let dock = bunker::docking_point(&w, b, u);
        assert!(until(&mut w, 1000, |w| cur(w, b) == Some(Behaviour::Repair)
            && w.obj(u).unwrap().pos == dock));
        assert_eq!(stage(&w, b), Stage::Clearing);
        // The second object stands in the docking cell, off the centre the
        // vehicle stands on.
        let s = testkit::spawn(&mut w, Kind::Infantry, "STAY", 0, dock.cell(), 0);
        let near = Pos::new(dock.x - 64, dock.y - 64, dock.z);
        w.obj_mut(s).unwrap().pos = near;
        assert_ne!(near, w.obj(u).unwrap().pos);
        assert!(until(&mut w, 100, |w| stage(w, b) == Stage::Waiting));
        // Watched until the vehicle starts its entry track (which may then
        // push the object aside by the ordinary movement rules).
        let mut moved = false;
        for _ in 0..200 {
            tick::step(&mut w);
            if !matches!(stage(&w, b), Stage::Waiting | Stage::TurningIn) {
                break;
            }
            moved |= w.obj(s).unwrap().pos != near;
        }
        if nearest {
            assert!(!moved, "on: the second object is not asked");
            assert_eq!(stage(&w, b), Stage::DrivingIn, "on: not waited for");
        } else {
            assert!(moved, "off: the second object is asked to scatter");
        }
        round_trip(&w);
    }
}

// ---------------------------------------------------------------------------
// Claim 8: the clear report.

// spec: S-0606 claim 8 (within 384 leptons, floating point truncated: accepting answer, contact kept; otherwise contact ends with the run-away reply)
#[test]
fn claim_8_clear_report() {
    let mut w = game();
    let b = pit(&mut w, "PIT");
    let u = tank(&mut w, 6, 1);
    let c = w.reference_point(b);
    w.obj_mut(u).unwrap().contact = Some(b);
    // 383 leptons: kept.
    w.obj_mut(u).unwrap().pos = Pos::new(c.x + 383, c.y, 0);
    assert!(bunker::clear_report(&mut w, b, u));
    assert_eq!(w.obj(u).unwrap().contact, Some(b));
    // (271, 272): 383.95 truncated to 383: kept.
    w.obj_mut(u).unwrap().pos = Pos::new(c.x + 271, c.y + 272, 0);
    assert!(bunker::clear_report(&mut w, b, u));
    // 384: contact ends.
    w.obj_mut(u).unwrap().pos = Pos::new(c.x + 384, c.y, 0);
    assert!(!bunker::clear_report(&mut w, b, u));
    assert_eq!(w.obj(u).unwrap().contact, None);
}

// ---------------------------------------------------------------------------
// Claims 9 to 11: while bunkered.

// spec: S-0606 claim 11 (band-box selection excludes the bunkered vehicle and the occupied bunker; click selection is not refused by the link; the vehicle is not movable)
// spec: S-0214 claim 19 and Terms (movable)
#[test]
fn claim_11_selection() {
    let mut w = game();
    let b = pit(&mut w, "PIT");
    let free = tank(&mut w, 6, 6);
    let u = fill(&mut w, b, 6, 1);
    assert!(targeting::box_eligible(&w, ALPHA, &[], free));
    assert!(!targeting::box_eligible(&w, ALPHA, &[], u), "bunkered vehicle");
    assert!(!targeting::box_eligible(&w, ALPHA, &[], b), "occupied bunker");
    assert!(targeting::selectable_by(&w, u, Some(ALPHA)), "click selection stays");
    assert!(targeting::movable(&w, ALPHA, free));
    assert!(!targeting::movable(&w, ALPHA, u));
}

// spec: S-0606 claim 7 (the vehicle keeps guard: S-0215 claim 15 row 12 returns a bunkered vehicle to guard, not area guard)
// spec: S-0215 claim 15 row 12 and claim 26 row 2
#[test]
fn claim_7_keeps_guard() {
    let mut w = game();
    let b = pit(&mut w, "PIT");
    let u = fill(&mut w, b, 6, 1);
    // An area-guard turn returns it to guard.
    {
        let o = w.obj_mut(u).unwrap();
        o.beh.current = Some(Behaviour::AreaGuard);
        o.beh.queued = None;
    }
    testkit::run(&mut w, 60);
    assert_eq!(cur(&w, u), Some(Behaviour::Guard));
    assert!(inside(&w, u, b));
}

// spec: S-0606 claim 9 and S-0307 claim 5 (the range of a bunkered firer gains 256 × the bonus leptons: a cell 6.5 cells away is in range only when bunkered)
#[test]
fn claim_9_range_bonus() {
    let mut w = game();
    let b = pit(&mut w, "PIT");
    let u = fill(&mut w, b, 6, 1);
    let free = tank(&mut w, -8, -8);
    testkit::run(&mut w, 2);
    let reach = |w: &World, id: ObjectId| {
        let p = w.obj(id).unwrap().pos;
        let c = CellXY::new((p.x + 6 * 256 + 128) / 256, p.y / 256);
        let wpn = glasnost_sim::combat::weapon::slot_weapon(w, id, 0);
        glasnost_sim::combat::range::in_range(w, id, wpn, Some(Target::Cell(c)), None)
    };
    assert!(reach(&w, u), "bunkered: 5.5 + 2 cells");
    assert!(!reach(&w, free), "open ground: 5.5 cells");
}

// spec: S-0606 claim 9 and S-0302 claim 8 (the hit table's bunker rows: a non-penetrating warhead does not hurt the bunkered vehicle; a penetrating one hurts the occupied bunker; ignoring defences skips both rows)
#[test]
fn claim_9_bunker_hit_rows() {
    let mut w = game();
    let b = pit(&mut w, "PIT");
    let u = fill(&mut w, b, 6, 1);
    let (hu, hb) = (w.obj(u).unwrap().health, w.obj(b).unwrap().health);
    let h = hit(&w, u, Some("FLAT"), 50);
    assert_eq!(damage::apply(&mut w, u, h), damage::HitResult::Unaffected);
    assert_eq!(w.obj(u).unwrap().health, hu);
    let h = hit(&w, b, Some("PEN"), 50);
    assert_eq!(damage::apply(&mut w, b, h), damage::HitResult::Unaffected);
    assert_eq!(w.obj(b).unwrap().health, hb);
    let h = hit(&w, u, Some("PEN"), 50);
    damage::apply(&mut w, u, h);
    assert!(w.obj(u).unwrap().health < hu);
    let h = hit(&w, b, Some("FLAT"), 50);
    damage::apply(&mut w, b, h);
    assert!(w.obj(b).unwrap().health < hb);
}

// spec: S-0606 claim 9 and S-0601 claim 3 (ordinary mind control cannot take over a bunkered vehicle)
#[test]
fn claim_9_no_mind_control() {
    let mut w = game();
    let b = pit(&mut w, "PIT");
    let u = fill(&mut w, b, 6, 1);
    let ctl = put(&mut w, Kind::Vehicle, "TANK", 1, -6, -6);
    assert!(!glasnost_sim::mind::take(&mut w, ctl, u));
    assert_eq!(w.obj(u).unwrap().owner, Some(ALPHA));
}

// ---------------------------------------------------------------------------
// Claims 12 to 14: leaving.

fn deploy(w: &mut World, u: ObjectId) {
    orders::issue(w, ALPHA, Order::Deploy { actor: u });
    tick::step(w);
}

// spec: S-0606 claim 12 (unload: walls-down effects; the release track toward F4; the nearby-cell destination and move; the bunker back at clearing with guard queued and no contact)
// spec: S-0606 table T2 (the release track ends at F4 facing 192)
#[test]
fn claim_12_unload_release() {
    let mut w = game();
    let b = pit(&mut w, "PIT");
    let u = fill(&mut w, b, 6, 1);
    let c = w.reference_point(b);
    orders::issue(&mut w, ALPHA, Order::Deploy { actor: u });
    let mut fx: Vec<Effect> = Vec::new();
    let mut on_track = false;
    let mut track_end: Option<(Pos, u16)> = None;
    for _ in 0..200 {
        tick::step(&mut w);
        fx.extend(w.effects.iter().cloned());
        let tracked = track(&w, u).is_some_and(|t| t.bunker == Some(BunkerTrack::Release));
        if tracked {
            on_track = true;
        } else if on_track && track_end.is_none() {
            track_end = Some((w.obj(u).unwrap().pos, w.body_facing(u)));
        }
        if track_end.is_some() {
            break;
        }
    }
    assert!(on_track, "put on the release track");
    let (p, f) = track_end.expect("track ended");
    assert_eq!((p.x - c.x, p.y - c.y), (-128, 128), "F4");
    assert_eq!(f, 192 * 256, "T2 ends facing 192");
    assert!(!bunker::bunkered(&w, u));
    assert_eq!(bunker::occupant(&w, b), None);
    assert_eq!(stage(&w, b), Stage::Clearing);
    assert_ne!(w.obj(u).unwrap().contact, Some(b));
    // Walls-down: special-1/2 stop, sound, special-3/4 start, in that order.
    let i_stop = fx.iter().position(|e| matches!(e, Effect::SpecialAnims { slots: [1, 2], start: false, .. })).expect("stop");
    let i_snd = fx.iter().position(|e| matches!(e, Effect::Sound { sound: Sound::BunkerWallsDown, .. })).expect("sound");
    let i_go = fx.iter().position(|e| matches!(e, Effect::SpecialAnims { slots: [3, 4], start: true, .. })).expect("start");
    assert!(i_stop < i_snd && i_snd < i_go);
    // Then it moves to the nearby-cell result, off the foundation.
    testkit::run(&mut w, 400);
    let cells = foundation(at(0, 0));
    assert!(!cells.contains(&w.obj(u).unwrap().pos.cell()), "left the foundation");
    round_trip(&w);
    // The bunker can take a vehicle again.
    assert!(bunker::permits(&w, b, u));
}

// spec: S-0606 claim 13 and S2 (destroyed: the occupant is let out within the hit, onto the release track, keeps guard, no walls-down effects; it is not hurt by S-0312 claim 17)
#[test]
fn claim_13_destroyed_bunker_lets_out() {
    let mut w = game();
    let b = pit(&mut w, "PIT");
    let u = fill(&mut w, b, 6, 1);
    let c = w.reference_point(b);
    let hu = w.obj(u).unwrap().health;
    w.effects.clear();
    let h = lethal(&w, b);
    damage::apply(&mut w, b, h);
    assert!(!bunker::bunkered(&w, u), "within the hit");
    assert_eq!(w.obj(u).unwrap().contact, None);
    assert!(track(&w, u).is_some_and(|t| t.bunker == Some(BunkerTrack::Release)));
    assert_eq!(sounds(&w, Sound::BunkerWallsDown), 0, "no walls-down effects");
    testkit::run(&mut w, 80);
    let o = w.obj(u).expect("alive");
    assert_eq!(o.health, hu, "not hurt by the demolition");
    assert_eq!((o.pos.x - c.x, o.pos.y - c.y), (-128, 128));
    assert!(matches!(o.beh.current, Some(Behaviour::Guard | Behaviour::AreaGuard)));
}

// spec: S-0606 claim 13 (a vehicle still entering, contact and no link, at a docking-cell centre is within 256 leptons and gets S-0312 claim 17's demolition damage)
// spec: S-0312 claim 17
#[test]
fn claim_13_entering_contact_is_hurt() {
    let mut w = game();
    let b = pit(&mut w, "PIT");
    let u = tank(&mut w, 6, 1);
    send(&mut w, u, b);
    assert!(until(&mut w, 1500, |w| stage(w, b) == Stage::Waiting));
    let hu = w.obj(u).unwrap().health;
    let h = lethal(&w, b);
    damage::apply(&mut w, b, h);
    testkit::run(&mut w, 3);
    assert!(w.obj(u).is_none_or(|o| o.health < hu), "demolition damage");
}

// spec: S-0606 claim 13 (sold: let out at the first selling processing, S-0408 claim 6)
#[test]
fn claim_13_sold_bunker_lets_out() {
    let mut w = game();
    let b = pit(&mut w, "PIT");
    let u = fill(&mut w, b, 6, 1);
    glasnost_sim::econ::sell::sell(&mut w, ALPHA, b);
    assert!(until(&mut w, 60, |w| !bunker::bunkered(w, u)));
    assert!(track(&w, u).is_some_and(|t| t.bunker == Some(BunkerTrack::Release)));
    round_trip(&w);
}

// spec: S-0606 claim 14 and S2 (the bunkered vehicle destroyed: walls-down effects before its death effects; link and contact end; the bunker back at clearing with guard queued)
#[test]
fn claim_14_vehicle_death_releases_in_place() {
    let mut w = game();
    let b = pit(&mut w, "PIT");
    let u = fill(&mut w, b, 6, 1);
    w.effects.clear();
    let h = lethal(&w, u);
    damage::apply(&mut w, u, h);
    assert_eq!(bunker::occupant(&w, b), None);
    assert_eq!(stage(&w, b), Stage::Clearing);
    assert_eq!(queued(&w, b), Some(Behaviour::Guard));
    let i_snd = w.effects.iter().position(|e| matches!(e, Effect::Sound { sound: Sound::BunkerWallsDown, .. })).expect("walls down");
    // Before the death effects (the death animations and detonations);
    // the killed marking itself comes first (S-0312).
    if let Some(i_death) = w
        .effects
        .iter()
        .position(|e| matches!(e, Effect::Anim { .. } | Effect::Detonation { .. }))
    {
        assert!(i_snd < i_death, "walls down before the death effects");
    }
    round_trip(&w);
}

// spec: S-0606 claim 15 (the bunker changes owner: the link holds, the vehicle stays bunkered under its own house)
// spec: S-0509 claim 1
#[test]
fn claim_15_capture_keeps_the_link() {
    let mut w = game();
    let b = pit(&mut w, "PIT");
    let u = fill(&mut w, b, 6, 1);
    glasnost_sim::structures::ownership::change(
        &mut w,
        b,
        Some(HouseId(1)),
        glasnost_sim::structures::ownership::Cause::Capture,
    );
    testkit::run(&mut w, 30);
    assert!(inside(&w, u, b));
    assert_eq!(w.obj(u).unwrap().owner, Some(ALPHA));
    assert_eq!(w.obj(b).unwrap().owner, Some(HouseId(1)));
    round_trip(&w);
}

// ---------------------------------------------------------------------------
// Tables.

// spec: S-0606 table T1 (16 points per row; rows E1 to E4 begin at (−120, 120), (−120, −120), (120, −120), (120, 120) with facings 32, 96, 160 and 224; every row ends at (0, 0))
// spec: S-0606 table T2 (17 points from (128, −128) facing 128 to (0, 0) facing 192)
#[test]
fn tables_t1_t2() {
    for (row, first, f) in [
        (1u8, (-120, 120), 32u8),
        (2, (-120, -120), 96),
        (3, (120, -120), 160),
        (4, (120, 120), 224),
    ] {
        let p = bunker::track_points(BunkerTrack::Entry(row));
        assert_eq!(p.len(), 16);
        assert_eq!((p[0].0, p[0].1), first, "row E{row}");
        assert!(p.iter().all(|q| q.2 == f));
        assert_eq!((p[15].0, p[15].1), (0, 0));
    }
    let t = bunker::track_points(BunkerTrack::Release);
    assert_eq!(t.len(), 17);
    assert_eq!(t[0], (128, -128, 128));
    assert_eq!(t[9], (84, -32, 164));
    assert_eq!(t[16], (0, 0, 192));
}
