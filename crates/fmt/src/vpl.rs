//! Voxel lighting table (VPL): for each shade index and each voxel colour
//! index, the palette index to draw (S-0105 claims 19 and 38).
//!
//! S-0105 claim 38 fixes how the content is used: a row count *R*, a remap
//! range, one more value and a palette, none of which but *R* has an
//! effect, then *R* rows of 256 entries; rows the file does not supply read
//! as zero (a file that ends part-way through a row supplies that row's
//! first entries), and only the first 128 rows are kept. Where each part
//! lies in the file is "the format's own"; no format source this project
//! may read gives it, so the layout stays provisional (Q-0022):
//!
//! - a header of four little-endian `u32` values,
//! - a 768-byte palette (kept as stored, never used for drawing),
//! - rows of 256 bytes,
//!
//! with the row count taken as the first header value for which the length
//! is exactly `16 + 768 + 256 · R`, or else the largest for which the file
//! ends before its last row (every row the file supplies is then kept; the
//! rows after it read as zero whichever value is the count). A file shorter than header and palette, or one
//! no header value fits, is rejected; the caller then uses
//! [`Vpl::empty`], whose rows all read as zero (claim 38).

use crate::bytes::{PutLe, Reader};
use crate::error::{Error, Result};
use crate::fixture::FixtureWriter;
use crate::pal::{PAL_LEN, Palette6};

/// Header length: four `u32` values (provisional, Q-0022).
pub const HEADER_LEN: usize = 16;
/// Entries per row: one per voxel colour index (S-0105 claim 19).
pub const ROW_LEN: usize = 256; // spec: S-0105 Import mapping (each row has 256 entries)
/// Rows kept from a table (S-0105 claim 38, Constants: our engine's limit).
pub const MAX_ROWS: usize = 128; // spec: S-0105 Constants (rows kept from a lighting table file)

/// A parsed voxel lighting table. Provisional layout (Q-0022).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vpl {
    /// The four header values as read. One of them is the row count.
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
        }
    }

    /// Parses a VPL file (provisional layout, Q-0022).
    pub fn parse(data: &[u8]) -> Result<Self> {
        let body = data
            .len()
            .checked_sub(HEADER_LEN + PAL_LEN)
            .ok_or(Error::Truncated {
                needed: (HEADER_LEN + PAL_LEN) as u64,
            })?;
        let mut r = Reader::new(data);
        let header = [r.u32()?, r.u32()?, r.u32()?, r.u32()?];
        let rows_of = |h: u32| usize::try_from(h).ok();
        // provisional (Q-0022): the first header value whose row count fits the length exactly,
        // else the largest for which the file ends before its last row
        let n = header
            .iter()
            .filter_map(|&h| rows_of(h))
            .find(|&h| h.checked_mul(ROW_LEN) == Some(body))
            .or_else(|| {
                header
                    .iter()
                    .filter_map(|&h| rows_of(h))
                    .filter(|&h| h.checked_mul(ROW_LEN).is_none_or(|l| l > body))
                    .max()
            })
            .ok_or(Error::Invalid(
                "VPL header gives no row count that fits the length",
            ))?;
        let palette = Palette6::parse(r.bytes(PAL_LEN)?)?;
        let rest = &data[HEADER_LEN + PAL_LEN..];
        // spec: S-0105 claim 38 (the first 128 rows; bytes after the last row are ignored)
        let kept = n.min(MAX_ROWS);
        let mut rows = Vec::with_capacity(kept.min(rest.len() / ROW_LEN + 1));
        for chunk in rest.chunks(ROW_LEN).take(kept) {
            // spec: S-0105 claim 38 (a row the file ends inside: its supplied entries, the rest zero)
            let mut row = [0u8; ROW_LEN];
            row[..chunk.len()].copy_from_slice(chunk);
            rows.push(row);
        }
        Ok(Self {
            header,
            palette,
            rows,
            stated_rows: n,
            short: n.saturating_mul(ROW_LEN) > body,
        })
    }

    /// The first header value, as read: taken as the start of the remap range;
    /// S-0105 claim 38: no effect on drawing (provisional position, Q-0022).
    pub fn remap_start(&self) -> u32 {
        self.header[0]
    }

    /// The second header value, as read: taken as the end of the remap range;
    /// S-0105 claim 38: no effect on drawing (provisional position, Q-0022).
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

    /// Serialises the table in the provisional layout (Q-0022). The header
    /// is written as stored; it should hold the row count somewhere for the
    /// result to parse.
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
/// maps colour `χ` to `(χ + q) mod 256`, keeping 0 at 0. The row count goes
/// in the third header value, an invented choice: the parser accepts it in
/// any of the four (Q-0022).
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

    /// Q-0022 (provisional layout round trip); S-0105 claim 38 (row q gives shade q)
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
    }

    /// spec: S-0105 claim 38 (rows the file does not supply read as zero)
    #[test]
    fn missing_rows_read_zero() {
        let t = synthetic_table(4);
        let back = Vpl::parse(&t.write()).unwrap();
        assert_eq!(back.get(3, 10), 13);
        assert_eq!(back.get(4, 10), 0);
        assert_eq!(Vpl::empty().get(0, 77), 0);
    }

    /// spec: S-0105 claim 38 (a file that ends part-way through a row: its supplied entries, the rest zero)
    #[test]
    fn short_file() {
        let mut bytes = synthetic_table(4).write();
        bytes.truncate(16 + 768 + 2 * 256 + 20);
        let v = Vpl::parse(&bytes).unwrap();
        assert!(v.short);
        // The row count is ambiguous in a short file; the supplied rows are kept either way.
        assert_eq!(v.stated_rows, 31);
        assert_eq!(v.rows.len(), 3);
        assert_eq!(v.get(2, 19), 21);
        assert_eq!(v.get(2, 20), 0);
        assert_eq!(v.get(3, 10), 0);
    }

    /// spec: S-0105 claim 38 (only the first 128 rows are kept; a departure)
    #[test]
    fn at_most_128_rows() {
        let t = synthetic_table(130);
        let v = Vpl::parse(&t.write()).unwrap();
        assert_eq!(v.stated_rows, 130);
        assert_eq!(v.rows.len(), MAX_ROWS);
        assert_eq!(v.get(129, 5), 0);
    }

    /// Q-0022 (the row count may sit in any header value)
    #[test]
    fn row_count_in_any_header_value() {
        for slot in 0..4 {
            let mut t = synthetic_table(5);
            t.header = [99, 98, 97, 96];
            t.header[slot] = 5;
            assert_eq!(Vpl::parse(&t.write()), Ok(t));
        }
    }

    /// Q-0022 (rejects files no header value fits)
    #[test]
    fn rejects_mismatches() {
        let mut t = synthetic_table(4);
        t.header = [1, 2, 3, 0];
        assert!(Vpl::parse(&t.write()).is_err());
        assert!(Vpl::parse(&[0; 100]).is_err());
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
