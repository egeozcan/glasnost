//! The synthetic M7b skirmish for the determinism gate (M7b step 3.3): the
//! M7a scenario ([`crate::fixture_m7a`]) plus a drain (S-0608). Every name
//! and number here is test data of our own, not game behaviour; the drain's
//! behaviour comes from the specs the sim crate cites, and this file only
//! supplies the types and the places.
//!
//! - **Alpha** (the Yuri-style side) only: DISC, a balloon-hovering
//!   jumpjet vehicle built at the war factory. Its primary is the
//!   fixture's gun, its secondary SUCK, a draining weapon with the
//!   vertical projectile DROP, fired in any direction and of a reach
//!   short enough (a tenth of a cell) that the disc fires it only once it
//!   is over the building (S-0608 claim 2: an attempt from a neighbouring
//!   cell forms no link). Alpha starts with [`DISCS`] of them, north of
//!   its start.
//! - **Beta**'s power plant POWR and refinery PROC are drainable; PROC is
//!   a resource destination, so a link to it takes credits, and one to POWR
//!   blacks out Beta's power.
//! - The game-wide drain settings: a payout every [`PERIOD`] ticks of
//!   [`AMOUNT`] credits, and the drain animation SUCKANIM.
//!
//! The gate's module is `crates/sim-gate/src/m7b.rs`.

use std::sync::Arc;

use glasnost_map::objects::{Placed, Unit};
use glasnost_map::{CellXY, Map};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    AnimationType, Decimal, LocomotionClass, Locomotor, MovementZone, ProjectileType, UnitType,
    WeaponType,
};
use glasnost_sim::econ::setup::{self, Player, Settings};
use glasnost_sim::rules::SimRules;
use glasnost_sim::world::GameSetup;

use crate::{fixture, fixture_m7a};

/// The name of the scenario's map.
pub const MAP_NAME: &str = "synthetic-m7b";

/// The discs Alpha starts with, in a row north of its start.
pub const DISCS: usize = 3;

/// The ticks between two payouts of a link.
pub const PERIOD: i32 = 30;

/// The credits one payout moves at most.
pub const AMOUNT: i32 = 40;

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

/// The scenario's ruleset: M7a's with the disc, the drainable buildings of
/// Beta and the drain settings.
pub fn ruleset() -> Ruleset {
    let mut r = fixture_m7a::ruleset();
    r.sim.projectiles.insert(
        "DROP".into(),
        ProjectileType {
            vertical: Some(true),
            inviso: Some(true),
            ..Default::default()
        },
    );
    let warhead = r.sim.weapons["Gun"].warhead.clone();
    r.sim.weapons.insert(
        "SUCK".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(40),
            range: Some(d("0.1")),
            speed: Some(100),
            projectile: Some("DROP".into()),
            warhead,
            drain_weapon: Some(true),
            omni_fire: Some(true),
            ..Default::default()
        },
    );
    let disc = UnitType {
        strength: Some(500),
        speed: Some(6),
        locomotion_class: Some(LocomotionClass::Hover),
        movement_zone: Some(MovementZone::Fly),
        locomotor: Some(Locomotor::Jumpjet),
        jumpjet: Some(true),
        sight: Some(8),
        jumpjet_speed: Some(14),
        jumpjet_climb: Some(d("5.0")),
        jumpjet_crash: Some(d("6.0")),
        jumpjet_height: Some(500),
        jumpjet_accel: Some(d("2.0")),
        jumpjet_wobbles: Some(d("0.15")),
        jumpjet_deviation: Some(40),
        jumpjet_turn_rate: Some(4),
        balloon_hover: Some(true),
        mobile_fire: Some(true),
        primary: Some("Gun".into()),
        secondary: Some("SUCK".into()),
        tech_level: Some(1),
        owner: Some(vec!["Alphaland".into(), "Betaland".into()]),
        cost: Some(1000),
        econ: glasnost_rules::sim_econ::EconFields {
            required_houses: Some(vec!["Alphaland".into()]),
            ..Default::default()
        },
        ..Default::default()
    };
    register(&mut r, "vehicles", "DISC");
    r.sim.vehicles.insert("DISC".into(), disc);
    if let Some(b) = r.sim.buildings.get_mut("POWR") {
        b.combat.drainable = Some(true);
    }
    if let Some(b) = r.sim.buildings.get_mut("PROC") {
        b.combat.drainable = Some(true);
        b.econ.resource_destination = Some(true);
    }
    register(&mut r, "animations", "SUCKANIM");
    r.sim.animations.insert(
        "SUCKANIM".into(),
        AnimationType {
            frame_count: Some(8),
            random_rate: Some([300, 150]),
            ..Default::default()
        },
    );
    let g = &mut r.sim.general;
    g.drain_money_frame_delay = Some(PERIOD);
    g.drain_money_amount = Some(AMOUNT);
    g.drain_animation_type = Some("SUCKANIM".into());
    r
}

/// The scenario's rules.
pub fn rules() -> Arc<SimRules> {
    Arc::new(SimRules::new(&ruleset(), MAP_NAME))
}

/// The scenario's map: M7a's with Alpha's start discs.
pub fn map() -> Map {
    let mut m = fixture_m7a::map();
    let first = m.objects.infantry.len() + m.objects.units.len();
    for i in 0..DISCS {
        let key = first + i;
        let c: CellXY = fixture::at(-fixture_m7a::START_DX + 2 * i as i32 - 2, -9);
        m.objects.units.push(Unit {
            placed: Placed {
                key: key.to_string().into_bytes(),
                owner: b"Alpha".to_vec(),
                type_name: b"DISC".to_vec(),
                health: 256,
                cell: c,
                facing: 0,
                raw: Vec::new(),
            },
        });
    }
    m
}

/// The lobby setup: Alpha at the west start, Beta at the east.
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
            credits: Some(fixture_m7a::CREDITS),
            ..Default::default()
        },
        0,
        seed,
    )
}
