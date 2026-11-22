//! Tests of S-0207 claim 31 (r103, Q-3374) and S-0417 claim 5, through a
//! factory: a vehicle released by a ground vehicle factory and given a cell
//! destination while it is still tethered, before the factory puts it on
//! the exit track, clears the exit point in the tick the exit track ends,
//! installs its destination's point at the next update and searches for it
//! R ticks after the exit ended, then drives on to it. The econ test kit's
//! WARFACT and TANK; every name and number is made up, no game data.

mod econ_common;

use econ_common::*;
use glasnost_map::CellXY;
use glasnost_sim::econ::{self, release};
use glasnost_sim::objects;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, Pos, Target};

// spec: S-0207 claim 31 (r103, Q-3374: in the update in which the exit track ends the vehicle searches for the apron's cell and the exit point is cleared; a destination held from before has its point installed at the next update, claim 30, and the vehicle searches for it R ticks after the exit track ended) and S-0417 claims 5 and 16
#[test]
fn claim_31_a_vehicle_given_a_destination_before_its_exit_leaves_the_apron() {
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
    let r = w.rules().general.retry_delay;
    assert!(r > 0);
    let on_exit = |w: &glasnost_sim::world::World| {
        w.obj(v).unwrap().drive.as_ref().is_some_and(|dd| {
            dd.track.as_ref().is_some_and(|t| t.exit)
        })
    };
    let mut exit_seen = false;
    let mut ended = None;
    let mut next_track = None;
    for _ in 0..1500 {
        let was_on_exit = on_exit(&w);
        let had_track = w.obj(v).unwrap().drive.as_ref().unwrap().track.is_some();
        step(&mut w);
        let t = w.tick - 1;
        let o = w.obj(v).unwrap();
        if on_exit(&w) && !exit_seen {
            exit_seen = true;
            // spec: S-0417 claim 5 (E's point at height 0, the order-level destination kept)
            assert_eq!(
                glasnost_sim::drive::goal(&w, v),
                Some(Pos::new(256 * e.x + 128, 256 * e.y + 128, 0))
            );
            assert_eq!(o.dest, Some(Target::Cell(d)), "D kept on the exit");
        }
        if was_on_exit && !on_exit(&w) && ended.is_none() {
            ended = Some(t);
            assert_eq!(o.pos.cell(), e, "the exit track ends at E");
            assert_eq!(glasnost_sim::drive::goal(&w, v), None, "E's point cleared");
            assert_eq!(o.dest, Some(Target::Cell(d)), "D kept");
            assert_eq!(o.path.retry_at, t + r, "R starts at the exit's end");
        } else if ended == Some(t - 1) {
            let p = glasnost_sim::drive::goal(&w, v).expect("D's point installed");
            assert_eq!(p.cell(), d);
        }
        if ended.is_some() && next_track.is_none() && !had_track {
            if o.drive.as_ref().unwrap().track.is_some() {
                next_track = Some(t);
            }
        }
        if exit_seen && o.pos.cell() == d {
            let (end, next) = (ended.unwrap(), next_track.expect("a track after the exit"));
            assert!(next >= end + r, "the search waits R: {next} < {end} + {r}");
            return;
        }
    }
    assert!(exit_seen, "the factory sent it out");
    panic!(
        "it never reached D; it stands in {:?} (E is {e:?})",
        w.obj(v).unwrap().pos.cell()
    );
}
