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

**M6a (first wave of M6) adds** aircraft and air pads, naval units (ships move on water, wake, sink, and are built at naval yards), the V3-style guided missile launcher, and the parasite (a drone that latches inside a vehicle). Carriers and their spawned craft fight. The computer opponent builds aircraft and pads, navies on maps with water, and missile launchers.

**M6b (second wave of M6) adds** hover and amphibious movement over water, transports (units board, ride and unload; the opponent ferries infantry across water), bridges (destruction from the hut, collapse, repair by an engineer), cloaking and stealth (cloaked units, cloak fields, sensors), walls and gates, the teleporting locomotor, the repair depot and the ship-repair yard, the superweapons (nuclear strike, weather storm, iron curtain, chronosphere, force shield, spy plane and paradrop, with the sidebar icons, aiming and radar launch), parachutes and paradrops, the remaining multiplayer map trigger actions, and animation playback. The opponent uses transports and every superweapon kind it can aim, hover units on water maps, and repair depots. Aircraft cast their shadow on the ground.

**M6c (last part of M6) adds** garrisons (infantry occupy civilian buildings and fire from them, and are evicted), engineers capturing buildings, spies infiltrating them, tech buildings (captured for their income and effects), jumpjet infantry and vehicles (they rise from their factory, fly and land), deployable units (in-place deploy, deploy-fire vehicles, deployed infantry), radiation fields, chrono weapons (temporal erasure), linked weapons (prism support beams and chargers), terror weapons (planted bombs, charges, suicide attacks, attack animals, air-strike designation) and crates with their power-ups. The opponent uses every one of these the way a person could, sets rally points on its barracks, and builds every unit class its side can build, jumpjet units included.

**M6 is done.** In the local lane our AI plays itself to a winner on every stock map run (0, 2, 3, 5, 6, 7 and 8, the naval maps 0, 3 and 5 included), Allied against Allied and Allied against Soviet on all seven and Soviet against Soviet on maps 2 and 8, and every game's replay and mid-game save reproduce it. The determinism gate replays an AI-against-AI game that exercises every M6c ability, natively and in WebAssembly.

