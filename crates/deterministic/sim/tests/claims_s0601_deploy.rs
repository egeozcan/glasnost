//! Claim tests for mind control and deploying (S-0601 claim 15, S-0214
//! Terms) on synthetic data: invented types, every number our own. Houses
//! Alpha (0) and Beta (1) of the testkit's sandbox game, not allied.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BuildingType, Decimal, LandRow, LandType, LocomotionClass, Locomotor, UnitType, WarheadType,
    WeaponType,
};
use glasnost_sim::mind;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, World};

const SIZE: i32 = 24;
const ALPHA: u8 = 0;
const BETA: u8 = 1;
/// The deploy facing of every building type here.
const D: i32 = 64;

fn deployer(into: &str) -> UnitType {
    UnitType {
        strength: Some(200),
        speed: Some(6),
        rot: Some(8),
        cost: Some(1000),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        deploys_into: Some(into.into()),
        ..Default::default()
    }
}

fn building(w: i32, h: i32, into: Option<&str>) -> BuildingType {
    let mut b = BuildingType {
        strength: Some(1000),
        cost: Some(1000),
        foundation: Some([w, h]),
        undeploys_into: into.map(Into::into),
        ..Default::default()
    };
    b.econ.buildup = Some(true);
    b.econ.buildup_frames = Some(10);
    b.econ.deploy_facing = Some(D);
    b
}

/// The combat ruleset with a takeover controller CTRL (Ray3, capacity 3),
/// KIT deploying into SHOP (2 × 2, not a yard) and MCV deploying into YARD
/// (a 3 × 3 construction yard).
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    for (reg, names) in [
        ("buildings", vec!["YARD", "SHOP"]),
        ("vehicles", vec!["MCV", "KIT", "CTRL"]),
    ] {
        let l = r.registries.entry(reg.into()).or_default();
        l.extend(names.into_iter().map(String::from));
    }
    let s = &mut r.sim;
    s.land_types.insert(
        LandType::Clear,
        LandRow {
            buildable: Some(true),
            ..Default::default()
        },
    );
    s.warheads.insert(
        "MIND".into(),
        WarheadType {
            verses: Some(vec![Decimal("1".into()); 11]),
            mind_control: Some(true),
            ..Default::default()
        },
    );
    s.weapons.insert(
        "Ray3".into(),
        WeaponType {
            damage: Some(3),
            rof: Some(30),
            range: Some(Decimal("10".into())),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("MIND".into()),
            ..Default::default()
        },
    );
    let mut ctrl = s.vehicles["BUGGY"].clone();
    ctrl.primary = Some("Ray3".into());
    s.vehicles.insert("CTRL".into(), ctrl);
    let mut yard = building(3, 3, Some("MCV"));
    yard.construction_yard = Some(true);
    yard.factory = Some("building".into());
    s.buildings.insert("YARD".into(), yard);
    s.buildings
        .insert("SHOP".into(), building(2, 2, Some("KIT")));
    s.vehicles.insert("MCV".into(), deployer("YARD"));
    s.vehicles.insert("KIT".into(), deployer("SHOP"));
    r
}

fn c(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

/// A Beta controller holding an Alpha deployer of type `name`.
fn setup(r: &Ruleset, name: &str) -> (World, ObjectId, ObjectId) {
    let mut w = testkit::game_with(r, SIZE, 17);
    testkit::run(&mut w, 4);
    let ctrl = testkit::spawn(&mut w, Kind::Vehicle, "CTRL", BETA, c(-6, -6), 0);
    let v = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        name,
        ALPHA,
        c(0, 0),
        (D * 256) as u16,
    );
    assert!(mind::take(&mut w, ctrl, v));
    (w, ctrl, v)
}

fn deploy(w: &mut World, v: ObjectId) {
    assert!(glasnost_sim::orders::issue(
        w,
        HouseId(BETA),
        Order::Deploy { actor: v }
    ));
}

fn buildings(w: &World) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| o.kind == Kind::Building && o.on_map)
        .map(|o| o.id)
        .collect()
}

fn run_until_gone(w: &mut World, v: ObjectId) {
    for _ in 0..60 {
        testkit::run(w, 1);
        if w.obj(v).is_none() {
            return;
        }
    }
    panic!("the vehicle did not deploy");
}

