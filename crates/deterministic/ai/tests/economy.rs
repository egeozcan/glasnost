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
//! - M7b AI round 2: a slave-miner house works every field it knows, with
//!   no cap at twice its count (S-0402 claim 14: the slaves fill one
//!   miner); a house that knew a field and knows none any more wants no
//!   harvester; with money beyond a harvester and the cheapest army unit,
//!   a house builds its army beside an extra harvester past the plan's.

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

fn army_builds(w: &World, orders: &[Order]) -> usize {
    orders
        .iter()
        .filter(|o| matches!(o, Order::Build { what } if role_of(w, *what) == Some(Role::Army)))
        .count()
}

// PLAN §4.7 (M7b AI round 2): a house that has had its harvesters and knows no resource left on the map wants none; it neither orders one nor holds the army for one (a house ended a game at the tick limit with 53 605 credits, waiting for a lost slave miner on a mined-out map).
#[test]
fn wants_no_harvester_on_a_mined_out_map() {
    let map = fixture::map();
    let mut w = settled(&map, 100);
    // The house's brain has seen the fields.
    let mut brain = Brain::with_plan(&w, &map, H, plan(3));
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let _ = brain.think(&w);
    let ore: Vec<_> = w
        .cells
        .grid
        .iter()
        .map(|(c, _)| c)
        .filter(|&c| glasnost_sim::econ::resource::resource_at(&w, c).is_some())
        .collect();
    assert!(!ore.is_empty());
    for c in ore {
        glasnost_sim::crush::remove_overlay(&mut w, c);
    }
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let orders = brain.think(&w);
    assert_eq!(harvester_builds(&w, &orders), 0, "{orders:?}");
    assert!(army_builds(&w, &orders) >= 1, "{orders:?}");
}

/// Alpha of the synthetic M7b skirmish (slave miners) with a war factory,
/// four slave miners beside home and 11 × 11 more explored ore cells there
/// (with the start field more than 140 cells, seven slave miners' worth),
/// and `credits`.
fn slave_miner_house(credits: i32) -> World {
    use glasnost_ai::fixture_m7b as kit;
    use glasnost_sim::world::Pos;
    let mut g = Game::new(
        kit::rules(),
        &kit::map(),
        kit::MAP_NAME,
        fixture::catalogue(),
        kit::setup(5),
    )
    .expect("the M7b skirmish loads");
    while g.world.tick <= g.world.orders_blocked_through || g.world.tick % 15 != 0 {
        g.step();
    }
    let mut w = g.world.clone();
    let home = fixture::at(-glasnost_ai::fixture_m7a::START_DX, 0);
    let near = |dx: i32, dy: i32| glasnost_map::CellXY::new(home.x + dx, home.y + dy);
    testkit::spawn(&mut w, Kind::Building, "FACT", 0, near(6, -6), 0);
    for dy in [-3, -5, -7, -9] {
        testkit::spawn(&mut w, Kind::Vehicle, "SMV", 0, near(-4, dy), 0);
    }
    for ox in 0..11 {
        for oy in 0..11 {
            if let Some(cell) = w.cells.grid.get_mut(near(-12 + ox, -5 + oy)) {
                cell.overlay = ((ox + oy) % 12) as u8;
                cell.overlay_data = 6;
            }
        }
    }
    glasnost_sim::econ::shroud::area_reveal(&mut w, H, Pos::centre(near(-6, 0), 0), 10);
    glasnost_sim::econ::shroud::area_reveal(&mut w, H, Pos::centre(near(0, 6), 0), 10);
    w.houses[0].credits = credits;
    w
}

// PLAN §4.7 (M7b AI round 2): a slave miner is its slaves' own refinery (S-0402 claim 14), so a slave-miner house buys one for every twenty explored resource cells near home with no cap at twice the plan's count, as a harvester house has (four slave miners, a plan of two).
#[test]
fn a_slave_miner_house_works_every_field() {
    let mut w = slave_miner_house(100_000);
    let orders = think(&mut w, &glasnost_ai::fixture_m7b::map(), plan(2));
    assert_eq!(harvester_builds(&w, &orders), 1, "{orders:?}");
}

// PLAN §4.7 (M7b AI round 2): beyond the plan's count, a slave-miner house that wants more miners builds its army beside the miner when it can pay for both, and holds the army for the miner when it cannot.
#[test]
fn builds_the_army_beside_an_extra_slave_miner_with_money() {
    let map = glasnost_ai::fixture_m7b::map();
    let mut rich = slave_miner_house(100_000);
    let with_money = think(&mut rich, &map, plan(2));
    assert_eq!(harvester_builds(&rich, &with_money), 1, "{with_money:?}");

    let mut poor = slave_miner_house(100);
    let without = think(&mut poor, &map, plan(2));
    assert_eq!(harvester_builds(&poor, &without), 1, "{without:?}");
    // The brain's own drainer rule is not the army round's.
    assert!(
        army_builds(&rich, &with_money) > army_builds(&poor, &without),
        "{with_money:?} against {without:?}"
    );
}
