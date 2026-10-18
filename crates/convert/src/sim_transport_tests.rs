//! Tests of the transport fields of the `sim` section (S-0507 Import
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

fn text(d: &Option<glasnost_rules::sim::Decimal>) -> Option<String> {
    d.as_ref().map(|d| d.0.clone())
}

/// spec: S-0507 Import mapping (Passengers an integer; Size and SizeLimit
/// floating point; OpenTopped, Gunner, IsTrain, IsDropship and Carryall
/// booleans; TurretCount an integer; EnterTransportSound and
/// LeaveTransportSound sound names)
#[test]
fn transport_fields() {
    let s = sim("[InfantryTypes]\n1=Trooper\n[VehicleTypes]\n1=Hauler\n\
         [AircraftTypes]\n1=Lifter\n\
         [Trooper]\nSize=1.5\nEnterTransportSound=InSound\nLeaveTransportSound=OutSound\n\
         [Hauler]\nPassengers=4\nSize=3\nSizeLimit=2.5\nOpenTopped=yes\nGunner=yes\n\
         TurretCount=2\nIsTrain=no\n\
         [Lifter]\nPassengers=6\nIsDropship=yes\nCarryall=no\n");
    let t = &s.infantry["Trooper"];
    assert_eq!(text(&t.size).as_deref(), Some("15e-1"));
    assert_eq!(t.enter_transport_sound.as_deref(), Some("InSound"));
    assert_eq!(t.leave_transport_sound.as_deref(), Some("OutSound"));
    let v = &s.vehicles["Hauler"];
    assert_eq!(v.passengers, Some(4));
    assert_eq!(text(&v.size).as_deref(), Some("3"));
    assert_eq!(text(&v.size_limit).as_deref(), Some("25e-1"));
    assert_eq!(v.open_topped, Some(true));
    assert_eq!(v.gunner, Some(true));
    assert_eq!(v.turret_count, Some(2));
    assert_eq!(v.is_train, Some(false));
    let a = &s.aircraft["Lifter"];
    assert_eq!(a.passengers, Some(6));
    assert_eq!(a.is_dropship, Some(true));
    assert_eq!(a.carryall, Some(false));
}

/// spec: S-0507 Import mapping (absent keys stay absent: the absent-key
/// defaults, size 1.0, size limit 0.0 and capacity 0, apply in the
/// simulation)
#[test]
fn absent_transport_fields() {
    let s = sim("[VehicleTypes]\n1=Hauler\n[Hauler]\nStrength=10\n");
    let v = &s.vehicles["Hauler"];
    assert_eq!(
        (
            v.passengers,
            text(&v.size),
            text(&v.size_limit),
            v.is_dropship,
            v.enter_transport_sound.clone(),
        ),
        (None, None, None, None, None)
    );
}

/// spec: S-0605 Import mapping (InfantryAbsorb, UnitAbsorb booleans;
/// Passengers an integer and SizeLimit floating point on a building type;
/// [AudioVisual] EnterBioReactorSound and LeaveBioReactorSound sound
/// names; every key absent leaves its field unset, read as the spec's
/// absent value: no, no, 0, 0.0, none)
#[test]
fn absorber_fields() {
    let s = sim("[BuildingTypes]\n1=Vat\n2=Shack\n\
         [Vat]\nInfantryAbsorb=yes\nUnitAbsorb=no\nPassengers=5\nSizeLimit=2.5\n\
         [Shack]\nStrength=10\n\
         [AudioVisual]\nEnterBioReactorSound=Gulp\nLeaveBioReactorSound=Burp\n");
    let v = &s.buildings["Vat"];
    assert_eq!(v.infantry_absorb, Some(true));
    assert_eq!(v.unit_absorb, Some(false));
    assert_eq!(v.passengers, Some(5));
    assert_eq!(text(&v.size_limit).as_deref(), Some("25e-1"));
    assert_eq!(s.general.enter_bio_reactor_sound.as_deref(), Some("Gulp"));
    assert_eq!(s.general.leave_bio_reactor_sound.as_deref(), Some("Burp"));
    let h = &s.buildings["Shack"];
    assert_eq!(h.infantry_absorb, None);
    assert_eq!(h.unit_absorb, None);
    assert_eq!(h.passengers, None);
    assert_eq!(h.size_limit, None);
    let s = sim("[BuildingTypes]\n1=Shack\n[Shack]\nStrength=10\n");
    assert_eq!(s.general.enter_bio_reactor_sound, None);
    assert_eq!(s.general.leave_bio_reactor_sound, None);
}

/// spec: S-0603 Import mapping (Grinding a boolean on a building type;
/// [AudioVisual] EnterGrinderSound and LeaveGrinderSound sound names; every
/// key absent leaves its field unset, read as the spec's absent value: no,
/// none, none)
#[test]
fn grinder_fields() {
    let s = sim("[BuildingTypes]\n1=Mill\n2=Shack\n\
         [Mill]\nGrinding=yes\nNumberOfDocks=2\n\
         [Shack]\nStrength=10\n\
         [AudioVisual]\nEnterGrinderSound=Crunch\nLeaveGrinderSound=Spit\n");
    assert_eq!(s.buildings["Mill"].grinding, Some(true));
    assert_eq!(s.buildings["Mill"].econ.number_of_docks, Some(2));
    assert_eq!(s.buildings["Shack"].grinding, None);
    assert_eq!(s.general.enter_grinder_sound.as_deref(), Some("Crunch"));
    assert_eq!(s.general.leave_grinder_sound.as_deref(), Some("Spit"));
    let s = sim("[BuildingTypes]\n1=Shack\n[Shack]\nStrength=10\n");
    assert_eq!(s.general.enter_grinder_sound, None);
    assert_eq!(s.general.leave_grinder_sound, None);
}

/// spec: S-0606 Import mapping (Bunker a boolean on a building type;
/// Bunkerable a boolean on an actor type; [AudioVisual] BunkerWallsUpSound and
/// BunkerWallsDownSound sound names; art SpecialAnimTwo to SpecialAnimFour and
/// their damaged forms; every key absent leaves its field unset, read as the
/// spec's absent value)
#[test]
fn bunker_fields() {
    let s = sim("[BuildingTypes]\n1=Pit\n2=Shack\n\
         [VehicleTypes]\n1=Tank\n2=Cart\n\
         [Pit]\nBunker=yes\nNumberOfDocks=1\n\
         [Shack]\nStrength=10\n\
         [Tank]\nBunkerable=no\nTurret=yes\n\
         [Cart]\nStrength=10\n\
         [AudioVisual]\nBunkerWallsUpSound=Clang\nBunkerWallsDownSound=Clunk\n");
    assert_eq!(s.buildings["Pit"].bunker, Some(true));
    assert_eq!(s.buildings["Shack"].bunker, None);
    assert_eq!(s.vehicles["Tank"].bunkerable, Some(false));
    assert_eq!(s.vehicles["Cart"].bunkerable, None);
    assert_eq!(s.general.bunker_walls_up_sound.as_deref(), Some("Clang"));
    assert_eq!(s.general.bunker_walls_down_sound.as_deref(), Some("Clunk"));
    let s = sim("[BuildingTypes]\n1=Shack\n[Shack]\nStrength=10\n");
    assert_eq!(s.general.bunker_walls_up_sound, None);
    assert_eq!(s.general.bunker_walls_down_sound, None);
}
