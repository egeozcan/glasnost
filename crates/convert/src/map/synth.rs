//! Synthetic map text for tests: our own invented maps, written with our
//! own pack writers (`glasnost_fmt::mappack`). Nothing here is stock data.

use std::fmt::Write as _;

use glasnost_fmt::mappack::{PackCodec, encode_base64_lines, pack};

/// One terrain record (S-0009 claim 7), written in the layout the decoder
/// reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record {
    /// x.
    pub x: u16,
    /// y.
    pub y: u16,
    /// Tile number.
    pub tile: i32,
    /// Sub-tile.
    pub sub_tile: u8,
    /// Height level as a byte.
    pub level: u8,
    /// Ice-growth flag byte.
    pub ice: u8,
}

/// Encodes records into terrain-stream bytes.
pub fn record_bytes(records: &[Record]) -> Vec<u8> {
    let mut out = Vec::new();
    for r in records {
        out.extend_from_slice(&r.x.to_le_bytes());
        out.extend_from_slice(&r.y.to_le_bytes());
        out.extend_from_slice(&r.tile.to_le_bytes());
        out.extend_from_slice(&[r.sub_tile, r.level, r.ice]);
    }
    out
}

/// Writes `[name]` holding `bytes` packed with `codec` as numbered base64
/// lines.
pub fn pack_section(name: &str, bytes: &[u8], codec: PackCodec) -> String {
    let mut s = format!("[{name}]\n");
    for (i, line) in encode_base64_lines(&pack(bytes, codec, 0x2000), 70)
        .iter()
        .enumerate()
    {
        let _ = writeln!(s, "{}={line}", i + 1);
    }
    s
}

/// A builder for synthetic map text.
#[derive(Clone, Debug, Default)]
pub struct MapText {
    /// Sections written before the packs, as raw text.
    pub ini: String,
    /// Terrain stream bytes (unpacked), if any.
    pub terrain: Option<Vec<u8>>,
    /// Overlay type stream (unpacked), if any.
    pub overlay: Option<Vec<u8>>,
    /// Overlay data stream (unpacked), if any.
    pub overlay_data: Option<Vec<u8>>,
    /// Preview stream (unpacked), if any.
    pub preview: Option<Vec<u8>>,
}

impl MapText {
    /// The map's text.
    pub fn text(&self) -> String {
        let mut s = self.ini.clone();
        if !s.ends_with('\n') {
            s.push('\n');
        }
        if let Some(t) = &self.terrain {
            s += &pack_section("IsoMapPack5", t, PackCodec::Lzo);
        }
        if let Some(t) = &self.overlay {
            s += &pack_section("OverlayPack", t, PackCodec::Lcw);
        }
        if let Some(t) = &self.overlay_data {
            s += &pack_section("OverlayDataPack", t, PackCodec::Lcw);
        }
        if let Some(t) = &self.preview {
            s += &pack_section("PreviewPack", t, PackCodec::Lzo);
        }
        s
    }
}

/// A full 512 × 512 overlay stream of `fill`, with `(x, y, value)` set.
pub fn overlay_stream(fill: u8, set: &[(usize, usize, u8)]) -> Vec<u8> {
    let mut v = vec![fill; 512 * 512];
    for &(x, y, val) in set {
        v[512 * y + x] = val;
    }
    v
}
