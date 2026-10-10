//! Tests of the map decoder on synthetic maps.

use glasnost_ini::{Document, Quirks};
use glasnost_map::{CellXY, Fill, ListKind, NO_OVERLAY, NO_TILE, Theater};

use super::synth::{MapText, Record, overlay_stream, record_bytes};
use super::*;

fn decode_text(text: &str) -> Decoded {
    let doc = Document::parse(text.as_bytes(), Quirks::default(), &mut Vec::new());
    decode(&doc, &DecodeContext::default())
}

fn rec(x: u16, y: u16, tile: i32, level: u8) -> Record {
    Record {
        x,
        y,
        tile,
        sub_tile: 0,
        level,
        ice: 0,
    }
}

const BASE: &str =
    "[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,10,6\nLocalSize=1,2,8,3\nTheater=snow\n";

/// BASE with extra lines in its `[Map]` section.
fn base_map(extra: &str) -> String {
    format!("{BASE}{extra}")
}

/// spec: S-0009 claims 1, 7 (records, duplicates, non-legal cells, terminator)
#[test]
fn terrain_records() {
    let mut m = MapText {
        ini: base_map("Level=1\n"),
        ..Default::default()
    };
    m.terrain = Some(record_bytes(&[
        rec(1, 10, 5, 3),
        rec(1, 10, 6, 4), // later duplicate wins
        rec(1, 9, 7, 2),  // not legal
        rec(0, 0, 0, 0),  // terminator
        rec(2, 10, 9, 9), // after the terminator: no effect
    ]));
    let d = decode_text(&m.text());
    let map = &d.map;
    assert_eq!(map.theater, Theater::Snow);
    assert_eq!(map.footprint.local, [1, 2, 8, 3]);
    let c = map.cells.get(CellXY::new(1, 10)).unwrap();
    assert_eq!((c.tile, c.level), (6, 4));
    // spec: S-0009 claim 6 (a fresh start: the clear base tile is 0, the level L0)
    let after = map.cells.get(CellXY::new(2, 10)).unwrap();
    assert_eq!((after.tile, after.level), (0, 1));
    assert!(map.cells.get(CellXY::new(1, 9)).is_none());
    assert!(d.warnings.is_empty(), "{:?}", d.warnings);
}

/// spec: S-0009 claim 7 (cut-off records: table rows)
#[test]
fn cut_off_records() {
    let full = record_bytes(&[rec(1, 10, 5, 3)]);
    let ini = base_map("Level=1\n");
    // Coordinates only: the tile becomes 65535, the level keeps the fill.
    let m = MapText {
        ini: ini.clone(),
        terrain: Some(full[..4].to_vec()),
        ..Default::default()
    };
    let d = decode_text(&m.text());
    let c = d.map.cells.get(CellXY::new(1, 10)).unwrap();
    assert_eq!((c.tile, c.level), (NO_TILE, 1));
    // Up to the tile number: the tile applies, the level keeps the fill.
    let m = MapText {
        ini: ini.clone(),
        terrain: Some(full[..8].to_vec()),
        ..Default::default()
    };
    let c = *decode_text(&m.text())
        .map
        .cells
        .get(CellXY::new(1, 10))
        .unwrap();
    assert_eq!((c.tile, c.level), (5, 1));
    // Part of the tile number: treated as not present, and warned.
    let m = MapText {
        ini,
        terrain: Some(full[..6].to_vec()),
        ..Default::default()
    };
    let d = decode_text(&m.text());
    let c = d.map.cells.get(CellXY::new(1, 10)).unwrap();
    assert_eq!(c.tile, NO_TILE);
    assert!(d.warnings.iter().any(|w| w.text.contains("partway")));
    assert!(d.warnings.iter().any(|w| w.text.contains("(0, 0)")));
}

/// spec: S-0009 claim 7 (negative height level), claim 6 (level modulo 256)
#[test]
fn levels() {
    let m = MapText {
        ini: base_map("Level=258\n"),
        terrain: Some(record_bytes(&[rec(1, 10, 5, 255), rec(0, 0, 0, 0)])),
        ..Default::default()
    };
    let d = decode_text(&m.text());
    assert_eq!(d.map.base_level, 2);
    assert_eq!(d.map.cells.get(CellXY::new(1, 10)).unwrap().level, -1);
    assert_eq!(d.map.cells.get(CellXY::new(5, 10)).unwrap().level, 2);
}

