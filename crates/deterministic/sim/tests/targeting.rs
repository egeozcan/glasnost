//! S-0214 input rules and S-0213 key commands on the synthetic scenario.

use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, Action, ClickOn, KeyCommand, Modifiers};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const ALPHA: HouseId = HouseId(0);

fn of(w: &World, h: u8, kind: Kind, type_index: u32) -> Vec<ObjectId> {
    testkit::house_objects(w, h)
        .into_iter()
        .filter(|&i| {
            w.obj(i)
                .is_some_and(|o| o.kind == kind && o.type_index == type_index)
        })
        .collect()
}

fn carts(w: &World, h: u8) -> Vec<ObjectId> {
    of(w, h, Kind::Vehicle, 0)
}

// spec: S-0214 claim 3 (with an empty selection a selectable object is selected)
// spec: S-0214 claim 12 (select replaces the selection with the clicked object)
#[test]
fn clicking_an_object_selects_it() {
    let w = testkit::new_game(3);
    let a = carts(&w, 0);
    let r = targeting::click(&w, ALPHA, &[], ClickOn::Object(a[0]), Modifiers::default());
    assert_eq!(r.selection, Some(vec![a[0]]));
    assert!(r.orders.is_empty());
    let r = targeting::click(
        &w,
        ALPHA,
        &[a[0]],
        ClickOn::Object(a[1]),
        Modifiers::default(),
    );
    assert_eq!(r.selection, Some(vec![a[1]]));
}

// spec: S-0214 table G row G4 (force-select on an own actor: toggle select)
// spec: S-0214 claim 12 (toggle select adds or removes the clicked object)
#[test]
fn force_select_toggles() {
    let w = testkit::new_game(3);
    let a = carts(&w, 0);
    let fs = Modifiers {
        force_select: true,
        ..Default::default()
    };
    let r = targeting::click(&w, ALPHA, &[a[0]], ClickOn::Object(a[1]), fs);
    assert_eq!(r.selection, Some(vec![a[0], a[1]]));
    let r = targeting::click(&w, ALPHA, &[a[0], a[1]], ClickOn::Object(a[0]), fs);
    assert_eq!(r.selection, Some(vec![a[1]]));
}

// spec: S-0214 claim 2 (each selected object resolves the click itself, in selection order)
// spec: S-0214 table GC row GC9 (a movable actor on a clear cell: move)
// spec: S-0214 claim 11 (the destination of a move on a cell comes from the nearby-cell search)
#[test]
fn a_cell_click_moves_every_selected_unit() {
    let w = testkit::new_game(3);
    let a = carts(&w, 0);
    let cell = testkit::cell(0, 10);
    assert_eq!(
        targeting::cursor(&w, ALPHA, &a, ClickOn::Cell(cell), Modifiers::default()),
        Action::Move
    );
    let r = targeting::click(&w, ALPHA, &a, ClickOn::Cell(cell), Modifiers::default());
    assert_eq!(r.orders.len(), a.len());
    for (o, &id) in r.orders.iter().zip(&a) {
        match o {
            Order::Behaviour {
                actor,
                behaviour: Behaviour::Move,
                dest: Some(Target::Cell(_)),
                ..
            } => assert_eq!(*actor, id),
            other => panic!("unexpected {other:?}"),
        }
    }
}

// spec: S-0214 claim 4 (the generic result g by table G)
// spec: S-0214 claim 6 (the attack rule: an enemy, armed, movable actor)
// spec: S-0214 claim 8 (a vehicle's final action by table V)
// spec: S-0214 claim 10 (table M: attack gives behaviour attack with the target)
// spec: S-0214 table G row G8 (the attack rule holds on an enemy for an armed, movable actor)
// spec: S-0214 table M (attack: behaviour attack with the object as target)
#[test]
fn an_enemy_click_attacks() {
    let w = testkit::new_game(3);
    let a = carts(&w, 0);
    let enemy = carts(&w, 1)[0];
    assert_eq!(
        targeting::action(
            &w,
            ALPHA,
            a[0],
            ClickOn::Object(enemy),
            Modifiers::default()
        ),
        Action::Attack
    );
    let r = targeting::click(
        &w,
        ALPHA,
        &a[..1],
        ClickOn::Object(enemy),
        Modifiers::default(),
    );
    assert_eq!(
        r.orders,
        vec![Order::Behaviour {
            actor: a[0],
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(enemy)),
            dest: None,
        }]
    );
}

// spec: S-0214 table G row G10 (an unarmed actor on another house's unit: select)
// spec: S-0214 table V row V7 (the generic result passes through)
#[test]
fn an_unarmed_vehicle_selects_an_enemy() {
    let w = testkit::new_game(3);
    let crusher = of(&w, 0, Kind::Vehicle, 1)[0];
    let buggy = of(&w, 1, Kind::Vehicle, 2)[0];
    let ff = Modifiers {
        force_fire: true,
        ..Default::default()
    };
    // Force-fire gives an unarmed actor no generic attack (claim 6 (c)).
    assert_eq!(
        targeting::action(&w, ALPHA, crusher, ClickOn::Object(buggy), ff),
        Action::Select
    );
    assert_eq!(
        targeting::action(
            &w,
            ALPHA,
            crusher,
            ClickOn::Object(buggy),
            Modifiers::default()
        ),
        Action::Select
    );
}

