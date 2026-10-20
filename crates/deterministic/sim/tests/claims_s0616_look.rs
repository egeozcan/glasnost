//! Claim tests for the particle state that S-0616's drawing reads (claims
//! 26 and 29, F15, F16): each particle's translucency value, its colour
//! state and its creation position and target point. Synthetic data:
//! invented particle and particle-system types, every number our own.

use glasnost_num::{F64, Rounding};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, ParticleSystemType, ParticleType};
use glasnost_sim::gas::{self, ColourState, ParticleLook};
use glasnost_sim::rules::nearest_double;
use glasnost_sim::world::{ObjectId, Pos, World};
use glasnost_sim::{particles, testkit};

const SIZE: i32 = 20;
const M: Rounding = Rounding::Nearest;

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
/// - HAZE, a smoke particle with no next type, translucency 281;
/// - FLARE, a fire particle (speed 6.0, no slowing, end stage 6, a stage
///   every tick) with translucent-25 stage 2 and translucent-50 stage 4;
///   FLARE2 with both fields 2; FLARE3 with neither;
/// - GLINT, a spark particle with colours (10, 20, 30), (200, 100, 0),
///   (0, 0, 250), black start colours and colour speed 0.3; GLINT2 the
///   same with start colours (0, 0, 0) and (100, 200, 40);
/// - systems FOGA and FOGB (smoke, HAZE, an emission every tick, timer
///   1.0, translucency cut-offs 0.5 and 1.0), TORCH, TORCH2 and TORCH3
///   (fire), SHOWER and SHOWER2 (spark, one turn, cap 3).
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let p = |kind: &str| ParticleType {
        behaves_like: Some(kind.into()),
        max_ec: Some(1000),
        state_ai_advance: Some(1),
        ..Default::default()
    };
    let flare = ParticleType {
        velocity: dec("6.0"),
        end_state_ai: Some(6),
        translucent25_state: Some(2),
        translucent50_state: Some(4),
        ..p("fire")
    };
    let glint = ParticleType {
        x_velocity: Some(5),
        y_velocity: Some(5),
        min_z_velocity: Some(40),
        z_velocity_range: Some(4),
        color_count: Some(3),
        color_list: Some(vec![[10, 20, 30], [200, 100, 0], [0, 0, 250]]),
        color_speed: dec("0.3"),
        ..p("spark")
    };
    for (n, t) in [
        (
            "HAZE",
            ParticleType {
                translucency: Some(281),
                end_state_ai: Some(100),
                ..p("smoke")
            },
        ),
        ("FLARE", flare.clone()),
        (
            "FLARE2",
            ParticleType {
                translucent50_state: Some(2),
                ..flare.clone()
            },
        ),
        (
            "FLARE3",
            ParticleType {
                translucent25_state: None,
                translucent50_state: None,
                ..flare
            },
        ),
        ("GLINT", glint.clone()),
        (
            "GLINT2",
            ParticleType {
                start_color2: Some([100, 200, 40]),
                ..glint
            },
        ),
    ] {
        register(&mut r, "particles", n);
        r.sim.particles.insert(n.into(), t);
    }
    let fog = |cut: &str| ParticleSystemType {
        behaves_like: Some("smoke".into()),
        holds_what: Some("HAZE".into()),
        spawn_frames: Some(1),
        spawn_cutoff: dec("100.0"),
        spawn_translucency_cutoff: dec(cut),
        ..Default::default()
    };
    let torch = |held: &str| ParticleSystemType {
        behaves_like: Some("fire".into()),
        holds_what: Some(held.into()),
        ..Default::default()
    };
    let shower = |held: &str| ParticleSystemType {
        behaves_like: Some("spark".into()),
        holds_what: Some(held.into()),
        spark_spawn_frames: Some(1),
        particle_cap: Some(3),
        ..Default::default()
    };
    for (n, t) in [
        ("FOGA", fog("0.5")),
        ("FOGB", fog("1.0")),
        ("TORCH", torch("FLARE")),
        ("TORCH2", torch("FLARE2")),
        ("TORCH3", torch("FLARE3")),
        ("SHOWER", shower("GLINT")),
        ("SHOWER2", shower("GLINT2")),
    ] {
        register(&mut r, "particle_systems", n);
        r.sim.particle_systems.insert(n.into(), t);
    }
    r
}

fn game() -> World {
    testkit::game_with(&rules(), SIZE, 7)
}

fn centre() -> Pos {
    Pos::centre(testkit::mid(SIZE, 0, 0), 0)
}

