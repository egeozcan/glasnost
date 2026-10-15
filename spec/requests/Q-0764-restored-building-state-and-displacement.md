---
id: Q-0764
title: State of a restored starting building, and where displaced units go
status: open
related: [S-0524, S-0212, S-0406]
answered-by: []
---

## Need
S-0524 claim 9 creates a missing starting building at its cell and displaces mobile occupants "to nearby cells". The spec leaves open the state of the new building and the options of the nearby search. `replace` in `crates/deterministic/sim/src/mp_triggers.rs` needs both. The failed-return removal is a quirk switch, but the successful path also has open choices.

## Questions
1. Does a restored building appear complete and working at once, or does it play its build-up animation first? Is it at full health? Does it start powered, and does it begin any production or superweapon charge from the start?
2. Which mobile objects are displaced: only ground units in the foundation cells, or also units on a bridge deck, aircraft landed there, or units in cells next to the foundation?
3. Where does a displaced unit go: the closest free cell by which search, with which movement-zone and locomotion options? Are infantry kept in a shared cell or given their own cell?
4. If no nearby cell is found for a displaced unit after a successful placement, is it removed, kept off the map, or left in place under the building?
5. Does a displaced unit keep its orders and facing?

## Possible observation
Sell a pre-placed 2 × 2 building, park two vehicles and three infantry in its foundation, and restore. Record the new building's frames, health and power, and each unit's new cell, facing and orders. Repeat with the foundation surrounded by blocked cells.

## Current workaround
The building is created complete, set up as a pre-placed building is (S-0009 claim 40), placed facing 0. Ground-layer mobile objects in the foundation cells are taken off the map before the placement test. After a successful placement, each goes to the result of S-0212 claim 25's substitution search from its old cell, keeping its facing. A unit with no result is removed with a note.
