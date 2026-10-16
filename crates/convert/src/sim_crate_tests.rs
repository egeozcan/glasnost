//! Tests of the crate fields of the `sim` section (S-0520 Import mapping) on
//! invented data (every name and number here is made up).

use glasnost_rules::sim::SimData;

use crate::inputs::{InputFile, Inputs};
use crate::{ImportOptions, convert};

fn sim(base: &str) -> SimData {
    let inputs = Inputs {
        base: Some(InputFile {
            name: "rules.ini".into(),
            origin: "test".into(),
            bytes: base.as_bytes().to_vec(),
        }),
        ..Inputs::default()
    };
    let import = convert(&inputs, &ImportOptions::default());
    glasnost_rules::resolve_ruleset(Some((import.tree.clone(), import.provenance.clone())), &[])
        .unwrap_or_else(|e| panic!("{e}"))
        .ruleset
        .sim
}

const BASE: &str = "[CrateRules]\nCrateRadius=3.5\nCrateRegen=0.5\nUnitCrateType=Goody\n\
    WoodCrateImg=Wood\nWaterCrateImg=Sea\n[CombatDamage]\nFlameDamage=Burn\n\
    [Powerups]\nMoney=60,Gold,yes,200\nUnit=30,,no,0\nHealBase=5\nGas=2,Fog,yes,12%\n\
    [OverlayTypes]\n1=Wood\n2=Sea\n[Wood]\nCrate=yes\n[Sea]\nCrate=yes\nCrateTrigger=yes\n\
    [VehicleTypes]\n1=Goody\n[Goody]\nCrateGoodie=yes\n\
    [BuildingTypes]\n1=Pit\n[Pit]\nCrateBeneath=yes\nCrateBeneathIsMoney=yes\n\
    [Warheads]\n1=Burn\n[Burn]\n[AnimationTypes]\n1=Gold\n2=Fog\n[Gold]\n[Fog]\n";

// spec: S-0520 Import mapping (the crate keys: radius and regeneration decimals, the unit-crate type, the two overlays, the flame warhead)
#[test]
fn the_crate_rules_keys() {
    let e = sim(BASE).general.economy;
    assert_eq!(e.crate_radius.as_ref().map(|d| d.0.as_str()), Some("35e-1"));
    assert_eq!(e.crate_regen.as_ref().map(|d| d.0.as_str()), Some("5e-1"));
    assert_eq!(e.unit_crate_type.as_deref(), Some("Goody"));
    assert_eq!(e.wood_crate_img.as_deref(), Some("Wood"));
    assert_eq!(e.water_crate_img.as_deref(), Some("Sea"));
    assert_eq!(e.flame_damage.as_deref(), Some("Burn"));
}

// spec: S-0520 Import mapping (the power-up list: weight, animation, water flag, argument; a key absent while the section exists: weight 0 and no animation; a short list keeps the water flag and argument)
#[test]
fn the_power_up_table() {
    let t = sim(BASE).general.economy.powerups.expect("a table");
    assert_eq!(t.len(), 19);
    assert_eq!(
        (t[0].weight, t[0].animation.as_deref(), t[0].water),
        (60, Some("Gold"), true)
    );
    assert_eq!(t[0].argument.as_ref().map(|d| d.0.as_str()), Some("200"));
    assert_eq!(
        (t[1].weight, t[1].animation.is_none(), t[1].water),
        (30, true, false)
    );
    assert_eq!((t[2].weight, t[2].animation.is_none()), (5, true));
    assert_eq!(
        (t[3].weight, t[3].animation.is_none()),
        (0, true),
        "Cloak is absent"
    );
    assert_eq!(t[16].weight, 2);
    assert_eq!(t[16].animation.as_deref(), Some("Fog"));
    assert!(t[16].water);
}

// spec: S-0520 Import mapping (without a [Powerups] section no table is written: the built-in values stay in force)
#[test]
fn no_section_no_table() {
    assert!(
        sim("[CrateRules]\nCrateRegen=1\n")
            .general
            .economy
            .powerups
            .is_none()
    );
}

// spec: S-0520 F3 (a crate radius of exactly -1 keeps the value in force: the importer drops it)
#[test]
fn a_radius_of_minus_one_is_dropped() {
    let e = sim("[CrateRules]\nCrateRadius=-1\n").general.economy;
    assert!(e.crate_radius.is_none());
    let e = sim("[CrateRules]\nCrateRadius=2\n").general.economy;
    assert_eq!(e.crate_radius.as_ref().map(|d| d.0.as_str()), Some("2"));
}

// spec: S-0520 Import mapping (the type flags: CrateGoodie of vehicle types, CrateTrigger of overlay types, and S-0312's CrateBeneath keys of building types)
#[test]
fn the_type_flags() {
    let s = sim(BASE);
    assert_eq!(s.vehicles["Goody"].crate_goodie, Some(true));
    assert_eq!(s.overlays["Sea"].crate_trigger, Some(true));
    assert_eq!(s.overlays["Wood"].crate_trigger, None);
    assert_eq!(s.overlays["Wood"].crate_, Some(true));
    assert_eq!(s.buildings["Pit"].crate_beneath, Some(true));
    assert_eq!(s.buildings["Pit"].crate_beneath_is_money, Some(true));
}

