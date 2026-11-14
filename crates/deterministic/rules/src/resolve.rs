//! The resolve pipeline, steps 1 to 3 of PLAN.md 4.6: a base tree, patch
//! layers merged in order (RFC 7396 merge patch), then inheritance, `null`
//! removal and set operations. Every value records the layers that set it
//! ([`Provenance`]), which `rules explain` prints.
//!
//! Inheritance (`inherits`, `^abstract` templates) and set operations
//! (`{"+": [...], "-": [...]}`) exist only in our own collections and their
//! set-like fields ([`COLLECTIONS`], [`SET_FIELDS`]). Imported INI data can
//! hold any bytes as names, including `^Foo`, `+` or `inherits`, so those
//! words mean nothing special anywhere else.

use std::collections::{BTreeMap, BTreeSet};

use crate::jsonc::{self, Member, Node, Span, Value};

/// A path into the tree: object member names and array positions.
pub type Path = Vec<String>;

/// Top-level collections whose members are instances that may inherit.
pub const COLLECTIONS: [&str; 4] = ["objects", "weapons", "warheads", "owners"]; // nocite: our format's shape

/// Set-like fields: (collection, field). Their values accept set
/// operations.
// nocite: our format's shape
pub const SET_FIELDS: [(&str, &str); 3] = [
    ("objects", "owners"),
    ("objects", "prerequisites"),
    ("objects", "immune_to"),
];

/// What happened to a value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// A layer set it.
    Set,
    /// A layer removed it (`null`).
    Removed,
    /// A layer changed a set with `+`/`-`.
    SetOperation,
    /// It came from a parent instance.
    Inherited(String),
    /// The converter derived it from imported data; the detail says how.
    Imported,
}

/// One entry of a value's history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Origin {
    /// The layer (`import`, `mod 1: name`, ...).
    pub layer: String,
    /// The file, with its archive or folder when imported.
    pub file: String,
    /// 1-based line, 0 when not known.
    pub line: u32,
    /// What happened.
    pub event: Event,
    /// The value as written, or a note.
    pub detail: String,
}

/// The history of every value, by path.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Provenance {
    entries: BTreeMap<Path, Vec<Origin>>,
}

impl Provenance {
    /// Appends `origin` to the history of `path`.
    pub fn record(&mut self, path: Path, origin: Origin) {
        self.entries.entry(path).or_default().push(origin);
    }

    /// The history of exactly `path`.
    pub fn get(&self, path: &[String]) -> &[Origin] {
        self.entries.get(path).map_or(&[], Vec::as_slice)
    }

    /// Every recorded path at or below `prefix`, in path order.
    pub fn below<'a>(
        &'a self,
        prefix: &'a [String],
    ) -> impl Iterator<Item = (&'a Path, &'a [Origin])> {
        self.entries
            .range(prefix.to_vec()..)
            .take_while(move |(p, _)| p.starts_with(prefix))
            .map(|(p, o)| (p, o.as_slice()))
    }

    /// Number of recorded paths.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether nothing is recorded.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn copy_below(&mut self, from: &[String], to: &[String], parent: &str) {
        let copied: Vec<(Path, Vec<Origin>)> = self
            .below(from)
            .map(|(p, o)| {
                let mut q = to.to_vec();
                q.extend_from_slice(&p[from.len()..]);
                let o = o
                    .iter()
                    .map(|x| Origin {
                        event: Event::Inherited(parent.to_owned()),
                        ..x.clone()
                    })
                    .collect();
                (q, o)
            })
            .collect();
        for (q, mut o) in copied {
            let own = self.entries.remove(&q).unwrap_or_default();
            o.extend(own);
            self.entries.insert(q, o);
        }
    }
}

/// One patch layer's text.
#[derive(Clone, Copy, Debug)]
pub struct Source<'a> {
    /// Layer label.
    pub layer: &'a str,
    /// File name.
    pub file: &'a str,
    /// JSONC text.
    pub text: &'a str,
}

