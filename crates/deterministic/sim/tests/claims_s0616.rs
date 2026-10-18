//! Claim tests for particle systems of the smoke, fire, spark and railgun
//! kinds (S-0616) on synthetic data: invented particle, particle-system and
//! warhead types, every number our own. The ledger is
//! `docs/m7-claims/S-0616.md`.

use glasnost_num::{F64, Rounding};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, ParticleSystemType, ParticleType, WarheadType};
use glasnost_sim::gas::{self, Particle};
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, ObjectId, Pos, World};
use glasnost_sim::{objects, particles, testkit};

const SIZE: i32 = 20;

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

/// The rules: testkit's combat rules plus
/// - SOOT, a smoke particle (speed 4.0, slowing 0.5, radius 40, next ASH),
///   and ASH, a smoke particle with no next type;
/// - EMBER, a fire particle (speed 6.0, slowing 1.0, damage 30 every tick,
///   warhead PSY, a berserk warhead);
/// - FLICK, a spark particle (X 5, Y 5, Zm 3, Zr 4);
/// - BOLT, a railgun particle;
/// - PLUME (smoke, SOOT, interval 2, slow-down 1.0, cut-off 5.0, radius 10,
///   cap 1), BLAZE (fire, EMBER), BURST (spark, FLICK, one spark turn, cap
///   6), DUD (spark, no held type, two spark turns), RAIL (railgun, BOLT,
///   0.015625 per lepton) and MIXED (gas, holding SOOT);
/// - for the successors across kinds (claim 3, scenario P3 part c): VAPOR,
///   a gas particle (radius 40, next ASH, next-particle displacement
///   (30, −20, 0)); SOOTX and VAPORX, SOOT and VAPOR without a next type;
///   systems MIXEDX (gas, SOOTX), PUFF and PUFFX (smoke, VAPOR and VAPORX),
///   BLAZES and BLAZESX (fire, SOOT and SOOTX), SPARKG and SPARKGX (spark,
///   VAPOR and VAPORX), MIST (gas, VAPOR; VAPOR's speed field 2.0).
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
            "SOOT",
            ParticleType {
                velocity: dec("4.0"),
                deacc: dec("0.5"),
                radius: Some(40),
                next_particle: Some("ASH".into()),
                ..p("smoke")
            },
        ),
        ("ASH", p("smoke")),
        (
            "EMBER",
            ParticleType {
                velocity: dec("6.0"),
                deacc: dec("1.0"),
                damage: Some(30),
                max_dc: Some(1),
                warhead: Some("PSY".into()),
                final_damage_state: Some(50),
                ..p("fire")
            },
        ),
        (
            "FLICK",
            ParticleType {
                x_velocity: Some(5),
                y_velocity: Some(5),
                min_z_velocity: Some(3),
                z_velocity_range: Some(4),
                ..p("spark")
            },
        ),
        ("BOLT", p("railgun")),
        (
            "SOOTX",
            ParticleType {
                velocity: dec("4.0"),
                deacc: dec("0.5"),
                radius: Some(40),
                ..p("smoke")
            },
        ),
        (
            "VAPOR",
            ParticleType {
                velocity: dec("2.0"),
                radius: Some(40),
                next_particle: Some("ASH".into()),
                next_particle_offset: Some([30, -20, 0]),
                ..p("gas")
            },
        ),
        (
            "VAPORX",
            ParticleType {
                radius: Some(40),
                next_particle_offset: Some([30, -20, 0]),
                ..p("gas")
            },
        ),
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
            "PLUME",
            ParticleSystemType {
                spawn_frames: Some(2),
                slowdown: dec("1.0"),
                spawn_cutoff: dec("5.0"),
                spawn_radius: Some(10),
                particle_cap: Some(1),
                ..s("smoke", Some("SOOT"))
            },
        ),
        ("BLAZE", s("fire", Some("EMBER"))),
        (
            "BURST",
            ParticleSystemType {
                spark_spawn_frames: Some(1),
                particle_cap: Some(6),
                ..s("spark", Some("FLICK"))
            },
        ),
        (
            "DUD",
            ParticleSystemType {
                spark_spawn_frames: Some(2),
                ..s("spark", None)
            },
        ),
        (
            "RAIL",
            ParticleSystemType {
                particles_per_coord: dec("0.015625"),
                ..s("railgun", Some("BOLT"))
            },
        ),
        ("MIXED", s("gas", Some("SOOT"))),
        ("MIXEDX", s("gas", Some("SOOTX"))),
        ("PUFF", s("smoke", Some("VAPOR"))),
        ("PUFFX", s("smoke", Some("VAPORX"))),
        ("BLAZES", s("fire", Some("SOOT"))),
        ("BLAZESX", s("fire", Some("SOOTX"))),
        ("SPARKG", s("spark", Some("VAPOR"))),
        ("SPARKGX", s("spark", Some("VAPORX"))),
        ("MIST", s("gas", Some("VAPOR"))),
    ] {
        register(&mut r, "particle_systems", n);
        r.sim.particle_systems.insert(n.into(), t);
    }
    register(&mut r, "warheads", "PSY");
    r.sim.warheads.insert(
        "PSY".into(),
        WarheadType {
            verses: Some((0..11).map(|_| Decimal("1".into())).collect()),
            psychedelic: Some(true),
            ..Default::default()
        },
    );
    r
}

