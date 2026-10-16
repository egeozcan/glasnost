//! Claim tests for S-0519 revision 4 (claims 2, 7 and 16, scenario T24):
//! bomb planting, disarming and designation are selected by S-0514 claim 20,
//! so a higher enabled row suppresses them even when its own effect cannot
//! succeed, and a selected one suppresses the lower rows. Synthetic data only.

#[path = "terror_common/mod.rs"]
mod terror_common;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::WarheadType;
use glasnost_sim::bomb::{self, Bomb};
use glasnost_sim::linked;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{ObjectId, World};
use terror_common::*;

type Setter = fn(&mut WarheadType);

/// The rows above bomb planting, designation and disarming that this file
/// sets one at a time (S-0514 claim 20 rows 1 to 6).
const ABOVE_DESIGNATION: [(&str, Setter); 6] = [
    ("takeover", |h| h.mind_control = Some(true)),
    ("planting", |h| h.ivan_bomb = Some(true)),
    ("charging", |h| h.electric_assault = Some(true)),
    ("parasite", |h| h.parasite = Some(true)),
    ("erasure", |h| h.temporal = Some(true)),
    ("locomotor", |h| h.is_locomotor = Some(true)),
];

fn with(name: &'static str, set: impl FnOnce(&mut WarheadType)) -> impl FnOnce(&mut Ruleset) {
    move |r| set(r.sim.warheads.get_mut(name).expect("warhead"))
}

fn plant_on(w: &mut World, x: ObjectId) {
    let tick = w.tick;
    w.obj_mut(x).unwrap().combat.bomb = Some(Bomb {
        planter: None,
        house: None,
        planted: tick,
    });
}

