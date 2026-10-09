//! Voxel lighting table (VPL): for each shade index and each voxel colour
//! index, the palette index to draw (S-0105 claim 19).
//!
//! **Provisional (Q-0014).** No spec gives the byte layout. S-0105 claim 19
//! and its Import mapping say only that the header gives the remap range and
//! the number of rows, that the file carries a palette not used for drawing,
//! and that each row has 256 entries, one per voxel colour index. Until a
//! formats spec answers Q-0014, this module reads the file as:
//!
//! - a header of four little-endian `u32` values,
//! - a 768-byte palette (kept as stored, never used for drawing),
//! - rows of 256 bytes,
//!
//! and accepts it only when one of the four header values equals the row
//! count *N* for which the file length is exactly `16 + 768 + 256 · N`.
//! Which header value that is, is not recorded. [`Vpl::remap_start`] and
//! [`Vpl::remap_end`] return the first two header values as read, and claim
//! nothing about them beyond S-0105's "the header gives the remap range".
//! Files that do not match are rejected; callers then draw voxels without
//! the table (S-0105 claim 19 has no fallback; see Q-0014).

use crate::bytes::{PutLe, Reader};
use crate::error::{Error, Result};
use crate::fixture::FixtureWriter;
use crate::pal::{PAL_LEN, Palette6};

/// Header length: four `u32` values (provisional, Q-0014).
pub const HEADER_LEN: usize = 16;
/// Entries per row: one per voxel colour index (S-0105 claim 19).
pub const ROW_LEN: usize = 256; // spec: S-0105 Import mapping (each row has 256 entries)

/// A parsed voxel lighting table. Provisional layout (Q-0014).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vpl {
    /// The four header values as read. One of them equals the row count.
    pub header: [u32; 4],
    /// The palette the file carries, as stored (not used for drawing,
    /// S-0105 Import mapping).
    pub palette: Palette6,
    /// Rows by shade index; entry `χ` of row `q` is the palette index drawn
    /// for voxel colour `χ` at shade `q` (S-0105 claim 19).
    pub rows: Vec<[u8; ROW_LEN]>,
}

impl Vpl {
    /// Parses a VPL file (provisional layout, Q-0014).
    pub fn parse(data: &[u8]) -> Result<Self> {
        let body = data
            .len()
            .checked_sub(HEADER_LEN + PAL_LEN)
            .ok_or(Error::Truncated {
                needed: (HEADER_LEN + PAL_LEN) as u64,
            })?;
        if body % ROW_LEN != 0 {
            return Err(Error::Invalid("VPL length is not a whole number of rows"));
        }
        let n = body / ROW_LEN;
        let mut r = Reader::new(data);
        let header = [r.u32()?, r.u32()?, r.u32()?, r.u32()?];
        if !header
            .iter()
            .any(|&h| usize::try_from(h).is_ok_and(|h| h == n))
        {
            return Err(Error::Invalid(
                "VPL header gives no row count matching the length",
            ));
        }
        let palette = Palette6::parse(r.bytes(PAL_LEN)?)?;
        let mut rows = Vec::with_capacity(n);
        for _ in 0..n {
            rows.push(r.array()?);
        }
        Ok(Self {
            header,
            palette,
            rows,
        })
    }

    /// The first header value, as read: the start of the remap range by
    /// S-0105 claim 19, provisionally (Q-0014).
    pub fn remap_start(&self) -> u32 {
        self.header[0]
    }

    /// The second header value, as read: the end of the remap range by
    /// S-0105 claim 19, provisionally (Q-0014).
    pub fn remap_end(&self) -> u32 {
        self.header[1]
    }

    /// The palette index for voxel colour `colour` at shade `shade`, or
    /// `None` when the table has no such row (S-0105 claim 19).
    pub fn get(&self, shade: u8, colour: u8) -> Option<u8> {
        self.rows
            .get(usize::from(shade))
            .map(|row| row[usize::from(colour)])
    }

    /// Serialises the table in the provisional layout (Q-0014). The header
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
/// any of the four (Q-0014).
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

    /// Q-0014 (provisional layout round trip)
    #[test]
    fn round_trip() {
        let t = synthetic_table(32);
        let bytes = t.write();
        assert_eq!(bytes.len(), 16 + 768 + 32 * 256);
        let back = Vpl::parse(&bytes).unwrap();
        assert_eq!(back, t);
        assert_eq!(back.write(), bytes);
        assert_eq!((back.remap_start(), back.remap_end()), (16, 31));
        assert_eq!(back.get(3, 10), Some(13));
        assert_eq!(back.get(31, 0), Some(0));
        assert_eq!(back.get(32, 10), None);
    }

    /// Q-0014 (the row count may sit in any header value)
    #[test]
    fn row_count_in_any_header_value() {
        for slot in 0..4 {
            let mut t = synthetic_table(5);
            t.header = [99, 98, 97, 96];
            t.header[slot] = 5;
            assert_eq!(Vpl::parse(&t.write()), Ok(t));
        }
    }

    /// Q-0014 (rejects files whose header does not match the length)
    #[test]
    fn rejects_mismatches() {
        let mut t = synthetic_table(4);
        t.header = [16, 31, 3, 0];
        assert!(Vpl::parse(&t.write()).is_err());
        let mut bytes = synthetic_table(4).write();
        bytes.push(0);
        assert!(Vpl::parse(&bytes).is_err());
        // Zero rows, with a zero header value, is accepted.
        let empty = synthetic_table(0);
        assert_eq!(Vpl::parse(&empty.write()), Ok(empty));
    }

    #[test]
    fn hostile_input_never_panics() {
        let bytes = synthetic_table(3).write();
        for cut in 0..bytes.len() {
            // A cut at a row boundary is a valid shorter table when a header
            // value happens to equal the shorter row count (here the zero).
            let at_row = cut >= 784 && (cut - 784) % 256 == 0;
            let parsed = Vpl::parse(&bytes[..cut]);
            assert_eq!(parsed.is_ok(), at_row && cut == 784, "cut {cut}");
        }
        let mut huge = bytes.clone();
        huge[..16].copy_from_slice(&[0xFF; 16]);
        assert!(Vpl::parse(&huge).is_err());
    }
}
