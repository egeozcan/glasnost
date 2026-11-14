//! S-0206 claim 9 (sim batch S13, #260): the seized exception belongs to
//! the movement-destination rest row only. A seized vehicle with no
//! movement destination (dropped, its grip ended, S-0607 claims 13 and 20)
//! rests its turret on its body's current facing. Lift fixture of
//! `lift_common` (synthetic data).

#[path = "lift_common/mod.rs"]
mod lift_common;

use glasnost_sim::lift;
use glasnost_sim::rules::Kind;
use glasnost_sim::turning::Turning;
use glasnost_sim::world::Target;
use lift_common::*;

// spec: S-0206 claim 9 (the no-destination rest row gives the body's current facing; the seized exception is the destination row's) and scenario R9 run e (a seized vehicle with no destination tests the no-destination rest row)
#[test]
fn a_seized_vehicle_without_destination_rests_on_its_body() {
    let mut w = world(|r| {
        r.sim.vehicles.get_mut("VICT").unwrap().turret = Some(true);
    });
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    beam(&mut w, l, v);
    assert!(lift::seized(&w, v));
    assert!(
        run_until(&mut w, 400, |w| w
            .obj(v)
            .is_some_and(|o| o.pos.cell() == at(3, 0))
            && height(w, v) > 300),
        "the victim did not pass over the cell"
    );
    // The lifter is ordered on: the grip ends and the victim is dropped.
    glasnost_sim::econ::assign_destination(&mut w, l, Some(Target::Cell(at(0, 1))));
    assert!(lift::dropped(&w, v), "dropped");
    glasnost_sim::objects::remove(&mut w, l);

    // The turret points a quarter turn away from the body, at rest.
    let t0 = w.tick;
    let body = w.obj(v).unwrap().body.value(t0);
    w.obj_mut(v).unwrap().turret = Turning::at(body.wrapping_add(16384));

    glasnost_sim::tick::step(&mut w);

    let g = w.rules().general.guard_area_targeting_delay;
    let o = w.obj(v).unwrap();
    assert!(lift::seized(&w, v), "still seized in its fall");
    assert_eq!(o.target, None, "no target");
    assert_eq!(o.dest, None, "no movement destination");
    assert!(t0 - o.last_fired >= g + 5, "G + 5 ticks since the last shot");
    assert_eq!(o.body.value(t0), body, "the body did not turn");
    assert_eq!(
        o.turret.target, body,
        "the no-destination rest row gives the body's current facing"
    );
}
