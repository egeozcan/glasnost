//! Claim tests for S-0301 revision 63 (intake 63, Q-2181): claim 5's
//! place-and-kind rule for a particle emitter whose system's kind does not
//! match its weapon's flag, scenario F4's unmatched part, and the creation
//! reading of the change for S-0616 (c).
//!
//! Every refire test runs the real tick pipeline (`testkit::run`) on
//! synthetic data: invented particle, system and weapon types, every number
//! our own. A firer's shots are read from its last-fired tick at the end of
//! every tick, and a system's leaving play from the system list.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, ParticleSystemType, ParticleType, WeaponType};
use glasnost_sim::combat::fire;
use glasnost_sim::emitters::{self, Place};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{gas, testkit};

/// Map size of every test here.
const N: i32 = 24;
/// Facing east (S-0204).
const EAST: u16 = 16384;
/// Facing west.
const WEST: u16 = 49152;
/// The rate of fire of every weapon here, shorter than every system's life.
const ROF: i32 = 10;

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

/// The rules: testkit's combat rules with a durable unarmed BUGGY, and
/// - particles FLAME (fire), SPARK (spark) and LINE (railgun);
/// - systems BLAZE (fire kind, FLAME, spawn interval 5, lifetime absent),
///   BLAZE45 (BLAZE with a lifetime of 45 turns), BURST (spark kind,
///   SPARK, 12 spark turns, cap 4), LONGBURST (spark kind, SPARK, 60 spark
///   turns, cap 4, spark chance 0) and RAIL (railgun kind, LINE);
/// - for every flag and system name, the weapon `<FLAG>-<SYSTEM>` of rate
///   [`ROF`], range 10 cells and that one effect flag.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
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
            "SPARK",
            ParticleType {
                x_velocity: Some(5),
                y_velocity: Some(5),
                min_z_velocity: Some(3),
                z_velocity_range: Some(4),
                ..p("spark")
            },
        ),
        ("LINE", p("railgun")),
    ] {
        register(&mut r, "particles", n);
        r.sim.particles.insert(n.into(), t);
    }
    let s = |kind: &str, holds: &str| ParticleSystemType {
        behaves_like: Some(kind.into()),
        holds_what: Some(holds.into()),
        ..Default::default()
    };
    let systems = [
        (
            "BLAZE",
            ParticleSystemType {
                spawn_frames: Some(5),
                ..s("fire", "FLAME")
            },
        ),
        (
            "BLAZE45",
            ParticleSystemType {
                spawn_frames: Some(5),
                lifetime: Some(45),
                ..s("fire", "FLAME")
            },
        ),
        (
            "BURST",
            ParticleSystemType {
                spark_spawn_frames: Some(12),
                particle_cap: Some(4),
                ..s("spark", "SPARK")
            },
        ),
        (
            "LONGBURST",
            ParticleSystemType {
                spark_spawn_frames: Some(60),
                particle_cap: Some(4),
                spawn_spark_percentage: dec("0"),
                ..s("spark", "SPARK")
            },
        ),
        (
            "RAIL",
            ParticleSystemType {
                particles_per_coord: dec("0.015625"),
                ..s("railgun", "LINE")
            },
        ),
    ];
    for (n, t) in systems.clone() {
        register(&mut r, "particle_systems", n);
        r.sim.particle_systems.insert(n.into(), t);
    }
    for flag in [Flag::Fire, Flag::Spark, Flag::Railgun] {
        for (sys, _) in &systems {
            let mut w = WeaponType {
                damage: Some(1),
                rof: Some(ROF),
                range: dec("10"),
                speed: Some(100),
                // The slow Rocket for the railgun flag, so that no
                // projectile hit lands at the shot (as S-0616's tests).
                projectile: Some(
                    if flag == Flag::Railgun {
                        "Rocket"
                    } else {
                        "Bullet"
                    }
                    .into(),
                ),
                warhead: Some("SA".into()),
                attached_particle_system: Some((*sys).into()),
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

fn weapon_name(flag: Flag, sys: &str) -> String {
    format!("{flag:?}-{sys}").to_uppercase()
}

fn place(flag: Flag) -> Place {
    match flag {
        Flag::Fire => Place::Fire,
        Flag::Spark => Place::Spark,
        Flag::Railgun => Place::Railgun,
    }
}

/// `rules()` with the CART armed with the one weapon `flag`-`sys` and no
/// secondary, so that no other weapon's gate plays a part.
fn armed(flag: Flag, sys: &str) -> Ruleset {
    let mut r = rules();
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.primary = Some(weapon_name(flag, sys));
    cart.secondary = None;
    r
}

/// A world of `r` with a Beta BUGGY two cells east of the middle and one
/// Alpha CART per row in `rows`, each three cells west of the BUGGY's
/// column, facing east. A few ticks run before the objects are placed, so
/// that orders are taken (S-0213 claim 1) and no shot comes before the
/// test's own.
fn field(r: &Ruleset, rows: &[i32]) -> (World, Vec<ObjectId>, ObjectId) {
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
    let carts = rows
        .iter()
        .map(|&y| {
            testkit::spawn(
                &mut w,
                Kind::Vehicle,
                "CART",
                0,
                testkit::mid(N, -1, y),
                EAST,
            )
        })
        .collect();
    (w, carts, b)
}

fn attack(w: &mut World, a: ObjectId, b: ObjectId) {
    assert!(orders::issue(
        w,
        HouseId(0),
        Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(b)),
            dest: None,
        }
    ));
}

/// A move order to the cell `to`, a neighbour of `a`'s from which the
/// BUGGY stays within range.
fn step_aside(w: &mut World, a: ObjectId, to: glasnost_map::CellXY) {
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

/// One firer's run: its shot ticks and, for each shot, the tick at whose
/// end the system its place took at that shot was no longer in play.
#[derive(Debug, Default)]
struct Run {
    shots: Vec<i32>,
    systems: Vec<ObjectId>,
    left: Vec<Option<i32>>,
}

/// Runs `n` ticks, reading every firer of `firers` (in its `flag`'s place)
/// at the end of each tick. `at` is called before each tick with the tick
/// number, to give orders.
fn record(
    w: &mut World,
    firers: &[(ObjectId, Flag)],
    n: i32,
    mut at: impl FnMut(&mut World, i32, &[Run]),
) -> Vec<Run> {
    let mut runs: Vec<Run> = firers.iter().map(|_| Run::default()).collect();
    for _ in 0..n {
        let tick = w.tick;
        at(w, tick, &runs);
        let before: Vec<i32> = firers
            .iter()
            .map(|&(a, _)| w.obj(a).unwrap().last_fired)
            .collect();
        testkit::run(w, 1);
        for (k, &(a, flag)) in firers.iter().enumerate() {
            let r = &mut runs[k];
            if w.obj(a).unwrap().last_fired != before[k] {
                r.shots.push(tick);
                let s = emitters::held(w, a, place(flag)).expect("the shot fills its place");
                r.systems.push(s);
                r.left.push(None);
            }
            for (i, s) in r.systems.iter().enumerate() {
                if r.left[i].is_none() && !w.gas.systems.contains_key(s) {
                    r.left[i] = Some(tick);
                }
            }
        }
    }
    runs
}

/// The scenario's Void line: no firer was removed or destroyed.
fn not_void(w: &World, firers: &[(ObjectId, Flag)]) {
    for &(a, _) in firers {
        assert!(
            w.obj(a).is_some_and(|o| o.health > 0),
            "void: a firer is gone"
        );
    }
}

// ---------------------------------------------------------------------------
// Scenario F4, unmatched part, first run

/// The first run for one firer: the flag-system pairing `flag`-`sys`, its
/// target kept and standing still, for `n` ticks.
fn first_run(flag: Flag, sys: &str, n: i32) -> Run {
    let (mut w, c, b) = field(&armed(flag, sys), &[0]);
    attack(&mut w, c[0], b);
    let firers = [(c[0], flag)];
    let mut runs = record(&mut w, &firers, n, |_, _, _| {});
    not_void(&w, &firers);
    assert_eq!(
        w.obj(c[0]).unwrap().target,
        Some(Target::Object(b)),
        "it keeps its target"
    );
    runs.remove(0)
}

/// Asserts the refire of a pairing whose place is freed by its system's
/// leaving play: it shoots again at its first attempt after that, and not
/// earlier, with every system leaving play later than the rate of fire.
fn refires_after_leaving(name: &str, r: &Run) {
    assert!(r.shots.len() >= 2, "{name}: shoots a second time: {r:?}");
    for i in 0..r.shots.len() - 1 {
        let left = r.left[i].unwrap_or_else(|| panic!("{name}: system {i} leaves play: {r:?}"));
        assert!(
            left - r.shots[i] > ROF,
            "{name}: the system's life is longer than the rate: {r:?}"
        );
        // The firer's next attempt after the place is freed: the tick after
        // the one at whose end the system was gone, give or take the
        // processing interval of an attacking unit.
        assert!(
            r.shots[i + 1] > left,
            "{name}: not before its system has left play: {r:?}"
        );
        assert!(
            r.shots[i + 1] <= left + 2,
            "{name}: at its first attempt after: {r:?}"
        );
    }
}

/// spec: S-0301 claim 5, place-and-kind rule (revision 63, Q-2181: fire flag, spark-kind or railgun-kind system: the weapon fires again at its first attempt at least R′ after its shot and after its system has left play, with no let-go event) and scenario F4, unmatched part (u1, u2: shoot again at the first attempt after their system has left play, and not earlier)
#[test]
fn f4_unmatched_u1_u2_fire_flag() {
    refires_after_leaving("u1", &first_run(Flag::Fire, "BURST", 240));
    refires_after_leaving("u2", &first_run(Flag::Fire, "RAIL", 3000));
}

/// spec: S-0301 claim 5, place-and-kind rule (revision 63, Q-2181: spark or railgun flag, spark-kind or railgun-kind system: the place is freed only by its system leaving play by its own counter or line) and scenario F4, unmatched part (u5, u6)
#[test]
fn f4_unmatched_u5_u6_spark_and_railgun_places() {
    refires_after_leaving("u5", &first_run(Flag::Spark, "RAIL", 3000));
    refires_after_leaving("u6", &first_run(Flag::Railgun, "BURST", 240));
}

/// spec: S-0301 claim 5, place-and-kind rule (revision 63, Q-2181: spark or railgun flag, fire-kind system: no let-go event frees the place and the system is not marked by its own turn while its owner is alive and on the map, so the weapon does not fire again; the fire system goes on making its flames) and scenario F4, unmatched part (u3, u4: shoot once only)
#[test]
fn f4_unmatched_u3_u4_fire_kind_in_the_spark_and_railgun_places() {
    for (name, flag) in [("u3", Flag::Spark), ("u4", Flag::Railgun)] {
        let (mut w, c, b) = field(&armed(flag, "BLAZE"), &[0]);
        attack(&mut w, c[0], b);
        let firers = [(c[0], flag)];
        let runs = record(&mut w, &firers, 300, |_, _, _| {});
        not_void(&w, &firers);
        let r = &runs[0];
        assert_eq!(r.shots.len(), 1, "{name}: shoots once only: {r:?}");
        let s = r.systems[0];
        let sys = &w.gas.systems[&s];
        assert!(!sys.ending, "{name}: not marked by its own turn");
        assert!(
            !sys.particles.is_empty(),
            "{name}: it goes on making flames"
        );
        assert_eq!(emitters::held(&w, c[0], place(flag)), Some(s));
    }
}

/// spec: S-0301 claim 5, place-and-kind rule (revision 63, Q-2181: every kind is also marked by a positive lifetime, S-0613 claim 17) and scenario F4, unmatched part (u7: u3 with a positive lifetime shoots again at the first attempt after its system has left play)
#[test]
fn f4_unmatched_u7_positive_lifetime() {
    let r = first_run(Flag::Spark, "BLAZE45", 3000);
    refires_after_leaving("u7", &r);
    assert!(
        r.left[0].unwrap() - r.shots[0] >= 45,
        "not before its lifetime: {r:?}"
    );
}

// ---------------------------------------------------------------------------
// Scenario F4, unmatched part, second run

/// spec: S-0301 claim 5, place-and-kind rule (revision 63, Q-2181: a move order is a let-go event of the fire place and marks the system it holds whatever its kind, which then leaves play in its first turn holding no particle; a move order frees neither the spark nor the railgun place) and scenario F4, unmatched part, second run (u1 against u8; u3 and u4 shoot no second time)
#[test]
fn f4_unmatched_second_run() {
    // Rows: u1 and u8 (fire flag, LONGBURST), u3 (spark flag, BLAZE) and
    // u4 (railgun flag, BLAZE). Each row has its own CART type
    // armed with that one weapon.
    let mut r = rules();
    for (ty, w) in [
        ("CARTU1", weapon_name(Flag::Fire, "LONGBURST")),
        ("CARTU8", weapon_name(Flag::Fire, "LONGBURST")),
        ("CARTU3", weapon_name(Flag::Spark, "BLAZE")),
        ("CARTU4", weapon_name(Flag::Railgun, "BLAZE")),
    ] {
        let mut t = r.sim.vehicles["CART"].clone();
        t.primary = Some(w);
        t.secondary = None;
        register(&mut r, "vehicles", ty);
        r.sim.vehicles.insert(ty.into(), t);
    }
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
    let mk = |w: &mut World, ty: &str, y: i32| {
        testkit::spawn(w, Kind::Vehicle, ty, 0, testkit::mid(N, -1, y), EAST)
    };
    // u8 is placed and ordered first, so that its first shot comes in u1's
    // tick or earlier.
    let u8_ = mk(&mut w, "CARTU8", -1);
    let u1 = mk(&mut w, "CARTU1", -3);
    let u3 = mk(&mut w, "CARTU3", 1);
    let u4 = mk(&mut w, "CARTU4", 3);
    for a in [u8_, u1, u3, u4] {
        attack(&mut w, a, b);
    }
    let firers = [
        (u1, Flag::Fire),
        (u8_, Flag::Fire),
        (u3, Flag::Spark),
        (u4, Flag::Railgun),
    ];
    // After each first shot of u1, u3 and u4: a move order to a
    // neighbouring cell, then, a few ticks later, a new attack order on the
    // same target.
    let rows = [-3, -1, 1, 3];
    let mut moved = [None::<i32>; 4];
    let runs = record(&mut w, &firers, 200, |w, tick, runs| {
        for k in [0, 2, 3] {
            let a = firers[k].0;
            match moved[k] {
                None if !runs[k].shots.is_empty() => {
                    step_aside(w, a, testkit::mid(N, -1, rows[k] + 1));
                    moved[k] = Some(tick);
                }
                Some(m) if tick == m + 4 => attack(w, a, b),
                _ => {}
            }
        }
    });
    not_void(&w, &firers);
    let (r1, r8, r3, r4) = (&runs[0], &runs[1], &runs[2], &runs[3]);
    assert!(
        r8.shots[0] <= r1.shots[0],
        "u8's first shot is in u1's tick or earlier"
    );
    let left8 = r8.left[0].expect("u8's system leaves play once its counter has run out");
    // Void line: neither LONGBURST system bursts before its last counter
    // turn, so neither holds a particle before then.
    let m1 = moved[0].unwrap();
    assert!(m1 < left8, "the orders come while u8's system is in play");
    // u1's first system leaves play in its first turn after the move order.
    let left1 = r1.left[0].expect("u1's first system leaves play");
    assert!(
        left1 <= m1 + 1,
        "the let-go marked it and it held no particle: moved {m1}, left {left1}"
    );
    // u1 shoots again after the new attack order, at least R′ after its
    // first shot, before u8's system has left play.
    assert!(r1.shots.len() >= 2, "u1 shoots again: {r1:?}");
    assert!(r1.shots[1] >= m1 + 4 && r1.shots[1] >= r1.shots[0] + ROF);
    assert!(r1.shots[1] < left8, "before u8's system has left play");
    // u8's system stays in play until its counter has run out.
    assert!(left8 - r8.shots[0] >= 60, "u8 left at {left8}: {r8:?}");
    // u3 and u4 shoot no second time.
    assert_eq!(r3.shots.len(), 1, "u3: {r3:?}");
    assert_eq!(r4.shots.len(), 1, "u4: {r4:?}");
}

// ---------------------------------------------------------------------------
// The let-go events of the fire place, whatever the held system's kind

/// spec: S-0301 claim 5, place-and-kind rule (revision 63, Q-2181: the fire place's let-go events are a retarget to a target not within reach, a move order and the firer's death; a let-go marks the system the place holds, whatever its kind; revision 67, Q-2246: the retarget test is made at the change of the target) and S-0616 claim 1 (a new target within reach keeps it)
#[test]
fn fire_place_let_go_of_a_spark_kind_system() {
    // A retarget out of reach.
    let (mut w, c, b) = field(&armed(Flag::Fire, "LONGBURST"), &[0]);
    let a = c[0];
    let near = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(N, 1, 2),
        WEST,
    );
    let far = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(N, -1, 16),
        WEST,
    );
    w.obj_mut(a).unwrap().target = Some(Target::Object(b));
    assert!(fire::shot(&mut w, a, 0, Target::Object(b)));
    let s = emitters::held(&w, a, Place::Fire).expect("held in the fire place");
    // Revision 67 (Q-2246): the test is made at the change of the target,
    // so each write is followed by the engine's at-change call.
    w.obj_mut(a).unwrap().target = Some(Target::Object(near));
    emitters::retarget_let_go(&mut w, a);
    gas::system_turn(&mut w, s);
    assert_eq!(
        emitters::held(&w, a, Place::Fire),
        Some(s),
        "within reach: kept"
    );
    assert!(!w.gas.systems[&s].ending);
    w.obj_mut(a).unwrap().target = Some(Target::Object(far));
    emitters::retarget_let_go(&mut w, a);
    assert_eq!(
        emitters::held(&w, a, Place::Fire),
        None,
        "out of reach: let go"
    );
    assert!(
        w.gas.systems.get(&s).is_none_or(|x| x.ending),
        "and marked, with its counter not run out"
    );

    // The firer's death.
    let (mut w, c, b) = field(&armed(Flag::Fire, "LONGBURST"), &[0]);
    let a = c[0];
    w.obj_mut(a).unwrap().target = Some(Target::Object(b));
    assert!(fire::shot(&mut w, a, 0, Target::Object(b)));
    let s = emitters::held(&w, a, Place::Fire).unwrap();
    w.obj_mut(a).unwrap().health = 0;
    gas::system_turn(&mut w, s);
    assert_eq!(emitters::held(&w, a, Place::Fire), None, "dead: let go");
    assert!(w.gas.systems.get(&s).is_none_or(|x| x.ending), "and marked");

    // A railgun-kind system in the fire place: marked by a let-go before
    // its first turn.
    let (mut w, c, b) = field(&armed(Flag::Fire, "RAIL"), &[0]);
    let a = c[0];
    assert!(fire::shot(&mut w, a, 0, Target::Object(b)));
    let s = emitters::held(&w, a, Place::Fire).unwrap();
    emitters::let_go_fire(&mut w, a);
    assert!(w.gas.systems[&s].ending, "the let-go marks it");
    assert_eq!(emitters::held(&w, a, Place::Fire), None);
}

