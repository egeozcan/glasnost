//! Claim tests for S-0417 (factory unloading) on synthetic data: invented
//! factory and vehicle types, no stock numbers.

mod econ_common;

use econ_common::*;
use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::behaviour::Door;
use glasnost_sim::econ::release::{self, Outcome};
use glasnost_sim::objects;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

/// Our test factory: a ground vehicle factory (vehicle factory flag) with
/// a 3 × 2 foundation and an invented door time.
fn ruleset_with(foundation: [i32; 2]) -> Ruleset {
    let mut r = ruleset();
    let mut b = factory("vehicle");
    b.foundation = Some(foundation);
    b.weapons_factory = Some(true);
    b.deploy_time = dec("0.01");
    r.registries.get_mut("buildings").unwrap().push("VF".into());
    r.sim.buildings.insert("VF".into(), b);
    let mut s = wheel(700);
    s.accelerates = Some(false);
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("STEADY".into());
    r.sim.vehicles.insert("STEADY".into(), s);
    r
}

fn new_unit(w: &mut World, kind: Kind, name: &str) -> ObjectId {
    let ti = w.rules().index_of(kind, name).unwrap();
    objects::create(w, kind, ti, Some(HouseId(0)))
}

fn top_left(w: &World, b: ObjectId) -> CellXY {
    w.obj(b).unwrap().pos.cell()
}

fn spawn_at(w: &mut World, kind: Kind, name: &str, c: CellXY) -> ObjectId {
    glasnost_sim::testkit::spawn(w, kind, name, 0, c, 0)
}

/// One record per processing of K: (tick, progress count after it).
fn run_recording(w: &mut World, k: ObjectId, ticks: i32) -> Vec<(i32, i32)> {
    let mut v = Vec::new();
    let mut last = w.obj(k).unwrap().beh.last;
    for _ in 0..ticks {
        step(w);
        let o = w.obj(k).unwrap();
        if o.beh.last != last && o.beh.current == Some(Behaviour::Unload) {
            v.push((w.tick, o.beh.progress));
        }
        last = o.beh.last;
        if o.beh.current != Some(Behaviour::Unload) {
            break;
        }
    }
    v
}

// spec: S-0417 claim 3 and F3 (E = (x0 + a − 1, y0 + b) from the eleventh pair; the 3 × 2 word gives (x0, y0 − 1))
#[test]
fn claim_3_apron() {
    let mut w = game(&ruleset_with([3, 2]));
    let k = building(&mut w, "VF", 0, 0, 0);
    let tl = top_left(&w, k);
    assert_eq!(release::apron(&w, k), Some(CellXY::new(tl.x, tl.y - 1)));
}

// spec: S-0417 Edge cases (quirks.factory_apron_short_list: (32767, 32767) for 2 × 1 and 1 × 2, wrapped; (0, 0) for 1 × 1, giving (x0 − 1, y0)) and the fixed alternative (switch off: the last pair)
#[test]
fn edge_short_list_pairs() {
    let wrap = |v: i32| i32::from(v as i16);
    for (f, on) in [
        ([2, 1], true),
        ([1, 2], true),
        ([1, 1], true),
        ([1, 1], false),
    ] {
        let mut r = ruleset_with(f);
        r.globals.quirks.factory_apron_short_list = on;
        let mut w = game(&r);
        let k = building(&mut w, "VF", 0, 0, 0);
        let tl = top_left(&w, k);
        let e = release::apron(&w, k).unwrap();
        let want = match (f, on) {
            ([1, 1], true) => CellXY::new(tl.x - 1, tl.y),
            (_, true) => CellXY::new(wrap(tl.x + 32766), wrap(tl.y + 32767)),
            _ => {
                let l = release::exit_list(1, 1, false, true);
                let (a, b) = *l.last().unwrap();
                CellXY::new(tl.x + a - 1, tl.y + b)
            }
        };
        assert_eq!(e, want, "{f:?} {on}");
    }
}

// spec: S-0417 Edge cases (the words with fewer than eleven entries: 2 × 1, 1 × 2, 1 × 1, the 3 × 3 refinery, 6 × 4 and 0 × 0)
#[test]
fn edge_short_list_words() {
    for (fw, fh, refinery, short) in [
        (2, 1, false, true),
        (1, 2, false, true),
        (1, 1, false, true),
        (3, 3, true, true),
        (6, 4, false, true),
        (0, 0, false, true),
        (2, 2, false, false),
        (3, 2, false, false),
        (3, 3, false, false),
        (4, 3, false, false),
    ] {
        let n = release::exit_list(fw, fh, refinery, true).len();
        assert_eq!(n < 11, short, "{fw} × {fh} {refinery}: {n}");
        // spec: S-0417 Edge cases (the special pair follows L: L = 10 for the 2 × 1 and 1 × 2 words, L < 10 for the other short words)
        if short {
            assert_eq!(
                n == 10,
                matches!((fw, fh), (2, 1) | (1, 2)),
                "{fw} × {fh}: {n}"
            );
        }
    }
}

// spec: S-0417 table T1 and claim 6 (68 points before the target, facing 64; 7 progress units a point, S-0207's end correction with m = 1)
#[test]
fn t1_exit_track() {
    let p = release::exit_track_points();
    assert_eq!(p.len(), 68);
    assert_eq!(p[0], (-670, -68, 64));
    assert_eq!(p[3], (-639, -65, 64));
    assert_eq!(p[4], (-630, -64, 64));
    assert_eq!(p[67], (0, -1, 64));
    let cost = 7 * (p.len() as i32 + 1) - glasnost_sim::tracks::end_correction(&p);
    assert_eq!(cost, 7 * 69 - 6);
}

