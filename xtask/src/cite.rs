//! Citation lint and determinism source scan for the simulation crates.
//!
//! **Citations** (CLAUDE.md, PROVENANCE.md section 6). In the non-test code of
//! every crate under `crates/deterministic/`:
//! - every numeric literal other than `0` and `1`, and
//! - every `const` and `static` item (numbers, quirk switches, tables)
//!
//! must be covered by a `// spec: S-xxxx` or `// measure: M-xxxx` comment,
//! either on the same line or in the comment and attribute block directly
//! above (or opening) an enclosing item, statement, field, variant or match
//! arm. A `// nocite: <reason>` comment in the same places marks a literal
//! that is not behaviour (a container size of our own design, say).
//! Whole files can be exempted in `xtask/citation-allowlist.txt`, with a
//! reason.
//!
//! **IDs exist.** Every `S-`, `M-` and `Q-` ID mentioned in `crates/**/*.rs`
//! and `data/**/*.jsonc` must name a file in `spec/`, `measurements/` or
//! `spec/requests/`; a cited spec must not be retired.
//!
//! **Floats** (determinism scan). Float literals are rejected in simulation
//! code; clippy's `disallowed-types` catches float types, but not an
//! inferred `let x = 1.5;`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use proc_macro2::{TokenStream, TokenTree};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};

use crate::workspace::{self, rel};

/// Allowlist of whole files, relative to the repository root.
pub const ALLOWLIST_FILE: &str = "xtask/citation-allowlist.txt";

/// Which rule a finding or allowlist entry concerns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rule {
    /// Missing `spec:`/`measure:` citation.
    Cite,
    /// Float literal.
    Float,
}

/// One problem in a source file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    /// 1-based line.
    pub line: usize,
    /// Rule broken.
    pub rule: Rule,
    /// Explanation.
    pub message: String,
}

// ---------------------------------------------------------------------------
// Comment markers

fn comment_text(line: &str) -> Option<&str> {
    let t = line.trim_start();
    if t.starts_with("/*") || t.starts_with('*') {
        return Some(t);
    }
    line.find("//").map(|i| &line[i..])
}

fn is_comment_or_attr(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("//") || t.starts_with("/*") || t.starts_with('*') || t.starts_with("#[")
}

/// Every ID token (`S-1234`, `M-1234`, `Q-1234`) in `text`.
pub fn id_tokens(text: &str) -> Vec<String> {
    let b = text.as_bytes();
    let mut out = Vec::new();
    for i in 0..b.len() {
        let starts = matches!(b[i], b'S' | b'M' | b'Q')
            && b.get(i + 1) == Some(&b'-')
            && (i == 0 || !b[i - 1].is_ascii_alphanumeric())
            && b.len() >= i + 6
            && b[i + 2..i + 6].iter().all(u8::is_ascii_digit)
            && !b.get(i + 6).is_some_and(u8::is_ascii_digit);
        if starts {
            out.push(text[i..i + 6].to_owned());
        }
    }
    out
}

/// Whether `comment` holds a citation (`spec: S-xxxx` / `measure: M-xxxx`).
fn has_citation(comment: &str) -> bool {
    [("spec:", 'S'), ("measure:", 'M')]
        .iter()
        .any(|(tag, letter)| {
            comment.match_indices(tag).any(|(i, _)| {
                let rest = comment[i + tag.len()..].trim_start();
                id_tokens(rest)
                    .first()
                    .is_some_and(|id| rest.starts_with(id.as_str()) && id.starts_with(*letter))
            })
        })
}

/// Whether `comment` holds `nocite:` with a reason.
fn has_nocite(comment: &str) -> bool {
    comment
        .match_indices("nocite:")
        .any(|(i, _)| !comment[i + "nocite:".len()..].trim().is_empty())
}

// ---------------------------------------------------------------------------
// Source scan

