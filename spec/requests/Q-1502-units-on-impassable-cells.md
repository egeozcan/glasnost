---
id: Q-1502
title: Units that stand on cells their movement zone cannot enter
status: answered
related: [S-0212, S-0309, S-0215, S-0211]
answered-by: [S-0309]
---

## Need
The arena of M-0001 to M-0003 places infantry groups of up to 60 units in columns of 8 west of
cell W. On `Arena.mmx`, five of side A's cells, (W.x − 11, W.y − 4), (W.x − 11, W.y − 3),
(W.x − 10, W.y − 4), (W.x − 10, W.y − 3) and (W.x − 10, W.y + 3), have land type *rock* after
the terrain recalculation (S-0211 claim 5). They are cliff cells between two height levels and
belong to no reach region of the infantry movement zone (S-0212 claim 4). M-0001 states that in
every game every unit stood on its stated cell.

In our engine an infantry unit placed there never moves and is never found:
- its own map-wide hunt search (S-0309 claim 7) finds nothing, because the zone test compares a
  cell without a region with region cells (S-0212 claim 4);
- the enemy's hunt search rejects it for the same reason;
- with the rest of its group dead, the game ends `none` at the frame limit with the survivors
  owning about 0.06 to 0.11 of their army.

The stock games end in the first or second sample with that group wiped out. Pairings with the
infantry on side A, in 6 of 6 seeds:
- M-0001: YTNK against E1;
- M-0001: MIND against E1;
- M-0002: INIT against E1;
- M-0002: VIRUS against E1 and VIRUS against E2;
- M-0001: YTNK against E2, partly.

The lead's note on M-0001 and M-0002 (main f81fc29, *Notes*, "Cliff cells in large groups")
confirms the setup side: the stock harness placed units on these cells like anywhere else, and in
the stock games of the pairings above every game still ended by the third sample with one side
owning nothing, none at the frame limit. Our placement already matches, so the difference is in
the simulation, which this request asks about.

Code: `crates/deterministic/sim/src/regions.rs` (`same_region`) and
`crates/deterministic/sim/src/combat/acquire.rs` (`candidate`, zone row).

## Questions
1. An infantry unit (or vehicle) that stands on a cell its movement zone cannot enter: does it
   move off that cell by itself, either at once or when it gets a hunt target?
2. In the zone test of S-0309 claim 7 (and claims 11, 37 a), which region does such a unit
   count as being in, as a searcher and as a target? Is the test skipped for it?
3. Does an enemy hunting unit find it with the map-wide search, and does it approach it?

## Possible observation
Place one enemy infantry unit on a rock cell of a cliff edge and a hunting enemy tank group on
open ground nearby; record whether the tank group attacks it, and whether the infantry unit moves.
Then swap the roles, with the infantry unit hunting.

## Current workaround
Current behaviour kept, by the literal reading of S-0212 claim 4: a cell without a region matches
no region cell, so a unit on such a cell does not move by itself, its own map-wide search finds
nothing, and an enemy map-wide search does not find it. The affected arena pairings are reported
as failing for this reason.
