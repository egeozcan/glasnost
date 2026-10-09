//! Documents: sections and entries with the original's name resolution
//! (S-0001 claims 12 to 14, S-0002).

use crate::lex::{self, FinalLine, RawSection};
use crate::table::{Table, name_hash};
use crate::{Quirks, Warning, WarningKind};

/// One entry: a key and its stored value (S-0003 Terms), with its line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Trimmed key bytes.
    pub key: Vec<u8>,
    /// Stored value bytes (never empty).
    pub value: Vec<u8>,
    /// 1-based line in the file the entry came from.
    pub line: u32,
}

/// One non-empty section of a document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    name: Vec<u8>,
    line: u32,
    entries: Vec<Entry>,
    table: Table,
    quirks: Quirks,
}

impl Section {
    fn new(raw: RawSection, quirks: Quirks) -> Self {
        let entries: Vec<Entry> = raw
            .entries
            .into_iter()
            .map(|e| Entry {
                key: e.key,
                value: e.value,
                line: e.line,
            })
            .collect();
        let table = Table::new(entries.iter().map(|e| e.key.as_slice()));
        Self {
            name: raw.name,
            line: raw.line,
            entries,
            table,
            quirks,
        }
    }

    fn rebuild(&mut self) {
        self.table = Table::new(self.entries.iter().map(|e| e.key.as_slice()));
    }

    /// The name as written between the brackets.
    pub fn name(&self) -> &[u8] {
        &self.name
    }

    /// 1-based line of the header.
    pub fn line(&self) -> u32 {
        self.line
    }

    /// Every entry, every occurrence of a repeated key included, in entry
    /// order (S-0001 claim 14, S-0002 claim 1).
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// The entry a lookup of `key` returns (S-0002 claims 1 and 3).
    pub fn get(&self, key: &[u8]) -> Option<&Entry> {
        let i = self
            .table
            .lookup(key, &self.quirks, |i| &self.entries[i as usize].key)?;
        self.entries.get(i as usize)
    }

    /// The stored value a lookup of `key` returns.
    pub fn value(&self, key: &[u8]) -> Option<&[u8]> {
        self.get(key).map(|e| e.value.as_slice())
    }

    /// Positions of a type list (S-0002 claim 4): one per entry in entry
    /// order, each holding the entry that a lookup *by that entry's key*
    /// returns, which for a repeated key is the winning occurrence.
    pub fn list_positions(&self) -> impl Iterator<Item = (&Entry, &Entry)> {
        self.entries
            .iter()
            .map(|e| (e, self.get(&e.key).unwrap_or(e)))
    }
}

/// A document: the non-empty sections of one file, in file order (S-0001
/// claims 12 and 14), with the original's section lookup (S-0002 claim 2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    sections: Vec<Section>,
    table: Table,
    quirks: Quirks,
}

/// A layered load that this engine refuses (see [`Document::layer`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayerError {
    /// The section whose table (or `None`: the section-name table) would
    /// be left holding two occurrences with one name hash.
    pub section: Option<Vec<u8>>,
    /// Line in the second file that changed that table.
    pub line: u32,
}

impl std::fmt::Display for LayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let what = match &self.section {
            Some(s) => format!("the keys of section [{}]", String::from_utf8_lossy(s)),
            None => "the section names".to_owned(),
        };
        write!(
            f,
            "line {}: a layered load changes {what}, which already hold two names with one \
             name hash; the winner afterwards is not specified (S-0002 open question, Q-0004)",
            self.line
        )
    }
}

impl std::error::Error for LayerError {}

impl Document {
    /// Reads `bytes` by a fresh load (S-0001, S-0002). Warnings are
    /// appended to `warnings`.
    pub fn parse(bytes: &[u8], quirks: Quirks, warnings: &mut Vec<Warning>) -> Self {
        let raw = lex::lex(bytes, &quirks, FinalLine::PerQuirk, warnings);
        // spec: S-0001 claim 12 (a section with no entries does not exist)
        let sections: Vec<Section> = raw
            .into_iter()
            .filter(|s| !s.entries.is_empty())
            .map(|s| Section::new(s, quirks))
            .collect();
        let table = Table::new(sections.iter().map(|s| s.name.as_slice()));
        let doc = Self {
            sections,
            table,
            quirks,
        };
        doc.duplicate_warnings(warnings);
        doc
    }

