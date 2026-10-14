//! INI reader used **only by the importer** (`convert`).
//!
//! It reads rules, art and map documents exactly as the original does:
//!
//! - [`lex`]: bytes to sections and entries, line limits, comments and the
//!   trimming defect (S-0001);
//! - [`table`] and [`Document`]: duplicate keys and sections, the name hash
//!   and the hash-order rule that picks the winning occurrence, layered
//!   loads (S-0002);
//! - [`value`]: text, booleans, integers, decimals and percentages, tuples,
//!   byte triples, lists, type names and named values (S-0003), with
//!   [`binfloat`] doing the exact binary conversions in integer arithmetic.
//!
//! Every quirk of these specs is a switch in [`Quirks`], on by default (the
//! original's behaviour). Inputs the specs name as worth reporting produce a
//! [`Warning`]; a warning never changes the result. There is no INI rules
//! engine at runtime: the simulation reads only the typed ruleset produced
//! by `convert` and `rules`.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod binfloat;
mod doc;
pub mod hostile;
pub mod lex;
pub mod table;
pub mod value;

pub use doc::{Document, Entry, Section};

/// Quirk switches of S-0001, S-0002 and S-0003. Each defaults to the
/// original's behaviour.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Quirks {
    /// `quirks.ini_final_line_needs_newline` (S-0001 claim 5): in a fresh
    /// load, a final line with no line feed after it is ignored.
    pub ini_final_line_needs_newline: bool,
    /// `quirks.ini_trim_defect` (S-0001 claim 10): trimming keeps *K*
    /// trailing whitespace bytes.
    pub ini_trim_defect: bool,
    /// `quirks.ini_duplicate_hash_order` (S-0002 claims 1 and 2): the
    /// hash-order rule picks the winning occurrence; off, the last does.
    pub ini_duplicate_hash_order: bool,
    /// `quirks.ini_hash_identity` (S-0002 claim 3): names with equal hashes
    /// are one name; off, names must be equal byte for byte.
    pub ini_hash_identity: bool,
    /// `quirks.percent_list_integer_prefix` (S-0003 claim 13): an item of a
    /// fixed-count percentage list that contains `%` uses only its integer
    /// prefix; off, its full decimal at 53 bits.
    pub percent_list_integer_prefix: bool,
    /// `quirks.list_items_untrimmed` (S-0003 claim 12): list items keep
    /// the whitespace around them; off, name items are trimmed (S-0001's
    /// trim without its defect) and an item of only whitespace is skipped.
    pub list_items_untrimmed: bool,
}

impl Default for Quirks {
    fn default() -> Self {
        // spec: S-0001 Edge cases, S-0002 Edge cases, S-0003 Edge cases (all on by default, for every input)
        Self {
            ini_final_line_needs_newline: true,
            ini_trim_defect: true,
            ini_duplicate_hash_order: true,
            ini_hash_identity: true,
            percent_list_integer_prefix: true,
            list_items_untrimmed: true,
        }
    }
}

/// An importer warning about one place in one file (glossary, *importer
/// warning*). The caller knows the file; the warning gives the line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Warning {
    /// 1-based line.
    pub line: u32,
    /// What was found.
    pub kind: WarningKind,
}

impl Warning {
    /// A warning at `line`.
    pub fn new(line: u32, kind: WarningKind) -> Self {
        Self { line, kind }
    }
}

