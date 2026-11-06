//! Display texts from the string table (S-0004 claim 17): a label's
//! lookup, its matching, the space normalisation of the entry's text and
//! the display-name label of a type.
//!
//! Which file the string table is, which of a label's strings is its text
//! and the placeholder's marker word are not given by any spec yet
//! (Q-2686). So no table is read from the install: [`StringTable`] is
//! built from entries already decoded, and the interface resolves every
//! label through [`resolve`], which gives our stand-in for a table it
//! cannot read.

use std::collections::BTreeMap;

use glasnost_fmt::csf::Csf;
use glasnost_ini::value;

/// The byte limit of a display-name label (S-0003 claim 1 text value).
// spec: S-0004 claim 17 (the label is read as a text value with a 31-byte limit)
pub const LABEL_LIMIT: usize = 31;

/// The line feed text unit.
const LF: u16 = 0x0a;
/// The tab text unit.
const TAB: u16 = 0x09;
/// The space text unit.
const SPACE: u16 = 0x20;

/// A string table: its entries in file order, found by label.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StringTable {
    /// Entry labels as stored and their texts, in file order.
    entries: Vec<(Vec<u8>, Vec<u16>)>,
    /// Entry by its label with A to Z folded: the first in file order.
    by_label: BTreeMap<Vec<u8>, usize>,
    /// The importer's warnings: entry labels that differ only in case.
    pub warnings: Vec<String>,
}

/// What a lookup finds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Found {
    /// The matching entry's text, its spaces normalised.
    Text(Vec<u16>),
    /// No entry matches: the table's placeholder shows (its marker word
    /// waits on Q-2686).
    Missing,
}

/// A label with the letters A to Z folded to a to z, every other byte
/// kept.
fn folded(label: &[u8]) -> Vec<u8> {
    // spec: S-0004 claim 17 (Matching: same length and bytes, the letters A to Z counting as equal to a to z; every other byte must be equal, spaces included)
    label.iter().map(u8::to_ascii_lowercase).collect()
}

impl StringTable {
    /// A table from its entries in file order: each entry label as the
    /// file stores it and its decoded text. Of two entry labels that differ
    /// only in letter case the first is found, and the importer warns,
    /// naming both.
    pub fn new(entries: Vec<(Vec<u8>, Vec<u16>)>) -> Self {
        let mut by_label = BTreeMap::new();
        let mut warnings = Vec::new();
        for (i, (label, _)) in entries.iter().enumerate() {
            match by_label.entry(folded(label)) {
                std::collections::btree_map::Entry::Vacant(v) => {
                    v.insert(i);
                }
                std::collections::btree_map::Entry::Occupied(o) => {
                    // spec: S-0004 Edge cases (two entries whose labels differ only in letter case, departure, no switch: the first in the file, and the importer warns, naming both labels)
                    let first = &entries[*o.get()].0;
                    warnings.push(format!(
                        "string table: the entry labels {:?} and {:?} differ only in letter case; {:?} is used",
                        String::from_utf8_lossy(first),
                        String::from_utf8_lossy(label),
                        String::from_utf8_lossy(first),
                    ));
                }
            }
        }
        Self {
            entries,
            by_label,
            warnings,
        }
    }

    /// A table from a parsed CSF file: each label's first string as its
    /// text, none for a label without strings.
    pub fn from_csf(csf: &Csf) -> Self {
        // design: Q-2686 (no formats spec gives the string table's decoding, S-0004 Open questions 8: each label's first string, as XCC's reader takes it, and an empty text for a label with none)
        Self::new(
            csf.labels
                .iter()
                .map(|l| {
                    let text = l
                        .strings
                        .first()
                        .map(|s| s.units.clone())
                        .unwrap_or_default();
                    (l.name.clone(), text)
                })
                .collect(),
        )
    }

    /// The number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the table has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Looks `label` up: the text of the entry it matches, normalised, or
    /// [`Found::Missing`].
    pub fn find(&self, label: &[u8]) -> Found {
        // spec: S-0004 claim 17 (Every lookup: the matching, the placeholder and the normalisation hold for every lookup of a label)
        match self.by_label.get(&folded(label)) {
            Some(&i) => Found::Text(normalised(&self.entries[i].1)),
            None => Found::Missing,
        }
    }
}

/// An entry's text with its spaces normalised: spaces at the start and
/// right after a line feed or a tab dropped, a run of spaces made one, a
/// space right before a line feed or a tab or at the end dropped. Every
/// other text unit is kept, a carriage return counting as an ordinary one.
pub fn normalised(text: &[u16]) -> Vec<u16> {
    // spec: S-0004 claim 17 (The entry's text: the space rules; a carriage return is an ordinary character for them; nothing is cut)
    let mut out: Vec<u16> = Vec::with_capacity(text.len());
    for &c in text {
        if c == SPACE {
            let after_break = out.last().is_none_or(|&p| p == LF || p == TAB);
            if after_break || out.last() == Some(&SPACE) {
                continue;
            }
        } else if (c == LF || c == TAB) && out.last() == Some(&SPACE) {
            out.pop();
        }
        out.push(c);
    }
    if out.last() == Some(&SPACE) {
        out.pop();
    }
    out
}

/// A type's display-name label from the value its display-name key holds
/// in force (the last layer's non-empty value, `None` without one): the
/// value cut to 31 bytes as a text value. `None` is no label.
pub fn label(stored: Option<&[u8]>) -> Option<&[u8]> {
    // spec: S-0004 claim 17 (Reading the label: a text value with a 31-byte limit; an absent key or an empty value keeps the label in force; a type starts with no label; a label spelled none is looked up like any other)
    stored
        .map(|v| value::text(v, LABEL_LIMIT))
        .filter(|l| !l.is_empty())
}

