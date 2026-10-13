---
id: Q-0072
title: How an attacking unit approaches a target it cannot fire at yet
status: answered
related: [S-0215, S-0307, S-0213]
answered-by: [S-0207, S-0215]
---

## Need
S-0215 claim 23 says that an actor in *attack* with a target "approaches and
engages the target (the combat and movement specs own how)", and S-0307 claim
12 says movement rules that look for a position from which to fire use the
range comparison with the candidate position as source point. No approved
spec gives the approach itself, so a unit ordered to attack an object beyond
its weapon's range (S-0213: the attack order sets the target and no
destination) does not move (`process::process_attack`).

## Questions
1. When does an infantry unit or vehicle in *attack* move towards its target:
   whenever the firing check answers *out of range*, or on other answers too
   (for example a blocked line of fire)?
2. Which destination does it take: the target's cell, the target object
   itself, or a cell found by a search for a position from which the range
   test of S-0307 passes (with which search order, radius and tie rules)?
3. When does it stop: as soon as the range test passes from its position,
   at a cell centre, or on arrival? What happens to its destination then?
4. Does it re-plan when the target moves, and how often (each processing of
   *attack*, on a timer, or only when it arrives)?
5. Does a building, or a unit whose type cannot move, do anything instead?

## Possible observation
S-0215 scenario B9 with the target placed beyond the attacker's range on
open ground, the recorder sampling the attacker's destination and position
each tick, for a short-range infantry unit and a vehicle; then the same with
the target moving away.

## Current workaround
An actor in *attack* does not approach: it fires when the firing check lets
it, and otherwise keeps its target and stays where it is.
