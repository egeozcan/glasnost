---
id: Q-2636
title: Whether a destroyed vehicle stays an object, off the map with no health, to the end of the tick it is destroyed in
status: answered
related: [S-0203, S-0312, S-0302]
answered-by: [S-0203]
---

## Need

M-0021 (revised 2026-11-05 for Q-2596), Result, *Reading points*, gives for every vehicle
destroyed by a hit at *t* in part B: "the row stamped *t*, off the map with no health left, and none
from *t* + 1 on; its last row on the map is the one stamped *t* − 1, with health left". It adds that
in every part B run the last F vehicle destroyed takes one more hit in the tick of its destruction,
after its destroying hit, which names no attacker, has a warhead from none of the test files and
takes no health; and that some destroyed vehicles have a shot recorded at *t* whose hit comes in a
later tick and names no attacker.

The record's rows are read after the tick has run. So in the record's runs a vehicle destroyed by a
hit at *t* is still one of the game's objects at the end of tick *t*, off the map and with no health,
and is gone by the end of tick *t* + 1.

S-0203 claim 6 (a) says such a vehicle "leaves play at the moment that damage is applied, at the end
of its death effects", and takes no further turn, "not even later in the same turn pass"; S-0312
claim 15 says the same. Neither says whether a vehicle that has left play stays an object of the
game (off the map, health 0) for the rest of the tick, nor when it stops being one. Our engine
removes the vehicle from the game's objects at the hit, so it has no row stamped *t*: the arena
harness (`crates/tools/src/oracle_m0021.rs`, the row *reading points: a vehicle destroyed by a hit
at t has a row stamped t, off the map with no health left*) fails for every death in part B. The
two other parts of the row (no row from *t* + 1 on; the last row on the map is *t* − 1 with health)
hold in ours. Keeping the object would change what can still find it within the tick (searches,
targets, hits, the update order of S-0203 claims 3 and 10), which no approved claim gives.

## Questions

1. Does a vehicle destroyed by damage stay one of the game's objects, off the map and with health 0,
   from the moment it leaves play to the end of that tick? When exactly does it stop being an object:
   at the end of tick *t*, during tick *t* + 1 (at which point), or later? Does the same hold for a
   vehicle destroyed outside a turn pass, and for infantry and aircraft?
2. While it is such an object: can it be hit (the record's last F vehicle takes one more hit after
   its destroying hit, naming no attacker and taking no health), be a target that units keep or
   pick, block or occupy its cell, count for its house (owned units, defeat), or be counted in the
   update order of S-0203 claims 3 and 10?
3. What is the extra hit the record's last F vehicle takes after its destroying hit, in the same
   tick: which object or effect deals it, why does it come only to the last F vehicle destroyed in
   each run, and why does it take no health?
4. A destroyed vehicle with a shot recorded in the tick of its destruction: did it fire before the
   destroying hit in that tick (in its own turn), or can it fire after it? Does a shot in flight
   from a destroyed firer name no attacker when it hits, as the record's later hits do?

## Possible observation

The record's own part B runs: for each death, the objects listed after the tick of the hit and
after the next tick, and any hit or shot that names the destroyed vehicle in those ticks. A second
reading inside tick *t* (after the destroying hit, before the next object's turn) would show whether
the vehicle is still an object while the tick runs.

## Current workaround

None in the simulation: S-0203 claim 6 (a) and S-0312 claim 15 are built as written (the vehicle
is removed at the hit). The harness reports the row's first part as failing with ours and changes
no rule.