/// spec: S-0301 claim 5, place-and-kind rule (revision 63, Q-2181: the spark and railgun places have no let-go events; a fire-kind system held there is not marked by its own turn while its owner is alive, and is marked by it once the owner is no longer alive, freeing its place when it leaves play)
#[test]
fn spark_and_railgun_places_have_no_let_go() {
    for flag in [Flag::Spark, Flag::Railgun] {
        let (mut w, c, b) = field(&armed(flag, "BLAZE"), &[0]);
        let a = c[0];
        let far = testkit::spawn(
            &mut w,
            Kind::Vehicle,
            "BUGGY",
            1,
            testkit::mid(N, -1, 16),
            WEST,
        );
        w.obj_mut(a).unwrap().target = Some(Target::Object(b));
        assert!(fire::shot(&mut w, a, 0, Target::Object(b)));
        let s = emitters::held(&w, a, place(flag)).unwrap();
        // A retarget out of reach and the fire place's let-go do nothing here.
        w.obj_mut(a).unwrap().target = Some(Target::Object(far));
        gas::system_turn(&mut w, s);
        emitters::let_go_fire(&mut w, a);
        assert_eq!(
            emitters::held(&w, a, place(flag)),
            Some(s),
            "{flag:?}: held"
        );
        assert!(!w.gas.systems[&s].ending, "{flag:?}: not marked");
        // The owner's death: its own turn marks it; the place frees when
        // it leaves play.
        w.obj_mut(a).unwrap().health = 0;
        gas::system_turn(&mut w, s);
        assert!(w.gas.systems.get(&s).is_none_or(|x| x.ending));
        for _ in 0..4000 {
            if !w.gas.systems.contains_key(&s) {
                break;
            }
            gas::system_turn(&mut w, s);
        }
        assert!(!w.gas.systems.contains_key(&s), "{flag:?}: leaves play");
        assert_eq!(emitters::held(&w, a, place(flag)), None, "{flag:?}: freed");
    }
}

