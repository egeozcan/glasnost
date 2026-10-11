//! Claim tests for weapon choice (S-0308), on synthetic data.
//!
//! Every type, weapon, warhead and number here is invented. The tests call
//! `weapon::choose_slot`, `weapon::slot_weapon` and `weapon::mean_damage`
//! directly on a world in which no tick has run. The ledger is
//! `docs/m4-claims/S-0308.md`; it lists the claims that are seams
//! (transports, garrisons, upgrades, aircraft in flight, vehicle deploying).

use std::sync::Arc;

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_num::{F64, Rounding};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    ArmourClass, BuildingType, Decimal, LocomotionClass, Locomotor, UnitType, WarheadType,
    WeaponType,
};
use glasnost_sim::combat::acquire::{self, SearchContext, SearchKind};
use glasnost_sim::combat::weapon;
use glasnost_sim::infantry::Seq;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};

const SIZE: i32 = 30;

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

/// Eleven versus values, all `v` except the light class (index 3, S-0305).
fn verses(all: &str, light: &str) -> Option<Vec<Decimal>> {
    let mut v = vec![d(all); 11];
    v[3] = d(light);
    Some(v)
}

fn weapon_type(damage: i32, warhead: &str) -> WeaponType {
    WeaponType {
        damage: Some(damage),
        rof: Some(20),
        range: Some(d("5")),
        speed: Some(100),
        projectile: Some("Bullet".into()),
        warhead: Some(warhead.into()),
        ..Default::default()
    }
}

fn vehicle(strength: i32, armour: ArmourClass) -> UnitType {
    let mut t = UnitType {
        strength: Some(strength),
        speed: Some(5),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        ..Default::default()
    };
    t.combat.armor = Some(armour);
    t
}

/// The test rules: the testkit's combat ruleset plus
/// - warheads `Full` (every versus 1), `NoLight` (0 against light, else 1),
///   `Strike` (airstrike), `Loco` (locomotor),
/// - weapons `A7` and `B13` (damages 7 and 13, warhead Full), `Z7` and
///   `Z13` (warhead NoLight), `N13` (never-use), `S13` (airstrike
///   warhead), `L7` (locomotor warhead), `D13` (draining), `F13`
///   (area-fire), and `C3`, `E11` for the multi-weapon types,
/// - vehicle `FIRER` (primary A7, secondary B13), light vehicle targets
///   `TGT` and `DRAIN` (drainable), building `HUT` (accepts charges) and
///   `HUT2` (does not), infantry `GRUNT` (A7 / B13).
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let s = &mut r.sim;
    s.warheads.insert(
        "Full".into(),
        WarheadType {
            verses: verses("1", "1"),
            ..Default::default()
        },
    );
    s.warheads.insert(
        "NoLight".into(),
        WarheadType {
            verses: verses("1", "0"),
            ..Default::default()
        },
    );
    s.warheads.insert(
        "Strike".into(),
        WarheadType {
            verses: verses("1", "1"),
            airstrike: Some(true),
            ..Default::default()
        },
    );
    s.warheads.insert(
        "Loco".into(),
        WarheadType {
            verses: verses("1", "1"),
            is_locomotor: Some(true),
            ..Default::default()
        },
    );
    for (name, dmg, wh) in [
        ("A7", 7, "Full"),
        ("B13", 13, "Full"),
        ("C3", 3, "Full"),
        ("E11", 11, "Full"),
        ("Z7", 7, "NoLight"),
        ("Z13", 13, "NoLight"),
        ("S13", 13, "Strike"),
        ("L7", 7, "Loco"),
    ] {
        s.weapons.insert(name.into(), weapon_type(dmg, wh));
    }
    s.weapons.insert(
        "N13".into(),
        WeaponType {
            never_use: Some(true),
            ..weapon_type(13, "Full")
        },
    );
    s.weapons.insert(
        "D13".into(),
        WeaponType {
            drain_weapon: Some(true),
            ..weapon_type(13, "Full")
        },
    );
    s.weapons.insert(
        "F13".into(),
        WeaponType {
            area_fire: Some(true),
            ..weapon_type(13, "Full")
        },
    );
    let mut firer = vehicle(200, ArmourClass::Heavy);
    firer.primary = Some("A7".into());
    firer.secondary = Some("B13".into());
    s.vehicles.insert("FIRER".into(), firer);
    s.vehicles
        .insert("TGT".into(), vehicle(100, ArmourClass::Light));
    let mut drain = vehicle(100, ArmourClass::Light);
    drain.combat.drainable = Some(true);
    s.vehicles.insert("DRAIN".into(), drain);
    let mut hover = vehicle(100, ArmourClass::Light);
    hover.locomotion_class = Some(LocomotionClass::Hover);
    s.vehicles.insert("HOVER".into(), hover);
    for (name, can_c4) in [("HUT", true), ("HUT2", false)] {
        s.buildings.insert(
            name.into(),
            BuildingType {
                strength: Some(400),
                foundation: Some([1, 1]),
                can_c4: Some(can_c4),
                combat: glasnost_rules::sim::CombatFields {
                    armor: Some(ArmourClass::Light),
                    ..Default::default()
                },
                ..Default::default()
            },
        );
    }
    let mut grunt = UnitType {
        strength: Some(50),
        speed: Some(4),
        locomotion_class: Some(LocomotionClass::Foot),
        locomotor: Some(Locomotor::Walk),
        primary: Some("A7".into()),
        secondary: Some("B13".into()),
        ..Default::default()
    };
    grunt.combat.armor = Some(ArmourClass::Light);
    s.infantry.insert("GRUNT".into(), grunt);
    for name in ["FIRER", "TGT", "DRAIN", "HOVER"] {
        r.registries
            .get_mut("vehicles")
            .expect("vehicles registry")
            .push(name.into());
    }
    r.registries
        .get_mut("buildings")
        .expect("buildings registry")
        .extend(["HUT".to_string(), "HUT2".to_string()]);
    r.registries
        .get_mut("infantry")
        .expect("infantry registry")
        .push("GRUNT".into());
    r
}

