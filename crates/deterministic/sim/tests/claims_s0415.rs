//! Claim tests for S-0415 (deploying a vehicle into a building and packing
//! it up again) on synthetic data: invented vehicle, building and overlay
//! types, no stock values. Houses Alpha (0) and Beta (1) of the testkit's
//! sandbox game, not allied.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BuildingType, LandRow, LandType, LocomotionClass, Locomotor, OverlayType, UnitType,
};
use glasnost_sim::combat::effects::{Effect, Sound};
use glasnost_sim::econ::deploy;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const SIZE: i32 = 24;
const A: HouseId = HouseId(0);
/// The deploy facing of every building type here.
const D: i32 = 64;

fn vehicle(into: &str) -> UnitType {
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
    // n = 5: build-up and build-down 36 ticks (S-0108 claim 5).
    b.econ.buildup_frames = Some(10);
    b.econ.deploy_facing = Some(D);
    b
}

/// MCV → YARD (3 × 3 construction yard, undeploys into MCV); KIT → SHOP
/// (2 × 2, undeploys into KIT); TRUCK (passengers) and HARV (harvester)
/// deploying into SHOP; PICK (resource gatherer) → SHOP; TWR → TOWER (the
/// wall tower); overlay O0 a wall built by WALLA.
fn rules() -> Ruleset {
    let mut r = testkit::ruleset();
    r.registries.insert(
        "buildings".into(),
        ["WALLA", "YARD", "SHOP", "TOWER"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
    );
    r.registries.get_mut("vehicles").unwrap().extend(
        ["MCV", "KIT", "TRUCK", "HARV", "PICK", "TWR"]
            .iter()
            .map(|s| s.to_string()),
    );
    r.registries.insert("overlays".into(), vec!["O0".into()]);
    let s = &mut r.sim;
    s.land_types.insert(
        LandType::Clear,
        LandRow {
            buildable: Some(true),
            ..Default::default()
        },
    );
    s.overlays.insert(
        "O0".into(),
        OverlayType {
            wall: Some(true),
            has_art: Some(true),
            ..Default::default()
        },
    );
    let mut wall = building(1, 1, None);
    wall.econ.wall = Some(true);
    wall.econ.to_overlay = Some("O0".into());
    s.buildings.insert("WALLA".into(), wall);
    let mut yard = building(3, 3, Some("MCV"));
    yard.construction_yard = Some(true);
    yard.factory = Some("building".into());
    s.buildings.insert("YARD".into(), yard);
    s.buildings
        .insert("SHOP".into(), building(2, 2, Some("KIT")));
    s.buildings.insert("TOWER".into(), building(1, 1, None));
    s.vehicles.insert("MCV".into(), vehicle("YARD"));
    s.vehicles.insert("KIT".into(), vehicle("SHOP"));
    let mut truck = vehicle("SHOP");
    truck.passengers = Some(3);
    s.vehicles.insert("TRUCK".into(), truck);
    let mut harv = vehicle("SHOP");
    harv.harvester = Some(true);
    s.vehicles.insert("HARV".into(), harv);
    let mut pick = vehicle("SHOP");
    pick.econ.resource_gatherer = Some(true);
    s.vehicles.insert("PICK".into(), pick);
    s.vehicles.insert("TWR".into(), vehicle("TOWER"));
    s.general.wall_tower = Some("TOWER".into());
    r
}

fn game(r: &Ruleset) -> World {
    let mut w = testkit::game_with(r, SIZE, 17);
    testkit::run(&mut w, 4);
    w
}

fn c(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn facing_d() -> u16 {
    (D * 256) as u16
}

fn spawn_v(w: &mut World, name: &str, cell: CellXY, facing: u16) -> ObjectId {
    testkit::spawn(w, Kind::Vehicle, name, 0, cell, facing)
}

fn order(w: &mut World, o: Order) {
    assert!(glasnost_sim::orders::issue(w, A, o), "order accepted");
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

fn buildings(w: &World) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| o.kind == Kind::Building && o.on_map)
        .map(|o| o.id)
        .collect()
}

fn sounds(w: &World) -> Vec<Sound> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Sound { sound, .. } => Some(*sound),
            _ => None,
        })
        .collect()
}

/// Runs until `f` holds, at most `n` ticks; returns the tick it held at.
fn until(w: &mut World, n: i32, mut f: impl FnMut(&World) -> bool) -> Option<i32> {
    for _ in 0..n {
        step(w);
        if f(w) {
            return Some(w.tick);
        }
    }
    None
}

