//! Tests of the naval fields of the `sim` section (S-0504 Import mapping)
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

fn sim(base: &str) -> (SimData, Vec<Warning>) {
    let inputs = Inputs {
        base: Some(file("rules.ini", base)),
        ..Inputs::default()
    };
    let import = convert(&inputs, &ImportOptions::default());
    let r = glasnost_rules::resolve_ruleset(Some((import.tree, import.provenance)), &[])
        .unwrap_or_else(|e| panic!("{e}"));
    (r.ruleset.sim, import.warnings)
}

fn warned(w: &[Warning], text: &str) -> bool {
    w.iter().any(|w| w.message.contains(text))
}

/// The ship locomotor's class identifier (S-0207 table T3).
const SHIP: &str = "{2BEA74E1-7CCA-11D3-BE14-00104B62A16C}";

fn base(general: &str) -> String {
    format!(
        "[VehicleTypes]\n1=Skiff\n2=Diver\n3=Raft\n4=Cart\n[BuildingTypes]\n1=Dock\n2=Shed\n\
         [Skiff]\nLocomotor={SHIP}\nNaval=yes\nWeight=4.5\n\
         [Diver]\nLocomotor={SHIP}\nNaval=yes\nUnderwater=yes\nCloakable=yes\n\
         [Raft]\nMovementZone=Amphibious\nUnderwater=yes\n[Cart]\nX=1\n\
         [Dock]\nNaval=yes\nUnitRepair=yes\n[Shed]\nUnitRepair=no\n\
         [General]\n{general}"
    )
}

/// spec: S-0504 Import mapping (Locomotor, Naval, Underwater, Cloakable, MovementZone, Weight,
/// building Naval and UnitRepair, [General] Wake and ShipSinkingWeight; absent: no)
#[test]
fn naval_fields() {
    let (s, w) = sim(&base("Wake=Swirl\nShipSinkingWeight=2.5\n"));
    let skiff = &s.vehicles["Skiff"];
    assert_eq!(skiff.locomotor, Some(Locomotor::Ship));
    assert_eq!(
        (
            skiff.combat.naval,
            skiff.combat.underwater,
            skiff.combat.cloakable
        ),
        (Some(true), None, None)
    );
    assert!(skiff.weight.is_some());
    let diver = &s.vehicles["Diver"];
    assert_eq!(
        (diver.combat.underwater, diver.combat.cloakable),
        (Some(true), Some(true))
    );
    let raft = &s.vehicles["Raft"];
    assert_eq!(raft.movement_zone, Some(MovementZone::Amphibious));
    assert_eq!(
        (raft.combat.underwater, raft.combat.cloakable),
        (Some(true), None)
    );
    let cart = &s.vehicles["Cart"];
    assert_eq!(
        (
            cart.combat.naval,
            cart.combat.cloakable,
            cart.weight.as_ref()
        ),
        (None, None, None)
    );
    assert_eq!(
        (s.buildings["Dock"].naval, s.buildings["Dock"].unit_repair),
        (Some(true), Some(true))
    );
    assert_eq!(
        (s.buildings["Shed"].naval, s.buildings["Shed"].unit_repair),
        (None, Some(false))
    );
    assert_eq!(s.general.wake.as_deref(), Some("Swirl"));
    assert_eq!(
        s.general.ship_sinking_weight.as_ref().map(|d| d.0.as_str()),
        Some("25e-1")
    );
    assert!(!warned(&w, "[General] Wake"), "{w:?}");
}

/// spec: S-0504 claim 4 and Import mapping (no wake animation: the importer warns)
#[test]
fn absent_wake_warns() {
    let (s, w) = sim(&base("X=1\n"));
    assert_eq!(s.general.wake, None);
    assert!(
        warned(&w, "[General] Wake is given by no rules layer"),
        "{w:?}"
    );
}

/// spec: S-0504 Import mapping (Cloakable is converted for the naval and cloaking topics)
#[test]
fn cloakable_coverage() {
    let inputs = Inputs {
        base: Some(file("rules.ini", &base("Wake=Swirl\n"))),
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
    use crate::coverage::Outcome::Field;
    assert_eq!(
        row("vehicles", "Cloakable"),
        (Field, "S-0504, S-0516".into())
    );
    assert_eq!(
        row("vehicles", "Underwater"),
        (Field, "S-0308, S-0504".into())
    );
}

/// spec: S-0314 Edge cases and Import mapping (NumberOfDocks above 1 on a repair depot: the importer warns; the count is still imported; a count of 1, or a building without the repair flag, gives no warning)
#[test]
fn depot_with_several_places_warns() {
    let text = "[BuildingTypes]\n1=Bay\n2=Pad\n3=Lot\n\
                [Bay]\nUnitRepair=yes\nNumberOfDocks=2\n\
                [Pad]\nUnitRepair=no\nNumberOfDocks=3\n\
                [Lot]\nUnitRepair=yes\nNumberOfDocks=1\n";
    let (s, w) = sim(text);
    assert_eq!(s.buildings["Bay"].econ.number_of_docks, Some(2));
    assert!(warned(
        &w,
        "buildings Bay NumberOfDocks=2 on a repair depot"
    ));
    assert!(!warned(&w, "buildings Pad NumberOfDocks"));
    assert!(!warned(&w, "buildings Lot NumberOfDocks"));
}
