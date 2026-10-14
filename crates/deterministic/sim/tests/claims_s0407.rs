//! Claim tests for S-0407 (tech tree) on synthetic data: invented types,
//! countries and lists, no stock numbers.

mod econ_common;

use econ_common::*;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::BuildingType;
use glasnost_sim::econ::TypeRef;
use glasnost_sim::econ::tech::{self, Answer};
use glasnost_sim::objects;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, World};

const H: HouseId = HouseId(0);

fn add_building(r: &mut Ruleset, n: &str, b: BuildingType) {
    r.registries.get_mut("buildings").unwrap().push(n.into());
    r.sim.buildings.insert(n.into(), b);
}

fn add_infantry(r: &mut Ruleset, n: &str, f: impl FnOnce(&mut glasnost_rules::sim::UnitType)) {
    let mut u = foot(100);
    f(&mut u);
    r.registries.get_mut("infantry").unwrap().push(n.into());
    r.sim.infantry.insert(n.into(), u);
}

fn offered(w: &World, ty: TypeRef) -> bool {
    tech::offered(w, H, ty)
}

fn vehicle(w: &mut World, name: &str, h: u8, dx: i32, dy: i32) -> glasnost_sim::world::ObjectId {
    testkit::spawn(w, Kind::Vehicle, name, h, testkit::mid(SIZE, dx, dy), 0)
}

// spec: S-0407 claim 1 (offered once a factory could make it; the recheck in the house update after the placement)
#[test]
fn claim_1_offer_after_placement() {
    let mut w = game(&ruleset());
    step(&mut w);
    let grunt = t(&w, Kind::Infantry, "GRUNT");
    assert!(!offered(&w, grunt));
    building(&mut w, "BARRACKS", 0, 0, 0);
    assert!(!offered(&w, grunt));
    step(&mut w);
    assert!(offered(&w, grunt));
    assert!(!offered(&w, t(&w, Kind::Vehicle, "TANK")));
}

// spec: S-0407 claim 1 (b) (T2 run 3: a naval type with only a ground factory never enters the set)
#[test]
fn claim_1_naval_kind() {
    let mut w = game(&ruleset());
    building(&mut w, "WARFACT", 0, 0, 0);
    step(&mut w);
    assert!(offered(&w, t(&w, Kind::Vehicle, "TANK")));
    assert!(!offered(&w, t(&w, Kind::Vehicle, "BOAT")));
}

// spec: S-0407 Import mapping (Factory: S-0003 claim 21; a recognised word that names no production kind makes no factory of a kind)
// spec: S-0003 claim 21 (the object-instance words are not aliases for the type-kind words)
#[test]
fn claim_1_object_category_is_no_factory() {
    let mut r = ruleset();
    add_building(&mut r, "CAMP", factory("infantry_object"));
    let mut w = game(&r);
    building(&mut w, "CAMP", 0, 0, 0);
    step(&mut w);
    step(&mut w);
    assert!(!offered(&w, t(&w, Kind::Infantry, "GRUNT")));
}

// spec: S-0407 claim 1 (c) (T2 run 4: no qualifying factory: not offered until a recheck finds one; then it stays)
#[test]
fn claim_1_qualifying_factory() {
    let mut w = game(&ruleset());
    let b = building(&mut w, "BARRACKS", 0, 0, 0);
    w.obj_mut(b).unwrap().econ.building.switched_off = true;
    step(&mut w);
    let grunt = t(&w, Kind::Infantry, "GRUNT");
    assert!(!offered(&w, grunt));
    switch(&mut w, b, true);
    step(&mut w);
    assert!(offered(&w, grunt));
    switch(&mut w, b, false);
    step(&mut w);
    assert!(offered(&w, grunt));
}

// spec: S-0407 claims 2 and 3 (T3: an override lifts tech level and prerequisites; refused takes precedence over at limit)
#[test]
fn claim_2_3_override() {
    let mut r = ruleset();
    add_building(&mut r, "KEY", plain(100));
    add_building(&mut r, "NEVER", plain(100));
    add_infantry(&mut r, "ELITE", |u| {
        u.tech_level = Some(50);
        u.econ.prerequisite = Some(vec!["NEVER".into()]);
        u.econ.prerequisite_override = Some(vec!["KEY".into()]);
        u.econ.build_limit = Some(1);
    });
    let mut w = game(&r);
    building(&mut w, "BARRACKS", 0, 0, 0);
    let elite = t(&w, Kind::Infantry, "ELITE");
    assert_eq!(tech::answer(&w, H, elite), Answer::Refused);
    let key = building(&mut w, "KEY", 0, 5, 5);
    assert_eq!(tech::answer(&w, H, elite), Answer::Allowed);
    step(&mut w);
    assert!(offered(&w, elite));
    testkit::spawn(
        &mut w,
        Kind::Infantry,
        "ELITE",
        0,
        testkit::mid(SIZE, -5, 5),
        0,
    );
    assert_eq!(tech::answer(&w, H, elite), Answer::AtLimit);
    objects::remove(&mut w, key);
    assert_eq!(tech::answer(&w, H, elite), Answer::Refused);
    step(&mut w);
    assert!(!offered(&w, elite));
}