/// What a [`Warning`] reports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WarningKind {
    /// The file starts with a byte-order mark (S-0001 claim 1).
    ByteOrderMark,
    /// A CR byte not directly followed by a line feed (S-0001 claim 2).
    StrayCarriageReturn,
    /// The trimming defect kept trailing whitespace on a key or value
    /// (S-0001 claim 10).
    TrimKeptWhitespace {
        /// The key as stored.
        key: Vec<u8>,
        /// The key kept whitespace (else the value did).
        in_key: bool,
    },
    /// The final line has no line feed and would have been a header or an
    /// entry (S-0001 claim 5).
    FinalLineIgnored,
    /// A key occurs more than once in one section (S-0002 claim 1).
    DuplicateKey {
        /// The section.
        section: Vec<u8>,
        /// The key.
        key: Vec<u8>,
        /// Line of its first occurrence.
        first_line: u32,
    },
    /// Two non-empty sections share a name (S-0002 claim 2).
    DuplicateSection {
        /// The name.
        name: Vec<u8>,
        /// Line of the first copy's header.
        first_line: u32,
    },
    /// Two different names in one table share a name hash (S-0002 claim 3).
    HashCollision {
        /// The section whose keys collide, or `None` for section names.
        section: Option<Vec<u8>>,
        /// The later name.
        name: Vec<u8>,
        /// The earlier name with the same hash.
        other: Vec<u8>,
        /// Line of the earlier name.
        other_line: u32,
    },
    /// A line of a layered load's second file replaced an entry of a
    /// different name, or wrote into a section of a different name, because
    /// the names share a name hash (S-0002 claim 11).
    LayeredSharedHash {
        /// The section whose key was replaced, or `None` for a section the
        /// line wrote into.
        section: Option<Vec<u8>>,
        /// The name in the second file.
        name: Vec<u8>,
        /// The name already in the document.
        other: Vec<u8>,
    },
    /// A layered load changes a table of the earlier file that holds a
    /// repeated name hash (S-0002 claim 12).
    LayeredOntoRepeatedHash {
        /// The section whose keys the load changes, or `None` for the
        /// section names.
        section: Option<Vec<u8>>,
    },
}

fn show(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

impl std::fmt::Display for Warning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: ", self.line)?;
        match &self.kind {
            WarningKind::ByteOrderMark => write!(
                f,
                "the file starts with a byte-order mark; a header on the first line is not \
                 recognised (S-0001 claim 1)"
            ),
            WarningKind::StrayCarriageReturn => write!(
                f,
                "a CR byte not followed by a line feed is deleted and joins its neighbours \
                 (S-0001 claim 2)"
            ),
            WarningKind::TrimKeptWhitespace { key, in_key } => write!(
                f,
                "deep indentation keeps trailing whitespace on the {} of {:?} (S-0001 claim 10)",
                if *in_key { "key" } else { "value" },
                show(key)
            ),
            WarningKind::FinalLineIgnored => write!(
                f,
                "the last line has no line feed after it and is ignored (S-0001 claim 5)"
            ),
            WarningKind::DuplicateKey {
                section,
                key,
                first_line,
            } => write!(
                f,
                "key {:?} repeats in [{}] (first on line {first_line}); the winner is chosen by \
                 name hash, not by position (S-0002 claim 1)",
                show(key),
                show(section)
            ),
            WarningKind::DuplicateSection { name, first_line } => write!(
                f,
                "section [{}] repeats (first on line {first_line}); only one copy is used and \
                 copies are not merged (S-0002 claim 2)",
                show(name)
            ),
            WarningKind::HashCollision {
                section,
                name,
                other,
                other_line,
            } => write!(
                f,
                "{:?} has the same name hash as {:?} (line {other_line}){}; lookups of either \
                 can find the other (S-0002 claim 3)",
                show(name),
                show(other),
                section
                    .as_ref()
                    .map(|s| format!(" in [{}]", show(s)))
                    .unwrap_or_default()
            ),
            WarningKind::LayeredSharedHash {
                section,
                name,
                other,
            } => match section {
                Some(s) => write!(
                    f,
                    "{:?} replaces {:?} in [{}] because the two share a name hash; the section \
                     now holds the key as {:?} (S-0002 claim 11)",
                    show(name),
                    show(other),
                    show(s),
                    show(name)
                ),
                None => write!(
                    f,
                    "[{}] writes into section [{}] because the two share a name hash; the \
                     section keeps its name (S-0002 claim 11)",
                    show(name),
                    show(other)
                ),
            },
            WarningKind::LayeredOntoRepeatedHash { section } => write!(
                f,
                "the layered load changes {}, which already hold two occurrences with one name \
                 hash; the original's result may differ from this engine's (S-0002 claim 12)",
                match section {
                    Some(s) => format!("the keys of [{}]", show(s)),
                    None => "the section names".to_owned(),
                }
            ),
        }
    }
}
