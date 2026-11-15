---
id: Q-3114
title: The deck height of a steering projectile's deck crossing when the two cells differ in height
status: open
related: [S-0306]
answered-by: []
---

## Need

S-0306 (revision 90) table T1 has three deck-crossing rows. The first row
(straight and arcing) and the vertical row both define the deck height as
the terrain height of the cell of *P*′ + 416, for a crossing in the old or
the new cell. The steering row ("*P*′ strictly crosses the bridge deck height
of a high-bridge cell (old or new cell)", "z at the deck height") does not
say which height that is when the old cell and the new cell have different
terrain heights.

The engine now uses the cell of *P*′ for the first row (sim batch S13, render
batch R9's seam). The steering row is in `combat::projectile::turn`.

## Questions

1. For a steering projectile that moves from one cell into another of a
   different terrain height, where one or both are high-bridge cells, which
   deck height does the steering row test: the terrain height of the cell of
   *P*′ + 416 (as the first row and the vertical row), or each high-bridge
   cell's own terrain height + 416?
2. Which height does "z at the deck height" put the detonation at in that
   case?

## Possible observation

A steering missile fired along a bridge whose two neighbouring cells differ
by a level, crossing the deck height as it moves from the lower cell into
the higher one, and the other way: record the turn and the height of the
detonation.

## Current workaround

The steering row tests each high-bridge cell's own deck height (the old
cell's first, then the new cell's) and puts the detonation at the height
that was crossed, as before.
