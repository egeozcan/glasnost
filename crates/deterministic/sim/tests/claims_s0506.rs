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
    // spec: S-0506 claim 25 (a pending clear with a nonempty controller target succeeds)
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
        assert!(
            w.spawned
                .kamikaze
                .contains(&(x, w.obj(victim).unwrap().pos.cell()))
        );
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

// spec: S-0506 claim 18 and Edge cases quirks.rocket_stalls_at_map_edge (a step into an illegal cell is not taken; the missile keeps its position, its speed updates still proceed)
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
    // spec: S-0506 claim 18 (revision 10: the phase, speed and pitch updates still proceed in the stalled tick)
    let speed = w.obj(x).unwrap().rocket.as_ref().unwrap().speed;
    assert!(
        speed.gt(glasnost_num::F64::from_i64(
            40,
            glasnost_num::Rounding::TowardZero
        )),
        "the ascent still accelerates"
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

fn phase_of(w: &World, x: ObjectId) -> Option<Phase> {
    w.obj(x)
        .and_then(|o| o.rocket.as_ref())
        .and_then(|r| r.phase)
}

/// Shoots LNA at a hostile VICTIM 8 cells east and runs until its missile
/// reaches `phase`; returns the world, the launcher, the victim and the
/// missile.
fn fly_until(r: &Ruleset, phase: Phase) -> (World, ObjectId, ObjectId, ObjectId) {
    let mut w = game(r);
    let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    let x = controller(&w, lna).slots[0].object.unwrap();
    spawn::shot(&mut w, lna, Target::Object(victim));
    assert!(testkit_run_until(&mut w, 300, |w| phase_of(w, x) == Some(phase)));
    (w, lna, victim, x)
}

fn warhead(w: &World, name: &str) -> u32 {
    w.rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == name)
        .unwrap() as u32
}

fn blast(w: &mut World, at: Pos) {
    let wh = warhead(w, "RKW");
    glasnost_sim::combat::area::detonate(
        w,
        glasnost_sim::combat::area::Detonation {
            point: at,
            damage: 30,
            warhead: Some(wh),
            source: None,
            house: None,
            resources: false,
        },
    );
}

// spec: S-0506 claim 15 and S-0303 claim 6 (a rocket missile is found by the flying-object search only after its tilt or raise: a blast at it in pause misses it, in ascent hits it)
#[test]
fn claim_15_tracked_only_after_tilt() {
    let r = rules();
    let (mut w, _, _, x) = fly_until(&r, Phase::Pause);
    let p = w.obj(x).unwrap().pos;
    let h = w.obj(x).unwrap().health;
    blast(&mut w, Pos::new(p.x, p.y, p.z + 1));
    assert_eq!(w.obj(x).unwrap().health, h, "not tracked in pause");
    let (mut w, _, _, x) = fly_until(&r, Phase::Ascent);
    let p = w.obj(x).unwrap().pos;
    let h = w.obj(x).unwrap().health;
    assert!(glasnost_sim::combat::flying::tracked(&w, x));
    blast(&mut w, Pos::new(p.x, p.y, p.z + 1));
    assert!(w.obj(x).is_none_or(|o| o.health < h), "tracked in ascent");
}

// spec: S-0506 claim 9 (a slot object destroyed while it belongs to its slot frees the slot with a regeneration countdown at once)
#[test]
fn claim_9_loss_notification() {
    let r = rules();
    let (mut w, lna, _, x) = fly_until(&r, Phase::Pause);
    // The slot is on the rail or preparing; kill the missile on the ground.
    w.obj_mut(x).unwrap().health = 0;
    glasnost_sim::combat::death::kill(&mut w, x, &hit(10));
    let c = controller(&w, lna);
    assert_eq!(c.slots[0].state, SlotState::Empty);
    assert_eq!(c.slots[0].until, w.tick + 50);
}

fn hit(amount: i32) -> glasnost_sim::combat::damage::Hit {
    glasnost_sim::combat::damage::Hit {
        amount,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: false,
        no_escape: false,
    }
}

// spec: S-0506 claim 20 (a missile destroyed on the ground, height below 1: removed at once without a detonation)
#[test]
fn claim_20_ground_removal() {
    let r = rules();
    let (mut w, _, _, x) = fly_until(&r, Phase::Pause);
    let p = w.obj(x).unwrap().pos;
    let ground = w.surface_at(p.x, p.y);
    w.obj_mut(x).unwrap().pos = Pos::new(p.x, p.y, ground);
    w.obj_mut(x).unwrap().health = 0;
    w.effects.clear();
    glasnost_sim::combat::death::kill(&mut w, x, &hit(10));
    assert!(w.obj(x).is_none());
    assert!(
        !w.effects
            .iter()
            .any(|e| matches!(e, Effect::Detonation { .. }))
    );
}

// spec: S-0506 claim 20 (destroyed in pause: no detonation until the first positive-speed movement attempt; the phase timers run on unchanged)
#[test]
fn claim_20_destroyed_in_pause() {
    let r = rules();
    let (mut w, _, _, x) = fly_until(&r, Phase::Pause);
    w.obj_mut(x).unwrap().health = 0;
    glasnost_sim::combat::death::kill(&mut w, x, &hit(10));
    assert!(w.obj(x).is_some(), "above the terrain it keeps flying");
    let mut det_phase = None;
    for _ in 0..60 {
        let before = phase_of(&w, x);
        w.effects.clear();
        testkit::run(&mut w, 1);
        if w.effects
            .iter()
            .any(|e| matches!(e, Effect::Detonation { damage: 300, .. }))
        {
            det_phase = before;
            break;
        }
    }
    assert_eq!(det_phase, Some(Phase::Ascent));
}

// spec: S-0506 claim 13 (a spawner destroyed while a missile is preparing: the missile is released and flies on)
#[test]
fn claim_13_killed_spawner_releases_preparing() {
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
    let docked = controller(&w, lnb).slots[1].object.unwrap();
    w.obj_mut(lnb).unwrap().health = 0;
    glasnost_sim::combat::death::kill(&mut w, lnb, &hit(500));
    assert!(w.obj(x).is_some());
    assert!(spawn::kamikaze(&w, x));
    assert!(
        w.obj(docked).is_none(),
        "the docked one is removed silently"
    );
}

// spec: S-0506 claim 8 and Edge cases quirks.raised_missile_rail_time_from_set_b (a set-C missile's rail time is set B's pause + tilt; off: set C's own)
#[test]
fn claim_8_set_c_rail_time() {
    for from_b in [true, false] {
        let mut r = rules();
        r.globals.quirks.raised_missile_rail_time_from_set_b = from_b;
        // Set C's own pause + tilt (5 + 9) differs from set B's (3 + 8).
        r.sim.general.c_misl_tilt_frames = Some(9);
        let mut w = game(&r);
        let lnc = spawn_at(&mut w, "LNC", 0, 0, 0, 0);
        let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
        spawn::shot(&mut w, lnc, Target::Object(victim));
        assert!(testkit_run_until(&mut w, 120, |w| controller(w, lnc).slots
            [0]
        .state
            == SlotState::OnRail));
        let expected = if from_b { 3 + 8 } else { 5 + 9 };
        assert_eq!(
            controller(&w, lnc).slots[0].until,
            w.tick - 1 + expected,
            "from_b {from_b}"
        );
    }
}

