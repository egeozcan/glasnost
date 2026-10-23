//! Tests of how the live game draws debris pieces (S-0317 claim 15, S-0007
//! claim 30; the drawing interim of Q-1835) on invented art: the
//! synthetic voxel model and the simulation's synthetic combat ruleset
//! with one invented piece type.

use glasnost_convert::art::TypeArt;
use glasnost_convert::kinds::Kind;
use glasnost_render::palette::{PaletteQuirks, PaletteState};
use glasnost_render::scene::{Group, Item, Scene};
use glasnost_rules::sim::VoxelAnimationType;
use glasnost_sim::testkit;
use glasnost_sim::world::Pos;

use super::build::{Assets, Inputs, Options, PIECE_STEP, PieceDraw, build_parts};
use super::live::{Entrant, Live, piece_pose};
use super::tests::{RULES, context_with};

/// The test context with the voxel animation `Chunk` drawn by the synthetic
/// model `chunk.vxl` (no motion file), and its registration index.
fn chunk_context() -> (
    glasnost_convert::Import,
    glasnost_convert::theater::Catalogue,
    Assets,
    usize,
) {
    use glasnost_fmt::vxl::{VxlWriter, synthetic_model};
    let (mut import, catalogue, mut assets) = context_with(&format!(
        "{RULES}[VoxelAnims]\n1=Chunk\n[Chunk]\nDuration=12\n"
    ));
    let idx = import.outcome.registries[&Kind::VoxelAnimations]
        .iter()
        .position(|t| t.name.eq_ignore_ascii_case(b"Chunk"))
        .expect("Chunk registered");
    let mut art = TypeArt {
        image: b"CHUNK".to_vec(),
        ..Default::default()
    };
    art.voxels.insert("body".into(), b"chunk.vxl".to_vec());
    import.arts.insert((Kind::VoxelAnimations, idx), art);
    assets.insert(
        b"chunk.vxl",
        VxlWriter {
            model: &synthetic_model(),
        }
        .write(),
    );
    (import, catalogue, assets, idx)
}

/// Draws `list` with the painter of a fresh scene of the context.
fn draw(
    import: &glasnost_convert::Import,
    catalogue: &glasnost_convert::theater::Catalogue,
    assets: &Assets,
    list: &[PieceDraw],
) -> (Scene, Vec<Vec<Item>>) {
    let map = testkit::empty_map(24);
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = Inputs {
        map: &map,
        import,
        catalogue,
        palettes: &palettes,
        assets,
        options: &options,
        rules: None,
        cells: None,
    };
    let (built, mut painter) = build_parts(&inputs, true);
    let mut scene = built.scene;
    let (items, _) = painter.pieces(&inputs, &mut scene, list);
    (scene, items)
}

fn piece(idx: usize, pose: (u8, u8)) -> PieceDraw {
    PieceDraw {
        idx,
        pos: (2000, 2200, 300),
        pose,
        owner: Vec::new(),
    }
}

/// spec: S-0317 claim 15 (the spin turns the drawn model each tick about the
/// spin axis), in our interim of Q-1835: the axis of the largest component,
/// the angle in whole 32-step facing steps modulo 32
#[test]
fn piece_pose_follows_the_spin() {
    // A zero axis: no turn.
    assert_eq!(piece_pose([0, 0, 0], 0.5, 10), (3, 0));
    // The largest component chooses the model axis; the first of equal ones.
    assert_eq!(piece_pose([5, -9, 2], 0.0, 4).0, 1);
    assert_eq!(piece_pose([7, -7, 0], 0.0, 4).0, 0);
    // One facing step per tick, then wrapped modulo 32.
    assert_eq!(piece_pose([0, 0, 3], PIECE_STEP, 5), (2, 5));
    assert_eq!(piece_pose([0, 0, 3], PIECE_STEP, 33), (2, 1));
    // A negative component turns the other way.
    assert_eq!(piece_pose([0, 0, -3], PIECE_STEP, 1), (2, 31));
    // Nothing before the first tick.
    assert_eq!(piece_pose([1, 0, 0], 1.0, 0), (0, 0));
}

/// spec: S-0007 claim 30 (the own model, drawn without a motion file) and
/// S-0317 claim 15 (its pose turns the drawn model); our interim of Q-1835
/// for the rest: one effects item, no depth test, a turned pose a different
/// image, the same pose the same image
#[test]
fn a_piece_draws_its_model() {
    let (import, catalogue, assets, idx) = chunk_context();
    let list = [
        piece(idx, (3, 0)),
        piece(idx, (2, 8)),
        piece(idx, (3, 0)),
        piece(idx + 99, (3, 0)),
    ];
    let (_, items) = draw(&import, &catalogue, &assets, &list);
    assert_eq!(items[0].len(), 1);
    assert_eq!(items[0][0].group, Group::Effects);
    assert_eq!(
        items[0][0].test,
        glasnost_render::compose::DepthTest::Ignore
    );
    assert_ne!(items[0][0].image, items[1][0].image, "a turned pose");
    assert_eq!(items[0][0].image, items[2][0].image, "the same pose, kept");
    // A type without art draws nothing.
    assert!(items[3].is_empty());
}

/// spec: S-0102 claims 3, 4 and 7 (a voxel animation is an entrant of the
/// air layer from when the drawing first sees it until it leaves play) and
/// S-0317 claim 6 (it leaves play when its lifetime runs out)
#[test]
fn live_pieces_are_drawn_in_the_air_layer_until_they_end() {
    let (import, catalogue, assets, _) = chunk_context();
    let map = testkit::empty_map(24);
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: None,
        cells: None,
    };
    let (mut built, painter) = build_parts(&inputs, true);
    let mut r = testkit::combat_ruleset();
    r.registries
        .entry("voxel_animations".into())
        .or_default()
        .push("Chunk".into());
    r.sim.voxel_animations.insert(
        "Chunk".into(),
        VoxelAnimationType {
            duration: Some(12),
            ..Default::default()
        },
    );
    let mut world = testkit::game_with(&r, 24, 3);
    let m = testkit::mid(24, 0, 0);
    glasnost_sim::pieces::create_named(&mut world, "Chunk", Pos::new(m.x, m.y, 400), None);
    let id = *world.pieces.keys().next().expect("a piece is made");
    let rules = world.rules_arc();
    let mut live = Live::new(
        &inputs,
        &mut built,
        painter,
        (world, rules, testkit::setup(3), Vec::new()),
    );
    let mut scene = built.scene;
    live.step();
    live.redraw(&inputs, &mut scene);
    let air = live.layer_list(super::slots::Layer::Air);
    assert!(air.contains(&Entrant::Piece(id)), "in the air layer");
    assert_eq!(live.piece_counts(), (1, 1), "drawn");
    for _ in 0..20 {
        live.step();
        live.redraw(&inputs, &mut scene);
    }
    assert!(live.world.pieces.is_empty(), "its lifetime ran out");
    let air = live.layer_list(super::slots::Layer::Air);
    assert!(!air.contains(&Entrant::Piece(id)), "gone from the layer");
    assert_eq!(live.piece_counts(), (0, 0));
}
