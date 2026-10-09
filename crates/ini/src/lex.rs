//! Lexical rules: bytes to sections and entries (S-0001).
//!
//! A document is a byte string with no encoding (S-0001 claim 1). Lines end
//! at line feeds, CR bytes vanish, lines are cut at 511 bytes and at a NUL,
//! and a final line with no line feed is ignored in a fresh load. Keys and
//! values are trimmed with the original's trimming defect.

use crate::{Quirks, Warning, WarningKind};

// spec: S-0001 claim 3 (line length limit)
const LINE_LIMIT: usize = 511;
// spec: S-0001 Terms (whitespace byte: values 1 to 32)
const HIGHEST_WHITESPACE: u8 = 32;
// spec: S-0001 claim 2 (CR is deleted wherever it appears)
const CR: u8 = 13;
// spec: S-0001 Terms (a line ends at a line feed)
const LF: u8 = 10;
// spec: S-0001 claim 1 (the 3-byte UTF-8 byte-order mark is not recognised)
const BOM: [u8; 3] = [239, 187, 191];

/// Whether `b` is a whitespace byte (S-0001 Terms): 1 to 32 inclusive.
pub fn is_whitespace(b: u8) -> bool {
    // spec: S-0001 Terms (bytes 128 to 255 and 0 are never whitespace)
    (1..=HIGHEST_WHITESPACE).contains(&b)
}

/// Lengths of the leading whitespace, the core and the trailing whitespace
/// of `s` (S-0001 Formulas: *L*, *C*, *T*). All-whitespace text has a core
/// of 0 bytes and counts as all leading.
fn split_lct(s: &[u8]) -> (usize, usize, usize) {
    let l = s.iter().take_while(|&&b| is_whitespace(b)).count();
    if l == s.len() {
        return (l, 0, 0);
    }
    let t = s.iter().rev().take_while(|&&b| is_whitespace(b)).count();
    (l, s.len() - l - t, t)
}

/// *K* of the trimming defect (S-0001 Formulas): the number of trailing
/// whitespace bytes that survive.
fn kept_trailing(l: usize, c: usize, t: usize, defect: bool) -> usize {
    // spec: S-0001 claim 10, Formulas (K = L - C if C < L <= C + T, else 0)
    if defect && c < l && l <= c + t {
        l - c
    } else {
        0
    }
}

/// Trims `s` with the original's defect when `defect` is on (S-0001 claim
/// 10). Returns the trimmed bytes and *K*.
pub fn trim_with_defect(s: &[u8], defect: bool) -> (&[u8], usize) {
    let (l, c, t) = split_lct(s);
    let k = kept_trailing(l, c, t, defect);
    (&s[l..l + c + k], k)
}

/// Trims without the defect: all leading and trailing whitespace goes.
pub fn trim_full(s: &[u8]) -> &[u8] {
    trim_with_defect(s, false).0
}

/// One classified line (S-0001 State and ordering).
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Line<'a> {
    /// A section header with its name, taken as written.
    Header(&'a [u8]),
    /// An entry; the flags say whether the defect kept whitespace.
    Entry {
        key: Vec<u8>,
        value: Vec<u8>,
        key_kept: bool,
        value_kept: bool,
    },
    /// Nothing: blank, comment, no `=`, or an empty key or value.
    Ignored,
}