**M7a (first part of M7) makes the Yuri side playable.** It adds psychic weapons (units driven berserk), the psychic sensor and the psychic reveal, mind control (taking over enemy units, the controller's limit and overload, release when the controller dies), gattling weapons that spin up and down, gas weapons and their particle clouds, the bio reactor that takes in infantry, the grinder that recycles units for credits, the tank bunker, and the magnetic lift that seizes vehicles and drops them. The opponent plays Yuri and uses each of these the way a person could. In the local lane, Yuri plays Allied and Soviet on maps 0, 2 and 8 to a winner, and every game's replay and mid-game save reproduce it. The determinism gate replays an AI-against-AI game with a Yuri house, natively and in WebAssembly.

**M7b (last part of M7) completes the Yuri side.** It adds:
- the floating disc's drain, which takes credits from refineries and power from power plants;
- the genetic mutator, which turns infantry in the blast into the house's own new infantry;
- the psychic dominator, which makes units in the blast change sides and dims the map's light while it charges and fires;
- the special hit effects and the rest of the particle systems.

The opponent drains enemy refineries and power, defends its harvesters and power against raiders and drainers, and builds and fires both new superweapons. It aims each by the value of what it would take or destroy. In M7 the dominator only did area damage in the full games on stock data; M8 makes it aim where it can take units. A wealthy house can also drive a second construction yard to a distant ore field.

In the local lane, Yuri plays a full game against Allied and against Soviet on maps 2, 8 and the naval map 0 to a winner, and every game's replay and mid-game save reproduce it. The determinism gate replays an AI-against-AI game in which the drain, the mutator and the dominator are each used several times, natively and in WebAssembly. Every interaction pair has a scenario test on synthetic data and a relation test on the installed data. 37 of the 42 relation tests pass. The other 5 skip because the installed data cannot form their pair. An arena mode (`glasnost-headless arena`) compares our combat with the committed measurement records using statistical tests.

**M7 is done except for its arena check**, which continues in M8. Many matchups agree with the records on the winner, but some measured pairings still fail a statistical test, mostly because our games vary less from seed to seed than the original's.

**M8 (fidelity, AI depth and polish) is about half done.** So far it adds:
- animations as part of the simulation: their damage over time, their random loop delays and follow-ups, and their removal now play the same on every machine;
- closer combat randomness: the disguise draw and the corpse and death-animation picks;
- an opponent that expands with a second construction yard in full games, saves for its superweapon without losing as much tempo, and aims the psychic dominator where it can take units;
- a settings screen: sound, voice and music volume, scroll speed and key bindings, kept between sessions natively and in the browser.

The arena now fails 49 of 155 measured pairings, down from 55. New spec revisions are taken in as they land. Still to come in M8: a colour-blind option and other accessibility work, music, performance on low-end laptops, and electric bolts.

**M9 (multiplayer over the network) has started.** It adds:
- a lockstep core: every machine runs the same simulation from the same orders and compares state hashes;
- a relay server (`glasnost-relay`) with rooms and room codes;
- a *Multiplayer* entry on the main menu: host or join a room, set the game up together (map, countries, colours, teams, starts, computer players), chat, and play.

Two native clients on one machine play the same game over the relay. The browser client builds but has not been tried yet. Still to come in M9: desync reports, reconnecting, a long soak test, direct WebRTC links and a guide to playing across the internet.

**Not there yet:**
- the rest of M8 and M9;
- the campaign (M10, optional).

## You need your own copy of the game

This repository contains **no EA game assets**: no graphics, sounds, maps, rules values or executables. At runtime the engine imports these from an install of Yuri's Revenge that you own, and everything it imports stays on your machine. Nothing derived from the game data is uploaded or redistributed.

## Building

Rust is pinned in `rust-toolchain.toml` (rustup installs it on first use).

```sh
cargo run --release -p glasnost-platform-native -- --game --game-dir <your install>   # the game (menu, skirmish)
cargo run --release -p glasnost-relay -- --listen 127.0.0.1:8787   # multiplayer relay (the game's default: --relay ws://127.0.0.1:8787/)
cargo run -p glasnost-platform-native   # native renderer demo (Vulkan/Metal/DX12/GL)
cargo run -p glasnost-platform-native -- --viewer --game-dir <your install>   # asset viewer
cargo xtask web --serve                 # web demo at http://127.0.0.1:8000/ (needs wasm-bindgen-cli, see docs/web.md);
                                        # add ?mode=game for the game or ?mode=viewer for the asset viewer,
                                        # then pick your install folder
cargo test --workspace                  # tests
cargo xtask check                       # dependency rule, determinism, citations, no stock numbers
```

## Reporting a pathing problem

When a unit takes a strange route or gets stuck, press **M** (mark) as it happens, then **P** (save pathing report) within 20 seconds. Both keys can be changed on the settings screen. The native client writes the report to `path-reports/path-<tick>-<n>.gpath` in the folder you started it from; the browser downloads it. The status line names the file. Run `glasnost-headless path-report <file>` on it to replay the last 20 seconds and explain what each selected or ordered unit did ([docs/development.md](docs/dev/play-notes.md#pathing-reports-m8-a4)). A report holds a save of your game, so keep it out of git: `path-reports/` and `*.gpath` are ignored.

## Documents

- [PLAN.md](PLAN.md): architecture, milestones and decisions.
- [PROVENANCE.md](PROVENANCE.md): how behaviour specs are produced and how they reach the code (the two-team process).
- [CONTRIBUTING.md](CONTRIBUTING.md): workflow, determinism rules, data rules and commit sign-off.
- [docs/multiplayer.md](docs/multiplayer.md): run the relay, host and join a network game, direct links, reconnecting, desync reports, and the cross-network exit run.
- [docs/development.md](docs/development.md): workspace layout, repository checks, CI. [docs/web.md](docs/web.md): the web build and browser tests.
- [spec/](spec/README.md): behaviour specs. [measurements/](measurements/README.md): measurement records from the original game.

## Disclaimer

This is an unofficial fan project. It is not affiliated with, endorsed by or supported by Electronic Arts Inc. Command & Conquer, Red Alert and Yuri's Revenge are trademarks of Electronic Arts Inc.; they are used here only to describe compatibility.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
