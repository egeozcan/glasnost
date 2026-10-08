//! Rules: our own data format for everything the simulation reads from the
//! user's import (PLAN.md 4.6).
//!
//! **Milestone M0**: the [`jsonc`] reader (spans, integer-only numbers) and a
//! synthetic test ruleset under `data/synthetic/ruleset/`, invented by us and
//! validated by this crate's tests.
//!
//! **Milestone M1**: schema types with schemars output, the serde
//! deserializer over the JSONC tree, merge-patch mod layers, inheritance and
//! set operations, semantic validation, ID interning, `rules explain`, and
//! the canonical `sim_rules_hash`. Every absent-key default in the schema
//! will cite its spec (`/// spec: S-xxxx`).
//!
//! This is a simulation crate: the determinism lints apply, and it never
//! depends on the importer (`ini`, `convert`) or on file formats (`fmt`).

pub mod jsonc;
