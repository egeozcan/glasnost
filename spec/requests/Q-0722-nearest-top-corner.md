---
id: Q-0722
title: Which occupant is nearest the top corner of a cell
status: open
related: [S-0516]
answered-by: []
---

## Need

S-0516 claims 13 and 18 test one candidate per cell: the object "nearest the top corner". The
spec leaves open which object that is when several share a cell. The code in
`crates/deterministic/sim/src/cloak/machine.rs` (`nearest_top_corner`) needs the rule.

## Questions

1. When several objects stand in one cell, which one is the candidate? Is it:
   - the one whose position is nearest the cell's top corner in straight-line distance;
   - the nearest by some other distance;
   - the first one listed; or
   - the last one listed?
2. On a tie, which one wins?

## Possible observation

Put three infantry in one cell, only one of them of a sensor type, each at a different
sub-position. Move a cloaked enemy vehicle next to the cell. Repeat with the sensor at each
sub-position.

## Current workaround

The candidate is the object with the smallest squared distance from its position to the cell's
top corner (the cell's lowest x and y, in leptons). On a tie, it is the first in the cell's list.
The choice is marked in the code with this request's ID.
