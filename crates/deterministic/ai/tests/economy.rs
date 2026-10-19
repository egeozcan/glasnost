//! The own AI and its economy (M7b AI audit): how many harvesters it
//! keeps, and paying for a lost harvester before the rest of the sidebar.
//!
//! - A house adds harvesters to a large explored field, one for every
//!   twenty resource cells near its refineries and home, at most twice the
//!   plan's count (PLAN §4.7; tuning of our own opponent).
//! - While the house is short of the plan's harvesters and a harvester on
//!   its line waits for money, every other running item is held with the
//!   hold order (S-0404 claim 5), and resumed with a build order once the
//!   economy is whole (S-0404 claims 3 and 5).

use glasnost_ai::brain::Brain;
use glasnost_ai::scripted::{Plan, Role, all_types, role_of};
use glasnost_ai::{fixture, game::Game};
use glasnost_map::Map;
use glasnost_sim::econ::production::{Hold, LineKind};
use glasnost_sim::econ::{TypeRef, tech};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, World};

const H: HouseId = HouseId(0);

/// [`fixture::map`] with a field of 9 × 9 ore cells south of start 0's
/// own field: 81 more resource cells near home.
fn big_field_map() -> Map {
    let mut m = fixture::map();
    for ox in 0..9 {
        for oy in 0..9 {
            let c = fixture::at(-fixture::START_DX + ox - 4, -12 + oy);
            if let Some(cell) = m.cells.get_mut(c) {
                cell.overlay = ((ox + oy) % 12) as u8;
                cell.overlay_data = 6;
            }
        }
    }
    m
}

/// [`fixture::map`] with no resource at all.
fn bare_map() -> Map {
    let mut m = fixture::map();
    let cells: Vec<_> = m.cells.iter().map(|(c, _)| c).collect();
    for c in cells {
        if let Some(cell) = m.cells.get_mut(c) {
            cell.overlay = glasnost_map::NO_OVERLAY;
        }
    }
    m
}

fn plan(harvesters: usize) -> Plan {
    Plan {
        harvesters,
        ..fixture::plan()
    }
}

fn harvesters(w: &World) -> usize {
    w.objects
        .values()
        .filter(|o| o.owner == Some(H) && o.on_map && o.econ.harvest.is_some())
        .count()
}

fn has_role(w: &World, role: Role) -> bool {
    w.objects
        .values()
        .filter(|o| o.owner == Some(H) && o.on_map && o.health > 0)
        .any(|o| role_of(w, tech::type_of(o)) == Some(role))
}

/// The first buildable type of `role` and `kind` for house 0.
fn first(w: &World, role: Role, kind: Kind) -> TypeRef {
    all_types(w)
        .into_iter()
        .filter(|&t| t.kind == kind && role_of(w, t) == Some(role))
        .find(|&t| tech::buildable(w, H, t))
        .expect("a buildable type of the role")
}

