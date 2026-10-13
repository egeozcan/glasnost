//! Voxel lighting table (VPL): for each shade index and each voxel colour
//! index, the palette index to draw (S-0105 claims 19 and 38).
//!
//! The file layout is S-0013's: a header of four little-endian `u32`
//! values (remap-range start, remap-range end, row count *R*, an extra
//! value), a 768-byte palette (kept as stored, never used for drawing),
//! then rows of 256 bytes from position 784. Our engine keeps at most the
//! first 128 rows; the supplied row bytes are the first
//! min(L − 784, 256 · min(R, 128)) bytes after the palette, a partial row
//! included, and every unsupplied entry is zero. A file shorter than header
//! and palette supplies no rows: [`Vpl::parse`] rejects it and the caller
//! uses [`Vpl::empty`] (S-0013 claim 4, S-0105 claim 38).

use crate::bytes::{PutLe, Reader};
use crate::error::{Error, Result};
use crate::fixture::FixtureWriter;
use crate::pal::{PAL_LEN, Palette6};

/// Header length: four `u32` values.
pub const HEADER_LEN: usize = 16; // spec: S-0013 Constants (header extent)
/// Entries per row: one per voxel colour index (S-0105 claim 19).
pub const ROW_LEN: usize = 256; // spec: S-0105 Import mapping (each row has 256 entries)
/// Rows kept from a table (S-0105 claim 38, Constants: our engine's limit).
pub const MAX_ROWS: usize = 128; // spec: S-0013 Constants (retained row limit), S-0105 Constants
/// Header values below this row count leave rows undeclared (an importer
/// warning, S-0013 Import mapping).
pub const USUAL_ROWS: usize = 32; // spec: S-0013 Import mapping (R below 32 warns)

/// A parsed voxel lighting table (S-0013).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vpl {
    /// The four header values as read; the third is the row count (S-0013
    /// claim 1).
    pub header: [u32; 4],
    /// The palette the file carries, as stored (no effect, S-0105 claim
    /// 38).
    pub palette: Palette6,
    /// Rows by shade index, at most 128; entry `χ` of row `q` is the
    /// palette index drawn for voxel colour `χ` at shade `q` (S-0105 claim
    /// 19). A row the file ends inside is padded with zeros (claim 38).
    pub rows: Vec<[u8; ROW_LEN]>,
    /// The row count the file states (before the limit of 128).
    pub stated_rows: usize,
    /// The file ends before the last of its rows (an importer warning,
    /// S-0105 claim 38).
    pub short: bool,
    /// Bytes follow the last declared row (ignored, with an importer
    /// warning; S-0013 claim 5).
    pub trailing: bool,
}

impl Vpl {
    /// A table without rows: every entry reads as zero (S-0105 claim 38,
    /// a missing file).
    pub fn empty() -> Self {
        Self {
            header: [0; 4],
            palette: Palette6::from_palette(&crate::image::Palette::default()),
            rows: Vec::new(),
            stated_rows: 0,
            short: false,
            trailing: false,
        }
    }

    /// Parses a VPL file (S-0013). A file shorter than its header and
    /// palette is an error: it supplies no rows (claim 4).
    pub fn parse(data: &[u8]) -> Result<Self> {
        // spec: S-0013 claim 4 (fewer than 784 bytes: no row entries)
        let body = data
            .len()
            .checked_sub(HEADER_LEN + PAL_LEN)
            .ok_or(Error::Truncated {
                needed: (HEADER_LEN + PAL_LEN) as u64,
            })?;
        let mut r = Reader::new(data);
        // spec: S-0013 claim 1 (four unsigned 32-bit little-endian values)
        let header = [r.u32()?, r.u32()?, r.u32()?, r.u32()?];
        // spec: S-0013 claim 1 (the row count is always positions 8–11)
        let declared = u64::from(header[2]);
        // spec: S-0013 claim 2 (positions 16–783: the palette, as stored)
        let palette = Palette6::parse(r.bytes(PAL_LEN)?)?;
        let rest = &data[HEADER_LEN + PAL_LEN..];
        // spec: S-0013 Formulas (supplied row bytes min(L − 784, 256 · min(R, 128)), never wrapped)
        let kept = declared.min(MAX_ROWS as u64) as usize;
        let supplied = body.min(kept * ROW_LEN);
        let mut rows = Vec::with_capacity(kept.min(supplied / ROW_LEN + 1));
        // spec: S-0013 claim 3 (row q at 784 + 256q), claim 4 (a partial row supplies its own entries)
        for chunk in rest[..supplied].chunks(ROW_LEN) {
            let mut row = [0u8; ROW_LEN];
            row[..chunk.len()].copy_from_slice(chunk);
            rows.push(row);
        }
        let exact = declared * ROW_LEN as u64;
        Ok(Self {
            header,
            palette,
            rows,
            stated_rows: usize::try_from(declared).unwrap_or(usize::MAX),
            // spec: S-0013 Import mapping (the short-content comparison uses exact 256R)
            short: exact > body as u64,
            // spec: S-0013 claim 5 (bytes after the declared rows have no effect)
            trailing: (body as u64) > exact,
        })
    }

