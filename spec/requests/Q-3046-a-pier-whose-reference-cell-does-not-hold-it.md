---
id: Q-3046
title: What does a pier attempt or notice do when the pier's reference cell does not hold the pier, or when the hit cell is a part of the pier cut off from it?
status: answered
related: [S-0511, S-0307, S-0010]
answered-by: [S-0511]
---

## Need

S-0511 claim 2 sends an attempt at a no-deck pier cell (state 0 to 3, the right sub-tile) to the
pier, and claim 4 sends a notice arriving at a pier cell to it. Claim 6 acts on the pier through
its *reference cell* (Terms: the pier's cell at sub-tile 4 along x, 2 along y): the pier's state,
the tile replacement over the eight-neighbour component of the reference cell's tile, the height
drop to the reference cell's level minus 4, the span checks and the named cell.

Our engine finds the reference cell from the hit cell by rectangle arithmetic: the tile's origin
from the hit cell's sub-tile and the tile's width (S-0307 claim 23), then the reference sub-tile.
On a map that paints another tile over part of a pier, that cell need not hold the pier at all.
GitHub issue #166 found two results the specs do not settle:

- (a) The computed cell holds another tile (clear ground, say). The tile replacement then took the
  component of *that* tile, so one hit rewrote every connected cell of it across the map, and a
  hit cell in state 3 lowered them all and counted as a success.
- (b) The hit cell is a pier cell that an overlaid tile has cut off from the reference cell's
  component (S-0511 Terms: "a different tile laid over part of a pier is not changed, and can
  disconnect cells from the affected component"). After the pier collapses, the cut-off cell
  keeps its old state (0 to 3). A later attempt at it acted on the collapsed pier with the cut-off
  cell's state: state 3 again, then a second collapse four levels lower.

## Questions

1. When the cell at the pier's reference position holds no pier tile (or another tile than the
   hit cell's), does an attempt at a remaining pier cell do anything? Does it count as a success
   or a failure, and does a notice arriving at such a cell do anything?
2. When the reference cell holds the same pier family and axis but in another state than the hit
   cell (a part of the pier cut off by an overlaid tile, the reference cell's part since
   collapsed or damaged), which state decides the attempt or notice: the hit cell's tile, or the
   reference cell's? In particular, can a collapsed pier be attempted again through such a cell?
3. Is the answer the same for direct attempts (claim 2's pier row) and for notices (claim 4)?

## Possible observation

A custom map with a pier along x whose centre cell (sub-tile 4) is painted over with clear ground,
and a second one whose sub-tile 8 cell is painted over so that the sub-tile 12 cell is cut off.
Attempt each remaining pier cell with a wall warhead (S-0511 claim 1) and watch the tiles, heights
and the bridge-destroyed event.

## Current workaround

`bridges::pieces::pier_reference_of` takes the computed cell as the reference cell only when it
holds the hit cell's own tile number at the reference sub-tile. Otherwise an attempt at the cell
does nothing and fails, and a notice arriving there changes nothing, each with a warning (as
claim 2 does for a high-bridge cell with no responsible section). So in case (b) a cut-off cell
whose state differs from the reference cell's is not attempted; a cut-off cell in the reference
cell's own state still acts on the pier. The code cites this request.
