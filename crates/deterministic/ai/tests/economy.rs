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
//! - M8 step B2: a house saves for a superweapon building (holds its
//!   army items that still owe money) while the building owes more than
//!   the money in hand and the house is safe, which now also means not
//!   losing a fight.
//! - M8 step B5: while the superweapon building owes more than the money
//!   in hand, a yard vehicle or an engineer on the lines stays held also
//!   while the house is not safe (neither fights); a yard vehicle bought to
//!   expand is cancelled (S-0404 claim 6) while the house is broke for the
//!   expansion; the brain reads its income off the credits and the lines.
//! - M8 step B8: a house whose harvesters wait for a lost refinery clears
//!   the building line for it (cancels the item there) and orders it before
//!   power; selling out, it keeps the construction yard that builds it.
//! - M8 AI stalled harvester: a harvester left in *unload* without contact
//!   (S-0402 claim 11, its refinery destroyed under it) is sent to harvest.

use glasnost_ai::brain::Brain;
use glasnost_ai::scripted::{Plan, Role, all_types, role_of};
use glasnost_ai::{fixture, game::Game};
use glasnost_map::Map;
use glasnost_sim::econ::production::{Hold, LineKind};
use glasnost_sim::econ::{TypeRef, tech};
use glasnost_sim::objects;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, World};

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
    settled_with(fixture::rules(), map, credits)
}