// spec: S-0407 claim 4 (a secret lab's secret type skips claims 5 to 8 while the house owns the lab on the map)
#[test]
fn claim_4_secret_type() {
    let mut r = ruleset();
    let mut lab = plain(100);
    lab.econ.secret_lab = Some(true);
    add_building(&mut r, "LAB", lab);
    add_infantry(&mut r, "SECRET", |u| u.tech_level = Some(-1));
    let mut w = game(&r);
    building(&mut w, "BARRACKS", 0, 0, 0);
    let secret = t(&w, Kind::Infantry, "SECRET");
    let l = building(&mut w, "LAB", 0, 5, 5);
    assert_eq!(tech::answer(&w, H, secret), Answer::Refused);
    w.obj_mut(l).unwrap().econ.secret = Some(secret);
    assert_eq!(tech::answer(&w, H, secret), Answer::Allowed);
    objects::remove(&mut w, l);
    assert_eq!(tech::answer(&w, H, secret), Answer::Refused);
}

// spec: S-0407 claim 5 (a) (T4: tech level −1, at, above and absent)
#[test]
fn claim_5_tech_level() {
    let mut r = ruleset();
    for (n, lvl) in [
        ("L_NEG", Some(-1)),
        ("L_AT", Some(10)),
        ("L_ABOVE", Some(11)),
        ("L_NONE", None),
    ] {
        add_infantry(&mut r, n, |u| u.tech_level = lvl);
    }
    let w = game(&r);
    let a = |n| tech::answer(&w, H, t(&w, Kind::Infantry, n));
    assert_eq!(a("L_NEG"), Answer::Refused);
    assert_eq!(a("L_AT"), Answer::Allowed);
    assert_eq!(a("L_ABOVE"), Answer::Refused);
    assert_eq!(a("L_NONE"), Answer::Refused);
}

// spec: S-0407 claim 5 (b) and (c) (T5: required and forbidden lists, unknown names, empty lists)
#[test]
fn claim_5_country_lists() {
    let mut r = ruleset();
    let s = |v: &[&str]| Some(v.iter().map(|x| x.to_string()).collect::<Vec<_>>());
    add_infantry(&mut r, "REQ_OWN", |u| {
        u.econ.required_houses = s(&["Alphaland"])
    });
    add_infantry(&mut r, "REQ_OTHER", |u| {
        u.econ.required_houses = s(&["Betaland"])
    });
    add_infantry(&mut r, "REQ_UNKNOWN", |u| {
        u.econ.required_houses = s(&["Nowhere"])
    });
    add_infantry(&mut r, "FORB_OWN", |u| {
        u.econ.forbidden_houses = s(&["Alphaland"])
    });
    add_infantry(&mut r, "FORB_UNKNOWN", |u| {
        u.econ.forbidden_houses = s(&["Nowhere"])
    });
    add_infantry(&mut r, "REQ_EMPTY", |u| u.econ.required_houses = s(&[]));
    add_infantry(&mut r, "FORB_EMPTY", |u| u.econ.forbidden_houses = s(&[]));
    let w = game(&r);
    let a = |n| tech::answer(&w, H, t(&w, Kind::Infantry, n)) == Answer::Allowed;
    assert!(a("REQ_OWN"));
    assert!(!a("REQ_OTHER"));
    assert!(!a("REQ_UNKNOWN"));
    assert!(!a("FORB_OWN"));
    assert!(a("FORB_UNKNOWN"));
    assert!(a("REQ_EMPTY"));
    assert!(a("FORB_EMPTY"));
}

