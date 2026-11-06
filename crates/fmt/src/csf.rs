//! CSF string tables: labels mapped to UTF-16 strings.
//!
//! Header (24 bytes): `" FSC"`, `u32 version`, `u32 label count`,
//! `u32 string count`, `u32 unused`, `u32 language`. Each label: `" LBL"`,
//! `u32 string count`, `u32 name length`, ASCII name; then per string
//! `" RTS"` or `"WRTS"`, `u32 length` in UTF-16 units, the units with every
//! bit inverted, and for `"WRTS"` an extra `u32 length` plus ASCII value.
//!
//! Layout from XCC's `csf_file.cpp` and `cc_structures.h`
//! (GPL-3.0-or-later, Copyright (C) 2000 Olaf van der Spek). XCC reads only
//! the first string of each label; we keep all of them.

use crate::bytes::{PutLe, Reader, len_u32};
use crate::error::{Error, Result};
use crate::fixture::FixtureWriter;

const FILE_ID: &[u8; 4] = b" FSC";
const LABEL_ID: &[u8; 4] = b" LBL";
const STRING_ID: &[u8; 4] = b" RTS";
const STRING_EXTRA_ID: &[u8; 4] = b"WRTS";

/// One string of a label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CsfString {
    /// UTF-16 code units (already un-inverted).
    pub units: Vec<u16>,
    /// Extra ASCII value of a `"WRTS"` string.
    pub extra: Option<Vec<u8>>,
}

impl CsfString {
    /// The text, with unpaired surrogates replaced.
    pub fn text(&self) -> String {
        String::from_utf16_lossy(&self.units)
    }
}

/// One label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CsfLabel {
    /// Label name as stored (ASCII bytes).
    pub name: Vec<u8>,
    /// Its strings (normally one).
    pub strings: Vec<CsfString>,
}

/// A parsed string table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Csf {
    /// Version field.
    pub version: u32,
    /// Language field.
    pub language: u32,
    /// Unused header field, kept for re-encoding.
    pub unused: u32,
    /// Labels in file order.
    pub labels: Vec<CsfLabel>,
}

impl Csf {
    /// Parses a CSF file.
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        if &r.array::<4>()? != FILE_ID {
            return Err(Error::Invalid("CSF magic"));
        }
        let version = r.u32()?;
        let label_count = r.u32()? as usize;
        let _string_count = r.u32()?;
        let unused = r.u32()?;
        let language = r.u32()?;
        // A label takes at least 12 bytes.
        let mut labels = Vec::with_capacity(label_count.min(r.remaining() / 12));
        for _ in 0..label_count {
            if &r.array::<4>()? != LABEL_ID {
                return Err(Error::Invalid("CSF label marker"));
            }
            let count = r.u32()? as usize;
            let name_len = r.u32()? as usize;
            let name = r.bytes(name_len)?.to_vec();
            // A string takes at least 8 bytes.
            let mut strings = Vec::with_capacity(count.min(r.remaining() / 8));
            for _ in 0..count {
                let id = r.array::<4>()?;
                let has_extra = match &id {
                    STRING_ID => false,
                    STRING_EXTRA_ID => true,
                    _ => return Err(Error::Invalid("CSF string marker")),
                };
                let units_len = r.u32()? as usize;
                let raw = r.bytes(
                    units_len
                        .checked_mul(2)
                        .ok_or(Error::Invalid("CSF length"))?,
                )?;
                let units = raw
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|b| !u16::from_le_bytes(*b))
                    .collect();
                let extra = if has_extra {
                    let n = r.u32()? as usize;
                    Some(r.bytes(n)?.to_vec())
                } else {
                    None
                };
                strings.push(CsfString { units, extra });
            }
            labels.push(CsfLabel { name, strings });
        }
        Ok(Self {
            version,
            language,
            unused,
            labels,
        })
    }

    /// Serialises the table.
    pub fn write(&self) -> Vec<u8> {
        let mut out = FILE_ID.to_vec();
        out.put_u32(self.version);
        out.put_u32(len_u32(self.labels.len()));
        out.put_u32(len_u32(self.labels.iter().map(|l| l.strings.len()).sum()));
        out.put_u32(self.unused);
        out.put_u32(self.language);
        for l in &self.labels {
            out.extend_from_slice(LABEL_ID);
            out.put_u32(len_u32(l.strings.len()));
            out.put_u32(len_u32(l.name.len()));
            out.extend_from_slice(&l.name);
            for s in &l.strings {
                out.extend_from_slice(if s.extra.is_some() {
                    STRING_EXTRA_ID
                } else {
                    STRING_ID
                });
                out.put_u32(len_u32(s.units.len()));
                for &u in &s.units {
                    out.put_u16(!u);
                }
                if let Some(e) = &s.extra {
                    out.put_u32(len_u32(e.len()));
                    out.extend_from_slice(e);
                }
            }
        }
        out
    }
}