/// A resolve error with its place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolveError {
    /// File.
    pub file: String,
    /// 1-based line (0 when not known).
    pub line: usize,
    /// 1-based column (0 when not known).
    pub column: usize,
    /// What went wrong.
    pub message: String,
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}: {}",
            self.file, self.line, self.column, self.message
        )
    }
}

impl std::error::Error for ResolveError {}

fn empty_object() -> Node {
    Node {
        value: Value::Object(Vec::new()),
        span: Span::default(),
    }
}

fn is_set_field(path: &[String]) -> bool {
    // nocite: collection, instance, field
    path.len() == 3
        && SET_FIELDS
            .iter()
            .any(|(c, f)| path[0] == *c && path[2] == *f)
}

/// A set operation: items to remove, then items to add.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct SetOp {
    add: Vec<String>,
    remove: Vec<String>,
}

impl SetOp {
    /// Reads `{"+": [...], "-": [...]}`; `None` if `node` is not one.
    fn read(node: &Node) -> Option<Self> {
        let members = node.members()?;
        let mut op = Self::default();
        for m in members {
            let list = match &m.value.value {
                Value::Array(items) => items
                    .iter()
                    .map(|i| i.as_str().map(str::to_owned))
                    .collect::<Option<Vec<_>>>()?,
                _ => return None,
            };
            match m.key.as_str() {
                "+" => op.add = list,
                "-" => op.remove = list,
                _ => return None,
            }
        }
        Some(op)
    }

    fn to_node(&self) -> Node {
        let list = |v: &[String]| Node {
            value: Value::Array(
                v.iter()
                    .map(|s| Node {
                        value: Value::String(s.clone()),
                        span: Span::default(),
                    })
                    .collect(),
            ),
            span: Span::default(),
        };
        Node {
            value: Value::Object(vec![
                Member {
                    key: "+".into(),
                    key_span: Span::default(),
                    value: list(&self.add),
                },
                Member {
                    key: "-".into(),
                    key_span: Span::default(),
                    value: list(&self.remove),
                },
            ]),
            span: Span::default(),
        }
    }

    /// This operation followed by `next`.
    fn then(&self, next: &SetOp) -> SetOp {
        let mut remove = self.remove.clone();
        for r in &next.remove {
            if !remove.contains(r) {
                remove.push(r.clone());
            }
        }
        let mut add: Vec<String> = self
            .add
            .iter()
            .filter(|a| !next.remove.contains(a))
            .cloned()
            .collect();
        for a in &next.add {
            if !add.contains(a) {
                add.push(a.clone());
            }
        }
        SetOp { add, remove }
    }

    /// Applies the operation to a list of strings, keeping order.
    fn apply(&self, base: &[String]) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for b in base {
            if !self.remove.contains(b) && !out.contains(b) {
                out.push(b.clone());
            }
        }
        for a in &self.add {
            if !out.contains(a) {
                out.push(a.clone());
            }
        }
        out
    }
}

fn string_list(node: &Node) -> Option<Vec<String>> {
    match &node.value {
        Value::Array(items) => items
            .iter()
            .map(|i| i.as_str().map(str::to_owned))
            .collect(),
        _ => None,
    }
}

fn list_node(items: &[String]) -> Node {
    Node {
        value: Value::Array(
            items
                .iter()
                .map(|s| Node {
                    value: Value::String(s.clone()),
                    span: Span::default(),
                })
                .collect(),
        ),
        span: Span::default(),
    }
}

/// Builds a resolved tree from a base and patch layers.
#[derive(Clone, Debug)]
pub struct Resolver {
    root: Node,
    provenance: Provenance,
}

impl Default for Resolver {
    fn default() -> Self {
        Self {
            root: empty_object(),
            provenance: Provenance::default(),
        }
    }
}