struct Scan<'a> {
    lines: Vec<&'a str>,
    /// Start lines of the enclosing anchors, outermost first.
    anchors: Vec<usize>,
    findings: Vec<Finding>,
}

impl Scan<'_> {
    fn line(&self, n: usize) -> &str {
        self.lines.get(n.wrapping_sub(1)).copied().unwrap_or("")
    }

    fn marked(&self, n: usize) -> bool {
        comment_text(self.line(n)).is_some_and(|c| has_citation(c) || has_nocite(c))
    }

    /// Marker in the comment/attribute block above `start`, or in the
    /// comments, attributes and first code line from `start` down.
    fn anchor_marked(&self, start: usize) -> bool {
        let mut n = start;
        while n > 1 && is_comment_or_attr(self.line(n - 1)) {
            n -= 1;
            if self.marked(n) {
                return true;
            }
        }
        let mut n = start;
        while n <= self.lines.len() {
            if self.marked(n) {
                return true;
            }
            if !is_comment_or_attr(self.line(n)) {
                break;
            }
            n += 1;
        }
        false
    }

    fn covered(&self, line: usize) -> bool {
        self.marked(line) || self.anchors.iter().rev().any(|&a| self.anchor_marked(a))
    }

    fn number(&mut self, line: usize, text: &str) {
        match classify_number(text) {
            Some(Number::Float) => self.findings.push(Finding {
                line,
                rule: Rule::Float,
                message: format!("float literal `{text}` in simulation code"),
            }),
            Some(Number::Int(v)) if v > 1 && !self.covered(line) => {
                self.findings.push(Finding {
                    line,
                    rule: Rule::Cite,
                    message: format!(
                        "numeric literal `{text}` has no `// spec: S-xxxx` or `// measure: M-xxxx` citation"
                    ),
                });
            }
            _ => {}
        }
    }

    fn constant(&mut self, line: usize, what: &str) {
        if !self.covered(line) {
            self.findings.push(Finding {
                line,
                rule: Rule::Cite,
                message: format!(
                    "{what} has no `// spec: S-xxxx` or `// measure: M-xxxx` citation"
                ),
            });
        }
    }

    fn tokens(&mut self, tokens: TokenStream) {
        for tt in tokens {
            match tt {
                TokenTree::Group(g) => self.tokens(g.stream()),
                TokenTree::Literal(l) => {
                    let text = l.to_string();
                    self.number(l.span().start().line, &text);
                }
                _ => {}
            }
        }
    }

    fn with_anchor(&mut self, start: usize, f: impl FnOnce(&mut Self)) {
        self.anchors.push(start);
        f(self);
        self.anchors.pop();
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Number {
    Int(u128),
    Float,
}

/// Classifies a literal token; `None` if it is not a number.
fn classify_number(text: &str) -> Option<Number> {
    if !text.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    let t = text.replace('_', "");
    for (prefix, radix) in [("0x", 16), ("0o", 8), ("0b", 2)] {
        if let Some(body) = t.strip_prefix(prefix) {
            let digits = body.split(['i', 'u']).next().unwrap_or("");
            return Some(Number::Int(
                u128::from_str_radix(digits, radix).unwrap_or(u128::MAX),
            ));
        }
    }
    if t.contains(['.', 'e', 'E']) || t.ends_with("f32") || t.ends_with("f64") {
        return Some(Number::Float);
    }
    let digits: String = t.chars().take_while(char::is_ascii_digit).collect();
    Some(Number::Int(digits.parse().unwrap_or(u128::MAX)))
}

fn is_test_attr(attr: &syn::Attribute) -> bool {
    if attr.path().is_ident("test") {
        return true;
    }
    attr.path().is_ident("cfg")
        && matches!(&attr.meta, syn::Meta::List(l) if l.tokens.clone().into_iter().any(
            |t| matches!(t, TokenTree::Ident(i) if i == "test")
        ))
}

fn item_attrs(item: &syn::Item) -> &[syn::Attribute] {
    use syn::Item as I;
    match item {
        I::Const(i) => &i.attrs,
        I::Enum(i) => &i.attrs,
        I::ExternCrate(i) => &i.attrs,
        I::Fn(i) => &i.attrs,
        I::ForeignMod(i) => &i.attrs,
        I::Impl(i) => &i.attrs,
        I::Macro(i) => &i.attrs,
        I::Mod(i) => &i.attrs,
        I::Static(i) => &i.attrs,
        I::Struct(i) => &i.attrs,
        I::Trait(i) => &i.attrs,
        I::TraitAlias(i) => &i.attrs,
        I::Type(i) => &i.attrs,
        I::Union(i) => &i.attrs,
        I::Use(i) => &i.attrs,
        _ => &[],
    }
}

fn start_line(node: &impl Spanned) -> usize {
    node.span().start().line
}

impl<'ast> Visit<'ast> for Scan<'_> {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        if item_attrs(item).iter().any(is_test_attr) {
            return;
        }
        let line = start_line(item);
        self.with_anchor(line, |s| {
            match item {
                syn::Item::Const(c) => s.constant(line, &format!("`const {}`", c.ident)),
                syn::Item::Static(c) => s.constant(line, &format!("`static {}`", c.ident)),
                _ => {}
            }
            visit::visit_item(s, item);
        });
    }

    fn visit_impl_item(&mut self, item: &'ast syn::ImplItem) {
        if let syn::ImplItem::Fn(f) = item
            && f.attrs.iter().any(is_test_attr)
        {
            return;
        }
        let line = start_line(item);
        self.with_anchor(line, |s| {
            if let syn::ImplItem::Const(c) = item {
                s.constant(line, &format!("`const {}`", c.ident));
            }
            visit::visit_impl_item(s, item);
        });
    }

    fn visit_trait_item(&mut self, item: &'ast syn::TraitItem) {
        let line = start_line(item);
        self.with_anchor(line, |s| {
            if let syn::TraitItem::Const(c) = item
                && c.default.is_some()
            {
                s.constant(line, &format!("`const {}`", c.ident));
            }
            visit::visit_trait_item(s, item);
        });
    }

    fn visit_stmt(&mut self, node: &'ast syn::Stmt) {
        self.with_anchor(start_line(node), |s| visit::visit_stmt(s, node));
    }

    fn visit_arm(&mut self, node: &'ast syn::Arm) {
        self.with_anchor(start_line(node), |s| visit::visit_arm(s, node));
    }

    fn visit_field(&mut self, node: &'ast syn::Field) {
        self.with_anchor(start_line(node), |s| visit::visit_field(s, node));
    }

    fn visit_field_value(&mut self, node: &'ast syn::FieldValue) {
        self.with_anchor(start_line(node), |s| visit::visit_field_value(s, node));
    }

    fn visit_variant(&mut self, node: &'ast syn::Variant) {
        self.with_anchor(start_line(node), |s| visit::visit_variant(s, node));
    }

    fn visit_lit(&mut self, lit: &'ast syn::Lit) {
        let text = match lit {
            syn::Lit::Int(i) => i.to_string(),
            syn::Lit::Float(f) => f.to_string(),
            _ => return,
        };
        self.number(start_line(lit), &text);
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        self.tokens(mac.tokens.clone());
    }
}