    /// The first header value, as read: taken as the start of the remap range;
    /// S-0013 claim 1, S-0105 claim 38: no effect on drawing.
    pub fn remap_start(&self) -> u32 {
        self.header[0]
    }

    /// The second header value, as read: taken as the end of the remap range;
    /// S-0013 claim 1, S-0105 claim 38: no effect on drawing.
    pub fn remap_end(&self) -> u32 {
        self.header[1]
    }

    /// The palette index for voxel colour `colour` at shade `shade` (S-0105
    /// claim 19); 0, transparent, for a row the file does not supply (claim
    /// 38, `quirks.vpl_unloaded_rows_blank`).
    pub fn get(&self, shade: u8, colour: u8) -> u8 {
        self.rows
            .get(usize::from(shade))
            .map_or(0, |row| row[usize::from(colour)])
    }

    /// Serialises the table in S-0013's layout. The header is written as
    /// stored; its third value should be the row count.
    pub fn write(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(HEADER_LEN + PAL_LEN + self.rows.len() * ROW_LEN);
        for &h in &self.header {
            out.put_u32(h);
        }
        out.extend(self.palette.write());
        for row in &self.rows {
            out.extend_from_slice(row);
        }
        out
    }
}

/// A synthetic table of `rows` shade rows with invented content: row `q`
/// maps colour `χ` to `(χ + q) mod 256`, keeping 0 at 0. The row count is
/// the third header value (S-0013 claim 1).
pub fn synthetic_table(rows: usize) -> Vpl {
    let mut out = Vec::with_capacity(rows);
    for q in 0..rows {
        let mut row = [0u8; ROW_LEN];
        for (c, v) in row.iter_mut().enumerate().skip(1) {
            *v = ((c + q) % ROW_LEN) as u8;
        }
        out.push(row);
    }
    Vpl {
        header: [16, 31, rows as u32, 0],
        palette: Palette6::from_palette(&crate::synth::ramp_palette(2)),
        rows: out,
        stated_rows: rows,
        short: false,
        trailing: false,
    }
}

struct SyntheticVpl;

impl FixtureWriter for SyntheticVpl {
    fn format(&self) -> &'static str {
        "vpl"
    }
    fn name(&self) -> String {
        "shift-rows".into()
    }
    fn extension(&self) -> &'static str {
        "vpl"
    }
    fn write(&self, out: &mut Vec<u8>) {
        out.extend(synthetic_table(32).write());
    }
}

