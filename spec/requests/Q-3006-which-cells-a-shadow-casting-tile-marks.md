---
id: Q-3006
title: Which cells does a shadow-casting terrain tile mark as shadowed, and are a high bridge's shadowed cells its deck cells?
status: open
related: [S-0104, S-0010, S-0211, Q-0009]
answered-by: []
---

## Need

S-0104 claim 13 darkens a vehicle or infantry unit in a *shadowed cell* (−500 under the conditions of
claim 13), and its Terms name two things that mark such cells:

- an intact high bridge marks "the cells it spans", cleared when the bridge is destroyed;
- a terrain tile of a tileset declared a shadow caster, with a shadow-tile count that is not 0, marks
  "a fixed set of cells around it" (Import mapping).

S-0104 Terms, its Open questions and S-0010 claim 18 all leave open which cells a shadow-casting tile
marks. Q-0009 is answered by S-0010 for the tileset semantics, but not for this set.

The client (`crates/app/src/mapview/build.rs`, `Painter::mobile`; render issue #74 on GitHub) takes the
bridge marks as the high-bridge cells of S-0211 claim 15 (the cells with a deck), and marks no cell for
a tile. It needs the tile set, and confirmation of the bridge set.

## Questions

1. For a tile of a shadow-casting tileset placed in cell (*x*, *y*), which cells does it mark as
   shadowed? Give them as offsets from (*x*, *y*), or as a rule. Does the set depend on the tile's
   index in its set, on its size in cells, on the shadow-tile count, or on the heights or slopes of
   the cells around it?
2. Does a multi-cell tile mark cells from each of its cells, or from one of them?
3. Is the mark fixed at map load, or can it change in play (for example, when the tile under it
   changes)?
4. Are the cells an intact high bridge marks as shadowed exactly its deck cells (S-0211 claim 15's
   high-bridge cells), or a different set (for example, without the cell one step against the marking
   direction, or with the bank cells at either end)?

## Possible observation

S-0104 Scenario L15 Part D already places one tile of a shadow-casting tileset on flat ground and
stops a vehicle on every reachable cell within 3 cells of it; the darkened cells give the set. For
question 4, stop a vehicle under a high bridge on each cell of S-0211 claim 15's four-cell marking
of one section, and on the cells next to them along the bridge, under neutral test lighting.

## Current workaround

The client marks a cell shadowed exactly when it is a high-bridge cell (S-0211 claim 15), and a
vehicle or infantry unit that is not on the deck takes claim 13's shadowed-cell rules there. No tile
marks any cell. Both are recorded in `docs/m8-claims/render-issues.md` (batch R4, #74).
