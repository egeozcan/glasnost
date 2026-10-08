# Glasnost: project plan

*Planning document. Written 2026-10-08. Self-contained: it is the reference for the project lead and for every implementer session in this repo.*

> **For implementer sessions.** Behavioural numbers and mechanics mentioned in this plan are *planning context only*. They are not specs. Sim code cites only scrubbed specs (`S-xxxx`) and measurements (`M-xxxx`), never this file. Anything below marked "to be confirmed by spec" is a working assumption.

## 1. Summary

An open-source (GPL-3.0-or-later) reimplementation of *Command & Conquer: Yuri's Revenge* in Rust. It compiles to WebAssembly for the browser and also builds natively. Graphics, sounds, rules and maps are imported at runtime from the player's own install (Steam app 2229850 or another copy). The project never ships EA bytes and never commits EA's numbers.

- **Scope order:** skirmish against our own AI (Allied, Soviet, then the Yuri faction, as part of skirmish) with mechanics closely replicated → multiplayer once mechanics are right → campaign (optional, later).
- **Sim:** deterministic, integer/fixed-point, serializable and lockstep-ready from day one.
- **Rules:** our own extensible format (JSONC + generated schema). An importer converts the user's INI files and maps locally.
- **AI:** our own, from scratch, playing by human rules.
- **Process:** a documented two-team setup. A spec side decompiles the stock game and writes behavioural specs; isolated AI implementer sessions build from scrubbed specs and measurements only. The project uses this documented two-team process and makes no certified clean-room claim.
- **Time:** about 16–24.5 focused person-months to the first full YR skirmish (M0–M7), which is **about 3.3–7.5 calendar years at 10–15 h/week** (§5).

*This project is not affiliated with, endorsed by or supported by Electronic Arts. Command & Conquer and Yuri's Revenge are trademarks of Electronic Arts Inc.*

### 1.1 Why this project

| Project | Status | Why it doesn't cover this |
|---|---|---|
| **Chrono Divide** | v0.84.0, Sep 2026 | Mature browser client, but closed source and RA2 1.006 only. Its mod SDK says YR is "NOT compatible". Reference for UX and features only; its client is banned as a source (§3). |
| **OpenRA/ra2** | GPL-3, last commit Nov 2025 | Native only, no releases, RA2 only. Re-expresses units through OpenRA traits instead of reproducing the original. |
| Browser YR forks (huangkaoya/redalert2, OpenRA2/RA2Web, ammaarreshi) | Exist | Undisclosed origin and licence, linked to the Chrono Divide client. **Banned on both sides. Never open them.** |
| EA source releases | TD, RA1, Generals only | No TS, RA2 or YR source exists publicly. |
| Rosebud's OpenRA browser port | RA1 only | Source not found. |

## 2. Decisions

