//! The synthetic M7a skirmish for the determinism gate (M7a step 4): the
//! fixture's ruleset and map with invented types, so that a plain
//! AI-versus-AI game uses the psychic weapons of S-0610 (berserk hits, the
//! psychic sensor and the psychic reveal) and a slave-miner economy. Every
//! name and number here is test data of our own, not game behaviour; the
//! abilities' behaviour comes from the specs the sim crate cites, and this
//! file only supplies the types and the places.
//!
//! - **Alpha** plays a Yuri-style side: no refinery and no harvester of the
//!   fixture, but SMV (a slave-miner vehicle with three SLAVE walkers,
//!   built at the war factory) and SMB (its deployed form, which is also
//!   the side's refinery on the sidebar). **Beta** keeps the fixture's
//!   refinery and harvester.
//! - Both sides: MADMAN (a rifleman whose weapon MADRAY has the
//!   berserk-causing warhead MAD; each house starts with a row of them and
//!   the barracks builds more), EYE (a psychic sensor building, built by
//!   the AI) and MINDB (a building that gives the psychic
//!   reveal REVEAL, buildable once a war factory stands). A skirmish drops
//!   the structures a map gives a player's house (S-0009 claim 27), so the
//!   houses start with units only.
//!
//! To add a type, insert it in [`ruleset`] (with `register`) and, for a
//! start unit, add its name to [`INFANTRY`] or [`VEHICLES`]; the gate's
//! module (`crates/sim-gate/src/m7a.rs`) has the counters.

use std::sync::Arc;

use glasnost_map::objects::{Infantry, Placed, Unit};
use glasnost_map::{CellXY, Map};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, LocomotionClass, Locomotor, UnitType, WarheadType, WeaponType};
use glasnost_rules::sim_econ::SuperweaponType;
use glasnost_sim::econ::setup::{self, Player, Settings};
use glasnost_sim::rules::SimRules;
use glasnost_sim::world::GameSetup;

use crate::fixture;

/// The start offset from the middle along x (closer than the fixture's, so
/// that the armies meet earlier).
pub const START_DX: i32 = 16;

/// The lobby's starting credits (more than the fixture's, so that both
/// houses reach the psychic buildings early).
pub const CREDITS: i32 = 15_000;

/// The name of the scenario's map.
pub const MAP_NAME: &str = "synthetic-m7a";

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

/// The required-country list of one country (S-0407 claim 5 (b)): only
/// that country's houses may build the type. The owner lists stay the
/// fixture's, which the factories share.
fn only(country: &str) -> Option<Vec<String>> {
    Some(vec![country.into()])
}