/// The string table as the program reads it (S-0015): each entry's label
/// and first text, with what the reading warns about. Unlike [`Csf::parse`]
/// it takes a file damaged after its header as far as it holds whole
/// entries, as S-0015 claim 9 gives.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StringFile {
    /// Entries in file order: the label as stored, cut before its first
    /// zero byte, and the decoded first text (empty for an entry with no
    /// texts), before the space normalisation.
    pub entries: Vec<(Vec<u8>, Vec<u16>)>,
    /// The importer's warnings (S-0015 claim 9).
    pub warnings: Vec<String>,
}

/// One entry read whole: label, first text, number of texts, end position.
type EntryRead = (Vec<u8>, Vec<u16>, usize, usize);

/// A byte cursor that fails instead of reading past the end.
struct Cur<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cur<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.pos.checked_add(n)?;
        let b = self.data.get(self.pos..end)?;
        self.pos = end;
        Some(b)
    }

    fn id(&mut self) -> Option<[u8; 4]> {
        self.take(4)?.try_into().ok()
    }

    /// A signed 32-bit little-endian value, `None` past the end or below 0.
    fn count(&mut self) -> Option<usize> {
        let v = i32::from_le_bytes(self.take(4)?.try_into().ok()?);
        usize::try_from(v).ok()
    }
}

/// Reads one entry at `pos`: `Err` names why it is damaged.
fn read_entry(data: &[u8], pos: usize) -> std::result::Result<EntryRead, &'static str> {
    let mut c = Cur { data, pos };
    if c.id() != Some(*LABEL_ID) {
        return Err("the entry does not start with its identifying bytes");
    }
    let texts = c.count().ok_or("a text count is below 0 or past the end")?;
    let len = c
        .count()
        .ok_or("a label length is below 0 or past the end")?;
    let label = c
        .take(len)
        .ok_or("an entry runs past the end of the file")?;
    let cut = label.iter().position(|&b| b == 0).unwrap_or(label.len());
    let mut first = Vec::new();
    for i in 0..texts {
        let extended = match c.id() {
            Some(id) if &id == STRING_ID => false,
            Some(id) if &id == STRING_EXTRA_ID => true,
            Some(_) => return Err("a text does not start with its identifying bytes"),
            None => return Err("an entry runs past the end of the file"),
        };
        let n = c.count().ok_or("a unit count is below 0 or past the end")?;
        let stored = c
            .take(n.checked_mul(2).ok_or("a unit count is too large")?)
            .ok_or("an entry runs past the end of the file")?;
        if extended {
            let m = c
                .count()
                .ok_or("an extra length is below 0 or past the end")?;
            c.take(m).ok_or("an entry runs past the end of the file")?;
        }
        if i == 0 {
            // spec: S-0015 claim 4 (the text unit is 65535 minus the stored unit; the text ends before the first stored 0 or 65535)
            first = stored
                .as_chunks::<2>()
                .0
                .iter()
                .map(|b| u16::from_le_bytes(*b))
                .take_while(|&s| s != 0 && s != u16::MAX)
                .map(|s| u16::MAX - s)
                .collect();
        }
    }
    Ok((label[..cut].to_vec(), first, texts, c.pos))
}

