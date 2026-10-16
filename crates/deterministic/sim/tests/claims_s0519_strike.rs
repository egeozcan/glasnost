//! Claim tests for the air-strike designation (S-0519 claims 16 to 21) on
//! synthetic data: invented designators, aircraft and warheads.

#[path = "terror_common/mod.rs"]
mod terror_common;

use glasnost_sim::combat::fire;
use glasnost_sim::objects;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::strike;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use terror_common::*;

/// A designator of house 0 and two enemy buildings of house 1.
fn scene(tweak: impl FnOnce(&mut glasnost_rules::schema::Ruleset)) -> (World, ObjectId, ObjectId, ObjectId) {
    let mut w = world(tweak);
    let a = quiet(&mut w, Kind::Building, "BLDG", 1, at(0, 0));
    let b = quiet(&mut w, Kind::Building, "BLDG", 1, at(4, 4));
    let d = quiet(&mut w, Kind::Vehicle, "DESIG", 0, at(-6, 0));
    (w, d, a, b)
}

fn promote(w: &mut World, d: ObjectId) {
    use glasnost_num::{F64, Rounding};
    let rz = Rounding::TowardZero;
    w.obj_mut(d).unwrap().combat.experience = F64::from_i64(2, rz).to_f32(rz);
}

fn kind_of(w: &World, id: ObjectId, name: &str) -> bool {
    w.obj(id).map(|o| o.type_index) == w.rules().index_of(Kind::Aircraft, name)
}

// spec: S-0519 claims 16 and 17 (T16, T17: the shot does no area damage; in its detonation tick n aircraft of the strike type are created for the designator's house, linked, aimed at the cell of the building's reference point; the building becomes the strike target)
#[test]
fn a_designation_starts_a_strike() {
    let (mut w, d, a, _) = scene(|_| {});
    designate(&mut w, d, a);
    assert_eq!(health(&w, a), 800, "no health loss from the shot");
    let air = linked(&w, d);
    assert_eq!(air.len(), 2);
    let ref_cell = w.reference_point(a).cell();
    for &x in &air {
        let o = w.obj(x).unwrap();
        assert!(kind_of(&w, x, "JET"));
        assert_eq!(o.owner, Some(HouseId(0)));
        assert_eq!(o.target, Some(Target::Cell(ref_cell)));
        assert_eq!(o.beh.current, Some(Behaviour::Attack));
        assert!(o.air.as_ref().unwrap().visiting);
    }
    let s = &w.strike.strikes[&d];
    assert!(s.progress && s.active);
    assert_eq!(s.target, Some(a));
    assert!(strike::marked(&w, a));
}

// spec: S-0519 claim 17 (all aircraft of one strike are placed at the same cell)
#[test]
fn one_arrival_cell_serves_the_whole_strike() {
    let (mut w, d, a, _) = scene(|_| {});
    designate(&mut w, d, a);
    let air = linked(&w, d);
    let cells: Vec<_> = air.iter().map(|&x| w.obj(x).unwrap().pos.cell()).collect();
    assert_eq!(cells[0], cells[1]);
}

// spec: S-0519 claim 17 and Terms (the elite values apply while the rank is elite: size 3 of the elite type)
#[test]
fn an_elite_designator_calls_the_elite_strike() {
    let (mut w, d, a, _) = scene(|_| {});
    promote(&mut w, d);
    designate(&mut w, d, a);
    let air = linked(&w, d);
    assert_eq!(air.len(), 3);
    assert!(air.iter().all(|&x| kind_of(&w, x, "JET2")));
}

// spec: S-0519 claim 18 (T18: while a strike is in progress further shots at its strike target do nothing)
#[test]
fn further_shots_at_the_target_add_nothing() {
    let (mut w, d, a, _) = scene(|_| {});
    designate(&mut w, d, a);
    let before = (w.objects.len(), w.rng.clone());
    strike::designation(&mut w, Some(d), Some(Target::Object(a)));
    assert_eq!(linked(&w, d).len(), 2);
    assert_eq!(w.objects.len(), before.0);
    assert!(w.rng == before.1, "a not-ready shot takes no draw (claim 21)");
}

// spec: S-0519 claim 16 (T16 a, f: a firer that is not a designator calls no strike; a shot at a cell calls none)
#[test]
fn only_designators_with_object_targets_call_strikes() {
    let (mut w, _, a, _) = scene(|r| vehicle(r, "NOTD", |t| t.primary = Some("Laser".into())));
    let n = quiet(&mut w, Kind::Vehicle, "NOTD", 0, at(-6, 1));
    designate(&mut w, n, a);
    assert!(w.strike.strikes.is_empty());
    let d = quiet(&mut w, Kind::Vehicle, "DESIG", 0, at(-6, -1));
    fire::shot(&mut w, d, 0, Target::Cell(at(0, 0)));
    for _ in 0..20 {
        testkit::run(&mut w, 1);
    }
    assert!(!w.strike.strikes.contains_key(&d));
}

