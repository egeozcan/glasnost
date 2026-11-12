//! The own AI does not click the same thing again and again when it cannot
//! take (PLAN §4.7: a person who sees a unit fail to go somewhere tries
//! something else). The lead's audit of our AI found orders re-given every
//! tick for a goal the unit can never reach: an attack at an intruder out of
//! reach (#4) and a construction-yard vehicle sent to deploy across water
//! (#5). The tests use the synthetic skirmish on maps with water.

use glasnost_ai::brain::Brain;
use glasnost_ai::{fixture, game::Game};
use glasnost_map::{CellXY, Map};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::regions;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

/// The centre of the lake north of start 0.
fn lake_centre() -> CellXY {
    fixture::at(-fixture::START_DX, -12)
}

/// [`fixture::map`] with a lake north of start 0: the cells within 9 of
/// its centre are water, but for an island of the cells within 1.
fn lake_map() -> Map {
    let mut m = fixture::map();
    let l = lake_centre();
    let cells: Vec<CellXY> = m.cells.iter().map(|(c, _)| c).collect();
    for c in cells {
        let d = (c.x - l.x).abs().max((c.y - l.y).abs());
        if (2..=9).contains(&d)
            && let Some(cell) = m.cells.get_mut(c)
        {
            cell.tile = fixture::WATER_TILE;
            cell.overlay = glasnost_map::NO_OVERLAY;
        }
    }
    m
}

fn is_yard(w: &World, o: &glasnost_sim::world::Object) -> bool {
    o.owner == Some(HouseId(0))
        && o.kind == Kind::Building
        && w.rules().buildings[o.type_index as usize].name == "YARD"
}

/// The lake skirmish once house 0's yard stands.
fn yard_up(map: &Map) -> World {
    let mut g = Game::new(
        fixture::rules(),
        map,
        "synthetic-lake",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the lake skirmish loads");
    let up = |w: &World| w.objects.values().any(|o| is_yard(w, o));
    while g.world.tick < 600 && !up(&g.world) {
        g.step();
    }
    assert!(up(&g.world), "the yard stands");
    g.world.clone()
}

// design: own AI (#9) and S-0212 claim 4 (reach regions): when the seen enemy building nearest home is a naval yard on the water, infantry that can walk to the enemy base is not held at home for a ferry (the yard's cell is in no reach region of the infantry, so every unit seemed to need one).
#[test]
fn a_naval_yard_does_not_keep_the_infantry_home() {
    let map = fixture::map_naval();
    let mut w = yard_up(&map);
    let x = -fixture::START_DX;
    let home = fixture::at(x, 0);
    testkit::spawn(&mut w, Kind::Building, "NAVY", 1, fixture::at(x, -14), 0);
    let walker = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        0,
        fixture::at(x + 20, 0),
        0,
    );
    // The whole map explored and the yard seen, so no unit is sent to scout.
    let spots: Vec<CellXY> = w
        .cells
        .grid
        .iter()
        .map(|(c, _)| c)
        .filter(|c| c.x % 4 == 0 && c.y % 4 == 0)
        .collect();
    for c in spots {
        glasnost_sim::econ::shroud::area_reveal(&mut w, HouseId(0), Pos::centre(c, 0), 4);
    }
    testkit::run(&mut w, 1);
    let mut brain = Brain::with_plan(&w, &map, HouseId(0), fixture::plan());
    while !brain.thinks_at(w.tick) {
        testkit::run(&mut w, 1);
    }
    let orders = brain.think(&w);
    let sent_home = orders.iter().any(|o| {
        matches!(o, Order::Behaviour {
            actor,
            behaviour: Behaviour::Move,
            dest: Some(Target::Cell(c)),
            ..
        } if *actor == walker && (c.x - home.x).pow(2) + (c.y - home.y).pow(2) <= 15 * 15)
    });
    assert!(
        !sent_home,
        "the walker is held home for a ferry: {orders:?}"
    );
}