/// Scans one Rust source file.
pub fn lint_source(src: &str) -> Result<Vec<Finding>, String> {
    let file = syn::parse_file(src).map_err(|e| {
        let at = e.span().start();
        format!("{}:{}: {e}", at.line, at.column + 1)
    })?;
    let mut scan = Scan {
        lines: src.lines().collect(),
        anchors: Vec::new(),
        findings: Vec::new(),
    };
    scan.visit_file(&file);
    scan.findings.sort_by_key(|f| f.line);
    Ok(scan.findings)
}

// ---------------------------------------------------------------------------
// ID index

/// IDs that exist in `spec/`, `measurements/` and `spec/requests/`, with the
/// spec's `status:` where it has one.
#[derive(Debug, Default)]
pub struct IdIndex {
    ids: BTreeMap<String, String>,
}

impl IdIndex {
    /// Scans the repository.
    pub fn load(root: &Path) -> Self {
        let mut idx = Self::default();
        for (dir, letter) in [("spec", 'S'), ("measurements", 'M'), ("spec/requests", 'Q')] {
            for path in workspace::files_with_extension(&root.join(dir), "md") {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                let Some(id) = id_tokens(&name).into_iter().next() else {
                    continue;
                };
                if !id.starts_with(letter) || !name.starts_with(&id) {
                    continue;
                }
                let status = std::fs::read_to_string(&path)
                    .ok()
                    .and_then(|text| {
                        text.lines()
                            .find_map(|l| l.strip_prefix("status:"))
                            .map(|s| s.split('#').next().unwrap_or("").trim().to_owned())
                    })
                    .unwrap_or_default();
                idx.ids.insert(id, status);
            }
        }
        idx
    }

