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
    // Part of the tile number: S-0012 Formulas F3 (two bytes: B), and warned.
    let m = MapText {
        ini,
        terrain: Some(full[..6].to_vec()),
        ..Default::default()
    };
    let d = decode_text(&m.text());
    let c = d.map.cells.get(CellXY::new(1, 10)).unwrap();
    assert_eq!(c.tile, 5);
    assert!(d.warnings.iter().any(|w| w.text.contains("partway")));
    assert!(d.warnings.iter().any(|w| w.text.contains("(0, 0)")));
}

fn decode_with(text: &str, quirks: MapQuirks) -> Decoded {
    let doc = Document::parse(text.as_bytes(), Quirks::default(), &mut Vec::new());
    decode(
        &doc,
        &DecodeContext {
            quirks,
            ..Default::default()
        },
    )
}

/// spec: S-0012 claim 9, Formulas F3 (partial terrain values, the first record included, quirks.partial_terrain_values)
#[test]
fn s0012_partial_terrain_values() {
    let full = record_bytes(&[rec(1, 10, 0x0304_0506, 3), rec(2, 10, 0x0102, 4)]);
    let ini = base_map("Level=1\n");
    let at = |d: &Decoded, x, y| *d.map.cells.get(CellXY::new(x, y)).unwrap();
    let off = MapQuirks {
        partial_terrain_values: false,
        ..MapQuirks::default()
    };
    // k available tile bytes of the second record: F3's T, translated (no catalogue: kept).
    for (k, want) in [(0usize, 65535), (1, 65280 + 0x02), (2, 0x0102), (3, 0x0102)] {
        let m = MapText {
            ini: ini.clone(),
            terrain: Some(full[..11 + 4 + k].to_vec()),
            ..Default::default()
        };
        let d = decode_text(&m.text());
        let c = at(&d, 2, 10);
        assert_eq!((c.tile, c.level), (want, 1), "k = {k}");
        // the complete first record stays in force
        assert_eq!(at(&d, 1, 10).tile, 0x0304_0506);
        // switch off: a partial tile number is the preset 65535
        let c = at(&decode_with(&m.text(), off), 2, 10);
        assert_eq!(c.tile, 65535, "k = {k} off");
    }
    // S-0012 revision 2: the first record's partial tile number also takes F3
    // when its complete coordinate pair selects a legal cell.
    for (k, want) in [(1usize, 65280 + 0x06), (2, 0x0506), (3, 0x04_0506)] {
        let m = MapText {
            ini: ini.clone(),
            terrain: Some(full[..4 + k].to_vec()),
            ..Default::default()
        };
        let d = decode_text(&m.text());
        assert_eq!(at(&d, 1, 10).tile, want, "first record, k = {k}");
        assert!(d.warnings.iter().any(|w| w.text.contains("partway")));
        let c = at(&decode_with(&m.text(), off), 1, 10);
        assert_eq!(c.tile, 65535, "first record, k = {k} off");
    }
    // A partial coordinate pair after a record: three bytes select (2, 10) with tile 65535.
    let m = MapText {
        ini: ini.clone(),
        terrain: Some(full[..11 + 3].to_vec()),
        ..Default::default()
    };
    let d = decode_text(&m.text());
    assert_eq!(at(&d, 2, 10).tile, NO_TILE);
    assert!(
        d.warnings
            .iter()
            .any(|w| w.text.contains("partial_terrain_values"))
    );
    // Switch off: the partial pair ends the stream, the cell keeps the fill.
    let d = decode_with(&m.text(), off);
    assert_ne!(at(&d, 2, 10).tile, NO_TILE);
    // One or two coordinate bytes leave y = 0: (2, 0) is not touched as (2, 10).
    let m = MapText {
        ini: ini.clone(),
        terrain: Some(full[..11 + 2].to_vec()),
        ..Default::default()
    };
    assert_ne!(at(&decode_text(&m.text()), 2, 10).tile, NO_TILE);
    // Missing one-byte values keep the values in force: sub-tile present, level missing.
    let m = MapText {
        ini: ini.clone(),
        terrain: Some(full[..11 + 9].to_vec()),
        ..Default::default()
    };
    let c = at(&decode_text(&m.text()), 2, 10);
    assert_eq!((c.tile, c.level), (0x0102, 1));
    // Fewer than four bytes for the very first pair: no record from the block, warned.
    let m = MapText {
        ini,
        terrain: Some(full[..3].to_vec()),
        ..Default::default()
    };
    let d = decode_text(&m.text());
    assert!(
        d.warnings
            .iter()
            .any(|w| w.text.contains("first coordinate pair"))
    );
    assert_ne!(at(&d, 1, 10).tile, NO_TILE);
}

