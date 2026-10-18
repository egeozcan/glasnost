---
id: M-0003
title: Unit arena: Allied against Soviet vehicles at equal cost
topic: combat
specs: []
requests: []
verdict: new-finding
value-visibility: committed
measured-on: 2026-10-18
method: harness
---

# M-0003: Unit arena: Allied against Soviet vehicles at equal cost

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

Per-game data: `M-0003-arena-allied-soviet-vehicles.csv` beside this record (columns: row, col, row_side, seed, winner as `row`/`col`/`none`, end_frame, row_left, col_left).

## Result

420 games, 35 pairings, all usable. Win rates and army shares under the stated settings are not rules data (PROVENANCE §5).

| Row | Column | n | Row wins | Column wins | None | Row win rate | Row win rate as A / as B | Mean row left | Mean column left |
|---|---|---|---|---|---|---|---|---|---|
| MTNK | HTNK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.442 |
| MTNK | APOC | 12 | 6 | 6 | 0 | 0.50 | 6/6 / 0/6 | 0.141 | 0.300 |
| MTNK | TTNK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 0.686 | 0.000 |
| MTNK | HTK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| MTNK | V3 | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| FV | HTNK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 1.000 |
| FV | APOC | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.967 |
| FV | TTNK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 1.000 |
| FV | HTK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.944 |
| FV | V3 | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| TNKD | HTNK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 0.833 | 0.000 |
| TNKD | APOC | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 0.400 | 0.000 |
| TNKD | TTNK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| TNKD | HTK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| TNKD | V3 | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| SREF | HTNK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 1.000 |
| SREF | APOC | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 1.000 |
| SREF | TTNK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 1.000 |
| SREF | HTK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.727 |
| SREF | V3 | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| MGTK | HTNK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.892 |
| MGTK | APOC | 12 | 1 | 11 | 0 | 0.08 | 0/6 / 1/6 | 0.009 | 0.467 |
| MGTK | TTNK | 12 | 9 | 3 | 0 | 0.75 | 5/6 / 4/6 | 0.102 | 0.052 |
| MGTK | HTK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 0.806 | 0.000 |
| MGTK | V3 | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| ROBO | HTNK | 12 | 0 | 4 | 8 | 0.00 | 0/6 / 0/6 | 0.189 | 1.000 |
| ROBO | APOC | 12 | 0 | 1 | 11 | 0.00 | 0/6 / 0/6 | 0.350 | 1.000 |
| ROBO | TTNK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 1.000 |
| ROBO | HTK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.981 |
| ROBO | V3 | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 1.000 |
| BFRT | HTNK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.783 |
| BFRT | APOC | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.717 |
| BFRT | TTNK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 | 0.000 | 0.740 |
| BFRT | HTK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |
| BFRT | V3 | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 | 1.000 | 0.000 |

**Tolerance for tests.** Per pairing, pooled over both sides (n = 12): the row's win proportion must not differ from the measured one under a two-proportion test at p < 0.01, and the per-game row-left and column-left shares must not differ under a two-sample KS test at p < 0.01 (PLAN §6, layer 2B). Many pairings gave the same outcome for every seed on a side; there a reproduction should give the same outcome too.

## Evidence

Raw per-game harness output stays with the oracle operator (outside this repository), 420 game directories; no game failed or was dropped.

## Notes

- **Reproducing it:** the setup uses a stock map, so a comparison against this record runs in the local lane, with the map read from the game directory; a test on a map of our own measures something else and is not checked against it.
- **Time to kill is not recorded:** the harness samples only every 304 frames, so `end_frame` is a coarse window (most fights end by the first or second sample). A finer measurement needs another method.
- **Fidelity check, not a balance target (PLAN D14):** a mismatch with this record points at the simulation or a spec, never at the own AI, which takes no part in these fights beyond the hunt mission.
- The houses' original skirmish AI is active but has no base, credits or production; whether it gives these units orders beyond their hunt mission was not checked.
