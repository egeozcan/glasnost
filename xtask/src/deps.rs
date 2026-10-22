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
//!
//! The native client's direct links (`glasnost-rtc`, M9 N7) are WebRTC
//! through matchbox and webrtc-rs, which bring a tokio runtime, UDP sockets
//! and, unused, matchbox's own async WebSocket. The client crates reach
//! those only through `glasnost-rtc`: the client rule does not look past
//! it, and `glasnost-rtc` has its own list, [`RTC_ALLOWED`]; never an HTTP
//! stack, the relay's WebSocket server or the relay. Neither the
//! simulation nor the lockstep crate may reach `glasnost-rtc`.

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

/// A dependency graph over packages: a package's node is its name, or
/// `name@version` where the lock has several versions of that name (so
/// that what one version brings is never put on another).
#[derive(Debug, Default)]
pub struct Graph {
    edges: BTreeMap<String, BTreeSet<String>>,
    /// The package name of each `name@version` node.
    names: BTreeMap<String, String>,
}

impl Graph {
    /// Adds an edge `from → to`.
    pub fn add(&mut self, from: &str, to: &str) {
        self.edges
            .entry(from.to_owned())
            .or_default()
            .insert(to.to_owned());
    }

    /// The package name of a node.
    fn name<'a>(&'a self, node: &'a str) -> &'a str {
        self.names.get(node).map_or(node, String::as_str)
    }

