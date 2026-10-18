//! Claim tests for the creators of particle systems of S-0616 claim 1 that
//! belong to other parts of play, the emitter places and S-0301 claim 5's
//! live-effect gate, and the fire emitter's sweep, swing and insertion
//! (claim 11, F5 to F7), on synthetic data: invented particle, system and
//! weapon types, every number our own. The ledger is
//! `docs/m7-claims/S-0616.md`.

use glasnost_num::{F64, Rounding};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, ParticleSystemType, ParticleType, WeaponType};
use glasnost_sim::combat::check::{Answer, check};
use glasnost_sim::combat::fire;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};
use glasnost_sim::{emitters, gas, particles, save, testkit};

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

/// An invented weapon: rate 30, range 10 cells, the invisible projectile
/// Bullet, warhead SA.
fn weapon(attached: Option<&str>) -> WeaponType {
    WeaponType {
        damage: Some(1),
        rof: Some(30),
        range: dec("10"),
        speed: Some(100),
        projectile: Some("Bullet".into()),
        warhead: Some("SA".into()),
        attached_particle_system: attached.map(Into::into),
        ..Default::default()
    }
}

/// The rules: testkit's combat rules plus
/// - particles FLAME (fire, speed 6.0, no damage), SPARK (spark), LINE
///   (railgun) and PUFF (smoke);
/// - systems BLAZE (fire, FLAME, spawn interval 5), BURST (spark, SPARK,
///   one spark turn, cap 4), RAIL (railgun, LINE), PLUME (smoke, PUFF,
///   interval 4), HAZE (smoke, PUFF), EMPTYS (spark, no held type) and
///   GASSY (gas, no held type);
/// - weapons FLAMER (fire particles, BLAZE), SPARKER (spark particles,
///   BURST), RAILER (railgun, RAIL), with FLAMER the CART's primary and
///   SPARKER its secondary; a durable BUGGY;
/// - the WALKER a cyborg whose damage list is PLUME, BURST, EMPTYS;
/// - the TOWER with natural smoke PLUME at (16, 32, 48), destruction list
///   GASSY, PLUME, BURST, HAZE at displacement (8, 0, 0), refinery smoke PLUME at
///   (0, 0, 0), (40, 0, 0), (128, 128, 0), (0, 40, 0), and a refinery smoke
///   time of 7;
/// - yellow ratio 0.5, red ratio 0.25.
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
        ("PUFF", p("smoke")),
    ] {
        register(&mut r, "particles", n);
        r.sim.particles.insert(n.into(), t);
    }
    let s = |kind: &str, holds: Option<&str>| ParticleSystemType {
        behaves_like: Some(kind.into()),
        holds_what: holds.map(Into::into),
        ..Default::default()
    };
    for (n, t) in [
        (
            "BLAZE",
            ParticleSystemType {
                spawn_frames: Some(5),
                ..s("fire", Some("FLAME"))
            },
        ),
        (
            "BURST",
            ParticleSystemType {
                spark_spawn_frames: Some(1),
                particle_cap: Some(4),
                ..s("spark", Some("SPARK"))
            },
        ),
        (
            "RAIL",
            ParticleSystemType {
                particles_per_coord: dec("0.015625"),
                ..s("railgun", Some("LINE"))
            },
        ),
        (
            "PLUME",
            ParticleSystemType {
                spawn_frames: Some(4),
                spawn_cutoff: dec("100.0"),
                ..s("smoke", Some("PUFF"))
            },
        ),
        ("HAZE", s("smoke", Some("PUFF"))),
        ("EMPTYS", s("spark", None)),
        ("GASSY", s("gas", None)),
    ] {
        register(&mut r, "particle_systems", n);
        r.sim.particle_systems.insert(n.into(), t);
    }
    r.sim.weapons.insert("FLAMER".into(), {
        let mut w = weapon(Some("BLAZE"));
        w.use_fire_particles = Some(true);
        w
    });
    r.sim.weapons.insert("SPARKER".into(), {
        let mut w = weapon(Some("BURST"));
        w.use_spark_particles = Some(true);
        w
    });
    r.sim.weapons.insert("RAILER".into(), {
        let mut w = weapon(Some("RAIL"));
        w.is_railgun = Some(true);
        w
    });
    r.sim.weapons.insert("PLAIN".into(), weapon(None));
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.primary = Some("FLAMER".into());
    cart.secondary = Some("SPARKER".into());
    r.sim.vehicles.get_mut("BUGGY").unwrap().strength = Some(30000);
    let walker = r.sim.infantry.get_mut("WALKER").unwrap();
    walker.cyborg = Some(true);
    walker.strength = Some(100);
    walker.combat.damage_smoke_systems = Some(vec!["PLUME".into()]);
    walker.combat.damage_spark_systems = Some(vec!["BURST".into(), "EMPTYS".into()]);
    walker.combat.damage_smoke_offset = Some([3, 4, 5]);
    let tower = r.sim.buildings.get_mut("TOWER").unwrap();
    tower.combat.natural_particle_system = Some("PLUME".into());
    tower.combat.natural_particle_location = Some([16, 32, 48]);
    tower.combat.destroy_particle_systems = Some(vec![
        "GASSY".into(),
        "PLUME".into(),
        "BURST".into(),
        "HAZE".into(),
    ]);
    tower.combat.destroy_smoke_offset = Some([8, 0, 0]);
    tower.refinery_smoke_particle_system = Some("PLUME".into());
    tower.refinery_smoke_offsets = Some(vec![[0, 0, 0], [40, 0, 0], [128, 128, 0], [0, 40, 0]]);
    let g = &mut r.sim.general;
    g.refinery_smoke_frames = Some(7);
    g.condition_yellow = dec("0.5");
    g.condition_red = dec("0.25");
    g.condition_yellow_sparking_probability = dec("1.0");
    g.condition_red_sparking_probability = dec("1.0");
    r
}

