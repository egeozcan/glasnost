//! Claim tests for S-0215 revision 63 (claim 11: a vehicle with the driving
//! locomotor whose body is still turning, for any reason, is moving for
//! readiness, so it is not ready to change), on synthetic data: the
//! combat ruleset of `testkit` with a turret-less, slow-turning cart, a
//! ship and a hovering vehicle of our own.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{LocomotionClass, Locomotor, UnitType};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::process;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::tick;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const SIZE: i32 = 16;
const ALPHA: HouseId = HouseId(0);

/// The combat ruleset with a turret-less cart that turns slowly (rot 1),
/// an unarmed tower, a ship `BOAT` and a hovering vehicle `SKIM` (both rot 1).
fn ruleset() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let s = &mut r.sim;
    let cart = s.vehicles.get_mut("CART").unwrap();
    cart.turret = Some(false);
    cart.rot = Some(1);
    s.buildings.get_mut("TOWER").unwrap().primary = None;
    for (name, class, loco) in [
        ("BOAT", LocomotionClass::Float, Locomotor::Ship),
        ("SKIM", LocomotionClass::Hover, Locomotor::Hover),
    ] {
        s.vehicles.insert(
            name.into(),
            UnitType {
                strength: Some(300),
                speed: Some(5),
                rot: Some(1),
                locomotion_class: Some(class),
                locomotor: Some(loco),
                ..Default::default()
            },
        );
        r.registries
            .get_mut("vehicles")
            .unwrap()
            .push(name.into());
    }
    r
}

fn game() -> World {
    let mut w = testkit::game_with(&ruleset(), SIZE, 1);
    testkit::run(&mut w, 4);
    w
}

fn spawn(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, testkit::mid(SIZE, dx, dy), 0)
}

/// Gives `id` a turn of its body to `facing` from this tick, at its type's rate.
fn start_turn(w: &mut World, id: ObjectId, facing: u16) {
    let t = w.tick;
    let o = w.obj(id).unwrap();
    let rate = w.rules().unit(o.kind, o.type_index).unwrap().turn_rate;
    assert!(w.obj_mut(id).unwrap().body.turn_to(facing, t, rate));
    assert!(w.obj(id).unwrap().body.turning(t));
}

/// Sets the slots and target of `id`.
fn slots(
    w: &mut World,
    id: ObjectId,
    current: Behaviour,
    queued: Behaviour,
    target: Option<Target>,
) {
    let o = w.obj_mut(id).unwrap();
    o.beh.current = Some(current);
    o.beh.queued = Some(queued);
    o.target = target;
}

// spec: S-0215 claim 11 (revision 63: a driving vehicle whose body is still turning to face a target, standing still with no destination, no track and no turn in place, is not ready; ready once the body is at rest)
#[test]
fn claim_11_turning_to_face_a_target_is_not_ready() {
    let mut w = game();
    let cart = spawn(&mut w, Kind::Vehicle, "CART", 0, 0, 0);
    let tower = spawn(&mut w, Kind::Building, "TOWER", 1, 0, 3);
    slots(
        &mut w,
        cart,
        Behaviour::Attack,
        Behaviour::Guard,
        Some(Target::Object(tower)),
    );
    assert!(process::ready(&w, cart), "at rest: ready");
    start_turn(&mut w, cart, 32768);
    let o = w.obj(cart).unwrap();
    assert!(o.dest.is_none() && o.path.steps.is_empty());
    assert!(!o.drive.as_ref().unwrap().turning_in_place);
    assert!(
        !glasnost_sim::occupancy::in_progress(&w, cart),
        "S-0206's locomotor moving term is not widened"
    );
    assert!(!process::ready(&w, cart), "turning: not ready");
    let target = w.obj(cart).unwrap().body.target;
    w.obj_mut(cart).unwrap().body = glasnost_sim::turning::Turning::at(target);
    assert!(process::ready(&w, cart), "turn ended: ready");
}

// spec: S-0215 claim 11 (revision 63: the exceptions apply to a turning vehicle as to a travelling one: guard current, attack without a target, enter queued, a guard queued with the allowance)
#[test]
fn claim_11_turning_exceptions() {
    let mut w = game();
    let cart = spawn(&mut w, Kind::Vehicle, "CART", 0, 0, 0);
    let tower = spawn(&mut w, Kind::Building, "TOWER", 1, 0, 3);
    start_turn(&mut w, cart, 32768);
    slots(&mut w, cart, Behaviour::Guard, Behaviour::Move, None);
    assert!(process::ready(&w, cart), "guard current");
    slots(&mut w, cart, Behaviour::Attack, Behaviour::Move, None);
    assert!(process::ready(&w, cart), "attack without a target");
    slots(
        &mut w,
        cart,
        Behaviour::Attack,
        Behaviour::Enter,
        Some(Target::Object(tower)),
    );
    assert!(process::ready(&w, cart), "enter queued");
    slots(
        &mut w,
        cart,
        Behaviour::Attack,
        Behaviour::Guard,
        Some(Target::Object(tower)),
    );
    w.obj_mut(cart).unwrap().beh.allowance = true;
    assert!(process::ready(&w, cart), "guard queued with the allowance");
    w.obj_mut(cart).unwrap().beh.allowance = false;
    assert!(!process::ready(&w, cart), "no exception");
}