/// Classifies one line as claims 2 to 4 leave it (S-0001 claims 6 to 11).
pub(crate) fn classify(line: &[u8], defect: bool) -> Line<'_> {
    // spec: S-0001 claim 6 (header: first non-whitespace byte `[`, and a `]` after it)
    if let Some(open) = line.iter().position(|&b| !is_whitespace(b))
        && line[open] == b'['
        && let Some(close) = line[open + 1..].iter().position(|&b| b == b']')
    {
        return Line::Header(&line[open + 1..open + 1 + close]);
    }
    // spec: S-0001 claim 8 (the first `;` starts a comment, wherever it stands)
    let (text, commented) = match line.iter().position(|&b| b == b';') {
        Some(i) => (&line[..i], true),
        None => (line, false),
    };
    // spec: S-0001 claim 9 (no `=` before the comment: ignored)
    let Some(eq) = text.iter().position(|&b| b == b'=') else {
        return Line::Ignored;
    };
    // spec: S-0001 claim 9 (an empty or all-whitespace key creates no entry, no re-split)
    if trim_full(&text[..eq]).is_empty() {
        return Line::Ignored;
    }
    let (key, key_k, value, value_k) = if commented {
        // spec: S-0001 Formulas (entry lines that carry a comment)
        let (l1, c1, t1) = split_lct(text);
        let k1 = kept_trailing(l1, c1, t1, defect);
        let core = &text[l1..l1 + c1];
        // `=` is not whitespace, so it lies inside the core.
        let eq_in_core = eq - l1;
        let key = trim_full(&core[..eq_in_core]).to_vec();
        let mut raw_value = core[eq_in_core + 1..].to_vec();
        raw_value.extend_from_slice(&text[l1 + c1..l1 + c1 + k1]);
        let (value, value_k) = trim_with_defect(&raw_value, defect);
        (key, 0, value.to_vec(), value_k)
    } else {
        // spec: S-0001 claims 9 and 10 (key and value each trimmed, with the defect)
        let (key, key_k) = trim_with_defect(&text[..eq], defect);
        let (value, value_k) = trim_with_defect(&text[eq + 1..], defect);
        (key.to_vec(), key_k, value.to_vec(), value_k)
    };
    // spec: S-0001 claim 11 (an empty key or value creates no entry)
    if key.is_empty() || value.is_empty() {
        return Line::Ignored;
    }
    Line::Entry {
        key,
        value,
        key_kept: key_k > 0,
        value_kept: value_k > 0,
    }
}

/// An entry as read, before tables are built.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawEntry {
    /// Trimmed key.
    pub key: Vec<u8>,
    /// Trimmed value (the stored value of S-0003).
    pub value: Vec<u8>,
    /// 1-based line number.
    pub line: u32,
}

/// A section as read, possibly empty.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawSection {
    /// Name as written between the brackets.
    pub name: Vec<u8>,
    /// 1-based line of the header.
    pub line: u32,
    /// Entries in file order.
    pub entries: Vec<RawEntry>,
}

/// Whether the final line (no line feed after it) is read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FinalLine {
    /// Follow `quirks.ini_final_line_needs_newline` (fresh load).
    PerQuirk,
    /// Always read it (the second file of a layered load, S-0002 claim 9).
    Read,
}

/// The kept content of one physical line (S-0001 claims 2 to 4).
fn kept_bytes(segment: &[u8]) -> Vec<u8> {
    let mut kept = Vec::with_capacity(segment.len().min(LINE_LIMIT));
    for &b in segment {
        // spec: S-0001 claim 4 (a NUL ends the line's content)
        if b == 0 || kept.len() == LINE_LIMIT {
            break;
        }
        // spec: S-0001 claims 2 and 3 (CR deleted; CR bytes do not count towards the limit)
        if b != CR {
            kept.push(b);
        }
    }
    kept
}

/// Splits `bytes` into sections (empty ones included), in file order.
pub(crate) fn lex(
    bytes: &[u8],
    quirks: &Quirks,
    final_line: FinalLine,
    warnings: &mut Vec<Warning>,
) -> Vec<RawSection> {
    // spec: S-0001 Edge cases (importer warning: byte-order mark)
    if bytes.starts_with(&BOM) {
        warnings.push(Warning::new(1, WarningKind::ByteOrderMark));
    }
    // spec: S-0001 Edge cases (importer warning: a CR not directly followed by a line feed)
    if let Some(i) = (0..bytes.len()).find(|&i| bytes[i] == CR && bytes.get(i + 1) != Some(&LF)) {
        let line = line_of(bytes, i);
        warnings.push(Warning::new(line, WarningKind::StrayCarriageReturn));
    }

    let mut sections: Vec<RawSection> = Vec::new();
    let segments: Vec<&[u8]> = bytes.split(|&b| b == LF).collect();
    let last = segments.len() - 1;
    for (i, segment) in segments.iter().enumerate() {
        let line_no = u32::try_from(i + 1).unwrap_or(u32::MAX);
        let kept = kept_bytes(segment);
        let line = classify(&kept, quirks.ini_trim_defect);
        // spec: S-0001 claim 5 (the final line, with no line feed after it, is ignored)
        let unterminated = i == last;
        if unterminated && final_line == FinalLine::PerQuirk && quirks.ini_final_line_needs_newline
        {
            let lost = match &line {
                Line::Header(_) => true,
                Line::Entry { .. } => !sections.is_empty(),
                Line::Ignored => false,
            };
            if lost {
                // spec: S-0001 Edge cases (warning: a lost final header or entry)
                warnings.push(Warning::new(line_no, WarningKind::FinalLineIgnored));
            }
            break;
        }
        match line {
            Line::Header(name) => sections.push(RawSection {
                name: name.to_vec(),
                line: line_no,
                entries: Vec::new(),
            }),
            Line::Entry {
                key,
                value,
                key_kept,
                value_kept,
            } => {
                // spec: S-0001 claim 7 (lines before the first header are ignored)
                let Some(section) = sections.last_mut() else {
                    continue;
                };
                if key_kept || value_kept {
                    // spec: S-0001 Edge cases (warning: the trimming defect kept whitespace)
                    warnings.push(Warning::new(
                        line_no,
                        WarningKind::TrimKeptWhitespace {
                            key: key.clone(),
                            in_key: key_kept,
                        },
                    ));
                }
                section.entries.push(RawEntry {
                    key,
                    value,
                    line: line_no,
                });
            }
            Line::Ignored => {}
        }
    }
    sections
}