/// The text of a terrain block written as `lines` (one entry each, keys 1, 2, …).
fn terrain_text(lines: &[String]) -> String {
    let mut s = base_map("Level=1\n");
    s += "[IsoMapPack5]\n";
    for (i, l) in lines.iter().enumerate() {
        s += &format!("{}={l}\n", i + 1);
    }
    s
}

fn terrain_b64(records: &[Record]) -> String {
    glasnost_fmt::mappack::encode_base64_lines(
        &glasnost_fmt::mappack::pack(&record_bytes(records), PackCodec::Lzo, 0x2000),
        0,
    )
    .concat()
}

/// spec: S-0012 claim 1 (enumeration order, 127-character limit, quirks.map_pack_line_limit)
#[test]
fn s0012_claim_1_joining() {
    let text = terrain_b64(&[rec(1, 10, 5, 3), rec(0, 0, 0, 0)]);
    // Keys are labels: reversed numbering in the same order reads the same.
    let (a, b) = text.split_at(8);
    let mut s = base_map("Level=1\n");
    s += &format!("[IsoMapPack5]\n9={a}\n3={b}\n");
    let d = decode_text(&s);
    assert_eq!(d.map.cells.get(CellXY::new(1, 10)).unwrap().tile, 5);
    // One entry of more than 127 characters: the rest is cut (default), read in full (switch off).
    let recs: Vec<Record> = (1..=12)
        .map(|x| rec(x, 10, i32::from(x) * 7919 + 13, (x * 3) as u8))
        .collect();
    let long = terrain_b64(&recs);
    assert!(long.len() > 127, "{}", long.len());
    let s = terrain_text(std::slice::from_ref(&long));
    let d = decode_text(&s);
    assert!(
        d.warnings
            .iter()
            .any(|w| w.text.contains("map_pack_line_limit"))
    );
    let off = MapQuirks {
        map_pack_line_limit: false,
        ..MapQuirks::default()
    };
    let d = decode_with(&s, off);
    assert_eq!(
        d.map.cells.get(CellXY::new(12, 10)).unwrap().tile,
        12 * 7919 + 13
    );
}

/// spec: S-0012 claim 2 (nonstandard groups, quirks.map_base64_groups)
#[test]
fn s0012_claim_2_nonstandard_groups() {
    let text = terrain_b64(&[rec(1, 10, 5, 3), rec(0, 0, 0, 0)]);
    // A stray character inside the first group: the group is read without it, and warned.
    let mut odd = text.clone();
    odd.insert(2, '*');
    let s = terrain_text(&[odd]);
    let d = decode_text(&s);
    assert!(
        d.warnings
            .iter()
            .any(|w| w.text.contains("map_base64_groups"))
    );
    // Switch off: the block is absent; the fill remains.
    let off = MapQuirks {
        map_base64_groups: false,
        ..MapQuirks::default()
    };
    let d = decode_with(&s, off);
    assert!(
        d.warnings
            .iter()
            .any(|w| w.text.contains("the block is ignored"))
    );
    assert_ne!(d.map.cells.get(CellXY::new(1, 10)).unwrap().tile, 5);
}

