//! Claim tests of S-0506 revision 37 (Q-2748 in part and Q-2749 answered).
//! A rocket missile launched at a controller target in the air is given no
//! destination and makes no flight: it waits at its placement point until a
//! destination not in the air is given to it, which fixes its aim point
//! from that destination's heading point at that moment and starts claim
//! 24's timer then; a destination in the air is ignored, and the missile
//! never follows the target (claims 7, 14 and 24; scenario R26). The
//! controller target is lost at the hit that destroys it, even while it
//! stays in play, and when it starts to cloak, whatever the spawner's house
//! and sensors; it is cleared before the reset, so a missile the reset
//! releases gets the neighbouring cell of F8 (claims 11 and 12; scenarios
//! R7 parts c and d, R21's lost-target variant). Which destination such a
//! waiting missile is first given is Q-2748's questions 2 and 3, still
//! open; our engine's interim gives it its release cell at the shared
//! re-order (`spawn::kamikaze_orders`). On synthetic data: invented types,
//! weapons and rocket parameter values, none of them stock values.

use glasnost_num::F64;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    Decimal, LocomotionClass, Locomotor, SequenceEntry, UnitType, WarheadType, WeaponType,
};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::combat::weapon::in_air;
use glasnost_sim::rocket::{self, Phase};
use glasnost_sim::rules::Kind;
use glasnost_sim::spawn::{self, ControllerState, SlotState};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Pos, Target, World};
use glasnost_sim::{cloak, objects};

const SIZE: i32 = 40;
/// The height a flying test aircraft is held at, in leptons.
const UP: i32 = 600;

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

/// Rocket types RKA (set A) and RKB (set B); launchers LNA (one RKA slot)
/// and LNC (four RKB slots, so that the 9-tick launch spacing keeps the
/// first missile preparing for 27 ticks) with the spawner weapon Launch;
/// the target vehicle VICTIM, the aircraft FLYER, the unarmed vehicles
/// PLAIN (no cloak flag of its own) and SEEKER (sensors, sight 3); the
/// testkit's WALKER. Invented set parameters (set A: pause 4, tilt 10; set
/// B: pause 3, tilt 8) and cloak settings (8 stages, a delay of 0.01).
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let push = |r: &mut Ruleset, reg: &str, n: &str| {
        r.registries.entry(reg.into()).or_default().push(n.into());
    };
    for n in ["RKA", "RKB", "FLYER"] {
        push(&mut r, "aircraft", n);
    }
    for n in ["LNA", "LNC", "VICTIM", "PLAIN", "SEEKER"] {
        push(&mut r, "vehicles", n);
    }
    let rocket = |speed| UnitType {
        strength: Some(40),
        speed: Some(speed),
        rot: Some(20),
        missile_spawn: Some(true),
        spawned: Some(true),
        locomotor: Some(Locomotor::Rocket),
        ..Default::default()
    };
    let launcher = |spawns: &str, n| {
        let mut t = UnitType {
            strength: Some(400),
            speed: Some(6),
            rot: Some(5),
            locomotion_class: Some(LocomotionClass::Wheel),
            locomotor: Some(Locomotor::Drive),
            primary: Some("Launch".into()),
            missile_spawn: Some(true),
            ..Default::default()
        };
        t.combat.spawns = Some(spawns.into());
        t.combat.spawns_number = Some(n);
        t.combat.spawn_regen_rate = Some(50);
        t.combat.spawn_reload_rate = Some(0);
        t.combat.primary_fire_flh = Some([60, 0, 80]);
        t
    };
    let vehicle = || UnitType {
        strength: Some(5000),
        speed: Some(4),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        ..Default::default()
    };
    let s = &mut r.sim;
    s.aircraft.insert("RKA".into(), rocket(50));
    s.aircraft.insert("RKB".into(), rocket(40));
    s.aircraft.insert(
        "FLYER".into(),
        UnitType {
            strength: Some(5000),
            speed: Some(8),
            rot: Some(5),
            locomotor: Some(Locomotor::Fly),
            ..Default::default()
        },
    );
    s.vehicles.insert("LNA".into(), launcher("RKA", 1));
    s.vehicles.insert("LNC".into(), launcher("RKB", 4));
    s.vehicles.insert("VICTIM".into(), vehicle());
    let mut plain = vehicle();
    plain.combat.cloaking_speed = Some(2);
    s.vehicles.insert("PLAIN".into(), plain);
    let mut seeker = vehicle();
    seeker.combat.sensors = Some(true);
    seeker.combat.sensors_sight = Some(3);
    s.vehicles.insert("SEEKER".into(), seeker);
    s.weapons.insert(
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
    let g = &mut s.general;
    g.v3_rocket_type = Some("RKA".into());
    g.d_misl_type = Some("RKB".into());
    g.v3_rocket_pause_frames = Some(4);
    g.v3_rocket_tilt_frames = Some(10);
    g.v3_rocket_altitude = Some(600);
    g.v3_rocket_pitch_final = dec("0.6");
    g.v3_rocket_acceleration = dec("3");
    g.v3_rocket_damage = Some(300);
    g.d_misl_pitch_final = dec("0.5");
    g.d_misl_turn_rate = dec("0.2");
    g.d_misl_pause_frames = Some(3);
    g.d_misl_tilt_frames = Some(8);
    g.d_misl_altitude = Some(500);
    g.d_misl_acceleration = dec("2.5");
    g.d_misl_damage = Some(200);
    g.v3_warhead = Some("HE".into());
    g.d_misl_warhead = Some("HE".into());
    g.cloaking_stages = Some(8);
    g.cloak_delay = dec("0.01");
    // WALKER's death sequence: die-1 and die-2 of 4 frames, and warhead W1
    // of infantry death kind 1 (a die-1 request, S-0107 claim 17).
    if let Some(t) = s.infantry.get_mut("WALKER") {
        for k in ["stand", "die_1", "die_2"] {
            t.sequences.insert(
                k.into(),
                SequenceEntry {
                    start: 0,
                    count: 4,
                    multiplier: 0,
                    direction: None,
                },
            );
        }
    }
    s.warheads.insert(
        "W1".into(),
        WarheadType {
            verses: Some((0..11).map(|_| Decimal("1".into())).collect()),
            inf_death: Some(1),
            ..Default::default()
        },
    );
    r
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, testkit::mid(SIZE, dx, dy), 0)
}

