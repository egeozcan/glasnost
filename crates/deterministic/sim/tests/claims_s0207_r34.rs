//! Claim tests of S-0207 revision 34 claim 27 (answers Q-1110 questions 1
//! and 2) with S-0502 revision 10 claim 27 and S-0402 revision 27 claim 10:
//! the warp-pending immobilization predicate of the dock test and the
//! restoring assignment's destination. Every name and number is invented;
//! no game data.

mod jump_common;

use glasnost_map::CellXY;
use glasnost_sim::econ::assign_destination;
use glasnost_sim::econ::release;
use glasnost_sim::rules::{Behaviour, Kind};
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

fn spawn(w: &mut World, kind: Kind, name: &str, c: CellXY) -> ObjectId {
    testkit::spawn(w, kind, name, 0, c, 0)
}

/// A vehicle of type `name` in contact with a dock-unload refinery.
fn docked(name: &str) -> (World, ObjectId) {
    let mut w = game();
    let refinery = spawn(&mut w, Kind::Building, "REFN", at(-6, 0));
    let u = spawn(&mut w, Kind::Vehicle, name, at(0, 0));
    w.obj_mut(u).unwrap().contact = Some(refinery);
    (w, u)
}

fn flags(w: &World, u: ObjectId) -> (bool, bool) {
    let o = w.obj(u).unwrap();
    (o.econ.refresh, o.econ.point_suppression)
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

// spec: S-0207 claim 27 (Which immobilization: the predicate reads the warp-pending immobilization, from the accepted warp through the leaving wait until departure, even when the current locomotor is not teleporting; it fails the dock test with no stop, restoration or pending state)
// spec: S-0502 claim 27 (the immobilization predicate applies independently of the current locomotor)
#[test]
fn warp_pending_immobilization_fails_the_dock_test() {
    for stage in [Stage::Pending, Stage::Leaving(3), Stage::Expired] {
        let (mut w, u) = docked("TELEDRIVE");
        let m = member(&w, u, stage);
        let o = w.obj_mut(u).unwrap();
        o.sw.warp = Some(m);
        o.beh.current = Some(Behaviour::Guard);
        offer(&mut w, u, at(3, 0));
        let o = w.obj(u).unwrap();
        assert_eq!(flags(&w, u), (false, false), "{stage:?}");
        assert_eq!(o.beh.current, Some(Behaviour::Guard), "{stage:?}");
        assert_eq!(o.beh.queued, None, "{stage:?}");
        assert!(
            o.drive.as_ref().unwrap().goal.is_some(),
            "{stage:?}: the point is applied as usual"
        );
    }
}

// spec: S-0207 claim 27 (Which immobilization: it does not read phasing in after a superweapon arrival, the arrival hold included, nor phasing in after an own-locomotor jump; such a vehicle still qualifies and falls back)
#[test]
fn phasing_in_is_not_the_immobilization() {
    let cases: [(&str, Option<Stage>, bool); 4] = [
        ("away", Some(Stage::Away), false),
        ("placed", Some(Stage::Placed), false),
        ("hold", Some(Stage::Hold(2)), false),
        ("own jump", None, true),
    ];
    for (label, stage, own) in cases {
        let (mut w, u) = docked("TELEDRIVE");
        let m = stage.map(|s| member(&w, u, s));
        let t = w.tick;
        let o = w.obj_mut(u).unwrap();
        o.sw.warp = m;
        if own {
            o.teleport = Some(TeleportState {
                phasing: Some((t, 50)),
                ..Default::default()
            });
        }
        offer(&mut w, u, at(3, 0));
        assert_eq!(flags(&w, u), (true, false), "{label}: the fallback");
        assert_eq!(
            w.obj(u).unwrap().beh.queued,
            Some(Behaviour::Enter),
            "{label}"
        );
    }
}

/// A teleporting vehicle temporarily driving with no movement report, in
/// contact with a dock-unload refinery.
fn restorable() -> (World, ObjectId) {
    let (mut w, u) = docked("TELE");
    release::install_temporary_drive(&mut w, u);
    assert!(release::restorable(&w, u, true));
    (w, u)
}

// spec: S-0207 claim 27 (The restoring assignment's destination: successful restoration makes that same assignment fix its landing point under claim 20, rather than install a driving destination point first; the vehicle jumps on its next teleporting movement update; restoration sets neither pending state)
// spec: S-0402 claim 10 (on qualified successful restoration the restored movement receives that same docking-cell offer; neither a preliminary driving leg nor a second offer is needed)
// spec: S-0502 claim 27 (that same assignment's otherwise due point goes to the restored predecessor, not to the former temporary locomotor)
#[test]
fn the_restored_predecessor_takes_the_offer_and_jumps() {
    let (mut w, u) = restorable();
    let c = at(3, 0);
    offer(&mut w, u, c);
    let o = w.obj(u).unwrap();
    assert!(!release::temporary_drive(&w, u), "restored");
    assert!(o.drive.is_none(), "no driving point was installed");
    let t = o.teleport.as_ref().unwrap();
    assert_eq!(
        t.landing.map(|l| l.cell()),
        Some(c),
        "the landing point is this offer's"
    );
    assert_eq!(t.for_dest, Some(Target::Cell(c)));
    assert_eq!(flags(&w, u), (false, false));
    assert_eq!(o.beh.queued, None, "no fallback");
    step(&mut w);
    assert_eq!(
        w.obj(u).unwrap().pos.cell(),
        c,
        "it jumps at its next teleporting movement update"
    );
}

// spec: S-0207 claim 27 (eligibility uses the driving points already present, not the destination offered by this assignment; a retained driving destination point prevents restoration even when the offer differs)
#[test]
fn eligibility_reads_the_stored_driving_points() {
    // No stored point: the offer, though far, does not count as a report.
    let (mut w, u) = restorable();
    offer(&mut w, u, at(9, 9));
    assert!(!release::temporary_drive(&w, u));
    // A stored point: the fallback, whatever the offer.
    let (mut w, u) = restorable();
    w.obj_mut(u).unwrap().drive.as_mut().unwrap().goal =
        Some(glasnost_sim::world::Pos::centre(at(1, 0), 0));
    offer(&mut w, u, at(0, 1));
    assert!(release::temporary_drive(&w, u), "not restored");
    assert_eq!(flags(&w, u), (true, false));
}

// spec: S-0207 claim 27 (a pending earlier teleport can be replaced by the offered destination of the restoring assignment)
#[test]
fn a_pending_earlier_teleport_is_replaced() {
    let (mut w, u) = docked("TELE");
    // An earlier teleport is fixed and not yet jumped to, then the vehicle drives temporarily.
    offer(&mut w, u, at(-2, 4));
    let earlier = w.obj(u).unwrap().teleport.as_ref().unwrap().landing;
    assert_eq!(earlier.map(|l| l.cell()), Some(at(-2, 4)));
    release::install_temporary_drive(&mut w, u);
    let c = at(3, 0);
    offer(&mut w, u, c);
    assert!(!release::temporary_drive(&w, u));
    let t = w.obj(u).unwrap().teleport.as_ref().unwrap();
    assert_eq!(
        t.landing.map(|l| l.cell()),
        Some(c),
        "replaced by the offer"
    );
    step(&mut w);
    assert_eq!(w.obj(u).unwrap().pos.cell(), c);
}

// spec: S-0207 claim 27 (restoration does not exempt this application from suppression already pending before the assignment: the order-level destination is installed, no landing point is fixed and the suppression is consumed)
#[test]
fn pending_suppression_still_skips_the_restored_point() {
    let (mut w, u) = restorable();
    w.obj_mut(u).unwrap().econ.point_suppression = true;
    let c = at(3, 0);
    offer(&mut w, u, c);
    let o = w.obj(u).unwrap();
    assert!(!release::temporary_drive(&w, u), "restored");
    assert_eq!(o.dest, Some(Target::Cell(c)));
    assert_eq!(o.teleport.as_ref().and_then(|t| t.landing), None);
    assert_eq!(flags(&w, u), (false, false), "the suppression is consumed");
    step(&mut w);
    assert_eq!(w.obj(u).unwrap().pos.cell(), at(0, 0), "no jump");
}

// spec: S-0207 claim 27 (the restored predecessor receives this assignment's otherwise due point: here the already-pending suppression consumes that application)
// spec: S-0207 claim 21 (installing temporary driving cancels the pending jump: the suspended state keeps no point for the cell)
#[test]
fn the_restored_predecessor_takes_an_offer_equal_to_its_suspended_one() {
    let (mut w, u) = docked("TELE");
    let c = at(3, 0);
    offer(&mut w, u, c);
    release::install_temporary_drive(&mut w, u);
    assign_destination(&mut w, u, None);
    let t = w.obj(u).unwrap().teleport.clone().unwrap();
    assert_eq!(t.for_dest, None, "the pending jump was cancelled");
    assert_eq!(t.landing, None);
    w.obj_mut(u).unwrap().econ.point_suppression = true;
    offer(&mut w, u, c);
    assert!(!release::temporary_drive(&w, u), "restored");
    assert_eq!(
        flags(&w, u),
        (false, false),
        "the restored locomotor took the application"
    );
}

// spec: S-0207 claim 20 (b) (warp-pending immobilization leaves L unchanged; when no L was ever fixed, no jump happens and the destination is removed; with an L fixed earlier, that L stays)
#[test]
fn claim_20_b_warp_pending_immobilization_keeps_the_landing_point() {
    let mut w = game();
    let u = spawn(&mut w, Kind::Vehicle, "TELEQ", at(0, 0));
    let m = member(&w, u, Stage::Pending);
    w.obj_mut(u).unwrap().sw.warp = Some(m);
    offer(&mut w, u, at(4, 0));
    let o = w.obj(u).unwrap();
    assert_eq!(o.dest, None, "no L was fixed: the destination is removed");
    assert_eq!(o.teleport.as_ref().and_then(|t| t.landing), None);
    // An L fixed before the warp stays when a new destination arrives.
    let mut w = game();
    let u = spawn(&mut w, Kind::Vehicle, "TELEQ", at(0, 0));
    offer(&mut w, u, at(3, 0));
    w.obj_mut(u).unwrap().sw.warp = Some(member(&w, u, Stage::Leaving(2)));
    offer(&mut w, u, at(5, 0));
    let o = w.obj(u).unwrap();
    assert_eq!(o.dest, Some(Target::Cell(at(5, 0))));
    assert_eq!(
        o.teleport
            .as_ref()
            .and_then(|t| t.landing)
            .map(|l| l.cell()),
        Some(at(3, 0)),
        "L unchanged"
    );
}
