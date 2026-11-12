//! Scene builder tests on an invented ruleset, art, theater and map (every
//! name and number here is made up).

use std::collections::BTreeMap;

use glasnost_convert::art::TypeArt;
use glasnost_convert::inputs::{InputFile, Inputs};
use glasnost_convert::kinds::Kind;
use glasnost_convert::map::synth::{MapText, Record, overlay_stream, record_bytes};
use glasnost_convert::map::{DecodeContext, decode};
use glasnost_convert::theater::Catalogue;
use glasnost_convert::{ImportOptions, convert};
use glasnost_fmt::image::IndexedImage;
use glasnost_fmt::shp::{ShpWriter, WriteFrame};
use glasnost_fmt::tmp::{TmpTile, TmpWriter, WriteTile};
use glasnost_ini::{Document, Quirks};
use glasnost_render::compose::{DepthTest, Paint, Profile};
use glasnost_render::palette::{PaletteQuirks, PaletteState};
use glasnost_render::scene::{Group, Item, ItemDepth};
use glasnost_vfs::theater::Theater;

use super::build::{Assets, Built, Options, build};

pub(super) const RULES: &str = "[BuildingTypes]\n1=HUTX\n[InfantryTypes]\n1=MANX\n[VehicleTypes]\n1=CARX\n[OverlayTypes]\n0=OREX\n1=CRATEX\n2=FENCEX\n\
    [OREX]\nTiberium=yes\n[CRATEX]\nCrate=yes\n[FENCEX]\nDrawFlat=no\n\
    [SmudgeTypes]\n0=CRATERX\n[CRATERX]\nWidth=2\n[TerrainTypes]\n0=TREEX\n1=MINEX\n[MINEX]\nSpawnsTiberium=yes\n\
    [Colors]\nC0=1,2,3\nC1=10,200,200\nC2=20,200,200\nC3=30,200,200\nC4=40,200,200\nC5=50,200,200\n\
    C6=60,200,200\nC7=70,200,200\nC8=80,200,200\nC9=90,200,200\nC10=100,200,200\nC11=110,200,200\n\
    C12=120,200,200\nC13=130,200,200\nC14=140,200,200\nLightGrey=0,0,150\n";
pub(super) const ART: &str = "[HUTX]\nFoundation=2x2\nAnimIdle=1,1,0\n[MANX]\nSequence=MANSEQ\n[MANSEQ]\nReady=3,1,2\n\
    [CARX]\nFacings=8\nWalkFrames=2\n";
const THEATER: &str = "[General]\nClearTile=0\n[TileSet0000]\nFileName=clr\nTilesInSet=1\n";

pub(super) fn shp(
    canvas: (u16, u16),
    frames: usize,
    frame_size: (u32, u32),
    at: (u16, u16),
) -> Vec<u8> {
    ShpWriter {
        width: canvas.0,
        height: canvas.1,
        frames: (0..frames)
            .map(|i| WriteFrame {
                x: at.0,
                y: at.1,
                image: IndexedImage::new(
                    frame_size.0,
                    frame_size.1,
                    vec![(i % 200 + 1) as u8; (frame_size.0 * frame_size.1) as usize],
                )
                .unwrap(),
                compression: 1,
            })
            .collect(),
    }
    .write()
}

fn tmp() -> Vec<u8> {
    let header = TmpTile {
        offset: 0,
        x: 0,
        y: 0,
        extra_offset: 0,
        depth_offset: 0,
        extra_depth_offset: 0,
        extra_x: 0,
        extra_y: 0,
        extra_width: 0,
        extra_height: 0,
        flags: 0,
        height: 0,
        terrain_type: 0,
        ramp_type: 0,
        radar_left: [1, 2, 3],
        radar_right: [4, 5, 6],
    };
    TmpWriter {
        cells_x: 1,
        cells_y: 1,
        tile_width: 60,
        tile_height: 30,
        tiles: vec![Some(WriteTile {
            header,
            diamond: vec![7; 60 * 30 / 2],
            depth: Some(vec![3; 60 * 30 / 2]),
            extra: None,
        })],
    }
    .write()
}

fn input(name: &str, text: &str) -> Option<InputFile> {
    Some(InputFile {
        name: name.into(),
        origin: "test".into(),
        bytes: text.as_bytes().to_vec(),
    })
}

fn rec(x: u16, y: u16, level: u8) -> Record {
    Record {
        x,
        y,
        tile: 0,
        sub_tile: 0,
        level,
        ice: 0,
    }
}

pub(super) fn context() -> (glasnost_convert::Import, Catalogue, Assets) {
    context_with(RULES)
}

/// A type's art: its image name and sprite file.
pub(super) fn art_of(image: &str, sprite: &str) -> TypeArt {
    TypeArt {
        image: image.as_bytes().to_vec(),
        sprite: Some(sprite.as_bytes().to_vec()),
        ..Default::default()
    }
}

/// The test context with the rules text `rules`.
pub(super) fn context_with(rules: &str) -> (glasnost_convert::Import, Catalogue, Assets) {
    context_full(rules, ART)
}

/// The test context with the rules text `rules` and the art text `art`.
pub(super) fn context_full(
    rules: &str,
    art: &str,
) -> (glasnost_convert::Import, Catalogue, Assets) {
    let mut import = convert(
        &Inputs {
            base: input("rules.ini", rules),
            art: input("art.ini", art),
            ..Default::default()
        },
        &ImportOptions::default(),
    );
    let art = art_of;
    let mut hut = art("HUTX", "hutx.shp");
    hut.building_sprites.insert("bib", b"hutxbib.shp".to_vec());
    import.arts = BTreeMap::from([
        ((Kind::Buildings, 0), hut),
        ((Kind::Infantry, 0), art("MANX", "manx.shp")),
        ((Kind::Overlays, 0), art("OREX", "orex.shp")),
        ((Kind::Overlays, 1), art("CRATEX", "cratex.shp")),
        ((Kind::Overlays, 2), art("FENCEX", "fencex.shp")),
        ((Kind::Vehicles, 0), art("CARX", "carx.shp")),
        ((Kind::Smudges, 0), art("CRATERX", "craterx.shp")),
        ((Kind::TerrainObjects, 0), art("TREEX", "treex.shp")),
        ((Kind::TerrainObjects, 1), art("MINEX", "minex.shp")),
    ]);
    let tdoc = Document::parse(THEATER.as_bytes(), Quirks::default(), &mut Vec::new());
    let catalogue = Catalogue::read(&tdoc, Theater::Temperate);
    let mut assets = Assets::default();
    assets.insert(b"clr01.tem", tmp());
    // Building sprite: canvas 40 × 30, 4 frames (2 body, 2 shadow) of 4 × 4 at (6, 8).
    assets.insert(b"hutx.shp", shp((40, 30), 4, (4, 4), (6, 8)));
    assets.insert(b"hutxbib.shp", shp((40, 30), 4, (4, 4), (6, 8)));
    // Infantry: canvas 20 × 20, 40 frames (20 body, 20 shadow).
    assets.insert(b"manx.shp", shp((20, 20), 40, (2, 2), (9, 9)));
    assets.insert(b"orex.shp", shp((10, 10), 8, (2, 2), (4, 4)));
    assets.insert(b"cratex.shp", shp((10, 10), 2, (2, 2), (4, 4)));
    assets.insert(b"fencex.shp", shp((10, 10), 2, (2, 2), (4, 4)));
    assets.insert(b"craterx.shp", shp((10, 10), 1, (2, 2), (4, 4)));
    assets.insert(b"carx.shp", shp((10, 10), 32, (2, 2), (4, 4)));
    assets.insert(b"treex.shp", shp((10, 10), 2, (2, 2), (4, 4)));
    assets.insert(b"minex.shp", shp((10, 10), 2, (2, 2), (4, 4)));
    (import, catalogue, assets)
}

/// Builds the scene of `text` in the test context.
fn build_text(text: &[u8], options: &Options) -> (Built, glasnost_map::Map) {
    build_in(context(), text, options)
}

/// Builds the scene of `text` in the context `ctx`.
fn build_in(
    ctx: (glasnost_convert::Import, Catalogue, Assets),
    text: &[u8],
    options: &Options,
) -> (Built, glasnost_map::Map) {
    let (import, catalogue, assets) = ctx;
    let doc = Document::parse(text, Quirks::default(), &mut Vec::new());
    let map = decode(
        &doc,
        &DecodeContext {
            catalogue: Some(&catalogue),
            ..Default::default()
        },
    )
    .map;
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    (
        build(&map, &import, &catalogue, &palettes, &assets, options),
        map,
    )
}

/// The test map's text.
pub(super) fn scene_map_text() -> Vec<u8> {
    let m = MapText {
        ini: "[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,10,10\nLocalSize=0,0,10,10\n\
              [Structures]\n0=Neutral,HUTX,256,6,10,0\n1=Americans,HUTX,256,8,12,0\n\
              [Infantry]\n0=Neutral,MANX,256,9,11,0,Guard,64\n1=Neutral,MANX,256,9,13,0,Guard,0\n"
            .into(),
        terrain: Some(record_bytes(&[rec(7, 9, 2), rec(0, 0, 0)])),
        overlay: Some(overlay_stream(255, &[(5, 9, 0), (5, 10, 1), (7, 9, 2)])),
        overlay_data: Some(overlay_stream(0, &[(5, 9, 3)])),
        ..Default::default()
    };
    m.text().into_bytes()
}

fn scene(options: &Options) -> (Built, glasnost_map::Map) {
    build_text(&scene_map_text(), options)
}

fn items_of(b: &Built, kind: &str) -> Vec<Item> {
    b.scene
        .items
        .iter()
        .filter(|i| i.object.is_some_and(|o| b.objects[o as usize].kind == kind))
        .copied()
        .collect()
}

fn bias(i: &Item) -> (Profile, i32) {
    match i.depth {
        ItemDepth::Profile { profile, bias, .. } => (profile, bias),
        ItemDepth::Tile { .. } => panic!("not a profile item"),
    }
}

/// spec: S-0101 claim 4 (terrain lift 15 per level), S-0102 F3 (t = top row + 15 × level)
#[test]
fn tiles_are_lifted_and_depth_anchored() {
    let (b, _) = scene(&Options::default());
    let tile = b
        .scene
        .items
        .iter()
        .find(|i| {
            i.group == Group::Tiles
                && matches!(i.depth, ItemDepth::Tile { reference_row, .. } if reference_row == 15 * 16)
                && i.x == 30 * (7 - 9) - 30
        })
        .expect("the tile of cell (7, 9)");
    // Cell (7, 9) is at level 2: drawn 30 pixels higher, depth-anchored at level 0.
    assert_eq!(tile.y, 15 * 16 - 30);
    assert_eq!(tile.test, DepthTest::Tile);
}

/// spec: S-0108 claims 2, 3, 6, 7, 8, 9; S-0102 claims 13, 16; S-0009 claim 40 (initial state)
#[test]
fn building_body_shadow_and_bib() {
    let (b, _) = scene(&Options::default());
    let items = items_of(&b, "building");
    // Only the Neutral building appears in the skirmish view (S-0009 claim 27).
    assert_eq!(items.len(), 3);
    let (body, shadow, bib) = (items[0], items[1], items[2]);
    // Top corner of (6, 10) at height 0: (30·(6 − 10), 15·16) = (−120, 240).
    // Canvas 40 × 30 centred there, frame at (6, 8): (−120 − 20 + 6, 240 − 15 + 8).
    assert_eq!((body.x, body.y), (-134, 233));
    assert_eq!(bias(&body), (Profile::Upright, -2));
    assert_eq!(bias(&shadow), (Profile::Flat, -4));
    assert_eq!(shadow.paint, Paint::Shadow);
    assert_eq!(bias(&bib), (Profile::Flat, -3));
    // Idle range starts at 1: body frame 1 (S-0009 claim 40: idle at once; claim 6 gives
    // stage a), shadow frame 1 + 2 = 3.
    assert_eq!(b.scene.images[body.image as usize].pixels()[0], 2);
    assert_eq!(b.scene.images[shadow.image as usize].pixels()[0], 4);
    // Bib uses the same frame without the clamp.
    assert_eq!(b.scene.images[bib.image as usize].pixels()[0], 2);
    // Without the skirmish view both buildings appear.
    let (all, _) = scene(&Options {
        skirmish: false,
        ..Options::default()
    });
    assert_eq!(items_of(&all, "building").len(), 6);
}

/// spec: S-0107 claims 2, 3, 19 (stand entry, facing group, shadow frame); S-0102 claims 13, 20, 28
#[test]
fn infantry_standing_frame_and_shadow() {
    let (b, _) = scene(&Options::default());
    let items = items_of(&b, "infantry");
    assert_eq!(items.len(), 4);
    // Facing value 64 → 16384 (map east) → group 5; frame = 3 + 2·5 = 13.
    let shadow = items.iter().find(|i| i.paint == Paint::Shadow).unwrap();
    let body = items
        .iter()
        .find(|i| i.paint != Paint::Shadow && i.object == shadow.object)
        .unwrap();
    assert_eq!(b.scene.images[body.image as usize].pixels()[0], 14);
    // Shadow frame 13 + 20 = 33.
    assert_eq!(b.scene.images[shadow.image as usize].pixels()[0], 34);
    // U = −L(0) − 1 + F; the cell two steps toward the viewer is not high. Body U − 12.
    assert_eq!(bias(body), (Profile::Upright, -13));
    // The own shadow: −L(z) − 4, flat, drawn right after the body (claim 28).
    assert_eq!(bias(shadow), (Profile::Flat, -4));
    assert_eq!(body.test, DepthTest::Reader);
    let at = |i: &Item| b.scene.items.iter().position(|x| x == i).unwrap();
    assert_eq!(at(shadow), at(body) + 1);
}

