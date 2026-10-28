//! Tests of how the live game draws debris pieces (S-0317 claims 15 and 22
//! to 28, S-0007 claim 30) on invented art: the synthetic voxel model and
//! the simulation's synthetic combat ruleset with invented piece types.

use glasnost_convert::art::TypeArt;
use glasnost_convert::kinds::Kind;
use glasnost_render::compose::{DepthTest, Paint, Profile, Translucency};
use glasnost_render::palette::{PaletteQuirks, PaletteState};
use glasnost_render::projection::{height_lift, project_lifted};
use glasnost_render::scene::{Group, Item, ItemDepth, Scene};
use glasnost_render::voxel::{self, Mat34};
use glasnost_rules::sim::VoxelAnimationType;
use glasnost_sim::testkit;
use glasnost_sim::world::Pos;

use super::build::{Assets, Inputs, Options, Painter, PieceDraw, build_parts};
use super::live::{Entrant, Live};
use super::tests::{RULES, context_with};

/// The test context with the voxel animations `Chunk` (section 0), `Orb`
/// (section 1, translucent), `Gone` (section 2, outside the model) and
/// `Ore` (the resource flag), each drawn by the synthetic model `chunk.vxl`
/// (two sections, no motion file), and `Chunk`'s registration index.
fn chunk_context() -> (
    glasnost_convert::Import,
    glasnost_convert::theater::Catalogue,
    Assets,
    usize,
) {
    use glasnost_fmt::vxl::{VxlWriter, synthetic_model};
    let (mut import, catalogue, mut assets) = context_with(&format!(
        "{RULES}[VoxelAnims]\n1=Chunk\n2=Orb\n3=Gone\n4=Ore\n[Chunk]\nDuration=12\n\
         [Orb]\nVoxelIndex=1\nTranslucent=yes\n[Gone]\nVoxelIndex=2\n[Ore]\nIsTiberium=yes\n"
    ));
    let at = |n: &[u8]| {
        import.outcome.registries[&Kind::VoxelAnimations]
            .iter()
            .position(|t| t.name.eq_ignore_ascii_case(n))
            .expect("registered")
    };
    let idx = at(b"Chunk");
    for i in [idx, at(b"Orb"), at(b"Gone"), at(b"Ore")] {
        let mut art = TypeArt {
            image: b"CHUNK".to_vec(),
            ..Default::default()
        };
        art.voxels.insert("body".into(), b"chunk.vxl".to_vec());
        import.arts.insert((Kind::VoxelAnimations, i), art);
    }
    assets.insert(
        b"chunk.vxl",
        VxlWriter {
            model: &synthetic_model(),
        }
        .write(),
    );
    (import, catalogue, assets, idx)
}

/// Draws each frame of `frames` with one painter over one scene of the
/// context, as the live game does.
fn draw_frames(
    import: &glasnost_convert::Import,
    catalogue: &glasnost_convert::theater::Catalogue,
    assets: &Assets,
    frames: &[Vec<PieceDraw>],
) -> (Scene, Vec<Vec<Vec<Item>>>, Painter) {
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
    let mut out = Vec::new();
    for list in frames {
        let (items, _) = painter.pieces(&inputs, &mut scene, list);
        out.push(items);
    }
    (scene, out, painter)
}

fn draw(
    import: &glasnost_convert::Import,
    catalogue: &glasnost_convert::theater::Catalogue,
    assets: &Assets,
    list: &[PieceDraw],
) -> (Scene, Vec<Vec<Item>>) {
    let (scene, mut frames, _) = draw_frames(import, catalogue, assets, &[list.to_vec()]);
    (scene, frames.remove(0))
}

fn piece(id: u32, idx: usize, turns: i32) -> PieceDraw {
    PieceDraw {
        id,
        idx,
        pos: (2000, 2200, 300),
        axis: [3, -2, 5],
        spin: 0.25f64.to_bits(),
        turns,
        owner: Vec::new(),
        ground: 40,
        bridge: false,
        on_map: true,
    }
}

