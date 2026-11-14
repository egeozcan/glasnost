//! S-0315 claim 2 and scenario P2 part c: a fraidycat's last round fired
//! from inside a transport does not trigger the panic (#277). Synthetic
//! types on `testkit::combat_ruleset`; every name and number is made up.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, LocomotionClass, Locomotor, UnitType};
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{ObjectId, World};
use glasnost_sim::{testkit, transport};

const SIZE: i32 = 24;

fn ruleset() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("OPEN".into());
    r.registries
        .get_mut("infantry")
        .unwrap()
        .push("FRAID".into());
    r.sim.vehicles.insert(
        "OPEN".into(),
        UnitType {
            strength: Some(400),
            speed: Some(6),
            rot: Some(8),
            locomotion_class: Some(LocomotionClass::Wheel),
            locomotor: Some(Locomotor::Drive),
            passengers: Some(4),
            size_limit: Some(Decimal("2".into())),
            open_topped: Some(true),
            ..Default::default()
        },
    );
    let mut fraid = UnitType {
        strength: Some(100),
        speed: Some(4),
        locomotion_class: Some(LocomotionClass::Foot),
        locomotor: Some(Locomotor::Walk),
        fraidycat: Some(true),
        primary: Some("Rifle".into()),
        ..Default::default()
    };
    fraid.combat.ammo = Some(2);
    fraid.combat.initial_ammo = Some(2);
    r.sim.infantry.insert("FRAID".into(), fraid);
    r.sim.vehicles.get_mut("BUGGY").unwrap().strength = Some(5000);
    r
}

fn at(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, testkit::mid(SIZE, dx, dy), 0)
}

// spec: S-0315 claim 2 and scenario P2 part c (a shot fired from inside a transport does not trigger the panic)
#[test]
fn issue_277_fraidycat_last_round_from_inside_an_open_topped_transport() {
    let r = ruleset();
    let mut w = testkit::game_with(&r, SIZE, 7);
    let t = at(&mut w, Kind::Vehicle, "OPEN", 0, 0, 0);
    let p = at(&mut w, Kind::Infantry, "FRAID", 0, 0, 3);
    at(&mut w, Kind::Vehicle, "BUGGY", 1, 3, 0);
    transport::board(&mut w, p, t);
    assert_eq!(transport::riding(&w, p), Some(t));
    assert_eq!(w.obj(p).unwrap().combat.ammo, 2);
    let mut emptied = None;
    for _ in 0..600 {
        testkit::run(&mut w, 1);
        let o = w.obj(p).unwrap();
        if o.combat.ammo == 0 {
            emptied = Some(o.infantry.as_ref().unwrap().fear);
            break;
        }
    }
    assert_eq!(transport::riding(&w, p), Some(t), "still inside");
    assert!(emptied.is_some(), "the passenger fired both rounds");
    assert_eq!(
        emptied,
        Some(0),
        "no panic for a last round fired from inside a transport"
    );
}
