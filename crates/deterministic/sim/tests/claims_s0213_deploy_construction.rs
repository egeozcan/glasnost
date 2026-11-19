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
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

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

// spec: S-0213 claim 10 (the deploy order's actor must not be under construction) and S-0406 claim 17 (the construction state, ended by the build-up's handover; r19: for the building a place order names it resumes at C, which ends the factory break-off window, with construction current) (#299, #440)
#[test]
fn deploy_order_to_a_building_under_construction() {
    let r = rules();

    // Control: no order. The construction state ends.
    let (mut w, id) = placed(&r);
    testkit::run(&mut w, 300);
    assert!(!w.obj(id).unwrap().econ.construction.building_up);

    // A deploy order in the construction state: from C on, after the
    // factory break-off window (S-0406 claim 17, r19), construction current.
    let (mut w, id) = placed(&r);
    for _ in 0..20 {
        if w.obj(id).unwrap().econ.construction.first.is_some() {
            break;
        }
        glasnost_sim::tick::step(&mut w);
    }
    let o = w.obj(id).unwrap();
    assert!(o.econ.construction.first.is_some(), "C");
    assert!(placement::in_construction_state(o));
    assert_eq!(o.beh.current, Some(Behaviour::Construction));
    assert!(orders::issue(&mut w, A, Order::Deploy { actor: id }));
    glasnost_sim::tick::step(&mut w);
    let o = w.obj(id).unwrap();
    let after = (o.beh.current, o.beh.queued);
    testkit::run(&mut w, 300);
    let o = w.obj(id).unwrap();
    assert_eq!(
        after,
        (Some(Behaviour::Construction), None),
        "the deploy order leaves the construction alone"
    );
    assert!(
        !o.econ.construction.building_up,
        "the construction state ends as without the order"
    );
}

/// BIG placed as a place order puts the building it names (before the
/// turn pass of its first turn *E*), given `order` before *E* and run
/// through *E*. Orders take effect after the turn pass (S-0203 claim 7), so
/// the order takes effect in *E*'s orders part, in the factory break-off
/// window (S-0406 claim 17), which has begun at the end of BIG's update in
/// *E*. `buildup`: BIG with its build-up sprite (n = 5, a positive delay),
/// whose construction range does not set the mark on *E*, so
/// *construction* is still queued; without one its zero-delay construction
/// range sets the mark on *E* and *construction* is current from *E*'s
/// second start. Returns the world, BIG and *E*.
fn in_the_window(buildup: bool, before: fn(&mut World, ObjectId)) -> (World, ObjectId, i32) {
    let mut r = rules();
    if !buildup {
        r.sim.buildings.get_mut("BIG").unwrap().econ.buildup = Some(false);
    }
    let (mut w, id) = placed(&r);
    before(&mut w, id);
    let e = w.tick;
    glasnost_sim::tick::step(&mut w);
    let o = w.obj(id).unwrap();
    assert!(
        o.econ.idle_body.is_some(),
        "in the factory break-off window"
    );
    assert!(o.econ.construction.building_up);
    assert_eq!(o.econ.construction.first, None);
    let current = if buildup {
        None
    } else {
        Some(Behaviour::Construction)
    };
    assert_eq!(o.beh.current, current);
    (w, id, e)
}

// spec: S-0213 claim 10 and S-0406 claim 17 and Changes for other specs (r19, Q-3243: in its factory break-off window, with construction queued, the building a place order names is outside its construction state and claim 10 applies as it reads: the deploy order queues unload in place of the queued construction; with construction current there it refuses the order, as in its construction state) (#440)
#[test]
fn deploy_order_in_the_factory_break_off_window() {
    let deploy: fn(&mut World, ObjectId) = |w, actor| {
        assert!(orders::issue(w, A, Order::Deploy { actor }));
    };
    let (w, id, _) = in_the_window(true, deploy);
    let o = w.obj(id).unwrap();
    assert_eq!(placement::seen_behaviour(o), Some(Behaviour::Unload));
    assert_eq!(o.econ.construction.first, None, "no first processing");
    let (mut w, id, e) = in_the_window(false, deploy);
    glasnost_sim::tick::step(&mut w);
    let o = w.obj(id).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Construction), "refused");
    // spec: S-0406 claim 17 (construction current in the window: C on the next turn)
    assert_eq!(o.econ.construction.first, Some(e + 1));
}

// spec: S-0213 claim 8, S-0215 claim 12 and S-0406 claim 17 and Changes for other specs (r19, Q-3243: in its factory break-off window, with construction queued, the building a place order names is outside its construction state and the stop order applies: its target becomes nothing; with construction current there the stop order does nothing, as in its construction state) (#440)
#[test]
fn stop_order_in_the_factory_break_off_window() {
    for buildup in [true, false] {
        let (w, id, _) = in_the_window(buildup, |w, id| {
            let o = w.obj_mut(id).unwrap();
            o.target = Some(Target::Cell(c(5, 5)));
            o.combat.opportunity = false;
            assert!(orders::issue(w, A, Order::Stop { actor: id }));
        });
        let kept = w.obj(id).unwrap().target;
        if buildup {
            assert_eq!(kept, None, "applied");
        } else {
            assert_eq!(kept, Some(Target::Cell(c(5, 5))), "refused");
        }
    }
}