struct Ctx<'a> {
    src: &'a Source<'a>,
    provenance: &'a mut Provenance,
}

impl Ctx<'_> {
    fn origin(&self, span: Span, event: Event, detail: String) -> Origin {
        let (line, _) = jsonc::line_col(self.src.text, span.start);
        Origin {
            layer: self.src.layer.to_owned(),
            file: self.src.file.to_owned(),
            line: u32::try_from(line).unwrap_or(u32::MAX),
            event,
            detail,
        }
    }

    /// Records `Set` for every leaf of `node` at `path`.
    fn record_leaves(&mut self, node: &Node, path: &mut Path) {
        match &node.value {
            Value::Object(members) if !members.is_empty() => {
                for m in members {
                    path.push(m.key.clone());
                    self.record_leaves(&m.value, path);
                    path.pop();
                }
            }
            _ => {
                let o = self.origin(node.span, Event::Set, render_value(node));
                self.provenance.record(path.clone(), o);
            }
        }
    }
}

/// A short rendering of a leaf value for histories.
pub fn render_value(node: &Node) -> String {
    match &node.value {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Integer(i) => i.text().to_owned(),
        Value::String(s) => format!("{s:?}"),
        Value::Array(items) => format!(
            "[{}]",
            items
                .iter()
                .map(render_value)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Value::Object(members) => format!("{{{} members}}", members.len()),
    }
}

impl Resolver {
    /// Starts from an already built tree (the converter's output) and its
    /// history.
    pub fn with_base(base: Node, provenance: Provenance) -> Self {
        Self {
            root: base,
            provenance,
        }
    }

    /// The tree so far.
    pub fn tree(&self) -> &Node {
        &self.root
    }

    /// Parses `src` and merges it as the next layer.
    pub fn apply(&mut self, src: &Source) -> Result<(), ResolveError> {
        let node = jsonc::parse(src.text).map_err(|e| {
            let (line, column) = jsonc::line_col(src.text, e.span.start);
            ResolveError {
                file: src.file.to_owned(),
                line,
                column,
                message: e.to_string(),
            }
        })?;
        self.apply_node(&node, src)
    }

    /// Merges an already parsed layer.
    pub fn apply_node(&mut self, node: &Node, src: &Source) -> Result<(), ResolveError> {
        if !matches!(node.value, Value::Object(_)) {
            let (line, column) = jsonc::line_col(src.text, node.span.start);
            return Err(ResolveError {
                file: src.file.to_owned(),
                line,
                column,
                message: "a layer must be an object".into(),
            });
        }
        let mut ctx = Ctx {
            src,
            provenance: &mut self.provenance,
        };
        merge(&mut self.root, node, &mut Vec::new(), &mut ctx);
        Ok(())
    }

    /// Steps 3 of the pipeline: inheritance, set operations, abstract
    /// templates and `null` tombstones. Returns the resolved tree and the
    /// history, or every error found.
    pub fn finish(mut self) -> Result<(Node, Provenance), Vec<ResolveError>> {
        let mut errors = Vec::new();
        if let Value::Object(members) = &mut self.root.value {
            for m in members.iter_mut() {
                if COLLECTIONS.contains(&m.key.as_str()) {
                    inherit_collection(&m.key, &mut m.value, &mut self.provenance, &mut errors);
                }
            }
        }
        finalize(&mut self.root, &mut Vec::new());
        if errors.is_empty() {
            Ok((self.root, self.provenance))
        } else {
            Err(errors)
        }
    }
}

/// RFC 7396 merge of `patch` into `target`, keeping `null` as a tombstone
/// (removed in [`finalize`], after inheritance) and applying set operations
/// at set-like fields.
fn merge(target: &mut Node, patch: &Node, path: &mut Path, ctx: &mut Ctx) {
    if is_set_field(path)
        && let Some(op) = SetOp::read(patch)
    {
        let o = ctx.origin(patch.span, Event::SetOperation, render_value(patch));
        ctx.provenance.record(path.clone(), o);
        let merged = match (string_list(target), SetOp::read(target)) {
            (Some(list), _) => list_node(&op.apply(&list)),
            (None, Some(earlier)) => earlier.then(&op).to_node(),
            _ => op.to_node(),
        };
        *target = merged;
        return;
    }
    let (Value::Object(patch_members), Value::Object(_)) = (&patch.value, &target.value) else {
        // Anything but object-into-object replaces.
        *target = patch.clone();
        ctx.record_leaves(patch, path);
        return;
    };
    for pm in patch_members {
        path.push(pm.key.clone());
        let Value::Object(target_members) = &mut target.value else {
            break;
        };
        if matches!(pm.value.value, Value::Null) {
            let o = ctx.origin(pm.value.span, Event::Removed, "null".into());
            ctx.provenance.record(path.clone(), o);
        }
        match target_members.iter_mut().find(|m| m.key == pm.key) {
            Some(tm) if matches!(pm.value.value, Value::Null) => tm.value = pm.value.clone(),
            Some(tm) => merge(&mut tm.value, &pm.value, path, ctx),
            None => {
                let mut fresh = Member {
                    key: pm.key.clone(),
                    key_span: pm.key_span,
                    value: empty_object(),
                };
                if matches!(pm.value.value, Value::Object(_)) && !is_set_field(path) {
                    merge(&mut fresh.value, &pm.value, path, ctx);
                } else if is_set_field(path) && SetOp::read(&pm.value).is_some() {
                    fresh.value = Node {
                        value: Value::Null,
                        span: Span::default(),
                    };
                    merge(&mut fresh.value, &pm.value, path, ctx);
                } else {
                    fresh.value = pm.value.clone();
                    if !matches!(pm.value.value, Value::Null) {
                        ctx.record_leaves(&pm.value, path);
                    }
                }
                target_members.push(fresh);
            }
        }
        path.pop();
    }
}

/// Merges an instance's own data over its inherited base: own members win,
/// objects merge member by member, set operations apply to the base's
/// list, `null` removes.
fn overlay(base: &mut Node, own: &Node, path: &mut Path) {
    if is_set_field(path)
        && let Some(op) = SetOp::read(own)
    {
        let list = string_list(base).unwrap_or_default();
        *base = list_node(&op.apply(&list));
        return;
    }
    let (Value::Object(own_members), Value::Object(_)) = (&own.value, &base.value) else {
        *base = own.clone();
        return;
    };
    for om in own_members {
        path.push(om.key.clone());
        if let Value::Object(base_members) = &mut base.value {
            match base_members.iter_mut().position(|m| m.key == om.key) {
                Some(i) if matches!(om.value.value, Value::Null) => {
                    base_members.remove(i);
                }
                Some(i) => overlay(&mut base_members[i].value, &om.value, path),
                None if matches!(om.value.value, Value::Null) => {}
                None => base_members.push(om.clone()),
            }
        }
        path.pop();
    }
}

fn inherit_collection(
    collection: &str,
    node: &mut Node,
    provenance: &mut Provenance,
    errors: &mut Vec<ResolveError>,
) {
    let Value::Object(members) = &node.value else {
        return;
    };
    let ids: Vec<String> = members.iter().map(|m| m.key.clone()).collect();
    let mut done: BTreeMap<String, Node> = BTreeMap::new();
    for id in &ids {
        let mut visiting = Vec::new();
        resolve_instance(
            collection,
            id,
            node,
            &mut done,
            &mut visiting,
            provenance,
            errors,
        );
    }
    // Abstract templates exist only to be inherited from.
    let resolved: Vec<Member> = ids
        .iter()
        .filter(|id| !id.starts_with('^'))
        .filter_map(|id| {
            Some(Member {
                key: id.clone(),
                key_span: Span::default(),
                value: done.get(id)?.clone(),
            })
        })
        .collect();
    node.value = Value::Object(resolved);
}

fn error(message: String) -> ResolveError {
    ResolveError {
        file: String::new(),
        line: 0,
        column: 0,
        message,
    }
}

fn resolve_instance(
    collection: &str,
    id: &str,
    node: &Node,
    done: &mut BTreeMap<String, Node>,
    visiting: &mut Vec<String>,
    provenance: &mut Provenance,
    errors: &mut Vec<ResolveError>,
) -> Option<Node> {
    if let Some(n) = done.get(id) {
        return Some(n.clone());
    }
    if visiting.iter().any(|v| v == id) {
        errors.push(error(format!(
            "{collection}: inheritance cycle {} -> {id}",
            visiting.join(" -> ")
        )));
        return None;
    }
    let own = node.get(id)?.clone();
    visiting.push(id.to_owned());
    // nocite: our own rules format (PLAN.md 4.6: `null` removes a key; a removed `inherits` names no parents)
    let parents = match own.get("inherits") {
        None => Vec::new(),
        Some(p) if matches!(p.value, Value::Null) => Vec::new(),
        Some(p) => string_list(p).unwrap_or_else(|| {
            errors.push(error(format!(
                "{collection}.{id}: inherits must be a list of names"
            )));
            Vec::new()
        }),
    };
    let mut base = empty_object();
    for parent in &parents {
        if node.get(parent).is_none() {
            errors.push(error(format!(
                "{collection}.{id}: unknown parent {parent:?}"
            )));
            continue;
        }
        if let Some(p) =
            resolve_instance(collection, parent, node, done, visiting, provenance, errors)
        {
            let mut path = vec![collection.to_owned(), id.to_owned()];
            overlay(&mut base, &p, &mut path);
            provenance.copy_below(
                &[collection.to_owned(), parent.clone()],
                &[collection.to_owned(), id.to_owned()],
                parent,
            );
        }
    }
    let mut own = own;
    if let Value::Object(m) = &mut own.value {
        m.retain(|x| x.key != "inherits");
    }
    let mut path = vec![collection.to_owned(), id.to_owned()];
    overlay(&mut base, &own, &mut path);
    visiting.pop();
    done.insert(id.to_owned(), base.clone());
    Some(base)
}

/// Removes `null` tombstones and turns leftover set operations into lists.
fn finalize(node: &mut Node, path: &mut Path) {
    if is_set_field(path)
        && let Some(op) = SetOp::read(node)
    {
        *node = list_node(&op.apply(&[]));
        return;
    }
    match &mut node.value {
        Value::Object(members) => {
            members.retain(|m| !matches!(m.value.value, Value::Null));
            for m in members.iter_mut() {
                path.push(m.key.clone());
                finalize(&mut m.value, path);
                path.pop();
            }
        }
        Value::Array(items) => {
            for (i, item) in items.iter_mut().enumerate() {
                path.push(i.to_string());
                finalize(item, path);
                path.pop();
            }
        }
        _ => {}
    }
}

/// Keys of `node` if it is an object.
pub fn keys(node: &Node) -> BTreeSet<String> {
    node.members()
        .map(|m| m.iter().map(|x| x.key.clone()).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolve(layers: &[&str]) -> (Node, Provenance) {
        let mut r = Resolver::default();
        for (i, text) in layers.iter().enumerate() {
            let file = format!("layer{i}.jsonc");
            r.apply(&Source {
                layer: "test",
                file: &file,
                text,
            })
            .unwrap();
        }
        r.finish().unwrap()
    }

    fn at<'a>(n: &'a Node, path: &[&str]) -> Option<&'a Node> {
        path.iter().try_fold(n, |n, k| n.get(k))
    }

    #[test]
    fn merge_patch_and_null() {
        let (t, p) = resolve(&[
            r#"{"globals": {"economy": {"a": 1, "b": 2}}, "x": [1, 2]}"#,
            "{\"globals\": {\"economy\": {\"a\": 5, \"b\": null}}, \"x\": [3]}",
        ]);
        assert_eq!(
            render_value(at(&t, &["globals", "economy", "a"]).unwrap()),
            "5"
        );
        assert!(at(&t, &["globals", "economy", "b"]).is_none());
        assert_eq!(render_value(at(&t, &["x"]).unwrap()), "[3]");
        let h = p.get(&["globals".into(), "economy".into(), "a".into()]);
        assert_eq!(h.len(), 2);
        assert_eq!((h[0].line, h[1].file.as_str()), (1, "layer1.jsonc"));
    }

    #[test]
    fn inheritance_sets_and_abstract_templates() {
        let (t, p) = resolve(&[
            r#"{"objects": {
                "^base": {"speed": 9, "owners": ["a"], "armament": {"primary": {"weapon": "w"}}},
                "child": {"inherits": ["^base"], "cost": 3, "owners": {"+": ["b"]}},
                "grand": {"inherits": ["child"], "armament": null, "owners": {"-": ["a"]}}
            }}"#,
            r#"{"objects": {"child": {"owners": {"+": ["c"]}}}}"#,
        ]);
        assert!(at(&t, &["objects", "^base"]).is_none());
        assert_eq!(
            render_value(at(&t, &["objects", "child", "speed"]).unwrap()),
            "9"
        );
        assert_eq!(
            render_value(at(&t, &["objects", "child", "owners"]).unwrap()),
            r#"["a", "b", "c"]"#
        );
        assert_eq!(
            render_value(at(&t, &["objects", "grand", "owners"]).unwrap()),
            r#"["b", "c"]"#
        );
        assert!(at(&t, &["objects", "grand", "armament"]).is_none());
        assert!(at(&t, &["objects", "child", "inherits"]).is_none());
        let h = p.get(&["objects".into(), "child".into(), "speed".into()]);
        assert_eq!(h[0].event, Event::Inherited("^base".into()));
    }

    #[test]
    fn cycles_and_unknown_parents_are_errors() {
        let mut r = Resolver::default();
        r.apply(&Source {
            layer: "t",
            file: "f",
            text: r#"{"objects": {"a": {"inherits": ["b"]}, "b": {"inherits": ["a"]}, "c": {"inherits": ["zz"]}}}"#,
        })
        .unwrap();
        let errors = r.finish().unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("cycle")));
        assert!(errors.iter().any(|e| e.message.contains("unknown parent")));
    }

    /// Imported names are bytes: `^`, `+`, `-` and `inherits` mean nothing there.
    #[test]
    fn imported_data_is_left_alone() {
        let (t, _) = resolve(&[
            r#"{"types": {"buildings": {"^Odd": {"ini": {"+": "1", "-": "2", "inherits": "x"}}}},
                "ini_globals": {"^Sec": {"inherits": "y"}}}"#,
            r#"{"types": {"buildings": {"^Odd": {"ini": {"+": "3"}}}}}"#,
        ]);
        let ini = at(&t, &["types", "buildings", "^Odd", "ini"]).unwrap();
        assert_eq!(render_value(ini.get("+").unwrap()), "\"3\"");
        assert_eq!(render_value(ini.get("-").unwrap()), "\"2\"");
        assert_eq!(render_value(ini.get("inherits").unwrap()), "\"x\"");
        assert!(at(&t, &["ini_globals", "^Sec", "inherits"]).is_some());
    }

    #[test]
    fn set_operation_composition() {
        let a = SetOp {
            add: vec!["x".into()],
            remove: vec!["y".into()],
        };
        let b = SetOp {
            add: vec!["y".into()],
            remove: vec!["x".into()],
        };
        let ab = a.then(&b);
        let base = vec!["x".into(), "y".into(), "z".into()];
        assert_eq!(ab.apply(&base), b.apply(&a.apply(&base)));
    }
}
