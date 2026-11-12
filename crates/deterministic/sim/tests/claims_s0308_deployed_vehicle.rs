//! S-0308 claim 9 (#177): a deployed vehicle whose type has the deploy-fire
//! flag uses its type's deploy weapon index against every target. Synthetic
//! data: the S-0517 fixtures (`deploy_common`), every name and number
//! invented.

mod deploy_common;

use deploy_common::*;
use glasnost_sim::combat::weapon;
use glasnost_sim::econ::inplace::{self, Form};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};

/// A game with BOTH: TANK (in place) with the deploy-fire flag, Cannon in
/// slot 0, Gas in slot 1, the deploy weapon index `index` and the
/// land-targeting mode `land`.
fn game_with_both(index: Option<i32>, land: Option<i32>) -> World {
    let mut r = ruleset();
    let mut both = r.sim.vehicles["TANK"].clone();
    both.deploy_fire = Some(true);
    both.secondary = Some("Gas".into());
    both.deploy_fire_weapon = index;
    both.combat.land_targeting = land;
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("BOTH".into());
    r.sim.vehicles.insert("BOTH".into(), both);
    let mut w = testkit::game_with(&r, SIZE, 7);
    testkit::run(&mut w, 4);
    w
}

fn deploy_in_place(w: &mut World, v: ObjectId) {
    deploy(w, v);
    for _ in 0..40 {
        if inplace::form(w, v) == Form::Deployed {
            break;
        }
        step(w);
    }
    assert_eq!(inplace::form(w, v), Form::Deployed, "it deploys in place");
}

// spec: S-0308 claim 9 (a deployed vehicle whose type has the deploy-fire flag uses its type's deploy weapon index against every target; otherwise table P applies) and scenario W8 part c
#[test]
fn claim_9_a_deployed_deploy_fire_vehicle_uses_its_deploy_weapon_index() {
    let mut w = game_with_both(Some(1), None);
    let v = put(&mut w, "BOTH", Kind::Vehicle, 0, 0, 0);
    let e = put(&mut w, "IMGT", Kind::Vehicle, 1, 3, 0);
    let t = Some(Target::Object(e));
    assert_eq!(weapon::choose_slot(&w, v, t), 0, "mobile: table P");
    deploy_in_place(&mut w, v);
    assert_eq!(
        weapon::choose_slot(&w, v, t),
        1,
        "deployed: the deploy weapon index"
    );
    assert_eq!(
        weapon::choose_slot(&w, v, None),
        1,
        "every target, none too"
    );
}

// spec: S-0308 claim 9 and scenario W8 part c (the other direction: deploy weapon index 0 against a target where table P gives 1, row 18)
#[test]
fn claim_9_deploy_weapon_index_0_beats_table_p() {
    let mut w = game_with_both(Some(0), Some(2));
    let v = put(&mut w, "BOTH", Kind::Vehicle, 0, 0, 0);
    let e = put(&mut w, "IMGT", Kind::Vehicle, 1, 3, 0);
    let t = Some(Target::Object(e));
    assert_eq!(weapon::choose_slot(&w, v, t), 1, "mobile: table P row 18");
    deploy_in_place(&mut w, v);
    assert_eq!(
        weapon::choose_slot(&w, v, t),
        0,
        "deployed: the deploy weapon index"
    );
}

// spec: S-0308 Edge cases (an index outside 0 … 17 is treated as 0, a departure)
#[test]
fn claim_9_an_index_outside_the_slots_is_0() {
    let mut w = game_with_both(Some(18), Some(2));
    let v = put(&mut w, "BOTH", Kind::Vehicle, 0, 0, 0);
    let e = put(&mut w, "IMGT", Kind::Vehicle, 1, 3, 0);
    let t = Some(Target::Object(e));
    deploy_in_place(&mut w, v);
    assert_eq!(weapon::choose_slot(&w, v, t), 0);
}
