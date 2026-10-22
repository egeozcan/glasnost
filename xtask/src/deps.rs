//! The dependency rule (PLAN.md 4.2): simulation crates consume only the
//! typed ruleset and typed map. They must not depend, directly or
//! transitively, on any other workspace crate (render, audio, net, fmt, ini,
//! convert, ...) or on crates that bring in graphics, host access,
//! threads or host randomness.
//!
//! Normal and build dependencies count, for every target platform; dev
//! dependencies (tests only) do not.
//!
//! The lockstep crate `glasnost-net` (M9) has no I/O either: it may depend on
//! the simulation crates, but on none of the external crates below (no
//! runtime, socket, clock or host-randomness crate) and not on the relay.
//!
//! The relay (`glasnost-relay`, M9 N2) is a native server: no crate of the
//! game client (the app and the platform shells) may depend on it. The
//! client crates may reach only the external crates of [`CLIENT_ALLOWED`]
//! among those below: graphics, the browser, what those bring, and (M9 N3)
//! the native client's WebSocket. Never a server runtime's I/O driver, an
//! async WebSocket or an HTTP stack.

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
    "futures",
    "futures-util",
    "futures-channel",
    "futures-executor",
    // Wall-clock time.
    "chrono",
    "time",
    "web-time",
    "instant",
    // Sockets and network protocols (M9: networking lives in glasnost-net,
    // the relay and the client, never in the simulation).
    "mio",
    "socket2",
    "tungstenite",
    "tokio-tungstenite",
    "async-tungstenite",
    "matchbox_socket",
    "webrtc",
    "hyper",
    "reqwest",
    "ureq",
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
                let counts = dep.dep_kinds.iter().any(|k| {
                    k.kind != DependencyKind::Development
                        && !k
                            .target
                            .as_ref()
                            .is_some_and(|t| never_built(&t.to_string()))
                });
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

/// Whether a dependency's target condition holds only on emscripten, a
/// target the project never builds (it builds for native hosts and for
/// `wasm32-unknown-unknown`), so that no build of ours compiles it.
pub fn never_built(target: &str) -> bool {
    let t = target.replace(' ', "");
    t == r#"cfg(target_os="emscripten")"# || t.starts_with(r#"cfg(all(target_os="emscripten","#)
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
    let net_present = other.iter().any(|(n, _)| n == NET_CRATE);
    let forbidden: BTreeSet<String> = other
        .into_iter()
        .map(|(n, _)| n)
        .chain(FORBIDDEN_EXTERNAL.iter().map(|s| (*s).to_owned()))
        .collect();
    let mut out = violations(&graph, &roots, &forbidden);
    // The lockstep core takes its clock and its bytes from the caller: no
    // runtime, socket or clock crate either (M9, docs/m9-plan.md).
    if net_present {
        let external: BTreeSet<String> = FORBIDDEN_EXTERNAL
            .iter()
            .map(|s| (*s).to_owned())
            .chain([RELAY_CRATE.to_owned()])
            .collect();
        out.extend(violations(&graph, &[NET_CRATE.to_owned()], &external));
    }
    // The relay is a native server, never part of the game client.
    let client: Vec<String> = CLIENT_CRATES
        .iter()
        .filter(|c| graph.edges.contains_key(**c))
        .map(|c| (*c).to_owned())
        .collect();
    out.extend(violations(&graph, &client, &client_forbidden()));
    Ok(out)
}

/// The lockstep crate, which has no I/O of its own.
pub const NET_CRATE: &str = "glasnost-net";

/// The relay and lobby server (native only).
pub const RELAY_CRATE: &str = "glasnost-relay";

/// The crates of the game client, native and web.
pub const CLIENT_CRATES: &[&str] = &[
    "glasnost-app",
    "glasnost-platform-native",
    "glasnost-platform-web",
    // M9 N6: the soak's wasm client (the game client under node).
    "glasnost-headless-web",
];

/// The external crates of [`FORBIDDEN_EXTERNAL`] the client crates may
/// reach.
pub const CLIENT_ALLOWED: &[&str] = &[
    // The window, the GPU and the browser.
    "wgpu",
    "winit",
    "web-sys",
    "js-sys",
    "wasm-bindgen",
    // What egui, wgpu and winit bring: hash maps, frame timing, futures
    // helpers.
    "ahash",
    "foldhash",
    "web-time",
    "futures-util",
    // M9 N3: the native client's blocking WebSocket to the relay, with the
    // host randomness of its frame masks.
    "tungstenite",
    "rand",
    "getrandom",
];