/// 1-based line number of byte offset `at`.
fn line_of(bytes: &[u8], at: usize) -> u32 {
    let n = bytes[..at].iter().filter(|&&b| b == LF).count() + 1;
    u32::try_from(n).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(key: &str, value: &str) -> Line<'static> {
        Line::Entry {
            key: key.as_bytes().to_vec(),
            value: value.as_bytes().to_vec(),
            key_kept: false,
            value_kept: false,
        }
    }

    /// spec: S-0001 Formulas (the trim examples table)
    #[test]
    fn trim_defect_examples() {
        let cases: [(&str, &str); 5] = [
            ("  k  ", "k "),
            ("     abcd ", "abcd "),
            ("    abcd ", "abcd"),
            ("       k  ", "k"),
            ("k   ", "k"),
        ];
        for (raw, want) in cases {
            assert_eq!(
                trim_with_defect(raw.as_bytes(), true).0,
                want.as_bytes(),
                "{raw:?}"
            );
            assert_eq!(
                trim_with_defect(raw.as_bytes(), false).0,
                raw.trim().as_bytes()
            );
        }
        assert_eq!(trim_with_defect(b"   ", true).0, b"");
    }

    /// spec: S-0001 Formulas (comment example: `··········k=···v····;`)
    #[test]
    fn comment_line_formula() {
        let line = format!("{}k=   v    ;", " ".repeat(10));
        match classify(line.as_bytes(), true) {
            Line::Entry {
                key,
                value,
                value_kept,
                ..
            } => {
                assert_eq!(key, b"k");
                assert_eq!(value, b"v  ");
                assert!(value_kept);
            }
            other => panic!("{other:?}"),
        }
    }

    /// spec: S-0001 claims 6, 8, 9 and 11
    #[test]
    fn line_classes() {
        assert_eq!(classify(b"[ Alpha ]", true), Line::Header(b" Alpha "));
        assert_eq!(classify(b"  [a;b=c] ; x", true), Line::Header(b"a;b=c"));
        assert_eq!(classify(b"[]", true), Line::Header(b""));
        assert_eq!(classify(b"[x=5", true), entry("[x", "5"));
        assert_eq!(classify(b"[x", true), Line::Ignored);
        assert_eq!(classify(b"a=b=c", true), entry("a", "b=c"));
        assert_eq!(classify(b"=a=b", true), Line::Ignored);
        assert_eq!(classify(b"my key = v", true), entry("my key", "v"));
        assert_eq!(classify(b"k=12;34", true), entry("k", "12"));
        assert_eq!(classify(b";k=1", true), Line::Ignored);
        assert_eq!(classify(b"k=", true), Line::Ignored);
        assert_eq!(classify(b"k=   ", true), Line::Ignored);
        assert_eq!(classify(b"k= ; note", true), Line::Ignored);
        assert_eq!(classify(b"k 5", true), Line::Ignored);
    }

    /// spec: S-0001 claims 2, 3 and 4
    #[test]
    fn kept_line_bytes() {
        assert_eq!(kept_bytes(b"12\r3"), b"123");
        assert_eq!(kept_bytes(b"12\x0034"), b"12");
        let long = vec![b'a'; 600];
        assert_eq!(kept_bytes(&long).len(), LINE_LIMIT);
        let mut with_cr = vec![b'\r'; 10];
        with_cr.extend(vec![b'b'; 600]);
        assert_eq!(kept_bytes(&with_cr).len(), LINE_LIMIT);
    }
}