fn firer_mut(r: &mut Ruleset) -> &mut UnitType {
    r.sim.vehicles.get_mut("FIRER").expect("FIRER")
}

fn game(r: &Ruleset) -> World {
    testkit::game_with(r, SIZE, 5)
}

fn put(w: &mut World, kind: Kind, ty: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, ty, house, at(dx, dy), 0)
}

fn slot(w: &World, firer: ObjectId, t: ObjectId) -> usize {
    weapon::choose_slot(w, firer, Some(Target::Object(t)))
}

fn weapon_named(w: &World, name: &str) -> glasnost_sim::combat::data::WeaponId {
    let r = w.rules();
    (0..r.combat.weapons.len() as u32)
        .find(|&i| r.combat.weapon(i).name == name)
        .unwrap_or_else(|| panic!("no weapon {name}"))
}

fn set_land(w: &mut World, c: CellXY, l: u8) {
    w.cells.grid.get_mut(c).expect("legal cell").land = l;
}

/// A firer `FIRER` at the centre and a target of `ty` two cells east.
fn duel(r: &Ruleset, kind: Kind, ty: &str, house: u8) -> (World, ObjectId, ObjectId) {
    let mut w = game(r);
    let f = put(&mut w, Kind::Vehicle, "FIRER", 0, 0, 0);
    let t = put(&mut w, kind, ty, house, 2, 0);
    (w, f, t)
}

// ---------------------------------------------------------------------------
// Weapon slots and elite weapons

// spec: S-0308 claim 1 (a slot list fills slots 0 … N − 1; the primary and secondary fill slots 0 and 1 when there is none)
#[test]
fn slots_come_from_the_type() {
    let mut r = rules();
    let mut multi = vehicle(200, ArmourClass::Heavy);
    multi.turret_count = Some(1);
    multi.weapon_count = Some(3);
    multi.primary = Some("A7".into());
    multi.combat.weapon_slots = Some(vec!["C3".into(), "B13".into(), "E11".into()]);
    r.sim.vehicles.insert("MULTI".into(), multi);
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("MULTI".into());
    let mut w = game(&r);
    let m = put(&mut w, Kind::Vehicle, "MULTI", 0, 0, 0);
    let f = put(&mut w, Kind::Vehicle, "FIRER", 0, 1, 1);
    let name = |w: &World, id, i| {
        weapon::slot_weapon(w, id, i).map(|x| w.rules().combat.weapon(x).name.clone())
    };
    assert_eq!(name(&w, m, 0).as_deref(), Some("C3"));
    assert_eq!(name(&w, m, 1).as_deref(), Some("B13"));
    assert_eq!(name(&w, m, 2).as_deref(), Some("E11"));
    assert_eq!(name(&w, m, 3), None);
    assert_eq!(name(&w, f, 0).as_deref(), Some("A7"));
    assert_eq!(name(&w, f, 1).as_deref(), Some("B13"));
    assert_eq!(name(&w, f, 2), None);
}