/// spec: S-0009 claim 6 (water fill: one draw per visited cell, case-insensitive)
#[test]
fn water_fill() {
    let m = MapText {
        ini: base_map("Fill=wAtEr\n"),
        ..Default::default()
    };
    let d = decode_text(&m.text());
    assert_eq!(d.map.fill, Fill::Water);
    assert_eq!(d.map.fill_draws, 6 * 19);
}

/// spec: S-0009 claims 11, 12, 13 (overlay streams and format version)
#[test]
fn overlays() {
    let types = overlay_stream(NO_OVERLAY, &[(1, 10, 7), (1, 9, 8)]);
    let data = overlay_stream(0, &[(1, 10, 3), (2, 10, 9)]);
    let m = MapText {
        ini: BASE.into(),
        overlay: Some(types.clone()),
        overlay_data: Some(data.clone()),
        ..Default::default()
    };
    let d = decode_text(&m.text());
    let c = d.map.cells.get(CellXY::new(1, 10)).unwrap();
    assert_eq!((c.overlay, c.overlay_data), (7, 3));
    let c = d.map.cells.get(CellXY::new(2, 10)).unwrap();
    assert_eq!((c.overlay, c.overlay_data), (NO_OVERLAY, 9));
    assert!(d.map.has_overlay_data);
    // Below format version 2: no overlay at all.
    let m = MapText {
        ini: BASE.replace("NewINIFormat=4", "NewINIFormat=1"),
        overlay: Some(types),
        overlay_data: Some(data),
        ..Default::default()
    };
    let d = decode_text(&m.text());
    assert_eq!(
        d.map.cells.get(CellXY::new(1, 10)).unwrap().overlay,
        NO_OVERLAY
    );
    // A short stream: positions past it have no overlay, and a warning.
    let m = MapText {
        ini: BASE.into(),
        overlay: Some(vec![7; 512 * 10]),
        ..Default::default()
    };
    let d = decode_text(&m.text());
    assert_eq!(
        d.map.cells.get(CellXY::new(1, 10)).unwrap().overlay,
        NO_OVERLAY
    );
    assert!(d.warnings.iter().any(|w| w.text.contains("OverlayPack")));
}

/// spec: S-0009 claims 15, 16 (waypoint spelling, 0 undefined, home default)
#[test]
fn waypoints() {
    let m = MapText {
        ini: format!(
            "{BASE}[Waypoints]\n0=10001\n01=10002\n+3=10003\n2=0\n700=4294967295\n702=10005\n"
        ),
        ..Default::default()
    };
    let d = decode_text(&m.text());
    let w = &d.map.waypoints;
    assert_eq!(w.get(&0), Some(&(1, 10)));
    assert!(!w.contains_key(&1) && !w.contains_key(&2) && !w.contains_key(&3));
    // −1 decodes to (−1, 0).
    assert_eq!(w.get(&700), Some(&(-1, 0)));
    // spec: S-0009 claim 16 (home 699 undefined: (c, c) with c = ⌊16 / 2⌋)
    assert_eq!(d.map.home_waypoint, 699);
    assert_eq!(w.get(&699), Some(&(8, 8)));
}

/// spec: S-0009 claim 30 (absent and partial rectangles)
#[test]
fn rectangles() {
    let d = decode_text("[Map]\nSize=0,0,30\n");
    assert_eq!((d.map.footprint.width, d.map.footprint.height), (30, 50));
    assert_eq!(d.map.footprint.local, [0, 0, 0, 0]);
    let d = decode_text("[Map]\nSize=x\nLocalSize=2,4,26\n");
    assert_eq!((d.map.footprint.width, d.map.footprint.height), (50, 50));
    assert_eq!(d.map.footprint.local, [2, 4, 26, 50]);
    let d = decode_text("[Basic]\nName=x\n");
    assert_eq!(d.map.footprint.width, 0);
    assert!(d.warnings.iter().any(|w| w.text.contains("no legal cells")));
}

/// spec: S-0009 claims 24, 25 (preview and header)
#[test]
fn preview_and_header() {
    let m = MapText {
        ini: format!(
            "{BASE}[Preview]\nSize=0,0,2,2\n[Header]\nNumberStartingPoints=2\nWidth=10\nHeight=10\nStartX=1\nStartY=1\nWaypoint1=3,4\n"
        ),
        preview: Some(vec![9; 3 * 3 + 1]),
        ..Default::default()
    };
    let d = decode_text(&m.text());
    let p = d.map.preview.unwrap();
    assert_eq!((p.width, p.height), (2, 2));
    assert_eq!(&p.rgb[..9], &[9; 9]);
    assert_eq!(&p.rgb[9..], &[0; 3]);
    assert_eq!(d.map.header.pairs, vec![Some((3, 4)), Some((0, 0))]);
}