/// [`settled`] under `rules`.
fn settled_with(
    rules: std::sync::Arc<glasnost_sim::rules::SimRules>,
    map: &Map,
    credits: i32,
) -> World {
    let mut g = Game::new(
        rules,
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

/// House 0 of [`settled`] with no credits, LAB standing (SILO's
/// prerequisite), and on its lines SILO (it gives the missile the brain
/// fires) and an army soldier, both started and running; returns the
/// world, SILO's type and the soldier's.
fn saving_for_a_silo() -> (World, TypeRef, TypeRef) {
    let map = fixture::map();
    let mut w = settled(&map, 0);
    let home = w
        .objects
        .values()
        .find(|o| o.owner == Some(H) && o.kind == Kind::Building)
        .map(|o| o.pos.cell())
        .unwrap();
    let lab = glasnost_sim::econ::TypeRef {
        kind: Kind::Building,
        index: w.rules().index_of(Kind::Building, "LAB").unwrap(),
    };
    let at = (4..20)
        .map(|d| glasnost_map::CellXY::new(home.x, home.y + d))
        .find(|&c| {
            glasnost_sim::econ::placement::cell_test(
                &w,
                lab.index,
                c,
                &glasnost_sim::econ::placement::Test::order(H),
            )
        })
        .expect("room for LAB");
    testkit::spawn(&mut w, Kind::Building, "LAB", 0, at, 0);
    // The sidebar offers SILO once the house has been updated with LAB.
    testkit::run(&mut w, 60);
    let silo = TypeRef {
        kind: Kind::Building,
        index: w.rules().index_of(Kind::Building, "SILO").unwrap(),
    };
    let soldier = first(&w, Role::Army, Kind::Infantry);
    w.houses[0].credits = 1;
    // The house's lines go: the silo starts a new Building line (S-0404
    // claim 1).
    w.econ.lines.lines.retain(|l| l.house != H);
    assert!(
        tech::buildable(&w, H, silo),
        "{:?}",
        tech::answer(&w, H, silo)
    );
    assert!(orders::issue(&mut w, H, Order::Build { what: silo }));
    assert!(orders::issue(&mut w, H, Order::Build { what: soldier }));
    testkit::run(&mut w, 1);
    let item = |w: &World, t: TypeRef| {
        w.econ
            .lines
            .lines
            .iter()
            .filter(|l| l.house == H)
            .filter_map(|l| l.item.as_ref())
            .find(|i| i.t == t)
            .map(|i| (i.hold, i.owed))
    };
    assert!(
        item(&w, silo).is_some_and(|(h, o)| h == Hold::None && o > 0),
        "{:?} {:?}",
        item(&w, silo),
        w.econ.lines.lines
    );
    assert!(item(&w, soldier).is_some_and(|(h, _)| h == Hold::None));
    (w, silo, soldier)
}

// S-0404 claim 5 (the hold order) and PLAN §4.7 (M7b AI round 3): while a building that gives a superweapon the brain fires waits on its line for money and no enemy is near the base, the house holds its army's unit lines, as a person stops training to finish the superweapon; the building itself runs on.
#[test]
fn holds_the_army_while_a_superweapon_building_waits_for_money() {
    let map = fixture::map();
    let (mut w, silo, soldier) = saving_for_a_silo();
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let mut brain = Brain::with_plan(&w, &map, H, plan(2));
    let orders = brain.think(&w);
    assert!(
        orders.contains(&Order::Hold { what: soldier }),
        "{orders:?}"
    );
    assert!(!orders.contains(&Order::Hold { what: silo }));
    assert_eq!(brain.special_uses().superweapon_holds, 1);
    assert_eq!(brain.special_uses().economy_holds, 0);
}

// PLAN §4.7 (M7b AI round 3): with an enemy seen near the base the house keeps training while it saves for the superweapon building.
#[test]
fn keeps_training_with_an_enemy_near_the_base() {
    let map = fixture::map();
    let (mut w, _, soldier) = saving_for_a_silo();
    let near = w
        .objects
        .values()
        .find(|o| o.owner == Some(H) && o.kind == Kind::Building)
        .map(|o| o.pos.cell())
        .unwrap();
    testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        1,
        glasnost_map::CellXY::new(near.x + 3, near.y - 3),
        0,
    );
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let mut brain = Brain::with_plan(&w, &map, H, plan(2));
    let orders = brain.think(&w);
    assert!(
        !orders.contains(&Order::Hold { what: soldier }),
        "{orders:?}"
    );
    assert_eq!(brain.special_uses().superweapon_holds, 0);
}

// PLAN §4.7 (M7b AI round 3): a house whose army is outnumbered by the enemy units it sees keeps training while it saves for the superweapon building, even with no enemy near the base.
#[test]
fn keeps_training_while_outnumbered() {
    let map = fixture::map();
    let (mut w, _, soldier) = saving_for_a_silo();
    let field = fixture::at(4, 0);
    for d in 0..3 {
        testkit::spawn(&mut w, Kind::Infantry, "WALKER", 1, fixture::at(4, d), 0);
    }
    glasnost_sim::econ::shroud::area_reveal(
        &mut w,
        H,
        glasnost_sim::world::Pos::centre(field, 0),
        5,
    );
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let mut brain = Brain::with_plan(&w, &map, H, plan(2));
    let orders = brain.think(&w);
    assert!(
        !orders.contains(&Order::Hold { what: soldier }),
        "{orders:?}"
    );
    assert_eq!(brain.special_uses().superweapon_holds, 0);
}

/// Runs `w` to the next think tick.
fn to_think(w: &mut World) {
    testkit::run(w, 1);
    while w.tick % glasnost_ai::scripted::THINK_PERIOD != 0 {
        testkit::run(w, 1);
    }
}

/// The item of `t` on house 0's lines.
fn item_of(w: &World, t: TypeRef) -> glasnost_sim::econ::production::Item {
    w.econ
        .lines
        .lines
        .iter()
        .filter(|l| l.house == H)
        .filter_map(|l| l.item.as_ref())
        .find(|i| i.t == t)
        .cloned()
        .expect("the item on its line")
}

// S-0404 claim 4 (a step is paid from the money in hand) and PLAN §4.7 (M7b AI round 3, kept in M8 step B2): with the superweapon building's whole balance in hand the house keeps training; with one credit less, every credit the army spends is one the building needs, and it holds the army, but not a held or finished item.
#[test]
fn holds_the_army_only_while_the_balance_is_not_in_hand() {
    let map = fixture::map();
    let (mut w, silo, soldier) = saving_for_a_silo();
    to_think(&mut w);
    let owed = item_of(&w, silo).owed;
    assert!(owed > 1);
    let holds = |credits: i32| {
        let mut w = w.clone();
        w.houses[0].credits = credits;
        let mut brain = Brain::with_plan(&w, &map, H, plan(2));
        let orders = brain.think(&w);
        orders.contains(&Order::Hold { what: soldier })
    };
    assert!(!holds(owed));
    assert!(holds(owed - 1));
}

/// The yard vehicle type.
fn mcv(w: &World) -> TypeRef {
    TypeRef {
        kind: Kind::Vehicle,
        index: w.rules().index_of(Kind::Vehicle, "MCV").unwrap(),
    }
}

/// Puts a yard vehicle on house 0's lines.
fn line_mcv(w: &mut World) -> TypeRef {
    let mcv = mcv(w);
    assert!(
        orders::issue(w, H, Order::Build { what: mcv }),
        "{:?}",
        tech::answer(w, H, mcv)
    );
    to_think(w);
    assert_eq!(item_of(w, mcv).hold, Hold::None);
    mcv
}

// S-0404 claim 5 (the hold order) and PLAN §4.7 (M8 step B1): while the house saves for the superweapon building, a yard vehicle on its line (bought to expand) is held with the army; with the balance in hand it runs on.
#[test]
fn holds_an_expansion_yard_vehicle_while_saving() {
    let map = fixture::map();
    let (mut w, silo, _) = saving_for_a_silo();
    let mcv = line_mcv(&mut w);
    let owed = item_of(&w, silo).owed;
    let holds = |credits: i32| {
        let mut w = w.clone();
        w.houses[0].credits = credits;
        let mut brain = Brain::with_plan(&w, &map, H, plan(2));
        brain.think(&w).contains(&Order::Hold { what: mcv })
    };
    assert!(holds(owed - 1));
    assert!(!holds(owed));
}

/// A brain for house 0 that has watched its income over two whole income
/// windows, its credits rising by `per_window` in each: the brain thinks
/// at the start of each window (orders dropped) while the simulation does
/// not run, so the credits are its only income and nothing pays.
fn earning(w: &mut World, map: &Map, per_window: i32) -> Brain {
    let window = glasnost_ai::scripted::INCOME_WINDOW;
    let mut brain = Brain::with_plan(w, map, H, plan(2));
    w.tick = (w.tick / window + 1) * window;
    brain.think(w);
    for _ in 0..2 {
        w.tick += window;
        w.houses[0].credits += per_window;
        brain.think(w);
    }
    w.tick += glasnost_ai::scripted::THINK_PERIOD;
    brain
}

// S-0404 claim 6 (the cancel-all order) and PLAN §4.7 (M8 step B5): a house broke for the expansion (earning less than half a yard vehicle's cost per income window) cancels the yard vehicle on its line once its credits do not cover what it still owes; earning half the cost, or with the balance in hand, it keeps it.
#[test]
fn cancels_an_expansion_yard_vehicle_while_broke() {
    let map = fixture::map();
    let mut w = settled(&map, 0);
    let mcv = line_mcv(&mut w);
    let cost = glasnost_sim::econ::production::cost_field(&w, mcv);
    let cancels = |per_window: i32, credits: i32| {
        let mut w = w.clone();
        let mut brain = earning(&mut w, &map, per_window);
        w.houses[0].credits = credits;
        let before = brain.special_uses().expansion_cancels;
        let orders = brain.think(&w);
        let n = brain.special_uses().expansion_cancels - before;
        assert_eq!(
            n,
            u32::from(orders.contains(&Order::CancelAll { what: mcv }))
        );
        n == 1
    };
    let owed = item_of(&w, mcv).owed;
    assert!(cancels(cost / 2 - 1, 0));
    assert!(!cancels(cost / 2, 0));
    assert!(!cancels(cost / 2 - 1, owed));
}

// S-0404 claim 4 (each step is paid from the credits) and PLAN §4.7 (M8 step B5): the brain's income over a window is what the credits counter gained plus what the lines paid, as the lane counts it tick by tick.
#[test]
#[ignore = "long in debug builds; run in release (`cargo test --release -p glasnost-ai -- --include-ignored`)"]
fn reads_income_off_the_credits_and_the_lines() {
    income_over_a_window(H);
}

// PLAN §4.7 (M8 step B7): a house whose brain thinks off the window's ticks (its phase, `scripted::think_phase`) reads its income at its first think after each window's end, the same sum.
#[test]
#[ignore = "long in debug builds; run in release (`cargo test --release -p glasnost-ai -- --include-ignored`)"]
fn a_house_thinking_off_the_window_ticks_reads_its_income_too() {
    let beta = HouseId(1);
    assert_ne!(glasnost_ai::scripted::think_phase(beta), 0);
    income_over_a_window(beta);
}

/// Plays `h`'s brain over one income window from its first think at or
/// after a window's end, counting the income tick by tick as
/// `GLASNOST_AI_ECON` does, and compares the brain's reading with it.
fn income_over_a_window(h: HouseId) {
    let map = fixture::map();
    let mut w = settled(&map, 5000);
    let hi = usize::from(h.0);
    let soldier = all_types(&w)
        .into_iter()
        .find(|&t| {
            t.kind == Kind::Infantry
                && role_of(&w, t) == Some(Role::Army)
                && tech::buildable(&w, h, t)
        })
        .expect("an infantry army type the house can build");
    let window = glasnost_ai::scripted::INCOME_WINDOW;
    let mut brain = Brain::with_plan(&w, &map, h, plan(2));
    while w.tick % window != 0 {
        testkit::run(&mut w, 1);
    }
    while !brain.thinks_at(w.tick) {
        testkit::run(&mut w, 1);
    }
    brain.think(&w);
    // Tick by tick, as `GLASNOST_AI_ECON` counts it.
    let owed = |w: &World| -> std::collections::BTreeMap<_, _> {
        w.econ
            .lines
            .lines
            .iter()
            .filter(|l| l.house == h)
            .filter_map(|l| l.item.as_ref())
            .map(|i| (i.object, (i.owed, i.start_price)))
            .collect()
    };
    let mut earned = 0i64;
    let mut last = owed(&w);
    let mut credits = w.houses[hi].credits;
    for _ in 0..window {
        if w.tick % 300 == 0 {
            assert!(orders::issue(&mut w, h, Order::Build { what: soldier }));
        }
        testkit::run(&mut w, 1);
        let now = owed(&w);
        let mut paid = 0i64;
        for (id, &(o, start)) in &now {
            paid += i64::from(last.get(id).map_or(start, |&(b, _)| b) - o);
        }
        for (id, &(o, _)) in &last {
            if !now.contains_key(id) {
                paid += i64::from(o);
            }
        }
        earned += (i64::from(w.houses[hi].credits - credits) + paid).max(0);
        credits = w.houses[hi].credits;
        last = now;
        if brain.thinks_at(w.tick) {
            brain.think(&w);
        }
    }
    assert!(earned > 0, "the harvesters deliver within a window");
    assert_eq!(brain.expansion_probe(&w).income, Some(earned));
}

/// Puts house 0 of `w` in a fight away from its base, an own soldier and
/// two enemy soldiers that see each other; returns the own soldier and the
/// enemy ones, which shoot at nothing yet.
fn a_fight(w: &mut World) -> (ObjectId, Vec<ObjectId>) {
    a_fight_at(w, 4)
}

/// [`a_fight`] with the fight `dx` cells east of the map's middle.
fn a_fight_at(w: &mut World, dx: i32) -> (ObjectId, Vec<ObjectId>) {
    for d in 0..4 {
        testkit::spawn(w, Kind::Infantry, "WALKER", 0, fixture::at(-10, 12 + d), 0);
    }
    let mine = testkit::spawn(w, Kind::Infantry, "WALKER", 0, fixture::at(dx, 3), 0);
    let shooters: Vec<_> = (0..2)
        .map(|d| testkit::spawn(w, Kind::Infantry, "WALKER", 1, fixture::at(dx, d), 0))
        .collect();
    glasnost_sim::econ::shroud::area_reveal(
        w,
        H,
        glasnost_sim::world::Pos::centre(fixture::at(dx, 0), 0),
        5,
    );
    to_think(w);
    for &s in &shooters {
        w.obj_mut(s).unwrap().target = None;
    }
    (mine, shooters)
}

/// The enemy `shooters` of [`a_fight`] shoot at `mine`.
fn shoot(w: &mut World, mine: ObjectId, shooters: &[ObjectId]) {
    for &s in shooters {
        w.obj_mut(s).unwrap().target = Some(glasnost_sim::world::Target::Object(mine));
    }
}

// PLAN §4.7 (M8 step B2): a house losing a fight away from its base (seen enemy army units shoot at its objects and are worth at least its own army units there) keeps training while it saves for the superweapon building, though it is not outnumbered overall.
#[test]
fn keeps_training_while_losing_a_fight() {
    let map = fixture::map();
    let (mut w, _, soldier) = saving_for_a_silo();
    let (mine, shooters) = a_fight(&mut w);
    let holds = |w: &World| {
        let mut brain = Brain::with_plan(w, &map, H, plan(2));
        brain.think(w).contains(&Order::Hold { what: soldier })
    };
    // Not shot at: the house saves.
    assert!(holds(&w));
    shoot(&mut w, mine, &shooters);
    assert!(!holds(&w));
}

// PLAN §4.7 (M8 step B7): a fight the house loses far from home (more than the recall distance from its home cell, as an attack going badly in the enemy's base) ends the saving as one at home does (B7 tried otherwise and the lane played worse); the probe tells the two apart.
#[test]
fn stops_saving_while_losing_a_fight_far_from_home() {
    let map = fixture::map();
    let (mut w, _, soldier) = saving_for_a_silo();
    let (mine, shooters) = a_fight_at(&mut w, 14);
    shoot(&mut w, mine, &shooters);
    let mut brain = Brain::with_plan(&w, &map, H, plan(2));
    let probe = brain.expansion_probe(&w);
    assert!(probe.losing && !probe.losing_at_home, "{probe:?}");
    assert!(probe.intruders == 0 && !probe.outnumbered, "{probe:?}");
    assert!(!brain.think(&w).contains(&Order::Hold { what: soldier }));
}

// S-0404 claims 5 and 6 and PLAN §4.7 (M8 steps B5 and B7): while the superweapon building owes more than the money in hand, a house losing a fight resumes the army that can join it, and cancels its yard vehicle and an engineer, which do not fight (held, they would keep their line from the fighters); while it saves, it holds them with the army.
#[test]
fn cancels_a_yard_vehicle_and_an_engineer_while_losing_a_fight() {
    let map = fixture::map();
    let (mut w, _, soldier) = saving_for_a_silo();
    let mcv = line_mcv(&mut w);
    let (mine, shooters) = a_fight(&mut w);
    let answer = |w: &World, losing: bool| {
        let mut brain = Brain::with_plan(w, &map, H, plan(2));
        assert_eq!(brain.expansion_probe(w).losing, losing);
        let orders = brain.think(w);
        let of = |t: TypeRef| {
            (
                orders.contains(&Order::Hold { what: t }),
                orders.contains(&Order::CancelAll { what: t }),
                orders.contains(&Order::Build { what: t }),
            )
        };
        (
            of(soldier),
            of(mcv),
            brain.special_uses().non_fighter_cancels,
        )
    };
    // Saving: both held.
    assert_eq!(
        answer(&w, false),
        ((true, false, false), (true, false, false), 0)
    );
    shoot(&mut w, mine, &shooters);
    // Losing: the soldier runs on, the yard vehicle is cancelled.
    assert_eq!(
        answer(&w, true),
        ((false, false, false), (false, true, false), 1)
    );
    // The soldier's type made an engineer (test data of our own).
    let mut r = (*w.rules_arc()).clone();
    r.infantry[soldier.index as usize].engineer = true;
    w.rules = Some(std::sync::Arc::new(r));
    assert_eq!(
        answer(&w, true),
        ((false, true, false), (false, true, false), 2)
    );
}

// S-0404 claim 6 (the cancel-all order: the item goes and, with an empty queue, its line) and PLAN §4.7 (M8 step B7): the engineer a house losing a fight cancels while the superweapon building waits leaves its line free, and the brain starts no engineer while the building owes more than the money in hand.
#[test]
fn a_cancelled_engineer_leaves_its_line_and_is_not_started_again() {
    let map = fixture::map();
    let (mut w, _, soldier) = saving_for_a_silo();
    let (mine, shooters) = a_fight(&mut w);
    shoot(&mut w, mine, &shooters);
    let mut r = (*w.rules_arc()).clone();
    r.infantry[soldier.index as usize].engineer = true;
    w.rules = Some(std::sync::Arc::new(r));
    assert!(item_of(&w, soldier).owed > 0);
    let mut brain = Brain::with_plan(&w, &map, H, plan(2));
    let orders = brain.think(&w);
    assert!(
        orders.contains(&Order::CancelAll { what: soldier }),
        "{orders:?}"
    );
    assert!(orders::issue(&mut w, H, Order::CancelAll { what: soldier }));
    // Applied at the end of the tick (S-0213 claim 2).
    testkit::run(&mut w, 1);
    let lined = |w: &World| {
        w.econ.lines.lines.iter().any(|l| {
            l.house == H
                && (l.item.as_ref().is_some_and(|i| i.t == soldier) || l.queue.contains(&soldier))
        })
    };
    assert!(!lined(&w), "{:?}", w.econ.lines.lines);
    // Still short and losing at the next think: no engineer is started.
    to_think(&mut w);
    shoot(&mut w, mine, &shooters);
    assert!(brain.expansion_probe(&w).losing);
    let orders = brain.think(&w);
    assert!(
        !orders.contains(&Order::Build { what: soldier }),
        "{orders:?}"
    );
}

/// House 0's live objects of `role`.
fn of_role(w: &World, role: Role) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| o.owner == Some(H) && o.on_map && o.health > 0)
        .filter(|o| role_of(w, tech::type_of(o)) == Some(role))
        .map(|o| o.id)
        .collect()
}

