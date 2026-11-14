//! S-0213 claim 10 (#299): a deploy order is not applied to an actor under
//! construction (S-0406 claim 17: no current behaviour, *construction*
//! queued). Synthetic types (a 2 × 2 building factory YARD and a 2 × 2
//! building BIG with a build-up) on the testkit's sandbox game; every name
//! and number is made up.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, LandRow, LandType};
use glasnost_sim::econ::{TypeRef, placement};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, World};

const SIZE: i32 = 24;
const A: HouseId = HouseId(0);

fn building(w: i32, h: i32) -> BuildingType {
    let mut b = BuildingType {
        strength: Some(400),
        foundation: Some([w, h]),
        owner: Some(vec!["Alphaland".into(), "Betaland".into()]),
        ..Default::default()
    };
    b.econ.buildup = Some(true);
    b.econ.buildup_frames = Some(10);
    b
}

fn rules() -> Ruleset {
    let mut r = testkit::ruleset();
    r.registries
        .insert("buildings".into(), vec!["YARD".into(), "BIG".into()]);
    let s = &mut r.sim;
    s.land_types.insert(
        LandType::Clear,
        LandRow {
            buildable: Some(true),
            ..Default::default()
        },
    );
    let mut yard = building(2, 2);
    yard.factory = Some("building".into());
    s.buildings.insert("YARD".into(), yard);
    s.buildings.insert("BIG".into(), building(2, 2));
    r
}

fn c(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn placed(r: &Ruleset) -> (World, ObjectId) {
    let mut w = testkit::game_with(r, SIZE, 11);
    testkit::run(&mut w, 4);
    glasnost_sim::econ::shroud::area_reveal(&mut w, A, Pos::centre(c(0, 0), 0), 10);
    testkit::spawn(&mut w, Kind::Building, "YARD", 0, c(-6, -6), 0);
    let t = TypeRef {
        kind: Kind::Building,
        index: w.rules().index_of(Kind::Building, "BIG").unwrap(),
    };
    assert!(placement::place_building(&mut w, A, t, c(1, 2)));
    let id = w
        .cells
        .list(c(1, 2), glasnost_sim::world::Layer::Ground)
        .iter()
        .copied()
        .find(|&id| w.obj(id).is_some_and(|o| o.kind == Kind::Building))
        .unwrap();
    (w, id)
}

// spec: S-0213 claim 10 (the deploy order's actor must not be under construction) and S-0406 claim 17 (the construction state: no current behaviour, construction queued, ended by the build-up's handover) (#299)
#[test]
fn deploy_order_to_a_building_under_construction() {
    let r = rules();

    // Control: no order. The construction state ends.
    let (mut w, id) = placed(&r);
    testkit::run(&mut w, 300);
    assert!(!w.obj(id).unwrap().econ.construction.building_up);

    // A deploy order in the construction state.
    let (mut w, id) = placed(&r);
    let o = w.obj(id).unwrap();
    assert_eq!(o.beh.current, None);
    assert_eq!(o.beh.queued, Some(Behaviour::Construction));
    assert!(orders::issue(&mut w, A, Order::Deploy { actor: id }));
    glasnost_sim::tick::step(&mut w);
    let o = w.obj(id).unwrap();
    let after = (o.beh.current, o.beh.queued);
    testkit::run(&mut w, 300);
    let o = w.obj(id).unwrap();
    assert_eq!(
        after,
        (None, Some(Behaviour::Construction)),
        "the deploy order leaves the queued construction alone"
    );
    assert!(
        !o.econ.construction.building_up,
        "the construction state ends as without the order"
    );
}
