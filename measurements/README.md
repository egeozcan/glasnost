# Measurements

Records of what the original game does when run and observed: timings, counts, credits, positions, health readings, win rates. The project lead (oracle operator) takes them from the stock game, by hand or with a scripted harness, and commits them. See [PROVENANCE.md](../PROVENANCE.md) sections 5 and 7.

## Rules

- **Stock game only.** Stock data, a hash-checked executable, every gameplay mod and AI change off. Record the settings.
- **Observables only.** No code, addresses, configuration or notes from the harness enter this repository.
- **Values that mirror stock rules data stay local.** If the measured value is the same thing as a rules value (a cost, a reload time, a range), the record commits the scenario, method and relation (`value-visibility: local`), and the value goes in the git-ignored `measurements/local/M-xxxx.json`. Values that are not rules data (tick rate, ordering, win rates under stated settings) are committed (`value-visibility: committed`).
- **The measurement wins** over a spec. A contradiction sends the spec back for revision.

## Layout

| Path | Contents |
|---|---|
| `<topic>/M-xxxx-*.md` | One record per file, same topics as `spec/` |
| `<topic>/M-xxxx-*.csv` | Per-trial data of a record, when it has many trials (columns described in the record) |
| `TEMPLATE.md` | The record template |
| `local/` | Git-ignored local-only values; never committed |

## Index

| ID | Title | Topic | Confirms | Visibility |
|---|---|---|---|---|
| [M-0001](combat/M-0001-arena-yuri-vehicles.md) | Unit arena: Yuri vehicles against Allied and Soviet units at equal cost | combat | new finding | committed |
| [M-0002](combat/M-0002-arena-yuri-infantry.md) | Unit arena: Yuri infantry against infantry and main battle tanks at equal cost | combat | new finding | committed |
| [M-0003](combat/M-0003-arena-allied-soviet-vehicles.md) | Unit arena: Allied against Soviet vehicles at equal cost | combat | new finding | committed |
| [M-0011](combat/M-0011-arena-caos-against-bfrt-replay.md) | Unit arena, CAOS against BFRT, stock replay watched per unit | combat | confirms | local |
| [M-0017](combat/M-0017-stock-replay.md) | Unit arena, stock replay of every archived game without SREF or V3 | combat | new finding | committed |
| [M-0015](combat/M-0015-arena-tele-against-bfrt-replay.md) | Unit arena, TELE against BFRT, stock replay watched per unit | combat | confirms | local |
| [M-0013](economy/M-0013-germinable-cells.md) | Which cells receive spreading ore (S-0401 scenario E8) | economy | contradicts | local |
| [M-0005](combat/M-0005-arena-ltnk-against-robo-stall.md) | Hunters against disabled robot tanks in the unit arena (the LTNK against ROBO stall) | combat | contradicts | local |
| [M-0006](combat/M-0006-arena-stock-replay-bunkers.md) | Unit arena, stock replay of archived games, the map's neutral bunkers and moved units | combat | new finding | committed |
| [M-0016](combat/M-0016-stock-replay-sref-v3.md) | Unit arena, stock replay of every archived game with SREF or V3 | combat | new finding | local |
| [M-0007](combat/M-0007-repair-depot-timing.md) | Repair depot timing and contact loss with the depot recorder (Q-1610) | combat | contradicts | local |
| [M-0014](rendering/M-0014-building-slots-power-off.md) | Building slots after a power-off order next to an ownership change or a pulse's end | rendering | confirms | local |
| [M-0008](simulation/M-0008-random-streams-recorder.md) | Random streams with the random-state recorder (seeding, ranges, creation, idle draws) | simulation | confirms | local |
| [M-0004](combat/M-0004-firing-cycle-recorder.md) | Firing cycle with the random-state recorder (delays, bursts, building rounds, draws) | combat | contradicts | local |
| [M-0012](rules-import/M-0012-rules-final-line-feed.md) | A rules file's final line feed and a tank's burst firing | rules-import | confirms | local |
| [M-0019](simulation/M-0019-unit-arena-placements-at-tick-0.md) | Unit arena, the records' placements at tick 0 (standing positions, raw values, refused cells) | simulation | new finding | committed |
| [M-0021](combat/M-0021-arena-fragment-gattling-test-pairing.md) | Unit arena, a test pairing after Q-1714's (fragments per shot, first shots, gaps, stages) | combat | confirms | local |
| [M-0020](simulation/M-0020-unit-arena-standing-positions-yuri-caos.md) | Unit arena, YURI against GGI and SHK and CAOS against E1 (placement at tick 0, standing positions and the winner, first targets, captures, the drones' end) | simulation | new finding | local |
| [M-0023](structures/M-0023-released-infantry-walled-pen-order.md) | A released infantry unit given a move order into a walled pen before its first turn (S-0417 U10 part g run 1) | structures | confirms | local |
| [M-0024](simulation/M-0024-hunt-scatter-from-damage.md) | The scatter from damage for hunting vehicles of computer-controlled houses (S-0215 B6 part f) | simulation | confirms | local |
| [M-0025](simulation/M-0025-hunt-scatter-own-allied-unharmable.md) | The scatter from damage for a hunting vehicle hit by its own house, an allied house or an enemy it cannot harm (S-0215 B6 part g) | simulation | confirms | local |
