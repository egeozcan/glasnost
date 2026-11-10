---
id: Q-1713
title: How the stock arena games with infantry on the cliff cells end
status: answered
related: [S-0309, S-0208, S-0212]
answered-by: [S-0216, M-0006]
---

## Need
Follow-up to Q-1502. S-0309 revision 21 and S-0208 revision 34 answered it, and both are built:
a searcher standing on a cell of no reach region considers only objects on such cells, an enemy
on open ground never finds it by the map-wide search, and a scatter gives it no escape. On
`Arena.mmx` five of side A's infantry placement cells are such cells.

With those rules our infantry pairings with a group on side A still end at the frame limit with
the units on the cliff cells (and sometimes some enemies) left, in every seed. In the stock games
they do not:

- M-0002 INIT against E2, INIT on side A: stock, the INIT side wins every game; ours, every game
  ends `none` with the INIT units on the cliff cells left and a seed-dependent part of the E2
  group still in play.
- M-0001 YTNK against E2, E2 on side A: stock, the YTNK side wins most games; ours, every game
  ends `none` with E2 units left.

INIT against E2 is among the ten pairings whose spread falls furthest short of the record's
(`docs/m8-claims/a1-arena-spread.md`). In the stock games something ends these games that the
answered rules do not give us.

Code: `crates/deterministic/sim/src/combat/acquire.rs` (zone row of the candidate test),
`crates/deterministic/sim/src/regions.rs`.

## Questions
1. In those stock games, how are the infantry units placed on the cliff cells destroyed, or how do
   they destroy the enemy: by enemies that attack them from open ground, by their own fire at
   enemies on open ground, or after they leave the cliff cells?
2. Does an infantry unit on such a cell, in the hunt behaviour, fire at an enemy that comes within
   its weapon's range, although its map-wide search does not consider that enemy?
3. Does a hunting unit on open ground fire at an enemy on such a cell when that enemy is within its
   weapon's range (for example by its own range search, or by retaliation after being hit)?

## Possible observation
The arena's INIT against E2 game with INIT on side A, watched: for each unit placed on a cliff
cell, when and by whom it is hit and what it fires at, until the game ends.

## Current workaround
The answered rules of S-0309 revision 21 and S-0208 revision 34, as built. The pairings are
reported as failing for this reason.