// spec: S-0601 claim 15 and Edge cases (quirks.deploy_makes_control_permanent on: the new building belongs to the controlling house and is not controlled; the vehicle leaves the list silently; killing the controller gives nothing back)
#[test]
fn claim_15_deploy_makes_control_permanent() {
    let r = rules();
    let (mut w, ctrl, v) = setup(&r, "KIT");
    deploy(&mut w, v);
    run_until_gone(&mut w, v);
    let b = buildings(&w)[0];
    assert_eq!(w.obj(b).unwrap().owner, Some(HouseId(BETA)));
    assert!(!mind::mind_controlled(&w, b));
    assert!(mind::controller(&w, ctrl).unwrap().victims.is_empty());
    glasnost_sim::objects::remove(&mut w, ctrl);
    assert_eq!(
        w.obj(b).unwrap().owner,
        Some(HouseId(BETA)),
        "nothing given back"
    );
}

// spec: S-0601 claim 15 and Edge cases (quirks.deploy_makes_control_permanent off: the new object inherits the control link, the original owner and its list place, and is released as the old object would have been)
#[test]
fn claim_15_quirk_off_inherits_the_link() {
    let mut r = rules();
    r.globals.quirks.deploy_makes_control_permanent = false;
    let (mut w, ctrl, v) = setup(&r, "KIT");
    deploy(&mut w, v);
    run_until_gone(&mut w, v);
    let b = buildings(&w)[0];
    assert_eq!(mind::controlled_by(&w, b), Some(ctrl));
    assert_eq!(mind::original_owner(&w, b), Some(HouseId(ALPHA)));
    assert_eq!(mind::controller(&w, ctrl).unwrap().victims, vec![b]);
    glasnost_sim::objects::remove(&mut w, ctrl);
    assert_eq!(w.obj(b).unwrap().owner, Some(HouseId(ALPHA)), "released");
}

// spec: S-0601 claim 15 (revision 9: a vehicle controlled by a controller is not deploy-ready when it would deploy into a construction yard: no yard appears) and S-0214 Terms (revision 39)
#[test]
fn controlled_mcv_does_not_deploy_into_a_yard() {
    let r = rules();
    let (mut w, _ctrl, v) = setup(&r, "MCV");
    let _ = glasnost_sim::orders::issue(&mut w, HouseId(BETA), Order::Deploy { actor: v });
    testkit::run(&mut w, 60);
    assert!(w.obj(v).is_some());
    assert!(buildings(&w).is_empty());
}

// spec: S-0601 claim 15 (revision 9: the deploy command gives a controlled construction vehicle no order, because it fails deploy readiness; a non-yard deployer is deploy-ready and gets one) and S-0213 the deploy rule
#[test]
fn deploy_command_skips_a_controlled_mcv() {
    use glasnost_sim::targeting::{KeyCommand, key_command};
    let r = rules();
    let (w, _ctrl, v) = setup(&r, "MCV");
    assert!(key_command(&w, HouseId(BETA), &[v], KeyCommand::Deploy).is_empty());
    let (w, _ctrl, k) = setup(&r, "KIT");
    assert_eq!(
        key_command(&w, HouseId(BETA), &[k], KeyCommand::Deploy),
        vec![Order::Deploy { actor: k }]
    );
}

// spec: S-0601 claim 15 (revision 9: after a release the construction vehicle is deploy-ready again, with no trace of the control)
#[test]
fn released_mcv_deploys() {
    let r = rules();
    let (mut w, ctrl, v) = setup(&r, "MCV");
    glasnost_sim::objects::remove(&mut w, ctrl);
    assert_eq!(w.obj(v).unwrap().owner, Some(HouseId(ALPHA)));
    assert!(glasnost_sim::orders::issue(
        &mut w,
        HouseId(ALPHA),
        Order::Deploy { actor: v }
    ));
    run_until_gone(&mut w, v);
    let b = buildings(&w)[0];
    assert_eq!(w.obj(b).unwrap().owner, Some(HouseId(ALPHA)));
}

// spec: S-0601 claims 13 and 15 (revision 9: a permanently controlled construction vehicle is not refused: it deploys and the yard belongs to the dominating house) and S-0214 Terms (revision 39)
#[test]
fn dominated_mcv_deploys_for_the_dominating_house() {
    let r = rules();
    let mut w = testkit::game_with(&r, SIZE, 17);
    testkit::run(&mut w, 4);
    let v = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "MCV",
        ALPHA,
        c(0, 0),
        (D * 256) as u16,
    );
    assert!(mind::dominate(&mut w, v, HouseId(BETA)));
    assert!(mind::mind_controlled(&w, v));
    deploy(&mut w, v);
    run_until_gone(&mut w, v);
    let b = buildings(&w)[0];
    assert_eq!(w.obj(b).unwrap().owner, Some(HouseId(BETA)));
}