/// Experience 2, the elite rank (S-0311 claim 1).
fn make_elite(w: &mut World, id: ObjectId) {
    let two = F64::from_i64(2, Rounding::TowardZero).to_f32(Rounding::TowardZero);
    w.obj_mut(id).unwrap().combat.experience = two;
}

// spec: S-0308 claim 2 (elite: the elite slot's weapon when it holds one, per slot; lower ranks never use the elite slots)
#[test]
fn elite_replacement_is_per_slot() {
    let mut r = rules();
    firer_mut(&mut r).elite_primary = Some("E11".into());
    let mut w = game(&r);
    let f = put(&mut w, Kind::Vehicle, "FIRER", 0, 0, 0);
    let a7 = weapon_named(&w, "A7");
    let b13 = weapon_named(&w, "B13");
    let e11 = weapon_named(&w, "E11");
    assert_eq!(weapon::slot_weapon(&w, f, 0), Some(a7), "rookie: the slot");
    let one = F64::from_i64(1, Rounding::TowardZero).to_f32(Rounding::TowardZero);
    w.obj_mut(f).unwrap().combat.experience = one;
    assert_eq!(weapon::slot_weapon(&w, f, 0), Some(a7), "veteran: the slot");
    make_elite(&mut w, f);
    assert_eq!(weapon::slot_weapon(&w, f, 0), Some(e11), "elite primary");
    assert_eq!(
        weapon::slot_weapon(&w, f, 1),
        Some(b13),
        "no elite secondary: the secondary"
    );
}

// ---------------------------------------------------------------------------
// The precedence

// spec: S-0308 claim 3 (no row draws from a random stream; asking twice gives the same answer)
#[test]
fn the_choice_draws_nothing_and_repeats() {
    let mut r = rules();
    firer_mut(&mut r).primary = Some("Z7".into());
    let (w, f, t) = duel(&r, Kind::Vehicle, "TGT", 1);
    let rng = w.rng.clone();
    let first = slot(&w, f, t);
    assert_eq!(first, 1);
    assert_eq!(slot(&w, f, t), first);
    assert_eq!(w.rng, rng);
}

// spec: S-0308 claim 4 (a firer with an empty secondary uses slot 0 against every target)
#[test]
fn single_weapon_firer_uses_slot_zero() {
    let mut r = rules();
    // A primary with a zero versus value against the target: row 16 would give 1.
    firer_mut(&mut r).primary = Some("Z7".into());
    firer_mut(&mut r).secondary = None;
    let (w, f, t) = duel(&r, Kind::Vehicle, "TGT", 1);
    assert_eq!(slot(&w, f, t), 0);
}

// spec: S-0308 claim 4 and table P row 3 (a never-use secondary: slot 0)
#[test]
fn never_use_secondary_gives_slot_zero() {
    let mut r = rules();
    firer_mut(&mut r).primary = Some("Z7".into());
    firer_mut(&mut r).secondary = Some("N13".into());
    let (w, f, t) = duel(&r, Kind::Vehicle, "TGT", 1);
    assert_eq!(slot(&w, f, t), 0);
}

// spec: S-0308 table P row 3 (slot 0 empty, or no target: 0)
#[test]
fn row_three_empty_primary_or_no_target() {
    let mut r = rules();
    firer_mut(&mut r).primary = Some("Z7".into());
    let (w, f, _) = duel(&r, Kind::Vehicle, "TGT", 1);
    assert_eq!(weapon::choose_slot(&w, f, None), 0, "no target");
    let mut r = rules();
    firer_mut(&mut r).primary = None;
    let (w, f, t) = duel(&r, Kind::Vehicle, "TGT", 1);
    assert_eq!(slot(&w, f, t), 0, "slot 0 empty");
}

// ---------------------------------------------------------------------------
// Multi-weapon types

fn multi_rules(gattling: bool, charge: bool, slots: &[&str]) -> Ruleset {
    let mut r = rules();
    let mut m = vehicle(200, ArmourClass::Heavy);
    m.turret_count = Some(slots.len() as i32);
    m.weapon_count = Some(slots.len() as i32);
    m.combat.weapon_slots = Some(slots.iter().map(|s| s.to_string()).collect());
    m.combat.is_gattling = Some(gattling);
    m.combat.is_charge_turret = Some(charge);
    r.sim.vehicles.insert("MULTI".into(), m);
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("MULTI".into());
    r
}

