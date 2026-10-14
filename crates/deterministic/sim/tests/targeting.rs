//! S-0214 input rules and S-0213 key commands on the synthetic scenario.

mod common;

use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, Action, ClickOn, KeyCommand, Modifiers};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const ALPHA: HouseId = HouseId(0);

/// The synthetic scenario under the combat rules (weapon records for its
/// armed types), with the crusher unarmed as in the plain rules.
fn game(seed: u32) -> World {
    let mut r = testkit::combat_ruleset();
    if let Some(v) = r.sim.vehicles.get_mut("CRUSHER") {
        v.primary = None;
    }
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "targeting"));
    glasnost_sim::load::new_game(
        rules,
        &testkit::map(),
        testkit::catalogue(),
        &testkit::setup(seed),
    )
    .expect("the scenario loads")
}

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
    let w = game(3);
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
// spec: S-0214 claim 12 (toggle select adds or removes the clicked object; an addition with positive primary damage goes at the front, one without at the end)
#[test]
fn force_select_toggles() {
    let w = game(3);
    let a = carts(&w, 0);
    let fs = Modifiers {
        force_select: true,
        ..Default::default()
    };
    let r = targeting::click(&w, ALPHA, &[a[0]], ClickOn::Object(a[1]), fs);
    let want = if targeting::positive_primary(&w, a[1]) {
        vec![a[1], a[0]]
    } else {
        vec![a[0], a[1]]
    };
    assert_eq!(r.selection, Some(want));
    let r = targeting::click(&w, ALPHA, &[a[0], a[1]], ClickOn::Object(a[0]), fs);
    assert_eq!(r.selection, Some(vec![a[1]]));
}

// spec: S-0214 claim 2 (each selected object resolves the click itself, in selection order)
// spec: S-0214 table GC row GC9 (a movable actor on a clear cell: move)
// spec: S-0214 claim 11 (a move on a cell in reach: the clicked cell itself, no search)
#[test]
fn a_cell_click_moves_every_selected_unit() {
    let w = game(3);
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
                dest: Some(Target::Cell(c)),
                ..
            } => assert_eq!((*actor, *c), (id, cell)),
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
    let w = game(3);
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
    let w = game(3);
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
    let w = game(3);
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
    let w = game(3);
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
    let w = game(3);
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
    let w = game(3);
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
    let w = game(3);
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
    let w = game(3);
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
    let w = game(3);
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

// ---------------------------------------------------------------------------
// S-0214 revision 11: destinations, tote, table M part 1, table BC, click
// feedback (synthetic scenes of `common`).

mod r11 {
    use super::*;
    use crate::common::{FOOT, Scene, WHEEL, cell, code_tile, run, unit_type};
    use glasnost_map::CellXY;
    use glasnost_rules::sim::{BuildingType, MovementZone, UnitType};
    use glasnost_sim::targeting::BuildingCell;

    const CAR: &str = "TESTCAR";
    const MAN: &str = "TESTMAN";

    fn scene() -> Scene {
        let mut s = Scene::new();
        // An invented weapon record for the armed test types (S-0214 Terms, armed).
        s.ruleset.sim.weapons.insert(
            "TestGun".into(),
            glasnost_rules::sim::WeaponType {
                damage: Some(10),
                range: Some(glasnost_rules::sim::Decimal("3".into())),
                ..Default::default()
            },
        );
        s.vehicle(CAR, unit_type(WHEEL, 40, 6));
        s.infantry(MAN, unit_type(FOOT, 20, 0));
        s
    }

    /// Makes `c` a rock cell (no land row: blocked for every class).
    fn rock(s: &mut Scene, c: CellXY) {
        s.set_cell(c, code_tile(7), 0);
    }

    /// A ring of rock around `c`, so that `c` is an enclosed cell.
    fn ring(s: &mut Scene, c: CellXY) {
        for dy in -1..=1 {
            for dx in -1..=1 {
                if (dx, dy) != (0, 0) {
                    rock(s, CellXY::new(c.x + dx, c.y + dy));
                }
            }
        }
    }

    fn built(s: &Scene) -> World {
        let mut w = s.build();
        run(&mut w, 1);
        w
    }