/// spec: S-0102 claim 6 (the ground layer is sorted by X + Y; equal keys keep insertion order)
#[test]
fn ground_layer_is_sorted() {
    let (b, _) = scene(&Options::default());
    let keys: Vec<i32> = b.scene.items[b.scene.cell_pass_len..]
        .iter()
        .filter(|i| i.paint != Paint::Shadow)
        .map(|i| b.objects[i.object.unwrap() as usize].cell)
        .map(|c| c.x + c.y)
        .collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted);
}

/// spec: S-0009 claim 12 (crates never appear in skirmish), S-0102 claims 12 and 13 (overlay biases)
#[test]
fn overlays() {
    let (b, map) = scene(&Options::default());
    let items = items_of(&b, "overlay");
    let names: Vec<&str> = b
        .objects
        .iter()
        .filter(|o| o.kind == "overlay")
        .map(|o| o.type_name.as_str())
        .collect();
    assert_eq!(names, ["OREX", "FENCEX"]);
    // Ore at level 0: flat, −2.
    assert_eq!(bias(&items[0]), (Profile::Flat, -2));
    // S-0112 claims 1 and 12 (no tiberium run: the stored type, frame = data value 3, pixel 4).
    assert_eq!(b.scene.images[items[0].image as usize].pixels()[0], 4);
    // S-0102 claim 24: the ore's draw point is 12 rows up. Cell (5, 9): centre (−120, 225);
    // canvas 10 × 10 centred, frame at (4, 4): (−120 − 5 + 4, 225 − 12 − 5 + 4).
    assert_eq!((items[0].x, items[0].y), (-121, 212));
    // S-0112 claim 8: the ore's shadow is the stored type's frame d + ⌊N/2⌋ = 3 + 4 (pixel 8).
    let shadows: Vec<&Item> = items.iter().filter(|i| i.paint == Paint::Shadow).collect();
    let shadow = shadows[0];
    assert_eq!(b.scene.images[shadow.image as usize].pixels()[0], 8);
    // The fence is upright (DrawFlat=no) on level 2: −17 − 30.
    let level = map
        .cells
        .get(glasnost_map::CellXY::new(7, 9))
        .unwrap()
        .level;
    assert_eq!(level, 2);
    let bodies: Vec<&Item> = items.iter().filter(|i| i.paint != Paint::Shadow).collect();
    assert_eq!(bias(bodies[1]), (Profile::Upright, -47));
    assert!(bodies.iter().all(|i| !i.provisional));
}

/// spec: S-0102 claims 13, 16 (rank 3) and 27; S-0112 claim 8 (the depth of every overlay shadow)
#[test]
fn overlay_shadow_depth_and_rank() {
    let (b, _) = scene(&Options::default());
    let items = items_of(&b, "overlay");
    let (bodies, shadows): (Vec<&Item>, Vec<&Item>) =
        items.iter().partition(|i| i.paint != Paint::Shadow);
    assert_eq!((bodies.len(), shadows.len()), (2, 2));
    for s in &shadows {
        // Flat writers, no longer provisional (Q-0021 answered).
        assert_eq!(s.test, DepthTest::Writer);
        assert!(!s.provisional);
    }
    // The ore at level 0: −2 − 15 × 0, the same as its flat body.
    assert_eq!(bias(shadows[0]), (Profile::Flat, -2));
    // The upright fence at level 2: −2 − 15 × 2, not its body's −17 − 30.
    assert_eq!(bias(shadows[1]), (Profile::Flat, -32));
    // Every overlay body ranks before every overlay shadow (claim 16, rank 3).
    let at = |i: &Item| b.scene.items.iter().position(|x| x == i).unwrap();
    let last_body = bodies.iter().map(|i| at(i)).max().unwrap();
    let first_shadow = shadows.iter().map(|i| at(i)).min().unwrap();
    assert!(last_body < first_shadow);
}

/// spec: S-0102 claims 13 and 27, S-0112 claims 1 and 8 (a deck cell's shadow takes the cell's
/// level alone: 60 depth units farther than the deck's own pixels); S-0102 claims 15 and 24 (the
/// deck's draw point is claim 24's, with no extra lift for its 4 levels)
#[test]
fn deck_overlay_shadow_skips_the_deck_levels() {
    // Overlay types 0 … 24, the deck at bridge position 24 (S-0112 claim 1).
    let mut rules = String::from("[OverlayTypes]\n");
    for i in 0..25 {
        rules.push_str(&format!("{i}=OV{i}\n"));
    }
    let mut ctx = context_with(&format!("{RULES}{rules}"));
    ctx.0
        .arts
        .insert((Kind::Overlays, 24), art_of("OV24", "deckx.shp"));
    ctx.2.insert(b"deckx.shp", shp((10, 10), 8, (2, 2), (4, 4)));
    let m = MapText {
        ini: "[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,10,10\nLocalSize=0,0,10,10\n".into(),
        terrain: Some(record_bytes(&[rec(5, 9, 1), rec(0, 0, 0)])),
        overlay: Some(overlay_stream(255, &[(5, 9, 24)])),
        overlay_data: Some(overlay_stream(0, &[])),
        ..Default::default()
    };
    let (b, _) = build_in(ctx, m.text().as_bytes(), &Options::default());
    let items = items_of(&b, "overlay");
    let body = items.iter().find(|i| i.paint != Paint::Shadow).unwrap();
    let shadow = items.iter().find(|i| i.paint == Paint::Shadow).unwrap();
    // Body: −2 − 15 × (1 + 4); shadow: −2 − 15 × 1.
    assert_eq!(bias(body), (Profile::Flat, -77));
    assert_eq!(bias(shadow), (Profile::Flat, -17));
    // Draw point: cell (5, 9)'s top corner (−120, 210), + 15 to the centre, − 15 for level 1,
    // s = −16 for a deck cell with data 0; the 10 × 10 canvas centred there, the frame at (4, 4).
    let (px, py) = (-120, 210 + 15 - 15 - 16);
    assert_eq!((body.x, body.y), (px - 5 + 4, py - 5 + 4));
    assert_eq!((shadow.x, shadow.y), (body.x, body.y));
}

/// spec: S-0112 claim 1 (deck and marker cells come before the tiberium row: outside them ore and
/// gems use the start-up resource table; a deck cell's overlay takes the tile table and the deck
/// frame of claim 6; a marker cell draws no body), claim 8 (a marker cell's shadow is its own
/// type's d + ⌊N/2⌋); S-0104 claims 3 and 13 (veinhole marker shadows are drawn); S-0103 claim 8
/// (ore and gems take the start-up resource table, with S-0112 claim 1's deck exception)
#[test]
fn deck_and_marker_cells_come_before_the_ore_row() {
    // Overlay types 0 … 167 (in place of the base list): a bridge position (24), a marker
    // position (167) and an ordinary position (5), each with the tiberium flag.
    let mut list = String::from("[OverlayTypes]\n");
    for i in 0..168 {
        list.push_str(&format!("{i}=OV{i}\n"));
    }
    let mut rules = RULES.replace("[OverlayTypes]\n0=OREX\n1=CRATEX\n2=FENCEX\n", &list);
    for i in [5, 24, 167] {
        rules.push_str(&format!("[OV{i}]\nTiberium=yes\n"));
    }
    let mut ctx = context_with(&rules);
    for i in [5usize, 24, 167] {
        let file = format!("ov{i}.shp");
        ctx.0
            .arts
            .insert((Kind::Overlays, i), art_of(&format!("OV{i}"), &file));
        ctx.2
            .insert(file.as_bytes(), shp((10, 10), 8, (2, 2), (4, 4)));
    }
    let m = MapText {
        ini: "[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,10,10\nLocalSize=0,0,10,10\n".into(),
        terrain: Some(record_bytes(&[rec(0, 0, 0)])),
        overlay: Some(overlay_stream(255, &[(5, 9, 24), (6, 9, 5), (7, 9, 167)])),
        overlay_data: Some(overlay_stream(0, &[])),
        ..Default::default()
    };
    let (b, _) = build_in(ctx, m.text().as_bytes(), &Options::default());
    let of = |name: &str| -> Vec<Item> {
        b.scene
            .items
            .iter()
            .filter(|i| {
                i.object
                    .is_some_and(|o| b.objects[o as usize].type_name == name)
            })
            .copied()
            .collect()
    };
    let pixel = |i: &Item| b.scene.images[i.image as usize].pixels()[0];
    let body = |items: &[Item]| items.iter().find(|i| i.paint != Paint::Shadow).copied();
    let (deck, ore, marker) = (of("OV24"), of("OV5"), of("OV167"));
    let deck_body = body(&deck).expect("the deck cell's body");
    let ore_body = body(&ore).expect("the ore cell's body");
    // Deck cell (5, 9), data 0: claim 6's frame 0 + T(1, 1) = 2 (pixel 3), not the data value.
    assert_eq!(pixel(&deck_body), 3);
    // Ore outside deck and marker cells: the data value 0 (pixel 1).
    assert_eq!(pixel(&ore_body), 1);
    // The deck's tile-table row is not the start-up resource table's row of the ore.
    assert_ne!(deck_body.paint, ore_body.paint);
    // The marker cell draws no body; its shadow is its own type's 0 + ⌊8/2⌋ (pixel 5).
    assert_eq!(body(&marker), None);
    assert_eq!(marker.len(), 1);
    assert_eq!(marker[0].paint, Paint::Shadow);
    assert_eq!(pixel(&marker[0]), 5);
}

/// A 1 × 1 tile file whose diamond holds `value`, with an optional extra
/// image (stored position, size) whose pixels hold `value + 1`; the tile
/// cell's stored position is (100, 200).
fn tmp_tile(value: u8, extra: Option<(i32, i32, i32, i32)>) -> Vec<u8> {
    let (ex, ey, ew, eh) = extra.unwrap_or((0, 0, 0, 0));
    let header = TmpTile {
        offset: 0,
        x: 100,
        y: 200,
        extra_offset: 0,
        depth_offset: 0,
        extra_depth_offset: 0,
        extra_x: ex,
        extra_y: ey,
        extra_width: ew,
        extra_height: eh,
        flags: 0,
        height: 0,
        terrain_type: 0,
        ramp_type: 0,
        radar_left: [1, 2, 3],
        radar_right: [4, 5, 6],
    };
    TmpWriter {
        cells_x: 1,
        cells_y: 1,
        tile_width: 60,
        tile_height: 30,
        tiles: vec![Some(WriteTile {
            header,
            diamond: vec![value; 900],
            depth: None,
            extra: extra.map(|_| (vec![value + 1; (ew * eh) as usize], None)),
        })],
    }
    .write()
}

/// Builds a 10 × 10 map whose terrain records are `records` (x, y, tile),
/// with `assets` changed by `edit`.
fn build_tiles(records: &[(u16, u16, i32)], edit: impl FnOnce(&mut Assets)) -> Built {
    let (import, catalogue, mut assets) = context();
    edit(&mut assets);
    let mut recs: Vec<Record> = records
        .iter()
        .map(|&(x, y, tile)| Record {
            x,
            y,
            tile,
            sub_tile: 0,
            level: 0,
            ice: 0,
        })
        .collect();
    recs.push(rec(0, 0, 0));
    let m = MapText {
        ini: "[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,10,10\nLocalSize=0,0,10,10\n".into(),
        terrain: Some(record_bytes(&recs)),
        ..Default::default()
    };
    let doc = Document::parse(m.text().as_bytes(), Quirks::default(), &mut Vec::new());
    let map = decode(
        &doc,
        &DecodeContext {
            catalogue: Some(&catalogue),
            ..Default::default()
        },
    )
    .map;
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    build(
        &map,
        &import,
        &catalogue,
        &palettes,
        &assets,
        &Options::default(),
    )
}

/// The tile items whose diamond lies at cell (x, y)'s top corner, level 0.
fn tile_at(b: &Built, x: i32, y: i32) -> Vec<Item> {
    b.scene
        .items
        .iter()
        .filter(|i| i.group == Group::Tiles)
        .filter(|i| matches!(i.paint, Paint::LitDiamond(_)))
        .filter(|i| i.x == 30 * (x - y) - 30 && i.y == 15 * (x + y))
        .copied()
        .collect()
}

/// spec: S-0010 claims 13 and 22 (diamonds placed from the top corner, no transparent index; the image by P4)
#[test]
fn claims_13_22_diamond_and_variant() {
    let b = build_tiles(&[(6, 9, 0), (7, 9, 0), (8, 9, 0), (6, 10, 0)], |a| {
        a.insert(b"clr01.tem", tmp_tile(7, None));
        a.insert(b"clr01a.tem", tmp_tile(8, None));
    });
    for (x, y) in [(6, 9), (7, 9), (8, 9), (6, 10)] {
        let t = tile_at(&b, x, y);
        assert_eq!(t.len(), 1, "cell ({x}, {y})");
        let px = b.scene.images[t[0].image as usize].pixels()[28];
        // Two images: P4(x mod 4, y mod 4) mod 2 picks the plain (7) or the variant (8).
        let p = glasnost_convert::theater::P4[(y % 4) as usize][(x % 4) as usize] % 2;
        assert_eq!(px, 7 + p, "cell ({x}, {y})");
    }
}

