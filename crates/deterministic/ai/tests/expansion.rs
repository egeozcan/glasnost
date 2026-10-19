//! The own AI expands toward a far resource field with a second
//! construction yard (M7b AI round 3), on synthetic data.
//!
//! - Deploying a yard vehicle gives a construction yard wherever the
//!   deploy's cell test passes, with no adjacency to the base (S-0415
//!   claim 3, S-0406 claim 4).
//! - A rich house with its economy kept builds one yard vehicle beyond its
//!   yard when an explored field lies beyond the reach of every yard on its
//!   half of the map, drives it to that field and deploys it there; a
//!   poor house, or one with no far field, builds none (tuning of our own
//!   opponent).

use glasnost_ai::brain::Brain;
use glasnost_ai::{fixture, game::Game};
use glasnost_map::{CellXY, Map};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

const H: HouseId = HouseId(0);

/// The far field: 24 cells north-east of start 0, on its half of the
/// map.
fn far() -> CellXY {
    fixture::at(-8, -20)
}

/// [`fixture::map`] with a field of 5 × 5 ore cells around [`far`].
fn far_field_map() -> Map {
    let mut m = fixture::map();
    let f = far();
    for ox in -2..=2 {
        for oy in -2..=2 {
            if let Some(cell) = m.cells.get_mut(CellXY::new(f.x + ox, f.y + oy)) {
                cell.overlay = ((ox + oy + 4) % 12) as u8;
                cell.overlay_data = 6;
            }
        }
    }
    m
}

fn name(w: &World, id: ObjectId) -> Option<String> {
    let o = w.obj(id)?;
    let r = w.rules();
    match o.kind {
        Kind::Building => r
            .buildings
            .get(o.type_index as usize)
            .map(|b| b.name.clone()),
        Kind::Vehicle => r
            .vehicles
            .get(o.type_index as usize)
            .map(|u| u.name.clone()),
        _ => None,
    }
}

/// House 0's live objects on the map named `n`.
fn of(w: &World, n: &str) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| o.owner == Some(H) && o.on_map && o.health > 0)
        .map(|o| o.id)
        .filter(|&id| name(w, id).as_deref() == Some(n))
        .collect()
}

fn dist2(a: CellXY, b: CellXY) -> i64 {
    let (dx, dy) = (i64::from(a.x - b.x), i64::from(a.y - b.y));
    dx * dx + dy * dy
}

/// Whether no object stands on cell `c`.
fn free(w: &World, c: CellXY) -> bool {
    w.objects.values().all(|o| !o.on_map || o.pos.cell() != c)
}