// spec: S-0506 claim 14 (the aim point: a building's centre, a cell's centre, a moving unit's step end; quirks.rocket_aim_at_heading_point off: its position)
#[test]
fn claim_14_heading_point() {
    use glasnost_sim::rocket::heading_point;
    let mut r = rules();
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("BIG".into());
    r.sim.buildings.insert(
        "BIG".into(),
        glasnost_rules::sim::BuildingType {
            strength: Some(500),
            foundation: Some([3, 2]),
            ..Default::default()
        },
    );
    let mut w = game(&r);
    let big = testkit::spawn(
        &mut w,
        Kind::Building,
        "BIG",
        1,
        testkit::mid(SIZE, 4, 4),
        0,
    );
    let p = w.obj(big).unwrap().pos;
    // spec: S-0506 claim 14 (position + 128 × (W − 1), 128 × (H − 1))
    assert_eq!(
        heading_point(&w, Target::Object(big)),
        Some(Pos::new(p.x + 256, p.y + 128, p.z))
    );
    let c = testkit::mid(SIZE, -3, 2);
    assert_eq!(heading_point(&w, Target::Cell(c)), Some(Pos::centre(c, 0)));
    // A moving unit: the end of its step.
    let v = spawn_at(&mut w, "VICTIM", 1, 0, 6, 0);
    open(&mut w);
    let mv = glasnost_sim::orders::Order::Behaviour {
        actor: v,
        behaviour: glasnost_sim::rules::Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(testkit::mid(SIZE, 6, 6))),
    };
    w.obj_mut(v).unwrap().owner = Some(ALPHA);
    assert!(orders::issue(&mut w, ALPHA, mv));
    assert!(testkit_run_until(&mut w, 60, |w| w
        .obj(v)
        .unwrap()
        .drive
        .as_ref()
        .is_some_and(|d| d.track.is_some())));
    let (tx, ty) = w
        .obj(v)
        .unwrap()
        .drive
        .as_ref()
        .unwrap()
        .track
        .as_ref()
        .unwrap()
        .target;
    let hp = heading_point(&w, Target::Object(v)).unwrap();
    assert_eq!((hp.x, hp.y), (tx, ty));
    assert_ne!(
        (hp.x, hp.y),
        (w.obj(v).unwrap().pos.x, w.obj(v).unwrap().pos.y)
    );
    let mut r2 = r.clone();
    r2.globals.quirks.rocket_aim_at_heading_point = false;
    let rules2 = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r2, "synthetic-test"));
    let mut w2 = w.clone();
    w2.rules = Some(rules2);
    assert_eq!(
        heading_point(&w2, Target::Object(v)),
        Some(w2.obj(v).unwrap().pos)
    );
}

// spec: S-0506 claim 16 and F4 (speed grows by a each tick up to the type speed)
#[test]
fn f4_speed() {
    use glasnost_num::{F64, Rounding};
    use glasnost_sim::rocket::next_speed;
    let rz = Rounding::TowardZero;
    let a = F64::from_i64(5, rz)
        .div(F64::from_i64(2, rz), rz)
        .to_f32(rz);
    let mut v = F64::ZERO;
    for _ in 0..3 {
        v = next_speed(v, a, 6, rz);
    }
    assert_eq!(v, F64::from_i64(6, rz));
    assert_eq!(next_speed(F64::ZERO, a, 6, rz), a.to_f64());
}

// spec: S-0506 F1 (facing 0 moves towards map north, 16384 towards map east; pitch π/2 straight up)
#[test]
fn f1_step() {
    use glasnost_num::{F32, F64, Rounding};
    use glasnost_sim::rocket::step;
    let rz = Rounding::TowardZero;
    let v = F64::from_i64(100, rz);
    let p0 = Pos::new(5000, 5000, 300);
    let n = step(p0, v, F32(0), 0, rz);
    assert!(n.y < 5000 && (n.x - 5000).abs() <= 1 && n.z == 300, "{n:?}");
    let e = step(p0, v, F32(0), 16384, rz);
    assert!(e.x > 5000 && (e.y - 5000).abs() <= 1, "{e:?}");
    let up = glasnost_num::geom::direction_constants().0.to_f32(rz);
    let u = step(p0, v, up, 0, rz);
    assert_eq!((u.x, u.y), (5000, 5000));
    assert!(u.z > 390);
}

// spec: S-0506 F3 (tilt pitch: α at 0, β at the end, linear between; a zero tilt time takes the endpoint)
#[test]
fn f3_tilt() {
    use glasnost_num::{F32, F64, Rounding};
    use glasnost_sim::rocket::tilt_pitch;
    let rz = Rounding::TowardZero;
    let one = F32(0x3f80_0000); // nocite: 1.0 as a single
    let zero = F32(0);
    let half_pi = glasnost_num::geom::direction_constants().0;
    assert_eq!(tilt_pitch(zero, one, 0, 10, rz), F32(0));
    assert_eq!(tilt_pitch(zero, one, 10, 10, rz), half_pi.to_f32(rz));
    assert_eq!(tilt_pitch(zero, one, 3, 0, rz), half_pi.to_f32(rz));
    let mid = tilt_pitch(zero, one, 5, 10, rz).to_f64();
    let expect = half_pi.div(F64::from_i64(2, rz), rz);
    assert_eq!(mid.to_f32(rz), expect.to_f32(rz));
}

// spec: S-0506 F5 (plain cruise pitch falls by τ to 0 and never below; a nonpositive pitch is unchanged)
#[test]
fn f5_plain_pitch() {
    use glasnost_num::{F32, F64, Rounding};
    use glasnost_sim::rocket::plain_pitch;
    let rz = Rounding::TowardZero;
    let s = |n: i64, d: i64| {
        F64::from_i64(n, rz)
            .div(F64::from_i64(d, rz), rz)
            .to_f32(rz)
    };
    let tau = s(1, 10);
    assert_eq!(plain_pitch(s(1, 20), tau, rz), F32(0));
    let neg = s(-1, 4);
    assert_eq!(plain_pitch(neg, tau, rz), neg);
    let p = plain_pitch(s(1, 2), tau, rz);
    assert!(p.to_f64().lt(s(1, 2).to_f64()) && p.to_f64().gt(F64::ZERO));
}

// spec: S-0506 F6 (raw table arctangent A: odd, T(0) = 0, an index beyond 4096 gives the single π/2; S-0204 F7's exact raw table)
#[test]
fn f6_raw_atan() {
    use glasnost_num::{F64, Rounding};
    use glasnost_sim::rocket::raw_atan;
    let rz = Rounding::TowardZero;
    assert_eq!(raw_atan(F64::ZERO, rz), F64::ZERO);
    let big = F64::from_i64(1_000_000, rz);
    let hp = glasnost_num::geom::direction_constants()
        .0
        .to_f32(rz)
        .to_f64();
    assert_eq!(raw_atan(big, rz), hp);
    assert_eq!(raw_atan(big.negated(), rz), hp.negated());
    let one = F64::from_i64(1, rz);
    assert_eq!(raw_atan(one.negated(), rz), raw_atan(one, rz).negated());
    // spec: S-0204 claim 13 and C5 (index 1024: the exact raw angle 12841411 / 8388608, its negation for a negative argument; no facing correction)
    let u = F64::from_i64(25, rz);
    let t1024 = F64::from_parts(false, 12841411, -23, rz);
    assert_eq!(raw_atan(u, rz), t1024);
    assert_eq!(raw_atan(u.negated(), rz), t1024.negated());
}

// spec: S-0506 F7 (descent pitch: turns towards the line of sight by at most τ; reaches it when within τ)
#[test]
fn f7_descent_pitch() {
    use glasnost_num::{F32, F64, Rounding};
    use glasnost_sim::rocket::descent_pitch;
    let rz = Rounding::TowardZero;
    let tau = F64::from_i64(1, rz)
        .div(F64::from_i64(20, rz), rz)
        .to_f32(rz);
    // Aim straight below: the line of sight is −π/2 (ρ = 0).
    let p1 = descent_pitch((0, 0, -500), F32(0), tau, true, rz);
    assert_eq!(p1, tau.to_f64().negated().to_f32(rz));
    // Aim level ahead: ℓ = 0; a pitch within τ goes to 0.
    let small = F64::from_i64(1, rz)
        .div(F64::from_i64(100, rz), rz)
        .to_f32(rz);
    assert_eq!(descent_pitch((1000, 0, 0), small, tau, true, rz), F32(0));
}

// spec: S-0506 F9 (nose point: L cos p ahead along the heading, L sin p up)
#[test]
fn f9_nose() {
    use glasnost_num::{F32, Rounding};
    use glasnost_sim::rocket::nose;
    let rz = Rounding::TowardZero;
    let p0 = Pos::new(5000, 5000, 300);
    let n = nose(p0, 200, F32(0), 16384, rz);
    assert!(
        (n.x - 5200).abs() <= 1 && (n.y - 5000).abs() <= 1 && n.z == 300,
        "{n:?}"
    );
    let up = glasnost_num::geom::direction_constants().0.to_f32(rz);
    let u = nose(p0, 200, up, 0, rz);
    assert_eq!((u.x, u.y), (5000, 5000));
    assert!((u.z - 500).abs() <= 1);
}