// spec: S-0417 claims 1, 2 and 7 and table S1 (U2: stages, door, sending out not before the third processing, closing after the tether ends, guard); S-0417 claim 12 (one delay draw a processing)
#[test]
fn claim_2_stages() {
    let mut w = game(&ruleset_with([3, 2]));
    let k = building(&mut w, "VF", 0, 0, 0);
    let tank = new_unit(&mut w, Kind::Vehicle, "TANK");
    assert_eq!(release::release(&mut w, k, tank), Outcome::Released);
    // spec: S-0417 claims 1 and 2 (unload queued at release, current at the next per-tick start that finds the ready mark: the default range's delay 0 sets it every building turn)
    assert_eq!(w.obj(k).unwrap().beh.current, Some(Behaviour::Guard));
    assert_eq!(w.obj(k).unwrap().beh.queued, Some(Behaviour::Unload));
    assert_eq!(w.obj(tank).unwrap().contact, Some(k));
    assert!(release::tethered(&w, tank));
    let before = w.obj(k).unwrap().beh.last;
    step(&mut w);
    assert_eq!(w.obj(k).unwrap().beh.current, Some(Behaviour::Unload));
    let mut last = w.obj(k).unwrap().beh.last;
    let mut processings = i32::from(last != before);
    let mut sent = None;
    let mut report = None;
    for _ in 0..600 {
        step(&mut w);
        let o = w.obj(k).unwrap().clone();
        if o.beh.last != last {
            processings += 1;
            last = o.beh.last;
        }
        let on_track = w
            .obj(tank)
            .unwrap()
            .drive
            .as_ref()
            .is_some_and(|d| d.track.as_ref().is_some_and(|t| t.exit));
        if on_track && sent.is_none() {
            sent = Some(processings);
            // The door is at rest open when the vehicle is sent out.
            assert_eq!(o.beh.gate.door, Door::Open);
            // The contacted unit was forced to guard at the start; move is queued now.
            let u = w.obj(tank).unwrap();
            assert!(matches!(
                (u.beh.current, u.beh.queued),
                (Some(Behaviour::Guard), Some(Behaviour::Move)) | (Some(Behaviour::Move), _)
            ));
        }
        if report.is_none() && w.obj(tank).unwrap().contact.is_none() {
            report = Some(w.obj(tank).unwrap().pos.cell());
            // Closing has not begun before the report.
            assert!(!matches!(
                o.beh.gate.door,
                Door::Closing { .. } | Door::Closed
            ));
        }
        if o.beh.current == Some(Behaviour::Guard) {
            break;
        }
    }
    assert!(
        sent.is_some_and(|n| n >= 3),
        "sent out at processing {sent:?}"
    );
    // The clear report at E itself.
    assert_eq!(report, release::apron(&w, k));
    let o = w.obj(k).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Guard));
    assert_eq!(o.beh.gate.door, Door::Closed);
    assert!(!release::tethered(&w, tank));
}

// spec: S-0417 claim 9 (U9: a naval factory: two processings, the door opened by the first, guard queued by the second; no contact with the ship)
#[test]
fn claim_9_naval() {
    let mut r = ruleset();
    r.sim.buildings.get_mut("NAVYARD").unwrap().weapons_factory = Some(true);
    let mut w = game(&r);
    let k = building(&mut w, "NAVYARD", 0, 0, 0);
    let boat = new_unit(&mut w, Kind::Vehicle, "BOAT");
    if release::release(&mut w, k, boat) != Outcome::Released {
        // No water on the synthetic map: start the unloading by hand.
        let t = w.tick;
        w.obj_mut(k).unwrap().beh.force(Behaviour::Unload, t, true);
    }
    assert_eq!(w.obj(boat).unwrap().contact, None);
    let rec = run_recording(&mut w, k, 200);
    assert_eq!(rec.len(), 1, "{rec:?}");
    for _ in 0..200 {
        step(&mut w);
        if w.obj(k).unwrap().beh.current == Some(Behaviour::Guard) {
            break;
        }
    }
    assert_eq!(w.obj(k).unwrap().beh.current, Some(Behaviour::Guard));
    assert_ne!(w.obj(k).unwrap().beh.gate.door, Door::Closed);
}

// spec: S-0417 claim 4 (U5: a blocked apron: requests to E's occupants, then to occupied neighbours; the factory stays in apron clearing; neighbours get nothing when E is free)
#[test]
fn claim_4_blocked_apron() {
    let mut w = game(&ruleset_with([3, 2]));
    let k = building(&mut w, "VF", 0, 0, 0);
    let e = release::apron(&w, k).unwrap();
    let blocker = spawn_at(&mut w, Kind::Vehicle, "JEEP", e);
    let neighbour = spawn_at(&mut w, Kind::Vehicle, "JEEP", CellXY::new(e.x - 2, e.y));
    let tank = new_unit(&mut w, Kind::Vehicle, "TANK");
    assert_eq!(release::release(&mut w, k, tank), Outcome::Released);
    // Two processings: starting, then a blocked apron clearing.
    let mut n = 0;
    let mut last = w.obj(k).unwrap().beh.last;
    while n < 2 {
        step(&mut w);
        let l = w.obj(k).unwrap().beh.last;
        if l != last {
            n += 1;
            last = l;
        }
    }
    assert_eq!(w.obj(k).unwrap().beh.progress, 1);
    // The blocker was asked to scatter: it has somewhere to go.
    assert!(w.obj(blocker).unwrap().dest.is_some());
    // A vehicle two cells away is no apron neighbour.
    assert!(w.obj(neighbour).unwrap().dest.is_none());

    // E free, a vehicle in a neighbour: nothing is sent to it.
    let mut w = game(&ruleset_with([3, 2]));
    let k = building(&mut w, "VF", 0, 0, 0);
    let e = release::apron(&w, k).unwrap();
    let side = spawn_at(&mut w, Kind::Vehicle, "JEEP", CellXY::new(e.x - 1, e.y - 1));
    let tank = new_unit(&mut w, Kind::Vehicle, "TANK");
    assert_eq!(release::release(&mut w, k, tank), Outcome::Released);
    let rec = run_recording(&mut w, k, 80);
    assert!(rec.iter().any(|&(_, p)| p >= 2), "{rec:?}");
    assert!(w.obj(side).unwrap().dest.is_none());
}

