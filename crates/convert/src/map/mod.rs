//! Map decoding (S-0009): a map file read as an INI document becomes a
//! [`glasnost_map::Map`].
//!
//! The byte encodings of the map packs are outside S-0009 (its Questions
//! for the project lead). The block framing and codecs are those of
//! `glasnost_fmt::mappack` (OpenRA and XCC, GPL-3.0-or-later): LZO1X for
//! `IsoMapPack5` and, following XCC's `misc/map_ts_encoder.cpp` (Copyright
//! (C) Olaf van der Spek), for `PreviewPack`; LCW for the two overlay
//! packs. A terrain record is read as the six values of S-0009 claim 7 in
//! that order: x and y as 16-bit, the tile number as a 32-bit signed value,
//! then the sub-tile, the height level and the ice-growth flag as single
//! bytes (the layout `fmt::mappack::IsoTile` documents).
//!
//! The object lists have no spec (Q-0010); they are read by the provisional
//! layout of Q-0010's workaround, after OpenRA's `ImportGen2MapCommand`.

use std::collections::BTreeMap;

use glasnost_fmt::mappack::{PackCodec, decode_base64, unpack};
use glasnost_ini::Document;
use glasnost_ini::binfloat::{self, Binary, Format};
use glasnost_ini::value::{self, Decimal, DecimalField};
use glasnost_map::geometry::{
    CELL_NUMBER_BASE, CELL_TAG_BASE_SWITCH_VERSION, LEGACY_CELL_TAG_BASE, decode_cell_number,
    wrap_coordinate,
};
use glasnost_map::objects::Placed;
use glasnost_map::{
    Aircraft, Basic, CellGrid, CellXY, DEFAULT_HOME_WAYPOINT, Fill, Footprint, Header, Infantry,
    Lighting, LightingSet, Map, NO_OVERLAY, NO_TILE, Objects, Preview, Smudge, Structure,
    TerrainObject, Theater, Unit, WAYPOINT_COUNT,
};

use crate::theater::Catalogue;

/// Quirk switches of S-0009 that decoding applies. Each defaults to the
/// original's behaviour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapQuirks {
    /// `quirks.narrow_map_partial_pass` (claim 3).
    pub narrow_map_partial_pass: bool,
    /// `quirks.absent_rect_is_zero` (claim 30).
    pub absent_rect_is_zero: bool,
    /// `quirks.stale_fill_tiles` (claim 6).
    pub stale_fill_tiles: bool,
    /// `quirks.no_overlay_below_format_2` (claim 11).
    pub no_overlay_below_format_2: bool,
}

impl Default for MapQuirks {
    fn default() -> Self {
        // spec: S-0009 Edge cases and quirks (every switch defaults to the original)
        Self {
            narrow_map_partial_pass: true,
            absent_rect_is_zero: true,
            stale_fill_tiles: true,
            no_overlay_below_format_2: true,
        }
    }
}

/// What decoding needs besides the map.
#[derive(Clone, Debug, Default)]
pub struct DecodeContext<'a> {
    /// The map's theater catalogue, for the tile translation (claim 8) and,
    /// with `quirks.stale_fill_tiles` off, the fill's base tiles.
    pub catalogue: Option<&'a Catalogue>,
    /// The clear and water base tiles of the theater loaded most recently
    /// before this map in the program run (claim 6); `(0, 0)` when none
    /// was (a fresh start, the default).
    pub previous_base: (i32, i32),
    /// Quirk switches.
    pub quirks: MapQuirks,
}

/// An importer warning about the map (glossary, *importer warning*).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapWarning {
    /// Line in the map file, when one applies.
    pub line: Option<u32>,
    /// What was found.
    pub text: String,
}

/// A decoded map with its warnings.
#[derive(Clone, Debug)]
pub struct Decoded {
    /// The map.
    pub map: Map,
    /// Importer warnings.
    pub warnings: Vec<MapWarning>,
}

/// Size of each overlay stream (S-0009 claim 12: 512 × 512 positions).
const OVERLAY_SIDE: usize = 512; // spec: S-0009 Constants (overlay stream size)

/// Minimum format version for overlay (S-0009 claim 11).
const OVERLAY_MIN_VERSION: i32 = 2; // spec: S-0009 Constants (minimum format version for overlay)

/// Map size fallback for numbers a non-empty value does not supply
/// (S-0009 claim 30).
const SIZE_FALLBACK: [i32; 4] = [1, 1, 50, 50]; // spec: S-0009 Constants (map size default)