fn game(r: &Ruleset) -> World {
    testkit::game_with(r, SIZE, 7)
}

fn centre(dx: i32, dy: i32) -> Pos {
    Pos::centre(testkit::mid(SIZE, dx, dy), 0)
}

/// A system of type `name` made by a warhead at `p`, owned by `house`.
fn system(w: &mut World, name: &str, p: Pos, house: Option<HouseId>) -> ObjectId {
    gas::warhead_system(w, name, p, house);
    *w.gas.systems.keys().last().unwrap()
}

fn list(w: &World, s: ObjectId) -> Vec<Particle> {
    w.gas.systems[&s].particles.clone()
}

fn named(w: &World, p: &Particle) -> String {
    w.rules().gas.particles[p.type_index as usize].name.clone()
}

fn m() -> Rounding {
    Rounding::Nearest
}

/// spec: S-0616 claim 1 and S-0303 claim 20 (a warhead's system of every kind keeps its first particle at P, with P its target point)
#[test]
fn claim_1_warhead_systems_keep_their_first_particle() {
    let mut w = game(&rules());
    let p = centre(1, -1);
    for name in ["PLUME", "BLAZE", "BURST", "RAIL"] {
        let s = system(&mut w, name, p, Some(HouseId(1)));
        let sys = &w.gas.systems[&s];
        assert_eq!(sys.particles.len(), 1, "{name}");
        assert_eq!(sys.particles[0].pos, p, "{name}");
        assert_eq!(sys.target, p, "{name}");
        assert_eq!(sys.owner, Some(HouseId(1)));
        assert!(w.update.contains(s));
    }
}

/// spec: S-0616 claim 3 (a smoke particle in a gas system updates and moves by the smoke rules: its speed slows by the slowing field above 3.0, and it rises by trunc(S))
#[test]
fn claim_3_particle_kind_chooses_the_update() {
    let mut w = game(&rules());
    w.tick = 10;
    let s = system(&mut w, "MIXED", centre(0, 0), None);
    // S = 4.5, above 3.0 whatever the creation variation.
    w.gas.systems.get_mut(&s).unwrap().particles[0].speed =
        glasnost_sim::rules::nearest_double("4.5").to_f32(m());
    let before = list(&w, s)[0].clone();
    gas::system_turn(&mut w, s);
    let after = &list(&w, s)[0];
    let slowed = before
        .speed
        .to_f64()
        .sub(glasnost_sim::rules::nearest_double("0.5"), m())
        .to_f32(m());
    assert_eq!(after.speed, slowed);
    assert_eq!(after.pos.z, before.pos.z + slowed.to_f64().to_i32_common());
    assert_eq!(after.life, before.life - 1);
}

