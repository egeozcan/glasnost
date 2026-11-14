//! The synthetic ruleset in `data/synthetic/ruleset/` through the whole
//! resolve pipeline: every file parses as JSONC (so every number is an
//! integer) and carries the synthetic-data header; the layers resolve,
//! type-check against schema 1 and validate; and the committed JSON Schema
//! is current.

// Tests read fixture files from disk; they are not simulation code.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::path::PathBuf;

use glasnost_rules::jsonc::{self, Node, Value};
use glasnost_rules::resolve::Source;
use glasnost_rules::schema::{Manifest, ObjectKind};

fn ruleset_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data/synthetic/ruleset")
}

fn load(name: &str) -> (String, Node) {
    let path = ruleset_dir().join(name);
    let src = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let node = jsonc::parse(&src).unwrap_or_else(|e| {
        let (line, col) = jsonc::line_col(&src, e.span.start);
        panic!("{name}:{line}:{col}: {e}")
    });
    (src, node)
}

fn strings(node: &Node) -> Vec<String> {
    match &node.value {
        Value::Array(items) => items
            .iter()
            .map(|i| i.as_str().expect("string expected").to_owned())
            .collect(),
        // Set operation: {"+": [...], "-": [...]}
        Value::Object(_) => ["+", "-"]
            .iter()
            .filter_map(|op| node.get(op))
            .flat_map(strings)
            .collect(),
        _ => panic!("list or set operation expected"),
    }
}

fn layers() -> Vec<String> {
    let (_, manifest) = load("manifest.jsonc");
    strings(manifest.get("layers").expect("manifest lists layers"))
}

#[test]
fn every_file_parses_and_is_marked_synthetic() {
    let mut on_disk: Vec<String> = std::fs::read_dir(ruleset_dir())
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    on_disk.sort();
    let mut listed = layers();
    listed.push("manifest.jsonc".into());
    listed.sort();
    assert_eq!(on_disk, listed, "every file is listed in manifest.jsonc");

    for name in &listed {
        let (src, _) = load(name);
        let head: Vec<&str> = src.lines().take(2).collect();
        assert_eq!(
            head.first(),
            Some(&"// SPDX-License-Identifier: CC0-1.0"),
            "{name}"
        );
        assert!(
            head.get(1)
                .is_some_and(|l| l.starts_with("// glasnost-synthetic:")),
            "{name}: second line must be the glasnost-synthetic marker"
        );
    }
}

fn resolved() -> glasnost_rules::Resolved {
    let texts: Vec<(String, String)> = layers()
        .into_iter()
        .map(|name| {
            let (src, _) = load(&name);
            (name, src)
        })
        .collect();
    let sources: Vec<Source> = texts
        .iter()
        .map(|(name, text)| Source {
            layer: name,
            file: name,
            text,
        })
        .collect();
    let (_, manifest) = load("manifest.jsonc");
    let manifest: Manifest =
        glasnost_rules::de::from_node(&manifest).expect("manifest matches the schema");
    let schema = jsonc::object([(
        "schema".to_owned(),
        jsonc::integer(i128::from(manifest.schema)),
    )]);
    let base = (schema, Default::default());
    glasnost_rules::resolve_ruleset(Some(base), &sources).unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn the_pipeline_resolves_and_validates_it() {
    let r = resolved();
    let objects = &r.ruleset.objects;
    // Abstract templates are gone; inherited fields arrived.
    assert!(!objects.keys().any(|k| k.starts_with('^')));
    let rover = &objects["syn_crack_rover_mk2"];
    assert_eq!(rover.kind, Some(ObjectKind::Vehicle));
    assert_eq!(rover.cost, Some(1029));
    assert_eq!(rover.speed, Some(13));
    // A set operation on an inherited set.
    assert_eq!(rover.owners, ["basalt", "heliotrope"]);
    assert_eq!(rover.armament.len(), 2);
    // IDs in first-appearance order; the hash is stable across runs.
    assert_eq!(r.ids.get_own("objects", "syn_depot"), Some(0));
    assert_eq!(r.sim_rules_hash, resolved().sim_rules_hash);
    let path = glasnost_rules::explain::find_path(&r.tree, "syn_crack_rover_mk2.speed").unwrap();
    let text = glasnost_rules::explain::explain(&r.tree, &r.provenance, &path);
    assert!(text.contains("inherited from syn_crack_rover"), "{text}");
}

#[test]
fn the_committed_json_schema_is_current() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data/schema/ruleset.schema.json");
    let generated = glasnost_rules::schema::json_schema();
    if std::env::var_os("GLASNOST_UPDATE_SCHEMA").is_some() {
        std::fs::write(&path, &generated).unwrap();
    }
    let committed = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        committed == generated,
        "data/schema/ruleset.schema.json is stale; rerun with GLASNOST_UPDATE_SCHEMA=1"
    );
}

#[test]
fn decimals_are_quoted_with_a_cell_unit() {
    fn walk(node: &Node, key: &str, file: &str) {
        match &node.value {
            Value::Object(members) => members.iter().for_each(|m| walk(&m.value, &m.key, file)),
            Value::Array(items) => items.iter().for_each(|i| walk(i, key, file)),
            Value::String(s) if key == "range" || key == "sight" => {
                let digits = s
                    .strip_suffix('c')
                    .unwrap_or_else(|| panic!("{file}: {key} {s:?} needs a unit"));
                let ok = digits.split_once('.').map_or_else(
                    || !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()),
                    |(i, f)| {
                        !i.is_empty()
                            && !f.is_empty()
                            && i.bytes().chain(f.bytes()).all(|b| b.is_ascii_digit())
                    },
                );
                assert!(ok, "{file}: {key} {s:?} is not a plain decimal");
            }
            _ => {}
        }
    }
    for name in layers() {
        let (_, node) = load(&name);
        walk(&node, "", &name);
    }
}
