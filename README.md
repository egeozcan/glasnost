# Glasnost

Glasnost is a free, open-source game engine that plays **Command & Conquer: Yuri's Revenge** with the game files you already own. It runs in your web browser or as a desktop program.

It comes with its own computer opponent that plays all three sides, multiplayer over the internet, optional fixes for units that get stuck, and touch and touchpad controls. Glasnost is still in development. The campaign is not playable yet, and some things are rough (see [What works and what doesn't yet](#what-works-and-what-doesnt-yet)).

## You need the original game

Glasnost contains none of the game: no graphics, sounds, maps, rules or programs. You need your own installed copy of Yuri's Revenge.

You point Glasnost at your game folder, the one with the game's `.mix` archive files directly inside it. Glasnost reads those archives and nothing is uploaded anywhere. It never changes your game folder.

## Play in your browser

There is no hosted version yet, so you build the web version yourself and run it on your own computer. You need the same tools as for the desktop version: install Rust as described in [Play on your desktop](#play-on-your-desktop) and get the source code. Then, in the source folder:

```sh
cargo install wasm-bindgen-cli --version 0.2.129 --locked   # once
cargo xtask web --release --serve
```

When it says it is serving, open **http://127.0.0.1:8000/?mode=game** (the address without `?mode=game` shows a test pattern instead of the game).

1. Click **Game folder** on the main menu and pick your game folder. The browser reads the files where they are; nothing is copied off your computer.
2. Wait until **Skirmish** can be clicked, then start a game.

Sound starts with your first click or key press, since browsers only allow sound after you do something on the page. You pick the game folder again on each visit. To leave, close the tab: the browser version has no Quit button.

**Browsers.** You need a browser with WebGL2. Firefox and Chromium-based browsers are the ones we test. Safari is not tested for play.

## Play on your desktop

1. **Install Rust.** On Linux or macOS, run the installer from [rustup.rs](https://rustup.rs):

   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

   On Windows, download and run `rustup-init.exe` from [rustup.rs](https://rustup.rs). The project picks its exact Rust version by itself the first time you build.

2. **Get the source code.**

   ```sh
   git clone https://github.com/egeozcan/glasnost.git
   cd glasnost
   ```

3. **On Linux, install the sound library's development package** (the game plays its sound through ALSA, which PipeWire and PulseAudio serve):

   ```sh
   sudo dnf install alsa-lib-devel      # Fedora
   sudo apt install libasound2-dev      # Debian and Ubuntu
   ```

   Windows and macOS need nothing extra.

4. **Start the game**, giving it your game folder in place of `GAME_FOLDER`:

   ```sh
   cargo run --release -p glasnost-platform-native -- --game --game-dir "GAME_FOLDER"
   ```

   The first build takes a while. After that, you can also start the built program directly: `target/release/glasnost --game --game-dir "GAME_FOLDER"`.

`--game-dir` is needed on the desktop. Without it, the **Game folder** button on the menu only tells you to start again with it.

**What is tested.** The desktop version is built and played on Linux, with sound and music on the system's default output device. Windows and macOS builds are untested. They should work, sound included, but nobody has tried them yet. If no sound device can be opened, the game runs silent, says why in its log, and, once a game has started, marks the volume rows in **Settings** "(no sound)". To build without sound, and without the sound library, add `--no-default-features` after `-p glasnost-platform-native`.

## Playing

**Skirmish.** From the main menu, **Skirmish** opens the setup screen:

- **Map:** any multiplayer map in your install with at least two start positions.
- **Players:** each row is **You**, **AI** or **Closed**, with a country, colour, team and start position. You need at least two players, and no more than the map has start positions.
- **Credits**, starting **Units** and **Game speed**.
- **Options:** Short game, Superweapons, Crates, Build off ally and Redeploy.
- **Pathing and other fixes:** our own fixes for units that get stuck or give up, and a few visual glitches. It is on by default. Turn it off to play the original behaviour.

**The computer opponent** is our own. It plays Allied, Soviet and Yuri. It gives only the orders a person could give and sees only what its own units see. There are no difficulty levels.

**Watch the computers play.** Set the first row to **AI** too, and you watch an all-computer game with the whole map revealed.

**During a game,** **Escape** opens the options menu. There you can change the speed, open Settings, surrender or leave. A skirmish pauses while that menu is open or the game window is not in focus.

**The end screen** shows victory or defeat and, for each player, what they built, killed and lost, and a score.

**Settings** (from the main menu, or from the options menu outside network games) has sound effects, voices and music volume, scroll speed, captions for announcements, colour-blind house colours, name tips over units, touchpad scrolling, **Display scale** and **Interface size**, and the command keys. Settings are kept between sessions.

## Controls

### Mouse

| Action | What it does |
|---|---|
| Left click | Select a unit or building. With units selected, give an order (move, attack, and so on). |
| Left drag | Select the units in a box |
| **Shift** + drag | Add to the selection |
| **Ctrl** + click | Force fire |
| **Alt** + click | Force move |
| Right click | Clear the selection, or cancel placing, selling, repairing or aiming |
| Right drag, or the pointer at the screen edge | Scroll the view |
| Wheel over the sidebar | Scroll the build icons |
| Left click on the radar | Jump the view there |

### Keys

You can change these on the settings screen (letters and digits only):

| Key | Action |
|---|---|
| S | Stop |
| X | Scatter |
| G | Guard |
| D | Deploy |
| C | Cheer |
| Z (hold) | Route mode: click to set a route, release to send it |
| T | Select units of the same type (tap again to reach further) |
| M, then P | Mark a moment, then save a pathing report (see [Reporting problems](#reporting-problems)) |
| N + digit | Make a control group in the browser, where Ctrl + digit may belong to the browser |

These are fixed:

| Key | Action |
|---|---|
| Escape | Options menu |
| Arrow keys | Scroll the view |
| Ctrl + digit | Make a control group |
| Digit | Select a control group; press it again quickly to centre the view on it |
| Shift + digit | Add a control group to the selection |
| Alt + digit | Select a control group and centre the view on it |
| Delete | Delete the selected route point |
| Enter | Chat (network games) |

Every menu and the sidebar also work from the keyboard alone (Tab, the arrows, Enter, Space and Page Up/Down). The settings screen lists those keys too. The details are in [Accessibility](docs/development.md#accessibility-m8-step-c3).

### Touch screens and touchpads

- **Touch:** drag one finger to move the view, or to scroll the build icons on the sidebar. Tap to click. Tap with two fingers to right-click.
- **Touchpad:** swipe with two fingers to move the view or scroll the build icons. This is on by default; turn it off with **Touchpad scrolling** in the settings.
- In the desktop version on Windows, and on Linux under X11, a touchpad swipe only scrolls the build icons. It does not move the view. Tablets have not been tried.

## Multiplayer

Every player runs the game on their own machine, and the players meet through a **relay**, a small server program that one of you runs. The relay needs no copy of the game.

1. **Everyone uses the same version.** Build from the same commit of the source, with no local changes. Use the same game files too. The lobby turns away a player whose version, rules or map differ, and tells you why.
2. **One person runs the relay:**

   ```sh
   cargo run --release -p glasnost-relay -- --listen 0.0.0.0:8787
   ```

   On its own, without `--listen`, the relay accepts only players on the same computer. For players outside your home network, open TCP port 8787 on your router and firewall.
3. **Point the game at the relay.** On the desktop, add `--relay ws://HOST:8787/` to the game's command line, or type the address into the **Relay** field of the Multiplayer screen. In the browser, add `&relay=ws://HOST:8787/` to the page address. `HOST` is the relay machine's address. A page served over `https://` needs a `wss://` relay.
4. **Host:** main menu, **Multiplayer**, type your name, click **Host**. Tell the others the eight-character room code. Pick the map, and set each row to **Open** (for a person), **AI** or **Closed**.
5. **Join:** **Multiplayer**, your name, the room code, **Join**. Set your own country, colour, team and start, then click **Ready**.
6. The host clicks **Start game**.

Press Enter to chat. Nobody can pause a network game, but any player can change its speed in the options menu. If your connection drops, you have 60 seconds of game time (by default) to click **Reconnect** on the Multiplayer screen, even after restarting the game. After that, your side surrenders.

The full guide covers putting the relay on the internet, browser players, direct connections between players, and what each message means: [docs/multiplayer.md](docs/multiplayer.md).

## Reporting problems

When you report a bug, say what you did and what you expected. Also give the **build id** shown at the foot of the Multiplayer screen, which names the exact version you run.

**A unit took a strange route or got stuck?** Press **M** as it happens, then **P** within 20 seconds. This saves a *pathing report* covering the last 20 seconds. The desktop version writes it to `path-reports/` in the folder you started the game from. The browser downloads it. Keep the file and say in your report that you have it. It holds a save of your game, so don't post it publicly or commit it.

**"Desync" in a network game?** Each player's game saves a report by itself, to `desync-reports/` (or as a download in the browser). Keep the files and say in your report that you have them; like pathing reports, they hold a save of your game. See [docs/multiplayer.md](docs/multiplayer.md#6-reports-desyncs).

## What works and what doesn't yet

**Works:**

- Skirmish against our computer opponent, or several of them, on the multiplayer maps of your install. All three sides are playable, with their units, buildings and superweapons. This includes air and naval units, garrisons, engineers and spies, tech buildings, crates, mind control and the rest of the Yuri side's specials.
- Network games with friends and computer players, chat, and reconnecting after a drop.
- Watching an all-computer game.
- Settings and accessibility options: captions, colour-blind colours, keyboard-only menus and sidebar.
- Sound, announcements and music, in the browser and on the desktop.
- The right size on high-resolution (2x) screens: the game is drawn at your display's scale and stays sharp, so a HiDPI laptop or a browser on a 2x screen shows the same area as an ordinary screen (**Display scale** in the settings: Auto, 1x, 2x or 3x).

**Not yet:**

- The campaign is not playable.
- You cannot save and load a game, and there is no replay viewer.
- Windows and macOS desktop builds are untested.
- There is no hosted web version, and the browser asks for your game folder on every visit.
- There is no zoom.
- Big late-game battles can stutter, especially in the browser.
- Some things look or play differently from the original. The sidebar has our own layout, and some fights don't yet end the way they do in the original. We are still working on these.
- On the desktop, the window does not respond for a few seconds while a map loads.

## FAQ

**Does Glasnost include any part of the game?** No. You need your own copy, and Glasnost reads it on your machine.

**Does it change my game folder?** No. Settings are kept in your user settings folder (on Linux `~/.config/glasnost/settings.txt`), or in the browser's storage. Reports are saved in the folder you started the game from.

**Can I play against people using the original game?** No. Glasnost's multiplayer only connects Glasnost players.

**Do mods work?** Not yet. Glasnost reads the game's archives and ignores loose data files in your game folder. Mods are untested.

**Is it on phones and tablets?** Touch controls work in the browser, but no tablet or phone has been tried.

**How is the engine written without the original game's code?** From written behaviour specs and measurements of the game, which a separate spec side prepares. See [How Glasnost is made (clean room)](#how-glasnost-is-made-clean-room).

## How Glasnost is made (clean room)

The engine is written without looking at the original game's program code. The project describes its method as a documented two-team process, and makes no claim to be a certified clean room. This is a summary; the full rules are in [PROVENANCE.md](PROVENANCE.md).

### Who does what

- **The spec side** studies the original game and writes behaviour specs: plain-language descriptions of what the game does, organised by game topic (combat, movement, production and so on). It works in a separate, private workspace that is never published.
- **The implementation side** writes the engine, working only from approved specs in [`spec/`](spec/README.md), measurement records in [`measurements/`](measurements/README.md) and this repository's own code.
- **The project lead** owns the specs, runs the measurements on the original game, and reviews and approves the work, but writes no engine code.

The spec writer, the scrub reviewer and the implementers are AI sessions, each confined to its own side. Future human contributors work under the implementation side's rules and state any earlier exposure to the sources that side may not read.

### What passes between the sides

- **Specs, after the scrub.** An automated check rejects hexadecimal numbers, code-like syntax and names taken from the original program or from the YRpp, Ares and Phobos projects named below. A fresh AI session with no access to the spec workspace then rewrites anything code-shaped into prose and rejects drafts that fail. Last, the lead reads and approves the spec. Only the draft text moves on; the spec side's raw notes stay in its workspace.
- **What a spec may say.** A spec describes what the game does, not how the original program is built: two different correct implementations must both satisfy it. Specs carry no stock rules values; they name a value by its role, such as "the weapon's reload time".
- **Measurement records** from the stock, unmodified game: win rates, times, tick counts, credits, unit counts, positions. Only these observations cross, never the scripts, configuration or notes that produced them. When a spec and a measurement disagree, the measurement wins and the spec goes back for revision.
- **Questions** go the other way only as written spec requests in [`spec/requests/`](spec/requests/README.md). An implementer who finds a spec missing, unclear or contradictory files a request and stops, rather than guessing or working from memory.

### What each side may read

- The spec side may study the stock game program and its decompilation, the YRpp, Ares and Phobos projects and their documentation, ModEnc, OpenRA, XCC and the lead's earlier modding repository.
- The implementation side never disassembles or decompiles the game, and never reads the spec workspace, the lead's earlier modding repository, YRpp, Ares, Phobos or ModEnc. It may read public file-format documentation, and OpenRA and XCC source code (GPL-3) for file formats and loaders. Code adapted from them keeps its copyright notice and names its origin.
- Neither side may use the Chrono Divide client or decompiled browser ports of the game.

The full table is in [PROVENANCE.md, section 1](PROVENANCE.md#1-roles).

### How the rules are checked

- **Citations.** Every behavioural constant, formula, ordering rule and quirk switch in the simulation code carries a comment naming the spec (`S-xxxx`) or measurement (`M-xxxx`) it comes from, and tests cite the same IDs. CI rejects uncited constants in the simulation crates and IDs that do not exist in `spec/` or `measurements/`. Instructions to implementers cite IDs too, and reviews judge code against tests and behaviour, not against memory of the original.
- **No game data.** Source, tests, fixtures, CI caches, release artifacts, screenshots and logs carry no EA data and no stock rules values. Tests use synthetic data the project makes itself. Tests that need a real install are opt-in, stay on the contributor's machine and never run in CI.
- **Sign-off.** Every commit carries a [DCO](https://developercertificate.org/) sign-off. AI-assisted commits keep the tool's attribution line, and the human who reviewed and accepted the change signs off.
- **Tool settings.** The implementer sessions' settings deny file access to the spec workspace, the scrub staging folders and the lead's modding repository, along with common disassembler commands and the documentation sites of those projects and ModEnc. File denies are the main guard. Command denies are best-effort, and a web deny cannot block a single GitHub repository, so implementers must also follow the written reading rules.
- **Incidents.** If forbidden material reaches the repository (an identifier or address from the program, decompiler output, code-shaped spec text, a stock rules value, game data, or a session that read a forbidden source), work that builds on it stops. The commits are reverted, and public history is rewritten to remove them where practical. A spec that carried the material goes back through the scrub, a fresh session that has not seen it rewrites the affected code, and the incident is logged in `spec/incidents/`. See [PROVENANCE.md, section 8](PROVENANCE.md#8-incidents).

### What the repository contains

It holds the engine's source code, the approved specs, the measurement records, synthetic test data and our own placeholder art. It holds no game files or pieces of them (archives, graphics, sounds, maps, string tables, rules files), no stock rules values and no install paths. When a measured value is itself a stock rules value, the record keeps the scenario, the method and the relation it confirms, and the value stays in a local-only file. Glasnost reads the game data from your own install, on your machine.

## For contributors

Glasnost has one unusual rule. Behaviour is implemented only from written behaviour specs in [`spec/`](spec/README.md) and measurement records in [`measurements/`](measurements/README.md), never from the original game's code. [How Glasnost is made](#how-glasnost-is-made-clean-room) explains the process, and [PROVENANCE.md](PROVENANCE.md) has the full rules, including what contributors may and may not read.

The code is GPL-3.0-or-later. Every commit needs a DCO sign-off (`git commit -s`). Start with [CONTRIBUTING.md](CONTRIBUTING.md), then [PLAN.md](PLAN.md) for the architecture and milestones, and [docs/development.md](docs/development.md) for the workspace, checks and CI.

```sh
cargo test --workspace   # tests
cargo xtask check        # repository checks: dependency rule, determinism, citations, no game data
```

## Disclaimer

This is an unofficial fan project. It is not affiliated with, endorsed by or supported by Electronic Arts Inc. Command & Conquer, Red Alert and Yuri's Revenge are trademarks of Electronic Arts Inc. They are used here only to describe compatibility.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
