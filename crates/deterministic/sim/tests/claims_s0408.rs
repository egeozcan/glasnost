//! Claim tests for S-0408 (selling and building repair) on synthetic data:
//! invented building, overlay and vehicle types, no stock values. Houses
//! Alpha (0) and Beta (1) of the testkit's sandbox game, not allied; the
//! neutral (2) and special (3) houses where a test needs them.

use glasnost_map::{CellXY, NO_OVERLAY};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, OverlayType, UnitType};
use glasnost_sim::combat::effects::{Effect, Sound};
use glasnost_sim::econ::{TypeRef, sell};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, World};

const SIZE: i32 = 24;
const A: HouseId = HouseId(0);
const B: HouseId = HouseId(1);

fn building(cost: i32) -> BuildingType {
    let mut b = BuildingType {
        strength: Some(400),
        cost: Some(cost),
        foundation: Some([1, 1]),
        ..Default::default()
    };
    b.econ.buildup = Some(true);
    // n = 5 build-up frames: build-down 36 ticks (S-0108 claim 5).
    b.econ.buildup_frames = Some(10);
    b
}

/// HUT (cost 1000, upgrade count 2), FREE (cost 1000, free unit CART of
/// cost 600), CHEAP (cost 300, free unit CART), PAD (the pad aircraft's
/// first dock, cost 500), OVER (refund override 77), FLAT (no build-up),
/// FIRE (firestorm wall, no build-up), LOCK (unsellable), UPG (upgrade of
/// HUT, cost 200), WALLA (wall type of overlay O0); aircraft JET (cost 400,
/// docks at PAD) and JET2 (cost 301).
fn rules() -> Ruleset {
    let mut r = testkit::ruleset();
    let names = [
        "WALLA", "HUT", "FREE", "CHEAP", "PAD", "OVER", "FLAT", "FIRE", "LOCK", "UPG",
    ];
    r.registries.insert(
        "buildings".into(),
        names.iter().map(|s| s.to_string()).collect(),
    );
    r.registries
        .insert("aircraft".into(), vec!["JET".into(), "JET2".into()]);
    r.registries
        .insert("overlays".into(), vec!["O0".into(), "O1".into()]);
    let s = &mut r.sim;
    s.overlays.insert(
        "O0".into(),
        OverlayType {
            wall: Some(true),
            has_art: Some(true),
            ..Default::default()
        },
    );
    s.overlays.insert("O1".into(), OverlayType::default());
    let mut hut = building(1000);
    hut.econ.upgrades = Some(2);
    s.buildings.insert("HUT".into(), hut);
    let mut free = building(1000);
    free.econ.free_unit = Some("CART".into());
    s.buildings.insert("FREE".into(), free);
    let mut cheap = building(300);
    cheap.econ.free_unit = Some("CART".into());
    s.buildings.insert("CHEAP".into(), cheap);
    s.buildings.insert("PAD".into(), building(500));
    let mut over = building(1000);
    over.econ.soylent = Some(77);
    s.buildings.insert("OVER".into(), over);
    let mut flat = building(1000);
    flat.econ.buildup = None;
    s.buildings.insert("FLAT".into(), flat);
    let mut fire = building(1000);
    fire.econ.buildup = None;
    fire.firestorm_wall = Some(true);
    s.buildings.insert("FIRE".into(), fire);
    let mut lock = building(1000);
    lock.econ.unsellable = Some(true);
    s.buildings.insert("LOCK".into(), lock);
    let mut upg = building(200);
    upg.econ.powers_up_building = Some("HUT".into());
    s.buildings.insert("UPG".into(), upg);
    let mut wall = building(100);
    wall.econ.wall = Some(true);
    wall.econ.to_overlay = Some("O0".into());
    s.buildings.insert("WALLA".into(), wall);
    s.vehicles.get_mut("CART").unwrap().cost = Some(600);
    let mut jet = UnitType {
        strength: Some(100),
        cost: Some(400),
        ..Default::default()
    };
    jet.econ.dock = Some(vec!["PAD".into()]);
    s.aircraft.insert("JET".into(), jet);
    s.aircraft.insert(
        "JET2".into(),
        UnitType {
            strength: Some(100),
            cost: Some(301),
            ..Default::default()
        },
    );
    s.general.economy.pad_aircraft = Some(vec!["JET".into(), "JET2".into()]);
    r
}

