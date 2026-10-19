//! The own AI and the walk of its base toward water (M5b's naval yard
//! rule, narrowed at the M7b AI audit): a house whose naval yard has no
//! cell grows its base toward the water only when an explored site there
//! would take the yard but for the distance to the base (S-0406 claims 3
//! to 5, what the placement cursor shows). Water too narrow for the yard
//! draws no building: on a stock map the base used to walk nine power
//! plants toward a narrow strip of water and starved its economy.

use glasnost_ai::brain::Brain;
use glasnost_ai::{fixture, game::Game};
use glasnost_map::Map;
use glasnost_sim::econ::{shroud, tech};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, World};

/// The offset along y, from the middle, of the band's southern row.
const BAND_DY: i32 = -15;

/// [`fixture::map`] with a band of water `rows` rows deep north of start
/// 0, out of the base's placement reach.
fn band_map(rows: i32) -> Map {
    let mut m = fixture::map();
    for dx in -fixture::START_DX - 12..=-fixture::START_DX + 12 {
        for dy in BAND_DY - rows + 1..=BAND_DY {
            if let Some(cell) = m.cells.get_mut(fixture::at(dx, dy)) {
                cell.tile = fixture::WATER_TILE;
                cell.overlay = glasnost_map::NO_OVERLAY;
            }
        }
    }
    m
}

fn named(w: &World, h: u8, n: &str) -> usize {
    let Some(i) = w.rules().index_of(Kind::Building, n) else {
        return 0;
    };
    w.objects
        .values()
        .filter(|o| {
            o.owner == Some(HouseId(h)) && o.on_map && o.kind == Kind::Building && o.type_index == i
        })
        .count()
}

fn harvesters(w: &World) -> usize {
    w.objects
        .values()
        .filter(|o| o.owner == Some(HouseId(0)) && o.on_map && o.econ.harvest.is_some())
        .count()
}

/// The power plants house 0's brain orders at the first think once its
/// base stands, the naval yard is buildable and the band is explored.
fn power_orders(rows: i32) -> usize {
    let map = band_map(rows);
    let mut g = Game::new(
        fixture::rules(),
        &map,
        "synthetic-band",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the band skirmish loads");
    let navy = g
        .world
        .rules()
        .index_of(Kind::Building, "NAVY")
        .map(|index| glasnost_sim::econ::TypeRef {
            kind: Kind::Building,
            index,
        })
        .expect("NAVY");
    while g.world.tick < 8000
        && !(tech::buildable(&g.world, HouseId(0), navy)
            && ["POWR", "PROC", "PILL", "FACT"]
                .iter()
                .all(|n| named(&g.world, 0, n) > 0)
            && harvesters(&g.world) >= 2)
    {
        g.step();
    }
    assert!(tech::buildable(&g.world, HouseId(0), navy), "NAVY offered");
    let mut w = g.world.clone();
    // Explore the band: a walker of the house stands at its shore.
    let shore = fixture::at(-fixture::START_DX, BAND_DY + 1);
    testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, shore, 0);
    testkit::run(&mut w, 1);
    assert!(shroud::explored(
        &w,
        HouseId(0),
        fixture::at(-fixture::START_DX, BAND_DY)
    ));
    // The rest of the base is in order: credits, power and no building on the line.
    w.houses[0].credits = 100_000;
    for l in w
        .econ
        .lines
        .lines
        .iter_mut()
        .filter(|l| l.house == HouseId(0))
    {
        if l.item.as_ref().is_some_and(|i| i.t.kind == Kind::Building) {
            l.item = None;
            l.queue.clear();
        }
    }
    let p = &w.houses[0].econ.power;
    assert!(p.output > p.drain, "power to spare: {p:?}");
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let mut brain = Brain::with_plan(&w, &map, HouseId(0), fixture::plan());
    let orders = brain.think(&w);
    let powr = w.rules().index_of(Kind::Building, "POWR");
    let plants = orders
        .iter()
        .filter(|o| matches!(o, Order::Build { what } if what.kind == Kind::Building && Some(what.index) == powr))
        .count();
    // A plant ordered before a building the plants cannot carry (M7b AI
    // round 2, `power::plant_first`) is not a walk.
    plants.saturating_sub(brain.special_uses().plants_first as usize)
}

// PLAN §4.7, S-0406 claims 3 to 5: a band of water two rows deep cannot take the 3 × 3 naval yard anywhere, so the base does not walk toward it.
#[test]
fn does_not_walk_toward_water_too_narrow_for_the_yard() {
    assert_eq!(power_orders(2), 0);
}

// PLAN §4.7, S-0406 claims 3 to 5: a band six rows deep takes the yard but for the distance to the base, so the base walks toward it with a power plant.
#[test]
fn walks_toward_water_that_takes_the_yard() {
    assert_eq!(power_orders(6), 1);
}
