//! INI reader used **only by the importer** (`convert`).
//!
//! **Milestone M1** fills this crate. Its parsing semantics (case, duplicate
//! keys and sections, value forms, comments, list splitting, registration
//! order) must come from approved specs in `spec/rules-import/`; none exist
//! yet. There is no INI rules engine at runtime: the simulation reads only the
//! typed ruleset produced by `convert` and `rules`.
