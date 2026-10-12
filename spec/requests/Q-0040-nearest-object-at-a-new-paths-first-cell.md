---
id: Q-0040
title: The object nearest the cell's centre in the look at a new path's first cell
status: answered
related: [S-0207, S-0209]
answered-by: [S-0207]
---

## Need
`glasnost_sim::drive` makes the look at the first cell of a new path
(S-0207 claim 15, the table *The cell the first step of the new path
enters*). Its rows for *stationary friendly* depend on "the object nearest
the cell's centre": whether that object's house is allied with the
vehicle's house decides between giving up, a scatter request and nothing.
The claim gives the cell layer to look in (deck when *z* > *H* + 208), but
not how "nearest" is measured or which objects take part, and the choice
changes which of the three rows applies when the cell holds objects of
more than one house.

## Questions
1. Which objects listed in that cell layer take part: every listed object
   (infantry, vehicles, buildings, terrain objects, aircraft on the
   ground), or only some kinds? Does the moving vehicle itself take part
   when it is listed there?
2. How is the distance measured: from each object's position to the cell's
   centre in three dimensions (and then at which height of the centre: the
   surface height *H*, the deck height, or 0) or along the map only? With
   the table square root of S-0204, with its floor, or on the squared
   distance?
3. When two objects are at the same distance, which one is taken: the
   first in the listing order of S-0209 claim 13, the last, or another?
4. When the layer looked in lists no object at all but the outcome is
   still *stationary friendly*, is the result "no object is found" (the
   last row: nothing)?

## Possible observation
S-0207 scenario D17 part a with the cell in front holding two idle units
of different houses (one allied with the mover, one not), placed at
different standing positions of the cell (an infantry pair) or one
infantry and one vehicle on a deck and below it, recording whether the
mover gives up, sends its request, or does nothing.

## Current workaround
Every object listed in the layer except the vehicle itself takes part; the
distance is the three-dimensional distance from its position to the cell's
centre at the surface height *H*, by S-0204's distance (table square root
under `quirks.table_root_distance`), and ties go to the object listed
first. No object found gives the last row. Marked "Q-0040" in
`drive.rs` and in the provisional table of `docs/development.md`.