/// `tmp_tile` with the tile cell's flag value set to `flags` (the writer
/// keeps the bit of value 4).
fn tmp_tile_flags(value: u8, flags: u32) -> Vec<u8> {
    let bytes = tmp_tile(value, None);
    let tmp = glasnost_fmt::tmp::Tmp::parse(&bytes).expect("tile file");
    let mut header = tmp.tiles[0].expect("tile cell");
    header.flags = flags;
    TmpWriter {
        cells_x: 1,
        cells_y: 1,
        tile_width: 60,
        tile_height: 30,
        tiles: vec![Some(WriteTile {
            header,
            diamond: vec![value; 900],
            depth: None,
            extra: None,
        })],
    }
    .write()
}

/// spec: S-0010 claim 13 (the alternate-image mark, the bit of value 4 of the plain image's tile cell,
/// read from the plain image's file only: a marked cell shows the plain image at map load, a mark in a
/// variant file has no effect)
#[test]
fn claim_13_alternate_image_mark() {
    let cells = [(6, 9, 0), (7, 9, 0), (8, 9, 0), (6, 10, 0), (7, 10, 0)];
    // Some of these cells show the variant by P4.
    assert!(cells.iter().any(|&(x, y, _)| {
        glasnost_convert::theater::P4[(y % 4) as usize][(x % 4) as usize] % 2 == 1
    }));
    let b = build_tiles(&cells, |a| {
        a.insert(b"clr01.tem", tmp_tile_flags(7, 4));
        a.insert(b"clr01a.tem", tmp_tile(8, None));
    });
    for &(x, y, _) in &cells {
        let t = tile_at(&b, i32::from(x), i32::from(y));
        assert_eq!(t.len(), 1, "cell ({x}, {y})");
        assert_eq!(
            b.scene.images[t[0].image as usize].pixels()[28],
            7,
            "cell ({x}, {y})"
        );
    }
    // The mark in the variant's file only: the pattern as usual.
    let b = build_tiles(&cells, |a| {
        a.insert(b"clr01.tem", tmp_tile(7, None));
        a.insert(b"clr01a.tem", tmp_tile_flags(8, 4));
    });
    for &(x, y, _) in &cells {
        let t = tile_at(&b, i32::from(x), i32::from(y));
        let p = glasnost_convert::theater::P4[(y % 4) as usize][(x % 4) as usize] % 2;
        assert_eq!(
            b.scene.images[t[0].image as usize].pixels()[28],
            7 + p,
            "cell ({x}, {y})"
        );
    }
}

/// `tmp_tile` with the tile cell's two radar colours set to `left` and
/// `right`.
fn tmp_tile_radar(value: u8, left: [u8; 3], right: [u8; 3]) -> Vec<u8> {
    let bytes = tmp_tile(value, None);
    let tmp = glasnost_fmt::tmp::Tmp::parse(&bytes).expect("tile file");
    let mut header = tmp.tiles[0].expect("tile cell");
    header.radar_left = left;
    header.radar_right = right;
    TmpWriter {
        cells_x: 1,
        cells_y: 1,
        tile_width: 60,
        tile_height: 30,
        tiles: vec![Some(WriteTile {
            header,
            diamond: vec![value; 900],
            depth: None,
            extra: None,
        })],
    }
    .write()
}

/// spec: S-0010 claim 20 (the first tile radar colour, from the image variant the cell shows; the
/// second colour has no effect), S-0010 claim 13 (the variant by P4)
#[test]
fn claim_20_radar_colour_of_the_shown_variant() {
    let cells = [(6, 9, 0), (7, 9, 0), (8, 9, 0), (6, 10, 0), (7, 10, 0)];
    let files = |a: &mut Assets| {
        a.insert(b"clr01.tem", tmp_tile_radar(7, [10, 20, 30], [1, 1, 1]));
        a.insert(b"clr01a.tem", tmp_tile_radar(8, [40, 50, 60], [2, 2, 2]));
    };
    let b = build_tiles(&cells, files);
    let mut assets = Assets::default();
    files(&mut assets);
    let colours = super::build::tile_radar_colours(&b.shown_tiles, &assets);
    for &(x, y, _) in &cells {
        let p = glasnost_convert::theater::P4[(y % 4) as usize][(x % 4) as usize] % 2;
        let want = if p == 0 { [10, 20, 30] } else { [40, 50, 60] };
        let c = glasnost_map::CellXY::new(i32::from(x), i32::from(y));
        assert_eq!(colours.get(&c), Some(&want), "cell ({x}, {y})");
    }
}

/// spec: S-0010 claim 23 (the extra image at the box corner plus the position difference, after its diamond)
#[test]
fn claim_23_extra_image() {
    let b = build_tiles(&[(6, 9, 0)], |a| {
        a.insert(b"clr01.tem", tmp_tile(7, Some((90, 170, 12, 20))));
    });
    let i = b
        .scene
        .items
        .iter()
        .position(|i| i.x == 30 * (6 - 9) - 30 && i.y == 15 * 15 && i.group == Group::Tiles)
        .expect("diamond");
    let extra = b.scene.items[i + 1];
    assert_eq!((extra.x, extra.y), (30 * (6 - 9) - 30 - 10, 15 * 15 - 30));
    // Same row as its diamond; index 0 transparent in the extra image.
    let Paint::LitDiamond(row) = b.scene.items[i].paint else {
        panic!("diamond paint");
    };
    assert_eq!(extra.paint, Paint::Lit(row));
}

/// spec: S-0010 claim 24 (a stray tile number paints position 0 of the clear tile)
#[test]
fn claim_24_stray_tile_shows_clear_tile() {
    let b = build_tiles(&[(6, 9, 5)], |a| {
        a.insert(b"clr01.tem", tmp_tile(7, None));
    });
    let t = tile_at(&b, 6, 9);
    assert_eq!(t.len(), 1);
    assert_eq!(b.scene.images[t[0].image as usize].pixels()[28], 7);
}

/// spec: S-0102 claim 25, S-0112 claim 3, S-0009 claim 42 (smudges: one painting per covered cell,
/// from the first cell's top corner lifted by the painted cell's level, without depth)
#[test]
fn smudges_painted_per_covered_cell() {
    let (b, _) = build_text(
        b"[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,10,10\nLocalSize=0,0,10,10\n\
          [Smudge]\n0=CRATERX,6,9,0\n1=CRATERX,6,10,1\n",
        &Options::default(),
    );
    let items = items_of(&b, "smudge");
    // Width 2: cells (6, 9) and (7, 9), both at level 0; the piece-1 entry places nothing.
    assert_eq!(items.len(), 2);
    for i in &items {
        assert_eq!(i.test, DepthTest::Ignore);
        // Top corner of (6, 9): (−90, 225); canvas 10 × 10 centred, frame at (4, 4).
        assert_eq!((i.x, i.y), (-90 - 5 + 4, 225 - 5 + 4));
    }
    // Smudges come before overlays in the cell pass.
    let first_overlay = b
        .scene
        .items
        .iter()
        .position(|i| i.group == Group::Overlays)
        .unwrap_or(usize::MAX);
    let last_smudge = b
        .scene
        .items
        .iter()
        .rposition(|i| i.group == Group::Smudges)
        .unwrap();
    assert!(last_smudge < first_overlay);
}

/// design: render (#111): a smudge whose rules give a huge footprint is walked only over the
/// cells of the map grid, so the build ends and paints only cells that exist.
/// spec: S-0102 claim 25, S-0112 claim 3 (one painting per covered cell)
#[test]
fn a_huge_smudge_footprint_is_walked_within_the_grid() {
    let ctx = context_with(&RULES.replace(
        "[CRATERX]\nWidth=2",
        "[CRATERX]\nWidth=2147483647\nHeight=2147483647",
    ));
    let (b, map) = build_in(
        ctx,
        b"[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,10,10\nLocalSize=0,0,10,10\n\
          [Smudge]\n0=CRATERX,6,9,0\n",
        &Options::default(),
    );
    let items = items_of(&b, "smudge");
    assert!(!items.is_empty());
    let side = i64::from(map.cells.footprint().coordinate_side());
    // At most one painting for each legal cell at or past the smudge's first cell.
    let legal = (side - 6) * (side - 9);
    let n = items.len() as i64;
    assert!(n <= legal, "{n} paintings, {legal} cells");
}

/// spec: S-0102 claim 23, S-0112 claim 3 (terrain objects at the cell centre; an ore spawner 16 rows
/// higher with the start-up mine table)
#[test]
fn terrain_objects_placement_and_tables() {
    let (b, _) = build_text(
        b"[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,10,10\nLocalSize=0,0,10,10\n\
          [Terrain]\n9006=TREEX\n10006=MINEX\n",
        &Options::default(),
    );
    let items = items_of(&b, "terrain object");
    let bodies: Vec<&Item> = items.iter().filter(|i| i.paint != Paint::Shadow).collect();
    assert_eq!(bodies.len(), 2);
    // (6, 9): centre (−90, 240) at height 0; canvas 10 × 10 centred, frame at (4, 4).
    assert_eq!((bodies[0].x, bodies[0].y), (-90 - 1, 240 - 1));
    // (6, 10): centre (−120, 255), raised 16 rows.
    assert_eq!((bodies[1].x, bodies[1].y), (-120 - 1, 255 - 16 - 1));
    // Different tables: the tree's tile table row, the spawner's mine table row.
    assert_ne!(bodies[0].paint, bodies[1].paint);
    // Shadows: body frame + ⌊N/2⌋ = 1 (pixel 2), flat −3.
    let shadow = items.iter().find(|i| i.paint == Paint::Shadow).unwrap();
    assert_eq!(b.scene.images[shadow.image as usize].pixels()[0], 2);
    assert_eq!(bias(shadow), (Profile::Flat, -3));
}

/// spec: S-0102 claim 23 (a terrain object stands at the centre of its cell at the terrain surface
/// height there) and S-0106 claim 2, F1 (the surface height of a sloped cell): on a ramp it is
/// raised by the ramp's height at the cell centre, not drawn at the cell's level (#96)
#[test]
fn a_terrain_object_on_a_slope_stands_on_the_surface() {
    use glasnost_render::projection::height_lift;
    let text: &[u8] = b"[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,10,10\nLocalSize=0,0,10,10\n\
          [Terrain]\n9006=TREEX\n";
    let body = |slope: u8| {
        let (import, catalogue, assets) = context();
        let doc = Document::parse(text, Quirks::default(), &mut Vec::new());
        let mut map = decode(
            &doc,
            &DecodeContext {
                catalogue: Some(&catalogue),
                ..Default::default()
            },
        )
        .map;
        let cell = map
            .cells
            .get_mut(glasnost_map::CellXY::new(6, 9))
            .expect("a legal cell");
        cell.level = 1;
        cell.slope = slope;
        let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
        let b = build(
            &map,
            &import,
            &catalogue,
            &palettes,
            &assets,
            &Options::default(),
        );
        let items = items_of(&b, "terrain object");
        let body = *items
            .iter()
            .find(|i| i.paint != Paint::Shadow)
            .expect("a body");
        let shadow = *items
            .iter()
            .find(|i| i.paint == Paint::Shadow)
            .expect("a shadow");
        (body, shadow)
    };
    let (flat, flat_shadow) = body(0);
    // A flat cell at level 1: 104 leptons, L = 15 (S-0101 F2); unchanged by the surface height.
    assert_eq!((flat.x, flat.y), (-90 - 1, 240 - 15 - 1));
    for slope in 1..=20u8 {
        let h = glasnost_sim::surface::surface_height(1, slope, 128, 128, true);
        let (sloped, shadow) = body(slope);
        let lift = height_lift(h) - height_lift(104);
        assert_eq!(
            (sloped.x, sloped.y),
            (flat.x, flat.y - lift),
            "slope {slope}"
        );
        assert_eq!(bias(&sloped).1, bias(&flat).1 - lift, "slope {slope}");
        assert_eq!(
            shadow.y,
            flat_shadow.y - lift,
            "slope {slope}: the shadow moves with it"
        );
    }
}

/// spec: S-0102 claims 21 and 22 (a sprite vehicle's facing part and frame; its own shadow right after it)
#[test]
fn sprite_vehicle_frame_and_shadow() {
    let (b, _) = build_text(
        b"[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,10,10\nLocalSize=0,0,10,10\n\
          [Units]\n0=Neutral,CARX,256,6,9,64\n",
        &Options::default(),
    );
    let items = items_of(&b, "vehicle");
    assert_eq!(items.len(), 2);
    let (body, shadow) = (items[0], items[1]);
    // Facing 16384: f₈ = 2, d = 3; walking frames 2, no standing frames: frame 3·2 = 6 (pixel 7).
    assert_eq!(b.scene.images[body.image as usize].pixels()[0], 7);
    // Shadow: 6 + ⌊32/2⌋ = 22 (pixel 23), −L(z) − 4, flat.
    assert_eq!(shadow.paint, Paint::Shadow);
    assert_eq!(b.scene.images[shadow.image as usize].pixels()[0], 23);
    assert_eq!(bias(&shadow), (Profile::Flat, -4));
    assert_eq!(bias(&body), (Profile::Upright, -3));
}