// spec: S-0417 claim 6 (U7: an accelerating type keeps the speed fraction 0.5 on the exit track; a non-accelerating type uses the target fraction 1.0)
#[test]
fn claim_6_speed_fraction() {
    let one = glasnost_num::F64::from_i64(1, glasnost_num::Rounding::default());
    for (name, want) in [("TANK", glasnost_sim::consts::HALF), ("STEADY", one)] {
        let mut w = game(&ruleset_with([3, 2]));
        let k = building(&mut w, "VF", 0, 0, 0);
        let v = new_unit(&mut w, Kind::Vehicle, name);
        assert_eq!(release::release(&mut w, k, v), Outcome::Released);
        let mut seen = None;
        for _ in 0..400 {
            step(&mut w);
            let d = w.obj(v).unwrap().drive.clone().unwrap();
            if d.track.as_ref().is_some_and(|t| t.exit && t.next > 0) {
                seen = Some(d.rho);
                break;
            }
        }
        assert_eq!(seen, Some(want), "{name}");
    }
}

// spec: S-0417 claim 8 (U8: the saved rally is the destination at the report, with no further move queued; a later rally change is not read; without a rally, a scatter destination)
#[test]
fn claim_8_run_away() {
    let mut w = game(&ruleset_with([3, 2]));
    let k = building(&mut w, "VF", 0, 0, 0);
    let tl = top_left(&w, k);
    let a = Target::Cell(CellXY::new(tl.x + 6, tl.y + 6));
    let b = Target::Cell(CellXY::new(tl.x - 6, tl.y + 6));
    w.obj_mut(k).unwrap().guard_spot = Some(a);
    let tank = new_unit(&mut w, Kind::Vehicle, "TANK");
    assert_eq!(release::release(&mut w, k, tank), Outcome::Released);
    assert_eq!(w.obj(tank).unwrap().guard_spot, Some(a));
    // The rally changes after release.
    w.obj_mut(k).unwrap().guard_spot = Some(b);
    for _ in 0..600 {
        step(&mut w);
        if w.obj(tank).unwrap().contact.is_none() {
            break;
        }
    }
    let o = w.obj(tank).unwrap();
    assert_eq!(o.contact, None);
    assert_eq!(o.dest, Some(a));

    // No rally: the run-away scatter gives a destination.
    let mut w = game(&ruleset_with([3, 2]));
    let k = building(&mut w, "VF", 0, 0, 0);
    let tank = new_unit(&mut w, Kind::Vehicle, "TANK");
    assert_eq!(release::release(&mut w, k, tank), Outcome::Released);
    for _ in 0..600 {
        step(&mut w);
        if w.obj(tank).unwrap().contact.is_none() {
            break;
        }
    }
    let o = w.obj(tank).unwrap();
    assert_eq!(o.contact, None);
    assert!(o.dest.is_some());
}

// spec: S-0417 claim 10 (an infantry factory: no unloading stages or door; contact and tether end at the next cell-arrival update, with no run-away reply)
#[test]
fn claim_10_infantry() {
    let mut w = game(&ruleset());
    let (k, _) = barracks_with_rally(&mut w);
    let grunt = new_unit(&mut w, Kind::Infantry, "GRUNT");
    assert_eq!(release::release(&mut w, k, grunt), Outcome::Released);
    assert!(release::tethered(&w, grunt));
    assert_ne!(w.obj(k).unwrap().beh.current, Some(Behaviour::Unload));
    for _ in 0..100 {
        step(&mut w);
        if w.obj(grunt).unwrap().contact.is_none() {
            break;
        }
    }
    assert_eq!(w.obj(grunt).unwrap().contact, None);
    assert!(!release::tethered(&w, grunt));
    assert_eq!(w.obj(k).unwrap().beh.gate.door, Door::Closed);
}

// spec: S-0208 claim 23 (a tethered infantry unit reports clear on a walking arrival tick: snapped to a standing position, not at a cell crossing)
#[test]
fn s0208_claim_23_report_at_first_arrival() {
    let mut w = game(&ruleset());
    let (k, _) = barracks_with_rally(&mut w);
    let grunt = new_unit(&mut w, Kind::Infantry, "GRUNT");
    assert_eq!(release::release(&mut w, k, grunt), Outcome::Released);
    let standing: Vec<(i32, i32)> = (0..5).map(glasnost_sim::walk::standing_point).collect();
    let mut reported = false;
    for _ in 0..100 {
        step(&mut w);
        let o = w.obj(grunt).unwrap();
        if o.contact.is_none() {
            let off = (o.pos.x.rem_euclid(256), o.pos.y.rem_euclid(256));
            assert!(
                standing.contains(&off),
                "the report tick is an arrival tick"
            );
            reported = true;
            break;
        }
    }
    assert!(reported);
}

// spec: S-0215 claim 11 (vehicles: in contact with a vehicle-factory building and the queued behaviour neither move nor enter; in contact with nothing on cell (x, y + 1) of one)
#[test]
fn s0215_claim_11_vehicle_factory_readiness() {
    let mut w = game(&ruleset_with([3, 2]));
    let k = building(&mut w, "VF", 0, 0, 0);
    let tl = top_left(&w, k);
    let v = spawn_at(&mut w, Kind::Vehicle, "JEEP", CellXY::new(tl.x, tl.y + 1));
    assert!(release::holds_back(&w, v));
    assert!(!glasnost_sim::process::ready(&w, v));
    w.obj_mut(v).unwrap().contact = Some(k);
    w.obj_mut(v).unwrap().beh.queued = Some(Behaviour::Guard);
    assert!(release::holds_back(&w, v));
    w.obj_mut(v).unwrap().beh.queued = Some(Behaviour::Move);
    assert!(!release::holds_back(&w, v));
    let other = spawn_at(
        &mut w,
        Kind::Vehicle,
        "JEEP",
        CellXY::new(tl.x + 1, tl.y + 1),
    );
    assert!(!release::holds_back(&w, other));
}