fn m() -> Rounding {
    Rounding::Nearest
}

/// A world of `r` with an Alpha CART two cells west of the middle facing
/// east, and a Beta BUGGY `gap` cells east of it, set as its target.
fn duel(r: &Ruleset, gap: i32) -> (World, ObjectId, ObjectId) {
    let mut w = testkit::game_with(r, N, 7);
    let a = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "CART",
        0,
        testkit::mid(N, -2, 0),
        EAST,
    );
    let b = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(N, gap - 2, 0),
        WEST,
    );
    w.obj_mut(a).unwrap().target = Some(Target::Object(b));
    (w, a, b)
}

fn sys_name(w: &World, s: ObjectId) -> String {
    let t = w.gas.systems[&s].type_index as usize;
    w.rules().gas.systems[t].name.clone()
}

fn round_trip(w: &World) {
    let bytes = save::save(w);
    let back = save::load(&bytes, w.rules_arc(), testkit::catalogue()).expect("loads");
    assert_eq!(save::state_hash(&back), save::state_hash(w));
}

// ---------------------------------------------------------------------------
// Effect-weapon emitters (claim 1; S-0301 claim 5)

/// spec: S-0616 claim 1 (an effect weapon's fire emitter: made at the shot after the projectile, at the fire point, owner object the firer, target point the target object's position, no first particle, no draw) and Terms (held in the fire place; the target object is the owner's target)
#[test]
fn claim_1_fire_emitter_at_the_shot() {
    let (mut w, a, b) = duel(&rules(), 4);
    let mut plain = rules();
    plain
        .sim
        .weapons
        .get_mut("FLAMER")
        .unwrap()
        .attached_particle_system = None;
    let (mut w0, a0, _) = duel(&plain, 4);
    let t = w.obj(a).unwrap().target.unwrap();
    let before = w.rng.taken();
    assert!(fire::shot(&mut w, a, 0, t));
    let taken = w.rng.taken() - before;
    let before0 = w0.rng.taken();
    assert!(fire::shot(&mut w0, a0, 0, t));
    assert_eq!(
        taken,
        w0.rng.taken() - before0,
        "making the emitter takes no draw"
    );
    let e = w
        .obj(a)
        .unwrap()
        .combat
        .fire_emitter
        .expect("held in the fire place");
    let s = &w.gas.systems[&e.system];
    assert_eq!(sys_name(&w, e.system), "BLAZE");
    assert_eq!(s.owner_object, Some(a));
    assert_eq!(s.target, w.obj(b).unwrap().pos);
    assert_eq!(s.target_object, Some(b));
    assert!(s.particles.is_empty(), "no first particle");
    assert_eq!(
        s.pos,
        glasnost_sim::combat::position::fire_point(&w, a, 0, 0),
        "at the weapon's fire point"
    );
    assert_eq!(e.target, Some(t));
    round_trip(&w);
}

/// spec: S-0616 claim 1 (spark and railgun emitters: owner object the firer; a spark's target point the target object's position, a railgun's the end of the beam) and Terms (held in the spark and railgun places)
#[test]
fn claim_1_spark_and_railgun_emitters() {
    let mut r = rules();
    r.sim.vehicles.get_mut("CART").unwrap().primary = Some("RAILER".into());
    let (mut w, a, b) = duel(&r, 4);
    let t = w.obj(a).unwrap().target.unwrap();
    assert!(fire::shot(&mut w, a, 1, t));
    let sp = w.obj(a).unwrap().combat.spark_emitter.expect("spark place");
    assert_eq!(sys_name(&w, sp), "BURST");
    assert_eq!(w.gas.systems[&sp].owner_object, Some(a));
    assert_eq!(w.gas.systems[&sp].target, w.obj(b).unwrap().pos);
    w.obj_mut(a).unwrap().combat.rearm_until = 0;
    assert!(fire::shot(&mut w, a, 0, t));
    let rg = w
        .obj(a)
        .unwrap()
        .combat
        .railgun_emitter
        .expect("railgun place");
    assert_eq!(sys_name(&w, rg), "RAIL");
    assert_eq!(w.gas.systems[&rg].owner_object, Some(a));
    round_trip(&w);
}