/// Removes every refinery of house 0.
fn lose_the_refineries(w: &mut World) {
    for id in of_role(w, Role::Refinery) {
        objects::remove(w, id);
    }
    assert!(!has_role(w, Role::Refinery));
}

// S-0402 claim 13 (a harvester whose owner owns no refinery guards and stands), S-0404 claim 6 (the cancel-all order) and PLAN §4.7 (M8 step B8): a house that lost its refinery while a power plant waits for money on the building line cancels the power plant, so the line is free for the refinery; once it is free the house orders the refinery, not a power plant, though short of power.
#[test]
fn rebuilds_a_lost_refinery_before_power() {
    let map = fixture::map();
    let mut w = settled(&map, 0);
    w.econ.lines.lines.retain(|l| l.house != H);
    let powr = first(&w, Role::Power, Kind::Building);
    let refinery = first(&w, Role::Refinery, Kind::Building);
    assert!(orders::issue(&mut w, H, Order::Build { what: powr }));
    to_think(&mut w);
    assert!(item_of(&w, powr).owed > 0);
    // With the refinery standing, the waiting plant stays on its line.
    let orders = think(&mut w.clone(), &map, plan(2));
    assert!(
        !orders.contains(&Order::CancelAll { what: powr }),
        "{orders:?}"
    );
    lose_the_refineries(&mut w);
    // Short of power as well: the refinery still comes first.
    for id in of_role(&w, Role::Power) {
        objects::remove(&mut w, id);
    }
    let orders = think(&mut w, &map, plan(2));
    assert!(
        orders.contains(&Order::CancelAll { what: powr }),
        "{orders:?}"
    );
    assert!(orders::issue(&mut w, H, Order::CancelAll { what: powr }));
    testkit::run(&mut w, 1);
    w.houses[0].credits = glasnost_sim::econ::production::cost_field(&w, refinery);
    let orders = think(&mut w, &map, plan(2));
    assert!(
        orders.contains(&Order::Build { what: refinery }),
        "{orders:?}"
    );
    assert!(!orders.contains(&Order::Build { what: powr }), "{orders:?}");
}

