//! Shared fixtures of the sim issues S21 targeting tests (S-0214 revision
//! 83 and S-0213; #412, #413 and #415 to #419): the testkit's combat rules
//! plus invented types. Every name and number is invented; no game data.
//! Ledger: `docs/m8-claims/sim-issues.md`, *Batch S21*.
#![allow(dead_code, unused_imports)]

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, LandRow, LandType, WeaponType};
use glasnost_sim::orders;
use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, World};

pub const SIZE: i32 = 24;
pub const ALPHA: u8 = 0;
pub const BETA: u8 = 1;
pub const A: HouseId = HouseId(ALPHA);

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn building(r: &mut Ruleset, name: &str, undeploys_into: Option<&str>) {
    let mut b = BuildingType {
        strength: Some(500),
        foundation: Some([1, 1]),
        ..Default::default()
    };
    b.undeploys_into = undeploys_into.map(Into::into);
    register(r, "buildings", name);
    r.sim.buildings.insert(name.into(), b);
}

/// The rules of the S21 targeting tests, on top of the testkit's combat
/// rules (CRUSHER armed with `Gun`, CART and BUGGY unarmed, WALKER armed
/// with `Rifle`, the building TOWER):
/// - buildings `YARD` (1 × 1), `HUT` (1 × 1, undeploys into `NOSUCH`, a
///   name no vehicle type has) and `PACK` (1 × 1, undeploys into `MCV`);
/// - vehicles `MCV` (deploys into `YARD`) and `DEPCAR` (deploys in place,
///   carries 2 passengers), both copies of BUGGY;
/// - infantry `AREA` (secondary `Spray`, an area-fire weapon), `SPRAY`
///   (the spray-attack flag, primary `Spray`, secondary `Rifle`), `SPRAY2`
///   (the spray-attack flag, primary `Rifle`, secondary `Spray`) and `DINF`
///   (a `DeploysInto` key naming `YARD`), copies of WALKER;
/// - clear land with the buildable flag.
pub fn ruleset() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let mut spray = r.sim.weapons["Rifle"].clone();
    spray.area_fire = Some(true);
    r.sim.weapons.insert("Spray".into(), spray);
    building(&mut r, "YARD", None);
    building(&mut r, "HUT", Some("NOSUCH"));
    building(&mut r, "PACK", Some("MCV"));
    let mut mcv = r.sim.vehicles["BUGGY"].clone();
    mcv.deploys_into = Some("YARD".into());
    register(&mut r, "vehicles", "MCV");
    r.sim.vehicles.insert("MCV".into(), mcv);
    let mut car = r.sim.vehicles["BUGGY"].clone();
    car.is_simple_deployer = Some(true);
    car.passengers = Some(2);
    register(&mut r, "vehicles", "DEPCAR");
    r.sim.vehicles.insert("DEPCAR".into(), car);
    let walker = r.sim.infantry["WALKER"].clone();
    let mut area = walker.clone();
    area.secondary = Some("Spray".into());
    let mut spray_a = walker.clone();
    spray_a.spray_attack = Some(true);
    spray_a.primary = Some("Spray".into());
    spray_a.secondary = Some("Rifle".into());
    let mut spray_b = walker.clone();
    spray_b.spray_attack = Some(true);
    spray_b.primary = Some("Rifle".into());
    spray_b.secondary = Some("Spray".into());
    let mut dinf = walker;
    dinf.deploys_into = Some("YARD".into());
    for (name, t) in [
        ("AREA", area),
        ("SPRAY", spray_a),
        ("SPRAY2", spray_b),
        ("DINF", dinf),
    ] {
        register(&mut r, "infantry", name);
        r.sim.infantry.insert(name.into(), t);
    }
    // Clear land is buildable, so a deploying vehicle's building passes the cell test.
    r.sim.land_types.insert(
        LandType::Clear,
        LandRow {
            buildable: Some(true),
            ..Default::default()
        },
    );
    r
}

/// A game of `r` whose order input is open.
pub fn game_of(r: &Ruleset) -> World {
    let mut w = testkit::game_with(r, SIZE, 7);
    while orders::blocked(&w) {
        testkit::run(&mut w, 1);
    }
    w
}

pub fn game() -> World {
    game_of(&ruleset())
}

pub fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

pub fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, at(dx, dy), 0)
}

/// Deactivates `id`: the control-centre state of S-0405 claims 13 and 20.
pub fn deactivate(w: &mut World, id: ObjectId) {
    w.obj_mut(id).unwrap().econ.building.unit_disabled = true;
}

/// The cursor of the selection `sel` over `on`, no modifiers.
pub fn cursor(w: &World, sel: &[ObjectId], on: ClickOn) -> Action {
    targeting::cursor(w, A, sel, on, Modifiers::default())
}

/// The cursor of `id` alone clicked on itself.
pub fn on_itself(w: &World, id: ObjectId) -> Action {
    cursor(w, &[id], ClickOn::Object(id))
}
