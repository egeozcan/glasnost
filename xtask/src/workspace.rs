//! Locating the repository and its crates.

use std::path::{Path, PathBuf};

use cargo_metadata::{Metadata, MetadataCommand};

/// Directory holding the simulation crates. Every workspace crate under it is
/// a simulation crate: the determinism lints, the citation lint and the
/// dependency rule apply to it.
pub const DETERMINISTIC_DIR: &str = "crates/deterministic";

/// The repository root (the parent of `xtask/`).
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one level below the repository root")
        .to_path_buf()
}

/// `cargo metadata` for the whole workspace, with the full dependency
/// resolve across all targets.
pub fn metadata() -> Result<Metadata, String> {
    MetadataCommand::new()
        .manifest_path(root().join("Cargo.toml"))
        .exec()
        .map_err(|e| format!("cargo metadata failed: {e}"))
}

/// A workspace crate: package name and crate directory.
pub type Member = (String, PathBuf);

/// Workspace members split into (simulation crates, other crates), each
/// sorted by name.
pub fn members(md: &Metadata) -> (Vec<Member>, Vec<Member>) {
    let det_root = root().join(DETERMINISTIC_DIR);
    let mut det = Vec::new();
    let mut other = Vec::new();
    for pkg in md.workspace_packages() {
        let dir = pkg
            .manifest_path
            .parent()
            .expect("manifest has a parent")
            .as_std_path()
            .to_path_buf();
        let entry = (pkg.name.to_string(), dir.clone());
        if dir.starts_with(&det_root) {
            det.push(entry);
        } else {
            other.push(entry);
        }
    }
    det.sort();
    other.sort();
    (det, other)
}

/// All files under `dir` (recursively) with the given extension, sorted.
pub fn files_with_extension(dir: &Path, ext: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().is_some_and(|n| n == "target") {
                    continue;
                }
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == ext) {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// `path` relative to the repository root, with `/` separators.
pub fn rel(path: &Path) -> String {
    path.strip_prefix(root())
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