/// The scenario's ruleset.
pub fn ruleset() -> Ruleset {
    let mut r = fixture::ruleset();

    // Beta's economy: the fixture's refinery and harvester.
    if let Some(b) = r.sim.buildings.get_mut("PROC") {
        b.econ.required_houses = only("Betaland");
    }
    if let Some(v) = r.sim.vehicles.get_mut("MINR") {
        v.econ.required_houses = only("Betaland");
    }

    // Alpha's economy: the slave miner, its slaves and its deployed form.
    let group = |e: &mut glasnost_rules::sim_econ::EconFields| {
        e.enslaves = Some("SLAVE".into());
        e.slaves_number = Some(3);
        e.slave_regen_rate = Some(50);
        e.slave_reload_rate = Some(40);
    };
    let mut slave = UnitType {
        strength: Some(100),
        speed: Some(6),
        locomotion_class: Some(LocomotionClass::Foot),
        locomotor: Some(Locomotor::Walk),
        storage: Some(2),
        sight: Some(4),
        ..Default::default()
    };
    slave.econ.harvest_rate = Some(3);
    register(&mut r, "infantry", "SLAVE");
    r.sim.infantry.insert("SLAVE".into(), slave);
    let mut smv = UnitType {
        strength: Some(1000),
        speed: Some(4),
        rot: Some(6),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        tech_level: Some(1),
        owner: Some(vec!["Alphaland".into(), "Betaland".into()]),
        cost: Some(1500),
        sight: Some(6),
        deploys_into: Some("SMB".into()),
        ..Default::default()
    };
    smv.econ.resource_gatherer = Some(true);
    smv.econ.resource_destination = Some(true);
    smv.econ.required_houses = only("Alphaland");
    group(&mut smv.econ);
    register(&mut r, "vehicles", "SMV");
    r.sim.vehicles.insert("SMV".into(), smv);
    let mut smb = fixture::building(3, 3, 1500, -40);
    smb.econ.required_houses = only("Alphaland");
    smb.undeploys_into = Some("SMV".into());
    smb.econ.refinery = Some(true);
    smb.econ.buildup = Some(true);
    smb.econ.buildup_frames = Some(10);
    group(&mut smb.econ);
    register(&mut r, "buildings", "SMB");
    r.sim.buildings.insert("SMB".into(), smb);

    // The berserk weapon and its carrier.
    r.sim.warheads.insert(
        "MAD".into(),
        WarheadType {
            psychedelic: Some(true),
            verses: Some(vec![d("1"); 11]),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "MADRAY".into(),
        WeaponType {
            damage: Some(200),
            rof: Some(60),
            range: Some(d("5")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("MAD".into()),
            ..Default::default()
        },
    );
    let mut madman = r.sim.infantry["WALKER"].clone();
    madman.primary = Some("MADRAY".into());
    madman.secondary = None;
    madman.cost = Some(300);
    register(&mut r, "infantry", "MADMAN");
    r.sim.infantry.insert("MADMAN".into(), madman);

    // The psychic sensor.
    let mut eye = fixture::building(1, 1, 150, 0);
    eye.econ.psychic_detection_radius = Some(12);
    register(&mut r, "buildings", "EYE");
    r.sim.buildings.insert("EYE".into(), eye);

    // The psychic reveal and the building that gives it.
    r.sim.superweapons.insert(
        "REVEAL".into(),
        SuperweaponType {
            kind: Some("PsychicReveal".into()),
            action: Some("PsychicReveal".into()),
            recharge_time: Some(d("1")),
            is_powered: Some(false),
            ..Default::default()
        },
    );
    register(&mut r, "superweapons", "REVEAL");
    let mut mind = fixture::building(2, 2, 600, 0);
    mind.econ.prerequisite = Some(vec!["FACT".into()]);
    mind.econ.super_weapon = Some("REVEAL".into());
    register(&mut r, "buildings", "MINDB");
    r.sim.buildings.insert("MINDB".into(), mind);
    r
}

/// The scenario's rules.
pub fn rules() -> Arc<SimRules> {
    Arc::new(SimRules::new(&ruleset(), MAP_NAME))
}

/// The cell (dx, dy) from the middle.
fn at(dx: i32, dy: i32) -> CellXY {
    fixture::at(dx, dy)
}

fn placed(owner: &str, ty: &str, c: CellXY, key: usize) -> Placed {
    Placed {
        key: key.to_string().into_bytes(),
        owner: owner.as_bytes().to_vec(),
        type_name: ty.as_bytes().to_vec(),
        health: 256,
        cell: c,
        facing: 0,
        raw: Vec::new(),
    }
}

/// The infantry each house starts with, in placement order, in rows of five
/// south of its start.
pub const INFANTRY: [&str; 6] = ["MADMAN", "MADMAN", "MADMAN", "MADMAN", "MADMAN", "MADMAN"];
/// The vehicles each house starts with, in a row south of its infantry.
pub const VEHICLES: [&str; 0] = [];

/// The scenario's map: the fixture's flat ground and resource fields with
/// the starts at ±[`START_DX`] and each house's start objects.
pub fn map() -> Map {
    let mut m = glasnost_sim::testkit::empty_map(fixture::SIZE);
    let mut waypoints = std::collections::BTreeMap::new();
    for (k, dx) in [(0u16, -START_DX), (1u16, START_DX)] {
        let c = at(dx, 0);
        waypoints.insert(k, (c.x as i16, c.y as i16));
        let side = dx.signum();
        for ox in 0..5 {
            for oy in 0..5 {
                let cell = at(dx + side * (ox - 2), 7 + oy);
                if let Some(cell) = m.cells.get_mut(cell) {
                    cell.overlay = ((ox + oy) % 12) as u8;
                    cell.overlay_data = 6;
                }
            }
        }
    }
    m.waypoints = waypoints;
    let mut key = 0;
    for (name, dx) in [("Alpha", -START_DX), ("Beta", START_DX)] {
        for (i, ty) in INFANTRY.iter().enumerate() {
            let c = at(dx + 2 * (i as i32 % 5) - 4, 3 + 2 * (i as i32 / 5));
            m.objects.infantry.push(Infantry {
                placed: placed(name, ty, c, key),
                sub_cell: 2,
            });
            key += 1;
        }
        let row = 3 + 2 * ((INFANTRY.len() as i32 + 4) / 5);
        for (i, ty) in VEHICLES.iter().enumerate() {
            let c = at(dx + i as i32 * 2 - 2, row);
            m.objects.units.push(Unit {
                placed: placed(name, ty, c, key),
            });
            key += 1;
        }
    }
    m
}

/// The lobby setup: Alpha (the Yuri-style side) at the west start, Beta at
/// the east.
pub fn setup(seed: u32) -> GameSetup {
    let player = |name: &str, country: &str, start: u32| Player {
        name: name.into(),
        country: country.into(),
        start: Some(start),
        ai: true,
        values: Default::default(),
    };
    setup::skirmish(
        &rules(),
        &[
            player("Alpha", "Alphaland", 0),
            player("Beta", "Betaland", 1),
        ],
        Settings {
            credits: Some(CREDITS),
            ..Default::default()
        },
        0,
        seed,
    )
}
