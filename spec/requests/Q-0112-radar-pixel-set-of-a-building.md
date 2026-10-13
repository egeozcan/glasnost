---
id: Q-0112
title: Which radar pixels a building is listed at
status: answered
related: [S-0410]
answered-by: [S-0410]
---

## Need
S-0410 claim 8: "a building lies at a set of radar pixels around the pixel
of its stored position that covers its foundation at the current scale (low
confidence for the set)". The set is not given: its size and shape at scales
below and above 1, and whether it depends on the foundation's width and
height or on the cells of the foundation. `econ::minimap::listed` must give
a building a pixel set.

## Questions
1. For a building with a foundation of W × H cells, which radar pixels show
   it at scale s, in terms of the F4 pixel of its stored position, s, W and H?
2. Does the set follow the foundation's cells (for example the F4 pixel of
   each cell's centre) or a rectangle around one pixel?

## Possible observation
S-0410 scenario R10 (two foundation sizes, on a large and on a small map).

## Current workaround
A building is listed at the F4 pixels of the centres of its foundation
cells (each pixel once), with its own arrival at each.
