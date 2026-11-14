//! S-0212 claim 5 and S-0215 claim 37 (a1) (#298): the per-tick approach
//! reads the reach regions as the new map content gives them, even after a
//! category change earlier in the same tick. The regions' own contract
//! (`regions::reach_in`) asserts in a build with debug assertions that no
//! read happens while they are out of date. Synthetic types only (TESTCAR,
//! TESTTREE); every name and number is made up. The rally spread's read is
//! `claims_s0417_rally_fresh.rs`.

mod common;

use common::{Scene, WHEEL, cell, objects_of, unit_type};
use glasnost_map::CellXY;
use glasnost_rules::sim::TerrainObjectType;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{ObjectId, Target, World};

const CAR: &str = "TESTCAR";
const TREE: &str = "TESTTREE";

/// A car at (0, 0) inside a closed ring of terrain objects.
fn world() -> (World, ObjectId) {
    let mut s = Scene::new();
    s.vehicle(CAR, unit_type(WHEEL, 40, 6));
    s.register("terrain_objects", TREE);
    s.ruleset
        .sim
        .terrain_objects
        .insert(TREE.into(), TerrainObjectType::default());
    for dy in -1..=1 {
        for dx in -1..=1 {
            if (dx, dy) != (0, 0) {
                s.place_terrain(TREE, CellXY::new(cell(0, 0).x + dx, cell(0, 0).y + dy));
            }
        }
    }
    s.place_unit(0, CAR, cell(0, 0), 64);
    let w = s.build();
    let car = objects_of(&w, Kind::Vehicle)[0];
    (w, car)
}

// spec: S-0212 claim 5 (a computation sees the reach regions the new map content gives, even one made later in the same tick as the change) and S-0215 claim 37 (a1) (cannot reach: claim 6's comparison of reach regions) (#298)
#[test]
fn per_tick_approach_after_a_category_change() {
    let (mut w, car) = world();
    common::run(&mut w, 2);
    {
        let o = w.obj_mut(car).unwrap();
        o.target = Some(Target::Cell(cell(5, 0)));
        o.dest = None;
    }
    let tree = objects_of(&w, Kind::Terrain)[0];
    glasnost_sim::objects::remove(&mut w, tree);
    assert!(w.regions.dirty, "the removal changed a category");
    glasnost_sim::approach::per_tick(&mut w, car);
    assert!(!w.regions.dirty, "the approach brought the regions up to date");
}

// spec: S-0212 claim 5 (the whole tick after the change: nothing reads the regions out of date) (#298)
#[test]
fn whole_tick_after_a_category_change() {
    let (mut w, car) = world();
    common::run(&mut w, 2);
    {
        let o = w.obj_mut(car).unwrap();
        o.target = Some(Target::Cell(cell(5, 0)));
        o.dest = None;
    }
    let tree = objects_of(&w, Kind::Terrain)[0];
    glasnost_sim::objects::remove(&mut w, tree);
    assert!(w.regions.dirty);
    glasnost_sim::tick::step(&mut w);
}
