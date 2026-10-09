# Development

How the workspace is laid out, which checks guard it, and how CI runs them. Read [CLAUDE.md](../CLAUDE.md), [PROVENANCE.md](../PROVENANCE.md) and [CONTRIBUTING.md](../CONTRIBUTING.md) first.

## Workspace

Rust stable (pinned in `rust-toolchain.toml`), one Cargo workspace. Package names are prefixed `glasnost-`.

| Crate | Path | Filled in | Role |
|---|---|---|---|
| `fmt` | `crates/fmt` | M0 scaffolding, M1 parsers | File-format parsers, each with a fixture writer; indexed-image model; synthetic generators; content sniffing |
| `vfs` | `crates/vfs` | M1 | Archive mounting (nested archives included), lookup by name hash in a pluggable precedence order, lazy reads |
| `ini` | `crates/ini` | M1 | INI reader with the original's parse semantics, for the importer only |
| `convert` | `crates/convert` | M1–M2 | Import: INI layers → our rules tree, `coverage.csv`, maps |
| `rules` | `crates/deterministic/rules` | M0 JSONC reader, M1 schema | Our rules format, resolve pipeline, `sim_rules_hash` |
| `map` | `crates/deterministic/map` | M2–M3 | Typed map for the simulation |
| `sim` | `crates/deterministic/sim` | M3–M7 | The deterministic simulation |
| `ai` | `crates/deterministic/ai` | M5+ | Our own opponent; produces Orders only |
| `render` | `crates/render` | M0 quad, M1+ | wgpu renderer: WebGL2 core path, WebGPU where available |
| `audio` | `crates/audio` | M1 preview, M5 | Sound playback (M1: viewer previews) |
| `ui` | `crates/ui` | M5 | Game UI |
| `net` | `crates/net` | M9 | Lockstep networking (the relay server gets its own crate in M9) |
| `app` | `crates/app` | M0 demo, M1 viewer | winit shell shared by native and web; the asset viewer |
| `platform-native` | `crates/platform-native` | M0 | Native binary `glasnost` |
| `platform-web` | `crates/platform-web` | M0 | wasm entry point |
| `tools` | `crates/tools` | M0 fixtures, M1 local lane | `glasnost-fixtures`, `glasnost-local-lane`; later headless runner, sync-diff |
| `xtask` | `xtask` | M0 | Repository checks and build helpers (`cargo xtask help`) |

`app` is not in PLAN.md section 4.2: it holds the event loop and rendering setup that both platform crates share, so native and web run the same code.

### Simulation crates

Every crate under `crates/deterministic/` is a simulation crate. The checks find them by location, so a new crate placed there is covered automatically. Simulation crates:

- carry their own `[lints]` table (identical across them) instead of the workspace one;
- pick up `crates/deterministic/clippy.toml`, which clippy finds by searching upwards from each crate;
- may depend only on each other and on external crates outside the deny list in `xtask/src/deps.rs`.

## Checks

`cargo xtask check` runs all of the following; each also runs on its own (`cargo xtask deps`, etc.).

### Dependency rule (`deps`)

The `sim` crate (and every other simulation crate) must not depend, directly or transitively, on any non-simulation workspace crate (`render`, `audio`, `net`, `fmt`, `ini`, `convert`, `vfs`, `ui`, `app`, the platform crates, `tools`). It also must not reach graphics, browser bindings, host randomness, hash-seeded maps, threads or clocks (`wgpu`, `winit`, `web-sys`, `getrandom`, `rand`, `ahash`, `rayon`, `tokio`, `chrono`, ...). The check reads `cargo metadata` for every target platform and counts normal and build dependencies. Dev-dependencies don't count. The rule also runs as a unit test in `xtask`, so plain `cargo test` enforces it.

### Determinism (`determinism`, `canary`, clippy)