fn controller(w: &World, id: ObjectId) -> spawn::SpawnController {
    w.obj(id).unwrap().spawn.clone().unwrap()
}

fn state(w: &World, x: ObjectId) -> rocket::RocketState {
    w.obj(x).unwrap().rocket.clone().unwrap()
}

/// Holds aircraft `f` at [`UP`] leptons above its cell.
fn hold_up(w: &mut World, f: Option<ObjectId>) {
    if let Some(o) = f.and_then(|f| w.obj_mut(f)) {
        o.pos = Pos::new(o.pos.x, o.pos.y, UP);
    }
}

/// Shoots `launcher` at `target` and runs until its slot 0 object is
/// placed on the map, holding `up` in the air before each tick; returns
/// that missile and the launch tick.
fn launch(
    w: &mut World,
    launcher: ObjectId,
    target: ObjectId,
    up: Option<ObjectId>,
) -> (ObjectId, i32) {
    let x = controller(w, launcher).slots[0].object.unwrap();
    spawn::shot(w, launcher, Target::Object(target));
    for _ in 0..200 {
        hold_up(w, up);
        testkit::run(w, 1);
        if w.obj(x).is_some_and(|o| o.on_map) {
            return (x, w.tick - 1);
        }
    }
    panic!("no launch");
}

fn near(a: Pos, b: Pos) -> bool {
    (a.x - b.x).abs() < 256 && (a.y - b.y).abs() < 256
}

/// The F8 neighbour of missile `x`'s own cell for its body facing.
fn f8_cell(w: &World, x: ObjectId) -> glasnost_map::CellXY {
    let o = w.obj(x).unwrap();
    let k = spawn::release_direction(o.body.value(w.tick));
    let (dx, dy) = glasnost_sim::regions::STEPS[k];
    let c = o.pos.cell();
    glasnost_map::CellXY::new(c.x + dx, c.y + dy)
}

fn release_cell(w: &World, x: ObjectId) -> Option<glasnost_map::CellXY> {
    w.spawned.kamikaze.iter().find(|e| e.0 == x).map(|e| e.1)
}

