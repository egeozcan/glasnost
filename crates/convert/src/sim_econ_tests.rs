//! Tests of the economy fields of the `sim` section on invented data (every
//! name and number here is made up).

use glasnost_rules::sim::SimData;

use super::econ::is_one_text;
use crate::inputs::{InputFile, Inputs};
use crate::{ImportOptions, convert};

fn file(name: &str, text: &str) -> InputFile {
    InputFile {
        name: name.into(),
        origin: "test".into(),
        bytes: text.as_bytes().to_vec(),
    }
}

fn import(base: &str, art: &str) -> (SimData, Vec<String>) {
    let inputs = Inputs {
        base: Some(file("rules.ini", base)),
        mode: Some(file("mode.ini", "")),
        map: Some(file("test.map", "")),
        art: Some(file("art.ini", art)),
        ..Inputs::default()
    };
    let import = convert(&inputs, &ImportOptions::default());
    let r = glasnost_rules::resolve_ruleset(Some((import.tree, import.provenance)), &[])
        .unwrap_or_else(|e| panic!("{e}"));
    let w = import.warnings.iter().map(|w| w.message.clone()).collect();
    (r.ruleset.sim, w)
}

const BASE: &str = "[Countries]\n0=Ruritania\n[Ruritania]\nCostUnitsMult=1.25\nIncomeMult=0.75\nSide=Blue\nMultiplay=yes\nCost=2\n\
[BuildingTypes]\n0=MILL\n1=PLANT\n[MILL]\nPower=-30\nRefinery=yes\nAdjacent=4\nPrerequisite=plant,power\nOwner=Ruritania,Nowhere\nUnitsCostBonus=0.5\nFactory=Whatever\nImage=MILLART\n\
[PLANT]\nPower=100\nExtraPower=7\n\
[VehicleTypes]\n0=TRUCK\n[TRUCK]\nDock=MILL,Elsewhere\nBuildLimit=3\nPoweredUnit=yes\n\
[Tiberiums]\n0=Rubies\n1=Opals\n[Rubies]\nValue=33\nGrowthPercentage=0.2\nImage=1\n[Opals]\nImage=2\n\
[SuperWeaponTypes]\n0=Zap\n[Zap]\nDisableableFromShell=yes\nType=Thing\n\
[General]\nHarvesterLoadRate=9\nPrerequisitePower=PLANT\nPadAircraft=Ghost\n\
[AI]\nBuildTech=MILL\n\
[AudioVisual]\nSavourDelay=0.5\n\
[CrateRules]\nCrateMaximum=4\n\
[MultiplayerDialogSettings]\nBases=no\nShortGame=yes\nMoney=777\n";

const ART: &str = "[MILLART]\nQueueingCell=2,3\nNumberOfDocks=2\nSpecialAnim=SPIN\n";

// spec: S-0402 Import mapping, S-0404 Import mapping, S-0405 Import mapping, S-0406 Import mapping, S-0407 Import mapping
#[test]
fn building_econ_fields_are_converted() {
    let (s, w) = import(BASE, ART);
    let mill = &s.buildings["MILL"];
    assert_eq!(mill.econ.power, Some(-30));
    assert_eq!(mill.econ.refinery, Some(true));
    assert_eq!(mill.econ.adjacent, Some(4));
    assert_eq!(
        mill.econ.prerequisite.as_deref(),
        Some(&["PLANT".to_owned(), "POWER".to_owned()][..])
    );
    assert_eq!(mill.owner.as_deref(), Some(&["Ruritania".to_owned()][..]));
    assert_eq!(mill.econ.units_cost_bonus.as_ref().map(|d| d.0.as_str()), Some("5e-1"));
    assert_eq!(mill.econ.queueing_cell, Some([2, 3]));
    assert_eq!(mill.econ.art_number_of_docks, Some(2));
    assert_eq!(mill.econ.special_anim.as_deref(), Some("SPIN"));
    assert_eq!(s.buildings["PLANT"].econ.extra_power, Some(7));
    // Q-0100: the factory field is not converted and the importer warns.
    assert_eq!(mill.factory, None);
    assert!(w.iter().any(|m| m.contains("Q-0100")));
}

