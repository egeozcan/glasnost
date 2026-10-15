---
id: Q-0763
title: Order of starting-building restoration and which cell must hold the building
status: open
related: [S-0524]
answered-by: []
---

## Need
S-0524 claim 9 restores each recorded building of a house, by type and cell. The spec leaves open the order in which the record is processed and what "a recorded cell holding a building" means for a building larger than one cell. `restore_starting_buildings` in `crates/deterministic/sim/src/mp_triggers.rs` needs both. The order matters when one replacement's displaced units or blocked foundation affect a later entry.

## Questions
1. In what order does a code-130 action process a house's recorded buildings? Is it the order the buildings were placed in the map, the order they were created, or something else?
2. Is the starting-building record taken in house order, or in one list across houses? Does the order across houses matter for anything?
3. For a multi-cell building, which cell is recorded: its top-left cell, its centre cell, or another one?
4. When a building of the recorded type and house stands so that one of its foundation cells (but not the recorded cell itself) covers the recorded cell, does it count as holding the cell, and is it healed? If it does not count, is a second building of that type attempted there?

## Possible observation
Pre-place two overlapping-candidate buildings of one house, sell both, and park one vehicle so that the first replacement's displacement would move it into the second's foundation. Restore and record which replacement exists and where the vehicle ends. For question 4, pre-place a 2 × 2 building, sell it, place a building of the same type one cell offset, and restore.

## Current workaround
The record is kept in house order, then in object creation order, and processed in that order. Each building records its top-left cell. A building of the recorded type and house holds the recorded cell when any of its foundation cells is that cell.
