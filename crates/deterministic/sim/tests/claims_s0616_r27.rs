//! Claim tests for S-0616 revision 27 (intake 72): the place-and-kind
//! reading that S-0616 takes in from S-0301 claim 5 (claims 1, 20 and 22,
//! F14), a marked system's successors and bursts (Terms *Marked*, claims 8
//! and 15, scenario P1 part b), and the fire place's let-go of a gas-kind or
//! no-kind system (claim 2; S-0613 revision 13, claim 17).
//!
//! Every test runs on synthetic data: invented particle, system, weapon and
//! object types, every number our own. Shots go through `fire::shot`, move
//! orders through `orders::issue` and the tick pipeline (`testkit::run`).

use std::collections::BTreeSet;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, ParticleSystemType, ParticleType, WeaponType};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::combat::fire;
use glasnost_sim::emitters::{self, Place};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};
use glasnost_sim::{gas, testkit};

/// Map size of every test here.
const N: i32 = 32;
/// Facing east (S-0204).
const EAST: u16 = 16384;
/// Facing west.
const WEST: u16 = 49152;
/// The rate of fire of every weapon here.
const ROF: i32 = 10;
/// The rules' gravity in these tests (a test value).
const GRAVITY: i32 = 3;

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

/// The flag of an effect weapon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Flag {
    Fire,
    Spark,
    Railgun,
}

fn place(flag: Flag) -> Place {
    match flag {
        Flag::Fire => Place::Fire,
        Flag::Spark => Place::Spark,
        Flag::Railgun => Place::Railgun,
    }
}

fn weapon_name(flag: Flag, sys: &str) -> String {
    format!("{flag:?}-{sys}").to_uppercase()
}

/// The systems every flag gets a weapon for.
const SYSTEMS: [&str; 6] = ["RAIL", "BLAZE", "NOKIND", "CLOUD", "BURST0", "BURST"];

/// The rules: testkit's combat rules with a durable unarmed BUGGY, and
/// - particles FLAME (fire), SPARK1 (spark), LINE (railgun), VAPOR (gas),
///   SOOT (smoke, life 12, radius 40, next type ASH) and ASH (smoke, life 8);
/// - systems RAIL (railgun, LINE), BLAZE (fire, FLAME), NOKIND (no kind),
///   CLOUD (gas, VAPOR), BURST0 (spark, SPARK1, spark-turns 0), BURST
///   (spark, SPARK1, 400 spark turns, chance 1) and PLUME (smoke, SOOT,
///   spawn interval 1, never marked by its timer);
/// - for every flag and every system of [`SYSTEMS`], the weapon
///   `<FLAG>-<SYSTEM>` of rate [`ROF`], range 10 cells and that one flag.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.general.gravity = Some(GRAVITY);
    let p = |kind: &str| ParticleType {
        behaves_like: Some(kind.into()),
        max_ec: Some(1000),
        end_state_ai: Some(100),
        state_ai_advance: Some(4),
        ..Default::default()
    };
    for (n, t) in [
        (
            "FLAME",
            ParticleType {
                velocity: dec("6.0"),
                ..p("fire")
            },
        ),
        (
            "SPARK1",
            ParticleType {
                x_velocity: Some(1),
                y_velocity: Some(1),
                z_velocity_range: Some(1),
                min_z_velocity: Some(2 * GRAVITY + 4),
                max_ec: Some(20),
                ..p("spark")
            },
        ),
        ("LINE", p("railgun")),
        ("VAPOR", p("gas")),
        (
            "SOOT",
            ParticleType {
                velocity: dec("4.0"),
                deacc: dec("0.5"),
                radius: Some(40),
                max_ec: Some(12),
                next_particle: Some("ASH".into()),
                ..p("smoke")
            },
        ),
        (
            "ASH",
            ParticleType {
                max_ec: Some(8),
                ..p("smoke")
            },
        ),
    ] {
        register(&mut r, "particles", n);
        r.sim.particles.insert(n.into(), t);
    }
    let s = |kind: Option<&str>, holds: Option<&str>| ParticleSystemType {
        behaves_like: kind.map(Into::into),
        holds_what: holds.map(Into::into),
        ..Default::default()
    };
    for (n, t) in [
        (
            "RAIL",
            ParticleSystemType {
                particles_per_coord: dec("0.015625"),
                ..s(Some("railgun"), Some("LINE"))
            },
        ),
        (
            "BLAZE",
            ParticleSystemType {
                spawn_frames: Some(5),
                ..s(Some("fire"), Some("FLAME"))
            },
        ),
        ("NOKIND", s(None, None)),
        ("CLOUD", s(Some("gas"), Some("VAPOR"))),
        (
            "BURST0",
            ParticleSystemType {
                spark_spawn_frames: Some(0),
                particle_cap: Some(4),
                ..s(Some("spark"), Some("SPARK1"))
            },
        ),
        (
            "BURST",
            ParticleSystemType {
                spark_spawn_frames: Some(400),
                particle_cap: Some(4),
                spawn_spark_percentage: dec("1"),
                ..s(Some("spark"), Some("SPARK1"))
            },
        ),
        (
            "PLUME",
            ParticleSystemType {
                spawn_frames: Some(1),
                slowdown: dec("0"),
                spawn_cutoff: dec("5"),
                particle_cap: Some(1000),
                ..s(Some("smoke"), Some("SOOT"))
            },
        ),
    ] {
        register(&mut r, "particle_systems", n);
        r.sim.particle_systems.insert(n.into(), t);
    }
    for flag in [Flag::Fire, Flag::Spark, Flag::Railgun] {
        for sys in SYSTEMS {
            let mut w = WeaponType {
                damage: Some(1),
                rof: Some(ROF),
                range: dec("10"),
                speed: Some(100),
                // The slow Rocket for every weapon, so that no projectile
                // hit lands at the shot (as S-0616's other tests).
                projectile: Some("Rocket".into()),
                warhead: Some("SA".into()),
                attached_particle_system: Some(sys.into()),
                ..Default::default()
            };
            match flag {
                Flag::Fire => w.use_fire_particles = Some(true),
                Flag::Spark => w.use_spark_particles = Some(true),
                Flag::Railgun => w.is_railgun = Some(true),
            }
            r.sim.weapons.insert(weapon_name(flag, sys), w);
        }
    }
    let buggy = r.sim.vehicles.get_mut("BUGGY").unwrap();
    buggy.strength = Some(30000);
    buggy.primary = None;
    buggy.secondary = None;
    r
}

