//! Claim tests for S-0506 (missiles: the spawn controller and rocket
//! flight) on synthetic data: invented types, weapons, warheads and rocket
//! parameter values, none of them stock values.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, LocomotionClass, Locomotor, UnitType, WeaponType};
use glasnost_sim::combat::effects::{Effect, Sound};
use glasnost_sim::orders;
use glasnost_sim::rocket::Phase;
use glasnost_sim::rules::Kind;
use glasnost_sim::spawn::{self, ControllerState, SlotState};
use glasnost_sim::targeting::{self, ClickOn, Modifiers};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

const SIZE: i32 = 40;
const ALPHA: HouseId = HouseId(0);

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

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

fn launcher(spawns: &str, n: i32, weapon: &str) -> UnitType {
    let mut t = UnitType {
        strength: Some(400),
        speed: Some(6),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        primary: Some(weapon.into()),
        missile_spawn: Some(true),
        ..Default::default()
    };
    t.combat.spawns = Some(spawns.into());
    t.combat.spawns_number = Some(n);
    t.combat.spawn_regen_rate = Some(50);
    t.combat.spawn_reload_rate = Some(0);
    t.combat.primary_fire_flh = Some([60, 0, 80]);
    t
}

/// The combat ruleset with these tests' types:
/// - rocket types RKA (set A), RKB (set B), RKC (set C), all with the
///   missile-spawn flag;
/// - launchers LNA (one RKA slot), LNB (two RKB slots), LNC (one RKC
///   slot), driving vehicles with the spawner weapon Launch (range 30 cells);
/// - a target vehicle VICTIM of strength 5000.
///
/// Warhead RKW (spread 2 cells, versus 1 everywhere). Set parameters are invented: short pauses and tilts, a fast
/// acceleration and a low cruise altitude, so that flights stay short.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let push = |r: &mut Ruleset, reg: &str, n: &str| {
        r.registries.entry(reg.into()).or_default().push(n.into());
    };
    for n in ["RKA", "RKB", "RKC"] {
        push(&mut r, "aircraft", n);
    }
    for n in ["LNA", "LNB", "LNC", "VICTIM"] {
        push(&mut r, "vehicles", n);
    }
    let s = &mut r.sim;
    s.aircraft.insert("RKA".into(), rocket(50));
    s.aircraft.insert("RKB".into(), rocket(40));
    s.aircraft.insert("RKC".into(), rocket(40));
    s.vehicles
        .insert("LNA".into(), launcher("RKA", 1, "Launch"));
    s.vehicles
        .insert("LNB".into(), launcher("RKB", 2, "Launch"));
    s.vehicles
        .insert("LNC".into(), launcher("RKC", 1, "Launch"));
    s.vehicles.insert(
        "VICTIM".into(),
        UnitType {
            strength: Some(5000),
            speed: Some(4),
            rot: Some(5),
            locomotion_class: Some(LocomotionClass::Wheel),
            locomotor: Some(Locomotor::Drive),
            ..Default::default()
        },
    );
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
    s.warheads.insert(
        "RKW".into(),
        glasnost_rules::sim::WarheadType {
            cell_spread: dec("2"),
            percent_at_max: dec("0.5"),
            verses: Some((0..11).map(|_| Decimal("1".into())).collect()),
            ..Default::default()
        },
    );
    let g = &mut s.general;
    g.v3_rocket_type = Some("RKA".into());
    g.d_misl_type = Some("RKB".into());
    g.c_misl_type = Some("RKC".into());
    g.v3_rocket_pause_frames = Some(4);
    g.v3_rocket_tilt_frames = Some(10);
    g.v3_rocket_altitude = Some(600);
    g.v3_rocket_pitch_final = dec("0.6");
    g.d_misl_pitch_final = dec("0.5");
    g.d_misl_turn_rate = dec("0.2");
    g.v3_rocket_acceleration = dec("3");
    g.v3_rocket_damage = Some(300);
    g.v3_rocket_elite_damage = Some(450);
    g.d_misl_pause_frames = Some(3);
    g.d_misl_tilt_frames = Some(8);
    g.d_misl_altitude = Some(500);
    g.d_misl_acceleration = dec("2.5");
    g.d_misl_damage = Some(200);
    g.d_misl_elite_damage = Some(250);
    g.c_misl_pause_frames = Some(5);
    g.c_misl_tilt_frames = Some(6);
    g.c_misl_raise_rate = dec("7.9");
    g.c_misl_altitude = Some(500);
    g.c_misl_acceleration = dec("3");
    g.c_misl_damage = Some(150);
    g.v3_warhead = Some("RKW".into());
    g.v3_elite_warhead = Some("SA".into());
    g.d_misl_warhead = Some("RKW".into());
    g.c_misl_warhead = Some("RKW".into());
    r
}

