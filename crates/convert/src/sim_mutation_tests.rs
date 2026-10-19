//! Import tests of S-0611 (mutation) on invented data: every name and
//! number here is made up.

use glasnost_rules::sim::SimData;

use crate::inputs::{InputFile, Inputs};
use crate::{ImportOptions, convert};

fn file(name: &str, text: &str) -> InputFile {
    InputFile {
        name: name.into(),
        origin: "test".into(),
        bytes: text.as_bytes().to_vec(),
    }
}

fn sim(base: &str, art: &str) -> SimData {
    let inputs = Inputs {
        base: Some(file("rules.ini", base)),
        art: Some(file("art.ini", art)),
        ..Inputs::default()
    };
    let import = convert(&inputs, &ImportOptions::default());
    let r = glasnost_rules::resolve_ruleset(Some((import.tree, import.provenance)), &[])
        .unwrap_or_else(|e| panic!("{e}"));
    r.ruleset.sim
}

const RULES: &str = "[General]\nMutateExplosion=yes\nInfantryMutate=Bloom\nAnimToInfantry=Hulk,Imp\nIonBlast=Zap\n\
    [SpecialWeapons]\nMutateExplosionWarhead=Wave\nMutateWarhead=Touch\n\
    [AudioVisual]\nGeneticMutatorActivateSound=Hum\n\
    [Animations]\n1=Bloom\n2=Zap\n[InfantryTypes]\n1=Hulk\n2=Imp\n\
    [Warheads]\n1=Wave\n2=Touch\n[Touch]\nInfDeath=9\n";

/// spec: S-0611 Import mapping (MutateExplosion a boolean; MutateExplosionWarhead and MutateWarhead of [SpecialWeapons] warhead references; InfantryMutate and IonBlast animation references; AnimToInfantry an infantry type list; GeneticMutatorActivateSound a sound name; InfDeath kept as the integer 9)
#[test]
fn mutation_keys() {
    let s = sim(RULES, "[Bloom]\nMakeInfantry=1\n");
    let g = &s.general.superweapons;
    assert_eq!(g.mutate_explosion, Some(true));
    assert_eq!(g.mutate_explosion_warhead.as_deref(), Some("Wave"));
    assert_eq!(g.mutate_warhead.as_deref(), Some("Touch"));
    assert_eq!(g.infantry_mutate.as_deref(), Some("Bloom"));
    assert_eq!(
        g.anim_to_infantry.as_deref(),
        Some(&["Hulk".to_string(), "Imp".to_string()][..])
    );
    assert_eq!(g.ion_blast.as_deref(), Some("Zap"));
    assert_eq!(g.genetic_mutator_activate_sound.as_deref(), Some("Hum"));
    assert_eq!(s.warheads["Touch"].inf_death, Some(9));
    // spec: S-0611 Import mapping (MakeInfantry of the art section, an integer: the make-infantry index of F4)
    assert_eq!(s.animations["Bloom"].make_infantry, Some(1));
    assert_eq!(s.animations["Zap"].make_infantry, None);
}

/// spec: S-0611 Import mapping (absent keys stay absent: cell mode, no warheads, no conversion or launch animation, an empty list, no sound; MakeInfantry absent −1 in the simulation)
#[test]
fn absent_mutation_keys() {
    let s = sim("[Animations]\n1=Bloom\n", "[Bloom]\nRate=10\n");
    let g = &s.general.superweapons;
    assert!(g.mutate_explosion.is_none());
    assert!(g.mutate_explosion_warhead.is_none() && g.mutate_warhead.is_none());
    assert!(g.infantry_mutate.is_none() && g.ion_blast.is_none());
    assert!(g.anim_to_infantry.is_none());
    assert!(g.genetic_mutator_activate_sound.is_none());
    assert_eq!(s.animations["Bloom"].make_infantry, None);
}
