//! Claim tests for the play side of electric-bolt shots (S-0316 claims 1
//! to 4 and 6): which shots make a bolt, its ends and attachment facts as
//! reported to the drawing, and the spark system at the bolt end, on
//! synthetic data: invented particle, system and weapon types, every number
//! our own. The drawing side is `glasnost-app`'s `mapview::bolts`; the
//! ledger is `docs/m8-claims/c5-S-0316.md`.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, ParticleSystemType, ParticleType, WeaponType};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::combat::{fire, position, range};
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{ObjectId, Pos, Target, World};
use glasnost_sim::{save, testkit};

const N: i32 = 24;
const EAST: u16 = 16384;
const WEST: u16 = 49152;

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

/// An invented electric-bolt weapon: rate 30, range 10 cells, the invisible
/// projectile Bullet, warhead SA.
fn zapper() -> WeaponType {
    WeaponType {
        damage: Some(1),
        rof: Some(30),
        range: dec("10"),
        speed: Some(100),
        projectile: Some("Bullet".into()),
        warhead: Some("SA".into()),
        is_electric_bolt: Some(true),
        ..Default::default()
    }
}

/// The rules: testkit's combat rules plus the particle SPARK, the spark
/// system BURST as the game-wide spark type, and the weapon ZAP (an
/// electric bolt) as the CART's primary, the WALKER's primary and the
/// TOWER's primary; a durable BUGGY.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    register(&mut r, "particles", "SPARK");
    r.sim.particles.insert(
        "SPARK".into(),
        ParticleType {
            behaves_like: Some("spark".into()),
            max_ec: Some(1000),
            end_state_ai: Some(100),
            state_ai_advance: Some(4),
            ..Default::default()
        },
    );
    register(&mut r, "particle_systems", "BURST");
    r.sim.particle_systems.insert(
        "BURST".into(),
        ParticleSystemType {
            behaves_like: Some("spark".into()),
            holds_what: Some("SPARK".into()),
            spark_spawn_frames: Some(1),
            particle_cap: Some(4),
            ..Default::default()
        },
    );
    r.sim.general.default_spark_system = Some("BURST".into());
    r.sim.weapons.insert("ZAP".into(), zapper());
    r.sim.vehicles.get_mut("CART").unwrap().primary = Some("ZAP".into());
    r.sim.vehicles.get_mut("BUGGY").unwrap().strength = Some(30000);
    r.sim.infantry.get_mut("WALKER").unwrap().primary = Some("ZAP".into());
    r.sim.buildings.get_mut("TOWER").unwrap().primary = Some("ZAP".into());
    r
}

/// The rules with ZAP changed by `f`.
fn with_zap(f: impl FnOnce(&mut WeaponType)) -> Ruleset {
    let mut r = rules();
    f(r.sim.weapons.get_mut("ZAP").unwrap());
    r
}

/// A world of `r` with an Alpha `firer` of `kind` two cells west of the
/// middle facing east, and a Beta BUGGY four cells east of it, set as its
/// target.
fn duel_with(r: &Ruleset, kind: Kind, firer: &str) -> (World, ObjectId, ObjectId) {
    let mut w = testkit::game_with(r, N, 7);
    let a = testkit::spawn(&mut w, kind, firer, 0, testkit::mid(N, -2, 0), EAST);
    let b = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(N, 2, 0),
        WEST,
    );
    w.obj_mut(a).unwrap().target = Some(Target::Object(b));
    (w, a, b)
}

fn duel(r: &Ruleset) -> (World, ObjectId, ObjectId) {
    duel_with(r, Kind::Vehicle, "CART")
}

/// The bolts the effects report, as (slot, burst, from, to, building,
/// vehicle on the map).
fn bolts(w: &World) -> Vec<(usize, i32, Pos, Pos, Option<Pos>, bool)> {
    w.effects
        .iter()
        .filter_map(|e| match *e {
            Effect::Bolt {
                slot,
                burst,
                from,
                to,
                building,
                vehicle_on_map,
                ..
            } => Some((slot, burst, from, to, building, vehicle_on_map)),
            _ => None,
        })
        .collect()
}