// spec: S-0415 claim 3 and F3 (the anchor: the vehicle's cell for W, H ≤ 2, else (x − 1, y − 1))
// spec: S-0415 claim 10 and F3 (the return cell: the top-left cell for W, H ≤ 2, else that cell + (1, 1))
#[test]
fn f3_anchor_and_return_cell() {
    let k = CellXY::new(10, 7);
    assert_eq!(deploy::anchor(k, (2, 2)), k);
    assert_eq!(deploy::anchor(k, (3, 3)), CellXY::new(9, 6));
    assert_eq!(deploy::anchor(k, (3, 2)), CellXY::new(9, 6));
    assert_eq!(deploy::return_cell(CellXY::new(9, 6), (3, 3)), k);
    assert_eq!(deploy::return_cell(k, (1, 2)), k);
}

// spec: S-0415 claim 6 (completion: the building at the anchor, construction begins, health max(1, trunc(r × S)), the deploy sound, the vehicle removed)
// spec: S-0415 claim 1 (the first attempt at the first processing after the order)
#[test]
fn claim_6_completion() {
    let mut w = game(&rules());
    let v = spawn_v(&mut w, "MCV", c(0, 0), facing_d());
    w.obj_mut(v).unwrap().health = 50;
    w.obj_mut(v).unwrap().combat.experience = glasnost_num::soft::F32(0x3f80_0000);
    order(&mut w, Order::Deploy { actor: v });
    w.effects.clear();
    until(&mut w, 30, |w| w.obj(v).is_none()).expect("deployed");
    let b = buildings(&w);
    assert_eq!(b.len(), 1);
    let o = w.obj(b[0]).unwrap();
    assert_eq!(o.pos.cell(), c(-1, -1));
    assert_eq!(o.pos.x, 256 * c(-1, -1).x + 128);
    assert_eq!(o.owner, Some(A));
    assert_eq!(o.beh.current, Some(Behaviour::Construction));
    // r = 50 / 200, S = 1000.
    assert_eq!(o.health, 250);
    assert_eq!(o.combat.experience, glasnost_num::soft::F32(0x3f80_0000));
    assert!(sounds(&w).contains(&Sound::Deploy));
}

// spec: S-0415 claim 6 (every object whose target was the vehicle gets the building as its target)
#[test]
fn claim_6_retarget() {
    let mut w = game(&rules());
    let v = spawn_v(&mut w, "KIT", c(0, 0), facing_d());
    let enemy = testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, c(6, 6), 0);
    w.obj_mut(enemy).unwrap().target = Some(Target::Object(v));
    order(&mut w, Order::Deploy { actor: v });
    until(&mut w, 30, |w| w.obj(v).is_none()).expect("deployed");
    let b = buildings(&w)[0];
    assert_eq!(
        w.obj(b).unwrap().pos.cell(),
        c(0, 0),
        "2 × 2: the vehicle's cell"
    );
    let t = w.obj(enemy).unwrap().target;
    assert!(t.is_none() || t == Some(Target::Object(b)));
}

// spec: S-0415 claim 1 (ranks 1 to 3: a passenger capacity, the harvester flag; such a type never deploys through unload)
#[test]
fn claim_1_ranks() {
    let mut w = game(&rules());
    let t = spawn_v(&mut w, "TRUCK", c(0, 0), facing_d());
    let h = spawn_v(&mut w, "HARV", c(6, 0), facing_d());
    order(&mut w, Order::Deploy { actor: t });
    order(&mut w, Order::Deploy { actor: h });
    for _ in 0..20 {
        step(&mut w);
    }
    assert!(w.obj(t).is_some());
    assert!(w.obj(h).is_some());
    assert!(buildings(&w).is_empty());
}

// spec: S-0415 claims 3 and 4 (the cell test fails: refused, the refusal announcement, guard queued and current by the next processing; the vehicle stays)
#[test]
fn claim_4_refusal() {
    let mut w = game(&rules());
    let v = spawn_v(&mut w, "KIT", c(0, 0), facing_d());
    testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, c(1, 1), 0);
    order(&mut w, Order::Deploy { actor: v });
    w.effects.clear();
    until(&mut w, 5, |w| {
        w.obj(v).unwrap().beh.current == Some(Behaviour::Unload)
    })
    .expect("unload");
    until(&mut w, 30, |w| {
        w.obj(v).unwrap().beh.current == Some(Behaviour::Guard)
    })
    .expect("guard");
    assert!(sounds(&w).contains(&Sound::CannotDeploy));
    assert!(buildings(&w).is_empty());
    assert_eq!(w.obj(v).unwrap().pos.cell(), c(0, 0));
}

