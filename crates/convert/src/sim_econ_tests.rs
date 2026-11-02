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
[SuperWeaponTypes]\n0=Zap\n[Zap]\nDisableableFromShell=yes\nType=IronCurtain\n\
[General]\nHarvesterLoadRate=9\nAIHateDelays=4,5,6\nPrerequisitePower=PLANT\nPadAircraft=Ghost\n\
[AI]\nBuildTech=MILL\n\
[AudioVisual]\nSavourDelay=0.5\n\
[CrateRules]\nCrateMaximum=4\n\
[MultiplayerDialogSettings]\nBases=no\nShortGame=yes\nBridgeDestruction=no\nMoney=777\n";

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
// characters, trimmed, with a warning each; values within their limits give none; an empty prefix
// or suffix value keeps the value in force, S-0001 claim 11; Q-0478)
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

// spec: S-0413 claims 1 and 4, Import mapping and scenario C4 (r4, Q-0478: an empty, all-space or
// comment-only prefix value is no entry and keeps the held prefix and suffix, in a nonempty section
// and alone; with no earlier affix the fields stay absent, so the defaults A and empty apply)
// spec: S-0001 claims 11 and 12 and Import mapping row I5 (r4: no key-specific exception; a section
// holding only such a line is absent, so it triggers no parent-country reset, S-0006 claims 2, 3 and 5)
#[test]
fn country_empty_prefix_keeps_the_held_affixes() {
    let variants = [
        ("[Zedland]\nSide=Blue\n", true),
        ("[Zedland]\nSide=Blue\nPrefix=\n", true),
        ("[Zedland]\nSide=Blue\nPrefix=   \n", true),
        ("[Zedland]\nSide=Blue\nPrefix= ; note\n", true),
        ("[Zedland]\nPrefix=\n", false),
        ("[Zedland]\nPrefix=   \n", false),
        ("[Zedland]\nPrefix= ; note\n", false),
    ];
    for earlier in [true, false] {
        let affixes = if earlier {
            "Prefix=Z\nSuffix=XYZ\n"
        } else {
            ""
        };
        let base = format!(
            "[Countries]\n0=Zedland\n1=Ruritania\n[Zedland]\nParentCountry=Ruritania\n{affixes}"
        );
        for (map, section) in variants {
            let (s, _) = import_full(&base, map, ART);
            let z = &s.countries["Zedland"].econ;
            let want = if earlier {
                (Some("Z"), Some("XYZ"))
            } else {
                (None, None)
            };
            assert_eq!(
                (z.prefix.as_deref(), z.suffix.as_deref()),
                want,
                "{map:?}, earlier affixes {earlier}"
            );
            // A section present omits the parent country and resets it; an
            // absent one leaves it alone.
            let kept = z.parent_country.as_deref() == Some("Ruritania");
            assert_eq!(kept, !section, "{map:?}: {:?}", z.parent_country);
        }
    }
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
    assert_eq!(s.superweapons["Zap"].kind.as_deref(), Some("ironcurtain"));
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
    // spec: S-0511 Import mapping (BridgeDestruction, the default of the game option)
    assert_eq!(g.bridge_destruction, Some(false));
    assert_eq!(g.money, Some(777));
    assert_eq!(g.ai_hate_delays.as_deref(), Some(&[4, 5, 6][..]));
    assert!(g.savour_delay.is_some());
}