/// What the client crates must not reach: the relay, and every external
/// crate of [`FORBIDDEN_EXTERNAL`] outside [`CLIENT_ALLOWED`].
fn client_forbidden() -> BTreeSet<String> {
    FORBIDDEN_EXTERNAL
        .iter()
        .filter(|c| !CLIENT_ALLOWED.contains(c))
        .map(|c| (*c).to_owned())
        .chain([RELAY_CRATE.to_owned()])
        .collect()
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
    fn only_emscripten_dependencies_are_never_built() {
        assert!(never_built(r#"cfg(target_os = "emscripten")"#));
        assert!(never_built(
            r#"cfg(all(target_os = "emscripten", wasm_bindgen_unstable_tokio))"#
        ));
        assert!(!never_built(r#"cfg(any(target_os = "emscripten", unix))"#));
        assert!(!never_built(r#"cfg(target_arch = "wasm32")"#));
        assert!(!never_built(r#"cfg(not(target_os = "emscripten"))"#));
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

    /// No simulation crate and not the lockstep crate reaches the relay's
    /// runtime, socket and randomness crates or the relay itself, while the
    /// relay does (so the graph seen is the real one).
    #[test]
    fn only_the_relay_reaches_the_async_runtime() {
        let md = workspace::metadata().expect("cargo metadata");
        let graph = Graph::from_metadata(&md).expect("a graph");
        let (det, _) = workspace::members(&md);
        let mut roots: Vec<String> = det.into_iter().map(|(n, _)| n).collect();
        roots.push(NET_CRATE.to_owned());
        let runtime = set(&[
            "tokio",
            "tokio-tungstenite",
            "tungstenite",
            "futures-util",
            "getrandom",
            RELAY_CRATE,
        ]);
        let v = violations(&graph, &roots, &runtime);
        assert!(v.is_empty(), "{}", v.join("\n"));
        let relay = graph.paths_from(RELAY_CRATE);
        for dep in ["tokio", "tokio-tungstenite", "futures-util", NET_CRATE] {
            assert!(relay.contains_key(dep), "the relay reaches {dep}");
        }
    }

    /// The client reaches the lockstep crate and the native WebSocket, but
    /// neither the relay nor a server runtime's I/O or async WebSocket; its
    /// allowance holds for the client crates only.
    #[test]
    fn the_client_reaches_the_socket_but_no_server_runtime() {
        let md = workspace::metadata().expect("cargo metadata");
        let graph = Graph::from_metadata(&md).expect("a graph");
        let app = graph.paths_from("glasnost-app");
        for dep in ["tungstenite", NET_CRATE] {
            assert!(app.contains_key(dep), "the client reaches {dep}");
        }
        let forbidden = client_forbidden();
        assert!(!app.contains_key("tokio"), "the client reaches no runtime");
        for dep in [
            "tokio",
            "mio",
            "socket2",
            "tokio-tungstenite",
            "hyper",
            RELAY_CRATE,
        ] {
            assert!(forbidden.contains(dep), "{dep} stays forbidden");
        }
        let client: Vec<String> = CLIENT_CRATES.iter().map(|c| (*c).to_owned()).collect();
        let v = violations(&graph, &client, &forbidden);
        assert!(v.is_empty(), "{}", v.join("\n"));
        // The simulation and the lockstep crate are not allowed what the
        // client is.
        let (det, _) = workspace::members(&md);
        let mut roots: Vec<String> = det.into_iter().map(|(n, _)| n).collect();
        roots.push(NET_CRATE.to_owned());
        let v = violations(&graph, &roots, &set(CLIENT_ALLOWED));
        assert!(v.is_empty(), "{}", v.join("\n"));
        // A client crate that reached the relay's socket stack would fail.
        let mut g = Graph::default();
        g.add("glasnost-app", "tungstenite");
        g.add("glasnost-app", "tokio-tungstenite");
        g.add("tokio-tungstenite", "tokio");
        g.add("tokio", "mio");
        let v = violations(&g, &["glasnost-app".into()], &forbidden);
        assert_eq!(
            v,
            vec![
                "glasnost-app must not depend on mio (via glasnost-app -> tokio-tungstenite -> tokio -> mio)",
                "glasnost-app must not depend on tokio (via glasnost-app -> tokio-tungstenite -> tokio)",
                "glasnost-app must not depend on tokio-tungstenite (via glasnost-app -> tokio-tungstenite)",
            ]
        );
    }
}
