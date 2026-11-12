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
    // spec: S-0406 claim 17 (construction queued at entry, current from the building's first turn)
    assert_eq!(
        glasnost_sim::econ::placement::seen_behaviour(o),
        Some(Behaviour::Construction)
    );
    assert!(o.econ.construction.building_up);
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
    assert_eq!(w.obj(enemy).unwrap().target, Some(Target::Object(b)));
}

/// Steps until `gone` no longer exists, at most `n` ticks, holding each of
/// `holders`' target on `gone` with a long targeting timer before every
/// step.
fn hold_until_gone(w: &mut World, holders: &[ObjectId], gone: ObjectId, n: i32) {
    for _ in 0..n {
        let tick = w.tick;
        for &h in holders {
            let ob = w.obj_mut(h).unwrap();
            ob.target = Some(Target::Object(gone));
            ob.targeting_timer = tick + 1000;
        }
        step(w);
        if w.obj(gone).is_none() {
            return;
        }
    }
    panic!("the object never left");
}

/// The rules plus THIEF, a WALKER that is a vehicle thief.
fn thief_rules() -> Ruleset {
    let mut r = rules();
    let mut t = r.sim.infantry["WALKER"].clone();
    t.vehicle_thief = Some(true);
    r.registries
        .get_mut("infantry")
        .unwrap()
        .push("THIEF".into());
    r.sim.infantry.insert("THIEF".into(), t);
    r
}

// spec: S-0415 claim 6 (every object whose target was the deploying vehicle gets the building, with no drop of S-0309 claim 27: its targeting timer is not restarted; a vehicle thief's target becomes nothing when the building is a construction yard)
#[test]
fn claim_6_retarget_hands_over_without_a_drop_and_the_thief_exception() {
    let mut w = game(&thief_rules());
    let v = spawn_v(&mut w, "MCV", c(0, 0), facing_d());
    let enemy = testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, c(6, 6), 0);
    let thief = testkit::spawn(&mut w, Kind::Infantry, "THIEF", 1, c(-6, 6), 0);
    order(&mut w, Order::Deploy { actor: v });
    hold_until_gone(&mut w, &[enemy, thief], v, 30);
    let y = buildings(&w)[0];
    let e = w.obj(enemy).unwrap();
    assert_eq!(e.target, Some(Target::Object(y)), "the yard");
    assert!(
        e.targeting_timer - w.tick > 10,
        "no drop: the timer was not restarted ({} left)",
        e.targeting_timer - w.tick
    );
    assert_eq!(w.obj(thief).unwrap().target, None, "the thief");
}

/// A deployed yard with a guard spot, built, so a sale packs it up.
fn packing_yard(w: &mut World) -> ObjectId {
    let y = deployed_yard(w);
    let spot = Target::Cell(c(6, 6));
    order(w, Order::GuardSpot { actor: y, spot });
    step(w);
    assert!(deploy::packs_up(w, y));
    y
}

// spec: S-0415 claim 10 (every object whose target was the building gets the vehicle as its target, with no drop of S-0309 claim 27)
#[test]
fn claim_10_pack_up_hands_the_holder_to_the_vehicle() {
    let mut w = game(&rules());
    let y = packing_yard(&mut w);
    let enemy = testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, c(-6, 6), 0);
    order(&mut w, Order::Sell { actor: y });
    hold_until_gone(&mut w, &[enemy], y, 80);
    let mcv = w.rules().index_of(Kind::Vehicle, "MCV").unwrap();
    let v = w
        .objects
        .values()
        .find(|o| o.kind == Kind::Vehicle && o.type_index == mcv && o.on_map)
        .map(|o| o.id)
        .expect("the vehicle");
    let e = w.obj(enemy).unwrap();
    assert_eq!(e.target, Some(Target::Object(v)));
    assert!(
        e.targeting_timer - w.tick > 10,
        "no drop: the timer was not restarted ({} left)",
        e.targeting_timer - w.tick
    );
}

