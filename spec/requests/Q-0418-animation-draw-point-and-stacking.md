---
id: Q-0418
title: Where an animation is drawn: draw point, vertical draw shift, shadow order, stacked copies and translucent blending
status: answered
related: [S-0102, S-0109, S-0108]
answered-by: [S-0109]
---

## Need

S-0102 claim 19 says animation sprites are drawn centred on "draw points
their own specs give", but S-0109 gives no draw point. S-0102 claim 13 says a
type's vertical draw shift enters the depth bias "because shifting an image
down by n rows would otherwise make it n depth units nearer", which suggests
but does not state that the shift moves the image. Building slot animations
(S-0108 claims 10 and 11, S-0102 claim 17) are now drawn and need these
answers. The code is `crates/app/src/mapview/build.rs` (`slot_animations`)
and `crates/app/src/mapview/slots.rs`.

## Questions

1. What is an animation's draw point: the screen pixel of its position
   (S-0101 F1 with the height lift), or another point?
2. Does a positive vertical draw shift move the image down by that many
   pixels? Does it move the shadow image frame of a shadow type (S-0109
   claim 10) too?
3. For a shadow type, is the shadow image frame drawn before or after the
   image itself?
4. A tiled (stacked) type: how many copies are drawn, and how far apart
   (S-0102 claim 13 says each further copy is shifted up)?
5. How are translucency codes 1, 2 and 3 (25 %, 50 %, 75 %) blended with the
   pixels below, in the 16-bit pixel format of S-0103 claim 9?

## Possible observation

A test animation type with a sprite of labelled frames, played by a trigger
at a known cell, in four variants: no shift, a positive shift, a shadow type
and a tiled type. Record the pixel rows the image and shadow occupy and the
colours where translucent pixels overlap a uniform background.

## Current workaround

The draw point is the screen pixel of the position moved down by the shift;
the shadow image frame is drawn first, at the same place as the image (S-0109
claim 10 says "at the same place"); a tiled type is drawn
as one copy; translucent animations are drawn opaque. Each is a scene note
and is listed in docs/development.md (interim rendering rules).
