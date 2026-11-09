//! Claim tests of S-0404 revision 26 (Q-2841 answered), claim 13's ground
//! vehicle factory row: the unit is put at the exit point without an entry
//! test of that cell, and its contact takes the free place of K with the
//! lowest number, or place 1 from its holder when none is free. Synthetic
//! rules only: invented types and numbers, no game data. Ledger:
//! `docs/m8-claims/intake120.md`.

mod econ_common;

use glasnost_rules::schema::Ruleset;
use glasnost_sim::econ::release::{self, Outcome};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};
use glasnost_sim::{objects, testkit};

/// The production ruleset with VF, a ground vehicle factory (vehicle
/// factory flag, 3 × 2 foundation) with `docks` docking places (absent:
/// the default).
fn vf_rules(docks: Option<i32>) -> Ruleset {
    let mut r = econ_common::ruleset();
    let mut b = econ_common::factory("vehicle");
    b.foundation = Some([3, 2]);
    b.weapons_factory = Some(true);
    b.econ.number_of_docks = docks;
    r.registries.get_mut("buildings").unwrap().push("VF".into());
    r.sim.buildings.insert("VF".into(), b);
    r
}

fn new_unit(w: &mut World, name: &str) -> ObjectId {
    let ti = w.rules().index_of(Kind::Vehicle, name).unwrap();
    objects::create(w, Kind::Vehicle, ti, Some(HouseId(0)))
}

/// A world with Alpha's VF, past the order block; returns K and its exit
/// point (the foundation centre: no exit displacement).
fn scene(r: &Ruleset) -> (World, ObjectId, Pos) {
    let mut w = econ_common::game(r);
    econ_common::ready(&mut w);
    let k = econ_common::building(&mut w, "VF", 0, 0, 0);
    let p = w.reference_point(k);
    (w, k, p)
}

/// K back in *guard* with nothing queued, as after an owner change ended
/// its unloading (S-0309 claim 31), so that its next release is its own.
fn guard_now(w: &mut World, k: ObjectId) {
    let t = w.tick;
    w.obj_mut(k).unwrap().beh.force(Behaviour::Guard, t, true);
}

fn place_of(w: &World, u: ObjectId) -> Option<u32> {
    w.obj(u).unwrap().econ.exit.map(|e| e.place)
}

// spec: S-0404 claim 13 (ground vehicle factory row: the unit is put at the exit point without an entry test; neither K itself, listed in the cell, nor any other occupant of any house, moving or not, nor the cell's terrain keeps it out, so the cell never gives failed; the unit shares the cell); scenario P12 part a and S-0309 scenario A19 part k
#[test]
fn claim_13_no_entry_test_at_the_exit_point() {
    let cases: [(&str, fn(&mut World, Pos)); 6] = [
        ("K itself only", |_, _| {}),
        ("an enemy vehicle", |w, p| {
            testkit::spawn(w, Kind::Vehicle, "TANK", 1, p.cell(), 0);
        }),
        ("an own infantry unit", |w, p| {
            testkit::spawn(w, Kind::Infantry, "GRUNT", 0, p.cell(), 0);
        }),
        ("an own vehicle with a destination", |w, p| {
            let v = testkit::spawn(w, Kind::Vehicle, "TANK", 0, p.cell(), 0);
            let far = Target::Cell(testkit::mid(econ_common::SIZE, 9, 9));
            w.obj_mut(v).unwrap().dest = Some(far);
        }),
        (
            "an own vehicle standing still with no destination",
            |w, p| {
                testkit::spawn(w, Kind::Vehicle, "JEEP", 0, p.cell(), 0);
            },
        ),
        ("water terrain", |w, p| {
            w.cells.grid.get_mut(p.cell()).unwrap().land = glasnost_map::terrain::land::WATER;
        }),
    ];
    for (what, set) in cases {
        let (mut w, k, p) = scene(&vf_rules(None));
        set(&mut w, p);
        let others: Vec<ObjectId> = w
            .objects
            .values()
            .filter(|o| o.on_map && o.id != k && o.pos.cell() == p.cell())
            .map(|o| o.id)
            .collect();
        let tank = new_unit(&mut w, "TANK");
        assert_eq!(
            release::release(&mut w, k, tank),
            Outcome::Released,
            "{what}"
        );
        let o = w.obj(tank).unwrap();
        assert!(o.on_map, "{what}");
        assert_eq!((o.pos.x, o.pos.y), (p.x, p.y), "{what}: at the exit point");
        assert_eq!(o.contact, Some(k), "{what}");
        assert_eq!(place_of(&w, tank), Some(0), "{what}");
        for x in others {
            let o = w.obj(x).unwrap();
            assert_eq!(o.pos.cell(), p.cell(), "{what}: the occupant stays");
            assert_eq!(o.contact, None, "{what}");
        }
    }
}