/// The positions of the systems of type BURST.
fn sparks(w: &World) -> Vec<Pos> {
    w.gas
        .systems
        .values()
        .filter(|s| w.rules().gas.systems[s.type_index as usize].name == "BURST")
        .map(|s| s.pos)
        .collect()
}

fn round_trip(w: &World) {
    let bytes = save::save(w);
    let back = save::load(&bytes, w.rules_arc(), testkit::catalogue()).expect("loads");
    assert_eq!(save::state_hash(&back), save::state_hash(w));
}

/// Fires the firer's primary at its target.
fn fire_at_target(w: &mut World, a: ObjectId) -> bool {
    let t = w.obj(a).unwrap().target.unwrap();
    fire::shot(w, a, 0, t)
}

/// spec: S-0316 claim 1 (a shot that put its projectile on the map makes one bolt and one spark system) and claim 3 (the start the fire point of the slot that fired, the end the shot target's target point by S-0518 claim 20) and claim 6 (the spark system of the game-wide spark type at the bolt end)
#[test]
fn claim_1_one_bolt_and_one_spark_system() {
    let (mut w, a, b) = duel(&rules());
    assert!(sparks(&w).is_empty());
    let fp = position::fire_point(&w, a, 0, 0);
    let end = range::target_point(&w, Target::Object(b));
    assert!(fire_at_target(&mut w, a));
    assert_eq!(bolts(&w), vec![(0, 0, fp, end, None, true)]);
    assert_eq!(sparks(&w), vec![end], "one system at the bolt end");
    round_trip(&w);
}

/// spec: S-0316 claim 1 (each shot of a burst makes its own bolt and spark system) and claim 3 (the start on the side of the shot's own burst counter k)
#[test]
fn claim_1_each_burst_shot_its_own_bolt() {
    let (mut w, a, _) = duel(&with_zap(|z| z.burst = Some(2)));
    assert!(fire_at_target(&mut w, a));
    assert!(fire_at_target(&mut w, a));
    let got = bolts(&w);
    assert_eq!(got.len(), 2);
    assert_eq!((got[0].1, got[1].1), (0, 1));
    assert_eq!(got[0].2, position::fire_point(&w, a, 0, 0));
    assert_eq!(got[1].2, position::fire_point(&w, a, 0, 1));
    assert_eq!(sparks(&w).len(), 2);
}

/// spec: S-0316 claim 2 (the laser flag wins over the electric-bolt flag: a laser, no bolt and no bolt sparks)
#[test]
fn claim_2_laser_wins_over_the_bolt() {
    let (mut w, a, _) = duel(&with_zap(|z| z.is_laser = Some(true)));
    assert!(fire_at_target(&mut w, a));
    assert!(bolts(&w).is_empty());
    assert!(sparks(&w).is_empty());
    assert!(
        w.effects.iter().any(|e| matches!(e, Effect::Line { .. })),
        "the line effect that draws the laser is still reported"
    );
}

/// spec: S-0316 claim 2 (the electric-bolt flag wins over the magnetic-beam flag: the bolt, no magnetic wave, the shot still fired; and over the radiation-beam flag)
#[test]
fn claim_2_bolt_wins_over_mag_and_rad_beams() {
    let (mut w, a, _) = duel(&with_zap(|z| {
        z.is_mag_beam = Some(true);
        z.is_rad_beam = Some(true);
    }));
    assert!(fire_at_target(&mut w, a), "the shot is fired");
    assert_eq!(bolts(&w).len(), 1);
    assert!(w.waves.is_empty(), "no magnetic wave");
    // Without the bolt flag the same weapon makes its wave.
    let (mut w, a, _) = duel(&with_zap(|z| {
        z.is_mag_beam = Some(true);
        z.is_electric_bolt = None;
    }));
    assert!(fire_at_target(&mut w, a));
    assert!(bolts(&w).is_empty());
    assert_eq!(w.waves.len(), 1, "the control makes a wave");
}

