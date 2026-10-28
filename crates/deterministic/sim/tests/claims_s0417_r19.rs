//! Claim tests of S-0417 revision 19 (factory unloading, answers Q-1070) on
//! synthetic data: the ascent clear report of a released jumpjet unit
//! (claim 15) with S-0502 claim 7's thresholds. Every name and number is
//! invented; no game data.

mod jump_common;

use glasnost_map::CellXY;
use glasnost_sim::econ::release::{self, Outcome};
use glasnost_sim::jump::Phase;
use glasnost_sim::objects;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use jump_common::{SIZE, at, phase, step, until};

fn game() -> World {
    let mut w = testkit::game_with(&jump_common::factory::ruleset(), SIZE, 7);
    testkit::run(&mut w, 4);
    w
}

fn building(w: &mut World, name: &str) -> ObjectId {
    testkit::spawn(w, Kind::Building, name, 0, at(0, 0), 0)
}

fn unit(w: &mut World, kind: Kind, name: &str) -> ObjectId {
    let ti = w.rules().index_of(kind, name).unwrap();
    objects::create(w, kind, ti, Some(HouseId(0)))
}

fn rally(w: &mut World, k: ObjectId) -> Target {
    let tl = w.obj(k).unwrap().pos.cell();
    let t = Target::Cell(CellXY::new(tl.x + 8, tl.y + 8));
    w.obj_mut(k).unwrap().guard_spot = Some(t);
    t
}

/// Runs until the unit's contact ends; returns the adjusted height before
/// that update (the last with contact) and in the update that ended it.
fn run_to_report(w: &mut World, u: ObjectId, limit: i32) -> Option<(i32, i32)> {
    let mut last = w.height_above_surface(u);
    for _ in 0..limit {
        step(w);
        let h = w.height_above_surface(u);
        if w.obj(u).unwrap().contact.is_none() {
            return Some((last, h));
        }
        last = h;
    }
    None
}

// spec: S-0417 claim 15 and S-0502 claim 7: a released jumpjet infantry unit reports clear in the first ascending update whose height lies strictly between a quarter of the target height and the target height; contact and tether end together, with no foundation or distance test
// spec: S-0417 claim 10 (a jumpjet type heads straight for the rally point)
#[test]
fn claim_15_infantry_ends_contact_in_the_quarter_window() {
    let mut w = game();
    let k = building(&mut w, "BAR");
    let r = rally(&mut w, k);
    let u = unit(&mut w, Kind::Infantry, "JJ");
    assert_eq!(release::release(&mut w, k, u), Outcome::Released);
    assert!(release::tethered(&w, u));
    assert_eq!(w.obj(u).unwrap().dest, Some(r));
    let point_before = w.obj(u).unwrap().jump.as_ref().unwrap().dest;
    assert!(point_before.is_some());
    let (before, at_report) = run_to_report(&mut w, u, 200).expect("the contact ended");
    let ht = w.obj(u).unwrap().jump.as_ref().unwrap().ht;
    assert_eq!(ht, 500);
    assert!(
        before <= ht / 4,
        "no report while at or below the quarter: {before}"
    );
    assert!(
        ht / 4 < at_report && at_report < ht,
        "the report is inside the window: {at_report}"
    );
    let o = w.obj(u).unwrap();
    assert!(!release::tethered(&w, u));
    assert!(o.econ.exit.is_none());
    // The jumpjet is still above its factory and climbing: no cell crossing was needed.
    assert_eq!(phase(&w, u), Phase::Ascending);
    // spec: S-0417 claim 15 (placement consumed the guard spot, so the order-level destination is discarded; a refused scatter leaves the movement point and commitment as they were)
    assert_eq!(o.guard_spot, None);
    let j = o.jump.as_ref().unwrap();
    if o.dest.is_none() {
        assert_eq!(j.dest, point_before);
        assert!(j.moving);
    } else {
        assert_ne!(o.dest, Some(r), "the rally destination was discarded");
    }
}

// spec: S-0417 claim 15 (an infantry factory has no unloading stage: the contacted place is free in the report tick, so the next release is no longer busy on account of the old contact; claim 14)
#[test]
fn claim_15_infantry_place_is_free_after_the_report() {
    let mut w = game();
    let k = building(&mut w, "BAR");
    rally(&mut w, k);
    let a = unit(&mut w, Kind::Infantry, "JJ");
    assert_eq!(release::release(&mut w, k, a), Outcome::Released);
    let b = unit(&mut w, Kind::Infantry, "JJ");
    assert_eq!(
        release::release(&mut w, k, b),
        Outcome::Busy,
        "one contact place is held"
    );
    assert!(run_to_report(&mut w, a, 200).is_some());
    assert_eq!(release::release(&mut w, k, b), Outcome::Released);
}

