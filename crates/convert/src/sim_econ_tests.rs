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
    import_full(base, "", art)
}

fn import_with_map(base: &str, map: &str) -> (SimData, Vec<String>) {
    import_full(base, map, "")
}

fn import_full(base: &str, map: &str, art: &str) -> (SimData, Vec<String>) {
    let inputs = Inputs {
        base: Some(file("rules.ini", base)),
        mode: Some(file("mode.ini", "")),
        map: Some(file("test.map", map)),
        art: Some(file("art.ini", art)),
        ..Inputs::default()
    };
    let import = convert(&inputs, &ImportOptions::default());
    let r = glasnost_rules::resolve_ruleset(Some((import.tree, import.provenance)), &[])
        .unwrap_or_else(|e| panic!("{e}"));
    let w = import.warnings.iter().map(|w| w.message.clone()).collect();
    (r.ruleset.sim, w)
}

const BASE: &str = "[Countries]\n0=Ruritania\n[Ruritania]\nCostUnitsMult=1.25\nIncomeMult=0.75\nSide=Blue\nMultiplay=yes\nCost=2\nSmartAI=yes\nPrefix=B\nSuffix=Ru\nParentCountry=Ruritania\n\
[BuildingTypes]\n0=MILL\n1=PLANT\n[MILL]\nPower=-30\nRefinery=yes\nAdjacent=4\nPrerequisite=plant,power\nOwner=Ruritania,Nowhere\nUnitsCostBonus=0.5\nFactory=Whatever\nImage=MILLART\n\
[PLANT]\nPower=100\nExtraPower=7\nSight=6\n\
[VehicleTypes]\n0=TRUCK\n[TRUCK]\nDock=MILL,Elsewhere\nBuildLimit=3\nPoweredUnit=yes\n\
[Tiberiums]\n0=Rubies\n1=Opals\n[Rubies]\nValue=33\nGrowthPercentage=0.2\nImage=1\n[Opals]\nImage=2\n\
[SuperWeaponTypes]\n0=Zap\n[Zap]\nDisableableFromShell=yes\nType=Thing\n\
[General]\nHarvesterLoadRate=9\nAIHateDelays=4,5,6\nPrerequisitePower=PLANT\nPadAircraft=Ghost\n\
[AI]\nBuildTech=MILL\n\
[AudioVisual]\nSavourDelay=0.5\n\
[CrateRules]\nCrateMaximum=4\n\
[MultiplayerDialogSettings]\nBases=no\nShortGame=yes\nMoney=777\n";

const ART: &str = "[MILLART]\nQueueingCell=2,3\nNumberOfDocks=2\nSpecialAnim=SPIN\nGateStages=4\nAnimIdle=5\nNormalized=yes\n";

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
    assert_eq!(
        mill.econ.units_cost_bonus.as_ref().map(|d| d.0.as_str()),
        Some("5e-1")
    );
    assert_eq!(mill.econ.queueing_cell, Some([2, 3]));
    assert_eq!(mill.econ.art_number_of_docks, Some(2));
    // spec: S-0108 Import mapping (GateStages, an art integer; S-0406 claim 15)
    assert_eq!(mill.econ.gate_stages, Some(4));
    // spec: S-0108 Import mapping (AnimIdle: start, count, delay; unread components keep the held 0, 1, 0, with a warning; Normalized a boolean)
    assert_eq!(mill.econ.anim_idle, Some([5, 1, 0]));
    assert_eq!(mill.econ.normalized, Some(true));
    assert!(w.iter().any(|m| m.contains("AnimIdle")));
    assert_eq!(s.buildings["PLANT"].econ.anim_idle, None);
    assert_eq!(mill.econ.special_anim.as_deref(), Some("SPIN"));
    assert_eq!(s.buildings["PLANT"].econ.extra_power, Some(7));
    // spec: S-0409 Import mapping (a building's Sight, cells)
    assert_eq!(s.buildings["PLANT"].econ.sight, Some(6));
    // spec: S-0003 claim 21 (an unmatched word gives no factory, with a warning)
    assert_eq!(mill.factory, None);
    assert!(
        w.iter()
            .any(|m| m.contains("Factory") && m.contains("claim 21"))
    );
}