fn bias(i: &Item) -> (DepthTest, Profile, i32) {
    match i.depth {
        ItemDepth::Profile { profile, bias, .. } => (i.test, profile, bias),
        _ => panic!("a profile item"),
    }
}

/// spec: S-0317 claim 22 and F10 (the orientation is the identity at creation and the right-handed turn about the normalised axis by the spin rate times the net turn count; a zero axis has no direction and draws without spin, F5)
#[test]
fn piece_rotation_turns_about_the_axis() {
    let id = |m: &Mat34| m.0.map(|r| [r[0], r[1], r[2]]);
    let near = |a: [f32; 3], b: [f32; 3]| a.iter().zip(&b).all(|(x, y)| (x - y).abs() < 1e-6);
    assert!(voxel::piece_rotation([0, 0, 0], 1.0).is_none());
    let r0 = voxel::piece_rotation([3, -2, 5], 0.0).unwrap();
    assert_eq!(id(&r0), id(&Mat34::IDENTITY), "the identity at creation");
    // A quarter turn about +z maps +x to +y (right-handed); the axis's length does not matter.
    let r = voxel::piece_rotation([0, 0, 7], std::f64::consts::FRAC_PI_2).unwrap();
    assert!(near(r.apply([1.0, 0.0, 0.0]), [0.0, 1.0, 0.0]));
    // The axis stays fixed.
    let r = voxel::piece_rotation([3, -2, 5], 0.9).unwrap();
    assert!(near(r.apply([3.0, -2.0, 5.0]), [3.0, -2.0, 5.0]));
    // The reversed sense undoes the turn.
    let back = voxel::piece_rotation([3, -2, 5], -0.9).unwrap();
    assert!(near(back.apply(r.apply([1.0, 2.0, 3.0])), [1.0, 2.0, 3.0]));
}

/// spec: S-0007 claim 30 (the own model, drawn without a motion file) and S-0317 claims 22 (a new pose is drawn afresh), 26 (the body reads depth with the flat profile, bias −2 − L(z), around the lifted draw point) and 27 (its shadow first, the ordinary shadow treatment, flat profile, bias −4 − L(f), at L(f))
#[test]
fn a_piece_draws_its_shadow_and_model() {
    let (import, catalogue, assets, idx) = chunk_context();
    let list = [piece(1, idx, 0), piece(2, idx, 8), piece(3, idx + 99, 0)];
    let (scene, items) = draw(&import, &catalogue, &assets, &list);
    assert_eq!(items[0].len(), 2);
    let (shadow, body) = (&items[0][0], &items[0][1]);
    assert_eq!(
        (shadow.paint, shadow.group),
        (Paint::Shadow, Group::Shadows)
    );
    assert_eq!(
        bias(shadow),
        (DepthTest::Reader, Profile::Flat, -4 - height_lift(40))
    );
    assert_eq!(body.group, Group::Effects);
    assert!(matches!(body.paint, Paint::Lit(_)));
    assert_eq!(
        bias(body),
        (DepthTest::Reader, Profile::Flat, -2 - height_lift(300))
    );
    // The body is placed around the lifted point, the shadow around the ground's.
    let (bx, by) = project_lifted(2000, 2200, 300);
    let (sx, sy) = project_lifted(2000, 2200, 40);
    assert!((body.x - bx).abs() < 64 && (body.y - by).abs() < 64);
    assert!((shadow.x - sx).abs() < 64 && (shadow.y - sy).abs() < 64);
    // Another piece in another pose holds its own images, drawn differently.
    assert_ne!(items[0][1].image, items[1][1].image);
    let px = |i: &Item| scene.images[i.image as usize].pixels().to_vec();
    assert_ne!(px(&items[0][0]), px(&items[1][0]), "a turned footprint");
    // A type without art draws nothing.
    assert!(items[2].is_empty());
}

