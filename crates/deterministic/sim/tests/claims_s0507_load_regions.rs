//! Claim test for S-0507 claim 5 on synthetic data: the loading reply of
//! an idle transport compares reach regions brought up to date after a
//! category change earlier in the tick (S-0212 claim 5).

mod common;

use common::{FOOT, Scene, WHEEL, cell, objects_of, open_orders, unit_type};
use glasnost_rules::sim::{Decimal, TerrainObjectType};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, ClickOn, Modifiers};
use glasnost_sim::world::HouseId;
use glasnost_sim::{orders, tick, transport};

const HAULER: &str = "TESTHAULER";
const MAN: &str = "TESTMAN";
const TREE: &str = "TESTTREE";

// spec: S-0507 claim 5 (no destination: the transport takes the requester as its destination when their cells have different reach regions for its zone) and S-0212 claim 5 (the regions the map content gives, a change earlier in the same tick included)
#[test]
fn claim_5_loading_reply_reads_regions_after_a_category_change() {
    let mut s = Scene::new();
    let mut hauler = unit_type(WHEEL, 6, 8);
    hauler.passengers = Some(4);
    hauler.size_limit = Some(Decimal("2".into()));
    s.vehicle(HAULER, hauler);
    s.infantry(MAN, unit_type(FOOT, 4, 8));
    s.register("terrain_objects", TREE);
    s.ruleset
        .sim
        .terrain_objects
        .insert(TREE.into(), TerrainObjectType::default());
    // A row of terrain objects far from the two units.
    for i in 0..30 {
        s.place_terrain(TREE, cell(-15 + i, 12));
    }
    s.place_unit(0, HAULER, cell(0, 0), 0);
    s.place_infantry(0, MAN, cell(4, 0), 1);
    let mut w = s.build();
    open_orders(&mut w);
    let t = objects_of(&w, Kind::Vehicle)[0];
    let p = objects_of(&w, Kind::Infantry)[0];
    let trees = objects_of(&w, Kind::Terrain);
    let r = targeting::click(
        &w,
        HouseId(0),
        &[p],
        ClickOn::Object(t),
        Modifiers::default(),
    );
    assert!(!r.orders.is_empty(), "the click gives an enter order");
    for o in r.orders {
        assert!(orders::issue(&mut w, HouseId(0), o));
    }
    let mut seen_enter = false;
    for &tree in &trees {
        if transport::riding(&w, p) == Some(t) {
            break;
        }
        seen_enter |= w
            .obj(p)
            .is_some_and(|o| o.beh.current == Some(Behaviour::Enter));
        // A category change before the tick stands for one earlier in it.
        glasnost_sim::objects::remove(&mut w, tree);
        assert!(w.regions.dirty, "the removal changed a category");
        tick::step(&mut w);
    }
    assert!(seen_enter, "the passenger was in enter");
}
