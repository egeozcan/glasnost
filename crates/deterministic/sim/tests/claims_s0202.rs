//! Claim tests for S-0202's draws during map loading on synthetic data:
//! low-bridge end markers and their pieces, the vein rebuild of S-0009
//! claim 14 and the visceroid placement draw.

mod common;

use common::{Scene, TRACK, cell, unit_type};
use glasnost_map::{CellXY, NO_OVERLAY};
use glasnost_rules::sim::OverlayType;
use glasnost_sim::rng::Rng;

/// A scene whose overlay list has 240 types O0 … O239, every one drawn.
fn overlay_scene() -> Scene {
    let mut s = Scene::new();
    for i in 0..240 {
        let name = format!("O{i}");
        s.register("overlays", &name);
        s.ruleset.sim.overlays.insert(
            name,
            OverlayType {
                has_art: Some(true),
                ..Default::default()
            },
        );
    }
    s
}

fn overlay(w: &glasnost_sim::world::World, c: CellXY) -> (u8, u8) {
    let cell = w.cells.cell(c).unwrap();
    (cell.overlay, cell.overlay_data)
}

/// spec: S-0202 claim 10 (low bridges: end rows, the partner found by the second marker, 3d pieces of type t₀ + (x mod 4), one raw value each, in step order from the partner's side)
#[test]
fn claim_10_low_bridge_pieces() {
    let mut s = overlay_scene();
    s.map.has_overlay_data = false;
    // Marker 123 (end row 94, looks east for 92) four cells west of marker 122.
    s.cell_mut(cell(-4, 0)).overlay = 123;
    s.cell_mut(cell(0, 0)).overlay = 122;
    let w = s.build();
    // The end rows: data 0, 1, 2 by increasing y.
    for (k, dy) in [-1, 0, 1].into_iter().enumerate() {
        assert_eq!(overlay(&w, cell(-4, dy)), (94, k as u8));
        assert_eq!(overlay(&w, cell(0, dy)), (92, k as u8));
    }
    // m = 4: three steps, x = −3, −2, −1 from the marker's side of the partner; nine raw values.
    let mut r = Rng::new(s.setup.seed);
    for dx in [-3, -2, -1] {
        for (k, dy) in [-1, 0, 1].into_iter().enumerate() {
            let x = r.raw();
            assert_eq!(overlay(&w, cell(dx, dy)), (74 + (x % 4) as u8, k as u8));
        }
    }
    // The load's last draw (S-0009 claim 20) follows the pieces.
    let _ = r.ranged(0, 65535, true);
    assert_eq!(w.rng, r);
}

/// spec: S-0202 claim 10 (m = 1: the marker's own end row becomes the pieces; an end row that meets an overlay writes nothing and takes no draw)
#[test]
fn claim_10_low_bridge_side_by_side_and_blocked() {
    let mut s = overlay_scene();
    s.map.has_overlay_data = false;
    s.cell_mut(cell(-1, 0)).overlay = 123;
    s.cell_mut(cell(0, 0)).overlay = 122;
    let w = s.build();
    let mut r = Rng::new(s.setup.seed);
    for (k, dy) in [-1, 0, 1].into_iter().enumerate() {
        let x = r.raw();
        assert_eq!(overlay(&w, cell(0, dy)), (74 + (x % 4) as u8, k as u8));
    }
    // A blocked end row: an overlay above the marker's cell.
    let mut s = overlay_scene();
    s.map.has_overlay_data = false;
    s.cell_mut(cell(0, -1)).overlay = 7;
    s.cell_mut(cell(0, 0)).overlay = 122;
    let w = s.build();
    assert_eq!(overlay(&w, cell(0, 0)).0, NO_OVERLAY);
    assert_eq!(overlay(&w, cell(0, 1)).0, NO_OVERLAY);
    assert_eq!(w.rng.taken(), 1, "only the load's last draw");
}