| # | Decision | Status |
|---|---|---|
| D1 | **License:** GPL-3.0-or-later. Dependencies must be GPL-compatible (MIT, Apache-2.0, MPL-2.0 fine). Allows reading and porting format/loader code from OpenRA and XCC with attribution. | DECIDED |
| D2 | **Scope order:** skirmish vs own AI (Allied, Soviet, then Yuri) → multiplayer after mechanics are right → campaign optional. Sim lockstep-ready from day one. | DECIDED |
| D3 | **AI:** our own, from scratch. No reimplementation of ai.ini/aimd.ini TeamTypes or ScriptTypes. Issues only human-possible orders, sees only its own shroud, deterministic inside the tick. Top difficulty uncapped. **Lower difficulties are simpler strategy modules, not APM or reaction-time caps.** | DECIDED |
| D4 | **Target game:** Yuri's Revenge only. No plain-RA2 rules mode. (YR still mounts the RA2 base archives underneath its own.) | DECIDED |
| D5 | **Data baseline:** stock YR 1.001. Rules from the single highest-precedence rulesmd (the 1.001 copy in expandmd01) plus artmd; art and sound from the YR archives including the RA2 base archives. | DECIDED |
| D6 | **Rules:** our own format (JSONC + schema generated from Rust types; numbers never via f64; inheritance; mod patch layers; rules hash pinned in replays). The user's INI files are *imported*, not interpreted. Stock numbers are never committed. YR quirks are named switches defaulting to YR behaviour. | DECIDED |
| D7 | **Process:** two teams. Spec side: a local Claude Code session (cloud-hosted Claude model) decompiles the stock, hash-checked gamemd.exe and writes behavioural specs, starting now. Every spec is scrubbed before entering `spec/`. Implementation side: isolated Claude Code sessions in this repo only. | DECIDED |
| D8 | **Fidelity:** exact formulas, rounding, timings and the original RNG algorithm where cheap; YR quirks by default; statistical match for battle outcomes; no replay or lockstep compatibility with original clients (§6.1). | DECIDED |
| D9 | **Language and graphics:** Rust stable, wgpu with both backends. Core renderer within the WebGL2 feature set (works on Firefox Linux). WebGPU-only extras behind runtime checks. Native Vulkan build for GPU debugging. | DECIDED |
| D10 | **Audio:** music deferred (M8). SFX, voices and EVA in later milestones (viewer in M1, in game from M5). | DECIDED |
| D11 | **Team:** the project lead is spec owner, oracle operator and reviewer/approver, and writes no engine code. No second human implementer; implementation is done only by isolated AI sessions. | DECIDED |
| D12 | **Repos:** `glasnost` (this repo: scrubbed `spec/`, measurements, code) and a separate, never-pushed spec workspace (`~/glasnost-spec-dirty`). Each has its own CLAUDE.md and Claude Code memory. | DECIDED |
| D13 | **Name:** Glasnost (Gorbachev's policy of "openness"). It avoids EA trademarks. Registry search (USPTO/EUIPO) and claiming the crate/GitHub name before going public. | DECIDED |

Deployment default (not a decision yet): static hosting such as GitHub Pages for v1; v1 uses no threads, so COOP/COEP headers are not needed.

## 3. Process: two teams

These rules are written up for implementers in `PROVENANCE.md`, `CONTRIBUTING.md` and `CLAUDE.md`, and for the spec side in its own workspace (`CLAUDE.md`, `SCRUBBER.md`), before any engine code is written. Where this section and PROVENANCE.md differ in detail, PROVENANCE.md wins.

### 3.1 Locations

| Location | Pushed? | Contents |
|---|---|---|
| `~/glasnost-spec-dirty` | **Never pushed anywhere** | Decompiler projects, raw analyst notes, the identifier denylist, the spec backlog, the scrub log (the full provenance record of every spec); spec-side session transcripts stay on that side too |
| `glasnost/spec/` | Yes, with this repo | Scrubbed specs (`S-xxxx`), signed off |
| `glasnost/measurements/` | Schemas, scenarios and M-ids yes; measured stock values no (git-ignored) | Measurement records (`M-xxxx`) and scenario definitions |
| `glasnost/` (rest) | Yes | Code, synthetic fixtures, converter, schema |

Decompiler raw notes never leave `~/glasnost-spec-dirty`. Personal machine details stay out of tracked files; they go in the gitignored `LOCAL.md`.

### 3.2 Roles

| Role | Who | Sees | Never sees |
|---|---|---|---|
| **Spec agent** | Local Claude Code sessions (cloud-hosted Claude model) in `~/glasnost-spec-dirty` | Stock, hash-checked gamemd.exe and decompiler output; YRpp, Ares, Phobos source and docs; ModEnc; OpenRA; XCC; the existing ra2-yr-setup repo | `glasnost/` code |
| **Oracle operator** | The project lead, plus scripted harnesses | The running stock game; measurement harnesses; may see the spec side | — |
| **Scrubber** | Automated lint → a fresh AI session with no access to the spec workspace → the project lead's sign-off | Draft spec text; lint verdicts; spec template, topic list and glossary | Raw notes, decompiler output, the spec workspace |
| **Implementers** | Isolated Claude Code sessions in `glasnost/` only | `glasnost/` (scrubbed `spec/`, measurements, code); OpenRA and XCC (GPL-3) for format/loader code, with attribution | `~/glasnost-spec-dirty` and the scrub staging folders; ra2-yr-setup; YRpp, Ares, Phobos, Syringe source *and docs*; ModEnc; disassembly and decompiler output; the Chrono Divide client; the browser forks |

- The spec agent works only on the **stock gamemd.exe, verified by Steam's file check**. Never the patched `gamemd-spawn.exe` or the launchers.
- The Chrono Divide client and the decompiled browser forks (huangkaoya/redalert2, OpenRA2/RA2Web, ammaarreshi) are banned on **both** sides.
- Never copy text from EA manuals or strategy guides into specs.
- **Implementer sandbox:** Claude Code permission deny rules for `~/glasnost-spec-dirty`, the scrub staging folders, ra2-yr-setup, any YRpp/Ares/Phobos checkout and decompiler output directories; web access blocked to the YRpp, Ares, Phobos and Syringe repos and docs, ModEnc and the browser forks. Keep logs showing the rules were in force.
- **The project lead on the implementation side:**
  - every instruction to an implementer cites a spec, measurement or spec-request ID (`S-`, `M-`, `Q-`);
  - no "do it like the original does internally";
  - no uncited numbers;
  - PRs are reviewed against tests and behaviour, not against memory of the internals;
  - every exception is logged as an incident (`spec/incidents/I-xxxx.md`).

### 3.3 Spec-writing rules (spec side)

**Write:**
- observable behaviour;
- formulas in mathematical notation, with their rounding rule ("truncates toward zero after the multiplication");
- constants with units (ticks, leptons, per-mille);
- state machines as tables of state, event and result, in **our own glossary**;
- ordering as an observable consequence ("damage from both shots in one tick is applied before death checks");
- edge cases and original bugs, as behaviour (they become `quirks.*` switches, §4.6);
- for every claim: a confidence level; a backing of `measured` (M-id), `measurable` or `unmeasured`; a proposed measurement scenario for each `measurable` claim.

**Never write:**
- addresses, offsets, vtable slots, struct layouts, field order or sizes;
- class, function, field or enum names from the binary, YRpp, Ares or Phobos;
- pseudo-code, C-like syntax, or step lists that mirror one function's control flow;
- decompiler output or quotes of it;
- organisation by code module.

Organise specs by game topic: combat, movement, production, economy, visibility, AI-visible rules and so on (the topic list lives in `spec/README.md`). **Test:** two different correct implementations must both satisfy the spec. INI key names appear only when describing the import mapping, because they are the user's data format.

**The analyst's main deliverable is a hypothesis plus the measurement that would confirm it:** decompile to know what to measure; measure to know what to cite. Where measurement is impractical (RNG internals, tick order, rare branches), the claim stays `unmeasured`, with the reason stated, and the spec gets a second scrub.

### 3.4 Scrubber, records and checks

1. **Lint** every draft: any `0x` hex number (short values are written in decimal) and bare 6-to-8-digit hex patterns; C-like tokens (`->`, `::`, `if (`, braces, lines ending in `;`); the denylist of YRpp, Ares and Phobos identifiers and any RTTI names (the list itself lives only in `~/glasnost-spec-dirty`).
2. **Scrubber session** (fresh context, no access to the spec workspace) rewrites anything code-shaped into behavioural prose and rejects specs that fail the two-implementations test.
3. **Sign-off** by the project lead, recorded.
4. **Spec front matter** in `spec/` (see `spec/TEMPLATE.md`): id, title, topic, revision, source class, status, approver, approval date, linked M- and Q-IDs. Per-claim backing sits in the spec body. The full record (draft and notes hashes, reviewer session ID, analyst session, sources) stays in the spec workspace's scrub log.
5. **Citations in code.** Engine code cites `// spec: S-0142` or `// measure: M-0042`; absent-key defaults in the schema carry the same tags. A CI lint fails any behavioural constant without a citation.
6. **Transcripts** of spec-side sessions are kept on the spec side and never pushed.
7. **Similarity checks** on `glasnost/`, run periodically by the project lead outside implementer sessions and outside public CI: JPlag against YRpp, Phobos and Ares; identifier overlap against the denylist. On a hit: revert, re-implement in a fresh session, log the incident.
8. Format scripts in ra2-yr-setup (`mixextract.py`, `vxl.py`, `csf.py`) are never copied; formats are re-implemented from specs and OpenRA/XCC.

### 3.5 Assets, numbers and naming

- Ship zero EA bytes and **zero EA numbers**: none in the repo, CI, releases, the site or crash reports. Strip asset content and file paths from logs.
- Public CI uses only a synthetic CC0 ruleset and fixtures produced by our own writers. Fidelity tests are written as relations ("shot interval = rules.weapon(W).rof") and run locally against the user's import. Measured values stay git-ignored.
- Multiplayer peers exchange hashes, never rules data. Stock maps are referenced by content hash.
- Do not extract the `[PrivateKey]` INIs inside the archives.
- Anything we ship that does not come from the user's files (fallback art, placeholder HUD, the default layout, screenshots) must look visibly original.
- A plain compatibility line only ("works with game data from *Command & Conquer: Yuri's Revenge*, not included"), the non-affiliation disclaimer in the README, the site and the title screen, no EA art in promotion, no money beyond donations.

## 4. Architecture

```
 user's game folder ──► import worker (fmt + vfs) ──► OPFS cache (optional, local only)
                                   │
          converter: rulesmd + mode INI + map INI ──► JSONC tree ──► mods ──► typed ruleset (hashed)
                                   │
   UI / own AI ──Orders──► ┌──────────────────────────┐ ──snapshot+events──► render (wgpu: WebGL2 core,
   net (later) ──Orders──► │ sim: integer, 1 thread,  │                        WebGPU extras)
                           │ tick-based, serializable │ ──state hash──► replay / sync / save
                           └──────────────────────────┘                      audio (WebAudio, SFX first)
```

### 4.1 Stack, and why WebGL2 is the core

**WebGPU is not a big improvement for this game's core renderer.**
- What the game needs fits WebGL2: palette-indexed sprites, isometric terrain, house-colour remap, lighting tints, shroud, laser and tesla bolts, voxels pre-rendered per facing. That is roughly 10–50 batched draw calls per frame, so lower per-draw overhead buys nothing.
- WebGPU only raises the ceiling (compute, storage buffers, timestamp queries). That helps optional extras (live voxels at any angle, GPU particles) and profiling.
- **The sim never uses GPU compute.** GPU maths differs between vendors and would break lockstep and replays.
- **Reach** (web3dsurvey, working adapter): WebGPU 82.7% vs WebGL2 97.8% overall; **on Linux 17.5% vs 87.8%**. Linux/Proton players are a natural audience.
- **Browser status, Oct 2026:** Chrome on Linux enables WebGPU by default only for Intel Gen12+ (Chrome 144) and NVIDIA on Wayland (147–148); AMD is still behind a flag. Firefox on Linux has WebGPU only in Nightly.

**Stack:**
- Rust workspace, wgpu 30 (`webgl` and `webgpu` features, `new_instance_with_webgpu_detection`), winit, trunk or wasm-bindgen.
- Request `adapter.limits()` and size atlases and uniform blocks from it. `downlevel_webgl2_defaults` (2048 texture size, 16 KiB uniform blocks) is a floor, not a target; desktop browsers usually report 16384 textures.
- **Renderer rules:** the core path uses no compute, storage buffers, indirect draws or non-zero `base_vertex`. Large per-unit tables go in data textures. Every WebGPU-only feature sits behind `adapter.features()` / `downlevel_capabilities()` checks. The WebGL2 lane runs in CI from M0; retrofitting it later would mean a rewrite.
- **Debugging:** native Vulkan build with RenderDoc or Radeon GPU Profiler. wgpu's WebGL backend has no timestamp queries.
- **Dev machine:** browser and GPU test recipes (e.g. a flagged Chromium for WebGPU) go in `LOCAL.md`.
- **UI:** egui for development tools only. The game UI is drawn from the user's SHP, PCX and font files using a layout data file; fallback art is visibly original.
- **Audio:** WebAudio on the web, kira or cpal on native. SFX, voices and EVA first; music deferred.
- **Threads:** none in v1 (no SharedArrayBuffer, no COOP/COEP).

### 4.2 Crates

- `fmt`: pure parsers, each with a writer for fixtures: MIX, SHP-TS (compression 0–3), TMP-TS, VXL/HVA (both normals tables), VPL, PAL, CSF, IDX/BAG, AUD, PCX, FNT, LZO1X, LCW, map packs. IMA-ADPCM WAV (music) deferred.
- `vfs`
- `ini`: reader that matches YR's parsing; used **only by the importer**.
- `convert`: INI → our tree, `coverage.csv`, art split, map conversion.
- `rules`: schema types, resolve pipeline, `rules explain`, canonical hash.
- `map`, `sim`, `ai` (produces Orders only), `render`, `audio`, `ui`, `net`.
- `platform-web`, `platform-native`.
- `tools`: headless runner, sync-diff, viewer, scenario DSL compiler.
- `server/relay` (later).

Dependency rule: `sim` does not depend on render, audio, net, fmt, ini or convert. It consumes only the typed ruleset and typed map.

### 4.3 Import

- **Picking the folder:** `<input webkitdirectory>` and drag-and-drop everywhere; Chromium also offers a persistable `showDirectoryPicker`. Verify the Flatpak browsers' document portal (fallback: a read-only `flatpak override --filesystem=…`).
- **Identifying the install:** table of content hashes (hashes only) for known YR versions, case-insensitive names. **Refuse installs without the YR expansion files.** Warn about non-stock EXPANDMD##/ECACHE files. **Loose files off by default**, offered only as an opt-in "mod layer".
- **Indexing:** decrypt MIX headers (RSA with the public Westwood modulus yields a 56-byte Blowfish ECB key; use the crate's `BlowfishLE` with a round-trip test). Mount nested archives, the RA2 base archives included, in a configurable precedence order. Build names from INI references plus naming conventions (archives store only hashes).
- **Reading and storage:** lazy reads with `File.slice`. Stage A: re-pick the folder each session (indexing reads only kilobytes). Stage B: optional OPFS copy of about 0.7 GB without music (629 MB core, 71 MB maps) with `persist()`, a start-up manifest check and handling for Safari's 7-day eviction. The converted, typed ruleset is cached in OPFS as a binary blob keyed by input hashes.
- **Deferred:** music (about 124 MB IMA-ADPCM WAV) and movies (about 1.06 GB BIK, possibly an opt-in ffmpeg.wasm transcode that tolerates the corrupt Steam files).

### 4.4 Rendering

- **Palette-indexed pipeline:** sprites as palette indices in R8 atlases; one 256×N palette texture holding palettes plus a remap row per house (remap and unlit index ranges to be confirmed by spec); per-cell lighting from the map's lighting section plus light sources; instanced quads ordered by an isometric depth key (row + height, sub-cell, layer, ZFudge); terrain in static chunks.
- **Voxels:** mesh each VXL limb once, render on demand into a pooled sprite cache keyed by (model, facing, turret, HVA frame, tilt), VPL shading in the shader. Facing count, HVA scaling and the house-colour ramp come from S-specs and are checked against screenshots. Live voxels at any angle are an optional WebGPU extra; the original quantises facings, so the baked path is also the faithful one.
- **Shroud:** vanilla skirmish has shroud only, no fog (to be confirmed by spec).

### 4.5 Simulation

**Determinism**
- Integer and fixed-point only. Clippy bans floats and `HashMap`. No time, threads or host calls in the sim. Stable IDs, sorted iteration.
- One sim RNG plus a separate cosmetic RNG outside the sim. The sim RNG uses **the original's algorithm** as given by spec.
- **Float exception path.** Where a spec shows the original computes game state in floating point and integer emulation cannot reproduce it, a narrow `Real` wrapper over f64 is allowed: only + − × ÷ and sqrt, no libm transcendentals, no FMA or `mul_add`, no relaxed SIMD. Decided per site and documented in the spec. These operations are bit-identical across wasm and x86-64/aarch64 (apart from NaN payloads). x87 extended precision is not emulated (§6.1).

**Units and ticks**
- 256 leptons per cell (to be confirmed by spec).
- Ticks per second at each game speed: from a spec, confirmed as **measurement #1**.
- Each tick runs a fixed pipeline order, documented in our own terms; from specs where given, from measurement where observable (e.g. a mutual kill). Components never define their own order.

**Required from the first sim milestone**
- **All sim state serializable** in a versioned format: save/load, recovery after a tab reload, replay keyframes, and a determinism check (identical hash after a save/load round-trip).
- A **trigger engine** sized for multiplayer maps, designed into the sim from M3 (map conversion already maps trigger codes in M2) and filled in with the multiplayer subset in M6. The stock multiplayer maps mostly use triggers, with a small set of event and action codes; exact set from the coverage inventory. Maps with local TeamTypes or ScriptTypes show a "needs unsupported scripts" warning.

**Out of scope:** Tunnel, Mech and Levitate locomotors, and veins (unused in stock YR).

**Pacing:** the web build runs the sim in a worker on its own clock, not on requestAnimationFrame, and catches up after background-tab throttling.

### 4.6 Rules: our own format, YR's mechanics

**Where the numbers come from: import.** At import a converter turns the user's own rulesmd, artmd, game-mode override file and map INI into our format. We ship the converter, the schema, hard-coded engine defaults (each with a spec tag) and a synthetic test ruleset, **never EA's values**.

**Import layering** (merged at INI level, then converted in one pass):
1. **The single highest-precedence rulesmd only.** In a stock install the 1.001 copy in expandmd01 *replaces* the base copy; the two are never merged.
2. **The game-mode override file** (e.g. MPBattleMD.ini; mpmodesmd.ini only lists the modes).
3. **The map's override sections.**

Converting each layer separately would break derived defaults (elite weapon defaulting to normal, `Image=` rebinding art, `WeaponCount`/`WeaponN`). Our own mods apply afterwards as patches on the converted tree.

The `ini` reader must parse exactly as YR does: case; duplicate keys, entries and sections; boolean and percent forms; value truncation; inline comments; list splitting; **type registration order** (sets array indices, hence sidebar order and some tie-breaks). These semantics come from S-specs. **No INI rules engine exists at runtime.** Ares and Phobos keys are unsupported and show up in the coverage report.

**Format: JSONC**, validated by a JSON Schema generated from the Rust types with schemars.
- A custom JSONC parser builds our own untyped tree with source spans; number tokens stay as text; a serde deserializer runs over that tree (serde_json is not in the typed path; its `arbitrary_precision` breaks with tagged enums and flatten).
- **Numbers never pass through f64.** Data holds integers in native units (leptons, ticks, angle units, per-mille). Decimals are quoted strings with a unit (`"5.75c"`, `"1.5"`) parsed straight into fixed point; bare decimals are rejected. Each numeric field declares its INI form, scale, rounding rule and spec tag.
- Rejected: JSON5, RON, KDL, YAML (loses spans and number text, implicit typing), TOML, OpenRA MiniYAML, a custom DSL. The merge semantics are fully specified here.

**Shape:** anything a mod may touch is an object keyed by instance name (`"spawner@missiles"`), because RFC 7396 Merge Patch replaces arrays whole. `inherits` is an ordered list (later wins, own keys win over all); `^Name` marks abstract templates; cycles are errors; `null` removes; set-like lists (`prerequisites`, `owners`, `immune_to`) accept `{"+": [...], "-": [...]}`. Imported types keep their INI section names as IDs; mod types are namespaced (`mymod:laser_tank`); display names come from the user's CSF (`ui_name: "csf:Name:HTNK"`).

**Resolve pipeline**
1. Converted base tree.
2. Mod patch layers in manifest order.
3. Inheritance, `null` removal, set operations.
4. Strict typed deserialization (`deny_unknown_fields`).
5. Semantic validation (references, ranges, kind/component compatibility).
6. IDs interned in a defined order (import order, then mods).
7. Canonical binary form of the sim part, hashed as `sim_rules_hash` (BLAKE3 or SHA-256). `render_rules_hash` is separate and informational.

- Every value records the layer, file and line that set it; `rules explain HTNK.armament.primary.weapon` prints the chain. Errors point at source spans.
- **Replay and lobby headers pin:** engine build, converter version, input file hashes, mod hashes, lobby options, `sim_rules_hash`. On mismatch the lobby names which input differs, never values.
- The resolved ruleset is cached as a compact binary blob (e.g. postcard).
- Hot reload is development-only, swaps the registry at a tick boundary and marks the session "dev-modified". An egui tuning panel writes a `dev-overrides.jsonc` patch layer.
- Schema versions are numbered; mod manifests declare `schema: N`; migrations upgrade old patches. The converter version is separate from the schema version.

**Art** goes through the same converter. **Sim-relevant fields are hashed** (`Foundation`, the `*FireFLH` firing offsets, animation lengths that gate timing). **Render-only fields** (SHP/VXL choice, sequences, palettes, cameos, ZFudge) stay out of the hash. A field of unclear use counts as sim until a spec rules on it.

**Extensibility model**
- **Four fixed kinds** (Infantry, Vehicle, Aircraft, Building) plus **optional capability components** with data parameters. Behaviour is Rust code. No OpenRA-style traits-for-everything and no general ECS (archetype iteration order is a determinism risk). Objects live in a generational arena with sorted iteration.
- **Tagged enums** in data for genuinely different behaviours: `locomotor.type`, `superweapon.kind`, `warhead.effects[]`.
- **Modding tiers:** (1) data mods as patch layers against stable IDs, shipped as diffs, never full rulesets (v1); (2) code mods as Rust components behind a cargo feature, changing the engine build hash; (3) sandboxed scripts only if a real mod needs them: wasmi 2.x with fuel metering, or Rhai with `no_float` (needs its `wasm-bindgen` feature on wasm32-unknown-unknown); mlua is ruled out (wasm only via emscripten).
- **Fidelity by default.** The *reference ruleset* is the converted install + engine defaults + `quirks.*` set to YR behaviour. A generalised component must reproduce YR exactly when filled with converted values (a generalised slaver gives the Slave Miner). The fidelity suite runs on the reference ruleset only.
- **Coverage.** A generated `coverage.csv` lists every (category, key) pair in the stock rulesmd, artmd, globals, mode files and map override keys, names only. Each row gets one outcome: `field`, `folded`, `render`, `ai-hint` (hashed, because the AI runs in the tick), `ignored`, `unsupported`. A first inventory of the base rulesmd found 1,021 pairs in 16 categories plus 164 global sections; **re-run on the 1.001 copy before freezing the schema.**

**Component sketch** (indicative; `coverage.csv` makes it complete)
- **Globals:** economy, production, combat (11 armor classes), veterancy, movement (jumpjet controls, bridges), power, shroud, quirks.
- **Every object:** health and armor, cost, tech and prerequisites (sets), sight, threat and acquisition, immunities, veterancy, self-heal, death weapon, crushable, size, cloak and sensors, AI hints.
- **Combat:** weapon-slot map (primary, secondary, elite, occupy, indexed slots for Gattling and IFV); weapon; projectile; warhead (Verses over 11 armor classes in per-mille; spread; tagged effects). The slot-selection policy, including the special 0/1/2% Verses, is code with quirk switches.
- **Locomotors:** drive, walk, hover, ship, fly, jumpjet, teleport, rocket.
- **Capabilities:** transport (FV, BFRT, LCRF, SAPC, YHVR, SHAD and others), gunner (IFV), garrisonable/occupier, bunker (Tank Bunker), absorber (Bio Reactor; infantry only in stock), grinder, cloning, mind_controller (incl. Mastermind overload), slaver, spawner, parasite, drainer/drainable (Floating Disc), gattling, deployer (MCVs and Slave Miner deploy into buildings; GI, Guardian GI, Desolator, Yuri, Yuri Prime, Chaos Drone deploy in place; the Siege Chopper's deploy needs its own spec), teleporter, harvester/refinery/dock, produce_cash, factory, cost_bonus, power (incl. robot-control link), radar/spy_sat/gap_generator/psychic_detection, infiltratable/infiltrator, capturable/repairable, wall/gate, secret_lab, airstrike_designator, foundation. Building upgrades look vestigial in stock and start as `ignored`.
- **Superweapons:** a tagged enum covering the 12 stock entries.
- **Cross-component interactions** live in code, each with a scenario test (a controlled unit in a transport, grinding a controlled unit, a parasite on a chronoshifted unit, Iron Curtain on infantry).

**Maps.** `.map`/`.mpr`/`.yrm` are imported and converted locally; stock maps referenced by content hash. Our container holds `map.jsonc` (metadata, objects, houses, waypoints, lighting, triggers, optional rules patch) plus a binary `terrain.bin`. Tile references stay as (theater, tile index, sub-tile). Trigger codes become named variants of our trigger schema. Conversion is deterministic (file order, no hash-map iteration). World-Altering Editor (GPL-3+) authors test maps through the same converter.

### 4.7 Own AI

- **Layers:** strategy (build order, goals); economy (harvesting, expansion, power); army composition; tactics (targets, retreat, superweapons).
- **Human rules:** issues only Orders a human could, sees only its player's shroud. No APM limits, reaction delays or resource cheats at any difficulty.
- **Difficulty:** the top level is uncapped. Lower levels use simpler strategy modules (fewer goals, simpler build orders and tactics), never APM or reaction caps.
- **Deterministic:** runs inside the tick, uses the sim RNG, reads the hashed `ai-hint` data.
- **Build-up:** (1) scripted builder and attack waves; (2) reactive economy and defence; (3) tactics and superweapons.
- Tuned against itself and against the project lead. Matching the original AI is not a goal.
- **Scope switches:** any temporary engine-scope switch applies to all players equally, is labelled as engine scope and removed at the next milestone. It is never an AI cap.

### 4.8 Networking (after skirmish fidelity)

- Lockstep: orders execute at tick N + delay, adaptive delay, an empty orders frame every tick.
- State hash every N ticks; on desync, dump the last frames and produce a readable per-object diff.
- WebSocket relay first (also the lobby), WebRTC via matchbox second with relay fallback.
- Late join and reconnect from a snapshot. Replays pin the build; old builds stay hosted at versioned URLs.
- Ranked play out of scope (every lockstep client sees the full state).

## 5. Milestones

**Effort basis.** Focused person-months (pm), **1 pm = 160 h**. At 10–15 h/week, 1 pm takes 160/15 ≈ 10.7 weeks to 160/10 = 16 weeks, i.e. about **2.5–3.7 calendar months**.

**How the estimate was set.**
- The spec side starts now, so decompiled specs are available from M0. The INI parse semantics needed in M1 and the rendering details needed in M1–M2 (voxel facings, HVA scaling, house-colour ramp) come straight from S-specs, confirmed by measurement.
- The earlier draft's M3–M7 ranges already assumed decompiled specs for tick order, rounding, the RNG and interaction rules (measurement shrinks from *discovery* to *confirmation*). Re-checked against that, they stand unchanged. Dropping the "no decompiler" contingency removes its +2–3 pm; nothing else moves.
- The spec pipeline (sandbox, lint, scrubber, sign-off) and the converter add work, mostly in M0–M1, plus about 10–15% overhead on every mechanics milestone; this is included in the ranges.
- Music sits in M8 (about 0.25 pm, outside the M0–M7 total).

**Totals (M0–M7):** low 1 + 2.5 + 1 + 2.5 + 1.5 + 3 + 2 + 2.5 = **16 pm**; high 1.5 + 3.5 + 2 + 3.5 + 2.5 + 4.5 + 3.5 + 3.5 = **24.5 pm**. Calendar: 16 pm × 160 h / 15 h/week ≈ 171 weeks ≈ **3.3 years**; 24.5 pm × 160 h / 10 h/week = 392 weeks ≈ **7.5 years** to the first full YR skirmish.

| M | Deliverable | Exit criterion | Effort (pm) |
|---|---|---|---|
| M0 | **Charter and spec pipeline:** CLAUDE.md, deny rules and web blocks for implementers; PROVENANCE, CONTRIBUTING; spec template, scrubber lint and denylist, sign-off log; CI with determinism, citation and "no stock numbers" lints; synthetic ruleset skeleton and fixture writers. **Oracle bootstrap:** stock-hash check, director and mods off; an independent screen-capture/OCR measurement path; measurement schema. | CI green, including the WebGL2 browser lane. A textured quad renders on Firefox/WebGL2, and on WebGPU where available. One pilot spec has gone analyst → lint → scrub → sign-off → implementer end to end. Measurement #1 and about 10 others recorded. | 1–1.5 |
| M1 | **Formats, VFS and importer:** `ini` reader with YR parse semantics; converter, schema crate, schemars output, resolve pipeline with spans and `rules explain`; `coverage.csv` on the 1.001 rulesmd and artmd with the art sim/render split. **Asset viewer:** SHP and voxels with house colour, tiles, SFX and voices, fonts. First public release candidate. | Imports a Steam install in Firefox (through the Flatpak portal) and Chromium. The expandmd01 rulesmd replaces the base copy. Loose files flagged. Parses 100% of SHP frames in conqmd and VXL limbs in ra2/ra2md. Every IDX sample plays. Every stock key has a coverage outcome. Parsers and JSONC parser fuzzed. | 2.5–3.5 |
| M2 | **Map viewer and map converter:** IsoMapPack5, overlays, heights, static objects, lighting, minimap; cached converted maps. | Every stock multiplayer map converts and renders without panic. Spot checks against original screenshots per theater. 60 fps scrolling on WebGL2. | 1–2 |
| M3 | **Sim core:** fixed point with per-field conversion rules; RNG (original algorithm); tick pipeline; occupancy, Drive/Walk locomotion, pathfinding (from S-specs, behind a swappable interface); Orders, `sim_rules_hash`, replay, save/load, headless runner. | Native and wasm hashes identical over a long scripted replay. Save/load round-trip gives identical hashes. Speeds and turn rates match specs exactly and agree with the oracle (±1 tick only where the measurement is quantised). | 2.5–3.5 |
| M4 | **Combat:** weapons, warheads, Verses (incl. 0/1/2% semantics), projectiles, veterancy, acquisition, slot selection. | ≥ 40 relation tests (ROF, damage per armor, rounding), each citing an S-spec and confirmed by an M-id. | 1.5–2.5 |
| M5 | **Economy, production, sidebar and shell UI** (layout from screenshots, visibly original fallback); shroud; **minimal own AI**; game-mode layer; first SFX/EVA. | **First playable game:** a full 1v1 against our own AI on 2 stock maps. Build times and harvest cycle match specs and oracle. | 3–4.5 |
| M6 | **Allied and Soviet completeness:** air, naval, Hover, Jumpjet, Teleport, Rocket; bridges, garrisons, engineers, spies, crates; superweapons, tech buildings; multiplayer trigger subset. | Naval maps playable. Every superweapon has a scenario test. Stock multiplayer maps run their triggers without "unimplemented" warnings, apart from flagged script maps. | 2–3.5 |
| M7 | **The Yuri faction:** mind control and Mastermind overload; Slave Miner, Grinder, Cloning Vats, Bio Reactor, Tank Bunker; Magnetron, Floating Disc, Gattling; Genetic Mutator, Psychic Dominator, Force Shield. | Every interaction pair has a scenario test on the synthetic ruleset and a local relation test on the reference ruleset. Arena win rates within tolerance of the stock oracle. Full YR skirmish against our own AI. | 2.5–3.5 |
| M8 | **Fidelity pass, AI depth, polish:** turning `unmeasured` and `measurable` claims into measured ones; movement and crowding feel; accessibility, settings; low-end iGPU performance; optional WebGPU extras; **music** (IMA-ADPCM WAV decoder, playlist, OPFS share). | Statistical suites green nightly. Playtester survey. | ongoing |
| M9 | **Multiplayer:** relay, lobby, lockstep, desync reports, then WebRTC. | 2v2 across networks for 30+ minutes without desync. A forced desync produces a readable diff. | 2–3 |
| M10 | **Campaign (optional):** full trigger and script set, briefings, fog as a map flag, movies. | Every campaign mission completable. | 6–12+ |

## 6. Fidelity and testing

### 6.1 Fidelity tiers

| Tier | Claim | Scope | Validation |
|---|---|---|---|
| **(a) Exact** | Same integer results | Deterministic single scenarios: ROF, damage incl. rounding, build time with factories and power, speeds, turn rates, harvest cycle, superweapon charge. Formulas and rounding from specs. | Measurement confirms each (±1 tick only where the measurement is quantised) |
| **(a′) Exact where cheap** | Original RNG algorithm and seeding; quirks as `quirks.*` defaults; type registration order for tie-breaks | Wherever a spec gives it and the cost is local | Unit tests against the spec; oracle spot checks |
| **(b) Statistical** | Distributions match | Unit-arena outcomes, time-to-kill, anything depending on random draws in a long fight | Two-proportion and KS tests across seeds |
| **(c) Not claimed** | Bit-exact replay of, or lockstep netplay with, the original game | — | — |

**Why tier (c) stays out:** bit-exactness would need x87 extended-precision emulation, the original's container iteration orders, pathfinding tie-breaks and every bug. Specs would have to describe the binary's control flow step by step, which the spec rules forbid, and the result could not be measured. YR has no usable native replay format, so the only real target would be lockstep with original clients, which is not a goal. Our own engine is bit-deterministic across its own builds.

### 6.2 Test layers

**Layer 0: formats and rules.** Synthetic fixtures with round-trip and property tests; cargo-fuzz on all parsers including JSONC; (optional, run by the project lead outside implementer sessions) third-party MIX/SHP crates as differential oracles; schema and resolve tests on the synthetic ruleset; a local-only lane against the real install (never in CI, never committed).

**Layer 1: determinism.** Golden replays with per-tick hashes; cross-target gate (native x86 and ARM vs wasm); save/load round-trips; nightly AI-vs-AI soak tests.

**Layer 2A: exact oracle tests.**
- Each S-spec proposes a scenario. The oracle operator runs it on the stock game with self-authored micro-maps (World-Altering Editor or the scenario DSL).
- The result is a measurement row (scenario, settings, value, n, method, video hash) with an M-id.
- Public tests assert the *relation* plus tolerance; measured values stay local.
- A spec that fails its measurement goes back to the spec side. **The measurement wins.**

**Layer 2B: statistical oracle tests.** Unit-arena win rates and time-to-kill across seeds (not curves of the original AI).

**Oracle.** The ra2-yr-setup bench and arena may be run on the **stock** game (stock data, director off, mods off) as a measurement instrument. Only observables cross into this repo as M-records: win rates, timings, ticks, credits, counts, cells, health, win/loss. Cross-check against the OCR path, and over time make the OCR path primary.

**Regression policy.** A golden relation changes only with a new spec revision or M-id. Tolerances widen only with written justification.

**Layer 3: browsers.**

| Lane | Status | Setup |
|---|---|---|
| WebGL2 | **Mandatory from M0** | Playwright on Chromium, Firefox and WebKit with software WebGL2 (SwiftShader/ANGLE). Import tests via `setInputFiles` on a synthetic folder. |
| WebGPU | Optional, non-blocking | Chromium with `--enable-unsafe-webgpu` where the runner supports it; covers feature-detected extras and the backend switch. |
| Manual, before each release | — | Firefox on real hardware (WebGL2); a flagged Chromium for WebGPU extras once any exist; native Vulkan build with RenderDoc/RGP for GPU issues. |

## 7. Top risks

| Risk | Mitigation |
|---|---|
| Solo burnout or scope collapse | Every milestone useful on its own (viewer, map viewer). Devlog per milestone. A hard cut list. |
| EA takedown (a non-commercial RA2 VR remake was refused in 2016) | Zero assets and zero EA numbers, neutral name, disclaimer, donations only, low profile until M5, portable static hosting. |
| Raw spec-side material leaking into `glasnost` | Raw notes never leave `~/glasnost-spec-dirty`; scrubber lint, denylist and sign-off on every spec; code cites opaque S/M IDs; implementer deny rules and web blocks with logs. |
| Specs that mirror the binary's code structure | Spec style rules, lint, denylist, scrubber session, two-implementations test, sign-off. |
| AI regurgitation of YRpp/Phobos/Ares code; shared training exposure | Implementers are asked only to "implement S-xxxx", never given the original's internal names. Periodic JPlag and identifier-overlap checks. |
| The owner's dual role weakens the split | The owner never implements. Every implementation-side instruction cites an ID. Exceptions logged. |
| Movement and pathing "feel" | Pathing specs plus debug overlays from M3, scenario suite, side-by-side video, swappable pathfinder. |
| YR ownership and container edge cases | One subsystem at a time, explicit hooks for owner changes, a test per interaction pair, quirk switches. |
| Original uses floats at sim-relevant sites | Per-site `Real` wrapper (no FMA, no relaxed SIMD) or integer emulation; tier (a) claimed only where it matches. |
| Own AI too weak or predictable | Layered design, deterministic AI-vs-AI harness, difficulty via strategy modules without cheats. |
| Undocumented UI and shell behaviour | Specs plus screenshots, budgeted as its own workstream; integer scaling from 800×600; original fallback art. |
| Determinism bugs | Lints, cross-target CI, save/load hash test, sync-diff tool. |
| Install variance (Steam, Origin, CD, languages, CnCNet, modded) | Content-hash table, stock-only default, warnings for unknown expansion files, refusal without the YR expansion; Ares/Phobos keys reported as unsupported; no mods hosted. |
| Browser limits (WebGPU reach, Flatpak sandbox, Safari eviction, tab throttling, Ctrl-key collisions) | WebGL2 core, verify the portal, cheap re-pick each session, worker clock with catch-up, remappable keymap. |

## 8. Key references

### 8.1 Implementer-readable

- OpenRA (format/loader code only, with attribution): https://github.com/OpenRA/OpenRA
- XCC (format/loader code, with attribution): https://github.com/OlafvdSpek/xcc
- World-Altering Editor (map authoring tool): https://github.com/CnCNet/WorldAlteringEditor
- RFC 7396 JSON Merge Patch: https://datatracker.ietf.org/doc/rfc7396
- schemars: https://docs.rs/crate/schemars/latest
- serde_json `arbitrary_precision` issues: https://github.com/serde-rs/json/issues/845
- wasmi: https://docs.rs/crate/wasmi/latest ; Rhai features: https://rhai.rs/book/start/features.html ; mlua: https://github.com/mlua-rs/mlua
- WebGPU status: https://github.com/gpuweb/gpuweb/wiki/Implementation-Status
- Chrome WebGPU news: https://developer.chrome.com/docs/web-platform/webgpu/news , https://developer.chrome.com/blog/new-in-webgpu-144 , https://developer.chrome.com/blog/new-in-webgpu-147-148
- Chrome WebGPU troubleshooting: https://developer.chrome.com/docs/web-platform/webgpu/troubleshooting-tips
- Reach: https://web3dsurvey.com/webgpu , https://web3dsurvey.com/webgl2
- wgpu 30.0.1 GLES backend: https://github.com/gfx-rs/wgpu/blob/v30.0.1/wgpu-hal/src/gles/adapter.rs
- wgpu WebGPU detection: https://docs.rs/wgpu/latest/wgpu/util/fn.new_instance_with_webgpu_detection.html
- WASM nondeterminism: https://www.webassembly.org/docs/nondeterminism/
- Blowfish crate: https://docs.rs/blowfish/latest/blowfish/
- matchbox: https://github.com/johanhelsing/matchbox
- Storage eviction: https://developer.mozilla.org/en-US/docs/Web/API/Storage_API/Storage_quotas_and_eviction_criteria
- `webkitdirectory`: https://developer.mozilla.org/en-US/docs/Web/API/HTMLInputElement/webkitdirectory
- Steam movie corruption: https://steamcommunity.com/app/2229850/discussions/0/7056650139344818877

### 8.2 Spec side and project lead only (implementers do not open these)

- ModEnc: https://modenc.renegadeprojects.com/
- ModdingWiki MIX format: https://moddingwiki.shikadi.net/wiki/MIX_Format_(Westwood)
- OpenRA/ra2 and OpenRA rules/network design: https://github.com/OpenRA/ra2 , https://github.com/OpenRA/OpenRA/blob/bleed/OpenRA.Game/Network/SyncReport.cs , https://raw.githubusercontent.com/OpenRA/OpenRA/bleed/mods/ra/rules/vehicles.yaml
- ra2-mix crate (differential oracle): https://crates.io/crates/ra2-mix
- Chrono Divide (UX reference only; its client is banned on both sides): https://chronodivide.com , https://github.com/chronodivide/mod-sdk
- Rosebud's OpenRA browser port: https://lab.rosebud.ai/engineering/porting-openra-to-the-browser
- EA source releases: https://github.com/electronicarts
- EA modding guidelines: https://www.ea.com/games/command-and-conquer/news/modding-faq
- 2016 RA2 VR refusal: https://uploadvr.com/red-alert-2-vr-remake-canceled/
