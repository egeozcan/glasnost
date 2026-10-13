//! End-to-end checks of attack orders on synthetic data: a unit ordered
//! to attack an enemy building in range damages it, and one ordered to
//! attack a far target approaches it.

use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit::{self, mid, spawn};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const SIZE: i32 = 40;

fn attack(w: &mut World, actor: ObjectId, target: ObjectId) {
    // Orders are refused while the game-start order block lasts.
    w.tick = w.tick.max(w.orders_blocked_through + 1);
    let issued = orders::issue(
        w,
        HouseId(0),
        Order::Behaviour {
            actor,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(target)),
            dest: None,
        },
    );
    assert!(issued);
}

fn world_with(size: i32, foundation: [i32; 2]) -> World {
    let mut r = testkit::combat_ruleset();
    let tower = r.sim.buildings.get_mut("TOWER").unwrap();
    // The tower does not shoot back, so the attacker survives.
    tower.primary = None;
    tower.foundation = Some(foundation);
    testkit::game_with(&r, size, 5)
}

fn world() -> World {
    world_with(SIZE, [1, 1])
}

// spec: S-0215 claim 37 and S-0307 claim 15 (an attacker with its target in range fires; out of range it approaches)
#[test]
fn attack_on_a_building_in_range_damages_it() {
    let mut w = world();
    let tower = spawn(&mut w, Kind::Building, "TOWER", 1, mid(SIZE, 0, 0), 0);
    let cart = spawn(&mut w, Kind::Vehicle, "CART", 0, mid(SIZE, -2, 0), 64);
    let before = w.obj(tower).unwrap().health;
    attack(&mut w, cart, tower);
    testkit::run(&mut w, 300);
    assert!(
        w.obj(tower).is_none_or(|o| o.health < before),
        "the tower took no damage"
    );
}

// spec: S-0215 claim 37 and S-0307 claim 15 (an attacker with its target out of range approaches it)
#[test]
fn attack_on_a_far_target_approaches() {
    let mut w = world();
    let foe = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 14, 0), 64);
    let cart = spawn(&mut w, Kind::Vehicle, "CART", 0, mid(SIZE, -14, 0), 64);
    let x0 = w.obj(cart).unwrap().pos.x;
    attack(&mut w, cart, foe);
    testkit::run(&mut w, 300);
    let o = w.obj(cart).unwrap();
    assert!(
        o.pos.x > x0 + 256 * 4,
        "no approach: {} from {}",
        o.pos.x,
        x0
    );
}

// spec: S-0215 claim 37 and S-0307 claim 15 (a large building next to the attacker: in range of its nearest part, it is damaged)
#[test]
fn attack_on_a_large_building_next_to_the_attacker_damages_it() {
    let mut w = world_with(SIZE, [4, 4]);
    let tower = spawn(&mut w, Kind::Building, "TOWER", 1, mid(SIZE, 0, 0), 0);
    let cart = spawn(&mut w, Kind::Vehicle, "CART", 0, mid(SIZE, -1, 1), 64);
    let before = w.obj(tower).unwrap().health;
    attack(&mut w, cart, tower);
    testkit::run(&mut w, 300);
    assert!(
        w.obj(tower).is_none_or(|o| o.health < before),
        "the tower took no damage"
    );
}

// spec: S-0215 claim 37 and S-0307 claim 15 (a target 80 cells away: the attacker approaches)
#[test]
fn attack_on_a_very_far_target_approaches() {
    let size = 110;
    let mut w = world_with(size, [1, 1]);
    let foe = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(size, 40, 0), 64);
    let cart = spawn(&mut w, Kind::Vehicle, "CART", 0, mid(size, -40, 0), 64);
    let x0 = w.obj(cart).unwrap().pos.x;
    attack(&mut w, cart, foe);
    testkit::run(&mut w, 300);
    let o = w.obj(cart).unwrap();
    assert!(
        o.pos.x > x0 + 256 * 4,
        "no approach: {} from {}",
        o.pos.x,
        x0
    );
}
