//! Claim tests of S-0207 claim 29 (r92, Q-3353), its readings of the
//! heading point: S-0402 claim 8's connection test reads the harvester's
//! heading point as S-0207 Terms gives it, the target of the track it is
//! on, or its own position with none, and never the point of its
//! destination. Synthetic data of the harvest tests: invented refineries,
//! harvesters and cells, no stock data.

mod harvest_common;

use glasnost_map::CellXY;
use glasnost_sim::econ::harvest;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::Behaviour;
use glasnost_sim::world::{HouseId, Target};
use harvest_common::*;

// spec: S-0207 claim 29 (r92, Q-3353: S-0402 claim 8's connection test reads the heading point of S-0207 Terms, the track's target, or the vehicle's own position when it has none; no point of the destination is taken) and S-0402 claim 8
#[test]
fn claim_29_the_connection_test_reads_the_track_target_not_the_destination() {
    let r = ruleset();
    let west = at(-2, 0).x;
    let mut w = game_with_map(&r, &[], 7, |m| {
        for y in 0..=2 * SIZE {
            for x in 0..=2 * SIZE {
                if let Some(c) = m.cells.get_mut(CellXY::new(x, y))
                    && x < west
                {
                    c.level = 4;
                }
            }
        }
    });
    let near = place_building(&mut w, "REFN", 0, at(-6, -1));
    let far = place_building(&mut w, "REFN", 0, at(8, -1));
    let h = place_vehicle(&mut w, "HARV", 0, at(0, 0), 0);
    while orders::blocked(&w) {
        glasnost_sim::tick::step(&mut w);
    }
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: h,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(at(4, 0))),
        },
    ));
    orders::apply_pending(&mut w);
    let mut n = 0;
    while w
        .obj(h)
        .and_then(|o| o.drive.as_ref())
        .is_none_or(|d| d.track.is_none())
    {
        glasnost_sim::tick::step(&mut w);
        n += 1;
        assert!(n < 100, "a track begins");
    }
    let track = w.obj(h).unwrap().drive.as_ref().unwrap().track.clone();
    let target = track.unwrap().target_cell;
    assert!(target.x >= west, "the track's target is on the low ground");
    // The destination is moved onto the plateau with no assignment, so the
    // destination's point and the track's target lie in different regions.
    w.obj_mut(h).unwrap().dest = Some(Target::Cell(at(-8, 3)));
    assert_eq!(
        harvest::choose_refinery(&mut w, h, false),
        Some(far),
        "connected from the track's target, not from the destination ({near:?} is on the plateau)"
    );
}