/// `rules()` with the CART armed with the one weapon `flag`-`sys`.
fn armed(flag: Flag, sys: &str) -> Ruleset {
    let mut r = rules();
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.primary = Some(weapon_name(flag, sys));
    cart.secondary = None;
    r
}

/// A world of `r` with an Alpha CART three cells west of the middle, facing
/// east, and a Beta BUGGY two cells east of the middle (in range). A few
/// ticks run first, so that orders are taken (S-0213 claim 1).
fn field(r: &Ruleset) -> (World, ObjectId, ObjectId) {
    let mut w = testkit::game_with(r, N, 7);
    testkit::run(&mut w, 6);
    let b = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(N, 2, 0),
        WEST,
    );
    let a = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "CART",
        0,
        testkit::mid(N, -1, 0),
        EAST,
    );
    (w, a, b)
}

/// A move order for `a` to the neighbouring cell to its north.
fn step_aside(w: &mut World, a: ObjectId) {
    let to = testkit::mid(N, -1, -1);
    assert!(orders::issue(
        w,
        HouseId(0),
        Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(to)),
        }
    ));
}

/// The railgun line effects reported since the last clear: from, to.
fn rail_lines(w: &World) -> Vec<(Pos, Pos)> {
    w.effects
        .iter()
        .filter_map(|e| match *e {
            Effect::RailLine { from, to, .. } => Some((from, to)),
            _ => None,
        })
        .collect()
}

fn ids(w: &World, s: ObjectId) -> BTreeSet<ObjectId> {
    w.gas.systems[&s].particles.iter().map(|p| p.id).collect()
}

fn named(w: &World, p: &glasnost_sim::gas::Particle) -> String {
    w.rules().gas.particles[p.type_index as usize].name.clone()
}

// ---------------------------------------------------------------------------
// The creation reading by place (claim 1, F14)