/// spec: S-0316 claim 1 (no bolt and no sparks for a firing that launches no projectile: a weapon with no projectile, a spawner's, a suicide's and a drain attempt)
#[test]
fn claim_1_no_bolt_without_a_launched_projectile() {
    type Change = fn(&mut WeaponType);
    let cases: [(&str, Change); 4] = [
        ("no projectile", |z| z.projectile = None),
        ("spawner", |z| z.spawner = Some(true)),
        ("suicide", |z| z.suicide = Some(true)),
        ("drain", |z| z.drain_weapon = Some(true)),
    ];
    for (name, f) in cases {
        let (mut w, a, _) = duel(&with_zap(f));
        fire_at_target(&mut w, a);
        assert!(bolts(&w).is_empty(), "{name}: no bolt");
        assert!(sparks(&w).is_empty(), "{name}: no sparks");
    }
}

/// spec: S-0316 claim 1 (a disc laser's shot launches no projectile: no bolt)
#[test]
fn claim_1_no_bolt_for_a_disc_laser() {
    let (mut w, a, _) = duel(&with_zap(|z| z.disk_laser = Some(true)));
    fire_at_target(&mut w, a);
    assert!(bolts(&w).is_empty());
    assert!(sparks(&w).is_empty());
}

/// spec: S-0316 claim 3 (a redirected shot, here an area-fire weapon's: the end is the redirected cell's centre at its height)
#[test]
fn claim_3_area_fire_end_is_the_redirected_cell() {
    let (mut w, a, _) = duel(&with_zap(|z| z.area_fire = Some(true)));
    let cell = w.obj(a).unwrap().pos.cell();
    let end = range::target_point(&w, Target::Cell(cell));
    assert!(fire_at_target(&mut w, a));
    let got = bolts(&w);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].3, end);
    assert_eq!(sparks(&w), vec![end]);
}

/// spec: S-0316 claim 3 (a ground target: the end is the cell's centre at its height) and claim 6 (sparks at the end whatever the outcome)
#[test]
fn claim_3_ground_target() {
    let (mut w, a, _) = duel(&rules());
    let cell = testkit::mid(N, 1, 1);
    let end = range::target_point(&w, Target::Cell(cell));
    assert!(fire::shot(&mut w, a, 0, Target::Cell(cell)));
    assert_eq!(bolts(&w)[0].3, end);
    assert_eq!(sparks(&w), vec![end]);
}

/// spec: S-0316 claim 4 (only a vehicle on the map can have its bolt attached: an infantry or building firer reports no vehicle) and F4 (a building firer carries its position for the building-source adjustment)
#[test]
fn claim_4_firer_kinds() {
    let (mut w, a, _) = duel_with(&rules(), Kind::Infantry, "WALKER");
    assert!(fire_at_target(&mut w, a));
    let got = bolts(&w);
    assert_eq!(got.len(), 1);
    assert_eq!((got[0].4, got[0].5), (None, false), "infantry");
    let (mut w, a, _) = duel_with(&rules(), Kind::Building, "TOWER");
    assert!(fire_at_target(&mut w, a));
    let got = bolts(&w);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].4, Some(w.obj(a).unwrap().pos), "building position");
    assert!(!got[0].5);
}

/// spec: S-0316 claim 6 (with no game-wide spark type there is no spark system; the bolt is still made)
#[test]
fn claim_6_absent_spark_type() {
    let mut r = rules();
    r.sim.general.default_spark_system = None;
    let (mut w, a, _) = duel(&r);
    let n = w.gas.systems.len();
    assert!(fire_at_target(&mut w, a));
    assert_eq!(bolts(&w).len(), 1);
    assert_eq!(w.gas.systems.len(), n);
}

/// spec: S-0316 claim 13 (the bolt takes no draw from the scenario stream: its draws are the drawing's; only the spark system's creation, S-0616, takes scenario draws)
#[test]
fn claim_13_the_bolt_takes_no_scenario_draw() {
    let mut no_sparks = rules();
    no_sparks.sim.general.default_spark_system = None;
    let mut plain = no_sparks.clone();
    plain.sim.weapons.get_mut("ZAP").unwrap().is_electric_bolt = None;
    let (mut w, a, _) = duel(&no_sparks);
    let (mut w0, a0, _) = duel(&plain);
    let before = w.rng.taken();
    let before0 = w0.rng.taken();
    assert!(fire_at_target(&mut w, a));
    assert!(fire_at_target(&mut w0, a0));
    assert_eq!(w.rng.taken() - before, w0.rng.taken() - before0);
    assert_eq!(bolts(&w).len(), 1);
}
