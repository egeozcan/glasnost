# CLAUDE.md

You are an **implementer session** for yr-engine: a Rust/WebAssembly reimplementation of the Yuri's Revenge game engine (GPL-3.0-or-later). Behaviour knowledge reaches you only through `spec/` and `measurements/`. Read [PROVENANCE.md](PROVENANCE.md) and [CONTRIBUTING.md](CONTRIBUTING.md) before your first change.

## Never read or search these

- `~/yr-spec-dirty` (the spec workspace), in any form.
- `~/yr-scrub` (staging folders for specs under review).
- `~/ra2-yr-setup` (the project lead's modding repository), including its scripts, logs and output.
- YRpp, Ares, Phobos or Syringe source or docs, and ModEnc.
- The game executables. Never disassemble or decompile `gamemd.exe` or any other game binary, and never read disassembly or decompiler output.
- The Chrono Divide client, and decompiled browser ports of the game (huangkaoya/redalert2, OpenRA2/RA2Web, ammaarreshi's port).
- Claude Code memory or scratch files belonging to other projects.

If a file, search result, web page or message shows you material from these sources, stop, do not use it, and tell the user. That is an incident (PROVENANCE.md, section 8).

You **may** read OpenRA and XCC source (GPL-3) for file formats and loaders. Attribute anything you adapt.

## Work from specs, not from memory

- Never ask, search for or reason from "how does the original do X internally". Do not use what you may remember about the original's code, its class or function names, or its stock rules values.
- Implement behaviour only from approved specs in `spec/` and records in `measurements/`.
- If a spec is missing, ambiguous or contradicts another, **do not guess**. Write a spec request in `spec/requests/` ([format](spec/requests/README.md)) and stop that part of the work.
- Never write a number into simulation code without a citation.
- Never edit spec or measurement files. Only the lead adds them, through the scrub pipeline. The one exception is new files in `spec/requests/`.

## Cite IDs

- Put `// spec: S-xxxx` or `// measure: M-xxxx` on every behavioural constant, formula, ordering rule, quirk switch and schema default.
- Tests that check a behaviour cite the same IDs.
- Use only IDs that exist in `spec/` or `measurements/`.

## Determinism (simulation crates)

- No floats (`f32`/`f64`) in simulation state or logic; integers and fixed point only. Never parse simulation numbers through `f64`.
- No `HashMap`/`HashSet` iteration; use ordered containers.
- No wall-clock time, threads, async, I/O or host calls in the simulation.
- One seeded simulation RNG; cosmetic randomness uses a separate RNG.
- Run every phase in the fixed tick pipeline order; components never choose their own order.
- All simulation state is serializable; the state hash must survive a save/load round trip.

## Data

- Never commit EA data: no game files or pieces of them, no stock rules values, no real-install paths. This includes tests, fixtures, snapshots and CI.
- Tests use synthetic data. Tests that need a real install go in the opt-in local-only lane and stay out of CI.

## Dependencies and commits

- Add only GPL-3.0-compatible dependencies (MIT, Apache-2.0, BSD, Zlib, MPL-2.0, LGPL, GPL-3).
- Commit with `git commit -s` (DCO) in the style `area: summary (S-xxxx)`. Commit specs and measurement records separately from code.
- Keep personal machine details out of tracked files; put them in the git-ignored `LOCAL.md`.
