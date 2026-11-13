//! Map decoding (S-0009): a map file read as an INI document becomes a
//! [`glasnost_map::Map`].
//!
//! The map blocks' text, chunk framing, compressed encodings and terrain
//! records are S-0012's: the text is joined and decoded group by group
//! (`glasnost_fmt::mappack::decode_text_groups`), the chunks expanded with
//! the valid-prefix rule (`unpack_prefix`), the codecs those of
//! `glasnost_fmt` (LZO1X for `IsoMapPack5` and `PreviewPack`, LCW for the
//! two overlay packs; OpenRA and XCC, GPL-3.0-or-later). A terrain record is
//! S-0012 claim 6's eleven bytes.
//!
//! The object lists are split and read as S-0009 claims 34 to 36, 40 and 42
//! say; the state each object starts in is the simulation's and the
//! viewer's (claims 37 to 41, 43).

use std::collections::BTreeMap;

use glasnost_fmt::mappack::{PackCodec, decode_text_groups, unpack_prefix};
use glasnost_ini::Document;
use glasnost_ini::binfloat::{self, Binary, Format};
use glasnost_ini::value::{self, Decimal, DecimalField};
use glasnost_map::geometry::{
    CELL_NUMBER_BASE, CELL_TAG_BASE_SWITCH_VERSION, LEGACY_CELL_TAG_BASE, decode_cell_number,
    wrap_coordinate,
};
use glasnost_map::objects::{EntryFields, ListKind, Placed, UPGRADE_FIELDS};
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
    /// `quirks.object_empty_fields_collapse` (claim 34).
    pub object_empty_fields_collapse: bool,
    /// `quirks.cell_index_alias` (claims 7 and 31): a pair whose x lies
    /// outside 0 … 511 selects the cell with the same stream position index.
    pub cell_index_alias: bool,
    /// `quirks.header_pairs_as_start_marks` (claims 25 and 29): all N_h
    /// header pairs are read; off: at most 8.
    pub header_pairs_as_start_marks: bool,
    /// `quirks.lobby_counts_zero_waypoint` (claim 23), consulted by the game
    /// client's map list (`glasnost_app::game::install::start_count`).
    pub lobby_counts_zero_waypoint: bool,
    /// `quirks.preview_markers_from_header` (claim 25), consulted by the
    /// game client's map picker: on, the preview's markers come from the
    /// header; off, the picker draws none (projecting the waypoint cells
    /// onto the preview is not specified).
    pub preview_markers_from_header: bool,
    /// `quirks.map_pack_line_limit` (S-0012 claim 1): each packed entry's
    /// value is cut to 127 characters; off: the full value.
    pub map_pack_line_limit: bool,
    /// `quirks.map_base64_groups` (S-0012 claim 2): nonstandard complete
    /// groups are read as the original reads them; off: the block is absent.
    pub map_base64_groups: bool,
    /// `quirks.partial_terrain_values` (S-0012 claim 9, S-0009 claim 7):
    /// partial terrain values replace low-order bytes; off: incomplete
    /// multi-byte values are absent.
    pub partial_terrain_values: bool,
}

