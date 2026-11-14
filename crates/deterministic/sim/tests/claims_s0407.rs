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

// spec: S-0407 claims 1 (a), 3, 4 and 8 (a): the recheck's answers match
// the tech-tree answer of each type, with prerequisite, override and lab
// buildings of another house, off the map, and of the house on the map
// (#395: the recheck reads its counts from one walk of the objects).
#[test]
fn claim_1_recheck_answers_match_the_answer() {
    let mut r = ruleset();
    add_building(&mut r, "HUT2", plain(100));
    add_building(&mut r, "KEY", plain(100));
    let mut lab = plain(100);
    lab.econ.secret_lab = Some(true);
    add_building(&mut r, "LAB", lab);
    add_infantry(&mut r, "PRE", |u| {
        u.econ.prerequisite = Some(vec!["HUT2".into()]);
    });
    add_infantry(&mut r, "LIFT", |u| {
        u.tech_level = Some(-1);
        u.econ.prerequisite_override = Some(vec!["KEY".into()]);
    });
    add_infantry(&mut r, "SEC", |u| u.tech_level = Some(-1));
    let mut w = game(&r);
    building(&mut w, "BARRACKS", 0, 0, 0);
    let names = ["PRE", "LIFT", "SEC"];
    let check = |w: &mut World, want: [bool; 3]| {
        tech::recheck(w, H);
        for (n, want) in names.into_iter().zip(want) {
            let ty = t(w, Kind::Infantry, n);
            assert_eq!(tech::answer(w, H, ty) != Answer::Refused, want, "{n}");
            assert_eq!(offered(w, ty), want, "{n}");
        }
    };
    check(&mut w, [false; 3]);
    // Another house's buildings on the map count for nobody else.
    building(&mut w, "HUT2", 1, 6, 0);
    building(&mut w, "KEY", 1, 6, 4);
    let other_lab = building(&mut w, "LAB", 1, 6, 8);
    let sec = t(&w, Kind::Infantry, "SEC");
    w.obj_mut(other_lab).unwrap().econ.secret = Some(sec);
    check(&mut w, [false; 3]);
    // The house's own buildings off the map are not present.
    let rr = w.rules_arc();
    for n in ["HUT2", "KEY", "LAB"] {
        let ti = rr.index_of(Kind::Building, n).unwrap();
        let id = objects::create(&mut w, Kind::Building, ti, Some(H));
        w.obj_mut(id).unwrap().econ.secret = Some(sec);
    }
    check(&mut w, [false; 3]);
    // On the map they are.
    building(&mut w, "HUT2", 0, -6, 0);
    check(&mut w, [true, false, false]);
    building(&mut w, "KEY", 0, -6, 4);
    check(&mut w, [true, true, false]);
    let own_lab = building(&mut w, "LAB", 0, -6, 8);
    check(&mut w, [true, true, false]);
    w.obj_mut(own_lab).unwrap().econ.secret = Some(sec);
    check(&mut w, [true, true, true]);
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

// spec: S-0407 claim 16 (T12 runs 1 and 2: the queue-time limit counts the item twice; *At issue*: the refused copy is refused when issued, so no order is given)
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
        assert_eq!(
            issue(&mut w, 0, Order::Build { what: tank }),
            accepted == 1,
            "L = {limit}: the second copy at issue"
        );
        step(&mut w);
        assert!(
            !issue(&mut w, 0, Order::Build { what: tank }),
            "L = {limit}: the last copy is refused at issue"
        );
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

/// A ruleset for T12 run 8: one-place test helipads (`AIRPAD`, one docking
/// place each) and a listed dock-bound armed type `PADB` with no binding
/// build limit; with `a_limit`, a second listed dock-bound type `PADA` with
/// that build limit.
fn run_8_rules(a_limit: Option<i32>) -> Ruleset {
    let mut r = ruleset();
    let pad = r.sim.buildings.get_mut("AIRPAD").unwrap();
    pad.helipad = Some(true);
    pad.econ.number_of_docks = Some(1);
    let mut list = Vec::new();
    let mut types = vec![("PADB", None)];
    if let Some(l) = a_limit {
        types.push(("PADA", Some(l)));
    }
    for (n, limit) in types {
        r.registries.get_mut("aircraft").unwrap().push(n.into());
        let mut a = r.sim.aircraft["PLANE"].clone();
        a.econ.airport_bound = Some(true);
        a.econ.build_limit = limit;
        r.sim.aircraft.insert(n.into(), a);
        list.push(n.into());
    }
    r.sim.general.economy.pad_aircraft = Some(list);
    r
}

/// T12 run 8's game: `pads` one-place helipads, `flying` PADB in flight
/// (no contact with a pad) and `docked` PADA standing on a pad (in contact).
fn run_8_game(r: &Ruleset, pads: i32, flying: usize, docked: usize) -> World {
    let mut w = game(r);
    let ks: Vec<_> = (0..pads)
        .map(|i| building(&mut w, "AIRPAD", 0, -8 + 4 * i, 0))
        .collect();
    for i in 0..flying {
        testkit::spawn(
            &mut w,
            Kind::Aircraft,
            "PADB",
            0,
            testkit::mid(SIZE, 6, 4 + 2 * i as i32),
            0,
        );
    }
    let mut on_pad = Vec::new();
    for i in 0..docked {
        on_pad.push(testkit::spawn(
            &mut w,
            Kind::Aircraft,
            "PADA",
            0,
            testkit::mid(SIZE, -6, 6 + 2 * i as i32),
            0,
        ));
    }
    ready(&mut w);
    set_credits(&mut w, 0, 100_000);
    for (i, a) in on_pad.into_iter().enumerate() {
        w.obj_mut(a).unwrap().contact = Some(ks[i]);
    }
    assert_eq!(pad_total(&w, 0), i64::from(pads), "T");
    w
}

/// A left click on the icon of `ty` as the local player, with the order it
/// gives issued (S-0411 claim 4): whether an order was given.
fn click_icon(w: &mut World, ty: TypeRef) -> bool {
    use glasnost_sim::econ::sidebar::{self, Click, ClickOutcome};
    match sidebar::click(w, H, ty, Click::Left, false) {
        ClickOutcome::Order(o) => {
            assert!(
                issue(w, 0, o.clone()),
                "an order the click gives is legal: {o:?}"
            );
            true
        }
        _ => false,
    }
}

/// Whether the tick's effects hold S-0404 claim 3's refusal sound for a
/// production request (S-0701 claim 24).
fn refusal_sound(w: &World) -> bool {
    w.effects.iter().any(|e| {
        matches!(
            e,
            glasnost_sim::combat::effects::Effect::Sound {
                sound: glasnost_sim::combat::effects::Sound::QueueRefused,
                ..
            }
        )
    })
}

fn helipads(w: &World) -> Vec<glasnost_sim::world::ObjectId> {
    w.objects
        .values()
        .filter(|o| o.kind == Kind::Building)
        .map(|o| o.id)
        .collect()
}

// spec: S-0407 claim 16 *At issue* and F3 (T12 run 8 part a, Q-2172: an aircraft in flight is on the map and counts in D like one on its pad; with both listed aircraft out and every pad empty, D = 2 = T, so the click on the icon gives no order, plays no sound and starts nothing) and claim 13 (every build order is refused when issued while claim 16's test refuses it). The lead's play note 35 (M8 A4 round 6): pad aircraft could be bought while the others flew.
#[test]
fn claim_16_pad_capacity_is_tested_at_issue_with_the_aircraft_in_flight() {
    let r = run_8_rules(None);
    let mut w = run_8_game(&r, 2, 2, 0);
    let b = t(&w, Kind::Aircraft, "PADB");
    assert!(!click_icon(&mut w, b), "no order: D = 2 = T");
    assert!(
        !issue(&mut w, 0, Order::Build { what: b }),
        "the build order itself is refused when issued (claim 13), for every house"
    );
    step(&mut w);
    assert!(line(&w, 0, b).is_none(), "nothing starts");
    assert!(!refusal_sound(&w), "no sound");
}

// spec: S-0407 claim 16 *At issue* and F3 (T12 run 8 part b, the control: with T = 3 and both listed aircraft in flight, D = 2 < 3, the first click gives an order and the item starts; a second click before any returns gives none, D = 2 + 1 copy = 3; no sound)
#[test]
fn claim_16_at_issue_counts_the_line_copy() {
    let r = run_8_rules(None);
    let mut w = run_8_game(&r, 3, 2, 0);
    let b = t(&w, Kind::Aircraft, "PADB");
    assert!(click_icon(&mut w, b), "the first click gives an order");
    step(&mut w);
    assert_eq!(item(&w, 0, b).map(|i| i.t), Some(b), "the item starts");
    assert!(!click_icon(&mut w, b), "the second click gives no order");
    step(&mut w);
    assert_eq!(line(&w, 0, b).unwrap().queue.len(), 0);
    assert!(!refusal_sound(&w), "a click refused at issue is silent");
}

// spec: S-0407 claim 13 (our engine gives no order for a dock-bound aircraft type at its own build limit, which the original's interface gives and S-0404 claim 3 then refuses: a design rule with no switch) and claim 16 *At issue* (T12 run 8 part c: T = 4, D = 3 < 4; control: the same type below its own limit is given its order and starts)
#[test]
fn claim_13_no_order_for_a_dock_bound_type_at_its_own_limit() {
    for limit in [1, 2] {
        let r = run_8_rules(Some(limit));
        let mut w = run_8_game(&r, 4, 2, 1);
        let a = t(&w, Kind::Aircraft, "PADA");
        let at_limit = limit == 1;
        assert_eq!(tech::at_limit(&w, H, a), at_limit, "L = {limit}");
        assert_eq!(
            click_icon(&mut w, a),
            !at_limit,
            "L = {limit}: an order only below its own limit (D = 3 < T = 4 either way)"
        );
        step(&mut w);
        assert_eq!(item(&w, 0, a).is_some(), !at_limit, "L = {limit}");
    }
}

// spec: S-0407 claim 16 *At issue* (for an idle line the line holds no copies, so D is the house's present count of the listed types: refused when that count is T or more, a pad aircraft in contact with its pad and one in flight alike) and S-0404 claim 3 (an order that passed the test at issue starts the item on the idle line)
#[test]
fn claim_16_idle_line_counts_present_aircraft_in_flight_or_docked() {
    for docked in [false, true] {
        let r = run_8_rules(None);
        let mut w = run_8_game(&r, 1, 1, 0);
        let k = helipads(&w)[0];
        let flyer = w
            .objects
            .values()
            .find(|o| o.kind == Kind::Aircraft)
            .unwrap()
            .id;
        if docked {
            w.obj_mut(flyer).unwrap().contact = Some(k);
        }
        let b = t(&w, Kind::Aircraft, "PADB");
        assert!(!click_icon(&mut w, b), "docked {docked}: D = 1 = T");
        step(&mut w);
        assert!(line(&w, 0, b).is_none(), "docked {docked}");
        // Control: with the aircraft gone, D = 0 < T and the order starts the item.
        objects::remove(&mut w, flyer);
        assert!(click_icon(&mut w, b), "docked {docked}: D = 0");
        step(&mut w);
        assert_eq!(item(&w, 0, b).map(|i| i.t), Some(b), "docked {docked}");
    }
}

// spec: S-0404 claim 3 and S-0701 claim 24 (the refusal sound for claim 16's test follows only an order that passed the test at issue and fails it when applied, after a count changed in between; here the pad total falls between issue and application) and S-0407 claim 16 *At issue*
#[test]
fn claim_16_refusal_sound_only_after_a_changed_count() {
    let r = run_8_rules(None);
    let mut w = run_8_game(&r, 2, 0, 0);
    let b = t(&w, Kind::Aircraft, "PADB");
    assert!(click_icon(&mut w, b));
    step(&mut w);
    assert!(item(&w, 0, b).is_some());
    // The queued copy passes at issue (D = 1 < 2); a pad is removed before
    // the order is applied, so T = 1 and the copy fails when applied.
    assert!(click_icon(&mut w, b), "D = 1 < T = 2 at issue");
    let pad = helipads(&w)[0];
    objects::remove(&mut w, pad);
    assert_eq!(pad_total(&w, 0), 1);
    step(&mut w);
    assert_eq!(
        line(&w, 0, b).unwrap().queue.len(),
        0,
        "refused when applied"
    );
    assert!(refusal_sound(&w), "with the refusal sound");
}

// spec: S-0407 claim 16 *At issue* (T12 run 4, r29: with an explicit docking-place count of 0, T = 0, so the start itself gives no order at issue, D = 0 = T, and nothing starts; control: a count of 1 gives T = 1 and the start is given)
#[test]
fn claim_16_explicit_zero_pad_total_gives_no_start_at_issue() {
    for docks in [0, 1] {
        let mut r = run_8_rules(None);
        r.sim
            .buildings
            .get_mut("AIRPAD")
            .unwrap()
            .econ
            .number_of_docks = Some(docks);
        let mut w = game(&r);
        building(&mut w, "AIRPAD", 0, -8, 0);
        ready(&mut w);
        set_credits(&mut w, 0, 100_000);
        assert_eq!(pad_total(&w, 0), i64::from(docks), "T");
        let b = t(&w, Kind::Aircraft, "PADB");
        assert_eq!(click_icon(&mut w, b), docks == 1, "T = {docks}");
        step(&mut w);
        assert_eq!(item(&w, 0, b).is_some(), docks == 1, "T = {docks}");
        assert!(!refusal_sound(&w), "T = {docks}: no sound");
    }
}

fn pad_total(w: &World, h: u8) -> i64 {
    w.houses[usize::from(h)].econ.tech.pad_total
}

// spec: S-0407 claim 16 and S-0501 claim 30 (the pad total: only helipad buildings count; an absent docking-place count adds 1, an explicit 0 adds 0; a saved running total grown on placement, moved on an owner change, shrunk on removal and never below 0)
#[test]
fn claim_16_pad_total_running() {
    let mut r = ruleset();
    let pad = |docks: Option<i32>, helipad: bool| {
        let mut b = plain(100);
        b.helipad = Some(helipad);
        b.econ.number_of_docks = docks;
        b
    };
    add_building(&mut r, "PADX", pad(None, true));
    add_building(&mut r, "PADZ", pad(Some(0), true));
    add_building(&mut r, "PADT", pad(Some(3), true));
    add_building(&mut r, "NOTPAD", pad(Some(4), false));
    let mut w = game(&r);
    assert_eq!(pad_total(&w, 0), 0);
    let x = building(&mut w, "PADX", 0, 0, 0);
    assert_eq!(pad_total(&w, 0), 1, "an absent count adds its default 1");
    building(&mut w, "PADZ", 0, 4, 0);
    assert_eq!(pad_total(&w, 0), 1, "an explicit 0 adds 0");
    building(&mut w, "NOTPAD", 0, 0, 4);
    assert_eq!(
        pad_total(&w, 0),
        1,
        "a building without the helipad flag adds nothing"
    );
    let t3 = building(&mut w, "PADT", 0, 4, 4);
    assert_eq!(pad_total(&w, 0), 4);
    // An owner change moves the count.
    w.obj_mut(t3).unwrap().owner = Some(HouseId(1));
    glasnost_sim::econ::events::owner_changed(&mut w, t3, Some(H));
    assert_eq!((pad_total(&w, 0), pad_total(&w, 1)), (1, 3));
    // Removal subtracts, never below 0.
    w.houses[1].econ.tech.pad_total = 2;
    objects::remove(&mut w, t3);
    assert_eq!(pad_total(&w, 1), 0, "the floor of 0");
    objects::remove(&mut w, x);
    assert_eq!(pad_total(&w, 0), 0);
}

// spec: S-0407 F3 and S-0501 F12 (the pad-capacity quantities use exact non-negative count arithmetic: the pad total does not wrap past the 32-bit range)
#[test]
fn f3_pad_total_exact_count() {
    let mut r = ruleset();
    let mut b = plain(100);
    b.helipad = Some(true);
    b.econ.number_of_docks = Some(3);
    add_building(&mut r, "PADT", b);
    let mut w = game(&r);
    w.houses[0].econ.tech.pad_total = i64::from(i32::MAX);
    building(&mut w, "PADT", 0, 0, 0);
    assert_eq!(pad_total(&w, 0), i64::from(i32::MAX) + 3);
}

// spec: S-0501 claim 30 and F12, S-0407 claim 16 (quirks.pad_limit_counts_list_only: on, a pad-bound type outside the list is limited by the list types only; off, its own present objects and line copies count too)
#[test]
fn claim_16_pad_limit_counts_list_only() {
    for (list_only, accepted) in [(true, 1), (false, 0)] {
        let mut r = ruleset();
        r.globals.quirks.pad_limit_counts_list_only = list_only;
        r.sim.buildings.get_mut("AIRPAD").unwrap().helipad = Some(true);
        r.sim
            .buildings
            .get_mut("AIRPAD")
            .unwrap()
            .econ
            .number_of_docks = Some(2);
        r.registries
            .get_mut("aircraft")
            .unwrap()
            .push("PADA".into());
        let mut a = r.sim.aircraft["PLANE"].clone();
        a.econ.airport_bound = Some(true);
        r.sim.aircraft.insert("PADA".into(), a);
        // PADA is not on the list; the list names only PLANE.
        r.sim.general.economy.pad_aircraft = Some(vec!["PLANE".into()]);
        let mut w = game(&r);
        building(&mut w, "AIRPAD", 0, 0, 0);
        testkit::spawn(
            &mut w,
            Kind::Aircraft,
            "PADA",
            0,
            testkit::mid(SIZE, 6, 6),
            0,
        );
        ready(&mut w);
        set_credits(&mut w, 0, 100_000);
        let pa = t(&w, Kind::Aircraft, "PADA");
        assert!(issue(&mut w, 0, Order::Build { what: pa }));
        step(&mut w);
        issue(&mut w, 0, Order::Build { what: pa });
        step(&mut w);
        assert_eq!(
            line(&w, 0, pa).unwrap().queue.len(),
            accepted,
            "list only {list_only}"
        );
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
