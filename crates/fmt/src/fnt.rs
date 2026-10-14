//! The game bitmap font (`fonT` variant): a full 16-bit text-unit mapping
//! and fixed-size one-bit glyph records (S-0014).
//!
//! The layout is S-0014's: a 28-byte header (the identifying bytes `fonT`,
//! then six little-endian `u32` values: an uninterpreted metadata value,
//! the row span *B*, the bitmap height *H*, the line pitch, the record
//! count *N* and the record span *S*), 65536 little-endian `u16` selectors
//! from byte 28, then *N* records of *S* bytes from byte 131100. A record is
//! one width byte *W* followed by *H* rows of *B* bytes, most significant
//! bit first; only the first *W* pixels of a row belong to the mask.
//!
//! A selector of zero means the text unit is missing; a positive selector
//! *v* selects glyph *v* − 1 (S-0014 claim 2). The fallback for a missing
//! unit is generated from the mapped capital X (claim 4): [`Font::fallback`].
//! How status text uses these is S-0213 claim 25's.
//!
//! A font outside S-0014's valid scope is rejected as a whole (claim 6):
//! another identifying sequence, an incomplete header, mapping or record
//! payload, any selector beyond the record count, a record span below
//! 1 + *HB*, any stored width beyond 8*B*, or no mapping for capital X. The
//! checks cover every selector and every record, used or not; nothing of a
//! rejected file is kept. The caller treats a rejected font as missing
//! (S-0213 claim 25).

use crate::bytes::{PutLe, Reader};
use crate::error::{Error, Result};
use crate::fixture::FixtureWriter;

/// The identifying bytes, decimal 102, 111, 110, 84.
pub const MAGIC: [u8; 4] = *b"fonT"; // spec: S-0014 claim 1 (identifying bytes)
/// Header length.
pub const HEADER_LEN: usize = 28; // spec: S-0014 Constants (header length)
/// Selectors in the mapping: one per 16-bit text unit.
pub const MAP_COUNT: usize = 65536; // spec: S-0014 Constants (text-unit mapping count)
/// Mapping length in bytes.
pub const MAP_LEN: usize = 2 * MAP_COUNT; // spec: S-0014 Constants (mapping length, two bytes per selector)
/// Where the first glyph record starts.
pub const RECORDS_START: usize = HEADER_LEN + MAP_LEN; // spec: S-0014 claim 3 (records start at file byte 131100)
/// The text unit whose glyph the fallback is generated from: capital X.
pub const FALLBACK_SOURCE: u16 = 88; // spec: S-0014 Constants (fallback source, text unit 88)
/// Pixels per mask byte, most significant bit first.
const PIXELS_PER_BYTE: usize = 8; // spec: S-0014 Constants (pixel packing)

/// A glyph's mask: `width` × `height` pixels, row-major, top row first;
/// `true` paints the foreground, `false` leaves the destination unchanged
/// (S-0014 claim 5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mask {
    /// Width in pixels.
    pub width: usize,
    /// Height in pixels: the font's common bitmap height.
    pub height: usize,
    /// The pixels, `bits[y * width + x]`.
    pub bits: Vec<bool>,
}

impl Mask {
    /// Pixel (`x`, `y`); false outside the mask.
    pub fn get(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height && self.bits[y * self.width + x]
    }
}

/// A parsed game font (S-0014).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Font {
    /// The value at byte 4, kept without a rendering role (S-0014 claim 1).
    pub metadata: u32,
    /// Bytes per bitmap row, *B*.
    pub row_span: usize,
    /// The common bitmap height *H*, pixels.
    pub height: usize,
    /// The line pitch, pixels (S-0014 claim 1): an imported text-layout
    /// metric, distinct from the bitmap height; status text clips each line
    /// to it (S-0213 claim 25).
    pub line_pitch: u32,
    /// Bytes per glyph record, *S*.
    pub record_span: usize,
    /// One selector per text unit (S-0014 claim 2).
    pub map: Vec<u16>,
    /// The *N* records, *S* bytes each, as stored.
    pub records: Vec<u8>,
    /// Bytes follow the last record (not part of the font).
    pub trailing: bool,
}

