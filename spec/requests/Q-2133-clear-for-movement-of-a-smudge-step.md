---
id: Q-2133
title: Which cells are clear for movement in a destroyed building's smudge steps
status: open
related: [S-0211, S-0312]
answered-by: []
---

## Need
S-0312 claim 19 (g) takes each foundation cell's smudge step (a percent draw and the raw value
of a nearby point) only when that cell is clear for movement. The claim says that buildings,
vehicles and infantry in the cell are ignored for this test, and that the cell's land type and
walls still decide (S-0211). S-0211 claim 11 gives a passability category per cell (open,
crushable, wall-like, partial, blocked, water, beach, off-map), but no spec says which of those
categories count as clear for movement here. The answer changes the draw count of every
building destruction with *C* > 0 that stands next to water, rock, walls or the map edge.

## Questions
1. Which S-0211 categories are clear for movement for the smudge step: only *open*, or also
   *crushable*, *partial* (a terrain object), *beach* and *water*?
2. Does an overlay that S-0211 claim 11 rule 2 decides on (a wall, rock or rubble overlay in the
   foundation cell) count, and is a terrain object in the cell ignored like other objects?
3. Is the test the same in claim 21's second round?

## Possible observation
S-0312 scenario D14: a crewed test building whose foundation covers a beach cell, a cell with a
crushable overlay and a cell with a terrain object (built with a map editor), destroyed with
the survivor chance held at 0; record the draw count of the destruction.

## Current workaround
Only S-0211 category *open* is clear for movement, computed from the land type and overlay as
S-0211 claim 11 does, without the cell's objects (terrain objects included, so a cell with one
counts as *partial*, not clear). Every other category takes no smudge step.
