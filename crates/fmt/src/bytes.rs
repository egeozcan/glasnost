//! Bounds-checked little-endian reading and writing helpers.
//!
//! Every read goes through [`Reader`] or [`slice`], which return
//! [`Error::Truncated`] instead of panicking, so parsers built on them stay
//! panic-free on hostile input.

use crate::error::{Error, Result};

/// Converts a length or offset to `u64` for error reporting.
pub(crate) fn wide(n: usize) -> u64 {
    n as u64
}

/// `data[offset..offset + len]`, or [`Error::Truncated`].
pub fn slice(data: &[u8], offset: usize, len: usize) -> Result<&[u8]> {
    let end = offset
        .checked_add(len)
        .ok_or(Error::Truncated { needed: u64::MAX })?;
    data.get(offset..end)
        .ok_or(Error::Truncated { needed: wide(end) })
}

/// Same as [`slice`] with `u64` offset and length (archive coordinates).
pub fn slice64(data: &[u8], offset: u64, len: u64) -> Result<&[u8]> {
    let too_big = Error::Truncated { needed: u64::MAX };
    let offset = usize::try_from(offset).map_err(|_| too_big)?;
    let len = usize::try_from(len).map_err(|_| too_big)?;
    slice(data, offset, len)
}

/// A forward cursor over a byte slice.
#[derive(Clone, Debug)]
pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    /// A reader at the start of `data`.
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// A reader at `pos` (which may be past the end; reads then fail).
    pub fn at(data: &'a [u8], pos: usize) -> Self {
        Self { data, pos }
    }

    /// Current offset from the start of the buffer.
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Bytes left after the cursor.
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    /// Whether the cursor is at or past the end.
    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    /// The next `n` bytes.
    pub fn bytes(&mut self, n: usize) -> Result<&'a [u8]> {
        let s = slice(self.data, self.pos, n)?;
        self.pos += n;
        Ok(s)
    }

    /// The next `N` bytes as an array.
    pub fn array<const N: usize>(&mut self) -> Result<[u8; N]> {
        let mut out = [0u8; N];
        out.copy_from_slice(self.bytes(N)?);
        Ok(out)
    }

    /// Skips `n` bytes.
    pub fn skip(&mut self, n: usize) -> Result<()> {
        self.bytes(n).map(|_| ())
    }

    /// Reads one byte.
    pub fn u8(&mut self) -> Result<u8> {
        Ok(self.array::<1>()?[0])
    }

    /// Reads a little-endian `u16`.
    pub fn u16(&mut self) -> Result<u16> {
        self.array().map(u16::from_le_bytes)
    }

    /// Reads a little-endian `i16`.
    pub fn i16(&mut self) -> Result<i16> {
        self.array().map(i16::from_le_bytes)
    }

    /// Reads a little-endian `u32`.
    pub fn u32(&mut self) -> Result<u32> {
        self.array().map(u32::from_le_bytes)
    }

    /// Reads a little-endian `i32`.
    pub fn i32(&mut self) -> Result<i32> {
        self.array().map(i32::from_le_bytes)
    }

    /// Reads a little-endian `u32` holding an IEEE-754 `f32`, keeping the raw
    /// bits so values compare and re-encode exactly (NaN included).
    pub fn f32_bits(&mut self) -> Result<F32Bits> {
        self.u32().map(F32Bits)
    }
}

/// An `f32` kept as its raw bit pattern.
///
/// Parsers store floats this way so that decoded structures implement `Eq`
/// and re-encode byte for byte. Convert with [`F32Bits::get`] where a float
/// is wanted (rendering only, never simulation).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct F32Bits(pub u32);

impl F32Bits {
    /// The float value.
    pub fn get(self) -> f32 {
        f32::from_bits(self.0)
    }

    /// Wraps a float.
    pub fn new(v: f32) -> Self {
        Self(v.to_bits())
    }
}

/// Little-endian appenders for writers.
pub trait PutLe {
    /// Appends a `u16`.
    fn put_u16(&mut self, v: u16);
    /// Appends an `i16`.
    fn put_i16(&mut self, v: i16);
    /// Appends a `u32`.
    fn put_u32(&mut self, v: u32);
    /// Appends an `i32`.
    fn put_i32(&mut self, v: i32);
}

impl PutLe for Vec<u8> {
    fn put_u16(&mut self, v: u16) {
        self.extend_from_slice(&v.to_le_bytes());
    }
    fn put_i16(&mut self, v: i16) {
        self.extend_from_slice(&v.to_le_bytes());
    }
    fn put_u32(&mut self, v: u32) {
        self.extend_from_slice(&v.to_le_bytes());
    }
    fn put_i32(&mut self, v: i32) {
        self.extend_from_slice(&v.to_le_bytes());
    }
}

/// Converts a writer-side length to `u32`.
///
/// # Panics
/// If `n` does not fit. Writers produce fixtures from our own values, so an
/// oversized input is a bug in the caller, not hostile data.
#[allow(clippy::expect_used)]
pub(crate) fn len_u32(n: usize) -> u32 {
    u32::try_from(n).expect("writer input exceeds the format's 32-bit size fields")
}

/// Converts a writer-side length to `u16` (see [`len_u32`]).
#[allow(clippy::expect_used)]
pub(crate) fn len_u16(n: usize) -> u16 {
    u16::try_from(n).expect("writer input exceeds the format's 16-bit size fields")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_are_bounds_checked() {
        let mut r = Reader::new(&[1, 2, 3]);
        assert_eq!(r.u16(), Ok(0x0201));
        assert_eq!(r.u16(), Err(Error::Truncated { needed: 4 }));
        assert_eq!(r.u8(), Ok(3));
        assert!(r.is_empty());
        assert_eq!(
            slice(&[0; 4], usize::MAX, 2).unwrap_err().kind(),
            "truncated"
        );
    }

    #[test]
    fn f32_bits_preserve_nan() {
        let nan = F32Bits(0x7FC0_0001);
        assert_eq!(nan, F32Bits(0x7FC0_0001));
        assert!(nan.get().is_nan());
        assert_eq!(F32Bits::new(1.5).get(), 1.5);
    }
}
