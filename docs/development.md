# Development

How the workspace is laid out, which checks guard it, and how CI runs them. Read [CLAUDE.md](../CLAUDE.md), [PROVENANCE.md](../PROVENANCE.md) and [CONTRIBUTING.md](../CONTRIBUTING.md) first.

## Workspace

Rust stable (pinned in `rust-toolchain.toml`), one Cargo workspace. Package names are prefixed `glasnost-`.

| Crate | Path | Filled in | Role |
|---|---|---|---|
| `fmt` | `crates/fmt` | M0 scaffolding, M1 parsers | File-format parsers, each with a fixture writer; indexed-image model; synthetic generators; content sniffing |
| `vfs` | `crates/vfs` | M1 | Archive mounting (nested archives included), the game's search order (S-0005), lazy reads |
| `ini` | `crates/ini` | M1 | INI reader with the original's parse semantics (S-0001 to S-0003), for the importer only |
| `convert` | `crates/convert` | M1–M2 | Import: INI layers → our rules tree (S-0004 to S-0007), `coverage.csv`; maps in M2 |
| `rules` | `crates/deterministic/rules` | M0 JSONC reader, M1 schema | Our rules format (schema 1), resolve pipeline, `rules explain`, `sim_rules_hash` |
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
| `tools` | `crates/tools` | M0 fixtures, M1 local lane, M3 runner | `glasnost-fixtures`, `glasnost-local-lane`, `glasnost-headless`; later sync-diff |
| `sim-gate` | `crates/sim-gate` | M3 | The determinism gate: per-tick state hashes of a synthetic replay, native and wasm32 |
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

`data/synthetic/ruleset/` is a small, invented ruleset in our JSONC format (CC0). It exercises the format's shape:

- objects keyed by instance name, `inherits` lists and `^abstract` templates;
- `{"+": [...]}` set operations;
- integers in native units, and decimals as quoted strings with a unit.

`crates/deterministic/rules/tests/synthetic_ruleset.rs` checks that every file parses (so every number is an integer), carries the header and is listed in `manifest.jsonc`, and runs the layers through the resolve pipeline: inheritance, set operations, schema 1 (strict, `deny_unknown_fields`), validation, ID interning, the rules hash and `rules explain`. It also checks that `data/schema/ruleset.schema.json`, the JSON Schema generated from the schema types with schemars, is current (`GLASNOST_UPDATE_SCHEMA=1` rewrites it).

### Fixture writers

`glasnost_fmt::fixture` defines `FixtureWriter` (one synthetic file), the registry, the manifest and `check_round_trip` (encode, decode, compare, re-encode). Each format module brings its writers and registers them. `glasnost-fixtures` writes the registry's output to `tests/fixtures/synthetic/`, and the stock guard checks the committed copy byte for byte: after changing a writer, regenerate with `cargo run -p glasnost-tools --bin glasnost-fixtures` and commit the result.

Encrypted MIX fixtures use a key pair of our own (`RsaKeyPair::synthetic`), so they parse only with that key's public half, not with the Westwood key.

### Hostile input and fuzzing

Every parser in `fmt` is panic-free on hostile input: reads are bounds-checked, input-driven arithmetic is checked, allocations are bounded by the input or by `fmt::limits`, and the crate denies `unwrap`, `expect` and `panic` outside tests. `fmt::hostile::run` drives one parser and all its decoders over arbitrary bytes; `ini::hostile::run` reads a document and every stored value in every value form; `rules::hostile::run` parses JSONC, writes it back and resolves it. Two users each:

- `cargo test -p glasnost-fmt --test hostile`, `-p glasnost-ini --test hostile` and `-p glasnost-rules --test hostile` (stable, in CI): synthetic inputs, their prefixes and mutated copies.
- `fuzz/` (cargo-fuzz, nightly, outside the workspace and CI): one target per parser, plus `ini` and `jsonc`; see [fuzz/README.md](../fuzz/README.md).

### Local-only lane

`GLASNOST_GAME_DIR=<install> cargo run --release -p glasnost-tools --bin glasnost-local-lane` mounts a real install and prints counts only: MIX headers decrypted, SHP files and frames (all archives and `conqmd.mix`), VXL files and limbs under `ra2.mix`/`ra2md.mix`, IDX samples, AUD, TMP, PAL, PCX, CSF, HVA and map packs, each by error kind. `cargo test -p glasnost-tools --test local_lane` asserts 100 % on the M1 checks when the variable is set and skips otherwise; CI never sets it. The lane opens only the MIX containers at the top of the folder (`.mix`, `.mmx`, `.yro`); every other file there is counted and never read. It classifies entries with the permissive sniffer (`fmt::sniff`) and counts the strict parsers against that, so a file the sniffer calls SHP but the parser rejects shows up as a failure.

The lane also builds the game's search order (S-0005) and checks the stock outcome (the expansion archive's rules file is the one used, every required archive resolves), imports the rules and art (temperate theater), checks that the result resolves through the rules pipeline, and counts types per kind, importer warnings, art found per kind and coverage outcomes. The binary writes `coverage.csv` (key names and outcomes, no values) to `local-lane/` in the current directory, which git ignores; never commit it.