// spec: S-0512 claim 24 (the twelve kind words, whole and without regard to case; an unknown word keeps the previous layer's kind and link; no hyphen, underscore or space is ignored) and claim 23 (a word outside the action vocabulary sets no action; an empty value keeps the previous action)
#[test]
fn superweapon_kind_link_and_action_words_layer() {
    let base = "[SuperWeaponTypes]\n0=Zap\n1=Bolt\n2=Fresh\n\
[Zap]\nType=IronCurtain\nPreDependent=ChronoSphere\nAction=Nuke\n\
[Bolt]\nType=MultiMissile\nAction=Nuke\n[Fresh]\nType=Iron-Curtain\nPreDependent=\nAction=\n";
    let map = "[Zap]\nType=Iron_Curtain\nPreDependent=chrono warp\nAction=\n\
[Bolt]\nType=PsychicReveal\nAction=Bogus\n";
    let (s, w) = import_with_map(base, map);
    let z = &s.superweapons["Zap"];
    // Unknown spellings keep the base words; the empty action keeps the base action.
    assert_eq!(z.kind.as_deref(), Some("ironcurtain"));
    assert_eq!(z.pre_dependent.as_deref(), Some("chronosphere"));
    assert_eq!(z.action.as_deref(), Some("nuke"));
    // A listed word replaces the kind; a word outside the vocabulary sets no action.
    let b = &s.superweapons["Bolt"];
    assert_eq!(b.kind.as_deref(), Some("psychicreveal"));
    assert_eq!(b.action.as_deref(), Some("none"));
    // A type with only unknown words has no kind, link or action.
    let f = s.superweapons.get("Fresh").cloned().unwrap_or_default();
    assert_eq!(f.kind, None);
    assert_eq!(f.pre_dependent, None);
    assert_eq!(f.action, None);
    assert!(w.iter().any(|m| m.contains("S-0512 claim 24")));
}

// spec: S-0513 claim 43 and Import mapping (the spy plane count compares the lengths of the compact creating infantry-reference list and the independently parsed count list: empty items and no-type names are omitted, an unknown name or a vehicle's name counts as an entry; S-0522 claim 2)
#[test]
fn spy_plane_count_lists_are_the_resolved_ones() {
    let base = "[VehicleTypes]\n0=Truck\n[Truck]\nStrength=1\n\
[General]\nAllyParaDropInf=Fresh,,Truck,none,<none>,Other\nAllyParaDropNum=1,2,3\n";
    let (s, _) = import_with_map(base, "");
    let g = &s.general.superweapons;
    let inf = g.ally_para_drop_inf.clone().unwrap_or_default();
    assert_eq!(inf.len(), 3, "resolved entries, not text tokens: {inf:?}");
    assert_eq!(g.ally_para_drop_num.as_ref().map(Vec::len), Some(3));
}