/// spec: S-0301 claim 5 (a live effect of the asked weapon's own kind: rearming at rank 5; of its other weapon's kind: impossible at rank 4) and S-0616 claim 1 (the gate applies while the emitter is held)
#[test]
fn s0301_claim_5_live_effect_gate() {
    let (mut w, a, _) = duel(&rules(), 4);
    let ask = |w: &World, slot| check(w, a, w.obj(a).unwrap().target, slot, true);
    let t = w.obj(a).unwrap().target.unwrap();
    assert!(fire::shot(&mut w, a, 0, t));
    // The countdown is over: only the gate can hold the weapons back.
    w.obj_mut(a).unwrap().combat.rearm_until = 0;
    assert_eq!(ask(&w, 0), Answer::Rearming, "own kind alive");
    assert_eq!(
        ask(&w, 1),
        Answer::Impossible,
        "the other weapon's kind alive"
    );
    // The emitter let go: both weapons fire again.
    emitters::let_go_fire(&mut w, a);
    assert_eq!(ask(&w, 0), Answer::CanFire);
    assert_eq!(ask(&w, 1), Answer::CanFire);
    // A spark emitter of the secondary blocks it (own kind) and the primary (other weapon).
    assert!(fire::shot(&mut w, a, 1, t));
    w.obj_mut(a).unwrap().combat.rearm_until = 0;
    assert_eq!(ask(&w, 1), Answer::Rearming);
    assert_eq!(ask(&w, 0), Answer::Impossible);
}

/// spec: S-0616 claim 1 (a spark emitter is not let go by a move order; it ends by its own counter and the place empties when it leaves play) and S-0301 claim 5 (a spark weapon fires again once its emitter has left play)
#[test]
fn claim_1_spark_emitter_ends_by_its_counter() {
    let (mut w, a, _) = duel(&rules(), 4);
    let t = w.obj(a).unwrap().target.unwrap();
    assert!(fire::shot(&mut w, a, 1, t));
    let sp = w.obj(a).unwrap().combat.spark_emitter.unwrap();
    w.tick = 10;
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(testkit::mid(N, -6, 0))),
        }
    ));
    for _ in 0..200 {
        if !w.gas.systems.contains_key(&sp) {
            break;
        }
        gas::system_turn(&mut w, sp);
    }
    assert!(
        !w.gas.systems.contains_key(&sp),
        "the burst ends and leaves play"
    );
    assert_eq!(
        w.obj(a).unwrap().combat.spark_emitter,
        None,
        "the place is empty again"
    );
}

/// spec: S-0616 claim 1 (a fire emitter is marked and let go on a move order, on a new target not within reach, and when the firer dies; a new target within reach keeps it)
#[test]
fn claim_1_fire_emitter_let_go_events() {
    // Move order.
    let (mut w, a, _) = duel(&rules(), 4);
    let t = w.obj(a).unwrap().target.unwrap();
    assert!(fire::shot(&mut w, a, 0, t));
    let s = w.obj(a).unwrap().combat.fire_emitter.unwrap().system;
    w.tick = 10;
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(testkit::mid(N, -6, 0))),
        }
    ));
    testkit::run(&mut w, 1);
    assert_eq!(
        w.obj(a).unwrap().combat.fire_emitter,
        None,
        "move order: let go"
    );
    assert!(w.gas.systems.get(&s).is_none_or(|s| s.ending), "and marked");

    // A new target within reach keeps it; one out of reach lets it go.
    let (mut w, a, _) = duel(&rules(), 4);
    let near = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(N, 0, 2),
        WEST,
    );
    let far = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(N, -2, 16),
        WEST,
    );
    let t = w.obj(a).unwrap().target.unwrap();
    assert!(fire::shot(&mut w, a, 0, t));
    let s = w.obj(a).unwrap().combat.fire_emitter.unwrap().system;
    w.obj_mut(a).unwrap().target = Some(Target::Object(near));
    particles::fire_turn(&mut w, s);
    let e = w
        .obj(a)
        .unwrap()
        .combat
        .fire_emitter
        .expect("within reach: kept");
    assert_eq!(e.target, Some(Target::Object(near)));
    assert_eq!(
        w.gas.systems[&s].target_object,
        Some(near),
        "the owner's current target"
    );
    w.obj_mut(a).unwrap().target = Some(Target::Object(far));
    particles::fire_turn(&mut w, s);
    assert_eq!(
        w.obj(a).unwrap().combat.fire_emitter,
        None,
        "out of reach: let go"
    );
    assert!(w.gas.systems[&s].ending);

    // Death.
    let (mut w, a, _) = duel(&rules(), 4);
    let t = w.obj(a).unwrap().target.unwrap();
    assert!(fire::shot(&mut w, a, 0, t));
    let s = w.obj(a).unwrap().combat.fire_emitter.unwrap().system;
    w.obj_mut(a).unwrap().health = 0;
    particles::fire_turn(&mut w, s);
    assert!(w.gas.systems[&s].ending, "dead owner: marked");
    assert_eq!(w.obj(a).unwrap().combat.fire_emitter, None);
}

/// spec: S-0616 claim 4 (a removed target object: the system loses it; its target point is unchanged)
#[test]
fn claim_4_target_object_removed() {
    let (mut w, a, b) = duel(&rules(), 4);
    let t = w.obj(a).unwrap().target.unwrap();
    assert!(fire::shot(&mut w, a, 0, t));
    let s = w.obj(a).unwrap().combat.fire_emitter.unwrap().system;
    let q = w.gas.systems[&s].target;
    glasnost_sim::objects::remove(&mut w, b);
    assert_eq!(w.gas.systems[&s].target_object, None);
    assert_eq!(w.gas.systems[&s].target, q);
}

