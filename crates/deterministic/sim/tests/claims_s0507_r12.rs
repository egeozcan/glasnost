//! Claim tests of S-0507 revision 12 (Q-3236 answered): claim 6a for a
//! jumpjet holder of an object that goes inside, on the synthetic jumpjet
//! fixtures of `jump_common`. Every name and number is made up; no game
//! data.

mod jump_common;

use glasnost_sim::econ::assign_destination;
use glasnost_sim::objects;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::Target;
use jump_common::{game, put, step};

// spec: S-0507 claim 6a (r12, Q-3236: jumpjet movement goes on with the movement point S-0502 claim 16 gave it at the assignment; the loss is not an effective clear of S-0502 claim 24, so it runs no reroute of claim 17 and leaves the movement point and the movement commitment as they were) and S-0502 claim 24
#[test]
fn a_jumpjet_holder_keeps_its_point_and_commitment() {
    let mut w = game();
    let v = put(&mut w, "JV", Kind::Vehicle, 0, 0, 0);
    let t = put(&mut w, "JV", Kind::Vehicle, 0, 10, 0);
    assign_destination(&mut w, v, Some(Target::Object(t)));
    step(&mut w);
    step(&mut w);
    let j = w.obj(v).unwrap().jump.clone().unwrap();
    assert!(j.moving && j.dest.is_some(), "void: no movement commitment");
    objects::go_inside(&mut w, t);
    let o = w.obj(v).unwrap();
    assert_eq!(o.dest, None, "the holder lost it");
    let k = o.jump.as_ref().unwrap();
    assert_eq!(
        (k.moving, k.dest, k.phase),
        (j.moving, j.dest, j.phase),
        "no reroute at the loss"
    );
    step(&mut w);
    let k = w.obj(v).unwrap().jump.clone().unwrap();
    assert!(k.moving, "the commitment stays");
    assert_eq!(k.dest, j.dest, "the point stays");
}
