//! Rules: our own data format for everything the simulation reads from the
//! user's import (PLAN.md 4.6).
//!
//! - [`jsonc`]: the JSONC reader and writer (spans, integer-only numbers).
//! - [`resolve`]: the resolve pipeline's merging steps: a base tree, patch
//!   layers in order, inheritance, `null` removal and set operations, with
//!   the history of every value ([`resolve::Provenance`]).
//! - [`de`] and [`schema`]: strict typed deserialization
//!   (`deny_unknown_fields`) into schema 1, and its JSON Schema.
//! - [`validate`]: semantic checks and ID interning.
//! - [`hash`]: the canonical binary form and `sim_rules_hash`.
//! - [`explain`]: `rules explain`.
//!
//! [`resolve_ruleset`] runs the whole pipeline. This is a simulation crate:
//! the determinism lints apply, and it never depends on the importer (`ini`,
//! `convert`) or on file formats (`fmt`).

pub mod de;
pub mod explain;
pub mod hash;
pub mod jsonc;
pub mod resolve;
pub mod schema;
pub mod validate;

use jsonc::Node;
use resolve::{Provenance, ResolveError, Resolver, Source};
use schema::Ruleset;
use validate::{Ids, Problem};

/// A resolved ruleset and everything derived from it.
#[derive(Clone, Debug)]
pub struct Resolved {
    /// The resolved tree.
    pub tree: Node,
    /// The typed ruleset.
    pub ruleset: Ruleset,
    /// History of every value.
    pub provenance: Provenance,
    /// Interned IDs.
    pub ids: Ids,
    /// Hash of the sim part (hexadecimal SHA-256).
    pub sim_rules_hash: String,
    /// Hash of the render part (informational).
    pub render_rules_hash: String,
}

/// Why a ruleset did not resolve.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// A layer did not parse or merge, or inheritance failed.
    Resolve(Vec<ResolveError>),
    /// The resolved tree does not match the schema.
    Schema(de::Error),
    /// The typed ruleset has semantic problems.
    Invalid(Vec<Problem>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resolve(errors) => {
                for e in errors {
                    writeln!(f, "{e}")?;
                }
                Ok(())
            }
            Self::Schema(e) => write!(f, "schema: {e}"),
            Self::Invalid(problems) => {
                for p in problems {
                    writeln!(f, "{p}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for Error {}

/// Runs the resolve pipeline (PLAN.md 4.6): an optional base tree with its
/// history (the converter's output), then each patch layer in order, then
/// inheritance and set operations, typed deserialization, validation, ID
/// interning and hashing.
pub fn resolve_ruleset(
    base: Option<(Node, Provenance)>,
    layers: &[Source],
) -> Result<Resolved, Error> {
    let mut resolver = match base {
        Some((tree, provenance)) => Resolver::with_base(tree, provenance),
        None => Resolver::default(),
    };
    for layer in layers {
        resolver.apply(layer).map_err(|e| Error::Resolve(vec![e]))?;
    }
    let (tree, provenance) = resolver.finish().map_err(Error::Resolve)?;
    let ruleset: Ruleset = de::from_node(&tree).map_err(Error::Schema)?;
    let problems = validate::validate(&ruleset);
    if !problems.is_empty() {
        return Err(Error::Invalid(problems));
    }
    let ids = validate::intern(&ruleset, &tree);
    Ok(Resolved {
        sim_rules_hash: hash::sim_rules_hash(&tree),
        render_rules_hash: hash::render_rules_hash(&tree),
        tree,
        ruleset,
        provenance,
        ids,
    })
}
