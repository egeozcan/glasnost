---
id: Q-1561
title: Is a rocket's explosion animation drawn at a nose point below the surface
status: open
related: [S-0506, S-0102, S-0109, S-0303]
answered-by: []
---

## Need

S-0506 claim 19 puts a rocket missile's explosion animation at its nose
point (F9). A missile diving steeply onto a building detonates when its
next move would reach the aim point's height, so its nose point can lie
well below the surface: in our lane a missile came down on a
construction yard standing at world height 624 and its explosion
animation was created at height 390, 234 leptons under the yard's
ground.

S-0102 claims 11 and 13 then give the animation the bias −*L*(390) − 2
= −58. The yard's body (−*L*(624) − 2, with its companion depth image)
and its shadow (−*L*(624) − 4 = −94, flat) are depth writers there, so
by F2 the upright animation loses its bottom 27 or 28 rows. Only its
top 5 or 6 rows could show, and in our picture no pixel of the
explosion is visible. No spec moves an animation's
position up to the surface, so the client draws it this way.

## Questions

1. When a rocket missile detonates with its nose point below the
   terrain or below the top of the building it hits, is its explosion
   animation created at the nose point's height, or at another height
   (for example the surface height under the nose point, or the aim
   point's height)?
2. In the original, is the explosion of a steeply diving missile that
   hits a building on raised ground visible over the building?
3. Does the same apply to other detonations whose point can lie below
   the surface (projectiles, S-0303)?

## Possible observation

A missile launcher on low ground fires at a construction yard on a
plateau several levels higher. Watch the frames right after the hit to
see whether the explosion shows over the yard or is hidden by it.

## Current workaround

The animation is created at the nose point (S-0506 claim 19, F9) and
drawn with the depth of its own position (S-0102 claims 10, 11 and 13,
F2), so it can be hidden by ground writers in front of its point. No
interim change.
