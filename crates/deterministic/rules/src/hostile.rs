//! Hostile-input entry point: drives the JSONC reader and the resolve
//! pipeline over arbitrary bytes. Used by `tests/hostile.rs` (stable, in
//! CI) and the `jsonc` cargo-fuzz target. Any panic is a bug.

use crate::jsonc;
use crate::resolve::Source;

/// Parses `data` as JSONC, writes it back and parses that again, and runs
/// it through the resolve pipeline as a base layer and as a patch.
pub fn run(data: &[u8]) {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(node) = jsonc::parse(text) else {
        return;
    };
    let written = jsonc::write(&node);
    // nocite: a round-trip check, not behaviour
    assert!(jsonc::parse(&written).is_ok(), "written JSONC must parse");
    let src = Source {
        layer: "hostile",
        file: "hostile.jsonc",
        text,
    };
    let _ = crate::resolve_ruleset(None, &[src]);
    let _ = crate::resolve_ruleset(Some((node, Default::default())), &[src]);
}