// spec: S-0415 claim 11 (a refused pack-up removes the building with no successor: no hand-over) and S-0309 claim 27 (the holders drop it)
#[test]
fn claim_11_refused_pack_up_drops_the_holder() {
    let mut w = game(&rules());
    let y = packing_yard(&mut w);
    // Something in the return cell: a vehicle put there for the test.
    testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, c(0, 0), 0);
    let enemy = testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, c(-6, 6), 0);
    order(&mut w, Order::Sell { actor: y });
    hold_until_gone(&mut w, &[enemy], y, 80);
    assert_eq!(w.obj(enemy).unwrap().target, None);
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
// spec: S-0215 claim 11 (revision 57: the late start makes the queued guard current at the end of the refusing update)
#[test]
fn claim_4_refusal() {
    let mut w = game(&rules());
    let v = spawn_v(&mut w, "KIT", c(0, 0), facing_d());
    testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, c(1, 1), 0);
    order(&mut w, Order::Deploy { actor: v });
    w.effects.clear();
    // The order queues unload; the refusal queues guard in the unload processing and the late start of the same update makes it current (S-0215 claim 11, revision 57), so unload is current only inside that update and is not seen between ticks.
    until(&mut w, 30, |w| sounds(w).contains(&Sound::CannotDeploy)).expect("refused");
    assert_eq!(w.obj(v).unwrap().beh.current, Some(Behaviour::Guard));
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
    // The order queues unload; the refusal queues guard in the unload processing and the late start of the same update makes it current (S-0215 claim 11, revision 57), so unload is current only inside that update and is not seen between ticks.
    for _ in 0..10 {
        step(&mut w);
    }
    assert_eq!(w.obj(v).unwrap().beh.current, Some(Behaviour::Guard));
    assert!(buildings(&w).is_empty(), "refused");
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

// spec: S-0415 claims 9 and 10 (a yard with a guard spot packs up: deploy sound, build-down, the vehicle at the return cell facing D × 256, health max(1, trunc(r × S)), the guard spot as destination with move queued in the conversion tick, no refund)
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
    // Revision 8: move is queued in the conversion tick, not forced; it starts at the vehicle's next behaviour start.
    assert_eq!(v.beh.current, None);
    assert_eq!(v.beh.queued, Some(Behaviour::Move));
    let id = v.id;
    until(&mut w, 5, |w| {
        w.obj(id).unwrap().beh.current == Some(Behaviour::Move)
    })
    .expect("move started");
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

// spec: S-0415 claim 12 (only a clear entry outcome accepts: one own standing infantry unit in the return cell refuses; a clear neighbour does not rescue the pack-up)
#[test]
fn claim_12_own_infantry_refuses() {
    let mut w = game(&rules());
    let y = deployed_yard(&mut w);
    let spot = Target::Cell(c(6, 6));
    order(&mut w, Order::GuardSpot { actor: y, spot });
    step(&mut w);
    let credits = w.houses[0].credits;
    order(&mut w, Order::Sell { actor: y });
    // The infantry unit stands in the return cell when the build-down ends.
    until(&mut w, 60, |w| {
        w.obj(y)
            .is_some_and(|o| o.econ.sale.is_some() && o.beh.current == Some(Behaviour::Selling))
    })
    .expect("selling");
    testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, c(0, 0), 0);
    until(&mut w, 60, |w| w.obj(y).is_none()).expect("removed");
    assert_eq!(w.houses[0].credits, credits + 500);
    let mcv = w.rules().index_of(Kind::Vehicle, "MCV").unwrap();
    assert!(
        w.objects
            .values()
            .filter(|o| o.kind == Kind::Vehicle && o.type_index == mcv)
            .all(|o| !o.on_map)
    );
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

// spec: S-0415 claim 10 and S-0207 claim 20 (a) (a packed-up teleporting vehicle's guard-spot destination is taken at once: its vehicle-present mark is on the landing cell in the tick it appears)
#[test]
fn claim_10_teleporter_mark_moves_at_once() {
    let mut r = rules();
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("TSHOP".into());
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("TKIT".into());
    r.sim
        .buildings
        .insert("TSHOP".into(), building(2, 2, Some("TKIT")));
    let mut tkit = vehicle("TSHOP");
    tkit.locomotor = Some(Locomotor::Teleport);
    r.sim.vehicles.insert("TKIT".into(), tkit);
    let mut w = game(&r);
    let y = testkit::spawn(&mut w, Kind::Building, "TSHOP", 0, c(0, 0), 0);
    let spot = Target::Cell(c(6, 6));
    order(&mut w, Order::GuardSpot { actor: y, spot });
    step(&mut w);
    order(&mut w, Order::Sell { actor: y });
    until(&mut w, 60, |w| w.obj(y).is_none()).expect("packed up");
    let v = w
        .objects
        .values()
        .find(|o| o.kind == Kind::Vehicle && o.owner == Some(A))
        .map(|o| o.id)
        .expect("the vehicle");
    assert_eq!(w.obj(v).unwrap().dest, Some(spot));
    assert_eq!(glasnost_sim::teleport::mark_cell(&w, v), Some(c(6, 6)));
}

