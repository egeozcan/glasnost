//! Display texts from the string table (S-0004 claim 17): a label's
//! lookup, its matching, the space normalisation of the entry's text and
//! the display-name label of a type.
//!
//! The string table is the install's `ra2md.csf` (S-0015 claim 1), read
//! at run time ([`read`]) by [`StringTable::from_bytes`] in the layout of
//! S-0015 claims 2 to 4. The interface resolves every label through
//! [`resolve`]: the entry's text, the missing-label placeholder of S-0015
//! claim 8 for a label no entry matches, and our stand-in only when no
//! table could be read.

use std::collections::BTreeMap;

use glasnost_fmt::csf::StringFile;
use glasnost_ini::value;
use glasnost_vfs::Vfs;

/// The string table file (S-0015 claim 1), resolved as S-0005 claim 9 gives.
pub const FILE: &str = "ra2md.csf"; // spec: S-0015 claim 1 (the program reads one string table, ra2md.csf; ra2.csf is never read)
/// The missing-label placeholder's marker word.
const MARKER: &str = "MISSING"; // spec: S-0015 Constants (marker word MISSING) and claim 8
/// The apostrophe text unit.
const APOSTROPHE: u16 = 39; // spec: S-0015 claim 8 (an apostrophe, text unit 39)
/// An entry label of this many bytes or more matches no lookup.
const LONGEST_MATCH: usize = 31; // spec: S-0015 Constants (longest label that a lookup can match: 31 bytes) and claim 5

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
    /// No entry matches: the placeholder shows ([`placeholder`]).
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
            if label.len() > LONGEST_MATCH {
                // spec: S-0015 claim 5 (an entry label of 32 bytes or more matches no lookup, and the importer warns)
                warnings.push(format!(
                    "string table: the entry label {:?} has {} bytes and matches no lookup",
                    String::from_utf8_lossy(label),
                    label.len()
                ));
                continue;
            }
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

    /// A table from the bytes of the string table file (S-0015). `Err` is
    /// a header that fails claim 2's checks.
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        // spec: S-0015 claims 2 to 9 (the layout, the decoding, the first text of each entry, an empty text for an entry with none, the reading stopped by damage)
        let file = StringFile::read(data).map_err(|e| e.to_string())?;
        let mut t = Self::new(file.entries);
        t.warnings.splice(0..0, file.warnings);
        Ok(t)
    }

    /// The entry labels as stored, in file order.
    pub fn labels(&self) -> impl Iterator<Item = &[u8]> {
        self.entries.iter().map(|(l, _)| l.as_slice())
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

/// The text a missing label shows: the marker word, a colon, an apostrophe,
/// the requested label with each byte as the text unit of the same value,
/// and an apostrophe. Spaces are kept, no normalisation.
pub fn placeholder(label: &[u8]) -> Vec<u16> {
    // spec: S-0015 claim 8 (the marker word, a colon, an apostrophe, the requested label's bytes as text units of the same value, an apostrophe; no space normalisation)
    let mut out: Vec<u16> = MARKER.encode_utf16().collect();
    out.push(u16::from(b':'));
    out.push(APOSTROPHE);
    out.extend(label.iter().map(|&b| u16::from(b)));
    out.push(APOSTROPHE);
    out
}

/// The text a label shows, as text units: the table's entry, the
/// placeholder for a label the table lacks, and, while no table is read
/// (a table that cannot be read), our stand-in `stand_in`. No label gives
/// empty text.
pub fn resolve(table: Option<&StringTable>, label: Option<&[u8]>, stand_in: &str) -> Vec<u16> {
    // spec: S-0004 claim 17 (Resolving it: empty text when the type has no label, never its type name)
    let Some(label) = label else {
        return Vec::new();
    };
    match table.map(|t| t.find(label)) {
        Some(Found::Text(t)) => t,
        // spec: S-0015 claim 8 (no entry matches: the placeholder)
        Some(Found::Missing) => placeholder(label),
        // design: S-0015 claim 1 (the program refuses to start without a table, so only a game built without an install, as in tests, has none: it shows our stand-in)
        None => stand_in.encode_utf16().collect(),
    }
}

/// Reads the install's string table: `ra2md.csf` resolved as S-0005
/// claim 9 gives while only the two language archives are mounted. `Err`
/// is a diagnostic naming the file and the reason; the table's importer
/// warnings are `StringTable::warnings`.
pub async fn read(vfs: &Vfs) -> Result<StringTable, String> {
    // spec: S-0015 claims 1 and 2 (one table, ra2md.csf, resolved while only the two language archives are mounted; a missing file or a header that fails the checks is a table that cannot be resolved, S-0005 claim 16, which in our session is a diagnostic and the stand-in names)
    let Some(r) = vfs.resolve_language_only(FILE) else {
        return Err(format!("{FILE}: the string table is missing"));
    };
    let bytes = vfs
        .read_resolved(&r)
        .await
        .map_err(|_| format!("{FILE}: the string table cannot be read"))?;
    StringTable::from_bytes(&bytes)
        .map_err(|e| format!("{FILE}: the string table is rejected: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use glasnost_fmt::csf::{Csf, CsfLabel, CsfString};

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
        StringTable::from_bytes(&csf.write()).unwrap()
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

    /// spec: S-0015 claim 8 (the placeholder: marker word, colon, apostrophe, the label's bytes as
    /// units of the same value with spaces kept, apostrophe), claim 5 (an entry label of 32 bytes
    /// or more matches nothing and warns), claim 6 (an entry with no texts gives empty text, not
    /// the placeholder) and claims 2 and 9 (the file's reading)
    #[test]
    fn s0015_placeholder_and_reading() {
        assert_eq!(placeholder(b"A  b"), u("MISSING:'A  b'"));
        assert_eq!(placeholder(&[0xE9]), {
            let mut v = u("MISSING:'");
            v.push(0xE9);
            v.push(39);
            v
        });
        let long = "L".repeat(32);
        let ok = "L".repeat(31);
        let t = table(&[
            (&long, &["x"]),
            (&ok, &["y"]),
            ("Cut\0tail", &["z"]),
            ("E", &[]),
        ]);
        assert_eq!(t.find(long.as_bytes()), Found::Missing);
        assert_eq!(t.find(ok.as_bytes()), Found::Text(u("y")));
        assert_eq!(
            t.find(b"cut"),
            Found::Text(u("z")),
            "cut before a zero byte"
        );
        assert_eq!(t.find(b"E"), Found::Text(Vec::new()), "present, no texts");
        assert!(t.warnings.iter().any(|w| w.contains("32 bytes")));
        assert!(StringTable::from_bytes(b"not a table at all, but long enough").is_err());
        assert!(StringTable::from_bytes(&[]).is_err());
        // A damaged tail keeps the entries before it and warns.
        let mut bytes = Csf {
            version: 3,
            language: 0,
            unused: 0,
            labels: ["A", "B"]
                .iter()
                .map(|n| CsfLabel {
                    name: n.as_bytes().to_vec(),
                    strings: vec![CsfString {
                        units: u(n),
                        extra: None,
                    }],
                })
                .collect(),
        }
        .write();
        let n = bytes.len();
        bytes.truncate(n - 1);
        let t = StringTable::from_bytes(&bytes).unwrap();
        assert_eq!(t.find(b"A"), Found::Text(u("A")));
        assert_eq!(t.find(b"B"), Found::Missing);
        assert!(!t.warnings.is_empty());
    }

    /// A synthetic install whose language archive `langmd.mix` holds `members`.
    fn install(members: &[(&str, Vec<u8>)]) -> Vfs {
        install_archives(&[("langmd.mix", members)])
    }

    /// An archive's members: a name and its bytes each.
    type Members<'a> = &'a [(&'a str, Vec<u8>)];

    /// A synthetic install of the loose top-level archives `archives`, each
    /// holding its members (and a filler, so that none is empty).
    fn install_archives(archives: &[(&str, Members<'_>)]) -> Vfs {
        use glasnost_fmt::mix::{MixWriter, WriteLayout, id_hash};
        use glasnost_vfs::{GameOptions, MemSource, PickedFile};
        let picked = archives
            .iter()
            .map(|(name, members)| {
                let mut files: Vec<_> = members
                    .iter()
                    .map(|(n, b)| (id_hash(n), b.clone()))
                    .collect();
                files.push((id_hash("filler.ini"), b"[Filler]\n".to_vec()));
                let bytes = MixWriter {
                    layout: WriteLayout::Plain,
                    files,
                }
                .write();
                PickedFile {
                    path: (*name).into(),
                    source: std::rc::Rc::new(MemSource(bytes)),
                }
            })
            .collect();
        let mut vfs = Vfs::default();
        pollster::block_on(vfs.mount_install(picked));
        pollster::block_on(vfs.mount_startup(GameOptions::default()));
        vfs
    }

    /// spec: S-0015 claim 1 (`ra2md.csf` is resolved as S-0005 claim 9 gives while only the two
    /// language archives are mounted: a copy in the base language archive is read, and a copy
    /// that only a later archive of the search order holds is not, so the table cannot be
    /// resolved and the program refuses to start, S-0005 claim 16) (seam sweep 6: intake 109's
    /// row)
    #[test]
    fn seams6_the_table_is_resolved_among_the_language_archives_only() {
        let good = Csf {
            version: 3,
            language: 0,
            unused: 0,
            labels: vec![CsfLabel {
                name: b"Probe".to_vec(),
                strings: vec![CsfString {
                    units: u("shown"),
                    extra: None,
                }],
            }],
        }
        .write();
        let base = install_archives(&[
            ("langmd.mix", &[]),
            ("language.mix", &[("ra2md.csf", good.clone())]),
        ]);
        let t = pollster::block_on(read(&base)).expect("the base language archive's copy");
        assert_eq!(t.find(b"probe"), Found::Text(u("shown")));
        let later = install_archives(&[
            ("langmd.mix", &[]),
            ("language.mix", &[]),
            ("ra2md.mix", &[("ra2md.csf", good)]),
        ]);
        assert!(
            later.resolve(FILE).is_some(),
            "set-up: the later archive is in the search order"
        );
        assert!(
            pollster::block_on(read(&later)).is_err(),
            "a copy only a later archive holds is not read"
        );
    }

    /// spec: S-0015 claims 1 and 2 (the program reads `ra2md.csf`, a member of the language
    /// archive, and never `ra2.csf`; a missing file or a header that fails the checks is a table
    /// that cannot be resolved)
    #[test]
    fn s0015_claims_1_and_2_the_file() {
        let good = Csf {
            version: 3,
            language: 0,
            unused: 0,
            labels: vec![CsfLabel {
                name: b"Probe".to_vec(),
                strings: vec![CsfString {
                    units: u("shown"),
                    extra: None,
                }],
            }],
        }
        .write();
        let t = pollster::block_on(read(&install(&[("ra2md.csf", good.clone())]))).unwrap();
        assert_eq!(t.find(b"probe"), Found::Text(u("shown")));
        let mut bad = good.clone();
        bad[0] = b'X';
        assert!(pollster::block_on(read(&install(&[("ra2md.csf", bad)]))).is_err());
        assert!(pollster::block_on(read(&install(&[]))).is_err(), "no file");
        assert!(
            pollster::block_on(read(&install(&[("ra2.csf", good)]))).is_err(),
            "ra2.csf alone is never read"
        );
    }

    /// spec: S-0004 claim 17 (Reading the label: a 31-byte text value; no label gives empty text,
    /// never the type name; Resolving it) and S-0015 claim 8 (the placeholder for a missing entry;
    /// our stand-in only without a table)
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
        assert_eq!(
            resolve(Some(&t), Some(b"Other"), "TYPE"),
            u("MISSING:'Other'")
        );
        assert_eq!(resolve(None, Some(b"Name:X"), "TYPE"), u("TYPE"));
        assert_eq!(resolve(None, None, "TYPE"), u(""));
    }
}