// spec: S-0308 claim 5 and table P row 1 (a switching type uses the slot of its weapon number against every target, before every other row)
#[test]
fn switching_type_uses_its_weapon_number() {
    // Slot 1 empty: row 3 would give 0, but row 1 comes first.
    let r = multi_rules(false, false, &["A7", "", "Z13", "E11"]);
    let mut w = game(&r);
    let m = put(&mut w, Kind::Vehicle, "MULTI", 0, 0, 0);
    let t = put(&mut w, Kind::Vehicle, "TGT", 1, 2, 0);
    assert_eq!(slot(&w, m, t), 0, "weapon number starts at 0");
    weapon::set_weapon_number(&mut w, m, 2);
    // Slot 2's warhead has a zero versus value against the light target: no versus row applies.
    assert_eq!(slot(&w, m, t), 2);
    assert_eq!(weapon::choose_slot(&w, m, None), 2, "also with no target");
    weapon::set_weapon_number(&mut w, m, 3);
    assert_eq!(slot(&w, m, t), 3);
}

// spec: S-0308 claim 5 (a weapon number outside 0 … 17 is set to 0)
#[test]
fn weapon_number_outside_the_slots_becomes_zero() {
    let r = multi_rules(false, false, &["A7", "B13", "E11"]);
    let mut w = game(&r);
    let m = put(&mut w, Kind::Vehicle, "MULTI", 0, 0, 0);
    weapon::set_weapon_number(&mut w, m, 2);
    assert_eq!(weapon::weapon_number(&w, m), 2);
    weapon::set_weapon_number(&mut w, m, 25);
    assert_eq!(weapon::weapon_number(&w, m), 0);
    weapon::set_weapon_number(&mut w, m, 17);
    assert_eq!(weapon::weapon_number(&w, m), 17);
    weapon::set_weapon_number(&mut w, m, -1);
    assert_eq!(weapon::weapon_number(&w, m), 0);
}

// spec: S-0308 claim 5 (a charge-turret type ignores every request to change its weapon number)
#[test]
fn charge_turret_keeps_weapon_number_zero() {
    let r = multi_rules(false, true, &["A7", "B13", "E11"]);
    let mut w = game(&r);
    let m = put(&mut w, Kind::Vehicle, "MULTI", 0, 0, 0);
    let t = put(&mut w, Kind::Vehicle, "TGT", 1, 2, 0);
    weapon::set_weapon_number(&mut w, m, 2);
    assert_eq!(weapon::weapon_number(&w, m), 0);
    assert_eq!(slot(&w, m, t), 0);
}

// spec: S-0308 claim 7, F2 and table P row 5 (a gattling type at stage s uses slot 2s against a target that is not airborne)
#[test]
fn gattling_type_uses_slot_two_s() {
    let r = multi_rules(true, false, &["C3", "B13", "A7", "E11", "A7", "E11"]);
    let mut w = game(&r);
    let g = put(&mut w, Kind::Vehicle, "MULTI", 0, 0, 0);
    let t = put(&mut w, Kind::Vehicle, "TGT", 1, 2, 0);
    assert_eq!(slot(&w, g, t), 0);
    w.obj_mut(g).unwrap().combat.gattling_stage = 1;
    assert_eq!(slot(&w, g, t), 2);
    w.obj_mut(g).unwrap().combat.gattling_stage = 2;
    assert_eq!(slot(&w, g, t), 4);
}

// spec: S-0308 claim 7 (a gattling type with an empty slot 1 uses slot 0 at every stage)
#[test]
fn gattling_type_with_empty_slot_one_uses_slot_zero() {
    let r = multi_rules(true, false, &["C3", "", "A7", "E11"]);
    let mut w = game(&r);
    let g = put(&mut w, Kind::Vehicle, "MULTI", 0, 0, 0);
    let t = put(&mut w, Kind::Vehicle, "TGT", 1, 2, 0);
    w.obj_mut(g).unwrap().combat.gattling_stage = 1;
    assert_eq!(slot(&w, g, t), 0);
}

// ---------------------------------------------------------------------------
// Deployed infantry

fn deploy_rules(index: Option<i32>) -> Ruleset {
    let mut r = rules();
    let g = r.sim.infantry.get_mut("GRUNT").unwrap();
    // A primary with a zero versus value against light targets: row 16 would give 1.
    g.primary = Some("Z7".into());
    g.deploy_fire = Some(true);
    g.deploy_fire_weapon = index;
    r
}

fn set_seq(w: &mut World, id: ObjectId, seq: Seq) {
    w.obj_mut(id).unwrap().infantry.as_mut().unwrap().seq = Some(seq);
}