/// House 0 of a skirmish on `map` once its refinery, both factories of the
/// test and two harvesters stand, its credits set and its unit lines
/// removed.
fn settled(map: &Map, credits: i32) -> World {
    let mut g = Game::new(
        fixture::rules(),
        map,
        "synthetic-economy",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the skirmish loads");
    while g.world.tick < 8000
        && !(has_role(&g.world, Role::Refinery)
            && harvesters(&g.world) >= 2
            && all_types(&g.world)
                .into_iter()
                .filter(|&t| t.kind == Kind::Infantry && role_of(&g.world, t) == Some(Role::Army))
                .any(|t| tech::buildable(&g.world, H, t)))
    {
        g.step();
    }
    assert!(harvesters(&g.world) >= 2, "two harvesters by tick 8000");
    let mut w = g.world.clone();
    w.houses[0].credits = credits;
    // House 0's unit lines go (a line is made by the first build order of
    // its kind, S-0404 claim 1).
    w.econ
        .lines
        .lines
        .retain(|l| l.house != H || l.kind == LineKind::Building);
    w
}

/// Runs `w` to the next think tick and returns house 0's orders there,
/// with the brain's economy holds and resumes.
fn think_counted(w: &mut World, map: &Map, plan: Plan) -> (Vec<Order>, u32, u32) {
    testkit::run(w, 1);
    while w.tick % 15 != 0 {
        testkit::run(w, 1);
    }
    let mut brain = Brain::with_plan(w, map, H, plan);
    let orders = brain.think(w);
    let u = brain.special_uses();
    (orders, u.economy_holds, u.resumes)
}

fn think(w: &mut World, map: &Map, plan: Plan) -> Vec<Order> {
    think_counted(w, map, plan).0
}

fn harvester_builds(w: &World, orders: &[Order]) -> usize {
    orders
        .iter()
        .filter(
            |o| matches!(o, Order::Build { what } if role_of(w, *what) == Some(Role::Harvester)),
        )
        .count()
}

// PLAN §4.7 (M7b AI audit): with no explored resource the house keeps the plan's two harvesters; no third is ordered.
#[test]
fn keeps_the_plans_harvesters_without_a_field() {
    let map = bare_map();
    let mut w = settled(&map, 100_000);
    let orders = think(&mut w, &map, plan(2));
    assert_eq!(harvesters(&w), 2);
    assert_eq!(harvester_builds(&w, &orders), 0);
}

// PLAN §4.7 (M7b AI audit): with 81 more explored ore cells near home the house adds a harvester beyond the plan's two.
#[test]
fn adds_harvesters_to_a_large_explored_field() {
    let map = big_field_map();
    let mut w = settled(&map, 100_000);
    let walker = fixture::at(-fixture::START_DX, -8);
    testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, walker, 0);
    let orders = think(&mut w, &map, plan(2));
    assert_eq!(harvesters(&w), 2);
    assert_eq!(harvester_builds(&w, &orders), 1);
}

// S-0404 claim 5 (the hold order) and PLAN §4.7: short of the plan's harvesters, with a harvester on its line waiting for money, the house holds its other running item; the harvester runs on.
#[test]
fn holds_the_army_while_a_harvester_waits_for_money() {
    let map = fixture::map();
    let mut w = settled(&map, 0);
    let harvester = first(&w, Role::Harvester, Kind::Vehicle);
    let soldier = first(&w, Role::Army, Kind::Infantry);
    assert!(orders::issue(&mut w, H, Order::Build { what: harvester }));
    assert!(orders::issue(&mut w, H, Order::Build { what: soldier }));
    testkit::run(&mut w, 1);
    let owed = |w: &World, t: TypeRef| {
        w.econ
            .lines
            .lines
            .iter()
            .filter(|l| l.house == H)
            .filter_map(|l| l.item.as_ref())
            .find(|i| i.t == t)
            .map(|i| (i.hold, i.owed))
    };
    assert!(owed(&w, harvester).is_some_and(|(h, o)| h == Hold::None && o > 0));
    assert!(owed(&w, soldier).is_some_and(|(h, _)| h == Hold::None));
    // The plan wants three harvesters: the house is short of one.
    let (orders, holds, _) = think_counted(&mut w, &map, plan(3));
    assert!(
        orders.contains(&Order::Hold { what: soldier }),
        "{orders:?}"
    );
    // The building on the Building line is held too.
    assert_eq!(holds, 2);
    assert!(!orders.contains(&Order::Hold { what: harvester }));
}

// S-0404 claims 3 and 5 and PLAN §4.7: with the economy whole, an item on the player's hold is resumed with a build order for its type.
#[test]
fn resumes_a_held_item_once_the_economy_is_whole() {
    let map = fixture::map();
    let mut w = settled(&map, 100_000);
    let soldier = first(&w, Role::Army, Kind::Infantry);
    assert!(orders::issue(&mut w, H, Order::Build { what: soldier }));
    testkit::run(&mut w, 1);
    assert!(orders::issue(&mut w, H, Order::Hold { what: soldier }));
    testkit::run(&mut w, 1);
    let held = w
        .econ
        .lines
        .lines
        .iter()
        .filter(|l| l.house == H)
        .filter_map(|l| l.item.as_ref())
        .any(|i| i.t == soldier && i.hold == Hold::Player);
    assert!(held, "the soldier is on the player's hold");
    let (orders, holds, resumes) = think_counted(&mut w, &map, plan(2));
    assert!(
        orders.contains(&Order::Build { what: soldier }),
        "{orders:?}"
    );
    assert_eq!((holds, resumes), (0, 1));
    assert!(!orders.iter().any(|o| matches!(o, Order::Hold { .. })));
}
