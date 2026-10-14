# Glasnost

An open-source reimplementation of the game engine for **Command & Conquer: Yuri's Revenge**, written in Rust. It targets the browser (WebAssembly, WebGL2 with optional WebGPU extras) and native desktop builds.

> Compatible with game data from *Command & Conquer: Yuri's Revenge* (not included).

## Status

**Milestone M5 (first playable game) is done.** You can play a 1v1 skirmish against our own computer opponent on stock multiplayer maps, natively and in the browser. A game runs from the main menu through skirmish setup (map, sides, countries, colours, starting credits and units) to the end screen and back. It has:

- the economy: ore and gems, harvesting (slave miners included), refineries and power;
- base building: production queues, placement, selling and repair, and the tech tree;
- movement, pathfinding and combat for ground units and buildings;
- shroud and radar;
- the multiplayer subset of map triggers;
- victory and defeat.

Sound effects and announcements are read from your files and play through WebAudio on the web.

The opponent plays every faction the setup offers. It uses only the orders a person could give and sees only its own shroud. The in-game sidebar and HUD have a layout of our own; the cameos, cursors and other art come from your files.

The simulation is deterministic: integers only and one seeded random stream. Native and WebAssembly builds produce identical state hashes, and saves restore the same hash. Behaviour is implemented only from the specs in `spec/`. Where a spec is missing, a request is filed in `spec/requests/` and the interim rule is listed in `docs/development.md`.

**Not there yet (M6 and later):**
- aircraft, naval units, garrisons, bridges and superweapons;
- full animation playback;
- multiplayer over the network.

## You need your own copy of the game

This repository contains **no EA game assets**: no graphics, sounds, maps, rules values or executables. At runtime the engine imports these from an install of Yuri's Revenge that you own, and everything it imports stays on your machine. Nothing derived from the game data is uploaded or redistributed.

## Building

Rust is pinned in `rust-toolchain.toml` (rustup installs it on first use).

```sh
cargo run --release -p glasnost-platform-native -- --game --game-dir <your install>   # the game (menu, skirmish)
cargo run -p glasnost-platform-native   # native renderer demo (Vulkan/Metal/DX12/GL)
cargo run -p glasnost-platform-native -- --viewer --game-dir <your install>   # asset viewer
cargo xtask web --serve                 # web demo at http://127.0.0.1:8000/ (needs wasm-bindgen-cli, see docs/web.md);
                                        # add ?mode=game for the game or ?mode=viewer for the asset viewer,
                                        # then pick your install folder
cargo test --workspace                  # tests
cargo xtask check                       # dependency rule, determinism, citations, no stock numbers
```

## Documents

- [PLAN.md](PLAN.md): architecture, milestones and decisions.
- [PROVENANCE.md](PROVENANCE.md): how behaviour specs are produced and how they reach the code (the two-team process).
- [CONTRIBUTING.md](CONTRIBUTING.md): workflow, determinism rules, data rules and commit sign-off.
- [docs/development.md](docs/development.md): workspace layout, repository checks, CI. [docs/web.md](docs/web.md): the web build and browser tests.
- [spec/](spec/README.md): behaviour specs. [measurements/](measurements/README.md): measurement records from the original game.

## Disclaimer

This is an unofficial fan project. It is not affiliated with, endorsed by or supported by Electronic Arts Inc. Command & Conquer, Red Alert and Yuri's Revenge are trademarks of Electronic Arts Inc.; they are used here only to describe compatibility.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
