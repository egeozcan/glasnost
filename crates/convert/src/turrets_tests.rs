//! Tests of the turret table and the fixed-name charge-turret rule (S-0105
//! Terms, claim 43, Import mapping *Turret table pairs* and Edge cases) on
//! invented data. Every type name except the spec's fixed type name is made
//! up, and so is every number.

use std::collections::BTreeMap;

use glasnost_rules::sim::SimData;

use crate::inputs::{InputFile, Inputs};
use crate::layers::Warning;
use crate::sim::SimQuirks;
use crate::turrets::{EMPTY, TurretTable};
use crate::{ImportOptions, convert};

fn file(name: &str, text: &str) -> InputFile {
    InputFile {
        name: name.into(),
        origin: "test".into(),
        bytes: text.as_bytes().to_vec(),
    }
}

/// Converts base rules, a game-mode layer and a map with the switches
/// `quirks`; returns the turret tables by type name, the typed `sim` data
/// and the warnings.
fn import(
    base: &str,
    mode: &str,
    map: &str,
    quirks: SimQuirks,
) -> (BTreeMap<String, TurretTable>, SimData, Vec<Warning>) {
    let inputs = Inputs {
        base: Some(file("rules.ini", base)),
        mode: Some(file("mode.ini", mode)),
        map: Some(file("test.map", map)),
        art: Some(file("art.ini", "")),
        ..Inputs::default()
    };
    let options = ImportOptions {
        sim: quirks,
        ..ImportOptions::default()
    };
    let import = convert(&inputs, &options);
    let names = &import.outcome.registries[&crate::kinds::Kind::Vehicles];
    let tables = import
        .turret_tables
        .iter()
        .map(|(i, t)| (String::from_utf8_lossy(&names[*i].name).into_owned(), *t))
        .collect();
    let r = glasnost_rules::resolve_ruleset(Some((import.tree, import.provenance)), &[])
        .unwrap_or_else(|e| panic!("{e}"));
    (tables, r.ruleset.sim, import.warnings)
}

fn on() -> SimQuirks {
    SimQuirks::default()
}

/// MS-33's subject pairs (S-0105 Verification): every pair with a weapon
/// number, Normal 0 → 2, Repair 1 → 0, MachineGun 2 → 1, Flak 3 → 5, and
/// the other 13 stems weapon numbers 4 … 16 to turret 0, in stem order.
fn subject_pairs() -> String {
    let mut s = String::from(
        "NormalTurretWeapon=0\nNormalTurretIndex=2\nRepairTurretWeapon=1\nRepairTurretIndex=0\n\
         MachineGunTurretWeapon=2\nMachineGunTurretIndex=1\nFlakTurretWeapon=3\nFlakTurretIndex=5\n",
    );
    for (w, (stem, _)) in crate::turrets::STEMS[4..].iter().enumerate() {
        s.push_str(&format!(
            "{stem}TurretWeapon={}\n{stem}TurretIndex=0\n",
            w + 4
        ));
    }
    s
}

fn table(entries: &[(usize, i32)]) -> TurretTable {
    let mut t = EMPTY;
    for &(w, k) in entries {
        t[w] = Some(k);
    }
    t
}

/// The subject's expected table: 0 → 2, 1 → 0, 2 → 1, 3 → 5, 4 … 16 → 0.
fn subject_table() -> TurretTable {
    let mut t = table(&[(0, 2), (1, 0), (2, 1), (3, 5)]);
    for e in &mut t[4..17] {
        *e = Some(0);
    }
    t
}

const BASE: &str = "[VehicleTypes]\n1=FV\n2=Carrier\n[FV]\nTurret=yes\nTurretCount=3\nGunner=yes\n\
                    [Carrier]\nTurret=yes\nTurretCount=3\nGunner=yes\n";

