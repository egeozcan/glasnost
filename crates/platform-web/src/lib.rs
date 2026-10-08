//! Browser entry point, built for `wasm32-unknown-unknown` and bound with
//! `wasm-bindgen` (see docs/web.md).
//!
//! **Milestone M0**: starts the palette-quad demo. The `backend` query
//! parameter (`?backend=webgl2` or `?backend=webgpu`) overrides the runtime
//! choice; the default picks WebGPU where the browser offers it and WebGL2
//! otherwise. **M1**: `?mode=viewer` starts the asset viewer with its
//! folder picker. **M5** moves the simulation into a worker on its own
//! clock.
//!
//! On native targets this crate is empty so that the whole workspace builds
//! and tests with plain `cargo`.

#[cfg(target_arch = "wasm32")]
mod web {
    use glasnost_app::{DemoOptions, Status, ViewerOptions};
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(start)]
    pub fn start() {
        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(log::Level::Info);

        let params = web_sys::window()
            .and_then(|w| w.location().search().ok())
            .and_then(|q| web_sys::UrlSearchParams::new_with_str(&q).ok());
        let param = |name: &str| {
            params
                .as_ref()
                .and_then(|p| p.get(name))
                .unwrap_or_default()
        };
        let backend = param("backend");
        let backend = match backend.parse() {
            Ok(b) => b,
            Err(e) => {
                Status::Error(e).publish();
                return;
            }
        };
        let result = if param("mode") == "viewer" {
            glasnost_app::run_viewer(ViewerOptions {
                backend,
                game_dir: None,
                exit_after_frames: None,
            })
        } else {
            glasnost_app::run_demo(DemoOptions {
                backend,
                exit_after_frames: None,
            })
        };
        if let Err(e) = result {
            Status::Error(e.to_string()).publish();
        }
    }
}
