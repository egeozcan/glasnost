---
id: Q-0800
title: Which position a building has in the storm strike witness test
status: answered
related: [S-0513, S-0209, S-0204]
answered-by: [S-0513]
---

## Need

S-0513 claim 18 (revision 10) picks the strike witness of a storm strike from the ground-layer listing of the struck cell. The candidates include buildings. Each candidate's distance is the table root of x² + y², where x and y are "its position's integer lepton coordinates modulo 256". The test is clear for an infantry unit or a vehicle, whose position lies inside the cell.

A building can cover several cells. The code is `cell_state` in `crates/deterministic/sim/src/superweapons/storm.rs`, which reads the position the simulation stores for the building (its `Object::pos`).

## Questions

1. For a building whose foundation covers more than one cell, which lepton position is meant: the centre of its foundation, the centre of its top-left cell, or another point? Is it the same position in every foundation cell's test?
2. Does the building appear in the ground-layer listing of every cell of its foundation, so that a strike on any of those cells can pick it as the witness?
3. For a one-cell building, is the position the cell centre (128, 128)? Then its distance is always 181, so any infantry unit or vehicle standing nearer the cell's top corner would win.

## Possible observation

S-0513 scenario SW7 part d, with a two-by-two building. Strike each of its four cells with an infantry unit standing at a known offset in the cell. Record whether debris appears, which tells whether the building or the infantry unit was the witness.

## Current workaround

The code reads `Object::pos` of the building as the simulation stores it, and lists the building in every foundation cell that the cell lists hold. No invented rule beyond that. This is recorded as an interim row in `docs/development.md`.
