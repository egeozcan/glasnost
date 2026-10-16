//! Tests of the linked-weapon fields of the `sim` section (S-0518 Import
//! mapping, F4) on invented data (every name and number here is made up).

use glasnost_rules::sim::SimData;

use crate::inputs::{InputFile, Inputs};
use crate::layers::LayerQuirks;
use crate::{ImportOptions, convert};

fn file(name: &str, text: &str) -> InputFile {
    InputFile {
        name: name.into(),
        origin: "test".into(),
        bytes: text.as_bytes().to_vec(),
    }
}

fn sim(base: &str, map: Option<&str>, q: LayerQuirks) -> (SimData, crate::Import) {
    let inputs = Inputs {
        base: Some(file("rules.ini", base)),
        map: map.map(|m| file("map.ini", m)),
        ..Inputs::default()
    };
    let opts = ImportOptions {
        quirks: q,
        ..ImportOptions::default()
    };
    let import = convert(&inputs, &opts);
    let r = glasnost_rules::resolve_ruleset(
        Some((import.tree.clone(), import.provenance.clone())),
        &[],
    )
    .unwrap_or_else(|e| panic!("{e}"));
    (r.ruleset.sim, import)
}

const BASE: &str = "[General]\nPrismType=Beam\nPrismSupportModifier=125%\nPrismSupportMax=4\n\
    PrismSupportDelay=90\nPrismSupportDuration=20\n\
    [BuildingTypes]\n1=Beam\n2=Coil\n[Beam]\nStrength=10\n[Coil]\nOverpowerable=yes\n";

/// spec: S-0518 Import mapping and F4 (the prism keys: the type name, the cap, the rearm and the duration as integers; the modifier "125%" becomes the held value 125; Overpowerable a boolean of building types)
#[test]
fn prism_keys() {
    let (s, _) = sim(BASE, None, LayerQuirks::default());
    let g = &s.general;
    assert_eq!(g.prism_type.as_deref(), Some("Beam"));
    assert_eq!(g.prism_support_modifier, Some(125));
    assert_eq!(
        (
            g.prism_support_max,
            g.prism_support_delay,
            g.prism_support_duration
        ),
        (Some(4), Some(90), Some(20))
    );
    assert_eq!(s.buildings["Coil"].overpowerable, Some(true));
    assert_eq!(s.buildings["Beam"].overpowerable, None);
}

/// spec: S-0518 F4 (a bare number is scaled by 100 as it stands: 125 becomes 12500; "10%" gives 9 because the percent product falls just below the exact value)
#[test]
fn f4_examples() {
    let (s, _) = sim(
        "[General]\nPrismSupportModifier=125\n",
        None,
        LayerQuirks::default(),
    );
    assert_eq!(s.general.prism_support_modifier, Some(12500));
    let (s, _) = sim(
        "[General]\nPrismSupportModifier=10%\n",
        None,
        LayerQuirks::default(),
    );
    assert_eq!(s.general.prism_support_modifier, Some(9));
}

/// spec: S-0518 claim 16 and F4 (a later layer with the general section but without the key rescales the held modifier by 100; quirks.prism_modifier_reread_scale off leaves it)
#[test]
fn a_general_section_rescales_the_held_modifier() {
    let map = "[General]\nPrismSupportMax=2\n";
    let (s, _) = sim(BASE, Some(map), LayerQuirks::default());
    assert_eq!(s.general.prism_support_modifier, Some(12500));
    let q = LayerQuirks {
        prism_modifier_reread_scale: false,
        ..LayerQuirks::default()
    };
    let (s, _) = sim(BASE, Some(map), q);
    assert_eq!(s.general.prism_support_modifier, Some(125));
    // A layer without a general section does not rescale it.
    let (s, _) = sim(BASE, Some("[Other]\nX=1\n"), LayerQuirks::default());
    assert_eq!(s.general.prism_support_modifier, Some(125));
    // A layer that gives the key sets it by F4.
    let (s, _) = sim(
        BASE,
        Some("[General]\nPrismSupportModifier=50%\n"),
        LayerQuirks::default(),
    );
    assert_eq!(s.general.prism_support_modifier, Some(50));
}

/// spec: S-0518 Import mapping (the keys are converted fields of S-0518)
#[test]
fn prism_coverage() {
    let (_, i) = sim(BASE, None, LayerQuirks::default());
    let row = |key: &str| {
        i.coverage
            .iter()
            .find(|r| r.category == "global:General" && r.key == key)
            .map(|r| (r.outcome, r.spec.clone()))
            .unwrap_or_else(|| panic!("{key}"))
    };
    use crate::coverage::Outcome::Field;
    for k in ["PrismType", "PrismSupportModifier", "PrismSupportMax"] {
        assert_eq!(row(k), (Field, "S-0518".into()), "{k}");
    }
}