    fn at(w: &World, c: CellXY) -> ObjectId {
        w.objects.values().find(|o| o.pos.cell() == c).unwrap().id
    }

    fn dest(r: &targeting::ClickResult) -> Option<Target> {
        match r.orders.as_slice() {
            [Order::Behaviour { dest, .. }] => *dest,
            other => panic!("one behaviour order expected: {other:?}"),
        }
    }

    // spec: S-0214 claim 11 (on a cell in reach: the clicked cell itself, no search, even when occupied and for a no-move)
    // spec: S-0214 table GC row GC11 (a movable actor whose entry outcome is not clear: no-move)
    #[test]
    fn in_reach_cell_is_the_destination() {
        let mut s = scene();
        s.place_unit(0, CAR, cell(0, 0), 64)
            .place_unit(0, CAR, cell(3, 0), 64);
        let w = built(&s);
        let a = at(&w, cell(0, 0));
        let on = ClickOn::Cell(cell(3, 0));
        assert_eq!(
            targeting::action(&w, ALPHA, a, on, Modifiers::default()),
            Action::NoMove
        );
        let r = targeting::click(&w, ALPHA, &[a], on, Modifiers::default());
        assert_eq!(dest(&r), Some(Target::Cell(cell(3, 0))));
    }

    // spec: S-0214 claim 11 (a cell not in reach: a cell found by the search; a self-teleporting type moving: the clicked cell itself)
    #[test]
    fn enclosed_cell_and_teleporter() {
        let mut s = scene();
        let enclosed = cell(6, 0);
        ring(&mut s, enclosed);
        s.vehicle(
            "TESTPORT",
            UnitType {
                teleporter: Some(true),
                ..unit_type(WHEEL, 40, 6)
            },
        );
        s.place_unit(0, CAR, cell(0, 0), 64)
            .place_unit(0, "TESTPORT", cell(0, 3), 64);
        let w = built(&s);
        let car = at(&w, cell(0, 0));
        let port = at(&w, cell(0, 3));
        let on = ClickOn::Cell(enclosed);
        for id in [car, port] {
            assert_eq!(
                targeting::action(&w, ALPHA, id, on, Modifiers::default()),
                Action::Move
            );
        }
        let r = targeting::click(&w, ALPHA, &[car], on, Modifiers::default());
        let d = dest(&r);
        assert!(matches!(d, Some(Target::Cell(c)) if c != enclosed), "{d:?}");
        let r = targeting::click(&w, ALPHA, &[port], on, Modifiers::default());
        assert_eq!(dest(&r), Some(Target::Cell(enclosed)));
    }

    // spec: S-0214 claim 11 (a free mover's no-move searches; another actor's no-move on a cell in reach takes the cell itself)
    // spec: S-0214 Terms (free mover: a jumpjet infantry type)
    #[test]
    fn free_mover_no_move_searches() {
        let mut s = scene();
        s.infantry(
            "TESTJET",
            UnitType {
                jumpjet: Some(true),
                ..unit_type(FOOT, 20, 0)
            },
        );
        s.place_infantry(0, MAN, cell(0, 0), 2)
            .place_infantry(0, "TESTJET", cell(0, 3), 2)
            .place_unit(0, CAR, cell(3, 0), 64);
        let w = built(&s);
        let man = at(&w, cell(0, 0));
        let jet = at(&w, cell(0, 3));
        let on = ClickOn::Cell(cell(3, 0));
        for id in [man, jet] {
            assert_eq!(
                targeting::action(&w, ALPHA, id, on, Modifiers::default()),
                Action::NoMove
            );
        }
        let r = targeting::click(&w, ALPHA, &[man], on, Modifiers::default());
        assert_eq!(dest(&r), Some(Target::Cell(cell(3, 0))));
        let r = targeting::click(&w, ALPHA, &[jet], on, Modifiers::default());
        let d = dest(&r);
        assert!(
            matches!(d, Some(Target::Cell(c)) if c != cell(3, 0)),
            "{d:?}"
        );
    }

