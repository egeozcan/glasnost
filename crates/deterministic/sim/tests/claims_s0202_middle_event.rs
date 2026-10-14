//! Claim tests for an animation's immediate middle event at creation
//! (S-0109 claims 13 and 14, S-0202 claim 15) on synthetic data: the
//! particle groups, the height test, the scorch-or-crater choice and the
//! crater's resource removal, and the tile animation's position (S-0010
//! Formulas).

mod harvest_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{AnimationType, ParticleType};
use glasnost_sim::load::{anim_created_at, tile_anim_position};
use glasnost_sim::rng::Rng;
use glasnost_sim::world::World;
use harvest_common::*;

/// Adds an animation type to the rules.
fn anim(r: &mut Ruleset, name: &str, a: AnimationType) {
    r.registries
        .entry("animations".into())
        .or_default()
        .push(name.into());
    r.sim.animations.insert(name.into(), a);
}

/// Adds a particle type to the rules.
fn particle(r: &mut Ruleset, name: &str, p: ParticleType) {
    r.registries
        .entry("particles".into())
        .or_default()
        .push(name.into());
    r.sim.particles.insert(name.into(), p);
}

/// The ground position of the centre of `c` at height `z`.
fn centre(c: CellXY, z: i32) -> (i32, i32, i32) {
    (256 * c.x + 128, 256 * c.y + 128, z)
}

/// The draws an animation's creation takes, from a fresh game on `r`.
fn draws(r: &Ruleset, name: &str, z: i32) -> (World, Rng) {
    let mut w = game(r);
    let before = w.rng.clone();
    anim_created_at(&mut w, name, Some(centre(at(0, 0), z)));
    (w, before)
}

/// A middle-event animation type with μ = 0 that requests `n` particles of
/// type `p`.
fn puff(p: &str, n: i32) -> AnimationType {
    AnimationType {
        middle_frame: Some(0),
        spawns_particle: Some(p.into()),
        num_particles: Some(n),
        ..Default::default()
    }
}

/// spec: S-0202 claim 15 (each particle: a smoke particle's starting-speed raw value, then the lifetime raw value, then R(0, 2147483646) when the colour list is nonempty and a starting colour is not black; one particle's group before the next)
#[test]
fn claim_15_particle_groups() {
    let mut r = ruleset();
    let lit = Some([0, 0, 1]);
    particle(
        &mut r,
        "SMOKY",
        ParticleType {
            behaves_like: Some("smoke".into()),
            color_count: Some(2),
            start_color2: lit,
            ..Default::default()
        },
    );
    particle(
        &mut r,
        "DARK",
        ParticleType {
            behaves_like: Some("smoke".into()),
            color_count: Some(2),
            start_color1: Some([0, 0, 0]),
            ..Default::default()
        },
    );
    particle(
        &mut r,
        "NOLIST",
        ParticleType {
            behaves_like: Some("gas".into()),
            start_color1: lit,
            ..Default::default()
        },
    );
    particle(
        &mut r,
        "RAIL",
        ParticleType {
            behaves_like: Some("railgun".into()),
            max_ec: Some(0),
            ..Default::default()
        },
    );
    anim(&mut r, "A1", puff("SMOKY", 2));
    anim(&mut r, "A2", puff("DARK", 1));
    anim(&mut r, "A3", puff("NOLIST", 3));
    anim(&mut r, "A4", puff("RAIL", 1));
    anim(&mut r, "A5", puff("SMOKY", 0));
    anim(&mut r, "A6", puff("SMOKY", -2));
    // μ ≠ 0: no middle event at the start event.
    anim(
        &mut r,
        "A7",
        AnimationType {
            middle_frame: Some(3),
            ..puff("SMOKY", 2)
        },
    );

    let (w, mut e) = draws(&r, "A1", 0);
    for _ in 0..2 {
        let _ = e.raw();
        let _ = e.raw();
        let _ = e.ranged(0, 2147483646, true);
    }
    assert_eq!(w.rng, e);

    // Both starting colours black: no colour draw.
    let (w, mut e) = draws(&r, "A2", 0);
    let _ = e.raw();
    let _ = e.raw();
    assert_eq!(w.rng, e);

    // An empty colour list: no colour draw, even with a nonblack starting colour.
    let (w, mut e) = draws(&r, "A3", 0);
    for _ in 0..3 {
        let _ = e.raw();
    }
    assert_eq!(w.rng, e);

    // Railgun: the lifetime raw value only, whatever the lifetime field.
    let (w, mut e) = draws(&r, "A4", 0);
    let _ = e.raw();
    assert_eq!(w.rng, e);

    // A count of at most zero, or μ ≠ 0: no particle group.
    for name in ["A5", "A6", "A7"] {
        let (w, e) = draws(&r, name, 0);
        assert_eq!(w.rng, e, "{name}");
    }
}

