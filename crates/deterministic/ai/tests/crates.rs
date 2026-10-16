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
        &fixture::map(),
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
    let mut w = g.world.clone();
    let home = w
        .objects
        .values()
        .find(|o| o.owner == Some(HouseId(0)) && o.on_map)
        .map(|o| o.pos.cell())
        .unwrap();
    let at = CellXY::new(home.x, home.y + 6);
    let unit = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, at, 0);
    let c = CellXY::new(at.x + 8, at.y);
    // The fixture has no crate overlay: the place is enough for the brain, which asks the world.
    let o = w
        .rules()
        .overlays
        .iter()
        .position(|o| o.crate_)
        .map(|i| i as u8);
    if let Some(o) = o {
        if let Some(cell) = w.cells.grid.get_mut(c) {
            cell.overlay = o;
        }
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