/// spec: S-0009 claim 14 and S-0202 claim 10 (veins are cleared; seeds with data 48 or more get a vein pattern: the centre 48 + (|σ(x)| rem 3), each flat empty edge neighbour 3c + |σ(x) rem 3| in the order north, east, south, west)
#[test]
fn claim_10_vein_rebuild() {
    let mut s = overlay_scene();
    for (c, d) in [(cell(-6, 0), 47), (cell(6, 0), 48)] {
        let cell = s.cell_mut(c);
        cell.overlay = 126;
        cell.overlay_data = d;
    }
    let w = s.build();
    assert_eq!(overlay(&w, cell(-6, 0)).0, NO_OVERLAY, "data 47 is no seed");
    let mut r = Rng::new(s.setup.seed);
    let x = r.raw() as i32;
    assert_eq!(
        overlay(&w, cell(6, 0)),
        (126, 48 + (x.unsigned_abs() % 3) as u8)
    );
    // Each neighbour's pattern has only the bit pointing back at the centre.
    for ((dx, dy), c) in [((0, -1), 4), ((1, 0), 8), ((0, 1), 1), ((-1, 0), 2)] {
        let x = r.raw() as i32;
        assert_eq!(
            overlay(&w, cell(6 + dx, dy)),
            (126, 3 * c + (x % 3).unsigned_abs() as u8)
        );
    }
}

/// spec: S-0202 claim 10, map-loading table (placing a vehicle of a type with either visceroid flag takes one ranged draw R(0, 29) after its creation draw)
#[test]
fn claim_10_visceroid_draw() {
    let mut s = Scene::new();
    let mut t = unit_type(TRACK, 4, 5);
    t.large_visceroid = Some(true);
    s.vehicle("BLOB", t);
    s.place_unit(0, "BLOB", cell(0, 0), 0);
    let w = s.build();
    let mut r = Rng::new(s.setup.seed);
    let _ = r.raw();
    let _ = r.ranged(0, 29, true);
    let _ = r.ranged(0, 65535, true);
    assert_eq!(w.rng, r);
}

/// spec: S-0010 claim 19 and S-0202 claim 10 (a tile animation starts at its cell's first recalculation before the second pass and again in the second pass, with its creation draws: the stage-delay draw of S-0109 claim 3 when d₁′ ≠ d₂′, a bouncing type's three raw values and three ranged draws over −65535 … 65535; afterwards at most one per cell)
#[test]
fn claim_10_tile_animation_draws() {
    use glasnost_map::terrain::TileAnim;
    use glasnost_rules::sim::AnimationType;
    let mut s = Scene::new();
    // Tile 0 (every cell's flat tile) keeps no animation; tile 1 names SPARK on sub-tile 0, tile 2 ROCK.
    s.catalogue.tiles[1].anim = Some(TileAnim {
        name: "spark".into(),
        attaches_to: 0,
    });
    s.catalogue.tiles[2].anim = Some(TileAnim {
        name: "ROCK".into(),
        attaches_to: 0,
    });
    s.register("animations", "SPARK");
    s.register("animations", "ROCK");
    // A descending pair: ρ = 2 and 1 give d = 450 and 900, a random delay.
    s.ruleset.sim.animations.insert(
        "SPARK".into(),
        AnimationType {
            random_rate: Some([2, 1]),
            ..Default::default()
        },
    );
    s.ruleset.sim.animations.insert(
        "ROCK".into(),
        AnimationType {
            bouncer: Some(true),
            ..Default::default()
        },
    );
    s.set_cell(cell(0, 0), 1, 0);
    let mut w = s.build();
    let mut r = Rng::new(s.setup.seed);
    // The first pass, then the second pass.
    let _ = r.ranged(450, 900, true);
    let _ = r.ranged(450, 900, true);
    let _ = r.ranged(0, 65535, true);
    assert_eq!(w.rng, r);
    // In play the cell has started its animation: no second one.
    glasnost_sim::load::recalc_cells(&mut w, &[cell(0, 0)]);
    assert_eq!(w.rng, r);
    // A bouncing type.
    s.set_cell(cell(0, 0), 2, 0);
    let w = s.build();
    let mut r = Rng::new(s.setup.seed);
    for _ in 0..2 {
        for _ in 0..3 {
            let _ = r.raw();
        }
        for _ in 0..3 {
            let _ = r.ranged(-65535, 65535, true);
        }
    }
    let _ = r.ranged(0, 65535, true);
    assert_eq!(w.rng, r);
    // An ascending pair collapses to a constant delay: no draw.
    s.ruleset
        .sim
        .animations
        .get_mut("SPARK")
        .unwrap()
        .random_rate = Some([1, 2]);
    s.set_cell(cell(0, 0), 1, 0);
    assert_eq!(s.build().rng.taken(), 1);
}
