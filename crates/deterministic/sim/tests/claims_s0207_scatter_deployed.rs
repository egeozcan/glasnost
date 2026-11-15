//! S-0207 claim 16 (f): a deployed in-place vehicle refuses a scatter
//! request (issue 319). Synthetic data only.

mod deploy_common;

use deploy_common::*;
use glasnost_sim::econ::inplace::{self, Form};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::scatter::{self, ScatterRequest};
use glasnost_sim::world::{ObjectId, Pos, World};

fn deployed() -> (World, ObjectId) {
    let mut w = game();
    let v = put(&mut w, "TANK", Kind::Vehicle, 0, 0, 0);
    deploy(&mut w, v);
    for _ in 0..40 {
        if inplace::form(&w, v) == Form::Deployed {
            break;
        }
        step(&mut w);
    }
    assert_eq!(inplace::form(&w, v), Form::Deployed);
    step(&mut w);
    assert_eq!(behaviour(&w, v), (Some(Behaviour::Guard), None));
    assert_eq!(w.obj(v).unwrap().dest, None);
    (w, v)
}

// spec: S-0207 claim 16 (f) and D18 part g (a deployed vehicle refuses a scatter request, with a threat point and without, and takes no draw)
#[test]
fn issue_319_a_deployed_vehicle_refuses_a_scatter_request() {
    for threat in [true, false] {
        let (mut w, v) = deployed();
        let o = w.obj(v).unwrap().clone();
        let taken = w.rng.taken();
        let accepted = scatter::request(
            &mut w,
            v,
            ScatterRequest {
                threat: threat.then(|| Pos::new(o.pos.x + 256, o.pos.y, o.pos.z)),
                forced: true,
                anywhere: false,
            },
        );
        assert!(!accepted, "threat {threat}: case f refuses");
        assert_eq!(w.rng.taken(), taken, "threat {threat}: no draw is taken");
        assert_eq!(w.obj(v).unwrap().dest, None, "threat {threat}");
        assert_eq!(behaviour(&w, v), (Some(Behaviour::Guard), None));
    }
}
