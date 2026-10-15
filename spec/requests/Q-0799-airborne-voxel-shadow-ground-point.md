---
id: Q-0799
title: Where an airborne aircraft's voxel shadow is drawn
status: open
related: [S-0102, S-0105, S-0501]
answered-by: []
---

## Need

S-0102 claim 18 and its open question ("Shadow point of voxel models") say the shadow
draw point of an aircraft depends on its movement type and "has not been determined".
S-0105's open question "Shadow vertical placement" leaves the airborne ground point to
S-0102 and S-0106. Our live view needs a point: a paradrop plane at cruise height drew
its shadow at the body's own lifted point, so a flying plane looked as if it sat on the
ground (local lane scene `paratroopers-dropping`).

## Questions

1. Where is the shadow of an aircraft whose height above the surface is greater than 0
   drawn? At the screen pixel of its position at the terrain surface under it, at the
   body's own point, or somewhere else?
2. Does the shadow's position depend on the height (a fixed ground point, or a point that
   moves with the height by some fraction)?
3. Is the answer the same for every aircraft locomotor and for aircraft hovering above a
   helipad or landing (height at or below 0)?
4. Does the surface under the aircraft include a bridge deck, or is it the terrain below?

## Possible observation

A recorded screen capture of a plane at cruise height over flat ground and over a cliff
edge, with the shadow's pixel offset from the body's centre and the terrain level under it.

## Current workaround

`mapview::build::vehicle_voxels` draws a flying aircraft's shadow at the projection of its
position lowered to the terrain level of its cell (`level × 104` leptons, never above the
aircraft's own z), and every other voxel shadow at the object's point. Marked with this
request's ID.
