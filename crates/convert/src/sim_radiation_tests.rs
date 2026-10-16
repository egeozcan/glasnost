//! Tests of the radiation fields of the `sim` section (S-0525 Import
//! mapping) on invented data (every name and number here is made up).

use glasnost_rules::sim::SimData;

use crate::inputs::{InputFile, Inputs};
use crate::layers::Warning;
use crate::{ImportOptions, convert};

fn file(name: &str, text: &str) -> InputFile {
    InputFile {
        name: name.into(),
        origin: "test".into(),
        bytes: text.as_bytes().to_vec(),
    }
}

fn import(base: &str, map: Option<&str>) -> crate::Import {
    let inputs = Inputs {
        base: Some(file("rules.ini", base)),
        map: map.map(|m| file("map.ini", m)),
        ..Inputs::default()
    };
    convert(&inputs, &ImportOptions::default())
}

fn sim(base: &str) -> (SimData, Vec<Warning>) {
    let import = import(base, None);
    let r = glasnost_rules::resolve_ruleset(Some((import.tree, import.provenance)), &[])
        .unwrap_or_else(|e| panic!("{e}"));
    (r.ruleset.sim, import.warnings)
}

fn warned(w: &[Warning], text: &str) -> bool {
    w.iter().any(|w| w.message.contains(text))
}

const FULL: &str = "[Radiation]\nRadDurationMultiple=2\nRadApplicationDelay=15\nRadLevelMax=500\n\
    RadLevelDelay=60\nRadLightDelay=90\nRadLevelFactor=0.0625\nRadLightFactor=50%\n\
    RadTintFactor=1.0\nRadColor=0,255,0\nRadSiteWarhead=SiteW\n[Warheads]\n1=SiteW\n[SiteW]\nVerses=100%\n";

/// spec: S-0525 Import mapping (the [Radiation] keys: integers, three decimals held as doubles, a byte triple, the warhead name)
#[test]
fn radiation_keys() {
    let (s, w) = sim(FULL);
    let g = &s.general;
    assert_eq!(
        (
            g.rad_duration_multiple,
            g.rad_application_delay,
            g.rad_level_max,
            g.rad_level_delay,
            g.rad_light_delay
        ),
        (Some(2), Some(15), Some(500), Some(60), Some(90))
    );
    let text = |d: &Option<glasnost_rules::sim::Decimal>| d.as_ref().map(|d| d.0.clone());
    assert_eq!(text(&g.rad_level_factor).as_deref(), Some("625e-4"));
    assert_eq!(text(&g.rad_light_factor).as_deref(), Some("50%"));
    assert_eq!(text(&g.rad_tint_factor).as_deref(), Some("10e-1"));
    assert_eq!(g.rad_color, Some([0, 255, 0]));
    assert_eq!(g.rad_site_warhead.as_deref(), Some("SiteW"));
    assert!(!warned(&w, "[Radiation]"), "{w:?}");
}

/// spec: S-0525 Import mapping (absent keys stay absent: the engine defaults of 0 apply in the simulation)
#[test]
fn absent_radiation_keys() {
    let (s, w) = sim("[General]\nX=1\n");
    let g = &s.general;
    assert!(
        g.rad_duration_multiple.is_none()
            && g.rad_application_delay.is_none()
            && g.rad_color.is_none()
            && g.rad_site_warhead.is_none()
    );
    // The missing radiation-site warhead is warned about (Edge cases).
    assert!(
        warned(&w, "RadSiteWarhead is given by no rules layer"),
        "{w:?}"
    );
}

/// spec: S-0525 Import mapping and S-0006 (a later layer that omits a key keeps the earlier value; a later colour with unreadable components keeps the earlier ones)
#[test]
fn radiation_layers() {
    let i = import(FULL, Some("[Radiation]\nRadLevelMax=900\nRadColor=7,x\n"));
    let r = glasnost_rules::resolve_ruleset(Some((i.tree, i.provenance)), &[])
        .unwrap_or_else(|e| panic!("{e}"));
    let g = &r.ruleset.sim.general;
    assert_eq!(g.rad_level_max, Some(900));
    assert_eq!(g.rad_application_delay, Some(15));
    assert_eq!(g.rad_color, Some([7, 255, 0]));
    assert!(warned(&i.warnings, "RadColor"), "{:?}", i.warnings);
}

/// spec: S-0525 claim 16 (a delay of 0 or less and a duration multiple of 0 or less: the importer warns)
#[test]
fn bad_delays_warn() {
    for (key, text) in [
        ("RadApplicationDelay", "RadApplicationDelay 0"),
        ("RadLevelDelay", "RadLevelDelay -3"),
        ("RadLightDelay", "RadLightDelay 0"),
        ("RadDurationMultiple", "RadDurationMultiple 0"),
    ] {
        let value = if key == "RadLevelDelay" { "-3" } else { "0" };
        let (_, w) = sim(&format!("[Radiation]\n{key}={value}\n"));
        assert!(warned(&w, &format!("[Radiation] {text}")), "{key}: {w:?}");
    }
}

/// spec: S-0525 Edge cases and claim 16 (a field duration with zero light steps, or zero level steps: the importer warns)
#[test]
fn zero_step_durations_warn() {
    let rules = format!(
        "{FULL}[InfantryTypes]\n1=Sh\n[Sh]\nPrimary=Short\n[Short]\nRadLevel=10\nWarhead=SiteW\n"
    );
    let (_, w) = sim(&rules);
    assert!(warned(&w, "shorter than the light delay"), "{w:?}");
    assert!(warned(&w, "has no level step"), "{w:?}");
}

/// spec: S-0525 Import mapping (the [Radiation] keys are converted fields of S-0525)
#[test]
fn radiation_coverage() {
    let i = import(FULL, None);
    let row = |key: &str| {
        i.coverage
            .iter()
            .find(|r| r.category == "global:Radiation" && r.key == key)
            .map(|r| (r.outcome, r.spec.clone()))
            .unwrap_or_else(|| panic!("{key}"))
    };
    use crate::coverage::Outcome::Field;
    for k in [
        "RadLevelMax",
        "RadColor",
        "RadLevelFactor",
        "RadSiteWarhead",
    ] {
        assert_eq!(row(k), (Field, "S-0525".into()), "{k}");
    }
}