/// spec: S-0109 claim 25 (μ comes from the sprite: an import that found no art leaves it unknown, and our engine takes none of the middle event's draws then, with a note)
#[test]
fn claim_25_unknown_mu_takes_no_middle_event_draw() {
    let mut r = ruleset();
    particle(&mut r, "P", ParticleType::default());
    anim(
        &mut r,
        "A",
        AnimationType {
            middle_frame: None,
            ..puff("P", 2)
        },
    );
    let (w, e) = draws(&r, "A", 0);
    assert_eq!(w.rng, e);
    assert!(w.notes.iter().any(|n| n.starts_with("S-0109 claim 25")));
}

/// A scorch-and-crater type with μ = 0 and one plain particle.
fn both(r: &mut Ruleset) {
    particle(r, "P", ParticleType::default());
    anim(
        r,
        "BOTH",
        AnimationType {
            scorch: Some(true),
            crater: Some(true),
            ..puff("P", 1)
        },
    );
}

/// spec: S-0109 claim 14 (the height test: Z − H(X, Y) < 30, no lower bound; failing it, no choice draw; the particle requests are not gated by height and precede the choice)
#[test]
fn claim_14_height_test_gates_the_choice() {
    let mut r = ruleset();
    both(&mut r);
    for (z, choice) in [
        (-500, true),
        (0, true),
        (29, true),
        (30, false),
        (200, false),
    ] {
        let (w, mut e) = draws(&r, "BOTH", z);
        let _ = e.raw();
        if choice {
            let _ = e.ranged(0, 2147483646, true);
        }
        assert_eq!(w.rng, e, "z = {z}");
    }
}

/// spec: S-0109 claim 14 and S-0401 claim 16 (c) (both flags: R(0, 2147483646) after the particles, a scorch mark below 1073741824, otherwise a crater, which takes 6 bales from the containing cell; a scorch mark takes none)
#[test]
fn claim_14_choice_and_crater_removal() {
    let mut r = ruleset();
    both(&mut r);
    let mut seen = [false; 2];
    for seed in 1..40 {
        let mut w = game_with_ore(&r, &[(at(0, 0), 0, 8)], seed);
        let mut e = w.rng.clone();
        anim_created_at(&mut w, "BOTH", Some(centre(at(0, 0), 0)));
        let _ = e.raw();
        let crater = e.ranged(0, 2147483646, true) >= 1073741824;
        assert_eq!(w.rng, e);
        assert_eq!(stage(&w, at(0, 0)), Some(if crater { 2 } else { 8 }));
        seen[usize::from(crater)] = true;
    }
    assert_eq!(seen, [true, true]);
}

/// spec: S-0109 claim 14 and S-0401 claims 3 and 16 (c) (a crater-only type takes no choice draw; its 6 bales come from the cell containing the position, a cell of 6 bales or fewer being emptied)
#[test]
fn claim_14_crater_only_takes_six_bales() {
    let mut r = ruleset();
    anim(
        &mut r,
        "PIT",
        AnimationType {
            middle_frame: Some(0),
            crater: Some(true),
            ..Default::default()
        },
    );
    let mut w = game_with_ore(&r, &[(at(0, 0), 0, 8), (at(1, 0), 0, 5)], 3);
    let before = w.rng.clone();
    // A position inside cell (1, 0), near its corner with cell (0, 0).
    let (x, y, _) = centre(at(1, 0), 0);
    anim_created_at(&mut w, "PIT", Some((x - 127, y - 127, 0)));
    assert_eq!(w.rng, before);
    assert_eq!(stage(&w, at(1, 0)), None);
    assert_eq!(stage(&w, at(0, 0)), Some(8));
    anim_created_at(&mut w, "PIT", Some(centre(at(0, 0), 0)));
    assert_eq!(stage(&w, at(0, 0)), Some(2));
}

