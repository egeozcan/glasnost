---
id: Q-2702
title: A scatter's extra locomotion that sends a scatter request of its own
status: answered
related: [S-0208, S-0209]
answered-by: [S-0208]
---

## Need

S-0208 r46 claim 6, *Destinations given during a unit's update*: a walker standing with no
walking target and no stored path that accepts a scatter whose destination comes from the
nearby-cell search (claim 13) has one more locomotion at once, in which it searches and chooses
its walking target. A locomotion is where a walker asks to step into its next cell (S-0209 claims
6 to 10). When that cell holds a stationary friendly unit, S-0209 claim 8 sends a scatter request
with no threat point, forced and move-anywhere, to every object listed there. Such a request takes
its destination from the nearby-cell search too, so a unit that accepts it and stands with no
walking target and no stored path has an extra locomotion of its own, inside the first one, and
so on.

A unit that accepts a request has its stored path dropped (claim 12), so a unit already inside its
own extra locomotion can accept a second request lower down and then qualify for a second extra
locomotion inside its first. The specs give no bound, so in a crowd the chain is not known to end.
The engine needs one. The code is `crate::scatter::answer` and `crate::walk::extra_locomotion`.

## Questions

1. Does a walker's extra locomotion at a scatter make the asks of S-0209 claims 7 to 10 (the
   re-search at a blocked step, the crowding request of claim 8, the gate and the moving-blocker
   wait), or does it only search and choose a walking target in a clear cell?
2. If it does make them: can a unit that is already in the middle of its own extra locomotion get
   another one, when a request sent from inside it reaches the unit again?

## Possible observation

Two allied infantry units *A* and *B* standing still on neighbouring cells of open ground, with
every cell around them held except one cell beyond *B*, and a crusher's request (or a third unit
asking for room) sent to *A* so that its nearby-cell search gives a cell beyond *B*. Record per
tick, with the draw recorder of S-0208 scenario W9, the destinations, walking targets and draws
of *A* and *B* in the tick of the request.

## Current workaround

`// design:` interim in `walk::extra_locomotion`. The extra locomotion is a full locomotion of the
standing walker, with the asks of S-0209. A unit gets no extra locomotion while one of its own is
already running further up the same call chain. Each unit is in the chain at most once, so the
chain ends. Every other unit that accepts such a request gets its extra locomotion as claim 6
says.
