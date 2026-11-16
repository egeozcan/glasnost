//! S-0207 revision 85 claim 20 (a) for a teleporting vehicle in a
//! superweapon warp's leaving stage (sim issues S24, seam d): the leaving
//! stage counts as being erased (glossary *leaving stage*; S-0214 Terms), so
//! the locomotor drops a destination it is offered and does nothing, before
//! part (b)'s landing point is reached. Part (b)'s warp-pending
//! immobilization keeps L only before the leaving stage begins. Every name
//! and number is invented; no game data.

mod jump_common;

use glasnost_map::CellXY;
use glasnost_sim::econ::assign_destination;
use glasnost_sim::rules::Kind;
use glasnost_sim::superweapons::warp::{Member, Stage};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use jump_common::{SIZE, at};

fn game() -> World {
    let mut w = testkit::game_with(&jump_common::factory::ruleset(), SIZE, 7);
    testkit::run(&mut w, 4);
    w
}

fn offer(w: &mut World, u: ObjectId, c: CellXY) {
    assign_destination(w, u, Some(Target::Cell(c)));
}

fn member(w: &World, u: ObjectId, stage: Stage) -> Member {
    Member {
        stage,
        house: HouseId(0),
        landing: w.obj(u).unwrap().pos,
        redirected: false,
        fell_back: false,
        refused: false,
        entry_ok: false,
    }
}

// spec: S-0207 claim 20 (a) (the locomotor drops the destination and does nothing when the object is being erased, S-0214 Terms; otherwise it fixes L, parts b and c); glossary leaving stage (it counts as being erased throughout the leaving stage, the wait-expiry update included)
#[test]
fn a_vehicle_in_the_leaving_stage_drops_an_offered_destination() {
    for stage in [Stage::Leaving(2), Stage::Expired] {
        let mut w = game();
        let u = testkit::spawn(&mut w, Kind::Vehicle, "TELEQ", 0, at(0, 0), 0);
        // An L fixed before the warp.
        offer(&mut w, u, at(3, 0));
        let before = w.obj(u).unwrap().teleport.as_ref().and_then(|t| t.landing);
        assert_eq!(before.map(|l| l.cell()), Some(at(3, 0)), "set-up");
        w.obj_mut(u).unwrap().sw.warp = Some(member(&w, u, stage));
        offer(&mut w, u, at(5, 0));
        let o = w.obj(u).unwrap();
        assert_eq!(o.dest, None, "{stage:?}: the destination is dropped");
        assert_eq!(
            o.teleport.as_ref().and_then(|t| t.landing),
            before,
            "{stage:?}: the drop does nothing to L"
        );
    }
}