/// spec: S-0616 claim 4 (a removed owner object: its systems lose it and are marked) and claim 5 (a smoke system follows its owner at its creation displacement)
#[test]
fn claim_4_and_5_owner_object() {
    let mut w = game(&rules());
    let v = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, testkit::mid(SIZE, 0, 0), 0);
    particles::damage_smoke(&mut w, v, "PLUME", [10, -20, 30]);
    let s = w.obj(v).unwrap().combat.damage_smoke.expect("a damage-smoke system");
    let at = w.obj(v).unwrap().pos;
    assert_eq!(w.gas.systems[&s].pos, Pos::new(at.x + 10, at.y - 20, at.z + 30));
    assert!(w.gas.systems[&s].particles.is_empty(), "no first particle");
    w.obj_mut(v).unwrap().pos.x += 256;
    gas::system_turn(&mut w, s);
    assert_eq!(w.gas.systems[&s].pos.x, at.x + 256 + 10, "it follows its owner");
    objects::remove(&mut w, v);
    let sys = &w.gas.systems[&s];
    assert_eq!(sys.owner_object, None);
    assert!(sys.ending);
}

/// spec: S-0616 claim 9 (T grows by the slow-down field after each emission step; the system is marked when T passes the cut-off)
#[test]
fn claim_9_timer_and_cut_off() {
    let mut w = game(&rules());
    let s = system(&mut w, "PLUME", centre(0, 0), None);
    for k in 1..=4 {
        gas::system_turn(&mut w, s);
        let sys = &w.gas.systems[&s];
        assert_eq!(sys.timer.to_f64(), F64::from_i64(2 + k, m()));
        assert_eq!(sys.ending, k == 4, "turn {k}: T = {}", 2 + k);
    }
}

/// spec: S-0616 claim 9 and Edge cases (quirks.smoke_ignores_particle_cap on: a smoke system emits whatever its cap; off: not at or above the cap; the two raw values are taken either way)
#[test]
fn smoke_ignores_particle_cap_both_positions() {
    for on in [true, false] {
        let mut r = rules();
        r.globals.quirks.smoke_ignores_particle_cap = on;
        let mut w = game(&r);
        let s = system(&mut w, "PLUME", centre(0, 0), None);
        // An even tick: the interval ⌊T⌋ = 2 divides it, and smoke takes no update draw.
        w.tick = 4;
        let taken = w.rng.taken();
        gas::system_turn(&mut w, s);
        assert_eq!(list(&w, s).len(), if on { 2 } else { 1 }, "quirk {on}");
        assert!(w.rng.taken() >= taken + 2);
    }
}

/// spec: S-0616 claim 8 and F3 (a dead smoke particle with a next type splits into two at (x ± oₓ, y ± o_y), both with its speed, then is removed)
#[test]
fn claim_8_smoke_successors() {
    let mut w = game(&rules());
    let s = system(&mut w, "PLUME", centre(0, 0), None);
    // An odd tick: no emission (⌊T⌋ = 2).
    w.tick = 5;
    w.gas.systems.get_mut(&s).unwrap().particles[0].life = 1;
    let dead = list(&w, s)[0].clone();
    gas::system_turn(&mut w, s);
    let l = list(&w, s);
    assert_eq!(l.len(), 2);
    assert!(l.iter().all(|p| named(&w, p) == "ASH"));
    assert_eq!(l[0].pos.x + l[1].pos.x, 2 * dead.pos.x);
    assert_eq!(l[0].pos.y + l[1].pos.y, 2 * dead.pos.y);
    assert_ne!(l[0].pos.x, dead.pos.x, "d = 5: the offset is never 0");
    assert_eq!(l[0].speed, l[1].speed);
}

/// spec: S-0616 claim 7, F2 and Edge cases (quirks.smoke_deck_extra_update on: a smoke particle stopped under a bridge deck keeps its dead mark until the next turn's removal pass; off: it is removed, with its successors, in the turn that stops it)
#[test]
fn smoke_deck_extra_update_both_positions() {
    for on in [true, false] {
        let mut r = rules();
        r.globals.quirks.smoke_deck_extra_update = on;
        let mut w = game(&r);
        let c = testkit::mid(SIZE, 0, 0);
        let i = w.cells.idx(c).unwrap();
        w.cells.deck[i] = true;
        let p = Pos::centre(c, 200);
        let s = system(&mut w, "PLUME", p, None);
        w.tick = 5;
        gas::system_turn(&mut w, s);
        let l = list(&w, s);
        let soot: Vec<_> = l.iter().filter(|p| named(&w, p) == "SOOT").collect();
        if on {
            assert_eq!(soot.len(), 1);
            assert!(soot[0].dead, "stopped at z = {}", soot[0].pos.z);
            assert_eq!(soot[0].pos.z, 200, "it does not move");
            w.tick = 7;
            gas::system_turn(&mut w, s);
            assert!(!list(&w, s).iter().any(|p| named(&w, p) == "SOOT"));
        } else {
            assert!(soot.is_empty());
            assert_eq!(l.len(), 2, "its two successors");
        }
    }
}

