---
id: Q-2212
title: Where does a projectile fly whose target object is a superweapon warp member that has departed?
status: answered
related: [S-0513, S-0306, S-0309]
answered-by: [S-0306]
---

## Need
S-0513 revision 43, claim 39, says that a projectile already in flight towards a warp member does
not lose it at departure. Its target does not become the cell the member stood in (S-0306 claim 17
does not apply), and it keeps the member as its target object. Where it then flies while the member
waits to land, and whether it can hit the member then or after it appears, are left open (Open
questions).

The projectile's own update must decide what to do with a target object that is off the map. Our
`combat::projectile` update has one rule for every target object that is off the map: the target
is lost and the projectile goes on with no target object. This was written for S-0306 claim 17's
lost targets.

## Questions
1. In the projectile's updates after the member's departure, while the member is off the map, does
   the projectile keep flying towards the member's last position? Towards its stored target point?
   Or does it follow the member's landing point?
2. If it reaches its impact point while the member is off the map, does it explode there? Can its
   blast damage the member, which is off the map and phasing in?
3. If the member appears at its landing point before the projectile ends, does the projectile turn
   towards the member's new position (a homing projectile, for example)? Can it hit the member while
   the member is phasing in?
4. If the member is killed by its landing check (claim 37) while the projectile is in flight, does
   the projectile's target become the cell where the member stood before departure, its landing
   cell, or nothing?

## Possible observation
S-0513 SW12 with one holder firing a slow projectile (a homing one and a ballistic one) at the
member a few ticks before its departure. Record the projectile's position and target every tick
until it ends, and the member's health.

## Current workaround
At departure the projectile keeps the member as its target object (claim 39). Its own next update
finds that object off the map and drops it, as for any off-map target object, and it goes on to its
stored target point. A member killed by its landing check makes such a projectile's target the cell
the member stood in before departure (S-0306 claim 17 at the killing hit, `combat::death`), if the
projectile has not dropped it already.