// spec: S-0404 claim 13 (K's max(d, 1) contact places, numbered from 1; the new unit's contact takes the free place with the lowest number; with none free the holder of place 1 loses its contact and tether link at the release and the new unit takes that place)
#[test]
fn claim_13_the_new_unit_takes_the_lowest_free_place() {
    let (mut w, k, p) = scene(&vf_rules(Some(2)));
    assert_eq!(release::contact_places(&w, k), 2);
    let u1 = new_unit(&mut w, "TANK");
    assert_eq!(release::release(&mut w, k, u1), Outcome::Released);
    assert_eq!(place_of(&w, u1), Some(0), "place 1");
    guard_now(&mut w, k);
    let u2 = new_unit(&mut w, "TANK");
    assert_eq!(release::release(&mut w, k, u2), Outcome::Released);
    assert_eq!(place_of(&w, u2), Some(1), "place 2, the lowest free");
    assert!(
        release::tethered(&w, u1),
        "place 1 is kept while place 2 is free"
    );
    guard_now(&mut w, k);
    let u3 = new_unit(&mut w, "TANK");
    assert_eq!(release::release(&mut w, k, u3), Outcome::Released);
    assert_eq!(place_of(&w, u3), Some(0), "place 1, taken");
    assert!(release::tethered(&w, u3));
    let o1 = w.obj(u1).unwrap();
    assert_eq!(o1.contact, None, "the holder of place 1 loses its contact");
    assert_eq!(o1.econ.exit, None, "and its tether link");
    assert_eq!(place_of(&w, u2), Some(1), "place 2's holder keeps it");
    assert!(release::tethered(&w, u2));
    for u in [u1, u2, u3] {
        assert_eq!(w.obj(u).unwrap().pos.cell(), p.cell());
    }
}

// spec: S-0404 claim 13 (a K with one place, held: the release takes it; the unit that held it loses its contact and tether link and keeps its behaviours, destination and position) and S-0309 claim 31
#[test]
fn claim_13_one_held_place_is_taken() {
    let (mut w, k, p) = scene(&vf_rules(None));
    assert_eq!(release::contact_places(&w, k), 1);
    let u1 = new_unit(&mut w, "TANK");
    assert_eq!(release::release(&mut w, k, u1), Outcome::Released);
    guard_now(&mut w, k);
    let before = w.obj(u1).unwrap().clone();
    let u2 = new_unit(&mut w, "JEEP");
    assert_eq!(release::release(&mut w, k, u2), Outcome::Released);
    let o1 = w.obj(u1).unwrap();
    assert_eq!(o1.contact, None);
    assert_eq!(o1.econ.exit, None);
    assert_eq!(o1.pos, before.pos);
    assert_eq!(o1.dest, before.dest);
    assert_eq!(
        (o1.beh.current, o1.beh.queued),
        (before.beh.current, before.beh.queued)
    );
    assert_eq!(w.obj(u2).unwrap().contact, Some(k));
    assert_eq!(place_of(&w, u2), Some(0));
    assert_eq!(w.obj(u2).unwrap().pos.cell(), p.cell());
    assert_eq!(w.obj(k).unwrap().beh.queued, Some(Behaviour::Unload));
}