impl Default for MapQuirks {
    fn default() -> Self {
        // spec: S-0009 Edge cases and quirks (every switch defaults to the original)
        Self {
            narrow_map_partial_pass: true,
            absent_rect_is_zero: true,
            stale_fill_tiles: true,
            no_overlay_below_format_2: true,
            object_empty_fields_collapse: true,
            cell_index_alias: true,
            header_pairs_as_start_marks: true,
            lobby_counts_zero_waypoint: true,
            preview_markers_from_header: true,
            // spec: S-0012 Edge cases and quirks (every switch defaults to the original)
            map_pack_line_limit: true,
            map_base64_groups: true,
            partial_terrain_values: true,
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
const RECORD_FIELDS: [usize; 6] = [2, 2, 4, 1, 1, 1]; // spec: S-0012 claim 6 (record layout)
/// Characters of a packed entry's value that are read.
const PACK_LINE_LIMIT: usize = 127; // spec: S-0012 Constants (packed-entry reader limit)

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

/// Cell tags by cell, and the same in entry order.
type CellTags = (BTreeMap<(i16, i16), Vec<u8>>, Vec<((i16, i16), Vec<u8>)>);

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
        let objects = self.objects(&footprint, format_version);
        let (cell_tags, cell_tag_order) = self.cell_tags(&footprint, format_version);
        let triggers = self.triggers(&cell_tag_order);
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
            triggers,
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
    /// are taken when a game loads (`glasnost_sim::load`, S-0202), so the
    /// decoded map gives every visited cell the base tile plus 0.
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
                    "water fill: {draws} cells take a draw over 0 … {} from the scenario random stream (S-0202) when a game loads; the map as stored shows the water base tile",
                    WATER_VARIANTS - 1
                ),
            );
        }
        draws
    }

    /// A map block's expanded bytes (S-0012 claims 1 to 5, 9 and 10):
    /// `None` when the block has no packed bytes (absent, empty, or its text
    /// rejected), otherwise the bytes of its valid chunks.
    fn pack(&mut self, name: &[u8], codec: PackCodec) -> Option<Vec<u8>> {
        let packed = self.pack_text(name)?;
        let line = self.doc.section(name).map(|s| s.line());
        let (bytes, problem) = unpack_prefix(&packed, codec);
        if let Some(p) = problem {
            // spec: S-0012 claims 9 and 10 (keep the earlier chunks, stop the block, warn naming block and chunk)
            self.warn(
                line,
                format!(
                    "[{}]: chunk {}: {}; the block stops there (S-0012 claims 9 and 10)",
                    show(name),
                    p.chunk + 1,
                    p.what
                ),
            );
        }
        Some(bytes)
    }

    /// A map block's packed bytes before its chunks are expanded: its text
    /// joined and decoded (S-0012 claims 1 and 2). `None` when the block
    /// has none (absent, empty, or its text rejected).
    fn pack_text(&mut self, name: &[u8]) -> Option<Vec<u8>> {
        let section = self.doc.section(name)?;
        let line = Some(section.line());
        let q = self.ctx.quirks;
        let mut text = Vec::new();
        let mut cut = false;
        // spec: S-0012 claim 1 (entries in enumeration order, each value by name resolution)
        for e in section.entries() {
            let v = section
                .get(&e.key)
                .map_or(e.value.as_slice(), |w| w.value.as_slice());
            // spec: S-0012 claim 1 (quirks.map_pack_line_limit: the first 127 characters, trimmed)
            let v = if q.map_pack_line_limit {
                cut |= v.len() > PACK_LINE_LIMIT;
                value::text(v, PACK_LINE_LIMIT)
            } else {
                v.trim_ascii_end()
            };
            text.extend_from_slice(v.trim_ascii());
        }
        if cut {
            // spec: S-0012 Edge cases (quirks.map_pack_line_limit: warn when characters are lost)
            self.warn(
                line,
                format!(
                    "[{}]: an entry is longer than {PACK_LINE_LIMIT} characters; the rest is not read (S-0012 claim 1, quirks.map_pack_line_limit)",
                    show(name)
                ),
            );
        }
        let decoded = decode_text_groups(&text);
        if let Some(g) = decoded.first_nonstandard {
            if !q.map_base64_groups {
                // spec: S-0012 claim 2 (quirks.map_base64_groups off: the block is absent, with a warning)
                self.warn(
                    line,
                    format!(
                        "[{}]: base-64 group {} is not standard; the block is ignored (S-0012 claim 2)",
                        show(name),
                        g + 1
                    ),
                );
                return None;
            }
            // spec: S-0012 claim 2 (nonstandard complete groups read as the original does, with a warning)
            self.warn(
                line,
                format!(
                    "[{}]: {} base-64 groups are not standard, the first is group {} (S-0012 claim 2, quirks.map_base64_groups)",
                    show(name),
                    decoded.nonstandard,
                    g + 1
                ),
            );
        }
        if decoded.short_tail > 0 {
            // spec: S-0012 claim 2 (a final group shorter than four characters is discarded, with a warning)
            self.warn(
                line,
                format!(
                    "[{}]: the text ends with {} characters short of a group; they are not read (S-0012 claim 2)",
                    show(name),
                    decoded.short_tail
                ),
            );
        }
        // spec: S-0012 Edge cases (a block yielding no packed bytes has no content to apply)
        if decoded.bytes.is_empty() {
            return None;
        }
        Some(decoded.bytes)
    }

    /// The terrain stream (S-0009 claims 7 and 8).
    fn terrain(&mut self, cells: &mut CellGrid) {
        let Some(data) = self.pack(b"IsoMapPack5", PackCodec::Lzo) else {
            return;
        };
        let line = self.doc.section(b"IsoMapPack5").map(|s| s.line());
        let partial_values = self.ctx.quirks.partial_terrain_values;
        if data.len() < 4 {
            // spec: S-0012 claim 9, S-0009 claim 7 (fewer than four bytes for the first coordinate
            // pair: no record from the block, with a warning; no switch)
            self.warn(
                line,
                "the terrain block has fewer than four bytes for its first coordinate pair; no terrain record is read (S-0012 claim 9)",
            );
            return;
        }
        let mut pos = 0usize;
        let mut terminated = false;
        let mut negative_tile = false;
        let mut negative_level = false;
        let mut alias_warned = false;
        let mut partial_warned = false;
        loop {
            if pos >= data.len() {
                // spec: S-0012 claim 9 (ending between records: as an end marker, with a warning below)
                break;
            }
            // Read up to six values, each the bytes available of its width.
            let mut vals = [None::<i64>; 6];
            let mut partial = None;
            for (i, width) in RECORD_FIELDS.iter().enumerate() {
                let have = (data.len() - pos).min(*width);
                if have == 0 {
                    break;
                }
                let bytes = &data[pos..pos + have];
                pos += have;
                if have < *width {
                    partial = Some(i);
                }
                vals[i] = Some(match (i, have == *width) {
                    // spec: S-0012 claim 6, Formulas F2 (x and y signed 16-bit, the tile number signed
                    // 32-bit, little-endian)
                    (0 | 1, true) => i64::from(i16::from_le_bytes([bytes[0], bytes[1]])),
                    (2, true) => {
                        i64::from(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
                    }
                    // spec: S-0012 claim 6 (sub-tile and ice flag unsigned, the level a signed byte)
                    (_, true) => i64::from(bytes[0]),
                    // spec: S-0012 claim 9 (a partial coordinate: the available low-order bytes of 0)
                    (0 | 1, false) => i64::from(bytes[0]),
                    // spec: S-0012 Formulas F3 (T = B + 256ᵏ·⌊65535 / 256ᵏ⌋, then signed)
                    (_, false) => {
                        let k = bytes.len() as u32;
                        let b = bytes.iter().rev().fold(0u32, |a, &v| a << 8 | u32::from(v));
                        let scale = 256u32.pow(k);
                        i64::from((b + scale * (65535 / scale)) as i32)
                    }
                });
                if have < *width {
                    break;
                }
            }
            // The pair: a partial x gives y 0; a partial y keeps its low byte.
            if let Some(i) = partial {
                if !partial_warned {
                    partial_warned = true;
                    // spec: S-0012 claim 9 (partial terrain values: the importer warns)
                    self.warn(
                        line,
                        "the terrain stream ends partway through a value (S-0012 claim 9, quirks.partial_terrain_values)",
                    );
                }
                if !partial_values {
                    // spec: S-0012 claim 9 (switch off: a partial coordinate pair ends the stream,
                    // a partial tile number is the preset 65535)
                    if i < 2 {
                        break;
                    }
                    vals[2] = None;
                }
            }
            let x = vals[0].unwrap_or(0);
            let y = vals[1].unwrap_or(0);
            let named = CellXY::new(x as i32, y as i32);
            // spec: S-0009 claim 7 (a record naming (0, 0) ends the stream)
            if named == CellXY::new(0, 0) {
                terminated = true;
                break;
            }
            // spec: S-0009 claim 7 (the record acts on its selected cell)
            let selected = selected_cell(named, self.ctx.quirks.cell_index_alias);
            if !(0..=MAX_STREAM_COORDINATE).contains(&named.x) && !alias_warned {
                alias_warned = true;
                // spec: S-0009 Edge cases (quirks.cell_index_alias: the importer warns either way)
                self.warn(
                    line,
                    "a terrain record's x lies outside 0 … 511 (S-0009 claim 7, quirks.cell_index_alias)",
                );
            }
            let catalogue = self.ctx.catalogue;
            if let Some(cell) = selected.and_then(|c| cells.get_mut(c)) {
                // spec: S-0009 claim 7 (coordinates of a legal cell but no tile number: the no-tile marker)
                let raw_tile = vals[2].map_or(NO_TILE, |t| t as i32);
                negative_tile |= raw_tile < 0;
                // spec: S-0009 claim 8 (translation; 65535 kept)
                cell.tile = catalogue.map_or(raw_tile, |cat| cat.translate(raw_tile));
                cell.stream = true;
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
                && s.len() != n
            {
                // spec: S-0009 claims 12 and 13, S-0012 claim 7 (a short or overlong array: the importer
                // warns; bytes beyond 262144 have no effect)
                self.warn(
                    None,
                    format!(
                        "[{name}] holds {} bytes for {n} positions (S-0012 claim 7)",
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
        // spec: S-0009 claim 25 (exactly pairs 1 … N_h are read), claim 29 (our engine stops at 16);
        // quirks.header_pairs_as_start_marks off: at most 8 pairs
        let last = if self.ctx.quirks.header_pairs_as_start_marks {
            LAST_HEADER_PAIR
        } else {
            PREVIEW_PAIRS
        };
        for i in 1..=count.min(last) {
            let key = format!("Waypoint{i}");
            let stored = self.doc.value(b"Header", key.as_bytes());
            let pair = match stored {
                // spec: S-0009 Import mapping (pairs 1 … 8 default to (0, 0); pairs 9 … 16 leave the marks)
                None => (i <= PREVIEW_PAIRS).then_some([Some(0), Some(0)]),
                Some(v) => {
                    let mut read = value::int_tuple::<2>(v);
                    if read.iter().any(Option::is_none) {
                        // spec: S-0009 claim 29 and Import mapping (an unread component: our engine keeps the readable ones and completes the rest from the value in force, with a warning)
                        self.warn(
                            self.line_of(b"Header", key.as_bytes()),
                            format!(
                                "[Header] {key}: a component cannot be read; our engine takes the value in force's (S-0009 claim 29, S-0003 claim 9)"
                            ),
                        );
                    }
                    if i <= PREVIEW_PAIRS {
                        // spec: S-0009 Import mapping (pairs 1 … 8: the value in force is (0, 0))
                        for c in &mut read {
                            c.get_or_insert(0);
                        }
                    }
                    // Pairs 9 … 16 keep unread components unread: the mark in force
                    // at each phase completes them (S-0009 claim 29).
                    Some(read)
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
        let (width, height) = self.preview_size()?;
        let data = self
            .pack(b"PreviewPack", PackCodec::Lzo)
            .unwrap_or_default();
        let (rgb, fit) = preview_pixels(width, height, &data);
        let line = self.doc.section(b"PreviewPack").map(|s| s.line());
        match fit {
            PixelFit::Short => self.warn(
                line,
                "the preview stream ends before the last pixel; the rest is black (S-0009 claim 24)",
            ),
            // spec: S-0012 claim 8 (an overlong preview is ignored beyond its required bytes, with a warning)
            PixelFit::Long => self.warn(
                line,
                "the preview stream is longer than its pixels need; the rest is not read (S-0012 claim 8)",
            ),
            PixelFit::Exact => {}
        }
        Some(Preview { width, height, rgb })
    }

    /// The preview's width and height (S-0009 claims 24 and 30), `None`
    /// when they give no preview.
    fn preview_size(&mut self) -> Option<(u32, u32)> {
        // spec: S-0009 claim 30 ([Preview] Size, absent: 0,0,0,0)
        let size = value::rect(
            self.doc.value(b"Preview", b"Size"),
            [0; 4],
            self.ctx.quirks.absent_rect_is_zero,
        );
        let bytes = i64::from(size[2]) * i64::from(size[3]) * 3;
        if size[2] <= 0 || size[3] <= 0 || bytes > i64::from(i32::MAX) {
            if self.doc.section(b"PreviewPack").is_some() {
                // spec: S-0012 Edge cases (invalid preview dimensions: no embedded preview, with a warning)
                self.warn(
                    self.line_of(b"Preview", b"Size"),
                    "the preview size is not positive or too large; no preview is shown (S-0012 Edge cases)",
                );
            }
            return None;
        }
        let (width, height) = (size[2] as u32, size[3] as u32);
        let n = bytes as usize;
        // A bound of our own: no preview larger than the decoded-data limit.
        if n > glasnost_fmt::limits::MAX_DECOMPRESSED {
            self.warn(
                None,
                "the preview is larger than our decoding limit and is not shown",
            );
            return None;
        }
        Some((width, height))
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
            // spec: S-0104 Edge cases (Dominator fade rate scale: with quirks.dominator_rate_thousandths off, ⌊v⌋ ticks, truncated as the nuke rate is)
            dominator_rate_whole: self.decimal_scaled(
                b"DominatorAmbientChangeRate",
                1,
                None,
                d.dominator_rate_whole,
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
            // spec: S-0410 Import mapping ([Basic] FreeRadar, absent: no)
            free_radar: self.boolean(b"Basic", b"FreeRadar", false),
            // spec: S-0401 Import mapping ([Basic] TiberiumGrowthEnabled, the map growth switch; absent: on)
            tiberium_growth: self
                .doc
                .value(b"Basic", b"TiberiumGrowthEnabled")
                .and_then(value::boolean),
            // spec: S-0303 Import mapping (the map's special-flags section, Inert, absent: no)
            inert: self.boolean(b"SpecialFlags", b"Inert", false),
            // spec: S-0303 Import mapping (the map's special-flags section, HarvesterImmune, absent: no)
            harvester_immune: self.boolean(b"SpecialFlags", b"HarvesterImmune", false),
        }
    }

    /// The object lists (S-0009 claims 34 to 36, 38, 40 to 42).
    fn objects(&mut self, footprint: &Footprint, format_version: i32) -> Objects {
        let mut o = Objects::default();
        let q = self.ctx.quirks;
        // spec: S-0009 claim 35 (below format version 4 one cell field, decoded base 128)
        let legacy = format_version < CELL_TAG_BASE_SWITCH_VERSION;
        for kind in [
            ListKind::Vehicles,
            ListKind::Infantry,
            ListKind::Aircraft,
            ListKind::Buildings,
        ] {
            let name: &[u8] = match kind {
                ListKind::Vehicles => b"Units",
                ListKind::Infantry => b"Infantry",
                ListKind::Aircraft => b"Aircraft",
                ListKind::Buildings => b"Structures",
            };
            let Some(section) = self.doc.section(name) else {
                continue;
            };
            let entries: Vec<_> = section
                .list_positions()
                .map(|(e, w)| (e.key.clone(), e.line, w.value.clone()))
                .collect();
            for (key, line, value) in entries {
                let fields = split_entry(&value, q.object_empty_fields_collapse);
                let at = |f: usize| entry_field(&fields, f, legacy);
                let (Some(owner), Some(type_name)) = (at(1), at(2)) else {
                    // spec: S-0009 claim 36 (no owner or type field: no match, dropped silently, no creation draw)
                    continue;
                };
                // spec: S-0009 claim 35 (fields up to 6 required, up to 5 below format version 4)
                let required = if legacy { 5 } else { 6 };
                let short =
                    fields.len() < required || fields[..required].iter().any(Option::is_none);
                if short {
                    // spec: S-0009 claim 36 (a short entry: dropped, a departure; the importer warns)
                    self.warn(
                        Some(line),
                        format!(
                            "[{}] {}: lacks a required field (S-0009 claim 36); our engine drops it",
                            show(name),
                            show(&key)
                        ),
                    );
                }
                let int = |f: usize| at(f).map(|v| value::decimal_integer(&v));
                let flag = |f: usize| int(f).map(|v| v != 0);
                let (x, y) = if legacy {
                    // spec: S-0009 Formulas (object cell: below format version 4, x = n rem 128, y = n quo 128)
                    decode_cell_number(int(4).unwrap_or(0), LEGACY_CELL_TAG_BASE)
                } else {
                    (int(4).unwrap_or(0), int(5).unwrap_or(0))
                };
                // spec: S-0009 Formulas (object cell: each coordinate wrapped modulo 65536)
                let cell =
                    CellXY::new(i32::from(wrap_coordinate(x)), i32::from(wrap_coordinate(y)));
                if !short && !footprint.is_legal(cell) {
                    // spec: S-0009 claim 38 (a non-legal cell: creation draw, dropped; the importer warns)
                    self.warn(
                        Some(line),
                        format!(
                            "[{}] {}: the cell is not legal (S-0009 claim 38); our engine drops the object",
                            show(name),
                            show(&key)
                        ),
                    );
                }
                let mut extra = EntryFields {
                    short,
                    ..Default::default()
                };
                // spec: S-0009 claim 35 (field layout by kind)
                let facing = match kind {
                    ListKind::Vehicles => {
                        extra.behaviour = at(7);
                        extra.tag = at(8);
                        extra.veterancy = int(9);
                        extra.group = int(10);
                        extra.bridge = flag(11);
                        extra.follow = int(12);
                        extra.recruit_no = flag(13);
                        extra.recruit_yes = flag(14);
                        int(6)
                    }
                    ListKind::Aircraft => {
                        extra.behaviour = at(7);
                        extra.tag = at(8);
                        extra.veterancy = int(9);
                        extra.group = int(10);
                        extra.recruit_no = flag(11);
                        extra.recruit_yes = flag(12);
                        int(6)
                    }
                    ListKind::Infantry => {
                        extra.behaviour = at(7);
                        extra.tag = at(9);
                        extra.veterancy = int(10);
                        extra.group = int(11);
                        extra.bridge = flag(12);
                        extra.recruit_no = flag(13);
                        extra.recruit_yes = flag(14);
                        // spec: S-0009 claim 39 (infantry: the facing value is field 8; absent faces 0)
                        int(8)
                    }
                    ListKind::Buildings => {
                        extra.tag = at(7);
                        extra.ai_sell = flag(8);
                        extra.ai_rebuild = flag(9);
                        extra.switched_on = flag(10);
                        extra.upgrade_count = int(11);
                        extra.searchlight = int(12);
                        extra.upgrades = [at(13), at(14), at(15)];
                        extra.ai_repair = flag(16);
                        extra.nominal = flag(17);
                        if extra
                            .upgrade_count
                            .is_some_and(|c| c > UPGRADE_FIELDS as i32)
                        {
                            // spec: S-0009 claim 40 (a count above 3: unspecified, our engine uses 3; the importer warns)
                            self.warn(
                                Some(line),
                                format!(
                                    "[{}] {}: upgrade count above 3 (S-0009 claim 40); our engine uses 3",
                                    show(name),
                                    show(&key)
                                ),
                            );
                        }
                        int(6)
                    }
                };
                let placed = Placed {
                    key: key.clone(),
                    owner,
                    type_name,
                    health: int(3).unwrap_or(0),
                    cell,
                    facing: facing.unwrap_or(0),
                    raw: fields
                        .iter()
                        .map(|f| f.clone().unwrap_or_default())
                        .collect(),
                };
                match kind {
                    ListKind::Vehicles => {
                        o.units.push(Unit { placed });
                        o.unit_fields.push(extra);
                    }
                    ListKind::Infantry => {
                        o.infantry.push(Infantry {
                            sub_cell: int(6).unwrap_or(0),
                            placed,
                        });
                        o.infantry_fields.push(extra);
                    }
                    ListKind::Aircraft => {
                        o.aircraft.push(Aircraft { placed });
                        o.aircraft_fields.push(extra);
                    }
                    ListKind::Buildings => {
                        o.structures.push(Structure { placed });
                        o.structure_fields.push(extra);
                    }
                }
            }
        }
        self.terrain_objects(&mut o, footprint, legacy);
        self.smudges(&mut o);
        o
    }

    /// The terrain-object list (S-0009 claim 42).
    fn terrain_objects(&mut self, o: &mut Objects, footprint: &Footprint, legacy: bool) {
        let Some(section) = self.doc.section(b"Terrain") else {
            return;
        };
        let entries: Vec<_> = section
            .list_positions()
            .map(|(e, w)| (e.key.clone(), e.line, w.value.clone()))
            .collect();
        // spec: S-0009 claim 42 (key decoded base 1000 from format version 4, base 128 below)
        let base = if legacy {
            LEGACY_CELL_TAG_BASE
        } else {
            CELL_NUMBER_BASE
        };
        for (key, line, value) in entries {
            // spec: S-0009 claim 42 (the key read as a leading integer)
            let (x, y) = decode_cell_number(value::decimal_integer(&key), base);
            // spec: S-0009 Formulas (object cell: each coordinate wrapped modulo 65536)
            let cell = CellXY::new(i32::from(wrap_coordinate(x)), i32::from(wrap_coordinate(y)));
            let name = value::text(&value, usize::MAX).to_vec();
            // spec: S-0009 claim 42 (the two "no tag" markers and an empty value place nothing)
            if name.is_empty()
                || name.eq_ignore_ascii_case(b"<none>")
                || name.eq_ignore_ascii_case(b"none")
            {
                continue;
            }
            if !footprint.is_legal(cell) {
                // spec: S-0009 claim 42 (a non-legal cell: our engine places nothing; the importer warns)
                self.warn(
                    Some(line),
                    format!(
                        "[Terrain] {}: the cell is not legal (S-0009 claim 42); nothing is placed",
                        show(&key)
                    ),
                );
                continue;
            }
            o.terrain.push(TerrainObject {
                cell,
                type_name: name,
            });
        }
    }

    /// The smudge list (S-0009 claim 42).
    fn smudges(&mut self, o: &mut Objects) {
        let Some(section) = self.doc.section(b"Smudge") else {
            return;
        };
        let q = self.ctx.quirks;
        let entries: Vec<_> = section
            .list_positions()
            .map(|(e, w)| (e.key.clone(), e.line, w.value.clone()))
            .collect();
        for (key, line, value) in entries {
            // spec: S-0009 claim 42 (fields type, cell x, cell y, piece number, read as in claim 34)
            let fields = split_entry(&value, q.object_empty_fields_collapse);
            let at = |i: usize| fields.get(i).cloned().flatten();
            let Some(type_name) = at(0) else {
                continue;
            };
            // spec: S-0009 claim 42 (no x field: nothing placed)
            let Some(x) = at(1) else {
                continue;
            };
            let Some(y) = at(2) else {
                // spec: S-0009 claim 42 (x but no y: unspecified, our engine places nothing; the importer warns)
                self.warn(
                    Some(line),
                    format!(
                        "[Smudge] {}: no cell y (S-0009 claim 42); nothing is placed",
                        show(&key)
                    ),
                );
                continue;
            };
            // spec: S-0009 claim 42 (a piece number present and not 0 places nothing)
            if at(3).is_some_and(|p| value::decimal_integer(&p) != 0) {
                continue;
            }
            let (x, y) = (value::decimal_integer(&x), value::decimal_integer(&y));
            o.smudges.push(Smudge {
                type_name,
                // spec: S-0009 Formulas (object cell: a smudge's cell is always two fields, wrapped)
                cell: CellXY::new(i32::from(wrap_coordinate(x)), i32::from(wrap_coordinate(y))),
            });
        }
    }

    /// Cell tags (S-0009 claim 31): by cell, and in entry order (S-0011
    /// claim 34). A value naming no defined tag attaches an empty tag; the
    /// trigger import registers it (S-0011 claim 7) and warns.
    fn cell_tags(&mut self, footprint: &Footprint, format_version: i32) -> CellTags {
        let mut out = BTreeMap::new();
        let mut order = Vec::new();
        let Some(section) = self.doc.section(b"CellTags") else {
            return (out, order);
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
            // spec: S-0009 claim 31 and Import mapping (the value <none> or none attaches nothing)
            if tag.eq_ignore_ascii_case(b"<none>") || tag.eq_ignore_ascii_case(b"none") {
                continue;
            }
            if !(0..=MAX_STREAM_COORDINATE).contains(&x) {
                // spec: S-0009 claim 31 (the importer warns about a decoded x outside 0 … 511)
                self.warn(
                    Some(entry.line),
                    format!(
                        "[CellTags] {}: x lies outside 0 … 511 (S-0009 claim 31, quirks.cell_index_alias)",
                        show(&entry.key)
                    ),
                );
            }
            // spec: S-0009 claim 31 (the entry acts on its selected cell)
            let selected = selected_cell(
                CellXY::new(wrap_coordinate(x).into(), wrap_coordinate(y).into()),
                self.ctx.quirks.cell_index_alias,
            );
            let Some(CellXY { x, y }) = selected.filter(|&c| footprint.is_legal(c)) else {
                // spec: S-0009 Edge cases (a key whose selected cell is not legal: ignored, and the importer warns)
                self.warn(
                    Some(entry.line),
                    format!("[CellTags] {}: not a legal cell; ignored", show(&entry.key)),
                );
                continue;
            };
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
                    order.push((c, tag.clone()));
                    out.insert(c, tag);
                }
            }
        }
        (out, order)
    }
}

/// The largest coordinate of a stream position (S-0009 Terms).
const MAX_STREAM_COORDINATE: i32 = 511; // spec: S-0009 Terms (stream positions 0 … 511 per axis)

/// The selected cell of a coordinate pair (S-0009 Terms, Formulas): the
/// cell whose stream position index 512·y + x equals the pair's, or none
/// when that index lies outside 0 … 262143. With `alias` off
/// (`quirks.cell_index_alias`), a pair with x outside 0 … 511 selects no
/// cell.
pub fn selected_cell(c: CellXY, alias: bool) -> Option<CellXY> {
    if !alias && !(0..=MAX_STREAM_COORDINATE).contains(&c.x) {
        // spec: S-0009 Edge cases (quirks.cell_index_alias off: x outside 0 … 511 selects no cell)
        return None;
    }
    // spec: S-0009 Formulas (i = 512·y + x; 0 ≤ i ≤ 262143 selects (i rem 512, i quo 512))
    let i = 512 * i64::from(c.y) + i64::from(c.x);
    (0..=262_143)
        .contains(&i)
        .then(|| CellXY::new((i % 512) as i32, (i / 512) as i32))
}

/// Characters of an object or smudge entry's value that are read (S-0009
/// claim 34).
const ENTRY_CHARS: usize = 127; // spec: S-0009 Constants (object entry characters read)

/// Splits an object or smudge entry's value (S-0009 claim 34): only the
/// first 127 characters are read, the value is cut at every comma, and with
/// `collapse` (`quirks.object_empty_fields_collapse`) an empty field is no
/// field at all. Without it an empty field keeps its place as `None`.
/// Spaces belong to their field.
pub fn split_entry(value: &[u8], collapse: bool) -> Vec<Option<Vec<u8>>> {
    let v = &value[..value.len().min(ENTRY_CHARS)];
    v.split(|&b| b == b',')
        .filter(|f| !(collapse && f.is_empty()))
        .map(|f| (!f.is_empty()).then(|| f.to_vec()))
        .collect()
}

/// Field `f` (numbered from 1 in the layout of format version 4 and above,
/// S-0009 claim 35) of a split entry. Below format version 4 fields 4 and 5
/// are one cell field (returned for 4, none for 5) and every later field
/// moves one place forward.
fn entry_field(fields: &[Option<Vec<u8>>], f: usize, legacy: bool) -> Option<Vec<u8>> {
    // spec: S-0009 claim 35 (below format version 4, one cell field; later fields move one place forward)
    let i = match (legacy, f) {
        (_, 1..=4) => f - 1,
        (true, 5) => return None,
        (true, _) => f - 2,
        (false, _) => f - 1,
    };
    fields.get(i).cloned().flatten()
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

/// How a decoded preview stream fits its image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PixelFit {
    /// Exactly the pixels' bytes.
    Exact,
    /// It ends before the last pixel.
    Short,
    /// It holds more than the pixels need.
    Long,
}

/// The preview's pixels from its decoded stream (S-0009 claim 24, S-0012
/// claim 8). `width × height × 3` is within the decoding limit.
fn preview_pixels(width: u32, height: u32, data: &[u8]) -> (Vec<u8>, PixelFit) {
    let n = width as usize * height as usize * 3;
    let mut rgb = vec![0u8; n];
    // spec: S-0009 claim 24 (only pixels whose three values are present show their colour; the rest black)
    let complete = data.len().min(n) / 3 * 3;
    rgb[..complete].copy_from_slice(&data[..complete]);
    let fit = match data.len().cmp(&n) {
        std::cmp::Ordering::Less => PixelFit::Short,
        std::cmp::Ordering::Equal => PixelFit::Exact,
        // spec: S-0012 claim 8 (an overlong preview is ignored beyond its required bytes)
        std::cmp::Ordering::Greater => PixelFit::Long,
    };
    (rgb, fit)
}

/// A map's embedded preview as the map list keeps it: its size and its
/// block's packed bytes, expanded only when the preview is shown, so that a
/// list of hundreds of maps holds no pictures (design: map picker (lead's
/// request 2026-11-13)).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreviewSource {
    /// Width and height: the third and fourth numbers of the preview size
    /// entry (S-0009 claim 24).
    pub size: (u32, u32),
    /// The preview block's text decoded (S-0012 claims 1 and 2), its chunks
    /// not yet expanded.
    pub packed: Vec<u8>,
}

impl PreviewSource {
    /// The preview: the block's valid chunks expanded (S-0012 claims 9 and
    /// 10), the pixels read as S-0009 claim 24 says; a stream that ends
    /// early leaves the rest black. Gives what [`decode`] gives as the
    /// map's preview.
    pub fn decode(&self) -> Preview {
        let (data, _) = unpack_prefix(&self.packed, PackCodec::Lzo);
        let (width, height) = self.size;
        let (rgb, _) = preview_pixels(width, height, &data);
        Preview { width, height, rgb }
    }
}

/// What a map list shows of a map besides its name and start count: read
/// from the document without decoding the rest of the map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListFacts {
    /// Width and height of the map size entry (S-0009 claim 30).
    pub size: (i32, i32),
    /// The theater (S-0009 Import mapping and Edge cases: absent or unknown
    /// is temperate).
    pub theater: Theater,
    /// The header block (S-0009 claim 25): the preview's markers.
    pub header: Header,
    /// The embedded preview, when the map has one: a valid preview size
    /// (S-0009 claim 30, S-0012 Edge cases) and a preview block with
    /// packed bytes (S-0009 Import mapping, `[PreviewPack]` absent: no
    /// preview).
    pub preview: Option<PreviewSource>,
}

/// The map-list facts of `doc` under `quirks` ([`ListFacts`]). Warnings
/// are not kept: the map is decoded in full, with its warnings, when a game
/// starts on it.
pub fn list_facts(doc: &Document, quirks: MapQuirks) -> ListFacts {
    let ctx = DecodeContext {
        quirks,
        ..DecodeContext::default()
    };
    let mut d = Decoder {
        doc,
        ctx: &ctx,
        warnings: Vec::new(),
    };
    // spec: S-0009 claim 30 ([Map] Size, numbers not supplied come from 1,1,50,50)
    let size = value::rect(
        doc.value(b"Map", b"Size"),
        SIZE_FALLBACK,
        quirks.absent_rect_is_zero,
    );
    let theater = d.theater();
    let header = d.header();
    let preview = d.preview_size().and_then(|size| {
        let packed = d.pack_text(b"PreviewPack")?;
        Some(PreviewSource { size, packed })
    });
    ListFacts {
        size: (size[2], size[3]),
        theater,
        header,
        preview,
    }
}

pub mod synth;
#[cfg(test)]
mod tests;
mod triggers;
