---
id: Q-1773
title: What a deploy order leaves of a moving hover vehicle's destination point and stored path
status: answered
related: [S-0213, S-0415, S-0503, S-0207]
answered-by: [S-0503]
---

## Need

S-0213 claim 10 says a deploy order requests an empty destination, and S-0415 claim 2 makes the
deploy attempt only once the vehicle is still. S-0207 claim 11 says that for a driving vehicle the
old path steps left after a cancelled destination are no continued movement, so a driving vehicle
given a deploy order while it moves along a path stops at the end of its current track and makes
its attempt there. Our engine now drops a non-hover vehicle's stored path when the deploy order
clears its destination (`crates/deterministic/sim/src/orders.rs`, the deploy order).

For a hovering vehicle, S-0503 claim 16 gives a point-equality rule for a person's stop order: the
locomotor's own destination and the stored path are cleared only when that destination and the
heading point differ. S-0503 claim 10 gives the same rule for the stop request of a vehicle that is
switched off. Neither names the deploy order's destination clear. A hover type that deploys into a
building (or deploys in place, S-0517) and is given a deploy order while it moves is therefore not
covered.

## Questions

1. A hovering vehicle whose type deploys into a building is moving along a path of several cells
   and is given a deploy order. Does it stop at once, at the next cell, or does it go on to the end
   of its old path before it makes its deploy attempt? Does the answer differ when the order comes
   while the vehicle is close to the next cell of its path (so that its own destination point and
   its heading point are the same point) from when it comes midway between cells?
2. After the deploy order, does such a vehicle ever move again along the old path without a new
   order (for example once its deploy attempt fails and *guard* is queued)?
3. The same two questions for a hovering vehicle that deploys in place (S-0517).

## Possible observation

A hover vehicle type given the deploy-into-a-building field (or the in-place deploy field), ordered
to a cell eight cells away on open ground and given a deploy order after two cells; record its cell
every tick until it deploys or stays put for 300 ticks. Repeat with the order given just as it
reaches a cell centre.

## Current workaround

A hover keeps its stored path when a deploy order clears its order-level destination (the
behaviour before this request); only non-hover vehicles drop theirs.
