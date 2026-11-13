//! Claim test for S-0502 claim 4 and S-0209 claim 1 (#188): removing a
//! jumpjet vehicle that is hovering high releases no vehicle-present claim,
//! so a vehicle standing in the cell under it keeps its own. Synthetic rules
//! (`jump_common`: DEPN is a jumpjet vehicle without the crash flag, CART a
//! driving vehicle).

mod jump_common;

use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::jump::{self, Phase};
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{Layer, ObjectId, World};
use jump_common::*;

fn kill(w: &mut World, id: ObjectId) {
    let health = w.obj(id).unwrap().health;
    damage::apply(
        w,
        id,
        Hit {
            amount: health * 10,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
}

fn vehicles(w: &World, dx: i32, dy: i32) -> u8 {
    w.cells.occ(at(dx, dy), Layer::Ground).unwrap().vehicles
}

// spec: S-0502 claim 4 (hovering and cruising physics make no actual-position mark: a hovering jumpjet holds none) and S-0209 claim 1 (a vehicle at rest keeps the flag of its cell)
#[test]
fn removing_a_hovering_jumpjet_vehicle_keeps_the_flag_of_the_vehicle_below() {
    let mut w = game();
    // A jumpjet vehicle without the crash flag, hovering 500 leptons up: its mark is lifted.
    let u = put(&mut w, "DEPN", Kind::Vehicle, 0, 0, 0);
    jump::start_in_flight(&mut w, u, Phase::Hovering, 500);
    assert_eq!(vehicles(&w, 0, 0), 0, "the hovering jumpjet holds no claim");
    assert!(w.obj(u).unwrap().jump.as_ref().unwrap().mark.is_none());
    // A driving vehicle is put in the cell below and holds the cell's claim.
    let tank = put(&mut w, "CART", Kind::Vehicle, 0, 0, 0);
    assert_eq!(vehicles(&w, 0, 0), 1, "the tank's claim");
    for _ in 0..5 {
        step(&mut w);
    }
    assert_eq!(vehicles(&w, 0, 0), 1, "hovering changes nothing");
    assert_eq!(phase(&w, u), Phase::Hovering);
    assert_eq!(w.obj(u).unwrap().pos.cell(), at(0, 0));
    assert!(w.obj(u).unwrap().jump.as_ref().unwrap().mark.is_none());
    kill(&mut w, u);
    assert!(w.obj(u).is_none(), "the jumpjet left play");
    let t = w.obj(tank).expect("the tank is alive");
    assert_eq!(t.pos.cell(), at(0, 0));
    assert_eq!(
        t.drive.as_ref().unwrap().claims,
        vec![(at(0, 0), Layer::Ground)],
        "the tank still records its claim"
    );
    assert_eq!(
        vehicles(&w, 0, 0),
        1,
        "the tank's vehicle-present claim survives the jumpjet's removal"
    );
}