/// spec: S-0012 claims 9 and 10 (a bad chunk keeps the earlier chunks' bytes)
#[test]
fn s0012_chunk_prefix() {
    // Two chunks: the first holds the record for (1, 10), the second is cut short.
    let first = record_bytes(&[rec(1, 10, 5, 3)]);
    let mut packed = glasnost_fmt::mappack::pack(&first, PackCodec::Lzo, 0x2000);
    packed.extend_from_slice(&[9, 0, 11, 0, 1]);
    let text = glasnost_fmt::mappack::encode_base64_lines(&packed, 0).concat();
    let d = decode_text(&terrain_text(&[text]));
    assert_eq!(d.map.cells.get(CellXY::new(1, 10)).unwrap().tile, 5);
    assert!(d.warnings.iter().any(|w| w.text.contains("chunk 2")));
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
    assert_eq!(
        d.map.header.pairs,
        vec![Some([Some(3), Some(4)]), Some([Some(0), Some(0)])]
    );
}

/// The map list's facts of a map text, and its full decode.
fn facts_and_decode(text: &str) -> (ListFacts, Decoded) {
    let doc = Document::parse(text.as_bytes(), Quirks::default(), &mut Vec::new());
    (
        list_facts(&doc, MapQuirks::default()),
        decode(&doc, &DecodeContext::default()),
    )
}

/// The map list's lazy preview gives the full decoder's preview, and the
/// size, theater and header the full decoder reads.
/// spec: S-0009 claims 24, 25 and 30 (preview pixels, header, map size), S-0012 claims 8 to 10 (chunks expanded, overlong stream cut)
#[test]
fn list_facts_match_the_full_decode() {
    let pixels: Vec<u8> = (0..4 * 3 * 3).map(|i| (i * 7) as u8).collect();
    for (stream, why) in [
        (pixels.clone(), "exact"),
        (pixels[..20].to_vec(), "short: the rest black"),
        (
            [pixels.clone(), vec![1; 9]].concat(),
            "long: the rest unread",
        ),
    ] {
        let m = MapText {
            ini: format!(
                "{BASE}[Preview]\nSize=0,0,4,3\n[Header]\nNumberStartingPoints=2\nWidth=10\nHeight=10\nStartX=1\nStartY=1\nWaypoint1=3,4\n"
            ),
            preview: Some(stream),
            ..Default::default()
        };
        let (f, d) = facts_and_decode(&m.text());
        let src = f.preview.as_ref().expect("a preview");
        assert_eq!(src.size, (4, 3));
        assert_eq!(Some(src.decode()), d.map.preview, "{why}");
        assert_eq!(f.header, d.map.header);
        assert_eq!(f.size, (10, 6));
        assert_eq!(f.theater, Theater::Snow);
    }
}

/// No usable preview: no size entry (S-0009 claim 30), dimensions that are
/// not positive (S-0012 Edge cases), no preview block (S-0009 Import
/// mapping, `[PreviewPack]` absent: no preview).
#[test]
fn list_facts_without_a_usable_preview() {
    let with = |ini: &str, stream: Option<Vec<u8>>| {
        let m = MapText {
            ini: format!("{BASE}{ini}"),
            preview: stream,
            ..Default::default()
        };
        facts_and_decode(&m.text())
    };
    let (f, d) = with("", Some(vec![9; 12]));
    assert!(
        f.preview.is_none() && d.map.preview.is_none(),
        "no size entry"
    );
    let (f, d) = with("[Preview]\nSize=0,0,0,5\n", Some(vec![9; 12]));
    assert!(f.preview.is_none() && d.map.preview.is_none(), "zero width");
    let (f, _) = with("[Preview]\nSize=0,0,-4,5\n", Some(vec![9; 12]));
    assert!(f.preview.is_none(), "negative width");
    let (f, _) = with("[Preview]\nSize=0,0,2,2\n", None);
    assert!(f.preview.is_none(), "no preview block");
    // No header block: count, origin and extent −1 (S-0009 claim 25).
    assert_eq!(f.header.count, -1);
    // An unknown theater is temperate (S-0009 Edge cases); absent too.
    let (f, _) = facts_and_decode("[Map]\nSize=0,0,7,9\nTheater=nowhere\n");
    assert_eq!((f.theater, f.size), (Theater::Temperate, (7, 9)));
    // spec: S-0009 Edge cases (quirks.absent_rect_is_zero: an absent map size reads as 0,0,0,0) and Constants (map size default 1,1,50,50 for the numbers a value does not supply)
    let (f, _) = facts_and_decode("[Map]\n");
    assert_eq!((f.theater, f.size), (Theater::Temperate, (0, 0)));
    let (f, _) = facts_and_decode("[Map]\nSize=3\n");
    assert_eq!(f.size, (50, 50));
}