/// Ranged-draw range size of a water fill: 0 … 3 (S-0009 claim 6).
const WATER_VARIANTS: u32 = 4; // spec: S-0009 Constants (water fill variants)

/// Header pairs that act as start marks: 9 … 16 (S-0009 claim 29); pairs
/// from the seventeenth on are ignored by our engine (a departure).
const LAST_HEADER_PAIR: i32 = 16; // spec: S-0009 Constants (header pairs that act as marks)

/// Header pairs that are preview markers, 1 … 8 (S-0009 claim 25).
const PREVIEW_PAIRS: i32 = 8; // spec: S-0009 claim 25 (markers drawn only when 1 ≤ N_h ≤ 8)

/// Byte widths of the six terrain-record values (module docs; encoding
/// outside S-0009).
const RECORD_FIELDS: [usize; 6] = [2, 2, 4, 1, 1, 1]; // nocite: byte layout from fmt::mappack (OpenRA, XCC)

/// Decodes `doc` as a map.
pub fn decode(doc: &Document, ctx: &DecodeContext<'_>) -> Decoded {
    let mut d = Decoder {
        doc,
        ctx,
        warnings: Vec::new(),
    };
    let map = d.run();
    Decoded {
        map,
        warnings: d.warnings,
    }
}

struct Decoder<'a> {
    doc: &'a Document,
    ctx: &'a DecodeContext<'a>,
    warnings: Vec<MapWarning>,
}