// spec: S-0407 F1 and quirks.country_index_wraps (T5 run 2: indices equal modulo 32)
#[test]
fn claim_5_index_wrap() {
    for wrap in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.country_index_wraps = wrap;
        for i in 0..31 {
            let n = format!("Filler{i}");
            r.registries.get_mut("countries").unwrap().push(n.clone());
            r.sim.countries.insert(n, Default::default());
        }
        // Far is registered at index 33, which equals Betaland's index 1 modulo 32.
        r.registries
            .get_mut("countries")
            .unwrap()
            .push("Far".into());
        r.sim.countries.insert("Far".into(), Default::default());
        let names = r.registries["countries"].clone();
        assert_eq!(names.iter().position(|n| n == "Far"), Some(33));
        assert_eq!(names.iter().position(|n| n == "Betaland"), Some(1));
        add_infantry(&mut r, "WRAPPED", |u| {
            u.econ.required_houses = Some(vec!["Far".into()])
        });
        let w = game(&r);
        // House 1 is Betaland, index 1 = 33 mod 32.
        let a = tech::answer(&w, HouseId(1), t(&w, Kind::Infantry, "WRAPPED"));
        assert_eq!(a == Answer::Allowed, wrap);
    }
}

// spec: S-0407 claim 6 (T6: the infiltrated building type's side gives the flag; radar, power output and the list exclude)
#[test]
fn claim_6_stolen_tech() {
    let mut r = ruleset();
    let lab = |side: Option<i32>| {
        let mut b = plain(100);
        b.econ.ai_base_planning_side = side;
        b
    };
    add_building(&mut r, "LAB0", lab(Some(0)));
    add_building(&mut r, "LAB1", lab(Some(1)));
    add_building(&mut r, "LABN", lab(None));
    add_building(&mut r, "LAB5", lab(Some(5)));
    let mut radar = lab(Some(1));
    radar.econ.radar = Some(true);
    add_building(&mut r, "LABR", radar);
    let mut power = lab(Some(1));
    power.econ.power = Some(50);
    add_building(&mut r, "LABP", power);
    add_building(&mut r, "NOTLAB", lab(Some(1)));
    r.sim.general.economy.build_tech = Some(
        ["LAB0", "LAB1", "LABN", "LAB5", "LABR", "LABP"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
    );
    for (n, i) in [("ST0", 0), ("ST1", 1), ("ST2", 2)] {
        add_infantry(&mut r, n, |u| match i {
            0 => u.econ.requires_stolen_allied_tech = Some(true),
            1 => u.econ.requires_stolen_soviet_tech = Some(true),
            _ => u.econ.requires_stolen_third_tech = Some(true),
        });
    }
    for (lab, want) in [
        ("LAB0", [true, false, false]),
        ("LAB1", [false, true, false]),
        ("LABN", [false, false, true]),
        ("LAB5", [false, false, true]),
        ("LABR", [false; 3]),
        ("LABP", [false; 3]),
        ("NOTLAB", [false; 3]),
    ] {
        let mut w = game(&r);
        let b = building(&mut w, lab, 1, 5, 5);
        assert_eq!(
            tech::answer(&w, H, t(&w, Kind::Infantry, "ST0")),
            Answer::Refused
        );
        tech::infiltrated(&mut w, H, b);
        let got = ["ST0", "ST1", "ST2"]
            .map(|n| tech::answer(&w, H, t(&w, Kind::Infantry, n)) == Answer::Allowed);
        assert_eq!(got, want, "{lab}");
        assert!(w.houses[0].econ.tech.recheck_due);
    }
}

// spec: S-0407 claim 8 (T8 (a), (b), (d), (h): named buildings, groups, don't-score buildings, dropped names)
#[test]
fn claim_8_prerequisites() {
    let mut r = ruleset();
    add_building(&mut r, "GEN", plain(100));
    add_building(&mut r, "NAMED", plain(100));
    let mut ds = plain(100);
    ds.combat.dont_score = Some(true);
    add_building(&mut r, "GHOST", ds);
    r.sim.general.economy.prerequisite_power = Some(vec!["GEN".into()]);
    add_infantry(&mut r, "NEEDS", |u| {
        u.econ.prerequisite = Some(vec!["power".into(), "NAMED".into()])
    });
    add_infantry(&mut r, "NEEDS_GHOST", |u| {
        u.econ.prerequisite = Some(vec!["GHOST".into()])
    });
    add_infantry(&mut r, "DROPPED", |u| {
        u.econ.prerequisite = Some(vec!["TANK".into(), "NOSUCH".into()])
    });
    let mut w = game(&r);
    let a = |w: &World, n| tech::answer(w, H, t(w, Kind::Infantry, n)) == Answer::Allowed;
    assert!(!a(&w, "NEEDS"));
    assert!(a(&w, "DROPPED"));
    building(&mut w, "GEN", 0, 0, 0);
    assert!(!a(&w, "NEEDS"));
    let named = building(&mut w, "NAMED", 0, 4, 4);
    assert!(a(&w, "NEEDS"));
    // Switched off or damaged: still counts (T8 (i)).
    w.obj_mut(named).unwrap().econ.building.switched_off = true;
    w.obj_mut(named).unwrap().health = 1;
    assert!(a(&w, "NEEDS"));
    building(&mut w, "GHOST", 0, -4, 4);
    assert!(!a(&w, "NEEDS_GHOST"));
}

// spec: S-0407 claim 8 (d) (quirks.prerequisite_names_dropped off: an unrecognised name remains an unmet entry; valid entries still count)
#[test]
fn claim_8d_dropped_names_switch_off() {
    for dropped in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.prerequisite_names_dropped = dropped;
        add_infantry(&mut r, "DROPPED", |u| {
            u.econ.prerequisite = Some(vec!["TANK".into(), "NOSUCH".into()])
        });
        add_infantry(&mut r, "VALID", |u| u.econ.prerequisite = Some(vec![]));
        let w = game(&r);
        let a = |n| tech::answer(&w, H, t(&w, Kind::Infantry, n)) == Answer::Allowed;
        assert!(a("VALID"));
        assert_eq!(a("DROPPED"), dropped);
    }
}