`cargo test --release -p glasnost-app snapshot -- --ignored` (with the same variable) writes viewer screenshots of a real install to the temp directory, including one of the rules view. Never commit them.

The map lane (M2): the local lane above also decodes every map it finds (`maps decoded (S-0009), no panic`, with importer warnings by kind). `GLASNOST_GAME_DIR=<install> cargo test --release -p glasnost-app --test map_lane -- --nocapture` finds every map by content, loads each with its rules (the map as the map layer), builds its scene, times the build, a CPU redraw of a 1280×720 view and 120 scrolling GPU redraws, counts provisional notes and dropped objects, and writes one screenshot per theater (plus a minimap and a close-up of the first unit) to `local-lane/`. `GLASNOST_MAP_LIMIT=n` limits the maps built, `GLASNOST_MAP_SHOTS=n` the screenshots per theater, and `WGPU_BACKEND=gl` times the GL backend.

### Provisional placeholders

Behaviour that needs a spec stays out of code; where a spec leaves a gap, the code carries an explicit placeholder tied to a spec request (see the table in "Import" below).

Q-0002 is answered by S-0005: `vfs::game` builds the game's search order by name (Table 1), honours loose files for the archives it mounts, finds members by the signed index bisection and mounts the theater and side groups per game. Loose *data* files are an opt-in mod layer (PLAN.md 4.3); with it off, a loose file that would have won is reported and never read. Archives outside the search order (for example the base game's map archives) stay browsable in the viewer but never answer a lookup.

Q-0001 and Q-0003 are answered by S-0103: `render::palette` loads palette files (×4 in 8-bit arithmetic), keeps the theater palettes in force, builds house-colour palettes from colour definitions and picks each object's palette. Shading by light (S-0104) is not applied yet; the viewer shows unshaded palettes.

## Import

The importer turns the user's install into our rules format (PLAN.md 4.3, 4.6):

