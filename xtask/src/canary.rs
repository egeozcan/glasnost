//! Proves that the determinism lints still fire.
//!
//! A misspelt or silently ignored entry in `crates/deterministic/clippy.toml`
//! would let floats, hash maps or clocks into the simulation unnoticed. This
//! check creates a throwaway crate *inside* `crates/deterministic/` (so clippy
//! finds the config by the same upward search the real crates use), copies
//! the simulation lint table into it, fills it with one violation per
//! configured entry, runs clippy and requires every expected diagnostic.

use std::collections::BTreeSet;
use std::io::BufReader;
use std::process::Command;

use cargo_metadata::Message;

use crate::workspace;

/// Violations keyed by the clippy.toml path or the lint they trigger.
/// Every disallowed-* entry and every lint in the simulation lint table
/// needs a snippet here; the check fails if one is missing.
const SNIPPETS: &[(&str, &str)] = &[
    // disallowed-types
    ("f32", "pub fn t_f32(x: f32) -> u32 { x as u32 }"),
    ("f64", "pub fn t_f64(x: f64) -> u32 { x as u32 }"),
    (
        "std::collections::HashMap",
        "pub fn t_hm() -> usize { std::collections::HashMap::<u8, u8>::new().len() }",
    ),
    (
        "std::collections::HashSet",
        "pub fn t_hs() -> usize { std::collections::HashSet::<u8>::new().len() }",
    ),
    (
        "std::hash::RandomState",
        "pub fn t_rs() { let _ = std::hash::RandomState::new(); }",
    ),
    (
        "std::hash::DefaultHasher",
        "pub fn t_dh() { let _ = std::hash::DefaultHasher::new(); }",
    ),
    (
        "std::time::Instant",
        "pub fn t_in(_: Option<std::time::Instant>) {}",
    ),
    (
        "std::time::SystemTime",
        "pub fn t_st(_: Option<std::time::SystemTime>) {}",
    ),
    (
        "std::thread::JoinHandle",
        "pub fn t_jh(_: Option<std::thread::JoinHandle<()>>) {}",
    ),
    (
        "std::thread::Builder",
        "pub fn t_tb() { let _ = std::thread::Builder::new(); }",
    ),
    (
        "std::sync::Mutex",
        "pub fn t_mx(_: Option<std::sync::Mutex<u8>>) {}",
    ),
    (
        "std::sync::RwLock",
        "pub fn t_rw(_: Option<std::sync::RwLock<u8>>) {}",
    ),
    (
        "std::sync::mpsc::Sender",
        "pub fn t_tx(_: Option<std::sync::mpsc::Sender<u8>>) {}",
    ),
    (
        "std::sync::mpsc::Receiver",
        "pub fn t_rx(_: Option<std::sync::mpsc::Receiver<u8>>) {}",
    ),
    ("std::fs::File", "pub fn t_fi(_: Option<std::fs::File>) {}"),
    (
        "std::net::TcpStream",
        "pub fn t_tcp(_: Option<std::net::TcpStream>) {}",
    ),
    (
        "std::net::UdpSocket",
        "pub fn t_udp(_: Option<std::net::UdpSocket>) {}",
    ),
    (
        "std::net::TcpListener",
        "pub fn t_tcpl(_: Option<std::net::TcpListener>) {}",
    ),
    (
        "std::process::Command",
        "pub fn t_cmd(_: Option<std::process::Command>) {}",
    ),
    // disallowed-methods
    (
        "std::time::Instant::now",
        "pub fn m_in() { let _ = std::time::Instant::now(); }",
    ),
    (
        "std::time::SystemTime::now",
        "pub fn m_st() { let _ = std::time::SystemTime::now(); }",
    ),
    (
        "std::thread::spawn",
        "pub fn m_sp() { let _ = std::thread::spawn(|| ()); }",
    ),
    (
        "std::thread::scope",
        "pub fn m_sc() { std::thread::scope(|_| ()); }",
    ),
    (
        "std::thread::sleep",
        "pub fn m_sl() { std::thread::sleep(std::time::Duration::ZERO); }",
    ),
    (
        "std::env::var",
        "pub fn m_ev() { let _ = std::env::var(\"X\"); }",
    ),
    (
        "std::env::var_os",
        "pub fn m_evo() { let _ = std::env::var_os(\"X\"); }",
    ),
    (
        "std::env::vars",
        "pub fn m_evs() { let _ = std::env::vars(); }",
    ),
    (
        "std::env::args",
        "pub fn m_ea() { let _ = std::env::args(); }",
    ),
    (
        "std::fs::read",
        "pub fn m_fr() { let _ = std::fs::read(\"x\"); }",
    ),
    (
        "std::fs::read_to_string",
        "pub fn m_frs() { let _ = std::fs::read_to_string(\"x\"); }",
    ),
    (
        "std::fs::write",
        "pub fn m_fw() { let _ = std::fs::write(\"x\", b\"\"); }",
    ),
    (
        "std::process::exit",
        "pub fn m_ex() -> ! { std::process::exit(0) }",
    ),
    // disallowed-macros
    ("std::println", "pub fn x_pl() { println!(\"x\"); }"),
    ("std::eprintln", "pub fn x_epl() { eprintln!(\"x\"); }"),
    ("std::print", "pub fn x_p() { print!(\"x\"); }"),
    ("std::eprint", "pub fn x_ep() { eprint!(\"x\"); }"),
    ("std::dbg", "pub fn x_dbg() { dbg!(1u8); }"),
    (
        "std::ptr::addr_of",
        "pub static X_ADDR: u8 = 0; pub fn x_ad() { let _ = std::ptr::addr_of!(X_ADDR); }",
    ),
    // lint table
    (
        "float_arithmetic",
        "pub fn l_fa(a: f32, b: f32) -> f32 { a + b }",
    ),
    (
        "float_cmp",
        "pub fn l_fc(a: f32, b: f32) -> bool { a == b }",
    ),
    (
        "float_cmp_const",
        "pub const L_C: f32 = 1.5; pub fn l_fcc(a: f32) -> bool { a == L_C }",
    ),
    (
        "lossy_float_literal",
        "pub fn l_lf() -> f32 { 16_777_217.0 }",
    ),
    (
        "cast_precision_loss",
        "pub fn l_cp(x: u64) -> f64 { x as f64 }",
    ),
];