    // spec: S-0214 claim 11 (a move onto an object on an enclosed cell: a free mover gets the object, a ground unit a searched cell)
    // spec: S-0214 Terms (underground type: movement zone subterranean, a free mover)
    #[test]
    fn object_on_an_enclosed_cell() {
        let mut s = scene();
        let enclosed = cell(6, 0);
        ring(&mut s, enclosed);
        s.vehicle(
            "TESTMOLE",
            UnitType {
                movement_zone: Some(MovementZone::Subterranean),
                ..unit_type(WHEEL, 40, 6)
            },
        );
        s.place_unit(0, CAR, cell(0, 0), 64)
            .place_unit(0, "TESTMOLE", cell(0, 3), 64)
            .place_unit(1, CAR, enclosed, 64);
        let w = built(&s);
        let car = at(&w, cell(0, 0));
        let mole = at(&w, cell(0, 3));
        let enemy = at(&w, enclosed);
        let fm = Modifiers {
            force_move: true,
            ..Default::default()
        };
        let on = ClickOn::Object(enemy);
        let r = targeting::click(&w, ALPHA, &[mole], on, fm);
        assert_eq!(dest(&r), Some(Target::Object(enemy)));
        let r = targeting::click(&w, ALPHA, &[car], on, fm);
        assert!(matches!(dest(&r), Some(Target::Cell(_))));
    }

    // spec: S-0214 table AC rows AC2 and AC3 (a carrier carrying nothing over an allied vehicle: tote, nothing on a vehicle factory's cell)
    // spec: S-0214 table M part 1 (tote: behaviour move, destination the vehicle)
    #[test]
    fn carrier_totes() {
        let mut s = scene();
        s.aircraft(
            "TESTLIFT",
            UnitType {
                carryall: Some(true),
                ..unit_type(glasnost_rules::sim::LocomotionClass::Winged, 40, 6)
            },
        );
        s.building(
            "TESTPLANT",
            BuildingType {
                strength: Some(500),
                weapons_factory: Some(true),
                ..Default::default()
            },
        );
        s.place_aircraft(0, "TESTLIFT", cell(0, 0), 64)
            .place_unit(0, CAR, cell(3, 0), 64)
            .place_structure(0, "TESTPLANT", cell(6, 6))
            .place_unit(0, CAR, cell(6, 6), 64);
        let w = built(&s);
        let lift = objects_of_kind(&w, Kind::Aircraft)[0];
        let car = at(&w, cell(3, 0));
        let parked = w
            .objects
            .values()
            .find(|o| o.kind == Kind::Vehicle && o.pos.cell() == cell(6, 6))
            .map(|o| o.id);
        let on = ClickOn::Object(car);
        assert_eq!(
            targeting::action(&w, ALPHA, lift, on, Modifiers::default()),
            Action::Tote
        );
        let r = targeting::click(&w, ALPHA, &[lift], on, Modifiers::default());
        assert_eq!(
            r.orders,
            vec![Order::Behaviour {
                actor: lift,
                behaviour: Behaviour::Move,
                attack_move: false,
                target: None,
                dest: Some(Target::Object(car)),
            }]
        );
        let parked = parked.expect("the map places a vehicle on the factory's cell");
        assert_eq!(
            targeting::action(
                &w,
                ALPHA,
                lift,
                ClickOn::Object(parked),
                Modifiers::default()
            ),
            Action::Nothing
        );
    }

    fn objects_of_kind(w: &World, k: Kind) -> Vec<ObjectId> {
        w.objects
            .values()
            .filter(|o| o.kind == k)
            .map(|o| o.id)
            .collect()
    }

    // spec: S-0214 claim 15 (berserk or melee-linked actors get no order, except tote for a berserk carrier aircraft)
    // spec: S-0214 Edge cases (quirks.berserk_carrier_tote off: the berserk carrier gets no order either)
    #[test]
    fn berserk_carrier_tote_switch() {
        use targeting::held_without_order as held;
        assert!(!held(false, false, true, Action::Tote, true));
        assert!(held(true, false, false, Action::Move, true));
        assert!(held(false, true, false, Action::Attack, true));
        assert!(held(true, false, true, Action::Move, true));
        assert!(!held(true, false, true, Action::Tote, true));
        assert!(held(true, false, true, Action::Tote, false));
        assert!(held(true, false, false, Action::Tote, true));
    }

