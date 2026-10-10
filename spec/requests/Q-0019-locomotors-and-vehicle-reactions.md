---
id: Q-0019
title: Which locomotor a type has, and what a driving vehicle does at a blocked step
status: answered
related: [S-0207, S-0209, S-0210, S-0212, S-0213, S-0215]
answered-by: [S-0207, S-0209, S-0215]
---

## Need
`glasnost_sim` moves vehicles with the drive locomotor (S-0207) and
infantry with the walk locomotor (S-0208). Four rules that these need
belong to "the movement specs", which do not state them.

## Questions
1. **Locomotor identification.** S-0210's import mapping names the type
   key `Locomotor` as a "class identifier" owned by the movement specs. Which
   values select the driving, walking, hovering, ship, flying, jumpjet,
   teleporting, tunnelling, mechanical-walking and rocket locomotors, and
   which locomotor does each kind of type have when the key is absent or
   names none of them?
2. **A driving vehicle at a blocked step.** When the next step's entry
   outcome (S-0209 claim 5f) is *stationary friendly*, *friendly obstacle*,
   *enemy obstacle*, *cloaked blocker* or *closed friendly gate*, what does a
   driving vehicle do (S-0212 claim 22 points to S-0209 and the movement
   specs; S-0209 claims 6 to 10 are for walkers)? In particular, does it send
   the move-anywhere scatter requests that S-0209 claim 12 mentions?
3. **Vehicle scatter.** Which scatter requests does a vehicle accept beyond
   the gate of S-0215 claim 19, and where does it go (S-0213 claim 9, S-0215
   claim 28)?
4. **The vehicle-present flag (S-0209 claim 1).** When does a vehicle clear
   the flag of the cell it leaves?

## Possible observation
A vehicle ordered through a one-cell gap held by an idle allied vehicle; a
vehicle given the scatter order on open ground; a recorder that reads the
vehicle-present flags around a moving vehicle.

## Current workaround
Infantry types walk, vehicle types drive, aircraft do not move. A driving
vehicle meeting any of the five outcomes waits and re-paths as for a moving
blocker (S-0212 claim 22), sending no scatter request. A vehicle refuses
every scatter request. A vehicle clears its flag on a cell when its position
leaves the cell and it has no step into it in progress. All marked
provisional.