// spec: S-0213 claims 8 and 9 (a tethered actor ignores stop and scatter orders) and table T3 (b) (a behaviour order keeps the link at a building that is not of the unloading kind)
#[test]
fn s0213_tethered_orders() {
    let mut w = game(&ruleset_with([3, 2]));
    ready(&mut w);
    let k = building(&mut w, "VF", 0, 0, 0);
    let tank = new_unit(&mut w, Kind::Vehicle, "TANK");
    assert_eq!(release::release(&mut w, k, tank), Outcome::Released);
    force(&mut w, 0, Order::Stop { actor: tank });
    step(&mut w);
    assert_eq!(w.obj(tank).unwrap().contact, Some(k));
    assert!(release::tethered(&w, tank));
}

// spec: S-0417 claims 2 and 7 (the unloading state survives a save and load: the state hash is unchanged in the middle of an unloading)
#[test]
fn save_round_trip_mid_unloading() {
    let mut w = game(&ruleset_with([3, 2]));
    let k = building(&mut w, "VF", 0, 0, 0);
    let tank = new_unit(&mut w, Kind::Vehicle, "TANK");
    assert_eq!(release::release(&mut w, k, tank), Outcome::Released);
    for _ in 0..60 {
        step(&mut w);
    }
    assert_eq!(w.obj(k).unwrap().beh.current, Some(Behaviour::Unload));
    let bytes = glasnost_sim::save::save(&w);
    let mut back =
        glasnost_sim::save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    assert_eq!(
        glasnost_sim::save::state_hash(&back),
        glasnost_sim::save::state_hash(&w)
    );
    for _ in 0..200 {
        step(&mut w);
        step(&mut back);
    }
    assert_eq!(
        glasnost_sim::save::state_hash(&back),
        glasnost_sim::save::state_hash(&w)
    );
}

/// An infantry factory with a rally point six cells off; returns the
/// factory and the rally point.
fn barracks_with_rally(w: &mut World) -> (ObjectId, Target) {
    let k = building(w, "BARRACKS", 0, 0, 0);
    let tl = top_left(w, k);
    let rally = Target::Cell(CellXY::new(tl.x + 6, tl.y + 6));
    w.obj_mut(k).unwrap().guard_spot = Some(rally);
    (k, rally)
}

// spec: S-0417 claim 10 (U10 a: with a rally point off its cell, the placement fall-back makes the rally the destination; then the guard spot becomes the rally, move is queued and the destination is the exit cell; contact and tether; the second leg to the rally after the first falls back at the exit cell)
// spec: S-0215 claim 13 (the guard-spot row)
#[test]
fn claim_10_infantry_two_legs() {
    let mut w = game(&ruleset());
    let (k, rally) = barracks_with_rally(&mut w);
    let grunt = new_unit(&mut w, Kind::Infantry, "GRUNT");
    let rng = w.rng.clone();
    assert_eq!(release::release(&mut w, k, grunt), Outcome::Released);
    // spec: S-0417 claim 12 (turning the guard spot into a destination and the contact exchange take no draw)
    assert_eq!(w.rng, rng);
    let o = w.obj(grunt).unwrap();
    assert_eq!(o.guard_spot, Some(rally));
    let Some(Target::Cell(exit)) = o.dest else {
        panic!("a cell destination: {:?}", o.dest)
    };
    assert_ne!(Target::Cell(exit), rally);
    assert_eq!(o.beh.current, None);
    assert_eq!(o.beh.queued, Some(Behaviour::Move));
    assert!(release::tethered(&w, grunt));
    let mut second = false;
    for _ in 0..400 {
        step(&mut w);
        let o = w.obj(grunt).unwrap();
        if o.dest == Some(rally) && o.guard_spot.is_none() {
            second = true;
            assert_eq!(
                o.contact, None,
                "the clear report came at the exit cell first"
            );
            break;
        }
    }
    assert!(second, "the second leg to the rally point");
}

// spec: S-0417 claim 10 (U10 c: a jumpjet or teleporting infantry type heads straight for the rally point with an empty guard spot)
#[test]
fn claim_10_teleporting_infantry_goes_straight() {
    let mut r = ruleset();
    r.sim.infantry.get_mut("GRUNT").unwrap().teleporter = Some(true);
    let mut w = game(&r);
    let (k, rally) = barracks_with_rally(&mut w);
    let grunt = new_unit(&mut w, Kind::Infantry, "GRUNT");
    assert_eq!(release::release(&mut w, k, grunt), Outcome::Released);
    let o = w.obj(grunt).unwrap();
    assert_eq!(o.dest, Some(rally));
    assert_eq!(o.guard_spot, None);
    assert!(release::tethered(&w, grunt));
}

