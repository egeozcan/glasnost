//! Claim tests for S-0215 revision 67 (claim 11: the readiness predicate
//! of a vehicle with the driving locomotor, which answers Q-1160 and
//! Q-1161): it is moving for readiness while its body still turns, or
//! while its locomotor moves it with a heading point and a positive integer
//! movement step; a retained destination or stored path alone does not
//! hold it back. Synthetic data: the combat ruleset of `testkit` with a
//! turret-less cart and an unarmed tower.

use glasnost_num::F64;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::process;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::tick;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const SIZE: i32 = 16;
const ALPHA: HouseId = HouseId(0);

fn ruleset() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let s = &mut r.sim;
    let cart = s.vehicles.get_mut("CART").unwrap();
    cart.turret = Some(false);
    cart.rot = Some(1);
    s.buildings.get_mut("TOWER").unwrap().primary = None;
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

fn move_to(w: &mut World, id: ObjectId, dx: i32, dy: i32) {
    orders::issue(
        w,
        ALPHA,
        Order::Behaviour {
            actor: id,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(testkit::mid(SIZE, dx, dy))),
        },
    );
}

fn on_track(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .unwrap()
        .drive
        .as_ref()
        .is_some_and(|d| d.track.is_some())
}

/// Puts *attack* on `tower` in the queued slot behind *move*.
fn queue_attack(w: &mut World, id: ObjectId, tower: ObjectId) {
    let o = w.obj_mut(id).unwrap();
    o.beh.current = Some(Behaviour::Move);
    o.beh.queued = Some(Behaviour::Attack);
    o.target = Some(Target::Object(tower));
}

// spec: S-0215 claim 11 (revision 67, Q-1161: a retained destination and stored path steps, with no heading point and no body turn, do not make a driving vehicle moving; it is ready)
#[test]
fn claim_11_retained_goal_and_path_alone_are_ready() {
    let mut w = game();
    let cart = spawn(&mut w, Kind::Vehicle, "CART", 0, 0, 0);
    let tower = spawn(&mut w, Kind::Building, "TOWER", 1, 0, 4);
    queue_attack(&mut w, cart, tower);
    let o = w.obj_mut(cart).unwrap();
    o.dest = Some(Target::Cell(testkit::mid(SIZE, 4, 0)));
    o.path.steps = vec![2, 2, 2];
    assert!(
        glasnost_sim::occupancy::in_progress(&w, cart),
        "S-0206's term still counts the stored path"
    );
    assert!(!on_track(&w, cart));
    assert!(process::ready(&w, cart), "no heading point, no turn: ready");
}

// spec: S-0215 claim 11 (revision 67: a heading point with a positive integer movement step while the locomotor moves the vehicle is moving, so not ready; the same heading point with step 0 is not)
#[test]
fn claim_11_heading_point_and_positive_step() {
    let mut w = game();
    let cart = spawn(&mut w, Kind::Vehicle, "CART", 0, 0, 0);
    let tower = spawn(&mut w, Kind::Building, "TOWER", 1, -4, 0);
    // Facing 0 (map north): a move north needs no turn.
    move_to(&mut w, cart, 0, -5);
    let mut n = 0;
    // Until it is on a track, not turning, with a positive step (an
    // accelerating vehicle's first steps can truncate to 0, S-0207 F2).
    while !on_track(&w, cart)
        || w.obj(cart).unwrap().body.turning(w.tick)
        || w.obj(cart).unwrap().drive.as_ref().unwrap().step <= 0
    {
        tick::step(&mut w);
        n += 1;
        assert!(n < 100, "the vehicle sets off");
    }
    queue_attack(&mut w, cart, tower);
    assert!(!process::ready(&w, cart), "heading point, s > 0: not ready");
    w.obj_mut(cart).unwrap().drive.as_mut().unwrap().rho = F64::ZERO;
    assert!(on_track(&w, cart));
    assert!(process::ready(&w, cart), "heading point, s = 0: ready");
}

// spec: S-0215 claim 11 (revision 67, scenario B48's fresh move: an idle vehicle in area guard ordered to a cell five cells away starts move at its next first start and processes it in that tick, although destination assignment already gave a locomotor destination)
#[test]
fn claim_11_b48_fresh_move_from_area_guard() {
    let mut w = game();
    let cart = spawn(&mut w, Kind::Vehicle, "CART", 0, 0, 0);
    w.obj_mut(cart).unwrap().beh.current = Some(Behaviour::AreaGuard);
    w.obj_mut(cart).unwrap().beh.queued = None;
    move_to(&mut w, cart, 0, -5);
    // The order takes effect within this tick, after the vehicle's update:
    // its destination assignment gives the locomotor destination and
    // queues move.
    tick::step(&mut w);
    assert_eq!(w.obj(cart).unwrap().beh.queued, Some(Behaviour::Move));
    assert!(glasnost_sim::drive::goal_present(&w, cart), "a locomotor destination");
    assert!(process::ready(&w, cart), "ready despite the destination");
    let t = w.tick;
    tick::step(&mut w);
    let o = w.obj(cart).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Move));
    assert_eq!(o.beh.start_tick, t, "started at the first start");
    assert_eq!(o.beh.last.map(|l| l.0), Some(t), "processed in that tick");
}