// spec: S-0407 claim 7 (superweapons off: a disableable first superweapon refuses; quirks.superweapon_setting_tests_first_only off: the second field refuses too; the tech-lab list exempts)
#[test]
fn claim_7_superweapon_fields() {
    for first_only in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.superweapon_setting_tests_first_only = first_only;
        r.sim.general.economy.super_weapons_allowed = Some(false);
        r.sim.superweapons.insert(
            "BLAST".into(),
            glasnost_rules::sim_econ::SuperweaponType {
                disableable_from_shell: Some(true),
                ..Default::default()
            },
        );
        add_building(&mut r, "PLAIN", plain(100));
        let mut one = plain(100);
        one.econ.super_weapon = Some("BLAST".into());
        add_building(&mut r, "SW1", one);
        let mut two = plain(100);
        two.econ.super_weapon2 = Some("BLAST".into());
        add_building(&mut r, "SW2", two.clone());
        add_building(&mut r, "LAB2", two);
        r.sim.general.economy.build_tech = Some(vec!["LAB2".into()]);
        let w = game(&r);
        let ans = |n| tech::answer(&w, H, t(&w, Kind::Building, n));
        let plain_answer = ans("PLAIN");
        assert_ne!(ans("SW1"), plain_answer);
        assert_eq!(ans("SW2") == plain_answer, first_only);
        assert_eq!(ans("LAB2"), plain_answer);
    }
}

// spec: S-0407 claim 8 (b) (the refinery group is also met by the refinery-alternative vehicle)
#[test]
fn claim_8_refinery_alternative() {
    let mut r = ruleset();
    r.sim.general.economy.prerequisite_proc_alternate = Some("JEEP".into());
    add_infantry(&mut r, "MINER", |u| {
        u.econ.prerequisite = Some(vec!["PROC".into()])
    });
    let mut w = game(&r);
    let miner = t(&w, Kind::Infantry, "MINER");
    assert_eq!(tech::answer(&w, H, miner), Answer::Refused);
    vehicle(&mut w, "JEEP", 0, 3, 3);
    assert_eq!(tech::answer(&w, H, miner), Answer::Allowed);
}

// spec: S-0407 claim 8 (c) and quirks.upgrade_prerequisite_last_building (T8 (e): only the most recently acquired eligible building decides)
#[test]
fn claim_8_upgrade() {
    for last in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.upgrade_prerequisite_last_building = last;
        let mut up = plain(100);
        up.econ.powers_up_building = Some("HUT".into());
        add_building(&mut r, "UPG", up);
        add_infantry(&mut r, "UPX", |u| {
            u.econ.prerequisite = Some(vec!["UPG".into()])
        });
        let mut w = game(&r);
        let upg = t(&w, Kind::Building, "UPG").index;
        let e1 = building(&mut w, "HUT", 0, 0, 0);
        building(&mut w, "HUT", 0, 4, 4);
        w.obj_mut(e1).unwrap().econ.building.upgrades[0] = Some(upg);
        let a = tech::answer(&w, H, t(&w, Kind::Infantry, "UPX"));
        assert_eq!(a == Answer::Allowed, !last);
    }
}