/// LNA at FLYER, held in the air at the launch: the world at the end of
/// the launch tick, the missile and the launch tick.
fn at_a_flyer() -> (World, ObjectId, ObjectId, i32) {
    let r = rules();
    let mut w = testkit::game_with(&r, SIZE, 7);
    let lna = put(&mut w, Kind::Vehicle, "LNA", 0, 0, 0);
    let flyer = put(&mut w, Kind::Aircraft, "FLYER", 1, 8, 0);
    hold_up(&mut w, Some(flyer));
    assert!(in_air(&w, flyer), "set-up: the target is in the air");
    let (x, t0) = launch(&mut w, lna, flyer, Some(flyer));
    assert!(in_air(&w, flyer), "in the air in the launch tick");
    (w, x, flyer, t0)
}

// spec: S-0506 claim 7 (r37: a controller target whose in-the-air answer is true gives the missile no destination at all, neither that target nor any point of its own) and claim 14 (A controller target in the air at the launch: no aim point and no flight; it stays at its placement point, in none of claim 15's phases, with speed 0 and pitch 0; for a set-A type its own in-the-air answer is false, claim 22); the Rocket table's first row; scenario R26 (a)
#[test]
fn r26_a_no_destination_and_no_flight_for_a_target_in_the_air() {
    let (w, x, _, _) = at_a_flyer();
    let o = w.obj(x).unwrap();
    assert_eq!(o.dest, None, "no destination");
    let rs = state(&w, x);
    assert_eq!(rs.phase, None, "no phase");
    assert_eq!(rs.aim, None, "no aim point");
    assert!(!rs.speed.gt(F64::ZERO), "speed 0");
    assert_eq!(rs.pitch, glasnost_num::F32::ZERO, "pitch 0");
    assert!(
        !in_air(&w, x),
        "a set-A type before its flight is not in the air"
    );
    // It waits at its placement point while it has no destination.
    let placed = o.pos;
    let mut v = w.clone();
    assert!(!rocket::tick(&mut v, x), "no update moves it");
    assert_eq!(v.obj(x).unwrap().pos, placed);
    assert_eq!(state(&v, x).phase, None);
}

// spec: S-0506 claim 14 (r37: a later destination whose in-the-air answer is true leaves it without a destination, so a destination in the air never fixes an aim point; the first destination later given whose in-the-air answer is false fixes its aim point from its heading point at that moment) and claim 24 (r37: t₀ is the tick of that fixing); the Rocket table's second and third rows
#[test]
fn r26_the_first_destination_not_in_the_air_fixes_the_aim_then() {
    let (mut w, x, flyer, _) = at_a_flyer();
    w.tick += 5;
    rocket::give_destination(&mut w, x, Target::Object(flyer));
    assert_eq!(
        state(&w, x).phase,
        None,
        "a destination in the air is ignored"
    );
    assert_eq!(state(&w, x).aim, None);
    assert_eq!(w.obj(x).unwrap().dest, None);
    let c = testkit::mid(SIZE, 4, 6);
    let t = w.tick;
    rocket::give_destination(&mut w, x, Target::Cell(c));
    let rs = state(&w, x);
    let aim = rocket::heading_point(&w, Target::Cell(c)).unwrap();
    assert_eq!(rs.aim, Some(aim), "from the heading point at that moment");
    assert_eq!(rs.phase, Some(Phase::Pause), "the table's fixing row");
    assert_eq!(rs.started, t, "claim 24's t₀: the tick of the fixing");
    // spec: S-0506 claim 14 (later move orders are then ignored as for any rocket missile)
    rocket::give_destination(&mut w, x, Target::Cell(testkit::mid(SIZE, -4, -6)));
    assert_eq!(state(&w, x).aim, Some(aim), "fixed: a later one is ignored");
}

// spec: S-0506 claim 14 (r37: the missile does not follow the controller target: the target landing does not itself give it a destination or fix its aim point); scenario R26 (b)
#[test]
fn r26_b_the_target_landing_gives_the_missile_nothing() {
    let (mut w, x, flyer, _) = at_a_flyer();
    let o = w.obj_mut(flyer).unwrap();
    o.pos = Pos::new(o.pos.x, o.pos.y, 0);
    assert!(!in_air(&w, flyer), "landed");
    let mut v = w.clone();
    assert!(!rocket::tick(&mut v, x));
    assert_eq!(state(&v, x).phase, None, "no flight from the landing");
    assert_eq!(v.obj(x).unwrap().dest, None);
}