fn show(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

impl Decoder<'_> {
    fn warn(&mut self, line: Option<u32>, text: impl Into<String>) {
        self.warnings.push(MapWarning {
            line,
            text: text.into(),
        });
    }

    fn line_of(&self, section: &[u8], key: &[u8]) -> Option<u32> {
        self.doc.get(section, key).map(|e| e.line)
    }

    fn int(&mut self, section: &[u8], key: &[u8], default: i32) -> i32 {
        match self.doc.value(section, key) {
            None => default,
            // spec: S-0003 claim 3 (a hexadecimal form with no digit keeps the value in force)
            Some(v) => value::integer(v).unwrap_or(default),
        }
    }

    fn boolean(&mut self, section: &[u8], key: &[u8], default: bool) -> bool {
        self.doc
            .value(section, key)
            .and_then(value::boolean)
            .unwrap_or(default)
    }

    fn run(&mut self) -> Map {
        let q = self.ctx.quirks;
        // spec: S-0009 Import mapping ([Basic] NewINIFormat, default 0)
        let format_version = self.int(b"Basic", b"NewINIFormat", 0);
        let theater = self.theater();
        // spec: S-0009 claim 30 ([Map] Size, numbers not supplied come from 1,1,50,50)
        let size = value::rect(
            self.doc.value(b"Map", b"Size"),
            SIZE_FALLBACK,
            q.absent_rect_is_zero,
        );
        let (w, h) = (size[2], size[3]);
        if w <= 0 || h <= 0 {
            self.warn(
                self.line_of(b"Map", b"Size"),
                "the map size gives no legal cells (S-0009 claim 30); our engine does not start a game on it",
            );
        }
        // spec: S-0009 claim 30 ([Map] LocalSize, numbers not supplied come from 0,0,W,H)
        let local = value::rect(
            self.doc.value(b"Map", b"LocalSize"),
            [0, 0, w, h],
            q.absent_rect_is_zero,
        );
        let footprint = Footprint::new(w, h, local);
        if i64::from(w) + i64::from(h) > OVERLAY_SIDE as i64 {
            self.warn(
                self.line_of(b"Map", b"Size"),
                "the map is larger than the supported coordinate range; cells beyond 511 do not exist (S-0009 claim 2)",
            );
        }
        // spec: S-0009 claim 6 (L0 reduced modulo 256 into −128 … 127)
        let base_level = self.int(b"Map", b"Level", 0) as u8 as i8;
        let fill = match self.doc.value(b"Map", b"Fill") {
            // spec: S-0009 claim 6 (the word for water, compared case-insensitively)
            Some(v) if v.eq_ignore_ascii_case(b"Water") => Fill::Water,
            _ => Fill::Clear,
        };
        let mut cells = CellGrid::new(footprint);
        let fill_draws = self.initial_fill(&mut cells, base_level, fill);
        self.terrain(&mut cells);
        let has_overlay_data = self.overlays(&mut cells, format_version);
        let (waypoints, home_waypoint) = self.waypoints(&footprint);
        let header = self.header();
        let preview = self.preview();
        let lighting = self.lighting();
        let basic = self.basic();
        let objects = self.objects(&footprint);
        let cell_tags = self.cell_tags(&footprint, format_version);
        for legacy in [
            &b"IsoMapPack"[..],
            b"IsoMapPack2",
            b"IsoMapPack3",
            b"IsoMapPack4",
        ] {
            if let Some(s) = self.doc.section(legacy) {
                self.warn(
                    Some(s.line()),
                    format!(
                        "legacy terrain stream [{}] is not decoded: its encoding has no spec (S-0009 claim 10)",
                        show(legacy)
                    ),
                );
            }
        }
        Map {
            format_version,
            theater,
            base_level,
            fill,
            footprint,
            cells,
            has_overlay_data,
            waypoints,
            home_waypoint,
            header,
            preview,
            lighting,
            basic,
            objects,
            cell_tags,
            fill_draws,
        }
    }

    fn theater(&mut self) -> Theater {
        let Some(v) = self.doc.value(b"Map", b"Theater") else {
            // spec: S-0009 Import mapping ([Map] Theater, absent or empty: TEMPERATE)
            return Theater::Temperate;
        };
        match glasnost_vfs::theater::Theater::from_identifier(v) {
            Some(t) => to_map_theater(t),
            None => {
                self.warn(
                    self.line_of(b"Map", b"Theater"),
                    format!(
                        "unknown theater {:?}: our engine uses TEMPERATE (S-0009 Edge cases)",
                        show(v)
                    ),
                );
                Theater::Temperate
            }
        }
    }

    /// The initial fill (S-0009 claim 6). Returns the number of draws a
    /// water fill takes from the scenario random stream. The draws' values
    /// need that stream (a planned spec), so every visited cell gets the
    /// base tile plus 0 until it exists.
    fn initial_fill(&mut self, cells: &mut CellGrid, level: i8, fill: Fill) -> u32 {
        let q = self.ctx.quirks;
        // spec: S-0009 claim 6 (quirks.stale_fill_tiles: the previous theater's base tiles, 0 before any)
        let (clear, water) = match (q.stale_fill_tiles, self.ctx.catalogue) {
            (true, _) | (false, None) => self.ctx.previous_base,
            (false, Some(c)) => (c.clear_base, c.water_base),
        };
        let order = cells.visiting_order(q.narrow_map_partial_pass);
        let mut draws = 0u32;
        for c in order {
            let tile = match fill {
                Fill::Clear => clear,
                Fill::Water => {
                    draws = draws.saturating_add(1);
                    water
                }
            };
            if let Some(cell) = cells.get_mut(c) {
                cell.level = level;
                cell.sub_tile = 0;
                cell.tile = tile;
            }
        }
        if draws > 0 {
            self.warn(
                self.line_of(b"Map", b"Fill"),
                format!(
                    "water fill: {draws} cells take a draw over 0 … {} from the scenario random stream, which has no spec yet; they show the water base tile",
                    WATER_VARIANTS - 1
                ),
            );
        }
        draws
    }

    fn pack(&mut self, name: &[u8], codec: PackCodec) -> Option<Vec<u8>> {
        let section = self.doc.section(name)?;
        let line = Some(section.line());
        let values = section.entries().iter().map(|e| e.value.as_slice());
        let joined: Vec<&str> = values.filter_map(|v| std::str::from_utf8(v).ok()).collect();
        let bytes = match decode_base64(joined) {
            Ok(b) => b,
            Err(e) => {
                self.warn(line, format!("[{}]: {e}; the block is ignored", show(name)));
                return None;
            }
        };
        match unpack(&bytes, codec, usize::MAX) {
            Ok(b) => Some(b),
            Err(e) => {
                self.warn(line, format!("[{}]: {e}; the block is ignored", show(name)));
                None
            }
        }
    }

    /// The terrain stream (S-0009 claims 7 and 8).
    fn terrain(&mut self, cells: &mut CellGrid) {
        let Some(data) = self.pack(b"IsoMapPack5", PackCodec::Lzo) else {
            return;
        };
        let line = self.doc.section(b"IsoMapPack5").map(|s| s.line());
        let mut pos = 0usize;
        let mut terminated = false;
        let mut negative_tile = false;
        let mut negative_level = false;
        loop {
            // Read up to six values; `None` once the data runs out.
            let mut vals = [None::<i64>; 6];
            let mut partial = false;
            for (i, width) in RECORD_FIELDS.iter().enumerate() {
                let Some(bytes) = data.get(pos..pos + width) else {
                    partial = pos < data.len();
                    pos = data.len();
                    break;
                };
                pos += width;
                vals[i] = Some(match width {
                    1 => i64::from(bytes[0]),
                    2 => i64::from(u16::from_le_bytes([bytes[0], bytes[1]])),
                    _ => i64::from(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])),
                });
            }
            if partial {
                // spec: S-0009 claim 7 (part of a value: our engine treats it as not present, and warns)
                self.warn(
                    line,
                    "the terrain stream ends partway through a value; that value is treated as not present (S-0009 claim 7)",
                );
            }
            let (Some(x), Some(y)) = (vals[0], vals[1]) else {
                // spec: S-0009 claim 7 (nothing, or x only: no cell changes)
                break;
            };
            let c = CellXY::new(x as i32, y as i32);
            // spec: S-0009 claim 7 (a record naming (0, 0) ends the stream)
            if c == CellXY::new(0, 0) {
                terminated = true;
                break;
            }
            let catalogue = self.ctx.catalogue;
            if let Some(cell) = cells.get_mut(c) {
                // spec: S-0009 claim 7 (coordinates of a legal cell but no tile number: the no-tile marker)
                let raw_tile = vals[2].map_or(NO_TILE, |t| t as i32);
                negative_tile |= raw_tile < 0;
                // spec: S-0009 claim 8 (translation; 65535 kept)
                cell.tile = catalogue.map_or(raw_tile, |cat| cat.translate(raw_tile));
                if let Some(s) = vals[3] {
                    cell.sub_tile = s as u8;
                }
                if let Some(l) = vals[4] {
                    // spec: S-0009 claim 7 (the height level is set outright, −128 … 127)
                    cell.level = l as u8 as i8;
                    negative_level |= cell.level < 0;
                }
                if let Some(f) = vals[5] {
                    cell.ice = f != 0;
                }
            }
            if vals[5].is_none() {
                break;
            }
        }
        if !terminated {
            // spec: S-0009 claim 7 (the importer warns about a stream without a (0, 0) record)
            self.warn(
                line,
                "the terrain stream ends without a (0, 0) record (S-0009 claim 7)",
            );
        }
        if negative_tile {
            // spec: S-0009 claim 8 (a negative tile number is treated like 65535 when drawn; the importer warns)
            self.warn(
                line,
                "a terrain record has a negative tile number (S-0009 claim 8)",
            );
        }
        if negative_level {
            // spec: S-0009 Edge cases (negative height levels: the importer warns)
            self.warn(
                line,
                "a terrain record has a negative height level (S-0009 Edge cases)",
            );
        }
    }

    /// The overlay streams (S-0009 claims 11 to 13). Returns whether the
    /// map has an overlay data stream.
    fn overlays(&mut self, cells: &mut CellGrid, format_version: i32) -> bool {
        // spec: S-0009 claim 11 (no overlay at all below format version 2, quirks.no_overlay_below_format_2)
        if format_version < OVERLAY_MIN_VERSION && self.ctx.quirks.no_overlay_below_format_2 {
            if self.doc.section(b"OverlayPack").is_some() {
                self.warn(
                    None,
                    "the map's format version is below 2, so its overlay is not read (S-0009 claim 11)",
                );
            }
            return false;
        }
        let n = OVERLAY_SIDE * OVERLAY_SIDE;
        let types = self.pack(b"OverlayPack", PackCodec::Lcw);
        let data = self.pack(b"OverlayDataPack", PackCodec::Lcw);
        for (name, stream) in [("OverlayPack", &types), ("OverlayDataPack", &data)] {
            if let Some(s) = stream
                && s.len() < n
            {
                // spec: S-0009 claims 12 and 13 (a short stream: the importer warns)
                self.warn(
                    None,
                    format!(
                        "[{name}] holds {} of {n} values (S-0009 claims 12 and 13)",
                        s.len()
                    ),
                );
            }
        }
        let cell_list: Vec<CellXY> = cells.iter().map(|(c, _)| c).collect();
        for c in cell_list {
            let (Ok(x), Ok(y)) = (usize::try_from(c.x), usize::try_from(c.y)) else {
                continue;
            };
            // spec: S-0009 claim 12 (the value at 512·y + x belongs to stream position (x, y))
            let i = OVERLAY_SIDE * y + x;
            let Some(cell) = cells.get_mut(c) else {
                continue;
            };
            if let Some(t) = &types {
                // spec: S-0009 claim 12 (shorter than a position: no overlay there)
                cell.overlay = t.get(i).copied().unwrap_or(NO_OVERLAY);
            }
            if let Some(dv) = &data {
                // spec: S-0009 claim 13 (shorter than a position: the value is 0)
                cell.overlay_data = dv.get(i).copied().unwrap_or(0);
            }
        }
        data.is_some()
    }

    /// The waypoint markers and the home waypoint (S-0009 claims 15, 16).
    fn waypoints(&mut self, footprint: &Footprint) -> (BTreeMap<u16, (i16, i16)>, u16) {
        let mut out = BTreeMap::new();
        if let Some(section) = self.doc.section(b"Waypoints") {
            let entries: Vec<(Vec<u8>, u32)> = section
                .entries()
                .iter()
                .map(|e| (e.key.clone(), e.line))
                .collect();
            for (key, line) in entries {
                // spec: S-0009 claim 15 (the plain decimal number: no sign, no leading zeros)
                let plain = !key.is_empty()
                    && key.iter().all(u8::is_ascii_digit)
                    && (key.len() == 1 || key[0] != b'0');
                let Some(k) = plain
                    .then(|| std::str::from_utf8(&key).ok()?.parse::<u16>().ok())
                    .flatten()
                    .filter(|k| *k < WAYPOINT_COUNT)
                else {
                    continue;
                };
                let Some(v) = section.value(&key).and_then(value::integer) else {
                    continue;
                };
                // spec: S-0009 claim 15 (the value 0 means undefined)
                if v == 0 {
                    continue;
                }
                let (x, y) = decode_cell_number(v, CELL_NUMBER_BASE);
                if i32::from(wrap_coordinate(y)) != y {
                    // spec: S-0009 claim 15 (the wrap is reproduced, and the importer warns)
                    self.warn(
                        Some(line),
                        format!("waypoint {k}: its y coordinate wraps (S-0009 claim 15)"),
                    );
                }
                let c = (wrap_coordinate(x), wrap_coordinate(y));
                if !footprint.is_legal(CellXY::new(c.0.into(), c.1.into()))
                    && k < glasnost_map::START_WAYPOINTS
                {
                    // spec: S-0009 Edge cases (start waypoint on a non-legal cell: the importer warns)
                    self.warn(
                        Some(line),
                        format!(
                            "start waypoint {k} names a cell that is not legal (S-0009 Edge cases)"
                        ),
                    );
                }
                out.insert(k, c);
            }
        }
        let raw_home = self.int(b"Basic", b"HomeCell", i32::from(DEFAULT_HOME_WAYPOINT));
        let home = match u16::try_from(raw_home) {
            Ok(h) if h < WAYPOINT_COUNT => h,
            _ => {
                // spec: S-0009 Edge cases (home waypoint outside 0 … 701: 699, and the importer warns)
                self.warn(
                    self.line_of(b"Basic", b"HomeCell"),
                    "the home waypoint number is outside 0 … 701; our engine uses 699 (S-0009 Edge cases)",
                );
                DEFAULT_HOME_WAYPOINT
            }
        };
        // spec: S-0009 claim 16 (an undefined home waypoint is defined at (c, c))
        out.entry(home).or_insert_with(|| {
            let c = footprint.home_default();
            (wrap_coordinate(c.x), wrap_coordinate(c.y))
        });
        (out, home)
    }

    /// The header block (S-0009 claims 25 and 29).
    fn header(&mut self) -> Header {
        if self.doc.section(b"Header").is_none() {
            return Header::default();
        }
        // spec: S-0009 Import mapping ([Header] keys, −1 when absent)
        let count = self.int(b"Header", b"NumberStartingPoints", -1);
        let origin = (
            self.int(b"Header", b"StartX", -1),
            self.int(b"Header", b"StartY", -1),
        );
        let extent = (
            self.int(b"Header", b"Width", -1),
            self.int(b"Header", b"Height", -1),
        );
        if (1..=PREVIEW_PAIRS).contains(&count) && (extent.0 == 0 || extent.1 == 0) {
            // spec: S-0009 claim 25, Edge cases (extent 0: no preview markers, and the importer warns)
            self.warn(
                self.line_of(b"Header", b"Width"),
                "the header's marker extent is 0; our engine draws no preview markers (S-0009 Edge cases)",
            );
        }
        if count > PREVIEW_PAIRS {
            // spec: S-0009 Edge cases (quirks.header_pairs_as_start_marks: the importer warns about N_h above 8)
            self.warn(
                self.line_of(b"Header", b"NumberStartingPoints"),
                "the header count is above 8: pairs 9 … 16 act as start position marks (S-0009 claim 29)",
            );
        }
        if count > LAST_HEADER_PAIR {
            // spec: S-0009 claim 29 (pairs from the seventeenth on: ignored by our engine, and the importer warns)
            self.warn(
                self.line_of(b"Header", b"NumberStartingPoints"),
                "header pairs from the seventeenth on are ignored (S-0009 claim 29)",
            );
        }
        let mut pairs = Vec::new();
        // spec: S-0009 claim 25 (exactly pairs 1 … N_h are read), claim 29 (our engine stops at 16)
        for i in 1..=count.min(LAST_HEADER_PAIR) {
            let key = format!("Waypoint{i}");
            let stored = self.doc.value(b"Header", key.as_bytes());
            let pair = match stored {
                // spec: S-0009 Import mapping (pairs 1 … 8 default to (0, 0); pairs 9 … 16 leave the marks)
                None => (i <= PREVIEW_PAIRS).then_some((0, 0)),
                Some(v) => {
                    // spec: S-0009 Import mapping (S-0003 claim 9: an unread component takes the value in force)
                    let ([x, y], unread) = value::int_tuple_or::<2>(v, [0, 0]);
                    if unread {
                        self.warn(
                            self.line_of(b"Header", key.as_bytes()),
                            format!("[Header] {key}: a component cannot be read (S-0003 claim 9)"),
                        );
                    }
                    Some((x, y))
                }
            };
            pairs.push(pair);
        }
        Header {
            count,
            origin,
            extent,
            pairs,
        }
    }

    /// The preview (S-0009 claim 24).
    fn preview(&mut self) -> Option<Preview> {
        // spec: S-0009 claim 30 ([Preview] Size, absent: 0,0,0,0)
        let size = value::rect(
            self.doc.value(b"Preview", b"Size"),
            [0; 4],
            self.ctx.quirks.absent_rect_is_zero,
        );
        let (Ok(width), Ok(height)) = (u32::try_from(size[2]), u32::try_from(size[3])) else {
            return None;
        };
        if width == 0 || height == 0 {
            return None;
        }
        let n = usize::try_from(u64::from(width) * u64::from(height) * 3).ok()?;
        // A bound of our own: no preview larger than the decoded-data limit.
        if n > glasnost_fmt::limits::MAX_DECOMPRESSED {
            self.warn(
                None,
                "the preview is larger than our decoding limit and is not shown",
            );
            return None;
        }
        let data = self
            .pack(b"PreviewPack", PackCodec::Lzo)
            .unwrap_or_default();
        let mut rgb = vec![0u8; n];
        // spec: S-0009 claim 24 (only pixels whose three values are present show their colour; the rest black)
        let complete = data.len().min(n) / 3 * 3;
        rgb[..complete].copy_from_slice(&data[..complete]);
        if data.len() < n {
            self.warn(
                self.doc.section(b"PreviewPack").map(|s| s.line()),
                "the preview stream ends before the last pixel; the rest is black (S-0009 claim 24)",
            );
        }
        Some(Preview { width, height, rgb })
    }

    fn decimal_scaled(
        &mut self,
        key: &[u8],
        scale: i64,
        plus: Option<&Binary>,
        default: i32,
    ) -> i32 {
        let Some(stored) = self.doc.value(b"Lighting", key) else {
            return default;
        };
        // spec: S-0104 claim 1 (v is the single-precision decimal of S-0003 claim 6)
        let Decimal::Value { value: v, .. } = value::decimal(stored, DecimalField::Single) else {
            // spec: S-0003 claim 8 (no number: our engine keeps the value in force, and warns)
            self.warn(
                self.line_of(b"Lighting", key),
                format!(
                    "[Lighting] {}: no number; the default is kept (S-0003 claim 8)",
                    show(key)
                ),
            );
            return default;
        };
        // spec: S-0104 Formulas (double precision, each step rounded toward zero, then truncated)
        let mut x = v.mul_rz(&Binary::from_i64(Format::Double, scale), Format::Double);
        if let Some(p) = plus {
            x = x.add_rz(p, Format::Double);
        }
        x.trunc_to_i64()
            .and_then(|t| i32::try_from(t).ok())
            .unwrap_or(default)
    }

    /// The map's lighting (S-0104 claim 1, Import mapping).
    fn lighting(&mut self) -> Lighting {
        let d = Lighting::default();
        let hundredth = hundredth_double();
        let p = Some(&hundredth);
        // spec: S-0104 claim 1 (hundredths ⌊v·100 + 0.01⌋, thousandths ⌊v·1000 + 0.01⌋)
        let (h, t) = (100, 1000);
        let set = |dec: &mut Self, prefix: &str, def: LightingSet| LightingSet {
            ambient: dec.decimal_scaled(format!("{prefix}Ambient").as_bytes(), h, p, def.ambient),
            tint: [
                dec.decimal_scaled(format!("{prefix}Red").as_bytes(), h, p, def.tint[0]),
                dec.decimal_scaled(format!("{prefix}Green").as_bytes(), h, p, def.tint[1]),
                dec.decimal_scaled(format!("{prefix}Blue").as_bytes(), h, p, def.tint[2]),
            ],
            ground: dec.decimal_scaled(format!("{prefix}Ground").as_bytes(), t, p, def.ground),
            level: dec.decimal_scaled(format!("{prefix}Level").as_bytes(), t, p, def.level),
        };
        let normal = set(self, "", d.normal);
        let ion = set(self, "Ion", d.ion);
        let dominator = set(self, "Dominator", d.dominator);
        Lighting {
            normal,
            ion,
            dominator,
            // spec: S-0104 Import mapping (DominatorAmbientChangeRate: thousandths, used as ticks)
            dominator_rate: self.decimal_scaled(
                b"DominatorAmbientChangeRate",
                t,
                p,
                d.dominator_rate,
            ),
            // spec: S-0104 Import mapping (NukeAmbientChangeRate: truncated to whole ticks)
            nuke_rate: self.decimal_scaled(b"NukeAmbientChangeRate", 1, None, d.nuke_rate),
        }
    }

    fn basic(&mut self) -> Basic {
        Basic {
            // spec: S-0009 Import mapping ([Basic] Name, default "No Name")
            name: self
                .doc
                .value(b"Basic", b"Name")
                .map_or_else(|| b"No Name".to_vec(), <[u8]>::to_vec),
            // spec: S-0009 Import mapping (MinPlayers 2, MaxPlayers 4)
            min_players: self.int(b"Basic", b"MinPlayers", 2),
            max_players: self.int(b"Basic", b"MaxPlayers", 4),
            game_modes: self
                .doc
                .value(b"Basic", b"GameMode")
                .unwrap_or_default()
                .to_vec(),
            official: self.boolean(b"Basic", b"Official", false),
            multiplayer_only: self.boolean(b"Basic", b"MultiplayerOnly", false),
        }
    }

    /// The object lists (Q-0010: provisional layout).
    fn objects(&mut self, footprint: &Footprint) -> Objects {
        let mut o = Objects::default();
        // provisional (Q-0010): owner, type, health, x, y; facing 6th (infantry: 8th, sub-cell 6th)
        for (name, kind) in [
            (&b"Units"[..], 0u8),
            (b"Infantry", 1),
            (b"Aircraft", 2),
            (b"Structures", 3),
        ] {
            let Some(section) = self.doc.section(name) else {
                continue;
            };
            for (entry, winner) in section.list_positions() {
                let fields: Vec<Vec<u8>> = winner
                    .value
                    .split(|&b| b == b',')
                    .map(|f| value::text(f.trim_ascii_start(), usize::MAX).to_vec())
                    .collect();
                let int = |i: usize| fields.get(i).map(|f| value::decimal_integer(f));
                let (Some(x), Some(y), Some(health)) = (int(3), int(4), int(2)) else {
                    self.warn(
                        Some(entry.line),
                        format!(
                            "[{}] {}: fewer than five values; the entry is skipped (Q-0010)",
                            show(name),
                            show(&entry.key)
                        ),
                    );
                    continue;
                };
                let cell = CellXY::new(x, y);
                if !footprint.is_legal(cell) {
                    self.warn(
                        Some(entry.line),
                        format!(
                            "[{}] {}: the cell is not legal",
                            show(name),
                            show(&entry.key)
                        ),
                    );
                }
                let facing_index = if kind == 1 { 7 } else { 5 }; // nocite: provisional layout (Q-0010)
                let placed = Placed {
                    key: entry.key.clone(),
                    owner: fields.first().cloned().unwrap_or_default(),
                    type_name: fields.get(1).cloned().unwrap_or_default(),
                    health,
                    cell,
                    facing: int(facing_index).unwrap_or(0),
                    raw: fields.clone(),
                };
                match kind {
                    0 => o.units.push(Unit { placed }),
                    1 => o.infantry.push(Infantry {
                        sub_cell: int(5).unwrap_or(0), // nocite: provisional layout (Q-0010)
                        placed,
                    }),
                    2 => o.aircraft.push(Aircraft { placed }),
                    _ => o.structures.push(Structure { placed }),
                }
            }
        }
        if let Some(section) = self.doc.section(b"Terrain") {
            for (entry, winner) in section.list_positions() {
                // provisional (Q-0010): the key is a cell number, decoded 1000-based
                let n = value::decimal_integer(&entry.key);
                let (x, y) = decode_cell_number(n, CELL_NUMBER_BASE);
                o.terrain.push(TerrainObject {
                    cell: CellXY::new(x, y),
                    type_name: value::text(&winner.value, usize::MAX).to_vec(),
                });
            }
        }
        if let Some(section) = self.doc.section(b"Smudge") {
            for (_, winner) in section.list_positions() {
                // provisional (Q-0010): type, x, y
                let fields: Vec<&[u8]> = winner.value.split(|&b| b == b',').collect();
                let int = |i: usize| {
                    fields
                        .get(i)
                        .map(|f| value::decimal_integer(f))
                        .unwrap_or(0)
                };
                o.smudges.push(Smudge {
                    type_name: fields
                        .first()
                        .map(|f| value::text(f, usize::MAX).to_vec())
                        .unwrap_or_default(),
                    cell: CellXY::new(int(1), int(2)),
                });
            }
        }
        o
    }

    /// Cell tags (S-0009 claim 31).
    fn cell_tags(
        &mut self,
        footprint: &Footprint,
        format_version: i32,
    ) -> BTreeMap<(i16, i16), Vec<u8>> {
        let mut out = BTreeMap::new();
        let Some(section) = self.doc.section(b"CellTags") else {
            return out;
        };
        // spec: S-0009 Formulas (1000-based from format version 4, 128-based below)
        let base = if format_version >= CELL_TAG_BASE_SWITCH_VERSION {
            CELL_NUMBER_BASE
        } else {
            LEGACY_CELL_TAG_BASE
        };
        let mut first_key: BTreeMap<(i16, i16), Vec<u8>> = BTreeMap::new();
        for (entry, winner) in section.list_positions() {
            // spec: S-0009 claim 31 (the key read leniently as its leading integer)
            let n = value::decimal_integer(&entry.key);
            let (x, y) = decode_cell_number(n, base);
            let tag = winner.value.clone();
            // spec: S-0009 claim 31 (a value naming no defined tag is ignored)
            if self.doc.get(b"Tags", &tag).is_none() {
                continue;
            }
            if !footprint.is_legal(CellXY::new(x, y)) {
                // spec: S-0009 Edge cases (a key decoding to a non-legal cell: ignored, and the importer warns)
                self.warn(
                    Some(entry.line),
                    format!("[CellTags] {}: not a legal cell; ignored", show(&entry.key)),
                );
                continue;
            }
            let c = (wrap_coordinate(x), wrap_coordinate(y));
            match first_key.get(&c) {
                // spec: S-0009 claim 31 (a cell keeps the first tag attached to it)
                Some(k) if *k != entry.key => {
                    self.warn(
                        Some(entry.line),
                        format!(
                            "[CellTags] {} decodes to the same cell as {} (S-0009 claim 31)",
                            show(&entry.key),
                            show(k)
                        ),
                    );
                }
                Some(_) => {}
                None => {
                    first_key.insert(c, entry.key.clone());
                    out.insert(c, tag);
                }
            }
        }
        out
    }
}