// spec: S-0407 claim 9 (T10 (a), (e), (f): limits, the production exception, the infantry exactness)
#[test]
fn claim_9_build_limit() {
    let mut r = ruleset();
    r.sim.vehicles.get_mut("TANK").unwrap().econ.build_limit = Some(1);
    r.sim.infantry.get_mut("GRUNT").unwrap().econ.build_limit = Some(1);
    r.sim.vehicles.get_mut("JEEP").unwrap().econ.build_limit = Some(0);
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("MINV".into());
    let mut minv = wheel(100);
    minv.econ.build_limit = Some(i32::MIN);
    r.sim.vehicles.insert("MINV".into(), minv);
    let mut w = game(&r);
    building(&mut w, "WARFACT", 0, 0, 0);
    building(&mut w, "BARRACKS", 0, -6, 0);
    ready(&mut w);
    set_credits(&mut w, 0, 100_000);
    let tank = t(&w, Kind::Vehicle, "TANK");
    let grunt = t(&w, Kind::Infantry, "GRUNT");
    assert_eq!(
        tech::answer(&w, H, t(&w, Kind::Vehicle, "JEEP")),
        Answer::Refused
    );
    assert_eq!(
        tech::answer(&w, H, t(&w, Kind::Vehicle, "MINV")),
        Answer::Refused
    );
    assert!(issue(&mut w, 0, Order::Build { what: tank }));
    assert!(issue(&mut w, 0, Order::Build { what: grunt }));
    step(&mut w);
    // The item's object is owned: O = L, with the item in production: allowed.
    assert_eq!(tech::owned_count(&w, H, tank), 1);
    assert_eq!(tech::answer(&w, H, tank), Answer::Allowed);
    assert_eq!(tech::answer(&w, H, grunt), Answer::Allowed);
    // Two more of each given: O = 3 > L; vehicles still allowed, infantry at limit.
    for dx in [4, 6] {
        vehicle(&mut w, "TANK", 0, dx, 6);
        testkit::spawn(
            &mut w,
            Kind::Infantry,
            "GRUNT",
            0,
            testkit::mid(SIZE, dx, -6),
            0,
        );
    }
    assert_eq!(tech::answer(&w, H, tank), Answer::Allowed);
    assert_eq!(tech::answer(&w, H, grunt), Answer::AtLimit);
}

// spec: S-0407 F2 and quirks.infantry_limit_exception_exact off (infantry follows the other kinds)
#[test]
fn claim_9_infantry_quirk_off() {
    let mut r = ruleset();
    r.globals.quirks.infantry_limit_exception_exact = false;
    r.sim.infantry.get_mut("GRUNT").unwrap().econ.build_limit = Some(1);
    let mut w = game(&r);
    building(&mut w, "BARRACKS", 0, -6, 0);
    ready(&mut w);
    set_credits(&mut w, 0, 100_000);
    let grunt = t(&w, Kind::Infantry, "GRUNT");
    assert!(issue(&mut w, 0, Order::Build { what: grunt }));
    step(&mut w);
    testkit::spawn(
        &mut w,
        Kind::Infantry,
        "GRUNT",
        0,
        testkit::mid(SIZE, 4, -6),
        0,
    );
    assert_eq!(tech::answer(&w, H, grunt), Answer::Allowed);
}

// spec: S-0407 claims 9 and 10 (T10 (b), (g): a negative limit counts releases from a factory only)
#[test]
fn claim_10_ever_built() {
    let mut r = ruleset();
    r.sim.vehicles.get_mut("JEEP").unwrap().econ.build_limit = Some(-1);
    r.sim.vehicles.get_mut("JEEP").unwrap().cost = Some(50);
    let mut w = game(&r);
    building(&mut w, "WARFACT", 0, 0, 0);
    ready(&mut w);
    set_credits(&mut w, 0, 100_000);
    let jeep = t(&w, Kind::Vehicle, "JEEP");
    // A given vehicle does not count.
    let given = vehicle(&mut w, "JEEP", 0, 6, 6);
    assert_eq!(tech::ever_built(&w, H, jeep), 0);
    assert_eq!(tech::answer(&w, H, jeep), Answer::Allowed);
    objects::remove(&mut w, given);
    assert!(issue(&mut w, 0, Order::Build { what: jeep }));
    let done = run_to_done(&mut w, 0, jeep, 200).expect("done");
    while w.tick <= done + 1 {
        step(&mut w);
    }
    assert_eq!(tech::ever_built(&w, H, jeep), 1);
    assert_eq!(tech::answer(&w, H, jeep), Answer::Refused);
}

