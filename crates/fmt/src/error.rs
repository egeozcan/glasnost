//! The error type shared by every parser in this crate.
//!
//! Parsers never panic on malformed input; they return one of these. The
//! variants carry only offsets, sizes and static descriptions, never file
//! contents, so they are safe to log (CONTRIBUTING.md: logs never include
//! asset content).

/// Why a parser rejected its input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// The input ended early. `needed` is the total input length, counted
    /// from the start of the parsed buffer, that the parser would need to
    /// make progress. Incremental readers (the MIX index reader) use it to
    /// decide how much more to fetch.
    Truncated {
        /// Total bytes needed from the start of the buffer.
        needed: u64,
    },
    /// A structural check failed.
    Invalid(&'static str),
    /// Well-formed, but a variant this parser does not handle.
    Unsupported(&'static str),
    /// Decoding would exceed a size limit (see [`crate::limits`]).
    TooLarge(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Truncated { needed } => write!(f, "input truncated ({needed} bytes needed)"),
            Self::Invalid(what) => write!(f, "invalid data: {what}"),
            Self::Unsupported(what) => write!(f, "unsupported: {what}"),
            Self::TooLarge(what) => write!(f, "too large: {what}"),
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    /// A short, stable name for the variant, for counting failures by kind.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Truncated { .. } => "truncated",
            Self::Invalid(_) => "invalid",
            Self::Unsupported(_) => "unsupported",
            Self::TooLarge(_) => "too-large",
        }
    }
}

/// Result alias for this crate.
pub type Result<T> = std::result::Result<T, Error>;
