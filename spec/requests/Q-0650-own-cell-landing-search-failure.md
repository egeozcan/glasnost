---
id: Q-0650
title: Landing spot when the own-cell search near a close building finds nothing
status: answered
related: [S-0501, S-0212]
answered-by: [S-0501]
---

## Need

S-0501 F8 and claim 41 (revision 7): an aircraft with dock types and no pad picks the
nearest house building that has a free spot nearby. When that building's weighted distance is
below 256 leptons, the aircraft seeks a free cell near its own cell instead (a 1 × 1 search on
its own cell). The spec does not say what happens when that own-cell search finds no cell,
for example because the aircraft's own cell and its neighbours are all occupied or impassable.
`air::landing_spot` must decide this.

## Questions

1. When a building is chosen below 256 leptons and no free cell is found near the aircraft's
   own cell, where does the aircraft go to land? The spot near the chosen building, the
   other-object fallback of claim 41, the aircraft's own current cell, or something else?
2. If it is the other-object fallback, does that fallback still skip the chosen building, or
   can the building count as one of the other objects?

## Possible observation

Place a house aircraft with a dock type, with no free pad, over a small patch of cells that is
fully occupied or impassable (for example surrounded by walls and units), with the house's
pad building (of the aircraft's first dock type) less than 4 cells away, so that its
weighted distance (divided by 4) is below 256 leptons. Order it to return to base and see where it tries to land.

## Current workaround

`air::landing_spot` uses the free spot near the chosen building (marked `Q-0650 (interim)` in
the code and in docs/development.md). No test depends on this case.