/// The double nearest to 0.01 (S-0104 Formulas, "Map value storage").
fn hundredth_double() -> Binary {
    let text = binfloat::DecimalText {
        neg: false,
        digits: b"1".to_vec(),
        exp10: -2,
    };
    // 0.01 is not near a tie, so the conversion gives the nearest double.
    binfloat::decimal_to_binary(&text, Format::Double).value
}

/// Our map crate's theater for a vfs theater.
pub fn to_map_theater(t: glasnost_vfs::theater::Theater) -> Theater {
    use glasnost_vfs::theater::Theater as V;
    match t {
        V::Temperate => Theater::Temperate,
        V::Snow => Theater::Snow,
        V::Urban => Theater::Urban,
        V::Desert => Theater::Desert,
        V::NewUrban => Theater::NewUrban,
        V::Lunar => Theater::Lunar,
    }
}

/// The vfs theater for our map crate's theater.
pub fn to_vfs_theater(t: Theater) -> glasnost_vfs::theater::Theater {
    use glasnost_vfs::theater::Theater as V;
    match t {
        Theater::Temperate => V::Temperate,
        Theater::Snow => V::Snow,
        Theater::Urban => V::Urban,
        Theater::Desert => V::Desert,
        Theater::NewUrban => V::NewUrban,
        Theater::Lunar => V::Lunar,
    }
}

/// Whether `doc` looks like a map: it has `[Map]` and `[IsoMapPack5]`.
pub fn is_map(doc: &Document) -> bool {
    doc.section(b"Map").is_some() && doc.section(b"IsoMapPack5").is_some()
}

pub mod synth;
#[cfg(test)]
mod tests;
