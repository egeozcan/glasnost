//! The importer's input files (S-0005 Import mapping): which copy is used,
//! the databases the game cannot start without, and the integrity digest
//! of the language rules file (S-0005 claims 16, 21 and 21b; S-0006 claim
//! 12).

use base64::Engine as _;
use glasnost_ini::value::{self, COMMON_TEXT_LIMIT};
use glasnost_ini::{Document, Quirks, Warning as IniWarning};
use glasnost_vfs::{Resolved, Vfs};
use sha1::Sha1;
use sha2::{Digest, Sha256};

/// The base rules (S-0005 Import mapping).
pub const BASE_RULES: &str = "rulesmd.ini"; // spec: S-0005 Import mapping
/// The art database.
pub const ART: &str = "artmd.ini"; // spec: S-0005 Import mapping
/// The language rules file (S-0005 claim 21).
pub const LANGUAGE_RULES: &str = "langrule.ini"; // spec: S-0005 Import mapping
/// Databases the game refuses to start without (S-0005 claim 16): they must
/// resolve and hold at least one section.
// spec: S-0005 claim 16 (art, sound, speech and music databases)
pub const REQUIRED_DATABASES: [&str; 4] = ["artmd.ini", "soundmd.ini", "evamd.ini", "thememd.ini"];
/// The integrity digest section (S-0005 Import mapping).
const DIGEST: &[u8] = b"Digest"; // spec: S-0005 Import mapping (integrity digest section)
/// Length of the checksum (S-0005 Constants).
const CHECKSUM_LEN: usize = 20; // spec: S-0005 Constants (integrity digest checksum length)

/// One input file as read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputFile {
    /// Requested name.
    pub name: String,
    /// Where it was found: an archive path, or `loose file NAME`.
    pub origin: String,
    /// Its bytes.
    pub bytes: Vec<u8>,
}

impl InputFile {
    /// `name (origin)`, for histories and warnings.
    pub fn label(&self) -> String {
        format!("{} ({})", self.name, self.origin)
    }

    /// SHA-256 of the bytes, hexadecimal.
    pub fn sha256(&self) -> String {
        Sha256::digest(&self.bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }
}

/// Everything read for one import.
#[derive(Clone, Debug, Default)]
pub struct Inputs {
    /// The base rules; `None` when missing (treated as empty, S-0005 claim
    /// 16).
    pub base: Option<InputFile>,
    /// The language rules file, if it resolves.
    pub language: Option<InputFile>,
    /// The game-mode file, if one was named and resolves.
    pub mode: Option<InputFile>,
    /// The map, if given.
    pub map: Option<InputFile>,
    /// The art database.
    pub art: Option<InputFile>,
    /// Required databases that did not resolve or hold no section.
    pub missing_databases: Vec<String>,
    /// Import-file names a loose file would supply if the mod layer were on
    /// (PLAN.md 4.3).
    pub shadowing_loose: Vec<String>,
    /// Top-level file names that differ only in letter case (S-0005 Edge
    /// cases).
    pub case_clashes: Vec<(String, String)>,
}

/// Reads one file through the game's search order.
pub async fn read(vfs: &Vfs, name: &str) -> Option<InputFile> {
    let r = vfs.resolve(name)?;
    let bytes = vfs.read_resolved(&r).await.ok()?;
    let origin = match &r {
        Resolved::Loose { name, .. } => format!("loose file {name}"),
        Resolved::Member(_) => vfs.describe(&r),
    };
    Some(InputFile {
        name: name.to_owned(),
        origin,
        bytes,
    })
}

/// Reads the inputs of an import. `mode` names the game-mode file
/// (Q-0006: the mode list is not read).
pub async fn read_inputs(
    vfs: &Vfs,
    mode: Option<&str>,
    map: Option<InputFile>,
    quirks: Quirks,
) -> Inputs {
    let mut inputs = Inputs {
        base: read(vfs, BASE_RULES).await,
        language: read(vfs, LANGUAGE_RULES).await,
        art: read(vfs, ART).await,
        map,
        case_clashes: vfs.case_clashes().to_vec(),
        ..Inputs::default()
    };
    if let Some(m) = mode {
        inputs.mode = read(vfs, m).await;
    }
    for name in REQUIRED_DATABASES {
        let ok = match read(vfs, name).await {
            Some(f) => !Document::parse(&f.bytes, quirks, &mut Vec::new()).is_empty(),
            None => false,
        };
        if !ok {
            inputs.missing_databases.push(name.to_owned());
        }
    }
    let mut names = vec![BASE_RULES, LANGUAGE_RULES];
    names.extend(REQUIRED_DATABASES);
    names.extend(mode);
    for n in names {
        if vfs.shadowing_loose(n).is_some() && !inputs.shadowing_loose.iter().any(|x| x == n) {
            inputs.shadowing_loose.push(n.to_owned());
        }
    }
    inputs
}

/// The outcome of the integrity digest check (S-0005 claim 21b).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DigestCheck {
    /// The file holds no section: accepted without a check, changes nothing.
    Empty,
    /// The digest section is present and matches.
    Valid,
    /// The digest section is missing (or decodes to nothing).
    Missing,
    /// The digest does not match (or is not valid base64, or decodes to
    /// fewer than 20 bytes: our engine treats both as a mismatch).
    Mismatch,
}

