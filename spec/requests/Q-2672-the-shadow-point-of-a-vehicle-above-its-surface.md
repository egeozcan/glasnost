---
id: Q-2672
title: Where the voxel shadow of a vehicle above its surface (a hovering one) is drawn
status: open
related: [S-0105, S-0102, S-0106, S-0101]
answered-by: []
---

## Need

The lead's play note 47 of 2026-11-05 (M8 A4 round 8): "hovercraft doesnt have a shadow". Our
view drew a vehicle's voxel shadow around the body's own draw point, its lifted point, for every
vehicle: S-0105 claim 40 gives the shadow draw point of aircraft only, and S-0105's Open
questions ("Vehicle shadow vertical placement") and S-0102 claim 18 leave vehicle shadow points
unresolved. A hovering vehicle stands above its surface, on land and on water, so its shadow,
drawn around the lifted point, fell under its own body, and claim
39's own-image exclusion (which "applies ... to hovering vehicles") then removed every shadow
pixel the body covers. Only a sliver showed at the edge of the hull: in the lead's eyes, no
shadow.

## Questions

1. For a vehicle whose height above its surface is greater than 0 (a hovering vehicle at rest or
   moving, on land and on water), where is its shadow draw point: at the screen projection of its
   horizontal position at the surface height under it (as claim 40 does for aircraft on
   terrain), at the body's own lifted point, or somewhere else?
2. Does that point move with the vehicle's height above the surface (a fixed surface point, or a
   point that moves by some fraction of the height), and does it follow the body's rocking or
   bobbing (S-0106)?
3. Is the surface under it the terrain, or the bridge deck when the vehicle hovers over a high
   bridge? Over water, is it the water surface height?
4. Is the answer the same for other vehicles drawn above their surface (one being lifted or
   dropped, one on a slope's upper edge), and for a vehicle below its surface (height less than
   0)?
5. Does claim 39's own-image exclusion then compare the shadow pixels with the body's image at the
   body's own screen position (so that a shadow moved off the body is no longer cut by it)?

## Possible observation

A hovering vehicle at rest on flat land, then on water, then on a high bridge, each at two
facings: a screen capture with the shadow's pixel offset from the body's draw point, the
vehicle's height above the surface, and whether any shadow pixel shows beyond the hull. S-0105
MS-27's set-up covers the exclusion.

## Current workaround

Our design (M8 A4 round 8), cited `design: Q-2672 interim`: a vehicle above its surface (the
height above the surface from the simulation, 0 on a bridge deck) casts its shadow at the
projection of its horizontal position at that surface's height (S-0101 claims 1 and 3), as claim
40 places an aircraft's on terrain; claim 39's exclusion compares each shadow pixel with the body,
turret and barrel images at their own screen position. A vehicle at or below its surface keeps
the body's point, as before (`mapview::build::vehicle_shadow_point`).