// spec: S-0308 claim 8 (deploy-fire infantry not in a deploy sequence: slot 0, table P not applied)
#[test]
fn deploy_fire_infantry_undeployed_uses_slot_zero() {
    let r = deploy_rules(None);
    let mut w = game(&r);
    let g = put(&mut w, Kind::Infantry, "GRUNT", 0, 0, 0);
    let t = put(&mut w, Kind::Vehicle, "TGT", 1, 2, 0);
    set_seq(&mut w, g, Seq::Stand);
    assert_eq!(slot(&w, g, t), 0);
}

// spec: S-0308 claim 8 and Constants (in a deploy sequence: the deploy weapon index, absent 1)
#[test]
fn deploy_fire_infantry_deployed_uses_deploy_index() {
    let r = deploy_rules(None);
    let mut w = game(&r);
    let g = put(&mut w, Kind::Infantry, "GRUNT", 0, 0, 0);
    let t = put(&mut w, Kind::Vehicle, "TGT", 1, 2, 0);
    for seq in [
        Seq::Deploy,
        Seq::Deployed,
        Seq::DeployedFire,
        Seq::DeployedIdle,
    ] {
        set_seq(&mut w, g, seq);
        assert_eq!(slot(&w, g, t), 1, "{seq:?}");
    }
    let r = deploy_rules(Some(0));
    let mut w = game(&r);
    let g = put(&mut w, Kind::Infantry, "GRUNT", 0, 0, 0);
    let t = put(&mut w, Kind::Vehicle, "TGT", 1, 2, 0);
    set_seq(&mut w, g, Seq::Deployed);
    assert_eq!(slot(&w, g, t), 0, "deploy weapon index 0");
}

// spec: S-0308 Edge cases (deploy weapon index outside 0 … 17: treated as 0, a departure)
#[test]
fn deploy_index_outside_the_slots_is_zero() {
    let r = deploy_rules(Some(30));
    let mut w = game(&r);
    let g = put(&mut w, Kind::Infantry, "GRUNT", 0, 0, 0);
    let t = put(&mut w, Kind::Vehicle, "TGT", 1, 2, 0);
    set_seq(&mut w, g, Seq::Deployed);
    assert_eq!(slot(&w, g, t), 0);
}

// ---------------------------------------------------------------------------
// Table P, rows 6 to 20

// spec: S-0308 table P row 6 (an airstrike secondary: 1 against a building that accepts charges)
#[test]
fn row_six_airstrike_secondary_against_charge_building() {
    let mut r = rules();
    firer_mut(&mut r).secondary = Some("S13".into());
    let (w, f, t) = duel(&r, Kind::Building, "HUT", 1);
    assert_eq!(slot(&w, f, t), 1);
    let (w, f, t) = duel(&r, Kind::Building, "HUT2", 1);
    assert_eq!(slot(&w, f, t), 0, "a building that does not accept charges");
}

// spec: S-0308 table P row 6 and Edge cases (the airstrike row is final: a zero primary versus value does not give 1)
#[test]
fn row_six_airstrike_row_is_final() {
    let mut r = rules();
    firer_mut(&mut r).primary = Some("Z7".into());
    firer_mut(&mut r).secondary = Some("S13".into());
    let (w, f, t) = duel(&r, Kind::Vehicle, "TGT", 1);
    assert_eq!(slot(&w, f, t), 0);
}

// spec: S-0308 table P row 7 (a locomotor primary against a building: 1)
#[test]
fn row_seven_locomotor_primary_against_building() {
    let mut r = rules();
    firer_mut(&mut r).primary = Some("L7".into());
    let (w, f, t) = duel(&r, Kind::Building, "HUT2", 1);
    assert_eq!(slot(&w, f, t), 1);
    let (w, f, t) = duel(&r, Kind::Vehicle, "TGT", 1);
    assert_eq!(slot(&w, f, t), 0, "not a building: row 20");
}

// spec: S-0308 table P row 8 (a draining secondary against a drainable target that is not allied: 1)
#[test]
fn row_eight_draining_secondary() {
    let mut r = rules();
    firer_mut(&mut r).secondary = Some("D13".into());
    let (w, f, t) = duel(&r, Kind::Vehicle, "DRAIN", 1);
    assert_eq!(slot(&w, f, t), 1);
    let (w, f, t) = duel(&r, Kind::Vehicle, "DRAIN", 0);
    assert_eq!(slot(&w, f, t), 0, "allied: row 20");
    let (w, f, t) = duel(&r, Kind::Vehicle, "TGT", 1);
    assert_eq!(slot(&w, f, t), 0, "not drainable: row 20");
}

