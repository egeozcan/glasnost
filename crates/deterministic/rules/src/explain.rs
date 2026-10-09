//! `rules explain`: the history of a value (PLAN.md 4.6).

use std::fmt::Write as _;

use crate::jsonc::{Node, Value};
use crate::resolve::{COLLECTIONS, Event, Origin, Path, Provenance, render_value};

/// Turns a dotted query into a path of the tree. Names may contain dots,
/// so at each level the longest member name that matches wins. A query
/// that starts with an instance or type name (`HTNK.ini.Cost`) is looked up
/// in our collections and in every imported kind.
pub fn find_path(tree: &Node, query: &str) -> Option<Path> {
    if let Some(p) = walk(tree, query) {
        return Some(p);
    }
    let mut prefixes: Vec<Path> = COLLECTIONS.iter().map(|c| vec![(*c).to_owned()]).collect();
    if let Some(kinds) = tree.get("types").and_then(Node::members) {
        prefixes.extend(
            kinds
                .iter()
                .map(|k| vec!["types".to_owned(), k.key.clone()]),
        );
    }
    prefixes.into_iter().find_map(|prefix| {
        let base = prefix.iter().try_fold(tree, |n, k| n.get(k))?;
        let mut p = walk(base, query)?;
        let mut full = prefix;
        full.append(&mut p);
        Some(full)
    })
}

fn walk(node: &Node, query: &str) -> Option<Path> {
    if query.is_empty() {
        return Some(Vec::new());
    }
    let members = node.members()?;
    // Longest match first.
    let mut candidates: Vec<&crate::jsonc::Member> = members
        .iter()
        .filter(|m| {
            query == m.key
                || query
                    .strip_prefix(m.key.as_str())
                    .is_some_and(|rest| rest.starts_with('.'))
        })
        .collect();
    candidates.sort_by_key(|m| std::cmp::Reverse(m.key.len()));
    candidates.into_iter().find_map(|m| {
        let rest = query[m.key.len()..].strip_prefix('.').unwrap_or("");
        let mut p = walk(&m.value, rest)?;
        p.insert(0, m.key.clone());
        Some(p)
    })
}

fn describe(o: &Origin) -> String {
    let what = match &o.event {
        Event::Set => format!("set {}", o.detail),
        Event::Removed => "removed".to_owned(),
        Event::SetOperation => format!("set operation {}", o.detail),
        Event::Inherited(from) => format!("inherited from {from}: {}", o.detail),
        Event::Imported => o.detail.clone(),
    };
    let line = if o.line > 0 {
        format!(" line {}", o.line)
    } else {
        String::new()
    };
    format!("{} | {}{line} | {what}", o.layer, o.file)
}

/// The value at `path` and the history of it and of every value below it.
pub fn explain(tree: &Node, provenance: &Provenance, path: &[String]) -> String {
    let mut out = String::new();
    let value = path.iter().try_fold(tree, |n, k| n.get(k));
    let shown = value.map_or_else(|| "(absent)".to_owned(), render_value);
    let _ = writeln!(out, "{} = {shown}", path.join("."));
    let mut any = false;
    for (p, origins) in provenance.below(path) {
        any = true;
        if p.len() > path.len() {
            let _ = writeln!(out, "  {}", p[path.len()..].join("."));
        }
        for o in origins {
            let _ = writeln!(out, "    {}", describe(o));
        }
    }
    if !any {
        let _ = writeln!(out, "  (no recorded history)");
    }
    if let Some(Node {
        value: Value::Object(m),
        ..
    }) = value
        && m.is_empty()
    {
        let _ = writeln!(out, "  (empty)");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolve::{Resolver, Source};

    #[test]
    fn explains_a_patched_value() {
        let mut r = Resolver::default();
        for (file, text) in [
            (
                "base.jsonc",
                "{\"types\": {\"vehicles\": {\"A.B\": {\"ini\": {\"Cost\": \"5\"}}}}}",
            ),
            (
                "mod.jsonc",
                "{\n\"types\": {\"vehicles\": {\"A.B\": {\"ini\": {\"Cost\": \"7\"}}}}}",
            ),
        ] {
            r.apply(&Source {
                layer: file,
                file,
                text,
            })
            .unwrap();
        }
        let (tree, prov) = r.finish().unwrap();
        let path = find_path(&tree, "A.B.ini.Cost").unwrap();
        assert_eq!(path, ["types", "vehicles", "A.B", "ini", "Cost"]);
        let text = explain(&tree, &prov, &path);
        assert!(text.contains("= \"7\""), "{text}");
        assert!(
            text.contains("base.jsonc | base.jsonc line 1 | set \"5\""),
            "{text}"
        );
        assert!(text.contains("mod.jsonc line 2 | set \"7\""), "{text}");
        assert!(find_path(&tree, "Nope.ini").is_none());
    }
}