// spec: S-0519 claim 2 (revision 4, scenario T24 a): takeover suppresses planting even when no takeover can happen, and the shot does no ordinary damage either
#[test]
fn takeover_suppresses_planting() {
    let mut w = world(with("IVAN", |h| h.mind_control = Some(true)));
    let v = quiet(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let p = quiet(&mut w, Kind::Infantry, "BOMBER", 0, at(-5, 0));
    assert!(shoot(&mut w, p, v));
    assert!(!bomb::carries(&w, v), "no bomb");
    assert_eq!(health(&w, v), 100_000, "no ordinary damage");
}

// spec: S-0519 claim 2 (revision 4, scenario T24 b): planting is selected over designation, over charging's lower rows and over everything below, and with it selected a bomb attaches
#[test]
fn planting_outranks_charging_and_the_lower_rows() {
    type Lower = (&'static str, Setter);
    let rows: [Lower; 5] = [
        ("charging", |h| h.electric_assault = Some(true)),
        ("parasite", |h| h.parasite = Some(true)),
        ("designation", |h| h.airstrike = Some(true)),
        ("disarming", |h| h.bomb_disarm = Some(true)),
        ("nuke-making", |h| h.nuke_maker = Some(true)),
    ];
    for (name, set) in rows {
        let mut w = world(with("IVAN", set));
        let b = quiet(&mut w, Kind::Building, "BLDG", 1, at(0, 0));
        let p = quiet(&mut w, Kind::Infantry, "BOMBER", 0, at(-5, 0));
        assert!(shoot(&mut w, p, b));
        assert!(bomb::carries(&w, b), "{name}: the bomb attaches");
        assert_eq!(linked::chargers(&w, b), 0, "{name}: no charger");
        assert!(w.strike.strikes.is_empty(), "{name}: no strike");
    }
}

// spec: S-0519 claims 2 and 16 (revision 4, scenario T24 b): with planting and designation enabled, a vehicle firer, even a designator, neither plants nor calls a strike
#[test]
fn a_vehicle_designator_with_planting_does_neither() {
    let mut w = world(with("STRIKE", |h| h.ivan_bomb = Some(true)));
    let a = quiet(&mut w, Kind::Building, "BLDG", 1, at(0, 0));
    let d = quiet(&mut w, Kind::Vehicle, "DESIG", 0, at(-6, 0));
    designate(&mut w, d, a);
    assert!(!bomb::carries(&w, a), "no bomb");
    assert!(w.strike.strikes.is_empty(), "no strike");
    assert_eq!(health(&w, a), 800, "no ordinary damage");
}

// spec: S-0519 claim 16 (revision 4, scenario T24 c): each higher enabled row suppresses designation, even when it cannot succeed for a designator and a building target, and nothing falls through
#[test]
fn higher_rows_suppress_designation() {
    for (name, set) in ABOVE_DESIGNATION {
        let mut w = world(with("STRIKE", set));
        let a = quiet(&mut w, Kind::Building, "BLDG", 1, at(0, 0));
        let d = quiet(&mut w, Kind::Vehicle, "DESIG", 0, at(-6, 0));
        designate(&mut w, d, a);
        assert!(w.strike.strikes.is_empty(), "{name}: no strike");
        assert_eq!(health(&w, a), 800, "{name}: no ordinary damage");
    }
    // The control: designation alone calls the strike.
    let mut w = world(|_| {});
    let a = quiet(&mut w, Kind::Building, "BLDG", 1, at(0, 0));
    let d = quiet(&mut w, Kind::Vehicle, "DESIG", 0, at(-6, 0));
    designate(&mut w, d, a);
    assert_eq!(linked(&w, d).len(), 2, "the control calls its strike");
}

// spec: S-0519 claim 16 (revision 4, scenario T24): selected designation suppresses direct rocking, disarming, disguise-making and nuke-making: the strike is called and a planted bomb stays
#[test]
fn designation_suppresses_the_lower_rows() {
    let mut w = world(with("STRIKE", |h| {
        h.direct_rocker = Some(true);
        h.bomb_disarm = Some(true);
        h.makes_disguise = Some(true);
        h.nuke_maker = Some(true);
    }));
    let v = quiet(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    plant_on(&mut w, v);
    let d = quiet(&mut w, Kind::Vehicle, "DESIG", 0, at(-6, 0));
    designate(&mut w, d, v);
    assert_eq!(linked(&w, d).len(), 2, "the strike is called");
    assert!(bomb::carries(&w, v), "the bomb is not disarmed");
}

// spec: S-0519 claim 7 (revision 4, scenario T24 d): every earlier enabled row suppresses disarming even when it cannot succeed, and a designation shot from a non-designator leaves the bomb
#[test]
fn earlier_rows_suppress_disarming() {
    for (name, set) in ABOVE_DESIGNATION
        .into_iter()
        .chain([("designation", (|h| h.airstrike = Some(true)) as Setter)])
    {
        let mut w = world(with("DISARM", set));
        let v = quiet(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
        plant_on(&mut w, v);
        let e = quiet(&mut w, Kind::Infantry, "ENG", 0, at(-5, 0));
        assert!(shoot(&mut w, e, v));
        assert!(bomb::carries(&w, v), "{name}: the bomb stays");
        assert!(w.strike.strikes.is_empty(), "{name}: no strike");
    }
}

// spec: S-0519 claim 7 (revision 4, scenario T24 e): direct rocking is an earlier row only for a vehicle target: a bombed vehicle keeps its bomb, a bombed building is disarmed
#[test]
fn direct_rocking_suppresses_disarming_only_against_a_vehicle() {
    let mut w = world(with("DISARM", |h| h.direct_rocker = Some(true)));
    let v = quiet(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let b = quiet(&mut w, Kind::Building, "BLDG", 1, at(4, 4));
    plant_on(&mut w, v);
    plant_on(&mut w, b);
    let e = quiet(&mut w, Kind::Infantry, "ENG", 0, at(-5, 0));
    assert!(shoot(&mut w, e, v));
    assert!(bomb::carries(&w, v), "the vehicle keeps its bomb");
    assert!(shoot(&mut w, e, b));
    assert!(!bomb::carries(&w, b), "the building is disarmed");
}

// spec: S-0519 claim 7 (revision 4, scenario T24 f): disguise-making and nuke-making are later rows and cannot suppress disarming
#[test]
fn disarming_outranks_disguise_and_nuke_making() {
    let mut w = world(with("DISARM", |h| {
        h.makes_disguise = Some(true);
        h.nuke_maker = Some(true);
    }));
    let v = quiet(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    plant_on(&mut w, v);
    let e = quiet(&mut w, Kind::Infantry, "ENG", 0, at(-5, 0));
    assert!(shoot(&mut w, e, v));
    assert!(!bomb::carries(&w, v), "the bomb is disarmed");
    assert_eq!(health(&w, v), 100_000, "no ordinary damage");
}