fn game(r: &Ruleset) -> World {
    let mut w = testkit::game_with_non_players(r, SIZE, 13);
    testkit::run(&mut w, 4);
    for h in &mut w.houses {
        h.credits = 10_000;
    }
    w
}

fn c(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn ty(w: &World, name: &str) -> u32 {
    w.rules().index_of(Kind::Building, name).unwrap()
}

fn tr(w: &World, name: &str) -> TypeRef {
    TypeRef {
        kind: Kind::Building,
        index: ty(w, name),
    }
}

fn put(w: &mut World, name: &str, house: u8, cell: CellXY) -> ObjectId {
    testkit::spawn(w, Kind::Building, name, house, cell, 0)
}

fn order(w: &mut World, h: HouseId, o: Order) {
    assert!(glasnost_sim::orders::issue(w, h, o), "order accepted");
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
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

fn set_wall(w: &mut World, cell: CellXY, owner: Option<HouseId>) {
    let g = w.cells.grid.get_mut(cell).unwrap();
    g.overlay = 0;
    g.overlay_data = 0;
    let i = w.cells.idx(cell).unwrap();
    w.cells.wall_owner[i] = owner;
}

// spec: S-0408 claim 1 and F1 (P = trunc(C × b × a), refund = trunc(P × r) for a player house: 1000 and r = 0.5 give 500)
// spec: S-0408 claim 1 (the neutral and special houses receive P)
// spec: S-0408 claim 1 and F1 (a refund override v: trunc(v × a), the percentage plays no part)
#[test]
fn claim_1_refund() {
    let w = game(&rules());
    assert_eq!(sell::refund(&w, A, tr(&w, "HUT")), 500);
    assert_eq!(sell::refund(&w, HouseId(2), tr(&w, "HUT")), 1000);
    assert_eq!(sell::refund(&w, HouseId(3), tr(&w, "HUT")), 1000);
    assert_eq!(sell::refund(&w, A, tr(&w, "OVER")), 77);
    let cart = w.rules().index_of(Kind::Vehicle, "CART").unwrap();
    assert_eq!(
        sell::refund(
            &w,
            A,
            TypeRef {
                kind: Kind::Vehicle,
                index: cart
            }
        ),
        300
    );
}

// spec: S-0408 claim 1 and F1 (the seam S-0312 F5 reads: the owner's refund; none without an owner)
#[test]
fn claim_1_building_refund_seam() {
    let mut w = game(&rules());
    let hut = put(&mut w, "HUT", 0, c(0, 0));
    let over = put(&mut w, "OVER", 0, c(3, 0));
    let neutral = put(&mut w, "HUT", 2, c(6, 0));
    assert_eq!(glasnost_sim::seams::building_refund(&w, hut), Some(500));
    assert_eq!(glasnost_sim::seams::building_refund(&w, over), Some(77));
    assert_eq!(
        glasnost_sim::seams::building_refund(&w, neutral),
        Some(1000)
    );
    w.obj_mut(hut).unwrap().owner = None;
    assert_eq!(glasnost_sim::seams::building_refund(&w, hut), None);
}

// spec: S-0408 F1 (r written as 0.7 is narrowed to the single just below 0.7: 1000 gives 699)
#[test]
fn f1_refund_percentage_single() {
    let mut r = rules();
    r.sim.general.economy.refund_percent = Some(Decimal("0.7".into()));
    let w = game(&r);
    assert_eq!(sell::refund(&w, A, tr(&w, "HUT")), 699);
}

// spec: S-0408 claim 2 and F2 (C = max(0, K − d − F) with a free unit; K − d without one, which can be negative)
// spec: S-0408 F2 (the pad-aircraft deduction trunc((C₀ + C₁) / 2) for the first dock of the first pad aircraft, separate aircraft off)
#[test]
fn claim_2_adjusted_cost() {
    let mut r = rules();
    let w = game(&r);
    assert_eq!(sell::adjusted_cost(&w, ty(&w, "HUT")), 1000);
    assert_eq!(sell::adjusted_cost(&w, ty(&w, "FREE")), 400);
    assert_eq!(sell::adjusted_cost(&w, ty(&w, "CHEAP")), 0);
    // (400 + 301) / 2 = 350.
    assert_eq!(sell::adjusted_cost(&w, ty(&w, "PAD")), 150);
    r.sim.buildings.get_mut("PAD").unwrap().cost = Some(100);
    let w = game(&r);
    assert_eq!(sell::adjusted_cost(&w, ty(&w, "PAD")), -250);
    assert_eq!(sell::refund(&w, A, tr(&w, "PAD")), -125);
    r.sim.general.economy.separate_aircraft = Some(true);
    let w = game(&r);
    assert_eq!(sell::adjusted_cost(&w, ty(&w, "PAD")), 100);
}

// spec: S-0408 claim 3 (a): a build-up sprite, construction completed, neither construction nor selling; (b) a firestorm wall; not unsellable
#[test]
fn claim_3_sellable() {
    let mut w = game(&rules());
    let hut = put(&mut w, "HUT", 0, c(0, 0));
    assert!(sell::sellable(&w, hut));
    let flat = put(&mut w, "FLAT", 0, c(2, 0));
    assert!(!sell::sellable(&w, flat));
    let fire = put(&mut w, "FIRE", 0, c(4, 0));
    assert!(sell::sellable(&w, fire));
    let lock = put(&mut w, "LOCK", 0, c(6, 0));
    assert!(!sell::sellable(&w, lock));
    let t = w.tick;
    w.obj_mut(hut)
        .unwrap()
        .beh
        .force(Behaviour::Construction, t, true);
    assert!(!sell::sellable(&w, hut));
    w.obj_mut(hut).unwrap().beh.force(Behaviour::Guard, t, true);
    w.obj_mut(hut).unwrap().econ.construction.building_up = true;
    assert!(!sell::sellable(&w, hut));
}

// spec: S-0408 claim 4 (no build-up sprite: a firestorm wall is removed at once without refund; any other type: nothing happens)
#[test]
fn claim_4_no_buildup() {
    let mut w = game(&rules());
    let flat = put(&mut w, "FLAT", 0, c(2, 0));
    let fire = put(&mut w, "FIRE", 0, c(4, 0));
    let credits = w.houses[0].credits;
    order(&mut w, A, Order::Sell { actor: flat });
    order(&mut w, A, Order::Sell { actor: fire });
    step(&mut w);
    assert!(w.obj(fire).is_none());
    let f = w.obj(flat).unwrap();
    assert_ne!(f.beh.current, Some(Behaviour::Selling));
    assert_eq!(w.houses[0].credits, credits);
}

// spec: S-0408 claims 4, 6 and 16 (selling: processings on consecutive ticks; release and build-down start at the second; the refund and removal at the first processing after the build-down)
// spec: S-0108 claim 5 (the build-down lasts (n − 1) × d_b = 36 ticks)
#[test]
fn claims_4_6_16_course_of_a_sale() {
    let mut w = game(&rules());
    let hut = put(&mut w, "HUT", 0, c(0, 0));
    let credits = w.houses[0].credits;
    order(&mut w, A, Order::Sell { actor: hut });
    w.effects.clear();
    let mut stages = Vec::new();
    let mut removed_at = None;
    for _ in 0..60 {
        let t = w.tick;
        step(&mut w);
        match w.obj(hut) {
            Some(o) => stages.push((t, o.econ.sale.clone().map(|s| s.stage))),
            None => {
                removed_at = Some(t);
                break;
            }
        }
    }
    let begun = stages
        .iter()
        .find(|(_, s)| matches!(s, Some(sell::Stage::Begun)))
        .unwrap()
        .0;
    let (down, since) = stages
        .iter()
        .find_map(|(t, s)| match s {
            Some(sell::Stage::BuildDown(x)) => Some((*t, *x)),
            _ => None,
        })
        .unwrap();
    assert_eq!(down, begun + 1, "one tick apart");
    assert_eq!(since, down);
    let removed = removed_at.expect("removed");
    assert_eq!(removed, down + 36);
    assert_eq!(w.houses[0].credits, credits + 500);
    assert!(sounds(&w).contains(&Sound::Sell));
}

// spec: S-0408 claim 4 (a sell order to a building already selling changes nothing)
#[test]
fn claim_4_second_order() {
    let mut w = game(&rules());
    let hut = put(&mut w, "HUT", 0, c(0, 0));
    order(&mut w, A, Order::Sell { actor: hut });
    step(&mut w);
    step(&mut w);
    let before = w.obj(hut).unwrap().econ.sale.clone();
    order(&mut w, A, Order::Sell { actor: hut });
    sell::sell(&mut w, A, hut);
    assert_eq!(w.obj(hut).unwrap().econ.sale, before);
}

// spec: S-0408 claim 5 (a sale on an upgraded building sells only its top upgrade: its refund, the upgrade removed, guard, one creation draw)
#[test]
fn claim_5_upgrade_sale() {
    let mut w = game(&rules());
    let hut = put(&mut w, "HUT", 0, c(0, 0));
    let upg = ty(&w, "UPG");
    glasnost_sim::econ::placement::add_upgrade(&mut w, hut, upg);
    glasnost_sim::econ::placement::add_upgrade(&mut w, hut, upg);
    assert_eq!(w.obj(hut).unwrap().econ.building.upgrades.len(), 2);
    let credits = w.houses[0].credits;
    order(&mut w, A, Order::Sell { actor: hut });
    for _ in 0..3 {
        step(&mut w);
    }
    let o = w.obj(hut).unwrap();
    assert_eq!(o.econ.building.upgrades.len(), 1);
    // Request Q-0131: the upgrade count drops by 1.
    assert_eq!(o.econ.building.upgrade_count, 1);
    assert_eq!(o.beh.current, Some(Behaviour::Guard));
    assert_eq!(w.houses[0].credits, credits + 100);
}

// spec: S-0408 claim 6 (a building destroyed during its sale pays no refund)
#[test]
fn claim_6_destroyed_during_sale() {
    let mut w = game(&rules());
    let hut = put(&mut w, "HUT", 0, c(0, 0));
    let credits = w.houses[0].credits;
    order(&mut w, A, Order::Sell { actor: hut });
    for _ in 0..5 {
        step(&mut w);
    }
    glasnost_sim::objects::remove(&mut w, hut);
    for _ in 0..40 {
        step(&mut w);
    }
    assert_eq!(w.houses[0].credits, credits);
}

// spec: S-0408 claim 6 (r4: the sale stops at the killing hit: while the dead building awaits removal, no later selling processing completes the sale or refunds)
#[test]
fn claim_6_sale_stops_at_the_killing_hit() {
    let mut w = game(&rules());
    let hut = put(&mut w, "HUT", 0, c(0, 0));
    let credits = w.houses[0].credits;
    order(&mut w, A, Order::Sell { actor: hut });
    for _ in 0..5 {
        step(&mut w);
    }
    // Dead but still on the map, as while S-0312 defers its removal.
    w.obj_mut(hut).unwrap().health = 0;
    for _ in 0..200 {
        glasnost_sim::econ::sell::process_selling(&mut w, hut);
        step(&mut w);
    }
    assert!(w.obj(hut).is_some_and(|o| o.on_map));
    assert_eq!(w.houses[0].credits, credits);
}

// spec: S-0408 claim 11 (a vehicle on a building cell: removed at once, its refund, the unit-sold announcement; elsewhere nothing)
#[test]
fn claim_11_vehicle_sale() {
    let mut w = game(&rules());
    put(&mut w, "HUT", 0, c(0, 0));
    let on = testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, c(0, 0), 0);
    let off = testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, c(3, 3), 0);
    let credits = w.houses[0].credits;
    w.effects.clear();
    sell::sell(&mut w, A, off);
    assert!(w.obj(off).is_some());
    sell::sell(&mut w, A, on);
    assert!(w.obj(on).is_none());
    assert_eq!(w.houses[0].credits, credits + 300);
    assert_eq!(sounds(&w), vec![Sound::UnitSold]);
}

