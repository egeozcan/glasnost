---
id: Q-1332
title: What the stop on dropping takes from the jumpjet stop rule
status: answered
related: [S-0607, S-0502]
answered-by: [S-0607]
---

## Need
S-0607 claim 13 says that a dropped unit's jumpjet movement "is stopped by S-0502 claim 17's rule", and that claim 15's detour to its cell centre "then decides where it falls". S-0502 claim 17's rule has three parts: it does nothing to a jumpjet that is not moving; it searches for a nearby cell and re-orders the unit to that cell's centre; and when no cell is found it destroys the unit (`quirks.jumpjet_no_cell_self_destruct`). The detour of claim 15 then replaces the destination. The engine must decide which parts happen to a dropped unit. The gap is in `jump::lift_dropped` (crates/deterministic/sim/src/jump.rs).

## Questions
1. When a moving lifted unit is dropped and S-0502 claim 17's nearby-cell search finds no cell (for example over deep water where the unit's own movement zone has no cell nearby), is the dropped unit destroyed by the demolition self-hit, or does it fall as claim 15 says?
2. Does the cell that the search finds have any observable effect on a dropped unit (a reservation, the cell it falls to, its facing), or does claim 15's detour always override it?
3. A lifted unit that is not moving when it is dropped (hovering over its fixed movement point): is anything of claim 17's rule observable for it?

## Possible observation
Scenario L3 with the lifter carrying a unit over a wide lake, letting go mid-crossing (a move order to the lifter); record whether the unit falls into the water or is destroyed in the air, and the cell it falls to.

## Current workaround
The dropped unit's reservation is released and nothing else of claim 17's rule happens: no nearby-cell search, no re-order and no self-destruct. Claim 15's detour to the current cell centre decides where it falls.
