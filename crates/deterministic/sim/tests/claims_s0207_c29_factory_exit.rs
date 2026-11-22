//! Tests of our interim for Q-3374 on S-0207 claim 29 (r92) and S-0417
//! claim 5, through a factory: a vehicle released by a ground vehicle
//! factory and given a cell destination while it is still tethered, before
//! the factory puts it on the exit track, drives on to that destination
//! after the exit. The econ test kit's WARFACT and TANK; every name and
//! number is made up, no game data.

mod econ_common;

use econ_common::*;
use glasnost_map::CellXY;
use glasnost_sim::econ::{self, release};
use glasnost_sim::objects;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, Pos, Target};

// design: sim (Q-3374) and S-0417 claims 5 and 16 (a vehicle sent out with a destination it already held drives on to it)
#[test]
fn q3374_a_vehicle_given_a_destination_before_its_exit_leaves_the_apron() {
    let mut w = game(&ruleset());
    ready(&mut w);
    let k = building(&mut w, "WARFACT", 0, 0, 0);
    let ti = w.rules().index_of(Kind::Vehicle, "TANK").unwrap();
    let v = objects::create(&mut w, Kind::Vehicle, ti, Some(HouseId(0)));
    assert_eq!(release::release(&mut w, k, v), release::Outcome::Released);
    assert!(release::tethered(&w, v));
    let e = release::apron(&w, k).expect("an apron");
    let kc = w.obj(k).unwrap().pos.cell();
    let d = CellXY::new(kc.x - 7, kc.y + 6);
    econ::assign_destination(&mut w, v, Some(Target::Cell(d)));
    let mut exit_seen = false;
    for _ in 0..1500 {
        step(&mut w);
        let o = w.obj(v).unwrap();
        let on_exit = o
            .drive
            .as_ref()
            .is_some_and(|dd| dd.track.as_ref().is_some_and(|t| t.exit));
        if on_exit && !exit_seen {
            exit_seen = true;
            // spec: S-0417 claim 5 (E's point at height 0, the order-level destination kept)
            assert_eq!(
                glasnost_sim::drive::goal(&w, v),
                Some(Pos::new(256 * e.x + 128, 256 * e.y + 128, 0))
            );
            assert_eq!(o.dest, Some(Target::Cell(d)), "D kept on the exit");
        }
        if exit_seen && o.pos.cell() == d {
            return;
        }
    }
    assert!(exit_seen, "the factory sent it out");
    panic!(
        "it never reached D; it stands in {:?} (E is {e:?})",
        w.obj(v).unwrap().pos.cell()
    );
}