// spec: S-0417 claim 15 and S-0502 claim 7: a balloon-hover vehicle uses a half, a non-balloon jumpjet vehicle a quarter; the vehicle factory leaves its unloading stage after the report and does not wait for the journey
#[test]
fn claim_15_vehicles_half_and_quarter() {
    for (name, q) in [("BALLOON", 2), ("JV", 4)] {
        let mut w = game();
        let k = building(&mut w, "VF");
        let spot = rally(&mut w, k);
        let u = unit(&mut w, Kind::Vehicle, name);
        assert_eq!(release::release(&mut w, k, u), Outcome::Released, "{name}");
        assert!(release::tethered(&w, u));
        let (before, at_report) = run_to_report(&mut w, u, 400).expect("the contact ended");
        let ht = w.obj(u).unwrap().jump.as_ref().unwrap().ht;
        assert!(
            before <= ht / q,
            "{name}: no report at or below ht/{q}: {before}"
        );
        assert!(
            ht / q < at_report && at_report < ht,
            "{name}: inside the window: {at_report}"
        );
        // spec: S-0417 claim 15 (a saved rally becomes the destination, not claim 8's arrival handling)
        assert_eq!(w.obj(u).unwrap().dest, Some(spot), "{name}");
        // The factory closes its door and falls back to rest well before the jumpjet arrives anywhere.
        let mut closed = false;
        for _ in 0..200 {
            step(&mut w);
            if w.obj(k).unwrap().beh.current != Some(Behaviour::Unload) {
                closed = true;
                break;
            }
        }
        assert!(closed, "{name}: the factory leaves its unloading stage");
        assert_ne!(phase(&w, u), Phase::Grounded, "{name}");
        // A further vehicle comes out of the same factory.
        let again = unit(&mut w, Kind::Vehicle, name);
        assert_eq!(
            release::release(&mut w, k, again),
            Outcome::Released,
            "{name}"
        );
    }
}

// spec: S-0417 claim 15 (without a saved rally the order-level destination is discarded and a forced scatter request follows; neither recreates contact or tether; the flight is not stopped)
#[test]
fn claim_15_no_rally_discards_the_destination() {
    let mut w = game();
    let k = building(&mut w, "VF");
    let u = unit(&mut w, Kind::Vehicle, "JV");
    assert_eq!(release::release(&mut w, k, u), Outcome::Released);
    // The exit instruction comes with the factory's unloading stages.
    assert!(until(&mut w, 200, |w| w.obj(u).unwrap().dest.is_some()));
    let exit = w.obj(u).unwrap().dest;
    assert!(w.obj(u).unwrap().contact.is_some(), "still in contact");
    let point = w.obj(u).unwrap().jump.as_ref().unwrap().dest;
    assert!(run_to_report(&mut w, u, 400).is_some());
    let o = w.obj(u).unwrap();
    assert_ne!(o.dest, exit, "the exit destination was discarded");
    assert!(o.contact.is_none() && o.econ.exit.is_none());
    // The locomotor's movement point and commitment survive the discard (a scatter that is accepted may replace them).
    let j = o.jump.as_ref().unwrap();
    assert!(j.moving);
    assert!(point.is_some());
    if o.dest.is_none() {
        // A refused scatter: the preceding point stays exactly.
        assert_eq!(j.dest, point);
    }
    for _ in 0..50 {
        step(&mut w);
        let o = w.obj(u).unwrap();
        assert!(o.contact.is_none() && o.econ.exit.is_none());
    }
}

// spec: S-0417 claim 15 (design: a unit whose ascent reaches or passes the target height without a qualifying report reports in the reaching update; no quirk switch) and S-0502 claim 7 and J26
#[test]
fn claim_15_skipped_window_reports_in_the_reaching_update() {
    let mut w = game();
    let k = building(&mut w, "VF");
    let u = unit(&mut w, Kind::Vehicle, "JVQUICK");
    assert_eq!(release::release(&mut w, k, u), Outcome::Released);
    let mut reported = None;
    for _ in 0..60 {
        step(&mut w);
        if w.obj(u).unwrap().contact.is_none() {
            reported = Some((w.height_above_surface(u), phase(&w, u)));
            break;
        }
    }
    let (h, ph) = reported.expect("the factory is not held for good");
    // The physics step rose through the whole window at once, so the reaching update reported.
    assert!(h >= 4, "the report came at or above the target height: {h}");
    assert_ne!(ph, Phase::Grounded);
    assert!(!release::tethered(&w, u));
}

// spec: S-0417 claim 15 (the unit's type is marked as jumpjet or balloon-hover: the locomotor alone, with neither flag, raises no report; the case is unmeasured and the claim reports only for the flags)
#[test]
fn claim_15_type_flags_gate_the_report() {
    let mut w = game();
    let k = building(&mut w, "BAR");
    rally(&mut w, k);
    let u = unit(&mut w, Kind::Infantry, "JJLOCO");
    assert_eq!(release::release(&mut w, k, u), Outcome::Released);
    for _ in 0..100 {
        step(&mut w);
    }
    assert!(
        w.obj(u).unwrap().contact.is_some(),
        "no report without a flag"
    );
}