// spec: S-0003 claim 21 and Import mapping row C19 (the factory kind words)
#[test]
fn factory_kind_words() {
    // Byte 32 is past the 31-byte cut; byte 31 is not.
    let cut_ok = format!("InfantryType{}x", " ".repeat(31 - 12));
    let cut_bad = format!("InfantryType{}x", " ".repeat(30 - 12));
    assert_eq!(cut_ok.len(), 32);
    let names = [
        ("A", "infantrytype"),
        ("B", "UNITTYPE"),
        ("C", "AircraftType  "),
        ("D", "BuildingType"),
        ("E", "<NONE>"),
        ("F", "none"),
        ("G", "TerrainType"),
        ("H", "Infantry"),
        ("I", "UnitTypex"),
        ("J", "Unit Type"),
        ("K", cut_ok.as_str()),
        ("L", cut_bad.as_str()),
    ];
    let mut base = String::from("[BuildingTypes]\n");
    for (i, (n, _)) in names.iter().enumerate() {
        base.push_str(&format!("{i}={n}\n"));
    }
    for (n, v) in names {
        base.push_str(&format!("[{n}]\nFactory={v}\n"));
    }
    let (s, w) = import(&base, "");
    // A type left with no field at all is not written out.
    let kind = |n: &str| s.buildings.get(n).and_then(|b| b.factory.clone());
    assert_eq!(kind("A").as_deref(), Some("infantry"));
    assert_eq!(kind("B").as_deref(), Some("vehicle"));
    assert_eq!(kind("C").as_deref(), Some("aircraft"));
    assert_eq!(kind("D").as_deref(), Some("building"));
    assert_eq!(kind("K").as_deref(), Some("infantry"));
    // spec: S-0214 Import mapping (Factory: recognised non-producing categories are carried, not aliases of the producing words)
    assert_eq!(kind("G").as_deref(), Some("terrain"));
    assert_eq!(kind("H").as_deref(), Some("infantry_object"));
    for n in ["E", "F", "I", "J", "L"] {
        assert_eq!(kind(n), None, "{n}");
    }
    // Recognised words give no warning; unmatched ones do.
    let warned = |n: &str| {
        w.iter()
            .any(|m| m.contains(&format!("buildings {n} Factory")))
    };
    for n in ["E", "G", "H"] {
        assert!(!warned(n), "{n}");
    }
    for n in ["F", "I", "J", "L"] {
        assert!(warned(n), "{n}");
    }
}