// spec: S-0408 claim 12 (a sell-wall order removes an owned wall whose builder is sellable; no owner left; the sell sound; the house receives nothing)
// spec: S-0408 claim 12 (the order does not check that the wall's owner is the issuing house)
#[test]
fn claim_12_wall_sale() {
    let mut w = game(&rules());
    set_wall(&mut w, c(0, 0), Some(A));
    set_wall(&mut w, c(2, 0), Some(B));
    set_wall(&mut w, c(4, 0), None);
    let credits = w.houses[0].credits;
    w.effects.clear();
    order(&mut w, A, Order::SellWall { cell: c(0, 0) });
    order(&mut w, A, Order::SellWall { cell: c(2, 0) });
    order(&mut w, A, Order::SellWall { cell: c(4, 0) });
    step(&mut w);
    for x in [0, 2] {
        assert_eq!(w.cells.cell(c(x, 0)).unwrap().overlay, NO_OVERLAY);
        let i = w.cells.idx(c(x, 0)).unwrap();
        assert_eq!(w.cells.wall_owner[i], None);
    }
    assert_eq!(w.cells.cell(c(4, 0)).unwrap().overlay, 0, "no owner: stays");
    assert_eq!(w.houses[0].credits, credits);
    assert_eq!(sounds(&w).iter().filter(|s| **s == Sound::Sell).count(), 2);
}