impl StringFile {
    /// Reads a string table file (S-0015). `Err` is a header that fails
    /// claim 2's checks, which stops the program from starting.
    pub fn read(data: &[u8]) -> Result<Self> {
        // spec: S-0015 claim 2 (24-byte header; refuse a shorter file, other identifying bytes, an entry count or text count of 0 or below)
        if data.len() < 24 {
            return Err(Error::Invalid("string table shorter than its header"));
        }
        if &data[..4] != FILE_ID {
            return Err(Error::Invalid("string table identifying bytes"));
        }
        let field =
            |at: usize| i32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
        let (entry_count, text_count) = (field(8), field(12));
        if entry_count <= 0 || text_count <= 0 {
            return Err(Error::Invalid("string table count of 0 or below"));
        }
        let (entry_count, text_count) = (entry_count as usize, text_count as usize);
        let mut out = Self::default();
        let mut pos = 24;
        let mut texts = 0;
        // spec: S-0015 claim 3 (entries follow the header with no gap; the reading ends normally when fewer than four bytes remain)
        while data.len() - pos >= 4 {
            if out.entries.len() >= entry_count {
                // spec: S-0015 claim 9 (entries beyond the header's entry count are not read; the importer warns)
                out.warnings.push(format!(
                    "string table: more entries than the header's {entry_count}; reading stopped at byte {pos}"
                ));
                return Ok(out);
            }
            match read_entry(data, pos) {
                Err(why) => {
                    // spec: S-0015 claim 9 (the reading stops at the first damaged entry; the entries read whole before it are kept; the importer warns where it stopped)
                    out.warnings.push(format!(
                        "string table: reading stopped at byte {pos}: {why}"
                    ));
                    return Ok(out);
                }
                Ok((label, first, n, end)) => {
                    if texts + n > text_count {
                        // spec: S-0015 claim 9 (the reading stops before an entry whose texts would take the texts read past the header's text count)
                        out.warnings.push(format!(
                            "string table: more texts than the header's {text_count}; reading stopped at byte {pos}"
                        ));
                        return Ok(out);
                    }
                    texts += n;
                    out.entries.push((label, first));
                    pos = end;
                }
            }
        }
        if out.entries.len() < entry_count || texts < text_count {
            // spec: S-0015 claim 9 (fewer entries or texts than the header's counts: the entries read are used, and the importer warns)
            out.warnings.push(format!(
                "string table: {} entries and {texts} texts read, the header counts {entry_count} and {text_count}",
                out.entries.len()
            ));
        }
        Ok(out)
    }
}

/// A synthetic table of invented labels and strings.
struct SyntheticCsf;

fn synthetic_table() -> Csf {
    let s = |t: &str| CsfString {
        units: t.encode_utf16().collect(),
        extra: None,
    };
    Csf {
        version: 3,
        language: 0,
        unused: 0,
        labels: vec![
            CsfLabel {
                name: b"Demo:Greeting".to_vec(),
                strings: vec![s("Hello from a synthetic table")],
            },
            CsfLabel {
                name: b"Demo:Accents".to_vec(),
                strings: vec![CsfString {
                    units: "Grüße, ¡hola!".encode_utf16().collect(),
                    extra: Some(b"voice-demo".to_vec()),
                }],
            },
            CsfLabel {
                name: b"Demo:Two".to_vec(),
                strings: vec![s("first"), s("second")],
            },
        ],
    }
}

impl FixtureWriter for SyntheticCsf {
    fn format(&self) -> &'static str {
        "csf"
    }
    fn name(&self) -> String {
        "demo-strings".into()
    }
    fn extension(&self) -> &'static str {
        "csf"
    }
    fn write(&self, out: &mut Vec<u8>) {
        out.extend(synthetic_table().write());
    }
}

