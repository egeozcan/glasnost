---
id: Q-2781
title: A seized vehicle that accepts a scatter request with a threat point
status: open
related: [S-0207, S-0607, S-0215]
answered-by: []
---

## Need

S-0607 claim 9 says a seized unit ignores every new destination, from any rule. S-0207 claim 16
says a vehicle that accepts a scatter request with a threat point queues *move* and takes the
neighbour cell it chose as destination. The two do not say what a seized vehicle does when it
accepts such a request: it takes no destination, but S-0207 claim 16 does not say whether *move*
is still queued, or whether the request's draws (the draw over 1 to 4 for a vehicle with a target,
the direction draw over 0 to 2) are still taken.

The gap is in `scatter::vehicle_request` (`crates/deterministic/sim/src/scatter.rs`), the branch
with a threat point. The branch without a threat point takes no draw and queues nothing, so there
S-0607 claim 9 alone decides: the M8 scatter-from-damage build leaves a seized vehicle's
destination unchanged in that branch.

## Questions

1. A seized vehicle (gripped by a magnetic lifter) accepts a scatter request with a threat point,
   for example the crush-avoidance request of an approaching crusher. Is *move* queued, so that
   it becomes the vehicle's behaviour once the grip ends?
2. Are the request's draws taken as for a vehicle that is not seized (S-0207 claim 16: one draw
   over 1 to 4 when it has a target and the request is not forced, then the direction draw)?
3. Does S-0207 claim 16's refusal list come before the seized unit's refusal, so that a seized
   vehicle refuses a request where the list says so, as any other vehicle does?

## Possible observation

A tiny map with a magnetic lifter that grips an enemy vehicle in *guard*, with no target. While it
is gripped, drive a crusher of a third house towards it so that a crush-avoidance request is sent.
Record each tick the gripped vehicle's behaviour slots and destination, and the scenario random
stream's position against a reference run without the crusher. Then make the lifter let go (kill
it) and record the vehicle's behaviour on the next ticks.

## Current workaround

The branch with a threat point is left as it was before the scatter-from-damage build: a seized
vehicle that accepts such a request takes its draws, queues *move* and is given the destination,
which S-0607 claim 9's other paths would refuse. No vehicle is sent such a request by a hit (the
scatter from damage has no threat point), so this build does not reach that branch.
