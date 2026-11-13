//! S-0213 claim 15 and table T2 (#207): the guard key gives an order only
//! to an own object, a harvester vehicle included. Synthetic data: the
//! testkit's combat ruleset with its CART marked as a harvester.

use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, ClickOn, KeyCommand, Modifiers};
use glasnost_sim::testkit::{self, mid, spawn};
use glasnost_sim::world::{HouseId, Target, World};

const SIZE: i32 = 32;
const ALPHA: HouseId = HouseId(0);

fn game() -> World {
    let mut r = testkit::combat_ruleset();
    r.sim.vehicles.get_mut("CART").unwrap().harvester = Some(true);
    let mut w = testkit::game_with(&r, SIZE, 7);
    testkit::run(&mut w, 4);
    w
}

// spec: S-0213 claim 15 (a house's orders act only on its own objects) and table T2 (guard: movable and able to fire, or a harvester vehicle movable or not); S-0214 Terms (an object of another house gets no order from input)
#[test]
fn the_guard_key_gives_no_order_to_another_houses_harvester() {
    let mut w = game();
    let foe = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 3, 0), 0);
    // A click with an empty selection selects the other house's harvester.
    let r = targeting::click(&w, ALPHA, &[], ClickOn::Object(foe), Modifiers::default());
    assert_eq!(r.selection, Some(vec![foe]));
    let sel = [foe];
    assert!(targeting::key_command(&w, ALPHA, &sel, KeyCommand::Stop).is_empty());
    assert!(targeting::key_command(&w, ALPHA, &sel, KeyCommand::Scatter).is_empty());
    let g = targeting::key_command(&w, ALPHA, &sel, KeyCommand::Guard);
    assert!(g.is_empty(), "an order for house 1's harvester: {g:?}");
}

// spec: S-0213 table T2 (guard: a harvester vehicle of the player's own, movable or not)
#[test]
fn the_guard_key_still_orders_an_own_harvester() {
    let mut w = game();
    let own = spawn(&mut w, Kind::Vehicle, "CART", 0, mid(SIZE, 0, 0), 0);
    let cell = w.obj(own).unwrap().pos.cell();
    let g = targeting::key_command(&w, ALPHA, &[own], KeyCommand::Guard);
    assert_eq!(
        g,
        vec![Order::Behaviour {
            actor: own,
            behaviour: Behaviour::AreaGuard,
            attack_move: false,
            target: Some(Target::Cell(cell)),
            dest: None,
        }]
    );
    assert!(orders::issue(&mut w, ALPHA, g[0].clone()));
}