// spec: S-0408 claim 12 (the first builder of the overlay with the unsellable flag: the wall stays)
#[test]
fn claim_12_unsellable_builder() {
    let mut r = rules();
    r.sim.buildings.get_mut("WALLA").unwrap().econ.unsellable = Some(true);
    let mut w = game(&r);
    set_wall(&mut w, c(0, 0), Some(A));
    assert!(!sell::wall_sale(&mut w, A, c(0, 0), false));
    assert_eq!(w.cells.cell(c(0, 0)).unwrap().overlay, 0);
}

// spec: S-0408 claim 12 (the owner must be a human-controlled house, in our engine a player house: a neutral house's wall stays)
#[test]
fn claim_12_non_player_owner() {
    let mut w = game(&rules());
    set_wall(&mut w, c(0, 0), Some(HouseId(2)));
    assert!(!sell::wall_sale(&mut w, A, c(0, 0), false));
}

// spec: S-0408 claim 13 and S-0314 claim 8 (repair on: a step at each multiple of P_b, paid by the owner; off at the strength)
// spec: S-0213 claim 13 (the repair order switches repair on and off)
#[test]
fn claim_13_building_repair() {
    let mut r = rules();
    r.sim.general.repair_step = Some(10);
    let mut w = game(&r);
    let hut = put(&mut w, "HUT", 0, c(0, 0));
    w.obj_mut(hut).unwrap().health = 385;
    order(&mut w, A, Order::Repair { actor: hut });
    step(&mut w);
    assert!(w.obj(hut).unwrap().econ.building.repairing);
    let pb = w.rules().combat.general.repair_period;
    let credits = w.houses[0].credits;
    let mut steps = Vec::new();
    for _ in 0..(3 * pb) {
        let t = w.tick;
        let h = w.obj(hut).unwrap().health;
        step(&mut w);
        if w.obj(hut).unwrap().health != h {
            steps.push(t);
        }
    }
    assert_eq!(steps.len(), 2, "395, then the strength");
    assert!(steps.iter().all(|t| t % pb == 0));
    let o = w.obj(hut).unwrap();
    assert_eq!(o.health, 400);
    assert!(!o.econ.building.repairing);
    assert!(w.houses[0].credits < credits);
}

// spec: S-0408 claim 14 (repairable: health in (0, S), the repairable and click-repairable flags, not vehicle-like)
#[test]
fn claim_14_repairable() {
    let mut r = rules();
    r.sim
        .buildings
        .get_mut("LOCK")
        .unwrap()
        .econ
        .click_repairable = Some(false);
    let mut w = game(&r);
    let hut = put(&mut w, "HUT", 0, c(0, 0));
    assert!(!sell::repairable(&w, hut), "full health");
    w.obj_mut(hut).unwrap().health = 100;
    assert!(sell::repairable(&w, hut));
    let lock = put(&mut w, "LOCK", 0, c(2, 0));
    w.obj_mut(lock).unwrap().health = 100;
    assert!(!sell::repairable(&w, lock));
}