// spec: S-0308 table P row 9 (an area-fire secondary while the current behaviour is unload: 1)
#[test]
fn row_nine_area_fire_secondary_in_unload() {
    let mut r = rules();
    firer_mut(&mut r).secondary = Some("F13".into());
    let (mut w, f, t) = duel(&r, Kind::Vehicle, "TGT", 1);
    assert_eq!(slot(&w, f, t), 0, "in guard");
    let tick = w.tick;
    w.obj_mut(f).unwrap().beh.force(Behaviour::Unload, tick);
    assert_eq!(slot(&w, f, t), 1);
}

fn land_mode(r: &mut Ruleset, mode: i32) {
    firer_mut(r).combat.land_targeting = Some(mode);
}

// spec: S-0308 table P row 13 (a land cell target with land-targeting mode 2: 1)
#[test]
fn row_thirteen_land_cell_with_land_mode_two() {
    let mut r = rules();
    land_mode(&mut r, 2);
    let (w, f, _) = duel(&r, Kind::Vehicle, "TGT", 1);
    assert_eq!(weapon::choose_slot(&w, f, Some(Target::Cell(at(3, 3)))), 1);
}

// spec: S-0308 table P rows 13 and 14 (a water cell, or a cell whose tile lies in the water role's 14 tile indices, is not row 13: row 14 gives 0)
#[test]
fn row_thirteen_water_cells_fall_through_to_row_fourteen() {
    let mut r = rules();
    land_mode(&mut r, 2);
    let (mut w, f, _) = duel(&r, Kind::Vehicle, "TGT", 1);
    let c = at(3, 3);
    set_land(&mut w, c, land::WATER);
    assert_eq!(weapon::choose_slot(&w, f, Some(Target::Cell(c))), 0);
    // A clear cell whose tile is the last of the 14 indices from the water role's start.
    let mut cat = (*testkit::catalogue()).clone();
    cat.roles.water = 1;
    w.catalogue = Some(Arc::new(cat));
    let c2 = at(-3, 3);
    w.cells.grid.get_mut(c2).unwrap().tile = 14;
    assert_eq!(weapon::choose_slot(&w, f, Some(Target::Cell(c2))), 0);
    let c3 = at(-3, -3);
    w.cells.grid.get_mut(c3).unwrap().tile = 15;
    assert_eq!(
        weapon::choose_slot(&w, f, Some(Target::Cell(c3))),
        1,
        "tile 15 is past the 14 indices from 1"
    );
}

// spec: S-0308 table P row 14 (a cell target with land mode 0: 0)
#[test]
fn row_fourteen_cell_target() {
    let mut r = rules();
    firer_mut(&mut r).primary = Some("Z7".into());
    let (w, f, _) = duel(&r, Kind::Vehicle, "TGT", 1);
    assert_eq!(weapon::choose_slot(&w, f, Some(Target::Cell(at(3, 3)))), 0);
}

// spec: S-0308 table P rows 15 and 16 (a zero secondary versus value gives 0, else a zero primary versus value gives 1; row 15 first)
#[test]
fn rows_fifteen_and_sixteen_zero_versus_values() {
    let mut r = rules();
    firer_mut(&mut r).secondary = Some("Z13".into());
    let (w, f, t) = duel(&r, Kind::Vehicle, "TGT", 1);
    assert_eq!(slot(&w, f, t), 0, "row 15");
    let mut r = rules();
    firer_mut(&mut r).primary = Some("Z7".into());
    let (w, f, t) = duel(&r, Kind::Vehicle, "TGT", 1);
    assert_eq!(slot(&w, f, t), 1, "row 16");
    let mut r = rules();
    firer_mut(&mut r).primary = Some("Z7".into());
    firer_mut(&mut r).secondary = Some("Z13".into());
    let (w, f, t) = duel(&r, Kind::Vehicle, "TGT", 1);
    assert_eq!(slot(&w, f, t), 0, "both zero: row 15 before row 16");
}

// spec: S-0308 table P rows 18 and 20 (land-targeting mode 2 against a target that is not airborne: 1; otherwise 0)
#[test]
fn rows_eighteen_and_twenty() {
    let r = rules();
    let (w, f, t) = duel(&r, Kind::Vehicle, "TGT", 1);
    assert_eq!(slot(&w, f, t), 0, "row 20");
    let mut r = rules();
    land_mode(&mut r, 2);
    let (w, f, t) = duel(&r, Kind::Vehicle, "TGT", 1);
    assert_eq!(slot(&w, f, t), 1, "row 18");
}