1. `vfs` mounts the folder for browsing and builds the game's search order (S-0005). Loose data files are an opt-in mod layer (`GameOptions::mod_layer`, off by default, PLAN.md 4.3); with it off, a loose file that would have won is reported and never read.
2. `convert::inputs` resolves the base rules, a language rules file with a valid digest, the game-mode file the user names and the map; it reports the databases without which the original refuses to start.
3. `ini` reads each file (S-0001 to S-0003) with importer warnings.
4. `convert::layers` applies the rules layers of one game (S-0004, S-0006): registration in list order with the rank rule for references, sides, tiberiums, per-key layering with its exceptions, colour definitions (S-0103 claim 5) and the colour-adjustment table.
5. `convert::art` looks up every type's art for one theater (S-0007), with the game's per-game archives mounted.
6. `convert::emit` writes a schema-1 tree: names are byte-exact (bytes 128 to 255 map to U+F780 to U+F7FF), keys no spec maps yet stay as INI text, values the layering computes (light values in thousandths, S-0104 claim 12) replace their text (`derived`, `ini_globals_derived`), and every value carries its INI history for `rules explain`. Import errors cover the conditions under which the original would not start, and loose names that differ only in letter case (S-0005 Edge cases).
7. `convert::sim` writes the typed `sim` section (`rules::sim`): the simulation fields of the rules and art data, each written only when the layered data gives it, folded over its history (values a form cannot read and "not given" values keep the held value), with decimals as text in our grammar that reads as S-0003 reads the stored text (a test cross-checks `num::decimal` against `ini::value::decimal`). Land-type rows take only the keys of the last layer that gives the section (S-0211 claim 2), the anti-air rate follows S-0215 claim 21, locomotion classes S-0211 claim 3, the locomotor field S-0207 claim 14 (a class identifier of table T3; another form or an unknown identifier keeps the locomotor in force, with a warning), and the art fields (infantry sequences, fire frames, `Crawls`, building foundations, overlay `has_art`, an overlay's `DamageLevels` from its image section, S-0210 claim 13) are rebuilt once the art lookup has run. Animation types take the fields their creation draws read from their art sections (`RandomRate`, the bouncing flags, the chain reaction and the middle-event effects, S-0109 Import mapping). Type-name lists and references (a type's `Owner`, `[General] BaseUnit`, the gate and wall tower types of S-0112) are resolved to type keys by S-0003 claim 14's comparison, unknown names dropped with a warning; the game's tech level (`[MultiplayerDialogSettings] TechLevel`) and the early phase's passive field (`multiplay_passive_base`, S-0009 claim 33) take the base rules' value only.
   The combat fields (M4, `convert::sim::combat`, S-0301 to S-0315) follow each combat spec's Import mapping: weapons (rate of fire, burst, speed, the lepton-form ranges as decimal text, the effect and choice flags), projectiles (flags, acceleration, arm time, cluster, trailer, art `Voxel`), warheads (`Verses` as eleven exact decimal expansions of S-0003 claim 13's doubles, spread, falloff, the damage rows, debris), the shared `combat` record of infantry, vehicle, aircraft and building types (armour class by S-0305's words, ammunition and reload, immunities, debris and death fields, targeting flags, the five threat coefficients, art muzzle positions and turret offset), the weapon slots of S-0308 claim 1 (both forms, with the clear-weapons flag applied at each reading of the section), the vehicle burst delays and firing frames, the building art fields (aim shift, height, voxel turret, pixel offsets, delayed fire), country factors and veteran lists, the three difficulty rows (`sim.difficulty`) and the game-wide combat values. Weapon `Speed`, the ranges, `AirRangeBonus`, `BallisticScatter` and `FireSupress` given as −1 count as not given. Type names in the new fields resolve to type keys where a type of that name exists and keep the name as written otherwise (the weapon and projectile registries are incomplete, Q-0005); fields the schema documents as type keys (veteran lists, `HarvesterUnit`, the crew types) drop unknown names with a warning. The importer warns for the bad-data cases of the combat specs' Edge cases. `Decimal::value` reads text at 24 bits (S-0003 claim 6), so a versus value that is not exactly a single needs a 53-bit reading of its text (`num::decimal::nearest_double`) to recover the double.
8. `rules::resolve_ruleset` applies mod patch layers, inheritance and set operations (only in our own collections, never inside imported INI data), types, validates, interns IDs and hashes.

Every quirk switch of the implemented specs has a field in `ini::Quirks` (including S-0003's `percent_list_integer_prefix`), `vfs::GameQuirks`, `convert::layers::LayerQuirks`, `convert::art::ArtQuirks`, `convert::sim::SimQuirks` (S-0308's `clear_weapons_any_turret_count`, S-0309's `coefficient_zero_replaced`), `convert::theater::TheaterQuirks`, `convert::map::MapQuirks` or `render::palette::PaletteQuirks`, defaulting to the original's behaviour; the import records them under `import.quirks`. The renderer's and the map viewer's switches are fields of `render::projection::ProjectionQuirks` (S-0101), `render::compose::ComposeQuirks` (S-0102 compositing), `render::light::LightQuirks` (S-0104), `render::voxel::VoxelQuirks` and `render::voxel::RecoilQuirks` (S-0105), `render::overlay::OverlayQuirks` (S-0112), `render::palette::PaletteQuirks` (S-0103) and `app::mapview::quirks::ViewQuirks` (S-0102 draw order and placement, S-0108 building frames), each with `named()`. `rules::quirks::tests::every_spec_quirk_has_a_switch` checks that every switch the simulation specs, S-0006, S-0009, S-0010 and S-0107 name is a field of `rules::Quirks` or listed with its owner in `EXTERNAL_QUIRKS`; `app::mapview::quirks::tests::every_rendering_spec_quirk_has_a_switch` checks that every switch `spec/rendering/S-01*.md` names is a field of one of the structs above (or of `rules::Quirks`, `LayerQuirks` or `ArtQuirks`) or is listed in `app::mapview::quirks::NOT_IMPLEMENTED` with its reason (the table below). Two S-0009 switches of `MapQuirks` are not consulted yet, because our engine has no lobby and draws no preview markers: `lobby_counts_zero_waypoint` and `preview_markers_from_header`. Four S-0006 switches of `LayerQuirks` are not consulted either, because the importer imports one game, the first of its session, without campaigns or the mode list (Q-0006): `persist_global_overrides`, `persist_tiberium_overrides`, `mission_rules_before_reset` and `mode_number_first_wins`. The importer warns about an infantry type whose die-1 or die-2 sequence entry has no image frames (S-0107 claim 17).

**Named switches of specs whose behaviour is not implemented yet.** These switches of the rendering specs have no field: the behaviour they switch is not drawn yet, its off state is open (a request or a question for the lead in the spec), or another part of the engine owns them. Where our code already draws one setting, the reason says which.

| Switch | Spec | Why it has no field |
|---|---|---|
| `screen_band_tests_position_only` | S-0101 claim 6 | our renderer does no on-screen culling (no screen band or ground box) |
| `screen_to_ground_not_exact_inverse` | S-0101 claim 9 | no F6 lepton position is used; the cell under a pixel is F7's pixel formula, the same in both settings |
| `unsorted_upper_layers` | S-0102 claim 7 | every mobile object is drawn in the ground layer; the upper layers are not modelled |
| `layer_change_on_events_only` | S-0102 claim 4 | render layer membership is not modelled (the moments of claim 4 may be simulation state) |
| `remembered_animation_in_cell_pass` | S-0102 claim 16 | no fog, so no remembered animations |
| `slot_values_replace_animation_adjustments` | S-0102 claim 17 | building slot animations are not drawn |
| `sprite_vehicle_eight_facings_only` | S-0102 claim 22 | the on state is drawn; the off state's facing part for counts 2 and 4 is asked in Q-0050 |
| `deploy_animation_neutral_brightness` | S-0103 claim 8 | deploy and undeploy animations are not drawn |
| `parachute_brightness_fixed` | S-0103 claim 8 | parachutes are not drawn |
| `zero_rate_freezes_ambient` | S-0104 claim 4 | the ambient fade's timing is not modelled (render::light::ambient_step takes one step) |
| `dominator_rate_thousandths` | S-0104 claim 4 | no dominator effect and no fade timing |
| `split_effect_priority` | S-0104 claim 2 | the lighting effects in force (nuke, dominator, ion storm, chrono flash) are not modelled |
| `chrono_flash_no_reevaluation` | S-0104 claims 2 and 5 | no chrono flash and no lighting re-evaluations |
| `dominator_height_terms_linger` | S-0104 claim 16 | no dominator effect |
| `trigger_tint_retints_all_tables` | S-0104 claim 17 | trigger actions do not change the lighting |
| `infantry_height_light_limited` | S-0104 claim 13 | the on state is drawn; the off state at a height of 208 leptons in a shadowed cell is asked in Q-0050 |
| `turret_recoil_leaves_barrel` | S-0105 claim 27 | recoil placement in a pose is not applied (no firing yet) |
| `vehicle_no_recoil` | S-0105 claim 28 | recoil placement is not applied; vehicles never recoil, as with the switch on |
| `barrel_only_no_recoil` | S-0105 claim 37 | recoil placement is not applied; a barrel without a turret never recoils, as with the switch on |
| `alt_body_turret_model` | S-0105 claim 12 | the alternative body and spawned-aircraft docking are not drawn; the main body always is |
| `voxel_single_pixel` | S-0105 claim 22 | the on state is drawn; the off state (no gaps in magnified sections) is not implemented |
| `voxel_no_intersection_depth` | S-0105 claim 23 | the on state is drawn; the off state (a depth test between sections) is not implemented |
| `voxel_buffer_wrap` | S-0105 claim 22 | the on state is drawn; which off state the switch selects is a question for the lead in S-0105 |
| `barrel_pivot_frame_mixup` | S-0105 claim 10 | the on state is drawn; which off state the switch selects is a question for the lead in S-0105 |
| `depth_mode_leak` | S-0105 claim 32 | the on state is drawn (render::voxel::VoxelCanvas); which off state the switch selects is asked in Q-0056 |
| `no_turret_shadow` | S-0105 claim 29 | turrets and barrels cast no shadow, as with the switch on; the off state's shadow is not specified |
| `slope_tilt_angles` | S-0106 claim 4 | voxel bodies are not tilted on slopes |
| `tilter_flag_ignored` | S-0106 claim 10 | voxel bodies are not tilted on slopes |
| `tilt_transition_restart_jump` | S-0106 claim 6 | no tilt state (it may belong to the simulation's locomotors) |
| `unbuildable_occupied_damaged_frame` | S-0108 claim 2 | no building is occupied in our engine yet |
| `full_state_unmaxed` | S-0108 claim 2 | buildings are drawn in their idle state only |
| `gate_double_reverse` | S-0108 claim 4 | buildings are not sold |
| `building_wrap_normalisation` | S-0108 claim 6 | building body ranges are not played |
| `turret_building_no_body_wrap` | S-0108 claim 6 | building body ranges are not played |
| `turret_start_clears_damaged` | S-0108 claims 11 and 12 | building slot animations are not drawn |
| `enable_unfreezes_unpowered` | S-0108 claim 13 | building slot animations are not drawn |
| `damage_swap_drops_occupied` | S-0108 claims 11 and 12 | building slot animations are not drawn |
| `roof_unload_no_fallback` | S-0108 claim 22 | units do not unload from buildings |
| `theater_refresh_buildup_range` | S-0108 claim 5 | no theater refresh |
| `anim_damage_starts_at_one` | S-0109 | animation playback is not implemented |
| `anim_erasure_late_slots_opaque` | S-0109 | animation playback is not implemented |
| `anim_fast_rate_freezes` | S-0109 | animation playback is not implemented |
| `anim_followup_keeps_step` | S-0109 | animation playback is not implemented |
| `anim_followup_start_doubled` | S-0109 | animation playback is not implemented |
| `anim_hide_at_level_15` | S-0109 | animation playback is not implemented |
| `anim_loop_count_wrap` | S-0109 | animation playback is not implemented |
| `anim_loop_end_from_sprite` | S-0109 | animation playback is not implemented |
| `anim_low_detail_shows_cloaked` | S-0109 | animation playback is not implemented |
| `anim_pingpong_lower_turn` | S-0109 | animation playback is not implemented |
| `anim_random_rate_collapse` | S-0109 | animation playback is not implemented |
| `anim_reverse_raw_loop_end` | S-0109 | animation playback is not implemented |
| `anim_translucency_levels` | S-0109 | animation playback is not implemented |
| `shroud_on_flat_grid` | S-0110 | shroud and fog are not drawn |
| `shroud_point_cell_wrap` | S-0110 | shroud and fog are not drawn |
| `building_resource_pips_hidden` | S-0111 | claim 11: the pip scale is not drawn (health bars, unit brackets and insignia are; they have no switch) |
| `passenger_overflow_hides_pips` | S-0111 | claims 11 and 12: the pip scale is not drawn |
| `pip_colour_default_shift` | S-0111 | Edge cases: the passenger and occupant pip colours are not imported; the pip scale is not drawn |

**Coverage.** `convert::coverage` gives every (category, key) of the rules and art files an outcome: `field` and `render` for keys interpreted per an approved spec, `folded` for type lists and similar, `unsupported` for keys kept as INI text because no approved spec maps them yet (the `spec` column names the specs whose import mapping mentions a key that is not converted yet).

**Open requests and their placeholders.**

| Where | Placeholder | Request |
|---|---|---|
| `ini::Document::layer` | a layered load that leaves two occurrences with one name hash in a changed table is refused (hash order on) | Q-0004 |
| `convert::layers` | only the reference fields the specs name create types; other kinds are marked `incomplete_registries` in the import record | Q-0005 |
| `convert` import options | the game-mode file is named by the user; the mode list is not read | Q-0006 |
| `convert::layers`, `convert::sim` | land-type resets (S-0211 claim 2) are applied in `sim.land_types` only; `ini_globals` keeps per-key layering | Q-0007 |
| `convert::art` | a building secondary sprite dropped by a later read's art section is left undetermined (a note, no file); no art for projectiles, particles and voxel animations | Q-0008 |
| `app::mapview::build` | a vehicle drawn as a sprite is taken to be in its standing pose (S-0102 claim 22) | Q-0021 |
| `app::mapview::build` | an overlay shadow takes its overlay's depth bias, flat (S-0112 claim 8 draws it; S-0102 claim 13 gives no bias for it) | Q-0021 |
| `app::mapview::build` | a terrain object type without a strength key is drawn with image frame 0; animated terrain objects show frame 0 (S-0102 claim 23) | Q-0023 |
| `convert::theater::variant_image` | the alternate-image mark of a tile cell (S-0010 claim 13) is not read: every cell uses the variant pattern | Q-0055 |
| `convert::sim::combat` | a type's threat coefficient starts at 0 before any layer gives it; `BuildCat` is a named value compared with `Combat`; `none` items are left out of name lists; an unreadable integer-list item is 0 | Q-0061 |
| `fmt::vpl` | four little-endian 32-bit header values, a palette, rows of 256; the row count is the first header value that fits the length exactly, else the largest the file ends inside; rows it does not supply read as zero (S-0105 claim 38) | Q-0022 |

## Map viewer (M2)

- `glasnost-map` (a simulation crate) holds the decoded map: footprint, legal and usable cells and their visiting order (S-0009 claims 1 to 4), the cell grid after the initial fill and the terrain stream, overlay stream values, waypoints, header, preview, lighting values in hundredths and thousandths (S-0104 claim 1) and the object lists as read.
- `convert::map` decodes a map document with importer warnings, the object lists by S-0009 claims 34 to 36, 40 and 42; `convert::theater` reads a theater's tile catalogue per S-0010 claims 1 to 5 (theater data file, declared tilesets, tile files with variants and the suffix fallback, role starts) with S-0009's base tiles and tile translation, builds the simulation's `map::terrain::TileCatalogue` (terrain and slope code of every tile cell, and each tile's animation and attaches-to sub-tile from the section its tileset's set name names, S-0010 claim 19) from the tile files, and holds the variant choice of claims 13 and 14 (the 4 × 4 pattern and the 8 × 8 pattern built from the unsynchronised stream, S-0202 claim 11).
- `render::projection` (S-0101), `render::light` (S-0104), `render::compose` (the CPU reference of S-0102's depth compositing), `render::scene` (items in world pixels) and `render::map_gpu` (the WebGL2 map pass, tested pixel for pixel against the CPU reference on a real or software GPU). Both honour `render::compose::ComposeQuirks` (`upright_depth_anchor_after_clip`, and `cell_pass_shadows_write_depth` of S-0102 claim 27: the GPU draws a writer shadow with a pipeline that blends and stores depth; `depth_reference_reset_on_jump` decides the depth reference the map pane sets at a view jump).
- `app::mapview` builds a map's scene from the decoded map, the rules import and the theater's files; the viewer's *Map viewer* mode lists the install's maps, loads one, scrolls within S-0101's limits, shows a minimap, layer toggles and object info with `rules explain`. Everything that rests on a request is marked provisional and can be hidden. Tiles are drawn from the cells after loading (S-0010 claim 12, when the game was set up; as stored otherwise): the image of claim 13, the tile cell of claim 24 (the clear tile for a cell without a usable tile), diamonds painted whole, index 0 included (claim 22, `Paint::Diamond`), extra images after their diamond (claim 23), at the scaled ground level (S-0104 claim 13). Pre-placed objects are drawn in their S-0009 initial state (claims 37 to 43), with buildings damaged when their starting health says so (S-0108 claim 2).

- **Live sandbox game (M3).** Loading a map also starts the simulation's sandbox game (`glasnost_sim::load::sandbox_setup`: one house per object owner, human-controlled except the neutral and special houses, fixed seed) from the same game inputs (`convert::game::load`). `app::mapview::build` keeps the static part of the scene (cell pass and building voxel parts) and a `Painter` with its caches; `app::mapview::live` draws vehicles, infantry and aircraft from the simulation after each tick (position from the lepton position, S-0101; voxel poses from the body and turret facings, S-0105; voxel part and shadow images kept in each type's image cache under the keys of S-0105 claims 26, 31 and 37, and drawn fresh into one voxel buffer kept between draws, with its clear area and the depth-buffered mode and its leak, claims 32 and 34; the cache-disable options are not honoured, because the scene keeps every image it is given; infantry frames from the sequence state, S-0107), keeps the object pass's ground layer as S-0102 claim 6 says (entry before the first greater key, one swap pass per tick on the previous keys) and paces ticks by S-0201 (16 × s ms, no catch-up; at most one tick per drawn frame, so index 0, whose period is 8 ms, runs at the display rate instead of 125 ticks per second). The GPU map pass uploads only the images and palette rows a scene gained (`MapRenderer::sync`). Problems that keep the game from starting are listed and the static view stays. While a game runs, every owner's objects are drawn, whatever the skirmish-view option says.
- **Controls.** Left click on an object selects it through `glasnost_sim::targeting::click` (shift, force-select, toggles it); left click on the ground clears the selection. Right click on a cell or object gives the selection's orders through the same function (shift force-select, ctrl force-fire, alt force-move), issued in the controlled house's name between ticks. S, X, G and D are stop, scatter, guard and deploy (`key_command`), C is the house's cheer (`CheerMemory`, kept for the session). The panel has pause, one-tick, the game-speed index for pacing (the game's own speed setting is not changed), the controlled house (default: the first house owning a mobile unit), the selected object's kind, type, owner, cell, behaviour and destination, and the game's and the drawing's notes. Selection markers are our own presentation.
- **Combat (M4).** `app::mapview::live` drains `World::effects` after every tick it runs and draws, after the object pass: a provisional placeholder of our own for each projectile in flight at its projected point with its height (S-0101; a 7 × 7 diamond, cyan, orange when arcing, magenta when steering; an invisible projectile is not drawn; no game art, because the importer looks up no projectile art, Q-0008), a muzzle cross at a shot's fire point for 3 ticks, a growing burst ellipse for a detonation for 8 ticks, sized from its spread radius by S-0101's ground projection, and a magenta square for each animation event for 12 ticks (animation playback, S-0109, is not implemented, so no animation art is drawn). Then S-0111's status extras (`app::mapview::combat`): the health bar of every selected object (with the unit bracket image) and of the object under the pointer (without it), at any health (claim 3), with the filled segments of F1 in soft-float under the rules' rounding mode, the colours of claim 5, the unit places of claim 6 (δ from `PixelSelectionBracketDelta`) and the building pip line of claim 7 (foundation and art `Height`), and every object's veterancy insignia (claim 18), from `PIPS.SHP` and `PIPBRD.SHP` in the unshaded overlay palette `PALETTE.PAL` (a missing sprite draws nothing). Not drawn yet: claim 1's culling, claim 2's shading (no shroud), building corner brackets (claim 8; our own markers stay for buildings and for units when the bracket sprite is missing), the pip scale (claims 9 to 15), the self-heal marker, group numbers and the wrench. The health bar has no quirk switch. Destroyed objects leave the picture: buildings the game removed are dropped from the cell pass and the ground layer; infantry dying are drawn by their death sequence (S-0107) until the game removes them. The layer toggles *combat effects* and *status extras* hide them. Attack orders are a right click on an enemy with a selection (`targeting::click`); units in guard pick their own targets. The panel shows the selected object's health of its strength, target, ammunition, experience and rank, the event counts and every house's kills, losses and score.
- **Live lane.** `GLASNOST_GAME_DIR=<install> cargo test --release -p glasnost-app --test live_lane -- --nocapture` loads a few maps with their game, selects units through the input rules, orders a move, runs 240 ticks and writes views before, during and after to `local-lane/screens/`, with per-tick step and redraw times (`GLASNOST_LIVE_MAPS`, `GLASNOST_LIVE_TICKS`, `GLASNOST_LIVE_FILTER`, `GLASNOST_LIVE_INFANTRY=1`). Its combat run (`live_maps_fight_and_draw`) picks the closest armed unit and enemy unit of two player houses that are not allied, orders up to 8 armed units near the first to attack with a right click, runs `GLASNOST_COMBAT_TICKS` (360) ticks on `GLASNOST_COMBAT_MAPS` (2) maps and writes `combat-<map index>-<tick>.png` before the order, at the first visible projectile, 8 ticks later and at the end, printing shot, detonation, animation and kill counts and timings. With a real install the import holds no weapons yet (weapons, projectiles and warheads exist only by reference, and the importer does not yet create them from the weapon reference fields of S-0004 claim 13 and its Import mapping), so no unit fires; the run then takes any two units of houses that are not allied and still shows the status extras.

## Simulation (M3)

`glasnost-sim` runs the game rules on the typed ruleset and map. Everything in it is integers and the soft-float types of `glasnost-num` (S-0205: IEEE binary32/64 in integers, round toward zero under `quirks.chop_rounding`), with ordered containers only.

- **Rules and data.** `rules::SimRules` is built from a resolved ruleset (`SimRules::new(&resolved.ruleset, &resolved.sim_rules_hash)`): typed per-kind tables, the 105 quirk switches of `rules::Quirks` (every one defaulting to the original; `refused_death_keeps_unit` of S-0107 claim 17 is not consulted, since M3 has no death handling), and engine defaults cited to their specs. A type without a locomotor field has the teleporting locomotor (S-0207 claim 14); vehicles with the driving locomotor drive and infantry with the walking one walk, and every other locomotor is not moved in M3. `consts` holds the spec constants given as decimals, as checked bit patterns.
- **A game.** `load::new_game(rules, map, catalogue, setup)` follows S-0009 and S-0202's table *Draws during map loading*: the early phase of start positions on blank cells (lists 1 and 2, the assignment against list 2, with the base rules' passive fields, claims 17 to 20 and 33; `starts`, with the start-cell variant of the nearby-cell search, `path::start_cell`), the initial fill, the overlay type stream placed overlay by overlay (`overlays`: the slope condition of claim 12, high-bridge cells as each overlay is placed, wall connection bits, low-bridge end markers with their pieces' draws), the overlay data stream, the first recalculation pass, the vein rebuild with its draws (claim 14), the tile animations each recalculation starts with their creation draws (S-0010 claim 19, S-0202 claim 10: the first recalculation of a cell before the second pass, then at most one per cell from the second pass on, in play too; `load::recalc_cells`), the pre-placed objects (with the visceroid draw of a vehicle's placement and its follower-car link, claim 41), the second pass, the late phase (list 3), the owners of the map's walls (S-0210 claim 14) and the starting units (`starting`, claims 45 and 46). Terrain recalculation itself is `glasnost-map`'s (S-0010, S-0211). `GameSetup` gives each house a role (player, neutral, special), the unit-count setting and the bases setting. `load::sandbox_setup` is our viewer's setup: one house per object owner of the map, no start positions and no starting units.
- **Wall connection bits.** `walls` keeps a wall's overlay data value as 16 × damage stage + connection bits (S-0112 claim 7), recomputed when a wall is placed from the map's overlay list, when crushing removes one, and when a gate or wall tower is placed or destroyed (`quirks.gate_tower_fixed_cells`). The map viewer draws the cells after loading, so walls show the bits the simulation gives them.
- **A tick.** `tick::step` runs S-0203's parts in their places: the turn pass over the update order (with S-0203's removal skips and second turns), each object's behaviour part (`process`, S-0215), its locomotor (`drive` with `tracks`, S-0207 and S-0206 claim 12; `walk`, S-0208), its infantry sequence (`infantry`, S-0107), then the orders (`orders`, S-0213), then the tick number. Occupancy is `cells`/`occupancy` (S-0209), crushing `crush` (S-0210), passability and regions `passability`/`regions` (S-0211, S-0212), pathfinding `path` (S-0212) behind the `Pathfinder` trait, over (cell, search layer) pairs, with the corridor of claims 13 to 14 in `corridor` (sectors and connections are derived data, rebuilt with the regions). Regions and sectors follow the rules of claims 3 and 13b and exact ties the recording order of claim 12a (departures D4 and D7, accepted by the project lead).
- **Orders and input.** Every change a player or our AI makes goes through `orders::issue` (pending lists, S-0213). `targeting` turns a click or key command into orders by S-0214 and S-0213 table T2, for people and AI houses alike; the selection is kept by the caller.
- **Saves, replays, hashes.** `save::save`/`load` store the whole state with the rules hash; `save::state_hash` is SHA-256 of the state's bytes (derived data such as regions and the path search's memory is not state). `save::Replay` holds the rules hash, the setup and every order with its tick.
- **Marks of the map load.** The loader marks the high-bridge cells of each high-bridge overlay as it is placed (S-0211 claim 15) and gives the map's walls their owners once the buildings are placed, before any starting unit (S-0210 claim 14). Gates start closed (S-0215 claim 35); S-0209's gate rows read `process::gate_open`, and a unit that meets the *closed friendly gate* outcome as it moves (not while it only asks) calls `process::gate_met`.
- **Owners that are not human-controlled.** An AI house counts as human-controlled (glossary *AI house*); the neutral and special houses do not, and keep the original's rules for such owners (glossary *Player houses only*): skill level 0 (S-0209 claim 11), starting units in *area guard* (S-0009 claim 45), crates blocking their walkers (S-0209 row WO1), the idle scatter of a frightened fraidycat (S-0107 claim 14), lying down while moving (S-0107 claim 15) and the computer-controlled branch of S-0208 claims 11 and 15. `House::human` carries the distinction.
- **Selection.** `targeting::selectable` is S-0214's *can be selected*: a live unit or building, but not a building whose type is invisible in the game. Units below the surface and cloaked units are an open question of S-0214; M3 has neither state.
- **Not yet in M3.** Combat (firing, damage, target acquisition: M4) is left as seams; the behaviours other topics own are processed as behaviours without a processing rule and leave a note in `World::notes`. Infantry death (S-0107 claim 17, `quirks.refused_death_keeps_unit`) and a prone unit's get-up on a movement order (S-0107 claim 15) wait for damage and fear (M4); hunt's target search and its idle attempts (S-0215 claim 27, S-0107 claim 14) likewise.

