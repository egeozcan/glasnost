---
id: Q-2112
title: How ground hunters find a hovering jumpjet again after losing their target
status: answered
related: [S-0309, S-0215, S-0502, S-0102, M-0001]
answered-by: [S-0309]
---

## Need
M-0001's DISK against FV pairing fails on both sides: the stock games are column wins on every
seed, ours are row wins on 58 of 64 (side A) and 64 of 64 (side B) seeds in the local lane.

In our games the column's vehicles take targets at tick 1, while the discs still stand on the
ground, and keep those targets as the discs rise (a held target is not re-searched). They kill
some discs. Once a target dies, no column vehicle takes another target for the rest of the game
(seed 1, side A: targets lost at four ticks, none taken again), and the remaining discs win.
The approved rules that give this:

- S-0309 claim 7: the map-wide search never finds an object outside the ground render layer, so
  a hovering jumpjet (air or top layer, S-0102 claim 3) is never found by *hunt*'s search
  (`quirks.hunt_ground_layer_only`).
- Retaliation against the discs' fire does not happen in our games: S-0215 claim 18's conditions
  refuse it for the hunters.
- No other rule of S-0309 (claim 22's limited scan included) gives them a target.

The code is `combat::acquire` (search and `retaliate`) and `process` (*hunt*).

## Questions
1. In those stock games, do the column's vehicles take new targets after their first targets
   die? If so, by which occasion: a search of *hunt* that finds the hovering discs, retaliation
   when a disc hits them, a scan of a limited radius, or another one?
2. Does `quirks.hunt_ground_layer_only`'s original behaviour apply to a searcher whose weapon can
   hit a hovering jumpjet, or does such a searcher have a search pass that finds units in the air
   or top layer?
3. Does a vehicle in *hunt* of a computer-controlled house retaliate against a hovering jumpjet
   that hits it?

## Possible observation
In one M-0001 DISK against FV game, record for each column vehicle the tick and object of every
target change, and the discs' layer and height at each change.

## Current workaround
None: the code follows S-0309 claim 7 and S-0215 claim 18 as approved, and the pairing keeps
failing. Nothing was changed for it.
