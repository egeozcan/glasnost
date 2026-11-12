//! Particle counts that come from rules data have a ceiling (#198): a
//! smoke next-type loop, a spark system's particle cap and a railgun
//! system's particles per lepton. Synthetic data only.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, ParticleSystemType, ParticleType};
use glasnost_sim::gas;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Pos, World};

const SIZE: i32 = 20;

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn particle(kind: &str) -> ParticleType {
    ParticleType {
        behaves_like: Some(kind.into()),
        max_ec: Some(4),
        end_state_ai: Some(100),
        ..Default::default()
    }
}

fn smoke(next: Option<&str>) -> ParticleType {
    ParticleType {
        radius: Some(40),
        next_particle: next.map(Into::into),
        ..particle("smoke")
    }
}

fn system(kind: &str, holds: &str) -> ParticleSystemType {
    ParticleSystemType {
        behaves_like: Some(kind.into()),
        holds_what: Some(holds.into()),
        ..Default::default()
    }
}

/// Smoke particle types by name and next type, a smoke system type per
/// type (`SYS` + the name), a spark system BURST of particle cap `cap` and
/// a railgun system LINE of `per` particles per lepton.
fn rules(smokes: &[(&str, Option<&str>)], cap: i32, per: &str) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    for &(n, next) in smokes {
        register(&mut r, "particles", n);
        r.sim.particles.insert(n.into(), smoke(next));
        let s = format!("SYS{n}");
        register(&mut r, "particle_systems", &s);
        r.sim.particle_systems.insert(
            s,
            ParticleSystemType {
                spawn_frames: Some(1_000_000),
                particle_cap: Some(4),
                ..system("smoke", n)
            },
        );
    }
    register(&mut r, "particles", "FLICK");
    r.sim.particles.insert(
        "FLICK".into(),
        ParticleType {
            x_velocity: Some(5),
            y_velocity: Some(5),
            min_z_velocity: Some(3),
            z_velocity_range: Some(4),
            ..particle("spark")
        },
    );
    register(&mut r, "particles", "BOLT");
    r.sim.particles.insert("BOLT".into(), particle("railgun"));
    register(&mut r, "particle_systems", "BURST");
    r.sim.particle_systems.insert(
        "BURST".into(),
        ParticleSystemType {
            spark_spawn_frames: Some(1),
            particle_cap: Some(cap),
            ..system("spark", "FLICK")
        },
    );
    register(&mut r, "particle_systems", "LINE");
    r.sim.particle_systems.insert(
        "LINE".into(),
        ParticleSystemType {
            particles_per_coord: Some(Decimal(per.into())),
            ..system("railgun", "BOLT")
        },
    );
    r
}

fn make(w: &mut World, name: &str, p: Pos) -> ObjectId {
    gas::warhead_system(w, name, p, None);
    *w.gas.systems.keys().last().unwrap()
}

fn centre() -> Pos {
    Pos::centre(testkit::mid(SIZE, 0, 0), 0)
}

/// The particle count of the system of smoke type `first` after each of 80
/// turns.
fn smoke_counts(smokes: &[(&str, Option<&str>)], first: &str) -> Vec<usize> {
    let mut r = rules(smokes, 4, "1");
    r.globals.quirks.smoke_ignores_particle_cap = false;
    let mut w = testkit::game_with(&r, SIZE, 7);
    let s = make(&mut w, &format!("SYS{first}"), centre());
    (0..80)
        .map(|t| {
            w.tick = 1 + t;
            gas::system_turn(&mut w, s);
            w.gas.systems.get(&s).map_or(0, |x| x.particles.len())
        })
        .collect()
}

// spec: S-0616 claim 8 (a dead particle whose type names a next type splits into two successors): a chain without a loop doubles once per type
#[test]
fn a_smoke_chain_without_a_loop_is_unchanged() {
    let counts = smoke_counts(&[("A", Some("B")), ("B", None)], "A");
    assert_eq!(counts.iter().max(), Some(&2), "{counts:?}");
}

// nocite: our own input validation (#198): a successor made through a next-type link that closes a loop of types splits no more through such a link
#[test]
fn a_smoke_type_that_names_itself_splits_once() {
    let counts = smoke_counts(&[("LOOP", Some("LOOP"))], "LOOP");
    assert!(counts.iter().all(|&n| n <= 2), "{counts:?}");
    assert_eq!(counts.last(), Some(&0), "the system empties: {counts:?}");
}

// nocite: our own input validation (#198): the same for a loop of two types
#[test]
fn a_smoke_loop_of_two_splits_once() {
    let counts = smoke_counts(&[("X", Some("Y")), ("Y", Some("X"))], "X");
    assert!(counts.iter().all(|&n| n <= 2), "{counts:?}");
}

/// The particles after the first turn of a spark system of particle cap
/// `cap`, and the stream's state after it.
fn burst(cap: i32) -> (usize, glasnost_sim::rng::Rng) {
    let mut w = testkit::game_with(&rules(&[], cap, "1"), SIZE, 7);
    let p = centre();
    let s = make(&mut w, "BURST", Pos::new(p.x, p.y, 500));
    gas::system_turn(&mut w, s);
    (w.gas.systems[&s].particles.len(), w.rng.clone())
}

// spec: S-0616 F10 (c = (|r₀| rem h) + h, h = trunc(κ / 2)): an ordinary cap is unchanged
#[test]
fn an_ordinary_spark_cap_is_unchanged() {
    let (n, _) = burst(40);
    assert!((20..40).contains(&n), "{n}");
}

// nocite: our own input validation (#198): a spark burst makes at most the engine's ceiling of sparks, with their draws, whatever the cap
#[test]
fn a_huge_spark_cap_meets_the_ceiling() {
    let (n, _) = burst(2_000_000_000);
    assert!(n <= 4096, "{n} sparks in one burst");
    assert!(n > 0);
}

/// The particles after a railgun line of `per` particles per lepton over
/// four cells.
fn line(per: &str) -> usize {
    let mut w = testkit::game_with(&rules(&[], 4, per), SIZE, 7);
    let p = centre();
    let s = make(&mut w, "LINE", p);
    {
        let sys = w.gas.systems.get_mut(&s).unwrap();
        sys.particles.clear();
        sys.target = Pos::new(p.x + 1024, p.y, p.z);
    }
    gas::system_turn(&mut w, s);
    w.gas.systems[&s].particles.len()
}

// spec: S-0616 F12 (n = trunc(L × p)): an ordinary line is unchanged
#[test]
fn an_ordinary_railgun_line_is_unchanged() {
    assert_eq!(line("0.5"), 512);
}

// nocite: our own input validation (#198): a railgun line makes at most the engine's ceiling of particles, with their draws, whatever the field
#[test]
fn a_huge_railgun_line_meets_the_ceiling() {
    let n = line("100");
    assert!(n <= 4096, "{n} particles in one line");
    assert!(n > 0);
}
