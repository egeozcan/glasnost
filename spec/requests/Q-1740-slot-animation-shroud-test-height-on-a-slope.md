---
id: Q-1740
title: Which height the slot-animation shroud test uses on a sloped cell
status: open
related: [S-0513, S-0110, S-0106, S-0410, S-0406]
answered-by: []
---

## Need
S-0513 claim 29b (slot animations, revision 30) says a cell is shrouded for a building's slot
animations when S-0110 claim 8's point test "at the centre of the cell, at its ground height"
finds it so. On a flat cell the height is the cell's level times 104 leptons and there is no
choice. On a sloped cell the surface height at the centre (S-0106 claims 2 and 3) can lie a
whole height level above the cell's level, so the point test's height count *u*, and with it
the cell or cell pair it reads, depends on which of the two heights is meant. S-0410 claim 6
says "surface height" for the radar's use of the same test; S-0513 says "ground height". A
building's foundation cell can be sloped (S-0406 claim 3 lets a water-bound type or a type that
may be placed anywhere stand on a sloped cell), and a slot animation may lie over a
neighbouring building's sloped cell. The code is `mapview::live::slot_cell_shrouded`.

## Questions
1. For the slot-animation shroud test of S-0513 claim 29b on a sloped cell, is the height the
   surface height at the cell's centre (as S-0410 claim 6 gives the radar), or the flat height
   of the cell's level?

## Possible observation
S-0513 SW10 part i1 with cell *E* a sloped cell whose centre lies a level above its level, and
the two cell states that the two heights would read set opposite (as part i4 sets own and
displaced states opposite).

## Current workaround
Interim rule: the surface height at the cell's centre (S-0106 claims 2 and 3), as S-0410 claim 6
gives the radar's point test.
