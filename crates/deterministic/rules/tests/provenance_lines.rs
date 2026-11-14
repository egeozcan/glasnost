//! The line of every recorded origin is the line `jsonc::line_col` gives
//! the value's offset in its layer's text (#397: the lines must not change
//! when the resolver stops scanning the text from its start for each one).

use glasnost_rules::jsonc::{self, Node, Value};
use glasnost_rules::resolve::{Event, Resolver, Source};

/// Awkward layer texts: line feeds of every kind, multi-byte characters,
/// comments, blank lines, no final line feed.
fn texts() -> Vec<String> {
    vec![
        "{\"a\": 1, \"b\": {\"c\": \"x\", \"d\": [1, 2]}}".into(),
        "{\n  \"a\": 1,\n  \"b\": {\n    \"c\": \"x\"\n  }\n}\n".into(),
        "{\r\n  \"a\": 1,\r\n\r\n  \"b\": {\"c\": true}\r\n}".into(),
        "\n\n\n{\n// note\n\"ä\": \"ö\", /* ü\n\n */ \"é\": {\"🙂\": null, \"k\": \"日本\"},\n\n\n\"z\": 3}"
            .into(),
        "{\"a\": {}, \"b\": [], \"c\": {\"d\": {\"e\": {\"f\": 0}}}}\n\n".into(),
        {
            let mut s = String::from("{\n");
            for i in 0..300 {
                let pad = "é".repeat(i % 7);
                s.push_str(&format!("  \"k{i}\": {{\"v\": \"{pad}\", \"n\": {i}}},\n"));
                if i % 11 == 0 {
                    s.push_str("\r\n\n");
                }
            }
            s.push_str("  \"end\": 0\n}");
            s
        },
    ]
}

/// Every leaf of `node` (a non-empty object recurses) with its path.
fn leaves<'a>(node: &'a Node, path: &mut Vec<String>, out: &mut Vec<(Vec<String>, &'a Node)>) {
    match &node.value {
        Value::Object(members) if !members.is_empty() => {
            for m in members {
                path.push(m.key.clone());
                leaves(&m.value, path, out);
                path.pop();
            }
        }
        _ => out.push((path.clone(), node)),
    }
}

// nocite: provenance of our own rules format (PLAN.md 4.6), not game behaviour
#[test]
fn every_origin_line_is_line_col_of_its_value() {
    for (i, text) in texts().iter().enumerate() {
        let node = jsonc::parse(text).unwrap_or_else(|e| panic!("text {i}: {e}"));
        let mut r = Resolver::default();
        r.apply(&Source {
            layer: "mod 1: test",
            file: "test.jsonc",
            text,
        })
        .unwrap();
        let (_, provenance) = r.finish().unwrap();
        let mut all = Vec::new();
        leaves(&node, &mut Vec::new(), &mut all);
        assert!(!all.is_empty());
        for (path, leaf) in all {
            // A null removes, and an empty object merges nothing: no origin is set.
            if matches!(leaf.value, Value::Null | Value::Object(_)) {
                continue;
            }
            let want = jsonc::line_col(text, leaf.span.start).0 as u32;
            let got = provenance.get(&path);
            assert!(
                got.iter().any(|o| o.event == Event::Set && o.line == want),
                "text {i}, {path:?}: {got:?}, want line {want}"
            );
        }
    }
}

// nocite: provenance of our own rules format (PLAN.md 4.6), not game behaviour
#[test]
fn a_removal_is_recorded_at_the_line_of_its_null() {
    let base = "{\"a\": {\"b\": 1, \"c\": 2}, \"d\": 3}";
    let patch = "{\r\n\n  \"ä\": 0,\n  \"a\": {\n    \"b\": null\n  },\n\n  \"d\":\r\n    null}";
    let mut r = Resolver::default();
    for (n, text) in [base, patch].into_iter().enumerate() {
        r.apply(&Source {
            layer: if n == 0 { "base" } else { "mod 1: patch" },
            file: "f.jsonc",
            text,
        })
        .unwrap();
    }
    let node = jsonc::parse(patch).unwrap();
    let mut all = Vec::new();
    leaves(&node, &mut Vec::new(), &mut all);
    let (_, provenance) = r.finish().unwrap();
    let mut removals = 0;
    for (path, leaf) in all {
        if !matches!(leaf.value, Value::Null) {
            continue;
        }
        let want = jsonc::line_col(patch, leaf.span.start).0 as u32;
        let got = provenance.get(&path);
        assert!(
            got.iter()
                .any(|o| o.event == Event::Removed && o.line == want),
            "{path:?}: {got:?}, want line {want}"
        );
        removals += 1;
    }
    assert_eq!(removals, 2);
}