    /// An empty document.
    pub fn empty(quirks: Quirks) -> Self {
        Self {
            sections: Vec::new(),
            table: Table::default(),
            quirks,
        }
    }

    /// The quirk switches this document was read with.
    pub fn quirks(&self) -> &Quirks {
        &self.quirks
    }

    /// Every non-empty section, every copy of a repeated name included, in
    /// file order.
    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    /// Whether the document holds no section.
    pub fn is_empty(&self) -> bool {
        self.sections.is_empty()
    }

    /// The section a lookup of `name` returns (S-0002 claims 2 and 3).
    pub fn section(&self, name: &[u8]) -> Option<&Section> {
        self.sections.get(self.section_index(name)?)
    }

    /// Position (in [`Document::sections`]) of the section a lookup of
    /// `name` returns.
    pub fn section_index(&self, name: &[u8]) -> Option<usize> {
        let i = self
            .table
            .lookup(name, &self.quirks, |i| &self.sections[i as usize].name)?;
        Some(i as usize)
    }

    /// The entry for `key` in the section a lookup of `section` returns.
    pub fn get(&self, section: &[u8], key: &[u8]) -> Option<&Entry> {
        self.section(section)?.get(key)
    }

    /// The stored value for `key` in `section`.
    pub fn value(&self, section: &[u8], key: &[u8]) -> Option<&[u8]> {
        self.get(section, key).map(|e| e.value.as_slice())
    }

    fn duplicate_warnings(&self, warnings: &mut Vec<Warning>) {
        // spec: S-0002 Edge cases (importer warnings: repeated section names, repeated keys, shared hashes)
        names_warnings(
            self.sections.iter().map(|s| (s.name.as_slice(), s.line)),
            None,
            warnings,
        );
        for s in &self.sections {
            names_warnings(
                s.entries.iter().map(|e| (e.key.as_slice(), e.line)),
                Some(&s.name),
                warnings,
            );
        }
    }

    /// Reads a second file into this document by a layered load (S-0002
    /// claims 6, 9 and 10). Each entry of `bytes` replaces the occurrence
    /// a lookup returns (or is added), moving to the end of its section's
    /// entry order; empty values and empty headers change nothing; the
    /// final line is read without a line feed.
    ///
    /// When hash order is on and a table that the load changes ends up
    /// holding two occurrences with one name hash, which one wins is not
    /// specified (S-0002 open question); this engine refuses the load and
    /// leaves the document unchanged (request Q-0004). The original reads
    /// no stock file this way.
    pub fn layer(&mut self, bytes: &[u8], warnings: &mut Vec<Warning>) -> Result<(), LayerError> {
        let mut out = self.clone();
        let raw = lex::lex(bytes, &self.quirks, FinalLine::Read, warnings);
        // The second file's own repeats warn too (S-0002 Edge cases).
        for s in &raw {
            names_warnings(
                s.entries.iter().map(|e| (e.key.as_slice(), e.line)),
                Some(&s.name),
                warnings,
            );
        }
        let order = out.quirks.ini_duplicate_hash_order;
        let identity = out.quirks.ini_hash_identity;
        let mut touched = std::collections::BTreeSet::new();
        for s in raw {
            // spec: S-0002 claim 10 (an empty header in the second file creates nothing)
            for e in s.entries {
                let idx = match out.section_index(&s.name) {
                    Some(i) => i,
                    None => {
                        // spec: S-0002 State and ordering (section created if needed, at the end)
                        out.sections.push(Section {
                            name: s.name.clone(),
                            line: s.line,
                            entries: Vec::new(),
                            table: Table::default(),
                            quirks: out.quirks,
                        });
                        out.table = Table::new(out.sections.iter().map(|x| x.name.as_slice()));
                        if order && out.table.has_repeated_hash() {
                            return Err(LayerError {
                                section: None,
                                line: s.line,
                            });
                        }
                        out.sections.len() - 1
                    }
                };
                let section = &mut out.sections[idx];
                // With hash order on, a table that holds a repeated hash keeps it, and
                // its winner after the load is open (S-0002 open question): refuse.
                if order && touched.insert(idx) && section.table.has_repeated_hash() {
                    return Err(LayerError {
                        section: Some(section.name.clone()),
                        line: e.line,
                    });
                }
                touched.insert(idx);
                // The occurrence a lookup returns: with hash order on the table holds no
                // repeated hash, so it is the only match; with it off, the last one.
                let h = name_hash(&e.key);
                let counts = |x: &Entry| {
                    if identity {
                        name_hash(&x.key) == h
                    } else {
                        x.key == e.key
                    }
                };
                // spec: S-0002 claim 10 (only the occurrence lookups return is replaced)
                if let Some(p) = section.entries.iter().rposition(counts) {
                    section.entries.remove(p);
                }
                if order && section.entries.iter().any(|x| name_hash(&x.key) == h) {
                    // A different name with the same hash stays beside the new entry.
                    return Err(LayerError {
                        section: Some(section.name.clone()),
                        line: e.line,
                    });
                }
                // spec: S-0002 claim 10 (a replaced or added entry goes to the end)
                section.entries.push(Entry {
                    key: e.key,
                    value: e.value,
                    line: e.line,
                });
            }
        }
        for idx in touched {
            out.sections[idx].rebuild();
        }
        *self = out;
        Ok(())
    }
}

