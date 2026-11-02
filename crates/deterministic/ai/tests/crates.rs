//! The own AI and crates (S-0520, PLAN §4.7): a crate on an explored cell
//! sends one idle unit to it, an unseen crate sends nobody, and a crate
//! that a unit already runs to gets no second unit.

use glasnost_ai::brain::Brain;
use glasnost_ai::fixture;
use glasnost_ai::game::Game;
use glasnost_map::CellXY;
use glasnost_sim::crates::Place;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

fn game() -> Game {
    game_on(&fixture::map())
}

fn game_on(map: &glasnost_map::Map) -> Game {
    let mut r = fixture::ruleset();
    r.registries
        .entry("overlays".into())
        .or_default()
        .push("CRT".into());
    r.sim.overlays.insert(
        "CRT".into(),
        glasnost_rules::sim::OverlayType {
            has_art: Some(true),
            crate_: Some(true),
            ..Default::default()
        },
    );
    let mut g = Game::new(
        std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-skirmish")),
        map,
        "synthetic",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the skirmish loads");
    while g.world.tick <= g.world.orders_blocked_through || g.world.tick % 15 != 0 {
        g.step();
    }
    g
}

/// A world with a unit of house 0 near its base and a crate of money 8 cells
/// from it; `seen` explores the crate's cell for the house.
fn scene(g: &Game, seen: bool) -> (World, ObjectId, CellXY) {
    scene_at(g, seen, |at, _| CellXY::new(at.x + 8, at.y))
}

/// As [`scene`], with the crate on the cell `crate_at` picks from the
/// unit's cell.
fn scene_at(
    g: &Game,
    seen: bool,
    crate_at: impl Fn(CellXY, &World) -> CellXY,
) -> (World, ObjectId, CellXY) {
    let mut w = g.world.clone();
    let home = w
        .objects
        .values()
        .find(|o| o.owner == Some(HouseId(0)) && o.on_map)
        .map(|o| o.pos.cell())
        .unwrap();
    let at = CellXY::new(home.x, home.y + 6);
    let unit = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, at, 0);
    let c = crate_at(at, &w);
    // The fixture has no crate overlay: the place is enough for the brain, which asks the world.
    let o = w
        .rules()
        .overlays
        .iter()
        .position(|o| o.crate_)
        .map(|i| i as u8);
    if let Some(o) = o
        && let Some(cell) = w.cells.grid.get_mut(c)
    {
        cell.overlay = o;
    }
    w.crates.places.insert(
        0,
        Place {
            cell: c,
            start: w.tick,
            life: 100_000,
            trigger: false,
        },
    );
    if seen {
        glasnost_sim::econ::shroud::area_reveal(&mut w, HouseId(0), Pos::centre(c, 0), 2);
    }
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    (w, unit, c)
}

fn moves(orders: &[Order], c: CellXY) -> Vec<ObjectId> {
    orders
        .iter()
        .filter_map(|o| match o {
            Order::Behaviour {
                actor,
                behaviour: Behaviour::Move,
                dest: Some(Target::Cell(d)),
                ..
            } if *d == c => Some(*actor),
            _ => None,
        })
        .collect()
}

// S-0520, PLAN §4.7: a seen crate gets one idle unit; the counter counts the order.
#[test]
fn sends_a_unit_to_a_seen_crate() {
    let g = game();
    let (w, _, c) = scene(&g, true);
    assert!(
        glasnost_sim::crates::is_crate(&w, c),
        "the crate is on the map"
    );
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    assert_eq!(moves(&orders, c).len(), 1);
    assert_eq!(brain.special_uses().crates, 1);
}

// S-0520, PLAN §4.7 (memory): a unit that did not take the order (it stands where it stood, idle, at the next think) is not sent at the same crate again every think.
#[test]
fn does_not_resend_a_unit_that_did_not_take_the_order() {
    let g = game();
    let (mut w, unit, c) = scene(&g, true);
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    assert_eq!(moves(&brain.think(&w), c), vec![unit]);
    // The order is not issued: the unit stays idle on its cell.
    for _ in 0..5 {
        testkit::run(&mut w, 15);
        // Another idle unit may be tried; the one that stood still is not.
        assert!(!moves(&brain.think(&w), c).contains(&unit));
    }
    assert!(brain.given_up() >= 1);
}

// S-0520, PLAN §4.7 (memory, M7a step 2e): a unit that holds the order for a think and then drops it where it stood, without firing, did not take it either: it is not sent at the same crate again.
#[test]
fn does_not_resend_a_unit_that_dropped_the_order_where_it_stood() {
    let g = game();
    let (mut w, unit, c) = scene(&g, true);
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    assert_eq!(moves(&brain.think(&w), c), vec![unit]);
    // At the next think the unit still holds a move on its own cell (busy).
    {
        let o = w.obj_mut(unit).unwrap();
        o.dest = Some(Target::Cell(c));
        o.beh.current = Some(Behaviour::Move);
    }
    w.tick += 15;
    assert!(!moves(&brain.think(&w), c).contains(&unit));
    // Then it drops the order where it stood and is idle again.
    {
        let o = w.obj_mut(unit).unwrap();
        o.dest = None;
        o.beh.current = Some(Behaviour::Guard);
    }
    for _ in 0..3 {
        w.tick += 15;
        assert!(
            !moves(&brain.think(&w), c).contains(&unit),
            "the unit that dropped the order is sent again"
        );
    }
    assert!(brain.given_up() >= 1);
}