fn layered(base: &str, mode: &str, map: &str) -> Vec<glasnost_rules::sim_econ::Powerup> {
    layered_with(base, mode, map, crate::layers::LayerQuirks::default())
}

fn layered_with(
    base: &str,
    mode: &str,
    map: &str,
    quirks: crate::layers::LayerQuirks,
) -> Vec<glasnost_rules::sim_econ::Powerup> {
    let file = |n: &str, t: &str| InputFile {
        name: n.into(),
        origin: "test".into(),
        bytes: t.as_bytes().to_vec(),
    };
    let inputs = Inputs {
        base: Some(file("rules.ini", base)),
        mode: Some(file("mode.ini", mode)),
        map: Some(file("test.map", map)),
        ..Inputs::default()
    };
    let opts = ImportOptions {
        quirks,
        ..ImportOptions::default()
    };
    let import = convert(&inputs, &opts);
    glasnost_rules::resolve_ruleset(Some((import.tree, import.provenance)), &[])
        .unwrap_or_else(|e| panic!("{e}"))
        .ruleset
        .sim
        .general
        .economy
        .powerups
        .expect("a table")
}

// spec: S-0520 claim 28 and S-0006 claim 5 (quirks.override_resets_keys off: an omitted entry retains all four held values; an entry no layer mentions has the fixed default of S-0006's switch-off paragraph, weight 0 and no animation: Q-1034 asks whether S-0520's built-in table applies instead)
#[test]
fn the_power_up_table_with_the_reset_switch_off() {
    let quirks = crate::layers::LayerQuirks {
        override_resets_keys: false,
        ..crate::layers::LayerQuirks::default()
    };
    let a = layered_with(LAYER_BASE, "[Powerups]\nUnit=7\n", "", quirks);
    assert_eq!(
        (a[0].weight, a[0].animation.as_deref(), a[0].water),
        (60, Some("Gold"), true),
        "the omitted entry keeps its held values"
    );
    assert_eq!(a[0].argument.as_ref().map(|d| d.0.as_str()), Some("200"));
    assert_eq!(a[1].weight, 7);
    assert_eq!(a[3].weight, 0, "never mentioned (Q-1034)");
}

const LAYER_BASE: &str =
    "[Powerups]\nMoney=60,Gold,yes,200\nUnit=30\n[AnimationTypes]\n1=Gold\n[Gold]\n";

// spec: S-0520 claim 28 and Import mapping, scenario CR19 (a power-up section in an intermediate layer resets every omitted entry's weight and animation to 0 and the NONE lookup (no animation) while keeping the water flag and the argument; a layer without the section changes none; a map restating an entry with only weight and animation keeps the earlier water flag and argument; a section of only empty entries does not exist)
#[test]
fn the_power_up_table_is_layered() {
    // A: the mode holds the section with another effect only; the map omits it.
    let a = layered(LAYER_BASE, "[Powerups]\nUnit=7\n", "");
    assert_eq!((a[0].weight, a[0].animation.is_none()), (0, true), "A");
    assert!(a[0].water, "A keeps the water flag");
    assert_eq!(
        a[0].argument.as_ref().map(|d| d.0.as_str()),
        Some("200"),
        "A keeps the argument"
    );
    assert_eq!(a[1].weight, 7);
    // B: no layer after the base holds the section.
    let b = layered(LAYER_BASE, "", "");
    assert_eq!(
        (b[0].weight, b[0].animation.as_deref()),
        (60, Some("Gold")),
        "B"
    );
    // C: A's mode, and the map restates Money with only weight and animation.
    let c = layered(
        LAYER_BASE,
        "[Powerups]\nUnit=7\n",
        "[Powerups]\nMoney=9,Gold\n",
    );
    assert_eq!(
        (c[0].weight, c[0].animation.as_deref()),
        (9, Some("Gold")),
        "C"
    );
    assert!(c[0].water);
    assert_eq!(c[0].argument.as_ref().map(|d| d.0.as_str()), Some("200"));
    // A section holding only empty entries does not exist in its layer.
    let e = layered(LAYER_BASE, "[Powerups]\nCloak=\n", "");
    assert_eq!(
        (e[0].weight, e[0].animation.as_deref()),
        (60, Some("Gold")),
        "empty entries"
    );
}

// spec: S-0520 claim 28 and Import mapping (the reset's animation is the lookup of NONE); S-0003 claim 14 (`none` names no type, so no animation type can carry that name in a type list: the lookup finds none; Q-1033 asks how CR19's separately defined type of that name arises)
#[test]
fn the_reset_animation_lookup_finds_no_type_named_none() {
    let base = "[Powerups]\nMoney=60,Gold,yes,200\nUnit=30\n[AnimationTypes]\n1=Gold\n2=NONE\n[Gold]\n[NONE]\n";
    let a = layered(base, "[Powerups]\nUnit=7\n", "");
    assert_eq!(a[0].animation, None);
}
