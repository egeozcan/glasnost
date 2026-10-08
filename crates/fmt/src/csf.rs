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