// spec: S-0003 claim 21 (empty keeps the value in force; an unmatched word does not)
#[test]
fn factory_kind_layers() {
    let base = "[BuildingTypes]\n0=A\n1=B\n[A]\nFactory=UnitType\n[B]\nFactory=UnitType\n";
    let mode = "[A]\nFactory=\n[B]\nFactory=Nonsense\n";
    let inputs = Inputs {
        base: Some(file("rules.ini", base)),
        mode: Some(file("mode.ini", mode)),
        map: Some(file("test.map", "")),
        art: Some(file("art.ini", "")),
        ..Inputs::default()
    };
    let import = convert(&inputs, &ImportOptions::default());
    let r = glasnost_rules::resolve_ruleset(Some((import.tree, import.provenance)), &[])
        .unwrap_or_else(|e| panic!("{e}"));
    let s = r.ruleset.sim;
    assert_eq!(s.buildings["A"].factory.as_deref(), Some("vehicle"));
    assert_eq!(s.buildings.get("B").and_then(|b| b.factory.clone()), None);
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

// spec: S-0413 claim 4 (a two-character prefix and a four-character suffix keep one and three
// characters, trimmed, with a warning each; values within their limits give none; an empty suffix
// keeps the value in force. The empty-prefix rule conflicts with S-0001 claim 11: Q-0478)
#[test]
fn country_prefix_and_suffix_cut() {
    let base = "[Countries]\n0=Ruritania\n1=Freedonia\n2=Sylvania\n\
                [Ruritania]\nPrefix=BC\nSuffix=Ru xZ\n\
                [Freedonia]\nPrefix=F\nSuffix=Fre\n\
                [Sylvania]\nPrefix=S\nSuffix=Syl\n";
    let map = "[Sylvania]\nPrefix=\nSuffix=\n";
    let (s, w) = import_full(base, map, ART);
    let r = &s.countries["Ruritania"].econ;
    assert_eq!(r.prefix.as_deref(), Some("B"));
    assert_eq!(r.suffix.as_deref(), Some("Ru"));
    let f = &s.countries["Freedonia"].econ;
    assert_eq!(
        (f.prefix.as_deref(), f.suffix.as_deref()),
        (Some("F"), Some("Fre"))
    );
    let y = &s.countries["Sylvania"].econ;
    assert_eq!(
        (y.prefix.as_deref(), y.suffix.as_deref()),
        (Some("S"), Some("Syl"))
    );
    let cut: Vec<_> = w.iter().filter(|m| m.contains("S-0413 claim 4")).collect();
    assert_eq!(cut.len(), 2, "{cut:?}");
}

// spec: S-0402 Import mapping, S-0404 Import mapping, S-0413 claims 2 and 4 and Import mapping
#[test]
fn country_econ_fields_and_no_effect_warning() {
    let (s, w) = import(BASE, ART);
    let c = &s.countries["Ruritania"].econ;
    assert_eq!(
        c.cost_units_mult.as_ref().map(|d| d.0.as_str()),
        Some("125e-2")
    );
    assert_eq!(c.income_mult.as_ref().map(|d| d.0.as_str()), Some("75e-2"));
    assert_eq!(c.side.as_deref(), Some("Blue"));
    assert_eq!(c.multiplay, Some(true));
    assert_eq!(c.smart_ai, Some(true));
    assert_eq!(c.prefix.as_deref(), Some("B"));
    assert_eq!(c.suffix.as_deref(), Some("Ru"));
    assert_eq!(c.parent_country.as_deref(), Some("Ruritania"));
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
    assert_eq!(
        g.prerequisite_power.as_deref(),
        Some(&["PLANT".to_owned()][..])
    );
    assert_eq!(g.pad_aircraft.as_deref(), Some(&["Ghost".to_owned()][..]));
    assert_eq!(g.build_tech.as_deref(), Some(&["MILL".to_owned()][..]));
    assert_eq!(g.crate_maximum, Some(4));
    assert_eq!(g.bases, Some(false));
    assert_eq!(g.short_game, Some(true));
    assert_eq!(g.money, Some(777));
    assert_eq!(g.ai_hate_delays.as_deref(), Some(&[4, 5, 6][..]));
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

// spec: S-0108 Import mapping (Buildup, BuildupTime), S-0406 Import mapping (the word 3x3Refinery)
#[test]
fn buildup_and_refinery_foundation_are_converted() {
    let base = "[BuildingTypes]\n0=MILL\n1=PLANT\n2=SHED\n[MILL]\nImage=MILLART\nStrength=5\n[PLANT]\nImage=PLANTART\nStrength=5\n[SHED]\nImage=SHEDART\nStrength=5\n\
                [General]\nBuildupTime=0.25\n";
    let art = "[MILLART]\nFoundation=3X3REFINERY\nBuildup=MILLMK\n[PLANTART]\nFoundation=3x3\nBuildup=none\n[SHEDART]\nBuildup=\n";
    let (s, _) = import(base, art);
    let b = &s.buildings;
    assert_eq!(b["MILL"].foundation, Some([3, 3]));
    assert_eq!(b["MILL"].foundation_refinery, Some(true));
    assert_eq!(b["PLANT"].foundation_refinery, None);
    assert_eq!(b["MILL"].econ.buildup, Some(true));
    assert_eq!(b["PLANT"].econ.buildup, None);
    assert_eq!(b["SHED"].econ.buildup, None);
    assert_eq!(
        s.general
            .economy
            .buildup_time
            .as_ref()
            .map(|d| d.0.as_str()),
        Some("25e-2")
    );
}

// spec: S-0402 Import mapping (building Weeder) and Edge cases (a warning for each building type in a harvester's dock list that lacks DockUnload)
#[test]
fn weeder_and_the_dock_unload_warning() {
    let base = "[BuildingTypes]\n0=BINA\n1=BINB\n[BINA]\nRefinery=yes\nDockUnload=yes\n[BINB]\nWeeder=yes\n\
[VehicleTypes]\n0=CART\n1=WAGON\n[CART]\nHarvester=yes\nDock=BINA,BINB\n[WAGON]\nDock=BINB\n";
    let (s, w) = import(base, "");
    assert_eq!(s.buildings["BINB"].econ.weeder, Some(true));
    assert_eq!(s.buildings["BINA"].econ.weeder, None);
    assert_eq!(s.buildings["BINA"].econ.dock_unload, Some(true));
    let dock: Vec<&String> = w.iter().filter(|m| m.contains("DockUnload")).collect();
    assert_eq!(dock.len(), 1, "{w:?}");
    assert!(
        dock[0].contains("CART") && dock[0].contains("BINB"),
        "{dock:?}"
    );
}

// spec: S-0415 Edge cases and S-0406 Import mapping (a DeployFacing outside 0 … 255 gets an importer warning)
#[test]
fn deploy_facing_out_of_range_warning() {
    let base = "[BuildingTypes]\n0=DFA\n1=DFB\n2=DFC\n[DFA]\nDeployFacing=256\n[DFB]\nDeployFacing=-1\n[DFC]\nDeployFacing=255\n";
    let (s, w) = import(base, "");
    assert_eq!(s.buildings["DFA"].econ.deploy_facing, Some(256));
    let df: Vec<&String> = w.iter().filter(|m| m.contains("DeployFacing")).collect();
    assert_eq!(df.len(), 2, "{w:?}");
    assert!(df.iter().all(|m| !m.contains("DFC")), "{df:?}");
}

// spec: S-0404 claim 15 and Import mapping (DockingOffset0, 1, …: integer triples from the art section, read only below the rules NumberOfDocks, absent 1; a place without a key is 0, 0, 0)
#[test]
fn docking_offsets() {
    let base = "[BuildingTypes]\n0=PADA\n1=PADB\n2=PADC\n[PADA]\nNumberOfDocks=3\n[PADB]\nPower=1\n[PADC]\nNumberOfDocks=0\n";
    let art = "[PADA]\nDockingOffset0=1,2,3\nDockingOffset2=-4,5,6\nDockingOffset3=9,9,9\n\
[PADB]\nDockingOffset0=7,8,9\nDockingOffset1=1,1,1\n[PADC]\nDockingOffset0=1,1,1\n";
    let (s, _) = import(base, art);
    assert_eq!(
        s.buildings["PADA"].econ.docking_offsets,
        Some(vec![[1, 2, 3], [0, 0, 0], [-4, 5, 6]])
    );
    assert_eq!(
        s.buildings["PADB"].econ.docking_offsets,
        Some(vec![[7, 8, 9]])
    );
    assert_eq!(s.buildings["PADC"].econ.docking_offsets, None);
}

// spec: S-0408 Edge cases (warnings for a non-turret upgrade with PowersUpToLevel 2 or 3 and for Upgrades above 3)
#[test]
fn upgrade_sale_warnings() {
    let base = "[BuildingTypes]\n0=TOPA\n1=TOPB\n2=TOPC\n3=BASE\n[TOPA]\nPowersUpToLevel=3\n\
[TOPB]\nPowersUpToLevel=2\nTurret=yes\n[TOPC]\nPowersUpToLevel=1\n[BASE]\nUpgrades=4\n";
    let (_, w) = import(base, "");
    let lv: Vec<&String> = w.iter().filter(|m| m.contains("PowersUpToLevel")).collect();
    assert_eq!(lv.len(), 1, "{w:?}");
    assert!(lv[0].contains("TOPA"), "{lv:?}");
    let up: Vec<&String> = w.iter().filter(|m| m.contains("Upgrades=")).collect();
    assert_eq!(up.len(), 1, "{w:?}");
    assert!(up[0].contains("BASE"), "{up:?}");
}

// spec: S-0404 Import mapping (ExitCoord an integer triple in leptons; GDIBarracks, NODBarracks, YuriBarracks and Cloning booleans; [AudioVisual] PoseDir an integer)
#[test]
fn exit_displacement_doors_cloning_and_pose_direction() {
    let base = "[BuildingTypes]\n0=HUTA\n1=HUTB\n[HUTA]\nExitCoord=10,-20,30\nGDIBarracks=yes\n\
NODBarracks=no\nYuriBarracks=yes\n[HUTB]\nCloning=yes\nExitCoord=5\n[AudioVisual]\nPoseDir=96\n";
    let (s, w) = import(base, "");
    let a = &s.buildings["HUTA"].econ;
    assert_eq!(a.exit_coord, Some([10, -20, 30]));
    assert_eq!(a.door_first, Some(true));
    assert_eq!(a.door_second, Some(false));
    assert_eq!(a.door_third, Some(true));
    assert_eq!(a.cloning, None);
    let b = &s.buildings["HUTB"].econ;
    assert_eq!(b.cloning, Some(true));
    assert_eq!(b.exit_coord, Some([5, 0, 0]));
    assert!(
        w.iter()
            .any(|m| m.contains("ExitCoord") && m.contains("HUTB")),
        "{w:?}"
    );
    assert_eq!(s.general.economy.pose_dir, Some(96));
}

// spec: S-0004 Import mapping (counts as aircraft = ConsideredAircraft, read for unit types; absent: no)
#[test]
fn considered_aircraft_flag() {
    let base = "[VehicleTypes]\n0=CARA\n1=CARB\n[AircraftTypes]\n0=PLANEA\n[CARA]\nConsideredAircraft=yes\n\
[CARB]\nNaval=no\n[PLANEA]\nConsideredAircraft=no\n";
    let (s, _) = import(base, "");
    assert_eq!(s.vehicles["CARA"].econ.considered_aircraft, Some(true));
    assert_eq!(s.vehicles["CARB"].econ.considered_aircraft, None);
    assert_eq!(s.aircraft["PLANEA"].econ.considered_aircraft, Some(false));
}

/// spec: S-0003 claim 23, rows C21 to C23 and scenario V18 (the virtual-purifier, extra-credit and computer-timer lists: decimal-prefix items, an unreadable non-empty item 0 in its place, empty items none, 511-byte limit, item warnings; a present value replaces the whole list, missing or empty keeps it) (Q-0234)
#[test]
fn economy_integer_list_items() {
    for key in ["AIVirtualPurifiers", "MultiplayerAICM", "AIHateDelays"] {
        let read = |base: &str, map: &str| {
            let (s, w) = import_with_map(base, map);
            let g = &s.general.economy;
            let v = match key {
                "AIVirtualPurifiers" => g.ai_virtual_purifiers.clone(),
                "MultiplayerAICM" => g.multiplayer_aicm.clone(),
                _ => g.ai_hate_delays.clone(),
            };
            (v, w)
        };
        let one = |text: &str| read(&format!("[General]\n{key}={text}\n"), "");
        let v = |text: &str| one(text).0;
        assert_eq!(v("5,abc,9"), Some(vec![5, 0, 9]), "{key}");
        assert_eq!(v("5,$10,9"), Some(vec![5, 0, 9]), "{key}");
        assert_eq!(v("5,10h,9"), Some(vec![5, 10, 9]), "{key}");
        assert_eq!(v("5,0x10,9"), Some(vec![5, 0, 9]), "{key}");
        assert_eq!(v("5,,7,9"), Some(vec![5, 7, 9]), "{key}");
        assert_eq!(v("5, ,9"), Some(vec![5, 0, 9]), "{key}");
        assert_eq!(v("5,4294967303,9"), Some(vec![5, 7, 9]), "{key}");
        assert_eq!(v("10%"), Some(vec![10]), "{key}");
        // Text beyond byte 127 is read.
        assert_eq!(
            v(&format!("5{}7,9", ",".repeat(130))),
            Some(vec![5, 7, 9]),
            "{key}"
        );
        // Claim 3's warnings apply item by item; a clean list does not warn.
        let warns = |text: &str| {
            one(text)
                .1
                .iter()
                .any(|m| m.contains(key) && m.contains("claims 1, 3, 4 and 23"))
        };
        assert!(warns("5,abc,9"), "{key}");
        assert!(warns("5,10h,9"), "{key}");
        assert!(warns("5,4294967303,9"), "{key}");
        assert!(!warns("5,7,9"), "{key}");
        // Layer retention: absent and empty keep the whole list; a present
        // value replaces it, and only commas replace it with an empty list.
        let base = format!("[General]\n{key}=5,4,9\n");
        assert_eq!(read(&base, "").0, Some(vec![5, 4, 9]), "{key}");
        assert_eq!(
            read(&base, &format!("[General]\n{key}=\n")).0,
            Some(vec![5, 4, 9]),
            "{key}"
        );
        assert_eq!(
            read(&base, &format!("[General]\n{key}=5,abc,9\n")).0,
            Some(vec![5, 0, 9]),
            "{key}"
        );
        assert_eq!(
            read(&base, &format!("[General]\n{key}=,,,\n")).0,
            Some(vec![]),
            "{key}"
        );
    }
}
