//! Tests of the chrono-weapon fields of the `sim` section (S-0514 Import
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

fn sim(base: &str) -> (SimData, crate::Import) {
    let inputs = Inputs {
        base: Some(file("rules.ini", base)),
        ..Inputs::default()
    };
    let import = convert(&inputs, &ImportOptions::default());
    let r = glasnost_rules::resolve_ruleset(
        Some((import.tree.clone(), import.provenance.clone())),
        &[],
    )
    .unwrap_or_else(|e| panic!("{e}"));
    (r.ruleset.sim, import)
}

const RULES: &str = "[General]\nWarpAway=Poof\nChronoSparkle1=Glint\n\
    [CombatDamage]\nOpenToppedWarpDistance=3\n\
    [Animations]\n1=Poof\n2=Glint\n[Poof]\nRate=10\n[Glint]\nRate=10\n\
    [VehicleTypes]\n1=Tank\n2=Ghost\n[Tank]\nWarpable=no\n[Ghost]\nStrength=10\n\
    [BuildingTypes]\n1=Hut\n[Hut]\nWarpable=yes\n\
    [Warheads]\n1=Tick\n[Tick]\nTemporal=yes\n";

/// spec: S-0514 Import mapping (WarpAway and ChronoSparkle1 animation names, OpenToppedWarpDistance an integer of cells, Warpable a boolean of object types, Temporal a warhead boolean)
#[test]
fn chrono_keys() {
    let (s, _) = sim(RULES);
    let g = &s.general;
    assert_eq!(g.warp_away.as_deref(), Some("Poof"));
    assert_eq!(g.chrono_sparkle.as_deref(), Some("Glint"));
    assert_eq!(g.open_topped_warp_distance, Some(3));
    assert_eq!(s.vehicles["Tank"].warpable, Some(false));
    assert_eq!(s.vehicles["Ghost"].warpable, None);
    assert_eq!(s.buildings["Hut"].warpable, Some(true));
    assert_eq!(s.warheads["Tick"].temporal, Some(true));
}

/// spec: S-0514 Import mapping (absent keys stay absent: no animations, warp distance 5 and erasable types by default in the simulation)
#[test]
fn absent_chrono_keys() {
    let (s, _) = sim("[General]\nX=1\n");
    let g = &s.general;
    assert!(
        g.warp_away.is_none()
            && g.chrono_sparkle.is_none()
            && g.open_topped_warp_distance.is_none()
    );
}

/// spec: S-0514 Import mapping (the keys are converted fields of S-0514)
#[test]
fn chrono_coverage() {
    let (_, i) = sim(RULES);
    let row = |cat: &str, key: &str| {
        i.coverage
            .iter()
            .find(|r| r.category == cat && r.key == key)
            .map(|r| (r.outcome, r.spec.clone()))
            .unwrap_or_else(|| panic!("{cat} {key}"))
    };
    use crate::coverage::Outcome::Field;
    assert_eq!(row("global:General", "WarpAway"), (Field, "S-0514".into()));
    assert_eq!(
        row("global:CombatDamage", "OpenToppedWarpDistance"),
        (Field, "S-0514".into())
    );
}
