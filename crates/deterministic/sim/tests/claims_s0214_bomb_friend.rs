//! S-0214 claim 6, *may attack as a friend* (#211): the
//! attack-cursor-on-friendlies flag does not count when the warhead of the
//! weapon the actor would use plants bombs and the target's type can carry
//! passengers. Synthetic data: the fixture of `tests/terror_common`
//! (BOMBER plants with `PlantW`, whose warhead IVAN plants bombs; VICT is a
//! vehicle without passengers) and invented types FBOMB, FTANK and VAN.

mod terror_common;

use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::world::{HouseId, ObjectId, World};
use terror_common::*;

fn act(w: &World, a: ObjectId, t: ObjectId) -> Action {
    targeting::action(w, HouseId(0), a, ClickOn::Object(t), Modifiers::default())
}

fn world() -> World {
    world_with(|_| {})
}

fn world_with(f: impl FnOnce(&mut glasnost_rules::schema::Ruleset)) -> World {
    terror_common::world(|r| {
        infantry(r, "FBOMB", |t| {
            t.primary = Some("PlantW".into());
            t.ivan = Some(true);
            t.attack_cursor_on_friendlies = Some(true);
        });
        vehicle(r, "FTANK", |t| {
            t.primary = Some("PlantW".into());
            t.attack_cursor_on_friendlies = Some(true);
        });
        vehicle(r, "VAN", |t| {
            t.passengers = Some(5);
            t.size_limit = Some(d("2"));
        });
        f(r);
    })
}

// spec: S-0214 claim 6 (may attack as a friend: the attack-cursor-on-friendlies flag, unless the warhead of the weapon it would use plants bombs and the target's type can carry passengers); rows G10 and x3 (the own transport is entered)
#[test]
fn the_cursor_flag_does_not_count_for_a_bomb_planter_over_a_transport() {
    let mut w = world();
    let van = quiet(&mut w, Kind::Vehicle, "VAN", 0, at(0, 0));
    let tank = quiet(&mut w, Kind::Vehicle, "VICT", 0, at(0, 3));
    let flagged = quiet(&mut w, Kind::Infantry, "FBOMB", 0, at(-3, 0));
    let plain = quiet(&mut w, Kind::Infantry, "BOMBER", 0, at(-3, 2));
    // Without the flag the own transport is entered.
    assert_eq!(act(&w, plain, van), Action::Enter);
    // The flag over an own vehicle without passengers: the friend attack
    // (row I19: bomb).
    assert_eq!(act(&w, flagged, tank), Action::Bomb);
    // Over a type that can carry passengers the flag does not count.
    assert_eq!(act(&w, flagged, van), Action::Enter);
}

// spec: S-0214 claim 6 (may attack as a friend) and scenario T10 (plants-bombs exception: a tank actor attacks an allied tank without passengers, not an allied transport)
#[test]
fn the_exception_for_a_vehicle_actor() {
    let mut w = world();
    let van = quiet(&mut w, Kind::Vehicle, "VAN", 0, at(0, 0));
    let tank = quiet(&mut w, Kind::Vehicle, "VICT", 0, at(0, 3));
    let flagged = quiet(&mut w, Kind::Vehicle, "FTANK", 0, at(-3, 0));
    assert_eq!(act(&w, flagged, tank), Action::Attack);
    assert_ne!(act(&w, flagged, van), Action::Attack);
}

// spec: S-0214 claim 6 (may attack as a friend: the exception asks for a warhead that plants bombs; an ordinary weapon with the flag attacks a transport)
#[test]
fn an_ordinary_weapon_with_the_flag_attacks_a_transport() {
    let mut w = world_with(|r| {
        r.sim.vehicles.get_mut("FTANK").unwrap().primary = Some("Gun".into());
    });
    let van = quiet(&mut w, Kind::Vehicle, "VAN", 0, at(0, 0));
    let flagged = quiet(&mut w, Kind::Vehicle, "FTANK", 0, at(-3, 0));
    assert_eq!(act(&w, flagged, van), Action::Attack);
}