fn game(r: &Ruleset) -> World {
    testkit::game_with(r, SIZE, 7)
}

fn spawn_at(w: &mut World, name: &str, house: u8, dx: i32, dy: i32, facing: u16) -> ObjectId {
    testkit::spawn(
        w,
        Kind::Vehicle,
        name,
        house,
        testkit::mid(SIZE, dx, dy),
        facing,
    )
}

fn open(w: &mut World) {
    w.tick = w.tick.max(w.orders_blocked_through + 1);
}

/// Orders `actor` to attack `target` through the input rules (a click
/// with the actor selected).
fn click_attack(w: &mut World, actor: ObjectId, target: ObjectId) {
    open(w);
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

fn controller(w: &World, id: ObjectId) -> spawn::SpawnController {
    w.obj(id).unwrap().spawn.clone().unwrap()
}

fn rockets(w: &World) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| o.rocket.is_some())
        .map(|o| o.id)
        .collect()
}

// spec: S-0506 claims 1 to 8, 14 to 19 (end to end: a set-A launcher ordered to attack by a click launches, the missile flies through its phases and detonates at its aim point, damaging the target through S-0303)
#[test]
fn set_a_launcher_attacks_in_play() {
    let r = rules();
    let mut w = game(&r);
    let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    let aim = w.obj(victim).unwrap().pos;
    let h0 = w.obj(victim).unwrap().health;
    // spec: S-0506 claim 1 (the full count from the start, docked, off the map)
    let c = controller(&w, lna);
    assert_eq!(c.slots.len(), 1);
    assert_eq!(c.slots[0].state, SlotState::Docked);
    let missile = c.slots[0].object.unwrap();
    assert!(!w.obj(missile).unwrap().on_map);
    click_attack(&mut w, lna, victim);
    // Launch.
    let launched = testkit_run_until(&mut w, 200, |w| w.obj(missile).is_some_and(|o| o.on_map));
    assert!(launched, "the missile is placed on the map");
    let start = w.obj(missile).unwrap().pos;
    // spec: S-0506 claim 7 (a missile slot's object gets the controller target as its destination)
    assert_eq!(w.obj(missile).unwrap().dest, Some(Target::Object(victim)));
    // Flight: record phases and positions until detonation.
    let mut phases = Vec::new();
    let mut positions = Vec::new();
    let mut detonated_at = None;
    for _ in 0..600 {
        testkit::run(&mut w, 1);
        if let Some(o) = w.obj(missile) {
            if let Some(p) = o.rocket.as_ref().and_then(|r| r.phase)
                && phases.last() != Some(&p)
            {
                phases.push(p);
            }
            positions.push(o.pos);
        }
        if w.obj(missile).is_none() {
            detonated_at = w.effects.iter().find_map(|e| match e {
                Effect::Detonation { pos, .. } => Some(*pos),
                _ => None,
            });
            break;
        }
    }
    assert_eq!(
        phases,
        vec![Phase::Pause, Phase::Tilt, Phase::Ascent, Phase::Cruise],
        "set A flies pause, tilt, ascent and lazy cruise"
    );
    assert!(positions.iter().any(|p| p.z > start.z + 500), "it climbs");
    let at = detonated_at.expect("the missile detonates");
    // spec: S-0506 claim 19 (the detonation at the nose point, near the aim point)
    assert!(
        (at.x - aim.x).abs() < 256 && (at.y - aim.y).abs() < 256,
        "{at:?} vs {aim:?}"
    );
    // spec: S-0506 claim 19 and S-0303 (the target is damaged by the area detonation)
    assert!(w.obj(victim).unwrap().health < h0);
}

fn testkit_run_until(w: &mut World, max: i32, mut done: impl FnMut(&World) -> bool) -> bool {
    for _ in 0..max {
        testkit::run(w, 1);
        if done(w) {
            return true;
        }
    }
    false
}

