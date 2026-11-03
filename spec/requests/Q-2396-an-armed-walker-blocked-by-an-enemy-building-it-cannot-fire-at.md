---
id: Q-2396
title: An armed walker blocked by an enemy building that its weapon cannot fire at
status: open
related: [S-0209, S-0212, S-0309, S-0301]
answered-by: []
---

## Need

S-0209 table WP row WP24 makes any other enemy building *enemy obstacle* to an armed walker, and
claim 9 has the walker take the first listed object that is not allied as its target, stop and
discard its path on the repeated ask. It tests only that the walker is armed, not whether the
firing check of the weapon it would use against that building answers *illegal* (S-0301 claim 9),
and claim 9 leaves what happens after the take to the combat specs.

The combat specs give only the drop: S-0309 r94 claim 29 drops the target on *illegal* in the next
check tick (t mod 16 = 0) whatever the behaviour. In the next ask the walker meets the same
building, gets *enemy obstacle* again and takes it again, in the same turn as the drop. In a
seed-10 AI skirmish (lane, real install) an infantry unit on an attack move, whose weapon answers
*illegal* against that building, stood in front of an enemy building in this cycle for about
1,260 ticks: its target was taken, dropped by claim 29, taken again in the same turn, and so on.
After its targeting timer ran out, claim 23 (a)'s search kept the target and queued *attack*,
and the cycle went on in *attack*. No spec says whether the game ever frees such a walker.

The engine's places are `walk.rs` (the *friendly obstacle* and *enemy obstacle* arm of the step
outcome, which sets the target) and `combat/held.rs` (`periodic_check`).

## Questions

1. Does an armed walker whose weapon's firing check answers *illegal* against an enemy building
   in its next cell still get *enemy obstacle* there, and take that building as its target? Or is
   it *blocked* (claim 7: stop, search a new path), as an unarmed walker is under WP22?
2. If it does take the building: after S-0309 claim 29 has dropped it, does the walker's next ask
   take it again in the same tick, with no new path search, so that the walker never moves on
   while the building stands? If not, what ends the cycle (a path search, a give-up count, a
   timer)?
3. Does the path search of S-0212 give such a walker the *enemy obstacle* cost for a cell of an enemy building that its weapon cannot fire at, so that the path it gets runs
   through that building's cells?

## Possible observation

S-0209 scenario C15 part d, with an armed walker whose only weapon's warhead has versus 0
against every building armour, ordered to move to a cell whose shortest path passes through
one cell of an enemy building's foundation and has a detour of a few cells. Record each tick for
200 ticks the walker's target, its cell and whether it has a path. For question 3, record the
walker's first path at the order.

## Current workaround

The engine follows S-0209 claim 9 and WP24 as written (the take, with no firing-check test) and
S-0309 claim 29 (the drop), so such a walker stays in the cycle while the building stands. No
switch.