/// Builds `text` in the context `ctx` with typed rules in which HUTX has
/// strength 100.
fn build_with_rules_in(ctx: (glasnost_convert::Import, Catalogue, Assets), text: &[u8]) -> Built {
    use glasnost_rules::sim::BuildingType;
    let (import, catalogue, assets) = ctx;
    let doc = Document::parse(text, Quirks::default(), &mut Vec::new());
    let map = decode(&doc, &DecodeContext::default()).map;
    let mut r = glasnost_rules::schema::Ruleset::default();
    r.registries.insert("buildings".into(), vec!["HUTX".into()]);
    r.sim.buildings.insert(
        "HUTX".into(),
        BuildingType {
            strength: Some(100),
            ..Default::default()
        },
    );
    let rules = glasnost_sim::rules::SimRules::new(&r, "test");
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = super::build::Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: Some(&rules),
        cells: None,
    };
    super::build::build_parts(&inputs, false).0
}

/// spec: S-0009 claim 37 (starting health from the health value), S-0108 claim 2 (damaged: k + 1 in idle)
#[test]
fn claim_37_damaged_building_frame() {
    // Idle from frame 0, so that the damaged frame 1 is still a body frame of
    // the 4-frame sprite (S-0108 claim 3 clamps to ⌊4/2⌋ − 1 = 1).
    let art = ART.replace("AnimIdle=1,1,0", "AnimIdle=0,1,0");
    let body = |health: i32| {
        let b = build_with_rules_in(
            context_full(RULES, &art),
            format!(
                "[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,10,10\nLocalSize=0,0,10,10\n\
                 [Structures]\n0=Neutral,HUTX,{health},6,10,0\n"
            )
            .as_bytes(),
        );
        let items = items_of(&b, "building");
        b.scene.images[items[0].image as usize].pixels()[0]
    };
    // Idle stage 0: frame 0 shows pixel 1. Damaged (h ≤ 0.5): frame 1, pixel 2.
    assert_eq!(body(256), 1);
    assert_eq!(body(128), 2);
    // 130 of 256 gives 50 (truncated), still damaged; 131 gives 51.
    assert_eq!(body(130), 2);
    assert_eq!(body(131), 1);
}

/// spec: S-0108 claim 3 (a request past the body half shows the last body frame and its
/// shadow, frame 2⌊N/2⌋ − 1)
#[test]
fn body_frame_past_the_body_half_shows_the_last() {
    // The 4-frame sprite in idle from frame 1, damaged: f = 2, clamped to 1 (pixel 2), its
    // shadow 1 + 2 = 3 (pixel 4).
    let pixels = building_pixels(context(), 128);
    assert_eq!(pixels[0].1, 2);
    assert_eq!(pixels[1], (Paint::Shadow, 4));
}

/// The first pixel of each building item of a one-building map whose HUTX
/// has health `health` of 256, in the context `ctx`, with its paint.
fn building_pixels(
    ctx: (glasnost_convert::Import, Catalogue, Assets),
    health: i32,
) -> Vec<(Paint, u8)> {
    let b = build_with_rules_in(
        ctx,
        format!(
            "[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,10,10\nLocalSize=0,0,10,10\n\
             [Structures]\n0=Neutral,HUTX,{health},6,10,0\n"
        )
        .as_bytes(),
    );
    items_of(&b, "building")
        .iter()
        .map(|i| (i.paint, b.scene.images[i.image as usize].pixels()[0]))
        .collect()
}

/// spec: S-0108 claim 3 and F2 (a negative image-frame index draws nothing: it is not raised to
/// image frame 0), claim 9 (the shadow is the drawn body frame + ⌊N/2⌋), claim 7 (the bib takes f)
#[test]
fn negative_idle_start_draws_no_body() {
    let art = ART.replace("AnimIdle=1,1,0", "AnimIdle=-1,1,0");
    let pixels = building_pixels(context_full(RULES, &art), 256);
    // f = −1: no body and no bib; the shadow index −1 + 2 = 1 is drawn (pixel 2).
    assert_eq!(pixels, [(Paint::Shadow, 2)]);
    // Damaged in idle: f = −1 + 1 = 0: body frame 0 (pixel 1), shadow 2 (pixel 3), bib 0.
    let pixels = building_pixels(context_full(RULES, &art), 128);
    assert_eq!(pixels.len(), 3);
    assert_eq!(pixels[0].1, 1);
    assert_eq!(pixels[1], (Paint::Shadow, 3));
    assert_eq!(pixels[2].1, 1);
}

/// spec: S-0108 claim 2 (a stationary gate that is not occupiable: 0, damaged G + 1, then the
/// clamp of claim 3), Import mapping (GateStages, an art key, absent 9)
#[test]
fn damaged_gate_body_frame() {
    let rules = format!("{RULES}[HUTX]\nGate=yes\n");
    let art = ART.replace("AnimIdle=1,1,0", "AnimIdle=1,1,0\nGateStages=0");
    let body = |art: &str, health: i32| building_pixels(context_full(&rules, art), health);
    // Undamaged: frame 0 (pixel 1), whatever the idle range says.
    assert_eq!(body(&art, 256)[0].1, 1);
    // Damaged with G = 0: frame 1 (pixel 2), shadow 1 + 2 = 3 (pixel 4).
    let damaged = body(&art, 128);
    assert_eq!(damaged[0].1, 2);
    assert_eq!(damaged[1], (Paint::Shadow, 4));
    // Damaged with the absent-key G = 9: frame 10, clamped to ⌊4/2⌋ − 1 = 1 (pixel 2); its
    // shadow is 1 + 2 = 3 (pixel 4).
    let pixels = body(ART, 128);
    assert_eq!(pixels[0].1, 2);
    assert_eq!(pixels[1], (Paint::Shadow, 4));
    // An occupiable gate takes the occupiable rule first: hurt at tech level 255 when damaged: 1.
    let rules = format!("{RULES}[HUTX]\nGate=yes\nCanBeOccupied=yes\n");
    assert_eq!(building_pixels(context_full(&rules, ART), 128)[0].1, 2);
}

/// Hostile maps never panic or hang in the scene builder: extreme sizes,
/// prefixes and byte flips of a synthetic map.
#[test]
fn hostile_maps_build() {
    for size in ["0,0,2147483647,2147483647", "0,0,100000,3", "0,0,-1,-1"] {
        let text = format!(
            "[Basic]\nNewINIFormat=4\n[Map]\nSize={size}\nLocalSize=0,0,2147483647,9\n\
             [Structures]\n0=Neutral,HUTX,256,2147483647,2147483647,0\n\
             [Infantry]\n0=Neutral,MANX,256,-5,-5,0,Guard,999999\n"
        );
        let _ = build_text(text.as_bytes(), &Options::default());
    }
    let m = MapText {
        ini: "[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,6,6\nLocalSize=0,0,6,6\n\
              [Structures]\n0=Neutral,HUTX,256,3,5,0\n[Infantry]\n0=Neutral,MANX,256,4,5,0,Guard,64\n"
            .into(),
        terrain: Some(record_bytes(&[rec(3, 5, 1), rec(0, 0, 0)])),
        overlay: Some(overlay_stream(255, &[(4, 5, 0), (3, 4, 2)])),
        ..Default::default()
    };
    let text = m.text().into_bytes();
    let mut seed = 99u32;
    for _ in 0..60 {
        let mut t = text.clone();
        for _ in 0..6 {
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
            let i = (seed as usize >> 4) % t.len();
            t[i] = (seed >> 9) as u8;
        }
        let _ = build_text(&t, &Options::default());
    }
}

/// spec: S-0105 claims 32, 34 and S2 (the depth-buffered mode is off between draws, drawn
/// frames and games: shadows never turn it on and a depth-option part's draw ends with it off;
/// lead-confirmed departure), claim 26 (our engine empties the image caches when a game starts);
/// S-0106 claim 12 (no shadow enables depth mode)
#[test]
fn voxel_canvas_carries_over_between_games() {
    use glasnost_render::voxel::{Mat34, VoxelCanvas, VoxelModel, VoxelQuirks};
    let q = VoxelQuirks::default();
    let model = VoxelModel::new(&glasnost_fmt::vxl::synthetic_model(), None, &q);
    let canvas = VoxelCanvas::new();
    // A shadow (of any type) and a depth-option part leave the mode off.
    canvas
        .draw_shadow(&model, 0, &Mat34::IDENTITY, 0, false, &q)
        .unwrap();
    assert!(!canvas.depth_mode());
    canvas
        .draw_part(&model, 0, &Mat34::IDENTITY, None, true, &q)
        .unwrap();
    assert!(!canvas.depth_mode());
    let (import, catalogue, assets) = context();
    let text = scene_map_text();
    let doc = Document::parse(&text, Quirks::default(), &mut Vec::new());
    let map = decode(&doc, &DecodeContext::default()).map;
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = super::build::Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: None,
        cells: None,
    };
    // The next game's scene continues with that canvas: the mode is still off.
    let (_, painter) = super::build::build_parts_with(&inputs, false, canvas);
    assert!(!painter.canvas().depth_mode());
    // A program start has it off too.
    let (_, fresh) = super::build::build_parts(&inputs, false);
    assert!(!fresh.canvas().depth_mode());
}

/// spec: S-0102 claims 10, 13, 16 and 23 (an animated terrain object is drawn in the object pass
/// as a reader, with image frame 0, its stage at placement, whatever its health; its sort key is
/// X + Y of its cell's centre), claim 30 (its body ranks right before its own shadow), S-0009
/// claim 43 (terrain objects enter the drawing order first)
#[test]
fn animated_terrain_object_in_the_object_pass() {
    let rules = format!(
        "{RULES}[TerrainTypes]\n0=TREEX\n1=MINEX\n2=ANIMX\n[ANIMX]\nIsAnimated=yes\nStrength=1\n[TREEX]\nStrength=1\n"
    );
    let mut ctx = context_with(&rules);
    ctx.0
        .arts
        .insert((Kind::TerrainObjects, 2), art_of("ANIMX", "animx.shp"));
    ctx.2.insert(b"animx.shp", shp((10, 10), 4, (2, 2), (4, 4)));
    // The animated object on (6, 9) and a vehicle on the same cell (equal sort keys).
    let (b, _) = build_in(
        ctx,
        b"[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,10,10\nLocalSize=0,0,10,10\n\
          [Terrain]\n9006=ANIMX\n10006=TREEX\n[Units]\n0=Neutral,CARX,256,6,9,64\n",
        &Options::default(),
    );
    let at = |i: &Item| b.scene.items.iter().position(|x| x == i).unwrap();
    let items = items_of(&b, "terrain object");
    let of = |name: &str| -> Vec<Item> {
        items
            .iter()
            .filter(|i| b.objects[i.object.unwrap() as usize].type_name == name)
            .copied()
            .collect()
    };
    let (anim, tree) = (of("ANIMX"), of("TREEX"));
    // The tree's shadow frame 1 + ⌊2/2⌋ lies beyond its sprite (S-0112 claim 9): body only.
    assert_eq!((anim.len(), tree.len()), (2, 1));
    // The tree stays in the cell pass as a writer, damaged frame 1 at strength 1 (pixel 2).
    assert!(tree.iter().all(|i| at(i) < b.scene.cell_pass_len));
    assert!(tree.iter().all(|i| i.test == DepthTest::Writer));
    assert_eq!(b.scene.images[tree[0].image as usize].pixels()[0], 2);
    // The animated object: object pass, readers, body frame 0 (pixel 1) despite strength 1,
    // shadow frame 0 + ⌊4/2⌋ = 2 (pixel 3), biases −12 upright and −3 flat.
    assert!(anim.iter().all(|i| at(i) >= b.scene.cell_pass_len));
    assert!(anim.iter().all(|i| i.test == DepthTest::Reader));
    let body = anim.iter().find(|i| i.paint != Paint::Shadow).unwrap();
    let shadow = anim.iter().find(|i| i.paint == Paint::Shadow).unwrap();
    assert_eq!(b.scene.images[body.image as usize].pixels()[0], 1);
    assert_eq!(b.scene.images[shadow.image as usize].pixels()[0], 3);
    assert_eq!(bias(body), (Profile::Upright, -12));
    assert_eq!(bias(shadow), (Profile::Flat, -3));
    // S-0102 claim 30: the body, then its own shadow, with nothing between them.
    assert!(!body.provisional && !shadow.provisional);
    assert_eq!(at(shadow), at(body) + 1);
    // Same key as the vehicle on its cell: the terrain object entered first, so it is drawn first.
    let vehicle = items_of(&b, "vehicle");
    assert!(!vehicle.is_empty());
    assert!(vehicle.iter().all(|v| at(v) > at(shadow)));
}

/// A synthetic shared depth image file: canvas 50 × 40, image frame 0 a
/// 6 × 4 rectangle at (3, 5) with the bytes 0, 64, 65, 66 repeated, image
/// frame 1 a different constant.
fn companion_file() -> Vec<u8> {
    let frame = |values: Vec<u8>| WriteFrame {
        x: 3,
        y: 5,
        image: IndexedImage::new(6, 4, values).unwrap(),
        compression: 1,
    };
    ShpWriter {
        width: 50,
        height: 40,
        frames: vec![
            frame((0..24).map(|i| [0, 64, 65, 66][i % 4]).collect()),
            frame(vec![99; 24]),
        ],
    }
    .write()
}