// spec: S-0103 Import mapping and claim 8 (row 3), S-0522 claim 13 (`Parachute` is a creating animation reference: an unlisted name creates a type; `none` and `<none>` clear it; an omitted or empty layered value retains the held type; initially none, which is allowed)
#[test]
fn parachute_reference_layers_and_absence() {
    let parachute = |base: &str, mode: &str| {
        let inputs = Inputs {
            base: Some(file("rules.ini", base)),
            mode: Some(file("mode.ini", mode)),
            map: Some(file("test.map", "")),
            art: Some(file("art.ini", "")),
            ..Inputs::default()
        };
        let import = convert(&inputs, &ImportOptions::default());
        let warned = import
            .warnings
            .iter()
            .any(|w| w.message.to_ascii_lowercase().contains("parachute"));
        let r = glasnost_rules::resolve_ruleset(Some((import.tree, import.provenance)), &[])
            .unwrap_or_else(|e| panic!("{e}"));
        (r.ruleset.sim.general.superweapons.parachute, warned)
    };
    // An unlisted name creates an animation type and is the held type.
    assert_eq!(
        parachute("[General]\nParachute=Fresh\n", "").0.as_deref(),
        Some("Fresh")
    );
    // Initially none, with no warning for the absence.
    assert_eq!(parachute("[General]\n", ""), (None, false));
    // An empty layered value retains the held type; `none` and `<none>` clear it.
    let held = "[General]\nParachute=Fresh\n";
    assert_eq!(
        parachute(held, "[General]\nParachute=\n").0.as_deref(),
        Some("Fresh")
    );
    assert_eq!(parachute(held, "[General]\nParachute=none\n").0, None);
    assert_eq!(parachute(held, "[General]\nParachute=<none>\n").0, None);
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

// spec: S-0402 claim 1 and Import mapping (a vehicle harvester of Storage 0, given or absent, warns; a positive one does not)
#[test]
fn zero_capacity_harvester_warning() {
    let base = "[VehicleTypes]\n0=CART\n1=WAGON\n2=BUGGY\n3=TRUCK\n[CART]\nHarvester=yes\nStorage=0\n\
[WAGON]\nHarvester=yes\n[BUGGY]\nHarvester=yes\nStorage=4\n[TRUCK]\nStorage=0\n";
    let (_, w) = import(base, "");
    let z: Vec<&String> = w.iter().filter(|m| m.contains("Storage 0")).collect();
    assert_eq!(z.len(), 2, "{w:?}");
    assert!(
        z.iter().any(|m| m.contains("CART")) && z.iter().any(|m| m.contains("WAGON")),
        "{z:?}"
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

// spec: S-0410 Import mapping (RadarEventSpeed and RadarEventRotationSpeed floating numbers or percent, RadarEventMinRadius an integer; a later layer without the key keeps the value)
#[test]
fn radar_event_zoom_keys() {
    let base =
        "[General]\nRadarEventSpeed=150%\nRadarEventMinRadius=3\nRadarEventRotationSpeed=0.2\n";
    let (s, _) = import_with_map(base, "[General]\nRadarEventMinRadius=4\n");
    let g = &s.general.economy;
    assert_eq!(
        g.radar_event_speed.as_ref().map(|d| d.0.as_str()),
        Some("150%")
    );
    assert_eq!(g.radar_event_min_radius, Some(4));
    assert_eq!(
        g.radar_event_rotation_speed.as_ref().map(|d| d.0.as_str()),
        Some("2e-1")
    );
    let (s, _) = import("[General]\nFlashFrameTime=7\n", "");
    assert_eq!(s.general.economy.radar_event_speed, None);
}

// spec: S-0510 Import mapping (SpyMoneyStealPercent: a percentage field, kept as written for the single-precision reading; SpyPowerBlackout an integer; a later layer that omits the key keeps the earlier value)
#[test]
fn spy_general_keys() {
    let base = "[General]\nSpyMoneyStealPercent=57%\nSpyPowerBlackout=300\n";
    let (s, _) = import_with_map(base, "[General]\nFlashFrameTime=7\n");
    let g = &s.general.economy;
    assert_eq!(
        g.spy_money_steal_percent.as_ref().map(|d| d.0.as_str()),
        Some("57%")
    );
    assert_eq!(g.spy_power_blackout, Some(300));
    let (s, _) = import("[General]\nFlashFrameTime=7\n", "");
    assert_eq!(s.general.economy.spy_money_steal_percent, None);
    // spec: S-0510 Edge cases (outside 0 to 1: a warning; 100% and 0% are inside)
    let warned = |v: &str| {
        let (_, w) = import(&format!("[General]\nSpyMoneyStealPercent={v}\n"), "");
        w.iter()
            .any(|m| format!("{m:?}").contains("SpyMoneyStealPercent"))
    };
    assert!(warned("150%"));
    assert!(warned("-10%"));
    assert!(warned("2"));
    assert!(!warned("100%"));
    assert!(!warned("0"));
    assert!(!warned("57%"));
}

// spec: S-0515 Import mapping (ProduceCashStartup, ProduceCashAmount and ProduceCashDelay: integers on a building type; absent: no value, read as 0; a later section that omits a key keeps the value in force)
#[test]
fn cash_keys_of_a_building_type() {
    let base = "[BuildingTypes]\n0=DERRICK\n1=PLAIN\n[DERRICK]\nProduceCashStartup=1500\nProduceCashAmount=-5\nProduceCashDelay=11\n[PLAIN]\nStrength=100\n";
    let (s, _) = import_with_map(base, "[DERRICK]\nProduceCashAmount=40\n");
    let d = &s.buildings["DERRICK"];
    assert_eq!(d.produce_cash_startup, Some(1500));
    assert_eq!(d.produce_cash_amount, Some(40));
    assert_eq!(d.produce_cash_delay, Some(11));
    let p = &s.buildings["PLAIN"];
    assert_eq!(
        (
            p.produce_cash_startup,
            p.produce_cash_amount,
            p.produce_cash_delay
        ),
        (None, None, None)
    );
}

// spec: S-0502 Import mapping and Key case (revision 17; S-0001 claim 13; scenario J1 c) (the jumpjet fields of a type: JumpjetSpeed, JumpjetClimb, JumpjetCrash, JumpjetHeight, JumpjetAccel, JumpjetWobbles, JumpjetDeviation, JumpjetNoWobbles, JumpjetTurnRate; DeployToLand; a key written with different letter case does not set the field: with no exact key in any layer the absent-key default applies, otherwise the value from an earlier layer remains; a later layer that omits a key keeps the value)
#[test]
fn jumpjet_type_keys() {
    let base = "[InfantryTypes]\n0=JJ\n1=CASE\n[JJ]\nJumpjetSpeed=20\nJumpjetClimb=3.5\nJumpjetCrash=7.25\nJumpjetHeight=600\nJumpjetAccel=1.5\nJumpjetWobbles=0.3\nJumpjetDeviation=25\nJumpjetNoWobbles=yes\nJumpjetTurnRate=9\nDeployToLand=yes\n[CASE]\njumpjetAccel=9.5\njumpjetTurnRate=77\n";
    let (s, _) = import_with_map(
        base,
        "[JJ]\nJumpjetSpeed=22\njumpjetAccel=4\njumpjetTurnRate=30\n",
    );
    let t = &s.infantry["JJ"];
    assert_eq!(t.jumpjet_speed, Some(22));
    let rz = glasnost_num::Rounding::TowardZero;
    let text = |d: &Option<glasnost_rules::sim::Decimal>| {
        d.as_ref()
            .map(|d| d.single_value(rz) == glasnost_num::decimal::nearest_double(&format_single(d)))
    };
    // The values read as the single-precision numbers of the written decimals.
    fn format_single(d: &glasnost_rules::sim::Decimal) -> String {
        d.0.clone()
    }
    let _ = text;
    let single = |d: &Option<glasnost_rules::sim::Decimal>, written: &str| {
        d.as_ref().map(|d| d.single_value(rz))
            == Some(glasnost_rules::sim::Decimal(written.into()).single_value(rz))
    };
    assert!(single(&t.jumpjet_climb, "3.5"));
    assert!(single(&t.jumpjet_crash, "7.25"));
    assert_eq!(t.jumpjet_height, Some(600));
    assert!(single(&t.jumpjet_accel, "1.5"));
    assert!(single(&t.jumpjet_wobbles, "0.3"));
    assert_eq!(t.jumpjet_deviation, Some(25));
    assert_eq!(t.jumpjet_no_wobbles, Some(true));
    assert_eq!(t.jumpjet_turn_rate, Some(9));
    assert_eq!(t.deploy_to_land, Some(true));
    // JJ above: the later layer's altered-case keys left the earlier layer's 1.5 and 9.
    let case = &s.infantry["CASE"];
    assert_eq!(
        (case.jumpjet_accel.clone(), case.jumpjet_turn_rate),
        (None, None),
        "keys with different letter case set nothing: the absent-key defaults apply"
    );
}

// spec: S-0517 Import mapping (DeployingAnim names an animation type, UnloadingClass a vehicle type, DeployTime is read and affects no vehicle; the presentation's Rate, Start, End and Shadow keys)
#[test]
fn deployable_unit_keys() {
    let base = "[VehicleTypes]\n0=MCV\n1=IMG\n[Animations]\n0=PACK\n[MCV]\nDeployingAnim=PACK\nUnloadingClass=IMG\nDeployTime=.02\nDeploysInto=\n[IMG]\n";
    let art = "[PACK]\nRate=450\nStart=2\nEnd=7\nShadow=yes\n";
    let (s, _) = import(base, art);
    let v = &s.vehicles["MCV"];
    assert_eq!(v.deploying_anim.as_deref(), Some("PACK"));
    assert_eq!(v.unloading_class.as_deref(), Some("IMG"));
    assert!(v.deploy_time.is_some());
    let a = &s.animations["PACK"];
    assert_eq!(
        (a.rate, a.start, a.end, a.shadow),
        (Some(450), Some(2), Some(7), Some(true))
    );
    // Absent keys: none.
    let (s, _) = import("[VehicleTypes]\n0=MCV\n[MCV]\nStrength=5\n", "");
    let v = &s.vehicles["MCV"];
    assert_eq!(
        (v.deploying_anim.clone(), v.unloading_class.clone()),
        (None, None)
    );
}

// spec: S-0502 Import mapping ([AudioVisual] DeployDir: an integer 8-step direction, absent: none given; the six [JumpjetControls] values are read and ignored)
#[test]
fn deploy_direction_and_jumpjet_controls() {
    let (s, _) = import(
        "[AudioVisual]\nDeployDir=9\n[JumpjetControls]\nSpeed=3\nClimb=2\n",
        "",
    );
    assert_eq!(s.general.economy.deploy_dir, Some(9));
    let (s, _) = import("[General]\nFlashFrameTime=7\n", "");
    assert_eq!(s.general.economy.deploy_dir, None);
}

// spec: S-0004 r20 Import mapping (counts as aircraft = ConsideredAircraft, read for unit types; an absent key is kept as absent here, and the simulation gives it the default by kind: yes for an aircraft type, no for a vehicle type, S-0501 Import mapping)
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

// spec: S-0501 Import mapping (Landable, AirportBound, Fighter, FlyBy and FlyBack booleans, FlightLevel an integer for aircraft types; UnitReload a boolean for building types; [General] FlightLevel, ReloadRate, CurleyShuffle and AttackingAircraftSightRange; [AudioVisual] SpyPlaneCameraFrames)
#[test]
fn aircraft_flight_and_pad_keys() {
    let base = "[AircraftTypes]\n0=KITE\n1=GULL\n[KITE]\nLandable=yes\nAirportBound=yes\n\
Fighter=yes\nFlyBy=no\nFlyBack=yes\nFlightLevel=333\n[GULL]\nStrength=7\n\
[BuildingTypes]\n0=NEST\n[NEST]\nHelipad=yes\nUnitReload=yes\nNumberOfDocks=3\n\
[General]\nFlightLevel=444\nReloadRate=0.07\nCurleyShuffle=yes\nAttackingAircraftSightRange=3\n\
[AudioVisual]\nSpyPlaneCameraFrames=12\n";
    let (s, _) = import(base, "");
    let k = &s.aircraft["KITE"].econ;
    assert_eq!(k.landable, Some(true));
    assert_eq!(k.airport_bound, Some(true));
    assert_eq!(k.fighter, Some(true));
    assert_eq!(k.fly_by, Some(false));
    assert_eq!(k.fly_back, Some(true));
    assert_eq!(k.flight_level, Some(333));
    // spec: S-0501 Import mapping (an absent FlightLevel stays unset: the game-wide value)
    let g = &s.aircraft["GULL"].econ;
    assert_eq!(
        (g.landable, g.fighter, g.fly_by, g.fly_back, g.flight_level),
        (None, None, None, None, None)
    );
    let n = &s.buildings["NEST"].econ;
    assert_eq!(n.unit_reload, Some(true));
    assert_eq!(n.number_of_docks, Some(3));
    let e = &s.general.economy;
    assert_eq!(e.flight_level, Some(444));
    assert_eq!(e.reload_rate.as_ref().map(|d| d.0.as_str()), Some("7e-2"));
    assert_eq!(e.curley_shuffle, Some(true));
    assert_eq!(e.attacking_aircraft_sight_range, Some(3));
    assert_eq!(e.spy_plane_camera_frames, Some(12));
}

// spec: S-0502 Edge cases (a jumpjet speed or turn rate of 0: an importer warning, no switch)
#[test]
fn zero_jumpjet_speed_and_turn_rate_warn() {
    let base = "[InfantryTypes]\n0=ZS\n1=ZT\n2=OK\n[ZS]\nJumpjetSpeed=0\n[ZT]\nJumpjetTurnRate=0\n[OK]\nJumpjetSpeed=5\nJumpjetTurnRate=4\n";
    let (_, w) = import(base, "");
    let hits = |k: &str| w.iter().filter(|m| m.contains(k)).count();
    assert_eq!(hits("JumpjetSpeed=0"), 1);
    assert_eq!(hits("JumpjetTurnRate=0"), 1);
}

// spec: S-0610 Import mapping (HasRadialIndicator, a building boolean, absent no; PsychicDetectionRadius, cells, absent 0; PsychicRevealRadius and BerzerkAllowed in [CombatDamage], PsychicRevealActivateSound and BerserkColor in [AudioVisual]; absent: no value)
#[test]
fn psychic_keys() {
    let base = "[BuildingTypes]\n1=EYE\n2=HUT\n\
                [EYE]\nPsychicDetectionRadius=7\nHasRadialIndicator=yes\n[HUT]\nStrength=10\n\
                [CombatDamage]\nPsychicRevealRadius=4\nBerzerkAllowed=no\n\
                [AudioVisual]\nPsychicRevealActivateSound=Hum\nBerserkColor=3\n";
    let (s, _) = import(base, "");
    assert_eq!(s.buildings["EYE"].econ.psychic_detection_radius, Some(7));
    assert_eq!(s.buildings["EYE"].econ.has_radial_indicator, Some(true));
    assert_eq!(s.buildings["HUT"].econ.psychic_detection_radius, None);
    assert_eq!(s.buildings["HUT"].econ.has_radial_indicator, None);
    let g = &s.general.superweapons;
    assert_eq!(g.psychic_reveal_radius, Some(4));
    assert_eq!(g.berzerk_allowed, Some(false));
    assert_eq!(g.psychic_reveal_activate_sound.as_deref(), Some("Hum"));
    assert_eq!(g.berserk_color, Some(3));
    let (s, _) = import("[CombatDamage]\nBallisticScatter=1\n", "");
    let g = &s.general.superweapons;
    assert_eq!(
        (g.psychic_reveal_radius, g.berzerk_allowed, g.berserk_color),
        (None, None, None)
    );
    assert_eq!(g.psychic_reveal_activate_sound, None);
}

// spec: S-0606 Import mapping (art SpecialAnimTwo, SpecialAnimThree, SpecialAnimFour and their damaged forms, as S-0108; absent: none)
#[test]
fn bunker_wall_anims_are_converted() {
    let art = "[MILLART]\nSpecialAnimTwo=W2\nSpecialAnimTwoDamaged=W2D\nSpecialAnimThree=W3\nSpecialAnimThreeDamaged=W3D\nSpecialAnimFour=W4\nSpecialAnimFourDamaged=W4D\n";
    let (s, _) = import(BASE, art);
    let e = &s.buildings["MILL"].econ;
    assert_eq!(e.special_anim_two.as_deref(), Some("W2"));
    assert_eq!(e.special_anim_two_damaged.as_deref(), Some("W2D"));
    assert_eq!(e.special_anim_three.as_deref(), Some("W3"));
    assert_eq!(e.special_anim_three_damaged.as_deref(), Some("W3D"));
    assert_eq!(e.special_anim_four.as_deref(), Some("W4"));
    assert_eq!(e.special_anim_four_damaged.as_deref(), Some("W4D"));
    let p = &s.buildings["PLANT"].econ;
    assert_eq!(p.special_anim_two, None);
    assert_eq!(p.special_anim_four_damaged, None);
}

// spec: S-0108 Import mapping and claim 20 (the depot slot model's names: production and special-1 to special-3, cut to 15 bytes, an empty damaged name taking the normal name; a slot with no name left out)
#[test]
fn slot_model_names_are_converted() {
    let art = "[MILLART]\nProductionAnim=ABCDEFGHIJKLMNOPQ\nSpecialAnim=S1\nSpecialAnimDamaged=S1D\nSpecialAnimThree=S3\nSpecialAnimThreeDamaged=\n";
    let (s, _) = import(BASE, art);
    let names = s.buildings["MILL"].econ.slot_names.clone().unwrap();
    let pair = |a: &str, b: &str| [a.to_owned(), b.to_owned()];
    assert_eq!(
        names.get("ProductionAnim"),
        Some(&pair("ABCDEFGHIJKLMNO", "ABCDEFGHIJKLMNO"))
    );
    assert_eq!(names.get("SpecialAnim"), Some(&pair("S1", "S1D")));
    assert_eq!(names.get("SpecialAnimThree"), Some(&pair("S3", "S3")));
    assert_eq!(names.get("SpecialAnimTwo"), None);
    assert_eq!(s.buildings["PLANT"].econ.slot_names, None);
}
