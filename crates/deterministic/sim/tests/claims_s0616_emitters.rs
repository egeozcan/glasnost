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
use glasnost_sim::combat::{fire, railgun};
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

/// spec: S-0616 claim 1 (spark and railgun emitters: owner object the firer; a spark's target point the target object's position, a railgun's the beam's end point of F14, which is the target's position across flat ground) and Terms (held in the spark and railgun places)
#[test]
fn claim_1_spark_and_railgun_emitters() {
    let mut r = rules();
    r.sim.vehicles.get_mut("CART").unwrap().primary = Some("RAILER".into());
    r.sim.weapons.get_mut("RAILER").unwrap().projectile = Some("Rocket".into());
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
    let q = w.obj(b).unwrap().pos;
    let p = w.gas.systems[&rg].pos;
    assert_eq!(
        railgun::walk(&w, p, q).end,
        q,
        "flat ground: the end point is Q"
    );
    assert_eq!(w.gas.systems[&rg].target, q);
    // The target moves: the target point stays fixed (S-0616 claim 1).
    w.obj_mut(b).unwrap().pos = Pos::new(q.x + 300, q.y + 300, q.z);
    let rg_sys = rg;
    let before = w.gas.systems[&rg_sys].target;
    gas::system_turn(&mut w, rg_sys);
    if let Some(s) = w.gas.systems.get(&rg_sys) {
        assert_eq!(s.target, before, "never recomputed from the target");
    }
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

/// spec: S-0306 claim 23 (within rank 4 the other weapon's live-effect impossible comes before the water and land rows; the own kind's rearming is rank 5, below them) and S-0301 claim 5
#[test]
fn s0306_claim_23_other_weapons_live_effect_before_the_land_row() {
    let mut r = rules();
    r.sim
        .vehicles
        .get_mut("CART")
        .unwrap()
        .combat
        .land_targeting = Some(1);
    let (mut w, a, _) = duel(&r, 4);
    let ask = |w: &World, slot| check(w, a, w.obj(a).unwrap().target, slot, true);
    // No live effect: the land row refuses the ground target to both weapons.
    assert_eq!(ask(&w, 0), Answer::Illegal);
    assert_eq!(ask(&w, 1), Answer::Illegal);
    // A shot made without the check leaves the primary's fire emitter alive.
    let t = w.obj(a).unwrap().target.unwrap();
    assert!(fire::shot(&mut w, a, 0, t));
    w.obj_mut(a).unwrap().combat.rearm_until = 0;
    assert_eq!(
        ask(&w, 1),
        Answer::Impossible,
        "the other weapon's kind alive"
    );
    assert_eq!(ask(&w, 0), Answer::Illegal, "own kind alive: rank 5");
    emitters::let_go_fire(&mut w, a);
    assert_eq!(ask(&w, 1), Answer::Illegal);
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

/// spec: S-0616 claim 1 (a fire emitter is marked and let go on a move order, on a new target not within reach, and when the firer dies; a new target within reach keeps it); S-0301 claim 5, Moment of a retarget's let-go (revision 67: the retarget's test at the change of the target)
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
    // spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: the test is made at the change of the firer's target, not in the system's turn)
    emitters::retarget_let_go(&mut w, a);
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
    emitters::retarget_let_go(&mut w, a);
    assert_eq!(
        w.obj(a).unwrap().combat.fire_emitter,
        None,
        "out of reach: let go at the change"
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
            let k = w.obj(a).unwrap().combat.burst;
            assert_eq!(
                sys.pos,
                glasnost_sim::combat::position::fire_point(&w, a, 0, k),
                "the owner's current burst counter"
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

/// spec: S-0616 claim 1 (natural smoke: made when the building is placed in play, at its position plus the displacement, no owner object, target point its position; T × 1.5 after each now-yellow hit, compounding) and S-0516 claim 18 (full cloaking removes it at once; full uncloaking makes a new one only with a displacement other than (0, 0, 0))
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
    // Each now-yellow hit: × 1.5, compounding.
    let t0 = sys.timer;
    emitters::natural_smoke_yellow(&mut w, b);
    let six = glasnost_sim::rules::nearest_double("6.0").to_f32(m());
    assert_eq!(t0, glasnost_sim::rules::nearest_double("4.0").to_f32(m()));
    assert_eq!(w.gas.systems[&s].timer, six, "4.0 × 1.5");
    emitters::natural_smoke_yellow(&mut w, b);
    let nine = glasnost_sim::rules::nearest_double("9.0").to_f32(m());
    assert_eq!(w.gas.systems[&s].timer, nine, "6.0 × 1.5: it compounds");
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

/// spec: S-0312 claim 19 f and S-0616 claim 1 (destruction smoke: one draw R(0, m − 1) over the smoke-kind entries counted from the end, at its position plus the destruction displacement, owner object the building, no owner house; the building's damage smoke and natural smoke do not count as attached)
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
    assert_eq!(w.gas.systems[&s].owner, None);
    // A damage smoke and a natural smoke do not stop it (Q-1400).
    let mut w = testkit::game_with(&rules(), N, 7);
    let b = tower(&mut w);
    assert!(w.obj(b).unwrap().combat.natural_smoke.is_some());
    let other = *w.gas.systems.keys().last().unwrap();
    w.obj_mut(b).unwrap().combat.damage_smoke = Some(other);
    let n = w.gas.systems.len();
    let before = w.rng.taken();
    emitters::destruction_smoke(&mut w, b);
    assert_eq!((w.gas.systems.len(), w.rng.taken()), (n + 1, before + 1));
    // One eligible entry: R(0, 0) takes no raw value (S-0202 claim 4).
    let mut r = rules();
    r.sim
        .buildings
        .get_mut("TOWER")
        .unwrap()
        .combat
        .destroy_particle_systems = Some(vec!["BURST".into(), "PLUME".into(), "BLAZE".into()]);
    let mut w = testkit::game_with(&r, N, 7);
    let b = tower(&mut w);
    let before = w.rng.taken();
    emitters::destruction_smoke(&mut w, b);
    let s = *w.gas.systems.keys().last().unwrap();
    assert_eq!(sys_name(&w, s), "PLUME", "only the smoke entry");
    assert_eq!(w.rng.taken(), before);
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

/// spec: S-0303 claim 19 and S-0616 claim 1 (the barrel particle system appears at P with its first particle when it has a held type; owner house none)
#[test]
fn claim_1_barrel_particle_system() {
    let mut r = rules();
    r.sim.general.barrel_particle = Some("PLUME".into());
    let mut w = testkit::game_with(&r, N, 7);
    let p = Pos::centre(testkit::mid(N, 0, 0), 0);
    emitters::barrel_particle_system(&mut w, p);
    let s = *w.gas.systems.keys().last().unwrap();
    assert_eq!(sys_name(&w, s), "PLUME");
    assert_eq!(w.gas.systems[&s].owner, None, "owner house: none (Q-1403)");
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

// ---------------------------------------------------------------------------
// Revision 8: several flags, the railgun shot, the sweep's burst counter,
// the now-yellow factor, the self-healing marks, the spark-turns fix

/// spec: S-0616 claim 1 (Q-1403: the fire, spark and railgun flags are tested independently at the shot, in that order; one emitter of the weapon's system type per flag whose place is empty) and S-0301 claim 5 (revision 24: the gate refuses the weapon while any place of its flags is held)
#[test]
fn claim_1_several_flags_make_several_emitters() {
    let mut r = rules();
    r.sim.weapons.insert("TRIPLE".into(), {
        let mut w = weapon(Some("BURST"));
        w.use_fire_particles = Some(true);
        w.use_spark_particles = Some(true);
        w.is_railgun = Some(true);
        w.projectile = Some("Rocket".into());
        w
    });
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.primary = Some("TRIPLE".into());
    cart.secondary = Some("PLAIN".into());
    let (mut w, a, _) = duel(&r, 4);
    let t = w.obj(a).unwrap().target.unwrap();
    let n = w.gas.systems.len();
    assert!(fire::shot(&mut w, a, 0, t));
    assert_eq!(w.gas.systems.len(), n + 3, "three emitters at one shot");
    let c = w.obj(a).unwrap().combat.clone();
    let (f, s, g) = (
        c.fire_emitter.expect("fire place").system,
        c.spark_emitter.expect("spark place"),
        c.railgun_emitter.expect("railgun place"),
    );
    assert!(f < s && s < g, "made in the order fire, spark, railgun");
    for x in [f, s, g] {
        assert_eq!(sys_name(&w, x), "BURST", "the weapon's one system type");
    }
    let ask = |w: &World| check(w, a, w.obj(a).unwrap().target, 0, true);
    w.obj_mut(a).unwrap().combat.rearm_until = 0;
    assert_eq!(ask(&w), Answer::Rearming);
    // Letting the fire emitter go is not enough: the others are still held.
    emitters::let_go_fire(&mut w, a);
    assert_eq!(ask(&w), Answer::Rearming, "spark and railgun still held");
    for x in [s, g] {
        for _ in 0..200 {
            if !w.gas.systems.contains_key(&x) {
                break;
            }
            gas::system_turn(&mut w, x);
        }
    }
    assert_eq!(ask(&w), Answer::CanFire, "every place empty again");
    // A second shot with the spark place held makes no spark emitter.
    assert!(fire::shot(&mut w, a, 0, t));
    let held = w.obj(a).unwrap().combat.spark_emitter.unwrap();
    w.obj_mut(a).unwrap().combat.fire_emitter = None;
    w.obj_mut(a).unwrap().combat.railgun_emitter = None;
    let n = w.gas.systems.len();
    assert!(fire::shot(&mut w, a, 0, t));
    assert_eq!(w.gas.systems.len(), n + 2, "fire and railgun only");
    assert_eq!(w.obj(a).unwrap().combat.spark_emitter, Some(held));
}

/// The railgun rules: CART's primary RAILER (damage 50, the slow Rocket
/// projectile so that no projectile hit lands at the shot).
fn rail_rules() -> Ruleset {
    let mut r = rules();
    r.sim.vehicles.get_mut("CART").unwrap().primary = Some("RAILER".into());
    let rw = r.sim.weapons.get_mut("RAILER").unwrap();
    rw.projectile = Some("Rocket".into());
    rw.damage = Some(50);
    r
}

fn hp(w: &World, id: ObjectId) -> i32 {
    w.obj(id).unwrap().health
}

/// spec: S-0306 claim 27 (objects of the cells the samples newly enter qualify when strictly nearer the beam line than the railgun hit distance, with no alliance test, never the firer; the target always; one ordinary hit each when no sample stops in terrain) and Import mapping (absent railgundamageradius 128, CollapseChance 100)
#[test]
fn s0306_claim_27_railgun_hits_along_the_beam() {
    let r = rail_rules();
    let (mut w, a, b) = duel(&r, 6);
    assert_eq!(w.rules().combat.general.railgun_radius, 128);
    assert_eq!(w.rules().combat.general.collapse_chance, 100);
    // A friendly bystander on the beam, an enemy two cells off it.
    let friend = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, testkit::mid(N, 0, 0), 0);
    let off = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, testkit::mid(N, 1, 2), 0);
    let before: Vec<i32> = [a, b, friend, off].iter().map(|&x| hp(&w, x)).collect();
    let t = w.obj(a).unwrap().target.unwrap();
    assert!(fire::shot(&mut w, a, 0, t));
    assert_eq!(hp(&w, a), before[0], "never the firer");
    assert!(hp(&w, b) < before[1], "the target");
    assert!(hp(&w, friend) < before[2], "no alliance test");
    assert_eq!(hp(&w, off), before[3], "beyond the hit distance");
    assert_eq!(
        before[1] - hp(&w, b),
        before[2] - hp(&w, friend),
        "the same ordinary hit"
    );
    // A second shot while the railgun place is held: no effect at all.
    let held = w.obj(a).unwrap().combat.railgun_emitter.unwrap();
    let h = hp(&w, b);
    w.obj_mut(a).unwrap().combat.rearm_until = 0;
    assert!(fire::shot(&mut w, a, 0, t));
    assert_eq!(hp(&w, b), h, "no beam while the place is held");
    assert_eq!(w.obj(a).unwrap().combat.railgun_emitter, Some(held));
    // The hit distance 0: only the target.
    let mut r0 = rail_rules();
    r0.sim.general.railgun_damage_radius = Some(0);
    let (mut w, a, b) = duel(&r0, 6);
    let friend = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, testkit::mid(N, 0, 0), 0);
    let (hb, hf) = (hp(&w, b), hp(&w, friend));
    assert!(fire::shot(&mut w, a, 0, Target::Object(b)));
    assert!(hp(&w, b) < hb);
    assert_eq!(hp(&w, friend), hf, "distance 0: nothing is strictly nearer");
}

/// spec: S-0306 claim 27 (revision 15, Q-1405 answered: Degenerate comparisons, a candidate exactly on an axis-aligned beam or any candidate of a zero-length beam has a line distance that is not a number and qualifies whatever the hit distance) and Edge cases (quirks.railgun_collinear_hits off: the finite Euclidean distance to the line, to the start for a zero-length beam, strictly below the hit distance)
#[test]
fn s0306_claim_27_collinear_hits_both_positions() {
    use railgun::near_line;
    let (p, q) = (Pos::new(1000, 1000, 0), Pos::new(5000, 1000, 0));
    let on_line = Pos::new(3000, 1000, 0);
    let off_start = Pos::new(1300, 1000, 0);
    for collinear in [true, false] {
        let mut r = rail_rules();
        r.globals.quirks.railgun_collinear_hits = collinear;
        let (w, _, _) = duel(&r, 6);
        // On the beam's line, hit distance 0 and below.
        assert_eq!(near_line(&w, p, q, on_line, 0), collinear, "on the line");
        assert_eq!(near_line(&w, p, q, on_line, -5), collinear, "below 0");
        // A zero-length beam: 300 leptons from its start.
        assert_eq!(
            near_line(&w, p, p, off_start, 128),
            collinear,
            "zero-length"
        );
        // Off the line by 200 leptons: finite in both positions.
        let off_line = Pos::new(3000, 1200, 0);
        assert!(!near_line(&w, p, q, off_line, 128));
        assert!(near_line(&w, p, q, off_line, 256));
        if !collinear {
            // The zero-length beam measures to its start.
            assert!(near_line(&w, p, p, off_start, 400), "to the start");
        }
    }
}

/// A destroyable-cliff tile in the catalogue of `w`, laid on cell `c` when given.
fn cliff_tile(w: &mut World, c: Option<glasnost_map::CellXY>) {
    use glasnost_map::terrain::{TileCell, TileInfo};
    let mut cat = (*testkit::catalogue()).clone();
    let n = cat.tiles.len() as i32;
    cat.tiles.push(TileInfo {
        width: 1,
        height: 1,
        has_image: true,
        cells: vec![Some(TileCell {
            terrain_code: 0,
            slope: 0,
        })],
        variants: 0,
        anim: None,
        no_burrowing: false,
    });
    cat.roles.destroyable_cliff = n;
    w.catalogue = Some(std::sync::Arc::new(cat));
    if let Some(c) = c {
        let cell = w.cells.grid.get_mut(c).unwrap();
        cell.tile = n;
        cell.sub_tile = 0;
    }
}

/// spec: S-0616 F14 (the first tested sample below the terrain is the end point) and S-0306 claim 27 (a terrain stop cancels every hit, the earlier bystander's too; one draw R(0, 99) only when the stop cell is a destroyable cliff)
#[test]
fn s0306_claim_27_terrain_stop() {
    let mut draws = Vec::new();
    for cliff in [false, true] {
        let r = rail_rules();
        let (mut w, a, b) = duel(&r, 6);
        let friend = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, testkit::mid(N, -1, 0), 0);
        let ridge = testkit::mid(N, 1, 0);
        w.cells.grid.get_mut(ridge).unwrap().level = 8;
        cliff_tile(&mut w, cliff.then_some(ridge));
        let (hb, hf) = (hp(&w, b), hp(&w, friend));
        let t = w.obj(a).unwrap().target.unwrap();
        let before = w.rng.taken();
        let mut probe = w.rng.clone();
        probe.ranged(0, 99, w.rules().quirks.ranged_draw_half_span);
        assert!(fire::shot(&mut w, a, 0, t));
        draws.push((w.rng.taken() - before, probe.taken() - before));
        assert_eq!(
            (hp(&w, b), hp(&w, friend)),
            (hb, hf),
            "cliff {cliff}: no hits"
        );
        let g = w.obj(a).unwrap().combat.railgun_emitter.unwrap();
        let end = w.gas.systems[&g].target;
        assert_eq!(end.cell(), ridge, "the first sample in the ridge");
        let p = w.gas.systems[&g].pos;
        let q = w.obj(b).unwrap().pos;
        let walk = railgun::walk(&w, p, q);
        assert_eq!((walk.end, walk.stop), (end, Some(ridge)));
        assert!((0..50).any(|i| railgun::sample(p, q, i, w.rules().mode) == end));
    }
    assert_eq!(draws[0].0, 0, "an ordinary ridge: no draw");
    assert_eq!(
        draws[1].0, draws[1].1,
        "a cliff stop: one ranged draw R(0, 99)"
    );
}

/// spec: S-0306 claim 27 (without a terrain stop, one draw R(0, 99) after the hits only when the shot target is a destroyable-cliff cell; an ordinary cell target takes none)
#[test]
fn s0306_claim_27_cliff_cell_target() {
    let mut draws = Vec::new();
    for cliff in [false, true] {
        let r = rail_rules();
        let (mut w, a, _) = duel(&r, 6);
        let c = testkit::mid(N, 2, 1);
        cliff_tile(&mut w, cliff.then_some(c));
        let before = w.rng.taken();
        let mut probe = w.rng.clone();
        probe.ranged(0, 99, w.rules().quirks.ranged_draw_half_span);
        assert!(fire::shot(&mut w, a, 0, Target::Cell(c)));
        draws.push((w.rng.taken() - before, probe.taken() - before));
    }
    assert_eq!(draws[0].0, 0, "an ordinary cell target: no draw");
    assert_eq!(
        draws[1].0, draws[1].1,
        "a cliff cell target: one ranged draw R(0, 99)"
    );
}

/// spec: S-0616 claim 11 (Q-1403: the sweep's fire point uses the owner's current burst counter, not the shot's and not 0) and S-0313 claim 2 (the lateral part negated for an odd counter)
#[test]
fn claim_11_sweep_uses_the_current_burst() {
    let mut r = rules();
    r.sim
        .vehicles
        .get_mut("CART")
        .unwrap()
        .combat
        .primary_fire_flh = Some([40, 64, 20]);
    let (mut w, a, _) = duel(&r, 9);
    let t = w.obj(a).unwrap().target.unwrap();
    assert!(fire::shot(&mut w, a, 0, t));
    let s = w.obj(a).unwrap().combat.fire_emitter.unwrap().system;
    for k in [1, 0, 3] {
        w.tick = 33;
        let tick = w.tick;
        let o = w.obj_mut(a).unwrap();
        o.combat.burst = k;
        o.body = glasnost_sim::turning::Turning::at(0);
        o.body.turn_to(20000, tick, 1);
        let even = glasnost_sim::combat::position::fire_point(&w, a, 0, 0);
        let odd = glasnost_sim::combat::position::fire_point(&w, a, 0, 1);
        assert_ne!(even, odd, "the fire point alternates");
        particles::fire_turn(&mut w, s);
        assert_eq!(
            w.gas.systems[&s].pos,
            if k % 2 == 1 { odd } else { even },
            "burst {k}"
        );
    }
}

/// A TOWER hit for `amount` from `from` health (strength 800), defences ignored.
fn hit_tower(
    w: &mut World,
    b: ObjectId,
    from: i32,
    amount: i32,
) -> glasnost_sim::combat::damage::HitResult {
    w.obj_mut(b).unwrap().health = from;
    glasnost_sim::combat::damage::apply(
        w,
        b,
        glasnost_sim::combat::damage::Hit {
            amount,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    )
}

/// spec: S-0616 claim 1 (after each hit that reports now yellow, S-0302 claim 12, T × 1.5; a hit from above half straight to red reports now red and leaves T unchanged; repair does not undo it and a later now-yellow hit compounds)
#[test]
fn claim_1_natural_smoke_now_yellow_hits_only() {
    use glasnost_sim::combat::damage::HitResult;
    let f = |s: &str| glasnost_sim::rules::nearest_double(s).to_f32(m());
    let mut w = testkit::game_with(&rules(), N, 7);
    let b = tower(&mut w);
    let s = w.obj(b).unwrap().combat.natural_smoke.unwrap();
    assert_eq!(hit_tower(&mut w, b, 800, 401), HitResult::NowYellow);
    assert_eq!(w.gas.systems[&s].timer, f("6.0"));
    // Repair above half (health set back), then a second now-yellow hit.
    assert_eq!(hit_tower(&mut w, b, 800, 401), HitResult::NowYellow);
    assert_eq!(w.gas.systems[&s].timer, f("9.0"), "it compounds");
    // A new tower, one hit from full straight below the red ratio.
    let mut w = testkit::game_with(&rules(), N, 7);
    let b = tower(&mut w);
    let s = w.obj(b).unwrap().combat.natural_smoke.unwrap();
    assert_eq!(hit_tower(&mut w, b, 800, 700), HitResult::NowRed);
    assert_eq!(w.gas.systems[&s].timer, f("4.0"), "now red: unchanged");
}

/// Whether a damage smoke of `kind`/`name` (house 0, at `z`) is marked by
/// one self-healing turn from `health`, with `own` self-healing and the
/// house's self-heal counts `house` (infantry, vehicles), organic or not.
fn heal_marks(
    kind: Kind,
    name: &str,
    health: i32,
    own: bool,
    house: [i32; 2],
    organic: bool,
    z: i32,
) -> bool {
    let mut r = rules();
    {
        let c = match kind {
            Kind::Infantry => &mut r.sim.infantry.get_mut(name).unwrap().combat,
            Kind::Vehicle => &mut r.sim.vehicles.get_mut(name).unwrap().combat,
            _ => &mut r.sim.buildings.get_mut(name).unwrap().combat,
        };
        c.self_healing = Some(own);
        c.organic = Some(organic);
    }
    let g = &mut r.sim.general;
    g.self_heal_infantry_frames = Some(1);
    g.self_heal_unit_frames = Some(1);
    g.self_heal_infantry_amount = Some(1);
    g.self_heal_unit_amount = Some(1);
    let mut w = testkit::game_with(&r, N, 7);
    let id = testkit::spawn(&mut w, kind, name, 0, testkit::mid(N, 4, -4), 0);
    w.houses[0].combat.self_heal = house;
    let ti = w.rules().gas.system("HAZE").unwrap() as i32;
    emitters::trigger_create(&mut w, ti, testkit::mid(N, -5, -5));
    let sys = *w.gas.systems.keys().last().unwrap();
    let o = w.obj_mut(id).unwrap();
    o.health = health;
    o.combat.damage_smoke = Some(sys);
    o.pos.z = z;
    // A multiple of the repair period (14) for the own self-healing.
    w.tick = 28;
    let h0 = w.obj(id).unwrap().health;
    glasnost_sim::combat::heal::turn(&mut w, id);
    assert!(w.obj(id).unwrap().health > h0, "{name}: a gain");
    assert_eq!(
        w.obj(id).unwrap().combat.damage_smoke,
        Some(sys),
        "kept in its place"
    );
    w.gas.systems[&sys].ending
}

/// spec: S-0616 claim 1 and S-0314 claims 9 and 10 (after its own self-healing gain, and after a non-organic vehicle's house-wide gain, the damage smoke is marked when the health ratio is strictly above the yellow ratio or the height above the surface is below −10 leptons; house-wide healing of infantry and organic vehicles takes no test)
#[test]
fn claim_1_self_healing_marks_damage_smoke() {
    // Own self-healing, each kind: 50 % + 1 is strictly above.
    assert!(heal_marks(
        Kind::Vehicle,
        "BUGGY",
        15000,
        true,
        [0, 0],
        false,
        0
    ));
    assert!(heal_marks(
        Kind::Infantry,
        "WALKER",
        50,
        true,
        [0, 0],
        false,
        0
    ));
    assert!(heal_marks(
        Kind::Building,
        "TOWER",
        400,
        true,
        [0, 0],
        false,
        0
    ));
    // Still at or below the yellow ratio: not marked.
    assert!(!heal_marks(
        Kind::Vehicle,
        "BUGGY",
        14000,
        true,
        [0, 0],
        false,
        0
    ));
    // Below −10 leptons: marked whatever the ratio.
    assert!(heal_marks(
        Kind::Vehicle,
        "BUGGY",
        100,
        true,
        [0, 0],
        false,
        -11
    ));
    assert!(!heal_marks(
        Kind::Vehicle,
        "BUGGY",
        100,
        true,
        [0, 0],
        false,
        -10
    ));
    // House-wide: a vehicle that is not organic; infantry and organic vehicles take no test.
    assert!(heal_marks(
        Kind::Vehicle,
        "BUGGY",
        15000,
        false,
        [0, 1],
        false,
        0
    ));
    assert!(!heal_marks(
        Kind::Infantry,
        "WALKER",
        50,
        false,
        [1, 0],
        false,
        0
    ));
    assert!(!heal_marks(
        Kind::Vehicle,
        "BUGGY",
        15000,
        false,
        [1, 1],
        true,
        0
    ));
}

/// spec: S-0616 claim 20 (a spark system whose type's spark-turns field is 0 or less is marked in its first turn with no burst and no draw, and leaves once it holds no particle, freeing the spark place) and Import mapping (SparkSpawnFrames absent: 0)
#[test]
fn claim_20_spark_turns_zero_frees_the_place() {
    for turns in [None, Some(0), Some(-3)] {
        let mut r = rules();
        let burst = r.sim.particle_systems.get_mut("BURST").unwrap();
        burst.spark_spawn_frames = turns;
        let (mut w, a, _) = duel(&r, 4);
        let t = w.obj(a).unwrap().target.unwrap();
        assert!(fire::shot(&mut w, a, 1, t));
        let sp = w.obj(a).unwrap().combat.spark_emitter.unwrap();
        let before = w.rng.taken();
        gas::system_turn(&mut w, sp);
        assert_eq!(w.rng.taken(), before, "{turns:?}: no burst, no draw");
        assert!(
            !w.gas.systems.contains_key(&sp),
            "{turns:?}: empty, it leaves"
        );
        assert_eq!(w.obj(a).unwrap().combat.spark_emitter, None);
    }
}

/// spec: S-0616 claim 1 and S-0405 claim 13 (revision 19: only S-0405 claim 13's disabling makes a vehicle deactivated) and S-0616 claim 1 (Q-1404: its spark step comes first in its turn, so its displacements are the turn's first two draws)
#[test]
fn claim_1_deactivated_sparks_first_in_the_turn() {
    let mut r = rules();
    r.sim.general.default_spark_system = Some("BURST".into());
    let mut w = testkit::game_with(&r, N, 7);
    let v = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, testkit::mid(N, 0, 0), 0);
    let o = w.obj(v).unwrap().pos;
    let term = |t: i32| (o.x / 10 + (o.y / 10) * 65536 + t + v.0 as i32).rem_euclid(30);
    // Not disabled: no sparks on any tick of the cadence.
    let n = w.gas.systems.len();
    for _ in 0..30 {
        testkit::run(&mut w, 1);
    }
    assert_eq!(w.gas.systems.len(), n, "not deactivated");
    // Disabled: the turn's first draws are its displacements.
    w.obj_mut(v).unwrap().econ.building.unit_disabled = true;
    while term(w.tick) != 0 {
        testkit::run(&mut w, 1);
    }
    let mut probe = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    let dy = probe.ranged(-100, 100, half);
    let dx = probe.ranged(-100, 100, half);
    let n = w.gas.systems.len();
    testkit::run(&mut w, 1);
    let made: Vec<_> = w
        .gas
        .systems
        .values()
        .filter(|s| s.owner_object.is_none() && s.pos == Pos::new(o.x + dx, o.y + dy, o.z))
        .collect();
    assert!(w.gas.systems.len() > n);
    assert_eq!(made.len(), 1, "displaced by the turn's first two draws");
}

/// The destruction smoke made when a TOWER (exploding or not) is killed:
/// (the system, whether the building is still in play, whether the system
/// is marked), right after the kill and after the next tick.
fn destroyed_tower(explodes: bool) -> [(bool, bool); 2] {
    destroyed_tower_selling(explodes, false)
}

fn destroyed_tower_selling(explodes: bool, selling: bool) -> [(bool, bool); 2] {
    let mut r = rules();
    r.sim.buildings.get_mut("TOWER").unwrap().combat.explodes = Some(explodes);
    let mut w = testkit::game_with(&r, N, 7);
    let b = tower(&mut w);
    if selling {
        w.obj_mut(b).unwrap().beh.current = Some(glasnost_sim::rules::Behaviour::Selling);
    }
    let n = w.gas.systems.keys().last().copied();
    hit_tower(&mut w, b, 800, 100_000);
    let made: Vec<ObjectId> = w
        .gas
        .systems
        .keys()
        .copied()
        .filter(|id| Some(*id) > n)
        .collect();
    assert_eq!(made.len(), 1, "one destruction smoke");
    let s = made[0];
    assert!(["PLUME", "HAZE"].contains(&sys_name(&w, s).as_str()));
    let now = (w.obj(b).is_some(), w.gas.systems[&s].ending);
    testkit::run(&mut w, 1);
    let next = (
        w.obj(b).is_some(),
        w.gas.systems.get(&s).is_none_or(|s| s.ending),
    );
    [now, next]
}

/// spec: S-0312 claim 19 f (revision 33) and S-0616 claim 1 (Q-1400: a building that does not explode leaves play at once and its destruction smoke is marked before its first turn; an exploding one stays until its next turn and its smoke is marked only at its removal) and claim 4
#[test]
fn claim_1_destruction_smoke_until_removal() {
    let [now, _] = destroyed_tower(false);
    assert_eq!(now, (false, true), "gone at once: marked at once");
    let [now, next] = destroyed_tower(true);
    assert_eq!(now, (true, false), "awaiting removal: not marked");
    assert_eq!(next, (false, true), "removed in its next turn: marked");
}

/// spec: S-0616 claim 1 (revision 9: a building awaiting removal under S-0312 claim 21 because it was being sold when destroyed, though its type does not explode, keeps its destruction smoke until its removal, which marks it) and scenario P21 part d
#[test]
fn claim_1_destruction_smoke_of_a_sold_building() {
    let [now, next] = destroyed_tower_selling(false, true);
    assert_eq!(
        now,
        (true, false),
        "being sold: awaiting removal, not marked"
    );
    assert_eq!(next, (false, true), "removed in its next turn: marked");
}

/// spec: S-0301 claim 5 and Terms (revision 58: a matched emitter is alive from the shot until it is let go or its system leaves play, whichever comes first; a fire-particle weapon whose firer keeps its target, stays put and lives fires at the first attempt after both its system has left play and R′ ticks after the shot) and scenario F4 particle part (the second fire firer, a system lifetime L longer than R)
#[test]
fn s0301_claim_5_matched_fire_emitter_ends_with_its_system() {
    let mut r = rules();
    // BLAZE with a positive lifetime of 45 turns, longer than FLAMER's rate 30.
    r.sim.particle_systems.get_mut("BLAZE").unwrap().lifetime = Some(45);
    let (mut w, a, _) = duel(&r, 4);
    let t = w.obj(a).unwrap().target.unwrap();
    let ask = |w: &World| check(w, a, w.obj(a).unwrap().target, 0, true);
    assert!(fire::shot(&mut w, a, 0, t));
    let sys = w.obj(a).unwrap().combat.fire_emitter.unwrap().system;
    // Past R′ the system is still in play: its own kind holds the weapon back.
    w.obj_mut(a).unwrap().combat.rearm_until = 0;
    assert_eq!(ask(&w), Answer::Rearming, "the emitter is alive");
    let mut turns = 0;
    while w.gas.systems.contains_key(&sys) && turns < 2000 {
        assert_eq!(ask(&w), Answer::Rearming, "alive at turn {turns}");
        gas::system_turn(&mut w, sys);
        turns += 1;
    }
    assert!(!w.gas.systems.contains_key(&sys), "the system leaves play");
    assert!(turns >= 45, "not before its lifetime");
    // The firer kept its target and stood still: the place is empty and the
    // weapon fires again.
    assert_eq!(w.obj(a).unwrap().combat.fire_emitter, None);
    assert_eq!(w.obj(a).unwrap().target, Some(t));
    assert_eq!(ask(&w), Answer::CanFire);
    // With a lifetime of 0 or less the system never times out: the weapon
    // waits for a let-go.
    let (mut w, a, _) = duel(&rules(), 4);
    let t = w.obj(a).unwrap().target.unwrap();
    assert!(fire::shot(&mut w, a, 0, t));
    let sys = w.obj(a).unwrap().combat.fire_emitter.unwrap().system;
    w.obj_mut(a).unwrap().combat.rearm_until = 0;
    for _ in 0..200 {
        gas::system_turn(&mut w, sys);
    }
    assert!(w.gas.systems.contains_key(&sys));
    assert_eq!(
        check(&w, a, w.obj(a).unwrap().target, 0, true),
        Answer::Rearming
    );
}