// ---------------------------------------------------------------------------
// The creation reading (S-0616 (c))

/// spec: S-0301 Changes for other specs, S-0616 (c), taken in by S-0616 claim 1 (revision 27) (revision 63: the creation follows the place: the system made for the fire or spark place takes the shot's target object and that object's position; the system made for the railgun place takes no target object, whatever the system's kind)
#[test]
fn creation_follows_the_place() {
    for (flag, sys) in [
        (Flag::Fire, "RAIL"),
        (Flag::Spark, "RAIL"),
        (Flag::Railgun, "BLAZE"),
        (Flag::Railgun, "BURST"),
    ] {
        let (mut w, c, b) = field(&armed(flag, sys), &[0]);
        let a = c[0];
        assert!(fire::shot(&mut w, a, 0, Target::Object(b)));
        let s = emitters::held(&w, a, place(flag)).unwrap();
        let x = &w.gas.systems[&s];
        assert_eq!(x.owner_object, Some(a));
        if flag == Flag::Railgun {
            assert_eq!(x.target_object, None, "{flag:?}-{sys}: no target object");
        } else {
            assert_eq!(
                x.target_object,
                Some(b),
                "{flag:?}-{sys}: the shot's target object"
            );
            assert_eq!(
                x.target,
                w.obj(b).unwrap().pos,
                "{flag:?}-{sys}: its position"
            );
        }
    }
}
