//! Tests of the hovering fields of the `sim` section (S-0503 Import mapping)
//! on invented data (every name and number here is made up).

use glasnost_rules::sim::{Locomotor, MovementZone, SimData};

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

/// The hovering locomotor's class identifier (S-0207 table T3).
const HOVER: &str = "{4A582742-9839-11d1-B709-00A024DDAFD1}";

fn base(general: &str, audio: &str) -> String {
    format!(
        "[VehicleTypes]\n1=Skimmer\n[Skimmer]\nLocomotor={HOVER}\nMovementZone=Amphibious\n\
         SpeedType=Hover\n[General]\n{general}[AudioVisual]\n{audio}"
    )
}

/// spec: S-0503 Import mapping (HoverHeight an integer; HoverBob, HoverBoost,
/// HoverAcceleration, HoverBrake and HoverDampen decimals; Gravity an integer;
/// the type's Locomotor and movement zone)
#[test]
fn hover_fields() {
    let (s, w) = sim(&base(
        "HoverHeight=90\nHoverBob=12.5\nHoverBoost=1.75\nHoverAcceleration=0.25\n\
         HoverBrake=50%\nHoverDampen=0.5\n",
        "Gravity=4\n",
    ));
    let g = &s.general;
    assert_eq!(g.hover_height, Some(90));
    let text = |d: &Option<glasnost_rules::sim::Decimal>| d.as_ref().map(|d| d.0.clone());
    assert_eq!(text(&g.hover_bob).as_deref(), Some("125e-1"));
    assert_eq!(text(&g.hover_boost).as_deref(), Some("175e-2"));
    assert_eq!(text(&g.hover_acceleration).as_deref(), Some("25e-2"));
    assert!(g.hover_brake.is_some());
    assert_eq!(text(&g.hover_dampen).as_deref(), Some("5e-1"));
    assert_eq!(g.gravity, Some(4));
    let v = &s.vehicles["Skimmer"];
    assert_eq!(v.locomotor, Some(Locomotor::Hover));
    assert_eq!(v.movement_zone, Some(MovementZone::Amphibious));
    assert!(!warned(&w, "HoverBob"), "{w:?}");
}

/// spec: S-0503 Import mapping (absent keys stay absent: the engine defaults apply in the simulation)
#[test]
fn absent_hover_fields() {
    let (s, _) = sim(&base("X=1\n", "X=1\n"));
    let g = &s.general;
    assert_eq!(
        (
            g.hover_height,
            g.hover_bob.is_none(),
            g.hover_boost.is_none(),
            g.hover_acceleration.is_none(),
            g.hover_brake.is_none(),
            g.hover_dampen.is_none(),
            g.gravity
        ),
        (None, true, true, true, true, true, None)
    );
}

/// spec: S-0503 Import mapping and S-0006 (a later layer that omits a key keeps the earlier value)
#[test]
fn hover_fields_layer() {
    let i = import(
        &base("HoverHeight=90\nHoverDampen=0.5\n", "X=1\n"),
        Some("[General]\nHoverHeight=60\n"),
    );
    let r = glasnost_rules::resolve_ruleset(Some((i.tree, i.provenance)), &[])
        .unwrap_or_else(|e| panic!("{e}"));
    let g = &r.ruleset.sim.general;
    assert_eq!(g.hover_height, Some(60));
    assert_eq!(g.hover_dampen.as_ref().map(|d| d.0.as_str()), Some("5e-1"));
}

/// spec: S-0503 Edge cases (a bob period of 0 ticks: the importer warns)
#[test]
fn zero_bob_period_warns() {
    let (_, w) = sim(&base("HoverBob=0.001\n", "X=1\n"));
    assert!(
        warned(&w, "[General] HoverBob 0.001 gives a bob period of 0"),
        "{w:?}"
    );
    let (_, w) = sim(&base("HoverBob=0.002\n", "X=1\n"));
    assert!(!warned(&w, "HoverBob"), "{w:?}");
}

/// spec: S-0503 Import mapping (the hover keys are converted fields of S-0503)
#[test]
fn hover_coverage() {
    let i = import(&base("HoverHeight=90\nHoverBob=1\n", "Gravity=3\n"), None);
    let row = |cat: &str, key: &str| {
        i.coverage
            .iter()
            .find(|r| r.category == cat && r.key == key)
            .map(|r| (r.outcome, r.spec.clone()))
            .unwrap_or_else(|| panic!("{cat} {key}"))
    };
    use crate::coverage::Outcome::Field;
    assert_eq!(
        row("global:General", "HoverHeight"),
        (Field, "S-0503".into())
    );
    assert_eq!(row("global:General", "HoverBob"), (Field, "S-0503".into()));
    assert_eq!(
        row("global:AudioVisual", "Gravity"),
        (Field, "S-0306, S-0503, S-0613".into())
    );
}