// spec: S-0415 claim 4 (no refusal announcement for a type that gathers resources)
#[test]
fn claim_4_gatherer_silent() {
    let mut w = game(&rules());
    let v = spawn_v(&mut w, "PICK", c(0, 0), facing_d());
    testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, c(1, 1), 0);
    order(&mut w, Order::Deploy { actor: v });
    w.effects.clear();
    until(&mut w, 5, |w| {
        w.obj(v).unwrap().beh.current == Some(Behaviour::Unload)
    })
    .expect("unload");
    until(&mut w, 30, |w| {
        w.obj(v).unwrap().beh.current == Some(Behaviour::Guard)
    })
    .expect("guard");
    assert!(!sounds(&w).contains(&Sound::CannotDeploy));
}

// spec: S-0415 claim 5 and F1 (a rounded facing other than D: the vehicle turns in place towards D × 256; the deploy completes when it faces D)
#[test]
fn claim_5_turn_then_deploy() {
    let mut w = game(&rules());
    let v = spawn_v(&mut w, "KIT", c(0, 0), facing_d().wrapping_add(32768));
    order(&mut w, Order::Deploy { actor: v });
    step(&mut w);
    step(&mut w);
    let t = w.tick;
    let o = w.obj(v).expect("still a vehicle while turning");
    assert!(o.econ.deploy.turning);
    assert!(o.body.turning(t));
    assert_eq!(o.pos.cell(), c(0, 0));
    until(&mut w, 200, |w| w.obj(v).is_none()).expect("deployed after the turn");
    assert_eq!(buildings(&w).len(), 1);
}

// spec: S-0415 claim 5 (a destination abandons the turn with no announcement; then the queued behaviour becomes current)
#[test]
fn claim_5_destination_abandons() {
    let mut w = game(&rules());
    let v = spawn_v(&mut w, "KIT", c(0, 0), facing_d().wrapping_add(32768));
    order(&mut w, Order::Deploy { actor: v });
    step(&mut w);
    step(&mut w);
    assert!(w.obj(v).unwrap().econ.deploy.turning);
    w.obj_mut(v).unwrap().dest = Some(Target::Cell(c(5, 5)));
    w.effects.clear();
    for _ in 0..100 {
        step(&mut w);
    }
    assert!(w.obj(v).is_some());
    assert!(buildings(&w).is_empty());
    assert!(w.obj(v).unwrap().econ.deploy.abandoned);
    assert!(!sounds(&w).contains(&Sound::CannotDeploy));
}

// spec: S-0415 claim 3 (the wall exception applies only to walls with no owner, never to the house's own: quirks.deploy_refuses_own_walls)
// spec: S-0415 claims 2 and 6 (an unowned wall passes the test but the placement refuses: no building, guard queued, no announcement)
#[test]
fn claim_3_walls() {
    let mut w = game(&rules());
    let own = c(0, 0);
    let free = c(6, 0);
    for (cell, owner) in [(own, Some(A)), (free, None)] {
        let g = w.cells.grid.get_mut(cell).unwrap();
        g.overlay = 0;
        g.overlay_data = 0;
        let i = w.cells.idx(cell).unwrap();
        w.cells.wall_owner[i] = owner;
    }
    // Vehicles cannot stand on a wall; they are put there for the test.
    let a = spawn_v(&mut w, "TWR", own, facing_d());
    let b = spawn_v(&mut w, "TWR", free, facing_d());
    order(&mut w, Order::Deploy { actor: a });
    order(&mut w, Order::Deploy { actor: b });
    w.effects.clear();
    for _ in 0..20 {
        step(&mut w);
    }
    assert!(buildings(&w).is_empty());
    assert_eq!(
        sounds(&w)
            .iter()
            .filter(|s| **s == Sound::CannotDeploy)
            .count(),
        1,
        "only the own wall is refused with the announcement"
    );
    assert_eq!(w.obj(b).unwrap().beh.current, Some(Behaviour::Guard));
}

/// Deploys an MCV at the middle and runs past the construction.
fn deployed_yard(w: &mut World) -> ObjectId {
    let v = spawn_v(w, "MCV", c(0, 0), facing_d());
    order(w, Order::Deploy { actor: v });
    until(w, 30, |w| w.obj(v).is_none()).expect("deployed");
    let y = buildings(w)[0];
    until(w, 60, |w| !w.obj(y).unwrap().econ.construction.building_up).expect("built");
    y
}

