---
id: Q-0012
title: Overlay image frames, and the palettes of overlays, smudges and terrain objects
status: open
related: [S-0103, S-0104, S-0007]
answered-by: []
---

## Need
S-0103 Open questions leaves the palette of overlays (ore and gems among
them), smudges and terrain objects open, and S-0104 Open questions which
shade table overlays use. S-0104 claim 13 gives a plain terrain object
its cell's tile shade table, but leaves the ore-spawning one open. No spec
says which image frame an overlay shows. Code gap:
`glasnost_render::map` (overlay and smudge drawing).

## Questions
1. Which palette and shade table does each overlay use: ore and gems,
   walls, bridge parts, other overlays? Does the overlay type's data decide
   it, and through which key?
2. Which palette do smudges use, and at which brightness are they drawn?
3. Which palette do ore-spawning terrain objects use (S-0104 claim 13)?
4. Which image frame does an overlay show in the map's initial state, for
   each kind: an ore or gem cell by its overlay data value; a wall or
   fence by its neighbours; a bridge part; other overlays?
5. Is an overlay's shadow drawn (S-0101 claim 11 mentions shadows), and
   which image frame is it?

## Possible observation
S-0103's PE-style screenshots of ore at several stages, a wall line, and
a smudge under a coloured light and a red map tint.

## Current workaround
Overlays and smudges are drawn with their cell's tile shade table and
the brightness S-0104 claim 13 gives them (smudges: scaled ground
level); ore-spawning terrain objects as plain ones. Each overlay shows
image frame 0, ore and gems the frame equal to the overlay data value
when the sprite has that many body frames. All of this is marked
provisional (a viewer toggle hides provisional drawings).
