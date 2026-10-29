---
id: M-0017
title: Unit arena, stock replay of every archived game without SREF or V3
topic: combat
specs: []
requests: []
verdict: new-finding
value-visibility: committed
measured-on: 2026-10-28
method: harness
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-29
---

# M-0017: Unit arena, stock replay of every archived game without SREF or V3

This record replays, with stock rules, every archived game of M-0001 to M-0003 in which neither
side is SREF or V3: 1584 games. Every replay is identical to its archived game on the columns a
record row is made of, so 1577 of those rows stand as stock rows. The other 7, all in M-0001's
MIND against E2 pairing, end in a game crash in both and have no result; M-0001 sets them apart
from its usable games. The Result restates that pairing's line on its 5 completed games.

## Setup

- Game build: stock Yuri's Revenge 1.001, executable hash verified; run under the oracle harness
  (PROVENANCE §7).
- Data: stock, no mods, no loose rules or art files.
- Games: the 1584 archived games of M-0001 to M-0003 with neither SREF nor V3 on either side,
  132 pairings of 12 games each (the row type on side A with seeds 0 to 5, and on side B with
  seeds 0 to 5):

  | Record | Pairings | Games |
  |---|---|---|
  | M-0001 | 84 of 96 | 1008 |
  | M-0002 | 24 of 24 | 288 |
  | M-0003 | 24 of 35 | 288 |

  Each is replayed from its archived setup and seed. The 276 games with SREF or V3 on either side
  are not part of this record.
- Map, houses and settings, as in M-0001 to M-0003: the stock map `Arena.mmx` around its start
  waypoint 2 (call its cell W); two computer houses, side A Americans at start 0 and side B
  Russians at start 1, difficulty hard, no starting construction vehicle, no buildings and 0
  credits; an idle observer human at start 3. Game speed fastest, tech level 10, superweapons
  off, crates off, short game off. Budget, stated cells, facings, the *hunt* mission and the end
  rule are those of M-0001.
- Placement and map content, as in the archived games: units are created on their stated cells
  before tick 0, and a unit that its stated cell does not take is placed on a free neighbouring
  cell. The map's four bunkers, owned by a neutral house, are in play from tick 0.
- **About M-0001 to M-0003.** Their archived games were run with five gameplay changes of the
  harness on: a siege rule for long-range artillery, a change to Magnetron carrying, changes to the
  computer houses' production, extra defences, and an expansion and factory queue. This replay has
  all five off. The house director is off for every house, in the archive and in the replay. The
  houses' own skirmish AI runs as in a normal game but has nothing to build with.
- What is not stock, in this replay:
  - no menus: the skirmish setup comes from a settings file
  - the computer players' starting construction vehicles are removed
  - the two computer players are not allied with each other (a stock skirmish would ally every
    computer player when nobody is on a team)
  - the idle observer house's objects cannot be targeted or damaged
  - the run ends by M-0001's rule: at the first sample (every 304 ticks) at which only one side
    still owns anything, or at the first sample at or after tick 6000
  - a periodic sampler reads state only
- What is not stock, in the archived games: the same, and the five gameplay changes above.

## Procedure

1. Replay each of the 1584 games once with the five changes off.
2. At every sample (every 304 ticks), record each house's unit count, the value of its units and
   whether it is defeated, and at the end the result.
3. Compare each replay with its archived game, sample by sample, on those columns and on the
   result. "Identical" below means those sampled columns and the result, not every tick.
4. Record which games end in a game crash, and the last sample each takes before it.

## Result

**Games.**

| Quantity | n | Value |
|---|---|---|
| Games identical to the archived game in every sample, and in the result where it has one (the 7 crashed games included) | 1584 | 1584 |
| Games that end in a game crash, in the archive and in the replay alike | 1584 | 7 |

- Each of a record row's columns (winner and end frame) follows from the compared columns and the
  archived setup. So 1577 of the 1584 rows in M-0001 to M-0003 stand as stock rows: all but the 7
  crashed games below, which have no result.
- **The crashed games** are MIND against E2 in M-0001: the row type MIND on side A with seeds 0
  to 5, and on side B with seed 0. Each crashes after its last sample and before the next. The
  last sample is at tick 304 (side A, seeds 0 to 4) or 608 (side A, seed 5; side B, seed 0), in
  the archive and in the replay alike, with identical samples up to it.
- M-0001's per-game data gives each of the 7 as winner `none` with that last sample as its end
  frame. A crashed game has no winner; by M-0001's Procedure, the harness records a crashed game as
  `none` with an end frame before the limit, and its Result names the 7 *None* games of the row as
  the crashed ones.
- **Scope, checked:** of the 1860 archived games of M-0001 to M-0003, these 7 are the only ones
  without a result. Of the 175 `none` rows in their per-game data, these 7 are the only ones with
  an end frame other than the limit sample 6080.

**MIND against E2 (M-0001), restated on its 5 completed games** (side B, seeds 1 to 5; columns
as in M-0001's Result).

| Row | Column | n | Row wins | Column wins | None | Row win rate | Row win rate as A / as B |
|---|---|---|---|---|---|---|---|
| MIND | E2 | 5 | 3 | 2 | 0 | 0.60 | no game / 3/5 |

- The completed games: seeds 2, 3 and 4 won by MIND, seeds 1 and 5 won by E2, all at end frame
  608.
- There is no completed game with MIND on side A, so the side-A rate is undefined (M-0001 gives
  0/6). M-0001's line was n 12: 3 row wins, 2 column wins, 7 none, win rate 0.25, as A / as B
  0/6 / 3/6.
- 7 of M-0001's 1152 games are crashes with no result, leaving 1145 with one.

**Tolerance for tests.** As M-0001's, per pairing. For MIND against E2, the tests run on the 5
side-B games of seeds 1 to 5 only (n = 5). The 7 crashed games have no result to compare with.

## Evidence

- 1584 games, each replayed once; the replays stay with the operator, and no recording or log
  hash was supplied with the results.
- The 7 crashed games' archived end frames and last samples equal M-0001's per-game rows for
  them. No other replay failed.

## Notes

- **M-0001 to M-0003** point here for the five gameplay changes of their archived games, for a
  unit placed on a neighbouring cell, and for which of their rows stand as stock rows. M-0001 also
  points here for its 7 crashed MIND against E2 games.
- DECIDED (autonomy, 2026-10-28): the 7 crashed games are not re-run, and the pairing's line is
  restated on its 5 completed games. They crash in the stock replay too.
- **Not shown:** whether the game crashes without the harness, and why it crashes. The tick in
  which each crash comes is not supplied here; it lies between the last sample and the next.
- **For a reproduction:** a reproduction that plays these 7 games to a result has no original
  result to compare with, so they are not a fidelity check.
