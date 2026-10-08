//! Guessing a file's format from its first bytes.
//!
//! Archives store no names, so tools classify entries by content. These
//! checks are deliberately **permissive** and independent of the strict
//! parsers: a file classified as SHP may still fail [`crate::shp::Shp`], and
//! the local lane counts exactly that. Magic-number formats are tested
//! first, header-shape formats after.

use crate::bytes::Reader;

/// How many leading bytes [`sniff`] looks at; callers may pass fewer.
pub const SNIFF_LEN: usize = 1024;

/// A guessed format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    /// MIX archive.
    Mix,
    /// VXL voxel model.
    Vxl,
    /// HVA voxel animation.
    Hva,
    /// CSF string table.
    Csf,
    /// IDX sound bank index.
    Idx,
    /// AUD sound.
    Aud,
    /// PCX image.
    Pcx,
    /// PAL palette.
    Pal,
    /// TMP-TS terrain template.
    Tmp,
    /// SHP-TS sprite.
    Shp,
    /// Text (INI and similar).
    Text,
    /// Anything else.
    Unknown,
}

impl Kind {
    /// Lower-case label, also the usual file extension.
    pub fn label(self) -> &'static str {
        match self {
            Self::Mix => "mix",
            Self::Vxl => "vxl",
            Self::Hva => "hva",
            Self::Csf => "csf",
            Self::Idx => "idx",
            Self::Aud => "aud",
            Self::Pcx => "pcx",
            Self::Pal => "pal",
            Self::Tmp => "tmp",
            Self::Shp => "shp",
            Self::Text => "text",
            Self::Unknown => "unknown",
        }
    }
}

/// Classifies a file of `len` bytes from its first bytes (`prefix`, up to
/// [`SNIFF_LEN`] or the whole file).
pub fn sniff(prefix: &[u8], len: u64) -> Kind {
    let starts = |m: &[u8]| prefix.starts_with(m);
    if crate::mix::may_be_mix(prefix, len) {
        return Kind::Mix;
    }
    if starts(crate::vxl::MAGIC) {
        return Kind::Vxl;
    }
    if starts(b" FSC") {
        return Kind::Csf;
    }
    if starts(b"GABA") {
        return Kind::Idx;
    }
    if is_hva(prefix, len) {
        return Kind::Hva;
    }
    if is_aud(prefix) {
        return Kind::Aud;
    }
    if is_pcx(prefix, len) {
        return Kind::Pcx;
    }
    if len == crate::pal::PAL_LEN as u64 && prefix.iter().all(|&b| b < 64) {
        return Kind::Pal;
    }
    if is_tmp(prefix, len) {
        return Kind::Tmp;
    }
    if crate::shp::looks_like_shp(prefix, len) {
        return Kind::Shp;
    }
    if looks_like_text(prefix) {
        return Kind::Text;
    }
    Kind::Unknown
}

fn is_hva(p: &[u8], len: u64) -> bool {
    let mut r = Reader::at(p, 16);
    match (r.u32(), r.u32()) {
        (Ok(frames), Ok(limbs)) if frames > 0 && limbs > 0 => {
            let (f, l) = (u64::from(frames), u64::from(limbs));
            24 + 16 * l + 48 * f * l == len
        }
        _ => false,
    }
}

fn is_aud(p: &[u8]) -> bool {
    // Header: rate, sizes, flags, compression 1 or 99; first chunk id.
    let mut r = Reader::at(p, 10);
    match (r.u8(), r.u8(), Reader::at(p, 16).u32()) {
        (Ok(flags), Ok(comp), Ok(id)) => flags < 4 && (comp == 1 || comp == 99) && id == 0xDEAF,
        _ => false,
    }
}

fn is_pcx(p: &[u8], len: u64) -> bool {
    matches!(p, [10, 0..=5, 0 | 1, 1 | 2 | 4 | 8, ..]) && len >= crate::pcx::HEADER_LEN as u64
}

fn is_tmp(p: &[u8], len: u64) -> bool {
    let mut r = Reader::new(p);
    let (Ok(cx), Ok(cy), Ok(w), Ok(h)) = (r.i32(), r.i32(), r.i32(), r.i32()) else {
        return false;
    };
    if cx <= 0 || cy <= 0 || w <= 0 || h <= 0 || cx > 64 || cy > 64 || w > 1024 || h > 1024 {
        return false;
    }
    let cells = (cx * cy) as u64;
    if 16 + 4 * cells > len {
        return false;
    }
    // As OpenRA's check: the first tile's depth offset sits right after its
    // header and diamond. If the tile lies beyond the prefix, accept on the
    // header shape alone.
    let mut first = None;
    for _ in 0..cells {
        match r.u32() {
            Ok(0) => continue,
            Ok(o) => {
                first = Some(o);
                break;
            }
            Err(_) => return true,
        }
    }
    let Some(offset) = first else { return false };
    if u64::from(offset) + 52 > len {
        return false;
    }
    match Reader::at(p, offset as usize + 12).i32() {
        Ok(depth) => i64::from(depth) == i64::from(w) * i64::from(h) / 2 + 52,
        Err(_) => true,
    }
}

/// Whether bytes look like text: no NUL, at least 95 % printable ASCII or
/// whitespace.
pub fn looks_like_text(bytes: &[u8]) -> bool {
    if bytes.is_empty() || bytes.contains(&0) {
        return false;
    }
    let printable = bytes
        .iter()
        .filter(|&&b| b.is_ascii_graphic() || matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
        .count();
    printable * 20 >= bytes.len() * 19
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{generate, registry};

    #[test]
    fn fixtures_are_classified_by_extension() {
        for f in generate(&registry()).unwrap() {
            let ext = f.path.rsplit('.').next().unwrap();
            let kind = sniff(
                &f.bytes[..f.bytes.len().min(SNIFF_LEN)],
                f.bytes.len() as u64,
            );
            let expected = match ext {
                "shp" | "vxl" | "hva" | "csf" | "idx" | "aud" | "pcx" | "pal" | "tmp" | "mix" => {
                    ext
                }
                _ => continue,
            };
            assert_eq!(kind.label(), expected, "{}", f.path);
        }
    }

    #[test]
    fn text_and_unknown() {
        assert_eq!(sniff(b"[General]\nName=x\n", 17), Kind::Text);
        assert_eq!(sniff(&[0xFF, 0, 1, 2], 4), Kind::Unknown);
    }
}