/// The target point F5 gives for owner `o`, target `g` and facing `f`.
fn sweep_point(w: &World, o: Pos, g: Pos, f: u16, on: bool) -> Pos {
    let mode = m();
    let dd = |v: i64| F64::from_i64(v, mode);
    let sum = [g.x - o.x, g.y - o.y, g.z - o.z]
        .iter()
        .fold(dd(0), |a, &v| {
            a.add(dd(i64::from(v)).mul(dd(i64::from(v)), mode), mode)
        });
    let table = w.rules().quirks.table_root_distance;
    let dist = dd(i64::from(
        glasnost_num::geom::table_root_real(sum, table, mode).to_i32_common(),
    ));
    let (q, full) = if on { (16383, 65534) } else { (16384, 65536) };
    let two_pi = glasnost_sim::rules::nearest_double("6.283185307179586");
    let theta = dd(i64::from(f as i16) - q).mul(two_pi.negated().div(dd(full), mode), mode);
    let c = glasnost_num::trig::cos_t(theta, mode);
    let s = glasnost_num::trig::sin_t(theta, mode);
    Pos::new(
        dd(i64::from(o.x))
            .add(c.mul(dist, mode), mode)
            .to_i32_common(),
        dd(i64::from(o.y))
            .sub(s.mul(dist, mode), mode)
            .to_i32_common(),
        o.z,
    )
}

/// spec: S-0616 claims 10 and 11 and F5 (an alive owner with a target object whose body is turning sweeps: the target point at the owner's distance in its body facing, the system at its first weapon's fire point; emits on a multiple of 3 in that turn) and Edge cases (quirks.flame_sweep_facing on: 16383 and 65534; off: 16384 and 65536)
#[test]
fn flame_sweep_both_positions() {
    assert!(glasnost_rules::quirks::Quirks::default().flame_sweep_facing);
    let mut differs = false;
    for f in (0..65536u32).step_by(1024).map(|f| f as u16) {
        let mut points = Vec::new();
        for on in [true, false] {
            let mut r = rules();
            r.globals.quirks.flame_sweep_facing = on;
            let (mut w, a, b) = duel(&r, 9);
            let t = w.obj(a).unwrap().target.unwrap();
            assert!(fire::shot(&mut w, a, 0, t));
            let s = w.obj(a).unwrap().combat.fire_emitter.unwrap().system;
            // A multiple of 3 and not of the spawn interval 5.
            w.tick = 33;
            let tick = w.tick;
            let o = w.obj_mut(a).unwrap();
            o.body = glasnost_sim::turning::Turning::at(f);
            o.body.turn_to(f.wrapping_add(20000), tick, 1);
            assert!(o.body.turning(tick));
            let face = o.body.value(tick);
            let (op, gp) = (w.obj(a).unwrap().pos, w.obj(b).unwrap().pos);
            particles::fire_turn(&mut w, s);
            let sys = &w.gas.systems[&s];
            assert_eq!(
                sys.target,
                sweep_point(&w, op, gp, face, on),
                "f {f} on {on}"
            );
            assert_eq!(
                sys.pos,
                glasnost_sim::combat::position::fire_point(&w, a, 0, 0)
            );
            assert_eq!(
                sys.particles.len(),
                1,
                "tick 33: a multiple of 3 after a sweep"
            );
            points.push(sys.target);
        }
        differs |= points[0] != points[1];
    }
    assert!(differs, "the two positions differ for some facing");
}

/// spec: S-0616 claim 11 (without a sweep a fire emitter emits on a multiple of its spawn interval only; a marked one never)
#[test]
fn claim_11_fire_cadence() {
    let (mut w, a, _) = duel(&rules(), 4);
    let t = w.obj(a).unwrap().target.unwrap();
    assert!(fire::shot(&mut w, a, 0, t));
    let s = w.obj(a).unwrap().combat.fire_emitter.unwrap().system;
    w.tick = 33;
    particles::fire_turn(&mut w, s);
    assert!(
        w.gas.systems[&s].particles.is_empty(),
        "33: no sweep, not a multiple of 5"
    );
    w.tick = 35;
    particles::fire_turn(&mut w, s);
    assert_eq!(w.gas.systems[&s].particles.len(), 1);
    particles::mark(&mut w, s);
    w.tick = 40;
    let taken = w.rng.taken();
    particles::fire_turn(&mut w, s);
    assert_eq!(w.gas.systems[&s].particles.len(), 1, "marked: no emission");
    assert_eq!(
        w.rng.taken() - taken,
        1,
        "only the one particle's update draw"
    );
}