// spec: S-0215 claim 11 (revision 63: the turning clause is for the driving locomotor only; a turning ship or hovering vehicle standing still stays ready)
#[test]
fn claim_11_turning_clause_is_the_driving_locomotors() {
    let mut w = game();
    let tower = spawn(&mut w, Kind::Building, "TOWER", 1, 0, 3);
    for (name, dx) in [("BOAT", -3), ("SKIM", 3)] {
        let v = spawn(&mut w, Kind::Vehicle, name, 0, dx, 0);
        slots(
            &mut w,
            v,
            Behaviour::Attack,
            Behaviour::Guard,
            Some(Target::Object(tower)),
        );
        start_turn(&mut w, v, 32768);
        assert!(
            !glasnost_sim::occupancy::in_progress(&w, v),
            "{name} stands still"
        );
        assert!(process::ready(&w, v), "{name}: turning but ready");
    }
}

/// A read after a tick: that tick's number, the body facing at that tick
/// (S-0206 F1) and the current behaviour. The world's tick number has
/// already grown by 1 (S-0203 claim 1).
fn read(w: &World, id: ObjectId) -> (i32, u16, Option<Behaviour>) {
    let k = w.tick - 1;
    let o = w.obj(id).unwrap();
    (k, o.body.value(k), o.beh.current)
}

// spec: S-0215 claim 11 (revision 67, scenario B47: a vehicle given a move that needs a half turn, then an attack order on a target along the turn's direction on the first tick of the turn, keeps attack queued while its body facing changes; with t the tick on which the facing reaches its target, the turn is over at that tick's first start, so attack becomes current there, its start tick is t and it is processed in tick t with a positive delay) and S-0207 claim 11 (revision 39, scenario D11 c: the cancelled step is not taken, no destination is left)
#[test]
fn claim_11_b47_turning_vehicle_keeps_a_queued_attack() {
    let mut w = game();
    let cart = spawn(&mut w, Kind::Vehicle, "CART", 0, 0, 0);
    // The target lies farther along the turn's direction (B47), so aiming at
    // it does not give the body a different target facing.
    let tower = spawn(&mut w, Kind::Building, "TOWER", 1, 0, 3);
    // Facing 0 (map north); the free cell to the south needs a half turn.
    orders::issue(
        &mut w,
        ALPHA,
        Order::Behaviour {
            actor: cart,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(testkit::mid(SIZE, 0, 1))),
        },
    );
    let last = read(&w, cart).1;
    let mut guard = 0;
    while read(&w, cart).1 == last {
        tick::step(&mut w);
        guard += 1;
        assert!(guard < 20, "the turn begins");
    }
    orders::issue(
        &mut w,
        ALPHA,
        Order::Behaviour {
            actor: cart,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(tower)),
            dest: None,
        },
    );
    // The run's validity (B47): after the order's tick, attack queued, move
    // current, the facing still changing, no destination and no route.
    let before = read(&w, cart).1;
    tick::step(&mut w);
    let o = w.obj(cart).unwrap();
    assert_eq!(
        (o.beh.current, o.beh.queued),
        (Some(Behaviour::Move), Some(Behaviour::Attack))
    );
    assert_ne!(read(&w, cart).1, before, "still turning");
    assert!(o.dest.is_none() && o.path.steps.is_empty());
    assert!(o.drive.as_ref().unwrap().track.is_none());
    assert!(!glasnost_sim::drive::goal_present(&w, cart));
    // After every tick: the facing and the slots.
    let mut log = vec![read(&w, cart)];
    // For each tick: whether attack was current and processed in it, and
    // the delay its first processing chose.
    let mut starts = std::collections::BTreeMap::new();
    let mut start_delay = None;
    for _ in 0..300 {
        tick::step(&mut w);
        let k = w.tick - 1;
        let o = w.obj(cart).unwrap();
        assert!(o.dest.is_none() && o.path.steps.is_empty(), "no route left");
        let processed = o.beh.current == Some(Behaviour::Attack)
            && o.beh.last.is_some_and(|l| l.0 == k)
            && !o.beh.expired;
        if processed && start_delay.is_none() {
            start_delay = o.beh.last.map(|l| l.1);
        }
        starts.insert(k, processed);
        log.push(read(&w, cart));
    }
    // t: the tick on which the body facing last changed.
    let t = log
        .windows(2)
        .filter(|p| p[0].1 != p[1].1)
        .map(|p| p[1].0)
        .last()
        .expect("the facing changes");
    assert!(t < log.last().unwrap().0 - 10, "the turn ends in the run");
    for &(k, _, cur) in &log {
        if k < t {
            assert_eq!(cur, Some(Behaviour::Move), "tick {k}: attack still queued");
        } else {
            assert_eq!(cur, Some(Behaviour::Attack), "tick {k}: attack current");
        }
    }
    // Same-tick processing (revision 67): started at tick t's first start.
    let o = w.obj(cart).unwrap();
    assert_eq!(o.beh.start_tick, t, "attack started in tick t");
    assert_eq!(starts.get(&t).copied(), Some(true), "processed in tick t");
    assert!(start_delay.is_some_and(|d| d > 0), "a positive chosen delay");
    assert_eq!(read(&w, cart).2, Some(Behaviour::Attack));
}