// spec: S-0407 claims 11 and 15 (T2: a captured factory of another country group gives its types; it counts for the new owner only)
#[test]
fn claim_11_15_captured_factory() {
    let mut r = ruleset();
    let mut bb = factory("infantry");
    bb.owner = Some(vec!["Betaland".into()]);
    add_building(&mut r, "BARR_B", bb);
    add_infantry(&mut r, "GRUNTB", |u| {
        u.owner = Some(vec!["Betaland".into()])
    });
    let mut ba = factory("infantry");
    ba.owner = Some(vec!["Alphaland".into()]);
    add_building(&mut r, "BARR_A", ba);
    let mut w = game(&r);
    building(&mut w, "BARR_A", 0, -6, 0);
    let b = building(&mut w, "BARR_B", 1, 4, 4);
    step(&mut w);
    let gb = t(&w, Kind::Infantry, "GRUNTB");
    assert!(!offered(&w, gb));
    assert!(tech::offered(&w, HouseId(1), gb));
    // The capture: a change of owner (the capture itself is the special-abilities specs').
    w.obj_mut(b).unwrap().owner = Some(H);
    glasnost_sim::econ::events::owner_changed(&mut w, b, Some(HouseId(1)));
    step(&mut w);
    assert!(offered(&w, gb));
    assert!(!tech::offered(&w, HouseId(1), gb));
}

// spec: S-0407 claim 12 and quirks.unit_changes_no_recheck (T14 run 6: a unit's entry is no event; a later event shows the whole set)
#[test]
fn claim_12_non_event() {
    for quirk in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.unit_changes_no_recheck = quirk;
        r.sim.general.economy.prerequisite_proc_alternate = Some("JEEP".into());
        add_infantry(&mut r, "MINER", |u| {
            u.econ.prerequisite = Some(vec!["PROC".into()])
        });
        let mut w = game(&r);
        building(&mut w, "BARRACKS", 0, -6, 0);
        step(&mut w);
        let miner = t(&w, Kind::Infantry, "MINER");
        assert!(!offered(&w, miner));
        vehicle(&mut w, "JEEP", 0, 4, 4);
        step(&mut w);
        assert_eq!(offered(&w, miner), !quirk);
        // A later event of the table: a building put on the map.
        building(&mut w, "HUT", 0, -4, 6);
        step(&mut w);
        assert!(offered(&w, miner));
    }
}

// spec: S-0407 claim 12 (f) (T14 run 5: a unit released by a factory calls for a recheck)
#[test]
fn claim_12_release_event() {
    let mut r = ruleset();
    r.sim.general.economy.prerequisite_proc_alternate = Some("JEEP".into());
    r.sim.vehicles.get_mut("JEEP").unwrap().cost = Some(50);
    add_infantry(&mut r, "MINER", |u| {
        u.econ.prerequisite = Some(vec!["PROC".into()])
    });
    let mut w = game(&r);
    building(&mut w, "BARRACKS", 0, -6, 0);
    building(&mut w, "WARFACT", 0, 2, 0);
    ready(&mut w);
    set_credits(&mut w, 0, 100_000);
    let jeep = t(&w, Kind::Vehicle, "JEEP");
    let miner = t(&w, Kind::Infantry, "MINER");
    assert!(issue(&mut w, 0, Order::Build { what: jeep }));
    let done = run_to_done(&mut w, 0, jeep, 200).expect("done");
    // Released at the end of tick done + 1; the recheck in the house update of tick done + 2.
    while w.tick <= done + 1 {
        step(&mut w);
    }
    assert!(!offered(&w, miner));
    step(&mut w);
    assert!(offered(&w, miner));
}