/// spec: S-0616 F6 (V[i] = trunc(sinₜ(0.125 × i) × 12.0 + 0.49); δ by the five ranges of q)
#[test]
fn f6_swing_table_and_direction() {
    assert_eq!(particles::swing(0, m()), 0);
    // sin(0.5) × 12 + 0.49 = 6.24…; sin(1.625) × 12 + 0.49 = 12.47…
    assert_eq!(particles::swing(4, m()), 6);
    assert_eq!(particles::swing(13, m()), 12);
    for i in 0..500 {
        assert!((-12..=12).contains(&particles::swing(i, m())));
    }
    let p = Pos::new(1000, 1000, 0);
    let at = |dx, dy| particles::step_direction(p, Pos::new(1000 + dx, 1000 + dy, 0), m());
    assert_eq!(at(0, -100), 0);
    assert_eq!(at(0, 100), 4);
    assert_eq!(at(100, -100), 1);
    assert_eq!(at(100, 0), 2);
    assert_eq!(at(100, 100), 3);
    assert_eq!(at(-100, 100), 5);
    assert_eq!(at(-100, 0), 6);
    assert_eq!(at(-100, -100), 7);
    assert_eq!(at(10, -100), 0, "steep: as Qₓ = Pₓ");
    assert_eq!(at(-10, 100), 4);
}

/// spec: S-0616 claim 11 and F6 (the particle is made 1 lepton above the system and aims at Q + a × e(δ), a = trunc(12 × V[t rem 500] / (3m))) and F7 (after its creation draws one raw value r; the new particle has j = |r| rem min(4, N) particles newer than it)
#[test]
fn f6_f7_swing_and_insertion() {
    let mut r = rules();
    // STILL: a particle with no kind, which takes no update draw (claim 3).
    register(&mut r, "particles", "STILL");
    r.sim.particles.insert(
        "STILL".into(),
        ParticleType {
            max_ec: Some(1000),
            ..Default::default()
        },
    );
    let (mut w, a, _) = duel(&r, 4);
    let still = w
        .rules()
        .gas
        .particles
        .iter()
        .position(|t| t.name == "STILL")
        .unwrap() as u32;
    let t = w.obj(a).unwrap().target.unwrap();
    assert!(fire::shot(&mut w, a, 0, t));
    let s = w.obj(a).unwrap().combat.fire_emitter.unwrap().system;
    w.tick = 5;
    particles::fire_turn(&mut w, s);
    assert_eq!(
        w.gas.systems[&s].particles.len(),
        1,
        "N = 1: one raw value, j = 0"
    );
    let mut filler = w.gas.systems[&s].particles[0].clone();
    filler.type_index = still;
    filler.life = 1000;
    let mut js = std::collections::BTreeSet::new();
    for k in 2..=60 {
        w.tick = 5 * k;
        let sys = w.gas.systems.get_mut(&s).unwrap();
        sys.particles = (0..3)
            .map(|i| {
                let mut p = filler.clone();
                p.id = ObjectId(900_000 + i);
                p
            })
            .collect();
        let mut probe = w.rng.clone();
        let _life = probe.raw(); // FLAME's lifetime variation (S-0202 claim 15)
        let raw = probe.raw() as i32;
        let j = (raw.wrapping_abs() % 4).max(0) as usize;
        let before = w.rng.taken();
        particles::fire_turn(&mut w, s);
        assert_eq!(
            w.rng.taken() - before,
            2,
            "the creation draw and the insertion draw"
        );
        let sys = &w.gas.systems[&s];
        let l = &sys.particles;
        assert_eq!(l.len(), 4);
        let at = l.iter().position(|p| p.type_index != still).unwrap();
        assert_eq!(at, 3 - j, "j = {j} particles newer than it");
        js.insert(j);
        // F6: the aim.
        let (p, q) = (sys.pos, sys.target);
        let v = particles::swing(w.tick % 500, m());
        let dl = [p.x - q.x, p.y - q.y, p.z - q.z];
        let sq: i64 = dl.iter().map(|&x| i64::from(x) * i64::from(x)).sum();
        let near = if sq < 200 * 200 { 3 } else { 1 };
        let amp = (12 * v) / (3 * near);
        let side = [
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
            (0, -1),
            (1, -1),
        ];
        let (ex, ey) = side[particles::step_direction(p, q, m())];
        let aim = Pos::new(q.x + amp * ex, q.y + amp * ey, q.z);
        let from = Pos::new(p.x, p.y, p.z + 1);
        assert_eq!(
            l[at].dir,
            particles::heading(&w, from, aim),
            "tick {}",
            w.tick
        );
    }
    assert!(js.len() >= 3, "several insertion places seen: {js:?}");
}

// ---------------------------------------------------------------------------
// Damage sparks, deactivated sparks (claim 1)

