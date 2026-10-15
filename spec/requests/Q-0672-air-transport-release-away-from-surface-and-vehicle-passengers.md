---
id: Q-0672
title: Where an air transport's release puts a passenger above surface height, and a vehicle passenger
status: open
related: [S-0507, S-0208, S-0501]
answered-by: []
---

## Need
S-0507 claim 13 (revision 4) says how *release* places an infantry passenger "at surface height":
S-0208 claim 3's choice at the aircraft's exact world position, with the aircraft itself excluded.
The same claim also lets a stopped airborne aircraft reach *release* ("a stopped airborne aircraft
can proceed to release"), and an air transport may carry vehicles (claim 2 counts heads, not
kinds). The claim does not say where the passenger appears in those two cases.
`transport::release_position` needs a rule for both.

## Questions
1. An air transport stopped in the air (height above the terrain above 0) with no destination
   reaches *release*. Where does an infantry passenger appear: at the surface below the
   aircraft, at the aircraft's height, or nowhere (placement fails)? Does it take the standing-position
   choice and its random draw in the cell below?
2. Where does a vehicle passenger released by an air transport appear: at the aircraft's exact
   position, at the centre of the aircraft's cell, or by some search? At what height, and does
   anything other than the entry outcome decide failure?
3. Does a vehicle passenger's facing and *move* destination follow the same neighbour preference
   (4, 5, 3, 7, 1, 0, 6, 2) as an infantry passenger's?

## Possible observation
On a tiny flat skirmish map, give an invented air transport type a hover-capable flight and room
for one infantry unit and one small vehicle (named test data). Stop it in the air with no
destination and order *unload*; record each passenger's position, height, facing, destination and
the random-state recorder's draws at release. Repeat landed for the vehicle.

## Current workaround
`transport::release_position` (Q-0672 interim): a vehicle passenger is put at the aircraft's exact
horizontal position on the surface; an infantry passenger released above surface height takes the
same choice in the cell below the aircraft and is put on the surface. Both take the neighbour
preference for facing and destination.