// spec: S-0417 claim 10 and S-0404 claim 13 (no rally point, or one in the unit's own cell: the release gives no destination; contact and tether)
#[test]
fn claim_10_no_rally_no_destination() {
    for own_cell in [false, true] {
        let mut w = game(&ruleset());
        let k = building(&mut w, "BARRACKS", 0, 0, 0);
        let grunt = new_unit(&mut w, Kind::Infantry, "GRUNT");
        if own_cell {
            // The rally point is the cell the unit is about to appear in:
            // release once in a scratch world to learn it.
            let mut scratch = w.clone();
            assert_eq!(release::release(&mut scratch, k, grunt), Outcome::Released);
            let c = scratch.obj(grunt).unwrap().pos.cell();
            w.obj_mut(k).unwrap().guard_spot = Some(Target::Cell(c));
        }
        assert_eq!(release::release(&mut w, k, grunt), Outcome::Released);
        let o = w.obj(grunt).unwrap();
        // spec: S-0417 claim 10 (the saved guard spot is K's rally point, if any; the guard-spot row gives no destination for the own cell)
        assert_eq!(
            o.guard_spot,
            w.obj(k).unwrap().guard_spot,
            "own cell {own_cell}"
        );
        assert_eq!(o.dest, None, "own cell {own_cell}");
        // spec: S-0417 claim 10 (no move is queued; the unit rests in guard)
        assert_ne!(o.beh.seen(), Some(Behaviour::Move), "own cell {own_cell}");
        assert_eq!(o.contact, Some(k));
        assert!(release::tethered(&w, grunt));
    }
}

// spec: S-0417 claim 14 (an infantry factory has n = max(d, 1) contact places; while each is held, a further release is busy)
#[test]
fn claim_14_infantry_contact_places() {
    for (docks, released) in [(None, 1), (Some(0), 1), (Some(2), 2)] {
        let mut r = ruleset();
        r.sim
            .buildings
            .get_mut("BARRACKS")
            .unwrap()
            .econ
            .number_of_docks = docks;
        let mut w = game(&r);
        let (k, _) = barracks_with_rally(&mut w);
        let mut n = 0;
        for _ in 0..3 {
            let g = new_unit(&mut w, Kind::Infantry, "GRUNT");
            match release::release(&mut w, k, g) {
                Outcome::Released => n += 1,
                Outcome::Busy => break,
                other => panic!("docks {docks:?}: {other:?}"),
            }
        }
        assert_eq!(n, released, "docks {docks:?}");
    }
}

// spec: S-0417 claim 11 (U12: docked at K, contact and a tether request on the same tick; a type that need not return to a dock is tethered, one that must return refuses; contact in both cases)
#[test]
fn claim_11_aircraft_tether() {
    for bound in [false, true] {
        let mut r = ruleset();
        r.sim.aircraft.get_mut("PLANE").unwrap().econ.airport_bound = Some(bound);
        let mut w = game(&r);
        let k = building(&mut w, "AIRPAD", 0, 0, 0);
        let plane = new_unit(&mut w, Kind::Aircraft, "PLANE");
        assert_eq!(release::release(&mut w, k, plane), Outcome::Released);
        let o = w.obj(plane).unwrap();
        assert_eq!(o.contact, Some(k), "bound {bound}");
        assert_eq!(release::tethered(&w, plane), !bound, "bound {bound}");
    }
}

/// Calls `each` with the tick of every processing of K over `ticks` ticks.
fn processings(w: &mut World, k: ObjectId, ticks: i32, mut each: impl FnMut(&World, i32)) {
    let mut last = w.obj(k).unwrap().beh.last;
    for _ in 0..ticks {
        let t = w.tick;
        step(w);
        let l = w.obj(k).unwrap().beh.last;
        if l != last {
            each(w, t);
            last = l;
        }
    }
}

// spec: S-0417 claim 13 (U11 a: an idle vehicle factory in guard makes no apron test at the first processing after guard starts, and sends claim 4's request at each later one while E is occupied; delay 3 × D_r + U(0, 2))
#[test]
fn claim_13_idle_apron_clearing() {
    let mut w = game(&ruleset_with([3, 2]));
    let k = building(&mut w, "VF", 0, 0, 0);
    let e = release::apron(&w, k).unwrap();
    let t = w.tick;
    w.obj_mut(k).unwrap().beh.force(Behaviour::Guard, t, true);
    let jeep = spawn_at(&mut w, Kind::Vehicle, "JEEP", e);
    let dr = w.rules().behaviour(Behaviour::Guard).rate_delay;
    let mut seen = Vec::new();
    processings(&mut w, k, 6 * dr + 20, |w, t| {
        seen.push((t, w.obj(jeep).unwrap().dest.is_some()));
    });
    assert!(seen.len() >= 2, "{seen:?}");
    assert!(!seen[0].1, "no request at the first processing");
    assert!(seen[1].1, "a request at the second");
    let gap = seen[1].0 - seen[0].0;
    assert!((3 * dr..=3 * dr + 2).contains(&gap), "{gap} for D_r {dr}");
}

// spec: S-0417 claim 13 (no apron test while the factory is armed or has the stupid-guard-mode flag, nor for a building without the vehicle factory flag)
#[test]
fn claim_13_no_clearing() {
    for stupid in [false, true] {
        let mut r = ruleset_with([3, 2]);
        r.sim.buildings.get_mut("VF").unwrap().stupid_guard_mode = Some(stupid);
        let mut w = game(&r);
        let k = building(&mut w, "VF", 0, 0, 0);
        let e = release::apron(&w, k).unwrap();
        let t = w.tick;
        w.obj_mut(k).unwrap().beh.force(Behaviour::Guard, t, true);
        let jeep = spawn_at(&mut w, Kind::Vehicle, "JEEP", e);
        let dr = w.rules().behaviour(Behaviour::Guard).rate_delay;
        testkit_run(&mut w, 6 * dr + 20);
        assert_eq!(w.obj(jeep).unwrap().dest.is_some(), !stupid);
    }
}

