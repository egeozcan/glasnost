---
id: Q-2577
title: Where landing where it is starts its selection over a cell outside the usable area
status: open
related: [S-0501]
answered-by: []
---

## Need

S-0501 r120 claim 6 (*Landing where it is*, Q-2507) takes the new flight point from claim 40's
selection around the cell the aircraft is over when it is given no destination. The claim adds:
"Over a cell outside the usable area, the selection starts from a cell on that area's edge, which
is not traced here."

The engine has to pick a start cell. An aircraft can be over such a cell when it is given no
destination. It may be flying in from a map entry, flying past the border to claim 17's edge cell,
or have drifted out on a fly-by. A stop order can reach it there.

## Questions

1. Which cell on the usable area's edge does the selection start from? Some candidates: the edge
   cell nearest the aircraft's position, the cell where its position's line toward the map centre
   crosses the edge, or the edge cell claim 17 would give.
2. Does the selection then run as it does over a usable cell? That is, F7 and the
   flight-reservation test on that start cell, then claim 40's replacement search around it with
   its draws.
3. If every radius of the search fails, is the flight point that edge cell's centre?

## Possible observation

A human-owned aircraft that may not leave the map now (claim 10). It enters from a map entry
outside the usable area and is given a stop order while still outside that area. Record the cell it
then flies to and lands in, and the position it was given the order at. Repeat with the order given
at several points along the border.

## Current workaround

`air::given_no_destination` starts from the cell the aircraft is over. F7 refuses every cell
outside the usable area, so the selection goes straight to claim 40's replacement search around that
cell. The search accepts only usable cells and takes its draws. If all radii fail, it keeps the cell
the aircraft is over. Cited `design: S-0501 claim 6 and Q-2577`.
