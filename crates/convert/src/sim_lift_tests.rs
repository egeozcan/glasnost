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
    [Warheads]\n1=Pull\n2=Odd\n3=Plain\n4=Bent\n5=Blank\n6=Bare\n7=Padded\n8=Quiet\n\
    [Pull]\nIsLocomotor=yes\nLocomotor={92612C46-F71F-11d1-AC9F-006008055BB5}\n\
    [Odd]\nIsLocomotor=yes\nLocomotor={00000000-0000-0000-0000-000000000001}\n\
    [Bent]\nIsLocomotor=yes\nLocomotor=hover\n\
    [Blank]\nIsLocomotor=yes\nLocomotor=\n\
    [Bare]\nIsLocomotor=yes\nLocomotor=92612C46F71F11d1AC9F006008055BB5\n\
    [Padded]\nIsLocomotor=yes\nLocomotor=  {92612C46-F71F-11d1-AC9F-006008055BB5}  \n\
    [Quiet]\nLocomotor=hover\n\
    [Plain]\n";

// spec: S-0607 Import mapping and claim 22 (FallingDamageMultiplier a decimal; CurrentStrengthDamage a boolean; warhead Locomotor a locomotor class identifier read as S-0207 claim 14, blanks trimmed; an unknown one kept as unknown with a warning; an empty or malformed value ignored as if absent, with a warning on a seizing warhead only)
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
    // Blanks around a class identifier are removed.
    assert_eq!(s.warheads["Padded"].locomotor.as_deref(), Some("jumpjet"));
    assert!(!warnings.iter().any(|m| m.contains("warheads Padded")));
    // A bare name and digits without braces are ignored as if absent (none
    // in force here), each with a warning on a seizing warhead.
    for name in ["Bent", "Bare"] {
        assert_eq!(s.warheads[name].locomotor, None, "{name}");
        assert!(
            warnings
                .iter()
                .any(|m| m.contains(&format!("warheads {name} Locomotor"))
                    && m.contains("not a class identifier")
                    && m.contains("S-0607 claim 22")),
            "{name}: {warnings:?}"
        );
    }
    // An empty value is no entry (S-0001 claim 8): absent, and no warning
    // (S-0607 claim 22, revision 7).
    assert_eq!(s.warheads["Blank"].locomotor, None);
    assert!(!warnings.iter().any(|m| m.contains("warheads Blank")));
    // Without the seizing flag the value is unused and does not warn.
    assert_eq!(s.warheads["Quiet"].locomotor, None);
    assert!(!warnings.iter().any(|m| m.contains("warheads Quiet")));
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

// spec: S-0607 claim 22 and Import mapping (an empty or malformed value in a later layer is ignored as if absent: the locomotor in force from the earlier layer stays; the malformed one warns on a seizing warhead, the empty one is no entry under S-0001 claim 8 and does not warn, revision 7)
#[test]
fn lift_locomotor_malformed_keeps_the_earlier_layer() {
    let inputs = Inputs {
        base: Some(InputFile {
            name: "rules.ini".into(),
            origin: "test".into(),
            bytes: b"[Warheads]\n1=Grab\n2=Hold\n[Grab]\nIsLocomotor=yes\nLocomotor={4A582741-9839-11d1-B709-00A024DDAFD1}\n[Hold]\nIsLocomotor=yes\nLocomotor={4A582741-9839-11d1-B709-00A024DDAFD1}\n".to_vec(),
        }),
        mode: Some(InputFile {
            name: "mode.ini".into(),
            origin: "test".into(),
            bytes: b"[Grab]\nLocomotor=\n[Hold]\nLocomotor=hover\n".to_vec(),
        }),
        ..Inputs::default()
    };
    let import = convert(&inputs, &ImportOptions::default());
    let warnings: Vec<String> = import.warnings.iter().map(|w| w.message.clone()).collect();
    let s = glasnost_rules::resolve_ruleset(
        Some((import.tree.clone(), import.provenance.clone())),
        &[],
    )
    .unwrap_or_else(|e| panic!("{e}"))
    .ruleset
    .sim;
    assert_eq!(s.warheads["Grab"].locomotor.as_deref(), Some("drive"));
    assert_eq!(s.warheads["Hold"].locomotor.as_deref(), Some("drive"));
    assert!(
        warnings
            .iter()
            .any(|m| m.contains("warheads Hold Locomotor") && m.contains("not a class identifier")),
        "{warnings:?}"
    );
    assert!(
        !warnings
            .iter()
            .any(|m| m.contains("warheads Grab Locomotor"))
    );
}
