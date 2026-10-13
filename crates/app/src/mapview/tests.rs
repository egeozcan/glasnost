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
    let shadow = items[1];
    assert_eq!(shadow.paint, Paint::Shadow);
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
        .filter(|i| matches!(i.paint, Paint::Diamond(_)))
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
    let Paint::Diamond(row) = b.scene.items[i].paint else {
        panic!("diamond paint");
    };
    assert_eq!(extra.paint, Paint::Row(row));
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
    use glasnost_rules::sim::BuildingType;
    let (import, catalogue, assets) = context();
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
