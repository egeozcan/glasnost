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

const RULES: &str = "[BuildingTypes]\n1=HUTX\n[InfantryTypes]\n1=MANX\n[OverlayTypes]\n0=OREX\n1=CRATEX\n2=FENCEX\n\
    [OREX]\nTiberium=yes\n[CRATEX]\nCrate=yes\n[FENCEX]\nDrawFlat=no\n\
    [Colors]\nC0=1,2,3\nC1=10,200,200\nC2=20,200,200\nC3=30,200,200\nC4=40,200,200\nC5=50,200,200\n\
    C6=60,200,200\nC7=70,200,200\nC8=80,200,200\nC9=90,200,200\nC10=100,200,200\nC11=110,200,200\n\
    C12=120,200,200\nC13=130,200,200\nC14=140,200,200\nLightGrey=0,0,150\n";
const ART: &str =
    "[HUTX]\nFoundation=2x2\nAnimIdle=1,1,0\n[MANX]\nSequence=MANSEQ\n[MANSEQ]\nReady=3,1,2\n";
const THEATER: &str = "[General]\nClearTile=0\n[TileSet0000]\nFileName=clr\nTilesInSet=1\n";

fn shp(canvas: (u16, u16), frames: usize, frame_size: (u32, u32), at: (u16, u16)) -> Vec<u8> {
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

fn context() -> (glasnost_convert::Import, Catalogue, Assets) {
    let mut import = convert(
        &Inputs {
            base: input("rules.ini", RULES),
            art: input("art.ini", ART),
            ..Default::default()
        },
        &ImportOptions::default(),
    );
    let art = |image: &str, sprite: &str| TypeArt {
        image: image.as_bytes().to_vec(),
        sprite: Some(sprite.as_bytes().to_vec()),
        ..Default::default()
    };
    let mut hut = art("HUTX", "hutx.shp");
    hut.building_sprites.insert("bib", b"hutxbib.shp".to_vec());
    import.arts = BTreeMap::from([
        ((Kind::Buildings, 0), hut),
        ((Kind::Infantry, 0), art("MANX", "manx.shp")),
        ((Kind::Overlays, 0), art("OREX", "orex.shp")),
        ((Kind::Overlays, 1), art("CRATEX", "cratex.shp")),
        ((Kind::Overlays, 2), art("FENCEX", "fencex.shp")),
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
    (import, catalogue, assets)
}

/// Builds the scene of `text` in the test context.
fn build_text(text: &[u8], options: &Options) -> (Built, glasnost_map::Map) {
    let (import, catalogue, assets) = context();
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

fn scene(options: &Options) -> (Built, glasnost_map::Map) {
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
    build_text(m.text().as_bytes(), options)
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

/// spec: S-0108 claims 2, 3, 6, 7, 8, 9; S-0102 claims 13, 16 (initial state: Q-0016)
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
    // Idle range starts at 1: body frame 1 (the idle start is provisional, Q-0016; claim 6 gives
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

/// spec: S-0107 claims 2, 3, 19 (stand entry, facing group, shadow frame); S-0102 claim 13
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
    // U = −L(0) − 1 + F; the cell two steps toward the viewer is not high.
    assert_eq!(bias(body), (Profile::Upright, -3));
    assert_eq!(bias(shadow), (Profile::Flat, -3));
    assert_eq!(body.test, DepthTest::Reader);
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
    // Ore shows the frame of its data value 3 (provisional, Q-0012).
    assert_eq!(b.scene.images[items[0].image as usize].pixels()[0], 4);
    // The fence is upright (DrawFlat=no) on level 2: −17 − 30.
    let level = map
        .cells
        .get(glasnost_map::CellXY::new(7, 9))
        .unwrap()
        .level;
    assert_eq!(level, 2);
    assert_eq!(bias(&items[1]), (Profile::Upright, -47));
    assert!(items.iter().all(|i| i.provisional));
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