    // spec: S-0214 table BC (rows BC1 and BC3 to BC7)
    // spec: S-0214 Edge cases (quirks.building_cell_attack_no_weapon: row BC3 gives attack with no weapon in the first slot; off: nothing)
    #[test]
    fn building_cell_table() {
        use targeting::building_cell as bc;
        let tower = BuildingCell {
            first_slot_weapon: true,
            fires_on_ground: true,
            ..Default::default()
        };
        assert_eq!(bc(Action::Attack, tower, true), Action::Attack);
        let empty_slot = BuildingCell::default();
        assert_eq!(bc(Action::Attack, empty_slot, true), Action::Attack);
        assert_eq!(bc(Action::Attack, empty_slot, false), Action::Nothing);
        let pulse = BuildingCell {
            first_slot_weapon: true,
            ..Default::default()
        };
        assert_eq!(bc(Action::Attack, pulse, true), Action::Nothing);
        let rally = BuildingCell {
            own_rally: true,
            ..Default::default()
        };
        assert_eq!(bc(Action::NoMove, rally, true), Action::Move);
        assert_eq!(bc(Action::Nothing, rally, true), Action::Move);
        // Row BC7: otherwise c, including a move of a building that is not movable.
        assert_eq!(
            bc(Action::Move, BuildingCell::default(), true),
            Action::Move
        );
        let hidden = BuildingCell {
            invisible: true,
            own_rally: true,
            ..Default::default()
        };
        assert_eq!(bc(Action::Move, hidden, true), Action::Nothing);
    }

    // spec: S-0214 Edge cases (disarm and heal go through the attack check: an unarmed engineer gets no order from disarm bomb)
    // spec: S-0214 Edge cases (quirks.disarm_bomb_needs_weapon off: it needs only a movable actor and gives the attack order)
    // spec: S-0214 table M part 1 (disarm bomb as for attack; none on the actor itself)
    #[test]
    fn disarm_bomb_switch() {
        for quirk in [true, false] {
            let mut s = scene();
            s.ruleset.globals.quirks.disarm_bomb_needs_weapon = quirk;
            s.infantry(
                "TESTENGI",
                UnitType {
                    engineer: Some(true),
                    ..unit_type(FOOT, 20, 0)
                },
            );
            s.infantry(
                "TESTARMED",
                UnitType {
                    engineer: Some(true),
                    primary: Some("TestGun".into()),
                    ..unit_type(FOOT, 20, 0)
                },
            );
            s.place_infantry(0, "TESTENGI", cell(0, 0), 2)
                .place_infantry(0, "TESTARMED", cell(0, 2), 2)
                .place_unit(0, CAR, cell(3, 0), 64);
            let w = built(&s);
            let engi = at(&w, cell(0, 0));
            let armed = at(&w, cell(0, 2));
            let car = at(&w, cell(3, 0));
            let on = ClickOn::Object(car);
            let attack = |id| {
                Some(Order::Behaviour {
                    actor: id,
                    behaviour: Behaviour::Attack,
                    attack_move: false,
                    target: Some(Target::Object(car)),
                    dest: None,
                })
            };
            let d = |id| {
                targeting::order(
                    &w,
                    ALPHA,
                    id,
                    Action::DisarmBomb,
                    on,
                    Modifiers::default(),
                    false,
                )
            };
            assert_eq!(d(engi), if quirk { None } else { attack(engi) });
            assert_eq!(d(armed), attack(armed));
            let own = targeting::order(
                &w,
                ALPHA,
                armed,
                Action::DisarmBomb,
                ClickOn::Object(armed),
                Modifiers::default(),
                false,
            );
            assert_eq!(own, None);
        }
    }

