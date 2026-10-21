//! Where a desync report goes (M9 N4): natively a file under
//! [`REPORT_DIR`] in the working directory; in the browser a download, by a
//! [`SaveReport`] function the web shell passes. The report itself
//! (`glasnost_net::desync::Bundle`) is made by the network game; this
//! module only stores its bytes. Our own netcode; no spec applies.

/// Stores a report's bytes under a file name; returns where they went (for
/// the notice) or why they could not be stored.
pub type SaveReport = fn(name: &str, bytes: &[u8]) -> Result<String, String>;

/// The folder, under the working directory, of the native client's reports.
// nocite: our own netcode
pub const REPORT_DIR: &str = "desync-reports";

/// The platform's own way to store a report, where it has one built in: a
/// file natively. The web shell passes its own (a download).
pub fn platform_save() -> Option<SaveReport> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Some(save_file)
    }
    #[cfg(target_arch = "wasm32")]
    {
        None
    }
}

/// Writes the report to `desync-reports/<name>` under the working
/// directory, making the folder if need be.
#[cfg(not(target_arch = "wasm32"))]
pub fn save_file(name: &str, bytes: &[u8]) -> Result<String, String> {
    let dir = std::path::Path::new(REPORT_DIR);
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot make {}: {e}", dir.display()))?;
    let path = dir.join(name);
    std::fs::write(&path, bytes).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(path.display().to_string())
}