// spec: S-0402 Import mapping, S-0405 Import mapping, S-0407 Import mapping
#[test]
fn unit_econ_fields_are_converted() {
    let (s, _) = import(BASE, ART);
    let truck = &s.vehicles["TRUCK"];
    assert_eq!(
        truck.econ.dock.as_deref(),
        Some(&["MILL".to_owned(), "Elsewhere".to_owned()][..])
    );
    assert_eq!(truck.econ.build_limit, Some(3));
    assert_eq!(truck.econ.powered_unit, Some(true));
}

// spec: S-0402 Import mapping, S-0404 Import mapping, S-0413 claim 2 and Import mapping
#[test]
fn country_econ_fields_and_no_effect_warning() {
    let (s, w) = import(BASE, ART);
    let c = &s.countries["Ruritania"].econ;
    assert_eq!(c.cost_units_mult.as_ref().map(|d| d.0.as_str()), Some("125e-2"));
    assert_eq!(c.income_mult.as_ref().map(|d| d.0.as_str()), Some("75e-2"));
    assert_eq!(c.side.as_deref(), Some("Blue"));
    assert_eq!(c.multiplay, Some(true));
    assert!(w.iter().any(|m| m.contains("S-0413 claim 2")));
}

// spec: S-0401 Import mapping, S-0407 Import mapping, S-0412 Import mapping, S-0414 Import mapping
#[test]
fn tiberiums_superweapons_and_general_econ() {
    let (s, _) = import(BASE, ART);
    assert_eq!(s.tiberiums["Rubies"].value, Some(33));
    assert_eq!(s.tiberiums["Rubies"].image_run, Some([102, 12, 8]));
    assert_eq!(s.tiberiums["Opals"].image_run, Some([27, 12, 0]));
    assert_eq!(s.superweapons["Zap"].disableable_from_shell, Some(true));
    assert_eq!(s.superweapons["Zap"].kind.as_deref(), Some("Thing"));
    let g = &s.general.economy;
    assert_eq!(g.harvester_load_rate, Some(9));
    assert_eq!(g.prerequisite_power.as_deref(), Some(&["PLANT".to_owned()][..]));
    assert_eq!(g.pad_aircraft.as_deref(), Some(&["Ghost".to_owned()][..]));
    assert_eq!(g.build_tech.as_deref(), Some(&["MILL".to_owned()][..]));
    assert_eq!(g.crate_maximum, Some(4));
    assert_eq!(g.bases, Some(false));
    assert_eq!(g.short_game, Some(true));
    assert_eq!(g.money, Some(777));
    assert!(g.savour_delay.is_some());
}

#[test]
fn one_text() {
    assert!(is_one_text("10e-1"));
    assert!(is_one_text("100%"));
    assert!(is_one_text("1"));
    assert!(!is_one_text("2"));
    assert!(!is_one_text("-1"));
    assert!(!is_one_text("11e-1"));
}

// spec: S-0112 claim 10 and Constants (the image run from the image numbers in layer order; 2 keeps an earlier S; −1 changes nothing)
#[test]
fn tiberium_image_runs() {
    use super::econ::image_run;
    assert_eq!(image_run(&[]), None);
    assert_eq!(image_run(&[-1]), None);
    assert_eq!(image_run(&[2]), Some([27, 12, 0]));
    assert_eq!(image_run(&[1, 2]), Some([27, 12, 8]));
    assert_eq!(image_run(&[3, -1]), Some([127, 12, 8]));
    assert_eq!(image_run(&[4]), Some([147, 12, 8]));
    assert_eq!(image_run(&[0]), Some([102, 12, 8]));
}