// spec: S-0506 claim 14 (r37: a controller target not in the air at the launch: the aim point is fixed at the launch toward its heading point); scenario R26 (d), the control
#[test]
fn r26_d_a_landed_aircraft_target_fixes_the_aim_at_the_launch() {
    let r = rules();
    let mut w = testkit::game_with(&r, SIZE, 7);
    let lna = put(&mut w, Kind::Vehicle, "LNA", 0, 0, 0);
    let flyer = put(&mut w, Kind::Aircraft, "FLYER", 1, 8, 0);
    assert!(!in_air(&w, flyer), "set-up: standing landed");
    let aim = w.obj(flyer).unwrap().pos;
    let (x, t0) = launch(&mut w, lna, flyer, None);
    let rs = state(&w, x);
    assert_eq!(rs.phase, Some(Phase::Pause));
    assert_eq!(rs.started, t0);
    assert!(near(rs.aim.unwrap(), aim));
    assert_eq!(w.obj(x).unwrap().dest, Some(Target::Object(flyer)));
}

// design: Q-2748 interim (questions 2 and 3 open: which destination not in the air a missile launched at a target in the air is first given, and when; our engine gives it its release cell, S-0506 claim 12, at the shared re-order cycle's attack on that cell, claim 26, so it does not stand without a flight); spec: S-0506 claim 14 (r37: that destination fixes the aim point then, from its heading point, and claim 24's ticks count from that tick; the aim point does not move afterwards) and claim 19 (it detonates there)
#[test]
fn q2748_interim_the_release_cell_given_at_the_re_order_fixes_the_flight() {
    let (mut w, x, flyer, t0) = at_a_flyer();
    let cell = release_cell(&w, x).expect("released onto the rail with the target's cell");
    assert_eq!(cell, w.obj(flyer).unwrap().pos.cell());
    let mut fixed = None;
    for _ in 0..10 {
        let t = w.tick;
        testkit::run(&mut w, 1);
        if fixed.is_none() && state(&w, x).phase.is_some() {
            fixed = Some(t);
        }
    }
    let t = fixed.expect("fixed at the re-order");
    assert!(t > t0, "not at the launch");
    let rs = state(&w, x);
    let aim = rocket::heading_point(&w, Target::Cell(cell)).unwrap();
    assert_eq!(rs.aim, Some(aim));
    assert_eq!(w.obj(x).unwrap().dest, Some(Target::Cell(cell)));
    let mut at = None;
    for k in 0..2000 {
        // The aircraft moves off; the aim point does not follow it.
        if k < 40
            && let Some(o) = w.obj_mut(flyer)
        {
            o.pos = Pos::new(o.pos.x + 20, o.pos.y, UP);
        }
        testkit::run(&mut w, 1);
        if w.obj(x).is_none() {
            at = w.effects.iter().find_map(|e| match e {
                Effect::Detonation { pos, .. } => Some(*pos),
                _ => None,
            });
            break;
        }
        assert_eq!(state(&w, x).aim, Some(aim), "the aim point does not move");
    }
    assert!(near(at.expect("it detonates"), aim));
}

/// LNC shot at `target`, the first missile preparing.
fn preparing(w: &mut World, lnc: ObjectId, target: ObjectId) -> ObjectId {
    let (x, _) = launch(w, lnc, target, None);
    let c = controller(w, lnc);
    assert_eq!(c.slots[0].state, SlotState::Preparing, "set-up: preparing");
    assert_eq!(c.target, Some(Target::Object(target)));
    assert_eq!(c.pending, None);
    x
}

fn assert_reset(w: &World, lnc: ObjectId, x: ObjectId, what: &str) {
    let c = controller(w, lnc);
    assert_eq!(c.state, ControllerState::Idle, "{what}: the reset");
    assert_eq!(c.target, None, "{what}");
    assert_eq!(
        c.slots[0].state,
        SlotState::Empty,
        "{what}: released from its slot"
    );
    // spec: S-0506 claim 12 (r37: the target is cleared before the reset, so the release cell is F8's neighbour of the missile's own cell)
    assert_eq!(release_cell(w, x), Some(f8_cell(w, x)), "{what}: F8's cell");
}

