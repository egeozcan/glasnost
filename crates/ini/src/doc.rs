//! Documents: sections and entries with the original's name resolution
//! (S-0001 claims 12 to 14, S-0002).

use std::collections::{BTreeMap, BTreeSet};

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
        let table = Table::new(entries.iter().map(|e| e.key.as_slice()), &quirks);
        Self {
            name: raw.name,
            line: raw.line,
            entries,
            table,
            quirks,
        }
    }

    /// A section that a layered load creates, before its first entry.
    fn created(name: Vec<u8>, line: u32, quirks: Quirks) -> Self {
        Self {
            name,
            line,
            entries: Vec::new(),
            table: Table::new(std::iter::empty(), &quirks),
            quirks,
        }
    }

    /// Drops the entries a layered load replaced and renumbers the table.
    fn compact(&mut self, discarded: &BTreeSet<u32>) {
        let mut i = 0u32;
        self.entries.retain(|_| {
            let keep = !discarded.contains(&i);
            i += 1;
            keep
        });
        self.table.compact(discarded);
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
        let i = self.table.lookup(key)?;
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
        let table = Table::new(sections.iter().map(|s| s.name.as_slice()), &quirks);
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
            table: Table::new(std::iter::empty(), &quirks),
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
        let i = self.table.lookup(name)?;
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
    /// claims 6 and 9 to 12). Each entry of `bytes` replaces the occurrence
    /// that a lookup returns, or is added, and goes to the end of its
    /// section's entry order; empty values and empty headers change
    /// nothing; the final line is read without a line feed. Which
    /// occurrence a lookup returns, during the load and after it, follows
    /// the name-table model of S-0002 State and ordering.
    pub fn layer(&mut self, bytes: &[u8], warnings: &mut Vec<Warning>) {
        let raw = lex::lex(bytes, &self.quirks, FinalLine::Read, warnings);
        // The second file's own repeats warn too (S-0002 Edge cases).
        for s in &raw {
            names_warnings(
                s.entries.iter().map(|e| (e.key.as_slice(), e.line)),
                Some(&s.name),
                warnings,
            );
        }
        // spec: S-0002 claim 12 (every table starts the layered load with nothing remembered)
        self.table.forget();
        for s in &mut self.sections {
            s.table.forget();
        }
        // Tables of the earlier file that hold a repeated name hash; the
        // first change to one warns (S-0002 claim 12, Edge cases). `None`
        // stands for the table of section names. This is a superset of the
        // tables that the original's rules reading looks up.
        let mut repeated: BTreeSet<Option<usize>> = self
            .sections
            .iter()
            .enumerate()
            .filter(|(_, s)| s.table.has_repeated_hash())
            .map(|(i, _)| Some(i))
            .collect();
        if self.table.has_repeated_hash() {
            repeated.insert(None);
        }
        let mut discarded: BTreeMap<usize, BTreeSet<u32>> = BTreeMap::new();
        for s in raw {
            // spec: S-0002 claim 10 (an empty header in the second file creates nothing)
            for e in s.entries {
                // spec: S-0002 claim 11 (each line looks up its section name, then its key)
                let idx = match self.table.lookup_mut(&s.name) {
                    Some(i) => {
                        let found = &self.sections[i as usize];
                        if found.name != s.name {
                            // spec: S-0002 claim 11 (writes into a section of a different name, which keeps its own)
                            warnings.push(Warning::new(
                                e.line,
                                WarningKind::LayeredSharedHash {
                                    section: None,
                                    name: s.name.clone(),
                                    other: found.name.clone(),
                                },
                            ));
                        }
                        i as usize
                    }
                    None => {
                        if repeated.remove(&None) {
                            warnings.push(Warning::new(
                                s.line,
                                WarningKind::LayeredOntoRepeatedHash { section: None },
                            ));
                        }
                        if let Some(o) = self
                            .table
                            .other_name_with_hash(&s.name, |i| &self.sections[i as usize].name)
                        {
                            let other = &self.sections[o as usize];
                            warnings.push(Warning::new(
                                s.line,
                                WarningKind::HashCollision {
                                    section: None,
                                    name: s.name.clone(),
                                    other: other.name.clone(),
                                    other_line: other.line,
                                },
                            ));
                        }
                        // spec: S-0002 State and ordering (section created if needed, at the end)
                        self.table.add(&s.name);
                        self.sections
                            .push(Section::created(s.name.clone(), s.line, self.quirks));
                        self.sections.len() - 1
                    }
                };
                let section = &mut self.sections[idx];
                if repeated.remove(&Some(idx)) {
                    warnings.push(Warning::new(
                        e.line,
                        WarningKind::LayeredOntoRepeatedHash {
                            section: Some(section.name.clone()),
                        },
                    ));
                }
                let gone = discarded.entry(idx).or_default();
                match section.table.lookup_mut(&e.key) {
                    Some(old) => {
                        let replaced = &section.entries[old as usize];
                        if replaced.key != e.key {
                            // spec: S-0002 claim 11 (replaces an entry of a different name with the same hash)
                            warnings.push(Warning::new(
                                e.line,
                                WarningKind::LayeredSharedHash {
                                    section: Some(section.name.clone()),
                                    name: e.key.clone(),
                                    other: replaced.key.clone(),
                                },
                            ));
                        }
                        // spec: S-0002 claim 10 (only the occurrence the line's lookup returns is replaced)
                        section.table.replace(old);
                        gone.insert(old);
                    }
                    None => {
                        if let Some(o) = section
                            .table
                            .other_name_with_hash(&e.key, |i| &section.entries[i as usize].key)
                        {
                            let other = &section.entries[o as usize];
                            warnings.push(Warning::new(
                                e.line,
                                WarningKind::HashCollision {
                                    section: Some(section.name.clone()),
                                    name: e.key.clone(),
                                    other: other.key.clone(),
                                    other_line: other.line,
                                },
                            ));
                        }
                    }
                }
                // spec: S-0002 claim 10 (a replaced or added entry goes to the end)
                section.table.add(&e.key);
                section.entries.push(Entry {
                    key: e.key,
                    value: e.value,
                    line: e.line,
                });
            }
        }
        for (idx, gone) in discarded {
            self.sections[idx].compact(&gone);
        }
        self.table.settle();
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
    /// measure: M-0012 (claim 5's fresh-load case, measured for one key: the end-to-end comparison is `glasnost_convert`'s `a_rules_files_unterminated_last_line_reads_as_absent`)
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

    fn cost_int(text: &[u8], q: Quirks) -> Option<i32> {
        let mut w = Vec::new();
        let d = Document::parse(text, q, &mut w);
        d.value(b"T", b"Cost").and_then(crate::value::integer)
    }

    /// spec: S-0001 claims 1, 2 and 3 (r4 scenarios L1c to L1e, L3f to L3h)
    #[test]
    fn high_bytes_bom_line_and_counted_bytes() {
        let q = Quirks::default();
        for b in 128..=255u8 {
            // L1c: a byte 128 to 255 before the key is part of the key.
            let mut t = b"[T]\n".to_vec();
            t.push(b);
            t.extend_from_slice(b"Cost=5\n");
            assert_eq!(cost_int(&t, q), None, "L1c byte {b}");
            // L1d: before the value it is kept, so no initial decimal number.
            let mut t = b"[T]\nCost=".to_vec();
            t.push(b);
            t.extend_from_slice(b"5\n");
            assert_eq!(cost_int(&t, q), Some(0), "L1d byte {b}");
        }
        // L1e: the mark followed by a line feed leaves the header intact.
        assert_eq!(cost(&parse("\u{feff}\n[T]\nCost=5\n").0), Some("5"));
        // L3f: CR bytes in the indentation do not count towards the limit.
        let pad = format!("{}{}", "\r".repeat(20), " ".repeat(508));
        assert_eq!(cost(&parse(&format!("{pad}[T]\nCost=5\n")).0), Some("5"));
        // L3g: `1Fh` ending at byte 511 is read; L3h: one byte more cuts the `h`.
        let zeros = "0".repeat(511 - "Cost=".len() - "1Fh".len());
        let t = format!("[T]\nCost={zeros}1Fh\n");
        assert_eq!(cost_int(t.as_bytes(), q), Some(31));
        let t = format!("[T]\nCost=0{zeros}1Fh\n");
        assert_eq!(cost_int(t.as_bytes(), q), Some(1));
    }

    /// spec: S-0001 claims 8, 11, 12 and 14 (r4 scenarios L6a, L6c, L9d, L12a; Import mapping row I5)
    #[test]
    fn comments_empty_prefix_sections_and_counts() {
        let q = Quirks::default();
        // L6a: the comment goes before the hexadecimal suffix is read.
        assert_eq!(cost_int(b"[T]\nCost=1Fh;123\n", q), Some(31));
        // L6c: the value before the comment is empty, so no entry.
        assert_eq!(cost(&parse("[T]\nCost=;1Fh\n").0), None);
        // L9d: an empty copy holding only an empty name-prefix line, in either
        // order and in each spelling, does not compete with the test section.
        for empty in ["Prefix=", "Prefix=   ", "Prefix= ; note"] {
            let (d, _) = parse(&format!("[T]\n{empty}\n[T]\nCost=5\n"));
            assert_eq!(cost(&d), Some("5"), "{empty:?} first");
            assert_eq!(d.sections().len(), 1);
            assert!(d.get(b"T", b"Prefix").is_none());
            let (d, _) = parse(&format!("[T]\nCost=5\n[T]\n{empty}\n"));
            assert_eq!(cost(&d), Some("5"), "{empty:?} second");
            assert!(parse(&format!("[C]\n{empty}\n")).0.section(b"C").is_none());
        }
        // L12a: 5000 filler sections, then the test section with 5000 filler
        // entries before the test entry: 5001 sections and 5001 entries.
        let mut t = String::new();
        for i in 1..=5000 {
            t.push_str(&format!("[Filler{i}]\nFiller1=1\n"));
        }
        t.push_str("[T]\n");
        for i in 1..=5000 {
            t.push_str(&format!("Filler{i}=1\n"));
        }
        t.push_str("Cost=5\n");
        let (d, _) = parse(&t);
        assert_eq!(d.sections().len(), 5001);
        assert_eq!(d.section(b"T").unwrap().entries().len(), 5001);
        assert_eq!(cost(&d), Some("5"));
    }

    /// spec: S-0001 Edge cases (r4 importer acceptance checks: the warnings on L1a/L1b,
    /// L2b/L2c, L7a/L7d and L4a, none on unchanged controls; switch-off results for L4a, L7a,
    /// L7d and L7f)
    #[test]
    fn importer_acceptance_checks() {
        let warned =
            |text: &str, f: fn(&WarningKind) -> bool| parse(text).1.iter().any(|w| f(&w.kind));
        let bom = |k: &WarningKind| matches!(k, WarningKind::ByteOrderMark);
        let cr = |k: &WarningKind| matches!(k, WarningKind::StrayCarriageReturn);
        let trim = |k: &WarningKind| matches!(k, WarningKind::TrimKeptWhitespace { .. });
        let last = |k: &WarningKind| matches!(k, WarningKind::FinalLineIgnored);
        assert!(warned("\u{feff}[T]\nCost=5\n", bom));
        assert!(warned("\u{feff}; test\n[T]\nCost=5\n", bom));
        assert!(warned("[T]\nCost=12\r3\n", cr));
        assert!(warned("[T]\rCost=5\n", cr));
        assert!(warned("[T]\n     Cost =5\n", trim));
        assert!(warned("[T]\nCost=    1Fh  \n", trim));
        assert!(warned("[T]\nCost=5", last));
        // Unchanged controls: no warning at all.
        for control in [
            "[T]\nCost=5\n",
            "[T]\r\nCost=5\r\n",
            "[T]\nCost=1Fh\n",
            "[T]\n    Cost =5\n",
        ] {
            assert!(parse(control).1.is_empty(), "{control:?}");
        }
        // L7d, L7e and L7f with the defect.
        let on = Quirks::default();
        assert_eq!(cost_int(b"[T]\nCost=    1Fh  \n", on), Some(1));
        assert_eq!(cost_int(b"[T]\nCost=1Fh\n", on), Some(31));
        let l7f = format!("{}Cost=     1Fh    ;\n", " ".repeat(16));
        let l7f = format!("[T]\n{l7f}");
        assert_eq!(cost_int(l7f.as_bytes(), on), Some(1));
        // Switches off.
        let off = Quirks {
            ini_trim_defect: false,
            ..Quirks::default()
        };
        assert_eq!(cost_int(b"[T]\n     Cost =5\n", off), Some(5));
        assert_eq!(cost_int(b"[T]\nCost=    1Fh  \n", off), Some(31));
        assert_eq!(cost_int(l7f.as_bytes(), off), Some(31));
        let off = Quirks {
            ini_final_line_needs_newline: false,
            ..Quirks::default()
        };
        assert_eq!(cost_int(b"[T]\nCost=5", off), Some(5));
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
        );
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

    fn layered(earlier: &str, later: &str, q: Quirks) -> (Document, Vec<Warning>) {
        let mut w = Vec::new();
        let mut d = Document::parse(earlier.as_bytes(), q, &mut w);
        w.clear();
        d.layer(later.as_bytes(), &mut w);
        (d, w)
    }

    /// A name made of `prefix` and a number whose hash satisfies `ok`.
    fn name_where(prefix: &str, ok: impl Fn(i32) -> bool) -> String {
        (0..10_000)
            .map(|i| format!("{prefix}{i}"))
            .find(|n| ok(name_hash(n.as_bytes())))
            .unwrap()
    }

    /// spec: S-0002 claim 10 (third item) and claim 11; State and ordering
    /// (the name-table model). Outcomes derived by hand from the model.
    #[test]
    fn layered_load_onto_a_repeated_key() {
        let q = Quirks::default();
        // (K1 = A, K2 = B): the line's lookup sorts to (K2, K1) and returns
        // K1, which is replaced; the table (K2, new) is re-sorted after the
        // load to (new, K2), and the bisection probes K2.
        let (d, w) = layered("[T]\nCost=A\nCost=B\n", "[T]\nCost=Own\n", q);
        assert_eq!(cost(&d), Some("B"));
        let s = d.section(b"T").unwrap();
        assert_eq!(s.entries().len(), 2);
        assert!(w.iter().any(|w| matches!(
            &w.kind,
            WarningKind::LayeredOntoRepeatedHash { section: Some(s) } if s == b"T"
        )));
        // (K1 = A, K2 = B, G) with G's hash smaller (fixture 2): the lookup
        // sorts to (G, K2, K1) and returns K2, which is replaced; (G, K1,
        // new) re-sorts to (G, new, K1), and the bisection probes the new
        // entry.
        let (d, _) = layered("[T]\nCost=A\nCost=B\nFiller0=1\n", "[T]\nCost=Own\n", q);
        assert_eq!(cost(&d), Some("Own"));
        // A later line of the same section changes the outcome. After the
        // first line the working order is (G, K1, new), not ordered; the
        // Filler0 line sorts it to (G, new, K1), finds G and replaces it,
        // leaving (new, K1, G').
        let (d, _) = layered(
            "[T]\nCost=A\nCost=B\nFiller0=1\n",
            "[T]\nCost=Own\nFiller0=2\n",
            q,
        );
        let s = d.section(b"T").unwrap();
        let keys: Vec<&[u8]> = s.entries().iter().map(|e| e.key.as_slice()).collect();
        assert_eq!(keys, [&b"Cost"[..], b"Cost", b"Filler0"]);
        // Sorting (new, K1, G') puts G' first, then (K1, new); the
        // bisection probes position 1, K1.
        assert_eq!(cost(&d), Some("A"));
        // With hash order off the last occurrence is replaced and wins.
        let off = Quirks {
            ini_duplicate_hash_order: false,
            ..Quirks::default()
        };
        let (d, _) = layered("[M]\nBriefing=X\nBriefing=Y\n", "[M]\nBriefing=Z\n", off);
        assert_eq!(d.value(b"M", b"Briefing"), Some(&b"Z"[..]));
        let values: Vec<&[u8]> = d
            .section(b"M")
            .unwrap()
            .entries()
            .iter()
            .map(|e| e.value.as_slice())
            .collect();
        assert_eq!(values, [&b"X"[..], b"Z"]);
    }

    /// spec: S-0002 claim 11 (two copies of a section) and State and
    /// ordering (the remembered occurrence outlasts an addition that is
    /// not made at a forgetting peak)
    #[test]
    fn remembered_copy_after_the_load() {
        let q = Quirks::default();
        // A section name whose hash is larger than that of "M": sorting
        // (M#1, M#0, N) would leave N last and return M#1.
        let n = name_where("N", |h| h > name_hash(b"M"));
        let earlier = "[M]\nBriefing=X\n[M]\nBriefing=Y\n";
        // A fresh load: copies (M#0, M#1) sort to (M#1, M#0); M#0 wins.
        assert_eq!(parse(earlier).0.value(b"M", b"Briefing"), Some(&b"X"[..]));
        // The line writes into M#0 and remembers it; adding N (2 sections,
        // not a forgetting peak) keeps it, and the lookup after the load
        // returns it without re-sorting.
        let (d, w) = layered(earlier, &format!("[M]\nBriefing=Own\n[{n}]\nA=1\n"), q);
        assert_eq!(d.value(b"M", b"Briefing"), Some(&b"Own"[..]));
        assert_eq!(d.sections().len(), 3);
        assert!(w.iter().any(|w| matches!(
            &w.kind,
            WarningKind::LayeredOntoRepeatedHash { section: None }
        )));
        // Re-sorting would return the other copy: a fresh load of the
        // sections in the load's working order (M#1, M#0, N) shows it.
        let fresh = format!("[M]\nBriefing=Y\n[M]\nBriefing=Own\n[{n}]\nA=1\n");
        assert_eq!(parse(&fresh).0.value(b"M", b"Briefing"), Some(&b"Y"[..]));
        // An empty header in the second file creates nothing.
        let (d, _) = layered(earlier, "[E]\n[M]\nBriefing=Own\n", q);
        assert!(d.section(b"E").is_none());
        assert_eq!(d.sections().len(), 2);
    }

    /// spec: S-0002 claim 11 (names identified by hash in a layered load),
    /// Edge cases (importer warnings, the switch combinations)
    #[test]
    fn layered_load_with_shared_hashes() {
        let q = Quirks::default();
        // A key that shares its hash replaces the entry, under its own
        // spelling.
        let (d, w) = layered("[T]\nCost=A\n", "[T]\nZdxeLZgd=B\n", q);
        let s = d.section(b"T").unwrap();
        assert_eq!(s.entries().len(), 1);
        assert_eq!(s.entries()[0].key, b"ZdxeLZgd");
        assert_eq!(cost(&d), Some("B"));
        assert!(w.iter().any(|w| matches!(
            w.kind,
            WarningKind::LayeredSharedHash {
                section: Some(_),
                ..
            }
        )));
        // A section name that shares its hash writes into the section,
        // which keeps its spelling.
        let (d, w) = layered("[Cost]\nA=1\n", "[ZdxeLZgd]\nB=2\n", q);
        assert_eq!(d.sections().len(), 1);
        assert_eq!(d.sections()[0].name(), b"Cost");
        assert_eq!(d.value(b"Cost", b"B"), Some(&b"2"[..]));
        assert!(
            w.iter()
                .any(|w| matches!(w.kind, WarningKind::LayeredSharedHash { section: None, .. }))
        );
        // Hash identity off: the names stay apart, and the pair warns as a
        // collision.
        let off = Quirks {
            ini_hash_identity: false,
            ..Quirks::default()
        };
        let (d, w) = layered("[T]\nCost=A\n", "[T]\nZdxeLZgd=B\n", off);
        assert_eq!(d.section(b"T").unwrap().entries().len(), 2);
        assert_eq!(cost(&d), Some("A"));
        assert_eq!(d.value(b"T", b"ZdxeLZgd"), Some(&b"B"[..]));
        assert!(w.iter().any(|w| matches!(
            w.kind,
            WarningKind::HashCollision {
                section: Some(_),
                ..
            }
        )));
        // No warning about repeated hashes when the earlier file has none.
        let (_, w) = layered("[T]\nCost=A\n", "[T]\nCost=B\n", q);
        assert!(w.is_empty());
    }
}