/// spec: S-0317 claim 23 (one section, the type's VoxelIndex; outside the model nothing) and claim 25 (Translucent: 50 % by S-0102 F8's row; a house's palette for a piece that holds one; the start-up mine table without one, the resource flag or not)
#[test]
fn a_piece_draws_its_section_in_its_colours() {
    let (import, catalogue, assets, idx) = chunk_context();
    let at = |n: &[u8]| {
        import.outcome.registries[&Kind::VoxelAnimations]
            .iter()
            .position(|t| t.name.eq_ignore_ascii_case(n))
            .unwrap()
    };
    let mut housed = piece(5, idx, 0);
    housed.owner = b"Neutral".to_vec();
    let list = [
        piece(1, idx, 0),
        piece(2, at(b"Orb"), 0),
        piece(3, at(b"Gone"), 0),
        piece(4, at(b"Ore"), 0),
        housed,
    ];
    let (scene, items) = draw(&import, &catalogue, &assets, &list);
    let size = |i: &Item| {
        let im = &scene.images[i.image as usize];
        im.pixels().iter().filter(|&&p| p != 0).count()
    };
    // The shadows show the footprint of the section drawn (the bodies need a lighting table).
    assert_ne!(size(&items[0][0]), size(&items[1][0]), "another section");
    assert!(
        items[2].is_empty(),
        "a section outside the model draws nothing"
    );
    let row = |i: &Item| match i.paint {
        Paint::Lit(r) | Paint::Translucent(r, _) => r,
        _ => panic!("a lit body"),
    };
    assert!(matches!(
        items[1][1].paint,
        Paint::Translucent(_, Translucency::Half)
    ));
    assert_eq!(
        row(&items[3][1]).base,
        row(&items[0][1]).base,
        "the mine table"
    );
    assert_ne!(
        row(&items[4][1]).base,
        row(&items[0][1]).base,
        "the house's palette"
    );
}

/// spec: S-0317 claim 27 (in a high-bridge cell with the piece at least f + 416 high, the shadow lies one lepton above the deck, L(f + 417), and reads depth from f + 416; below that height, on the ground)
#[test]
fn a_piece_over_a_deck_shadows_the_deck() {
    let (import, catalogue, assets, idx) = chunk_context();
    let mut high = piece(1, idx, 0);
    high.bridge = true;
    high.pos.2 = 40 + 416;
    let mut low = high.clone();
    low.id = 2;
    low.pos.2 = 40 + 415;
    let (_, items) = draw(&import, &catalogue, &assets, &[high, low]);
    assert_eq!(bias(&items[0][0]).2, -4 - height_lift(40 + 416));
    assert_eq!(bias(&items[1][0]).2, -4 - height_lift(40));
    let lift = project_lifted(2000, 2200, 40 + 417).1 - project_lifted(2000, 2200, 40).1;
    assert_eq!(items[0][0].y - items[1][0].y, lift);
}

/// spec: S-0317 claims 22, 24 and 27 (every drawn frame shows the current orientation and its light, which needs a drawing per pose; ours: each piece's images are rewritten in place, so the scene's images stay bounded by the pieces in play)
#[test]
fn a_spinning_piece_keeps_its_image_slots() {
    let (import, catalogue, assets, idx) = chunk_context();
    let frames: Vec<Vec<PieceDraw>> = (0..50).map(|t| vec![piece(1, idx, t)]).collect();
    let (scene, items, painter) = draw_frames(&import, &catalogue, &assets, &frames);
    let held = painter.piece_images();
    assert!(held <= 4, "at most a regrowth of each slot: {held}");
    // The images rewritten in place carry the current pose.
    let last = &items[49][0][1];
    assert!(scene.rewritten.contains_key(&last.image));
    // A piece that leaves play gives its slots to the next one.
    let mut next = frames;
    next.push(vec![]);
    next.push(vec![piece(9, idx, 0)]);
    let (scene2, _, painter) = draw_frames(&import, &catalogue, &assets, &next);
    assert_eq!(painter.piece_images(), held);
    assert_eq!(scene2.images.len(), scene.images.len());
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