// spec: S-0415 claim 6 and Constants (D15: the deployed building is inserted facing 0, body and turret, whatever the deploy facing and the vehicle's facing)
#[test]
fn claim_6_insertion_facing() {
    let mut w = game(&rules());
    let v = spawn_v(&mut w, "KIT", c(0, 0), facing_d());
    order(&mut w, Order::Deploy { actor: v });
    until(&mut w, 30, |w| w.obj(v).is_none()).expect("deployed");
    let b = w.obj(buildings(&w)[0]).unwrap();
    assert_ne!(facing_d(), 0);
    assert_eq!(b.body.value(w.tick), 0);
    assert_eq!(b.turret.value(w.tick), 0);
}

/// Steps until the vehicle `v` is gone; returns the building it deployed
/// into.
fn deployed_building(w: &mut World, v: ObjectId) -> ObjectId {
    until(w, 30, |w| w.obj(v).is_none()).expect("deployed");
    buildings(w)[0]
}

/// Steps a deployed building `b` to its handover; returns (U, C, H): its
/// first turn, its first construction processing and its handover.
fn deployed_ticks(w: &mut World, b: ObjectId, limit: i32) -> (i32, Option<i32>, Option<i32>) {
    let mut u = None;
    let mut c = None;
    // The turn pass that deployed it may already have given it its turn.
    let o = w.obj(b).unwrap();
    if o.last_turn == w.tick - 1 {
        u = Some(o.last_turn);
        assert_eq!(o.beh.current, Some(Behaviour::Construction));
    }
    for _ in 0..limit {
        let t = w.tick;
        step(w);
        let o = w.obj(b).unwrap();
        if u.is_none() && o.last_turn == t {
            u = Some(t);
            // spec: S-0406 claim 17 (construction current at the second per-tick start of the first turn U)
            assert_eq!(o.beh.current, Some(Behaviour::Construction));
        }
        if c.is_none() {
            c = o.econ.construction.first;
        }
        if !o.econ.construction.building_up {
            return (u.unwrap(), c, Some(t));
        }
    }
    (u.unwrap(), c, None)
}

// spec: S-0406 claim 17 and F7 (deployed: the ready mark at entry, construction current at the building's first turn U, C = U + 1, H = C + (n − 1) × d_b, one build-up run)
#[test]
fn claim_6_deployed_construction_times() {
    let mut w = game(&rules());
    let v = spawn_v(&mut w, "KIT", c(0, 0), facing_d());
    order(&mut w, Order::Deploy { actor: v });
    let b = deployed_building(&mut w, v);
    let (u, cc, h) = deployed_ticks(&mut w, b, 100);
    assert_eq!(cc, Some(u + 1));
    assert_eq!(h, Some(u + 1 + 36));
}

// spec: S-0406 claim 16 (PL13, Q-0220 question 3: a deploy facing outside 0 … 255 never permits deploy completion, since the rounded vehicle facing of S-0415 F1 cannot equal it; no building appears)
#[test]
fn claim_6_out_of_range_deploy_facing_never_deploys() {
    for d in [-1, 256] {
        let mut r = rules();
        r.sim.buildings.get_mut("SHOP").unwrap().econ.deploy_facing = Some(d);
        let mut w = game(&r);
        let v = spawn_v(&mut w, "KIT", c(0, 0), 0);
        order(&mut w, Order::Deploy { actor: v });
        assert_eq!(
            until(&mut w, 200, |w| w.obj(v).is_none()),
            None,
            "deploy facing {d}"
        );
        assert!(buildings(&w).is_empty());
    }
}

// spec: S-0415 claim 6 (D16: the building enters at the end of the update order in the deploy tick T and the vehicle's removal counts for quirks.update_order_removal_skip: directly after the vehicle it loses its turn in T, U = T + 1 and C = T + 2; with another object between them, U = T and C = T + 1; with the switch off U = T in both)
#[test]
fn claim_6_first_turn_after_deploy() {
    for skip in [true, false] {
        for between in [false, true] {
            let mut r = rules();
            r.globals.quirks.update_order_removal_skip = skip;
            let mut w = game(&r);
            let v = spawn_v(&mut w, "KIT", c(0, 0), facing_d());
            let other =
                between.then(|| testkit::spawn(&mut w, Kind::Building, "SHOP", 0, c(-8, -8), 0));
            order(&mut w, Order::Deploy { actor: v });
            let mut t_deploy = None;
            for _ in 0..30 {
                let t = w.tick;
                step(&mut w);
                if w.obj(v).is_none() {
                    t_deploy = Some(t);
                    break;
                }
            }
            let t = t_deploy.expect("deployed");
            let b = buildings(&w)
                .into_iter()
                .find(|&b| Some(b) != other)
                .unwrap();
            let (u, cc, _) = deployed_ticks(&mut w, b, 10);
            let expect = if skip && !between { t + 1 } else { t };
            assert_eq!(u, expect, "skip {skip}, between {between}");
            assert_eq!(cc, Some(expect + 1), "skip {skip}, between {between}");
        }
    }
}