/// The building items of one neutral HUTX on (6, 10) whose art section
/// holds `hut_art`, with the shared depth image file when `file`.
fn hut_items(hut_art: &str, file: bool) -> (Built, Vec<Item>) {
    let art = format!(
        "[HUTX]\nAnimIdle=1,1,0\n{hut_art}[MANX]\nSequence=MANSEQ\n[MANSEQ]\nReady=3,1,2\n\
         [CARX]\nFacings=8\nWalkFrames=2\n"
    );
    let mut ctx = context_full(RULES, &art);
    if file {
        ctx.2.insert(b"buildngz.sha", companion_file());
    }
    let (b, _) = build_in(
        ctx,
        b"[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,10,10\nLocalSize=0,0,10,10\n\
          [Structures]\n0=Neutral,HUTX,256,6,10,0\n",
        &Options::default(),
    );
    let items = items_of(&b, "building");
    (b, items)
}

/// spec: S-0102 claim 31, F5 (every eligible body uses image frame 0 of the shared file, its bytes
/// decoded as signed adjustments; its stored rectangle begins at (pₓ − Dₓ − c_z, pᵧ − Dᵧ − r_z) with
/// the live constants (198, 446), D = K + m − (30(w − h), 15(w + h − 2)), and the art displacement
/// ZShapePointMove); S-0102 claim 13 (the shadow and bib take none)
#[test]
fn building_body_companion() {
    // The body's draw point: the top corner of cell (6, 10) at level 0.
    let p = glasnost_render::projection::project_lifted(256 * 6, 256 * 10, 0);
    for (hut_art, d) in [
        // 2 × 2, no displacement: D = (198, 446 − 30).
        ("Foundation=2x2\n", (198, 416)),
        // 3 × 2, displacement (17, 23): D = (198 + 17 − 30, 446 + 23 − 45).
        ("Foundation=3x2\nZShapePointMove=17,23\n", (185, 424)),
    ] {
        let (b, items) = hut_items(hut_art, true);
        let body = items.iter().find(|i| i.group == Group::Buildings).unwrap();
        let ItemDepth::Profile {
            profile,
            companion: Some((id, x, y)),
            ..
        } = body.depth
        else {
            panic!("{hut_art}: the body has a companion");
        };
        assert_eq!(profile, Profile::Upright);
        // The stored rectangle's canvas origin (3, 5) moves it left and up.
        assert_eq!((x, y), (p.0 - d.0 - 3, p.1 - d.1 - 5), "{hut_art}");
        // Image frame 0, decoded by F5 (0 → 0, 64 → −1, 65 → 0, 66 → +1); not frame 1.
        let img = &b.scene.signed[id as usize];
        assert_eq!((img.width, img.height), (6, 4));
        assert_eq!(img.values[..4], [0, -1, 0, 1]);
        assert!(!body.provisional);
        for i in items.iter().filter(|i| i.group != Group::Buildings) {
            assert!(matches!(
                i.depth,
                ItemDepth::Profile {
                    companion: None,
                    ..
                }
            ));
            assert!(!i.provisional);
        }
        assert!(!b.notes.iter().any(|n| n.contains("BUILDNGZ")));
    }
}

/// The building items of one neutral HUTX on (6, 10) whose art section
/// holds `hut_art`, with `file` as the shared depth image file (none when
/// `None`), and its body.
fn hut_body_with(hut_art: &str, file: Option<Vec<u8>>) -> (Built, Item) {
    let art = format!("[HUTX]\nAnimIdle=1,1,0\n{hut_art}");
    let mut ctx = context_full(RULES, &art);
    if let Some(f) = file {
        ctx.2.insert(b"buildngz.sha", f);
    }
    let (b, _) = build_in(
        ctx,
        b"[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,10,10\nLocalSize=0,0,10,10\n\
          [Structures]\n0=Neutral,HUTX,256,6,10,0\n",
        &Options::default(),
    );
    let body = items_of(&b, "building")
        .into_iter()
        .find(|i| i.group == Group::Buildings)
        .expect("the body");
    (b, body)
}

/// A hand-written shape file: canvas `canvas`, one image frame per entry of
/// `frames` (rectangle width and height, pixel-data position), then `tail`.
fn raw_shape(canvas: (u16, u16), frames: &[((u16, u16), u32)], tail: &[u8]) -> Vec<u8> {
    let mut v = Vec::new();
    for x in [0, canvas.0, canvas.1, frames.len() as u16] {
        v.extend_from_slice(&x.to_le_bytes());
    }
    for &((w, h), offset) in frames {
        for x in [0u16, 0, w, h] {
            v.extend_from_slice(&x.to_le_bytes());
        }
        // Compression 0 (uncompressed), three raw bytes, colour, reserved.
        v.extend_from_slice(&[0; 12]);
        v.extend_from_slice(&offset.to_le_bytes());
    }
    v.extend_from_slice(tail);
    v
}

/// Whether `body` is drawn with the plain upright profile of S-0102 F2: no
/// companion, an upright writer.
fn plain_upright(body: &Item) -> bool {
    matches!(
        body.depth,
        ItemDepth::Profile {
            profile: Profile::Upright,
            companion: None,
            ..
        }
    ) && body.test == DepthTest::Writer
}

/// spec: S-0102 claim 33 and D42 (our engine's departure: a missing file, zero image frames at any
/// canvas, a truncated or non-shape file, or an undecodable image frame 0 warn naming the shared
/// file and the failure; loading continues and an eligible body uses no companion, keeping its
/// draw point and bias, with the plain upright profile of F2, writing depth; not provisional)
#[test]
fn building_body_unavailable_companion_uses_plain_upright() {
    let header_only = raw_shape((16, 16), &[((16, 16), 32)], &[]);
    let cases: [(&str, Option<Vec<u8>>, &str); 7] = [
        ("missing", None, "is missing"),
        (
            "zero frames, 16 × 16",
            Some(raw_shape((16, 16), &[], &[])),
            "has no image frames",
        ),
        (
            "zero frames, 0 × 16",
            Some(raw_shape((0, 16), &[], &[])),
            "has no image frames",
        ),
        (
            "zero frames, 16 × 0",
            Some(raw_shape((16, 0), &[], &[])),
            "has no image frames",
        ),
        // One frame header, no pixel data after it.
        (
            "header-only",
            Some(header_only[..8 + 24].to_vec()),
            "image frame 0 that cannot be decoded",
        ),
        // A truncated frame table.
        (
            "truncated",
            Some(header_only[..20].to_vec()),
            "cannot be read as a shape file",
        ),
        (
            "non-shape",
            Some(b"NOT A SHAPE FILE".to_vec()),
            "cannot be read as a shape file",
        ),
    ];
    let (_, reference) = hut_body_with("Foundation=2x2\n", Some(companion_file()));
    for (name, file, failure) in cases {
        let (b, body) = hut_body_with("Foundation=2x2\n", file);
        assert!(plain_upright(&body), "{name}");
        assert!(!body.provisional, "{name}");
        assert!(b.scene.signed.is_empty(), "{name}");
        // The draw point and the claim 13 bias are those of a body with a readable companion.
        assert_eq!((body.x, body.y), (reference.x, reference.y), "{name}");
        let bias = |i: &Item| match i.depth {
            ItemDepth::Profile { bias, .. } => bias,
            _ => panic!("a profile"),
        };
        assert_eq!(bias(&body), bias(&reference), "{name}");
        let warning: Vec<&String> = b
            .notes
            .iter()
            .filter(|n| n.contains("BUILDNGZ.SHA"))
            .collect();
        assert_eq!(warning.len(), 1, "{name}: {:?}", b.notes);
        assert!(warning[0].contains(failure), "{name}: {}", warning[0]);
        assert!(warning[0].contains("claim 33"), "{name}");
    }
}

/// spec: S-0102 claim 33 and D42 (a valid, decodable image frame 0 with an empty stored rectangle,
/// 0 × 16 or 16 × 0 on a 16 × 16 canvas with 256 readable zero bytes, keeps its empty coverage:
/// the body samples a companion that covers none of its pixels, so it draws nothing and writes no
/// depth; no warning solely for being empty)
#[test]
fn building_body_empty_companion_rectangle_covers_nothing() {
    for size in [(0, 16), (16, 0)] {
        let file = raw_shape((16, 16), &[(size, 32)], &[0; 256]);
        let (b, body) = hut_body_with("Foundation=2x2\n", Some(file));
        let ItemDepth::Profile {
            companion: Some((id, ..)),
            ..
        } = body.depth
        else {
            panic!("{size:?}: the body keeps its empty companion");
        };
        let img = &b.scene.signed[id as usize];
        assert_eq!(img.width * img.height, 0, "{size:?}");
        assert!(!body.provisional, "{size:?}");
        assert!(!b.notes.iter().any(|n| n.contains("BUILDNGZ")), "{size:?}");
    }
}

/// spec: S-0102 claim 33, Import mapping and D42 (Q-0520: for the shared companion, image frame 0
/// on a 16 × 16 canvas with a 16 × 16 rectangle and pixel-data position 0 is failed input, with no
/// payload and with 256 unused bytes after it, not empty coverage: one warning naming the file and
/// the failure, and the eligible body keeps its draw point and bias and uses no companion, the
/// plain upright profile of F2, writing depth, not provisional)
#[test]
fn building_body_companion_without_pixel_data_uses_plain_upright() {
    let (_, reference) = hut_body_with("Foundation=2x2\n", Some(companion_file()));
    let bias = |i: &Item| match i.depth {
        ItemDepth::Profile { bias, .. } => bias,
        _ => panic!("a profile"),
    };
    for payload in [&[][..], &[0u8; 256][..]] {
        let n = payload.len();
        let file = raw_shape((16, 16), &[((16, 16), 0)], payload);
        let (b, body) = hut_body_with("Foundation=2x2\n", Some(file));
        assert!(plain_upright(&body), "{n}");
        assert!(!body.provisional, "{n}");
        assert!(b.scene.signed.is_empty(), "{n}: not empty coverage");
        assert_eq!((body.x, body.y), (reference.x, reference.y), "{n}");
        assert_eq!(bias(&body), bias(&reference), "{n}");
        let warning: Vec<&String> = b
            .notes
            .iter()
            .filter(|n| n.contains("BUILDNGZ.SHA"))
            .collect();
        assert_eq!(warning.len(), 1, "{n}: {:?}", b.notes);
        assert!(warning[0].contains("pixel-data position 0"), "{n}");
        assert!(warning[0].contains("claim 33"), "{n}");
    }
}

/// The slot-animation test context: HUTX's art starts five slot animations
/// (S-0108 claim 11) and names a special slot that the set-up never starts.
fn slot_context() -> (glasnost_convert::Import, Catalogue, Assets) {
    slot_context_with("")
}

/// The slot-animation test context with `extra` added to SLOTA's art section.
fn slot_context_with(extra: &str) -> (glasnost_convert::Import, Catalogue, Assets) {
    let rules =
        format!("{RULES}[Animations]\n0=SLOTA\n1=SLOTB\n2=SLOTC\n3=SLOTD\n4=SLOTE\n5=SLOTF\n");
    let art = format!(
        "{ART}[HUTX]\nActiveAnim=SLOTA\nActiveAnimX=30\nActiveAnimY=15\nActiveAnimZAdjust=-5\n\
         ActiveAnimYSort=40\nIdleAnim=SLOTB\nActiveAnimTwo=SLOTC\nActiveAnimThree=SLOTD\n\
         ActiveAnimFour=SLOTE\nSpecialAnim=SLOTF\n\
         [SLOTA]\nLayer=ground\nShadow=yes\nStart=1\nYDrawOffset=3\nZAdjust=99\n{extra}\
         [SLOTB]\nFlat=yes\nShadow=yes\n\
         [SLOTC]\nLayer=ground\nDetailLevel=1\n\
         [SLOTD]\nLayer=top\nAltPalette=yes\nShouldUseCellDrawer=no\n\
         [SLOTE]\nLayer=top\nShouldUseCellDrawer=no\n\
         [SLOTF]\nLayer=ground\n"
    );
    let mut ctx = context_full(&rules, &art);
    for (i, name) in ["SLOTA", "SLOTB", "SLOTC", "SLOTD", "SLOTE", "SLOTF"]
        .iter()
        .enumerate()
    {
        let file = format!("{}.shp", name.to_ascii_lowercase());
        ctx.0
            .arts
            .insert((Kind::Animations, i), art_of(name, &file));
        // Canvas 20 × 20, 6 image frames of 2 × 2 at (9, 9); frame i has pixel value i + 1.
        ctx.2
            .insert(file.as_bytes(), shp((20, 20), 6, (2, 2), (9, 9)));
    }
    ctx
}

/// The default options at detail level 0.
fn low_detail() -> Options {
    Options {
        detail: glasnost_render::light::DetailLevel::Low,
        ..Options::default()
    }
}

/// The scene of one neutral HUTX on (6, 10) in the slot context.
fn slot_scene(options: &Options) -> Built {
    build_in(
        slot_context(),
        b"[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,10,10\nLocalSize=0,0,10,10\n\
          [Structures]\n0=Neutral,HUTX,256,6,10,0\n",
        options,
    )
    .0
}