The determinism gate: `cargo test` checks the first 300 ticks of `crates/sim-gate`'s scripted synthetic replay against `golden/scripted.txt` in a debug build; `cargo xtask gate` checks all 3000 ticks natively in release and as wasm32 under node (CI job *gate*). After a deliberate change of behaviour, regenerate the file with `GLASNOST_UPDATE_GOLDEN=1 cargo test --release -p glasnost-sim-gate -- --include-ignored` and say why in the commit.

The headless runner: `glasnost-headless synthetic [ticks] [seed]` prints the per-tick hashes of the synthetic replay, `glasnost-headless replay <file>` those of a replay file. The local lane: `GLASNOST_GAME_DIR=<install> cargo run --release -p glasnost-tools --bin glasnost-headless -- smoke [ticks] [seed] [index]` loads every map of the install in the sandbox setup, orders every mobile unit to another unit's cell at tick 4, runs the ticks and writes counts and timings (no names, no values) to `local-lane/sim-smoke.txt`. A skirmish pass also sets each map up as a skirmish of the tool's design (`headless::skirmish_setup`: as many players as the lobby allows, all on random positions, the neutral and special houses, unit count 10, bases on), loads it without running it and counts the maps loaded and refused (with the reason) and the starting units created.

Provisional rules of the simulation, each tied to a request:

