//! Claim tests for S-0413 (country bonuses) on synthetic data.

mod common;

use common::{Scene, TRACK, cell, unit_type};
use glasnost_rules::schema::ColourDefinition;
use glasnost_rules::sim::{Country, Decimal};
use glasnost_sim::econ::setup;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseRole, HouseSetup};

fn country(passive: bool) -> Country {
    Country {
        multiplay_passive: Some(passive),
        ..Default::default()
    }
}

/// A skirmish of Alpha and Beta with a third, unplayed country Gammaland.
fn skirmish() -> Scene {
    let mut s = Scene::new();
    for n in ["Alphaland", "Betaland"] {
        s.ruleset.sim.countries.insert(n.into(), country(false));
    }
    s.register("countries", "Gammaland");
    s.ruleset.sim.countries.insert("Gammaland".into(), country(false));
    for (i, (x, y)) in [(40i16, 40i16), (52, 40), (40, 52), (52, 52)].into_iter().enumerate() {
        s.map.waypoints.insert(i as u16, (x, y));
    }
    s.setup.sandbox = false;
    s.setup.bases = false;
    s.setup.unit_count = 0;
    s
}

// spec: S-0413 claim 1 (the country fields of the table are read: speeds, side, listing flag, colour and the fields with no effect)
#[test]
fn claim_1_fields_read() {
    let mut s = skirmish();
    let c = s.ruleset.sim.countries.get_mut("Alphaland").unwrap();
    c.econ.speed_aircraft_mult = Some(Decimal("1.5".into()));
    c.econ.side = Some("Civilian".into());
    c.econ.multiplay = Some(true);
    c.econ.smart_ai = Some(true);
    let r = s.sim_rules();
    let mode = r.mode;
    assert_eq!(r.countries[0].speed_aircraft, Decimal("1.5".into()).single_value(mode));
    assert!(r.econ.setup.civilian[0]);
    assert!(!r.econ.setup.civilian[1]);
}

// spec: S-0413 claim 3 (an aircraft's speed uses its owner's aircraft factor, S-0207 F2's order and rounding; each kind its own factor)
#[test]
fn claim_3_aircraft_speed() {
    let mut s = skirmish();
    let mut jet = unit_type(TRACK, 10, 5);
    jet.speed = Some(10);
    s.aircraft("JET", jet);
    let c = s.ruleset.sim.countries.get_mut("Alphaland").unwrap();
    c.econ.speed_aircraft_mult = Some(Decimal("1.5".into()));
    c.speed_units_mult = Some(Decimal("3".into()));
    let mut w = s.build();
    let a = glasnost_sim::testkit::spawn(&mut w, Kind::Aircraft, "JET", 0, cell(0, 0), 0);
    let b = glasnost_sim::testkit::spawn(&mut w, Kind::Aircraft, "JET", 1, cell(4, 0), 0);
    let mode = w.rules().mode;
    let one = glasnost_num::F64::from_i64(1, mode);
    // V is the type's speed as S-0207 F1 takes it; Beta's factor is 1.0.
    let v = setup::aircraft_speed(&w, b, one);
    assert!(v > 0);
    // s₁ = trunc(V × 1.5), the vehicle factor 3 not used.
    assert_eq!(setup::aircraft_speed(&w, a, one), v * 3 / 2);
    let half = glasnost_sim::rules::nearest_double("0.5");
    assert_eq!(setup::aircraft_speed(&w, a, half), v * 3 / 2 / 2);
}

// spec: S-0413 claim 4 (parent country, prefix, suffix, smart-computer and listing flags change no game state)
#[test]
fn claim_4_no_effect_fields() {
    let mut s = skirmish();
    let plain = glasnost_sim::save::state_hash(&s.build());
    let c = s.ruleset.sim.countries.get_mut("Alphaland").unwrap();
    c.econ.parent_country = Some("Betaland".into());
    c.econ.prefix = Some("Z".into());
    c.econ.suffix = Some("Xy".into());
    c.econ.smart_ai = Some(true);
    c.econ.multiplay = Some(true);
    assert_eq!(glasnost_sim::save::state_hash(&s.build()), plain);
}

// spec: S-0413 claim 5 (the colour field of a played country holds the lobby slot of its last player house; an unplayed country keeps its imported colour position)
#[test]
fn claim_5_colour_field() {
    let mut s = skirmish();
    s.ruleset.colours = ["Gold", "Red", "Blue"]
        .iter()
        .map(|n| ColourDefinition {
            name: (*n).into(),
            hsv: [0, 0, 0],
        })
        .collect();
    s.ruleset.sim.countries.get_mut("Gammaland").unwrap().econ.color = Some("blue".into());
    s.ruleset.sim.countries.get_mut("Alphaland").unwrap().econ.color = Some("Red".into());
    s.setup.houses.push(HouseSetup {
        name: "Alpha2".into(),
        ..s.setup.houses[0].clone()
    });
    s.setup.skirmish.houses = vec![Default::default(); 3];
    s.setup.skirmish.houses[0].colour = Some(3);
    s.setup.skirmish.houses[2].colour = Some(6);
    let w = s.build();
    assert_eq!(setup::colour_field(&w, 0), 6, "the last Alphaland house");
    assert_eq!(setup::colour_field(&w, 2), 2, "Gammaland: Blue, position 2");
    assert_eq!(setup::colour_field(&w, 1), 0, "Betaland: no slot, imported default 0");
}

// spec: S-0413 claim 6 (played and not passive: on; played and passive: off; unplayed: unchanged; the neutral and special houses count)
#[test]
fn claim_6_wall_owner_rewrite() {
    let mut s = skirmish();
    s.ruleset.sim.countries.get_mut("Alphaland").unwrap().wall_owner = Some(false);
    s.ruleset.sim.countries.get_mut("Gammaland").unwrap().wall_owner = Some(false);
    s.register("countries", "Special");
    let mut sp = country(true);
    sp.wall_owner = Some(true);
    s.ruleset.sim.countries.insert("Special".into(), sp);
    s.setup.houses.push(HouseSetup {
        name: "Special".into(),
        country: "Special".into(),
        human: false,
        start: None,
        role: HouseRole::Special,
        ai: false,
    });
    let w = s.build();
    assert!(setup::wall_owner(&w, 0), "played, not passive");
    assert!(!setup::wall_owner(&w, 3), "played, passive");
    assert!(!setup::wall_owner(&w, 2), "unplayed: imported");
    // The sandbox has no game start: imported values.
    let mut s2 = Scene::new();
    s2.ruleset.sim.countries.get_mut("Alphaland").unwrap().wall_owner = Some(false);
    assert!(!setup::wall_owner(&s2.build(), 0));
}

// spec: S-0413 claim 8 (an AI house and a person's house of one country get the same factors)
#[test]
fn claim_8_same_advantages() {
    let mut s = skirmish();
    let c = s.ruleset.sim.countries.get_mut("Alphaland").unwrap();
    c.firepower = Some(Decimal("1.25".into()));
    c.rof = Some(Decimal("0.75".into()));
    s.setup.houses[1].country = "Alphaland".into();
    s.setup.houses[1].ai = true;
    let w = s.build();
    assert_eq!(w.houses[0].combat.firepower, w.houses[1].combat.firepower);
    assert_eq!(w.houses[0].combat.rof_factor, w.houses[1].combat.rof_factor);
    let mode = w.rules().mode;
    assert_eq!(w.houses[0].combat.firepower, Decimal("1.25".into()).value(mode));
}