/// Fixture writers for this module.
pub fn fixture_writers() -> Vec<Box<dyn FixtureWriter>> {
    vec![Box::new(SyntheticVpl)]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// spec: S-0013 claims 1 to 3 (layout round trip); S-0105 claim 38 (row q gives shade q)
    #[test]
    fn round_trip() {
        let t = synthetic_table(32);
        let bytes = t.write();
        assert_eq!(bytes.len(), 16 + 768 + 32 * 256);
        let back = Vpl::parse(&bytes).unwrap();
        assert_eq!(back, t);
        assert_eq!(back.write(), bytes);
        assert_eq!((back.remap_start(), back.remap_end()), (16, 31));
        assert_eq!(back.get(3, 10), 13);
        assert_eq!(back.get(31, 0), 0);
        // spec: S-0013 claim 3 (entry (q, χ) at 784 + 256q + χ)
        assert_eq!(bytes[784 + 256 * 3 + 10], 13);
    }

    /// spec: S-0013 claim 2 (256 three-byte colour entries, each component one whole byte, kept as stored)
    #[test]
    fn palette_components_are_whole_bytes() {
        let mut bytes = synthetic_table(1).write();
        bytes[16..19].copy_from_slice(&[64, 200, 255]);
        bytes[16 + 3 * 255 + 2] = 128;
        let t = Vpl::parse(&bytes).unwrap();
        assert_eq!(t.palette.colors[0], [64, 200, 255]);
        assert_eq!(t.palette.colors[255][2], 128);
        assert_eq!(t.write(), bytes);
    }

    /// spec: S-0013 claim 1 (the row count is the value at positions 8–11, whatever the others hold)
    #[test]
    fn row_count_is_the_third_value() {
        let mut t = synthetic_table(5);
        t.header = [5, 6, 3, 5];
        let v = Vpl::parse(&t.write()).unwrap();
        assert_eq!((v.stated_rows, v.rows.len()), (3, 3));
        assert!(v.trailing);
        assert_eq!(v.get(3, 10), 0, "rows past R are not supplied");
    }

    /// spec: S-0105 claim 38, S-0013 claim 4 (rows the file does not supply read as zero)
    #[test]
    fn missing_rows_read_zero() {
        let t = synthetic_table(4);
        let back = Vpl::parse(&t.write()).unwrap();
        assert_eq!(back.get(3, 10), 13);
        assert_eq!(back.get(4, 10), 0);
        assert_eq!(Vpl::empty().get(0, 77), 0);
    }

    /// spec: S-0013 claim 4 (a file that ends part-way through a row: its supplied entries, the rest zero)
    #[test]
    fn short_file() {
        let mut bytes = synthetic_table(4).write();
        bytes.truncate(16 + 768 + 2 * 256 + 20);
        let v = Vpl::parse(&bytes).unwrap();
        assert!(v.short);
        assert_eq!(v.stated_rows, 4);
        assert_eq!(v.rows.len(), 3);
        assert_eq!(v.get(2, 19), 21);
        assert_eq!(v.get(2, 20), 0);
        assert_eq!(v.get(3, 10), 0);
        // spec: S-0013 claim 4 (shorter than header and palette, or ending at 784: no rows)
        assert!(Vpl::parse(&bytes[..783]).is_err());
        assert!(Vpl::parse(&bytes[..10]).is_err());
        assert!(Vpl::parse(&bytes[..784]).unwrap().rows.is_empty());
    }

    /// spec: S-0013 claim 5 (bytes after the declared rows have no effect)
    #[test]
    fn long_file() {
        let mut t = synthetic_table(4);
        t.header[2] = 2;
        let v = Vpl::parse(&t.write()).unwrap();
        assert_eq!(v.rows.len(), 2);
        assert!(v.trailing && !v.short);
        assert_eq!(v.get(2, 10), 0);
    }

    /// spec: S-0013 claim 6, S-0105 claim 38 (at most the first 128 rows, never a wrapped length)
    #[test]
    fn at_most_128_rows() {
        let t = synthetic_table(130);
        let v = Vpl::parse(&t.write()).unwrap();
        assert_eq!(v.stated_rows, 130);
        assert_eq!(v.rows.len(), MAX_ROWS);
        assert_eq!(v.get(129, 5), 0);
        // 16777216 rows: our engine never wraps 256R to 0; the 128 available rows are kept
        let mut t = synthetic_table(130);
        t.header[2] = 16_777_216;
        let v = Vpl::parse(&t.write()).unwrap();
        assert_eq!(v.rows.len(), MAX_ROWS);
        assert!(v.short);
    }

    #[test]
    fn hostile_input_never_panics() {
        let bytes = synthetic_table(3).write();
        for cut in 0..bytes.len() {
            let _ = Vpl::parse(&bytes[..cut]);
        }
        let mut huge = bytes.clone();
        huge[..16].copy_from_slice(&[0xFF; 16]);
        let v = Vpl::parse(&huge).unwrap();
        assert!(v.rows.len() <= MAX_ROWS);
    }
}
