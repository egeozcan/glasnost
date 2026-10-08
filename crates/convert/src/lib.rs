//! Converter from the user's install to our own formats.
//!
//! **Milestone M1** fills the rules side: INI layers → our JSONC tree,
//! `coverage.csv`, and the sim/render split of art data. **Milestone M2** adds
//! map conversion. Conversion runs locally on the user's machine; its output is
//! never committed.
