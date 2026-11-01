---
id: Q-2226
title: Which point a point delete selects after the last point of a route whose loop ends early
status: answered
related: [S-0213]
answered-by: [S-0213]
---

## Need
S-0213 r84 claim 16e, *The delete key*: when a point-delete order of the
local player's house is applied, the point after the deleted one in the
route of its longest holder, going round a loop, becomes the selected point.
Claim 16e (r84) leaves open which point that is when the deleted point is the
last point of its longest holder's route and that route's loop ends early
(claim 16c, *A loop that ends early*), and lists it under Open questions.

The engine must pick a point (or none) in that case.

## Questions
1. A route holds points before its loop, the loop, and points after the
   loop (a loop that ends early). The player deletes the route's last point,
   which this route's holder has held the longest. Which point becomes the
   selected point: the loop's first point, the first point of the route,
   another point, or none?
2. Does the answer change when the deleted point also stands earlier in the
   same route (a point the route holds twice)?

## Possible observation
Build a loop that ends early on an unselected holder, as in S-0213 claim 16b,
*Who gets a press's orders* (an unselected holder that lacks the pressed
point). Make that holder the deleted point's longest holder, select the
route's last point and press the delete key. Record which marker is
highlighted as selected afterwards.

## Current workaround
`routes::next_after` treats the route as the holder goes round it (claim
16c, r84): after the route's last point comes the loop's first point, the
same rule as for a loop that runs to the route's end. Marked
`design (Q-2226)` in `crates/deterministic/sim/src/routes.rs`.