// spec: S-0308 table P rows 15 and 18 (a zero versus value overrides the land row)
#[test]
fn zero_versus_rows_before_land_row() {
    let mut r = rules();
    land_mode(&mut r, 2);
    firer_mut(&mut r).secondary = Some("Z13".into());
    let (w, f, t) = duel(&r, Kind::Vehicle, "TGT", 1);
    assert_eq!(slot(&w, f, t), 0);
}

// ---------------------------------------------------------------------------
// Water targets and table N

fn naval_mode(r: &mut Ruleset, mode: i32) {
    firer_mut(r).combat.naval_targeting = Some(mode);
}

/// A light target type `WET` with the given underwater, organic and
/// unnatural flags.
fn wet(r: &mut Ruleset, underwater: bool, organic: bool, unnatural: bool) {
    let mut t = vehicle(100, ArmourClass::Light);
    t.combat.underwater = Some(underwater);
    t.combat.organic = Some(organic);
    t.combat.unnatural = Some(unnatural);
    r.sim.vehicles.insert("WET".into(), t);
    r.registries.get_mut("vehicles").unwrap().push("WET".into());
}

/// The slot `FIRER` of naval mode `mode` chooses against a `ty` target
/// standing on a cell of land type `l`.
fn water_slot(mut r: Ruleset, mode: i32, ty: &str, l: u8) -> usize {
    naval_mode(&mut r, mode);
    let (mut w, f, t) = duel(&r, Kind::Vehicle, ty, 1);
    let c = w.obj(t).unwrap().pos.cell();
    set_land(&mut w, c, l);
    slot(&w, f, t)
}

fn wet_rules(underwater: bool, organic: bool, unnatural: bool) -> Ruleset {
    let mut r = rules();
    wet(&mut r, underwater, organic, unnatural);
    r
}

// spec: S-0308 claim 11 and table N mode 1 (1 against an underwater type on water, else 0)
#[test]
fn table_n_mode_one() {
    assert_eq!(
        water_slot(wet_rules(true, false, false), 1, "WET", land::WATER),
        1
    );
    assert_eq!(
        water_slot(wet_rules(false, false, false), 1, "WET", land::WATER),
        0
    );
}

// spec: S-0308 Terms (water target: water or beach) and table P row 17
#[test]
fn beach_cell_is_a_water_target() {
    assert_eq!(
        water_slot(wet_rules(true, false, false), 1, "WET", land::BEACH),
        1
    );
    assert_eq!(
        water_slot(wet_rules(true, false, false), 1, "WET", land::CLEAR),
        0,
        "on land: row 20"
    );
}

// spec: S-0308 table N mode 3 (1 against an organic or unnatural type, else 0)
#[test]
fn table_n_mode_three() {
    assert_eq!(
        water_slot(wet_rules(false, true, false), 3, "WET", land::WATER),
        1
    );
    assert_eq!(
        water_slot(wet_rules(false, false, true), 3, "WET", land::WATER),
        1
    );
    assert_eq!(
        water_slot(wet_rules(true, false, false), 3, "WET", land::WATER),
        0
    );
}

// spec: S-0308 table N mode 4 (1 when the target's speed type is not hover and its type is not organic)
#[test]
fn table_n_mode_four() {
    assert_eq!(
        water_slot(wet_rules(false, false, false), 4, "WET", land::WATER),
        1
    );
    assert_eq!(
        water_slot(wet_rules(false, true, false), 4, "WET", land::WATER),
        0
    );
    assert_eq!(
        water_slot(rules(), 4, "HOVER", land::WATER),
        0,
        "a hover target"
    );
}

// spec: S-0308 table N modes 0, 2, 5, 6, 7 and other values (slot 0 for every target; the "no weapon" answers are slot 0)
#[test]
fn table_n_modes_giving_slot_zero() {
    for mode in [0, 2, 5, 6, 7, 8, -1] {
        for (u, o, n) in [
            (true, false, false),
            (false, true, false),
            (false, false, true),
            (false, false, false),
        ] {
            assert_eq!(
                water_slot(wet_rules(u, o, n), mode, "WET", land::WATER),
                0,
                "mode {mode}, flags {u} {o} {n}"
            );
        }
    }
}

// spec: S-0308 table P row 17 (a water target follows table N even with land-targeting mode 2, row 17 before row 18)
#[test]
fn water_row_before_land_row() {
    let mut r = wet_rules(false, false, false);
    land_mode(&mut r, 2);
    assert_eq!(water_slot(r, 1, "WET", land::WATER), 0);
}

// ---------------------------------------------------------------------------
// Uses of the weapon choice