/// spec: S-0616 claims 12 and 14, Edge cases (quirks.fire_particle_no_house on: a fire particle's hit has no attacking house; off: the system's owner house, so a berserk warhead leaves that house's own unit alone)
#[test]
fn fire_particle_no_house_both_positions() {
    for on in [true, false] {
        let mut r = rules();
        r.globals.quirks.fire_particle_no_house = on;
        let mut w = game(&r);
        let c = testkit::mid(SIZE, 0, 0);
        let victim = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, c, 0);
        let s = system(&mut w, "BLAZE", Pos::centre(c, 0), Some(HouseId(0)));
        gas::system_turn(&mut w, s);
        let berserk = w.obj(victim).unwrap().combat.berserk.is_some();
        assert_eq!(berserk, on, "quirk {on}");
        // No owner object: the fire system is marked (claim 10).
        assert!(w.gas.systems[&s].ending);
    }
}

/// spec: S-0616 claim 12 (S ≤ 0: S becomes 0 and the dead mark, no draw) and claim 10 (the dead are removed in the same turn, no successors)
#[test]
fn claim_12_fire_burns_out() {
    let mut w = game(&rules());
    let s = system(&mut w, "BLAZE", centre(0, 0), None);
    w.gas.systems.get_mut(&s).unwrap().particles[0].speed = glasnost_num::F32(0);
    let rng = w.rng.clone();
    gas::system_turn(&mut w, s);
    assert!(w.gas.systems.get(&s).is_none_or(|x| x.particles.is_empty()));
    assert_eq!(w.rng, rng, "no draw");
}

/// spec: S-0616 claim 15 and F10 (K = 1: no decision draw, a burst of c = (|r₀| rem h) + h sparks, h = trunc(κ / 2), then K = 0 marks the system) and claim 16, Edge cases (quirks.spark_double_gravity on: the vertical step counts gravity twice; off: once)
#[test]
fn spark_burst_and_double_gravity_both_positions() {
    let mut zs = Vec::new();
    for on in [true, false] {
        let mut r = rules();
        r.globals.quirks.spark_double_gravity = on;
        let mut w = game(&r);
        let p = Pos::new(centre(0, 0).x, centre(0, 0).y, 500);
        let s = system(&mut w, "BURST", p, None);
        gas::system_turn(&mut w, s);
        let l = list(&w, s);
        assert!((4..=6).contains(&l.len()), "1 + c with c in 3 … 5: {}", l.len());
        assert!(w.gas.systems[&s].ending);
        assert_eq!(w.gas.systems[&s].sparks, 0);
        zs.push((l.iter().map(|p| p.pos.z).collect::<Vec<_>>(), w.rng.clone()));
    }
    let g = rules().sim.general.gravity.unwrap_or(3);
    let (on, off) = (&zs[0], &zs[1]);
    assert_eq!(on.1, off.1, "the same draws");
    assert_eq!(on.0.len(), off.0.len());
    for (a, b) in on.0.iter().zip(&off.0) {
        assert_eq!(*a, b - g);
    }
}

/// spec: S-0616 claim 15 and claim 20 (no held type: the decision draw at K > 1 and nothing else; K still counts down and marks the system)
#[test]
fn claim_15_spark_without_held_type() {
    let mut w = game(&rules());
    let s = system(&mut w, "DUD", centre(0, 0), None);
    let mut expect = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    let _ = expect.ranged(0, 2147483646, half);
    gas::system_turn(&mut w, s);
    assert_eq!(w.rng, expect, "K = 2: one decision draw");
    assert!(!w.gas.systems[&s].ending);
    gas::system_turn(&mut w, s);
    assert_eq!(w.rng, expect, "K = 1: no draw");
    assert!(w.gas.systems.get(&s).is_none_or(|x| x.ending));
}