/// spec: S-0616 claim 1 (damage sparks: a cyborg infantry object strictly below the yellow ratio draws R(0, 2147483646), makes a system when the scaled draw is below the sparking probability, picks a spark-kind entry by R(0, n − 1) from the start of the list, at its position plus the damage-smoke displacement, held in the spark place, owner object itself)
#[test]
fn claim_1_damage_sparks() {
    let mut w = testkit::game_with(&rules(), N, 7);
    let i = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        0,
        testkit::mid(N, 0, 0),
        0,
    );
    // At the yellow ratio (50 of 100): not strictly below, nothing and no draw.
    w.obj_mut(i).unwrap().health = 50;
    let before = w.rng.taken();
    emitters::damage_sparks(&mut w, i);
    assert_eq!(w.rng.taken(), before);
    assert_eq!(w.obj(i).unwrap().combat.spark_emitter, None);
    // Below it, with probability 1.0: made after two draws.
    w.obj_mut(i).unwrap().health = 49;
    let mut probe = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    let _ = probe.ranged(0, 2147483646, half);
    let k = probe.ranged(0, 1, half) as usize;
    emitters::damage_sparks(&mut w, i);
    assert_eq!(w.rng.taken(), before + 2);
    let s = w.obj(i).unwrap().combat.spark_emitter.expect("spark place");
    assert_eq!(
        sys_name(&w, s),
        ["BURST", "EMPTYS"][k],
        "counted from the start"
    );
    let o = w.obj(i).unwrap().pos;
    assert_eq!(w.gas.systems[&s].pos, Pos::new(o.x + 3, o.y + 4, o.z + 5));
    assert_eq!(w.gas.systems[&s].owner_object, Some(i));
    // The place is held: no draw.
    let before = w.rng.taken();
    emitters::damage_sparks(&mut w, i);
    assert_eq!(w.rng.taken(), before);
    round_trip(&w);
    // Probability 0.0: the first draw only.
    let mut r = rules();
    r.sim.general.condition_yellow_sparking_probability = dec("0.0");
    let mut w = testkit::game_with(&r, N, 7);
    let i = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        0,
        testkit::mid(N, 0, 0),
        0,
    );
    w.obj_mut(i).unwrap().health = 40;
    let before = w.rng.taken();
    emitters::damage_sparks(&mut w, i);
    assert_eq!(w.rng.taken(), before + 1, "yellow probability 0: one draw");
    assert_eq!(w.obj(i).unwrap().combat.spark_emitter, None);
    // Below the red ratio the red probability (1.0) counts.
    w.obj_mut(i).unwrap().health = 20;
    emitters::damage_sparks(&mut w, i);
    assert!(
        w.obj(i).unwrap().combat.spark_emitter.is_some(),
        "red probability"
    );
    // Not a cyborg: nothing, no draw.
    let mut r = rules();
    r.sim.infantry.get_mut("WALKER").unwrap().cyborg = Some(false);
    let mut w = testkit::game_with(&r, N, 7);
    let i = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        0,
        testkit::mid(N, 0, 0),
        0,
    );
    w.obj_mut(i).unwrap().health = 10;
    let before = w.rng.taken();
    emitters::damage_sparks(&mut w, i);
    assert_eq!(w.rng.taken(), before);
}

/// spec: S-0616 claim 1 (a deactivated vehicle makes one system of the game-wide spark type when (⌊x / 10⌋ + ⌊y / 10⌋ × 65536 + t + object number) is a multiple of 30, displaced by R(−100, 100) for y then x; no owner object)
#[test]
fn claim_1_deactivated_vehicle_sparks() {
    let mut r = rules();
    r.sim.general.default_spark_system = Some("BURST".into());
    let mut w = testkit::game_with(&r, N, 7);
    let v = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, testkit::mid(N, 0, 0), 0);
    let o = w.obj(v).unwrap().pos;
    let term = |t: i32| (o.x / 10 + (o.y / 10) * 65536 + t + v.0 as i32).rem_euclid(30);
    let due = (0..30).find(|&t| term(t) == 0).unwrap();
    // Not deactivated: nothing.
    w.tick = due;
    let n = w.gas.systems.len();
    emitters::deactivated_sparks(&mut w, v);
    assert_eq!(w.gas.systems.len(), n);
    w.obj_mut(v).unwrap().econ.building.unit_disabled = true;
    w.tick = due + 1;
    let before = w.rng.taken();
    emitters::deactivated_sparks(&mut w, v);
    assert_eq!(
        (w.gas.systems.len(), w.rng.taken()),
        (n, before),
        "off the cadence"
    );
    w.tick = due + 30;
    let mut probe = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    let dy = probe.ranged(-100, 100, half);
    let dx = probe.ranged(-100, 100, half);
    emitters::deactivated_sparks(&mut w, v);
    let s = *w.gas.systems.keys().last().unwrap();
    assert_eq!(sys_name(&w, s), "BURST");
    assert_eq!(w.gas.systems[&s].pos, Pos::new(o.x + dx, o.y + dy, o.z));
    assert_eq!(w.gas.systems[&s].owner_object, None);
    // No game-wide spark type: nothing and no draw.
    let mut w = testkit::game_with(&rules(), N, 7);
    let v = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, testkit::mid(N, 0, 0), 0);
    w.obj_mut(v).unwrap().econ.building.unit_disabled = true;
    for t in 0..30 {
        w.tick = t;
        let before = w.rng.taken();
        emitters::deactivated_sparks(&mut w, v);
        assert_eq!(w.rng.taken(), before);
    }
}

// ---------------------------------------------------------------------------
// Buildings: natural, destruction and refinery smoke; the barrel

fn tower(w: &mut World) -> ObjectId {
    testkit::spawn(w, Kind::Building, "TOWER", 0, testkit::mid(N, 3, 3), 0)
}

