//! Row I19's second condition: a bomb planter over a target that already
//! carries a bomb gets no-bomb and no order (#355; S-0214 table I row I19,
//! table M). On the shared `common` scene with invented types and weapon,
//! no game data.

mod common;

use common::{FOOT, Scene, WHEEL, cell, run, unit_type};
use glasnost_rules::sim::UnitType;
use glasnost_sim::bomb::Bomb;
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::world::{HouseId, World};

// spec: S-0214 table I row I19 (bomb if the target's type accepts bombs and it carries none, else no-bomb) and table M (actions not listed give no order)
#[test]
fn issue_355_a_planter_over_a_carrier_gets_no_bomb() {
    let mut s = Scene::new();
    s.ruleset.sim.weapons.insert(
        "TestGun".into(),
        glasnost_rules::sim::WeaponType {
            damage: Some(10),
            range: Some(glasnost_rules::sim::Decimal("3".into())),
            ..Default::default()
        },
    );
    s.vehicle("TESTCAR", unit_type(WHEEL, 40, 6));
    s.infantry(
        "TESTPLANTER",
        UnitType {
            ivan: Some(true),
            primary: Some("TestGun".into()),
            ..unit_type(FOOT, 20, 0)
        },
    );
    s.place_infantry(0, "TESTPLANTER", cell(0, 0), 2)
        .place_unit(1, "TESTCAR", cell(3, 0), 64);
    let mut w = s.build();
    run(&mut w, 1);
    let at = |w: &World, c| w.objects.values().find(|o| o.pos.cell() == c).unwrap().id;
    let planter = at(&w, cell(0, 0));
    let car = at(&w, cell(3, 0));
    let ff = Modifiers {
        force_fire: true,
        ..Default::default()
    };
    let on = ClickOn::Object(car);
    assert_eq!(
        targeting::action(&w, HouseId(0), planter, on, ff),
        Action::Bomb,
        "control: no bomb on the car, the row gives bomb"
    );
    let tick = w.tick;
    w.obj_mut(car).unwrap().combat.bomb = Some(Bomb {
        planter: Some(planter),
        house: Some(HouseId(0)),
        planted: tick,
    });
    assert!(glasnost_sim::bomb::carries(&w, car), "set-up");
    let a = targeting::action(&w, HouseId(0), planter, on, ff);
    let r = targeting::click(&w, HouseId(0), &[planter], on, ff);
    assert_eq!(
        (a, r.orders.len()),
        (Action::NoBomb, 0),
        "orders: {:?}",
        r.orders
    );
}
