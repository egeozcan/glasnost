//! Embeds the commit the build was made from (M9 N8b): `GLASNOST_COMMIT` is
//! the short hash, with `+dirty` when tracked files differ from it, or
//! `unknown` outside a git checkout. Never fails the build.

use std::path::PathBuf;
use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8(out.stdout).ok()?.trim().to_owned())
}

fn main() {
    // Re-run when the checked-out commit or the index changes. The paths
    // come from git, so a worktree (whose `.git` is a file) works too.
    for path in ["HEAD", "index", "refs"] {
        if let Some(p) = git(&["rev-parse", "--git-path", path]) {
            let p = PathBuf::from(p);
            if p.exists() {
                println!("cargo:rerun-if-changed={}", p.display());
            }
        }
    }
    println!("cargo:rerun-if-env-changed=GLASNOST_COMMIT");
    let commit = std::env::var("GLASNOST_COMMIT")
        .ok()
        .filter(|c| !c.is_empty())
        .or_else(|| {
            let hash = git(&["rev-parse", "--short=9", "HEAD"])?;
            // Untracked files (editor scratch, build output) do not count.
            let dirty = git(&["status", "--porcelain", "--untracked-files=no"])
                .is_some_and(|s| !s.is_empty());
            Some(if dirty { format!("{hash}+dirty") } else { hash })
        })
        .unwrap_or_else(|| "unknown".to_owned());
    println!("cargo:rustc-env=GLASNOST_COMMIT={commit}");
}
