//! Browser entry point, built for `wasm32-unknown-unknown` and bound with
//! `wasm-bindgen` (see docs/web.md).
//!
//! **Milestone M0**: starts the palette-quad demo. The `backend` query
//! parameter (`?backend=webgl2` or `?backend=webgpu`) overrides the runtime
//! choice; the default picks WebGPU where the browser offers it and WebGL2
//! otherwise. **M1**: `?mode=viewer` starts the asset viewer with its
//! folder picker. **M5b**: `?mode=game` starts the game client (menu,
//! skirmish setup, a game against the AI, end screen) with the same folder
//! picker. **M9 N3**: the game client's multiplayer screen reaches a relay
//! through the browser's WebSocket ([`socket`]); `?relay=ws://HOST:PORT/`
//! fills its relay address. **M9 N7**: network games also try direct
//! links between the peers through the browser's `RTCPeerConnection`
//! ([`rtc`]); `?direct=off` keeps a game on the relay only, and
//! `?ice=URL[ URL…]` names STUN or TURN servers (none by default).
//!
//! The page's entry point is the default feature `start`; without it (the
//! soak's wasm client, `glasnost-headless-web`, M9 N6) the crate is only its
//! WebSocket link and its report download.
//!
//! On native targets this crate is empty so that the whole workspace builds
//! and tests with plain `cargo`.

#[cfg(target_arch = "wasm32")]
pub mod download;
#[cfg(target_arch = "wasm32")]
pub mod rtc;
#[cfg(target_arch = "wasm32")]
pub mod socket;

#[cfg(all(target_arch = "wasm32", feature = "start"))]
mod web {
    use glasnost_app::{DemoOptions, GameOptions, Status, ViewerOptions};
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
        let mode = param("mode");
        // The browser lane's page for `rtc::rtc_probe`: nothing runs.
        if mode == "probe" {
            return;
        }
        // `?bench=SPEC` runs the performance benchmark (M8 step C4) once
        // the game folder is picked.
        let bench = match Some(param("bench")).filter(|b| !b.is_empty()) {
            Some(spec) => match glasnost_app::game::bench::BenchOptions::parse(&spec) {
                Ok(b) => Some(b),
                Err(e) => {
                    Status::Error(e).publish();
                    return;
                }
            },
            None => None,
        };
        let result = if mode == "game" {
            glasnost_app::run_game(GameOptions {
                backend,
                game_dir: None,
                exit_after_frames: None,
                relay: Some(param("relay")).filter(|r| !r.is_empty()),
                connect: Some(crate::socket::connect),
                direct: Some(crate::rtc::open),
                relay_only: param("direct") == "off",
                ice: Some(param("ice")).filter(|i| !i.trim().is_empty()),
                report: Some(crate::download::save),
                test: None,
                bench,
                // `?probe=1`: the browser check of touch scrolling reads the view.
                probe: param("probe") == "1",
            })
        } else if mode == "viewer" {
            glasnost_app::run_viewer(ViewerOptions {
                backend,
                game_dir: None,
                exit_after_frames: None,
                // `?mode=viewer&map=NAME` opens the map viewer once an install is picked.
                open_map: Some(param("map")).filter(|m| !m.is_empty()),
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