// S-0404 claim 5 (the hold order), claim 6 (the cancel-all order; the refund P′ − B) and PLAN §4.7 (M8 step B9): a house whose harvesters wait for a lost refinery, with less money in hand than the refinery still owes, cancels the held item that has paid the most for its refund, as a person clears the sidebar to finish the refinery; with the refinery's balance in hand it cancels nothing.
#[test]
fn cancels_a_held_item_to_pay_for_a_lost_refinery() {
    let map = fixture::map();
    let mut w = settled(&map, 0);
    w.econ.lines.lines.retain(|l| l.house != H);
    let refinery = first(&w, Role::Refinery, Kind::Building);
    let soldier = first(&w, Role::Army, Kind::Infantry);
    let price = glasnost_sim::econ::production::price(&w, H, soldier);
    w.houses[0].credits = price;
    assert!(orders::issue(&mut w, H, Order::Build { what: soldier }));
    testkit::run(&mut w, 1);
    let from = w.tick;
    while item_of(&w, soldier).progress < 20 && w.tick < from + 3000 {
        testkit::run(&mut w, 1);
    }
    assert!(
        item_of(&w, soldier).owed < price,
        "the soldier has paid steps"
    );
    assert!(orders::issue(&mut w, H, Order::Hold { what: soldier }));
    lose_the_refineries(&mut w);
    w.houses[0].credits = 0;
    assert!(orders::issue(&mut w, H, Order::Build { what: refinery }));
    to_think(&mut w);
    assert_eq!(item_of(&w, soldier).hold, Hold::Player);
    let owed = item_of(&w, refinery).owed;
    assert!(owed > 0);
    let orders = think(&mut w.clone(), &map, plan(2));
    assert!(
        orders.contains(&Order::CancelAll { what: soldier }),
        "{orders:?}"
    );
    w.houses[0].credits = owed;
    let orders = think(&mut w, &map, plan(2));
    assert!(
        !orders.contains(&Order::CancelAll { what: soldier }),
        "{orders:?}"
    );
}

