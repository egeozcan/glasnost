//! Claim tests for S-0301 revision 67 (intake 70, Q-2246): claim 5's
//! *Moment of a retarget's let-go*, scenario F4's unmatched part, third run,
//! and the change for S-0616 (d), a marked spark system's bursts.
//!
//! Every run goes through the real tick pipeline (`testkit::run`) and real
//! attack orders, on synthetic data: invented particle, system and weapon
//! types, every number our own.

use std::collections::BTreeSet;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, ParticleSystemType, ParticleType, WeaponType};
use glasnost_sim::emitters::{self, Place};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{gas, testkit};

/// Map size of every test here.
const N: i32 = 48;
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

/// The three firers of the third run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Firer {
    /// u9: u1's pairing (fire flag, spark-kind system) with a spark chance
    /// of 1, a cap of 4 and a counter that does not run out in the run.
    U9,
    /// u10: fire flag, smoke-kind system with a spawn interval of 1.
    U10,
    /// The particle part's first fire firer: fire flag, fire-kind system,
    /// spawn interval 1.
    Fire,
}

impl Firer {
    fn vehicle(self) -> &'static str {
        match self {
            Firer::U9 => "CARTU9",
            Firer::U10 => "CARTU10",
            Firer::Fire => "CARTFIRE",
        }
    }
}

/// testkit's combat rules with a durable unarmed BUGGY, gravity
/// [`GRAVITY`], and S-0301 scenario F4's third-run types (all test values):
/// - SPARK1: spark kind, x and y velocity fields 1, vertical velocity range
///   1, a minimum vertical velocity above twice the gravity, a lifetime of
///   20 ticks; held by SPARKS (spark kind, chance field 1, cap 4, 400 spark
///   turns, the default spawn direction);
/// - PUFF: smoke kind, no next particle, a lifetime of 1000 ticks; held by
///   PLUME1 (smoke kind, spawn interval 1, slow-down 0, cut-off 5, a cap
///   larger than the run makes);
/// - FLAME: fire kind; held by BLAZE1 (fire kind, spawn interval 1);
/// - for each, a fire-flag weapon of rate [`ROF`] and range 10 cells, and a
///   CART type armed with that one weapon.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.general.gravity = Some(GRAVITY);
    let p = |kind: &str| ParticleType {
        behaves_like: Some(kind.into()),
        end_state_ai: Some(100),
        state_ai_advance: Some(4),
        ..Default::default()
    };
    for (n, t) in [
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
        (
            "PUFF",
            ParticleType {
                max_ec: Some(1000),
                ..p("smoke")
            },
        ),
        (
            "FLAME",
            ParticleType {
                max_ec: Some(1000),
                velocity: dec("6.0"),
                ..p("fire")
            },
        ),
    ] {
        register(&mut r, "particles", n);
        r.sim.particles.insert(n.into(), t);
    }
    let s = |kind: &str, holds: &str| ParticleSystemType {
        behaves_like: Some(kind.into()),
        holds_what: Some(holds.into()),
        ..Default::default()
    };
    for (n, t) in [
        (
            "SPARKS",
            ParticleSystemType {
                spark_spawn_frames: Some(400),
                particle_cap: Some(4),
                spawn_spark_percentage: dec("1"),
                ..s("spark", "SPARK1")
            },
        ),
        (
            "PLUME1",
            ParticleSystemType {
                spawn_frames: Some(1),
                slowdown: dec("0"),
                spawn_cutoff: dec("5"),
                particle_cap: Some(1000),
                ..s("smoke", "PUFF")
            },
        ),
        (
            "BLAZE1",
            ParticleSystemType {
                spawn_frames: Some(1),
                ..s("fire", "FLAME")
            },
        ),
    ] {
        register(&mut r, "particle_systems", n);
        r.sim.particle_systems.insert(n.into(), t);
    }
    for (firer, sys) in [
        (Firer::U9, "SPARKS"),
        (Firer::U10, "PLUME1"),
        (Firer::Fire, "BLAZE1"),
    ] {
        let wn = format!("FIRE-{sys}");
        r.sim.weapons.insert(
            wn.clone(),
            WeaponType {
                damage: Some(1),
                rof: Some(ROF),
                range: dec("10"),
                speed: Some(100),
                projectile: Some("Bullet".into()),
                warhead: Some("SA".into()),
                attached_particle_system: Some(sys.into()),
                use_fire_particles: Some(true),
                ..Default::default()
            },
        );
        let mut t = r.sim.vehicles["CART"].clone();
        t.primary = Some(wn);
        t.secondary = None;
        register(&mut r, "vehicles", firer.vehicle());
        r.sim.vehicles.insert(firer.vehicle().into(), t);
    }
    let buggy = r.sim.vehicles.get_mut("BUGGY").unwrap();
    buggy.strength = Some(30000);
    buggy.primary = None;
    buggy.secondary = None;
    r
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

fn enemy(w: &mut World, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, Kind::Vehicle, "BUGGY", 1, testkit::mid(N, dx, dy), WEST)
}

