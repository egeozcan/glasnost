---
id: Q-0011
title: Where sprites are drawn relative to their object, and sprite vehicle facings
status: answered
related: [S-0101, S-0102, S-0107, S-0108]
answered-by: [S-0102]
---

## Need
S-0108 claim 8 says a building's frames are "drawn centred" on its
foundation's top corner, and S-0105 F6 places voxel images. No spec says
where an infantry sprite, a sprite-drawn vehicle or aircraft, a terrain
object, an overlay or a smudge is drawn relative to its position, and
"centred" is not defined for a shape file whose frames are smaller than
its full canvas. Code gap: `glasnost_render::map` (sprite placement).

## Questions
1. "Centred" (S-0108 claim 8): is the centre of the shape file's full
   canvas (width and height from its header) placed on the point, with
   each frame at its own offset inside the canvas? How is an odd canvas
   size rounded?
2. Infantry (S-0107): which point of the body frame lies on the screen
   point of the unit's position (S-0101 F1 with *L(z)*)? Is the shadow
   frame placed the same way?
3. Vehicles and aircraft drawn as sprites: placement, and which image
   frame shows for a facing (how many facings, in which order, and their
   rounding)?
4. Terrain objects (trees and the like): placement relative to the cell,
   and which image frame shows in the initial state? S-0102 Open questions
   mentions a 16-row shift for ore-spawning ones.
5. Overlays and smudges: placement on the cell (S-0101 claim 11 leaves the
   fixed per-kind shifts open).

## Possible observation
For each kind, one object on a flat level-0 cell of a known screen
position, compared with the frame's pixels and its offsets in the file.

## Current workaround
Every sprite is drawn with the centre of its shape file's canvas
(truncated halves) on the object's screen point, the same reading of
"centred" for buildings. This is marked provisional in code and in the
viewer. Sprite-drawn vehicles and aircraft show image frame 0.
