---
id: Q-3141
title: Which landing cell is "a road cell that is water" for quirks.warp_naval_road_cell off
status: open
related: [S-0513, S-0211]
answered-by: []
---

## Need
S-0513 claim 38 kills a naval warp member "landing on a deck cell or on a road cell
(`quirks.warp_naval_road_cell`, default on = original; off: a road cell that is water is not counted)",
and then "sinks with no credit when the cell is a water tile and the member is not infantry, and is
otherwise killed with no credit".

A cell has one land type. Read with "road cell" as "a cell of the road land type" and "is water" as "a
cell of the water land type", no cell is both, and the off position changes nothing. That is what our
engine does now: in both positions the road test is exactly "the land type is road". (Issue #364.)

A cell can be road and water in another sense: the terrain recalculation gives a cell under a row-one
overlay of S-0211 claim 5 the overlay's land type and keeps the tile, so a road-land overlay on one of
the theater's water tiles makes a cell of the road land type on a water tile.

The same bullet's "when the cell is a water tile" (sink rather than kill) has the same two readings.

## Questions
1. With `quirks.warp_naval_road_cell` off, which cells are not counted as road cells: cells of the
   road land type whose tile is a water tile, or some other cell?
2. In "sinks with no credit when the cell is a water tile", is "water tile" the cell's tile (a water
   tile of the theater, whatever land type an overlay gives the cell), or the water land type?
3. If question 1 has no cell in this model (the off position has no effect), may the switch be
   recorded as having no effect?

## Possible observation
A synthetic map with a road-land overlay on a water tile, a naval unit whose locomotion class has a
non-zero speed multiplier on road land, and a superweapon teleport that carries it onto that cell.
Record whether the ship survives, sinks or is killed.

## Current workaround
The road test is "the land type is road" with the switch in either position, and the sink-or-kill test
reads the water land type (`arrive` in `crates/deterministic/sim/src/superweapons/warp.rs`). Issue
#364 is left open until this is answered.