fn system(w: &mut World, name: &str, p: Pos) -> ObjectId {
    gas::warhead_system(w, name, p, None);
    *w.gas.systems.keys().last().unwrap()
}

fn look(w: &World, name: &str) -> ParticleLook {
    let rules = w.rules();
    let i = rules
        .gas
        .particles
        .iter()
        .position(|t| t.name == name)
        .unwrap();
    rules.gas.particles[i].look.clone()
}

fn f(s: &str) -> F64 {
    nearest_double(s)
}

/// spec: S-0616 claim 26 (at creation: the type's translucency field,
/// kept modulo 256) and F15 (the creation position and the target point
/// given at creation are kept)
#[test]
fn claim26_f15_creation_state() {
    let mut w = game();
    let p = centre();
    let s = system(&mut w, "FOGB", p);
    let q = &w.gas.systems[&s].particles[0];
    assert_eq!(q.translucency, 25, "281 modulo 256");
    assert_eq!(q.origin, q.pos);
    assert_eq!(
        q.aim, p,
        "a warhead system's particle aims at the detonation point"
    );
}

/// spec: S-0616 claims 9 and 26, F4 (an emitted smoke particle's
/// translucency rises by 25 when T > τ, compared in single precision: with
/// T = 1.0, τ = 0.5 raises it and τ = 1.0 does not)
#[test]
fn claim9_emission_raises_translucency_above_the_cut_off() {
    for (name, want) in [("FOGA", 50u8), ("FOGB", 25u8)] {
        let mut w = game();
        let s = system(&mut w, name, centre());
        testkit::run(&mut w, 3);
        let list = &w.gas.systems[&s].particles;
        assert!(list.len() >= 2, "{name}: emissions happened");
        // The first particle is the warhead's own, made without emission.
        assert_eq!(list[0].translucency, 25, "{name}: not raised");
        for q in &list[1..] {
            assert_eq!(q.translucency, want, "{name}");
        }
    }
}

/// spec: S-0616 claims 8 and 26 (a successor takes the dead particle's
/// value, plus 25 when its raw value r has r rem 6 ≠ 0, kept modulo 256;
/// a chain steps 0, 25, 50 … and wraps)
#[test]
fn claim26_successor_translucency() {
    assert_eq!(particles::successor_translucency(0, 6), 0);
    assert_eq!(particles::successor_translucency(0, -12), 0);
    assert_eq!(particles::successor_translucency(0, 7), 25);
    assert_eq!(particles::successor_translucency(0, -5), 25);
    assert_eq!(particles::successor_translucency(125, 1), 150);
    assert_eq!(particles::successor_translucency(250, 1), 19);
}

/// The (stage, translucency) pairs of every particle of `name` seen over
/// `ticks` ticks.
fn fire_states(name: &str, ticks: i32) -> Vec<(i8, u8)> {
    let mut w = game();
    let s = system(&mut w, name, centre());
    let mut seen = Vec::new();
    for _ in 0..ticks {
        testkit::run(&mut w, 1);
        if let Some(x) = w.gas.systems.get(&s) {
            seen.extend(x.particles.iter().map(|q| (q.stage, q.translucency)));
        }
    }
    assert!(
        seen.iter().any(|(s, _)| *s >= 4),
        "{name}: a particle reached stage 4"
    );
    seen
}

/// spec: S-0616 claim 26 (fire stage fields: at an advance to the
/// translucent-25 stage the value becomes 25, at the translucent-50 stage
/// 50; set, not added; 50 wins when both name the stage; the absent 255
/// never takes effect)
#[test]
fn claim26_fire_stage_fields() {
    for (stage, t) in fire_states("TORCH", 20) {
        let want = match stage {
            0 | 1 => 0,
            2 | 3 => 25,
            _ => 50,
        };
        assert_eq!(t, want, "TORCH at stage {stage}");
    }
    for (stage, t) in fire_states("TORCH2", 20) {
        assert_eq!(
            t,
            if stage >= 2 { 50 } else { 0 },
            "TORCH2 at stage {stage}"
        );
    }
    for (stage, t) in fire_states("TORCH3", 20) {
        assert_eq!(t, 0, "TORCH3 at stage {stage}");
    }
}

