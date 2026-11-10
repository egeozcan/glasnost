//! `cargo xtask web`: builds the wasm demo into `dist/web` and optionally
//! serves it on 127.0.0.1 (see docs/web.md).

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::workspace;

const PACKAGE: &str = "glasnost-platform-web";
const WASM_NAME: &str = "glasnost_platform_web.wasm";
const TARGET: &str = "wasm32-unknown-unknown";

/// The `wasm-bindgen` version pinned in Cargo.lock.
fn locked_wasm_bindgen(root: &Path) -> Result<String, String> {
    let lock =
        std::fs::read_to_string(root.join("Cargo.lock")).map_err(|e| format!("Cargo.lock: {e}"))?;
    let mut lines = lock.lines();
    while let Some(line) = lines.next() {
        if line == "name = \"wasm-bindgen\""
            && let Some(v) = lines.next().and_then(|l| l.strip_prefix("version = \""))
        {
            return Ok(v.trim_end_matches('"').to_owned());
        }
    }
    Err("wasm-bindgen is not in Cargo.lock".into())
}

fn check_cli(version: &str) -> Result<(), String> {
    let install = format!("cargo install wasm-bindgen-cli --version {version} --locked");
    let out = Command::new("wasm-bindgen")
        .arg("--version")
        .output()
        .map_err(|_| format!("wasm-bindgen not found; install it with `{install}`"))?;
    let found = String::from_utf8_lossy(&out.stdout);
    let found = found.split_whitespace().nth(1).unwrap_or("?");
    if found != version {
        return Err(format!(
            "wasm-bindgen CLI is {found}, Cargo.lock has {version}; run `{install}`"
        ));
    }
    Ok(())
}

fn run_cmd(cmd: &mut Command) -> Result<(), String> {
    let status = cmd.status().map_err(|e| format!("{cmd:?}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{cmd:?} failed with {status}"))
    }
}

/// Builds the demo. Returns the output directory.
pub fn build(release: bool) -> Result<PathBuf, String> {
    let root = workspace::root();
    let version = locked_wasm_bindgen(&root)?;
    check_cli(&version)?;

    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut build = Command::new(cargo);
    build
        .current_dir(&root)
        .args(["build", "--package", PACKAGE, "--target", TARGET]);
    if release {
        build.arg("--release");
    }
    run_cmd(&mut build)?;

    let profile = if release { "release" } else { "debug" };
    // Cargo's own target directory: `CARGO_TARGET_DIR` when set (relative
    // to the workspace root, or absolute), else `target`.
    let target_dir =
        std::env::var_os("CARGO_TARGET_DIR").map_or_else(|| root.join("target"), |d| root.join(d));
    let wasm = target_dir.join(TARGET).join(profile).join(WASM_NAME);
    let out = root.join("dist/web");
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    run_cmd(
        Command::new("wasm-bindgen")
            .args(["--target", "web", "--no-typescript", "--out-dir"])
            .arg(&out)
            .arg(&wasm),
    )?;
    for entry in std::fs::read_dir(root.join("web")).map_err(|e| format!("web/: {e}"))? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_file() {
            std::fs::copy(&path, out.join(path.file_name().unwrap())).map_err(|e| e.to_string())?;
        }
    }
    println!("built {} ({profile})", workspace::rel(&out));
    Ok(out)
}

/// `cargo xtask web [--release] [--serve] [--port N]`.
pub fn run(args: &[String]) -> Result<(), String> {
    let mut release = false;
    let mut serve_it = false;
    let mut port: u16 = 8000;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--release" => release = true,
            "--serve" => serve_it = true,
            "--port" => {
                port = it
                    .next()
                    .and_then(|p| p.parse().ok())
                    .ok_or("--port needs a number")?;
            }
            other => return Err(format!("web: unknown argument {other:?}")),
        }
    }
    let out = build(release)?;
    if serve_it {
        serve(&out, port)?;
    }
    Ok(())
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    }
}

/// A minimal static file server for local testing: GET only, loopback only,
/// no caching.
pub fn serve(dir: &Path, port: u16) -> Result<(), String> {
    let listener =
        TcpListener::bind(("127.0.0.1", port)).map_err(|e| format!("port {port}: {e}"))?;
    println!(
        "serving {} at http://127.0.0.1:{port}/ (Ctrl-C to stop)",
        workspace::rel(dir)
    );
    println!("force a backend with ?backend=webgl2 or ?backend=webgpu");
    for stream in listener.incoming().flatten() {
        if let Err(e) = respond(dir, stream) {
            eprintln!("request failed: {e}");
        }
    }
    Ok(())
}

fn respond(dir: &Path, mut stream: TcpStream) -> std::io::Result<()> {
    let mut request_line = String::new();
    let mut reader = BufReader::new(stream.try_clone()?);
    reader.read_line(&mut request_line)?;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header)? == 0 || header == "\r\n" || header == "\n" {
            break;
        }
    }
    let mut parts = request_line.split_whitespace();
    let (method, target) = (parts.next().unwrap_or(""), parts.next().unwrap_or("/"));
    let path = target.split(['?', '#']).next().unwrap_or("/");
    let rel = if path == "/" {
        "index.html"
    } else {
        path.trim_start_matches('/')
    };
    let file = dir.join(rel);
    let safe = !rel.split('/').any(|seg| seg == ".." || seg.is_empty());
    let (status, body, ctype) = match (method, safe, std::fs::read(&file)) {
        ("GET" | "HEAD", true, Ok(body)) => ("200 OK", body, content_type(&file)),
        ("GET" | "HEAD", _, _) => ("404 Not Found", b"not found\n".to_vec(), "text/plain"),
        _ => (
            "405 Method Not Allowed",
            b"GET only\n".to_vec(),
            "text/plain",
        ),
    };
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    if method != "HEAD" {
        stream.write_all(&body)?;
    }
    stream.flush()
}