/// spec: S-0105 Import mapping (Turret table pairs: each pair sets the entry for its weapon
/// number; read only for the fixed type name, in either letter case; −1 means none; indices
/// outside −1 … N − 1 are kept) and claim 43; MS-33 Setup (subject and control) and part a
#[test]
fn the_fixed_type_name_reads_its_pairs() {
    let map = format!("[FV]\n{}[Carrier]\n{}", subject_pairs(), subject_pairs());
    let (tables, s, w) = import(BASE, "", &map, on());
    assert_eq!(tables.get("FV"), Some(&subject_table()));
    // spec: S-0105 claim 43 (quirks.turret_table_fixed_type_only: every other type keeps an all-none table)
    assert_eq!(
        tables.get("Carrier"),
        None,
        "the control's table is all none"
    );
    // The base layer's section has no pair keys, so its last pair (Guardian, default index 0)
    // has no weapon number and writes the flag off; the map's pairs all have one.
    assert_eq!(s.vehicles["FV"].combat.is_charge_turret, Some(false));
    assert_eq!(s.vehicles["Carrier"].combat.is_charge_turret, None);
    // spec: S-0105 Import mapping (an index outside −1 … N − 1 is kept; the importer warns)
    assert!(w.iter().any(|w| w.message.contains("FlakTurretIndex 5")));
    // In either letter case.
    let lower = BASE.replace("FV", "fv");
    let (tables, _, _) = import(&lower, "", &map.replace("FV", "fv"), on());
    assert_eq!(tables.get("fv"), Some(&subject_table()));
    // −1 means none.
    let map = "[FV]\nNormalTurretWeapon=4\nNormalTurretIndex=-1\nRepairTurretWeapon=4\nRepairTurretIndex=2\n\
               MachineGunTurretWeapon=4\nMachineGunTurretIndex=-1\n";
    let (tables, _, _) = import(BASE, "", map, on());
    assert_eq!(tables.get("FV"), None, "a later pair setting none wins");
}

/// spec: S-0105 Edge cases (quirks.turret_table_fixed_type_only off: every multi-turret type
/// reads the same pairs from its own section, by the same rules; the charge-turret rule stays
/// with the fixed type name); MS-33 part d (engine conformance)
#[test]
fn every_type_reads_its_pairs_with_the_switch_off() {
    let off = SimQuirks {
        turret_table_fixed_type_only: false,
        ..on()
    };
    let map = format!("[FV]\n{}[Carrier]\n{}", subject_pairs(), subject_pairs());
    let (tables, _, _) = import(BASE, "", &map, off);
    assert_eq!(tables.get("FV"), Some(&subject_table()));
    assert_eq!(tables.get("Carrier"), Some(&subject_table()));
    // A pair with no weapon number turns no other type's charge-turret flag on (MS-33 part g's
    // pairs under the other name).
    let g = subject_pairs().replace("MachineGunTurretWeapon=2\n", "");
    let (tables, s, _) = import(BASE, "", &format!("[Carrier]\n{g}"), off);
    assert_eq!(s.vehicles["Carrier"].combat.is_charge_turret, None);
    let mut want = subject_table();
    want[2] = None;
    assert_eq!(tables.get("Carrier"), Some(&want));
}

/// spec: S-0105 Import mapping (pairs apply in the order listed, so a later pair naming the same
/// weapon number wins; the table is never cleared across layers; an absent index key takes its
/// stem's default in every layer, never an earlier layer's index); MS-33 part h, both runs
#[test]
fn later_pairs_win_and_entries_stay_across_layers() {
    let base = format!("{BASE}[FV]\nNormalTurretWeapon=17\nNormalTurretIndex=1\n");
    // First run: Pistol maps weapon 3 to turret 1 instead of weapon 4 to turret 0.
    let first = subject_pairs().replace(
        "PistolTurretWeapon=4\nPistolTurretIndex=0\n",
        "PistolTurretWeapon=3\nPistolTurretIndex=1\n",
    );
    let (tables, _, _) = import(&base, "", &format!("[FV]\n{first}"), on());
    let mut want = subject_table();
    want[3] = Some(1);
    want[4] = None;
    want[17] = Some(1);
    assert_eq!(tables["FV"], want, "the later pair for 3 wins; 17 is kept");
    // Second run: the override's Normal pair gives weapon number 17 and no index key.
    let second = subject_pairs().replace(
        "NormalTurretWeapon=0\nNormalTurretIndex=2\n",
        "NormalTurretWeapon=17\n",
    );
    let (tables, _, _) = import(&base, "", &format!("[FV]\n{second}"), on());
    let mut want = subject_table();
    want[0] = None;
    want[17] = Some(0);
    assert_eq!(tables["FV"], want, "the stem's default, not the base's 1");
    // The other stems' defaults: Repair 1, MachineGun 2, Flak 3, the rest 0.
    let map = "[FV]\nRepairTurretWeapon=5\nMachineGunTurretWeapon=6\nFlakTurretWeapon=7\n\
               CowTurretWeapon=8\n";
    let (tables, _, _) = import(BASE, "", map, on());
    assert_eq!(
        tables["FV"],
        table(&[(5, 1), (6, 2), (7, 3), (8, 0)]),
        "absent index keys"
    );
}