/// The text form whose SHA-1 the digest holds (S-0005 Formulas,
/// *Integrity digest*): every non-empty section in document order except
/// the digest section that a lookup finds, each as `[name]` CRLF and
/// `key=value` CRLF per entry, sections joined by CRLF.
pub fn digest_text(doc: &Document) -> Vec<u8> {
    let skip = doc.section_index(DIGEST);
    // spec: S-0005 Formulas (L = CR LF)
    let crlf: &[u8] = b"\r\n";
    let mut out = Vec::new();
    let mut first = true;
    for (i, s) in doc.sections().iter().enumerate() {
        if Some(i) == skip {
            continue;
        }
        if !first {
            out.extend_from_slice(crlf);
        }
        first = false;
        out.push(b'[');
        out.extend_from_slice(s.name());
        out.push(b']');
        out.extend_from_slice(crlf);
        for e in s.entries() {
            out.extend_from_slice(&e.key);
            out.push(b'=');
            out.extend_from_slice(&e.value);
            out.extend_from_slice(crlf);
        }
    }
    out
}

/// Checks the integrity digest of a language rules file (S-0005 claims 21
/// and 21b).
pub fn check_digest(doc: &Document) -> DigestCheck {
    if doc.is_empty() {
        // spec: S-0005 claim 21 (a file with no section is accepted without any check)
        return DigestCheck::Empty;
    }
    let Some(section) = doc.section(DIGEST) else {
        return DigestCheck::Missing;
    };
    // spec: S-0005 claim 21b, Formulas (each entry's piece: the lookup's value, cut to 127 bytes, trailing whitespace removed)
    let mut text = Vec::new();
    for e in section.entries() {
        let w = section.value(&e.key).unwrap_or(&e.value);
        text.extend_from_slice(value::text(w, COMMON_TEXT_LIMIT));
    }
    let decoded = match base64::engine::general_purpose::STANDARD.decode(&text) {
        Ok(d) => d,
        Err(_) => return DigestCheck::Mismatch,
    };
    if decoded.is_empty() {
        // spec: S-0005 Formulas (a section whose text decodes to no bytes counts as missing)
        return DigestCheck::Missing;
    }
    if decoded.len() < CHECKSUM_LEN {
        return DigestCheck::Mismatch;
    }
    let checksum = Sha1::digest(digest_text(doc));
    if decoded[..CHECKSUM_LEN] == checksum[..] {
        DigestCheck::Valid
    } else {
        DigestCheck::Mismatch
    }
}

/// Parses `file` and labels its warnings.
pub fn parse(
    file: &InputFile,
    quirks: Quirks,
    warnings: &mut Vec<(String, IniWarning)>,
) -> Document {
    let mut w = Vec::new();
    let doc = Document::parse(&file.bytes, quirks, &mut w);
    let label = file.label();
    warnings.extend(w.into_iter().map(|w| (label.clone(), w)));
    doc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(text: &str) -> Document {
        Document::parse(text.as_bytes(), Quirks::default(), &mut Vec::new())
    }

    fn with_digest(body: &str, split: usize) -> String {
        let d = doc(body);
        let sum = Sha1::digest(digest_text(&d));
        let b64 = base64::engine::general_purpose::STANDARD.encode(sum);
        let (a, b) = b64.split_at(split);
        format!("{body}[Digest]\n1={a}\n2={b}   \n")
    }

    /// spec: S-0005 claim 21b (text form, SHA-1, base64 over entries in order)
    #[test]
    fn digests() {
        let body = "; comment\n[A]\nk = v\nempty=\n[B]\nx=1\n";
        assert_eq!(digest_text(&doc(body)), b"[A]\r\nk=v\r\n\r\n[B]\r\nx=1\r\n");
        assert_eq!(
            check_digest(&doc(&with_digest(body, 10))),
            DigestCheck::Valid
        );
        // Comments, blank lines and trimmed whitespace make no difference.
        let reformatted = with_digest(body, 10).replace("k = v", "  k=v ;c");
        assert_eq!(check_digest(&doc(&reformatted)), DigestCheck::Valid);
        // Entry order and letter case do.
        let changed = with_digest(body, 10).replace("x=1", "X=1");
        assert_eq!(check_digest(&doc(&changed)), DigestCheck::Mismatch);
        assert_eq!(check_digest(&doc(body)), DigestCheck::Missing);
        assert_eq!(check_digest(&doc("")), DigestCheck::Empty);
        assert_eq!(
            check_digest(&doc("[A]\nk=v\n[Digest]\n1=@@@@\n")),
            DigestCheck::Mismatch
        );
    }
}
