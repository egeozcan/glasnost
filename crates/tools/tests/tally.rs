//! The AI lane's counters of transport loads, hovering transports and
//! superweapon launches by kind (M6c step 2b and 2c), on the synthetic
//! skirmish (no game data).

use glasnost_ai::{fixture, game::Game};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Kind;
use glasnost_sim::superweapons::charge;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, World};
use glasnost_tools::skirmish_lane::Tally;

fn of(w: &World, h: u8, name: &str) -> Vec<glasnost_sim::world::ObjectId> {
    w.objects
        .values()
        .filter(|o| o.owner == Some(HouseId(h)) && o.on_map && o.health > 0)
        .filter(|o| {
            let r = w.rules();
            match o.kind {
                Kind::Building => r.buildings[o.type_index as usize].name == name,
                Kind::Vehicle => r.vehicles[o.type_index as usize].name == name,
                _ => false,
            }
        })
        .map(|o| o.id)
        .collect()
}

// Riflemen carried over the strait by a hovering transport are counted as
// loads onto a hovering transport, and their unloading as unloads
// (S-0507, S-0503).
#[test]
fn counts_the_loads_onto_a_hovering_transport() {
    let mut g = Game::new(
        fixture::rules(),
        &fixture::map_strait(),
        "synthetic-strait",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the strait skirmish loads");
    let mut tally = Tally::default();
    while of(&g.world, 0, "YARD").is_empty() && g.world.tick < 3000 {
        g.step();
        tally.observe(&g.world);
    }
    let yard = g.world.obj(of(&g.world, 0, "YARD")[0]).unwrap().pos.cell();
    let free = |w: &World, c: glasnost_map::CellXY| {
        w.cells.usable(c) && w.objects.values().all(|o| o.pos.cell() != c)
    };
    let cells: Vec<glasnost_map::CellXY> = (0..12)
        .map(|d| glasnost_map::CellXY::new(yard.x, yard.y + 5 + d))
        .filter(|&c| free(&g.world, c))
        .take(4)
        .collect();
    assert_eq!(cells.len(), 4);
    testkit::spawn(&mut g.world, Kind::Vehicle, "LIFT", 0, cells[0], 0);
    for &c in &cells[1..] {
        testkit::spawn(&mut g.world, Kind::Infantry, "WALKER", 0, c, 0);
    }
    let limit = g.world.tick + 6000;
    while g.world.tick < limit && tally.unloads == 0 {
        g.step();
        tally.observe(&g.world);
    }
    assert!(tally.loads > 0, "riflemen boarded: {tally:?}");
    assert_eq!(
        tally.hover_loads, tally.loads,
        "onto the hovering transport"
    );
    assert!(tally.unloads > 0, "they were let out: {tally:?}");
}

// A launch is counted when the ready mark falls, by the kind of the type
// (S-0512, S-0513).
#[test]
fn counts_a_launch_by_kind() {
    let mut g = Game::new(
        fixture::rules(),
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the synthetic skirmish loads");
    let mut tally = Tally::default();
    while of(&g.world, 0, "YARD").is_empty() && g.world.tick < 3000 {
        g.step();
        tally.observe(&g.world);
    }
    let yard = g.world.obj(of(&g.world, 0, "YARD")[0]).unwrap().pos.cell();
    let at = (2..12)
        .map(|d| glasnost_map::CellXY::new(yard.x - d, yard.y))
        .find(|&c| g.world.cells.usable(c) && g.world.objects.values().all(|o| o.pos.cell() != c))
        .expect("a free cell for the silo");
    testkit::spawn(&mut g.world, Kind::Building, "SILO", 0, at, 0);
    let limit = g.world.tick + 3000;
    while g.world.tick < limit && !charge::get(&g.world, HouseId(0), 0).is_some_and(|r| r.ready) {
        g.step();
        tally.observe(&g.world);
    }
    assert!(charge::get(&g.world, HouseId(0), 0).is_some_and(|r| r.ready));
    assert_eq!(tally.launches, 0);
    // The fire order a person gives, at the enemy's start.
    let enemy = fixture::at(fixture::START_DX, 0);
    assert!(g.issue(HouseId(0), Order::FireSuperweapon { sw: 0, cell: enemy }));
    for _ in 0..3 {
        g.step();
        tally.observe(&g.world);
    }
    assert_eq!(tally.launches, 1, "{tally:?}");
    assert_eq!(tally.launches_by_kind.get("Missile"), Some(&1));
}
