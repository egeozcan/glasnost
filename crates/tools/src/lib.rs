//! Developer tools.
//!
//! **Milestone M0**: `glasnost-fixtures`, which writes every synthetic fixture
//! from `glasnost_fmt::fixture::registry()` plus its manifest.
//! **M1** adds the local-only lane ([`local_lane`], binary
//! `glasnost-local-lane`); the asset viewer lives in `glasnost-app`. **M3**
//! adds the headless simulation runner ([`headless`], binary
//! `glasnost-headless`).

pub mod headless;
pub mod local_lane;
pub mod skirmish_lane;
pub mod trigger_lane;

use std::io;
use std::path::Path;

use glasnost_fmt::fixture::{self, MANIFEST_FILE};

/// Default output directory for committed synthetic fixtures, relative to the
/// repository root. `cargo xtask check` regenerates and compares it.
pub const DEFAULT_FIXTURE_DIR: &str = "tests/fixtures/synthetic";

/// Writes all registered fixtures and the manifest under `root`. Returns the
/// number of fixture files written.
pub fn write_fixtures(root: &Path) -> io::Result<usize> {
    let fixtures = fixture::generate(&fixture::registry())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    for f in &fixtures {
        let path = root.join(&f.path);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, &f.bytes)?;
    }
    std::fs::create_dir_all(root)?;
    std::fs::write(root.join(MANIFEST_FILE), fixture::manifest(&fixtures))?;
    Ok(fixtures.len())
}
