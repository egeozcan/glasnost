---
id: Q-2466
title: The height at which a warped aircraft that cannot land appears at its landing point
status: open
related: [S-0306, S-0513, S-0501, S-0102]
answered-by: []
---

## Need

S-0306 r72 claim 17 (*Render layer of a warp member*) says that a member that does not move from
one bridge deck to another "then stands at no height above its surface" after its arrival hold,
and *An aircraft that cannot land, in flight* says that such an aircraft "at rest at height 0
meets that test (for a warp member, after its arrival hold, by the design rule above)".
Scenario P46 run 2 warps the aircraft *A* and has parts c2 and d2 with *A* "at rest at height 0"
after its hold.

Our engine appears a member at its landing point by an ordinary placement (S-0513 claim 39).
For an aircraft that placement runs S-0501 claim 1: it puts the aircraft on the ground only when
its visiting mark is not set, and at its cruise height otherwise. S-0501 Terms sets the visiting
mark at the type test of a placement when the type is not both selectable and landable, and
never clears it. So an aircraft whose type cannot land, placed on the ground at the start (its
first placement sees no mark), carries the mark from then on, and its warp places it at its cruise
height, in flight, not at height 0.

The two readings give different results for P46 run 2 parts c2 and d2 (seizing and erasure at
rest), and for anything that reads such an aircraft's height after a warp.

## Questions

1. When a warp places an aircraft whose type cannot land (and whose movement is neither rocket
   nor drop-pod movement) at its landing point, at what height above its surface does it appear:
   0, or its cruise height?
2. If at 0, does it stay at rest there after its arrival hold until it gets an order, as one
   placed on the ground by the map does?
3. Does the answer depend on whether the aircraft stood on the ground or (for a type that can
   land) was landed when it was warped, or on its visiting mark?

## Possible observation

P46 run 2 as written: record *A*'s height above its surface in each tick from its placement at
the landing point to some ticks after its arrival hold ends, with no order given.

## Current workaround

The engine keeps S-0501 claim 1's placement, so a warped aircraft that cannot land appears at its
cruise height. The S-0306 r72 claim 17 tests run P46 parts a, c and d in both runs, and parts c2
and d2 in run 1 only; run 2's controls wait on this request.
