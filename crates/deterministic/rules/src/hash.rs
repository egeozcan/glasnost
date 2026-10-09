//! Canonical binary form and the two rules hashes (PLAN.md 4.6, step 7).
//!
//! The resolved tree is encoded canonically: integers by value (not by
//! token text), strings as UTF-8 bytes, objects with their members sorted
//! by key bytes, every item tagged and length-prefixed. `sim_rules_hash`
//! covers everything except the render-only paths of [`is_render_path`];
//! `render_rules_hash` covers only those, and is informational.

use sha2::{Digest, Sha256};

use crate::jsonc::{Node, Value};

/// Paths that only affect drawing: import provenance, colour definitions
/// and tints, and the art file names the converter looked up. Raw art
/// sections stay in the sim part: a field of unclear use counts as sim
/// (PLAN.md 4.6).
pub fn is_render_path(path: &[&str]) -> bool {
    matches!(
        path,
        ["import" | "colours" | "colour_add", ..] | ["types", _, _, "art", ..]
    )
}

// Tags of the canonical form.
const TAG_NULL: u8 = 0; // nocite: encoding tag of our own
const TAG_BOOL: u8 = 1; // nocite: encoding tag of our own
const TAG_INT: u8 = 2; // nocite: encoding tag of our own
const TAG_STRING: u8 = 3; // nocite: encoding tag of our own
const TAG_ARRAY: u8 = 4; // nocite: encoding tag of our own
const TAG_OBJECT: u8 = 5; // nocite: encoding tag of our own

fn int_value(text: &str) -> i128 {
    let (neg, digits) = match text.strip_prefix('-') {
        Some(d) => (true, d),
        None => (false, text),
    };
    let mut v: i128 = 0;
    for d in digits.bytes() {
        // nocite: decimal radix of the integer token
        v = v.saturating_mul(10).saturating_add(i128::from(d - b'0'));
    }
    if neg { -v } else { v }
}

fn put_len(out: &mut Vec<u8>, n: usize) {
    out.extend_from_slice(&(n as u64).to_be_bytes());
}

/// Appends the canonical form of `node` at `path` to `out`, keeping only
/// the parts for which `keep(path)` holds (objects and arrays on the way
/// to a kept part are kept).
fn encode(
    node: &Node,
    path: &mut Vec<String>,
    keep: &dyn Fn(&[&str]) -> Option<bool>,
    out: &mut Vec<u8>,
) {
    match &node.value {
        Value::Null => out.push(TAG_NULL),
        Value::Bool(b) => {
            out.push(TAG_BOOL);
            out.push(u8::from(*b));
        }
        Value::Integer(i) => {
            out.push(TAG_INT);
            out.extend_from_slice(&int_value(i.text()).to_be_bytes());
        }
        Value::String(s) => {
            out.push(TAG_STRING);
            put_len(out, s.len());
            out.extend_from_slice(s.as_bytes());
        }
        Value::Array(items) => {
            out.push(TAG_ARRAY);
            put_len(out, items.len());
            for (i, item) in items.iter().enumerate() {
                path.push(i.to_string());
                encode(item, path, keep, out);
                path.pop();
            }
        }
        Value::Object(members) => {
            let mut kept: Vec<&crate::jsonc::Member> = members
                .iter()
                .filter(|m| {
                    path.push(m.key.clone());
                    let p: Vec<&str> = path.iter().map(String::as_str).collect();
                    let k = keep(&p) != Some(false);
                    path.pop();
                    k
                })
                .collect();
            kept.sort_by(|a, b| a.key.as_bytes().cmp(b.key.as_bytes()));
            out.push(TAG_OBJECT);
            put_len(out, kept.len());
            for m in kept {
                out.push(TAG_STRING);
                put_len(out, m.key.len());
                out.extend_from_slice(m.key.as_bytes());
                path.push(m.key.clone());
                encode(&m.value, path, keep, out);
                path.pop();
            }
        }
    }
}

/// Whether `path` lies on the way to a render path (a prefix of one).
fn leads_to_render(path: &[&str]) -> bool {
    match path {
        [] | ["types"] | ["types", _] | ["types", _, _] => true,
        _ => is_render_path(path),
    }
}

/// The canonical form of the sim part of a resolved tree.
pub fn sim_bytes(tree: &Node) -> Vec<u8> {
    let mut out = Vec::new();
    let keep = |p: &[&str]| Some(!is_render_path(p));
    encode(tree, &mut Vec::new(), &keep, &mut out);
    out
}

/// The canonical form of the render part of a resolved tree.
pub fn render_bytes(tree: &Node) -> Vec<u8> {
    let mut out = Vec::new();
    let keep = |p: &[&str]| Some(leads_to_render(p));
    encode(tree, &mut Vec::new(), &keep, &mut out);
    out
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// `sim_rules_hash`: SHA-256 of [`sim_bytes`], hexadecimal.
pub fn sim_rules_hash(tree: &Node) -> String {
    hex(&Sha256::digest(sim_bytes(tree)))
}

/// `render_rules_hash`: SHA-256 of [`render_bytes`], hexadecimal.
pub fn render_rules_hash(tree: &Node) -> String {
    hex(&Sha256::digest(render_bytes(tree)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jsonc::parse;

    #[test]
    fn integers_by_value_and_keys_sorted() {
        let a = parse(r#"{"b": 7, "a": -0}"#).unwrap();
        let b = parse(r#"{"a": 0, "b": 7}"#).unwrap();
        assert_eq!(sim_rules_hash(&a), sim_rules_hash(&b));
        let c = parse(r#"{"a": 0, "b": 8}"#).unwrap();
        assert_ne!(sim_rules_hash(&a), sim_rules_hash(&c));
    }

    #[test]
    fn render_paths_are_split_off() {
        let a = parse(r#"{"types": {"vehicles": {"V": {"ini": {"X": "1"}, "art": {"image": "A"}}}}, "colours": []}"#).unwrap();
        let b = parse(r#"{"types": {"vehicles": {"V": {"ini": {"X": "1"}, "art": {"image": "B"}}}}, "colours": [1]}"#).unwrap();
        assert_eq!(sim_rules_hash(&a), sim_rules_hash(&b));
        assert_ne!(render_rules_hash(&a), render_rules_hash(&b));
        let c = parse(r#"{"types": {"vehicles": {"V": {"ini": {"X": "2"}, "art": {"image": "A"}}}}, "colours": []}"#).unwrap();
        assert_ne!(sim_rules_hash(&a), sim_rules_hash(&c));
        assert_eq!(render_rules_hash(&a), render_rules_hash(&c));
    }
}