/// spec: S-0104 claim 1 (map values in hundredths and thousandths)
#[test]
fn lighting_precision() {
    let d = decode_text(&format!(
        "{BASE}[Lighting]\nAmbient=0.995\nRed=0.574\nGround=0.0\nLevel=0.012\nNukeAmbientChangeRate=2.9\n"
    ));
    let l = d.map.lighting;
    // Scenario L1: 0.995 → 99, 0.574 → 57.
    assert_eq!(l.normal.ambient, 99);
    assert_eq!(l.normal.tint, [57, 100, 100]);
    assert_eq!((l.normal.ground, l.normal.level), (0, 12));
    assert_eq!(l.nuke_rate, 2);
}

/// spec: S-0009 claim 31 (first tag wins, unknown tags ignored, legacy base)
#[test]
fn cell_tags() {
    let d = decode_text(&format!(
        "{BASE}[Tags]\nT1=a\nT2=b\n[CellTags]\n10001=T1\n010001=T2\n10002=NOPE\n"
    ));
    assert_eq!(d.map.cell_tags.get(&(1, 10)), Some(&b"T1".to_vec()));
    assert_eq!(d.map.cell_tags.len(), 1);
}

/// spec: S-0009 claim 35 (field layout by kind)
#[test]
fn claim_35_field_layout() {
    let d = decode_text(&format!(
        "{BASE}[Units]\n0=Neutral,TANKX,200,3,9,64,Guard,TAGA,1,2,1,5,1,0\n\
         [Infantry]\n0=Neutral,MANX,128,4,9,2,Hunt,32,TAGB,7,8,1,0,1\n\
         [Aircraft]\n0=Neutral,JETX,256,5,9,96,Sleep,TAGC,3,4,1,1\n\
         [Structures]\n0=Neutral,HUTX,256,5,8,16,TAGD,1,0,0,2,3,UPA,UPB,UPC,1,1\n"
    ));
    let o = &d.map.objects;
    let u = &o.units[0].placed;
    assert_eq!((u.health, u.cell, u.facing), (200, CellXY::new(3, 9), 64));
    let uf = o.fields(ListKind::Vehicles, 0);
    assert_eq!(uf.behaviour.as_deref(), Some(&b"Guard"[..]));
    assert_eq!(uf.tag.as_deref(), Some(&b"TAGA"[..]));
    assert_eq!(
        (
            uf.veterancy,
            uf.group,
            uf.bridge,
            uf.follow,
            uf.recruit_no,
            uf.recruit_yes
        ),
        (
            Some(1),
            Some(2),
            Some(true),
            Some(5),
            Some(true),
            Some(false)
        )
    );
    assert_eq!(o.infantry[0].sub_cell, 2);
    assert_eq!(o.infantry[0].placed.facing, 32);
    let inf = o.fields(ListKind::Infantry, 0);
    assert_eq!(inf.behaviour.as_deref(), Some(&b"Hunt"[..]));
    assert_eq!(inf.tag.as_deref(), Some(&b"TAGB"[..]));
    assert_eq!(
        (inf.veterancy, inf.group, inf.bridge),
        (Some(7), Some(8), Some(true))
    );
    let af = o.fields(ListKind::Aircraft, 0);
    assert_eq!(o.aircraft[0].placed.facing, 96);
    assert_eq!((af.recruit_no, af.recruit_yes), (Some(true), Some(true)));
    let s = o.fields(ListKind::Buildings, 0);
    assert_eq!(o.structures[0].placed.facing, 16);
    assert_eq!(s.tag.as_deref(), Some(&b"TAGD"[..]));
    assert_eq!(
        (
            s.ai_sell,
            s.ai_rebuild,
            s.switched_on,
            s.upgrade_count,
            s.searchlight
        ),
        (Some(true), Some(false), Some(false), Some(2), Some(3))
    );
    assert_eq!(s.upgrades[2].as_deref(), Some(&b"UPC"[..]));
    assert_eq!((s.ai_repair, s.nominal), (Some(true), Some(true)));
    assert!(d.warnings.is_empty(), "{:?}", d.warnings);
}

