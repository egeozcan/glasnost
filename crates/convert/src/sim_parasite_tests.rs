//! Tests of the parasite fields of the `sim` section (S-0614 Import
//! mapping) on invented data (every name and number here is made up).

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

fn sim(base: &str) -> SimData {
    let inputs = Inputs {
        base: Some(file("rules.ini", base)),
        ..Inputs::default()
    };
    let import = convert(&inputs, &ImportOptions::default());
    glasnost_rules::resolve_ruleset(Some((import.tree, import.provenance)), &[])
        .unwrap_or_else(|e| panic!("{e}"))
        .ruleset
        .sim
}

fn texts(v: &Option<Vec<String>>) -> Vec<&str> {
    v.iter().flatten().map(String::as_str).collect()
}

const BASE: &str = "[InfantryTypes]\n1=Bug\n[VehicleTypes]\n1=Drone\n2=Tank\n[AircraftTypes]\n1=Kite\n\
                    [Warheads]\n1=Grip\n2=Plain\n\
                    [Drone]\nPrimary=Latch\nParasiteable=no\nReselectIfLimboed=yes\nWake=Foam\n\
                    [Bug]\nParasiteable=yes\n[Kite]\nReselectIfLimboed=no\n[Tank]\nPrimary=Gun\n\
                    [Latch]\nWarhead=Grip\nLimboLaunch=yes\nAnim=A0,none,A1\nROF=7\nDamage=3\n\
                    [Gun]\nWarhead=Plain\n\
                    [Grip]\nParasite=yes\nParalyzes=12\nCulling=yes\n[Plain]\nCellSpread=1\n\
                    [General]\nWake=Ripple\n[CombatDamage]\nDefaultSparkSystem=Spark\n";

/// spec: S-0614 Import mapping (Parasite, Paralyzes, Culling, LimboLaunch, Anim, Parasiteable,
/// ReselectIfLimboed, [General] Wake, [CombatDamage] DefaultSparkSystem; absent keys stay absent)
#[test]
fn parasite_fields() {
    let s = sim(BASE);
    let l = &s.weapons["Latch"];
    assert_eq!(l.limbo_launch, Some(true));
    assert_eq!(texts(&l.anim), ["A0", "A1"]);
    let g = &s.weapons["Gun"];
    assert_eq!((g.limbo_launch, g.anim.as_ref()), (None, None));
    let h = &s.warheads["Grip"];
    assert_eq!(
        (h.parasite, h.paralyzes, h.culling),
        (Some(true), Some(12), Some(true))
    );
    let p = &s.warheads["Plain"];
    assert_eq!((p.parasite, p.paralyzes, p.culling), (None, None, None));
    let d = &s.vehicles["Drone"];
    assert_eq!(
        (d.parasiteable, d.reselect_if_limboed),
        (Some(false), Some(true))
    );
    assert_eq!(s.infantry["Bug"].parasiteable, Some(true));
    assert_eq!(s.aircraft["Kite"].reselect_if_limboed, Some(false));
    assert_eq!(s.vehicles["Tank"].parasiteable, None);
    assert_eq!(s.general.wake.as_deref(), Some("Ripple"));
    assert_eq!(s.general.default_spark_system.as_deref(), Some("Spark"));
}

/// spec: S-0614 Import mapping (coverage outcomes of the parasite keys; the per-type wake of
/// S-0312 stays unconverted)
#[test]
fn parasite_coverage() {
    let inputs = Inputs {
        base: Some(file("rules.ini", BASE)),
        ..Inputs::default()
    };
    let import = convert(&inputs, &ImportOptions::default());
    let row = |cat: &str, key: &str| {
        import
            .coverage
            .iter()
            .find(|r| r.category == cat && r.key == key)
            .map(|r| (r.outcome, r.spec.clone()))
            .unwrap_or_else(|| panic!("{cat} {key}"))
    };
    use crate::coverage::Outcome::{Field, Unsupported};
    for (cat, key) in [
        ("warheads", "Parasite"),
        ("warheads", "Paralyzes"),
        ("warheads", "Culling"),
        ("weapons", "LimboLaunch"),
        ("weapons", "Anim"),
        ("vehicles", "Parasiteable"),
        ("infantry", "Parasiteable"),
        ("vehicles", "ReselectIfLimboed"),
        ("aircraft", "ReselectIfLimboed"),
        ("global:CombatDamage", "DefaultSparkSystem"),
    ] {
        assert_eq!(row(cat, key), (Field, "S-0614".into()), "{cat} {key}");
    }
    assert_eq!(
        row("global:General", "Wake"),
        (Field, "S-0614, S-0207, S-0312, S-0504, S-0503".into())
    );
    assert_eq!(row("vehicles", "Wake"), (Unsupported, "S-0312".into()));
}