/// spec: S-0616 claim 1 (revision 27: for a system made for the fire or spark place, whatever its kind, the target point is the position of the shot's target object at the shot; a railgun-kind system held in the fire or spark place makes its line to that place's target point, not to F14's end point; the point is never recomputed from the target) and claim 17 (the line is made in the system's first turn, to the fixed target point)
#[test]
fn a_railgun_kind_system_in_the_fire_or_spark_place_lines_to_the_shots_target() {
    for flag in [Flag::Fire, Flag::Spark] {
        let (mut w, a, b) = field(&armed(flag, "RAIL"));
        let at_shot = w.obj(b).unwrap().pos;
        assert!(fire::shot(&mut w, a, 0, Target::Object(b)));
        let s = emitters::held(&w, a, place(flag)).expect("held in its weapon's place");
        // The target moves before the system's first turn: the point stays.
        w.obj_mut(b).unwrap().pos = Pos::new(at_shot.x + 300, at_shot.y - 200, at_shot.z);
        w.effects.clear();
        gas::system_turn(&mut w, s);
        let lines = rail_lines(&w);
        assert_eq!(lines.len(), 1, "{flag:?}: one line in its first turn");
        assert_eq!(lines[0].1, at_shot, "{flag:?}: to the target at the shot");
        assert!(
            w.gas.systems.get(&s).is_none_or(|x| x.ending),
            "{flag:?}: marked after its line"
        );
    }
}

/// spec: S-0616 claim 1 (revision 27: the fire or spark place's target point is the world origin when the shot has no target object, whatever the system's kind; the railgun place's is F14's end point, whatever its kind) and F14 (revision 27: computed at the shot that makes the system of the railgun place; on flat ground with nothing on the beam the end point is the target's own position)
#[test]
fn a_cell_shot_lines_to_the_origin_from_the_fire_place_and_to_f14_from_the_railgun_place() {
    for flag in [Flag::Fire, Flag::Spark, Flag::Railgun] {
        let (mut w, a, _) = field(&armed(flag, "RAIL"));
        // A cell south of the firer, off the BUGGY's row, so nothing is on the beam.
        let c = testkit::mid(N, -1, 4);
        assert!(fire::shot(&mut w, a, 0, Target::Cell(c)));
        let s = emitters::held(&w, a, place(flag)).expect("held in its weapon's place");
        assert_eq!(w.gas.systems[&s].target_object, None, "{flag:?}");
        w.effects.clear();
        gas::system_turn(&mut w, s);
        let lines = rail_lines(&w);
        assert_eq!(lines.len(), 1, "{flag:?}");
        let expected = if flag == Flag::Railgun {
            Pos::centre(c, 0)
        } else {
            Pos::default()
        };
        assert_eq!(lines[0].1, expected, "{flag:?}");
    }
}

/// spec: S-0616 claim 17 (revision 27: a system marked by a let-go of the fire place before its first turn) and claim 22 (revision 27: a railgun-kind system is marked, while it is held in the fire place, when that place is let go) and S-0301 claim 5, Moment of a retarget's let-go (the test at the change of the target): with the mark before its first turn it makes no line, takes no draw and leaves play in that turn, and the place is free
#[test]
fn a_railgun_kind_system_let_go_before_its_first_turn_makes_no_line() {
    let (mut w, a, b) = field(&armed(Flag::Fire, "RAIL"));
    let far = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(N, -1, 14),
        WEST,
    );
    w.obj_mut(a).unwrap().target = Some(Target::Object(b));
    assert!(fire::shot(&mut w, a, 0, Target::Object(b)));
    let s = emitters::held(&w, a, Place::Fire).unwrap();
    w.obj_mut(a).unwrap().target = Some(Target::Object(far));
    assert!(emitters::retarget_let_go(&mut w, a), "out of reach: let go");
    assert_eq!(
        emitters::held(&w, a, Place::Fire),
        None,
        "the place is free"
    );
    assert!(w.gas.systems[&s].ending, "marked");
    w.effects.clear();
    let before = w.rng.taken();
    gas::system_turn(&mut w, s);
    assert!(rail_lines(&w).is_empty(), "no line");
    assert_eq!(w.rng.taken(), before, "no draw");
    assert!(!w.gas.systems.contains_key(&s), "it leaves play");
}

// ---------------------------------------------------------------------------
// The fire place's let-go of a no-kind or gas-kind system

/// The three let-go events of the fire place (claim 1).
#[derive(Clone, Copy, Debug)]
enum Event {
    Retarget,
    Move,
    Death,
}