// spec: S-0506 claim 2 and claim 5 (passes at creation + 20 and every 10; a shot between passes is adopted at the next pass and the first object appears at the pass after)
#[test]
fn claim_2_pass_timing() {
    let r = rules();
    let mut w = game(&r);
    let created = w.tick;
    let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
    // A friendly target, so that the launcher acquires nothing by itself.
    let victim = spawn_at(&mut w, "VICTIM", 0, 8, 0, 0);
    // spec: S-0506 claim 2 (the first pass 20 ticks after creation)
    assert_eq!(controller(&w, lna).next_pass, created + 20);
    let missile = controller(&w, lna).slots[0].object.unwrap();
    // Deliver the shot directly between passes.
    while w.tick < created + 25 {
        testkit::run(&mut w, 1);
    }
    w.obj_mut(lna).unwrap().target = Some(Target::Object(victim));
    spawn::shot(&mut w, lna, Target::Object(victim));
    assert_eq!(controller(&w, lna).pending, Some(Target::Object(victim)));
    let mut appeared = None;
    for _ in 0..40 {
        let t = w.tick;
        testkit::run(&mut w, 1);
        if appeared.is_none() && w.obj(missile).unwrap().on_map {
            appeared = Some(t);
        }
    }
    // spec: S-0506 claim 5 (the pass at +30 adopts, the pass at +40 launches)
    assert_eq!(appeared, Some(created + 40));
}

// spec: S-0506 claim 3 (a shot at the controller target changes nothing; another target becomes pending)
#[test]
fn claim_3_pending_target() {
    let r = rules();
    let mut w = game(&r);
    let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
    let a = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    let b = spawn_at(&mut w, "VICTIM", 1, 8, 3, 0);
    w.obj_mut(lna).unwrap().spawn.as_mut().unwrap().target = Some(Target::Object(a));
    spawn::shot(&mut w, lna, Target::Object(a));
    assert_eq!(controller(&w, lna).pending, None);
    spawn::shot(&mut w, lna, Target::Object(b));
    assert_eq!(controller(&w, lna).pending, Some(Target::Object(b)));
    // A shot at the adopted target while another is pending leaves the pending one.
    spawn::shot(&mut w, lna, Target::Object(a));
    assert_eq!(controller(&w, lna).pending, Some(Target::Object(b)));
    // Clearing the spawner's own target clears the pending target.
    spawn::target_cleared(&mut w, lna);
    assert_eq!(controller(&w, lna).pending, None);
}

// spec: S-0506 claim 6 and Edge cases quirks.launch_spacing_from_spawner_type (s = 9 for a spawner with the flag: one launch per pass; 20 without: every second pass)
#[test]
fn claim_6_spacing() {
    for (flag, gap) in [(true, 10), (false, 20)] {
        let mut r = rules();
        r.sim.vehicles.get_mut("LNB").unwrap().missile_spawn = Some(flag);
        let mut w = game(&r);
        let lnb = spawn_at(&mut w, "LNB", 0, 0, 0, 0);
        let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
        let objs: Vec<ObjectId> = controller(&w, lnb)
            .slots
            .iter()
            .map(|s| s.object.unwrap())
            .collect();
        spawn::shot(&mut w, lnb, Target::Object(victim));
        let mut seen = [None, None];
        for _ in 0..120 {
            let t = w.tick;
            testkit::run(&mut w, 1);
            for (k, &x) in objs.iter().enumerate() {
                if seen[k].is_none() && w.obj(x).is_some_and(|o| o.on_map) {
                    seen[k] = Some(t);
                }
            }
        }
        let (a, b) = (seen[0].unwrap(), seen[1].unwrap());
        assert_eq!(b - a, gap, "flag {flag}");
    }
}