// spec: S-0407 claim 13 and quirks.stale_offer_orderable (T11: at limit not orderable; a stale offer orderable; off: refused)
#[test]
fn claim_13_orders_follow_offer() {
    for stale in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.stale_offer_orderable = stale;
        r.sim.general.economy.prerequisite_proc_alternate = Some("JEEP".into());
        add_infantry(&mut r, "MINER", |u| {
            u.econ.prerequisite = Some(vec!["PROC".into()])
        });
        r.sim.vehicles.get_mut("TANK").unwrap().econ.build_limit = Some(1);
        let mut w = game(&r);
        building(&mut w, "BARRACKS", 0, -6, 0);
        building(&mut w, "WARFACT", 0, 2, 0);
        let v = vehicle(&mut w, "JEEP", 0, 6, 6);
        vehicle(&mut w, "TANK", 0, 8, 6);
        ready(&mut w);
        set_credits(&mut w, 0, 100_000);
        let miner = t(&w, Kind::Infantry, "MINER");
        let tank = t(&w, Kind::Vehicle, "TANK");
        assert!(offered(&w, miner));
        assert!(offered(&w, tank));
        // T11 run 1: at limit, not legal.
        assert!(!issue(&mut w, 0, Order::Build { what: tank }));
        // The vehicle's end is no event: the offer is stale.
        objects::remove(&mut w, v);
        step(&mut w);
        assert!(offered(&w, miner));
        assert_eq!(issue(&mut w, 0, Order::Build { what: miner }), stale);
        // The stale order is produced: applying it tests only the factory
        // conditions and the build limit (claim 13; S-0404 claim 3).
        step(&mut w);
        assert_eq!(
            glasnost_sim::econ::production::line_of(&w, H, miner).is_some(),
            stale
        );
    }
}

fn labs_rules(labs: usize, entries: usize, fixed_first: bool) -> Ruleset {
    let mut r = ruleset();
    let mut lab = plain(100);
    lab.econ.secret_lab = Some(true);
    add_building(&mut r, "LAB", lab.clone());
    let mut fixed = lab;
    fixed.econ.secret_infantry = Some("GRUNT".into());
    fixed.econ.secret_building = Some("HUT".into());
    add_building(&mut r, "LABF", fixed);
    let list: Vec<String> = ["MEDIC", "GRUNT", "MEDIC"][..entries]
        .iter()
        .map(|s| s.to_string())
        .collect();
    r.sim.general.economy.secret_infantry = Some(list);
    let _ = (labs, fixed_first);
    r
}

// spec: S-0407 claim 14 and F4 (T9: one ranged draw per lab over 0 … N − 1 − k; the last over a single value takes no raw value)
#[test]
fn claim_14_secret_labs() {
    let r = labs_rules(3, 3, false);
    let mut w = game(&r);
    let labs: Vec<_> = (0..3)
        .map(|i| building(&mut w, "LAB", 1, -6 + 4 * i, 6))
        .collect();
    let before = w.rng.taken();
    tech::assign_secret_labs(&mut w);
    let draws = w.rng.taken() - before;
    // Two ranged draws over 0 … 2 and 0 … 1 (each takes at least one raw value), none for 0 … 0.
    assert!(draws >= 2);
    for l in &labs {
        assert!(w.obj(*l).unwrap().econ.secret.is_some());
    }
    // Run 2: a lab with fixed types gives the infantry type and still draws.
    let mut w2 = game(&r);
    let f = building(&mut w2, "LABF", 1, -6, 6);
    building(&mut w2, "LAB", 1, -2, 6);
    building(&mut w2, "LAB", 1, 2, 6);
    let b2 = w2.rng.taken();
    tech::assign_secret_labs(&mut w2);
    assert_eq!(w2.rng.taken() - b2, draws);
    assert_eq!(
        w2.obj(f).unwrap().econ.secret,
        Some(t(&w2, Kind::Infantry, "GRUNT"))
    );
}

// spec: S-0407 claim 14 (T9 runs 4 and 5: more labs than entries, or none: no draw, no type from the lists; the fixed type kept)
#[test]
fn claim_14_no_draws() {
    let r = labs_rules(4, 3, false);
    let mut w = game(&r);
    let f = building(&mut w, "LABF", 1, -8, 6);
    let others: Vec<_> = (0..3)
        .map(|i| building(&mut w, "LAB", 1, -4 + 4 * i, 6))
        .collect();
    let before = w.rng.taken();
    tech::assign_secret_labs(&mut w);
    assert_eq!(w.rng.taken(), before);
    assert_eq!(
        w.obj(f).unwrap().econ.secret,
        Some(t(&w, Kind::Infantry, "GRUNT"))
    );
    for l in others {
        assert_eq!(w.obj(l).unwrap().econ.secret, None);
    }
}

