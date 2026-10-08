//! File names for archive IDs.
//!
//! Archives store only name hashes. Names come from what the caller knows
//! (a name the user typed, a name in our own code) and from text the user's
//! own files contain: [`harvest_tokens`] pulls name-like tokens out of text
//! entries, and [`candidates`] turns each into the file names it could
//! denote by appending the extensions of the formats we read. A candidate
//! is kept only if its hash occurs in a mounted archive.

use std::collections::{BTreeMap, BTreeSet};

use glasnost_fmt::mix::id_hash;

/// Known names by ID. On a hash collision the first name added is kept.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Names {
    by_id: BTreeMap<u32, String>,
}

impl Names {
    /// Adds `name` (stored lower-case) and returns its ID.
    pub fn insert(&mut self, name: &str) -> u32 {
        let id = id_hash(name);
        self.by_id
            .entry(id)
            .or_insert_with(|| name.to_ascii_lowercase());
        id
    }

    /// The name for `id`, if known.
    pub fn get(&self, id: u32) -> Option<&str> {
        self.by_id.get(&id).map(String::as_str)
    }

    /// Number of known names.
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// Whether no names are known.
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// All known (ID, name) pairs in ID order.
    pub fn iter(&self) -> impl Iterator<Item = (u32, &str)> {
        self.by_id.iter().map(|(&id, n)| (id, n.as_str()))
    }
}

/// Extensions tried for every harvested token: the formats this project
/// reads plus the tile-set extensions of the TMP format, as XCC's name
/// finder (`findf.cpp`, GPL-3.0-or-later) tries them.
pub const CANDIDATE_EXTENSIONS: &[&str] = &[
    "shp", "vxl", "hva", "tmp", "pal", "pcx", "mix", "ini", "csf", "aud", "wav", "idx", "bag",
    "vpl", "fnt", "map", "mpr", "tem", "sno", "urb", "ubn", "des", "lun",
];

/// Longest token kept by [`harvest_tokens`]; longer runs are not file names.
const MAX_TOKEN: usize = 48;

/// Name-like tokens in `text`: runs of ASCII letters, digits and `_ . - ~`.
///
/// Text that holds a `[PrivateKey]` section is skipped entirely (PLAN.md
/// 3.5: we do not extract those).
pub fn harvest_tokens(text: &[u8]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    if text
        .windows(b"[PrivateKey]".len())
        .any(|w| w.eq_ignore_ascii_case(b"[PrivateKey]"))
    {
        return out;
    }
    let name_byte = |b: u8| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-' | b'~');
    for token in text.split(|&b| !name_byte(b)) {
        if !token.is_empty() && token.len() <= MAX_TOKEN {
            // All bytes are ASCII by construction.
            out.insert(String::from_utf8_lossy(token).to_ascii_lowercase());
        }
    }
    out
}

/// File names a token could denote: itself if it has an extension, and the
/// token with each of [`CANDIDATE_EXTENSIONS`].
pub fn candidates(token: &str) -> impl Iterator<Item = String> + '_ {
    let itself = token.contains('.').then(|| token.to_owned());
    itself.into_iter().chain(
        CANDIDATE_EXTENSIONS
            .iter()
            .map(move |ext| format!("{token}.{ext}")),
    )
}

/// Whether `bytes` look like text (an INI file or similar): no NUL bytes,
/// and nearly all bytes printable ASCII or whitespace.
pub fn looks_like_text(bytes: &[u8]) -> bool {
    if bytes.is_empty() || bytes.contains(&0) {
        return false;
    }
    let printable = bytes
        .iter()
        .filter(|&&b| b.is_ascii_graphic() || matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
        .count();
    // At least 95 % printable: tolerates a few stray high bytes.
    printable * 20 >= bytes.len() * 19
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harvests_and_expands() {
        let text = b"[Section]\nKey=Widget,Gadget2 ; note\nImage=gizmo.shp\n";
        let tokens = harvest_tokens(text);
        assert!(tokens.contains("widget"));
        assert!(tokens.contains("gadget2"));
        assert!(tokens.contains("gizmo.shp"));
        let c: Vec<_> = candidates("gizmo.shp").collect();
        assert_eq!(c[0], "gizmo.shp");
        assert!(candidates("widget").any(|n| n == "widget.vxl"));
    }

    #[test]
    fn private_key_text_is_skipped() {
        assert!(harvest_tokens(b"[privatekey]\nA=B\n").is_empty());
    }

    #[test]
    fn names_are_case_insensitive() {
        let mut n = Names::default();
        let id = n.insert("Foo.SHP");
        assert_eq!(n.get(id), Some("foo.shp"));
        assert_eq!(id, id_hash("foo.shp"));
    }

    #[test]
    fn text_detection() {
        assert!(looks_like_text(b"[A]\r\nB=C\r\n"));
        assert!(!looks_like_text(&[0, 1, 2, 3]));
        assert!(!looks_like_text(b""));
    }
}
