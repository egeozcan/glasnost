---
id: Q-2111
title: A steering projectile fired from a muzzle position at the surface, and absent muzzle positions
status: answered
related: [S-0306, S-0313, S-0308, M-0001]
answered-by: [S-0306, S-0313]
---

## Need
M-0001's DISK against APOC pairing fails on both sides: the stock games are column wins on
every seed, ours are row wins on every seed (64 of 64 seeds on each side in the local lane). In
our games every missile the column fires at a hovering disc detonates in its first turn at the
launcher's own position, so the discs take no damage.

Two approved rules give that, and we follow both:

- We traced the fire point of these missiles (local lane): it is the launcher's own position, at
  the surface. By S-0313 claim 1 that is what a muzzle position of 0, 0, 0 gives, and 0, 0, 0 is
  the Import mapping's value for an absent muzzle-position key.
- S-0306 T1, steering row "*h* ≤ 0: detonate" (high confidence), uses the height above the
  surface at the start of the projectile's turn. A steering projectile created at the surface
  therefore detonates in its first turn.

The stock outcome suggests that one of these does not hold in this case. The code that decides
is `combat::fire::fire_point` (the muzzle position) and the steering branch of
`combat::projectile::turn`.

## Questions
1. Does a steering projectile whose fire point lies at the surface (height above the surface 0
   when it is created) detonate in its first turn, at the launcher? If not, what does its first
   turn do: is the height of T1's row taken after the first move, or is the row skipped in the
   turn of creation, or is the projectile created higher?
2. When a type's art gives no muzzle position for the slot it fires, is the fire point really
   computed from 0, 0, 0, or does it fall back to another key (for example the other slot's
   muzzle position, or the turret pivot of S-0313)?
3. In M-0001's DISK against APOC games, which of the column's weapons hits the discs, and what
   damage do the discs take from it?

## Possible observation
One stock column vehicle of that pairing ordered to attack one hovering disc on flat ground:
record the missile's position and height above the surface for its first 5 turns, where it
detonates, and the disc's damage events.

## Current workaround
None: the code follows S-0313's Import mapping and S-0306 T1 as approved, and the pairing
keeps failing. Nothing was changed for it.
