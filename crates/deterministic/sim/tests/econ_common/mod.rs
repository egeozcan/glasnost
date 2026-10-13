//! Shared builders for the production claim tests (S-0403, S-0404, S-0407,
//! S-0411): a synthetic ruleset with invented factory buildings and unit
//! types (no game data), and small helpers. Each `tests/claims_s04xx.rs`
//! file includes this module with `mod econ_common;`.

#![allow(dead_code)]

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, LocomotionClass, Locomotor, UnitType};
use glasnost_sim::econ::TypeRef;
use glasnost_sim::econ::production::{Item, Line};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, World};

/// Map size.
pub const SIZE: i32 = 30;

/// Decimal text.
pub fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

fn push(r: &mut Ruleset, reg: &str, n: &str) {
    r.registries.entry(reg.into()).or_default().push(n.into());
}

fn owners() -> Option<Vec<String>> {
    Some(vec!["Alphaland".into(), "Betaland".into()])
}

/// A factory building type of our format's factory word.
pub fn factory(word: &str) -> BuildingType {
    BuildingType {
        strength: Some(500),
        foundation: Some([2, 2]),
        factory: Some(word.into()),
        tech_level: Some(1),
        owner: owners(),
        cost: Some(1000),
        ..Default::default()
    }
}

/// A plain building type (no factory) of cost `cost`.
pub fn plain(cost: i32) -> BuildingType {
    BuildingType {
        strength: Some(300),
        foundation: Some([1, 1]),
        tech_level: Some(1),
        owner: owners(),
        cost: Some(cost),
        ..Default::default()
    }
}

/// An infantry type of cost `cost`.
pub fn foot(cost: i32) -> UnitType {
    UnitType {
        strength: Some(100),
        speed: Some(4),
        locomotion_class: Some(LocomotionClass::Foot),
        locomotor: Some(Locomotor::Walk),
        tech_level: Some(1),
        owner: owners(),
        cost: Some(cost),
        ..Default::default()
    }
}

/// A vehicle type of cost `cost`.
pub fn wheel(cost: i32) -> UnitType {
    UnitType {
        strength: Some(200),
        speed: Some(5),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        tech_level: Some(1),
        owner: owners(),
        cost: Some(cost),
        ..Default::default()
    }
}

/// The production ruleset: the testkit's, plus
/// - factories BARRACKS (infantry), WARFACT (vehicles), NAVYARD (naval
///   vehicles), AIRPAD (aircraft), CONYARD (buildings);
/// - infantry GRUNT (cost 1000), MEDIC (cost 1000);
/// - vehicles TANK (cost 1000), JEEP (cost 500), BOAT (naval, cost 1000);
/// - aircraft PLANE (cost 1000);
/// - buildings HUT (cost 1000), TOWER (a defence, cost 1000), PLANT (cost
///   500).
pub fn ruleset() -> Ruleset {
    let mut r = testkit::ruleset();
    for (n, b) in [
        ("BARRACKS", factory("infantry")),
        ("WARFACT", factory("vehicle")),
        ("NAVYARD", {
            let mut b = factory("vehicle");
            b.naval = Some(true);
            b
        }),
        ("AIRPAD", factory("aircraft")),
        ("CONYARD", factory("building")),
        ("HUT", plain(1000)),
        ("TOWER", {
            let mut b = plain(1000);
            b.defense = Some(true);
            b
        }),
        ("PLANT", plain(500)),
    ] {
        push(&mut r, "buildings", n);
        r.sim.buildings.insert(n.into(), b);
    }
    for (n, u) in [("GRUNT", foot(1000)), ("MEDIC", foot(1000))] {
        push(&mut r, "infantry", n);
        r.sim.infantry.insert(n.into(), u);
    }
    for (n, u) in [
        ("TANK", wheel(1000)),
        ("JEEP", wheel(500)),
        ("BOAT", {
            let mut u = wheel(1000);
            u.combat.naval = Some(true);
            u
        }),
    ] {
        push(&mut r, "vehicles", n);
        r.sim.vehicles.insert(n.into(), u);
    }
    push(&mut r, "aircraft", "PLANE");
    r.sim.aircraft.insert(
        "PLANE".into(),
        UnitType {
            strength: Some(150),
            tech_level: Some(1),
            owner: owners(),
            cost: Some(1000),
            ..Default::default()
        },
    );
    r
}

/// A new game on the production ruleset (or a variant of it).
pub fn game(r: &Ruleset) -> World {
    testkit::game_with(r, SIZE, 7)
}

/// A type by kind and name.
pub fn t(w: &World, kind: Kind, name: &str) -> TypeRef {
    TypeRef {
        kind,
        index: w.rules().index_of(kind, name).expect("type"),
    }
}

/// Puts a building of house `h` on the map at offset (dx, dy) from the
/// middle.
pub fn building(w: &mut World, name: &str, h: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, Kind::Building, name, h, testkit::mid(SIZE, dx, dy), 0)
}

/// Runs ticks until the order block of ticks 0 to 3 is over (S-0201 claim
/// 14), so that orders can be issued.
pub fn ready(w: &mut World) {
    while w.tick < 4 {
        testkit::run(w, 1);
    }
}

/// Issues an order for house `h` (S-0213 claim 1); returns whether it was
/// taken (legal and not blocked).
pub fn issue(w: &mut World, h: u8, o: Order) -> bool {
    orders::issue(w, HouseId(h), o)
}

/// Puts an order straight into house `h`'s pending list, past the legality
/// test of issue time, to test how an order is applied.
pub fn force(w: &mut World, h: u8, o: Order) {
    w.houses[usize::from(h)].pending.push(o);
}

/// One tick.
pub fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

/// The line of house `h` for type `ty`.
pub fn line(w: &World, h: u8, ty: TypeRef) -> Option<Line> {
    glasnost_sim::econ::production::line_of(w, HouseId(h), ty).cloned()
}

/// The item of house `h`'s line for type `ty`.
pub fn item(w: &World, h: u8, ty: TypeRef) -> Option<Item> {
    line(w, h, ty).and_then(|l| l.item)
}

/// House `h`'s credits.
pub fn credits(w: &World, h: u8) -> i32 {
    w.houses[usize::from(h)].credits
}

/// Sets house `h`'s credits.
pub fn set_credits(w: &mut World, h: u8, c: i32) {
    w.houses[usize::from(h)].credits = c;
}

/// Switches a building off or on through the power switch's event (S-0405
/// owns the switch; the claim tests set the state and send the event).
pub fn switch(w: &mut World, b: ObjectId, on: bool) {
    w.obj_mut(b).unwrap().econ.building.switched_off = !on;
    glasnost_sim::econ::events::switched(w, b);
}

/// Runs ticks until house `h`'s item of `ty` is done, at most `limit`
/// ticks; returns the done tick.
pub fn run_to_done(w: &mut World, h: u8, ty: TypeRef, limit: i32) -> Option<i32> {
    for _ in 0..limit {
        if let Some(d) = item(w, h, ty).and_then(|i| i.done_tick) {
            return Some(d);
        }
        step(w);
    }
    item(w, h, ty).and_then(|i| i.done_tick)
}
