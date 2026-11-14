//! A warhead's versus items in our rules format (#293). The importer writes
//! each item as the exact decimal of its double, after applying S-0003 claim
//! 13's `%` readings to the INI text; an item written with `%` by hand has no
//! reading in our format and is refused when the rules load.

use glasnost_rules::resolve::Source;
use glasnost_rules::{Error, resolve_ruleset};

fn load(text: &str) -> Result<glasnost_rules::Resolved, Error> {
    resolve_ruleset(
        None,
        &[Source {
            layer: "mod",
            file: "mod.jsonc",
            text,
        }],
    )
}

// spec: S-0003 claim 13 (the importer applies the `%` readings and writes plain doubles); S-0304 (versus values)
#[test]
fn a_percent_versus_item_is_refused() {
    let text = r#"{"schema": 1, "sim": {"warheads": {"Pow": {
        "verses": ["1", "50%", "0.5", "1", "1", "1", "1", "1", "1", "1", "1"],
        "prone_damage": "50%"
    }}}}"#;
    match load(text) {
        Err(Error::Invalid(problems)) => {
            assert!(
                problems
                    .iter()
                    .any(|p| p.path == "sim.warheads.Pow.verses" && p.message.contains("50%")),
                "{problems:?}"
            );
        }
        other => panic!("a `%` versus item must be refused, got {:?}", other.err()),
    }
}

// spec: S-0003 claim 13 (items without `%` are doubles); S-0304
#[test]
fn plain_versus_items_load() {
    let text = r#"{"schema": 1, "sim": {"warheads": {"Pow": {
        "verses": ["1", "0.5", "0.25", "1", "1", "1", "1", "1", "1", "1", "1"],
        "prone_damage": "50%"
    }}}}"#;
    let r = load(text).expect("plain items load");
    assert_eq!(
        r.ruleset.sim.warheads["Pow"].verses.as_ref().map(Vec::len),
        Some(11)
    );
}