// spec: S-0406 claim 18 (a deployed building with a positive-delay one-stage range makes construction current but never hands over)
#[test]
fn claim_6_deployed_single_stage_never_completes() {
    let mut r = rules();
    r.sim.buildings.get_mut("SHOP").unwrap().econ.buildup_frames = Some(2);
    let mut w = game(&r);
    let v = spawn_v(&mut w, "KIT", c(0, 0), facing_d());
    order(&mut w, Order::Deploy { actor: v });
    let b = deployed_building(&mut w, v);
    let (u, cc, h) = deployed_ticks(&mut w, b, 300);
    assert_eq!(cc, Some(u + 1));
    assert_eq!(h, None);
    assert_eq!(w.obj(b).unwrap().beh.current, Some(Behaviour::Construction));
}

/// A deploy-fire vehicle type FIRE without the simple-deployer flag and
/// with no weapon, so every shot at its own cell is refused (S-0517
/// claim 2).
fn deploy_fire_rules() -> Ruleset {
    let mut r = rules();
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("FIRE".into());
    let mut f = vehicle("SHOP");
    f.deploys_into = None;
    f.deploy_fire = Some(true);
    r.sim.vehicles.insert("FIRE".into(), f);
    r
}

// spec: S-0415 claim 1 rank 2 and S-0517 claim 2, F3 (a deploy-fire vehicle's unload: delay 14 + u, u one draw over 0 … 2 from the scenario stream, taken even when the shot is refused)
#[test]
fn claim_1_deploy_fire_delay_takes_one_draw_when_refused() {
    let mut seen = std::collections::BTreeSet::new();
    for seed in 0..24u32 {
        let mut w = game(&deploy_fire_rules());
        w.rng = glasnost_sim::rng::Rng::new(seed);
        let half = w.rules().quirks.ranged_draw_half_span;
        let v = spawn_v(&mut w, "FIRE", c(0, 0), facing_d());
        w.obj_mut(v).unwrap().beh.current = Some(Behaviour::Unload);
        let mut probe = w.rng.clone();
        let u = probe.ranged(0, 2, half);
        let rule = deploy::process_unload(&mut w, v);
        assert_eq!(rule, Some(deploy::UnloadRule::DeployFire(14 + u)));
        assert_eq!(w.rng, probe, "exactly one draw, seed {seed}");
        seen.insert(14 + u);
    }
    assert_eq!(seen.into_iter().collect::<Vec<_>>(), vec![14, 15, 16]);
}

// spec: S-0415 claim 1 rank 2 and S-0517 claim 2 (unload stays current; every processing chooses 14 … 16 ticks, not the 450-tick no-rule wait; the vehicle neither deploys nor moves)
#[test]
fn claim_1_deploy_fire_processing_delays() {
    let mut w = game(&deploy_fire_rules());
    let v = spawn_v(&mut w, "FIRE", c(0, 0), facing_d());
    order(&mut w, Order::Deploy { actor: v });
    let mut delays = Vec::new();
    let mut prev = None;
    for _ in 0..120 {
        step(&mut w);
        let o = w.obj(v).expect("stays");
        if o.beh.current == Some(Behaviour::Unload) && o.beh.last != prev {
            if let Some((_, d)) = o.beh.last {
                delays.push(d);
            }
            prev = o.beh.last;
        }
    }
    assert!(delays.len() >= 5, "processings: {delays:?}");
    assert!(delays.iter().all(|d| (14..=16).contains(d)), "{delays:?}");
    let o = w.obj(v).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Unload));
    assert_eq!(o.pos.cell(), c(0, 0));
    assert!(buildings(&w).is_empty());
}

/// A KIT ordered to move six cells along open ground, run until it has
/// left its first cell with path steps still stored, then given a deploy
/// order and run for the tick that applies it; returns the vehicle.
fn deploy_while_moving(w: &mut World) -> ObjectId {
    let v = spawn_v(w, "KIT", c(-3, 0), facing_d());
    order(
        w,
        Order::Behaviour {
            actor: v,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(c(3, 0))),
        },
    );
    until(w, 300, |w| {
        w.obj(v)
            .is_some_and(|o| o.pos.cell() != c(-3, 0) && o.path.steps.len() >= 2)
    })
    .expect("on its way with steps still stored");
    order(w, Order::Deploy { actor: v });
    step(w);
    v
}