// spec: S-0519 claim 16 (T16 c: a designator killed while its shot is in flight: no strike)
#[test]
fn a_dead_firer_calls_no_strike() {
    let (mut w, d, a, _) = scene(|_| {});
    w.obj_mut(d).unwrap().target = Some(Target::Object(a));
    fire::shot(&mut w, d, 0, Target::Object(a));
    objects::remove(&mut w, d);
    for _ in 0..30 {
        testkit::run(&mut w, 1);
    }
    assert!(w.strike.strikes.is_empty());
    assert!(w.objects.values().all(|o| o.kind != Kind::Aircraft));
}

// spec: S-0519 claim 19 (T19 a: a change of the designator's target, here to none, cancels: the aircraft lose target and destination and queue retreat; they stay linked; the strike target is forgotten)
#[test]
fn a_change_of_target_cancels() {
    let (mut w, d, a, _) = scene(|_| {});
    designate(&mut w, d, a);
    w.obj_mut(d).unwrap().target = None;
    testkit::run(&mut w, 1);
    let s = &w.strike.strikes[&d];
    assert_eq!((s.target, s.progress, s.active), (None, true, false));
    assert!(!strike::marked(&w, a));
    for x in linked(&w, d) {
        let o = w.obj(x).unwrap();
        assert!(o.target.is_none() && o.dest.is_none() || o.dest.is_some(), "no target");
        assert!(o.target.is_none());
        assert!(o.beh.queued == Some(Behaviour::Retreat) || o.beh.current == Some(Behaviour::Retreat));
    }
}

// spec: S-0519 claim 19 (T19 f: setting the same target again does not cancel)
#[test]
fn the_same_target_does_not_cancel() {
    let (mut w, d, a, _) = scene(|_| {});
    designate(&mut w, d, a);
    w.obj_mut(d).unwrap().target = Some(Target::Object(a));
    testkit::run(&mut w, 2);
    assert_eq!(w.strike.strikes[&d].target, Some(a));
}

// spec: S-0519 claim 19 (T19 b: a ready designation at another object while in progress redirects: every aircraft with ammunition takes the object itself as target and queues attack)
#[test]
fn a_new_designation_redirects() {
    let (mut w, d, a, b) = scene(|_| {});
    designate(&mut w, d, a);
    designate(&mut w, d, b);
    let s = &w.strike.strikes[&d];
    assert_eq!(s.target, Some(b));
    assert!(strike::marked(&w, b) && !strike::marked(&w, a));
    for x in linked(&w, d) {
        assert_eq!(w.obj(x).unwrap().target, Some(Target::Object(b)));
    }
    assert_eq!(linked(&w, d).len(), 2, "no new aircraft");
}

// spec: S-0519 claim 19 (T19 e: aircraft with unequal ammunition: the strike is cancelled again and nothing is redirected)
#[test]
fn unequal_ammunition_cancels_again() {
    let (mut w, d, a, b) = scene(|_| {});
    designate(&mut w, d, a);
    let air = linked(&w, d);
    w.obj_mut(air[0]).unwrap().combat.ammo = 1;
    designate(&mut w, d, b);
    let s = &w.strike.strikes[&d];
    assert_eq!(s.target, None);
    for x in air {
        assert!(w.obj(x).unwrap().target != Some(Target::Object(b)));
    }
}

// spec: S-0519 claim 19 (T19 c: once an aircraft has spent its last round, a cancel leaves every linked aircraft with 0 rounds)
#[test]
fn a_spent_aircraft_empties_them_all_on_cancel() {
    let (mut w, d, a, _) = scene(|_| {});
    designate(&mut w, d, a);
    let air = linked(&w, d);
    w.obj_mut(air[0]).unwrap().combat.ammo = 0;
    w.obj_mut(d).unwrap().target = None;
    testkit::run(&mut w, 1);
    for x in air {
        assert_eq!(w.obj(x).unwrap().combat.ammo, 0);
    }
}

// spec: S-0519 claim 19 (T19 d: the marked building destroyed cancels)
#[test]
fn a_destroyed_marked_building_cancels() {
    let (mut w, d, a, _) = scene(|_| {});
    designate(&mut w, d, a);
    objects::remove(&mut w, a);
    assert_eq!(w.strike.strikes[&d].target, None);
}

// spec: S-0519 claims 18 and 20 (T18: the last aircraft leaving makes the strike idle with e = that tick and the strike target kept; a repeat designation of the same object waits until e + R = e + 100)
#[test]
fn recharge_holds_back_a_repeat_designation() {
    let (mut w, d, a, b) = scene(|_| {});
    designate(&mut w, d, a);
    for x in linked(&w, d) {
        objects::remove(&mut w, x);
    }
    let e = w.tick;
    let s = &w.strike.strikes[&d];
    assert_eq!((s.progress, s.end, s.target), (false, e, Some(a)));
    designate(&mut w, d, a);
    assert!(linked(&w, d).is_empty(), "before e + R: nothing");
    // A second building is ready at once (T18 c).
    designate(&mut w, d, b);
    assert_eq!(linked(&w, d).len(), 2);
    for x in linked(&w, d) {
        objects::remove(&mut w, x);
    }
    let e2 = w.tick;
    run_to(&mut w, e2 + 100);
    designate(&mut w, d, b);
    assert_eq!(linked(&w, d).len(), 2, "from e + R: a new strike");
}

