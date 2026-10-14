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

const RULES: &str = "[BuildingTypes]\n1=HUTX\n[InfantryTypes]\n1=MANX\n[VehicleTypes]\n1=CARX\n[OverlayTypes]\n0=OREX\n1=CRATEX\n2=FENCEX\n\
    [OREX]\nTiberium=yes\n[CRATEX]\nCrate=yes\n[FENCEX]\nDrawFlat=no\n\
    [SmudgeTypes]\n0=CRATERX\n[CRATERX]\nWidth=2\n[TerrainTypes]\n0=TREEX\n1=MINEX\n[MINEX]\nSpawnsTiberium=yes\n\
    [Colors]\nC0=1,2,3\nC1=10,200,200\nC2=20,200,200\nC3=30,200,200\nC4=40,200,200\nC5=50,200,200\n\
    C6=60,200,200\nC7=70,200,200\nC8=80,200,200\nC9=90,200,200\nC10=100,200,200\nC11=110,200,200\n\
    C12=120,200,200\nC13=130,200,200\nC14=140,200,200\nLightGrey=0,0,150\n";
const ART: &str = "[HUTX]\nFoundation=2x2\nAnimIdle=1,1,0\n[MANX]\nSequence=MANSEQ\n[MANSEQ]\nReady=3,1,2\n\
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
fn art_of(image: &str, sprite: &str) -> TypeArt {
    TypeArt {
        image: image.as_bytes().to_vec(),
        sprite: Some(sprite.as_bytes().to_vec()),
        ..Default::default()
    }
}

/// The test context with the rules text `rules`.
fn context_with(rules: &str) -> (glasnost_convert::Import, Catalogue, Assets) {
    context_full(rules, ART)
}

/// The test context with the rules text `rules` and the art text `art`.
fn context_full(rules: &str, art: &str) -> (glasnost_convert::Import, Catalogue, Assets) {
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

/// Builds `text` with typed rules in which HUTX has strength 100.
fn build_with_rules(text: &[u8]) -> Built {
    build_with_rules_in(context(), text)
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
    let body = |health: i32| {
        let b = build_with_rules(
            format!(
                "[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,10,10\nLocalSize=0,0,10,10\n\
                 [Structures]\n0=Neutral,HUTX,{health},6,10,0\n"
            )
            .as_bytes(),
        );
        let items = items_of(&b, "building");
        b.scene.images[items[0].image as usize].pixels()[0]
    };
    // Idle stage 1 (AnimIdle=1,1,0): frame 1 shows pixel 2. Damaged (h ≤ 0.5): frame 2, pixel 3.
    assert_eq!(body(256), 2);
    assert_eq!(body(128), 3);
    // 130 of 256 gives 50 (truncated), still damaged; 131 gives 51.
    assert_eq!(body(130), 3);
    assert_eq!(body(131), 2);
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
    // Damaged with the absent-key G = 9: frame 10, clamped to ⌊4/2⌋ = 2 (pixel 3); its shadow
    // index 4 is past the sprite, so no shadow.
    let pixels = body(ART, 128);
    assert_eq!(pixels[0].1, 3);
    assert!(pixels.iter().all(|(p, _)| *p != Paint::Shadow));
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

/// spec: S-0105 claim 34 and S2 (the voxel buffer and the depth-buffered mode are kept across
/// games: only the start of the program resets them), claim 26 (our engine empties the image
/// caches when a game starts)
#[test]
fn voxel_canvas_carries_over_between_games() {
    use glasnost_render::voxel::{Mat34, VoxelCanvas, VoxelModel, VoxelQuirks};
    let q = VoxelQuirks::default();
    let model = VoxelModel::new(&glasnost_fmt::vxl::synthetic_model(), None, &q);
    let mut canvas = VoxelCanvas::new();
    // A depth-option type's fresh shadow opens a claim 32 window.
    canvas
        .draw_shadow(&model, 0, &Mat34::IDENTITY, 0, false, true, &q)
        .unwrap();
    assert!(canvas.depth_mode());
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
    // The next game's scene continues with that canvas: the window is still open.
    let (_, painter) = super::build::build_parts_with(&inputs, false, canvas);
    assert!(painter.canvas().depth_mode());
    // A program start has it off.
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
