---
id: Q-1570
title: How a projectile's body is drawn (sprite frame by direction, voxel orientation, palette)
status: answered
related: [S-0007, S-0306, S-0102, S-0103, S-0104, S-0105, S-0101]
answered-by: [S-0306]
---

## Need
The specs give a projectile's image name and art (S-0007 claims 26 and 27), whether it is drawn as
a voxel model (S-0306 Import mapping, `Voxel`), its layer (S-0102 claim 3; the flat art flag puts it
in the surface layer), its depth bias and profile (S-0102 claims 12 and 13, "projectile sprite") and
its shadow flag (S-0102 Import mapping, `Shadow`). They do not say which picture is drawn for a
projectile in flight. The game client (`crates/app/src/mapview/live.rs`, `draw_effects`) therefore
draws our own 7 × 7 diamond at each projectile's projected point with its height (cyan, orange when
arcing, magenta when steering) instead of the projectile's art. The lead's screenshot review of
2026-10-19 asked about shots whose pictures do not show what is flying.

## Questions
1. For a projectile drawn as a sprite: which image frame is drawn? Is it chosen by the direction
   of flight (on screen or on the map, with or without the pitch), from how many directions, and
   how are directions mapped to frame numbers? Does the frame also advance with time (an animated
   projectile sprite), and at what rate?
2. For a projectile drawn as a voxel model: which body facing and pitch is the model drawn with
   (the flight direction of S-0306, the launch facing, or something else), and with which light
   and shade table (S-0105)?
3. Which palette and brightness draw a sprite projectile (S-0103 claim 8 rows; S-0104 light of its
   cell, at which level), and does the art's translucent flag (S-0007 claim 23) apply as 50 %
   translucency?
4. Where is the picture placed relative to the projectile's projected position with the height
   lift (S-0101 F1, F2): centred, or with an offset?
5. How is the projectile's shadow drawn when the `Shadow` art flag holds: which image (the body's
   own shadow frame, a generic shadow), at which point (the terrain under it), and with which
   darkening?
6. Is any picture drawn for a projectile whose image name is empty (S-0007 claim 26) or whose art
   names no existing file?

## Possible observation
Fire a test weapon whose projectile uses a named test sprite with distinct numbered frames, at
targets in sixteen directions and at a higher and a lower target; record the frame drawn each tick.
Repeat with a test voxel projectile and with the `Shadow` flag on and off, on flat, evenly lit
ground and on a cell lit by a coloured building light.

## Current workaround
Interim rule (docs/development.md, *Visual review 2026-10-19*): the game client and the viewer
draw our own 7 × 7 diamond for each non-invisible projectile in flight, as they have since M4. No
projectile art and no projectile shadow is drawn.
