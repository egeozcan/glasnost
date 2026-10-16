//! Claim tests of S-0417 revision 19 (factory unloading, answers Q-1070) on
//! synthetic data: the ascent clear report of a released jumpjet unit
//! (claim 15) with S-0502 claim 7's thresholds. Every name and number is
//! invented; no game data.

mod jump_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::BuildingType;
use glasnost_sim::econ::release::{self, Outcome};
use glasnost_sim::jump::Phase;
use glasnost_sim::objects;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use jump_common::{SIZE, at, d, phase, step, until};

fn ruleset() -> Ruleset {
    let mut r = jump_common::ruleset();
    // A vehicle factory with a 3 x 2 foundation and an invented door time, and a barracks with the first door kind.
    let mut vf = BuildingType {
        strength: Some(500),
        foundation: Some([3, 2]),
        factory: Some("vehicle".into()),
        weapons_factory: Some(true),
        ..Default::default()
    };
    vf.deploy_time = Some(d("0.01"));
    vf.combat.armor = None;
    let mut bar = BuildingType {
        strength: Some(500),
        foundation: Some([2, 2]),
        factory: Some("infantry".into()),
        ..Default::default()
    };
    bar.econ.door_first = Some(true);
    bar.combat.armor = None;
    // A building with a rally-capable foundation but neither door kind nor the vehicle factory flag.
    let mut plain = BuildingType {
        strength: Some(500),
        foundation: Some([2, 2]),
        factory: Some("infantry".into()),
        ..Default::default()
    };
    plain.combat.armor = None;
    let b = r.registries.get_mut("buildings").unwrap();
    b.extend(["VF", "BAR", "PLAIN"].map(String::from));
    r.sim.buildings.insert("VF".into(), vf);
    r.sim.buildings.insert("BAR".into(), bar);
    r.sim.buildings.insert("PLAIN".into(), plain);
    // A jumpjet vehicle and infantry whose cruise height is tiny and whose climb passes it in one step (the skipped window).
    let mut quick_v = r.sim.vehicles.get("JV").unwrap().clone();
    quick_v.jumpjet_height = Some(4);
    quick_v.jumpjet_climb = Some(d("60.0"));
    r.sim.vehicles.insert("JVQUICK".into(), quick_v);
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("JVQUICK".into());
    r
}

fn game() -> World {
    let mut w = testkit::game_with(&ruleset(), SIZE, 7);
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
    let (before, at_report) = run_to_report(&mut w, u, 200).expect("the contact ended");
    let ht = w.obj(u).unwrap().jump.as_ref().unwrap().ht;
    assert_eq!(ht, 500);
    assert!(before <= ht / 4, "no report while at or below the quarter: {before}");
    assert!(
        ht / 4 < at_report && at_report < ht,
        "the report is inside the window: {at_report}"
    );
    let o = w.obj(u).unwrap();
    assert!(!release::tethered(&w, u));
    assert!(o.econ.exit.is_none());
    // The jumpjet is still above its factory's foundation and climbing: no cell crossing was needed.
    assert_eq!(phase(&w, u), Phase::Ascending);
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
        assert!(before <= ht / q, "{name}: no report at or below ht/{q}: {before}");
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
        assert_eq!(release::release(&mut w, k, again), Outcome::Released, "{name}");
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
    assert!(j.dest.is_some() || point.is_none());
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
    assert!(w.obj(u).unwrap().contact.is_some(), "no report without a flag");
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
