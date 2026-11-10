---
id: Q-1761
title: An anti-air-only defence building and ground units (retaliation, orders on aircraft)
status: answered
related: [S-0306, S-0309, S-0214, S-0304, S-0215]
answered-by: [S-0306, S-0309]
---

## Need

The lead's play note: an anti-air defence building fired at ground units. The approved specs
allow this in one route. S-0309 claim 5 keeps such a building's automatic searches to aircraft
when its weapons' projectiles are not anti-ground. But retaliation (S-0309 claim 18) tests only
the firing check, and S-0306 claim 23 says "The firing check does not test the anti-ground flag
for infantry, vehicle, aircraft or building targets". That absence has only medium confidence.
So a building whose only weapon's projectile is anti-air and not anti-ground would retaliate
against a ground unit that hits it, whenever the versus value passes S-0304 claim 3.

S-0214 row B7 ("*g* is attack; it has a weapon in its first slot, and that weapon's projectile
cannot hit ground targets ... | nothing") gives *nothing* for every hovered object, airborne
aircraft included. That reading would forbid a player from ordering such a building to attack an
aircraft. The scenario list tests B7 only over an enemy tank.

Code: `glasnost_sim::combat::acquire::retaliate` and `combat::check::projectile_illegal`
(retaliation), `glasnost_sim::targeting::building_on` (row B7).

## Questions

1. A defence building whose only weapon's projectile is anti-air and not anti-ground is hit by
   an enemy ground vehicle (or infantry) within its range. Does it fire back at that unit? Does
   the answer change when the attacker is out of range, or when the building's owner is a
   computer player?
2. Can such a building's weapon be fired at a ground unit in any other way (an attack order of
   any kind, a script or trigger, a target kept from an earlier search)? If so, does the shot
   reach and damage the ground unit?
3. With such a building selected, hovering over an enemy aircraft in flight, and over a landed
   enemy aircraft: does the player get an attack action? Is the answer the same with force-fire
   held?

## Possible observation

S-0306 scenario P16 and S-0309 scenario A5, with a test building type whose only weapon has an
anti-air, not anti-ground projectile. Let an enemy tank attack it from inside its range and
record whether it shoots back. Then select it and hover over an enemy aircraft that is flying,
then landed, with and without force-fire, and record the cursor and any order given.

## Current workaround

The code follows the specs as written: retaliation against a ground attacker is possible, and
row B7 is to be implemented literally. Today the B7 anti-ground and being-sold conditions are
missing, which is a separate code fix.
