//! A teleporting-locomotor unit that is phasing in after a superweapon warp
//! drops a destination given to it, as after its own jump (#361; S-0207
//! claim 20 a and claim 27, S-0214 Terms, S-0513 Terms and claim 40). On
//! the synthetic rules of `jump_common` (`TELEQ` a vehicle with the
//! teleporting locomotor and no teleport-capable flag), no game data.

mod jump_common;

use glasnost_map::CellXY;
use glasnost_sim::econ::assign_destination;
use glasnost_sim::rules::Kind;
use glasnost_sim::superweapons::warp::{Member, Stage};
use glasnost_sim::teleport::TeleportState;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use jump_common::{SIZE, at, step};

fn game() -> World {
    let mut w = testkit::game_with(&jump_common::factory::ruleset(), SIZE, 7);
    testkit::run(&mut w, 4);
    w
}

fn tele(w: &mut World, c: CellXY) -> ObjectId {
    testkit::spawn(w, Kind::Vehicle, "TELEQ", 0, c, 0)
}

fn landing(w: &World, u: ObjectId) -> Option<CellXY> {
    w.obj(u)
        .unwrap()
        .teleport
        .as_ref()
        .and_then(|t| t.landing)
        .map(|l| l.cell())
}

/// Puts the unit in a warp's arrival hold from this tick, as the arrival
/// effects leave it (S-0513 claim 40).
fn hold(w: &mut World, u: ObjectId, held_delay: i32, redirected: bool) {
    let t = w.tick;
    let pos = w.obj(u).unwrap().pos;
    let o = w.obj_mut(u).unwrap();
    o.sw.held_delay = held_delay;
    o.sw.warp = Some(Member {
        stage: Stage::Hold(t),
        house: HouseId(0),
        landing: pos,
        redirected,
        fell_back: false,
        refused: false,
        entry_ok: true,
    });
}

// spec: S-0207 claim 20 (a) (dropped while phasing in) and claim 27 (Q-1130 table, third row)
#[test]
fn issue_361_control_own_jump_phasing_drops_the_destination() {
    let mut w = game();
    let u = tele(&mut w, at(0, 0));
    let t = w.tick;
    w.obj_mut(u).unwrap().teleport = Some(TeleportState {
        phasing: Some((t, 50)),
        ..Default::default()
    });
    assert!(glasnost_sim::teleport::phasing_in(&w, u), "set-up");
    assign_destination(&mut w, u, Some(Target::Cell(at(4, 0))));
    assert_eq!(w.obj(u).unwrap().dest, None, "dropped");
    assert_eq!(landing(&w, u), None);
    step(&mut w);
    assert_eq!(w.obj(u).unwrap().pos.cell(), at(0, 0), "no jump");
}

// spec: S-0207 claim 20 (a) (dropped while phasing in) and claim 27 (Q-1130 table, third row); S-0214 Terms (phasing in: also a superweapon teleport's arrival hold); S-0513 Terms and claim 40 (phasing in until the hold ends)
#[test]
fn issue_361_a_destination_given_in_a_warp_arrival_hold_is_dropped() {
    let mut w = game();
    let u = tele(&mut w, at(0, 0));
    hold(&mut w, u, 50, true);
    assert!(glasnost_sim::teleport::phasing_in(&w, u), "set-up");
    assign_destination(&mut w, u, Some(Target::Cell(at(4, 0))));
    let dest = w.obj(u).unwrap().dest;
    let l = landing(&w, u);
    step(&mut w);
    assert!(
        glasnost_sim::superweapons::warp::phasing(&w, u),
        "set-up: the arrival hold has not ended"
    );
    let cell = w.obj(u).unwrap().pos.cell();
    assert_eq!(
        (dest, l, cell),
        (None, None, at(0, 0)),
        "destination dropped, no landing point, no jump while in the arrival hold"
    );
}

// spec: S-0214 Terms (a destination given to the teleporting locomotor while phasing in is dropped, not saved for the end) and S-0207 claim 20 (a)
#[test]
fn issue_361_a_clean_landing_destination_is_not_saved_for_the_end() {
    let mut w = game();
    let u = tele(&mut w, at(0, 0));
    hold(&mut w, u, 0, false);
    assert!(glasnost_sim::teleport::phasing_in(&w, u), "set-up");
    assign_destination(&mut w, u, Some(Target::Cell(at(4, 0))));
    let dest = w.obj(u).unwrap().dest;
    step(&mut w);
    let cell = w.obj(u).unwrap().pos.cell();
    assert_eq!(
        (dest, cell),
        (None, at(0, 0)),
        "dropped, not saved for the end"
    );
}