/// Lints in the simulation table that need no canary snippet: group levels,
/// and the `disallowed_*` lints, which the per-entry snippets exercise.
const TABLE_LINTS_WITHOUT_SNIPPET: &[&str] = &[
    "all",
    "disallowed_types",
    "disallowed_methods",
    "disallowed_macros",
];

struct Expectation {
    key: String,
    /// Diagnostic code, e.g. `clippy::disallowed_types`.
    code: String,
    /// Substring of the message, if the code alone is not specific.
    needle: Option<String>,
}

fn expectations(
    clippy_toml: &toml::Table,
    lint_table: &toml::Table,
) -> Result<Vec<Expectation>, String> {
    let mut out = Vec::new();
    for (key, lint, what) in [
        ("disallowed-types", "disallowed_types", "type"),
        ("disallowed-methods", "disallowed_methods", "method"),
        ("disallowed-macros", "disallowed_macros", "macro"),
    ] {
        let entries = clippy_toml
            .get(key)
            .and_then(toml::Value::as_array)
            .ok_or_else(|| format!("clippy.toml has no {key} list"))?;
        for e in entries {
            let path = e
                .get("path")
                .and_then(toml::Value::as_str)
                .or_else(|| e.as_str())
                .ok_or_else(|| format!("{key}: entry without a path"))?;
            out.push(Expectation {
                key: path.to_owned(),
                code: format!("clippy::{lint}"),
                needle: Some(format!("disallowed {what} `{path}`")),
            });
        }
    }
    for lint in lint_table.keys() {
        if !TABLE_LINTS_WITHOUT_SNIPPET.contains(&lint.as_str()) {
            out.push(Expectation {
                key: lint.clone(),
                code: format!("clippy::{lint}"),
                needle: None,
            });
        }
    }
    Ok(out)
}

