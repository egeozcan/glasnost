# Web build

The web build is the same code as the native one: `glasnost-app` holds the winit event loop and wgpu setup, `glasnost-platform-native` wraps it in a binary, and `glasnost-platform-web` wraps it in a `wasm32-unknown-unknown` library bound with `wasm-bindgen`.

## Backends

The renderer's core path stays within the WebGL2 feature set (PLAN.md 4.1). On the web the app asks the browser for a working WebGPU adapter first and falls back to WebGL2. The page reports which backend it got. Add a query parameter to force one:

| URL | Backend |
|---|---|
| `/` | WebGPU where the browser offers a working adapter, else WebGL2 |
| `/?backend=webgl2` | WebGL2 only |
| `/?backend=webgpu` | WebGPU only (fails where unavailable) |

Natively, `glasnost --backend gl` uses the OpenGL backend, which is the closest stand-in for the WebGL2 path, and `--backend vulkan` uses the platform's primary API. The `WGPU_BACKEND` environment variable also works.

## Build and serve locally

One-time setup: the wasm target (installed by `rust-toolchain.toml`) and `wasm-bindgen-cli` at **exactly** the version in `Cargo.lock`:

```sh
cargo install wasm-bindgen-cli --version 0.2.129 --locked
```

`cargo xtask web` checks the version and prints the right install command if they differ.

Build and serve:

```sh
cargo xtask web --serve            # debug build, http://127.0.0.1:8000/
cargo xtask web --release --serve  # optimised build
cargo xtask web --serve --port 9000
```

The output goes to `dist/web/` (git-ignored): `index.html` from `web/`, plus the bound JavaScript and wasm. Any static file server works; the build needs no special headers, because v1 uses no threads and so no COOP/COEP.

The page shows a palette-indexed checkerboard: an R8 index texture resolved through a 256×N palette texture in the shader, with palette cycling. Space switches the palette row. The top-left label shows the backend in use.

## Asset viewer

`/?mode=viewer` opens the asset viewer instead of the demo. **Pick install folder…** opens the browser's folder dialog (`<input webkitdirectory>`). Only the MIX containers at the top of the folder are mounted, and they are read lazily with `File.slice`, so nothing is copied or uploaded. Natively: `cargo run -p glasnost-platform-native -- --viewer [--game-dir DIR]`.

The viewer lists archives (nested ones included) and their entries, previews sprites, tiles, images, palettes, string tables and a simple voxel view with a chosen palette and house-colour rows, and plays IDX samples and AUD sounds. Names come from text entries in the install, or type a file name under **Find file**. The remap range and the palette expansion are provisional settings (spec requests Q-0001 and Q-0003). Natively, sound previews go through a command-line player (`pw-play`, `paplay`, `aplay` or `afplay`). In-game sound goes through `glasnost-audio`'s mixer instead (docs/development.md, "Audio (M5b)").

## Game client

`/?mode=game` opens the game client (M5b): main menu, skirmish setup, a game against our AI, and the end screen (docs/development.md, "Game client (M5b)"). **Game folder** on the main menu opens the same folder dialog as the viewer. The install is mounted lazily as above, then the rules, maps and samples are read. **Skirmish** is offered once that is done. Sound starts on the first click or key press (browsers only allow audio after a gesture). Music (M8 C1) plays in a skirmish; each track read is kept in the browser's origin-private file system (the music share, folder `glasnost-music`) and read from there on later visits, with the picked file as the fallback when storage is refused, full or evicted (docs/development.md, "Music (M8 step C1 part 1)"). The battlefield is drawn on the GPU into a texture that the 2D batch shows. Everything stays within the WebGL2 core. There is no Quit button on the web.

**Multiplayer** (M9 N3) on the main menu reaches a relay through the browser's WebSocket (docs/development.md, "The client (N3)"). `?relay=ws://HOST:PORT/` fills in the relay address; without it the address is `ws://127.0.0.1:8787/`. To try it locally, run `cargo run -p glasnost-relay` and `cargo xtask web --serve`, then open `/?mode=game&relay=ws://127.0.0.1:8787/` in two tabs. A page served over `https://` needs a `wss://` relay. Since M9 N7 a network game also tries direct links between the players through the browser's `RTCPeerConnection`, beside the relay (docs/development.md, "WebRTC (N7)"): `?direct=off` keeps the game on the relay alone, and `?ice=stun:HOST:PORT` (space-separated URLs, with `username=` and `credential=` words for a TURN server) names ICE servers, of which there are none by default. `node web-tests/rtc.mjs --browser chromium|firefox` checks the browser's direct link against a `cargo xtask web` build.

**Benchmark** (M8 C4): `/?mode=game&bench=default` (or a spec of `key=value` words, docs/development.md, "Performance (M8 C4)") runs the performance benchmark once the install is picked with **Game folder**, at the page's size, and offers `web.txt` and `web.csv` as downloads at the end. Use a release build (`cargo xtask web --release --serve`). `node web-tests/bench.mjs --install DIR --browser firefox --gpu --headed` does the same in a Playwright browser (headed: a headless Firefox may have no WebGL) and writes the outputs under `local-lane/m8/c4/web/`.

Natively:

```sh
cargo run --release -p glasnost-platform-native -- --game --game-dir DIR
```

## Native demo

```sh
cargo run -p glasnost-platform-native                    # wgpu's default backend
cargo run -p glasnost-platform-native -- --backend gl    # OpenGL
cargo run -p glasnost-platform-native -- --frames 120    # close after 120 frames
```

## Browser lane (tests)

`web-tests/smoke.mjs` serves `dist/web`, opens it in a Playwright browser with a forced backend, waits for the app to report its first presented frame (`data-glasnost-status` on `<body>`), and checks the canvas pixels. Chromium uses SwiftShader, so no GPU is needed. Firefox needs `--headed`: a headless Playwright Firefox creates no WebGL context, while a headed one gets Mesa's software renderer (llvmpipe) when there is no GPU, so CI runs it under `xvfb-run -a`.

```sh
cargo xtask web
cd web-tests
npm ci
npx playwright install chromium firefox     # once; add --with-deps on Debian/Ubuntu
node smoke.mjs --browser chromium --backend webgl2
node smoke.mjs --browser firefox --backend webgl2 --headed   # under xvfb-run -a without a display
node smoke.mjs --browser chromium --backend webgpu   # optional lane: forced WebGPU
node smoke.mjs --browser chromium --backend auto --webgpu --expect WebGPU   # runtime choice
node smoke.mjs --browser chromium --backend webgl2 --viewer   # viewer: imports the synthetic MIX fixtures
node smoke.mjs --browser chromium --backend webgl2 --game     # game client: its main menu draws
```

`--viewer` loads `?mode=viewer`, gives the folder input the synthetic MIX fixtures with `setInputFiles` and waits for the viewer to report the mount on `<body>` (`data-glasnost-mount`).

`--webgpu` turns on Chromium's WebGPU flags; with `--backend auto` it checks that the app picks WebGPU when the browser offers it (and, without the flag, that it falls back to WebGL2).

Screenshots go to `web-tests/out/`. Browser and GPU recipes specific to your machine (a flagged Chromium for WebGPU, say) belong in `LOCAL.md`.