// spec: S-0214 table G row G9 (an armed own actor on another house's object it may not attack: nothing)
// spec: S-0214 claim 3 (nothing over a selectable, unselected object acts as select)
#[test]
fn nothing_over_an_object_selects_it() {
    let w = testkit::new_game(3);
    let crusher = of(&w, 0, Kind::Vehicle, 1)[0];
    let a = carts(&w, 0);
    let r = targeting::click(
        &w,
        ALPHA,
        &[crusher],
        ClickOn::Object(a[2]),
        Modifiers::default(),
    );
    assert_eq!(r.selection, Some(vec![a[2]]));
}

// spec: S-0214 claim 10 (table M: move gives behaviour move with the destination of claim 11)
// spec: S-0214 table G row G7 (force-move: move, also onto an own unit)
// spec: S-0214 claim 11 (a move on an object: the object itself when reachable)
#[test]
fn force_move_onto_an_own_unit() {
    let w = testkit::new_game(3);
    let a = carts(&w, 0);
    let fm = Modifiers {
        force_move: true,
        ..Default::default()
    };
    assert_eq!(
        targeting::action(&w, ALPHA, a[0], ClickOn::Object(a[1]), fm),
        Action::Move
    );
    let r = targeting::click(&w, ALPHA, &a[..1], ClickOn::Object(a[1]), fm);
    assert!(matches!(
        r.orders.as_slice(),
        [Order::Behaviour {
            behaviour: Behaviour::Move,
            ..
        }]
    ));
}

// spec: S-0214 claim 5 (the attack-move gesture turns move orders into attack-move orders)
#[test]
fn attack_move_gesture() {
    let w = testkit::new_game(3);
    let a = carts(&w, 0);
    let am = Modifiers {
        force_fire: true,
        force_select: true,
        ..Default::default()
    };
    let r = targeting::click(&w, ALPHA, &a, ClickOn::Cell(testkit::cell(0, 10)), am);
    assert!(!r.orders.is_empty());
    assert!(r.orders.iter().all(|o| matches!(
        o,
        Order::Behaviour {
            attack_move: true,
            ..
        }
    )));
    // One ineligible (unarmed) object turns the gesture off for the whole selection.
    let crusher = of(&w, 0, Kind::Vehicle, 1)[0];
    let mut sel = a.clone();
    sel.push(crusher);
    let r = targeting::click(&w, ALPHA, &sel, ClickOn::Cell(testkit::cell(0, 10)), am);
    assert!(r.orders.iter().all(|o| matches!(
        o,
        Order::Behaviour {
            attack_move: false,
            ..
        }
    )));
}

// spec: S-0214 claim 1 (the representative: the highest rank decides the cursor)
#[test]
fn the_representative_has_the_highest_rank() {
    let w = testkit::new_game(3);
    let crusher = of(&w, 0, Kind::Vehicle, 1)[0];
    let cart = carts(&w, 0)[0];
    let enemy = carts(&w, 1)[0];
    // The unarmed crusher (rank 2) selected first; the armed cart (rank 4) represents the group.
    let c = targeting::cursor(
        &w,
        ALPHA,
        &[crusher, cart],
        ClickOn::Object(enemy),
        Modifiers::default(),
    );
    assert_eq!(c, Action::Attack);
}

// spec: S-0214 claim 9 (a click outside the usable area gives an order only for no-move)
#[test]
fn outside_the_usable_area() {
    let w = testkit::new_game(3);
    let a = carts(&w, 0);
    let outside = glasnost_map::CellXY::new(1, 1);
    assert!(!w.cells.usable(outside));
    assert_eq!(
        targeting::action(
            &w,
            ALPHA,
            a[0],
            ClickOn::Cell(outside),
            Modifiers::default()
        ),
        Action::NoMove
    );
}

// spec: S-0213 table T2 (stop, scatter, guard and deploy for the selection)
#[test]
fn key_commands() {
    let w = testkit::new_game(3);
    let a = carts(&w, 0);
    let crusher = of(&w, 0, Kind::Vehicle, 1)[0];
    let enemy = carts(&w, 1)[0];
    let sel = vec![a[0], crusher, enemy];
    // Stop: movable or able to fire; the enemy unit is not the acting house's.
    assert_eq!(
        targeting::key_command(&w, ALPHA, &sel, KeyCommand::Stop),
        vec![Order::Stop { actor: a[0] }, Order::Stop { actor: crusher }]
    );
    // Guard: movable and able to fire; the unarmed crusher gets nothing.
    let g = targeting::key_command(&w, ALPHA, &sel, KeyCommand::Guard);
    assert_eq!(g.len(), 1);
    assert!(matches!(
        g[0],
        Order::Behaviour {
            behaviour: Behaviour::AreaGuard,
            ..
        }
    ));
    assert_eq!(
        targeting::key_command(&w, ALPHA, &sel, KeyCommand::Scatter).len(),
        2
    );
}

// spec: S-0214 claim 7 (infantry on an object: table I; an armed infantry unit attacks an enemy, row I29)
#[test]
fn infantry_attack_an_enemy() {
    let w = testkit::new_game(3);
    let walker = of(&w, 1, Kind::Infantry, 0)[0];
    let enemy = carts(&w, 0)[0];
    let beta = HouseId(1);
    assert_eq!(
        targeting::action(
            &w,
            beta,
            walker,
            ClickOn::Object(enemy),
            Modifiers::default()
        ),
        Action::Attack
    );
    // An own unit: select (row G10, then row I29).
    let friend = of(&w, 1, Kind::Infantry, 0)[1];
    assert_eq!(
        targeting::action(
            &w,
            beta,
            walker,
            ClickOn::Object(friend),
            Modifiers::default()
        ),
        Action::Select
    );
}