/// `cargo xtask canary`.
pub fn run() -> Result<Vec<String>, String> {
    let root = workspace::root();
    let det_dir = root.join(workspace::DETERMINISTIC_DIR);
    let clippy_toml: toml::Table = std::fs::read_to_string(det_dir.join("clippy.toml"))
        .map_err(|e| format!("clippy.toml: {e}"))?
        .parse()
        .map_err(|e| format!("clippy.toml: {e}"))?;
    let sim_manifest: toml::Table = std::fs::read_to_string(det_dir.join("sim/Cargo.toml"))
        .map_err(|e| format!("sim/Cargo.toml: {e}"))?
        .parse()
        .map_err(|e| format!("sim/Cargo.toml: {e}"))?;
    let lints = sim_manifest
        .get("lints")
        .and_then(toml::Value::as_table)
        .ok_or("sim/Cargo.toml has no [lints] table")?;
    let clippy_lints = lints
        .get("clippy")
        .and_then(toml::Value::as_table)
        .ok_or("sim/Cargo.toml has no [lints.clippy] table")?;

    let expected = expectations(&clippy_toml, clippy_lints)?;
    let mut findings = Vec::new();
    let mut source =
        String::from("//! Determinism-lint canary (generated by `cargo xtask canary`).\n");
    for e in &expected {
        match SNIPPETS.iter().find(|(k, _)| *k == e.key) {
            Some((_, code)) => {
                source.push_str(code);
                source.push('\n');
            }
            None => findings.push(format!(
                "canary has no snippet for `{}`; add one to SNIPPETS in xtask/src/canary.rs",
                e.key
            )),
        }
    }
    if !findings.is_empty() {
        return Ok(findings);
    }

    // The canary crate must sit beside the real ones so clippy's upward
    // config search behaves the same; it is its own workspace so the main
    // workspace ignores it, and it is deleted afterwards.
    let crate_dir = det_dir.join(".canary");
    let _ = std::fs::remove_dir_all(&crate_dir);
    std::fs::create_dir_all(crate_dir.join("src")).map_err(|e| e.to_string())?;
    let mut manifest = toml::Table::new();
    manifest.insert(
        "package".into(),
        toml::Value::Table(
            [
                ("name".to_owned(), toml::Value::from("determinism-canary")),
                ("version".to_owned(), toml::Value::from("0.0.0")),
                ("edition".to_owned(), toml::Value::from("2024")),
                ("publish".to_owned(), toml::Value::from(false)),
            ]
            .into_iter()
            .collect(),
        ),
    );
    manifest.insert("workspace".into(), toml::Value::Table(toml::Table::new()));
    manifest.insert("lints".into(), toml::Value::Table(lints.clone()));
    std::fs::write(crate_dir.join("Cargo.toml"), manifest.to_string())
        .map_err(|e| e.to_string())?;
    std::fs::write(crate_dir.join("src/lib.rs"), &source).map_err(|e| e.to_string())?;

    let output = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args([
            "clippy",
            "--quiet",
            "--message-format=json",
            "--manifest-path",
        ])
        .arg(crate_dir.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", root.join("target/xtask-canary"))
        .env_remove("CLIPPY_CONF_DIR")
        .output();
    let _ = std::fs::remove_dir_all(&crate_dir);
    let output = output.map_err(|e| format!("running cargo clippy: {e}"))?;

    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    for message in Message::parse_stream(BufReader::new(output.stdout.as_slice())).flatten() {
        if let Message::CompilerMessage(m) = message
            && let Some(code) = &m.message.code
        {
            seen.insert((code.code.clone(), m.message.message.clone()));
        }
    }
    if output.status.success() {
        findings
            .push("clippy accepted the canary crate; the determinism lints are not firing".into());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    if seen.is_empty() && !stderr.trim().is_empty() {
        findings.push(format!("clippy produced no diagnostics:\n{stderr}"));
    }
    for e in &expected {
        let hit = seen.iter().any(|(code, msg)| {
            *code == e.code && e.needle.as_ref().is_none_or(|n| msg.contains(n.as_str()))
        });
        if !hit {
            findings.push(format!("`{}` did not trigger {}", e.key, e.code));
        }
    }
    Ok(findings)
}
