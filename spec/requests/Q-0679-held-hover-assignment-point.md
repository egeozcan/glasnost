---
id: Q-0679
title: The point a held hover remembers for a cell destination
status: open
related: [S-0503, S-0614]
answered-by: []
---

## Need

S-0503 Terms say of the destination *D*: "Unless held, assignment sets its height to the
surface height of its cell, plus 416 leptons on a high-bridge cell. Held assignment retains the
supplied point without this height adjustment." Claims 5 and 17 add that a held vehicle "merely
remembers the supplied destination without this immediate choice or height adjustment".

A move order names a cell, or an object. For an object the supplied point is the object's
position. For a cell no spec says what point is supplied before the adjustment: its centre at
height 0, its centre at the cell's ground height, its centre at the height of the order's
source, or something else. The point matters after the hold timer ends. The throttle distance
of claim 2 is three-dimensional, and claim 7's completion compares heights against 208 leptons,
so a remembered point at height 0 on raised ground would never complete there.

`hover::take` and `hover::destination_point` need the remembered point.

## Questions

1. For a cell destination given to a held hover, what height does the remembered *D* have?
2. When the hold timer ends, is that point adjusted at some later time (a later assignment, the
   next search, or the next choice)? Or is it kept unchanged until the move ends?
3. Does the order-level destination read by behaviours and factory reporting change on a held
   assignment as on any other? The claims only speak of the locomotor's *D*.

## Possible observation

Use S-0503 scenario H14. Raise the ground of the destination cell by several levels, give the
move order while the hold timer runs, and record *D* and the vehicle's final position and
height once the timer ends. Repeat on a high-bridge cell.

## Current workaround

The remembered *D* is derived from the order-level destination as for an unheld vehicle, so a
cell destination takes the ordinary adjusted height. Held assignment makes no choice and no
search, and does not change ρ. The order-level destination is set as usual. The choice is
marked in the code with this request's ID.