// spec: S-0417 claim 15 (balloon-hover infantry still uses a quarter, q = 4)
#[test]
fn claim_15_balloon_hover_infantry_uses_a_quarter() {
    let mut w = game();
    let k = building(&mut w, "BAR");
    rally(&mut w, k);
    let u = unit(&mut w, Kind::Infantry, "JJBAL");
    assert_eq!(release::release(&mut w, k, u), Outcome::Released);
    let (before, at_report) = run_to_report(&mut w, u, 200).expect("the contact ended");
    let ht = w.obj(u).unwrap().jump.as_ref().unwrap().ht;
    assert!(before <= ht / 4);
    assert!(ht / 4 < at_report && at_report < ht);
}

// spec: S-0417 claim 15 (the report needs a vehicle factory or a barracks door kind: a contacted building with neither gives none)
#[test]
fn claim_15_other_buildings_give_no_report() {
    let mut w = game();
    let k = building(&mut w, "PLAIN");
    rally(&mut w, k);
    let u = unit(&mut w, Kind::Infantry, "JJ");
    assert_eq!(release::release(&mut w, k, u), Outcome::Released);
    for _ in 0..100 {
        step(&mut w);
    }
    assert!(w.obj(u).unwrap().contact.is_some());
}

// spec: S-0417 claim 5 and S-0207 claim 27 (revision 19: restoration of a temporarily driving teleporting vehicle needs a suspended predecessor, no driving movement report, no restoration lock and no seizure; a retained locomotor destination point prevents it; restoration alone enables neither pending state)
// spec: S-0417 claim 16 (an accepted scatter supplies a destination: its movement point replaces the exit point in the active driving locomotion) and S-0207 claim 16 b (the refusal reads the current locomotor: the temporarily driving vehicle accepts the contact end's scatter request, S-0417 claim 15)
#[test]
fn claim_5_restoration_needs_no_movement_report() {
    let mut w = game();
    let k = building(&mut w, "VF");
    let u = unit(&mut w, Kind::Vehicle, "TELEQ");
    assert_eq!(release::release(&mut w, k, u), Outcome::Released);
    // The exit track runs to the apron and the vehicle leaves the factory's contact.
    assert!(until(&mut w, 400, |w| {
        let o = w.obj(u).unwrap();
        o.contact.is_none() && o.econ.temporary_drive
    }));
    // The track is complete but its destination point is retained: it prevents restoration.
    assert!(w.obj(u).unwrap().drive.as_ref().unwrap().goal.is_some());
    assert!(!release::restore_locomotor(&mut w, u));
    assert!(w.obj(u).unwrap().econ.temporary_drive);
    // The contact end's scatter request was accepted while driving (its point replaces the exit point at the
    // next locomotion, claims_s0417_r20); a stop in this same tick removes the destination and the point at
    // once, and the next fall-back restores.
    assert!(matches!(w.obj(u).unwrap().dest, Some(Target::Cell(_))));
    glasnost_sim::econ::assign_destination(&mut w, u, None);
    assert!(!glasnost_sim::drive::movement_report(&w, u, false));
    glasnost_sim::process::fall_back(&mut w, u, false);
    let o = w.obj(u).unwrap();
    assert!(!o.econ.temporary_drive && o.drive.is_none());
    // Restoration alone enables neither the refresh state nor the suppression.
    assert!(!o.econ.refresh && !o.econ.point_suppression);
}

// spec: S-0417 claim 5 and S-0207 claim 27 (a heading point horizontally different from the actual position is a driving movement report too)
#[test]
fn claim_5_a_different_heading_point_is_a_report() {
    let mut w = game();
    let k = building(&mut w, "VF");
    let u = unit(&mut w, Kind::Vehicle, "TELE");
    assert_eq!(release::release(&mut w, k, u), Outcome::Released);
    assert!(until(&mut w, 400, |w| w
        .obj(u)
        .unwrap()
        .drive
        .as_ref()
        .is_some_and(|d| d.track.is_some())));
    // On the track: its heading point is the apron centre, away from the vehicle.
    glasnost_sim::drive::stop_point(&mut w, u);
    assert!(glasnost_sim::drive::movement_report(&w, u, false));
    assert!(!glasnost_sim::econ::release::restorable(&w, u, false));
}

// spec: S-0417 claim 15 (the state is serialisable and hashed: a save taken after the report restores the same world)
#[test]
fn claim_15_report_survives_a_save() {
    let mut w = game();
    let k = building(&mut w, "BAR");
    rally(&mut w, k);
    let u = unit(&mut w, Kind::Infantry, "JJ");
    assert_eq!(release::release(&mut w, k, u), Outcome::Released);
    assert!(run_to_report(&mut w, u, 200).is_some());
    let bytes = glasnost_sim::save::save(&w);
    let mut back =
        glasnost_sim::save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    for _ in 0..100 {
        step(&mut w);
        step(&mut back);
    }
    assert_eq!(
        glasnost_sim::save::state_hash(&back),
        glasnost_sim::save::state_hash(&w)
    );
}
