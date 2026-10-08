//! The dependency rule (PLAN.md 4.2): simulation crates consume only the
//! typed ruleset and typed map. They must not depend, directly or
//! transitively, on any other workspace crate (render, audio, net, fmt, ini,
//! convert, ...) or on crates that bring in graphics, host access,
//! threads or host randomness.
//!
//! Normal and build dependencies count, for every target platform; dev
//! dependencies (tests only) do not.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use cargo_metadata::{DependencyKind, Metadata};

use crate::workspace;

/// External crates no simulation crate may reach.
pub const FORBIDDEN_EXTERNAL: &[&str] = &[
    // Graphics, windowing, browser bindings.
    "wgpu",
    "winit",
    "web-sys",
    "js-sys",
    "wasm-bindgen",
    // Host randomness and hash-seeded maps.
    "getrandom",
    "rand",
    "ahash",
    "foldhash",
    // Threads and async runtimes.
    "rayon",
    "crossbeam",
    "tokio",
    "async-std",
    // Wall-clock time.
    "chrono",
    "time",
    "web-time",
    "instant",
];

/// A dependency graph over package names.
#[derive(Debug, Default)]
pub struct Graph {
    edges: BTreeMap<String, BTreeSet<String>>,
}

impl Graph {
    /// Adds an edge `from → to`.
    pub fn add(&mut self, from: &str, to: &str) {
        self.edges
            .entry(from.to_owned())
            .or_default()
            .insert(to.to_owned());
    }

    /// Builds the graph of normal and build dependencies from cargo metadata.
    pub fn from_metadata(md: &Metadata) -> Result<Self, String> {
        let resolve = md.resolve.as_ref().ok_or("cargo metadata has no resolve")?;
        let name_of: BTreeMap<_, _> = md
            .packages
            .iter()
            .map(|p| (p.id.clone(), p.name.to_string()))
            .collect();
        let mut g = Self::default();
        for node in &resolve.nodes {
            let from = &name_of[&node.id];
            for dep in &node.deps {
                let counts = dep
                    .dep_kinds
                    .iter()
                    .any(|k| k.kind != DependencyKind::Development);
                if counts {
                    g.add(from, &name_of[&dep.pkg]);
                }
            }
        }
        Ok(g)
    }

    /// Shortest dependency path from `root` to each reachable package.
    fn paths_from(&self, root: &str) -> BTreeMap<String, Vec<String>> {
        let mut paths = BTreeMap::new();
        paths.insert(root.to_owned(), vec![root.to_owned()]);
        let mut queue = VecDeque::from([root.to_owned()]);
        while let Some(n) = queue.pop_front() {
            let path = paths[&n].clone();
            for next in self.edges.get(&n).into_iter().flatten() {
                if !paths.contains_key(next) {
                    let mut p = path.clone();
                    p.push(next.clone());
                    paths.insert(next.clone(), p);
                    queue.push_back(next.clone());
                }
            }
        }
        paths
    }
}

/// Every forbidden package reachable from each root, with the path.
pub fn violations(graph: &Graph, roots: &[String], forbidden: &BTreeSet<String>) -> Vec<String> {
    let mut out = Vec::new();
    for root in roots {
        for (pkg, path) in graph.paths_from(root) {
            if forbidden.contains(&pkg) {
                out.push(format!(
                    "{root} must not depend on {pkg} (via {})",
                    path.join(" -> ")
                ));
            }
        }
    }
    out
}

/// The rule applied to the real workspace.
pub fn check(md: &Metadata) -> Result<Vec<String>, String> {
    let graph = Graph::from_metadata(md)?;
    let (det, other) = workspace::members(md);
    if det.is_empty() {
        return Err(format!(
            "no simulation crates found under {}",
            workspace::DETERMINISTIC_DIR
        ));
    }
    let roots: Vec<String> = det.into_iter().map(|(n, _)| n).collect();
    let forbidden: BTreeSet<String> = other
        .into_iter()
        .map(|(n, _)| n)
        .chain(FORBIDDEN_EXTERNAL.iter().map(|s| (*s).to_owned()))
        .collect();
    Ok(violations(&graph, &roots, &forbidden))
}

/// `cargo xtask deps`.
pub fn run() -> Result<Vec<String>, String> {
    check(&workspace::metadata()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn finds_transitive_violations_with_paths() {
        let mut g = Graph::default();
        g.add("sim", "rules");
        g.add("rules", "helper");
        g.add("helper", "render");
        g.add("render", "wgpu");
        let v = violations(&g, &["sim".into()], &set(&["render", "wgpu"]));
        assert_eq!(
            v,
            vec![
                "sim must not depend on render (via sim -> rules -> helper -> render)",
                "sim must not depend on wgpu (via sim -> rules -> helper -> render -> wgpu)",
            ]
        );
    }

    #[test]
    fn clean_graph_passes() {
        let mut g = Graph::default();
        g.add("sim", "rules");
        g.add("render", "sim");
        assert!(violations(&g, &["sim".into()], &set(&["render"])).is_empty());
    }

    /// The real workspace obeys the rule. Runs `cargo metadata`, so plain
    /// `cargo test` enforces the rule too.
    #[test]
    fn workspace_obeys_dependency_rule() {
        let md = workspace::metadata().expect("cargo metadata");
        let (det, _) = workspace::members(&md);
        let names: Vec<_> = det.iter().map(|(n, _)| n.as_str()).collect();
        for expected in [
            "glasnost-sim",
            "glasnost-ai",
            "glasnost-map",
            "glasnost-rules",
        ] {
            assert!(
                names.contains(&expected),
                "{expected} is a simulation crate"
            );
        }
        let v = check(&md).expect("check runs");
        assert!(v.is_empty(), "dependency rule violated:\n{}", v.join("\n"));
    }
}