// spec: S-0417 claim 13, Refinery boundary (Q-0371) (apron clearing is the vehicle factory's: an idle refinery without the vehicle factory flag requests nothing of a unit resting on its docking cell; a refinery that is also a vehicle factory tests E only, not its docking cell as such)
#[test]
fn claim_13_refinery_boundary() {
    for vehicle_factory in [false, true] {
        let mut r = ruleset_with([3, 2]);
        let mut b = r.sim.buildings["VF"].clone();
        b.weapons_factory = Some(vehicle_factory);
        b.econ.refinery = Some(true);
        b.econ.dock_unload = Some(true);
        r.registries.get_mut("buildings").unwrap().push("RF".into());
        r.sim.buildings.insert("RF".into(), b);
        let mut w = game(&r);
        let k = building(&mut w, "RF", 0, 0, 0);
        let tl = top_left(&w, k);
        let dock = CellXY::new(tl.x + 3, tl.y + 1);
        let t = w.tick;
        w.obj_mut(k).unwrap().beh.force(Behaviour::Guard, t, true);
        let resting = spawn_at(&mut w, Kind::Vehicle, "JEEP", dock);
        let on_e = release::apron(&w, k)
            .filter(|&e| e != dock)
            .map(|e| spawn_at(&mut w, Kind::Vehicle, "JEEP", e));
        let dr = w.rules().behaviour(Behaviour::Guard).rate_delay;
        testkit_run(&mut w, 6 * dr + 20);
        assert!(
            w.obj(resting).unwrap().dest.is_none(),
            "no request to the unit on the docking cell (vehicle factory {vehicle_factory})"
        );
        let j = on_e.expect("E is not the docking cell");
        assert_eq!(w.obj(j).unwrap().dest.is_some(), vehicle_factory);
    }
}

fn testkit_run(w: &mut World, n: i32) {
    for _ in 0..n {
        step(w);
    }
}

/// VF with idle body range `idle` (S-0108 Import mapping, `AnimIdle`).
fn with_idle(idle: [i32; 3], waits: bool) -> Ruleset {
    let mut r = ruleset_with([3, 2]);
    r.sim.buildings.get_mut("VF").unwrap().econ.anim_idle = Some(idle);
    r.globals.quirks.factory_unload_waits_ready_mark = waits;
    r
}

// spec: S-0417 claim 2 and Edge cases (quirks.factory_unload_waits_ready_mark: a positive-delay range of one stage never sets the ready mark, so the queued unload never becomes current and the vehicle stays in contact at its appear point; off, queueing unload sets the mark and it starts at the next per-tick start)
#[test]
fn claim_2_unload_waits_ready_mark() {
    for waits in [true, false] {
        let mut w = game(&with_idle([0, 1, 5], waits));
        let k = building(&mut w, "VF", 0, 0, 0);
        step(&mut w);
        w.obj_mut(k).unwrap().beh.ready_mark = false;
        let tank = new_unit(&mut w, Kind::Vehicle, "TANK");
        assert_eq!(release::release(&mut w, k, tank), Outcome::Released);
        let at = w.obj(tank).unwrap().pos;
        assert_eq!(w.obj(k).unwrap().beh.ready_mark, !waits);
        step(&mut w);
        if waits {
            for _ in 0..100 {
                step(&mut w);
            }
            let o = w.obj(k).unwrap();
            assert_eq!(o.beh.current, Some(Behaviour::Guard));
            assert_eq!(o.beh.queued, Some(Behaviour::Unload));
            assert_eq!(w.obj(tank).unwrap().contact, Some(k));
            assert_eq!(w.obj(tank).unwrap().pos, at);
        } else {
            assert_eq!(w.obj(k).unwrap().beh.current, Some(Behaviour::Unload));
        }
    }
}

// spec: S-0417 claim 2 (Q-0222: closing queues guard without a start; a zero-delay body range sets the mark every building turn, so guard becomes current at the second per-tick start of the closing tick, the tick of the processing that queued it)
#[test]
fn claim_2_guard_starts_at_the_second_start() {
    let mut w = game(&with_idle([0, 1, 0], true));
    let k = building(&mut w, "VF", 0, 0, 0);
    let tank = new_unit(&mut w, Kind::Vehicle, "TANK");
    assert_eq!(release::release(&mut w, k, tank), Outcome::Released);
    let mut started = false;
    for _ in 0..600 {
        step(&mut w);
        let o = w.obj(k).unwrap();
        if o.beh.current == Some(Behaviour::Unload) {
            started = true;
        }
        if started && o.beh.current == Some(Behaviour::Guard) {
            let (processed, _) = o.beh.last.expect("a processing");
            assert_eq!(
                o.beh.start_tick, processed,
                "guard starts in the closing tick"
            );
            assert_eq!(o.beh.queued, None);
            return;
        }
    }
    panic!("guard never started");
}

// spec: S-0417 claim 2 and Edge cases (quirks.factory_unload_waits_ready_mark: once unload is current, a range that never sets the mark never lets the queued guard start; off, the closing processing sets it)
#[test]
fn claim_2_guard_waits_ready_mark() {
    for waits in [true, false] {
        let mut w = game(&with_idle([0, 1, 5], waits));
        let k = building(&mut w, "VF", 0, 0, 0);
        let tank = new_unit(&mut w, Kind::Vehicle, "TANK");
        assert_eq!(release::release(&mut w, k, tank), Outcome::Released);
        // The mark left by construction starts the unloading.
        w.obj_mut(k).unwrap().beh.ready_mark = true;
        let mut queued_guard = false;
        for _ in 0..600 {
            step(&mut w);
            let o = w.obj(k).unwrap();
            queued_guard |= o.beh.queued == Some(Behaviour::Guard);
            if o.beh.current == Some(Behaviour::Guard) {
                break;
            }
        }
        let o = w.obj(k).unwrap();
        if waits {
            assert!(queued_guard);
            assert_eq!(o.beh.current, Some(Behaviour::Unload));
            assert_eq!(o.beh.queued, Some(Behaviour::Guard));
        } else {
            assert_eq!(o.beh.current, Some(Behaviour::Guard));
        }
    }
}