/// A skirmish on `map` once house 0's weapons factory and two harvesters
/// (the plan's) stand.
fn built_up(map: &Map) -> Game {
    let mut g = Game::new(
        fixture::rules(),
        map,
        "synthetic-expansion",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the skirmish loads");
    while g.world.tick < 9000 && (of(&g.world, "FACT").is_empty() || of(&g.world, "MINR").len() < 2)
    {
        g.step();
    }
    assert!(
        !of(&g.world, "FACT").is_empty() && of(&g.world, "MINR").len() >= 2,
        "a weapons factory and two harvesters by tick 9000"
    );
    g
}

/// House 0 scouts the far field.
fn scout(w: &mut World) {
    glasnost_sim::econ::shroud::area_reveal(w, H, Pos::centre(far(), 0), 6);
}

/// House 0's orders at the next think tick, with the expansion builds and
/// moves counted.
fn think(w: &mut World, map: &Map) -> (Vec<Order>, u32, u32) {
    testkit::run(w, 1);
    while w.tick % glasnost_ai::scripted::THINK_PERIOD != 0 {
        testkit::run(w, 1);
    }
    let mut brain = Brain::with_plan(w, map, H, fixture::plan());
    let orders = brain.think(w);
    let u = brain.special_uses();
    (orders, u.expansion_builds, u.expansion_moves)
}

fn builds_mcv(w: &World, orders: &[Order]) -> bool {
    let mcv = w.rules().index_of(Kind::Vehicle, "MCV").unwrap();
    orders.iter().any(
        |o| matches!(o, Order::Build { what } if what.kind == Kind::Vehicle && what.index == mcv),
    )
}

// S-0415 claim 3 and S-0406 claim 4 (a deployed yard needs no adjacency) and S-0403 claim 6 (a second building factory): a rich house that has scouted a far field builds a yard vehicle.
#[test]
fn a_rich_house_with_a_far_field_builds_a_yard_vehicle() {
    let map = far_field_map();
    let g = built_up(&map);
    let mut w = g.world.clone();
    scout(&mut w);
    w.houses[0].credits = 100_000;
    assert!(w.cells.usable(far()) && glasnost_sim::econ::shroud::explored(&w, H, far()));
    let (orders, builds, _) = think(&mut w, &map);
    assert!(builds_mcv(&w, &orders), "{orders:?}");
    assert_eq!(builds, 1);
    // Without the far field scouted, none.
    let mut w = g.world.clone();
    w.houses[0].credits = 100_000;
    let (orders, builds, _) = think(&mut w, &map);
    assert!(!builds_mcv(&w, &orders), "{orders:?}");
    assert_eq!(builds, 0);
}

// Tuning of our own opponent: a house with less than twice a yard vehicle's cost in hand builds none to expand.
#[test]
fn a_poor_house_builds_no_yard_vehicle() {
    let map = far_field_map();
    let g = built_up(&map);
    let mut w = g.world.clone();
    scout(&mut w);
    let mcv = glasnost_sim::econ::TypeRef {
        kind: Kind::Vehicle,
        index: w.rules().index_of(Kind::Vehicle, "MCV").unwrap(),
    };
    w.houses[0].credits = 2 * glasnost_sim::econ::production::cost_field(&w, mcv) - 1;
    let (orders, builds, _) = think(&mut w, &map);
    assert!(!builds_mcv(&w, &orders), "{orders:?}");
    assert_eq!(builds, 0);
}

// S-0415 claims 1 to 3 (the deploy order and its cell test, no adjacency): a spare yard vehicle is driven to the far field and deployed there; no second one is built.
#[test]
#[ignore = "long in debug builds; run in release (`cargo test --release -p glasnost-ai -- --include-ignored`)"]
fn drives_a_yard_vehicle_to_the_far_field_and_deploys_it() {
    let map = far_field_map();
    let mut g = built_up(&map);
    scout(&mut g.world);
    g.world.houses[0].credits = 100_000;
    let yard = g.world.obj(of(&g.world, "YARD")[0]).unwrap().pos.cell();
    let spare = (2..12)
        .map(|d| CellXY::new(yard.x + d, yard.y - 4))
        .find(|&c| g.world.cells.usable(c) && free(&g.world, c))
        .expect("a free cell by the yard");
    let mcv = testkit::spawn(&mut g.world, Kind::Vehicle, "MCV", 0, spare, 0);
    let mut w = g.world.clone();
    let (orders, builds, moves) = think(&mut w, &map);
    assert_eq!(builds, 0, "the spare counts toward the second yard");
    assert_eq!(moves, 1);
    let dest = orders.iter().find_map(|o| match o {
        Order::Behaviour {
            actor,
            behaviour: Behaviour::Move,
            dest: Some(Target::Cell(c)),
            ..
        } if *actor == mcv => Some(*c),
        _ => None,
    });
    let dest = dest.expect("a move order for the spare");
    // The spiral of cells a person looks at around the field (12 rings).
    let reach = 12;
    assert!(
        dist2(dest, far()) <= 2 * reach * reach,
        "{dest:?} near the field {:?}",
        far()
    );
    let start = g.world.tick;
    while g.world.tick < start + 6000 && of(&g.world, "YARD").len() < 2 {
        g.step();
    }
    let yards: Vec<CellXY> = of(&g.world, "YARD")
        .into_iter()
        .filter_map(|id| g.world.obj(id))
        .map(|o| o.pos.cell())
        .collect();
    assert_eq!(
        yards.len(),
        2,
        "the spare deployed by tick {}",
        g.world.tick
    );
    // The reach the house counts harvesters in (14 cells).
    let r = 14;
    assert!(
        yards.iter().any(|&y| dist2(y, far()) <= r * r),
        "a yard by the far field: {yards:?}"
    );
    assert!(of(&g.world, "MCV").is_empty());
}