// spec: S-0308 claim 14 (the threat search's candidate tests look at the weapon chosen here)
#[test]
fn searches_test_the_chosen_weapon() {
    // A primary with a zero versus value: row 16 gives the secondary, which passes table C's versus row.
    let mut r = rules();
    firer_mut(&mut r).primary = Some("Z7".into());
    firer_mut(&mut r).guard_range = Some(d("4"));
    let (mut w, f, t) = duel(&r, Kind::Vehicle, "TGT", 1);
    assert_eq!(
        acquire::search(&mut w, f, SearchKind::Range, SearchContext::PerTick),
        Some(t)
    );
    // Both zero: row 15 gives the primary, whose versus value rejects the target.
    let mut r = rules();
    firer_mut(&mut r).primary = Some("Z7".into());
    firer_mut(&mut r).secondary = Some("Z13".into());
    firer_mut(&mut r).guard_range = Some(d("4"));
    let (mut w, f, _) = duel(&r, Kind::Vehicle, "TGT", 1);
    assert_eq!(
        acquire::search(&mut w, f, SearchKind::Range, SearchContext::PerTick),
        None
    );
}

// ---------------------------------------------------------------------------
// Mean weapon damage

/// Mean weapon damage of a FIRER whose slots hold weapons of the given
/// damages (`None`: empty), with ambient damage `amb` on the primary.
fn mean(p: Option<i32>, s: Option<i32>, amb: i32) -> i32 {
    let mut r = rules();
    if let Some(p) = p {
        r.sim.weapons.insert(
            "MP".into(),
            WeaponType {
                ambient_damage: Some(amb),
                ..weapon_type(p, "Full")
            },
        );
    }
    if let Some(s) = s {
        r.sim.weapons.insert("MS".into(), weapon_type(s, "Full"));
    }
    firer_mut(&mut r).primary = p.map(|_| "MP".into());
    firer_mut(&mut r).secondary = s.map(|_| "MS".into());
    let (w, f, _) = duel(&r, Kind::Vehicle, "TGT", 1);
    weapon::mean_damage(&w, f)
}

// spec: S-0308 claim 15 and F1 (M = trunc((d₀ + d₁) / 2), or the one slot's d, or 0)
#[test]
fn mean_weapon_damage_values() {
    assert_eq!(mean(Some(-5), Some(3), 0), -1, "trunc(−2 / 2)");
    assert_eq!(mean(Some(-3), Some(3), 0), 0);
    assert_eq!(mean(Some(-1), Some(2), 0), 0, "trunc(1 / 2)");
    assert_eq!(mean(Some(-1), Some(4), 0), 1, "trunc(3 / 2)");
    assert_eq!(mean(Some(-4), Some(1), 0), -1, "trunc(−3 / 2) toward zero");
    assert_eq!(mean(Some(9), None, 0), 9, "only slot 0");
    assert_eq!(mean(None, Some(6), 0), 6, "only slot 1");
    assert_eq!(mean(None, None, 0), 0, "neither");
    assert_eq!(mean(Some(3), Some(5), 2), 5, "d₀ = damage + ambient = 5");
}

// spec: S-0308 claim 15 (a healer for acquisition: M below 0)
#[test]
fn healer_for_acquisition_is_negative_mean() {
    let mut r = rules();
    r.sim.weapons.insert("NEG".into(), weapon_type(-5, "Full"));
    r.sim.weapons.insert("POS".into(), weapon_type(3, "Full"));
    firer_mut(&mut r).primary = Some("NEG".into());
    firer_mut(&mut r).secondary = Some("POS".into());
    let (w, f, t) = duel(&r, Kind::Vehicle, "TGT", 1);
    assert!(weapon::healer_for_acquisition(&w, f));
    assert!(!weapon::healer_for_acquisition(&w, t), "unarmed: M = 0");
}

// spec: S-0308 claim 15 (a switching type: d of the slot of its weapon number, 0 when that slot is empty)
#[test]
fn mean_damage_of_switching_type() {
    let r = multi_rules(false, false, &["A7", "B13", "", "E11"]);
    let mut w = game(&r);
    let m = put(&mut w, Kind::Vehicle, "MULTI", 0, 0, 0);
    assert_eq!(weapon::mean_damage(&w, m), 7);
    weapon::set_weapon_number(&mut w, m, 3);
    assert_eq!(weapon::mean_damage(&w, m), 11);
    weapon::set_weapon_number(&mut w, m, 2);
    assert_eq!(weapon::mean_damage(&w, m), 0);
}