// spec: S-0415 claims 9 and 10 (a yard with a guard spot packs up: deploy sound, build-down, the vehicle at the return cell facing D × 256, health max(1, trunc(r × S)), the guard spot as destination with move, no refund)
// spec: S-0408 claim 6 and S-0415 claim 9 (no sell sound for a packing building)
#[test]
fn claims_9_10_pack_up() {
    let mut w = game(&rules());
    let y = deployed_yard(&mut w);
    w.obj_mut(y).unwrap().health = 500;
    let spot = Target::Cell(c(6, 6));
    order(&mut w, Order::GuardSpot { actor: y, spot });
    step(&mut w);
    let credits = w.houses[0].credits;
    order(&mut w, Order::Sell { actor: y });
    w.effects.clear();
    until(&mut w, 60, |w| w.obj(y).is_none()).expect("packed up");
    let s = sounds(&w);
    assert!(s.contains(&Sound::Deploy));
    assert!(s.contains(&Sound::Packup));
    assert!(!s.contains(&Sound::Sell));
    assert_eq!(w.houses[0].credits, credits);
    let v: Vec<_> = w
        .objects
        .values()
        .filter(|o| o.kind == Kind::Vehicle && o.owner == Some(A))
        .collect();
    assert_eq!(v.len(), 1);
    let v = v[0];
    assert_eq!(v.pos.cell(), c(0, 0));
    assert_eq!(v.body.value(w.tick), facing_d());
    // r = 500 / 1000, S = 200.
    assert_eq!(v.health, 100);
    assert_eq!(v.dest, Some(spot));
    assert_eq!(v.beh.current, Some(Behaviour::Move));
}

// spec: S-0213 claim 12 via S-0415 claim 9 (a construction yard without a guard spot does not pack up: it is sold, with the refund)
#[test]
fn claim_9_yard_without_guard_spot_is_sold() {
    let mut w = game(&rules());
    let y = deployed_yard(&mut w);
    assert!(!deploy::packs_up(&w, y));
    let credits = w.houses[0].credits;
    order(&mut w, Order::Sell { actor: y });
    until(&mut w, 60, |w| w.obj(y).is_none()).expect("sold");
    assert_eq!(w.houses[0].credits, credits + 500);
    assert!(w.objects.values().all(|o| o.kind != Kind::Vehicle));
}

// spec: S-0415 claim 11 (the vehicle cannot be put on the map: the sale refund instead, the building removed; quirks.failed_packup_keeps_vehicle keeps the vehicle off the map)
#[test]
fn claim_11_failed_pack_up() {
    let mut w = game(&rules());
    let y = deployed_yard(&mut w);
    let spot = Target::Cell(c(6, 6));
    order(&mut w, Order::GuardSpot { actor: y, spot });
    step(&mut w);
    // Something in the return cell: a vehicle put there for the test.
    testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, c(0, 0), 0);
    let credits = w.houses[0].credits;
    order(&mut w, Order::Sell { actor: y });
    until(&mut w, 60, |w| w.obj(y).is_none()).expect("removed");
    assert_eq!(w.houses[0].credits, credits + 500);
    let mcv = w.rules().index_of(Kind::Vehicle, "MCV").unwrap();
    let kept: Vec<_> = w
        .objects
        .values()
        .filter(|o| o.kind == Kind::Vehicle && o.type_index == mcv)
        .collect();
    assert_eq!(kept.len(), 1);
    assert!(!kept[0].on_map);
    assert_eq!(kept[0].owner, Some(A));
}

// spec: S-0415 claim 11 and Edge cases (quirks.failed_packup_keeps_vehicle off: no vehicle remains)
#[test]
fn claim_11_quirk_off() {
    let mut r = rules();
    r.globals.quirks.failed_packup_keeps_vehicle = false;
    let mut w = game(&r);
    assert!(!w.rules().quirks.failed_packup_keeps_vehicle);
    let y = deployed_yard(&mut w);
    order(
        &mut w,
        Order::GuardSpot {
            actor: y,
            spot: Target::Cell(c(6, 6)),
        },
    );
    step(&mut w);
    testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, c(0, 0), 0);
    order(&mut w, Order::Sell { actor: y });
    until(&mut w, 60, |w| w.obj(y).is_none()).expect("removed");
    let mcv = w.rules().index_of(Kind::Vehicle, "MCV").unwrap();
    assert!(
        w.objects
            .values()
            .all(|o| o.type_index != mcv || o.kind != Kind::Vehicle)
    );
}