/// spec: S-0009 claim 35 (below format version 4: one cell field, base 128, later fields shifted)
#[test]
fn claim_35_legacy_cell_field() {
    let d = decode_text(
        "[Map]\nSize=0,0,10,6\nLocalSize=1,2,8,3\n[Units]\n0=Neutral,TANKX,256,1283,64,Guard\n\
         [Terrain]\n1283=TREEX\n",
    );
    let o = &d.map.objects;
    // 1283 = 10·128 + 3.
    assert_eq!(o.units[0].placed.cell, CellXY::new(3, 10));
    assert_eq!(o.units[0].placed.facing, 64);
    assert_eq!(
        o.fields(ListKind::Vehicles, 0).behaviour.as_deref(),
        Some(&b"Guard"[..])
    );
    assert_eq!(o.terrain[0].cell, CellXY::new(3, 10));
}

/// spec: S-0009 claim 34 (empty fields collapse, lenient integers, 127 characters, spaces kept)
#[test]
fn claim_34_splitting() {
    let d = decode_text(&format!(
        "{BASE}[Units]\n0=Neutral,TANKX,256,,3,9x,  +64z,Guard\n1=Neutral, TANKX,256,3,9,0\n"
    ));
    let o = &d.map.objects;
    // "A,B,256,,3,…" reads 3 as the cell x; "9x" reads 9; "  +64z" reads 64.
    assert_eq!(o.units[0].placed.cell, CellXY::new(3, 9));
    assert_eq!(o.units[0].placed.facing, 64);
    // Spaces belong to the field: the type is " TANKX".
    assert_eq!(o.units[1].placed.type_name, b" TANKX");
    // Switch off: an empty field keeps its place, and an empty required field makes the entry short.
    let fields = split_entry(b"A,B,,4", false);
    assert_eq!(fields.len(), 4);
    assert_eq!(fields[2], None);
    assert_eq!(split_entry(b"A,B,,4", true).len(), 3);
    // Only the first 127 characters are read.
    let long = format!("A,B,{}", "1".repeat(200));
    let fields = split_entry(long.as_bytes(), true);
    assert_eq!(fields[2].as_ref().map(Vec::len), Some(123));
}

/// spec: S-0009 claims 36 and 38 (short entries kept and marked; non-legal cells warned)
#[test]
fn claims_36_38_short_and_non_legal() {
    let d = decode_text(&format!(
        "{BASE}[Units]\n0=Neutral,TANKX,256,3,9\n1=Neutral\n2=Neutral,TANKX,256,1,1,0\n"
    ));
    let o = &d.map.objects;
    assert_eq!(
        o.units.len(),
        2,
        "an entry without a type field is not kept"
    );
    assert!(o.fields(ListKind::Vehicles, 0).short);
    assert!(!o.fields(ListKind::Vehicles, 1).short);
    assert_eq!(o.units[1].placed.cell, CellXY::new(1, 1));
    assert_eq!(d.warnings.len(), 2, "{:?}", d.warnings);
}

/// spec: S-0009 claim 42 (terrain-object keys and values; smudge fields)
#[test]
fn claim_42_terrain_objects_and_smudges() {
    let d = decode_text(&format!(
        "{BASE}[Terrain]\n10003=TREEX\n10004=<None>\n10005=none\n9=TREEX\n\
         [Smudge]\n0=CRATERX,4,10,0\n1=CRATERX,4,10,1\n2=CRATERX,5,10\n3=CRATERX,6\n4=CRATERX\n"
    ));
    let o = &d.map.objects;
    assert_eq!(o.terrain.len(), 1);
    assert_eq!(o.terrain[0].cell, CellXY::new(3, 10));
    assert_eq!(
        o.smudges.iter().map(|s| s.cell).collect::<Vec<_>>(),
        vec![CellXY::new(4, 10), CellXY::new(5, 10)]
    );
    // The non-legal terrain cell (9, 0) and the smudge without y are warned.
    assert_eq!(d.warnings.len(), 2, "{:?}", d.warnings);
}

/// spec: S-0009 claim 40 (an upgrade count above 3 is warned)
#[test]
fn claim_40_upgrade_count_warned() {
    let d = decode_text(&format!(
        "{BASE}[Structures]\n0=Neutral,HUTX,256,5,8,0,None,0,0,1,4\n"
    ));
    assert_eq!(
        d.map.objects.fields(ListKind::Buildings, 0).upgrade_count,
        Some(4)
    );
    assert_eq!(d.warnings.len(), 1, "{:?}", d.warnings);
}

