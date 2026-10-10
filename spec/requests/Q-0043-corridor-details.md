---
id: Q-0043
title: Corridor details S-0212 r15 leaves open (end of the coarse search, a bridge's axis, the exemption's layers)
status: open
related: [S-0212, S-0211]
answered-by: []
---

## Need
`glasnost_sim::corridor` implements the corridor of S-0212 claims 13, 13a,
13b and 14 and the estimate *E* of claim 24 (F8). Three points need a rule
the spec does not give.

## Questions
1. **When the coarse search ends.** Claim 13 calls each level's search "a
   cheapest-first search without an estimate" and says it lowers recorded
   costs "so it ends with each sector at the cost of the cheapest route
   found to it". Does a level's search end when a route to the
   destination's sector is taken from the pending routes to be continued
   from (so that its chain is the corridor), when such a route is first
   recorded, or only when no route is pending? Which route's chain is the
   corridor when the destination's sector holds more than one route?
2. **A bridge's axis.** The bridge ends (Terms), the coarse end cells of a
   bridge cell without a link (claim 13b, "none found" rows) and the anchors
   of F8 walk along "the bridge's axis", which "the original keeps with the
   bridge's cells", while S-0211 claim 15 (e) leaves a bridge's direction for
   pathfinding to the bridge specs and S-0212. How does our engine know the
   axis of a bridge cell: from the marking direction of the high-bridge
   overlay that marked it (S-0211 claim 15: indices 24 and 237 along cell y,
   25 and 238 along cell x), or otherwise? What is the axis of a cell marked
   by overlays of both directions?
3. **The corridor exemption.** "A cell one of whose eight neighbours holds a
   vehicle, an infantry unit, a terrain object or a wall section": does an
   object on a neighbour's bridge deck count, or only objects on its ground
   layer? Does an aircraft on the ground count?

## Possible observation
1. Scenario P1 and P12 routes with a reference implementation of each
   reading; maps where the destination's sector is reached first by an
   expensive route and then by a cheaper one through crushable sectors
   (cost 0).
2. Scenario P7 run 10 with bridges of both directions.
3. Scenario P1 with a unit parked on a deck beside the corridor.

## Current workaround
1. A level's search ends when a route to the destination's sector is taken
   from the pending routes; the corridor is that route's chain.
2. No axis is kept, so no bridge end is found: the coarse end cell of a
   bridge cell is the cell itself (as departure D6 does when no bridge end
   lies inside the usable area), and the legs of F8 fall back to it (0).
3. Objects listed on either layer of a neighbour count; an aircraft does
   not.
All three are in the provisional table of `docs/development.md`.