// spec: S-0506 claim 11 (r37: the controller target is lost at the hit that destroys it, even while it then stays in play, a death sequence; cleared before the reset) and claim 12 (r37, Q-2749: the released missile gets the neighbouring cell of F8, not the cell the lost object stood in); scenario R7 part c and R21's lost-target variant part c
#[test]
fn r7_c_the_reset_comes_at_the_destroying_hit_of_an_infantry_target() {
    let r = rules();
    let mut w = testkit::game_with(&r, SIZE, 7);
    let lnc = put(&mut w, Kind::Vehicle, "LNC", 0, 0, 0);
    let man = put(&mut w, Kind::Infantry, "WALKER", 1, 8, 0);
    let x = preparing(&mut w, lnc, man);
    let last = w.obj(man).unwrap().pos.cell();
    let w1 = r.sim.warheads.keys().position(|k| k == "W1").unwrap() as u32;
    let hit = Hit {
        amount: 100_000,
        warhead: Some(w1),
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: false,
        no_escape: false,
    };
    damage::apply(&mut w, man, hit);
    assert!(
        w.obj(man).is_some(),
        "set-up: still in play, showing its death"
    );
    assert_reset(&w, lnc, x, "the destroying hit");
    assert_ne!(f8_cell(&w, x), last, "set-up: the cells differ");
}

// spec: S-0506 claim 11 (r37: a destination that goes inside a transport is lost, cleared before the reset) and claim 12 (r37, Q-2749: the released missile gets F8's cell, not the cell the object went inside at); scenario R21's lost-target variant part a; and claim 11 (its removal: the same)
#[test]
fn r21_going_inside_or_removed_the_release_cell_is_f8s() {
    for inside in [true, false] {
        let r = rules();
        let mut w = testkit::game_with(&r, SIZE, 7);
        let lnc = put(&mut w, Kind::Vehicle, "LNC", 0, 0, 0);
        let man = put(&mut w, Kind::Infantry, "WALKER", 1, 8, 0);
        let x = preparing(&mut w, lnc, man);
        if inside {
            objects::take_inside(&mut w, man);
        } else {
            objects::remove(&mut w, man);
        }
        assert_reset(&w, lnc, x, if inside { "inside" } else { "removed" });
    }
}

/// LNC shot at a PLAIN of `house` that then gets its own cloak flag (a
/// cloaking crate's effect), with a SEEKER of the spawner's house next to
/// it when `sensor`; runs until it starts to cloak and checks the reset in
/// that tick.
fn r7_d(house: u8, sensor: bool) {
    let r = rules();
    let mut w = testkit::game_with(&r, SIZE, 7);
    let lnc = put(&mut w, Kind::Vehicle, "LNC", 0, 0, 0);
    let plain = put(&mut w, Kind::Vehicle, "PLAIN", house, 8, 0);
    if sensor {
        put(&mut w, Kind::Vehicle, "SEEKER", 0, 8, 1);
    }
    let x = preparing(&mut w, lnc, plain);
    cloak::machine::grant_own_flag(&mut w, plain);
    let what = format!("house {house}, sensor {sensor}");
    for _ in 0..30 {
        testkit::run(&mut w, 1);
        if w.obj(plain).unwrap().cloak.state == cloak::CloakState::Cloaking {
            assert_reset(&w, lnc, x, &what);
            return;
        }
        assert_eq!(
            controller(&w, lnc).slots[0].state,
            SlotState::Preparing,
            "{what}: no reset before the cloaking starts"
        );
    }
    panic!("{what}: no start of cloaking");
}

// spec: S-0506 claim 11 (r37: the controller target is lost when it starts to cloak, whatever the spawner's house and whatever sensors cover it: no exception, unlike S-0309 claim 28 and S-0516 claim 14's drop); scenario R7 part d (a sensor of the spawner's house covers the target's cell; the target of the spawner's house, a forced shot)
#[test]
fn r7_d_the_reset_comes_at_the_start_of_cloaking_whatever_house_and_sensors() {
    r7_d(1, false);
    r7_d(1, true);
    r7_d(0, false);
}
