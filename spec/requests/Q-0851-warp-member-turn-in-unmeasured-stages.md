---
id: Q-0851
title: What a warp member does with the rest of its turn in the stages that S-0513 claim 48 marks unmeasured
status: answered
related: [S-0513, S-0203, S-0215, S-0314]
answered-by: [S-0513]
---

## Need

S-0513 claim 48 (revision 11) and S-0203 claim 4 confirm that a member still in its leaving stage skips ordinary healing, behaviour processing, movement, firing and turret turning, and that in the arrival hold the hold-end test comes first. The same claim marks the rest unmeasured: the first pending turn, the departure turn, each off-map retry, the placement update and the arrival-effects update. It adds that neither the old whole-turn assumption nor its opposite is established.

The engine must still decide, for every update, whether the member takes the rest of its turn after its warp step. The code is `warp::turn` (its return value) and `tick::object_turn`. A decision is observable: a damaged self-healing vehicle heals or not, a due behaviour action runs or not, a vehicle drives or not, a turret turns or not, in each of those updates.

Also unmeasured: whether launch clears the member's path and queued destinations at once (claim 48 confirms only that former driving does not continue, and that target and destination are dropped).

## Questions

1. For each of the stages named above, and for each of self-healing, behaviour processing, movement, firing and turret turning, does the member take that part of its turn in that update?
2. Does launch clear the member's path and queued destinations immediately?
3. If the answers are not known and will not be measured soon, which engine rule does the spec side want for a deterministic default?

## Possible observation

S-0513 scenario SW23 parts (a), (c) and (d).

## Current workaround

The warp step is the member's whole turn in every stage before the arrival hold: nothing else of the turn runs in the pending, departure, retry, placement and arrival-effects updates (the leaving updates are confirmed by claim 48). Launch clears the path, the queued destinations and the driving step. In the arrival hold the hold-end test runs first and the ordinary turn follows. This is the interim rule of the former request Q-0738 narrowed to the unmeasured rows; it is marked `Q-0851 interim` in the code and in `docs/development.md`.