/// spec: S-0108 claim 10 (placement at the anchor moved by the displacement, S-0101 F6);
/// S-0108 claim 11 (the set-up starts idle and active-1 … active-4, not the special slot);
/// S-0102 claim 17 (a slot animation is an unattached animation in its type's layer, a depth
/// reader; the slot's depth adjustment and y-sort replace the type's); S-0102 claim 13 (the
/// animation rows: ordinary, flat, shadow); S-0102 claims 1 and 7 (layer order);
/// S-0109 claims 1 and 10 (first image frame a; shadow frame f + ⌊N/2⌋, none for a flat type;
/// the image before its shadow, the shadow at the unshifted point); S-0109 claim 26 (the image
/// moved down by the vertical draw shift); S-0109 claim 19 (a detail field above the player's
/// level is not drawn)
#[test]
fn slot_animations_of_a_placed_building() {
    let b = slot_scene(&low_detail());
    let slots: Vec<Item> = b.scene.items[b.scene.cell_pass_len..].to_vec();
    let px = |i: &Item| b.scene.images[i.image as usize].pixels()[0];
    // SLOTA (ground: image, shadow), SLOTB (air: image only, flat types draw no shadow),
    // SLOTD and SLOTE (top, in entering order). SLOTC's detail field 1 is above the player's
    // level 0, SLOTF is in a slot the set-up does not start.
    assert_eq!(slots.len(), 5);
    assert!(slots.iter().all(|i| i.test == DepthTest::Reader));
    assert!(slots.iter().all(|i| i.group == Group::Buildings));
    let (a, a_shadow, flat) = (slots[0], slots[1], slots[2]);
    // Anchor (−120, 240); displacement (30, 15) is (256, 0) leptons (F6), so the position
    // (1792, 2560, 0) projects to (−90, 255); the vertical draw shift 3 moves it to (−90, 258).
    // Canvas 20 × 20 centred there, frame at (9, 9).
    assert_eq!((a.x, a.y), (-90 - 10 + 9, 258 - 10 + 9));
    // The shadow stays on the unshifted point (−90, 255).
    assert_eq!((a_shadow.x, a_shadow.y), (a.x, 255 - 10 + 9));
    // Image frame Start = 1 (pixel 2); the shadow frame 1 + ⌊6/2⌋ = 4 (pixel 5).
    assert_eq!((px(&a), px(&a_shadow)), (2, 5));
    assert_eq!(a_shadow.paint, Paint::Shadow);
    // Ordinary row: shift 3 + slot adjustment −5 (the type's 99 is replaced) − L(0) − 2.
    assert_eq!(bias(&a), (Profile::Upright, -4));
    // Shadow row: −2 − L(0), flat.
    assert_eq!(bias(&a_shadow), (Profile::Flat, -2));
    // Flat row: 0 + 0 − L(0) − 3; its image frame 0.
    assert_eq!(bias(&flat), (Profile::Flat, -3));
    assert_eq!(px(&flat), 1);
    // No request is open for these: none is translucent or tiled.
    assert!(slots.iter().all(|i| !i.provisional && i.stack.is_none()));
}

/// spec: S-0007 claim 34 (an animation's playback and drawing properties come exclusively from its own
/// art section, even when its image name names another section; there is no per-property fallback to
/// the image-named section: a conflicting value takes the own one, a missing one its default, never the
/// image section's); S-0108 claim 12 (a damaged slot variant is a separate animation type sharing the
/// image with the normal variant); S-0103 Import mapping (ShouldUseCellDrawer defaults to yes)
#[test]
fn animation_properties_come_from_the_own_art_section() {
    // SLOTA names the image section NORM, which has its own conflicting values.
    let scene = |own: &str| {
        let rules = format!("{RULES}[Animations]\n0=SLOTA\n");
        // One [HUTX] section only: which copy of a repeated section a lookup returns is the
        // original's hash order (S-0002 claim 2).
        let art = format!(
            "{}[SLOTA]\nImage=NORM\nLayer=ground\n{own}\
             [NORM]\nLayer=top\nStart=1\nShadow=yes\nYDrawOffset=9\nTiled=yes\n",
            ART.replacen("[HUTX]\n", "[HUTX]\nActiveAnim=SLOTA\n", 1)
        );
        let mut ctx = context_full(&rules, &art);
        ctx.0
            .arts
            .insert((Kind::Animations, 0), art_of("NORM", "norm.shp"));
        ctx.2.insert(b"norm.shp", shp((20, 20), 6, (2, 2), (9, 9)));
        let b = build_in(
            ctx,
            b"[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,10,10\nLocalSize=0,0,10,10\n\
              [Structures]\n0=Neutral,HUTX,256,6,10,0\n",
            &low_detail(),
        )
        .0;
        let items: Vec<Item> = b.scene.items[b.scene.cell_pass_len..].to_vec();
        let pixels: Vec<u8> = items
            .iter()
            .map(|i| b.scene.images[i.image as usize].pixels()[0])
            .collect();
        (items, pixels)
    };
    // A conflicting value: the own section's start frame 3 (pixel 4), no shadow (the own section
    // has none), not the image section's start frame 1 and shadow.
    let (items, pixels) = scene("Start=3\n");
    assert_eq!(items.len(), 1, "no shadow: only the own section decides");
    assert_eq!(pixels, vec![4]);
    // An omitted own value: the start frame is the default 0 (pixel 1), never the image section's 1.
    let (items, pixels) = scene("");
    assert_eq!(items.len(), 1);
    assert_eq!(pixels, vec![1]);
    // The own section's shadow and displacement are used, the image section's are not.
    let (items, _) = scene("Shadow=yes\nYDrawOffset=3\n");
    assert_eq!(items.len(), 2, "image and its shadow");
    let plain = scene("Shadow=yes\n").0;
    assert_eq!(
        items[0].y,
        plain[0].y + 3,
        "the own section's displacement, not the image section's 9"
    );
}

/// spec: S-0109 claim 20 (a translucency field of 25, 50 or 75 gives code 1, 2 or 3, any other
/// positive value 0); S-0102 claim 32 and F8 (a nonzero code draws the image with the blend, the
/// shadow keeps the plain halving)
#[test]
fn translucent_slot_animation_blends() {
    use glasnost_render::compose::Translucency;
    for (field, code) in [
        (25, Some(Translucency::Quarter)),
        (50, Some(Translucency::Half)),
        (75, Some(Translucency::ThreeQuarter)),
        (40, None),
    ] {
        let b = build_in(
            slot_context_with(&format!("Translucency={field}\n")),
            b"[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,10,10\nLocalSize=0,0,10,10\n\
              [Structures]\n0=Neutral,HUTX,256,6,10,0\n",
            &low_detail(),
        )
        .0;
        let slots: Vec<Item> = b.scene.items[b.scene.cell_pass_len..].to_vec();
        let (a, a_shadow) = (slots[0], slots[1]);
        match (a.paint, code) {
            (Paint::Translucent(_, t), Some(c)) => assert_eq!(t, c, "field {field}"),
            (Paint::Lit(_), None) => {}
            (p, c) => panic!("field {field}: paint {p:?}, expected code {c:?}"),
        }
        assert_eq!(a_shadow.paint, Paint::Shadow);
        assert!(!a.provisional && !a_shadow.provisional);
        assert!(!b.notes.iter().any(|n| n.contains("drawn opaque")));
    }
}

/// spec: S-0102 claims 5 and 17 (the sort key is X + Y of the position plus the slot's y-sort;
/// the slot's values count in the first drawn frame), claim 6
#[test]
fn slot_animation_sort_key() {
    let (import, catalogue, assets) = slot_context();
    let doc = Document::parse(
        b"[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,10,10\nLocalSize=0,0,10,10\n\
          [Structures]\n0=Neutral,HUTX,256,6,10,0\n",
        Quirks::default(),
        &mut Vec::new(),
    );
    let map = decode(
        &doc,
        &DecodeContext {
            catalogue: Some(&catalogue),
            ..Default::default()
        },
    )
    .map;
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = super::build::Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: None,
        cells: None,
    };
    let (built, _) = super::build::build_parts_with(&inputs, true, Default::default());
    // SLOTA: 1792 + 2560 + its slot's y-sort 40; SLOTC: the anchor (1536, 2560) and no y-sort.
    let keys: Vec<i64> = built.ground.iter().map(|g| g.key).collect();
    assert_eq!(keys, [1792 + 2560 + 40, 1536 + 2560]);
    // S-0102 claim 17 (the slot's values count in the first drawn frame, ordered under claim 6):
    // an object whose key lies between SLOTA's X + Y and X + Y + 40 is drawn before SLOTA from
    // the first frame on, whichever entered first.
    use super::live::{Entrant, GroundLayer};
    let mover = Entrant::Object(glasnost_sim::world::ObjectId(1));
    for mover_first in [true, false] {
        let mut g = GroundLayer::default();
        let mut members = vec![(mover, 1792 + 2560 + 20)];
        members.extend(
            built
                .ground
                .iter()
                .enumerate()
                .map(|(i, e)| (Entrant::Static(i), e.key)),
        );
        if !mover_first {
            members.rotate_left(1);
        }
        g.update(&members);
        assert_eq!(
            g.order().collect::<Vec<_>>(),
            [Entrant::Static(1), mover, Entrant::Static(0)]
        );
    }
    // SLOTB (air) and SLOTD, SLOTE (top) are kept apart, in entering order.
    let layers: Vec<_> = built.layered.iter().map(|(l, _)| *l).collect();
    use super::slots::Layer;
    assert_eq!(layers, [Layer::Air, Layer::Top, Layer::Top]);
}

/// spec: S-0103 claim 8 (row 3: drawn with its building, the owner's 53-shade palette at the
/// building's brightness; row 4: the alternative palette, one row; row 5: the anim palette's
/// 53-row untinted table)
#[test]
fn slot_animation_palettes() {
    let b = slot_scene(&low_detail());
    let body = items_of(&b, "building")[0];
    let slots: Vec<Item> = b.scene.items[b.scene.cell_pass_len..].to_vec();
    let lit = |i: &Item| match i.paint {
        Paint::Lit(s) => s,
        p => panic!("not lit: {p:?}"),
    };
    // SLOTA is drawn with its building: the body's own table and brightness.
    assert_eq!(lit(&slots[0]), lit(&body));
    // SLOTD: the single-shade table of the first colour definition.
    assert_eq!(lit(&slots[3]).rows, 1);
    // SLOTE: the anim palette's table, 53 rows, not the house table.
    assert_eq!(lit(&slots[4]).rows, 53);
    assert_ne!(lit(&slots[4]).base, lit(&body).base);
}

/// spec: S-0109 claim 19 (a type whose detail field is 1 is drawn at level 2, not at level 0)
#[test]
fn slot_animation_detail_level() {
    let high = slot_scene(&Options::default());
    let low = slot_scene(&low_detail());
    let n = |b: &Built| b.scene.items.len() - b.scene.cell_pass_len;
    assert_eq!((n(&high), n(&low)), (6, 5));
}

/// A synthetic SHP whose frame i is a column of `heights[i]` rows of pixel
/// value i + 1, at (9, `tops[i]`) on a 20 × `canvas_height` canvas.
fn shp_rects(canvas_height: u16, heights: &[u32], tops: &[u16]) -> Vec<u8> {
    ShpWriter {
        width: 20,
        height: canvas_height,
        frames: heights
            .iter()
            .zip(tops)
            .enumerate()
            .map(|(i, (&h, &y))| WriteFrame {
                x: 9,
                y,
                image: IndexedImage::new(u32::from(h > 0), h, vec![(i + 1) as u8; h as usize])
                    .unwrap(),
                compression: 1,
            })
            .collect(),
    }
    .write()
}

/// The slot items of one neutral HUTX on (6, 10) whose active slot
/// (displacement (30, 15), so its position projects to (−90, 255) as in
/// [`slot_animations_of_a_placed_building`]) plays SLOTA, a ground-layer type
/// with the art lines `section` and the sprite `sprite`.
fn one_slot(section: &str, sprite: Vec<u8>) -> (Built, Vec<Item>) {
    let rules = format!("{RULES}[Animations]\n0=SLOTA\n");
    let art = format!(
        "{ART}[HUTX]\nActiveAnim=SLOTA\nActiveAnimX=30\nActiveAnimY=15\n\
         [SLOTA]\nLayer=ground\n{section}"
    );
    let mut ctx = context_full(&rules, &art);
    ctx.0
        .arts
        .insert((Kind::Animations, 0), art_of("SLOTA", "slota.shp"));
    ctx.2.insert(b"slota.shp", sprite);
    let b = build_in(
        ctx,
        b"[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,10,10\nLocalSize=0,0,10,10\n\
          [Structures]\n0=Neutral,HUTX,256,6,10,0\n",
        &Options::default(),
    )
    .0;
    let items = b.scene.items[b.scene.cell_pass_len..].to_vec();
    (b, items)
}