// S-0408 claim 3 (a building with a build-up sprite is sellable), S-0402 claim 13 and PLAN §4.7 (M8 step B8): a broke house whose harvesters wait for a lost refinery sells its other buildings, not the construction yard that would build the refinery, though the yard is the costliest.
#[test]
fn keeps_the_yard_while_selling_for_a_lost_refinery() {
    let mut r = fixture::ruleset();
    for n in ["YARD", "POWR"] {
        let b = r.sim.buildings.get_mut(n).expect("fixture building");
        b.econ.buildup = Some(true);
        b.econ.buildup_frames = Some(8);
    }
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-economy"));
    let mut g = Game::new(
        rules,
        &fixture::map(),
        "synthetic-economy",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the skirmish loads");
    while g.world.tick < 8000
        && !(has_role(&g.world, Role::Refinery)
            && has_role(&g.world, Role::Power)
            && harvesters(&g.world) >= 1)
    {
        g.step();
    }
    assert!(harvesters(&g.world) >= 1, "a harvester by tick 8000");
    let yards = of_role(&g.world, Role::Yard);
    assert!(!yards.is_empty());
    lose_the_refineries(&mut g.world);
    g.world.houses[0].credits = 0;
    let from = g.world.tick;
    let mut sold = Vec::new();
    // Until the refinery stands again or the harvester is gone (then
    // nothing waits for a refinery).
    while g.world.tick < from + 6000
        && g.outcome().is_none()
        && harvesters(&g.world) >= 1
        && !has_role(&g.world, Role::Refinery)
    {
        g.step();
        for o in g.world.objects.values() {
            if o.owner == Some(H) && o.econ.sale.is_some() && !sold.contains(&o.id) {
                sold.push(o.id);
            }
        }
    }
    assert!(!sold.is_empty(), "the broke house sold a building");
    // (The enemy house may destroy the yard meanwhile; it is never sold.)
    assert!(sold.iter().all(|id| !yards.contains(id)), "{sold:?}");
}

/// The squared distance from `c` to the nearest resource cell of `w`.
fn to_ore(w: &World, c: glasnost_map::CellXY) -> i64 {
    w.cells
        .grid
        .iter()
        .map(|(o, _)| o)
        .filter(|&o| glasnost_sim::econ::resource::resource_at(w, o).is_some())
        .map(|o| {
            let (dx, dy) = (i64::from(o.x - c.x), i64::from(o.y - c.y));
            dx * dx + dy * dy
        })
        .min()
        .expect("a resource cell")
}

// PLAN §4.7 (M8 step B8): the house puts its refinery beside its field, nearer the ore than its construction yard stands, with no ore under the foundation.
#[test]
fn places_the_refinery_beside_the_field() {
    let map = fixture::map();
    let mut g = Game::new(
        fixture::rules(),
        &map,
        "synthetic-economy",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the skirmish loads");
    while g.world.tick < 8000 && !has_role(&g.world, Role::Refinery) {
        g.step();
    }
    let w = &g.world;
    let yard = *of_role(w, Role::Yard).first().expect("a yard");
    let refinery = *of_role(w, Role::Refinery).first().expect("a refinery");
    let centre = |id: ObjectId| {
        let o = w.obj(id).expect("own object");
        let c = o.pos.cell();
        glasnost_map::CellXY::new(c.x + o.foundation.0 / 2, c.y + o.foundation.1 / 2)
    };
    assert!(
        to_ore(w, centre(refinery)) < to_ore(w, centre(yard)),
        "refinery at {:?}, yard at {:?}",
        centre(refinery),
        centre(yard)
    );
    let o = w.obj(refinery).expect("the refinery");
    for dy in 0..o.foundation.1 {
        for dx in 0..o.foundation.0 {
            let c = glasnost_map::CellXY::new(o.pos.cell().x + dx, o.pos.cell().y + dy);
            assert!(glasnost_sim::econ::resource::resource_at(w, c).is_none());
        }
    }
}

/// [`fixture::map`] with a moat of water around each start's field (the
/// cells three to four cells from its middle, nine cells south of the
/// start), so that no harvester reaches it, and a second field of 5 × 5
/// cells on open ground, eight cells toward the map's middle and eleven
/// cells north of the start, farther from it. Returns the map and the
/// middle cells of the moated and the open fields of each start.
fn map_moated_fields() -> (Map, Vec<(glasnost_map::CellXY, glasnost_map::CellXY)>) {
    let mut m = fixture::map();
    let mut fields = Vec::new();
    for dx in [-fixture::START_DX, fixture::START_DX] {
        let side = dx.signum();
        let moated = fixture::at(dx, 9);
        let open = fixture::at(dx - side * 8, -11);
        fields.push((moated, open));
        let cells: Vec<glasnost_map::CellXY> = m.cells.iter().map(|(c, _)| c).collect();
        for c in cells {
            let (ox, oy) = (c.x - moated.x, c.y - moated.y);
            let ring = ox.abs().max(oy.abs());
            if (3..=4).contains(&ring)
                && let Some(cell) = m.cells.get_mut(c)
            {
                cell.tile = fixture::WATER_TILE;
                cell.overlay = glasnost_map::NO_OVERLAY;
            }
            if (c.x - open.x).abs() <= 2
                && (c.y - open.y).abs() <= 2
                && let Some(cell) = m.cells.get_mut(c)
            {
                cell.overlay =
                    (((c.x - open.x + 2) * side + c.y - open.y + 2).rem_euclid(12)) as u8;
                cell.overlay_data = 6;
            }
        }
    }
    (m, fields)
}

// PLAN §4.7 (M8 step B9): the refinery goes beside a field a harvester of the house can reach (its reach region, S-0212 claim 4), not beside a nearer one across water.
#[test]
fn places_the_refinery_beside_a_field_a_harvester_reaches() {
    let (map, fields) = map_moated_fields();
    let mut g = Game::new(
        fixture::rules(),
        &map,
        "synthetic-economy",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the skirmish loads");
    // Both fields are in sight from the start, as a person who scouted
    // them sees them: which one the house builds beside is then its
    // choice, not what it has explored.
    for &(moated, open) in &fields {
        for c in [moated, open] {
            for h in [H, HouseId(1)] {
                let p = glasnost_sim::world::Pos::centre(c, 0);
                glasnost_sim::econ::shroud::area_reveal(&mut g.world, h, p, 5);
            }
        }
    }
    while g.world.tick < 8000 && !has_role(&g.world, Role::Refinery) {
        g.step();
    }
    let w = &g.world;
    let yard = *of_role(w, Role::Yard).first().expect("a yard");
    let refinery = *of_role(w, Role::Refinery).first().expect("a refinery");
    let centre = |id: ObjectId| {
        let o = w.obj(id).expect("own object");
        let c = o.pos.cell();
        glasnost_map::CellXY::new(c.x + o.foundation.0 / 2, c.y + o.foundation.1 / 2)
    };
    let d2 = |a: glasnost_map::CellXY, b: glasnost_map::CellXY| {
        let (dx, dy) = (i64::from(a.x - b.x), i64::from(a.y - b.y));
        dx * dx + dy * dy
    };
    let y = centre(yard);
    let &(moated, open) = fields
        .iter()
        .min_by_key(|(m, _)| d2(*m, y))
        .expect("the house's fields");
    assert!(
        d2(moated, y) < d2(open, y),
        "the moated field is the nearer one"
    );
    let r = centre(refinery);
    assert!(
        d2(r, open) < d2(r, moated),
        "refinery at {r:?} toward the open field {open:?}, not the moated {moated:?}"
    );
}

/// [`map_moated_fields`]'s game, with both fields in sight of both
/// houses and a bridge link (S-0212 claim 3) across each
/// moat's north side, from the field's edge to the open ground beyond;
/// run until house 0's first refinery stands, the link's cells named as a
/// destroyed bridge's (S-0212 claim 3) at tick `cut` when given. Returns
/// the refinery's foundation centre and the moated and open fields of
/// house 0.
fn refinery_by_a_bridged_moat(
    cut: Option<i32>,
) -> (
    glasnost_map::CellXY,
    glasnost_map::CellXY,
    glasnost_map::CellXY,
) {
    let (map, fields) = map_moated_fields();
    let mut g = Game::new(
        fixture::rules(),
        &map,
        "synthetic-economy",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the skirmish loads");
    for &(moated, open) in &fields {
        for c in [moated, open] {
            for h in [H, HouseId(1)] {
                let p = glasnost_sim::world::Pos::centre(c, 0);
                glasnost_sim::econ::shroud::area_reveal(&mut g.world, h, p, 5);
            }
        }
        // spec: S-0212 claim 3 (an enabled link makes the regions of its two end cells adjacent)
        g.world.cells.links.push(glasnost_sim::cells::BridgeLink {
            first: glasnost_map::CellXY::new(moated.x, moated.y - 5),
            second: glasnost_map::CellXY::new(moated.x, moated.y - 2),
            along_x: false,
            enabled: true,
        });
    }
    g.world.regions.dirty = true;
    glasnost_sim::regions::ensure(&mut g.world);
    while g.world.tick < 8000 && !has_role(&g.world, Role::Refinery) {
        if cut == Some(g.world.tick) {
            for &(moated, _) in &fields {
                // spec: S-0212 claim 3 (a destroyed bridge: the links within 3 cells of the named cell stop being enabled)
                glasnost_sim::bridges::high::name_cell(
                    &mut g.world,
                    glasnost_map::CellXY::new(moated.x, moated.y - 4),
                    false,
                );
            }
            glasnost_sim::regions::ensure(&mut g.world);
        }
        g.step();
    }
    let w = &g.world;
    let yard = *of_role(w, Role::Yard).first().expect("a yard");
    let refinery = *of_role(w, Role::Refinery).first().expect("a refinery");
    let centre = |id: ObjectId| {
        let o = w.obj(id).expect("own object");
        let c = o.pos.cell();
        glasnost_map::CellXY::new(c.x + o.foundation.0 / 2, c.y + o.foundation.1 / 2)
    };
    let y = centre(yard);
    let d2 = |a: glasnost_map::CellXY, b: glasnost_map::CellXY| {
        let (dx, dy) = (i64::from(a.x - b.x), i64::from(a.y - b.y));
        dx * dx + dy * dy
    };
    let &(moated, open) = fields
        .iter()
        .min_by_key(|(m, _)| d2(*m, y))
        .expect("the house's fields");
    (centre(refinery), moated, open)
}

// S-0212 claims 3 to 5 and PLAN §4.7 (M8 step B10): a field a bridge joins to home is one the house's harvesters reach, so the refinery goes beside it; once the bridge is destroyed (its link disabled, the regions brought up to date) the field is cut off, as a person's move order there learns, and the refinery goes beside the open field instead.
#[test]
fn a_destroyed_bridge_cuts_a_field_off() {
    let d2 = |a: glasnost_map::CellXY, b: glasnost_map::CellXY| {
        let (dx, dy) = (i64::from(a.x - b.x), i64::from(a.y - b.y));
        dx * dx + dy * dy
    };
    let (r, moated, open) = refinery_by_a_bridged_moat(None);
    assert!(
        d2(r, moated) < d2(r, open),
        "bridged: refinery at {r:?} toward the moated field {moated:?}, not the open {open:?}"
    );
    let (r, moated, open) = refinery_by_a_bridged_moat(Some(30));
    assert!(
        d2(r, open) < d2(r, moated),
        "bridge destroyed: refinery at {r:?} toward the open field {open:?}, not the moated {moated:?}"
    );
}

// S-0404 claims 1 and 5 (the earlier line is paid first; the hold order) and PLAN §4.7 (M8 step B10): a house that still has a refinery and builds another (the refinery by a new field) with less money in hand than it owes, pays for it first, holding its army, as a person pauses the rest of the sidebar to get his income back, earning or not; with the balance in hand, or with an enemy near its base, it holds nothing.
#[test]
fn pays_first_for_a_refinery_by_a_new_field() {
    let map = fixture::map();
    let mut w = settled(&map, 0);
    w.econ.lines.lines.retain(|l| l.house != H);
    let refinery = first(&w, Role::Refinery, Kind::Building);
    let soldier = first(&w, Role::Army, Kind::Infantry);
    assert!(orders::issue(&mut w, H, Order::Build { what: refinery }));
    assert!(orders::issue(&mut w, H, Order::Build { what: soldier }));
    to_think(&mut w);
    assert!(has_role(&w, Role::Refinery), "the house keeps its refinery");
    let owed = item_of(&w, refinery).owed;
    assert!(owed > 0);
    let holds = |w: &World, per_window: i32, credits: i32| {
        let mut w = w.clone();
        let mut brain = earning(&mut w, &map, per_window);
        w.houses[0].credits = credits;
        let orders = brain.think(&w);
        assert!(!orders.contains(&Order::Hold { what: refinery }));
        orders.contains(&Order::Hold { what: soldier })
    };
    // (Earning nothing at all, the house would sell out, `Script::sell_when_broke`.)
    assert!(holds(&w, 1, 0));
    // Earning, but short of the balance in hand: the earlier line may
    // take the income first.
    assert!(holds(&w, owed, owed - 1));
    assert!(!holds(&w, 1, owed + 1000));
    let near = w
        .objects
        .values()
        .find(|o| o.owner == Some(H) && o.kind == Kind::Building)
        .map(|o| o.pos.cell())
        .unwrap();
    testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        1,
        glasnost_map::CellXY::new(near.x + 3, near.y - 3),
        0,
    );
    assert!(!holds(&w, 1, 0));
}

// S-0404 claim 2 and S-0402 claim 18 (a refinery's free harvester), S-0408 claim 3 (a building with a build-up sprite is sellable) and PLAN §4.7 (M8 step B10): a house whose refinery and harvesters are all gone, broke, keeps its construction yard, sells its other buildings and rebuilds the refinery that brings a harvester with their refunds; selling out (which lasts until it earns again) does not leave it ordering nothing.
#[test]
fn a_house_with_no_harvester_rebuilds_the_refinery_that_brings_one() {
    let mut r = fixture::ruleset();
    let proc_ = r.sim.buildings.get_mut("PROC").expect("the refinery type");
    proc_.econ.free_unit = Some("MINR".into());
    // Less than the refunds of the house's other buildings come to.
    proc_.cost = Some(1200);
    // Sellable buildings (S-0408 claim 3).
    for b in r.sim.buildings.values_mut() {
        b.econ.buildup = Some(true);
        b.econ.buildup_frames = Some(8);
    }
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(
        &r,
        "synthetic-free-harvester",
    ));
    let map = fixture::map();
    let mut w = settled_with(rules, &map, 0);
    let yards = of_role(&w, Role::Yard);
    assert!(!yards.is_empty());
    lose_the_refineries(&mut w);
    // Its harvesters and army go, those docked or inside a building too.
    let gone: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| o.owner == Some(H) && o.kind != Kind::Building)
        .map(|o| o.id)
        .collect();
    for id in gone {
        objects::remove(&mut w, id);
    }
    w.econ.lines.lines.retain(|l| l.house != H);
    // Broke: what it has from here on comes from its sales. Only house 0
    // thinks, so no enemy comes.
    let mut brain = Brain::with_plan(&w, &map, H, plan(2));
    let from = w.tick;
    let mut sold = Vec::new();
    let mut sold_out = false;
    while w.tick < from + 8000 && !has_role(&w, Role::Harvester) {
        testkit::run(&mut w, 1);
        if w.tick % 15 != 0 {
            continue;
        }
        for o in brain.think(&w) {
            if let Order::Sell { actor } = o {
                sold.push(actor);
            }
            orders::issue(&mut w, H, o);
        }
        sold_out |= brain.broke_probe(&w).selling;
    }
    assert!(sold_out, "the house sold out");
    assert!(!sold.is_empty(), "the broke house sold a building");
    assert!(
        sold.iter().all(|id| !yards.contains(id)),
        "the yard is never sold"
    );
    assert!(
        has_role(&w, Role::Refinery) && has_role(&w, Role::Harvester),
        "a refinery and its harvester by tick {}",
        w.tick
    );
}

/// House 0's orders to `h` among `orders`.
fn orders_to(orders: &[Order], h: ObjectId) -> Vec<Order> {
    orders
        .iter()
        .filter(|o| o.actor() == Some(h))
        .cloned()
        .collect()
}

/// Runs `w` one tick at a time until house 0's brain thinks with one of
/// its harvesters unloading at a refinery in contact; returns the harvester
/// and the refinery.
fn unloading_at_a_think(w: &mut World, brain: &Brain) -> (ObjectId, ObjectId) {
    for _ in 0..12_000 {
        testkit::run(w, 1);
        if !brain.thinks_at(w.tick) {
            continue;
        }
        for h in of_role(w, Role::Harvester) {
            if glasnost_sim::econ::harvest::in_unloading_state(w, h)
                && let Some(r) = w.obj(h).and_then(|o| o.contact)
            {
                return (h, r);
            }
        }
    }
    panic!("no harvester unloading at a think by tick {}", w.tick);
}

// spec: S-0402 claim 11 (*Contact lost otherwise while unloading*: the refinery destroyed under a harvester that unloads, with nothing queued, leaves it in unload for good with its load, quirks.unload_without_contact_stalls on, the reference default) and PLAN §4.7 (our design, M8 AI stalled harvester): the house sends that harvester to harvest with the order a person gives, once its unload has lasted the wait, and the order frees it; a harvester unloading in contact with its refinery gets no order, however long it has unloaded.
#[test]
fn sends_a_harvester_stalled_in_unload_to_harvest() {
    let map = fixture::map();
    let mut w = settled(&map, 0);
    assert!(
        w.rules().quirks.unload_without_contact_stalls,
        "the reference ruleset keeps the quirk"
    );
    // A second refinery, so that the house still owns one when the first
    // is destroyed (the plan's harvester rule needs a refinery).
    testkit::spawn(&mut w, Kind::Building, "PROC", 0, fixture::at(0, 8), 0);
    let brain = Brain::with_plan(&w, &map, H, plan(2));
    let (h, r) = unloading_at_a_think(&mut w, &brain);

    // Unloading in contact: no order, even with an unload long past the wait.
    let mut long = w.clone();
    long.obj_mut(h).unwrap().beh.start_tick -= 10_000;
    assert!(!glasnost_ai::scripted::stalled_unload(&long, h));
    let orders = Brain::with_plan(&long, &map, H, plan(2)).think(&long);
    assert!(orders_to(&orders, h).is_empty(), "{orders:?}");

    // The refinery is destroyed under it: the harvester survives (it is
    // beyond the contact kill distance of S-0312 claim 17) and stalls.
    glasnost_sim::combat::death::kill(
        &mut w,
        r,
        &glasnost_sim::combat::damage::Hit {
            amount: 100_000,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: false,
            no_escape: false,
        },
    );
    assert!(w.obj(r).is_none_or(|o| o.health <= 0 || !o.on_map));
    assert!(has_role(&w, Role::Refinery), "the second refinery stands");
    let load = glasnost_sim::econ::harvest::load(&w, h);
    assert!(load.iter().any(|&n| n > 0), "it still holds a load");
    testkit::run(&mut w, 300);
    let o = w.obj(h).expect("the harvester survives");
    assert_eq!(o.beh.current, Some(Behaviour::Unload), "stalled in unload");
    assert_eq!((o.contact, o.dest, o.beh.queued), (None, None, None));
    assert_eq!(glasnost_sim::econ::harvest::load(&w, h), load, "load kept");
    assert!(glasnost_ai::scripted::stalled_unload(&w, h));

    // The next think sends it to harvest, the order a person gives.
    let mut brain = Brain::with_plan(&w, &map, H, plan(2));
    while !brain.thinks_at(w.tick) {
        testkit::run(&mut w, 1);
    }
    let orders = orders_to(&brain.think(&w), h);
    let harvest = Order::Behaviour {
        actor: h,
        behaviour: Behaviour::Harvest,
        attack_move: false,
        target: None,
        dest: None,
    };
    assert_eq!(orders, vec![harvest.clone()]);
    assert!(orders::issue(&mut w, H, harvest), "the order is taken");
    testkit::run(&mut w, 30);
    let o = w.obj(h).unwrap();
    assert_ne!(o.beh.current, Some(Behaviour::Unload), "it left unload");
    assert!(!glasnost_ai::scripted::stalled_unload(&w, h));
}