    // spec: S-0214 claim 14 (only the first object of the selection acknowledges, only when it gets an order; the marker only for move)
    #[test]
    fn click_feedback() {
        let mut s = scene();
        s.place_unit(0, CAR, cell(0, 0), 64)
            .place_unit(0, CAR, cell(0, 2), 64)
            .place_unit(1, CAR, cell(-6, 0), 64);
        let w = built(&s);
        let a = at(&w, cell(0, 0));
        let b = at(&w, cell(0, 2));
        let enemy = at(&w, cell(-6, 0));
        let open = ClickOn::Cell(cell(4, 4));
        let r = targeting::click(&w, ALPHA, &[a, b], open, Modifiers::default());
        assert_eq!(r.orders.len(), 2);
        assert_eq!((r.acknowledge, r.marker), (Some(a), true));
        // A no-move order: acknowledged, no marker.
        let r = targeting::click(
            &w,
            ALPHA,
            &[a, b],
            ClickOn::Cell(cell(0, 2)),
            Modifiers::default(),
        );
        assert_eq!(r.orders.len(), 2);
        assert_eq!((r.acknowledge, r.marker), (Some(a), false));
        // The first selected object gets no order: nobody acknowledges.
        let r = targeting::click(&w, ALPHA, &[enemy, a], open, Modifiers::default());
        assert_eq!(r.orders.len(), 1);
        assert_eq!((r.acknowledge, r.marker), (None, false));
    }

    // spec: S-0214 F5 (i = x mod n, unsigned; none for an empty applicable list)
    #[test]
    fn order_response_choice() {
        assert_eq!(targeting::order_response(7, 3), Some(1));
        assert_eq!(targeting::order_response(u32::MAX, 2), Some(1));
        assert_eq!(targeting::order_response(5, 0), None);
    }
}

// S-0214 revision 12: the term *can be selected*.

mod r12 {
    use super::*;
    use crate::common::{Scene, WHEEL, cell, run, unit_type};
    use glasnost_rules::sim::BuildingType;

    // spec: S-0214 Terms (can be selected: a building whose type is invisible in the game is not), claim 3 (nothing over an object that cannot be selected does nothing; select over one that can) and claim 12 (a toggle adds only an object that can be selected)
    #[test]
    fn invisible_building_cannot_be_selected() {
        let mut s = Scene::new();
        s.vehicle("TESTCAR", unit_type(WHEEL, 40, 6));
        for (n, hidden) in [("TESTHUT", false), ("TESTGHOST", true)] {
            s.register("buildings", n);
            s.ruleset.sim.buildings.insert(
                n.into(),
                BuildingType {
                    strength: Some(300),
                    foundation: Some([1, 1]),
                    invisible_in_game: Some(hidden),
                    ..Default::default()
                },
            );
        }
        s.place_structure(0, "TESTHUT", cell(4, 0))
            .place_structure(0, "TESTGHOST", cell(6, 0))
            .place_unit(0, "TESTCAR", cell(0, 0), 64);
        let mut w = s.build();
        run(&mut w, 1);
        let find = |c| w.objects.values().find(|o| o.pos.cell() == c).unwrap().id;
        let (hut, ghost) = (find(cell(4, 0)), find(cell(6, 0)));
        assert!(targeting::selectable(&w, hut));
        assert!(!targeting::selectable(&w, ghost));
        let click =
            |on| targeting::click(&w, ALPHA, &[], ClickOn::Object(on), Modifiers::default());
        assert_eq!(click(hut).selection, Some(vec![hut]));
        assert_eq!(click(ghost).selection, None);
        assert_eq!(
            targeting::cursor(&w, ALPHA, &[], ClickOn::Object(ghost), Modifiers::default()),
            Action::Nothing
        );
        // Claim 12: a force-select toggle adds the hut but not the ghost.
        let car = common::objects_of(&w, Kind::Vehicle)[0];
        let fs = Modifiers {
            force_select: true,
            ..Default::default()
        };
        let toggle = |on| targeting::click(&w, ALPHA, &[car], ClickOn::Object(on), fs);
        assert_eq!(toggle(hut).selection, Some(vec![car, hut]));
        assert_eq!(toggle(ghost).selection, None);
    }
}

mod r14 {
    use super::*;
    use crate::common::{Scene, cell, run};
    use glasnost_rules::sim::BuildingType;