// S-0520, PLAN §4.7 (memory): a unit that runs to the crate is not given up.
#[test]
fn a_unit_on_its_way_to_a_crate_is_not_given_up() {
    let g = game();
    let (mut w, _, c) = scene(&g, true);
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    for o in brain.think(&w) {
        glasnost_sim::orders::issue(&mut w, HouseId(0), o);
    }
    for _ in 0..3 {
        testkit::run(&mut w, 15);
        let _ = brain.think(&w);
        assert_eq!(brain.given_up(), 0);
    }
    let _ = c;
}

// S-0520, PLAN §4.7 (design: on its way by its destination, M8 AI seams): a unit that heads for the crate in *guard* with the crate's cell as its destination (a deployer infantry packing up, S-0215 claim 31, takes the move so) is on its way: no second unit is sent.
#[test]
fn a_unit_heading_for_a_crate_in_guard_is_on_its_way() {
    let g = game();
    let (mut w, unit, c) = scene(&g, true);
    // A second idle walker near the crate, free to be sent.
    testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        0,
        CellXY::new(c.x - 4, c.y),
        0,
    );
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    {
        let o = w.obj_mut(unit).unwrap();
        o.dest = Some(Target::Cell(c));
        o.beh.current = Some(Behaviour::Guard);
    }
    let orders = brain.think(&w);
    assert!(moves(&orders, c).is_empty(), "{orders:?}");
    assert_eq!(brain.special_uses().crates, 0);
    // Once it has dropped the destination it is on its way no more.
    {
        let o = w.obj_mut(unit).unwrap();
        o.dest = None;
    }
    w.tick += 15;
    assert_eq!(moves(&brain.think(&w), c).len(), 1, "a unit is sent again");
}

// S-0520, PLAN §4.7 (design: a unit that did not keep its crate order is not sent at that crate again for a while, M8 AI seams): a unit that leaves its cell for the crate and drops the order on the way each time is not sent at the crate again every think.
#[test]
fn a_unit_that_keeps_dropping_the_order_is_not_resent_every_think() {
    let g = game();
    let (mut w, unit, c) = scene(&g, true);
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let mut sent = 0;
    for step in 0..10 {
        if moves(&brain.think(&w), c).contains(&unit) {
            sent += 1;
        }
        // It walks a cell, then drops the order there and stands in guard,
        // free again (the memory does not block it: it left its cell).
        let o = w.obj_mut(unit).unwrap();
        let at = o.pos.cell();
        o.pos = Pos::centre(CellXY::new(at.x, at.y + 1), 0);
        o.dest = None;
        o.beh.current = Some(Behaviour::Guard);
        assert!(
            glasnost_ai::scripted::idle(&w, unit),
            "free again at step {step}"
        );
        w.tick += 15;
    }
    assert_eq!(sent, 1, "sent once in ten thinks");
    // After a while it may be tried again.
    w.tick += 15 * 600;
    let orders = brain.think(&w);
    assert!(moves(&orders, c).contains(&unit), "{orders:?}");
}

// S-0520, PLAN §4.7: it sees only what its house has explored.
#[test]
fn ignores_an_unseen_crate() {
    let g = game();
    let (w, _, c) = scene(&g, false);
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let seen = glasnost_sim::econ::shroud::explored(&w, HouseId(0), c);
    let orders = brain.think(&w);
    assert_eq!(moves(&orders, c).is_empty(), !seen);
}

// S-0520, S-0212 claim 4, PLAN §4.7: a crate on a water cell is not given to a unit that walks: it would keep its move for ever without getting there.
#[test]
fn a_crate_on_water_sends_no_walker() {
    let g = game_on(&fixture::map_strait());
    let (w, _, c) = scene_at(&g, true, |at, w| {
        w.cells
            .grid
            .iter()
            .filter(|(_, x)| x.land == glasnost_map::terrain::land::WATER)
            .map(|(k, _)| k)
            .min_by_key(|k| ((k.x - at.x).pow(2) + (k.y - at.y).pow(2), k.y, k.x))
            .expect("the strait has water")
    });
    assert!(
        glasnost_sim::crates::is_crate(&w, c),
        "the crate is on the map"
    );
    let mut brain = Brain::new(&w, &fixture::map_strait(), HouseId(0));
    let orders = brain.think(&w);
    let walkers: Vec<ObjectId> = moves(&orders, c)
        .into_iter()
        .filter(|&id| w.obj(id).is_some_and(|o| o.kind == Kind::Infantry))
        .collect();
    assert!(walkers.is_empty(), "{orders:?}");
}

// S-0520, PLAN §4.7 (M9 rehearsal): the vehicle that deploys into a construction yard is never sent at a crate, even as the only idle unit near it: sent there it was drawn back and forth between the crate and its deploy spot and never deployed.
#[test]
fn a_yard_vehicle_is_not_sent_at_a_crate() {
    let g = game();
    let (mut w, walker, c) = scene(&g, true);
    let mcv = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "MCV",
        0,
        CellXY::new(c.x - 2, c.y),
        0,
    );
    // The walker is busy with a move far from the crate, so the yard
    // vehicle is the only idle candidate near it.
    let far = CellXY::new(c.x - 20, c.y);
    glasnost_sim::orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: walker,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(far)),
        },
    );
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    assert!(glasnost_sim::crates::is_crate(&w, c));
    assert!(w.obj(mcv).is_some_and(|o| o.on_map));
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    assert!(!moves(&orders, c).contains(&mcv), "{orders:?}");
}