/// Fixture writers for this module.
pub fn fixture_writers() -> Vec<Box<dyn FixtureWriter>> {
    vec![Box::new(SyntheticCsf)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let t = synthetic_table();
        let bytes = t.write();
        let back = Csf::parse(&bytes).unwrap();
        assert_eq!(back, t);
        assert_eq!(back.labels[1].strings[0].text(), "Grüße, ¡hola!");
        assert_eq!(back.write(), bytes);
    }

    #[test]
    fn strings_are_inverted_on_disk() {
        let bytes = synthetic_table().write();
        // First string unit 'H' (0x48) is stored inverted: 0xFFB7.
        let pos = bytes.windows(4).position(|w| w == STRING_ID).unwrap() + 8;
        assert_eq!(&bytes[pos..pos + 2], &[0xB7, 0xFF]);
    }

    /// A synthetic S-0015 file from raw parts: `entries` are already
    /// encoded entry bytes.
    fn raw(entries: &[Vec<u8>], entry_count: i32, text_count: i32) -> Vec<u8> {
        let mut out = b" FSC".to_vec();
        for v in [3, entry_count, text_count, 7, 9] {
            out.extend_from_slice(&i32::to_le_bytes(v));
        }
        for e in entries {
            out.extend_from_slice(e);
        }
        out
    }

    /// An entry: its label bytes and texts as (id, stored units, extra).
    fn entry(label: &[u8], texts: &[(&[u8; 4], Vec<u16>, Option<&[u8]>)]) -> Vec<u8> {
        let mut out = b" LBL".to_vec();
        out.extend_from_slice(&(texts.len() as i32).to_le_bytes());
        out.extend_from_slice(&(label.len() as i32).to_le_bytes());
        out.extend_from_slice(label);
        for (id, units, extra) in texts {
            out.extend_from_slice(*id);
            out.extend_from_slice(&(units.len() as i32).to_le_bytes());
            for u in units {
                out.extend_from_slice(&u.to_le_bytes());
            }
            if let Some(x) = extra {
                out.extend_from_slice(&(x.len() as i32).to_le_bytes());
                out.extend_from_slice(x);
            }
        }
        out
    }

    fn stored(s: &str) -> Vec<u16> {
        s.encode_utf16().map(|u| !u).collect()
    }

    /// spec: S-0015 claims 3 to 7 (layout, the complement, the early end at a stored 0 or 65535,
    /// the first text, an entry with no texts, an extended text, the label cut at a zero byte)
    #[test]
    fn s0015_reads_entries() {
        let mut early = stored("ab");
        early.push(0);
        early.extend(stored("zz"));
        let mut early2 = stored("ab");
        early2.push(65535);
        early2.extend(stored("zz"));
        let f = raw(
            &[
                entry(
                    b"One",
                    &[
                        (b" RTS", stored("first"), None),
                        (b" RTS", stored("2nd"), None),
                    ],
                ),
                entry(b"None", &[]),
                entry(b"Ext", &[(b"WRTS", stored("ext"), Some(b"data"))]),
                entry(b"Cut\0xy", &[(b" RTS", stored("c"), None)]),
                entry(b"Early", &[(b" RTS", early, None)]),
                entry(b"Early2", &[(b" RTS", early2, None)]),
                entry(b"Zero", &[(b" RTS", vec![], None)]),
            ],
            7,
            7,
        );
        let t = StringFile::read(&f).unwrap();
        let u = |s: &str| -> Vec<u16> { s.encode_utf16().collect() };
        let want: Vec<(Vec<u8>, Vec<u16>)> = vec![
            (b"One".to_vec(), u("first")),
            (b"None".to_vec(), vec![]),
            (b"Ext".to_vec(), u("ext")),
            (b"Cut".to_vec(), u("c")),
            (b"Early".to_vec(), u("ab")),
            (b"Early2".to_vec(), u("ab")),
            (b"Zero".to_vec(), vec![]),
        ];
        assert_eq!(t.entries, want);
        assert!(t.warnings.is_empty());
        // One to three trailing bytes are ignored, silently.
        let mut f3 = f.clone();
        f3.extend_from_slice(b"xyz");
        assert_eq!(StringFile::read(&f3).unwrap(), t);
    }

    /// spec: S-0015 claim 2 (the header's refusals; the version, unused value and language code
    /// have no effect)
    #[test]
    fn s0015_header_checks() {
        let e = entry(b"A", &[(b" RTS", stored("a"), None)]);
        let ok = raw(std::slice::from_ref(&e), 1, 1);
        assert!(StringFile::read(&ok).is_ok());
        assert!(StringFile::read(&ok[..20]).is_err());
        let mut bad = ok.clone();
        bad[0] = b'X';
        assert!(StringFile::read(&bad).is_err());
        for (n, t) in [(0, 1), (1, 0), (-1, 1), (1, -1)] {
            assert!(StringFile::read(&raw(std::slice::from_ref(&e), n, t)).is_err());
        }
        let mut other = ok.clone();
        other[4..8].copy_from_slice(&1i32.to_le_bytes());
        other[16..24].copy_from_slice(&[9; 8]);
        assert_eq!(
            StringFile::read(&other).unwrap(),
            StringFile::read(&ok).unwrap()
        );
    }

    /// spec: S-0015 claim 9 (the reading stops at the first damaged entry and keeps the entries
    /// before it; fewer entries or texts than the header's, more than the header's: the entries
    /// read whole are kept and a warning names the place)
    #[test]
    fn s0015_damage_keeps_whole_entries() {
        let a = entry(b"A", &[(b" RTS", stored("a"), None)]);
        let b = entry(b"B", &[(b" RTS", stored("b"), None)]);
        let mut broken = b.clone();
        broken[0] = b'X';
        let t = StringFile::read(&raw(&[a.clone(), broken, b.clone()], 3, 3)).unwrap();
        assert_eq!(t.entries.len(), 1);
        assert!(t.warnings[0].contains(&format!("byte {}", 24 + a.len())));
        // A wrong text id, a negative count, a run past the end.
        let mut bad_text = b.clone();
        bad_text[14] = b'Q';
        assert_eq!(
            StringFile::read(&raw(&[a.clone(), bad_text], 2, 2))
                .unwrap()
                .entries
                .len(),
            1
        );
        let mut neg = b.clone();
        neg[4..8].copy_from_slice(&(-1i32).to_le_bytes());
        assert_eq!(
            StringFile::read(&raw(&[a.clone(), neg], 2, 2))
                .unwrap()
                .entries
                .len(),
            1
        );
        let cut = &raw(&[a.clone(), b.clone()], 2, 2);
        assert_eq!(
            StringFile::read(&cut[..cut.len() - 1])
                .unwrap()
                .entries
                .len(),
            1
        );
        // Fewer than the header's counts: used, with a warning.
        let few = StringFile::read(&raw(&[a.clone()], 2, 2)).unwrap();
        assert_eq!((few.entries.len(), few.warnings.len()), (1, 1));
        // More entries than the header's: not read.
        let more = StringFile::read(&raw(&[a.clone(), b.clone()], 1, 2)).unwrap();
        assert_eq!((more.entries.len(), more.warnings.len()), (1, 1));
        // More texts than the header's: stops before the entry that passes it.
        let many = StringFile::read(&raw(&[a, b], 2, 1)).unwrap();
        assert_eq!((many.entries.len(), many.warnings.len()), (1, 1));
    }

    #[test]
    fn truncation_never_panics() {
        let bytes = synthetic_table().write();
        for cut in 0..bytes.len() {
            assert!(Csf::parse(&bytes[..cut]).is_err());
        }
        let mut huge = bytes.clone();
        huge[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(Csf::parse(&huge).is_err());
    }
}
