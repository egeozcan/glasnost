//! Claim tests of S-0309 revision 129 (Q-2841 answered), claim 31's unit
//! that a vehicle factory has not yet sent out: the factory's later
//! release puts its new unit at the exit point all the same, and the new
//! unit's contact takes the factory's one contact place, so the unit left
//! standing there loses its contact and tether link, keeps its behaviours
//! and stays (S-0404 r26 claim 13). Synthetic rules only: invented types
//! and numbers, no game data. Ledger: `docs/m8-claims/intake120.md`.

mod econ_common;
mod structures_common;

use glasnost_rules::schema::Ruleset;
use glasnost_sim::behaviour::Door;
use glasnost_sim::econ::release::{self, Outcome};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::ownership::{self, Cause};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{objects, testkit, tick};

const A: HouseId = HouseId(0);
const B: HouseId = HouseId(1);

/// The production ruleset with VF, a ground vehicle factory (vehicle
/// factory flag, 3 × 2 foundation, an invented door time), one docking
/// place (the imported count absent).
fn vf_rules() -> Ruleset {
    let mut r = econ_common::ruleset();
    let mut b = econ_common::factory("vehicle");
    b.foundation = Some([3, 2]);
    b.weapons_factory = Some(true);
    b.deploy_time = econ_common::dec("0.01");
    r.registries.get_mut("buildings").unwrap().push("VF".into());
    r.sim.buildings.insert("VF".into(), b);
    r
}

fn beh(w: &World, id: ObjectId) -> (Option<Behaviour>, Option<Behaviour>) {
    let o = w.obj(id).unwrap();
    (o.beh.current, o.beh.queued)
}

fn door(w: &World, k: ObjectId) -> Door {
    w.obj(k).unwrap().beh.gate.door
}

fn on_exit_track(w: &World, u: ObjectId) -> bool {
    w.obj(u)
        .unwrap()
        .drive
        .as_ref()
        .is_some_and(|d| d.track.as_ref().is_some_and(|t| t.exit))
}

/// The door leaves *open* only inside an unloading of K, at its closing
/// stage (S-0417 table S1): K's current behaviour is then *unload*.
fn check_door(w: &World, k: ObjectId, was_open: &mut bool) {
    let open = door(w, k) == Door::Open;
    if *was_open && !open {
        assert_eq!(
            beh(w, k).0,
            Some(Behaviour::Unload),
            "closed only by an unloading"
        );
    }
    *was_open = open;
}

// spec: S-0309 claim 31 (the factory's releases go on while the unit it has not sent out stands at its exit point: the new unit appears in that cell, its price is not refunded, its contact takes the factory's one place, and the unit standing there loses its contact and tether link, keeps its behaviours and stays until an order moves it; the door stays open until an unloading reaches its closing stage) and S-0404 claim 13; scenario A19 part k
#[test]
// The test prints its Open-question observation (a test harness line, not
// simulation output).
#[allow(clippy::disallowed_macros)]
fn a_later_release_takes_the_stranded_units_contact() {
    let r = vf_rules();
    let mut w = econ_common::game(&r);
    econ_common::ready(&mut w);
    let k = econ_common::building(&mut w, "VF", 0, 0, 0);
    let ti = w.rules().index_of(Kind::Vehicle, "TANK").unwrap();
    let v1 = objects::create(&mut w, Kind::Vehicle, ti, Some(A));
    assert_eq!(release::release(&mut w, k, v1), Outcome::Released);
    let p = w.obj(v1).unwrap().pos;
    // The owner change, before V1 is put on the exit track.
    ownership::change(&mut w, k, Some(B), Cause::Trigger);
    assert_eq!(structures_common::owner(&w, v1), Some(B), "V1 follows K");
    assert_eq!(beh(&w, k).0, Some(Behaviour::Guard));
    for _ in 0..30 {
        tick::step(&mut w);
    }
    assert_eq!(w.obj(v1).unwrap().contact, Some(k));
    assert!(release::tethered(&w, v1));
    assert!(!on_exit_track(&w, v1));

    // The player orders V2.
    econ_common::set_credits(&mut w, 1, 5000);
    let jeep = econ_common::t(&w, Kind::Vehicle, "JEEP");
    assert!(econ_common::issue(&mut w, 1, Order::Build { what: jeep }));
    let mut was_open = door(&w, k) == Door::Open;
    let mut v2 = None;
    let mut v1_before = beh(&w, v1);
    for _ in 0..2000 {
        v1_before = beh(&w, v1);
        tick::step(&mut w);
        check_door(&w, k, &mut was_open);
        v2 = w
            .objects
            .values()
            .find(|o| o.kind == Kind::Vehicle && o.id != v1 && o.on_map && o.owner == Some(B))
            .map(|o| o.id);
        if v2.is_some() {
            break;
        }
        assert_eq!(
            w.obj(v1).unwrap().contact,
            Some(k),
            "V1 holds the contact until V2"
        );
    }
    let v2 = v2.expect("V2 appears");
    assert_eq!(econ_common::credits(&w, 1), 5000 - 500, "no refund");
    assert_eq!(w.obj(v2).unwrap().pos.cell(), p.cell(), "V2 in V1's cell");
    assert_eq!(w.obj(v2).unwrap().contact, Some(k), "V2 holds K's contact");
    assert!(release::tethered(&w, v2));
    let o1 = w.obj(v1).unwrap();
    assert_eq!(o1.contact, None, "V1 loses its contact");
    assert_eq!(o1.econ.exit, None, "and its tether link");
    assert_eq!(beh(&w, v1), v1_before, "V1 keeps its behaviours");

    // 300 ticks later the player moves V1 away.
    let mut v2_out_first = false;
    let mut v2_left_first = false;
    for _ in 0..300 {
        tick::step(&mut w);
        check_door(&w, k, &mut was_open);
        let o1 = w.obj(v1).unwrap();
        assert_eq!(o1.pos, p, "V1 stays at the exit point");
        assert_eq!(o1.contact, None);
        assert_eq!(o1.econ.exit, None);
        v2_out_first |= on_exit_track(&w, v2);
        v2_left_first |= w.obj(v2).unwrap().pos.cell() != p.cell();
    }
    // Recorded for S-0309's Open question (whether V1 holds up V2's exit
    // track); not asserted.
    println!(
        "V2 put on its exit track before V1 left: {v2_out_first}; V2 left the exit point's cell: {v2_left_first}; K: {:?}",
        beh(&w, k)
    );
    let far = Target::Cell(testkit::mid(econ_common::SIZE, 8, -6));
    assert!(orders::issue(
        &mut w,
        B,
        Order::Behaviour {
            actor: v1,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(far),
        }
    ));
    let mut left = false;
    let mut v2_out = v2_out_first;
    for _ in 0..1200 {
        tick::step(&mut w);
        check_door(&w, k, &mut was_open);
        left |= w.obj(v1).unwrap().pos.cell() != p.cell();
        v2_out |= on_exit_track(&w, v2);
        assert_eq!(w.obj(v1).unwrap().contact, None, "V1 gets no contact back");
    }
    assert!(left, "the order moves V1");
    assert!(v2_out, "V2 is sent out");
}
