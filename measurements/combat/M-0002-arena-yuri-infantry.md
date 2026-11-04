---
id: M-0002
title: Unit arena: Yuri infantry against infantry and main battle tanks at equal cost
topic: combat
specs: []
requests: []
verdict: new-finding
value-visibility: committed
measured-on: 2026-10-18
method: harness
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-04
---

# M-0002: Unit arena: Yuri infantry against infantry and main battle tanks at equal cost

## Setup

- Game build: stock Yuri's Revenge 1.001, executable hash verified; run under the oracle harness (PROVENANCE §7).
- Data: stock, no mods, no loose rules or art files.
- Map: the stock multiplayer map `Arena.mmx` (sha256 `ed5fa0d7ec4b5e2d8a7110a0fff6cad413ae0437c62521c3c75fd9bc7422c874`), on the open ground around its start waypoint 2 (call its cell W).
- Players: two computer houses, side A = Americans (start 0) and side B = Russians (start 1), difficulty hard, each with no starting MCV, no buildings and 0 credits, so each owns only its placed group; an observer human of the British country at start 3 that the computer houses ignore and that owns nothing that fights. Game speed fastest, tech level 10, superweapons off, crates off, short game off.
- Harness: the oracle's unit arena. Its house director is off for every house. Five other gameplay
  changes of the harness were on in these games: a siege rule for long-range artillery, a change to
  Magnetron carrying, changes to the computer houses' production, extra defences, and an expansion
  and factory queue (M-0017 Setup). The houses' own skirmish AI runs as in a normal game but has
  nothing to build with.
- What is not stock in these games, as M-0017's Setup gives it:
  - no menus: the skirmish setup comes from a settings file
  - the computer players' starting construction vehicles are removed
  - the two computer players are not allied with each other (a stock skirmish would ally every
    computer player when nobody is on a team)
  - the idle observer house's objects cannot be targeted or damaged
  - units are created on their stated cells before tick 0
  - the map's four bunkers, owned by a neutral house, are in play from tick 0
  - the run ends by the rule in the Procedure, and a periodic sampler reads state only
  - the five gameplay changes of the harness above
- Groups: each side gets one unit type and a budget of 9000 credits: round(9000 / the type's cost)
  units, at least 1 (an infantry group at most 60). Unit *i* (from 0) of side A stands on cell (W.x
  − 6 − ⌊i/8⌋, W.y − 4 + (i mod 8)), facing east; side B mirrors it at (W.x + 6 + ⌊i/8⌋, W.y − 4 +
  (i mod 8)), facing west, so the groups start 12 cells apart in columns of 8 running away from the
  enemy. Infantry also take one cell each. A unit that its stated cell does not take is placed on a
  free neighbouring cell (M-0017 Setup). Every unit starts in the *hunt* mission. The group counts
  are not stated here (they follow from stock costs); compute them from the imported rules.
- Each pairing (row type against column type) is played with the row type on side A (6 games, seeds 0 to 5) and on side B (6 games, seeds 0 to 5). The game for seed *s* is played with the random seed 1000 + *s*, as its settings file gives it (the same seed list on both sides).

## Procedure

1. Start the match with the groups placed as above and both houses' units hunting.
2. Every 304 game frames the harness samples each house; the game is over at the first sample at which only one side still owns anything, or at the first sample at or after frame 6000.
3. Record per game: the winner (the side that still owns something when the other owns nothing, or
   `none` if both still own something at the limit) and the sample frame at which the game was over.

Per-game data: `M-0002-arena-yuri-infantry.csv` beside this record (columns: row, col, row_side,
seed, winner as `row`/`col`/`none`, end_frame).

## Result

288 games, 24 pairings, all usable. Win rates under the stated settings are not rules data
(PROVENANCE §5).

| Row | Column | n | Row wins | Column wins | None | Row win rate | Row win rate as A / as B |
|---|---|---|---|---|---|---|---|
| INIT | E1 | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 |
| INIT | GGI | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 |
| INIT | E2 | 12 | 7 | 5 | 0 | 0.58 | 6/6 / 1/6 |
| INIT | SHK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 |
| INIT | MTNK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 |
| INIT | HTNK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 |
| BRUTE | E1 | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 |
| BRUTE | GGI | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 |
| BRUTE | E2 | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 |
| BRUTE | SHK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 |
| BRUTE | MTNK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 |
| BRUTE | HTNK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 |
| YURI | E1 | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 |
| YURI | GGI | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 |
| YURI | E2 | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 |
| YURI | SHK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 |
| YURI | MTNK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 |
| YURI | HTNK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 |
| VIRUS | E1 | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 |
| VIRUS | GGI | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 |
| VIRUS | E2 | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 |
| VIRUS | SHK | 12 | 12 | 0 | 0 | 1.00 | 6/6 / 6/6 |
| VIRUS | MTNK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 |
| VIRUS | HTNK | 12 | 0 | 12 | 0 | 0.00 | 0/6 / 0/6 |

**Tolerance for tests.** Per pairing, pooled over both sides (n = 12): the row's win proportion must
not differ from the measured one under a two-proportion test at p < 0.01 (PLAN §6, layer 2B). This
record gives no per-game army shares, so no share test runs against it. Many pairings gave the same
outcome for every seed on a side; there a reproduction should give the same outcome too.

## Evidence

Raw per-game harness output stays with the oracle operator (outside this repository), 288 game directories; no game failed or was dropped.
- Each game's settings file gives the seed and every house's country and start. In every game of
  this record the seed is 1000 + the game's seed number and the observer house is British.

## Notes

- **Computer players:** both houses were the original's computer players, so every rule the original gives only to computer-controlled owners applied to their units (for example retaliation and scatter outside the human rules). A comparison must give its two houses the same rules (S-0215, houses in our engine).
- **Cliff cells in large groups:** the ground around W is not flat everywhere. Large side-A groups
  reach cells west of W that are cliff cells between two height levels (Q-1502 lists them). A unit
  stated on such a cell that the cell does not take stands on a free neighbouring cell (Setup).
- **Stock rows:** M-0017 replays every game of this record with the five changes off, and each
  replay is identical to its archived game on this record's columns.
- **Reproducing it:** the setup uses a stock map, so a comparison against this record runs in the local lane, with the map read from the game directory; a test on a map of our own measures something else and is not checked against it.
- **Time to kill is not recorded:** the harness samples only every 304 frames, so `end_frame` is a coarse window (most fights end by the first or second sample). A finer measurement needs another method.
- **Fidelity check, not a balance target (PLAN D14):** a mismatch with this record points at the simulation or a spec, never at the own AI, which takes no part in these fights beyond the hunt mission.
- The houses' original skirmish AI is active but has no base, credits or production; whether it gives these units orders beyond their hunt mission was not checked.