/// spec: S-0616 claim 1 (revision 27: the system held in the fire place, whatever its kind, is marked and let go when its firer takes a new target not within reach, is given a move order, or dies; a let-go marks whatever system that place holds), claim 2 (revision 27: a system with no behaviour kind leaves play when, while held in the fire place, a let-go of that place marks it and it holds no particle) and claim 22 (revision 27: the no-kind row and the gas-kind sentence) and S-0613 claim 17 (revision 13: a let-go of the fire place marks the system that place holds, a gas-kind system included)
#[test]
fn a_no_kind_or_gas_kind_system_in_the_fire_place_is_marked_by_a_let_go() {
    for sys in ["NOKIND", "CLOUD"] {
        for event in [Event::Retarget, Event::Move, Event::Death] {
            let name = format!("{sys}, {event:?}");
            let (mut w, a, b) = field(&armed(Flag::Fire, sys));
            w.obj_mut(a).unwrap().target = Some(Target::Object(b));
            assert!(fire::shot(&mut w, a, 0, Target::Object(b)), "{name}");
            let s = emitters::held(&w, a, Place::Fire).expect("held in the fire place");
            // Its own turns do not mark it while its owner is alive.
            for _ in 0..3 {
                gas::system_turn(&mut w, s);
            }
            assert!(!w.gas.systems[&s].ending, "{name}: not marked by its turns");
            assert_eq!(emitters::held(&w, a, Place::Fire), Some(s), "{name}");
            match event {
                Event::Retarget => {
                    let far = testkit::spawn(
                        &mut w,
                        Kind::Vehicle,
                        "BUGGY",
                        1,
                        testkit::mid(N, -1, 14),
                        WEST,
                    );
                    w.obj_mut(a).unwrap().target = Some(Target::Object(far));
                    assert!(emitters::retarget_let_go(&mut w, a), "{name}");
                    assert!(w.gas.systems[&s].ending, "{name}: marked at the change");
                    gas::system_turn(&mut w, s);
                }
                Event::Move => {
                    step_aside(&mut w, a);
                    // The order applies in this tick (after the turn pass);
                    // the system's turn in the next tick finds it marked.
                    testkit::run(&mut w, 2);
                }
                Event::Death => {
                    w.obj_mut(a).unwrap().health = 0;
                    gas::system_turn(&mut w, s);
                }
            }
            assert_eq!(emitters::held(&w, a, Place::Fire), None, "{name}: let go");
            assert!(
                !w.gas.systems.contains_key(&s),
                "{name}: marked, holding no particle, it leaves play"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Claim 20 in the fire place

/// spec: S-0616 claim 20 (revision 27: our engine's fix applies to every spark system of a type with a spark-turns field of 0 or less, whatever place holds it: it is marked in its first turn, takes no burst and no draw of claim 15, leaves play once empty and frees its place, so a later let-go of the fire place finds the place free)
#[test]
fn claim_20_fix_frees_the_fire_place_after_one_turn() {
    let (mut w, a, b) = field(&armed(Flag::Fire, "BURST0"));
    w.obj_mut(a).unwrap().target = Some(Target::Object(b));
    assert!(fire::shot(&mut w, a, 0, Target::Object(b)));
    let s = emitters::held(&w, a, Place::Fire).expect("held in the fire place");
    let before = w.rng.taken();
    gas::system_turn(&mut w, s);
    assert_eq!(w.rng.taken(), before, "no burst, no draw");
    assert!(
        !w.gas.systems.contains_key(&s),
        "it leaves in its first turn"
    );
    assert_eq!(
        emitters::held(&w, a, Place::Fire),
        None,
        "the place is free"
    );
    // A let-go now finds the place free and marks nothing.
    let marks = |w: &World| -> Vec<(ObjectId, bool)> {
        w.gas.systems.iter().map(|(k, x)| (*k, x.ending)).collect()
    };
    let before = marks(&w);
    emitters::let_go_fire(&mut w, a);
    assert_eq!(marks(&w), before, "no system is marked");
    assert_eq!(emitters::held(&w, a, Place::Fire), None);
}

// ---------------------------------------------------------------------------
// Damage sparks and the spark place

/// spec: S-0616 claim 1 (revision 27: damage sparks need nothing in the spark place, so a fire-kind system that a spark-flag weapon put there stops them as a spark emitter does: no system and no draw)
#[test]
fn damage_sparks_wait_for_an_empty_spark_place_whatever_it_holds() {
    let mut r = rules();
    let walker = r.sim.infantry.get_mut("WALKER").unwrap();
    walker.cyborg = Some(true);
    walker.strength = Some(100);
    walker.primary = Some(weapon_name(Flag::Spark, "BLAZE"));
    walker.secondary = None;
    walker.combat.damage_spark_systems = Some(vec!["BURST".into()]);
    r.sim.general.condition_yellow_sparking_probability = dec("1.0");
    r.sim.general.condition_red_sparking_probability = dec("1.0");
    let mut w = testkit::game_with(&r, N, 7);
    testkit::run(&mut w, 6);
    let b = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(N, 2, 0),
        WEST,
    );
    let i = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        0,
        testkit::mid(N, -1, 0),
        EAST,
    );
    assert!(fire::shot(&mut w, i, 0, Target::Object(b)));
    let s = emitters::held(&w, i, Place::Spark).expect("the fire-kind system in the spark place");
    w.obj_mut(i).unwrap().health = 20;
    let before = w.rng.taken();
    emitters::damage_sparks(&mut w, i);
    assert_eq!(w.rng.taken(), before, "no draw");
    assert_eq!(
        emitters::held(&w, i, Place::Spark),
        Some(s),
        "no new system"
    );
    // Control: with the place empty, the same object makes damage sparks.
    w.obj_mut(i).unwrap().combat.spark_emitter = None;
    emitters::damage_sparks(&mut w, i);
    assert!(w.rng.taken() > before, "its draws");
    assert!(
        emitters::held(&w, i, Place::Spark).is_some_and(|x| x != s),
        "a damage-sparks system"
    );
}

// ---------------------------------------------------------------------------
// A marked system's successors and bursts (scenario P1 part b)

/// spec: S-0616 Terms (revision 27, *Marked*: a marked system makes no emission, though the mark does not stop a spark system's bursts while its K is above 0, and a smoke system's dead particles still split into successors; it leaves play in the first turn in which it holds no particle), claim 1 (the trigger action that marks the systems of a cell), claim 8 (revision 27: in a marked system) and claim 15 (revision 27: the table reads only K) and scenario P1 part b (revision 27)
#[test]
fn p1_part_b_a_trigger_mark_stops_emission_but_not_successors_or_bursts() {
    let mut w = testkit::game_with(&rules(), N, 7);
    let c = testkit::mid(N, 0, 0);
    let p = Pos::centre(c, 0);
    gas::warhead_system(&mut w, "PLUME", p, None);
    let smoke = *w.gas.systems.keys().last().unwrap();
    gas::warhead_system(&mut w, "BURST", p, None);
    let spark = *w.gas.systems.keys().last().unwrap();
    // A few ticks: the smoke system emits each tick and holds particles.
    testkit::run(&mut w, 6);
    assert!(!w.gas.systems[&smoke].particles.is_empty());
    assert!(!w.gas.systems[&spark].particles.is_empty());
    emitters::trigger_mark(&mut w, c);
    assert!(w.gas.systems[&smoke].ending && w.gas.systems[&spark].ending);
    let k = w.gas.systems[&spark].sparks;
    assert!(k > 5, "the spark system's K is above 0 at the mark");
    let mut successors = 0;
    let mut smoke_left = None;
    for t in 0..80 {
        let smoke_before = w.gas.systems.get(&smoke).map(|_| ids(&w, smoke));
        let spark_before = ids(&w, spark);
        testkit::run(&mut w, 1);
        // The spark system bursts in every turn while its K is above 0.
        if t < 5 {
            assert!(
                w.gas.systems[&spark]
                    .particles
                    .iter()
                    .any(|q| !spark_before.contains(&q.id)),
                "turn {t}: a burst while marked"
            );
        }
        let Some(before) = smoke_before else { continue };
        match w.gas.systems.get(&smoke) {
            None => {
                smoke_left.get_or_insert(t);
            }
            Some(x) => {
                assert!(
                    !x.particles.is_empty(),
                    "turn {t}: in play only while it holds one"
                );
                for q in x.particles.iter().filter(|q| !before.contains(&q.id)) {
                    assert_eq!(
                        named(&w, q),
                        "ASH",
                        "turn {t}: no emission, only successors"
                    );
                    successors += 1;
                }
            }
        }
    }
    assert!(successors >= 2, "dead SOOT particles split after the mark");
    assert!(smoke_left.is_some(), "the marked smoke system leaves play");
}