/// A preview block whose chunks do not expand still gives the image: no
/// valid chunk leaves every pixel black (S-0012 claims 9 and 10, S-0009
/// claim 24), as the full decoder does.
#[test]
fn a_broken_preview_block_decodes_black() {
    let text = format!("{BASE}[Preview]\nSize=0,0,2,2\n[PreviewPack]\n1=QUJDREVGR0hJSktM\n");
    let (f, d) = facts_and_decode(&text);
    let p = f.preview.expect("a preview size and packed bytes").decode();
    assert_eq!((p.width, p.height), (2, 2));
    assert_eq!(p.rgb, vec![0; 12]);
    assert_eq!(Some(p), d.map.preview);
}

/// The full decoder's preview warnings are unchanged by the split.
/// spec: S-0009 claim 24 (short stream: black rest, warned), S-0012 claim 8 (overlong stream: warned)
#[test]
fn preview_warnings_are_kept() {
    let warned = |stream: Vec<u8>| {
        let m = MapText {
            ini: format!("{BASE}[Preview]\nSize=0,0,2,2\n"),
            preview: Some(stream),
            ..Default::default()
        };
        decode_text(&m.text())
            .warnings
            .into_iter()
            .map(|w| w.text)
            .filter(|t| t.contains("preview"))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        warned(vec![1; 5]),
        ["the preview stream ends before the last pixel; the rest is black (S-0009 claim 24)"]
    );
    assert_eq!(
        warned(vec![1; 13]),
        [
            "the preview stream is longer than its pixels need; the rest is not read (S-0012 claim 8)"
        ]
    );
    assert!(warned(vec![1; 12]).is_empty());
}

/// spec: S-0401 claim 4 and Import mapping ([Basic] TiberiumGrowthEnabled, the map growth switch; absent: on)
#[test]
fn map_growth_switch() {
    let on = decode_text(&format!("{BASE}[Map]\n"));
    assert_eq!(on.map.basic.tiberium_growth, None);
    let off = decode_text(
        "[Basic]\nNewINIFormat=4\nTiberiumGrowthEnabled=no\n[Map]\nSize=0,0,10,6\nLocalSize=1,2,8,3\nTheater=snow\n",
    );
    assert_eq!(off.map.basic.tiberium_growth, Some(false));
}

/// spec: S-0303 Import mapping (the map's special-flags section, `Inert` and `HarvesterImmune`, booleans, absent: no; `DestroyableBridges` is not read, S-0511 claim 1)
#[test]
fn special_flags_inert_and_harvester_immune() {
    let none = decode_text(&format!("{BASE}[Map]\n"));
    assert!(!none.map.basic.inert && !none.map.basic.harvester_immune);
    let both = decode_text(&format!(
        "{BASE}[Map]\n[SpecialFlags]\nInert=yes\nHarvesterImmune=true\nDestroyableBridges=no\n"
    ));
    assert!(both.map.basic.inert && both.map.basic.harvester_immune);
    let one = decode_text(&format!(
        "{BASE}[Map]\n[SpecialFlags]\nHarvesterImmune=yes\n"
    ));
    assert!(!one.map.basic.inert && one.map.basic.harvester_immune);
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
    // spec: S-0009 claim 31 (a value naming no defined tag attaches an empty tag of that name)
    assert_eq!(d.map.cell_tags.get(&(2, 10)), Some(&b"NOPE".to_vec()));
    assert_eq!(d.map.cell_tags.len(), 2);
    let t = &d.map.triggers;
    assert_eq!(t.cell_tags.len(), 2);
    let nope = t.cell_tags[1].1 as usize;
    assert_eq!(t.tags[nope].id, b"NOPE");
    assert!(t.tags[nope].chain.is_empty());
}