/// `a × b` as `usize`, or an error when it does not fit.
fn mul(a: usize, b: usize, what: &'static str) -> Result<usize> {
    a.checked_mul(b).ok_or(Error::TooLarge(what))
}

impl Font {
    /// An empty font with no glyphs and every selector zero; records get the
    /// minimum span 1 + *HB* (S-0014 Formulas). For building synthetic
    /// fonts.
    pub fn empty(row_span: usize, height: usize, line_pitch: u32) -> Self {
        Self {
            metadata: 0,
            row_span,
            height,
            line_pitch,
            // spec: S-0014 Formulas (minimum record span 1 + HB)
            record_span: 1 + height * row_span,
            map: vec![0; MAP_COUNT],
            records: Vec::new(),
            trailing: false,
        }
    }

    /// Parses a font file (S-0014 claims 1 to 3), rejecting the whole file
    /// when it is outside the valid scope (claim 6).
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        // spec: S-0014 claim 1 (the first four bytes spell fonT)
        if r.array::<4>()? != MAGIC {
            return Err(Error::Unsupported("font identifying bytes are not fonT"));
        }
        // spec: S-0014 claim 1 (six unsigned 32-bit little-endian values after the identifying bytes)
        let metadata = r.u32()?;
        let row_span = r.u32()? as usize;
        let height = r.u32()? as usize;
        let line_pitch = r.u32()?;
        let count = r.u32()? as usize;
        let record_span = r.u32()? as usize;
        // spec: S-0014 claim 2 (65536 unsigned 16-bit little-endian selectors immediately after the header)
        let map_bytes = r.bytes(MAP_LEN)?;
        let map: Vec<u16> = map_bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&p| u16::from_le_bytes(p))
            .collect();
        // spec: S-0014 claim 3 (records must hold the width byte and H × B row bytes; minimum span 1 + HB)
        let rows = mul(height, row_span, "font bitmap rows")?;
        if record_span < rows.saturating_add(1) {
            return Err(Error::Invalid("font record span cannot hold its mask"));
        }
        // spec: S-0014 Formulas (header, mapping and records occupy 131100 + NS bytes)
        let body = mul(count, record_span, "font records")?;
        let end = RECORDS_START
            .checked_add(body)
            .ok_or(Error::TooLarge("font records"))?;
        let records = r.bytes(body)?.to_vec();
        // spec: S-0014 Edge cases (selectors are zero or within 1 to N)
        if map.iter().any(|&v| usize::from(v) > count) {
            return Err(Error::Invalid("font selector beyond the record count"));
        }
        // spec: S-0014 claim 6 (an absent mapping for capital X rejects the entire font; a zero-width mapped X is present)
        if map[usize::from(FALLBACK_SOURCE)] == 0 {
            return Err(Error::Invalid("font maps no capital X (text unit 88)"));
        }
        let font = Self {
            metadata,
            row_span,
            height,
            line_pitch,
            record_span,
            map,
            records,
            trailing: data.len() > end,
        };
        // spec: S-0014 claim 3 (the width must fit within 8B pixels)
        let limit = row_span.saturating_mul(PIXELS_PER_BYTE);
        if (0..count).any(|j| usize::from(font.records[j * record_span]) > limit) {
            return Err(Error::Invalid("font glyph wider than its rows"));
        }
        Ok(font)
    }

    /// The number of glyph records, *N*.
    pub fn count(&self) -> usize {
        self.records
            .len()
            .checked_div(self.record_span)
            .unwrap_or(0)
    }

    /// Text unit `c`'s selector (S-0014 claim 2).
    pub fn selector(&self, c: u16) -> u16 {
        self.map.get(usize::from(c)).copied().unwrap_or(0)
    }

    /// The zero-based glyph number text unit `c` selects, or `None` when it
    /// is missing: a zero selector, never a blank bitmap or a zero width
    /// (S-0014 claims 2 and 4).
    pub fn glyph_number(&self, c: u16) -> Option<usize> {
        // spec: S-0014 claim 2 (zero denotes absence; a positive selector v selects zero-based glyph v − 1)
        match self.selector(c) {
            0 => None,
            v => Some(usize::from(v) - 1),
        }
    }

    /// Glyph `j`'s stored record, or `None` past the last.
    fn record(&self, j: usize) -> Option<&[u8]> {
        let start = j.checked_mul(self.record_span)?;
        self.records
            .get(start..start.checked_add(self.record_span)?)
    }

    /// Glyph `j`'s stored width in pixels (S-0014 claim 3).
    pub fn width(&self, j: usize) -> Option<u8> {
        // spec: S-0014 claim 3 (each record starts with one unsigned width byte)
        self.record(j).map(|r| r[0])
    }

    /// Glyph `j`'s mask: its stored width by the common height (S-0014
    /// claim 3). A zero-width glyph has no pixels, whatever its rows hold.
    pub fn mask(&self, j: usize) -> Option<Mask> {
        let rec = self.record(j)?;
        // spec: S-0014 claim 3 (only the first W pixels of each row belong to the mask; a zero-width glyph paints no pixels in our engine)
        let width = usize::from(rec[0]);
        if width == 0 {
            return Some(Mask {
                width,
                height: self.height,
                bits: Vec::new(),
            });
        }
        let mut bits = Vec::with_capacity(width * self.height);
        for y in 0..self.height {
            for x in 0..width {
                // spec: S-0014 Formulas (pixel (x, y) at byte 1 + yB + ⌊x/8⌋ of the record, bit 7 − (x mod 8))
                let byte = rec[1 + y * self.row_span + x / PIXELS_PER_BYTE];
                bits.push((byte >> (7 - x % PIXELS_PER_BYTE)) & 1 == 1);
            }
        }
        Some(Mask {
            width,
            height: self.height,
            bits,
        })
    }

    /// The generated fallback image: the complement of the mapped capital
    /// X's mask, with that glyph's stored width and the common height
    /// (S-0014 claim 4). `None` when X is not mapped, which a parsed font
    /// never is (claim 6).
    pub fn fallback(&self) -> Option<Mask> {
        // spec: S-0014 claim 4 (complement of the mapped capital X, text unit 88, within its stored width and the common height)
        let mut m = self.mask(self.glyph_number(FALLBACK_SOURCE)?)?;
        for b in &mut m.bits {
            *b = !*b;
        }
        Some(m)
    }

    /// Appends a glyph record of `width` pixels whose rows are `rows`: one
    /// string per row, top first, `#` a set pixel and anything else clear;
    /// pixels beyond `width` are stored too. Rows past the strings, and
    /// pixels past a string's end, are clear. Returns the glyph's number.
    /// For building synthetic fonts.
    pub fn push_glyph(&mut self, width: u8, rows: &[&str]) -> usize {
        let j = self.count();
        let mut rec = vec![0u8; self.record_span];
        rec[0] = width;
        for (y, row) in rows.iter().enumerate().take(self.height) {
            for (x, ch) in row
                .chars()
                .enumerate()
                .take(self.row_span * PIXELS_PER_BYTE)
            {
                if ch == '#' {
                    rec[1 + y * self.row_span + x / PIXELS_PER_BYTE] |=
                        0x80 >> (x % PIXELS_PER_BYTE);
                }
            }
        }
        self.records.extend(rec);
        j
    }

    /// Maps text unit `c` to glyph number `j` (selector `j + 1`), or to
    /// nothing with `None`.
    pub fn map_unit(&mut self, c: u16, j: Option<usize>) {
        self.map[usize::from(c)] = j.map_or(0, |j| (j + 1) as u16);
    }

    /// Serialises the font in S-0014's layout.
    pub fn write(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(RECORDS_START + self.records.len());
        out.extend_from_slice(&MAGIC);
        out.put_u32(self.metadata);
        out.put_u32(self.row_span as u32);
        out.put_u32(self.height as u32);
        out.put_u32(self.line_pitch);
        out.put_u32(self.count() as u32);
        out.put_u32(self.record_span as u32);
        for &v in &self.map {
            out.put_u16(v);
        }
        out.extend_from_slice(&self.records);
        out
    }
}