/// spec: S-0109 claim 26 (an ordinary or flat animation's image is centred on its projected
/// position moved down by its vertical draw shift v: up for a negative v; the slot displacement
/// stays in the position, S-0108 claim 10); S-0109 claim 10 (a shadow type's shadow is drawn
/// right after its image, at the unshifted point, without v; a flat type draws none);
/// S-0102 claim 13 (the shift enters the image's bias, not the shadow's)
#[test]
fn slot_animation_draw_shift_and_shadow() {
    let sprite = || shp((20, 20), 6, (2, 2), (9, 9));
    for v in [0, 7, -7] {
        let (_, items) = one_slot(&format!("Shadow=yes\nYDrawOffset={v}\n"), sprite());
        let [image, shadow] = items[..] else {
            panic!("image and shadow: {items:?}")
        };
        assert!(matches!(image.paint, Paint::Lit(_)));
        assert_eq!(
            shadow.paint,
            Paint::Shadow,
            "the shadow is drawn after the image"
        );
        assert_eq!((image.x, image.y), (-90 - 10 + 9, 255 + v - 10 + 9));
        assert_eq!((shadow.x, shadow.y), (-90 - 10 + 9, 255 - 10 + 9));
        assert_eq!(bias(&image), (Profile::Upright, v - 2));
        assert_eq!(bias(&shadow), (Profile::Flat, -2));
    }
    // A flat shadow type: claim 26's image point, no shadow image frame.
    let (_, items) = one_slot("Shadow=yes\nFlat=yes\nYDrawOffset=-7\n", sprite());
    assert_eq!(items.len(), 1);
    assert_eq!((items[0].x, items[0].y), (-91, 255 - 7 - 1));
    assert_eq!(bias(&items[0]), (Profile::Flat, -7 - 3));
}

/// spec: S-0109 claim 27 (a tiled type: copies of its current image frame spaced by the stored
/// rectangle height h of image frame 0, not the canvas or the current frame; the lowest copy
/// centred at (pₓ, pᵧ − ⌊h/2⌋ + v); the count from the unshifted pᵧ − ⌊h/2⌋; no shadow image
/// frame for a shadow type); S-0102 claim 13 (the tiled row: v + adjustment − L(z) − 50 − j·h)
#[test]
fn slot_animation_tiled_copies() {
    for (h, v) in [(10, 0), (11, 0), (10, 7)] {
        // Canvas 32 rows; frame 0 is h rows, the shown frame 1 (Start=1) is 14 rows at row 9.
        let sprite = shp_rects(32, &[h, 14, h, 14], &[11, 9, 11, 9]);
        let (b, items) = one_slot(
            &format!("Tiled=yes\nShadow=yes\nStart=1\nYDrawOffset={v}\n"),
            sprite,
        );
        assert_eq!(items.len(), 1, "one stack, no shadow image frame");
        let it = items[0];
        let h = h as i32;
        // Image frame 1 (pixel 2), centred at (−90, 255 − ⌊h/2⌋ + v) on the 20 × 32 canvas.
        assert_eq!(b.scene.images[it.image as usize].pixels()[0], 2);
        assert_eq!((it.x, it.y), (-90 - 10 + 9, 255 - h / 2 + v - 16 + 9));
        let stack = it.stack.expect("a stack");
        assert_eq!((stack.spacing, stack.base_row), (h, 255 - h / 2));
        assert_eq!(bias(&it), (Profile::Upright, v - 50));
        // The view's top row at world row 0: C = ⌊(255 − ⌊h/2⌋) / h⌋ + 2, whatever v.
        assert_eq!(stack.copies(0), (255 - h / 2) / h + 2);
        let ys: Vec<i32> = it.copies(0).map(|c| c.y).take(3).collect();
        assert_eq!(ys, [it.y, it.y - h, it.y - 2 * h]);
        assert!(!it.provisional);
    }
}

/// spec: S-0103 claim 8 (tiled copies all use the anim palette, even when the ordinary selection is
/// the owner's or the alternative palette, and keep the selected brightness: an own animation's
/// building brightness, the cell's unscaled ground level for an alternative-palette type, 1000
/// with the normal-light flag)
#[test]
fn slot_animation_tiled_palette() {
    let lit = |i: &Item| match i.paint {
        Paint::Lit(s) => s,
        p => panic!("not lit: {p:?}"),
    };
    let sprite = || shp((20, 20), 6, (2, 2), (9, 9));
    // Drawn with its building (row 3 when untiled): the owner's table at the building's brightness.
    let (b, items) = one_slot("", sprite());
    let body = lit(&items_of(&b, "building")[0]);
    assert_eq!(lit(&items[0]), body);
    // Tiled: the anim palette's 53 rows, not the house table, at the building's brightness.
    let (b, items) = one_slot("Tiled=yes\n", sprite());
    let body = lit(&items_of(&b, "building")[0]);
    let tiled = lit(&items[0]);
    assert_eq!((tiled.rows, tiled.rho), (53, body.rho));
    assert_ne!(tiled.base, body.base);
    // The same anim table as an untiled type of row 5.
    let (_, row5) = one_slot("ShouldUseCellDrawer=no\n", sprite());
    let row5 = lit(&row5[0]);
    assert_eq!(tiled.rows, row5.rows);
    // An alternative-palette type (row 4, one shade row untiled) tiled: the anim table at its
    // cell's unscaled ground level, as the row-5 type there.
    let (_, alt) = one_slot("ShouldUseCellDrawer=no\nAltPalette=yes\n", sprite());
    assert_eq!(lit(&alt[0]).rows, 1);
    let (_, alt) = one_slot(
        "ShouldUseCellDrawer=no\nAltPalette=yes\nTiled=yes\n",
        sprite(),
    );
    assert_eq!((lit(&alt[0]).rows, lit(&alt[0]).rho), (53, row5.rho));
    // The normal-light flag: brightness 1000 for the copies too.
    let (_, normal) = one_slot("Tiled=yes\nUseNormalLight=yes\n", sprite());
    let neutral = glasnost_render::compose::Shade::new(0, 53, 1000).rho;
    assert_eq!((lit(&normal[0]).rows, lit(&normal[0]).rho), (53, neutral));
}

/// spec: S-0103 claim 8 (an untiled row-4 animation has only one shade row, so the normal-light
/// flag cannot change its pixels)
#[test]
fn untiled_alt_palette_ignores_normal_light() {
    use glasnost_render::compose::{ComposeQuirks, DEPTH_REFERENCE};
    use glasnost_render::scene::{Filter, View};
    let sprite = || shp((20, 20), 6, (2, 2), (9, 9));
    let (plain_b, plain) = one_slot("ShouldUseCellDrawer=no\nAltPalette=yes\n", sprite());
    let (normal_b, normal) = one_slot(
        "ShouldUseCellDrawer=no\nAltPalette=yes\nUseNormalLight=yes\n",
        sprite(),
    );
    let it = plain[0];
    assert!(matches!(it.paint, Paint::Lit(s) if s.rows == 1));
    let view = View {
        origin: (it.x - 4, it.y - 4),
        size: (12, 12),
        reference: DEPTH_REFERENCE,
    };
    // Every light-alpha value over the drawing: the single row is taken whatever the brightness.
    let alpha: Vec<u8> = (0..144).map(|i| (i * 7 % 256) as u8).collect();
    let draw = |b: &Built| {
        b.scene
            .compose_lit(
                &view,
                &Filter::default(),
                &ComposeQuirks::default(),
                Some(alpha.clone()),
            )
            .to_rgba()
    };
    assert_eq!((normal[0].x, normal[0].y), (it.x, it.y));
    assert_eq!(draw(&plain_b), draw(&normal_b));
}

/// spec: S-0109 claim 27 (a zero-height first rectangle: one copy at (pₓ, pᵧ + v) and a warning
/// naming the sprite, our engine's safe departure)
#[test]
fn slot_animation_tiled_zero_height() {
    let sprite = shp_rects(32, &[0, 14], &[0, 9]);
    let (b, items) = one_slot("Tiled=yes\nStart=1\nYDrawOffset=7\n", sprite);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].stack, None);
    assert_eq!((items[0].x, items[0].y), (-91, 255 + 7 - 16 + 9));
    assert!(
        b.notes
            .iter()
            .any(|n| n.contains("slota.shp") && n.contains("zero-height"))
    );
}

/// The map owners of `map_text` over the synthetic rules with `countries` added.
fn map_owners(
    countries: &str,
    map_text: &str,
) -> (super::build::MapOwners, glasnost_convert::Import) {
    let rules = RULES.replace(
        "LightGrey=0,0,150\n",
        &format!("LightGrey=0,0,150\n{countries}"),
    );
    let import = convert(
        &Inputs {
            base: input("rules.ini", &rules),
            map: input("test.map", map_text),
            art: input("art.ini", ART),
            ..Default::default()
        },
        &ImportOptions::default(),
    );
    let doc = Document::parse(map_text.as_bytes(), Quirks::default(), &mut Vec::new());
    let map = decode(&doc, &DecodeContext::default()).map;
    (super::build::map_owner_colours(&map, &import, &doc), import)
}

/// The map text of a 10 × 10 map with `sections` and one vehicle per owner.
fn owners_map(sections: &str, owners: &[&str]) -> String {
    let units: String = owners
        .iter()
        .enumerate()
        .map(|(i, o)| format!("{i}={o},CARX,256,6,{},64\n", i + 1))
        .collect();
    format!(
        "[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,10,10\nLocalSize=0,0,10,10\n{sections}[Units]\n{units}"
    )
}

/// spec: S-0009 claim 47 (a listed house's country is its own section's selector, not its name:
/// the first 19 bytes of the list value, trimmed; an absent or empty selector uses registration
/// index 0; type name or name field, case ignored, earliest wins; an unknown name creates a
/// default country; reserved selectors are treated as absent, with a warning). spec: S-0009
/// claim 48 (an owner must equal a created house's name exactly; a country-only or unknown owner
/// is unresolved). spec: S-0103 claim 14 (the country's selector through the rules layers, an
/// unknown one keeping position 0; a created country's colour is position 0; a house's own valid
/// selector, case ignored, cut to 31 characters, replaces the inherited colour).
#[test]
fn map_owner_colours_by_s0009_claims_47_48() {
    const LONG: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ01234"; // 31 characters
    let countries = format!(
        "{LONG}=7,7,7\n[Countries]\n1=Americans\n2=Soviets\n3=Yuri\n\
         [Americans]\nColor=c3\n[Soviets]\nName=Reds\nColor=C2\n[Yuri]\nColor=Nope\n"
    );
    let sections = format!(
        "[Houses]\n0=Alpha\n1=Bravo\n2=Charlie\n3=Delta\n4=Americans\n5=Echo\n6=  Foxtrot\n\
         7=ABCDEFGHIJKLMNOPQRSXYZ\n8=Golf\n9=Hotel\n\
         [Alpha]\nCountry=reds\n[Charlie]\nCountry=\n[Delta]\nCountry=Atlantis\n\
         [Americans]\nCountry=YURI\n[Echo]\nCountry=Atlantis\nColor={LONG}EXTRA\n\
         [Foxtrot]\nCountry=<RANDOM>\n[ABCDEFGHIJKLMNOPQRS]\nCountry=Soviets\nColor=12\n\
         [Golf]\nCountry=<Player @ B>\n[Hotel]\nCountry=Yuri\nColor=c6\n"
    );
    let owners = [
        "Alpha",
        "Bravo",
        "Charlie",
        "Delta",
        "Americans",
        "Echo",
        "Foxtrot",
        "ABCDEFGHIJKLMNOPQRS",
        "Golf",
        "Hotel",
        "Soviets",
        "alpha",
        "Nobody",
        "<human player>",
    ];
    let (got, import) = map_owners(&countries, &owners_map(&sections, &owners));
    let c = |n: &str, hsv: [u8; 3]| (n.as_bytes().to_vec(), hsv);
    assert_eq!(
        got.colours,
        [
            // The name field, case ignored.
            c("Alpha", [20, 200, 200]),
            // Absent and empty selectors: country index 0, whose mixed-case selector names C3.
            c("Bravo", [30, 200, 200]),
            c("Charlie", [30, 200, 200]),
            // An unknown name: a created default country, position 0.
            c("Delta", [1, 2, 3]),
            // Named like one country, selecting another, whose unknown selector keeps position 0.
            c("Americans", [1, 2, 3]),
            // The house's own selector, cut to 31 characters, replaces the inheritance.
            c("Echo", [7, 7, 7]),
            // Reserved selectors: as absent.
            c("Foxtrot", [30, 200, 200]),
            // The list value cut to 19 bytes; a numeric own selector names nothing and keeps it.
            c("ABCDEFGHIJKLMNOPQRS", [20, 200, 200]),
            c("Golf", [30, 200, 200]),
            c("Hotel", [60, 200, 200]),
        ]
    );
    // A country-only owner, a case variant of a house and an unknown owner are unresolved; the
    // placeholder is the viewer's.
    let u: Vec<&[u8]> = got.unresolved.iter().map(Vec::as_slice).collect();
    assert_eq!(u, [&b"Soviets"[..], b"alpha", b"Nobody"]);
    assert_eq!(got.warnings.len(), 2);
    let options = Options {
        skirmish: false,
        map_colours: got.colours,
        unresolved_owners: got.unresolved,
        ..Options::default()
    };
    let def = |o: &[u8]| super::build::owner_definition(&import, &options, o);
    assert_eq!(def(b"Hotel"), ([60, 200, 200], false));
    assert_eq!(def(b"Soviets"), ([0, 0, 150], true));
}