/// Warnings for one name table: repeated names and different names that
/// share a hash.
fn names_warnings<'a>(
    names: impl Iterator<Item = (&'a [u8], u32)>,
    section: Option<&[u8]>,
    warnings: &mut Vec<Warning>,
) {
    let names: Vec<(&[u8], u32)> = names.collect();
    let mut first_of: std::collections::BTreeMap<&[u8], u32> = Default::default();
    let mut by_hash: std::collections::BTreeMap<i32, (&[u8], u32)> = Default::default();
    for &(name, line) in &names {
        if let Some(&first) = first_of.get(name) {
            let kind = match section {
                Some(s) => WarningKind::DuplicateKey {
                    section: s.to_vec(),
                    key: name.to_vec(),
                    first_line: first,
                },
                None => WarningKind::DuplicateSection {
                    name: name.to_vec(),
                    first_line: first,
                },
            };
            warnings.push(Warning::new(line, kind));
            continue;
        }
        first_of.insert(name, line);
        let h = name_hash(name);
        match by_hash.get(&h) {
            Some(&(other, other_line)) if other != name => {
                warnings.push(Warning::new(
                    line,
                    WarningKind::HashCollision {
                        section: section.map(<[u8]>::to_vec),
                        name: name.to_vec(),
                        other: other.to_vec(),
                        other_line,
                    },
                ));
            }
            Some(_) => {}
            None => {
                by_hash.insert(h, (name, line));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> (Document, Vec<Warning>) {
        let mut w = Vec::new();
        let d = Document::parse(text.as_bytes(), Quirks::default(), &mut w);
        (d, w)
    }

    fn cost(d: &Document) -> Option<&str> {
        d.value(b"T", b"Cost")
            .map(|v| std::str::from_utf8(v).unwrap())
    }

    /// spec: S-0001 claims 5, 7 and 14; S-0001 scenarios L4, L5g, L12b, L12c
    #[test]
    fn final_line_and_order() {
        let (d, w) = parse("[T]\nCost=5");
        assert_eq!(cost(&d), None);
        assert!(
            w.iter()
                .any(|w| matches!(w.kind, WarningKind::FinalLineIgnored))
        );
        let (d, _) = parse("[T]\nCost=5\n");
        assert_eq!(cost(&d), Some("5"));
        let mut w = Vec::new();
        let q = Quirks {
            ini_final_line_needs_newline: false,
            ..Quirks::default()
        };
        let d = Document::parse(b"[T]\nCost=5", q, &mut w);
        assert_eq!(d.value(b"T", b"Cost"), Some(&b"5"[..]));
        assert_eq!(cost(&parse("Cost=7\n[T]\nA=1\n").0), None);
        assert_eq!(cost(&parse("[T]\nCost=150\nCost=160\n").0), Some("150"));
        assert_eq!(cost(&parse("[T]\nCost=160\nCost=150\n").0), Some("160"));
    }

    /// spec: S-0001 claim 1 (scenarios L1a, L1b) and claim 2 (L2a to L2c)
    #[test]
    fn bom_and_carriage_returns() {
        let (d, w) = parse("\u{feff}[T]\nCost=5\n");
        assert_eq!(cost(&d), None);
        assert!(
            w.iter()
                .any(|w| matches!(w.kind, WarningKind::ByteOrderMark))
        );
        assert_eq!(cost(&parse("\u{feff}; c\n[T]\nCost=5\n").0), Some("5"));
        assert_eq!(cost(&parse("[T]\r\nCost=5\r\n").0), Some("5"));
        let (d, w) = parse("[T]\nCost=12\r3\n");
        assert_eq!(cost(&d), Some("123"));
        assert!(
            w.iter()
                .any(|w| matches!(w.kind, WarningKind::StrayCarriageReturn))
        );
        assert_eq!(cost(&parse("[T]\rCost=5\n").0), None);
        assert!(parse("[T]\rCost=5\r").0.is_empty());
    }

    /// spec: S-0001 claims 3 and 6 (scenarios L3a to L3e)
    #[test]
    fn line_limit() {
        let pad = " ".repeat(520);
        assert_eq!(cost(&parse(&format!("[T]\nCost{pad}=5\n")).0), None);
        let tail = " ".repeat(600);
        assert_eq!(cost(&parse(&format!("[T]\nCost=5{tail}\n")).0), Some("5"));
        let filler = "9".repeat(600);
        assert_eq!(
            cost(&parse(&format!("[T]\nFiller1={filler}\nCost=5\n")).0),
            Some("5")
        );
        // `]` at byte 511 of the line: a header; at byte 512: not one.
        let ok = " ".repeat(509 - 1);
        assert_eq!(cost(&parse(&format!("{ok}[T]\nCost=5\n")).0), Some("5"));
        let long = " ".repeat(509);
        assert_eq!(cost(&parse(&format!("{long}[T]\nCost=5\n")).0), None);
    }

    /// spec: S-0001 claim 12 (scenarios L9a to L9c), S-0002 claim 2
    #[test]
    fn empty_sections_do_not_compete() {
        assert_eq!(cost(&parse("[T]\nCost=\n[T]\nCost=5\n").0), Some("5"));
        assert_eq!(cost(&parse("[T]\nCost=5\n[T]\nCost=\n").0), Some("5"));
        assert_eq!(cost(&parse("[T]\n[T]\nCost=5\n").0), Some("5"));
        let (d, w) = parse("[T]\nCost=\n[T]\nCost=5\n");
        assert_eq!(d.sections().len(), 1);
        assert!(
            !w.iter()
                .any(|w| matches!(w.kind, WarningKind::DuplicateSection { .. }))
        );
    }

    /// spec: S-0001 claim 13 (scenarios L10a, L10b)
    #[test]
    fn case_sensitive_names() {
        let (d, _) = parse("[Alpha]\nkey=1\n");
        assert!(d.section(b"ALPHA").is_none());
        assert!(d.get(b"Alpha", b"Key").is_none());
        assert!(d.get(b"Alpha", b"key").is_some());
    }

    /// spec: S-0001 claim 10 (scenarios L7a to L7c)
    #[test]
    fn trim_defect_in_documents() {
        let (d, w) = parse("[T]\n     Cost =5\n");
        assert_eq!(cost(&d), None);
        assert_eq!(d.value(b"T", b"Cost "), Some(&b"5"[..]));
        assert!(
            w.iter()
                .any(|w| matches!(w.kind, WarningKind::TrimKeptWhitespace { in_key: true, .. }))
        );
        assert_eq!(cost(&parse("[T]\n     Cost =5  ; note\n").0), Some("5"));
        assert_eq!(cost(&parse("[T]\n    Cost =5\n").0), Some("5"));
        let mut w = Vec::new();
        let q = Quirks {
            ini_trim_defect: false,
            ..Quirks::default()
        };
        let d = Document::parse(b"[T]\n     Cost =5\n", q, &mut w);
        assert_eq!(d.value(b"T", b"Cost"), Some(&b"5"[..]));
        assert!(w.is_empty());
    }

    /// spec: S-0002 claims 1, 2 and 4; Import mapping rows D-c to D-e
    #[test]
    fn duplicates() {
        let (d, w) = parse("[T]\nCost=A\nCost=B\n");
        assert_eq!(cost(&d), Some("A"));
        assert!(
            w.iter()
                .any(|w| matches!(w.kind, WarningKind::DuplicateKey { .. }))
        );
        assert_eq!(
            cost(&parse("[T]\nCost=A\nCost=B\nFiller0=1\n").0),
            Some("B")
        );
        let (d, w) = parse("[T]\nZdxeLZgd=B\nCost=A\n");
        assert_eq!(cost(&d), Some("B"));
        assert!(
            w.iter()
                .any(|w| matches!(w.kind, WarningKind::HashCollision { .. }))
        );
        // Duplicate sections: one copy, never merged.
        let (d, w) = parse("[T]\nCost=A\n[T]\nUIName=X\n");
        let s = d.section(b"T").unwrap();
        assert_eq!(s.entries().len(), 1);
        assert!(
            w.iter()
                .any(|w| matches!(w.kind, WarningKind::DuplicateSection { .. }))
        );
        // A type list with a repeated key: the winner's value at every position.
        let (d, _) = parse("[L]\n3=Alpha\n3=Beta\n4=Gamma\n");
        let values: Vec<&[u8]> = d
            .section(b"L")
            .unwrap()
            .list_positions()
            .map(|(_, e)| e.value.as_slice())
            .collect();
        let winner = d.value(b"L", b"3").unwrap();
        assert_eq!(values, [winner, winner, &b"Gamma"[..]]);
    }

    /// spec: S-0002 claims 6, 9 and 10 (scenarios D5, D7)
    #[test]
    fn layered_load() {
        let (mut d, _) = parse("[M]\nBriefing=Two\nUIName=Name1\n");
        let mut w = Vec::new();
        d.layer(
            b"[M]\nBriefing=One\nBriefing=Own\nUIName=\n[]\n[New]\nA=1",
            &mut w,
        )
        .unwrap();
        assert_eq!(d.value(b"M", b"Briefing"), Some(&b"Own"[..]));
        assert_eq!(d.value(b"M", b"UIName"), Some(&b"Name1"[..]));
        assert_eq!(d.value(b"New", b"A"), Some(&b"1"[..]));
        // The replaced entry moved to the end of the entry order.
        let keys: Vec<&[u8]> = d
            .section(b"M")
            .unwrap()
            .entries()
            .iter()
            .map(|e| e.key.as_slice())
            .collect();
        assert_eq!(keys, [&b"UIName"[..], b"Briefing"]);
        assert!(
            w.iter()
                .any(|w| matches!(w.kind, WarningKind::DuplicateKey { .. }))
        );
    }

    /// spec: S-0002 claim 10 (third item) and its open question (Q-0004)
    #[test]
    fn layered_load_onto_duplicates_is_refused() {
        let (mut d, _) = parse("[M]\nBriefing=X\nBriefing=Y\n");
        let before = d.clone();
        let err = d.layer(b"[M]\nBriefing=Z\n", &mut Vec::new()).unwrap_err();
        assert_eq!(err.section.as_deref(), Some(&b"M"[..]));
        assert_eq!(d, before);
        // With hash order off the result is defined: the last occurrence.
        let mut w = Vec::new();
        let q = Quirks {
            ini_duplicate_hash_order: false,
            ..Quirks::default()
        };
        let mut d = Document::parse(b"[M]\nBriefing=X\nBriefing=Y\n", q, &mut w);
        d.layer(b"[M]\nBriefing=Z\n", &mut w).unwrap();
        assert_eq!(d.value(b"M", b"Briefing"), Some(&b"Z"[..]));
        assert_eq!(d.section(b"M").unwrap().entries().len(), 2);
    }
}