    /// Builds an index from `(id, status)` pairs (tests).
    #[cfg(test)]
    pub fn from_pairs(pairs: &[(&str, &str)]) -> Self {
        Self {
            ids: pairs
                .iter()
                .map(|(i, s)| ((*i).to_owned(), (*s).to_owned()))
                .collect(),
        }
    }

    /// Problems with the IDs mentioned in `text`, as (line, message).
    pub fn check_text(&self, text: &str) -> Vec<(usize, String)> {
        let mut out = Vec::new();
        for (n, line) in text.lines().enumerate() {
            for id in id_tokens(line) {
                match self.ids.get(&id) {
                    None => out.push((
                        n + 1,
                        format!("{id} does not exist in spec/ or measurements/"),
                    )),
                    Some(status) if status == "retired" => {
                        out.push((n + 1, format!("{id} is retired; cite its replacement")));
                    }
                    Some(_) => {}
                }
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Allowlist and commands

/// Parses the allowlist: `<path> <rule>[,<rule>] # <reason>` per line.
pub fn parse_allowlist(text: &str) -> Result<BTreeMap<String, BTreeSet<Rule>>, String> {
    let mut out = BTreeMap::new();
    for (n, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (entry, reason) = line
            .split_once('#')
            .ok_or_else(|| format!("{ALLOWLIST_FILE}:{}: entry needs a `# reason`", n + 1))?;
        if reason.trim().is_empty() {
            return Err(format!("{ALLOWLIST_FILE}:{}: empty reason", n + 1));
        }
        let mut parts = entry.split_whitespace();
        let (Some(path), Some(rules), None) = (parts.next(), parts.next(), parts.next()) else {
            return Err(format!(
                "{ALLOWLIST_FILE}:{}: expected `<path> <rules> # <reason>`",
                n + 1
            ));
        };
        let rules = rules
            .split(',')
            .map(|r| match r {
                "cite" => Ok(Rule::Cite),
                "float" => Ok(Rule::Float),
                other => Err(format!(
                    "{ALLOWLIST_FILE}:{}: unknown rule {other:?}",
                    n + 1
                )),
            })
            .collect::<Result<BTreeSet<_>, _>>()?;
        out.insert(path.to_owned(), rules);
    }
    Ok(out)
}

/// Runs the source scan over every simulation crate's `src/` and returns
/// findings for `rule`, honouring the allowlist.
fn scan_simulation_sources(rule: Rule) -> Result<Vec<String>, String> {
    let root = workspace::root();
    let md = workspace::metadata()?;
    let (det, _) = workspace::members(&md);
    let allow_text = std::fs::read_to_string(root.join(ALLOWLIST_FILE))
        .map_err(|e| format!("{ALLOWLIST_FILE}: {e}"))?;
    let allow = parse_allowlist(&allow_text)?;

    let mut out = Vec::new();
    for path in allow.keys() {
        if !root.join(path).is_file() {
            out.push(format!(
                "{ALLOWLIST_FILE}: stale entry {path} (file does not exist)"
            ));
        }
    }
    for (_, dir) in det {
        for file in workspace::files_with_extension(&dir.join("src"), "rs") {
            let rel_path = rel(&file);
            if allow.get(&rel_path).is_some_and(|r| r.contains(&rule)) {
                continue;
            }
            let src = std::fs::read_to_string(&file).map_err(|e| format!("{rel_path}: {e}"))?;
            let findings = lint_source(&src).map_err(|e| format!("{rel_path}:{e}"))?;
            out.extend(
                findings
                    .into_iter()
                    .filter(|f| f.rule == rule)
                    .map(|f| format!("{rel_path}:{}: {}", f.line, f.message)),
            );
        }
    }
    Ok(out)
}

/// `cargo xtask cite`.
pub fn run() -> Result<Vec<String>, String> {
    let root = workspace::root();
    let mut out = scan_simulation_sources(Rule::Cite)?;
    let index = IdIndex::load(&root);
    let files = workspace::files_with_extension(&root.join("crates"), "rs")
        .into_iter()
        .chain(workspace::files_with_extension(&root.join("data"), "jsonc"));
    for file in files {
        let text = std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", rel(&file)))?;
        out.extend(
            index
                .check_text(&text)
                .into_iter()
                .map(|(line, msg)| format!("{}:{line}: {msg}", rel(&file))),
        );
    }
    Ok(out)
}

/// `cargo xtask determinism`: float literals, and one lint table for all
/// simulation crates.
pub fn run_determinism() -> Result<Vec<String>, String> {
    let mut out = scan_simulation_sources(Rule::Float)?;
    let md = workspace::metadata()?;
    let (det, _) = workspace::members(&md);
    let mut tables = Vec::new();
    for (name, dir) in &det {
        let manifest = dir.join("Cargo.toml");
        let text = std::fs::read_to_string(&manifest).map_err(|e| format!("{name}: {e}"))?;
        let value: toml::Table = text.parse().map_err(|e| format!("{name}: {e}"))?;
        let lints = value.get("lints").cloned();
        match &lints {
            Some(toml::Value::Table(t)) if t.get("workspace").is_none() && t.contains_key("clippy") => {}
            _ => out.push(format!(
                "{}: a simulation crate needs its own [lints] table with the determinism lints, not `workspace = true`",
                rel(&manifest)
            )),
        }
        tables.push((name.clone(), lints));
    }
    if let Some((first_name, first)) = tables.first() {
        for (name, t) in &tables[1..] {
            if t != first {
                out.push(format!("{name}: [lints] differs from {first_name}; keep the simulation lint tables identical"));
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lint(src: &str) -> Vec<(usize, Rule)> {
        lint_source(src)
            .unwrap()
            .into_iter()
            .map(|f| (f.line, f.rule))
            .collect()
    }

    #[test]
    fn uncited_numbers_and_consts_are_flagged() {
        let src = "\
fn speed() -> u32 {
    let base = 7;
    base * 3
}
const LIMIT: u8 = 1;
";
        assert_eq!(
            lint(src),
            vec![(2, Rule::Cite), (3, Rule::Cite), (5, Rule::Cite)]
        );
    }

    #[test]
    fn zero_and_one_are_free() {
        assert!(lint("fn f(x: u32) -> u32 { x + 1 - 0 }").is_empty());
        assert!(lint("fn f(v: &[u8]) -> u8 { v[0] }").is_empty());
    }

    #[test]
    fn citations_cover_lines_items_and_blocks() {
        let src = "\
// spec: S-0042
const RELOAD_RESETS_ON_MOVE: bool = true;

/// Reload interval rule.
/// measure: M-0007
fn reload(x: u32) -> u32 {
    x * 15
}

fn other() -> u32 {
    let a = 12; // spec: S-0001
    // spec: S-0002
    let b = 13;
    match a {
        // nocite: lookup table size of our own design
        5 => 6,
        _ => b,
    }
}
";
        assert!(lint(src).is_empty(), "{:?}", lint(src));
    }

    #[test]
    fn a_blank_line_breaks_the_comment_block() {
        let src = "\
// spec: S-0042

const X: u32 = 5;
";
        assert_eq!(lint(src), vec![(3, Rule::Cite), (3, Rule::Cite)]);
    }

    #[test]
    fn malformed_citations_do_not_count() {
        for comment in [
            "// spec: S-42",
            "// spec S-0042",
            "// spec: M-0042",
            "// nocite:",
        ] {
            let src = format!("{comment}\nconst X: u32 = 5;\n");
            assert_eq!(
                lint(&src).len(),
                2,
                "{comment} must not count as a citation"
            );
        }
    }

    #[test]
    fn macro_tokens_are_scanned() {
        let src = "fn f() -> Vec<u32> { vec![1, 2, 0x10] }";
        assert_eq!(lint(src), vec![(1, Rule::Cite), (1, Rule::Cite)]);
    }

    #[test]
    fn floats_are_flagged_even_when_cited() {
        let src = "fn f() { let x = 1.5; let y = 2f32; } // spec: S-0001";
        assert_eq!(lint(src), vec![(1, Rule::Float), (1, Rule::Float)]);
    }

    #[test]
    fn test_code_is_skipped() {
        let src = "\
#[cfg(test)]
mod tests {
    const N: u32 = 99;
    #[test]
    fn t() { assert_eq!(2 + 2, 4); }
}
#[test]
fn top() { let _ = 77; }
";
        assert!(lint(src).is_empty(), "{:?}", lint(src));
    }

    #[test]
    fn classifies_number_tokens() {
        assert_eq!(classify_number("1_000u32"), Some(Number::Int(1000)));
        assert_eq!(classify_number("0xFFu8"), Some(Number::Int(255)));
        assert_eq!(classify_number("0b11"), Some(Number::Int(3)));
        assert_eq!(classify_number("1e3"), Some(Number::Float));
        assert_eq!(classify_number("2.0"), Some(Number::Float));
        assert_eq!(classify_number("\"12\""), None);
    }

    #[test]
    fn id_index_checks_existence_and_status() {
        let idx = IdIndex::from_pairs(&[
            ("S-0001", "approved"),
            ("S-0002", "retired"),
            ("M-0003", ""),
        ]);
        let text =
            "// spec: S-0001\n// spec: S-0002\n// measure: M-0003, M-0004\nlet s = \"XS-0001\";";
        let problems = idx.check_text(text);
        assert_eq!(
            problems,
            vec![
                (2, "S-0002 is retired; cite its replacement".to_owned()),
                (
                    3,
                    "M-0004 does not exist in spec/ or measurements/".to_owned()
                ),
            ]
        );
    }

    #[test]
    fn id_tokens_need_exactly_four_digits() {
        assert_eq!(
            id_tokens("S-0001 S-00012 xS-0003 (Q-0004)"),
            vec!["S-0001", "Q-0004"]
        );
    }

    #[test]
    fn allowlist_requires_reasons() {
        let ok = parse_allowlist("# comment\na/b.rs cite,float # parser, not behaviour\n").unwrap();
        assert_eq!(ok["a/b.rs"], BTreeSet::from([Rule::Cite, Rule::Float]));
        assert!(parse_allowlist("a/b.rs cite\n").is_err());
        assert!(parse_allowlist("a/b.rs cite #  \n").is_err());
        assert!(parse_allowlist("a/b.rs bogus # why\n").is_err());
    }
}
