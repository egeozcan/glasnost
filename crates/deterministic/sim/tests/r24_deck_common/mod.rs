//! Shared fixtures of the S-0613 revision 24 deck tests (#424, sim batch
//! S28): a synthetic map with a synthetic high bridge (three deck cells in
//! an east-west row over flat level-0 ground) and invented rules. Every name
//! and number is our own, none is game data. M-0026 ran on a stock map;
//! these fixtures let the tests check the relations its rows state, not
//! that map.
#![allow(dead_code)]

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    AnimationType, Decimal, ParticleSystemType, ParticleType, UnitType, WarheadType, WeaponType,
};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{Layer, ObjectId, World};

pub const SIZE: i32 = 20;
/// The deck height of a level-0 cell, 416 leptons above its terrain
/// (S-0306 Terms, *points of a cell target*; S-0303 claim 4; S-0616 claim 14).
pub const DECK: i32 = 416;
/// The drone's owner, and the house of the objects it and the gas reach.
pub const OURS: u8 = 0;
pub const THEIRS: u8 = 1;

pub fn dec(s: &str) -> Decimal {
    Decimal(s.into())
}

fn every(v: &str) -> Option<Vec<Decimal>> {
    Some(vec![dec(v); 11])
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

/// The rules: testkit's combat rules with gravity 10, plus
/// - FOG, a gas particle (lifetime 30000, damage interval 5, damage 7,
///   warhead VENOM) held by the shared gas system GasCloudSys;
/// - VENOM, a warhead of infantry-death kind 8, and PLAGUE, the
///   virus-death animation (middle frame 2 of 6, three FOG particles);
/// - CHAOS, a berserk-causing warhead with a 2-cell spread and the
///   explosion animation HAZE (start 0, end 5, rate 450), which names no
///   particle system;
/// - Pulse, an area-fire, omni-fire weapon (damage 100, range 5, warhead
///   CHAOS) with testkit's invisible straight projectile Bullet;
/// - MADBOT, a deploy-fire vehicle with Pulse in slot 0, psionics
///   immunity and the berserk-friendly flag (S-0610 claim 8);
/// - MAN, an infantry type without a weapon and with speed 0, so that a
///   berserk one stays in its cell.
pub fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.general.gravity = Some(10);
    register(&mut r, "particles", "FOG");
    r.sim.particles.insert(
        "FOG".into(),
        ParticleType {
            behaves_like: Some("gas".into()),
            max_ec: Some(30000),
            max_dc: Some(5),
            damage: Some(7),
            warhead: Some("VENOM".into()),
            state_ai_advance: Some(4),
            end_state_ai: Some(100),
            ..Default::default()
        },
    );
    register(&mut r, "particle_systems", "GasCloudSys");
    r.sim.particle_systems.insert(
        "GasCloudSys".into(),
        ParticleSystemType {
            behaves_like: Some("gas".into()),
            holds_what: Some("FOG".into()),
            ..Default::default()
        },
    );
    for (n, h) in [
        (
            "VENOM",
            WarheadType {
                verses: every("1"),
                inf_death: Some(8),
                ..Default::default()
            },
        ),
        (
            "CHAOS",
            WarheadType {
                verses: every("1"),
                psychedelic: Some(true),
                cell_spread: Some(dec("2")),
                percent_at_max: Some(dec("1")),
                anim_list: Some(vec!["HAZE".into()]),
                ..Default::default()
            },
        ),
    ] {
        register(&mut r, "warheads", n);
        r.sim.warheads.insert(n.into(), h);
    }
    register(&mut r, "animations", "PLAGUE");
    r.sim.animations.insert(
        "PLAGUE".into(),
        AnimationType {
            middle_frame: Some(2),
            frame_count: Some(6),
            spawns_particle: Some("FOG".into()),
            num_particles: Some(3),
            ..Default::default()
        },
    );
    r.sim.general.infantry_virus = Some("PLAGUE".into());
    register(&mut r, "animations", "HAZE");
    r.sim.animations.insert(
        "HAZE".into(),
        AnimationType {
            rate: Some(450),
            start: Some(0),
            end: Some(5),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "Pulse".into(),
        WeaponType {
            damage: Some(100),
            rof: Some(30),
            range: Some(dec("5")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("CHAOS".into()),
            area_fire: Some(true),
            // The drone fires at its own cell, whose centre is its own
            // reference point, so the direction to face is the degenerate
            // one; the omni-fire flag keeps the fixture clear of that.
            // spec: S-0310 claim 2 (an omni-fire weapon is never not facing)
            omni_fire: Some(true),
            ..Default::default()
        },
    );
    let mut bot: UnitType = r.sim.vehicles["BUGGY"].clone();
    bot.strength = Some(1000);
    bot.deploy_fire = Some(true);
    bot.primary = Some("Pulse".into());
    bot.combat.immune_to_psionics = Some(true);
    bot.combat.berserk_friendly = Some(true);
    register(&mut r, "vehicles", "MADBOT");
    r.sim.vehicles.insert("MADBOT".into(), bot);
    let mut man: UnitType = r.sim.infantry["WALKER"].clone();
    man.strength = Some(1000);
    man.speed = Some(0);
    man.primary = None;
    register(&mut r, "infantry", "MAN");
    r.sim.infantry.insert("MAN".into(), man);
    r
}

pub fn game() -> World {
    let mut w = testkit::game_with(&rules(), SIZE, 7);
    testkit::run(&mut w, 4);
    w
}

/// The test cell, in the middle of the bridge.
pub fn test_cell() -> CellXY {
    testkit::mid(SIZE, 0, 0)
}

/// The deck cell east of the test cell.
pub fn east_cell() -> CellXY {
    testkit::mid(SIZE, 1, 0)
}

/// Makes the three cells west, at and east of the test cell a high
/// bridge's deck over flat level-0 ground (S-0211 claim 15).
pub fn bridge(w: &mut World) {
    for dx in -1..=1 {
        let i = w.cells.idx(testkit::mid(SIZE, dx, 0)).unwrap();
        w.cells.deck[i] = true;
        w.cells.deck_entry[i] = true;
        w.cells.bridge_axis[i] = glasnost_sim::cells::Axis::X;
    }
}

/// Moves `id` from the ground of its cell onto the deck there, 416 leptons
/// above the terrain, with a vehicle's occupancy and driving claims.
pub fn onto_deck(w: &mut World, id: ObjectId) {
    let c = w.obj(id).unwrap().pos.cell();
    let vehicle = w.obj(id).unwrap().kind == Kind::Vehicle;
    w.cells.leave(c, Layer::Ground, id);
    w.cells.enter(c, Layer::Deck, id, false);
    if vehicle {
        w.cells.claim_vehicle(c, Layer::Ground, false);
        w.cells.claim_vehicle(c, Layer::Deck, true);
    }
    let o = w.obj_mut(id).unwrap();
    o.layer = Layer::Deck;
    o.pos.z += DECK;
    if let Some(d) = o.drive.as_mut() {
        d.claims = vec![(c, Layer::Deck)];
    }
    assert_eq!(w.height_above_surface(id), 0, "on the deck");
}