/// One firer's record from its first shot: its first system, the tick *t*
/// at whose end its new target is first recorded, and for each tick of
/// the record whether that system took its turn and made a particle in it.
#[derive(Debug, Default)]
struct Record {
    shot: Option<i32>,
    system: Option<ObjectId>,
    t: Option<i32>,
    /// (tick, took its turn, made a particle, marked at the tick's end,
    /// held in the fire place at the tick's end)
    turns: Vec<(i32, bool, bool, bool, bool)>,
    left: Option<i32>,
}

/// The third run: a world with a near enemy B in range of the three
/// firers and, for each firer, a second enemy outside its weapon's range.
/// Each firer attacks B; after its first shot it is given an attack order
/// on its second enemy. Runs `n` ticks.
fn third_run(n: i32) -> (World, Vec<(Firer, ObjectId, ObjectId)>, Vec<Record>) {
    let mut w = testkit::game_with(&rules(), N, 7);
    testkit::run(&mut w, 6);
    let b = enemy(&mut w, 2, 0);
    let firers: Vec<(Firer, ObjectId, ObjectId)> =
        [(Firer::U9, -3), (Firer::U10, 0), (Firer::Fire, 3)]
            .into_iter()
            .map(|(f, y)| {
                let a = testkit::spawn(
                    &mut w,
                    Kind::Vehicle,
                    f.vehicle(),
                    0,
                    testkit::mid(N, -1, y),
                    EAST,
                );
                (f, a, enemy(&mut w, -18, y))
            })
            .collect();
    for &(_, a, _) in &firers {
        attack(&mut w, a, b);
    }
    let mut recs: Vec<Record> = firers.iter().map(|_| Record::default()).collect();
    for _ in 0..n {
        let tick = w.tick;
        for (k, &(_, a, far)) in firers.iter().enumerate() {
            if recs[k].shot.is_some_and(|s| tick == s + 1) {
                attack(&mut w, a, far);
            }
        }
        let before: Vec<Option<(i32, BTreeSet<ObjectId>)>> = recs
            .iter()
            .map(|r| {
                r.system.and_then(|s| {
                    w.gas
                        .systems
                        .get(&s)
                        .map(|x| (x.lifetime, x.particles.iter().map(|p| p.id).collect()))
                })
            })
            .collect();
        let fired: Vec<i32> = firers
            .iter()
            .map(|&(_, a, _)| w.obj(a).unwrap().last_fired)
            .collect();
        testkit::run(&mut w, 1);
        for (k, &(_, a, far)) in firers.iter().enumerate() {
            let r = &mut recs[k];
            if r.shot.is_none() && w.obj(a).unwrap().last_fired != fired[k] {
                r.shot = Some(tick);
                r.system = emitters::held(&w, a, Place::Fire);
                assert!(r.system.is_some(), "the shot fills the fire place");
                continue;
            }
            if r.t.is_none() && w.obj(a).unwrap().target == Some(Target::Object(far)) {
                r.t = Some(tick);
            }
            let (Some(s), Some((life, old))) = (r.system, &before[k]) else {
                continue;
            };
            if r.left.is_some() {
                continue;
            }
            let Some(x) = w.gas.systems.get(&s) else {
                r.left = Some(tick);
                continue;
            };
            let turned = x.lifetime != *life;
            let made = x.particles.iter().any(|p| !old.contains(&p.id));
            let held = emitters::held(&w, a, Place::Fire) == Some(s);
            r.turns.push((tick, turned, made, x.ending, held));
        }
    }
    (w, firers, recs)
}

/// The record's rows from tick `t` on.
fn from(r: &Record, t: i32) -> Vec<(i32, bool, bool, bool, bool)> {
    r.turns.iter().copied().filter(|x| x.0 >= t).collect()
}

/// spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: the test of a retarget is made at the change of the firer's target, in whichever part of the tick makes that change, not in the turn of the held system, and reads nothing of that system's kind; an attack order takes effect after the turn pass, S-0203 claim 7, so the system's turn in the next tick is the first the mark affects; smoke and fire systems emit no new particle when marked, S-0616 claims 9 and 10) and scenario F4, unmatched part, third run (u10 and the fire firer: a particle in tick t's turn and none in tick t + 1's turn or later)
#[test]
fn f4_third_run_smoke_and_fire_systems_stop_at_t_plus_1() {
    let (w, firers, recs) = third_run(80);
    for (k, &(f, a, _)) in firers.iter().enumerate() {
        if f == Firer::U9 {
            continue;
        }
        let r = &recs[k];
        let t =
            r.t.unwrap_or_else(|| panic!("{f:?}: its new target is recorded: {r:?}"));
        let rows = from(r, t);
        assert!(rows.len() >= 3, "{f:?}: recorded from t: {r:?}");
        // Void line: the system takes a turn in tick t and every later tick while in play.
        assert!(
            rows.iter().all(|x| x.1),
            "void: a tick with no turn: {rows:?}"
        );
        let (_, _, made_t, marked_t, held_t) = rows[0];
        assert!(made_t, "{f:?}: a particle in tick t's turn: {rows:?}");
        // At the change itself: marked and the fire place freed at tick t's end.
        assert!(
            marked_t && !held_t,
            "{f:?}: let go at the change in tick t: {rows:?}"
        );
        assert!(
            rows[1..].iter().all(|x| !x.2),
            "{f:?}: no particle in tick t + 1 or later: {rows:?}"
        );
        assert!(
            w.obj(a).is_some_and(|o| o.health > 0),
            "void: a firer is gone"
        );
    }
}

/// spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: the mark does not stop a spark system's bursts: while its counter K is above 0 it bursts as S-0616 claim 15 gives) and Changes for other specs, S-0616 (d); scenario F4, unmatched part, third run (u9: sparks in every turn, in tick t + 1 and later as before)
#[test]
fn f4_third_run_a_marked_spark_system_keeps_bursting() {
    let (w, firers, recs) = third_run(80);
    let k = firers.iter().position(|x| x.0 == Firer::U9).unwrap();
    let r = &recs[k];
    let t = r.t.expect("u9's new target is recorded");
    let rows = from(r, t);
    assert!(rows.len() >= 10, "recorded from t: {r:?}");
    assert!(
        rows.iter().all(|x| x.1),
        "void: a tick with no turn: {rows:?}"
    );
    // Let go and marked at the change.
    assert!(rows[0].3 && !rows[0].4, "let go at the change: {rows:?}");
    // Sparks in every turn, tick t + 1 and later included.
    assert!(rows.iter().all(|x| x.2), "a burst in every turn: {rows:?}");
    let s = r.system.unwrap();
    let x = &w.gas.systems[&s];
    assert!(x.sparks > 0, "its counter has not run out");
    assert!(x.ending, "marked");
}

/// spec: S-0301 Changes for other specs, S-0616 (d), taken in by S-0616 Terms *Marked* and claim 15 (revision 27) (revision 67, Q-2246: a marked system makes no emission, though the mark does not stop a spark system's bursts while its K is above 0, S-0616 claim 15) and S-0616 claim 15 (K counts down by 1 each turn whatever the mark)
#[test]
fn a_marked_spark_system_bursts_while_its_counter_lasts() {
    let mut w = testkit::game_with(&rules(), N, 7);
    let p = glasnost_sim::world::Pos::centre(testkit::mid(N, 0, 0), 0);
    gas::warhead_system(&mut w, "SPARKS", p, None);
    let s = *w.gas.systems.keys().last().unwrap();
    glasnost_sim::particles::mark(&mut w, s);
    let k = w.gas.systems[&s].sparks;
    assert!(k > 1);
    for i in 0..5 {
        let old: BTreeSet<ObjectId> = w.gas.systems[&s].particles.iter().map(|p| p.id).collect();
        gas::system_turn(&mut w, s);
        let x = &w.gas.systems[&s];
        assert!(
            x.particles.iter().any(|p| !old.contains(&p.id)),
            "turn {i}: a burst while marked"
        );
        assert_eq!(x.sparks, k - 1 - i, "K counts down");
    }
}

/// spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: the test is made at the change of the firer's target, not in the held system's turn; a new target within reach is remembered and keeps the place) and S-0616 claim 1 (a new target that is not within reach lets the fire emitter go)
#[test]
fn the_retarget_test_is_made_at_the_order_not_in_the_systems_turn() {
    let mut w = testkit::game_with(&rules(), N, 7);
    testkit::run(&mut w, 6);
    let b = enemy(&mut w, 2, 0);
    let near = enemy(&mut w, 1, 2);
    let far = enemy(&mut w, -18, 0);
    let a = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        Firer::U10.vehicle(),
        0,
        testkit::mid(N, -1, 0),
        EAST,
    );
    attack(&mut w, a, b);
    let mut s = None;
    for _ in 0..40 {
        testkit::run(&mut w, 1);
        s = emitters::held(&w, a, Place::Fire);
        if s.is_some() {
            break;
        }
    }
    let s = s.expect("a shot fills the fire place");
    // A new target within reach: remembered, the place kept.
    attack(&mut w, a, near);
    testkit::run(&mut w, 1);
    assert_eq!(w.obj(a).unwrap().target, Some(Target::Object(near)));
    assert_eq!(
        emitters::held(&w, a, Place::Fire),
        Some(s),
        "within reach: kept"
    );
    assert_eq!(
        w.obj(a).unwrap().combat.fire_emitter.unwrap().target,
        Some(Target::Object(near)),
        "remembered at the order"
    );
    assert!(!w.gas.systems[&s].ending);
    // A new target out of reach: let go and marked at the order's tick,
    // before the system's next turn, with no turn of the system needed.
    attack(&mut w, a, far);
    let life = w.gas.systems[&s].lifetime;
    testkit::run(&mut w, 1);
    assert_eq!(w.obj(a).unwrap().target, Some(Target::Object(far)));
    assert_eq!(
        emitters::held(&w, a, Place::Fire),
        None,
        "out of reach: let go"
    );
    let x = &w.gas.systems[&s];
    assert!(x.ending, "and marked");
    assert_eq!(
        x.lifetime,
        life - 1,
        "after one turn, the one before the order"
    );
}