/// spec: S-0009 claim 31 (the value <none> or none attaches nothing)
#[test]
fn cell_tag_none_markers() {
    let d = decode_text(&format!("{BASE}[CellTags]\n10001=<None>\n10002=NONE\n"));
    assert!(d.map.cell_tags.is_empty());
    assert!(d.map.triggers.tags.is_empty());
}

/// spec: S-0011 I1, I2, claim 7 and Edge cases (the trigger list, events, actions and tags, resolved)
#[test]
fn triggers_import() {
    let d = decode_text(&format!(
        "{BASE}[Triggers]\n\
         T1=Alphaland,T2,first,0,1,0,1,0\n\
         T2=<none>,<none>,second,0,1,1,1,0\n\
         T3=<random>,UNDEF,third\n\
         [Events]\n\
         T1=2,13,0,4,60,2,3,TANKNAMEISLONGERTHANTWENTYFOUR\n\
         T2=1,51,0,10\n\
         [Actions]\n\
         T1=4,53,2,T3,0,0,0,0,A,99,7,BirdSound,0,0,0,0,AB,22,3,TGG,0,0,0,0,0,54,2,UNDEF,0,0,0,0\n\
         T2=1,54,2,1,0,0,0,0\n\
         [Tags]\n\
         TA=2,Alpha tag,T1\n\
         TGG=0,late,none\n\
         TB=1,short\n"
    ));
    let t = &d.map.triggers;
    // spec: S-0011 claim 7 (the trigger list's IDs, then the created definition UNDEF)
    let ids: Vec<&[u8]> = t.triggers.iter().map(|x| x.id.as_slice()).collect();
    assert_eq!(ids, [&b"T1"[..], b"T2", b"T3", b"UNDEF"]);
    let t1 = &t.triggers[0];
    assert_eq!(
        t1.owner,
        glasnost_map::triggers::Owner::Name(b"Alphaland".to_vec())
    );
    assert_eq!(t1.link, Some(1));
    assert!(!t1.disabled);
    assert_eq!(t1.difficulty, [true, false, true]);
    assert_eq!(t1.events.len(), 2);
    assert_eq!((t1.events[0].code, t1.events[0].value), (13, 4));
    // spec: S-0011 Import mapping (kind 2: the fourth item is the type name, cut to 24 bytes)
    assert_eq!(
        t1.events[1].type_name.as_deref(),
        Some(&b"TANKNAMEISLONGERTHANTWEN"[..])
    );
    // spec: S-0011 Edge cases (a reference shorter than 3 characters is a registration index: no digits, 0)
    assert_eq!(t1.actions[0].trigger, Some(0));
    // spec: S-0011 I2 (a trigger reference by ID, sound names kept, waypoint letters)
    assert_eq!(t1.actions[3].trigger, Some(3));
    assert_eq!(t1.actions[0].waypoint, 0);
    assert_eq!(t1.actions[1].name, b"BirdSound");
    assert_eq!(t1.actions[1].waypoint, 27);
    assert_eq!(t1.actions[2].tag, Some(0));
    let t2 = &t.triggers[1];
    assert_eq!(t2.owner, glasnost_map::triggers::Owner::NoneMarker);
    assert_eq!(t2.link, None);
    // spec: S-0011 Edge cases (a short action reference is a registration index)
    assert_eq!(t2.actions[0].trigger, Some(1));
    // spec: S-0011 Edge cases (missing disabled field: disabled; missing difficulty fields: clear)
    let t3 = &t.triggers[2];
    assert_eq!(t3.owner, glasnost_map::triggers::Owner::None);
    assert!(t3.disabled);
    assert_eq!(t3.difficulty, [false; 3]);
    assert_eq!(t3.link, Some(3));
    // spec: S-0011 Edge cases (a created definition: not disabled, every difficulty flag set)
    assert!(!t.triggers[3].disabled);
    assert_eq!(t.triggers[3].difficulty, [true; 3]);
    // spec: S-0011 claim 7 (the action's tag ID TGG registers first, then the tag list's rest; TB is skipped)
    let tags: Vec<&[u8]> = t.tags.iter().map(|x| x.id.as_slice()).collect();
    assert_eq!(tags, [&b"TGG"[..], b"TA"]);
    let ta = &t.tags[1];
    assert_eq!((ta.repeat, ta.name.as_slice()), (2, &b"Alpha tag"[..]));
    // spec: S-0011 claim 1 (the chain: T1, then its link T2)
    assert_eq!(ta.chain, [0, 1]);
    assert!(t.tags[0].chain.is_empty());
}

