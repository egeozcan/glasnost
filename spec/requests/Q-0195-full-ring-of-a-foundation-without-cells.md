---
id: Q-0195
title: The full exit ring of a foundation without cells, with the exit-list switch off
status: answered
related: [S-0404, S-0108, S-0406]
answered-by: [S-0404]
---

## Need

S-0404 revision 4, Edge cases (`quirks.foundation_exit_lists`): with the switch off, "every
foundation's list is its full ring of outside cells": the row below the foundation from left to
right (corners included), then the cells beside it from the bottom row upwards, left before right,
then the row above it from left to right, corners included. The 0 × 0 word (S-0108) has no
foundation cells (S-0406 claim 2, S-0009 claim 38), so it has no rows or sides. Read literally with
W = H = 0, the formula gives (−1, 0) and (0, 0) for the row below, no side cells, and (−1, −1) and
(0, −1) for the row above: four cells around a point, one of them the top-left cell itself. With the
switch on the 0 × 0 list is empty (Constants). The code in `econ::release` (the infantry row of
claim 13) has to choose a list for such a factory building with the switch off.

## Questions

1. With `quirks.foundation_exit_lists` off, is the exit list of the 0 × 0 word empty (as with the
   switch on), the four cells of the literal formula, the eight neighbours of the top-left cell, or
   something else?
2. If it is not empty, in what order are its cells tried, and where does the unit appear (claim 13
   moves each coordinate of the exit cell that lies outside the foundation one cell back into it,
   which a foundation without cells does not define)?

## Possible observation

None in the original: the switch off is our engine's own behaviour, so this is a design decision
for the spec rather than a measurement.

## Current workaround

`econ::release::ring`: a foundation without cells has an empty ring, so with the switch off a 0 × 0
infantry factory finds an exit only at its door cells, as with the switch on. Marked as interim in
docs/development.md.
