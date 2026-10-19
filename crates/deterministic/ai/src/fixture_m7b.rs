//! The synthetic M7b skirmish for the determinism gate (M7b steps 3.3 and
//! 3.4): the M7a scenario ([`crate::fixture_m7a`]) plus a drain (S-0608)
//! and a genetic mutator (S-0611). Every name
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
//! - **Alpha** only (S-0611, M7b step 3.4): GENEB, a building that gives
//!   the genetic mutator GENE (charge [`GENE_CHARGE`] minutes, an invented
//!   short time so that the AI fires it several times in the run; cell
//!   mode), cheap and with no prerequisite, built by the AI's superweapon
//!   step once its base allows. The cell warhead MUTATE has the mutating
//!   death kind; the conversion animation MUTANIM makes a MADMAN (the
//!   animation-to-infantry list's only entry); the launch animation
//!   IONANIM.
//! - **Alpha** only (S-0612, M7b step 3.5): DOMB, a building that gives
//!   the psychic dominator DOM (charge [`DOM_CHARGE`] minutes, an invented
//!   short time so that the AI fires it several times in the run), cheap
//!   (cheaper than GENEB, so built first) and with no prerequisite, built
//!   by the AI's superweapon step. The lead-in DOMLEAD (8 frames) and the sweep DOMSWEEP (12
//!   frames), the area damage warhead PSIWH with an invented amount
//!   [`DOM_DAMAGE`]; the capture range and the fire-at percentage are the
//!   absent-key defaults.
//! - **Beta** starts with crowds of CROWD, an unarmed walker nobody can
//!   build, three on neighbouring cells at each of [`CROWDS`] around its
//!   start: dense infantry that stays where it is, apart enough that one
//!   launch takes one crowd. Beta builds no barracks within the gate's run,
//!   so without them its six start riflemen would be the only infantry the
//!   mutator could take.
//!
//! The gate's module is `crates/sim-gate/src/m7b.rs`.

use std::sync::Arc;

use glasnost_map::objects::{Infantry, Placed, Unit};
use glasnost_map::{CellXY, Map};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    AnimationType, Decimal, LocomotionClass, Locomotor, MovementZone, ProjectileType, UnitType,
    WarheadType, WeaponType,
};
use glasnost_rules::sim_econ::SuperweaponType;
use glasnost_sim::econ::setup::{self, Player, Settings};
use glasnost_sim::rules::SimRules;
use glasnost_sim::world::GameSetup;

use crate::{fixture, fixture_m7a};

/// The name of the scenario's map.
pub const MAP_NAME: &str = "synthetic-m7b";

/// The discs Alpha starts with, in a row north of its start.
pub const DISCS: usize = 3;

/// Beta's extra start infantry for the mutator (S-0611): crowds of three
/// unarmed CROWD walkers on neighbouring cells, at these offsets from its
/// start, apart enough that one launch reaches one crowd only. The
/// mutator and the dominator (M7b step 3.5) take from the same crowds, so
/// there are enough for at least three launches of each.
pub const CROWDS: [(i32, i32); 11] = [
    (-6, -8),
    (4, -8),
    (-6, 8),
    (4, 8),
    (-12, 0),
    (-12, -12),
    (-12, 12),
    (-6, -15),
    (4, -15),
    (-6, 15),
    (4, 15),
];

/// The genetic mutator's charge time in minutes (a decimal of the rules).
pub const GENE_CHARGE: &str = "0.2";

/// The psychic dominator's charge time in minutes (a decimal of the
/// rules).
pub const DOM_CHARGE: &str = "0.3";

