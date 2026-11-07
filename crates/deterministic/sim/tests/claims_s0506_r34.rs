//! Claim tests of S-0506 revision 34 (Q-2711 answered): a rocket missile
//! whose destination's in-the-air answer is false at its launch has its aim
//! point and its flight fixed at that launch (claims 7, 14 and 24, the
//! Rocket table's launch row), so a destination that leaves play after the
//! launch, by its removal or by going inside, changes neither; claim 24's
//! timer runs on the tick number, also when a removal before the missile in
//! the update order costs it a turn in its tilt (S-0203 claim 10, through
//! the tick pipeline); and the controller resets when its target
//! leaves play by going inside (claim 11 through claim 14). On synthetic
//! data: invented types, weapons and rocket parameter values, none of them
//! stock values.

use glasnost_num::F32;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    AnimationType, Decimal, LocomotionClass, Locomotor, UnitType, WeaponType,
};
use glasnost_sim::anims;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::rocket::{self, Phase};
use glasnost_sim::rules::Kind;
use glasnost_sim::spawn::{self, ControllerState, SlotState};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Pos, Target, World};

const SIZE: i32 = 40;

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

/// Rocket types RKA (set A) and RKB (set B), launchers LNA (one RKA slot)
/// and LNB (two RKB slots) with the spawner weapon Launch, the target
/// vehicle VICTIM; invented set parameters (set A: pause 4, tilt 10; set
/// B: pause 3, tilt 8).
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let push = |r: &mut Ruleset, reg: &str, n: &str| {
        r.registries.entry(reg.into()).or_default().push(n.into());
    };
    for n in ["RKA", "RKB"] {
        push(&mut r, "aircraft", n);
    }
    for n in ["LNA", "LNB", "VICTIM"] {
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
    let s = &mut r.sim;
    s.aircraft.insert("RKA".into(), rocket(50));
    s.aircraft.insert("RKB".into(), rocket(40));
    s.vehicles.insert("LNA".into(), launcher("RKA", 1));
    s.vehicles.insert("LNB".into(), launcher("RKB", 2));
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

/// Shoots `launcher` at `target` and runs until its slot 0 object is
/// placed on the map; returns that missile and the launch tick (the tick
/// just run: the tick number grows by 1 at the end of each tick).
fn launch(w: &mut World, launcher: ObjectId, target: ObjectId) -> (ObjectId, i32) {
    let x = controller(w, launcher).slots[0].object.unwrap();
    spawn::shot(w, launcher, Target::Object(target));
    for _ in 0..200 {
        testkit::run(w, 1);
        if w.obj(x).is_some_and(|o| o.on_map) {
            return (x, w.tick - 1);
        }
    }
    panic!("no launch");
}

/// Runs until `x` leaves play; the detonation point, if any, and whether
/// it ever had a positive speed.
fn fly_out(w: &mut World, x: ObjectId) -> (Option<Pos>, bool) {
    let mut moved = false;
    for _ in 0..2000 {
        testkit::run(w, 1);
        let Some(o) = w.obj(x) else {
            let at = w.effects.iter().find_map(|e| match e {
                Effect::Detonation { pos, .. } => Some(*pos),
                _ => None,
            });
            return (at, moved);
        };
        if let Some(rs) = o.rocket.as_ref() {
            moved |= rs.speed.gt(glasnost_num::F64::ZERO);
        }
    }
    panic!("still on the map 2000 ticks after the launch");
}

fn near(a: Pos, b: Pos) -> bool {
    (a.x - b.x).abs() < 256 && (a.y - b.y).abs() < 256
}

// spec: S-0506 claim 14 (r34: a destination whose in-the-air answer is false at the launch: the aim point, its heading point, is fixed at the launch, in the launch pass of its placement tick) and the Rocket table's launch row (pause, pitch Pᵢ; claim 24's t₀ the launch tick)
#[test]
fn the_aim_and_the_flight_are_fixed_at_the_launch() {
    let r = rules();
    let mut w = testkit::game_with(&r, SIZE, 7);
    let lna = put(&mut w, Kind::Vehicle, "LNA", 0, 0, 0);
    let victim = put(&mut w, Kind::Vehicle, "VICTIM", 1, 8, 0);
    let aim = w.obj(victim).unwrap().pos;
    let (x, t0) = launch(&mut w, lna, victim);
    let rs = state(&w, x);
    assert_eq!(rs.aim, Some(aim), "fixed at the launch");
    assert_eq!(rs.phase, Some(Phase::Pause));
    assert_eq!(rs.started, t0, "claim 24's timer starts in the launch tick");
    // spec: S-0506 claim 7 (r34: the controller target as destination, its in-the-air answer false)
    assert_eq!(w.obj(x).unwrap().dest, Some(Target::Object(victim)));
}

// spec: S-0506 claim 14 (r34, A destination that leaves play after the launch: removed in the very tick of the launch, the missile flies on through claim 15's phases to the aim point fixed at the launch and detonates by claim 19; no missile stays on the map without a flight); scenario R25 and play note round 9 item 2 (Q-2711)
#[test]
fn a_destination_removed_in_the_launch_tick_does_not_stop_the_flight() {
    let r = rules();
    let mut w = testkit::game_with(&r, SIZE, 7);
    let lna = put(&mut w, Kind::Vehicle, "LNA", 0, 0, 0);
    let victim = put(&mut w, Kind::Vehicle, "VICTIM", 1, 8, 0);
    let aim = w.obj(victim).unwrap().pos;
    // The world at the start of the launch tick: the launch pass alone is
    // run, so the missile has made no update of its own when its
    // destination is removed (as when the removal costs it its first turn,
    // S-0203 claims 4 and 10).
    let x = controller(&w, lna).slots[0].object.unwrap();
    spawn::shot(&mut w, lna, Target::Object(victim));
    let mut at_start = None;
    for _ in 0..200 {
        let before = w.clone();
        testkit::run(&mut w, 1);
        if w.obj(x).is_some_and(|o| o.on_map) {
            at_start = Some(before);
            break;
        }
    }
    let mut w = at_start.expect("a launch");
    let t0 = w.tick;
    spawn::turn(&mut w, lna);
    assert!(w.obj(x).is_some_and(|o| o.on_map), "launched by the pass");
    let rs = state(&w, x);
    assert_eq!(
        rs.phase,
        Some(Phase::Pause),
        "the flight begins at the launch"
    );
    assert_eq!(rs.started, t0);
    assert_eq!(rs.aim, Some(aim));
    glasnost_sim::objects::remove(&mut w, victim);
    assert_eq!(state(&w, x).aim, Some(aim), "the leaving changes no aim");
    let (at, moved) = fly_out(&mut w, x);
    assert!(moved, "it flies");
    let at = at.expect("it detonates");
    assert!(
        near(at, aim),
        "at the aim point fixed at the launch: {at:?} vs {aim:?}"
    );
}

// spec: S-0506 claim 14 (r34: a destination that goes inside after the launch: the missile flies on to the aim point fixed at the launch; the controller target leaving play with no pending target resets the controller when it goes inside, claim 11, releasing the preparing missile, which belongs to no slot from then on); scenario R25 part a
#[test]
fn a_destination_that_goes_inside_resets_the_controller_and_the_missile_flies_on() {
    let r = rules();
    let mut w = testkit::game_with(&r, SIZE, 7);
    let lnb = put(&mut w, Kind::Vehicle, "LNB", 0, 0, 0);
    let walker = put(&mut w, Kind::Infantry, "WALKER", 1, 8, 0);
    let (x, _) = launch(&mut w, lnb, walker);
    let aim = state(&w, x).aim.expect("fixed at the launch");
    assert_eq!(controller(&w, lnb).slots[0].state, SlotState::Preparing);
    glasnost_sim::objects::take_inside(&mut w, walker);
    let c = controller(&w, lnb);
    assert_eq!(
        c.state,
        ControllerState::Idle,
        "the reset at the going inside"
    );
    assert_eq!(c.target, None);
    assert_eq!(c.slots[0].state, SlotState::Empty, "released from its slot");
    let (at, moved) = fly_out(&mut w, x);
    assert!(moved);
    assert!(near(at.expect("it detonates"), aim));
}

// spec: S-0506 claim 14 (r34: claim 11 gives no reset when the leaving object is not then the controller target: here a pending target waits, and the controller goes on with it, claim 25)
#[test]
fn no_reset_when_a_pending_target_waits() {
    let r = rules();
    let mut w = testkit::game_with(&r, SIZE, 7);
    let lnb = put(&mut w, Kind::Vehicle, "LNB", 0, 0, 0);
    let walker = put(&mut w, Kind::Infantry, "WALKER", 1, 8, 0);
    let other = put(&mut w, Kind::Vehicle, "VICTIM", 1, 8, 4);
    let (_, _) = launch(&mut w, lnb, walker);
    // A later shot at another target: the spawner's own target is that
    // target and the shot waits as pending (claim 3).
    spawn::set_own_target(w.obj_mut(lnb).unwrap(), Some(Target::Object(other)));
    spawn::shot(&mut w, lnb, Target::Object(other));
    assert_eq!(controller(&w, lnb).pending, Some(Target::Object(other)));
    assert_eq!(controller(&w, lnb).target, Some(Target::Object(walker)));
    glasnost_sim::objects::take_inside(&mut w, walker);
    let c = controller(&w, lnb);
    assert_ne!(c.state, ControllerState::Idle, "no reset");
    assert_eq!(c.slots[0].state, SlotState::Preparing);
}

// spec: S-0506 claim 14 (r34: a leaving by going inside a building or a transport is itself a removal) and claim 25 (the loss notification for the pending target clears it at once, whatever the own target); claim 11 (the leaving object is not the controller target, so no reset: the controller goes on with its target) (seam sweep 6: intake 110's row, a pending target that goes inside)
#[test]
fn a_pending_target_that_goes_inside_is_cleared_with_no_reset() {
    for inside in [true, false] {
        let r = rules();
        let mut w = testkit::game_with(&r, SIZE, 7);
        let lnb = put(&mut w, Kind::Vehicle, "LNB", 0, 0, 0);
        let victim = put(&mut w, Kind::Vehicle, "VICTIM", 1, 8, 0);
        let walker = put(&mut w, Kind::Infantry, "WALKER", 1, 8, 4);
        let (_, _) = launch(&mut w, lnb, victim);
        spawn::set_own_target(w.obj_mut(lnb).unwrap(), Some(Target::Object(walker)));
        spawn::shot(&mut w, lnb, Target::Object(walker));
        assert_eq!(
            controller(&w, lnb).pending,
            Some(Target::Object(walker)),
            "set-up: pending"
        );
        if inside {
            glasnost_sim::objects::take_inside(&mut w, walker);
            assert!(w.obj(walker).is_some(), "it stays in existence");
        } else {
            // Control: the removal that going inside is read as.
            glasnost_sim::objects::remove(&mut w, walker);
        }
        let c = controller(&w, lnb);
        assert_eq!(c.pending, None, "inside {inside}: pending cleared at once");
        assert_eq!(
            c.target,
            Some(Target::Object(victim)),
            "inside {inside}: the controller target kept"
        );
        assert_ne!(c.state, ControllerState::Idle, "inside {inside}: no reset");
        assert_eq!(c.slots[0].state, SlotState::Preparing, "inside {inside}");
    }
}

// spec: S-0506 claim 14 (r34: claim 24's timer runs on the tick number, S-0203 claim 9, so a turn the missile does not make within a phase does not shift its ticks: its next update finds the timer run on; a phase's end is acted on at the first update at or after its timer runs out, and the next phase's timer starts at that update)
#[test]
fn the_flight_timer_runs_on_the_tick_number() {
    let r = rules();
    let mut w = testkit::game_with(&r, SIZE, 7);
    let lna = put(&mut w, Kind::Vehicle, "LNA", 0, 0, 0);
    let victim = put(&mut w, Kind::Vehicle, "VICTIM", 1, 8, 0);
    let (x, t0) = launch(&mut w, lna, victim);
    // As if the missile made no update for 6 ticks: the pause of 4 has run out.
    w.tick = t0 + 6;
    rocket::tick(&mut w, x);
    let rs = state(&w, x);
    assert_eq!(
        rs.phase,
        Some(Phase::Tilt),
        "acted on at the first update after"
    );
    assert_eq!(rs.started, t0 + 6, "the tilt's timer starts at that update");
}

// spec: S-0506 claim 24 (r34: with uninterrupted updates the boundaries are those of t₀ the launch tick: pause over at t₀ + P, tilt over T ticks later)
#[test]
fn uninterrupted_boundaries_from_the_launch_tick() {
    let r = rules();
    let mut w = testkit::game_with(&r, SIZE, 7);
    let lna = put(&mut w, Kind::Vehicle, "LNA", 0, 0, 0);
    let victim = put(&mut w, Kind::Vehicle, "VICTIM", 1, 8, 0);
    let (x, t0) = launch(&mut w, lna, victim);
    let mut tilt = None;
    let mut ascent = None;
    for _ in 0..40 {
        testkit::run(&mut w, 1);
        match w
            .obj(x)
            .and_then(|o| o.rocket.as_ref())
            .and_then(|r| r.phase)
        {
            Some(Phase::Tilt) if tilt.is_none() => tilt = Some(w.tick - 1),
            Some(Phase::Ascent) if ascent.is_none() => ascent = Some(w.tick - 1),
            _ => {}
        }
    }
    // spec: S-0506 claim 24 (P = 4: the pause ends at the update of t₀ + 4; T = 10: the tilt ends 10 ticks after it began)
    assert_eq!(tilt, Some(t0 + 4));
    assert_eq!(ascent, Some(t0 + 4 + 10));
}

// design: Q-2748 interim (a missile whose destination's in-the-air answer was true at the launch has no flight fixed then: its aim point is fixed at its first update from its destination's heading point, and claim 24's timer starts at that update, as in r20; with no destination by then it has no flight)
#[test]
fn q2748_a_flight_not_fixed_at_the_launch_starts_at_the_first_update() {
    let r = rules();
    let mut w = testkit::game_with(&r, SIZE, 7);
    let lna = put(&mut w, Kind::Vehicle, "LNA", 0, 0, 0);
    let victim = put(&mut w, Kind::Vehicle, "VICTIM", 1, 8, 0);
    let (x, _) = launch(&mut w, lna, victim);
    // As if not fixed at the launch.
    let o = w.obj_mut(x).unwrap();
    let rs = o.rocket.as_mut().unwrap();
    rs.phase = None;
    rs.aim = None;
    let mut gone = w.clone();
    let t = w.tick;
    rocket::tick(&mut w, x);
    let rs = state(&w, x);
    assert_eq!(rs.aim, Some(w.obj(victim).unwrap().pos));
    assert_eq!(rs.phase, Some(Phase::Pause));
    assert_eq!(rs.started, t);
    gone.obj_mut(x).unwrap().dest = None;
    assert!(!rocket::tick(&mut gone, x));
    assert_eq!(
        state(&gone, x).phase,
        None,
        "no flight without a destination"
    );
}

/// [`rules`] with two ordinary animations of one pass and no follow-up
/// type, AN10 and AN17: 10 and 17 frames at rate 900 (one tick a frame), so
/// each leaves play in its own update 10 or 17 ticks after the tick it was
/// made before (S-0109 claims 5 and 16). Invented types.
fn rules_with_anims() -> Ruleset {
    let mut r = rules();
    for (name, frames) in [("AN10", 10), ("AN17", 17)] {
        r.registries
            .entry("animations".into())
            .or_default()
            .push(name.into());
        r.sim.animations.insert(
            name.into(),
            AnimationType {
                frame_count: Some(frames),
                rate: Some(900),
                ..Default::default()
            },
        );
    }
    r
}

/// The missile as one tick's turn pass leaves it.
#[derive(Clone, Debug, PartialEq)]
struct Seen {
    /// It took its turn in the tick.
    turn: bool,
    phase: Option<Phase>,
    pitch: F32,
}

/// What [`flight_with`] saw.
struct Flight {
    /// The launch tick t₀.
    t0: i32,
    /// The tick the animation left play, and whether it then stood directly
    /// before the missile in the update order.
    gone: Option<(i32, bool)>,
    /// From t₀ on, per tick.
    seen: Vec<Seen>,
}

/// LNA's missile shot at VICTIM, through the tick pipeline alone. With
/// `anim`, that animation is made between ticks three ticks before the
/// launch tick, so it enters play before the missile and stands directly
/// before it once the launch appends the missile (S-0203 claim 3).
fn flight_with(r: &Ruleset, anim: Option<&str>) -> Flight {
    let mut w = testkit::game_with(r, SIZE, 7);
    let lna = put(&mut w, Kind::Vehicle, "LNA", 0, 0, 0);
    let victim = put(&mut w, Kind::Vehicle, "VICTIM", 1, 8, 0);
    let x = controller(&w, lna).slots[0].object.unwrap();
    spawn::shot(&mut w, lna, Target::Object(victim));
    let placed = |w: &World| w.obj(x).is_some_and(|o| o.on_map);
    let mut probe = w.clone();
    while !placed(&probe) {
        assert!(probe.tick < 200, "no launch");
        testkit::run(&mut probe, 1);
    }
    let t0 = probe.tick - 1;
    assert!(t0 - 3 >= w.tick, "room to make the animation first");
    while w.tick < t0 - 3 {
        testkit::run(&mut w, 1);
    }
    let a = anim.map(|n| {
        let at = w.obj(victim).unwrap().pos;
        anims::spawn(&mut w, n, Some(at), anims::Spawn::default())
            .id
            .expect("a listed type")
    });
    let mut gone = None;
    let mut seen = Vec::new();
    while w.tick <= t0 + 20 {
        let ran = w.tick;
        let before = |id| w.update.order.iter().position(|o| *o == id);
        let adjacent =
            a.is_some_and(|a| before(x).is_some_and(|p| p > 0 && before(a) == Some(p - 1)));
        let was_placed = placed(&w);
        testkit::run(&mut w, 1);
        if !was_placed && placed(&w) {
            assert_eq!(ran, t0, "the animation does not move the launch");
        }
        if let Some(a) = a
            && gone.is_none()
            && !w.update.order.contains(&a)
        {
            gone = Some((ran, adjacent));
        }
        if ran >= t0 {
            let o = w.obj(x).expect("in flight");
            let rs = o.rocket.as_ref().unwrap();
            seen.push(Seen {
                turn: o.last_turn == ran,
                phase: rs.phase,
                pitch: rs.pitch,
            });
        }
    }
    Flight { t0, gone, seen }
}

/// The first index of `seen` (ticks after t₀) in phase `p`.
fn first(seen: &[Seen], p: Phase) -> Option<usize> {
    seen.iter().position(|s| s.phase == Some(p))
}

// spec: S-0203 claim 10 (a counting removal costs the object directly after the current one its turn; cause: an animation that ends, removed in its own turn, S-0109 claim 16) and claim 9 (the missile's delays run on); S-0506 claim 14 (r34: a lost turn within a phase does not shift its ticks; the missile makes no update in that turn; in tilt, whose pitch F3 computes from the elapsed ticks, the next update's pitch is the same as without the loss) and claim 24 (pause 4, tilt 10: tilt from t₀ + 4, ascent from t₀ + 14) (the missed turns of seam sweep 6's seed 7 map 0 and seed 10 map 8 games: an ordinary animation directly before the missile ends its last pass in its tilt)
#[test]
fn an_animation_ending_before_the_missile_costs_it_a_tilt_turn_and_shifts_nothing() {
    let r = rules_with_anims();
    let twin = flight_with(&r, None);
    let f = flight_with(&r, Some("AN10"));
    assert_eq!(f.t0, twin.t0);
    let t0 = f.t0;
    assert_eq!(
        f.gone,
        Some((t0 + 7, true)),
        "the animation leaves play in its own update of t₀ + 7, directly before the missile"
    );
    assert_eq!(f.seen[7].phase, Some(Phase::Tilt), "in the tilt");
    for (k, s) in f.seen.iter().enumerate() {
        assert_eq!(s.turn, k != 7, "turn at t₀ + {k}");
        assert!(twin.seen[k].turn, "twin turn at t₀ + {k}");
    }
    // The lost turn makes no update: the pitch stays at the one before.
    assert_eq!(f.seen[7].pitch, twin.seen[6].pitch);
    assert_ne!(twin.seen[7].pitch, twin.seen[6].pitch, "the tilt turns");
    // From the next update on, the flight is the twin's, tick for tick.
    assert_eq!(f.seen[8..], twin.seen[8..]);
    assert_eq!(f.seen[..7], twin.seen[..7]);
    assert_eq!(first(&f.seen, Phase::Tilt), Some(4));
    assert_eq!(first(&f.seen, Phase::Ascent), Some(14));
    assert_eq!(first(&twin.seen, Phase::Ascent), Some(14));
}

// spec: S-0506 claim 14 (r34: a phase's end is acted on at the missile's first update at or after its timer runs out, and the next phase's timer starts at that update, so where the lost turn is the one in which a phase's timer runs out, the later phases come one tick later) and S-0203 claim 10 (the same cause, an animation that ends directly before the missile, in the tilt's last tick)
#[test]
fn a_tilt_turn_lost_at_its_end_starts_the_ascent_a_tick_later() {
    let r = rules_with_anims();
    let twin = flight_with(&r, None);
    let f = flight_with(&r, Some("AN17"));
    let t0 = f.t0;
    assert_eq!(
        f.gone,
        Some((t0 + 14, true)),
        "gone in the tilt's last tick"
    );
    assert!(!f.seen[14].turn);
    assert_eq!(f.seen[14].phase, Some(Phase::Tilt), "no update: still tilt");
    assert_eq!(first(&twin.seen, Phase::Ascent), Some(14));
    assert_eq!(first(&f.seen, Phase::Ascent), Some(15), "a tick later");
}

// spec: S-0203 claim 10 and Edge cases (`quirks.update_order_removal_skip` off: no removal costs a turn, so the missile keeps its tilt turn)
#[test]
fn with_the_removal_skip_off_the_missile_keeps_its_tilt_turn() {
    let mut r = rules_with_anims();
    r.globals.quirks.update_order_removal_skip = false;
    let twin = flight_with(&r, None);
    let f = flight_with(&r, Some("AN10"));
    assert_eq!(f.gone.map(|g| g.0), Some(f.t0 + 7));
    assert!(f.seen.iter().all(|s| s.turn));
    assert_eq!(f.seen, twin.seen);
}