// spec: S-0519 claim 18 (T18 b: a cancel forgets the strike target, so an idle strike is ready at once for the same object)
#[test]
fn a_cancel_makes_the_same_object_ready() {
    let (mut w, d, a, _) = scene(|_| {});
    designate(&mut w, d, a);
    for x in linked(&w, d) {
        objects::remove(&mut w, x);
    }
    w.obj_mut(d).unwrap().target = None;
    testkit::run(&mut w, 1);
    designate(&mut w, d, a);
    assert_eq!(linked(&w, d).len(), 2);
}

// spec: S-0519 claim 18 and Edge cases (quirks.airstrike_recharge_same_target_only off: an idle strike that has ended before is ready only after e + R, even for another object)
#[test]
fn the_recharge_quirk_off_holds_back_any_object() {
    for same_only in [true, false] {
        let (mut w, d, a, b) = scene(|r| r.globals.quirks.airstrike_recharge_same_target_only = same_only);
        designate(&mut w, d, a);
        for x in linked(&w, d) {
            objects::remove(&mut w, x);
        }
        designate(&mut w, d, b);
        assert_eq!(linked(&w, d).len(), if same_only { 2 } else { 0 });
    }
}

// spec: S-0519 claim 20 (T20 b: a destroyed designator ends its strike: the aircraft stop being linked and the strike is gone)
#[test]
fn a_lost_designator_unlinks_its_aircraft() {
    let (mut w, d, a, _) = scene(|_| {});
    designate(&mut w, d, a);
    let air = linked(&w, d);
    objects::remove(&mut w, d);
    assert!(w.strike.strikes.is_empty());
    assert!(air.iter().all(|&x| !strike::linked(&w, x)));
}

// spec: S-0519 claim 20 (T20 d: the aircraft keep the designator's house at the start tick when its owner changes)
#[test]
fn the_aircraft_keep_their_house() {
    let (mut w, d, a, _) = scene(|_| {});
    designate(&mut w, d, a);
    w.obj_mut(d).unwrap().owner = Some(HouseId(1));
    for x in linked(&w, d) {
        assert_eq!(w.obj(x).unwrap().owner, Some(HouseId(0)));
    }
}

// spec: S-0519 Edge cases and quirks.airstrike_empty_strike_stuck (an elite designator of size 0 places no aircraft: the strike stays in progress for ever and later designations only set the target; off: idle, target and end unchanged)
#[test]
fn an_empty_start_sticks_unless_the_quirk_is_off() {
    for stuck in [true, false] {
        let (mut w, d, a, b) = scene(|r| {
            r.globals.quirks.airstrike_empty_strike_stuck = stuck;
            r.sim.vehicles.get_mut("DESIG").unwrap().combat.elite_airstrike_team = Some(0);
        });
        promote(&mut w, d);
        designate(&mut w, d, a);
        let s = w.strike.strikes[&d].clone();
        assert_eq!((s.progress, s.target, s.aircraft.len()), (stuck, None, 0));
        designate(&mut w, d, b);
        let s = &w.strike.strikes[&d];
        assert_eq!(s.aircraft.len(), 0, "never an aircraft");
        assert_eq!(s.target, if stuck { Some(b) } else { None });
    }
}

// spec: S-0519 claim 20 and S-0501 T2 row 2 (a strike aircraft in retreat is not exempt from falling back; the linked test)
#[test]
fn a_strike_aircraft_is_linked_until_it_leaves() {
    let (mut w, d, a, _) = scene(|_| {});
    designate(&mut w, d, a);
    let x = linked(&w, d)[0];
    assert!(strike::linked(&w, x));
    objects::remove(&mut w, x);
    assert!(!strike::linked(&w, x));
    assert_eq!(linked(&w, d).len(), 1);
}

// spec: S-0519 claim 19 (Several strikes on one building: cancelling one keeps the mark while the other holds it)
#[test]
fn two_strikes_share_a_mark() {
    let (mut w, d, a, _) = scene(|_| {});
    let d2 = quiet(&mut w, Kind::Vehicle, "DESIG", 0, at(-6, 2));
    designate(&mut w, d, a);
    designate(&mut w, d2, a);
    w.obj_mut(d).unwrap().target = None;
    testkit::run(&mut w, 1);
    assert!(strike::marked(&w, a));
    w.obj_mut(d2).unwrap().target = None;
    testkit::run(&mut w, 1);
    assert!(!strike::marked(&w, a));
}
