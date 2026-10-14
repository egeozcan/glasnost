---
id: Q-0312
title: Which centre a neutral or special house chooses for a unit that gave up after a failed path search
status: open
related: [S-0212, S-0215]
answered-by: []
---

## Need

S-0212 revision 24 claim 23a answers Q-0238 in part. A unit of the neutral or special house
that gives up after a failed path search gets, on the same tick, a destination found by the
nearby-cell search of table T5 around a centre that its house chooses. Choosing the centre takes
scenario randomness, at least one ranged draw, even when the house has no base or start cell.
The claim leaves the centre, its fallback and the draws to S-0215, and S-0215 does not give them
yet. Without the centre our engine cannot assign the destination or take the draws, so it cannot
keep the original's random stream after such a failure. The code is in `seams::give_up_centre`
(crates/deterministic/sim/src/seams.rs), called from `path::failed`.

## Questions

1. Which cell does the house choose as the centre for such a unit: one tied to the house's base
   or start cell, to the unit, or to something else?
2. What does the house choose when it has no base and no start cell, as the neutral and special
   houses usually do not?
3. Which draws does the choice take, in order, with their ranges, and does their number depend
   on the house or the map?

## Possible observation

S-0212 scenario P26: a neutral unit and a special-house unit ordered to an unreachable cell on
a small map, with the random stream recorded around the failure tick. Repeat with and without a
building of that house on the map, and with different seeds.

## Current workaround

No centre is supplied and no draw is taken, so the final search does not run and the unit gets
no destination; it queues *area guard* as claim 23 row 2 says. Interim row in
`docs/development.md`.