// spec: S-0417 Edge cases (an apron that is not a legal map cell, here the 2 × 1 word's (32767, 32767) pair: the apron counts as free, the exit instruction is refused, u stays where it appeared and reports clear at once, so K closes; under both states of quirks.factory_apron_short_list)
#[test]
fn edge_illegal_apron() {
    for short in [true, false] {
        let mut r = ruleset_with([2, 1]);
        r.globals.quirks.factory_apron_short_list = short;
        let mut w = game(&r);
        let k = building(&mut w, "VF", 0, 0, 0);
        let illegal = !w.cells.footprint().is_legal(release::apron(&w, k).unwrap());
        assert_eq!(
            illegal, short,
            "the last pair of the 2 x 1 list is a legal cell"
        );
        if !illegal {
            continue;
        }
        let tank = new_unit(&mut w, Kind::Vehicle, "TANK");
        assert_eq!(release::release(&mut w, k, tank), Outcome::Released);
        let at = w.obj(tank).unwrap().pos;
        let mut cleared = false;
        for _ in 0..200 {
            step(&mut w);
            if w.obj(tank).unwrap().contact.is_none() {
                cleared = true;
                break;
            }
        }
        assert!(cleared, "the clear report");
        assert_eq!(
            w.obj(tank).unwrap().pos.cell(),
            at.cell(),
            "u stays where it appeared"
        );
        assert!(!release::tethered(&w, tank));
    }
}

// spec: S-0417 claim 5 (a hovering vehicle gets the destination (x0 + 4, y0 + 1) and moves by its own locomotor); S-0503 claim 4 (ρ = ½ after the exit instruction, kept until a listed reset); S-0503 claim 15 (a clear heading-point choice is a cell-arrival update at the actual position, so the factory gets its clear report and closes; Q-0693 answered)
#[test]
fn hover_exit_reports_clear_and_factory_closes() {
    let mut r = ruleset_with([3, 2]);
    let hover = glasnost_rules::sim::UnitType {
        locomotion_class: Some(glasnost_rules::sim::LocomotionClass::Hover),
        locomotor: Some(glasnost_rules::sim::Locomotor::Hover),
        ..wheel(700)
    };
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("HOVR".into());
    r.sim.vehicles.insert("HOVR".into(), hover);
    let mut w = game(&r);
    let k = building(&mut w, "VF", 0, 0, 0);
    let c = top_left(&w, k);
    let h = new_unit(&mut w, Kind::Vehicle, "HOVR");
    assert_eq!(release::release(&mut w, k, h), Outcome::Released);
    let half = glasnost_sim::consts::HALF;
    let mut sent = false;
    let mut half_seen = false;
    let mut report = false;
    for _ in 0..1500 {
        step(&mut w);
        let u = w.obj(h).unwrap();
        let dest = Some(Target::Cell(CellXY::new(c.x + 4, c.y + 1)));
        if u.dest == dest {
            sent = true;
        }
        if sent && u.hover.as_ref().unwrap().rho == half {
            half_seen = true;
        }
        report |= u.contact.is_none();
        let o = w.obj(k).unwrap();
        if report && o.beh.current == Some(Behaviour::Guard) && o.beh.gate.door == Door::Closed {
            break;
        }
    }
    assert!(sent, "sent to (x0 + 4, y0 + 1)");
    assert!(half_seen, "ρ = ½ after the exit instruction");
    assert!(report, "the hover reports clear");
    let o = w.obj(k).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Guard), "the factory closes");
    assert_eq!(o.beh.gate.door, Door::Closed);
    assert!(!release::tethered(&w, h));
    // A second hover can leave after it.
    let h2 = new_unit(&mut w, Kind::Vehicle, "HOVR");
    assert_eq!(release::release(&mut w, k, h2), Outcome::Released);
}

// spec: S-0417 claim 5 and F4 (the hover's destination (x0 + 4, y0 + 1) can lie on K's own foundation); S-0209 claim 5 (row VP3: a building the vehicle is in contact with gives none on a cell that is not one of its blocking cells, S-0209 Terms); S-0503 claim 15 (normal completion in the contacted factory's cell is a cell-arrival update); S-0417 claims 7 and 2 (the report there ends the tether, K closes)
#[test]
fn hover_exit_across_own_foundation_reports_clear() {
    // An invented 5 × 3 factory whose first two columns block (an
    // impassable-rows value of 2): the fixed hover destination is a foundation
    // cell the contacted hover may enter, and so are the cells between.
    let mut r = ruleset_with([5, 3]);
    r.sim
        .buildings
        .get_mut("VF")
        .unwrap()
        .number_impassable_rows = Some(2);
    let hover = glasnost_rules::sim::UnitType {
        locomotion_class: Some(glasnost_rules::sim::LocomotionClass::Hover),
        locomotor: Some(glasnost_rules::sim::Locomotor::Hover),
        ..wheel(700)
    };
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("HOVR".into());
    r.sim.vehicles.insert("HOVR".into(), hover);
    let mut w = game(&r);
    let k = building(&mut w, "VF", 0, 0, 0);
    let c = top_left(&w, k);
    let dest = CellXY::new(c.x + 4, c.y + 1);
    let h = new_unit(&mut w, Kind::Vehicle, "HOVR");
    assert_eq!(release::release(&mut w, k, h), Outcome::Released);
    // Row VP3: in contact, a non-blocking cell of K gives no blocker; a
    // blocking cell still does.
    let level = glasnost_sim::occupancy::reference_level(&w, h);
    assert_eq!(
        glasnost_sim::occupancy::vehicle_outcome(&w, h, dest, level),
        glasnost_sim::occupancy::Outcome::Clear
    );
    assert_eq!(
        glasnost_sim::occupancy::vehicle_outcome(&w, h, CellXY::new(c.x + 1, c.y + 1), level),
        glasnost_sim::occupancy::Outcome::Blocked
    );
    let mut sent = false;
    let mut report_at = None;
    for _ in 0..1500 {
        step(&mut w);
        let u = w.obj(h).unwrap();
        sent |= u.dest == Some(Target::Cell(dest));
        if report_at.is_none() && u.contact.is_none() {
            report_at = Some(u.pos.cell());
        }
        let o = w.obj(k).unwrap();
        if report_at.is_some()
            && o.beh.current == Some(Behaviour::Guard)
            && o.beh.gate.door == Door::Closed
        {
            break;
        }
    }
    assert!(sent, "sent to (x0 + 4, y0 + 1)");
    assert_eq!(
        report_at,
        Some(dest),
        "the report at completion in K's cell"
    );
    assert!(!release::tethered(&w, h));
    let o = w.obj(k).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Guard), "the factory closes");
    assert_eq!(o.beh.gate.door, Door::Closed);
    // The next vehicle is not refused as busy, and it leaves in its turn:
    // the first one does not hold the apron (claim 8's reply moves it on).
    let h2 = new_unit(&mut w, Kind::Vehicle, "HOVR");
    assert_eq!(release::release(&mut w, k, h2), Outcome::Released);
    let mut second = false;
    for _ in 0..1500 {
        step(&mut w);
        second |= w.obj(h2).unwrap().contact.is_none();
        let o = w.obj(k).unwrap();
        if second && o.beh.current == Some(Behaviour::Guard) && o.beh.gate.door == Door::Closed {
            break;
        }
    }
    assert!(second, "the second hover reports clear");
    let o = w.obj(k).unwrap();
    assert_eq!(
        o.beh.current,
        Some(Behaviour::Guard),
        "the factory closes again"
    );
    assert_eq!(o.beh.gate.door, Door::Closed);
}