/// spec: S-0105 Import mapping (a weapon number outside 0 … 17 other than −1: our engine ignores
/// the pair and the importer warns, design)
#[test]
fn out_of_range_weapon_numbers_are_ignored_with_a_warning() {
    let map = "[FV]\nNormalTurretWeapon=18\nNormalTurretIndex=1\nRepairTurretWeapon=-2\n\
               RepairTurretIndex=2\nFlakTurretWeapon=2\nFlakTurretIndex=-3\n";
    let (tables, _, w) = import(BASE, "", map, on());
    assert_eq!(tables["FV"], table(&[(2, -3)]), "kept, outside −1 … N − 1");
    for text in [
        "NormalTurretWeapon 18 is outside 0 … 17",
        "RepairTurretWeapon -2 is outside 0 … 17",
        "FlakTurretIndex -3 is outside −1 … 2",
    ] {
        assert!(w.iter().any(|w| w.message.contains(text)), "{text}");
    }
}

/// spec: S-0105 Edge cases (quirks.turret_pair_no_weapon_sets_charge_flag: in each layer that
/// has the section, the last pair with no weapon number overrides that layer's charge-turret
/// key, on exactly when its turret index is not 0 and not a multiple of 256; off, such a pair
/// changes nothing); MS-33 part g and its mirror run
#[test]
fn a_pair_with_no_weapon_number_sets_the_charge_flag() {
    let charge = |base: &str, map: &str, q: SimQuirks| {
        import(base, "", map, q).1.vehicles["FV"]
            .combat
            .is_charge_turret
    };
    // Part g: the override omits only the MachineGun pair's weapon key (turret index 1).
    let g = subject_pairs().replace("MachineGunTurretWeapon=2\n", "");
    assert_eq!(charge(BASE, &format!("[FV]\n{g}"), on()), Some(true));
    // Its mirror: the charge flag on, only the Repair pair's weapon key omitted (index 0).
    let mirror = subject_pairs().replace("RepairTurretWeapon=1\n", "");
    let map = format!("[FV]\nIsChargeTurret=yes\n{mirror}");
    assert_eq!(charge(BASE, &map, on()), Some(false));
    // Switch off: neither pair changes the flag.
    let off = SimQuirks {
        turret_pair_no_weapon_sets_charge_flag: false,
        ..on()
    };
    assert_eq!(
        charge(BASE, &format!("[FV]\n{g}"), off),
        None,
        "the key is never given"
    );
    assert_eq!(charge(BASE, &map, off), Some(true));
    // A non-zero multiple of 256 turns it off; another index turns it on; −1 is a weapon number
    // of none, like an absent key.
    for (index, want) in [(512, false), (-256, false), (257, true), (-1, true)] {
        let m = subject_pairs().replace(
            "MachineGunTurretWeapon=2\nMachineGunTurretIndex=1\n",
            &format!("MachineGunTurretWeapon=-1\nMachineGunTurretIndex={index}\n"),
        );
        assert_eq!(
            charge(BASE, &format!("[FV]\nIsChargeTurret=yes\n{m}"), on()),
            Some(want),
            "index {index}"
        );
    }
    // A section with no pair keys: every pair has no weapon number, the last one (Guardian,
    // default index 0) decides, so the layer's own charge-turret key is overridden.
    let bare = format!("{BASE}[FV]\nIsChargeTurret=yes\n");
    assert_eq!(charge(&bare, "", on()), Some(false));
    assert_eq!(charge(&bare, "", off), Some(true));
    // With several, the last in stem order decides: Repair (index 1 by default) has no weapon
    // number, every later pair has one.
    let only_repair = subject_pairs().replace("RepairTurretWeapon=1\nRepairTurretIndex=0\n", "");
    assert_eq!(
        charge(BASE, &format!("[FV]\n{only_repair}"), on()),
        Some(true)
    );
    // A later layer without the section changes nothing; a later layer with it decides again.
    let mode = format!("[FV]\n{g}");
    assert_eq!(
        import(BASE, &mode, "[General]\n", on()).1.vehicles["FV"]
            .combat
            .is_charge_turret,
        Some(true)
    );
    assert_eq!(
        import(BASE, &mode, &format!("[FV]\n{}", subject_pairs()), on())
            .1
            .vehicles["FV"]
            .combat
            .is_charge_turret,
        Some(true),
        "no pair without a weapon number and no key of its own: the held flag stays"
    );
}