// spec: S-0506 claim 6 and Edge cases (quirks.launch_spacing_from_spawner_type off: the spawned type's flag decides)
#[test]
fn claim_6_spacing_quirk_off() {
    let mut r = rules();
    r.globals.quirks.launch_spacing_from_spawner_type = false;
    r.sim.vehicles.get_mut("LNB").unwrap().missile_spawn = Some(false);
    let mut w = game(&r);
    let lnb = spawn_at(&mut w, "LNB", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    let objs: Vec<ObjectId> = controller(&w, lnb)
        .slots
        .iter()
        .map(|s| s.object.unwrap())
        .collect();
    spawn::shot(&mut w, lnb, Target::Object(victim));
    let mut seen = [None, None];
    for _ in 0..120 {
        let t = w.tick;
        testkit::run(&mut w, 1);
        for (k, &x) in objs.iter().enumerate() {
            if seen[k].is_none() && w.obj(x).is_some_and(|o| o.on_map) {
                seen[k] = Some(t);
            }
        }
    }
    assert_eq!(seen[1].unwrap() - seen[0].unwrap(), 10);
}

// spec: S-0506 claim 8 (all preparing missiles are released in the pass of the last launch: rail, controller reset, cooling down)
#[test]
fn claim_8_release_and_rail() {
    let r = rules();
    let mut w = game(&r);
    let lnb = spawn_at(&mut w, "LNB", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    spawn::shot(&mut w, lnb, Target::Object(victim));
    let done = testkit_run_until(&mut w, 120, |w| {
        controller(w, lnb)
            .slots
            .iter()
            .all(|s| s.state == SlotState::OnRail)
    });
    assert!(done);
    let c = controller(&w, lnb);
    // spec: S-0506 claim 8 (a missile released: the controller resets, then cools down)
    assert_eq!(c.state, ControllerState::CoolingDown);
    assert_eq!(c.target, None);
    // spec: S-0506 claim 8 (rail time = set B pause + tilt)
    let rail = 3 + 8;
    assert_eq!(c.slots[0].until, w.tick - 1 + rail);
    // Both released missiles are kamikazes with the target's cell.
    for s in &c.slots {
        let x = s.object.unwrap();
        assert!(spawn::kamikaze(&w, x));
        assert_eq!(w.spawned.kamikaze[&x], w.obj(victim).unwrap().pos.cell());
    }
}

// spec: S-0506 claim 9 (rail over: empty with the regeneration countdown; regeneration over: a new docked object at a later pass)
#[test]
fn claim_9_regeneration() {
    let r = rules();
    let mut w = game(&r);
    let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    let first = controller(&w, lna).slots[0].object.unwrap();
    spawn::shot(&mut w, lna, Target::Object(victim));
    assert!(testkit_run_until(&mut w, 200, |w| controller(w, lna).slots
        [0]
    .state
        == SlotState::Empty));
    let t_empty = w.tick - 1;
    assert_eq!(controller(&w, lna).slots[0].until, t_empty + 50);
    assert!(testkit_run_until(&mut w, 200, |w| controller(w, lna).slots
        [0]
    .state
        == SlotState::Docked));
    let t_docked = w.tick - 1;
    assert!(t_docked >= t_empty + 50 && t_docked < t_empty + 60);
    assert_ne!(controller(&w, lna).slots[0].object, Some(first));
}

// spec: S-0506 claim 10 (busy while launching for a vehicle; rearming with no slot object alive; not movable while 0 < count < N)
#[test]
fn claim_10_lockouts() {
    use glasnost_sim::combat::check::{Answer, check};
    let r = rules();
    let mut w = game(&r);
    let lnb = spawn_at(&mut w, "LNB", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    spawn::shot(&mut w, lnb, Target::Object(victim));
    assert!(testkit_run_until(&mut w, 120, |w| spawn::launching_count(
        w, lnb
    ) == 1));
    // spec: S-0214 Terms (movable) and S-0506 claim 10 (count 1 of 2: not movable)
    assert!(!targeting::movable(&w, ALPHA, lnb));
    // spec: S-0301 claim 9 rank 2 (busy)
    assert_eq!(
        check(&w, lnb, Some(Target::Object(victim)), 0, true),
        Answer::Busy
    );
    assert!(testkit_run_until(&mut w, 120, |w| spawn::launching_count(
        w, lnb
    ) == 2));
    assert!(targeting::movable(&w, ALPHA, lnb));
    // Rearming: every slot empty.
    for s in w
        .obj_mut(lnb)
        .unwrap()
        .spawn
        .as_mut()
        .unwrap()
        .slots
        .iter_mut()
    {
        s.state = SlotState::Empty;
        s.object = None;
        s.until = i32::MAX;
    }
    assert!(!spawn::any_alive(&w, lnb));
    assert_eq!(
        check(&w, lnb, Some(Target::Object(victim)), 0, true),
        Answer::Rearming
    );
}

// spec: S-0506 claim 10 (a driving spawner with a positive launching count begins no track; S-0207 claim 6)
#[test]
fn claim_10_track_hold() {
    let r = rules();
    let mut w = game(&r);
    let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    spawn::shot(&mut w, lna, Target::Object(victim));
    assert!(testkit_run_until(&mut w, 120, |w| spawn::launching_count(
        w, lna
    ) == 1));
    let p0 = w.obj(lna).unwrap().pos;
    open(&mut w);
    let mv = glasnost_sim::orders::Order::Behaviour {
        actor: lna,
        behaviour: glasnost_sim::rules::Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(testkit::mid(SIZE, 0, 6))),
    };
    assert!(orders::issue(&mut w, ALPHA, mv));
    while spawn::launching_count(&w, lna) > 0 {
        testkit::run(&mut w, 1);
        assert_eq!(w.obj(lna).unwrap().pos, p0, "no track while launching");
    }
    testkit::run(&mut w, 40);
    assert_ne!(w.obj(lna).unwrap().pos, p0, "it moves once launching ends");
    assert!(w.obj(lna).unwrap().dest.is_some() || w.obj(lna).unwrap().pos.cell() != p0.cell());
}

// spec: S-0506 claim 11 (reset: preparing missiles released with the controller target, slots empty with regeneration, target and pending cleared, idle)
#[test]
fn claim_11_reset() {
    let r = rules();
    let mut w = game(&r);
    let lnb = spawn_at(&mut w, "LNB", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    spawn::shot(&mut w, lnb, Target::Object(victim));
    assert!(testkit_run_until(&mut w, 120, |w| controller(w, lnb).slots
        [0]
    .state
        == SlotState::Preparing));
    let x = controller(&w, lnb).slots[0].object.unwrap();
    spawn::impossible(&mut w, lnb);
    let c = controller(&w, lnb);
    assert_eq!(c.slots[0].state, SlotState::Empty);
    assert_eq!(c.slots[0].until, w.tick + 50);
    assert_eq!(c.target, None);
    assert_eq!(c.pending, None);
    assert_eq!(c.state, ControllerState::Idle);
    assert!(spawn::kamikaze(&w, x));
    // The released missile flies on (it keeps its destination).
    assert!(w.obj(x).is_some());
}

// spec: S-0506 claim 11 (the controller target leaving play with no pending target resets; claim 14 the missile keeps its fixed aim)
#[test]
fn claim_11_target_leaves_play() {
    let r = rules();
    let mut w = game(&r);
    let lnb = spawn_at(&mut w, "LNB", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    spawn::shot(&mut w, lnb, Target::Object(victim));
    assert!(testkit_run_until(&mut w, 120, |w| controller(w, lnb).slots
        [0]
    .state
        == SlotState::Preparing));
    glasnost_sim::objects::remove(&mut w, victim);
    let c = controller(&w, lnb);
    assert_eq!(c.state, ControllerState::Idle);
    assert_eq!(c.slots[0].state, SlotState::Empty);
}

// spec: S-0506 claim 12 and F8 (the release cell without a target: the F8 neighbour of the missile's cell)
#[test]
fn claim_12_release_direction() {
    // spec: S-0506 F8 (k = ((⌊F / 4096⌋ + 1) div 2) mod 8)
    assert_eq!(spawn::release_direction(0), 0);
    assert_eq!(spawn::release_direction(4095), 0);
    assert_eq!(spawn::release_direction(4096), 1);
    assert_eq!(spawn::release_direction(12288), 2);
    assert_eq!(spawn::release_direction(61440), 0);
    assert_eq!(spawn::release_direction(65535), 0);
}

// spec: S-0506 claim 13 and Edge cases quirks.rail_missile_vanishes_with_spawner (a spawner destroyed while a missile is on the rail: it vanishes, no detonation; off: it flies on)
#[test]
fn claim_13_rail_missile_vanishes() {
    for vanish in [true, false] {
        let mut r = rules();
        r.globals.quirks.rail_missile_vanishes_with_spawner = vanish;
        let mut w = game(&r);
        let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
        let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
        spawn::shot(&mut w, lna, Target::Object(victim));
        assert!(testkit_run_until(&mut w, 120, |w| controller(w, lna).slots
            [0]
        .state
            == SlotState::OnRail));
        let x = controller(&w, lna).slots[0].object.unwrap();
        glasnost_sim::objects::remove(&mut w, lna);
        assert_eq!(w.obj(x).is_some(), !vanish, "vanish {vanish}");
    }
}

// spec: S-0506 claim 13 (docked slot objects are removed silently with the spawner)
#[test]
fn claim_13_docked_removed() {
    let r = rules();
    let mut w = game(&r);
    let lnb = spawn_at(&mut w, "LNB", 0, 0, 0, 0);
    let objs: Vec<ObjectId> = controller(&w, lnb)
        .slots
        .iter()
        .map(|s| s.object.unwrap())
        .collect();
    glasnost_sim::objects::remove(&mut w, lnb);
    for x in objs {
        assert!(w.obj(x).is_none());
    }
}

// spec: S-0506 claim 13 (a house change clears the slots: the regeneration countdown is 0 while the spawner lives, so replacements appear at the next pass)
#[test]
fn claim_13_owner_change() {
    let r = rules();
    let mut w = game(&r);
    let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
    let old = controller(&w, lna).slots[0].object.unwrap();
    testkit::run(&mut w, 25);
    w.obj_mut(lna).unwrap().owner = Some(HouseId(1));
    spawn::owner_changed(&mut w, lna);
    assert!(w.obj(old).is_none());
    assert_eq!(controller(&w, lna).slots[0].until, w.tick);
    testkit::run(&mut w, 10);
    let c = controller(&w, lna);
    assert_eq!(c.slots[0].state, SlotState::Docked);
    let new = c.slots[0].object.unwrap();
    assert_eq!(w.obj(new).unwrap().owner, Some(HouseId(1)));
}

// spec: S-0506 F2 (placement facing rounded to 256 steps)
#[test]
fn f2_placement_facing() {
    assert_eq!(spawn::placement_facing(0), 0);
    assert_eq!(spawn::placement_facing(127), 0);
    assert_eq!(spawn::placement_facing(128), 256);
    assert_eq!(spawn::placement_facing(383), 256);
    assert_eq!(spawn::placement_facing(384), 512);
    assert_eq!(spawn::placement_facing(65535), 0);
}

// spec: S-0506 claim 7 (launch point: the fire point raised by 10 leptons; facing by F2)
#[test]
fn claim_7_launch_point() {
    let r = rules();
    let mut w = game(&r);
    let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 300);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    let x = controller(&w, lna).slots[0].object.unwrap();
    spawn::shot(&mut w, lna, Target::Object(victim));
    let mut fp = None;
    for _ in 0..80 {
        fp = Some(glasnost_sim::combat::position::fire_point(&w, lna, 0, 0));
        let f = w.obj(lna).unwrap().body.value(w.tick);
        testkit::run(&mut w, 1);
        if w.obj(x).unwrap().on_map {
            let p = fp.unwrap();
            assert_eq!(w.obj(x).unwrap().pos, Pos::new(p.x, p.y, p.z + 10));
            assert_eq!(
                w.obj(x).unwrap().body.value(w.tick),
                spawn::placement_facing(f)
            );
            return;
        }
    }
    panic!("no launch {fp:?}");
}

// spec: S-0506 claim 7 (the set-C type: 40 leptons towards map north and west)
#[test]
fn claim_7_set_c_shift() {
    let r = rules();
    let mut w = game(&r);
    let lnc = spawn_at(&mut w, "LNC", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    let x = controller(&w, lnc).slots[0].object.unwrap();
    spawn::shot(&mut w, lnc, Target::Object(victim));
    for _ in 0..80 {
        let p = glasnost_sim::combat::position::fire_point(&w, lnc, 0, 0);
        testkit::run(&mut w, 1);
        if w.obj(x).unwrap().on_map {
            assert_eq!(
                w.obj(x).unwrap().pos,
                Pos::new(p.x - 40, p.y - 40, p.z + 10)
            );
            return;
        }
    }
    panic!("no launch");
}

// spec: S-0506 claim 15 (set C after a pause: raise, climbing by the truncated raise rate per tick; tracked only after the raise)
#[test]
fn claim_15_set_c_raise() {
    let r = rules();
    let mut w = game(&r);
    let lnc = spawn_at(&mut w, "LNC", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    let x = controller(&w, lnc).slots[0].object.unwrap();
    spawn::shot(&mut w, lnc, Target::Object(victim));
    assert!(testkit_run_until(&mut w, 200, |w| w
        .obj(x)
        .and_then(|o| o.rocket.as_ref())
        .and_then(|r| r.phase)
        == Some(Phase::Raise)));
    assert!(!glasnost_sim::combat::flying::tracked(&w, x));
    let z0 = w.obj(x).unwrap().pos.z;
    testkit::run(&mut w, 1);
    // spec: S-0506 Import mapping (raise rate: real, truncated: 7.9 gives 7)
    assert_eq!(w.obj(x).unwrap().pos.z, z0 + 7);
    assert!(testkit_run_until(&mut w, 20, |w| w
        .obj(x)
        .and_then(|o| o.rocket.as_ref())
        .and_then(|r| r.phase)
        == Some(Phase::Ascent)));
    assert!(glasnost_sim::combat::flying::tracked(&w, x));
    // spec: S-0506 claim 15 (the take-off sound at the end of the raise)
    assert!(w.effects.iter().any(|e| matches!(
        e,
        Effect::Sound {
            sound: Sound::TakeOff,
            ..
        }
    )));
}

// spec: S-0506 claim 15 and 17 (set B: plain cruise enters descent)
#[test]
fn claim_17_set_b_descent() {
    let r = rules();
    let mut w = game(&r);
    let lnb = spawn_at(&mut w, "LNB", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 10, 0, 0);
    let h0 = w.obj(victim).unwrap().health;
    let x = controller(&w, lnb).slots[0].object.unwrap();
    spawn::shot(&mut w, lnb, Target::Object(victim));
    let mut phases = Vec::new();
    for _ in 0..800 {
        testkit::run(&mut w, 1);
        match w
            .obj(x)
            .and_then(|o| o.rocket.as_ref())
            .and_then(|r| r.phase)
        {
            Some(p) if phases.last() != Some(&p) => phases.push(p),
            _ => {}
        }
        if w.obj(x).is_none() && !phases.is_empty() {
            break;
        }
    }
    assert_eq!(
        phases,
        vec![
            Phase::Pause,
            Phase::Tilt,
            Phase::Ascent,
            Phase::Cruise,
            Phase::Descent
        ]
    );
    assert!(w.obj(victim).unwrap().health < h0);
}

// spec: S-0506 claim 19 (an elite launch uses the elite damage and warhead)
#[test]
fn claim_19_elite_launch() {
    let r = rules();
    let mut w = game(&r);
    let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    w.obj_mut(lna).unwrap().combat.experience = glasnost_num::F32(0x4040_0000); // nocite: 3.0 as a single, an elite experience (S-0311 claim 1)
    let x = controller(&w, lna).slots[0].object.unwrap();
    spawn::shot(&mut w, lna, Target::Object(victim));
    let mut damage = None;
    for _ in 0..600 {
        testkit::run(&mut w, 1);
        if w.obj(x).is_none() {
            damage = w.effects.iter().find_map(|e| match e {
                Effect::Detonation { damage, .. } => Some(*damage),
                _ => None,
            });
            break;
        }
    }
    assert_eq!(damage, Some(450));
}

// spec: S-0506 claim 20 and Edge cases quirks.destroyed_rocket_still_detonates (a missile destroyed above the terrain keeps flying and detonates on its next positive-speed move; off: it is removed without a detonation)
#[test]
fn claim_20_interception() {
    for keep in [true, false] {
        let mut r = rules();
        r.globals.quirks.destroyed_rocket_still_detonates = keep;
        let mut w = game(&r);
        let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
        let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
        let x = controller(&w, lna).slots[0].object.unwrap();
        spawn::shot(&mut w, lna, Target::Object(victim));
        assert!(testkit_run_until(&mut w, 300, |w| w
            .obj(x)
            .and_then(|o| o.rocket.as_ref())
            .and_then(|r| r.phase)
            == Some(Phase::Cruise)));
        w.obj_mut(x).unwrap().health = 0;
        glasnost_sim::combat::death::kill(
            &mut w,
            x,
            &glasnost_sim::combat::damage::Hit {
                amount: 40,
                warhead: None,
                distance: 0,
                attacker: None,
                house: None,
                ignore_defences: false,
                no_escape: false,
            },
        );
        assert_eq!(w.obj(x).is_some(), keep);
        w.effects.clear();
        testkit::run(&mut w, 1);
        let det = w
            .effects
            .iter()
            .any(|e| matches!(e, Effect::Detonation { damage: 300, .. }));
        assert_eq!(det, keep, "keep {keep}");
        assert!(w.obj(x).is_none());
    }
}

// spec: S-0506 claim 18 and Edge cases quirks.rocket_stalls_at_map_edge (a step into an illegal cell is not taken)
#[test]
fn claim_18_map_edge() {
    let r = rules();
    let mut w = game(&r);
    let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
    let x = controller(&w, lna).slots[0].object.unwrap();
    // Put the missile in ascent at the north corner of the map, facing out.
    glasnost_sim::rocket::place(&mut w, x, Pos::new(10, 10, 300), 0, lna);
    {
        let o = w.obj_mut(x).unwrap();
        let rs = o.rocket.as_mut().unwrap();
        rs.phase = Some(Phase::Ascent);
        rs.aim = Some(Pos::new(10, 10, 0));
        rs.speed = glasnost_num::F64::from_i64(40, glasnost_num::Rounding::TowardZero);
    }
    let p0 = w.obj(x).unwrap().pos;
    testkit::run(&mut w, 1);
    let p1 = w.obj(x).unwrap().pos;
    assert_eq!(
        (p1.x, p1.y),
        (p0.x, p0.y),
        "the step out of the map is not taken"
    );
}

// spec: S-0506 claim 21 (a trailer animation every 3 ticks in positive-speed flight)
#[test]
fn claim_21_trailer() {
    let mut r = rules();
    r.sim.aircraft.get_mut("RKA").unwrap().trailer = Some("TRL".into());
    let mut w = game(&r);
    let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 12, 0, 0);
    let x = controller(&w, lna).slots[0].object.unwrap();
    spawn::shot(&mut w, lna, Target::Object(victim));
    assert!(testkit_run_until(&mut w, 300, |w| w
        .obj(x)
        .and_then(|o| o.rocket.as_ref())
        .and_then(|r| r.phase)
        == Some(Phase::Ascent)));
    let mut ticks = Vec::new();
    for _ in 0..12 {
        let t = w.tick;
        w.effects.clear();
        testkit::run(&mut w, 1);
        if w.effects
            .iter()
            .any(|e| matches!(e, Effect::Anim { name, .. } if name == "TRL"))
        {
            ticks.push(t);
        }
    }
    assert!(ticks.len() >= 3);
    for p in ticks.windows(2) {
        assert_eq!(p[1] - p[0], 3);
    }
}

// spec: S-0311 claim 8 and S-0506 claim 19 (an untrainable missile gives its kill experience to a trainable launcher)
#[test]
fn kill_experience_to_launcher() {
    let mut r = rules();
    r.sim.vehicles.get_mut("LNA").unwrap().combat.trainable = Some(true);
    r.sim.aircraft.get_mut("RKA").unwrap().combat.trainable = Some(false);
    r.sim.vehicles.get_mut("VICTIM").unwrap().strength = Some(100);
    r.sim.vehicles.get_mut("VICTIM").unwrap().cost = Some(900);
    r.sim.vehicles.get_mut("LNA").unwrap().cost = Some(300);
    let mut w = game(&r);
    let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    let x = controller(&w, lna).slots[0].object.unwrap();
    spawn::shot(&mut w, lna, Target::Object(victim));
    assert!(testkit_run_until(&mut w, 600, |w| w.obj(x).is_none()));
    assert!(w.obj(victim).is_none_or(|o| o.health <= 0));
    assert!(w.obj(lna).unwrap().combat.experience.0 != 0);
}

// The state hash survives a save/load round trip in the middle of a flight, and both copies go on alike.
// spec: S-0506 claims 14 to 19 (rocket state is simulation state)
#[test]
fn save_load_mid_flight() {
    let r = rules();
    let mut w = game(&r);
    let lnb = spawn_at(&mut w, "LNB", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    spawn::shot(&mut w, lnb, Target::Object(victim));
    assert!(testkit_run_until(&mut w, 300, |w| rockets(w).iter().any(
        |&x| w.obj(x).unwrap().rocket.as_ref().unwrap().phase == Some(Phase::Ascent)
    )));
    let bytes = glasnost_sim::save::save(&w);
    let mut w2 =
        glasnost_sim::save::load(&bytes, w.rules_arc(), testkit::catalogue()).expect("loads");
    assert_eq!(
        glasnost_sim::save::state_hash(&w),
        glasnost_sim::save::state_hash(&w2)
    );
    for _ in 0..200 {
        testkit::run(&mut w, 1);
        testkit::run(&mut w2, 1);
        assert_eq!(
            glasnost_sim::save::state_hash(&w),
            glasnost_sim::save::state_hash(&w2)
        );
    }
}

// spec: S-0506 claims 1 to 9, 15 to 19 (end to end, set B: a two-slot launcher standing in for a Dreadnought-like vessel, on the driving locomotor because the ship locomotor is still a scaffold, ordered by a click, launches both missiles one spacing apart, releases them together, and both fly plain cruise and descent onto the target)
#[test]
fn set_b_launcher_attacks_in_play() {
    let r = rules();
    let mut w = game(&r);
    let lnb = spawn_at(&mut w, "LNB", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 9, 2, 0);
    let h0 = w.obj(victim).unwrap().health;
    let objs: Vec<ObjectId> = controller(&w, lnb)
        .slots
        .iter()
        .map(|s| s.object.unwrap())
        .collect();
    click_attack(&mut w, lnb, victim);
    let mut detonations = 0;
    let mut descended = [false, false];
    for _ in 0..800 {
        w.effects.clear();
        testkit::run(&mut w, 1);
        detonations += w
            .effects
            .iter()
            .filter(|e| matches!(e, Effect::Detonation { damage: 200, .. }))
            .count();
        for (k, &x) in objs.iter().enumerate() {
            if w.obj(x)
                .and_then(|o| o.rocket.as_ref())
                .and_then(|r| r.phase)
                == Some(Phase::Descent)
            {
                descended[k] = true;
            }
        }
        if objs.iter().all(|&x| w.obj(x).is_none()) {
            break;
        }
    }
    assert_eq!(descended, [true, true]);
    assert!(detonations >= 2);
    assert!(w.obj(victim).unwrap().health < h0);
}
