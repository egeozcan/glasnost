//! Tests of the magnetic-lift fields of the `sim` section (S-0607 Import
//! mapping) on invented data (every name and number here is made up).

use glasnost_rules::sim::SimData;

use crate::inputs::{InputFile, Inputs};
use crate::layers::LayerQuirks;
use crate::{ImportOptions, convert};

fn sim(base: &str) -> SimData {
    imported(base).0
}

/// The typed `sim` data and the import's warning messages.
fn imported(base: &str) -> (SimData, Vec<String>) {
    let inputs = Inputs {
        base: Some(InputFile {
            name: "rules.ini".into(),
            origin: "test".into(),
            bytes: base.as_bytes().to_vec(),
        }),
        ..Inputs::default()
    };
    let opts = ImportOptions {
        quirks: LayerQuirks::default(),
        ..ImportOptions::default()
    };
    let import = convert(&inputs, &opts);
    let warnings = import.warnings.iter().map(|w| w.message.clone()).collect();
    let s = glasnost_rules::resolve_ruleset(
        Some((import.tree.clone(), import.provenance.clone())),
        &[],
    )
    .unwrap_or_else(|e| panic!("{e}"))
    .ruleset
    .sim;
    (s, warnings)
}

const BASE: &str = "[CombatDamage]\nFallingDamageMultiplier=0.3\nCurrentStrengthDamage=yes\n\
    [Warheads]\n1=Pull\n2=Odd\n3=Plain\n4=Bent\n\
    [Pull]\nIsLocomotor=yes\nLocomotor={92612C46-F71F-11d1-AC9F-006008055BB5}\n\
    [Odd]\nIsLocomotor=yes\nLocomotor={00000000-0000-0000-0000-000000000001}\n\
    [Bent]\nIsLocomotor=yes\nLocomotor=hover\n\
    [Plain]\n";

// spec: S-0607 Import mapping and claim 22 (FallingDamageMultiplier a decimal; CurrentStrengthDamage a boolean; warhead Locomotor a locomotor class identifier, an unknown one kept as unknown with a warning); Q-1321 interim (a value of another form keeps the value in force)
#[test]
fn lift_fields() {
    let (s, warnings) = imported(BASE);
    let g = &s.general;
    assert_eq!(
        g.falling_damage_multiplier.as_ref().map(|d| d.0.as_str()),
        Some("3e-1")
    );
    assert_eq!(g.current_strength_damage, Some(true));
    assert_eq!(s.warheads["Pull"].locomotor.as_deref(), Some("jumpjet"));
    assert_eq!(s.warheads["Pull"].is_locomotor, Some(true));
    assert_eq!(s.warheads["Odd"].locomotor.as_deref(), Some("unknown"));
    assert!(
        warnings
            .iter()
            .any(|m| m.contains("warheads Odd Locomotor") && m.contains("S-0607 claim 22")),
        "{warnings:?}"
    );
    // A value that is not a class identifier keeps the value in force (none here).
    assert_eq!(s.warheads["Bent"].locomotor, None);
    assert!(!warnings.iter().any(|m| m.contains("warheads Bent")));
    // No death weapon: claim 18's design rule, with a warning.
    assert!(
        warnings
            .iter()
            .any(|m| m.contains("DeathWeapon") && m.contains("S-0607 claim 18")),
        "{warnings:?}"
    );
}

// spec: S-0607 claim 18 (a configured death weapon: no warning)
#[test]
fn lift_death_weapon_given() {
    let (_, warnings) = imported("[CombatDamage]\nDeathWeapon=Doom\n[Weapons]\n1=Doom\n[Doom]\n");
    assert!(
        !warnings.iter().any(|m| m.contains("S-0607 claim 18")),
        "{warnings:?}"
    );
}

// spec: S-0607 Import mapping (absent keys stay absent so that the engine defaults apply)
#[test]
fn lift_fields_absent() {
    let s = sim("[CombatDamage]\n[Warheads]\n1=Plain\n[Plain]\n");
    assert!(s.general.falling_damage_multiplier.is_none());
    assert!(s.general.current_strength_damage.is_none());
    assert!(s.warheads["Plain"].locomotor.is_none());
}
