---
id: Q-0761
title: Buildings left outside the local area after a code-40 shrink
status: answered
related: [S-0524, S-0009]
answered-by: [S-0524]
---

## Need
S-0524 claim 8 leaves open what happens in play to a building that a code-40 resize leaves outside the new local area. `resize_local_area` in `crates/deterministic/sim/src/mp_triggers.rs` changes the area, recalculates the cells and rebuilds the reach regions, but does nothing to buildings outside the new area.

## Questions
1. Does a building wholly outside the shrunk area keep working: does it fire, produce, supply power and count for its owner's defeat?
2. Can units still be ordered to attack it, and can units outside the area move back inside?
3. Does a building partly inside the new area behave differently from one wholly outside?
4. Are units outside the shrunk area affected (removed, moved inside, stopped)?

## Possible observation
On a small test map, place a powered defence and a factory near an edge, shrink the local area with code 40 so they lie outside, and then record their firing, production and power, an attack order against them, and the defeat check after the owner's other buildings are destroyed.

## Current workaround
Nothing happens to buildings or units outside the new area; only usable-area tests, the radar and the view follow it.