    /// Builds the graph of normal and build dependencies from cargo metadata.
    pub fn from_metadata(md: &Metadata) -> Result<Self, String> {
        let resolve = md.resolve.as_ref().ok_or("cargo metadata has no resolve")?;
        let mut versions: BTreeMap<String, usize> = BTreeMap::new();
        for p in &md.packages {
            *versions.entry(p.name.to_string()).or_default() += 1;
        }
        let mut g = Self::default();
        let name_of: BTreeMap<_, _> = md
            .packages
            .iter()
            .map(|p| {
                let name = p.name.to_string();
                let node = if versions[&name] > 1 {
                    let node = format!("{name}@{}", p.version);
                    g.names.insert(node.clone(), name);
                    node
                } else {
                    name
                };
                (p.id.clone(), node)
            })
            .collect();
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
    #[cfg(test)]
    fn paths_from(&self, root: &str) -> BTreeMap<String, Vec<String>> {
        self.paths_past(root, &[])
    }

    /// [`Graph::paths_from`], not looking past the packages of `stop`
    /// (they are reached, what they bring is not).
    fn paths_past(&self, root: &str, stop: &[&str]) -> BTreeMap<String, Vec<String>> {
        let mut paths = BTreeMap::new();
        paths.insert(root.to_owned(), vec![root.to_owned()]);
        let mut queue = VecDeque::from([root.to_owned()]);
        while let Some(n) = queue.pop_front() {
            if n != root && stop.contains(&self.name(&n)) {
                continue;
            }
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
    violations_past(graph, roots, forbidden, &[])
}

/// [`violations`], not looking past the packages of `stop` (which have
/// rules of their own).
pub fn violations_past(
    graph: &Graph,
    roots: &[String],
    forbidden: &BTreeSet<String>,
    stop: &[&str],
) -> Vec<String> {
    let mut out = Vec::new();
    for root in roots {
        for (pkg, path) in graph.paths_past(root, stop) {
            if forbidden.contains(graph.name(&pkg)) {
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
            .chain([RELAY_CRATE.to_owned(), RTC_CRATE.to_owned()])
            .collect();
        out.extend(violations(&graph, &[NET_CRATE.to_owned()], &external));
    }
    // The relay is a native server, never part of the game client; the
    // direct links' stack is checked on its own, below.
    let client: Vec<String> = CLIENT_CRATES
        .iter()
        .filter(|c| graph.edges.contains_key(**c))
        .map(|c| (*c).to_owned())
        .collect();
    out.extend(violations_past(
        &graph,
        &client,
        &client_forbidden(),
        &[RTC_CRATE],
    ));
    if graph.edges.contains_key(RTC_CRATE) {
        out.extend(violations(
            &graph,
            &[RTC_CRATE.to_owned()],
            &rtc_forbidden(),
        ));
    }
    Ok(out)
}

/// The lockstep crate, which has no I/O of its own.
pub const NET_CRATE: &str = "glasnost-net";

/// The relay and lobby server (native only).
pub const RELAY_CRATE: &str = "glasnost-relay";

/// The native client's direct links (M9 N7).
pub const RTC_CRATE: &str = "glasnost-rtc";

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
    // M9 N7: the in-process queues of `futures-util`'s `channel` feature,
    // which matchbox turns on for the whole workspace (no I/O, no thread).
    "futures-channel",
];

/// The external crates of [`FORBIDDEN_EXTERNAL`] the direct links' crate
/// ([`RTC_CRATE`]) may reach besides [`CLIENT_ALLOWED`] (M9 N7).
pub const RTC_ALLOWED: &[&str] = &[
    // WebRTC: matchbox over webrtc-rs.
    "matchbox_socket",
    "webrtc",
    // webrtc-rs's runtime and UDP sockets (ICE).
    "tokio",
    "mio",
    "socket2",
    // matchbox's message loop, run on the link's own thread.
    "futures",
    "futures-executor",
    // matchbox's own WebSocket signaller: compiled, never used (the relay
    // carries the signals).
    "async-tungstenite",
    "async-std",
    // The certificate of the DTLS handshake (its validity dates).
    "time",
];

/// What [`RTC_CRATE`] must not reach: the relay, and every external crate
/// of [`FORBIDDEN_EXTERNAL`] outside [`CLIENT_ALLOWED`] and
/// [`RTC_ALLOWED`] (an HTTP stack, the relay's WebSocket server, ...).
fn rtc_forbidden() -> BTreeSet<String> {
    FORBIDDEN_EXTERNAL
        .iter()
        .filter(|c| !CLIENT_ALLOWED.contains(c) && !RTC_ALLOWED.contains(c))
        .map(|c| (*c).to_owned())
        .chain([RELAY_CRATE.to_owned()])
        .collect()
}

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
        // What the client reaches past the direct links' crate (N7).
        let app = graph.paths_past("glasnost-app", &[RTC_CRATE]);
        for dep in ["tungstenite", NET_CRATE] {
            assert!(
                app.keys().any(|n| graph.name(n) == dep),
                "the client reaches {dep}"
            );
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
        let v = violations_past(&graph, &client, &forbidden, &[RTC_CRATE]);
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

    /// The direct links' stack (M9 N7) is reached by the client only
    /// through glasnost-rtc, which reaches WebRTC and its runtime but no
    /// HTTP stack, no server WebSocket and not the relay; the simulation
    /// and the lockstep crate never reach it.
    #[test]
    fn the_direct_links_stack_stays_inside_its_crate() {
        let md = workspace::metadata().expect("cargo metadata");
        let graph = Graph::from_metadata(&md).expect("a graph");
        let rtc = graph.paths_from(RTC_CRATE);
        for dep in ["matchbox_socket", "webrtc", "tokio", NET_CRATE] {
            assert!(rtc.contains_key(dep), "glasnost-rtc reaches {dep}");
        }
        let forbidden = rtc_forbidden();
        for dep in ["hyper", "reqwest", "tokio-tungstenite", RELAY_CRATE] {
            assert!(forbidden.contains(dep), "{dep} stays forbidden");
        }
        let v = violations(&graph, &[RTC_CRATE.to_owned()], &forbidden);
        assert!(v.is_empty(), "{}", v.join("\n"));
        // The app reaches tokio, through glasnost-rtc only.
        let app = graph.paths_from("glasnost-app");
        assert!(
            app["tokio"].contains(&RTC_CRATE.to_owned()),
            "{:?}",
            app["tokio"]
        );
        assert!(
            !graph
                .paths_past("glasnost-app", &[RTC_CRATE])
                .contains_key("tokio")
        );
        let (det, _) = workspace::members(&md);
        let mut roots: Vec<String> = det.into_iter().map(|(n, _)| n).collect();
        roots.push(NET_CRATE.to_owned());
        let v = violations(&graph, &roots, &set(&[RTC_CRATE]));
        assert!(v.is_empty(), "{}", v.join("\n"));
        // A client crate that reached tokio past glasnost-rtc would fail.
        let mut g = Graph::default();
        g.add("glasnost-app", RTC_CRATE);
        g.add(RTC_CRATE, "tokio");
        g.add("glasnost-app", "hyper");
        g.add("hyper", "tokio");
        let v = violations_past(
            &g,
            &["glasnost-app".into()],
            &client_forbidden(),
            &[RTC_CRATE],
        );
        assert_eq!(
            v,
            vec![
                "glasnost-app must not depend on hyper (via glasnost-app -> hyper)",
                "glasnost-app must not depend on tokio (via glasnost-app -> hyper -> tokio)",
            ]
        );
    }

    /// Two versions of a package are two nodes: what one brings is not put
    /// on the other.
    #[test]
    fn versions_of_a_package_keep_their_own_dependencies() {
        let mut g = Graph::default();
        g.add("sim", "digest@0.11");
        g.add("digest@0.11", "crypto-common@0.2");
        g.add("webrtc", "crypto-common@0.1");
        g.add("crypto-common@0.1", "getrandom");
        for (node, name) in [
            ("digest@0.11", "digest"),
            ("crypto-common@0.2", "crypto-common"),
            ("crypto-common@0.1", "crypto-common"),
        ] {
            g.names.insert(node.into(), name.into());
        }
        assert!(violations(&g, &["sim".into()], &set(&["getrandom"])).is_empty());
        assert_eq!(
            violations(&g, &["sim".into()], &set(&["crypto-common"])),
            vec![
                "sim must not depend on crypto-common@0.2 (via sim -> digest@0.11 -> crypto-common@0.2)"
            ]
        );
    }
}