/// A small synthetic font with invented shapes: row span 2, height 5, line
/// pitch 7, a 5-pixel capital X, `A` and its alias `a`, a glyph for unit
/// 300 (whose low byte 44 maps elsewhere), a blank space and a zero-width
/// glyph for unit 1.
pub fn synthetic_font() -> Font {
    let mut f = Font::empty(2, 5, 7);
    let x = f.push_glyph(5, &["#...#", ".#.#.", "..#..", ".#.#.", "#...#"]);
    let a = f.push_glyph(4, &[".##.", "#..#", "####", "#..#", "#..#"]);
    let high = f.push_glyph(
        10,
        &["##########", "#........#", "#........#", "##########"],
    );
    let comma = f.push_glyph(2, &["", "", "", ".#", "#."]);
    let space = f.push_glyph(3, &[]);
    let zero = f.push_glyph(0, &["#"]);
    f.map_unit(88, Some(x));
    f.map_unit(65, Some(a));
    f.map_unit(97, Some(a));
    f.map_unit(300, Some(high));
    f.map_unit(44, Some(comma));
    f.map_unit(32, Some(space));
    f.map_unit(1, Some(zero));
    f
}

struct SyntheticFont;

impl FixtureWriter for SyntheticFont {
    fn format(&self) -> &'static str {
        "fnt"
    }
    fn name(&self) -> String {
        "small-font".into()
    }
    fn extension(&self) -> &'static str {
        "fnt"
    }
    fn write(&self, out: &mut Vec<u8>) {
        out.extend(synthetic_font().write());
    }
}

