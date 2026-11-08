//! Claim tests of S-0506 revision 40 (Q-2811 answered). A controller target
//! lost while a pending target waits is cleared at the loss, at each loss
//! event alike, and the loss gives no reset. At the controller's next pass a
//! launching controller resets for having no controller target, unless a
//! non-missile slot whose object is preparing, attacking or returning has
//! adopted the pending target earlier in that pass; the reset releases its
//! preparing missiles with no target and clears the pending target, so the
//! salvo ends there (claims 5, 11 and 25, the Slot and Controller tables,
//! `quirks.launching_spawner_resets_after_target_loss`; scenario R7 part f).
//! A non-missile slot of a flagged type that is not a rocket type is moved by
//! the Slot table's preparing rows like any other non-missile slot (Terms,
//! *Missile type*). On synthetic data: invented types, weapons and rocket
//! parameter values, none of them stock values.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    Decimal, LocomotionClass, Locomotor, SequenceEntry, UnitType, WarheadType, WeaponType,
};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::rules::Kind;
use glasnost_sim::spawn::{self, ControllerState, SlotState};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};
use glasnost_sim::{objects, regions};

const SIZE: i32 = 40;

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

/// The rocket type RKB (set B) and the launcher LNC (four RKB slots, so
/// the 9-tick launch spacing keeps one launch a pass); the aircraft HAWK
/// (no missile-spawn flag) and HAWKF (the flag, not a rocket type); the
/// carriers CAR1 (one HAWK slot), CAR2 (two HAWK slots) and CARF (two
/// HAWKF slots), without the flag themselves (spacing 20); the spawner
/// weapon Launch; the target vehicle VICTIM and the testkit's WALKER with
/// a death sequence. Invented set parameters (set A: pause 4, tilt 10; set
/// B: pause 3, tilt 8).
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let push = |r: &mut Ruleset, reg: &str, n: &str| {
        r.registries.entry(reg.into()).or_default().push(n.into());
    };
    for n in ["RKA", "RKB", "HAWK", "HAWKF"] {
        push(&mut r, "aircraft", n);
    }
    for n in ["LNC", "CAR1", "CAR2", "CARF", "VICTIM"] {
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
    let hawk = |flagged| UnitType {
        strength: Some(80),
        speed: Some(10),
        spawned: Some(true),
        missile_spawn: Some(flagged),
        ..Default::default()
    };
    let launcher = |spawns: &str, n, flagged| {
        let mut t = UnitType {
            strength: Some(400),
            speed: Some(6),
            rot: Some(5),
            locomotion_class: Some(LocomotionClass::Wheel),
            locomotor: Some(Locomotor::Drive),
            primary: Some("Launch".into()),
            missile_spawn: Some(flagged),
            ..Default::default()
        };
        t.combat.spawns = Some(spawns.into());
        t.combat.spawns_number = Some(n);
        t.combat.spawn_regen_rate = Some(50);
        t.combat.spawn_reload_rate = Some(0);
        t.combat.primary_fire_flh = Some([60, 0, 80]);
        t
    };
    let s = &mut r.sim;
    s.aircraft.insert("RKA".into(), rocket(50));
    s.aircraft.insert("RKB".into(), rocket(40));
    s.aircraft.insert("HAWK".into(), hawk(false));
    s.aircraft.insert("HAWKF".into(), hawk(true));
    s.vehicles.insert("LNC".into(), launcher("RKB", 4, true));
    s.vehicles.insert("CAR1".into(), launcher("HAWK", 1, false));
    s.vehicles.insert("CAR2".into(), launcher("HAWK", 2, false));
    s.vehicles.insert("CARF".into(), launcher("HAWKF", 2, false));
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

fn release_cell(w: &World, x: ObjectId) -> Option<glasnost_map::CellXY> {
    w.spawned.kamikaze.iter().find(|e| e.0 == x).map(|e| e.1)
}

/// The F8 neighbour of missile `x`'s own cell for its body facing.
fn f8_cell(w: &World, x: ObjectId) -> glasnost_map::CellXY {
    let o = w.obj(x).unwrap();
    let k = spawn::release_direction(o.body.value(w.tick));
    let (dx, dy) = regions::STEPS[k];
    let c = o.pos.cell();
    glasnost_map::CellXY::new(c.x + dx, c.y + dy)
}

fn run_until(w: &mut World, max: i32, mut done: impl FnMut(&World) -> bool) -> bool {
    for _ in 0..max {
        testkit::run(w, 1);
        if done(w) {
            return true;
        }
    }
    false
}

/// A loss event of S-0506 claim 11.
#[derive(Clone, Copy, Debug)]
enum Loss {
    /// The hit that destroys it, while it then stays in play (a death
    /// sequence).
    Hit,
    /// Going inside a transport.
    Inside,
    /// Its removal.
    Removal,
    /// The loss hook that the start of cloaking calls (and going
    /// underground would): `spawn::target_lost`.
    Cloak,
}

const LOSSES: [Loss; 4] = [Loss::Hit, Loss::Inside, Loss::Removal, Loss::Cloak];

fn lose(w: &mut World, r: &Ruleset, id: ObjectId, how: Loss) {
    match how {
        Loss::Hit => {
            let w1 = r.sim.warheads.keys().position(|k| k == "W1").unwrap() as u32;
            damage::apply(
                w,
                id,
                Hit {
                    amount: 100_000,
                    warhead: Some(w1),
                    distance: 0,
                    attacker: None,
                    house: None,
                    ignore_defences: false,
                    no_escape: false,
                },
            );
        }
        Loss::Inside => objects::take_inside(w, id),
        Loss::Removal => objects::remove(w, id),
        Loss::Cloak => spawn::target_lost(w, id),
    }
}

/// LNC with its first missile preparing at a WALKER (`first`), and a
/// second shot at a VICTIM (`second`) waiting as the pending target:
/// scenario R7 part f's set-up. Returns the world, LNC, the preparing
/// missile, `first` and `second`.
fn lnc_with_pending(r: &Ruleset) -> (World, ObjectId, ObjectId, ObjectId, ObjectId) {
    let mut w = testkit::game_with(r, SIZE, 7);
    let lnc = put(&mut w, Kind::Vehicle, "LNC", 0, 0, 0);
    let first = put(&mut w, Kind::Infantry, "WALKER", 1, 8, 0);
    let second = put(&mut w, Kind::Vehicle, "VICTIM", 1, 8, 4);
    let x = controller(&w, lnc).slots[0].object.unwrap();
    spawn::shot(&mut w, lnc, Target::Object(first));
    assert!(
        run_until(&mut w, 200, |w| w.obj(x).is_some_and(|o| o.on_map)),
        "set-up: a launch"
    );
    let c = controller(&w, lnc);
    assert_eq!(c.state, ControllerState::Launching, "set-up");
    assert_eq!(c.slots[0].state, SlotState::Preparing, "set-up");
    assert_eq!(c.slots[1].state, SlotState::Docked, "set-up");
    spawn::set_own_target(w.obj_mut(lnc).unwrap(), Some(Target::Object(second)));
    spawn::shot(&mut w, lnc, Target::Object(second));
    let c = controller(&w, lnc);
    assert_eq!(c.pending, Some(Target::Object(second)), "set-up: pending");
    assert_eq!(c.target, Some(Target::Object(first)), "set-up");
    (w, lnc, x, first, second)
}

/// The slots after the first are still docked, their objects off the map:
/// no launch.
fn no_later_launch(w: &World, lnc: ObjectId, what: &str) {
    let c = controller(w, lnc);
    for s in &c.slots[1..] {
        assert_eq!(s.state, SlotState::Docked, "{what}: no launch");
        assert!(
            w.obj(s.object.unwrap()).is_some_and(|o| !o.on_map),
            "{what}: no launch"
        );
    }
}

// spec: S-0506 claim 11 (r40, Q-2811, A loss while a pending target waits: the controller target is cleared at the loss, at each loss event alike, and the loss itself gives no reset; a launching controller then resets at its next pass for having no controller target, claim 5; that reset releases each preparing missile with no target, so with claim 12's cell for no target, empties its slot with a regeneration countdown and clears the pending target; no launch at the pending target in that pass, and a rocket missile so released flies on to its aim point, claim 14), the Controller table's launching row (r40) and claim 25 (r40); scenario R7 part f, runs 1, 2 and 4, and the unmeasured events (removal, cloaking)
#[test]
fn r7_f_a_loss_while_a_pending_target_waits_clears_then_the_next_pass_resets() {
    for how in LOSSES {
        let r = rules();
        let (mut w, lnc, x, first, second) = lnc_with_pending(&r);
        let aim = w.obj(x).unwrap().rocket.as_ref().unwrap().aim;
        assert!(aim.is_some(), "set-up: the aim point fixed at the launch");
        lose(&mut w, &r, first, how);
        let what = format!("{how:?}");
        let c = controller(&w, lnc);
        assert_eq!(c.target, None, "{what}: cleared at the loss");
        assert_eq!(c.state, ControllerState::Launching, "{what}: no reset");
        assert_eq!(c.pending, Some(Target::Object(second)), "{what}");
        assert_eq!(c.slots[0].state, SlotState::Preparing, "{what}");
        assert_eq!(release_cell(&w, x), None, "{what}: not released");
        // The controller's next pass.
        spawn::pass(&mut w, lnc);
        let c = controller(&w, lnc);
        assert_eq!(c.state, ControllerState::Idle, "{what}: the reset");
        assert_eq!(c.target, None, "{what}");
        assert_eq!(c.pending, None, "{what}: the pending target cleared");
        assert_eq!(c.slots[0].state, SlotState::Empty, "{what}: not on the rail");
        assert_eq!(c.slots[0].until, w.tick + 50, "{what}: regeneration");
        assert_eq!(
            release_cell(&w, x),
            Some(f8_cell(&w, x)),
            "{what}: claim 12's cell for no target"
        );
        assert_eq!(
            w.obj(x).unwrap().rocket.as_ref().unwrap().aim,
            aim,
            "{what}: it flies on to its aim point"
        );
        no_later_launch(&w, lnc, &what);
        // No launch at the second target until a later shot is adopted by an idle pass.
        spawn::pass(&mut w, lnc);
        no_later_launch(&w, lnc, &what);
        assert_eq!(controller(&w, lnc).target, None, "{what}");
    }
}

// spec: S-0506 claim 11 (r40: the salvo's next launch needs a later shot of its spawner weapon to supply a pending target again, claim 3, and an idle pass to adopt it, claim 5); scenario R7 part f's expected launch at the second target only after a later shot
#[test]
fn r7_f_the_next_launch_needs_a_later_shot_and_an_idle_pass() {
    let r = rules();
    let (mut w, lnc, _, first, second) = lnc_with_pending(&r);
    lose(&mut w, &r, first, Loss::Hit);
    spawn::pass(&mut w, lnc);
    assert_eq!(controller(&w, lnc).state, ControllerState::Idle);
    spawn::shot(&mut w, lnc, Target::Object(second));
    spawn::pass(&mut w, lnc);
    let c = controller(&w, lnc);
    assert_eq!(c.target, Some(Target::Object(second)), "adopted when idle");
    assert_eq!(c.state, ControllerState::Launching);
    let y = c.slots[1].object.unwrap();
    assert!(
        run_until(&mut w, 40, |w| w.obj(y).is_some_and(|o| o.on_map)),
        "a launch at the second target"
    );
    assert_eq!(w.obj(y).unwrap().dest, Some(Target::Object(second)));
}

// spec: S-0506 claim 11 (r40: when the pending target is lost as well before it is adopted, it is cleared with no reset, claim 25, and a launching controller resets at its next pass; neither lost object gets a launch); scenario R7 part f run 3
#[test]
fn r7_f_both_lost_the_next_pass_resets_and_launches_at_neither() {
    for on in [true, false] {
        let mut r = rules();
        r.globals.quirks.launching_spawner_resets_after_target_loss = on;
        let (mut w, lnc, x, first, second) = lnc_with_pending(&r);
        lose(&mut w, &r, first, Loss::Hit);
        lose(&mut w, &r, second, Loss::Removal);
        let what = format!("switch {on}");
        let c = controller(&w, lnc);
        assert_eq!(c.pending, None, "{what}: cleared at its loss");
        assert_eq!(c.target, None, "{what}");
        assert_eq!(c.state, ControllerState::Launching, "{what}: no reset");
        spawn::pass(&mut w, lnc);
        let c = controller(&w, lnc);
        assert_eq!(c.state, ControllerState::Idle, "{what}: the reset");
        assert_eq!(c.slots[0].state, SlotState::Empty, "{what}");
        assert_eq!(release_cell(&w, x), Some(f8_cell(&w, x)), "{what}");
        no_later_launch(&w, lnc, &what);
    }
}

// spec: S-0506 Edge cases (`quirks.launching_spawner_resets_after_target_loss` off: a launching controller with no controller target and a pending target waiting adopts the pending target at that check, with no range test, as a launch does, and goes on launching; its preparing missiles stay preparing until claim 8 releases them with that target)
#[test]
fn quirk_off_the_launching_controller_adopts_the_pending_target() {
    let mut r = rules();
    r.globals.quirks.launching_spawner_resets_after_target_loss = false;
    let (mut w, lnc, x, first, second) = lnc_with_pending(&r);
    lose(&mut w, &r, first, Loss::Hit);
    assert_eq!(controller(&w, lnc).target, None, "cleared at the loss");
    spawn::pass(&mut w, lnc);
    let c = controller(&w, lnc);
    assert_eq!(c.state, ControllerState::Launching, "no reset");
    assert_eq!(c.target, Some(Target::Object(second)), "adopted");
    assert_eq!(c.pending, None);
    assert_eq!(c.slots[0].state, SlotState::Preparing, "still preparing");
    assert_eq!(release_cell(&w, x), None);
    // The later slots launch at it, and claim 8 releases the salvo with it.
    assert!(run_until(&mut w, 200, |w| controller(w, lnc).slots[0].state
        != SlotState::Preparing));
    let at = w.obj(second).unwrap().pos.cell();
    assert_eq!(release_cell(&w, x), Some(at), "released with that target");
    assert_eq!(controller(&w, lnc).slots[0].state, SlotState::OnRail);
    let y = controller(&w, lnc).slots[1].object.unwrap();
    assert_eq!(w.obj(y).unwrap().dest, Some(Target::Object(second)));
}

// spec: S-0506 Edge cases (`quirks.launching_spawner_resets_after_target_loss` on, the default, as the original)
#[test]
fn the_switch_defaults_on() {
    assert!(
        glasnost_rules::quirks::Quirks::default().launching_spawner_resets_after_target_loss
    );
}

/// A carrier `name` with its slot 0 object launched and preparing at a
/// VICTIM (`first`), its other slots docked, and a shot at a second VICTIM
/// (`second`) waiting as the pending target.
fn carrier_with_pending(r: &Ruleset, name: &str) -> (World, ObjectId, ObjectId, ObjectId, ObjectId) {
    let mut w = testkit::game_with(r, SIZE, 7);
    let car = put(&mut w, Kind::Vehicle, name, 0, 0, 0);
    let first = put(&mut w, Kind::Vehicle, "VICTIM", 1, 8, 0);
    let second = put(&mut w, Kind::Vehicle, "VICTIM", 1, 8, 4);
    let x = controller(&w, car).slots[0].object.unwrap();
    w.obj_mut(car).unwrap().target = Some(Target::Object(first));
    spawn::shot(&mut w, car, Target::Object(first));
    // The aircraft's own flight is a seam here: the hooks are read straight
    // after a pass, as in claims_s0506.rs.
    assert!(run_until(&mut w, 120, |w| controller(w, car).state
        == ControllerState::Launching));
    spawn::pass(&mut w, car);
    let c = controller(&w, car);
    assert_eq!(c.slots[0].state, SlotState::Preparing, "set-up: {name}");
    assert_eq!(c.slots[1].state, SlotState::Docked, "set-up: {name}");
    assert_eq!(c.state, ControllerState::Launching, "set-up: {name}");
    // A later shot at another target: the carrier's own target is that
    // target (so the loss of the first is no own-target clear, claim 25)
    // and the shot waits as pending (claim 3).
    spawn::set_own_target(w.obj_mut(car).unwrap(), Some(Target::Object(second)));
    spawn::shot(&mut w, car, Target::Object(second));
    assert_eq!(controller(&w, car).pending, Some(Target::Object(second)));
    (w, car, x, first, second)
}

/// The step direction 4 neighbour of `car`'s cell.
fn step_4(w: &World, car: ObjectId) -> Option<Target> {
    let c = w.obj(car).unwrap().pos.cell();
    let (dx, dy) = regions::STEPS[4];
    Some(Target::Cell(glasnost_map::CellXY::new(c.x + dx, c.y + dy)))
}

// spec: S-0506 claim 11 (r40: a launching controller adopts the pending target in its next pass when a non-missile slot whose object is preparing adopts it, claim 3, before the controller's check of the same pass, and goes on launching with it) and the Slot table (r40: that adoption counts for the controller's launching check of the same pass; the preparing row for a non-missile slot, flagged or not, with a controller target after the adoption: re-sent); scenario R7 part f run 5 (an ordinary carrier: its preparing aircraft does not return at that pass), and unmeasured for a flagged slot that is not a rocket slot
#[test]
fn r7_f_a_preparing_non_missile_slot_adopts_and_the_controller_goes_on() {
    for (name, on) in [
        ("CAR2", true),
        ("CARF", true),
        ("CAR2", false),
        ("CARF", false),
    ] {
        let mut r = rules();
        r.globals.quirks.launching_spawner_resets_after_target_loss = on;
        let (mut w, car, x, first, second) = carrier_with_pending(&r, name);
        lose(&mut w, &r, first, Loss::Removal);
        let what = format!("{name}, switch {on}");
        let c = controller(&w, car);
        assert_eq!(c.target, None, "{what}: cleared at the loss");
        assert_eq!(c.state, ControllerState::Launching, "{what}: no reset");
        spawn::pass(&mut w, car);
        let c = controller(&w, car);
        assert_eq!(c.state, ControllerState::Launching, "{what}: no reset");
        assert_eq!(c.target, Some(Target::Object(second)), "{what}: adopted");
        assert_eq!(c.pending, None, "{what}");
        assert_eq!(c.slots[0].state, SlotState::Preparing, "{what}: not returning");
        assert_eq!(w.obj(x).unwrap().dest, step_4(&w, car), "{what}: re-sent");
        assert!(!spawn::kamikaze(&w, x), "{what}: not released");
    }
}

// spec: S-0506 claim 11 (r40: with the pending target lost as well, a preparing object of a flagged slot that is not a rocket slot has turned returning earlier in the pass for want of a controller target, Slot table, so the launching controller's reset in that pass does not release it, and it flies back to the spawner); unmeasured
#[test]
fn both_lost_a_flagged_non_rocket_object_returns_and_is_not_released() {
    let r = rules();
    let (mut w, car, x, first, second) = carrier_with_pending(&r, "CARF");
    lose(&mut w, &r, first, Loss::Removal);
    lose(&mut w, &r, second, Loss::Removal);
    assert_eq!(controller(&w, car).pending, None);
    assert_eq!(controller(&w, car).state, ControllerState::Launching);
    spawn::pass(&mut w, car);
    let c = controller(&w, car);
    assert_eq!(c.state, ControllerState::Idle, "the reset");
    assert_eq!(c.slots[0].state, SlotState::Returning, "returning");
    assert!(!spawn::kamikaze(&w, x), "not released");
    assert!(w.obj(x).is_some_and(|o| o.health > 0));
    assert_eq!(w.obj(x).unwrap().dest, Some(Target::Object(car)), "flies back");
}

// spec: S-0506 Terms (r40, Missile type: the Slot table's preparing rows for a non-missile slot move its preparing object, re-sent, or returning when there is no controller target, flagged or not) and the Slot table (r40: preparing, non-missile slot, flagged or not); unmeasured for the flagged slot
#[test]
fn the_preparing_rows_move_a_flagged_non_rocket_object() {
    for name in ["CAR2", "CARF"] {
        let r = rules();
        let (mut w, car, x, first, _) = carrier_with_pending(&r, name);
        // With a controller target (the pending one adopted in the pass): re-sent.
        spawn::pass(&mut w, car);
        assert_eq!(w.obj(x).unwrap().dest, step_4(&w, car), "{name}: re-sent");
        assert_eq!(controller(&w, car).slots[0].state, SlotState::Preparing);
        // Without one: returning. (The controller target is cleared by hand: no
        // loss event, so no claim 11 effect.)
        let _ = first;
        w.obj_mut(car).unwrap().spawn.as_mut().unwrap().target = None;
        spawn::pass(&mut w, car);
        assert_eq!(
            controller(&w, car).slots[0].state,
            SlotState::Returning,
            "{name}: returning"
        );
        assert_eq!(w.obj(x).unwrap().dest, Some(Target::Object(car)), "{name}");
    }
}

// spec: S-0506 Terms (r40: while a flagged non-missile slot's object is still preparing, a reset, claim 11, releases it as it releases any flagged object) and claim 11 (r40: a preparing object without the missile-spawn flag is not affected by a reset); unmeasured for the flagged slot
#[test]
fn a_reset_releases_a_preparing_flagged_non_rocket_object_only() {
    for (name, flagged) in [("CAR2", false), ("CARF", true)] {
        let r = rules();
        let (mut w, car, x, _, _) = carrier_with_pending(&r, name);
        spawn::impossible(&mut w, car);
        let c = controller(&w, car);
        assert_eq!(c.state, ControllerState::Idle, "{name}: the reset");
        if flagged {
            assert_eq!(c.slots[0].state, SlotState::Empty, "{name}: released");
            assert!(spawn::kamikaze(&w, x), "{name}");
        } else {
            assert_eq!(c.slots[0].state, SlotState::Preparing, "{name}: unaffected");
            assert!(!spawn::kamikaze(&w, x), "{name}");
        }
    }
}

// spec: S-0506 claim 11 (r40: a controller that is cooling down at the loss keeps the pending target until a slot attacking or returning adopts it, claims 3 and 5; it gets no reset at the loss); unmeasured
#[test]
fn a_cooling_down_controller_keeps_the_pending_target_for_an_attacking_slot() {
    let r = rules();
    let mut w = testkit::game_with(&r, SIZE, 7);
    let car = put(&mut w, Kind::Vehicle, "CAR1", 0, 0, 0);
    let first = put(&mut w, Kind::Vehicle, "VICTIM", 1, 8, 0);
    let second = put(&mut w, Kind::Vehicle, "VICTIM", 1, 8, 4);
    let x = controller(&w, car).slots[0].object.unwrap();
    w.obj_mut(car).unwrap().target = Some(Target::Object(first));
    spawn::shot(&mut w, car, Target::Object(first));
    assert!(run_until(&mut w, 120, |w| controller(w, car).state
        == ControllerState::Launching));
    // One slot: the pass that launches it releases it (claim 8), and the controller cools down.
    spawn::pass(&mut w, car);
    let c = controller(&w, car);
    assert_eq!(c.slots[0].state, SlotState::Attacking, "set-up");
    assert_eq!(c.state, ControllerState::CoolingDown, "set-up");
    w.obj_mut(x).unwrap().combat.ammo = 3;
    spawn::set_own_target(w.obj_mut(car).unwrap(), Some(Target::Object(second)));
    spawn::shot(&mut w, car, Target::Object(second));
    lose(&mut w, &r, first, Loss::Removal);
    let c = controller(&w, car);
    assert_eq!(c.target, None, "cleared at the loss");
    assert_eq!(c.state, ControllerState::CoolingDown, "no reset");
    assert_eq!(c.pending, Some(Target::Object(second)), "kept");
    spawn::pass(&mut w, car);
    let c = controller(&w, car);
    assert_eq!(c.target, Some(Target::Object(second)), "adopted by the attacking slot");
    assert_eq!(c.slots[0].state, SlotState::Attacking);
    assert_eq!(w.obj(x).unwrap().target, Some(Target::Object(second)));
    assert_eq!(c.state, ControllerState::CoolingDown);
}
