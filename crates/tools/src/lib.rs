//! Developer tools.
//!
//! **Milestone M0**: `glasnost-fixtures`, which writes every synthetic fixture
//! from `glasnost_fmt::fixture::registry()` plus its manifest.
//! **M1** adds the local-only lane ([`local_lane`], binary
//! `glasnost-local-lane`); the asset viewer lives in `glasnost-app`. **M3**
//! adds the headless simulation runner ([`headless`], binary
//! `glasnost-headless`). **M7b** adds its arena mode ([`arena`], with the
//! statistics of the comparison in [`stats`]). **M9 N4** adds the desync
//! tool ([`desync`], `glasnost-headless desync`), with a field-by-field
//! view of any serializable value in [`inspect`]. **M9 N6** adds the
//! soak: a client that plays our AI over the relay ([`net_client`],
//! `glasnost-headless net-client`), a bad network between it and the
//! relay ([`netem`]) and the local 2v2 of four such clients ([`soak`],
//! `glasnost-headless soak`). **M8 A4** adds the pathing-report tool
//! ([`path_report`], `glasnost-headless path-report`).

pub mod arena;
pub mod desync;
pub mod headless;
pub mod inspect;
pub mod local_lane;
pub mod net_client;
pub mod netem;
pub mod oracle;
pub mod oracle_m0020;
pub mod oracle_m0021;
pub mod oracle_m0023;
pub mod oracle_trace;
pub mod path_report;
pub mod side_audit;
pub mod skirmish_lane;
pub mod soak;
pub mod stats;
pub mod tick_bench;
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