/// spec: S-0011 I1 (field 8, the carry-over flag) and I2 (the fourth to seventh action items as integers; S-0524 I1, code 40)
#[test]
fn carry_over_flag_and_action_items() {
    let d = decode_text(&format!(
        "{BASE}[Triggers]\n\
         T1=<none>,<none>,first,0,1,1,1,1\n\
         T2=<none>,<none>,second,0,1,1,1\n\
         [Actions]\n\
         T1=1,40,0,0,3,-2,7,9,A\n"
    ));
    let t = &d.map.triggers;
    assert!(t.triggers[0].carry_over);
    // spec: S-0011 I1 (field 8 absent: clear)
    assert!(!t.triggers[1].carry_over);
    assert_eq!(t.triggers[0].actions[0].items, [3, -2, 7, 9]);
}

/// spec: S-0011 claim 1 (our engine ends a chain that comes back before the repeated member)
#[test]
fn trigger_chain_cycle_is_cut() {
    let d = decode_text(&format!(
        "{BASE}[Triggers]\nA=<none>,B,a,0,1,1,1,0\nB=<none>,A,b,0,1,1,1,0\n[Tags]\nT=0,t,A\n"
    ));
    assert_eq!(d.map.triggers.tags[0].chain, [0, 1]);
    assert!(
        d.warnings
            .iter()
            .any(|w| w.text.contains("chain comes back"))
    );
}

/// spec: S-0009 claim 32 (initial states: set when not 0, no second item leaves the state; numbers outside 0 … 99 have no effect)
#[test]
fn variable_names() {
    let d = decode_text(&format!(
        "{BASE}[VariableNames]\n0=zero,1\n1=one,0\n2=two\n05=five,1\n150=far,1\n"
    ));
    let v = &d.map.triggers.variables;
    assert_eq!(v.len(), 100);
    assert_eq!(&v[..3], &[Some(true), Some(false), None]);
    assert_eq!(v[5], Some(true));
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
        if let Some(p) = list_facts(&doc, MapQuirks::default()).preview {
            let _ = p.decode();
        }
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
    assert_eq!(d.map.header.pairs[9], Some([Some(10), Some(10)]));
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

/// spec: S-0009 claim 29 and Import mapping ([Header] WaypointN: an unread component of a present pair is completed from the value in force, pairs 1 … 8 at import and pairs 9 … 16 from the mark at each phase, with a warning; a missing pair keeps the whole value in force)
#[test]
fn claim_29_unread_header_components() {
    let text = format!(
        "{BASE}[Header]\nNumberStartingPoints=11\nWaypoint1=5\nWaypoint9=-1\nWaypoint10=x\n"
    );
    let d = decode_text(&text);
    let p = &d.map.header.pairs;
    assert_eq!(p[0], Some([Some(5), Some(0)]));
    assert_eq!(p[1], Some([Some(0), Some(0)]));
    assert_eq!(p[8], Some([Some(-1), None]));
    assert_eq!(p[9], Some([None, None]));
    assert_eq!(p[10], None);
    let warned = |k: &str| {
        d.warnings
            .iter()
            .any(|w| w.text.contains(k) && w.text.contains("cannot be read"))
    };
    assert!(warned("Waypoint1:") && warned("Waypoint9:") && warned("Waypoint10:"));
}
