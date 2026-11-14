//! A parent's set field written as a set operation is inherited as its
//! list (#328).

use glasnost_rules::resolve::Source;
use glasnost_rules::resolve_ruleset;

fn owners(text: &str, id: &str) -> Vec<String> {
    let src = Source {
        layer: "test",
        file: "layer0.jsonc",
        text,
    };
    let r = resolve_ruleset(None, &[src]).unwrap_or_else(|e| panic!("{e}"));
    r.ruleset.objects[id].owners.clone()
}

const OWNERS: &str = r#""schema": 1, "owners": {"a": {}, "b": {}, "c": {}}"#;

// nocite: our own rules format (PLAN.md 4.6: set operations apply to the inherited list), #328 (control)
#[test]
fn parent_written_as_list() {
    let text = format!(
        r#"{{{OWNERS}, "objects": {{
            "p": {{"owners": ["b"]}},
            "c": {{"inherits": ["p"], "owners": {{"+": ["c"]}}}}
        }}}}"#
    );
    assert_eq!(owners(&text, "p"), ["b"]);
    assert_eq!(owners(&text, "c"), ["b", "c"]);
}

// nocite: our own rules format (PLAN.md 4.6: set operations apply to the inherited list), #328
#[test]
fn parent_written_as_set_operation() {
    let text = format!(
        r#"{{{OWNERS}, "objects": {{
            "p": {{"owners": {{"+": ["b"]}}}},
            "c": {{"inherits": ["p"], "owners": {{"+": ["c"]}}}}
        }}}}"#
    );
    assert_eq!(owners(&text, "p"), ["b"]);
    assert_eq!(owners(&text, "c"), ["b", "c"]);
}

// nocite: our own rules format (PLAN.md 4.6: `inherits` is ordered, later wins), #328
#[test]
fn a_later_parent_replaces_whichever_way_it_is_written() {
    for p2 in [r#"["b"]"#, r#"{"+": ["b"]}"#] {
        let text = format!(
            r#"{{{OWNERS}, "objects": {{
                "p1": {{"owners": ["a"]}},
                "p2": {{"owners": {p2}}},
                "c": {{"inherits": ["p1", "p2"]}}
            }}}}"#
        );
        assert_eq!(owners(&text, "p2"), ["b"], "{p2}");
        assert_eq!(owners(&text, "c"), ["b"], "{p2}");
    }
}