// spec: S-0207 claim 22 (an own-locomotor teleport of any object kind has its arrival update in the jump tick); S-0417 claim 10 and S-0208 claim 23 (a tethered infantry unit reports clear on a cell-arrival update; no run-away reply); Q-0697 (interim: the jump's update is a tethered teleporting infantry unit's report); S-0417 claim 14 (the place is free again)
#[test]
fn claim_10_teleporting_infantry_reports_at_its_jump() {
    let mut r = ruleset();
    let g = r.sim.infantry.get_mut("GRUNT").unwrap();
    g.teleporter = Some(true);
    g.locomotor = Some(glasnost_rules::sim::Locomotor::Teleport);
    let mut w = game(&r);
    let (k, rally) = barracks_with_rally(&mut w);
    let Target::Cell(rc) = rally else {
        unreachable!()
    };
    let grunt = new_unit(&mut w, Kind::Infantry, "GRUNT");
    assert_eq!(release::release(&mut w, k, grunt), Outcome::Released);
    assert!(release::tethered(&w, grunt));
    let mut jumped = false;
    for _ in 0..200 {
        step(&mut w);
        let o = w.obj(grunt).unwrap();
        if o.pos.cell() == rc {
            jumped = true;
            assert_eq!(o.contact, None, "the report in the jump tick");
            assert!(!release::tethered(&w, grunt));
            break;
        }
        assert_eq!(o.contact, Some(k), "contact until the jump");
    }
    assert!(jumped, "it jumps to the rally point");
    let next = new_unit(&mut w, Kind::Infantry, "GRUNT");
    assert_eq!(release::release(&mut w, k, next), Outcome::Released);
}

// spec: S-0417 claims 5, 7 and 8, S-0212 claim 24 and departure D3 (a hover whose fixed destination holds another building is sent to the nearby-cell substitute, which becomes its destination: it reports clear, leaves the apron and comes to rest there)
#[test]
fn hover_exit_cell_with_building_adopts_substitute() {
    let mut r = ruleset_with([3, 2]);
    let hover = glasnost_rules::sim::UnitType {
        locomotion_class: Some(glasnost_rules::sim::LocomotionClass::Hover),
        locomotor: Some(glasnost_rules::sim::Locomotor::Hover),
        ..wheel(700)
    };
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("HOVR".into());
    r.sim.vehicles.insert("HOVR".into(), hover);
    let mut w = game(&r);
    let k = building(&mut w, "VF", 0, 0, 0);
    let c = top_left(&w, k);
    let exit = CellXY::new(c.x + 4, c.y + 1);
    let other = glasnost_sim::testkit::spawn(&mut w, Kind::Building, "VF", 0, exit, 0);
    let o0 = top_left(&w, other);
    let foot: Vec<CellXY> = (0..2)
        .flat_map(|dy| (0..3).map(move |dx| CellXY::new(o0.x + dx, o0.y + dy)))
        .collect();
    assert!(foot.contains(&exit));
    let h = new_unit(&mut w, Kind::Vehicle, "HOVR");
    assert_eq!(release::release(&mut w, k, h), Outcome::Released);
    let mut substitute = None;
    let mut reported = false;
    let mut rested = false;
    // Well before the give-up of S-0212 claim 23a, which would otherwise end
    // an orbit around the unreachable building cell much later.
    for _ in 0..300 {
        step(&mut w);
        let u = w.obj(h).unwrap();
        if let Some(Target::Cell(d)) = u.dest
            && d != exit
        {
            substitute = Some(d);
        }
        reported |= u.contact.is_none();
        let o = w.obj(k).unwrap();
        if reported
            && u.dest.is_none()
            && o.beh.current == Some(Behaviour::Guard)
            && o.beh.gate.door == Door::Closed
        {
            rested = true;
            break;
        }
    }
    let s = substitute.expect("the substitute replaces the destination");
    assert!(
        !foot.contains(&s),
        "the substitute is not a cell of the other building"
    );
    assert!(reported, "the hover reports clear");
    assert!(rested, "the move ends and the factory closes");
    assert_eq!(
        w.obj(h).unwrap().pos.cell(),
        s,
        "at rest in the substitute cell"
    );
    assert!(!release::tethered(&w, h));
}