- **Clippy** bans `f32`/`f64`, `HashMap`/`HashSet`, host-seeded hashers, `Instant`/`SystemTime`, threads and locks, file and network I/O, the environment, and printing macros. It also denies float arithmetic, float comparison, lossy float literals and precision-losing casts. See `crates/deterministic/clippy.toml` and the lint table in each simulation crate's `Cargo.toml`.
- **Float-literal scan.** `let x = 1.5;` uses no float type by name, so clippy alone misses it. `cargo xtask determinism` rejects float literals in simulation code and checks that the simulation lint tables are identical.
- **Canary.** `cargo xtask canary` builds a throwaway crate inside `crates/deterministic/` with one violation per configured entry and requires every expected diagnostic. A misspelt or silently ignored entry fails here instead of letting a float through.
- The narrow float exception from PLAN.md 4.5 (a `Real` wrapper) will be allowed site by site with `#[allow]`, a spec citation and an allowlist entry when a spec requires it.

### Citations (`cite`)

In non-test code of the simulation crates:

- every numeric literal other than `0` and `1`, and
- every `const` and `static` item

must be covered by `// spec: S-xxxx` or `// measure: M-xxxx`. The comment can sit on the same line, or in the comment and attribute block directly above (or opening) an enclosing item, statement, field, variant or match arm. A blank line ends the block.

```rust
// spec: S-0042
const RELOAD_RESETS_ON_MOVE: bool = true;

fn reload_ticks(base: u32) -> u32 {
    base * 15 // measure: M-0007
}
```

- `// nocite: <reason>` in the same places marks a literal that is not behaviour, such as a buffer size of our own design. The reason is required.
- `xtask/citation-allowlist.txt` exempts whole files, again with a reason. It holds the JSONC parser, whose literals are syntax. Stale entries fail the check.
- Every `S-`, `M-` and `Q-` ID mentioned in `crates/**/*.rs` and `data/**/*.jsonc` must exist as a file in `spec/`, `measurements/` or `spec/requests/`, and a cited spec must not be retired.

`spec/` is empty at M0, so the simulation crates hold no behavioural constants yet. Anything that needs one waits for its spec or goes to `spec/requests/`.

### No stock numbers (`stock-guard`)

Stock values can't be recognised without knowing them, and this repository must never know them. So the guard enforces the structure that keeps them out. It covers every file git tracks or would add:

1. **Game-format files** (`.mix`, `.shp`, `.vxl`, `.pal`, `.csf`, `.ini`, `.map`, ... by extension) may only live in fixture directories:
   - In `tests/fixtures/synthetic/` they must match, byte for byte, what our writers generate (`cargo run -p glasnost-tools --bin glasnost-fixtures`), and `FIXTURES.manifest` must be current.
   - In any other `tests/fixtures/` directory they must be text with a `glasnost-synthetic:` line near the top.
2. **Synthetic data** (`data/synthetic/**`) starts with `SPDX-License-Identifier: CC0-1.0`, followed by a `glasnost-synthetic:` line.
3. **Measurements:** nothing under `measurements/local/` is tracked. Records with `value-visibility: local` show no numbers in the value and spread columns of their Result table.
4. **Paths:** no install paths (Steam libraries, Windows program folders, Westwood or EA folders) or home-directory paths in text files. Use `~` or a placeholder. `LOCAL.md` and `.claude/settings.local.json` stay untracked.
5. **Size:** no file over 1 MiB.

Tests that need the user's real install run in a local-only lane (opt-in, never in CI; see "Local-only lane" below); their inputs and outputs are git-ignored. A local comparison of committed synthetic data against the user's imported values needs the converter's (field, value) output. It arrives with the converter in M1 and will run only on the developer's machine.

### Synthetic ruleset

`data/synthetic/ruleset/` is a small, invented ruleset in our JSONC format (CC0). It exercises the format's shape before the schema exists:

- objects keyed by instance name, `inherits` lists and `^abstract` templates;
- `{"+": [...]}` set operations;
- integers in native units, and decimals as quoted strings with a unit.

`crates/deterministic/rules/tests/synthetic_ruleset.rs` checks that every file parses (so every number is an integer), carries the header and is listed in `manifest.jsonc`, that all references resolve, and that inheritance has no cycles. In M1 the schema crate takes over these checks.

### Fixture writers