/// Extreme sizes and positions neither overflow nor take long (S-0009
/// claim 2: no cell beyond coordinate 511 exists).
#[test]
fn extreme_sizes() {
    for size in [
        "0,0,2147483647,2147483647",
        "0,0,100000,100000",
        "0,0,-5,7",
        "0,0,1,2147483647",
        "0,0,2147483647,1",
    ] {
        let m = MapText {
            ini: format!(
                "[Basic]\nNewINIFormat=4\nHomeCell=3\n[Map]\nSize={size}\nLocalSize=2147483647,2147483647,2147483647,2147483647\nFill=Water\n\
                 [Waypoints]\n3=2147483647\n[Structures]\n0=Neutral,X,256,2147483647,-2147483648,0\n"
            ),
            terrain: Some(record_bytes(&[rec(65535, 65535, 1, 1)])),
            ..Default::default()
        };
        let d = decode_text(&m.text());
        assert!(d.map.cells.iter().count() <= 512 * 512);
        let f = d.map.footprint;
        let _ = f.is_usable(CellXY::new(300, 300), 127, true);
        let _ = f.legal_count();
    }
}

/// Hostile input never panics: prefixes and byte flips of a synthetic map.
#[test]
fn hostile_maps_never_panic() {
    let m = MapText {
        ini: format!("{BASE}[Waypoints]\n0=10001\n[Header]\nNumberStartingPoints=99\n"),
        terrain: Some(record_bytes(&[rec(1, 10, 5, 3), rec(2, 10, -7, 200)])),
        overlay: Some(overlay_stream(NO_OVERLAY, &[(1, 10, 7)])),
        preview: Some(vec![1; 30]),
        ..Default::default()
    };
    let text = m.text().into_bytes();
    for cut in (0..text.len()).step_by(7) {
        let doc = Document::parse(&text[..cut], Quirks::default(), &mut Vec::new());
        let _ = decode(&doc, &DecodeContext::default());
    }
    let mut seed = 12345u32;
    for _ in 0..200 {
        let mut t = text.clone();
        for _ in 0..8 {
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
            let i = (seed as usize >> 4) % t.len();
            t[i] = (seed >> 8) as u8;
        }
        let doc = Document::parse(&t, Quirks::default(), &mut Vec::new());
        let _ = decode(&doc, &DecodeContext::default());
    }
}

/// spec: S-0009 claim 7 and Formulas (selected cell: a record with x beyond 511 acts on the cell with the same stream index, quirks.cell_index_alias)
#[test]
fn claim_7_selected_cell() {
    let mut m = MapText {
        ini: base_map(""),
        ..Default::default()
    };
    // (517, 10): index 512·10 + 517 = 5637, the cell (5, 11).
    m.terrain = Some(record_bytes(&[rec(517, 10, 5, 3), rec(0, 0, 0, 0)]));
    let text = m.text();
    let d = decode_text(&text);
    let c = d.map.cells.get(CellXY::new(5, 11)).unwrap();
    assert_eq!((c.tile, c.level), (5, 3));
    assert!(!d.warnings.is_empty(), "the importer warns either way");
    assert_eq!(
        selected_cell(CellXY::new(527, 10), true),
        Some(CellXY::new(15, 11))
    );
    assert_eq!(selected_cell(CellXY::new(-1, 0), true), None);
    // Switch off: such a pair selects no cell.
    let doc = Document::parse(text.as_bytes(), Quirks::default(), &mut Vec::new());
    let ctx = DecodeContext {
        quirks: MapQuirks {
            cell_index_alias: false,
            ..MapQuirks::default()
        },
        ..DecodeContext::default()
    };
    let off = decode(&doc, &ctx);
    let c = off.map.cells.get(CellXY::new(5, 11)).unwrap();
    assert_eq!((c.tile, c.level), (0, 0));
    assert_eq!(selected_cell(CellXY::new(527, 10), false), None);
}

/// spec: S-0009 claims 25 and 29 (all N_h pairs are read; quirks.header_pairs_as_start_marks off: at most 8)
#[test]
fn claim_29_header_pairs_read() {
    let mut pairs = String::new();
    for i in 1..=10 {
        pairs.push_str(&format!("Waypoint{i}={i},{i}\n"));
    }
    let text = format!("{BASE}[Header]\nNumberStartingPoints=10\n{pairs}");
    let d = decode_text(&text);
    assert_eq!(d.map.header.pairs.len(), 10);
    assert_eq!(d.map.header.pairs[9], Some((10, 10)));
    let doc = Document::parse(text.as_bytes(), Quirks::default(), &mut Vec::new());
    let ctx = DecodeContext {
        quirks: MapQuirks {
            header_pairs_as_start_marks: false,
            ..MapQuirks::default()
        },
        ..DecodeContext::default()
    };
    assert_eq!(decode(&doc, &ctx).map.header.pairs.len(), 8);
}