/// spec: S-0616 claim 17 and F12 (an unmarked empty railgun system emits n = trunc(L × p) particles once, then is marked)
#[test]
fn claim_17_railgun_line() {
    let mut w = game(&rules());
    let p = centre(-3, 0);
    let s = system(&mut w, "RAIL", p, None);
    {
        let sys = w.gas.systems.get_mut(&s).unwrap();
        sys.particles.clear();
        sys.target = Pos::new(p.x + 1024, p.y, p.z);
    }
    gas::system_turn(&mut w, s);
    assert_eq!(list(&w, s).len(), 16, "trunc(1024 × 0.015625)");
    assert!(w.gas.systems[&s].ending);
    let n = list(&w, s).len();
    gas::system_turn(&mut w, s);
    assert_eq!(list(&w, s).len(), n, "a marked system emits no more");
}

/// spec: S-0616 F5 and Edge cases (quirks.flame_sweep_facing: the original's position on by default; it belongs to the effect-weapon emitter seam, so a fire system's turn is the same in both positions)
#[test]
fn flame_sweep_facing_changes_nothing_yet() {
    assert!(glasnost_rules::quirks::Quirks::default().flame_sweep_facing);
    let mut out = Vec::new();
    for on in [true, false] {
        let mut r = rules();
        r.globals.quirks.flame_sweep_facing = on;
        let mut w = game(&r);
        let c = testkit::mid(SIZE, 0, 0);
        let v = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, c, 0);
        let s = system(&mut w, "BLAZE", Pos::centre(c, 0), Some(HouseId(1)));
        w.gas.systems.get_mut(&s).unwrap().owner_object = Some(v);
        for _ in 0..5 {
            gas::system_turn(&mut w, s);
        }
        out.push((w.gas.systems.get(&s).cloned(), w.rng.clone()));
    }
    assert_eq!(out[0], out[1]);
}

/// spec: S-0616 claim 19 and F13 (without the normalised flag the type's stage-advance field; with it trunc(t / (δ_F + 1) + 1.0), t = |Pₓ − Qₓ| / aₓ; t = 9999.0 with no motion; a divisor of 0 gives 0; no draw)
#[test]
fn claim_19_normalised_stage_advance() {
    let w = game(&rules());
    let ti = w
        .rules()
        .gas
        .particles
        .iter()
        .position(|t| t.name == "EMBER")
        .unwrap();
    let mut t = w.rules().gas.particles[ti].clone();
    let at = Pos { x: 0, y: 0, z: 0 };
    let to = Pos { x: 1000, y: 0, z: 0 };
    let one = glasnost_sim::rules::nearest_double("1.0").to_f32(m());
    let zero = glasnost_sim::rules::nearest_double("0.0").to_f32(m());
    let s = glasnost_sim::rules::nearest_double("10.0").to_f32(m());
    let u = [one, zero, zero];
    t.advance = 3;
    t.normalized = false;
    assert_eq!(particles::own_advance(&w, &t, at, to, u, s), 3);
    t.normalized = true;
    // aₓ = 10, t = 1000 / 10 = 100, δ_F = 4: trunc(100 / 5 + 1) = 21.
    t.final_stage = 4;
    let rng = w.rng.clone();
    assert_eq!(particles::own_advance(&w, &t, at, to, u, s), 21);
    assert_eq!(w.rng, rng, "no draw");
    // No motion: t = 9999.0, trunc(9999 / 5 + 1) = 2000, reduced modulo 256 to −48.
    assert_eq!(particles::own_advance(&w, &t, at, to, [zero; 3], s), -48);
    // δ_F = −1: the quotient has no integer form and gives 0.
    t.final_stage = -1;
    assert_eq!(particles::own_advance(&w, &t, at, to, u, s), 0);
}

