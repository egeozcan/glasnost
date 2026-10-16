---
id: Q-0972
title: Exit-ring cells outside the map or usable area, and a failed placement of a put-out occupant
status: answered
related: [S-0508, S-0209, S-0208]
answered-by: [S-0508]
---

## Need

S-0508 claim 10 chooses the exit cell among the cells of the exit ring that give the first
occupant the entry outcome *clear* (S-0209, S-0208 claim 19). Its Open questions list (a) whether
a ring cell outside the map or outside the usable area can qualify and (b) which conditions other
than standing-position occupancy can make the centre placement of claim 11 fail after a cell
qualified (such an occupant is removed, not destroyed). Both change which cell is chosen and whether
occupants are lost, so the code needs a rule (`structures::garrison::exit_cell`, `put_out`).

## Questions

1. Can an exit-ring cell with no cell on the map qualify? One outside the usable area?
2. Besides the standing-position occupancy that claim 11 says is ignored, what can make the
   centre placement fail?

## Possible observation

G7 and G8 with the building at the map edge and at the edge of the usable area.

## Current workaround

Interim rule: a ring cell with no cell on the map never qualifies; any other cell qualifies
exactly when the first occupant's entry outcome there is *clear* (a cell outside the usable area
gets whatever outcome S-0209 gives it); the centre placement of a put-out occupant never fails
while the occupant exists.