// spec: S-0407 claim 16 (T12 runs 1 and 2: the queue-time limit counts the item twice)
#[test]
fn claim_16_queue_limit() {
    for (limit, accepted) in [(2, 0), (3, 1)] {
        let mut r = ruleset();
        r.sim.vehicles.get_mut("TANK").unwrap().econ.build_limit = Some(limit);
        let mut w = game(&r);
        building(&mut w, "WARFACT", 0, 0, 0);
        ready(&mut w);
        set_credits(&mut w, 0, 100_000);
        let tank = t(&w, Kind::Vehicle, "TANK");
        assert!(issue(&mut w, 0, Order::Build { what: tank }));
        step(&mut w);
        issue(&mut w, 0, Order::Build { what: tank });
        step(&mut w);
        issue(&mut w, 0, Order::Build { what: tank });
        step(&mut w);
        assert_eq!(
            line(&w, 0, tank).unwrap().queue.len(),
            accepted,
            "L = {limit}"
        );
    }
}

// spec: S-0407 claim 16 and quirks.dock_bound_aircraft_ignores_limit (T12 run 3: a dock-bound aircraft judged by the docks)
#[test]
fn claim_16_dock_bound() {
    for (docks, accepted) in [(2, 0), (3, 1)] {
        let mut r = ruleset();
        r.sim.buildings.get_mut("AIRPAD").unwrap().helipad = Some(true);
        r.sim
            .buildings
            .get_mut("AIRPAD")
            .unwrap()
            .econ
            .number_of_docks = Some(docks);
        for (n, bound, limit) in [("PADA", true, Some(1)), ("PADB", false, None)] {
            r.registries.get_mut("aircraft").unwrap().push(n.into());
            let mut a = r.sim.aircraft["PLANE"].clone();
            a.econ.airport_bound = Some(bound);
            a.econ.build_limit = limit;
            r.sim.aircraft.insert(n.into(), a);
        }
        r.sim.general.economy.pad_aircraft = Some(vec!["PADA".into(), "PADB".into()]);
        let mut w = game(&r);
        building(&mut w, "AIRPAD", 0, 0, 0);
        testkit::spawn(
            &mut w,
            Kind::Aircraft,
            "PADB",
            0,
            testkit::mid(SIZE, 6, 6),
            0,
        );
        ready(&mut w);
        set_credits(&mut w, 0, 100_000);
        let a = t(&w, Kind::Aircraft, "PADA");
        assert!(issue(&mut w, 0, Order::Build { what: a }));
        step(&mut w);
        issue(&mut w, 0, Order::Build { what: a });
        step(&mut w);
        assert_eq!(line(&w, 0, a).unwrap().queue.len(), accepted, "T = {docks}");
    }
}

// spec: S-0407 claim 16 and S-0404 claim 10 (T12 run 6: a dock-bound copy queued past its limit is accepted but does not start once the first is released; control L = 2 starts it)
#[test]
fn claim_16_queued_copy_needs_the_allowance() {
    for (limit, starts) in [(1, false), (2, true)] {
        let mut r = ruleset();
        r.sim.buildings.get_mut("AIRPAD").unwrap().helipad = Some(true);
        r.sim
            .buildings
            .get_mut("AIRPAD")
            .unwrap()
            .econ
            .number_of_docks = Some(3);
        r.registries
            .get_mut("aircraft")
            .unwrap()
            .push("PADA".into());
        let mut a = r.sim.aircraft["PLANE"].clone();
        a.econ.airport_bound = Some(true);
        a.econ.build_limit = Some(limit);
        a.cost = Some(54);
        r.sim.aircraft.insert("PADA".into(), a);
        r.sim.general.economy.pad_aircraft = Some(vec!["PADA".into(), "PLANE".into()]);
        let mut w = game(&r);
        building(&mut w, "AIRPAD", 0, 0, 0);
        ready(&mut w);
        set_credits(&mut w, 0, 100_000);
        let a = t(&w, Kind::Aircraft, "PADA");
        assert!(issue(&mut w, 0, Order::Build { what: a }));
        step(&mut w);
        assert!(issue(&mut w, 0, Order::Build { what: a }));
        step(&mut w);
        assert_eq!(line(&w, 0, a).unwrap().queue, vec![a], "L = {limit}");
        run_to_done(&mut w, 0, a, 20_000).expect("done");
        for _ in 0..3 {
            step(&mut w);
        }
        let owned = w
            .objects
            .values()
            .filter(|o| o.on_map && o.kind == Kind::Aircraft)
            .count();
        assert_eq!(owned, 1, "L = {limit}");
        assert_eq!(item(&w, 0, a).is_some(), starts, "L = {limit}");
    }
}