/// spec: S-0616 claim 1 (natural smoke: made when the building is placed in play, at its position plus the displacement, no owner object, target point its position; T × 1.5 once on turning yellow) and S-0516 claim 18 (full cloaking removes it at once; full uncloaking makes a new one only with a displacement other than (0, 0, 0))
#[test]
fn claim_1_natural_smoke() {
    let mut w = testkit::game_with(&rules(), N, 7);
    let b = tower(&mut w);
    let s = w
        .obj(b)
        .unwrap()
        .combat
        .natural_smoke
        .expect("made at placement");
    let o = w.obj(b).unwrap().pos;
    let sys = &w.gas.systems[&s];
    assert_eq!(sys_name(&w, s), "PLUME");
    assert_eq!(sys.pos, Pos::new(o.x + 16, o.y + 32, o.z + 48));
    assert_eq!(sys.owner_object, None);
    assert_eq!(sys.target, o);
    // Yellow: × 1.5 once.
    let t0 = sys.timer;
    emitters::natural_smoke_yellow(&mut w, b);
    emitters::natural_smoke_yellow(&mut w, b);
    let six = glasnost_sim::rules::nearest_double("6.0").to_f32(m());
    assert_eq!(t0, glasnost_sim::rules::nearest_double("4.0").to_f32(m()));
    assert_eq!(w.gas.systems[&s].timer, six, "4.0 × 1.5, once");
    round_trip(&w);
    // Full cloaking removes it at once; uncloaking makes a new one.
    let n = w.gas.systems.len();
    emitters::natural_smoke_cloaked(&mut w, b);
    assert!(!w.gas.systems.contains_key(&s));
    assert_eq!(w.gas.systems.len(), n - 1);
    emitters::natural_smoke(&mut w, b, true);
    let s2 = w
        .obj(b)
        .unwrap()
        .combat
        .natural_smoke
        .expect("remade at uncloak");
    assert_ne!(s2, s);
    // It has one: no second.
    emitters::natural_smoke(&mut w, b, false);
    assert_eq!(w.obj(b).unwrap().combat.natural_smoke, Some(s2));
    // A (0, 0, 0) displacement: placed, but not remade at uncloak.
    let mut r = rules();
    r.sim
        .buildings
        .get_mut("TOWER")
        .unwrap()
        .combat
        .natural_particle_location = None;
    let mut w = testkit::game_with(&r, N, 7);
    let b = tower(&mut w);
    assert!(w.obj(b).unwrap().combat.natural_smoke.is_some());
    emitters::natural_smoke_cloaked(&mut w, b);
    emitters::natural_smoke(&mut w, b, true);
    assert_eq!(w.obj(b).unwrap().combat.natural_smoke, None);
}

/// spec: S-0516 claim 18 and S-0616 claim 1 (the cloak state machine: level 15 removes the natural smoke, level 0 remakes it)
#[test]
fn natural_smoke_follows_the_building_cloak() {
    let mut w = testkit::game_with(&rules(), N, 7);
    let b = tower(&mut w);
    let s = w.obj(b).unwrap().combat.natural_smoke.unwrap();
    {
        let o = w.obj_mut(b).unwrap();
        o.cloak.state = glasnost_sim::cloak::CloakState::Cloaking;
        o.cloak.stage = 14;
        o.cloak.stepped = -10;
    }
    glasnost_sim::cloak::turn(&mut w, b);
    assert_eq!(
        w.obj(b).unwrap().cloak.state,
        glasnost_sim::cloak::CloakState::Cloaked
    );
    assert!(!w.gas.systems.contains_key(&s), "removed at level 15");
    {
        let o = w.obj_mut(b).unwrap();
        o.cloak.state = glasnost_sim::cloak::CloakState::Uncloaking;
        o.cloak.stage = 1;
        o.cloak.stepped = -10;
    }
    glasnost_sim::cloak::turn(&mut w, b);
    assert_eq!(
        w.obj(b).unwrap().cloak.state,
        glasnost_sim::cloak::CloakState::Uncloaked
    );
    assert!(
        w.obj(b).unwrap().combat.natural_smoke.is_some(),
        "remade at level 0"
    );
}

/// spec: S-0312 claim 19 f and S-0616 claim 1 (destruction smoke: with no system attached, one draw R(0, m − 1) over the eligible entries counted from the end, at its position plus the destruction displacement, owner object the building; eligible: the smoke kind, interim Q-1400)
#[test]
fn claim_1_destruction_smoke() {
    let mut r = rules();
    r.sim
        .buildings
        .get_mut("TOWER")
        .unwrap()
        .combat
        .natural_particle_system = None;
    let mut w = testkit::game_with(&r, N, 7);
    let b = tower(&mut w);
    let before = w.rng.taken();
    let mut probe = w.rng.clone();
    let k = probe.ranged(0, 1, w.rules().quirks.ranged_draw_half_span) as usize;
    emitters::destruction_smoke(&mut w, b);
    assert_eq!(w.rng.taken(), before + 1, "one ranged draw R(0, 1)");
    let s = *w.gas.systems.keys().last().unwrap();
    assert_eq!(
        sys_name(&w, s),
        ["HAZE", "PLUME"][k],
        "counted from the end of PLUME, HAZE"
    );
    let o = w.obj(b).unwrap().pos;
    assert_eq!(w.gas.systems[&s].pos, Pos::new(o.x + 8, o.y, o.z));
    assert_eq!(w.gas.systems[&s].owner_object, Some(b));
    // A system attached already: none.
    let n = w.gas.systems.len();
    w.obj_mut(b).unwrap().combat.damage_smoke = Some(s);
    let before = w.rng.taken();
    emitters::destruction_smoke(&mut w, b);
    assert_eq!((w.gas.systems.len(), w.rng.taken()), (n, before));
}