| Where | Interim rule | Request |
|---|---|---|
| `corridor`, `path` | a level's coarse search ends when a route to the destination's sector comes up; no bridge's axis is kept, so a bridge cell is its own coarse end cell and the legs of F8 are 0; objects on either layer of a neighbour give the corridor exemption | Q-0043 |
| `path::nearby`, `path::burrow_ok` | the no-move rows of S-0212 table T5 never ask for the further test; the burrowing test applies only its stated conditions, and the tileset's allow-burrowing flag reads as allowed (the tile catalogue does not carry it yet) | Q-0041 |
| `path` | a 4-level step is allowed between a bridge cell on its deck and a higher cell that is not a bridge cell, cell levels compared (a deck pair may step sideways onto ground of its cell's level); entry outcomes in the search use the expanded pair's level; no bridge links, and a deck is compared by its cell's region, so the gate refuses orders onto a deck or across water | Q-0042 |
| `drive` | the look at a new path's first cell (S-0207 claim 15) takes as nearest object every listed object but the vehicle, by S-0204's distance to the cell centre at the surface height, ties to the first listed | Q-0040 |
| `load` | not implemented yet: the unused-house cleanup (S-0009 claim 28; Q-0045 asks whether AI houses count as computer players for its condition), header pairs as start position marks (claim 29), observers (claim 22), the tags of cell tags and of pre-placed objects with their draws (S-0011 claim 34), wall buildings (S-0112 Terms) and a departing wall tower's wall hits (S-0112 claim 7, S-0303 F4) | — |
| `starting` | the placement search's acceptance test reads the ground layer of every cell, bridge cells included, and takes as the nearest occupant the infantry unit, vehicle, aircraft or building with the smallest squared plane distance to the cell centre, the first listed on ties; with `starting_infantry_share_centre` off the standing-position draw is taken at each cell tested | Q-0045 |
| `starts` | with `start_waypoint_gap` off, N stays max(P, K) with K counted as with the switch on; with `random_start_asymmetric_range` off, the first draw gives y | Q-0045 |
| `load`, `overlays` | no overlay type has a cell animation (no spec names its key), so cell animations take no creation draw; an animation's start event (its chain reaction and middle-event effects) takes no draw | Q-0046 |
| `drive` | the attack override of a refused step (S-0207 table T4) gives no destination; no step check follows the search made after a curve's second cell gives an obstacle (table T5) | Q-0035 |
| `crush` | a map wall takes the owner of the nearest qualifying building at any distance (S-0210 claim 14) | Q-0036 |
| `process` | a gate's door moves at the start of the gate's turn, before its per-tick start and processing | Q-0030 |
| `objects` | pre-placed objects (S-0009 claims 34 to 43) carry no veterancy value or tag (a vehicle with the bridge flag stands on the deck where its cell has one, else on the ground), start switched on, and aircraft stand landed; follower cars carry their leader (claim 41) but are not sent after it (S-0212 claim 26 is a seam); a terrain object starts with health 0 (no strength field yet) | Q-0023 |

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
- the determinism gate (`cargo xtask gate`: native release and wasm32 under node);
- the release web build;
- the browser lanes:
  - WebGL2 on Chromium, Firefox and WebKit (WebKit non-blocking until it has run green);
  - optional WebGPU on Chromium.

CI uses synthetic data only.