/// Fixture writers for this module.
pub fn fixture_writers() -> Vec<Box<dyn FixtureWriter>> {
    vec![Box::new(SyntheticFont)]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(m: &Mask) -> Vec<String> {
        (0..m.height)
            .map(|y| {
                (0..m.width)
                    .map(|x| if m.get(x, y) { '#' } else { '.' })
                    .collect()
            })
            .collect()
    }

    fn put_u32(bytes: &mut [u8], at: usize, v: u32) {
        bytes[at..at + 4].copy_from_slice(&v.to_le_bytes());
    }

    /// spec: S-0014 claim 1 (28-byte header: fonT, then six little-endian u32 values in the stated order)
    #[test]
    fn claim_1_header() {
        let mut f = synthetic_font();
        f.metadata = 0x0102_0304;
        let bytes = f.write();
        assert_eq!(&bytes[..4], &[102, 111, 110, 84]);
        assert_eq!(&bytes[4..8], &[4, 3, 2, 1]);
        assert_eq!(u32::from_le_bytes(bytes[8..12].try_into().unwrap()), 2);
        assert_eq!(u32::from_le_bytes(bytes[12..16].try_into().unwrap()), 5);
        assert_eq!(u32::from_le_bytes(bytes[16..20].try_into().unwrap()), 7);
        assert_eq!(u32::from_le_bytes(bytes[20..24].try_into().unwrap()), 6);
        assert_eq!(u32::from_le_bytes(bytes[24..28].try_into().unwrap()), 11);
        let back = Font::parse(&bytes).unwrap();
        assert_eq!(back, f);
        assert_eq!(back.write(), bytes);
        assert_eq!(
            (back.metadata, back.row_span, back.height, back.line_pitch),
            (0x0102_0304, 2, 5, 7),
            "bitmap height and line pitch are distinct values"
        );
        assert_eq!((back.count(), back.record_span), (6, 11));
        // A differing identifying sequence is not this variant.
        let mut other = bytes.clone();
        other[3] = b't';
        assert_eq!(
            Font::parse(&other),
            Err(Error::Unsupported("font identifying bytes are not fonT"))
        );
    }

    /// spec: S-0014 claim 2 (65536 selectors from byte 28 + 2c; zero is absence, v selects glyph v − 1; aliases; no low-byte reduction)
    #[test]
    fn claim_2_mapping() {
        let f = synthetic_font();
        let bytes = f.write();
        // Selector c at 28 + 2c, little-endian.
        assert_eq!(&bytes[28 + 2 * 88..28 + 2 * 88 + 2], &[1, 0]);
        assert_eq!(&bytes[28 + 2 * 300..28 + 2 * 300 + 2], &[3, 0]);
        assert_eq!(bytes.len(), RECORDS_START + 6 * 11);
        let f = Font::parse(&bytes).unwrap();
        assert_eq!(f.map.len(), 65536);
        assert_eq!(f.glyph_number(88), Some(0));
        // Aliases select the same glyph.
        assert_eq!(f.glyph_number(65), Some(1));
        assert_eq!(f.glyph_number(97), Some(1));
        // Unit 300 is not reduced to its low byte 44.
        assert_eq!(f.glyph_number(300), Some(2));
        assert_eq!(f.glyph_number(44), Some(3));
        // Zero selectors anywhere, including above and below populated ones.
        assert_eq!(f.glyph_number(0), None);
        assert_eq!(f.glyph_number(66), None);
        assert_eq!(f.glyph_number(65535), None);
        // The last unit's selector is the mapping's final two bytes.
        let mut top = synthetic_font();
        top.map_unit(65535, Some(4));
        let tb = top.write();
        assert_eq!(&tb[RECORDS_START - 2..RECORDS_START], &[5, 0]);
        assert_eq!(Font::parse(&tb).unwrap().glyph_number(65535), Some(4));
    }

    /// spec: S-0014 claim 3 (records at 131100 + jS; width byte; H rows of B bytes, top to bottom, most significant bit first; only W pixels belong to the mask)
    #[test]
    fn claim_3_records_and_pixels() {
        let mut f = Font::empty(2, 3, 4);
        // A 12-pixel glyph spanning both bytes of each row, asymmetric.
        let wide = f.push_glyph(12, &["#.........##", ".#......#...", "##.........#"]);
        // A narrow glyph with set bits beyond its width: not part of the mask.
        let narrow = f.push_glyph(3, &["#.##", ".#.#", "..##"]);
        f.map_unit(88, Some(wide));
        let bytes = f.write();
        let base = RECORDS_START + narrow * f.record_span;
        assert_eq!(bytes[RECORDS_START], 12);
        // Record 1 starts at 131100 + S.
        assert_eq!(bytes[base], 3);
        // Row 0 of the wide glyph: bits 7 − (x mod 8) of bytes ⌊x/8⌋.
        assert_eq!(
            &bytes[RECORDS_START + 1..RECORDS_START + 3],
            &[0b1000_0000, 0b0011_0000]
        );
        // Row 1 starts B bytes later.
        assert_eq!(
            &bytes[RECORDS_START + 3..RECORDS_START + 5],
            &[0b0100_0000, 0b1000_0000]
        );
        let f = Font::parse(&bytes).unwrap();
        assert_eq!(f.width(wide), Some(12));
        assert_eq!(
            rows(&f.mask(wide).unwrap()),
            ["#.........##", ".#......#...", "##.........#"]
        );
        assert_eq!(f.width(narrow), Some(3));
        assert_eq!(rows(&f.mask(narrow).unwrap()), ["#.#", ".#.", "..#"]);
        assert_eq!(f.mask(2), None);
    }

    /// spec: S-0014 claim 3 (a zero-width glyph paints no pixels in our engine, even with its first stored bit set)
    #[test]
    fn claim_3_zero_width_paints_nothing() {
        let f = Font::parse(&synthetic_font().write()).unwrap();
        let j = f.glyph_number(1).unwrap();
        let rec = RECORDS_START + j * f.record_span;
        assert_eq!(f.write()[rec + 1], 0x80, "first stored bit set");
        let m = f.mask(j).unwrap();
        assert_eq!((m.width, m.height), (0, 5));
        assert!(m.bits.is_empty());
        // Present, not missing.
        assert_eq!(f.glyph_number(1), Some(5));
    }

    /// spec: S-0014 claim 3 (bytes within S after the row bytes are not part of the bitmap)
    #[test]
    fn claim_3_record_padding_is_ignored() {
        let mut f = Font::empty(1, 2, 3);
        f.record_span = 6;
        let j = f.push_glyph(8, &["########", "#......#"]);
        f.map_unit(88, Some(j));
        let mut bytes = f.write();
        bytes[RECORDS_START + 3..RECORDS_START + 6].copy_from_slice(&[0xFF; 3]);
        let g = Font::parse(&bytes).unwrap();
        assert_eq!(rows(&g.mask(0).unwrap()), ["########", "#......#"]);
    }

    /// spec: S-0014 claim 4 (missing exactly when the selector is zero; a positive-width blank glyph is present)
    #[test]
    fn claim_4_missing_is_a_zero_selector() {
        let f = Font::parse(&synthetic_font().write()).unwrap();
        let space = f.glyph_number(32).unwrap();
        let m = f.mask(space).unwrap();
        assert_eq!(m.width, 3);
        assert!(m.bits.iter().all(|&b| !b), "blank yet present");
        assert_eq!(f.glyph_number(33), None);
    }

    /// spec: S-0014 claim 4 (the fallback is the complement of the mapped capital X within its stored width and the common height; changing X changes it)
    #[test]
    fn claim_4_fallback_from_capital_x() {
        let f = Font::parse(&synthetic_font().write()).unwrap();
        let fb = f.fallback().unwrap();
        assert_eq!(rows(&fb), [".###.", "#.#.#", "##.##", "#.#.#", ".###."]);
        // Changing the mapped X changes the fallback's shape and width; the
        // fallback follows unit 88's mapping, not a fixed record number.
        let mut g = synthetic_font();
        let x2 = g.push_glyph(3, &["#..", "", "", "", "..#"]);
        g.map_unit(88, Some(x2));
        let g = Font::parse(&g.write()).unwrap();
        let fb = g.fallback().unwrap();
        assert_eq!(rows(&fb), [".##", "###", "###", "###", "##."]);
        // No mapped X: no fallback; such a file is rejected (claim 6).
        let mut h = synthetic_font();
        h.map_unit(88, None);
        assert_eq!(h.fallback(), None);
        assert_eq!(
            Font::parse(&h.write()),
            Err(Error::Invalid("font maps no capital X (text unit 88)"))
        );
    }

    /// spec: S-0014 claims 4 and 6 (a zero-width mapped X is present and does not itself fail validation; its fallback has width 0)
    #[test]
    fn claim_6_zero_width_x_is_accepted() {
        let mut f = synthetic_font();
        let zero = f.glyph_number(1).unwrap();
        f.map_unit(88, Some(zero));
        let g = Font::parse(&f.write()).expect("a zero-width X is mapped");
        let fb = g.fallback().unwrap();
        assert_eq!((fb.width, fb.height), (0, 5));
        assert!(fb.bits.is_empty());
    }

    /// spec: S-0014 claim 6 and Scenario F5 (each single invalid change rejects the whole font,
    /// even where the change is in a selector or record no notice uses; the unchanged copy is the
    /// control)
    #[test]
    fn claim_6_whole_font_rejection() {
        let valid = synthetic_font().write();
        assert!(Font::parse(&valid).is_ok(), "the control parses");
        let n = synthetic_font().count();
        let mut cases: Vec<(&str, Vec<u8>)> = Vec::new();
        let mut b = valid.clone();
        b[0] = b'F';
        cases.push(("identifying bytes", b));
        cases.push(("header truncation", valid[..HEADER_LEN - 1].to_vec()));
        cases.push(("mapping truncation", valid[..RECORDS_START - 2].to_vec()));
        cases.push(("payload truncation", valid[..valid.len() - 1].to_vec()));
        // Unit 1000 is used by no notice; its selector names record N + 1.
        let mut b = valid.clone();
        b[28 + 2 * 1000..28 + 2 * 1000 + 2].copy_from_slice(&((n + 1) as u16).to_le_bytes());
        cases.push(("out-of-range unused selector", b));
        // A record span one byte below 1 + HB = 11.
        let mut b = valid.clone();
        put_u32(&mut b, 24, 10);
        cases.push(("too-small record span", b));
        // A record no selector references, wider than 8B = 16.
        let mut f = synthetic_font();
        f.push_glyph(5, &[]);
        let mut b = f.write();
        let last = RECORDS_START + n * f.record_span;
        b[last] = 17;
        assert!(
            f.map.iter().all(|&v| usize::from(v) != n + 1),
            "unreferenced"
        );
        cases.push(("over-wide unused record", b));
        let mut b = valid.clone();
        b[28 + 2 * 88..28 + 2 * 88 + 2].copy_from_slice(&[0, 0]);
        cases.push(("zero X selector", b));
        for (name, bytes) in cases {
            assert!(Font::parse(&bytes).is_err(), "{name}");
        }
    }

    /// spec: S-0014 Edge cases (truncated files, out-of-range selectors and dimensions that cannot hold the mask are outside the valid scope)
    #[test]
    fn out_of_scope_inputs_are_rejected() {
        let bytes = synthetic_font().write();
        assert!(matches!(
            Font::parse(&bytes[..27]),
            Err(Error::Truncated { .. })
        ));
        assert!(matches!(
            Font::parse(&bytes[..RECORDS_START - 1]),
            Err(Error::Truncated { .. })
        ));
        assert!(matches!(
            Font::parse(&bytes[..bytes.len() - 1]),
            Err(Error::Truncated { .. })
        ));
        // A selector beyond N.
        let mut b = bytes.clone();
        b[28 + 2 * 50..28 + 2 * 50 + 2].copy_from_slice(&7u16.to_le_bytes());
        assert_eq!(
            Font::parse(&b),
            Err(Error::Invalid("font selector beyond the record count"))
        );
        // A record span below 1 + HB.
        let mut b = bytes.clone();
        put_u32(&mut b, 24, 10);
        assert_eq!(
            Font::parse(&b),
            Err(Error::Invalid("font record span cannot hold its mask"))
        );
        // A width beyond 8B.
        let mut b = bytes.clone();
        b[RECORDS_START] = 17;
        assert_eq!(
            Font::parse(&b),
            Err(Error::Invalid("font glyph wider than its rows"))
        );
        b[RECORDS_START] = 16;
        assert!(Font::parse(&b).is_ok(), "exactly 8B fits");
        // Bytes after the records are not part of the font.
        let mut b = bytes.clone();
        b.extend([1, 2, 3]);
        let f = Font::parse(&b).unwrap();
        assert!(f.trailing);
        assert_eq!(f.write(), bytes);
    }

    #[test]
    fn hostile_input_never_panics() {
        let bytes = synthetic_font().write();
        for cut in (0..bytes.len()).step_by(997) {
            let _ = Font::parse(&bytes[..cut]);
        }
        for at in [8, 12, 20, 24] {
            for v in [0, 1, u32::MAX, 0x8000_0000] {
                let mut b = bytes.clone();
                put_u32(&mut b, at, v);
                if let Ok(f) = Font::parse(&b) {
                    for j in 0..f.count() {
                        let _ = f.mask(j);
                    }
                    let _ = f.fallback();
                }
            }
        }
    }
}