// design: own AI (#4) and S-0307 claim 17 (an attack with no approach position drops the target): an enemy on an island within the defence radius of the base but out of every reachable cell's weapon range is not attacked again at every tick by the house's army at home.
#[test]
fn an_intruder_out_of_reach_is_not_attacked_every_tick() {
    let map = lake_map();
    let mut w = yard_up(&map);
    let l = lake_centre();
    let enemy = testkit::spawn(&mut w, Kind::Vehicle, "GUNR", 1, l, 0);
    let x = -fixture::START_DX;
    let own: Vec<ObjectId> = [3, 5]
        .iter()
        .map(|&dx| testkit::spawn(&mut w, Kind::Vehicle, "GUNR", 0, fixture::at(x + dx, 5), 0))
        .collect();
    testkit::run(&mut w, 1);
    let mut brain = Brain::with_plan(&w, &map, HouseId(0), fixture::plan());
    let mut attacks = 0;
    for _ in 0..90 {
        glasnost_sim::econ::shroud::area_reveal(&mut w, HouseId(0), Pos::centre(l, 0), 3);
        for o in brain.think(&w) {
            if matches!(&o, Order::Behaviour {
                actor,
                behaviour: Behaviour::Attack,
                target: Some(Target::Object(e)),
                ..
            } if *e == enemy && own.contains(actor))
            {
                attacks += 1;
            }
            orders::issue(&mut w, HouseId(0), o);
        }
        testkit::run(&mut w, 1);
    }
    assert!(w.obj(enemy).is_some(), "the island enemy lives");
    assert!(
        attacks <= own.len(),
        "{attacks} attack orders on the island enemy in 90 ticks"
    );
}

/// [`fixture::map`] with a one-cell island south of start 0's ore field,
/// in a lake three cells wide around it.
fn islet_map() -> Map {
    let mut m = fixture::map();
    let i = islet();
    let cells: Vec<CellXY> = m.cells.iter().map(|(c, _)| c).collect();
    for c in cells {
        let d = (c.x - i.x).abs().max((c.y - i.y).abs());
        if (1..=3).contains(&d)
            && let Some(cell) = m.cells.get_mut(c)
        {
            cell.tile = fixture::WATER_TILE;
            cell.overlay = glasnost_map::NO_OVERLAY;
        }
    }
    m
}

fn islet() -> CellXY {
    fixture::at(-fixture::START_DX, 16)
}

// design: own AI (#5) and S-0212 claim 4 (reach regions): a house with no construction yard whose yard vehicle stands where it cannot deploy, on an island too small for the yard, is never sent to a deploy cell across the water (the move cannot be made, and it was given again at every tick).
#[test]
fn a_yard_vehicle_is_not_sent_across_water_to_deploy() {
    let map = islet_map();
    let mut w = yard_up(&map);
    let yards: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| is_yard(&w, o))
        .map(|o| o.id)
        .collect();
    for id in yards {
        glasnost_sim::objects::remove(&mut w, id);
    }
    let mcv = testkit::spawn(&mut w, Kind::Vehicle, "MCV", 0, islet(), 0);
    testkit::run(&mut w, 1);
    let zone = w
        .rules()
        .unit(Kind::Vehicle, w.obj(mcv).unwrap().type_index)
        .unwrap()
        .zone;
    let here = regions::reach_in(&w, islet(), zone);
    let mut brain = Brain::with_plan(&w, &map, HouseId(0), fixture::plan());
    let mut moves = Vec::new();
    for _ in 0..45 {
        for o in brain.think(&w) {
            if let Order::Behaviour {
                actor,
                behaviour: Behaviour::Move,
                dest: Some(Target::Cell(c)),
                ..
            } = &o
                && *actor == mcv
            {
                moves.push(*c);
            }
            orders::issue(&mut w, HouseId(0), o);
        }
        testkit::run(&mut w, 1);
    }
    assert!(
        moves
            .iter()
            .all(|&c| regions::reach_in(&w, c, zone) == here),
        "moves across the water: {moves:?}"
    );
}