/// The text a label shows, as text units: the table's entry, or for a
/// label the table lacks (and while no table is read) our stand-in
/// `stand_in`. No label gives empty text.
pub fn resolve(table: Option<&StringTable>, label: Option<&[u8]>, stand_in: &str) -> Vec<u16> {
    // spec: S-0004 claim 17 (Resolving it: empty text when the type has no label, never its type name)
    let Some(label) = label else {
        return Vec::new();
    };
    match table.map(|t| t.find(label)) {
        Some(Found::Text(t)) => t,
        // design: Q-2686 (the placeholder's marker word and the string table's file are not given: our stand-in for a missing entry and for a table not read)
        Some(Found::Missing) | None => stand_in.encode_utf16().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glasnost_fmt::csf::{CsfLabel, CsfString};

    fn u(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    /// A synthetic table written and parsed as a CSF file.
    fn table(entries: &[(&str, &[&str])]) -> StringTable {
        let csf = Csf {
            version: 3,
            language: 0,
            unused: 0,
            labels: entries
                .iter()
                .map(|(name, strings)| CsfLabel {
                    name: name.as_bytes().to_vec(),
                    strings: strings
                        .iter()
                        .map(|s| CsfString {
                            units: u(s),
                            extra: None,
                        })
                        .collect(),
                })
                .collect(),
        };
        StringTable::from_csf(&Csf::parse(&csf.write()).unwrap())
    }

    /// spec: S-0004 claim 17 (Matching: the letters A to Z equal a to z, every other byte equal,
    /// the same length; leading and trailing spaces of an entry label are significant; a label
    /// spelled none is looked up like any other) and Edge cases (a case-only duplicate: the first
    /// in the file, with a warning naming both)
    #[test]
    fn claim_17_matching() {
        let t = table(&[
            ("Name:Probe", &["one"]),
            ("NAME:PROBE", &["two"]),
            ("Tail ", &["tail"]),
            ("none", &["nothing"]),
            ("Ä:x", &["umlaut"]),
            ("[x", &["bracket"]),
        ]);
        assert_eq!(t.find(b"name:probe"), Found::Text(u("one")));
        assert_eq!(
            t.find(b"NAME:PROBE"),
            Found::Text(u("one")),
            "the first wins"
        );
        assert_eq!(t.warnings.len(), 1);
        assert!(t.warnings[0].contains("Name:Probe") && t.warnings[0].contains("NAME:PROBE"));
        assert_eq!(t.find(b"Tail"), Found::Missing, "the trailing space counts");
        assert_eq!(t.find(b"tail "), Found::Text(u("tail")));
        assert_eq!(t.find(b" Name:Probe"), Found::Missing);
        assert_eq!(t.find(b"Name:Prob"), Found::Missing, "same length");
        assert_eq!(t.find(b"NONE"), Found::Text(u("nothing")));
        assert_eq!(t.find("ä:x".as_bytes()), Found::Missing, "only A to Z fold");
        assert_eq!(t.find(b"{x"), Found::Missing, "[ is not {{");
        assert_eq!(t.len(), 6);
    }

    /// spec: S-0004 claim 17 (The entry's text: leading spaces and spaces after a line feed or a
    /// tab dropped, runs made one, a space before a line feed, a tab or the end dropped; a
    /// carriage return is ordinary; other control characters kept; nothing cut)
    #[test]
    fn claim_17_normalisation() {
        let n = |s: &str| String::from_utf16(&normalised(&u(s))).unwrap();
        assert_eq!(n("  a   b  "), "a b");
        assert_eq!(n("a \n  b"), "a\nb");
        assert_eq!(n("a \t b"), "a\tb");
        assert_eq!(n("a \r b"), "a \r b", "a carriage return is ordinary");
        assert_eq!(n("a  \r  b"), "a \r b");
        assert_eq!(n("\u{1} x\u{7}"), "\u{1} x\u{7}");
        assert_eq!(n("   "), "");
        assert_eq!(n("\n \n"), "\n\n");
        let long: String = "x".repeat(400);
        assert_eq!(n(&long).len(), 400, "the lookup cuts nothing");
        let t = table(&[("L", &["  two  words "])]);
        assert_eq!(t.find(b"l"), Found::Text(u("two words")));
    }

    /// spec: S-0004 claim 17 (Reading the label: a 31-byte text value; no label gives empty text,
    /// never the type name; Resolving it) and Q-2686 (the stand-in for a missing entry)
    #[test]
    fn claim_17_labels_and_resolving() {
        assert_eq!(label(None), None);
        let long = b"0123456789012345678901234567890123";
        assert_eq!(label(Some(long)), Some(&long[..31]));
        assert_eq!(label(Some(b"Name:X")), Some(&b"Name:X"[..]));
        let t = table(&[("Name:X", &["Shown"]), ("Empty", &[])]);
        assert_eq!(resolve(Some(&t), None, "TYPE"), u(""));
        assert_eq!(resolve(Some(&t), Some(b"name:x"), "TYPE"), u("Shown"));
        assert_eq!(resolve(Some(&t), Some(b"Empty"), "TYPE"), u(""));
        assert_eq!(resolve(Some(&t), Some(b"Other"), "TYPE"), u("TYPE"));
        assert_eq!(resolve(None, Some(b"Name:X"), "TYPE"), u("TYPE"));
        assert_eq!(resolve(None, None, "TYPE"), u(""));
    }
}