/// One turn of a fresh warhead system of type `name` whose first particle
/// has the dead mark, on odd tick 5: the dead particle, the ASH particles
/// (successors) after the turn, the number of particles after the turn
/// and the raw values taken.
fn dying_turn(name: &str) -> (Particle, Vec<Particle>, usize, u64) {
    let mut w = game(&rules());
    let s = system(&mut w, name, centre(0, 0), None);
    w.tick = 5;
    w.gas.systems.get_mut(&s).unwrap().particles[0].dead = true;
    let dead = list(&w, s)[0].clone();
    let taken = w.rng.taken();
    gas::system_turn(&mut w, s);
    let drawn = w.rng.taken() - taken;
    let all = w.gas.systems.get(&s).map(|x| x.particles.clone()).unwrap_or_default();
    let ash: Vec<Particle> = all.iter().filter(|p| named(&w, p) == "ASH").cloned().collect();
    (dead, ash, all.len(), drawn)
}

/// spec: S-0616 claim 3 and scenario P3 part c (successors follow the system's kind, not the particle's: (i) a smoke particle dying in a gas system gets S-0613 claim 7's one successor at its displacement with its creation draws and no claim 8 draw; (ii) a gas particle dying in a smoke system gets claim 8's two mirrored successors with claim 8's draws; (iii) a smoke particle in a fire system and (iv) a gas particle in a spark system get none and no successor draw) and S-0613 claim 7 (revision 9: a dead particle of any kind in a gas system)
#[test]
fn claim_3_successors_follow_the_system_kind() {
    // (i) Gas system, smoke particle: one successor, by S-0613 claim 7.
    let (dead, l, n, with) = dying_turn("MIXED");
    let (_, lx, nx, without) = dying_turn("MIXEDX");
    assert!(lx.is_empty(), "no next type: no successor");
    assert_eq!(l.len(), 1, "one successor");
    assert_eq!(n, nx + 1);
    // SOOT names no displacement: the successor is at the dead particle's position.
    assert_eq!((l[0].pos.x, l[0].pos.y), (dead.pos.x, dead.pos.y));
    assert_eq!(l[0].vel, dead.vel, "the dead particle's velocity");
    assert_eq!(l[0].speed, dead.speed, "the dead particle's speed (S-0613 claim 7, revision 10)");
    let creation = with - without;
    assert!(creation > 0, "the successor's creation draws");
    // (ii) Smoke system, gas particle: claim 8's two successors.
    let (dead, l, n, with) = dying_turn("PUFF");
    let (_, lx, nx, without) = dying_turn("PUFFX");
    assert!(lx.is_empty());
    assert_eq!(l.len(), 2, "two successors");
    assert_eq!(n, nx + 2);
    assert_eq!(l[0].pos.x + l[1].pos.x, 2 * dead.pos.x, "mirrored, not the gas displacement");
    assert_eq!(l[0].pos.y + l[1].pos.y, 2 * dead.pos.y);
    assert_ne!(l[0].pos.x, dead.pos.x, "d = 5 from the dead particle's type: never 0");
    // oₓ and o_y, then per successor its creation draws and one translucency draw.
    assert_eq!(with - without, 2 + 2 * (creation + 1), "claim 8's draws");
    // (iii) Fire system, smoke particle; (iv) spark system, gas particle: none.
    for (name, twin) in [("BLAZES", "BLAZESX"), ("SPARKG", "SPARKGX")] {
        let (_, l, n, with) = dying_turn(name);
        let (_, _, nx, without) = dying_turn(twin);
        assert!(l.is_empty(), "{name}: no successor");
        assert_eq!(n, nx, "{name}");
        assert_eq!(with, without, "{name}: no successor draw");
    }
}

/// spec: S-0613 claim 7 (revision 10) and S-0616 claim 3 (revision 7): a gas successor takes exactly the dead particle's speed and velocity, overwriting its own creation's; a gas parent's speed is its own type's from its creation; nothing else is copied (scenario G5-speed, first variant)
#[test]
fn gas_successor_takes_the_dead_particles_speed() {
    let (dead, l, _, _) = dying_turn("MIST");
    assert_eq!(l.len(), 1, "one smoke successor of the gas particle");
    let ash = &l[0];
    assert_eq!(ash.speed, dead.speed, "the gas parent's speed, not the smoke type's own");
    assert_ne!(dead.speed.to_f64(), F64::from_i64(0, m()), "VAPOR's speed field is 2.0");
    assert_eq!(ash.vel, dead.vel);
    assert_eq!(ash.stage, 0, "its own start stage");
}