// spec: S-0601 claim 15 (revision 9: a deploy order already queued when the vehicle is taken does not survive the guard reset of claim 6)
#[test]
fn queued_deploy_does_not_survive_the_takeover() {
    use glasnost_sim::rules::Behaviour;
    let r = rules();
    let mut w = testkit::game_with(&r, SIZE, 17);
    testkit::run(&mut w, 4);
    let ctrl = testkit::spawn(&mut w, Kind::Vehicle, "CTRL", BETA, c(-6, -6), 0);
    let k = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "KIT",
        ALPHA,
        c(0, 0),
        (D * 256) as u16,
    );
    // The deploy order's effect (S-0213 claim 10): unload queued, not yet started.
    w.obj_mut(k).unwrap().beh.queue(Some(Behaviour::Unload));
    assert!(mind::take(&mut w, ctrl, k));
    let o = w.obj(k).unwrap();
    assert_ne!(o.beh.queued, Some(Behaviour::Unload));
    assert_ne!(o.beh.current, Some(Behaviour::Unload));
    testkit::run(&mut w, 60);
    assert!(w.obj(k).is_some(), "no deploy");
    assert!(buildings(&w).is_empty());
}

mod vd_click {
    use super::*;
    use glasnost_sim::targeting::{self, Action, ClickOn, KeyCommand, Modifiers};

    fn plain(r: &Ruleset, name: &str) -> (World, ObjectId) {
        let mut w = testkit::game_with(r, SIZE, 17);
        testkit::run(&mut w, 4);
        let v = testkit::spawn(
            &mut w,
            Kind::Vehicle,
            name,
            ALPHA,
            c(0, 0),
            (D * 256) as u16,
        );
        (w, v)
    }

    fn shown(w: &World, house: u8, v: ObjectId) -> Action {
        targeting::cursor(
            w,
            HouseId(house),
            &[v],
            ClickOn::Object(v),
            Modifiers::default(),
        )
    }

    // spec: S-0214 table VD row VD2 (a vehicle that deploys into a building, where the building fits and it has deploy readiness: deploy; the click's order is the deploy key's effect)
    #[test]
    fn a_construction_vehicle_clicked_on_itself_deploys_where_it_fits() {
        let r = rules();
        for name in ["MCV", "KIT"] {
            let (mut w, v) = plain(&r, name);
            assert_eq!(shown(&w, ALPHA, v), Action::Deploy, "{name}");
            let res = targeting::click(
                &w,
                HouseId(ALPHA),
                &[v],
                ClickOn::Object(v),
                Modifiers::default(),
            );
            assert_eq!(res.orders.len(), 1, "{name}");
            let key = targeting::key_command(&w, HouseId(ALPHA), &[v], KeyCommand::Deploy);
            assert_eq!(key, vec![Order::Deploy { actor: v }]);
            let mut w2 = w.clone();
            for o in res.orders {
                assert!(glasnost_sim::orders::issue(&mut w, HouseId(ALPHA), o));
            }
            for o in key {
                assert!(glasnost_sim::orders::issue(&mut w2, HouseId(ALPHA), o));
            }
            run_until_gone(&mut w, v);
            run_until_gone(&mut w2, v);
            let (b1, b2) = (buildings(&w), buildings(&w2));
            assert_eq!(b1.len(), 1);
            assert_eq!(
                w.obj(b1[0]).unwrap().pos.cell(),
                w2.obj(b2[0]).unwrap().pos.cell()
            );
        }
    }

    // spec: S-0214 table VD row VD2 (the building cannot be placed where the vehicle stands, the vehicle itself not counting: no-deploy; a foundation wider than 2 is anchored one step up and left)
    #[test]
    fn no_deploy_where_the_building_does_not_fit() {
        let r = rules();
        let (mut w, v) = plain(&r, "MCV");
        testkit::spawn(&mut w, Kind::Vehicle, "KIT", BETA, c(-1, -1), 0);
        assert_eq!(shown(&w, ALPHA, v), Action::NoDeploy);
        let res = targeting::click(
            &w,
            HouseId(ALPHA),
            &[v],
            ClickOn::Object(v),
            Modifiers::default(),
        );
        assert!(res.orders.is_empty());
        let (mut w, k) = plain(&r, "KIT");
        testkit::spawn(&mut w, Kind::Vehicle, "KIT", BETA, c(-1, -1), 0);
        assert_eq!(shown(&w, ALPHA, k), Action::Deploy);
    }

    // spec: S-0214 table VD row VD2 and Terms (deploy readiness fails for a controlled construction vehicle: no-deploy), S-0601 claim 15
    #[test]
    fn no_deploy_when_not_deploy_ready() {
        let r = rules();
        let (w, _ctrl, v) = setup(&r, "MCV");
        assert_eq!(shown(&w, BETA, v), Action::NoDeploy);
        let (w, _ctrl, k) = setup(&r, "KIT");
        assert_eq!(shown(&w, BETA, k), Action::Deploy);
    }
}