// spec: S-0506 claim 18 and Edge cases (quirks.rocket_stalls_at_map_edge off: a step out of the legal cells detonates)
#[test]
fn claim_18_map_edge_quirk_off() {
    let mut r = rules();
    r.globals.quirks.rocket_stalls_at_map_edge = false;
    let mut w = game(&r);
    let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
    let x = controller(&w, lna).slots[0].object.unwrap();
    glasnost_sim::rocket::place(&mut w, x, Pos::new(10, 10, 300), 0, lna);
    {
        let o = w.obj_mut(x).unwrap();
        let rs = o.rocket.as_mut().unwrap();
        rs.phase = Some(Phase::Ascent);
        rs.aim = Some(Pos::new(10, 10, 0));
        rs.speed = glasnost_num::F64::from_i64(40, glasnost_num::Rounding::TowardZero);
    }
    testkit::run(&mut w, 1);
    assert!(w.obj(x).is_none(), "it detonates at the edge");
}

// spec: S-0506 claim 22 (the set-A type: not in the air before flight and in pause and tilt; in the air from ascent on, even below 208 leptons; a destroying hit does not change the answer)
#[test]
fn claim_22_in_air_by_phase() {
    use glasnost_sim::combat::weapon::in_air;
    let r = rules();
    let (w, _, _, x) = fly_until(&r, Phase::Pause);
    assert!(!in_air(&w, x));
    let (mut w, _, _, x) = fly_until(&r, Phase::Tilt);
    assert!(!in_air(&w, x));
    w.obj_mut(x).unwrap().health = 0;
    assert!(!in_air(&w, x), "the hit alone does not change it");
    let (w, _, _, x) = fly_until(&r, Phase::Ascent);
    let o = w.obj(x).unwrap();
    assert!(o.pos.z - w.surface_at(o.pos.x, o.pos.y) < 208);
    assert!(in_air(&w, x), "ascent, below 208 leptons");
    let (w, _, _, x) = fly_until(&r, Phase::Cruise);
    assert!(in_air(&w, x));
}