/// spec: S-0009 claim 48 (with no house-list entries, an empty-valued line being none, one house
/// per country named by the first 20 bytes of its type name, inheriting its colour subject to its
/// own section's override; owners match those houses exactly). spec: S-0103 claim 14.
#[test]
fn map_owner_colours_without_a_house_list() {
    let countries = "[Countries]\n1=Americans\n2=Soviets\n3=LongCountryNameXYZ0123\n\
                     [Americans]\nColor=C3\n[LongCountryNameXYZ0123]\nColor=C4\n";
    let sections = "[Houses]\n0=\n[LongCountryNameXYZ01]\nColor=C9\n";
    let owners = [
        "Americans",
        "Soviets",
        "LongCountryNameXYZ01",
        "LongCountryNameXYZ0123",
        "americans",
    ];
    let (got, _) = map_owners(countries, &owners_map(sections, &owners));
    let c = |n: &str, hsv: [u8; 3]| (n.as_bytes().to_vec(), hsv);
    assert_eq!(
        got.colours,
        [
            c("Americans", [30, 200, 200]),
            c("Soviets", [1, 2, 3]),
            c("LongCountryNameXYZ01", [90, 200, 200]),
        ]
    );
    let u: Vec<&[u8]> = got.unresolved.iter().map(Vec::as_slice).collect();
    assert_eq!(u, [&b"LongCountryNameXYZ0123"[..], b"americans"]);
}

/// spec: S-0103 claim 10 (a slot picks the definition at position P(s): slot 0 → position 1,
/// slot 1 → position 5); the neutral house shows light grey. spec: S-0103 claim 15 and Formulas,
/// S-0410 claim 8 (the radar colour is ramp index 16 at row 26: max(0, u − 1) kept to the display's
/// 5-6-5 bits and widened with zeros; scenario PN's (0, 255, 255) → (248, 56, 56) and
/// (170, 128, 200) → (112, 116, 192)).
#[test]
fn owner_definition_and_radar_colour() {
    let rules = RULES
        .replace("C1=10,200,200", "C1=0,255,255")
        .replace("C5=50,200,200", "C5=170,128,200");
    let (import, _, _) = context_with(&rules);
    let options = Options {
        slots: vec![(b"Americans".to_vec(), 0), (b"Yuri".to_vec(), 1)],
        ..Options::default()
    };
    let def = |o: &[u8]| super::build::owner_definition(&import, &options, o).0;
    assert_eq!(def(b"Americans"), [0, 255, 255]);
    assert_eq!(def(b"Yuri"), [170, 128, 200]);
    assert_eq!(def(b"Neutral"), [0, 0, 150]);
    let radar = |o: &[u8]| super::build::owner_radar_colour(&import, &options, o, [5, 6, 5]);
    assert_eq!(radar(b"Americans"), [248, 56, 56]);
    assert_eq!(radar(b"Yuri"), [112, 116, 192]);
}

/// Our colour-blind house colours (M8 step C3, our own presentation choice) stand in for a
/// player house's slot definition, for named slots and the human and computer slots alike; map
/// colours, the neutral grey and the unresolved-owner mark stay as they are. spec: S-0103 claim 10
/// (the slot picks the definition the ramp shades)
#[test]
fn colour_blind_colours_replace_slot_definitions_only() {
    use super::build::COLOUR_BLIND_SLOTS;
    let (import, _, _) = context_with(RULES);
    let plain = Options {
        slots: vec![(b"Americans".to_vec(), 0), (b"Yuri".to_vec(), 1)],
        human_slot: 2,
        computer_slot: 3,
        map_colours: vec![(b"Mapped".to_vec(), [1, 2, 3])],
        unresolved_owners: vec![b"Lost".to_vec()],
        ..Options::default()
    };
    let blind = Options {
        colour_blind: true,
        ..plain.clone()
    };
    let def = |o: &Options, h: &[u8]| super::build::owner_definition(&import, o, h);
    assert_eq!(def(&blind, b"Americans"), (COLOUR_BLIND_SLOTS[0], false));
    assert_eq!(def(&blind, b"Yuri"), (COLOUR_BLIND_SLOTS[1], false));
    assert_eq!(
        def(&blind, b"<human player>"),
        (COLOUR_BLIND_SLOTS[2], false)
    );
    assert_eq!(def(&blind, b"Computer"), (COLOUR_BLIND_SLOTS[3], false));
    assert_ne!(def(&plain, b"Americans"), def(&blind, b"Americans"));
    for owner in [&b"Mapped"[..], b"Neutral", b"Lost"] {
        assert_eq!(def(&plain, owner), def(&blind, owner), "{owner:?}");
    }
    // The eight colours are told apart.
    for (i, a) in COLOUR_BLIND_SLOTS.iter().enumerate() {
        for b in &COLOUR_BLIND_SLOTS[i + 1..] {
            assert_ne!(a, b);
        }
    }
}

/// #94: a client game gives the walls the local house's lobby slot, the local house chosen as
/// the session chooses it. spec: S-0112 claim 1 row 4 (walls take the local player's house
/// colour), S-0103 claim 10 (a house's lobby colour slot picks its colour definition); design:
/// render (#94)
#[test]
fn walls_take_the_local_house_slot() {
    use super::build::LocalChoice;
    use glasnost_sim::econ::setup::HouseValues;
    use glasnost_sim::world::HouseId;
    let mut setup = glasnost_sim::testkit::setup(7);
    setup.skirmish.houses = vec![
        HouseValues {
            colour: Some(5),
            ..Default::default()
        },
        HouseValues {
            colour: Some(3),
            ..Default::default()
        },
    ];
    let slot = |setup: &glasnost_sim::world::GameSetup, house, watch| {
        super::load::local_slot(setup, LocalChoice { house, watch })
    };
    // The first person's house, or the house a network game asks for.
    assert_eq!(slot(&setup, None, false), Some(5));
    assert_eq!(slot(&setup, Some(HouseId(1)), false), Some(3));
    // A computer house is only watched; a game played from it does not start.
    setup.houses[0].ai = true;
    assert_eq!(slot(&setup, None, false), Some(3));
    assert_eq!(slot(&setup, Some(HouseId(0)), false), None);
    assert_eq!(slot(&setup, Some(HouseId(0)), true), Some(5));
    // No person's house: watched from the first player house, played from none.
    setup.houses[1].ai = true;
    assert_eq!(slot(&setup, None, true), Some(5));
    assert_eq!(slot(&setup, None, false), None);
    // The session's options carry the choice; the viewer's do not.
    assert_eq!(Options::default().local, None);
}

/// spec: S-0101 claim 3 and F2 (r10: the exact lift is the drawing lift only; the sight
/// centre keeps S-0409 claim 5's floating-point simulation lift), S-0409 F2
#[test]
fn drawing_lift_is_not_the_sight_lift() {
    use glasnost_render::projection::height_lift;
    use glasnost_sim::econ::shroud;
    let sim = |z| shroud::height_lift(z, glasnost_num::Rounding::TowardZero);
    // Level 20: the drawing lift is 300 pixels, the simulation lift 299, so the sight
    // centre moves trunc(299 / 30) = 9 cells, not 10.
    assert_eq!(height_lift(2080), 300);
    assert_eq!(sim(2080), 299);
    assert_eq!(sim(2080) / 30, 9);
    // Whole levels below 20 agree.
    for h in 0..20 {
        assert_eq!(height_lift(104 * h), sim(104 * h), "level {h}");
    }
}

/// spec: S-0105 claim 22 and Import mapping (Q-0470: a model whose part or shadow image would
/// have a copy-rectangle side above 4096 pixels gives one warning, naming the model file, not one
/// per pose, appearance or image role; a model within the limit gives none)
#[test]
fn oversized_voxel_model_warns_once() {
    use glasnost_fmt::bytes::F32Bits;
    use glasnost_fmt::vxl::{VxlWriter, synthetic_model};
    let notes = |stretch: f32| {
        let mut ctx = context();
        let car = ctx.0.arts.get_mut(&(Kind::Vehicles, 0)).unwrap();
        car.voxels.insert("body".into(), b"carx.vxl".to_vec());
        let mut model = synthetic_model();
        // The body section stretched along x.
        model.limbs[0].max[0] = F32Bits::new(stretch);
        ctx.2
            .insert(b"carx.vxl", VxlWriter { model: &model }.write());
        // Two cars facing differently: two body images and two shadows of the same model.
        let (b, _) = build_in(
            ctx,
            b"[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,10,10\nLocalSize=0,0,10,10\n\
              [Units]\n0=Neutral,CARX,256,6,9,64\n1=Neutral,CARX,256,7,9,192\n",
            &Options::default(),
        );
        assert!(!items_of(&b, "vehicle").is_empty());
        b.notes
            .into_iter()
            .filter(|n| n.contains("voxel model"))
            .collect::<Vec<_>>()
    };
    let warned = notes(1.0e6);
    assert_eq!(warned.len(), 1, "{warned:?}");
    assert!(warned[0].contains("carx.vxl"), "{warned:?}");
    assert!(notes(5.0).is_empty());
}

/// A live game can create aircraft the map does not hold (production,
/// spawned aircraft and missiles), so their voxel models and motion files
/// are read with the game's files, as for vehicles (S-0102 claim 21: an
/// aircraft is drawn only from its voxel model).
#[test]
fn game_files_hold_aircraft_voxels() {
    let (mut import, _, _) = context();
    import.arts.insert(
        (Kind::Aircraft, 0),
        TypeArt {
            voxels: [("body".to_string(), b"jet.vxl".to_vec())].into(),
            ..Default::default()
        },
    );
    let files = super::build::game_files(&import);
    assert!(files.contains(&b"JET.VXL"[..]));
    assert!(files.contains(&b"JET.HVA"[..]));
}

/// A map that stores one overlay type for all its ore still draws each ore
/// cell with the body its tiberium's run gives the cell position, so the
/// map's files hold every body of every run (M8 round 6 note 38: most ore
/// cells drew nothing, as only the stored type's sprite was read).
///
/// spec: S-0112 claim 4 (the body is the run's type at P + ((x·y) mod F), whatever type the cell
/// stores; the image frame is the data value), Terms (a run: F flat then S sloped positions from P),
/// claim 10 (image number 1: P 102, F 12, S 8)
#[test]
fn a_maps_ore_reads_every_body_of_its_run() {
    // Overlay types 0 … 129; 102 … 121 carry the tiberium flag. Each of 100 … 129 has its own
    // sprite, whose frames are (i − 98) pixels wide, so the drawn body tells its type.
    let mut list = String::from("[OverlayTypes]\n");
    for i in 0..130 {
        list.push_str(&format!("{i}=OV{i}\n"));
    }
    let mut rules = RULES.replace("[OverlayTypes]\n0=OREX\n1=CRATEX\n2=FENCEX\n", &list);
    rules.push_str("[Tiberiums]\n0=RUBIES\n[RUBIES]\nImage=1\n");
    for i in 102..122 {
        rules.push_str(&format!("[OV{i}]\nTiberium=yes\n"));
    }
    let (mut import, catalogue, mut assets) = context_with(&rules);
    let mut pool = Assets::default();
    for i in 100..130usize {
        let file = format!("ov{i}.shp");
        import
            .arts
            .insert((Kind::Overlays, i), art_of(&format!("OV{i}"), &file));
        pool.insert(
            file.as_bytes(),
            shp((40, 10), 24, (i as u32 - 98, 2), (4, 4)),
        );
    }
    // Ore stored as type 102 in cell (5, 9), stage 3: (5 · 9) mod 12 = 9 picks type 111.
    let m = MapText {
        ini: "[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,10,10\nLocalSize=0,0,10,10\n".into(),
        terrain: Some(record_bytes(&[rec(0, 0, 0)])),
        overlay: Some(overlay_stream(255, &[(5, 9, 102)])),
        overlay_data: Some(overlay_stream(0, &[(5, 9, 3)])),
        ..Default::default()
    };
    let doc = Document::parse(m.text().as_bytes(), Quirks::default(), &mut Vec::new());
    let map = decode(
        &doc,
        &DecodeContext {
            catalogue: Some(&catalogue),
            ..Default::default()
        },
    )
    .map;
    let wanted = super::build::wanted_files(&map, &map.cells, &import, &catalogue, &|_| true);
    let ov = |i: usize| format!("OV{i}.SHP").into_bytes();
    for i in 102..122 {
        assert!(wanted.contains(&ov(i)), "the run's type {i}");
    }
    for i in [100, 101, 122, 129] {
        assert!(!wanted.contains(&ov(i)), "type {i} lies outside the run");
    }
    // Built from the files read, the cell draws type 111's frame 3.
    for name in &wanted {
        if let Some(b) = pool.get(name) {
            assets.insert(name, b.to_vec());
        }
    }
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let b = build(
        &map,
        &import,
        &catalogue,
        &palettes,
        &assets,
        &Options::default(),
    );
    let bodies: Vec<Item> = items_of(&b, "overlay")
        .into_iter()
        .filter(|i| i.paint != Paint::Shadow)
        .collect();
    assert_eq!(bodies.len(), 1);
    let image = &b.scene.images[bodies[0].image as usize];
    assert_eq!(image.width(), 111 - 98);
    assert_eq!(image.pixels()[0], 4);
}

/// A crate placed during play is drawn, though a skirmish map's own crates
/// never bring their sprites: a live game reads the crate types' sprites.
///
/// spec: S-0520 claim 3 (play places a crate overlay in a cell and redraws it); S-0009 claim 12
/// (the map's crate overlays never appear in skirmish)
#[test]
fn game_files_hold_crate_sprites() {
    let (import, _, _) = context();
    let files = super::build::game_files(&import);
    assert!(files.contains(&b"CRATEX.SHP"[..]));
    assert!(!files.contains(&b"FENCEX.SHP"[..]));
}