/// spec: S-0616 F16 (own colour: black for an empty list; list entry 0
/// when both start colours are black, without the draw; otherwise
/// trunc(s₁ × (1 − f) + s₂ × f) with f = r × 4.656613 × 10⁻¹⁰ in single
/// precision; i = 0 and t = 0)
#[test]
fn f16_own_colour() {
    let list = vec![[10, 20, 30], [200, 100, 0]];
    let none = ParticleLook::default();
    assert_eq!(ColourState::new(&none, None, M).own, [0, 0, 0]);
    let black = ParticleLook {
        colours: list.clone(),
        ..Default::default()
    };
    let c = ColourState::new(&black, None, M);
    assert_eq!(
        (c.own, c.index, c.t),
        ([10, 20, 30], 0, F64::from_i64(0, M))
    );
    let mixed = ParticleLook {
        colours: list,
        start: [[0, 0, 0], [100, 200, 40]],
        ..Default::default()
    };
    assert_eq!(ColourState::new(&mixed, Some(0), M).own, [0, 0, 0]);
    // f = 1073741823 × 4.656613e-10 ≈ 0.5: halfway, truncated.
    assert_eq!(
        ColourState::new(&mixed, Some(1073741823), M).own,
        [50, 100, 20]
    );
    // f just under 1 truncates the top channel down.
    let top = ColourState::new(&mixed, Some(2147483646), M).own;
    assert!(top[0] >= 99 && top[1] >= 199 && top[2] >= 39, "{top:?}");
}

/// spec: S-0616 claim 29 and F16 (t ← (r × 4.656612877414201 × 10⁻¹⁰ ×
/// 0.05 + v) + t; past 1.0 the next pair with t = 0 while i < n − 2, else t
/// held at 1.0; the drawn colour mixes A and B; short lists move towards
/// A, the departure)
#[test]
fn claim29_f16_progress_and_drawn_colour() {
    let look = ParticleLook {
        colours: vec![[0, 0, 0], [200, 100, 0], [0, 0, 250]],
        speed: f("0.3"),
        ..Default::default()
    };
    let mut c = ColourState::new(&look, None, M);
    c.advance(&look, 0, M);
    assert_eq!(c.t, f("0.3"));
    assert_eq!(c.drawn(&look, M), [60, 30, 0], "0.3 of the way to entry 1");
    // The largest draw adds 0.05 at most.
    c.advance(&look, 2147483646, M);
    assert!(c.t.gt(f("0.649")) && c.t.lt(f("0.651")));
    c.advance(&look, 0, M);
    c.advance(&look, 0, M);
    assert_eq!(
        (c.index, c.t),
        (1, F64::from_i64(0, M)),
        "past 1.0: the next pair"
    );
    assert_eq!(c.drawn(&look, M), [200, 100, 0]);
    for _ in 0..4 {
        c.advance(&look, 0, M);
    }
    assert_eq!(
        (c.index, c.t),
        (1, F64::from_i64(1, M)),
        "the last pair holds at 1.0"
    );
    assert_eq!(c.drawn(&look, M), [0, 0, 250]);
    // One entry: its own colour throughout; no entry: black.
    let one = ParticleLook {
        colours: vec![[7, 8, 9]],
        speed: f("0.6"),
        ..Default::default()
    };
    let mut c = ColourState::new(&one, None, M);
    c.advance(&one, 0, M);
    c.advance(&one, 0, M);
    assert_eq!((c.index, c.drawn(&one, M)), (0, [7, 8, 9]));
    let empty = ParticleLook::default();
    assert_eq!(
        ColourState::new(&empty, None, M).drawn(&empty, M),
        [0, 0, 0]
    );
}

/// spec: S-0616 claims 16 and 29, S-0202 claim 15 (a spark's own colour
/// lies between its start colours when it takes the creation draw, list
/// entry 0 when both are black; each update's colour draw moves t by the
/// colour speed plus at most 0.05)
#[test]
fn claim16_29_spark_colour_in_play() {
    let mut w = game();
    let s = system(&mut w, "SHOWER", centre());
    testkit::run(&mut w, 1);
    let list = w.gas.systems[&s].particles.clone();
    assert!(!list.is_empty());
    let l = look(&w, "GLINT");
    for q in &list {
        assert_eq!(q.colour.own, [10, 20, 30]);
        let ok = q.colour.index == 0 && !q.colour.t.lt(l.speed) && !q.colour.t.gt(f("0.35"));
        assert!(ok, "one update: t in 0.3 … 0.35, {:?}", q.colour);
    }
    let mut w = game();
    let s = system(&mut w, "SHOWER2", centre());
    testkit::run(&mut w, 1);
    for q in &w.gas.systems[&s].particles {
        let [r, g, b] = q.colour.own;
        assert!(r <= 100 && g <= 200 && b <= 40, "{:?}", q.colour.own);
    }
}