/// Fires LNA (or LNC with `set_c`) with pause `p` and tilt `t` and returns
/// the aim-fixing tick, the tick ascent starts and the tick of the first
/// positive-speed movement attempt, plus the raise height gained (set C).
fn boundaries(p: i32, t: i32, set_c: bool) -> (i32, i32, i32, i32) {
    let mut r = rules();
    let g = &mut r.sim.general;
    if set_c {
        g.c_misl_pause_frames = Some(p);
        g.c_misl_tilt_frames = Some(t);
    } else {
        g.v3_rocket_pause_frames = Some(p);
        g.v3_rocket_tilt_frames = Some(t);
    }
    let mut w = game(&r);
    let ln = spawn_at(&mut w, if set_c { "LNC" } else { "LNA" }, 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    let x = controller(&w, ln).slots[0].object.unwrap();
    spawn::shot(&mut w, ln, Target::Object(victim));
    let (mut t0, mut ascent, mut moving) = (None, None, None);
    let (mut z_raise, mut z_end) = (None, None);
    for _ in 0..400 {
        let t = w.tick;
        testkit::run(&mut w, 1);
        let Some(o) = w.obj(x) else { break };
        let Some(rs) = o.rocket.as_ref() else {
            continue;
        };
        if t0.is_none() && rs.phase.is_some() {
            t0 = Some(t);
        }
        if rs.phase == Some(Phase::Raise) && z_raise.is_none() {
            z_raise = Some(o.pos.z);
        }
        if ascent.is_none() && rs.phase == Some(Phase::Ascent) {
            ascent = Some(t);
            z_end = Some(o.pos.z);
        }
        if moving.is_none() && rs.speed.gt(glasnost_num::F64::ZERO) {
            moving = Some(t);
            break;
        }
    }
    let gained = match (z_raise, z_end) {
        (Some(a), Some(b)) => b - a,
        _ => 0,
    };
    (t0.unwrap(), ascent.unwrap(), moving.unwrap(), gained)
}

// spec: S-0506 claim 24 (exact phase boundaries: the timer starts at t₀ with zero elapsed ticks; P = 0: ascent at t₀ + T, first positive-speed attempt one tick later; P > 0: ascent at t₀ + P + max(T, 1), movement one tick later)
#[test]
fn claim_24_phase_boundaries() {
    for (p, t) in [(0, 0), (0, 1), (1, 0), (1, 1), (3, 4)] {
        let (t0, ascent, moving, _) = boundaries(p, t, false);
        let expect = if p == 0 { t0 + t } else { t0 + p + t.max(1) };
        assert_eq!(ascent - t0, expect - t0, "ascent, pause {p}, tilt {t}");
        assert_eq!(moving, ascent + 1, "movement, pause {p}, tilt {t}");
    }
}

// spec: S-0506 claim 24 (set C: a positive pause ends in raise, which adds its rate on updates before expiry only, T − 1 increments; a zero pause tilts, including set C)
#[test]
fn claim_24_set_c_raise() {
    let rate = 7; // the fixture's raise rate 7.9, truncated to the integer world-height step
    let (t0, ascent, moving, gained) = boundaries(5, 6, true);
    assert_eq!(ascent, t0 + 5 + 6);
    assert_eq!(moving, ascent + 1);
    // The raise phase is first seen after the pause-expiry tick, which adds nothing.
    assert_eq!(gained, (6 - 1) * rate);
    let (t0, ascent, _, gained) = boundaries(0, 6, true);
    assert_eq!(ascent, t0 + 6);
    assert_eq!(gained, 0, "no raise without a pause");
}

// spec: S-0506 claim 25 and Edge cases (quirks.spawner_pending_target_survives_idle_clear: an own-target change to empty with an empty controller target leaves pending; off: it empties pending)
#[test]
fn claim_25_idle_clear() {
    for on in [true, false] {
        let mut r = rules();
        r.globals.quirks.spawner_pending_target_survives_idle_clear = on;
        let mut w = game(&r);
        let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
        let a = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
        w.obj_mut(lna).unwrap().target = Some(Target::Object(a));
        spawn::shot(&mut w, lna, Target::Object(a));
        spawn::clear_own_target(w.obj_mut(lna).unwrap());
        spawn::settle_clear(&mut w, lna);
        let expect = on.then_some(Target::Object(a));
        assert_eq!(controller(&w, lna).pending, expect, "quirk {on}");
    }
}

// spec: S-0506 claim 25 and Edge cases (an explicit clear of an already empty own target leaves pending with the quirk on; off: it empties pending even then)
#[test]
fn claim_25_explicit_clear_of_empty_target() {
    for on in [true, false] {
        let mut r = rules();
        r.globals.quirks.spawner_pending_target_survives_idle_clear = on;
        let mut w = game(&r);
        let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
        let a = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
        let b = spawn_at(&mut w, "VICTIM", 1, 8, 3, 0);
        w.obj_mut(lna).unwrap().spawn.as_mut().unwrap().target = Some(Target::Object(a));
        spawn::shot(&mut w, lna, Target::Object(b));
        assert_eq!(w.obj(lna).unwrap().target, None);
        spawn::explicit_clear_own_target(w.obj_mut(lna).unwrap());
        spawn::settle_clear(&mut w, lna);
        let expect = on.then_some(Target::Object(b));
        assert_eq!(controller(&w, lna).pending, expect, "quirk {on}");
    }
}

// spec: S-0506 claim 25 (with a nonempty controller target the clear succeeds; clearing and restoring the own target before a pass does not undo it; direct nonempty replacement does not touch pending; a targetless spawner is not swept at a pass; the pending target's loss clears it at once)
#[test]
fn claim_25_clear_and_loss() {
    let r = rules();
    let mut w = game(&r);
    let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
    let a = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    let b = spawn_at(&mut w, "VICTIM", 1, 8, 3, 0);
    let c = spawn_at(&mut w, "VICTIM", 1, 8, 6, 0);
    w.obj_mut(lna).unwrap().spawn.as_mut().unwrap().target = Some(Target::Object(a));
    w.obj_mut(lna).unwrap().target = Some(Target::Object(a));
    spawn::shot(&mut w, lna, Target::Object(b));
    // Direct nonempty replacement: pending unchanged.
    spawn::set_own_target(w.obj_mut(lna).unwrap(), Some(Target::Object(c)));
    spawn::settle_clear(&mut w, lna);
    assert_eq!(controller(&w, lna).pending, Some(Target::Object(b)));
    // Clear and restore in one go, before any pass: the clear still takes effect.
    spawn::clear_own_target(w.obj_mut(lna).unwrap());
    w.obj_mut(lna).unwrap().target = Some(Target::Object(a));
    spawn::pass(&mut w, lna);
    assert_eq!(
        controller(&w, lna).pending,
        None,
        "an effective clear, not undone by the restore"
    );
    // A pass with no own target does not drop pending (no per-pass sweep).
    w.obj_mut(lna).unwrap().spawn.as_mut().unwrap().target = None;
    w.obj_mut(lna).unwrap().target = None;
    spawn::shot(&mut w, lna, Target::Object(b));
    spawn::settle_clear(&mut w, lna);
    assert_eq!(controller(&w, lna).pending, Some(Target::Object(b)));
    // The pending target's own loss clears it at once.
    glasnost_sim::objects::remove(&mut w, b);
    assert_eq!(controller(&w, lna).pending, None);
}

// spec: S-0506 claims 7 and 8 and S-0501 claims 35 and 36 (a non-missile slot aircraft is sent to the step direction 0 neighbour of the spawner's cell, then set on the controller target at release and attacking)
#[test]
fn carrier_slot_cycle() {
    let mut r = rules();
    r.registries
        .entry("aircraft".into())
        .or_default()
        .push("HAWK".into());
    r.sim.aircraft.insert(
        "HAWK".into(),
        UnitType {
            strength: Some(80),
            speed: Some(10),
            spawned: Some(true),
            ..Default::default()
        },
    );
    r.sim
        .vehicles
        .insert("CAR".into(), launcher("HAWK", 1, "Launch"));
    r.sim.vehicles.get_mut("CAR").unwrap().missile_spawn = Some(false);
    r.registries.get_mut("vehicles").unwrap().push("CAR".into());
    let mut w = game(&r);
    let car = spawn_at(&mut w, "CAR", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    let x = controller(&w, car).slots[0].object.unwrap();
    w.obj_mut(car).unwrap().target = Some(Target::Object(victim));
    spawn::shot(&mut w, car, Target::Object(victim));
    // spec: S-0506 claims 4 and 5 (slot steps run before the controller step, so the launch pass that makes the slot preparing also releases it)
    // The aircraft's own flight is the aircraft lane's (a seam here), so the
    // hooks are read straight after a pass, before the aircraft's turn.
    assert!(testkit_run_until(&mut w, 120, |w| controller(w, car).state
        == ControllerState::Launching));
    spawn::pass(&mut w, car);
    assert_eq!(controller(&w, car).slots[0].state, SlotState::Attacking);
    let cell = w.obj(car).unwrap().pos.cell();
    let (dx, dy) = glasnost_sim::regions::STEPS[0];
    assert_eq!(
        w.obj(x).unwrap().dest,
        Some(Target::Cell(glasnost_map::CellXY::new(
            cell.x + dx,
            cell.y + dy
        )))
    );
    assert!(w.obj(x).unwrap().on_map);
    assert_eq!(w.obj(x).unwrap().target, Some(Target::Object(victim)));
    // spec: S-0506 claim 8 (no missile released: no reset, the controller keeps its target and cools down)
    assert_eq!(controller(&w, car).state, ControllerState::CoolingDown);
    // The target leaves play: attacking → returning at the next pass.
    glasnost_sim::objects::remove(&mut w, victim);
    spawn::pass(&mut w, car);
    assert_eq!(controller(&w, car).slots[0].state, SlotState::Returning);
    assert_eq!(w.obj(x).unwrap().dest, Some(Target::Object(car)));
}

/// A two-slot carrier CAR2 of HAWK aircraft (flagged with the missile-spawn
/// flag when `flagged`, never a rocket type), the carrier itself without
/// the flag (launch spacing 20: one launch every second pass). Returns the
/// world, the carrier, the victim and a second target.
fn carrier2(flagged: bool) -> (World, ObjectId, ObjectId, ObjectId) {
    let mut r = rules();
    r.registries
        .entry("aircraft".into())
        .or_default()
        .push("HAWK".into());
    r.sim.aircraft.insert(
        "HAWK".into(),
        UnitType {
            strength: Some(80),
            speed: Some(10),
            spawned: Some(true),
            missile_spawn: Some(flagged),
            ..Default::default()
        },
    );
    r.sim
        .vehicles
        .insert("CAR2".into(), launcher("HAWK", 2, "Launch"));
    r.sim.vehicles.get_mut("CAR2").unwrap().missile_spawn = Some(false);
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("CAR2".into());
    let mut w = game(&r);
    let car = spawn_at(&mut w, "CAR2", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    let other = spawn_at(&mut w, "VICTIM", 1, 8, 4, 0);
    (w, car, victim, other)
}

// spec: S-0506 Terms (missile slot: the flag does not make a slot a missile slot), claims 3, 6, 7 and 8 and the Slot table (a flagged type that is not a rocket type has a non-missile slot: it launches while the carrier moves, is sent to the step direction 0 neighbour of the spawner's cell, is re-sent by the preparing rows like an unflagged one (r40 Terms and Slot table: preparing, non-missile slot, flagged or not), a pending target adopted while it is preparing replaces the controller target, and claim 8 releases it); scenario variant of the flag/type distinction
#[test]
fn flagged_non_rocket_slot() {
    for flagged in [true, false] {
        let (mut w, car, victim, other) = carrier2(flagged);
        let x0 = controller(&w, car).slots[0].object.unwrap();
        w.obj_mut(car).unwrap().target = Some(Target::Object(victim));
        // The carrier moves (claim 6's motion lockout is for rocket slots only).
        open(&mut w);
        assert!(orders::issue(
            &mut w,
            ALPHA,
            orders::Order::Behaviour {
                actor: car,
                behaviour: glasnost_sim::rules::Behaviour::Move,
                attack_move: false,
                target: None,
                dest: Some(Target::Cell(testkit::mid(SIZE, -12, 0))),
            },
        ));
        testkit::run(&mut w, 3);
        spawn::shot(&mut w, car, Target::Object(victim));
        // The aircraft's own flight is a seam here: the hooks are read straight after a pass.
        assert!(testkit_run_until(&mut w, 120, |w| controller(w, car).state
            == ControllerState::Launching));
        assert!(
            glasnost_sim::occupancy::in_progress(&w, car),
            "the carrier is moving"
        );
        spawn::pass(&mut w, car);
        assert_eq!(
            controller(&w, car).slots[0].state,
            SlotState::Preparing,
            "launched while moving (flagged {flagged})"
        );
        let cell = w.obj(car).unwrap().pos.cell();
        let (dx, dy) = glasnost_sim::regions::STEPS[0];
        let first = Some(Target::Cell(glasnost_map::CellXY::new(
            cell.x + dx,
            cell.y + dy,
        )));
        assert_eq!(
            w.obj(x0).unwrap().dest,
            first,
            "claim 7: the step direction 0 neighbour"
        );
        // A pending target, then one pass with slot 0 preparing and slot 1 docked.
        spawn::shot(&mut w, car, Target::Object(other));
        spawn::pass(&mut w, car);
        assert_eq!(controller(&w, car).slots[0].state, SlotState::Preparing);
        assert_eq!(
            controller(&w, car).target,
            Some(Target::Object(other)),
            "claim 3"
        );
        let (sx, sy) = glasnost_sim::regions::STEPS[4];
        let now = w.obj(car).unwrap().pos.cell();
        let resent = Some(Target::Cell(glasnost_map::CellXY::new(
            now.x + sx,
            now.y + sy,
        )));
        assert_ne!(
            resent, first,
            "the re-send cell differs from the launch cell"
        );
        assert_eq!(
            w.obj(x0).unwrap().dest,
            resent,
            "r40: the preparing rows re-send it, flagged {flagged} or not"
        );
        // Claim 8: once every slot is preparing, a flagged one is detached (its slot empty), an unflagged one attacks.
        // A few passes later slot 1 launches and every slot is preparing.
        assert!(testkit_run_until(&mut w, 200, |w| controller(w, car).slots
            [0]
        .state
            != SlotState::Preparing));
        let want = if flagged {
            SlotState::Empty
        } else {
            SlotState::Attacking
        };
        assert_eq!(
            controller(&w, car).slots[0].state,
            want,
            "flagged {flagged}"
        );
    }
}

// spec: S-0506 Edge cases and S-0303 claim 6 (quirks.ground_detonation_misses_aircraft: a blast at the terrain surface does not search flying objects; off: it does, at half distance for an object in the air)
#[test]
fn ground_detonation_misses_aircraft() {
    for misses in [true, false] {
        let mut r = rules();
        r.globals.quirks.ground_detonation_misses_aircraft = misses;
        let (mut w, _, _, x) = fly_until(&r, Phase::Cruise);
        let p = w.obj(x).unwrap().pos;
        let h = w.obj(x).unwrap().health;
        let ground = w.surface_at(p.x, p.y);
        blast(&mut w, Pos::new(p.x, p.y, ground));
        let hit = w.obj(x).is_none_or(|o| o.health < h);
        assert_eq!(hit, !misses, "misses {misses}");
    }
}

// spec: S-0506 Edge cases (reset with a target out of range at adoption: the controller resets and launches nothing) and claim 5 (the S-0307 range test from where the spawner stands)
#[test]
fn reset_when_out_of_range_at_adoption() {
    let mut r = rules();
    r.sim.weapons.get_mut("Launch").unwrap().range = dec("3");
    let mut w = game(&r);
    let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    w.obj_mut(lna).unwrap().target = Some(Target::Object(victim));
    spawn::shot(&mut w, lna, Target::Object(victim));
    assert_eq!(controller(&w, lna).pending, Some(Target::Object(victim)));
    testkit::run(&mut w, 25);
    let c = controller(&w, lna);
    assert_eq!(c.state, ControllerState::Idle);
    assert_eq!((c.target, c.pending), (None, None));
    assert_eq!(c.slots[0].state, SlotState::Docked);
    let x = c.slots[0].object.unwrap();
    assert!(!w.obj(x).unwrap().on_map);
}

// spec: S-0506 F6 (lazy curve pitch: at the recorded distance with no height difference the pitch is P_f × π/2; with the aim straight below, ρ = 0, ℓ = −π/2 and q = 0)
#[test]
fn f6_lazy_pitch() {
    use glasnost_num::{F64, Rounding};
    use glasnost_sim::rocket::lazy_pitch;
    let rz = Rounding::TowardZero;
    let pf = F64::from_i64(3, rz)
        .div(F64::from_i64(5, rz), rz)
        .to_f32(rz);
    let hp = glasnost_num::geom::direction_constants().0;
    let at_start = lazy_pitch((1000, 0, 0), 1000, pf, true, rz);
    assert_eq!(at_start, pf.to_f64().mul(hp, rz).to_f32(rz));
    let below = lazy_pitch((0, 0, -400), 1000, pf, true, rz);
    assert_eq!(below, hp.negated().to_f32(rz));
}

// spec: S-0506 claim 4 (slot steps run in index order: the lowest docked slot launches first) and claim 6 (one launch per pass with the 9-tick spacing)
#[test]
fn claim_4_lowest_index_first() {
    let r = rules();
    let mut w = game(&r);
    let lnb = spawn_at(&mut w, "LNB", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    let first = controller(&w, lnb).slots[0].object.unwrap();
    let second = controller(&w, lnb).slots[1].object.unwrap();
    w.obj_mut(lnb).unwrap().target = Some(Target::Object(victim));
    spawn::shot(&mut w, lnb, Target::Object(victim));
    assert!(testkit_run_until(&mut w, 120, |w| controller(w, lnb).slots
        [0]
    .state
        != SlotState::Docked));
    let c = controller(&w, lnb);
    assert_eq!(c.slots[0].state, SlotState::Preparing);
    assert_eq!(c.slots[1].state, SlotState::Docked);
    assert!(w.obj(first).unwrap().on_map);
    assert!(!w.obj(second).unwrap().on_map);
}

// spec: S-0303 claim 11 and F7 (sector arrival order: a crossing object follows every object already in its destination sector; the global order of tracking starts, used with quirks.flying_search_sectors off, is unchanged by the crossing; ending and restarting tracking puts an object last)
#[test]
fn s0303_sector_arrival_order() {
    use glasnost_sim::combat::flying;
    let r = rules();
    let mut w = game(&r);
    let a = spawn_at(&mut w, "VICTIM", 1, 0, 0, 0);
    let b = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    let (sa, sb) = (
        flying::sector(&w, w.obj(a).unwrap().pos.cell()),
        flying::sector(&w, w.obj(b).unwrap().pos.cell()),
    );
    assert_ne!(sa, sb);
    flying::register(&mut w, a);
    flying::register(&mut w, b);
    let pb = w.obj(b).unwrap().pos;
    w.obj_mut(a).unwrap().pos = pb;
    flying::moved(&mut w, a);
    assert_eq!(
        w.flying.sectors[&sb],
        vec![b, a],
        "the later arrival follows"
    );
    assert_eq!(w.flying.order, vec![a, b], "the global order is unchanged");
    flying::unregister(&mut w, a);
    flying::register(&mut w, a);
    assert_eq!(
        w.flying.order,
        vec![b, a],
        "a new tracking period goes last"
    );
}

// ---------------------------------------------------------------------
// Claim 26: the shared kamikaze re-order cycle (Q-0645)
// ---------------------------------------------------------------------

/// Shoots `spawner` at `victim` and runs tick by tick until its salvo is
/// released (a kamikaze is enrolled); returns the release tick.
fn run_to_release(w: &mut World, spawner: ObjectId, victim: ObjectId) -> i32 {
    let x = controller(w, spawner).slots[0].object.unwrap();
    spawn::shot(w, spawner, Target::Object(victim));
    for _ in 0..400 {
        testkit::run(w, 1);
        if spawn::kamikaze(w, x) {
            return w.tick - 1;
        }
    }
    panic!("no release");
}

fn has_cycle_order(w: &World, x: ObjectId, cell: glasnost_map::CellXY) -> bool {
    let o = w.obj(x).unwrap();
    o.target == Some(Target::Cell(cell))
        && o.combat.ammo == 1
        && o.beh.queued == Some(glasnost_sim::rules::Behaviour::Attack)
}

// spec: S-0506 claim 26 (the first cycle is due at tick 1, then every 30 ticks of its own tick even with no kamikaze)
#[test]
fn claim_26_empty_cadence() {
    let r = rules();
    let mut w = game(&r);
    assert_eq!(w.spawned.next_reorder, 1);
    while w.tick <= 1 {
        testkit::run(&mut w, 1);
    }
    assert_eq!(w.spawned.next_reorder, 31);
    while w.tick <= 31 {
        testkit::run(&mut w, 1);
    }
    assert_eq!(w.spawned.next_reorder, 61);
}

// spec: S-0506 claims 12 and 26 (a release at r replaces the deadline with r + 2: no order in r or r + 1; in r + 2 ammunition 1, the release cell as target and attack queued, then every 30 ticks)
#[test]
fn claim_26_release_deadline_and_order() {
    let mut r = rules();
    // An armed rocket, as scenario R21 arms its flagged ordinary-flight aircraft: a weaponless
    // one answers *impossible* at its release cell (S-0301 claim 9) and drops it at a check tick
    // (S-0309 claim 29 r94, which covers the missile behaviour)
    r.sim.aircraft.get_mut("RKA").unwrap().primary = Some("Gun".into());
    let mut w = game(&r);
    let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    let x = controller(&w, lna).slots[0].object.unwrap();
    let rt = run_to_release(&mut w, lna, victim);
    let cell = w.obj(victim).unwrap().pos.cell();
    assert_eq!(w.spawned.kamikaze, vec![(x, cell)]);
    assert_eq!(w.spawned.next_reorder, rt + 2);
    assert!(!has_cycle_order(&w, x, cell), "none in r");
    testkit::run(&mut w, 1);
    assert!(!has_cycle_order(&w, x, cell), "none in r + 1");
    testkit::run(&mut w, 1);
    assert!(has_cycle_order(&w, x, cell), "the order in r + 2");
    assert_eq!(w.spawned.next_reorder, rt + 32);
}

// spec: S-0506 claim 26 (a release on an old due tick: the older group gets that tick's pre-turn order, the newly released object does not; the release then replaces the deadline)
#[test]
fn claim_26_release_on_a_due_tick() {
    let r = rules();
    let setup = |w: &mut World| {
        let lna = spawn_at(w, "LNA", 0, 0, 0, 0);
        let victim = spawn_at(w, "VICTIM", 1, 8, 0, 0);
        let old = spawn_at(w, "VICTIM", 0, -4, 4, 0);
        (lna, victim, old)
    };
    let mut w = game(&r);
    let (lna, victim, _) = setup(&mut w);
    let rt = run_to_release(&mut w, lna, victim);
    // The same game again, with an older enrolled object and the deadline on tick r.
    let mut w = game(&r);
    let (lna, victim, old) = setup(&mut w);
    let old_cell = glasnost_map::CellXY::new(3, 3);
    w.spawned.kamikaze.push((old, old_cell));
    w.spawned.next_reorder = rt;
    let x = controller(&w, lna).slots[0].object.unwrap();
    assert_eq!(run_to_release(&mut w, lna, victim), rt);
    // The old object's own turn may consume the order (claim 26), so only the ammunition is read here.
    assert_eq!(w.obj(old).unwrap().combat.ammo, 1);
    assert_ne!(
        w.obj(x).unwrap().target,
        Some(Target::Cell(w.obj(victim).unwrap().pos.cell()))
    );
    assert_eq!(w.spawned.next_reorder, rt + 2);
    assert_eq!(w.spawned.kamikaze[0].0, old, "later enrollments follow");
}

// spec: S-0506 claims 11 and 26 (a reset that releases a preparing flagged object replaces the deadline with its tick + 2; one that releases none changes no deadline)
#[test]
fn claim_26_reset_deadline() {
    let r = rules();
    let mut w = game(&r);
    let lnb = spawn_at(&mut w, "LNB", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    // No preparing object: no change.
    let d0 = w.spawned.next_reorder;
    spawn::impossible(&mut w, lnb);
    assert_eq!(w.spawned.next_reorder, d0);
    spawn::shot(&mut w, lnb, Target::Object(victim));
    assert!(testkit_run_until(&mut w, 120, |w| controller(w, lnb).slots
        [0]
    .state
        == SlotState::Preparing));
    spawn::impossible(&mut w, lnb);
    assert_eq!(w.spawned.next_reorder, w.tick + 2);
}

// spec: S-0506 claims 13 and 26 (slot clearing enrolls in descending slot index and does not replace the shared deadline)
#[test]
fn claim_26_slot_clearing_enrollment() {
    let mut r = rules();
    r.globals.quirks.rail_missile_vanishes_with_spawner = true;
    let mut w = game(&r);
    let lnb = spawn_at(&mut w, "LNB", 0, 0, 0, 0);
    let c = controller(&w, lnb);
    let (x0, x1) = (c.slots[0].object.unwrap(), c.slots[1].object.unwrap());
    for s in &mut w.obj_mut(lnb).unwrap().spawn.as_mut().unwrap().slots {
        s.state = SlotState::Preparing;
    }
    w.spawned.next_reorder = 77;
    spawn::clear_slots(&mut w, lnb);
    let order: Vec<ObjectId> = w.spawned.kamikaze.iter().map(|e| e.0).collect();
    assert_eq!(order, vec![x1, x0]);
    assert_eq!(w.spawned.next_reorder, 77);
}

// spec: S-0506 claim 26 (orders follow enrollment order, not creation order; removing an enrolled object keeps the others' relative order)
#[test]
fn claim_26_enrollment_order() {
    let r = rules();
    let mut w = game(&r);
    // Created in the order a, b, c; released in the order c, a, b.
    let a = spawn_at(&mut w, "LNA", 0, 0, -6, 0);
    let b = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
    let c = spawn_at(&mut w, "LNA", 0, 0, 6, 0);
    // Far targets of their own house (no automatic acquisition), so that every missile is still in flight at the last release.
    let va = spawn_at(&mut w, "VICTIM", 0, 18, -6, 0);
    let vb = spawn_at(&mut w, "VICTIM", 0, 18, 0, 0);
    let vc = spawn_at(&mut w, "VICTIM", 0, 18, 6, 0);
    let slot = |w: &World, s: ObjectId| controller(w, s).slots[0].object.unwrap();
    let (xa, xb, xc) = (slot(&w, a), slot(&w, b), slot(&w, c));
    assert!(xa < xb && xb < xc);
    run_to_release(&mut w, c, vc);
    run_to_release(&mut w, a, va);
    run_to_release(&mut w, b, vb);
    let order = |w: &World| -> Vec<ObjectId> { w.spawned.kamikaze.iter().map(|e| e.0).collect() };
    assert_eq!(order(&w), vec![xc, xa, xb]);
    glasnost_sim::objects::remove(&mut w, xa);
    assert_eq!(order(&w), vec![xc, xb]);
}

// spec: S-0506 claim 26 (a pending shared deadline survives a save and load: equal state hashes before and after the cycle)
#[test]
fn claim_26_save_load_pending_deadline() {
    let r = rules();
    let mut w = game(&r);
    let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    let rt = run_to_release(&mut w, lna, victim);
    assert_eq!(w.spawned.next_reorder, rt + 2);
    let bytes = glasnost_sim::save::save(&w);
    let mut w2 =
        glasnost_sim::save::load(&bytes, w.rules_arc(), testkit::catalogue()).expect("loads");
    assert_eq!(w2.spawned, w.spawned);
    assert_eq!(
        glasnost_sim::save::state_hash(&w),
        glasnost_sim::save::state_hash(&w2)
    );
    for _ in 0..40 {
        testkit::run(&mut w, 1);
        testkit::run(&mut w2, 1);
        assert_eq!(
            glasnost_sim::save::state_hash(&w),
            glasnost_sim::save::state_hash(&w2)
        );
    }
    // Cycles in r + 2 and r + 32 have run since.
    assert_eq!(w2.spawned.next_reorder, rt + 62);
}

// spec: S-0312 claim 16 and S-0506 claim 20 (interception neither restarts nor suspends the phase timer and adds no crash displacement: a destroyed missile keeps the undamaged twin's phase, timer and position through pause and tilt)
#[test]
fn s0312_claim_16_interception_keeps_timer_and_position() {
    let r = rules();
    let (mut w, _, _, x) = fly_until(&r, Phase::Pause);
    let mut twin = w.clone();
    w.obj_mut(x).unwrap().health = 0;
    glasnost_sim::combat::death::kill(&mut w, x, &hit(10));
    let mut compared = 0;
    for _ in 0..60 {
        testkit::run(&mut w, 1);
        testkit::run(&mut twin, 1);
        let (Some(a), Some(b)) = (w.obj(x), twin.obj(x)) else {
            break;
        };
        let (ra, rb) = (a.rocket.as_ref().unwrap(), b.rocket.as_ref().unwrap());
        if !matches!(ra.phase, Some(Phase::Pause | Phase::Tilt)) {
            break;
        }
        assert_eq!(ra.phase, rb.phase);
        assert_eq!(ra.started, rb.started);
        assert_eq!(a.pos, b.pos);
        compared += 1;
    }
    assert!(compared > 5, "pause and tilt were compared");
}

// spec: S-0506 claim 24 (a vehicle spawner's ordinary firing step on a tick with a due launch pass cannot supply that pass: the pass comes first in the turn)
#[test]
fn claim_24_vehicle_shot_waits_for_next_pass() {
    let r = rules();
    let mut w = game(&r);
    let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    click_attack(&mut w, lna, victim);
    // Run until the firing step makes its first spawner shot (pending set).
    let mut shot_tick = None;
    for _ in 0..200 {
        let t = w.tick;
        let due = controller(&w, lna).next_pass <= t;
        testkit::run(&mut w, 1);
        if controller(&w, lna).pending.is_some() || controller(&w, lna).target.is_some() {
            shot_tick = Some((t, due));
            break;
        }
    }
    let (t, _) = shot_tick.expect("a spawner shot");
    // Force the same situation with a pass due on the shot's tick: replay to t and make the pass due then.
    let mut w2 = game(&r);
    let lna2 = spawn_at(&mut w2, "LNA", 0, 0, 0, 0);
    let victim2 = spawn_at(&mut w2, "VICTIM", 1, 8, 0, 0);
    click_attack(&mut w2, lna2, victim2);
    while w2.tick < t {
        testkit::run(&mut w2, 1);
    }
    w2.obj_mut(lna2).unwrap().spawn.as_mut().unwrap().next_pass = t;
    testkit::run(&mut w2, 1);
    let c = controller(&w2, lna2);
    assert_eq!(
        c.pending,
        Some(Target::Object(victim2)),
        "the shot set pending"
    );
    assert_eq!(c.target, None, "the pass due on that tick did not adopt it");
    assert_eq!(c.state, ControllerState::Idle);
}

// ---------------------------------------------------------------------
// S-0309 claims 4, 23 and 24: spawner pending-target clears
// ---------------------------------------------------------------------

/// LNA with a nonempty controller target and a pending target.
/// The spawner weapon's range is cut to 5 cells, so that the two far
/// targets are out of range.
fn pending_setup(r: &Ruleset) -> (World, ObjectId, ObjectId, ObjectId) {
    let mut r = r.clone();
    r.sim.weapons.get_mut("Launch").unwrap().range = dec("5");
    let mut w = game(&r);
    let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
    let a = spawn_at(&mut w, "VICTIM", 1, -19, -19, 0);
    let b = spawn_at(&mut w, "VICTIM", 1, -19, 19, 0);
    w.obj_mut(lna).unwrap().spawn.as_mut().unwrap().target = Some(Target::Object(a));
    spawn::shot(&mut w, lna, Target::Object(b));
    assert_eq!(controller(&w, lna).pending, Some(Target::Object(b)));
    (w, lna, a, b)
}

// spec: S-0309 claim 4 and S-0506 claim 25 (dropping an unusable opportunity target requests the conditional pending clear, even when the same search picks a new target)
#[test]
fn s0309_claim_4_opportunity_drop_requests_clear() {
    let r = rules();
    let (mut w, lna, a, _) = pending_setup(&r);
    // An opportunity target out of range is unusable; an enemy in range is picked anew.
    let fresh = spawn_at(&mut w, "VICTIM", 1, 3, 0, 0);
    {
        let o = w.obj_mut(lna).unwrap();
        o.target = Some(Target::Object(a));
        o.combat.opportunity = true;
    }
    glasnost_sim::combat::acquire::search(
        &mut w,
        lna,
        glasnost_sim::combat::acquire::SearchKind::Range,
        glasnost_sim::combat::acquire::SearchContext::PerTick,
    );
    assert_eq!(w.obj(lna).unwrap().target, Some(Target::Object(fresh)));
    spawn::settle_clear(&mut w, lna);
    assert_eq!(controller(&w, lna).pending, None);
}

// spec: S-0309 claim 4 and S-0506 claim 25 (a search that finds nothing while already targetless does not cancel pending)
#[test]
fn s0309_claim_4_failed_search_keeps_pending() {
    let r = rules();
    let (mut w, lna, _, b) = pending_setup(&r);
    glasnost_sim::combat::acquire::search(
        &mut w,
        lna,
        glasnost_sim::combat::acquire::SearchKind::Range,
        glasnost_sim::combat::acquire::SearchContext::PerTick,
    );
    assert_eq!(w.obj(lna).unwrap().target, None);
    spawn::settle_clear(&mut w, lna);
    assert_eq!(controller(&w, lna).pending, Some(Target::Object(b)));
}

// spec: S-0309 claim 23 and S-0506 claim 25 (an attack-move search run as for an object with no target requests no pending clear, with either quirk setting: not when it finds nothing, nor when it restores the remembered target)
#[test]
fn s0309_claim_23_attack_move_search_keeps_pending() {
    for on in [true, false] {
        let mut r = rules();
        r.globals.quirks.spawner_pending_target_survives_idle_clear = on;
        // Destination memory, engaged, present target out of range: the search finds nothing.
        let (mut w, lna, a, b) = pending_setup(&r);
        {
            let o = w.obj_mut(lna).unwrap();
            o.target = Some(Target::Object(a));
            o.attack_move = Some(glasnost_sim::world::AttackMoveMemory::Dest(Target::Cell(
                o.pos.cell(),
            )));
            o.combat.engaged = true;
            o.targeting_timer = 0;
        }
        glasnost_sim::combat::acquire::per_tick(&mut w, lna);
        assert_eq!(w.obj(lna).unwrap().target, None, "the search found nothing");
        spawn::settle_clear(&mut w, lna);
        assert_eq!(
            controller(&w, lna).pending,
            Some(Target::Object(b)),
            "quirk {on}"
        );
        // Target memory out of range: the search finds nothing and restores the remembered target.
        let (mut w, lna, a, b) = pending_setup(&r);
        {
            let o = w.obj_mut(lna).unwrap();
            o.target = Some(Target::Object(a));
            o.attack_move = Some(glasnost_sim::world::AttackMoveMemory::Target(
                Target::Object(a),
            ));
            o.targeting_timer = 0;
        }
        glasnost_sim::combat::acquire::per_tick(&mut w, lna);
        assert_eq!(
            w.obj(lna).unwrap().target,
            Some(Target::Object(a)),
            "restored"
        );
        spawn::settle_clear(&mut w, lna);
        assert_eq!(
            controller(&w, lna).pending,
            Some(Target::Object(b)),
            "quirk {on}"
        );
    }
}

// spec: S-0516 claim 22 and Edge cases (Q-0795: rocket flight supplies no cell-arrival update, so the sensor disc added when the missile entered play stays at its launch cell; quirks.flying_sensor_disc_stationary off: each cell-boundary crossing in flight moves it to the new cell)
#[test]
fn s0516_claim_22_rocket_flight_moves_no_sensor_disc() {
    for stationary in [true, false] {
        let mut r = rules();
        r.globals.quirks.flying_sensor_disc_stationary = stationary;
        r.sim.aircraft.get_mut("RKA").unwrap().combat.sensors_sight = Some(2);
        let mut w = game(&r);
        let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
        let victim = spawn_at(&mut w, "VICTIM", 1, 12, 0, 0);
        let x = controller(&w, lna).slots[0].object.unwrap();
        let mut launch = None;
        spawn::shot(&mut w, lna, Target::Object(victim));
        let mut crossed = 0;
        for _ in 0..300 {
            testkit::run(&mut w, 1);
            let Some(o) = w.obj(x) else { break };
            if o.rocket.is_none() {
                continue;
            }
            let start = *launch.get_or_insert(o.pos.cell());
            if o.pos.cell() == start {
                continue;
            }
            crossed += 1;
            let disc = o.cloak.disc.map(|(_, c)| c);
            if stationary {
                assert_eq!(disc, Some(start), "the disc stays at the launch cell");
            } else {
                assert_eq!(disc, Some(o.pos.cell()), "the disc follows the crossing");
            }
        }
        assert!(crossed > 0, "the missile left its launch cell");
    }
}

// spec: S-0309 claim 24 and S-0506 claim 25 (a nonempty target redirected to none requests the conditional clear even when the old own target was empty; an explicit clear of an empty target does not, quirk on)
#[test]
fn s0309_claim_24_redirect_to_none_requests_clear() {
    let r = rules();
    let (mut w, lna, _, b) = pending_setup(&r);
    assert_eq!(w.obj(lna).unwrap().target, None);
    spawn::clear_own_target(w.obj_mut(lna).unwrap());
    spawn::settle_clear(&mut w, lna);
    assert_eq!(controller(&w, lna).pending, Some(Target::Object(b)));
    spawn::redirected_to_none(w.obj_mut(lna).unwrap());
    spawn::settle_clear(&mut w, lna);
    assert_eq!(controller(&w, lna).pending, None);
    // With an empty controller target the request leaves pending (quirk on); off, it clears pending.
    for on in [true, false] {
        let mut r = rules();
        r.globals.quirks.spawner_pending_target_survives_idle_clear = on;
        let (mut w, lna, _, b) = pending_setup(&r);
        w.obj_mut(lna).unwrap().spawn.as_mut().unwrap().target = None;
        spawn::redirected_to_none(w.obj_mut(lna).unwrap());
        spawn::settle_clear(&mut w, lna);
        let expect = on.then_some(Target::Object(b));
        assert_eq!(controller(&w, lna).pending, expect, "quirk {on}");
    }
}

// spec: S-0105 Terms (docked: a spawned aircraft inside its spawner is docked; one that has been launched, or destroyed and not yet replaced, is not; a type without a controller has none) and claim 12 (the drawing's alternative-body state reads it)
#[test]
fn any_docked_follows_the_slots() {
    let r = rules();
    let mut w = game(&r);
    let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    assert_eq!(spawn::any_docked(&w, lna), Some(true));
    assert_eq!(spawn::any_docked(&w, victim), None);
    let missile = controller(&w, lna).slots[0].object.unwrap();
    click_attack(&mut w, lna, victim);
    assert!(testkit_run_until(&mut w, 200, |w| w
        .obj(missile)
        .is_some_and(|o| o.on_map)));
    // The only slot is launched: none is docked while the missile is out.
    assert_eq!(spawn::any_docked(&w, lna), Some(false));
}

// spec: S-0105 claim 41 and S-0506 claims 14 to 18, 24 (the drawing reads the missile's current single-precision pitch, read-only, and its set's initial and final pitch fractions; an object that is not a rocket missile has none)
#[test]
fn drawing_reads_the_current_pitch() {
    let r = rules();
    let mut w = game(&r);
    let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    assert_eq!(glasnost_sim::rocket::drawing(&w, victim), None);
    let missile = controller(&w, lna).slots[0].object.unwrap();
    assert_eq!(
        glasnost_sim::rocket::drawing(&w, missile),
        None,
        "docked: no flight state yet"
    );
    click_attack(&mut w, lna, victim);
    assert!(testkit_run_until(&mut w, 200, |w| w
        .obj(missile)
        .is_some_and(|o| o.on_map)));
    let mut seen = std::collections::BTreeSet::new();
    for _ in 0..80 {
        testkit::run(&mut w, 1);
        let Some(o) = w.obj(missile) else { break };
        let rs = o.rocket.as_ref().unwrap();
        let d = glasnost_sim::rocket::drawing(&w, missile).unwrap();
        assert_eq!(d.pitch, rs.pitch.0, "the current pitch");
        seen.insert(d.pitch);
        let set = w.rules().missiles.set(rs.set);
        assert_eq!(
            (d.initial, d.final_),
            (set.pitch_initial.0, set.pitch_final.0)
        );
    }
    assert!(seen.len() > 3, "the pitch changes through the flight");
}

/// The rules with RKW given an explosion list (RKBOOM) and the
/// conventional flag, and a splash list (RKSPLASH).
fn detonation_rules() -> Ruleset {
    let mut r = rules();
    let h = r.sim.warheads.get_mut("RKW").unwrap();
    h.anim_list = Some(vec!["RKBOOM".into()]);
    h.conventional = Some(true);
    r.sim.general.splash_list = Some(vec!["RKSPLASH".into()]);
    r
}

/// A world of [`detonation_rules`] with the cell at the middle made water;
/// returns the world and that cell.
fn detonation_world() -> (World, glasnost_map::CellXY) {
    let mut w = game(&detonation_rules());
    let c = testkit::mid(SIZE, 0, 0);
    w.cells.grid.get_mut(c).unwrap().land = glasnost_map::terrain::land::WATER;
    w.effects.clear();
    (w, c)
}

/// The explosion animations of the effects: (name, position, depth
/// adjustment).
fn explosion_anims(w: &World) -> Vec<(String, Pos, Option<i32>)> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Anim {
                name,
                pos,
                z_adjust,
                ..
            } => Some((name.clone(), *pos, *z_adjust)),
            _ => None,
        })
        .collect()
}

fn detonation_point(w: &World) -> Option<Pos> {
    w.effects.iter().find_map(|e| match e {
        Effect::Detonation { pos, .. } => Some(*pos),
        _ => None,
    })
}

// spec: S-0506 claim 19 and Constants (the explosion animation and the area damage take the nose point exactly, with no height correction, and the animation's depth adjustment is -15 in place of its type's)
#[test]
fn claim_19_detonation_at_the_nose_point_with_depth_adjustment() {
    let (mut w, c) = detonation_world();
    // A dry cell next to the water one; a nose point below the surface.
    let dry = glasnost_map::CellXY::new(c.x + 1, c.y);
    let at = Pos::new(dry.x * 256 + 100, dry.y * 256 + 40, -30);
    let wh = warhead(&w, "RKW");
    glasnost_sim::combat::projectile::detonate_rocket(&mut w, wh, 30, at, None);
    assert_eq!(
        explosion_anims(&w),
        vec![("RKBOOM".to_string(), at, Some(-15))]
    );
    assert_eq!(
        detonation_point(&w),
        Some(at),
        "the area damage at the nose point"
    );
}

// spec: S-0506 claim 19 and F9 (the F5 land type is that of the cell containing the nose point, x and y divided by 256 truncated, whatever the height; F5's own splash height test still applies)
#[test]
fn claim_19_land_type_from_the_nose_cell() {
    let detonate = |dx: i32, dy: i32, z: i32| {
        let (mut w, c) = detonation_world();
        let at = Pos::new(c.x * 256 + dx, c.y * 256 + dy, z);
        let wh = warhead(&w, "RKW");
        glasnost_sim::combat::projectile::detonate_rocket(&mut w, wh, 30, at, None);
        explosion_anims(&w)[0].0.clone()
    };
    // Low over the water cell, at its last lepton: the splash.
    assert_eq!(detonate(255, 255, 100), "RKSPLASH");
    // One lepton further east or south: the dry neighbour.
    assert_eq!(detonate(256, 255, 100), "RKBOOM");
    assert_eq!(detonate(255, 256, 100), "RKBOOM");
    // High over the water cell: water, but above F5's splash height.
    assert_eq!(detonate(128, 128, 600), "RKBOOM");
}

// spec: S-0506 claim 19 (a missile in flight detonates through the same rule: its explosion animation at the detonation point, with the depth adjustment -15)
#[test]
fn claim_19_flight_detonation_carries_the_depth_adjustment() {
    let r = detonation_rules();
    let (mut w, _, _, x) = fly_until(&r, Phase::Cruise);
    let mut seen = None;
    for _ in 0..600 {
        w.effects.clear();
        testkit::run(&mut w, 1);
        if w.obj(x).is_none() {
            seen = detonation_point(&w).map(|p| (p, explosion_anims(&w)));
            break;
        }
    }
    let (at, anims) = seen.expect("the missile detonates");
    assert!(
        anims.contains(&("RKBOOM".to_string(), at, Some(-15))),
        "{anims:?} at {at:?}"
    );
}

/// The rules with a ground anti-air vehicle FLAK: a driving vehicle whose
/// weapon Flak (range 12 cells, damage 1) fires a projectile that can hit
/// flying objects. RKA is made too strong for that weapon to destroy.
fn flak_rules() -> Ruleset {
    let mut r = rules();
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("FLAK".into());
    let s = &mut r.sim;
    s.aircraft.get_mut("RKA").unwrap().strength = Some(100_000);
    s.projectiles.insert(
        "FlakShell".into(),
        glasnost_rules::sim::ProjectileType {
            aa: Some(true),
            inviso: Some(true),
            ..Default::default()
        },
    );
    s.weapons.insert(
        "Flak".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(30),
            range: dec("12"),
            projectile: Some("FlakShell".into()),
            warhead: Some("HE".into()),
            ..Default::default()
        },
    );
    s.vehicles.insert(
        "FLAK".into(),
        UnitType {
            strength: Some(400),
            speed: Some(6),
            rot: Some(5),
            locomotion_class: Some(LocomotionClass::Wheel),
            locomotor: Some(Locomotor::Drive),
            primary: Some("Flak".into()),
            ..Default::default()
        },
    );
    r
}

// spec: S-0309 claim 27 (playtest note 36: a missile removed at its detonation, S-0506 claim 19, is dropped by every holder, a ground anti-air vehicle on guard included, inside the removal, with the timer draw; the holder then searches again)
#[test]
fn a_detonated_missile_is_dropped_by_a_ground_anti_air_holder() {
    let r = flak_rules();
    let mut w = game(&r);
    let lna = spawn_at(&mut w, "LNA", 0, 0, 0, 0);
    let victim = spawn_at(&mut w, "VICTIM", 1, 8, 0, 0);
    let flak = spawn_at(&mut w, "FLAK", 1, 8, 4, 0);
    let x = controller(&w, lna).slots[0].object.unwrap();
    spawn::shot(&mut w, lna, Target::Object(victim));
    assert!(testkit_run_until(&mut w, 300, |w| phase_of(w, x) == Some(Phase::Cruise)));
    // The anti-air vehicle holds the missile, with more than 10 ticks on its timer, until the missile detonates.
    let mut detonated = false;
    for _ in 0..600 {
        let tick = w.tick;
        {
            let o = w.obj_mut(flak).unwrap();
            o.target = Some(Target::Object(x));
            o.targeting_timer = tick + 50;
        }
        w.effects.clear();
        testkit::run(&mut w, 1);
        if w.obj(x).is_none() {
            detonated = detonation_point(&w).is_some();
            break;
        }
    }
    assert!(detonated, "the missile detonated and left play");
    let o = w.obj(flak).unwrap();
    assert_ne!(o.target, Some(Target::Object(x)), "the holder dropped it");
    let left = o.targeting_timer - w.tick;
    assert!(
        (0..=8).contains(&left),
        "the drop's timer draw: {left} left"
    );
    // It searches again: the enemy launcher within its weapon's reach becomes its target.
    assert!(
        testkit_run_until(&mut w, 60, |w| w.obj(flak).unwrap().target
            == Some(Target::Object(lna))),
        "the anti-air vehicle searched again"
    );
}
