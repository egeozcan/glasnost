---
id: Q-0015
title: How a shadow image changes the pixels under it
status: answered
related: [S-0102, S-0104, S-0105, S-0107, S-0108]
answered-by: [S-0102]
---

## Need
S-0107 claim 19 and S-0108 claim 9 draw shadow image frames "as a
darkening", and S-0105 claim 31 uses "the ordinary shadow treatment
(S-0102, S-0104)". Neither S-0102 nor S-0104 says what that treatment does
to a pixel. S-0102 gives the shadows' depth (claims 12, 13, 18) but not
their colour. Code gap: `glasnost_render::compose` (shadow pixels).

## Questions
1. What colour does a screen pixel take where a shadow pixel passes the
   depth test: a fixed fraction of its colour, a darker shade row of the
   palette it was drawn with, a darkening of its 16-bit value, or other?
   Exactly how, with the rounding?
2. Does the light at the shadow's cell (S-0104) or the shadowed pixel's
   own brightness change the result?
3. Where two shadow images overlap on one pixel (an infantry shadow over a
   building shadow), is the pixel darkened twice?
4. Do the shadow pixels of the image frame differ among themselves (index
   values), or does every non-transparent shadow pixel act the same?

## Possible observation
A unit's shadow over flat ground of a known palette colour under neutral
test lighting; read the shadowed pixels' 5-6-5 top bits and compare them
with the unshadowed ones. Repeat with two overlapping shadows.

## Current workaround
Every non-zero pixel of a shadow image that passes the depth test halves
each 8-bit channel of the pixel beneath (rounding down) before 16-bit
packing; overlapping shadows halve again. Marked provisional; the viewer
can hide shadows.
