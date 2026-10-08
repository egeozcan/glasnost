# yr-engine

*Working name. The final project name is not chosen yet.*

An open-source reimplementation of the game engine for **Command & Conquer: Yuri's Revenge**, written in Rust. It targets the browser (WebAssembly, WebGL2 with optional WebGPU extras) and native desktop builds.

> Compatible with game data from *Command & Conquer: Yuri's Revenge* (not included).

## Status

**Planning. There is no code yet.** The first goal is skirmish against our own AI with the Allied, Soviet and Yuri factions, with the mechanics replicated closely. Multiplayer comes once the mechanics are right. The simulation is deterministic and lockstep-ready from the start.

## You need your own copy of the game

This repository contains **no EA game assets**: no graphics, sounds, maps, rules values or executables. At runtime the engine imports these from an install of Yuri's Revenge that you own, and everything it imports stays on your machine. Nothing derived from the game data is uploaded or redistributed.

## Documents

- [PLAN.md](PLAN.md): architecture, milestones and decisions.
- [PROVENANCE.md](PROVENANCE.md): how behaviour specs are produced and how they reach the code (the two-team process).
- [CONTRIBUTING.md](CONTRIBUTING.md): workflow, determinism rules, data rules and commit sign-off.
- [spec/](spec/README.md): behaviour specs. [measurements/](measurements/README.md): measurement records from the original game.

## Disclaimer

This is an unofficial fan project. It is not affiliated with, endorsed by or supported by Electronic Arts Inc. Command & Conquer, Red Alert and Yuri's Revenge are trademarks of Electronic Arts Inc.; they are used here only to describe compatibility.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