    /// Buildings of house 0 with the given factory words, one per cell
    /// (2k, 0); returns the world and the buildings in that order.
    fn buildings(types: &[(&str, Option<&str>, bool)]) -> (World, Vec<ObjectId>) {
        let mut s = Scene::new();
        for (i, (n, factory, cloning)) in types.iter().enumerate() {
            s.building(
                n,
                BuildingType {
                    strength: Some(300),
                    foundation: Some([1, 1]),
                    factory: factory.map(str::to_owned),
                    cloning: Some(*cloning),
                    ..Default::default()
                },
            );
            s.place_structure(0, n, cell(2 * i as i32, 0));
        }
        let mut w = s.build();
        run(&mut w, 1);
        let ids = (0..types.len())
            .map(|i| {
                w.objects
                    .values()
                    .find(|o| o.pos.cell() == cell(2 * i as i32, 0))
                    .unwrap()
                    .id
            })
            .collect();
        (w, ids)
    }

    fn deploy(w: &World, b: ObjectId) -> Vec<Order> {
        targeting::click(w, ALPHA, &[b], ClickOn::Object(b), Modifiers::default()).orders
    }

    // spec: S-0214 claim 17 (a recognised non-producing category counts as factory-marked for rows B3 and B5 and table M's deploy row)
    // spec: S-0214 table B rows B3 and B5, table M part 3 (deploy: a primary order if factory-marked)
    // spec: S-0213 claim 13 (the primary order: any category but no factory; it clears others of exactly the same category)
    #[test]
    fn factory_marked_deploy_and_primary() {
        let (mut w, ids) = buildings(&[
            ("TESTFACT", Some("vehicle"), false),
            ("TESTTERR", Some("terrain"), false),
            ("TESTTERR2", Some("terrain"), false),
            ("TESTPLAIN", None, false),
        ]);
        let (fact, terr, terr2, plain) = (ids[0], ids[1], ids[2], ids[3]);
        for b in [fact, terr, terr2] {
            assert_eq!(deploy(&w, b), vec![Order::Primary { actor: b }]);
        }
        assert!(deploy(&w, plain).is_empty(), "no factory: nothing");
        let primary = |w: &World, b: ObjectId| w.obj(b).unwrap().econ.building.primary;
        // Past the order block of the game start (S-0213 claim 1).
        while w.tick <= w.orders_blocked_through {
            run(&mut w, 1);
        }
        for b in [fact, terr, plain] {
            assert!(glasnost_sim::orders::issue(
                &mut w,
                ALPHA,
                Order::Primary { actor: b }
            ));
        }
        run(&mut w, 1);
        assert!(primary(&w, fact) && primary(&w, terr));
        assert!(!primary(&w, plain), "the no-factory value is not changed");
        // B3: already primary, nothing.
        assert!(deploy(&w, terr).is_empty());
        // The second terrain-category building takes the mark from the first, not from the vehicle factory.
        assert!(glasnost_sim::orders::issue(
            &mut w,
            ALPHA,
            Order::Primary { actor: terr2 }
        ));
        run(&mut w, 1);
        assert!(primary(&w, terr2) && !primary(&w, terr) && primary(&w, fact));
    }

    // spec: S-0214 Terms (rally-capable: a factory of infantry or vehicles, a cloning building or a repair depot) and claim 17 (a non-producing category alone does not grant it, nor cancels cloning)
    #[test]
    fn rally_capable_by_category() {
        let (w, ids) = buildings(&[
            ("TESTBARR", Some("infantry"), false),
            ("TESTTERR", Some("terrain"), false),
            ("TESTCLONE", Some("infantry_object"), true),
            ("TESTAIR", Some("aircraft"), false),
        ]);
        let spot = |b: ObjectId| {
            targeting::click(
                &w,
                ALPHA,
                &[b],
                ClickOn::Cell(cell(3, 4)),
                Modifiers::default(),
            )
            .orders
            .iter()
            .any(|o| matches!(o, Order::GuardSpot { .. }))
        };
        assert!(spot(ids[0]), "a factory of infantry");
        assert!(!spot(ids[1]), "a non-producing category alone");
        assert!(
            spot(ids[2]),
            "a cloning building of a non-producing category"
        );
        assert!(!spot(ids[3]), "a factory of aircraft");
    }
}
