//! An own-format collection does not replace the imported registry of the
//! same name in the interned IDs (#325).

use glasnost_rules::resolve::Source;
use glasnost_rules::resolve_ruleset;

const IMPORT: &str = r#"{
  "schema": 1,
  "registries": {"weapons": ["A", "B", "C"]},
  "types": {"weapons": {
    "A": {"index": 0, "created": "list"},
    "B": {"index": 1, "created": "list"},
    "C": {"index": 2, "created": "list"}
  }}
}"#;

fn src<'a>(name: &'a str, text: &'a str) -> Source<'a> {
    Source {
        layer: name,
        file: name,
        text,
    }
}

// nocite: our own rules format (PLAN.md 4.6 step 6: imported kinds keep their registration order, S-0004 for the order itself), #325
#[test]
fn an_own_weapons_collection_keeps_the_imported_weapons_registry() {
    let plain = resolve_ruleset(None, &[src("import", IMPORT)]).unwrap();
    assert_eq!(plain.ids.get("weapons", "B"), Some(1));

    let layer = r#"{"weapons": {"mymod:laser": {"damage": 1}}}"#;
    let modded = resolve_ruleset(None, &[src("import", IMPORT), src("mod", layer)]).unwrap();
    assert_eq!(modded.ruleset.registries["weapons"], ["A", "B", "C"]);
    assert_eq!(
        modded.ids.kinds["weapons"],
        ["A", "B", "C"],
        "imported kind in registration order"
    );
    assert_eq!(modded.ids.get("weapons", "B"), Some(1));
    assert_eq!(modded.ids.get("weapons", "mymod:laser"), None);
}
