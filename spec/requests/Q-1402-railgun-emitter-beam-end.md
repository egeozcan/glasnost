---
id: Q-1402
title: The railgun emitter's target point: where is "the end of the beam"
status: answered
related: [S-0616, S-0306]
answered-by: [S-0616]
---

## Need
S-0616 claim 1 gives a railgun emitter the target point "the end of the beam" and says "the
railgun's beam is the shot itself (S-0306)". No approved spec gives the beam's end point for a
railgun shot: whether it is the target's position, the projectile's impact point, or a point
moved by inaccuracy or by an obstacle.

## Questions
1. At a railgun weapon's shot, which point does the railgun line (claim 17) run to: the target
   object's position, the target cell's centre, the projectile's impact point, or another point?
2. Does that point change when the target moves before the line is made?

## Possible observation
P1's railgun row, with the recorder logging the line's last particle against the target's
position for a moving and a standing target.

## Current workaround
The projectile's stored impact point (as the magnetic beam of S-0607 F3 uses), or the target's
position when the shot launches no projectile (`emitters::effect_weapon_emitter`).