/// The psychic dominator's area damage amount.
pub const DOM_DAMAGE: i32 = 30;

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
    let mut r = fixture_m7a::base_ruleset();
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
    // CROWD: an unarmed walker nobody can build (tech level -1); Beta
    // starts with crowds of them ([`CROWDS`]).
    let mut crowd = r.sim.infantry["WALKER"].clone();
    crowd.primary = None;
    crowd.secondary = None;
    crowd.tech_level = Some(-1);
    register(&mut r, "infantry", "CROWD");
    r.sim.infantry.insert("CROWD".into(), crowd);
    // The genetic mutator (S-0611), Alpha's only.
    r.sim.superweapons.insert(
        "GENE".into(),
        SuperweaponType {
            kind: Some("GeneticConverter".into()),
            action: Some("GeneticConverter".into()),
            recharge_time: Some(d(GENE_CHARGE)),
            is_powered: Some(false),
            ..Default::default()
        },
    );
    register(&mut r, "superweapons", "GENE");
    let mut gene = fixture::building(2, 2, 300, 0);
    gene.econ.super_weapon = Some("GENE".into());
    gene.econ.required_houses = Some(vec!["Alphaland".into()]);
    register(&mut r, "buildings", "GENEB");
    r.sim.buildings.insert("GENEB".into(), gene);
    r.sim.warheads.insert(
        "MUTATE".into(),
        WarheadType {
            inf_death: Some(9),
            verses: Some(vec![d("1"); 11]),
            ..Default::default()
        },
    );
    for (name, frames, make) in [("MUTANIM", 8, Some(0)), ("IONANIM", 6, None)] {
        register(&mut r, "animations", name);
        r.sim.animations.insert(
            name.into(),
            AnimationType {
                frame_count: Some(frames),
                make_infantry: make,
                ..Default::default()
            },
        );
    }
    // The psychic dominator (S-0612), Alpha's only.
    r.sim.superweapons.insert(
        "DOM".into(),
        SuperweaponType {
            kind: Some("PsychicDominator".into()),
            action: Some("PsychicDominator".into()),
            recharge_time: Some(d(DOM_CHARGE)),
            is_powered: Some(false),
            ..Default::default()
        },
    );
    register(&mut r, "superweapons", "DOM");
    let mut dom = fixture::building(2, 2, 100, 0);
    dom.econ.super_weapon = Some("DOM".into());
    dom.econ.required_houses = Some(vec!["Alphaland".into()]);
    register(&mut r, "buildings", "DOMB");
    r.sim.buildings.insert("DOMB".into(), dom);
    r.sim.warheads.insert(
        "PSIWH".into(),
        WarheadType {
            cell_spread: Some(d("2")),
            verses: Some(vec![d("1"); 11]),
            ..Default::default()
        },
    );
    for (name, frames) in [("DOMLEAD", 8), ("DOMSWEEP", 12)] {
        register(&mut r, "animations", name);
        r.sim.animations.insert(
            name.into(),
            AnimationType {
                frame_count: Some(frames),
                ..Default::default()
            },
        );
    }
    let s = &mut r.sim.general.superweapons;
    s.dominator_first_anim = Some("DOMLEAD".into());
    s.dominator_second_anim = Some("DOMSWEEP".into());
    s.dominator_warhead = Some("PSIWH".into());
    s.dominator_damage = Some(DOM_DAMAGE);
    s.mutate_explosion = Some(false);
    s.mutate_warhead = Some("MUTATE".into());
    s.infantry_mutate = Some("MUTANIM".into());
    s.anim_to_infantry = Some(vec!["MADMAN".into()]);
    s.ion_blast = Some("IONANIM".into());
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

/// The scenario's map: M7a's with Alpha's start discs and Beta's crowds.
pub fn map() -> Map {
    let mut m = fixture_m7a::base_map();
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
    let mut key = m.objects.infantry.len() + m.objects.units.len();
    for (dx, dy) in CROWDS {
        for (x, y) in [(0, 0), (1, 0), (0, 1)] {
            let c: CellXY = fixture::at(fixture_m7a::START_DX + dx + x, dy + y);
            m.objects.infantry.push(Infantry {
                placed: Placed {
                    key: key.to_string().into_bytes(),
                    owner: b"Beta".to_vec(),
                    type_name: b"CROWD".to_vec(),
                    health: 256,
                    cell: c,
                    facing: 0,
                    raw: Vec::new(),
                },
                sub_cell: 2,
            });
            key += 1;
        }
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
