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

## Native demo

```sh
cargo run -p glasnost-platform-native                    # wgpu's default backend
cargo run -p glasnost-platform-native -- --backend gl    # OpenGL
cargo run -p glasnost-platform-native -- --frames 120    # close after 120 frames
```

## Browser lane (tests)

`web-tests/smoke.mjs` serves `dist/web`, opens it in a Playwright browser with a forced backend, waits for the app to report its first presented frame (`data-glasnost-status` on `<body>`), and checks the canvas pixels. Chromium uses SwiftShader, so no GPU is needed.

```sh
cargo xtask web
cd web-tests
npm ci
npx playwright install chromium firefox     # once; add --with-deps on Debian/Ubuntu
node smoke.mjs --browser chromium --backend webgl2
node smoke.mjs --browser firefox --backend webgl2
node smoke.mjs --browser chromium --backend webgpu   # optional lane: forced WebGPU
node smoke.mjs --browser chromium --backend auto --webgpu --expect WebGPU   # runtime choice
```

`--webgpu` turns on Chromium's WebGPU flags; with `--backend auto` it checks that the app picks WebGPU when the browser offers it (and, without the flag, that it falls back to WebGL2).

Screenshots go to `web-tests/out/`. Browser and GPU recipes specific to your machine (a flagged Chromium for WebGPU, say) belong in `LOCAL.md`.
