//! Claim tests for jumpjets in flight in combat: a jumpjet between take-off
//! and landing is a tracked flying object that the above-terrain detonations
//! of S-0303 claim 6 hit (S-0303 Terms; S-0502 claims 5 and 15; Q-1500), and
//! its in-the-air answer is S-0307's height test (S-0502 Terms, *Airborne*;
//! S-0308 table P row 19; Q-1501). Synthetic data only.

mod jump_common;

use glasnost_sim::combat::area::{self, Detonation};
use glasnost_sim::combat::{flying, weapon};
use glasnost_sim::jump::{self, Phase};
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, ObjectId, World};
use jump_common::*;

fn he(w: &World) -> u32 {
    w.rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == "HE")
        .expect("the synthetic HE warhead") as u32
}

/// A detonation of the synthetic HE warhead at the unit's position.
fn blast_at(w: &mut World, u: ObjectId) {
    let point = pos(w, u);
    let warhead = Some(he(w));
    area::detonate(
        w,
        Detonation {
            point,
            damage: 50,
            warhead,
            source: None,
            house: Some(HouseId(1)),
            resources: false,
        },
    );
}

// spec: S-0303 Terms and claim 6 (a jumpjet between lift-off and landing is a tracked flying object; a detonation above the terrain considers it); S-0502 claim 4 (above 208 leptons it is not listed in a cell)
#[test]
fn a_jumpjet_in_flight_is_hit_through_the_flying_object_search() {
    let mut w = game();
    let u = put(&mut w, "JV", Kind::Vehicle, 0, 0, 0);
    assert!(!flying::tracked(&w, u), "a grounded jumpjet is not tracked");
    jump::start_in_flight(&mut w, u, Phase::Hovering, 500);
    assert!(flying::tracked(&w, u), "a jumpjet in the air is tracked");
    let c = pos(&w, u).cell();
    assert!(
        !w.cells
            .list(c, glasnost_sim::world::Layer::Ground)
            .contains(&u),
        "at 500 leptons it is not listed in its cell (S-0502 claim 4)"
    );
    let before = w.obj(u).unwrap().health;
    blast_at(&mut w, u);
    assert!(
        w.obj(u).unwrap().health < before,
        "the detonation at its position hits it"
    );
}

// spec: S-0502 claims 5 and 15 (take-off and touchdown are the boundaries of the tracking; Q-1500)
#[test]
fn a_jumpjet_is_tracked_from_take_off_until_touchdown() {
    let mut w = game();
    let u = put(&mut w, "JV", Kind::Vehicle, 0, 0, 0);
    go(&mut w, u, at(4, 0));
    assert!(
        until(&mut w, 60, |w| flying::tracked(w, u)),
        "it is tracked once it takes off"
    );
    assert_ne!(phase(&w, u), Phase::Grounded);
    assert!(
        until(&mut w, 2000, |w| phase(w, u) == Phase::Grounded),
        "it lands"
    );
    assert!(!flying::tracked(&w, u), "it is not tracked after touchdown");
}

// spec: S-0502 Terms (Airborne: a jumpjet counts as in the air by S-0307's height test, at least 208 leptons above the surface; Q-1501)
#[test]
fn a_jumpjet_is_in_the_air_by_the_height_test() {
    let mut w = game();
    let u = put(&mut w, "JV", Kind::Vehicle, 0, 0, 0);
    assert!(!weapon::in_air(&w, u), "on the ground it is not in the air");
    jump::start_in_flight(&mut w, u, Phase::Hovering, 500);
    assert!(weapon::in_air(&w, u), "at 500 leptons it is in the air");
    let surface = pos(&w, u).z - height(&w, u);
    w.obj_mut(u).unwrap().pos.z = surface + 207;
    assert!(!weapon::in_air(&w, u), "at 207 leptons it is not");
    w.obj_mut(u).unwrap().pos.z = surface + 208;
    assert!(weapon::in_air(&w, u), "at 208 leptons it is");
}
