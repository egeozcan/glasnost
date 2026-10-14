---
id: Q-0225
title: When does the sale of a building with a positive-delay range of one stage or fewer finish?
status: open
related: [S-0408, S-0406, S-0108]
answered-by: []
---

## Need

S-0408 claim 17 (revision 7) gives the tick *E* at which a sale pays and removes the building
for a positive-delay range of at least 2 stages and for a zero-delay range. It leaves
positive-delay one-stage ranges and non-positive stage counts open, "as in S-0108 and S-0406".

S-0406 claim 18 (revision 4) settles the construction side of such ranges: the build-up never
reports completion, so a placed or deployed building of such a type never becomes sellable.
A building of such a type that the map places is complete from the start, though (S-0009
claim 40), so it can be sold, and its build-down then runs the same range.

`econ::sell::process_selling` needs the tick at which the build-down of such a range reports
completion, if it ever does.

## Questions

1. A complete building of a type whose construction range has 1 stage and a positive delay
   (a build-up sprite of 2 or 3 image frames, or a gate with a gate stage count of 0) is sold.
   Does its build-down ever report completion? If so, on which tick relative to the build-down
   start *D*: *D* + 1, *D* + *d_b*, or another?
2. The same for a range of 0 stages or fewer (a build-up sprite of 0 or 1 image frames, so
   *d_b* = 1).
3. If the build-down never completes, does the building stay in *selling* for good, at its
   current health, with no refund and no removal?
4. For a type that names a vehicle to pack into and has no guard spot, does the stage-23 early
   report of S-0408 claim 17 change any of these answers?

## Possible observation

S-0406 scenario PL11 part (e)'s test type (a build-up sprite of 2 image frames and a positive
build-up delay), placed by the map as a complete building and sold. Record the selling stage,
the body stage and the build-down's report per tick for 20 × *d_b* ticks after the build-down
starts. Repeat with a 1-frame build-up sprite.

## Current workaround

`econ::sell` (build-down): a positive-delay range of one stage or fewer is treated like a
zero-delay one and pays and removes at *D* + 1. Listed as an interim row in
docs/development.md.
