//! Shape checks for the synthetic ruleset in `data/synthetic/ruleset/`.
//!
//! These stand in for the schema and semantic validation that arrive in M1:
//! every file parses as JSONC (so every number is an integer), carries the
//! synthetic-data header, and every cross-reference resolves.

// Tests read fixture files from disk; they are not simulation code.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use glasnost_rules::jsonc::{self, Node, Value};

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

fn keys(node: &Node) -> BTreeSet<String> {
    node.members()
        .expect("object expected")
        .iter()
        .map(|m| m.key.clone())
        .collect()
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

#[test]
fn references_resolve() {
    let (_, combat) = load("combat.jsonc");
    let (_, objects_file) = load("objects.jsonc");
    let armor = keys(combat.get("armor_classes").unwrap());
    let warheads = combat.get("warheads").unwrap();
    let weapons = combat.get("weapons").unwrap();
    let owners = keys(objects_file.get("owners").unwrap());
    let objects = objects_file.get("objects").unwrap();
    let object_ids = keys(objects);

    for wh in warheads.members().unwrap() {
        let classes = keys(wh.value.get("versus").unwrap());
        assert!(classes.is_subset(&armor), "{}: unknown armor class", wh.key);
    }
    let warhead_ids = keys(warheads);
    for w in weapons.members().unwrap() {
        let wh = w.value.get("warhead").unwrap().as_str().unwrap();
        assert!(warhead_ids.contains(wh), "{}: unknown warhead {wh}", w.key);
    }

    let weapon_ids = keys(weapons);
    for o in objects.members().unwrap() {
        let id = &o.key;
        if let Some(a) = o.value.get("armor") {
            assert!(armor.contains(a.as_str().unwrap()), "{id}: unknown armor");
        }
        for parent in o.value.get("inherits").map(strings).unwrap_or_default() {
            assert!(
                object_ids.contains(&parent),
                "{id}: unknown parent {parent}"
            );
        }
        for owner in o.value.get("owners").map(strings).unwrap_or_default() {
            assert!(owners.contains(&owner), "{id}: unknown owner {owner}");
        }
        for pre in o
            .value
            .get("prerequisites")
            .map(strings)
            .unwrap_or_default()
        {
            assert!(
                object_ids.contains(&pre),
                "{id}: unknown prerequisite {pre}"
            );
            assert!(
                !pre.starts_with('^'),
                "{id}: abstract template as prerequisite"
            );
        }
        if let Some(arm) = o.value.get("armament") {
            for slot in arm.members().unwrap() {
                let w = slot.value.get("weapon").unwrap().as_str().unwrap();
                assert!(weapon_ids.contains(w), "{id}: unknown weapon {w}");
            }
        }
    }
}

#[test]
fn inheritance_has_no_cycles() {
    let (_, objects_file) = load("objects.jsonc");
    let objects = objects_file.get("objects").unwrap();
    let parents = |id: &str| {
        objects
            .get(id)
            .and_then(|o| o.get("inherits"))
            .map(strings)
            .unwrap_or_default()
    };
    for o in objects.members().unwrap() {
        let mut stack = vec![(o.key.clone(), vec![o.key.clone()])];
        while let Some((id, path)) = stack.pop() {
            for p in parents(&id) {
                assert!(!path.contains(&p), "inheritance cycle: {path:?} -> {p}");
                let mut next = path.clone();
                next.push(p.clone());
                stack.push((p, next));
            }
        }
    }
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
