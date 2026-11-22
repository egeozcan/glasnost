---
id: Q-3373
title: What a driving vehicle's readings measure to once its installed point is cleared
status: answered
related: [S-0207, S-0212]
answered-by: [S-0207]
---

## Need

S-0207 claim 29 (r92) says that a driving vehicle's movement readings that need its destination's
point use the locomotor destination point of claim 25, "the point that the last assignment to
install one supplied, until claim 25's table clears it". It names the path searches of S-0212
claims 20 to 23, the 512-lepton distance that starts claim 20's recomputation, the braking test of
claim 10 and the give-up test of claim 15.

Claim 25's table can clear that point while the order-level destination stays. The completion
row does so when a track ends with the destination still present and the vehicle in the cell of
the point the destination supplies at that moment. After that the vehicle holds a destination
(a building or a unit) and no locomotor destination point, and no assignment has been made. The
vehicle can then still be moved out of that cell, for example by a scatter request (claim 16), or
the destination can move, when it is a unit.

Claim 29 does not say what those readings measure to in that state.

## Questions

1. For a driving vehicle that holds an order-level destination but no locomotor destination point
   (cleared by claim 25's completion row), and is later moved out of that cell with no new
   assignment, does it begin a track towards its destination at all?
2. If it does, which cell do its searches look for, and which point do its braking and give-up
   test measure to: the point the destination supplies at the time of each reading, the point
   cleared by the completion row, or none?

## Possible observation

A driving vehicle given a move order onto a stationary friendly vehicle *U*, so that its
destination is *U*. It drives to *U*'s cell's neighbour and ends a track in *U*'s cell only if *U*
moves off first; let *U* leave its cell before the driver arrives, then return to it, so that the
driver ends a track in the cell *U* then supplies. Then scatter the driver one cell away with no
new order, and record over the next 100 ticks whether it begins a track, and the cell its stored
path ends in.

## Current workaround

`drive::measured_point` falls back to the point the destination supplies at the time of the
reading when the installed point is absent: S47's rule before claim 29. In the completion row's
own case the vehicle stands in that point's cell, so its no-steps check brings it to rest as
before.