/// spec: S-0202 claim 15 (the crater's 6 bales under S-0401 claim 3: stages 0 and 5 are emptied, stage 6 becomes stage 0 and keeps its resource, stage 7 becomes stage 1)
#[test]
fn claim_15_crater_take_by_stage() {
    let mut r = ruleset();
    anim(
        &mut r,
        "PIT",
        AnimationType {
            middle_frame: Some(0),
            crater: Some(true),
            ..Default::default()
        },
    );
    let cells = [(at(0, 0), 0), (at(2, 0), 5), (at(4, 0), 6), (at(6, 0), 7)];
    let ore: Vec<_> = cells.iter().map(|&(c, d)| (c, 0, d)).collect();
    let mut w = game_with_ore(&r, &ore, 3);
    for &(c, _) in &cells {
        anim_created_at(&mut w, "PIT", Some(centre(c, 0)));
    }
    let after: Vec<Option<u8>> = cells.iter().map(|&(c, _)| stage(&w, c)).collect();
    assert_eq!(after, vec![None, None, Some(0), Some(1)]);
}

/// spec: S-0010 Formulas (tile animation position: X = 256x + 128 + trunc₀(a(dx + 2dy)), Y = 256y + 128 + trunc₀(a(2dy − dx)), z = 104ℓ; a = 4473959 / 1048576, S-0101 F6)
#[test]
fn tile_animation_position() {
    let c = CellXY::new(3, 5);
    assert_eq!(tile_anim_position(c, 0, 0, 0), (896, 1408, 0));
    // dx = 10: a · 10 = 42.67, so +42 and −42.
    assert_eq!(tile_anim_position(c, 2, 10, 0), (938, 1366, 208));
    // dy = 15: a · 30 = 128.0001, so +128 on both axes.
    assert_eq!(tile_anim_position(c, -1, 0, 15), (1024, 1536, -104));
    // dx = −3, dy = 1: a · −1 = −4.27 and a · 5 = 21.33.
    assert_eq!(tile_anim_position(c, 0, -3, 1), (892, 1429, 0));
}

/// spec: S-0202 claim 15 (a crater from a tile animation that an overlay placement's recalculation starts acts before the overlay data stream: it empties the resource then present, and the data stream writes the cell's data without restoring the removed overlay type) and S-0010 claim 19
#[test]
fn claim_15_crater_before_the_data_stream() {
    use glasnost_map::NO_OVERLAY;
    use glasnost_map::terrain::TileAnim;
    use glasnost_sim::testkit;
    let mut r = ruleset();
    anim(
        &mut r,
        "PIT",
        AnimationType {
            middle_frame: Some(0),
            crater: Some(true),
            ..Default::default()
        },
    );
    let mut cat = (*testkit::catalogue()).clone();
    cat.tiles[1].anim = Some(TileAnim {
        name: "PIT".into(),
        attaches_to: 0,
        ..TileAnim::default()
    });
    let mut map = testkit::empty_map(SIZE);
    for (c, tile) in [(at(0, 0), 1), (at(3, 0), 0)] {
        let cell = map.cells.get_mut(c).unwrap();
        cell.tile = tile;
        cell.overlay = 0;
        cell.overlay_data = 8;
    }
    assert!(map.has_overlay_data);
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-test"));
    let w = glasnost_sim::load::new_game(rules, &map, std::sync::Arc::new(cat), &testkit::setup(5))
        .unwrap();
    let pit = w.cells.cell(at(0, 0)).unwrap();
    assert_eq!((pit.overlay, pit.overlay_data), (NO_OVERLAY, 8));
    assert_eq!(stage(&w, at(3, 0)), Some(8));
}

/// spec: S-0202 claim 15 (an animation created during play with no loop delay and μ = 0 takes the same middle-event draws, in the same order, as one created during loading; its creator does not change that rule)
#[test]
fn in_play_creation_takes_the_loading_composition() {
    let mut r = ruleset();
    particle(
        &mut r,
        "P",
        ParticleType {
            behaves_like: Some("smoke".into()),
            ..Default::default()
        },
    );
    anim(&mut r, "A", puff("P", 2));
    let (loaded, before) = draws(&r, "A", 0);
    let mut expect = before.clone();
    for _ in 0..4 {
        let _ = expect.raw();
    }
    assert_eq!(loaded.rng, expect);
    // The same type made by a weapon's or a death's animation in play.
    let mut w = game(&r);
    let start = w.rng.clone();
    let (x, y, z) = centre(at(0, 0), 0);
    glasnost_sim::combat::effects::anim(&mut w, "A", glasnost_sim::world::Pos::new(x, y, z));
    let mut expect = start;
    for _ in 0..4 {
        let _ = expect.raw();
    }
    assert_eq!(w.rng, expect);
    assert!(!w.notes.iter().any(|n| n.contains("Q-0217")));
}
