---
id: Q-0570
title: What does an oversized view show beyond the local area?
status: answered
related: [S-0101, S-0110, S-0009, S-0409]
answered-by: [S-0101]
---

## Need

S-0101 claim 8 (r10) centres an oversized axis of the view on the local area, a departure from the
original confirmed by the project lead. On a window wider than the local area this leaves a band on
each side of the battlefield that lies outside every position F5 lets an ordinary view reach. No
claim says what is drawn there.

S-0110 claim 8 and its Edge cases leave light unchanged outside all legal canvases and say whether
the border cells "lie inside the local area, and so can be seen, depends on the map". With the
whole map revealed (S-0409 claim 15), the tiles of the map's border cells outside the local area are
drawn fully lit in that band. On the lead's review map they show as a broken line of lit tile
pixels (dashes) running down the band at the map's right edge, which the lead reported as a
defect. An ordinary view never shows them, because F5 keeps it inside the local area.

## Questions

1. On an axis where the view is oversized, what does the battlefield show at pixels outside the
   local area's pixel rectangle of F5 (x from 30·(2·V_x − M_w), width 60·V_w; y from
   15·(M_w − 5 + 2·V_y), height 30·V_h + 135): the world drawn as at any other pixel (terrain,
   objects, shroud), black, or something else?
2. If it is the world, do cells outside the local area count as shrouded there regardless of the
   shared off-map state, or does S-0110's rule for non-legal cells apply unchanged?
3. Does the answer differ for the vertical band, where F5's fixed centre lies 7.5 pixels above the
   rectangle's middle?

## Possible observation

None in the original: claim 8's centring is our departure, so this is a design decision for the
lead. M3b's oversized runs on our engine can confirm the result once decided.

## Current workaround

On an oversized view the client fills every battlefield pixel outside the local area's pixel
rectangle with black, after drawing the map image and before the overlays
(`crates/app/src/game/session.rs`, `outside_local_area`; `glasnost_render::projection::local_area_rect`).
A view that is not oversized is unaffected. The simulation is unaffected.