// spec: S-0213 claim 10 (the deploy order requests an empty destination) and S-0207 claim 11 (the old path steps of a cancelled destination are no continued movement: the engine drops them with it)
#[test]
fn claim_2_deploy_order_drops_the_stored_path() {
    let mut w = game(&rules());
    let v = deploy_while_moving(&mut w);
    let o = w.obj(v).unwrap();
    assert!(o.dest.is_none());
    assert!(o.path.steps.is_empty(), "steps {:?}", o.path.steps);
}

// spec: S-0415 Terms (still) and claim 2 (a vehicle ordered to deploy while moving along a path finishes its current track, stands still with no destination and makes its attempt) and S-0207 claim 11 (it takes no further step of the old path)
#[test]
fn claim_2_deploy_order_while_moving_deploys() {
    let mut w = game(&rules());
    let v = deploy_while_moving(&mut w);
    let stop = w.obj(v).unwrap().pos.cell();
    until(&mut w, 400, |w| w.obj(v).is_none()).expect("deployed after it stopped");
    let b = buildings(&w);
    assert_eq!(b.len(), 1);
    // It deployed within a cell of where it was when the order came, not at the old goal.
    let at = w.obj(b[0]).unwrap().pos.cell();
    assert!(
        (at.x - stop.x).abs() <= 1 && (at.y - stop.y).abs() <= 1,
        "at {at:?}, order at {stop:?}"
    );
}

/// The conversions reported to the presentation since the effects were
/// last cleared.
fn conversions(w: &World) -> Vec<(ObjectId, ObjectId)> {
    w.effects
        .iter()
        .filter_map(|e| match *e {
            Effect::Converted { old, new } => Some((old, new)),
            _ => None,
        })
        .collect()
}

// spec: S-0415 claim 6 (the building takes over the vehicle's control group and selection: the client's; the deploy reports the hand-over once the building is on the map)
#[test]
fn claim_6_deploy_reports_the_hand_over() {
    let mut w = game(&rules());
    let v = spawn_v(&mut w, "MCV", c(0, 0), facing_d());
    order(&mut w, Order::Deploy { actor: v });
    w.effects.clear();
    until(&mut w, 30, |w| w.obj(v).is_none()).expect("deployed");
    let y = buildings(&w)[0];
    assert_eq!(conversions(&w), vec![(v, y)]);
}

// spec: S-0415 claims 4 and 6 (a refused deploy puts no building on the map and reports no hand-over)
#[test]
fn claim_6_refused_deploy_reports_nothing() {
    let mut w = game(&rules());
    let v = spawn_v(&mut w, "KIT", c(0, 0), facing_d());
    testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, c(1, 1), 0);
    order(&mut w, Order::Deploy { actor: v });
    w.effects.clear();
    until(&mut w, 30, |w| sounds(w).contains(&Sound::CannotDeploy)).expect("refused");
    assert!(conversions(&w).is_empty());
}

// spec: S-0415 claim 10 (the vehicle takes over the building's control group and selection, reported on a successful pack-up)
#[test]
fn claim_10_pack_up_reports_the_hand_over() {
    let mut w = game(&rules());
    let y = deployed_yard(&mut w);
    let spot = Target::Cell(c(6, 6));
    order(&mut w, Order::GuardSpot { actor: y, spot });
    step(&mut w);
    order(&mut w, Order::Sell { actor: y });
    w.effects.clear();
    until(&mut w, 60, |w| w.obj(y).is_none()).expect("packed up");
    let v = w
        .objects
        .values()
        .find(|o| o.kind == Kind::Vehicle && o.owner == Some(A))
        .map(|o| o.id)
        .expect("the vehicle");
    assert_eq!(conversions(&w), vec![(y, v)]);
}

// spec: S-0415 claims 10 and 11 (the take-overs require successful placement: a failed pack-up reports no hand-over)
#[test]
fn claim_11_failed_pack_up_reports_nothing() {
    let mut w = game(&rules());
    let y = deployed_yard(&mut w);
    let spot = Target::Cell(c(6, 6));
    order(&mut w, Order::GuardSpot { actor: y, spot });
    step(&mut w);
    testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, c(0, 0), 0);
    order(&mut w, Order::Sell { actor: y });
    w.effects.clear();
    until(&mut w, 60, |w| w.obj(y).is_none()).expect("removed");
    assert!(conversions(&w).is_empty());
}