`glasnost_fmt::fixture` defines `FixtureWriter` (one synthetic file), the registry, the manifest and `check_round_trip` (encode, decode, compare, re-encode). Each format module brings its writers and registers them. `glasnost-fixtures` writes the registry's output to `tests/fixtures/synthetic/`, and the stock guard checks the committed copy byte for byte: after changing a writer, regenerate with `cargo run -p glasnost-tools --bin glasnost-fixtures` and commit the result.

Encrypted MIX fixtures use a key pair of our own (`RsaKeyPair::synthetic`), so they parse only with that key's public half, not with the Westwood key.

### Hostile input and fuzzing

Every parser in `fmt` is panic-free on hostile input: reads are bounds-checked, input-driven arithmetic is checked, allocations are bounded by the input or by `fmt::limits`, and the crate denies `unwrap`, `expect` and `panic` outside tests. `fmt::hostile::run` drives one parser and all its decoders over arbitrary bytes. Two users:

- `cargo test -p glasnost-fmt --test hostile` (stable, in CI): every fixture, its prefixes and mutated copies through every parser.
- `fuzz/` (cargo-fuzz, nightly, outside the workspace and CI): one target per parser; see [fuzz/README.md](../fuzz/README.md).

### Local-only lane

`GLASNOST_GAME_DIR=<install> cargo run --release -p glasnost-tools --bin glasnost-local-lane` mounts a real install and prints counts only: MIX headers decrypted, SHP files and frames (all archives and `conqmd.mix`), VXL files and limbs under `ra2.mix`/`ra2md.mix`, IDX samples, AUD, TMP, PAL, PCX, CSF, HVA and map packs, each by error kind. `cargo test -p glasnost-tools --test local_lane` asserts 100 % on the M1 checks when the variable is set and skips otherwise; CI never sets it. The lane opens only the MIX containers at the top of the folder (`.mix`, `.mmx`, `.yro`); every other file there is counted and never read. It classifies entries with the permissive sniffer (`fmt::sniff`) and counts the strict parsers against that, so a file the sniffer calls SHP but the parser rejects shows up as a failure.

`cargo test --release -p glasnost-app snapshot -- --ignored` (with the same variable) writes viewer screenshots of a real install to the temp directory. Never commit them.

### Provisional placeholders

Behaviour that needs a spec stays out of code; where a spec leaves a gap, the code carries an explicit placeholder tied to a spec request:

| Where | Placeholder | Request |
|---|---|---|
| `vfs::precedence` | lookup order = mount order (top-level archives by name, nested ones after their parent) | Q-0002 |

Q-0001 and Q-0003 are answered by S-0103: `render::palette` loads palette files (×4 in 8-bit arithmetic), keeps the theater palettes in force, builds house-colour palettes from colour definitions and picks each object's palette. Shading by light (S-0104) is not applied yet; the viewer shows unshaded palettes.

## Tests

```sh
cargo test --workspace          # unit tests, render tests, dependency rule
cargo xtask check               # the checks above
```

`crates/render/tests/offscreen_quad.rs` renders the palette quad on a real or software GPU and compares every pixel with the CPU palette lookup. Without an adapter it skips, unless `GLASNOST_REQUIRE_GPU=1` is set (CI sets it and installs lavapipe). `WGPU_BACKEND=gl` runs it on the native GL backend. `crates/render/tests/webgl2_shaders.rs` translates every shader to GLSL ES 3.00, so constructs WebGL2 can't express fail without a browser.

The browser lane is described in [web.md](web.md).

## CI

`.github/workflows/ci.yml` runs these jobs:

- rustfmt;
- clippy, native and wasm32;
- tests, with lavapipe and `GLASNOST_REQUIRE_GPU=1`;
- `cargo xtask check`;
- cargo-deny licence check against `deny.toml` (GPL-3.0-or-later compatible licences only);
- the release web build;
- the browser lanes:
  - WebGL2 on Chromium, Firefox and WebKit (WebKit non-blocking until it has run green);
  - optional WebGPU on Chromium.

CI uses synthetic data only.