/// spec: S-0616 claim 1 (refinery smoke: one system per set smoke point, (0, 0, 0) and (128, 128, 0) unset, owner object the refinery, lifetime the game-wide refinery-smoke time; no draw)
#[test]
fn claim_1_refinery_smoke() {
    let mut r = rules();
    r.sim
        .buildings
        .get_mut("TOWER")
        .unwrap()
        .combat
        .natural_particle_system = None;
    let mut w = testkit::game_with(&r, N, 7);
    let b = tower(&mut w);
    let n = w.gas.systems.len();
    let before = w.rng.taken();
    emitters::refinery_smoke(&mut w, b);
    assert_eq!(w.rng.taken(), before, "no draw");
    let made: Vec<_> = w.gas.systems.keys().copied().skip(n).collect();
    assert_eq!(made.len(), 2, "two set points");
    let o = w.obj(b).unwrap().pos;
    let at: Vec<Pos> = made.iter().map(|s| w.gas.systems[s].pos).collect();
    assert_eq!(
        at,
        vec![Pos::new(o.x + 40, o.y, o.z), Pos::new(o.x, o.y + 40, o.z)]
    );
    for s in &made {
        assert_eq!(w.gas.systems[s].lifetime, 7);
        assert_eq!(w.gas.systems[s].owner_object, Some(b));
    }
}

/// spec: S-0303 claim 19 and S-0616 claim 1 (the barrel particle system appears at P with its first particle when it has a held type)
#[test]
fn claim_1_barrel_particle_system() {
    let mut r = rules();
    r.sim.general.barrel_particle = Some("PLUME".into());
    let mut w = testkit::game_with(&r, N, 7);
    let p = Pos::centre(testkit::mid(N, 0, 0), 0);
    emitters::barrel_particle_system(&mut w, p, Some(HouseId(1)));
    let s = *w.gas.systems.keys().last().unwrap();
    assert_eq!(sys_name(&w, s), "PLUME");
    assert_eq!(w.gas.systems[&s].particles.len(), 1);
    assert_eq!(w.gas.systems[&s].target, p);
}

// ---------------------------------------------------------------------------
// Trigger actions 88 and 89 (claim 1; S-0011)

/// spec: S-0616 claim 1 and S-0011's particle-system actions (88: one system of the indexed type at the cell centre, at ground height, no owner; 89: every system whose position lies in the cell is marked, nothing removed at once)
#[test]
fn claim_1_trigger_actions() {
    use glasnost_sim::mp_triggers::{Coverage, action_coverage};
    assert_eq!(action_coverage(88), Coverage::Implemented);
    assert_eq!(action_coverage(89), Coverage::Implemented);
    let mut w = testkit::game_with(&rules(), N, 7);
    let c = testkit::mid(N, 1, 1);
    let ti = w.rules().gas.system("PLUME").unwrap() as i32;
    let before = w.rng.taken();
    emitters::trigger_create(&mut w, ti, c);
    assert_eq!(w.rng.taken(), before);
    let s = *w.gas.systems.keys().last().unwrap();
    assert_eq!(w.gas.systems[&s].pos, Pos::centre(c, 0));
    assert_eq!(w.gas.systems[&s].owner_object, None);
    assert_eq!(w.gas.systems[&s].owner, None);
    let n = w.gas.systems.len();
    emitters::trigger_create(&mut w, 999, c);
    assert_eq!(w.gas.systems.len(), n, "no such type: nothing");
    let other = testkit::mid(N, 4, 1);
    emitters::trigger_create(&mut w, ti, other);
    let s2 = *w.gas.systems.keys().last().unwrap();
    emitters::trigger_mark(&mut w, c);
    assert!(w.gas.systems[&s].ending, "in the cell: marked");
    assert!(!w.gas.systems[&s2].ending, "elsewhere: not");
    assert!(w.gas.systems.contains_key(&s), "nothing removed at once");
}

/// spec: S-0616 Import mapping (absent RefinerySmokeFrames: 25; absent sparking probabilities: red 0.02, yellow 0.01)
#[test]
fn absent_creator_defaults() {
    let w = testkit::game_with(&testkit::combat_ruleset(), N, 7);
    let g = &w.rules().gas;
    assert_eq!(g.refinery_frames, 25);
    assert!(
        g.sparking
            .0
            .eq_num(glasnost_sim::rules::nearest_double("0.02"))
    );
    assert!(
        g.sparking
            .1
            .eq_num(glasnost_sim::rules::nearest_double("0.01"))
    );
}
