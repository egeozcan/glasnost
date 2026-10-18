---
id: M-0001
title: Unit arena: Yuri vehicles against Allied and Soviet units at equal cost
topic: combat
specs: []
requests: []
verdict: new-finding
value-visibility: committed
measured-on: 2026-10-18
method: harness
---

# M-0001: Unit arena: Yuri vehicles against Allied and Soviet units at equal cost

## Setup

- Game build: stock Yuri's Revenge 1.001, executable hash verified; run under the oracle harness (PROVENANCE §7).
- Data: stock, no mods, no loose rules or art files.
- Map: the stock multiplayer map `Arena.mmx` (sha256 `ed5fa0d7ec4b5e2d8a7110a0fff6cad413ae0437c62521c3c75fd9bc7422c874`), on the open ground around its start waypoint 2 (call its cell W).
- Players: two computer houses, side A = Americans (start 0) and side B = Russians (start 1), difficulty hard, each with no starting MCV, no buildings and 0 credits, so each owns only its placed group; an observer human at start 3 that the computer houses ignore and that owns nothing that fights. Game speed fastest, tech level 10, superweapons off, crates off, short game off.
- Harness: the oracle's unit arena. Its own AI additions are off (no director; its other additions act only in another game mode). The houses' own skirmish AI runs as in a normal game but has nothing to build with.
- Groups: each side gets one unit type and a budget of 9000 credits: round(9000 / the type's cost) units, at least 1 (an infantry group at most 60). Unit *i* (from 0) of side A stands on cell (W.x − 6 − ⌊i/8⌋, W.y − 4 + (i mod 8)), facing east; side B mirrors it at (W.x + 6 + ⌊i/8⌋, W.y − 4 + (i mod 8)), facing west, so the groups start 12 cells apart in columns of 8 running away from the enemy. Infantry also take one cell each. In every game every unit stood on its stated cell. Every unit starts in the *hunt* mission. The group counts are not stated here (they follow from stock costs); compute them from the imported rules.
- Each pairing (row type against column type) is played with the row type on side A (6 games, seeds 0 to 5) and on side B (6 games, seeds 0 to 5). The seed is the game's random seed for the match (the same seed list on both sides).

## Procedure

1. Start the match with the groups placed as above and both houses' units hunting.
2. Every 304 game frames the harness samples each house; the game is over at the first sample at which only one side still owns anything, or at the first sample at or after frame 6000.
3. Record per game: the winner (the side that still owns something when the other owns nothing; `none` if both still own something at the limit), the sample frame at which the game was over, and for each side the share of its starting army value (the sum of its units' costs at the start) that it still owns at the end. Units taken over by mind control count for the house that owns them at the end, so a share can exceed 1.

Per-game data: `M-0001-arena-yuri-vehicles.csv` beside this record (columns: row, col, row_side, seed, winner as `row`/`col`/`none`, end_frame, row_left, col_left).

## Result

1152 games, 96 pairings, all usable. Win rates and army shares under the stated settings are not rules data (PROVENANCE §5).

| Row | Column | n | Row wins | Column wins | None | Row win rate | Row win rate as A / as B | Mean row left | Mean column left |
|---|---|---|---|---|---|---|---|---|---|
| LTNK | MTNK | 12 | 5 | 7 | 0 | 0.42 | 4/6 / 1/6 | 0.083 | 0.096 |
| LTNK | FV | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| LTNK | TNKD | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 1.000 |
| LTNK | SREF | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 0.846 | 0.000 |
| LTNK | MGTK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.417 |
| LTNK | ROBO | 12 | 5 | 0 | 7 | 0.42 | 3/6 / 2/6 | 1.000 | 0.217 |
| LTNK | BFRT | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 0.750 | 0.000 |
| LTNK | HTNK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.458 |
| LTNK | APOC | 12 | 6 | 6 | 0 | 0.50 | 6/6 / 0/6 | 0.103 | 0.317 |
| LTNK | TTNK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 0.679 | 0.000 |
| LTNK | HTK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| LTNK | V3 | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| LTNK | E1 | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.772 |
| LTNK | GGI | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 0.833 | 0.000 |
| LTNK | E2 | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.719 |
| LTNK | SHK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.986 |
| YTNK | MTNK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 1.000 |
| YTNK | FV | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 0.939 | 0.000 |
| YTNK | TNKD | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 1.000 |
| YTNK | SREF | 12 | 8 | 4 | 0 | 0.67 | 3/6 / 5/6 | 0.122 | 0.125 |
| YTNK | MGTK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 0.833 | 0.000 |
| YTNK | ROBO | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| YTNK | BFRT | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.667 |
| YTNK | HTNK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 1.000 |
| YTNK | APOC | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.983 |
| YTNK | TTNK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 1.000 |
| YTNK | HTK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.995 |
| YTNK | V3 | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| YTNK | E1 | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 0.822 | 0.000 |
| YTNK | GGI | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| YTNK | E2 | 12 | 10 | 2 | 0 | 0.83 | 6/6 / 4/6 | 0.444 | 0.037 |
| YTNK | SHK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 0.972 | 0.000 |
| MIND | MTNK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 2.040 | 0.000 |
| MIND | FV | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 2.029 | 0.000 |
| MIND | TNKD | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 2.029 | 0.000 |
| MIND | SREF | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.949 | 0.000 |
| MIND | MGTK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.776 | 0.000 |
| MIND | ROBO | 12 | 0 | 0 | 12 | 0.00 | 0/6 / 0/6 | 1.000 | 1.000 |
| MIND | BFRT | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.914 | 0.000 |
| MIND | HTNK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 2.029 | 0.000 |
| MIND | APOC | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 2.000 | 0.000 |
| MIND | TTNK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 2.097 | 0.000 |
| MIND | HTK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 2.029 | 0.000 |
| MIND | V3 | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 2.006 | 0.000 |
| MIND | E1 | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.250 | 0.000 |
| MIND | GGI | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.975 | 0.000 |
| MIND | E2 | 12 | 3 | 2 | 7 | 0.25 | 0/6 / 3/6 | 0.761 | 0.319 |
| MIND | SHK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.981 | 0.000 |
| TELE | MTNK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.929 |
| TELE | FV | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.928 |
| TELE | TNKD | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.917 |
| TELE | SREF | 12 | 0 | 11 | 1 | 0.00 | 0/6 / 0/6 | 0.046 | 0.635 |
| TELE | MGTK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.778 |
| TELE | ROBO | 12 | 0 | 0 | 12 | 0.00 | 0/6 / 0/6 | 0.426 | 0.300 |
| TELE | BFRT | 12 | 0 | 2 | 10 | 0.00 | 0/6 / 0/6 | 0.472 | 0.375 |
| TELE | HTNK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.742 |
| TELE | APOC | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.633 |
| TELE | TTNK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.594 |
| TELE | HTK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.759 |
| TELE | V3 | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.723 |
| TELE | E1 | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.956 |
| TELE | GGI | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 1.000 |
| TELE | E2 | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.996 |
| TELE | SHK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 1.000 |
| CAOS | MTNK | 12 | 0 | 9 | 3 | 0.00 | 0/6 / 0/6 | 0.056 | 0.224 |
| CAOS | FV | 12 | 1 | 6 | 5 | 0.08 | 1/6 / 0/6 | 0.315 | 0.283 |
| CAOS | TNKD | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.675 |
| CAOS | SREF | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.615 |
| CAOS | MGTK | 12 | 0 | 0 | 12 | 0.00 | 0/6 / 0/6 | 0.722 | 0.111 |
| CAOS | ROBO | 12 | 0 | 0 | 12 | 0.00 | 0/6 / 0/6 | 1.000 | 1.000 |
| CAOS | BFRT | 12 | 0 | 7 | 5 | 0.00 | 0/6 / 0/6 | 0.120 | 0.521 |
| CAOS | HTNK | 12 | 0 | 8 | 4 | 0.00 | 0/6 / 0/6 | 0.046 | 0.117 |
| CAOS | APOC | 12 | 0 | 0 | 12 | 0.00 | 0/6 / 0/6 | 0.296 | 0.200 |
| CAOS | TTNK | 12 | 0 | 0 | 12 | 0.00 | 0/6 / 0/6 | 1.000 | 0.125 |
| CAOS | HTK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.852 |
| CAOS | V3 | 12 | 4 | 1 | 7 | 0.33 | 4/6 / 0/6 | 0.667 | 0.084 |
| CAOS | E1 | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.959 |
| CAOS | GGI | 12 | 0 | 1 | 11 | 0.00 | 0/6 / 0/6 | 0.778 | 0.046 |
| CAOS | E2 | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.851 |
| CAOS | SHK | 12 | 0 | 0 | 12 | 0.00 | 0/6 / 0/6 | 0.981 | 0.056 |
| DISK | MTNK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| DISK | FV | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.583 |
| DISK | TNKD | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| DISK | SREF | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| DISK | MGTK | 12 | 0 | 0 | 12 | 0.00 | 0/6 / 0/6 | 1.000 | 0.435 |
| DISK | ROBO | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| DISK | BFRT | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| DISK | HTNK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| DISK | APOC | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 1.000 |
| DISK | TTNK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| DISK | HTK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.921 |
| DISK | V3 | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| DISK | E1 | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| DISK | GGI | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| DISK | E2 | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| DISK | SHK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |

**Tolerance for tests.** Per pairing, pooled over both sides (n = 12): the row's win proportion must not differ from the measured one under a two-proportion test at p < 0.01, and the per-game row-left and column-left shares must not differ under a two-sample KS test at p < 0.01 (PLAN §6, layer 2B). Many pairings gave the same outcome for every seed on a side; there a reproduction should give the same outcome too.

## Evidence

Raw per-game harness output stays with the oracle operator (outside this repository), 1152 game directories; no game failed or was dropped.

## Notes

- **Reproducing it:** the setup uses a stock map, so a comparison against this record runs in the local lane, with the map read from the game directory; a test on a map of our own measures something else and is not checked against it.
- **Time to kill is not recorded:** the harness samples only every 304 frames, so `end_frame` is a coarse window (most fights end by the first or second sample). A finer measurement needs another method.
- **Fidelity check, not a balance target (PLAN D14):** a mismatch with this record points at the simulation or a spec, never at the own AI, which takes no part in these fights beyond the hunt mission.
- The houses' original skirmish AI is active but has no base, credits or production; whether it gives these units orders beyond their hunt mission was not checked.
