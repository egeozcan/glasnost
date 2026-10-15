---
id: Q-0762
title: Entered-by raises for transports, repair pads and other building entries
status: answered
related: [S-0524, S-0011, S-0509, S-0510]
answered-by: [S-0524]
---

## Need
S-0524 claim 12 names garrison entry, infantry and vehicle entry into an ordered target building, sabotage and spy infiltration. It says the remaining entry paths are not classified. Our engine already moves units into transports and onto repair or service pads, and these entries raise no tag (`crates/deterministic/sim/src/mp_triggers.rs`, seams `retaining_entry_seam` and `consuming_entry_seam`).

## Questions
1. Does infantry or a vehicle boarding a tagged transport vehicle raise that vehicle's entered-by tag? Is the passenger the causing object?
2. Does a vehicle docking on a tagged repair pad or service building raise the building's entered-by tag? Does an aircraft landing on a tagged airfield pad raise it? Does a harvester docking at a tagged refinery raise it?
3. Does a unit entering a building that consumes it (for example a unit that is recycled, or one that becomes the building's occupant) get a destroyed-by-anything raise on its own tag before it leaves play?
4. For each entry that raises the tag: is it raised once per entry, or also again when the unit leaves?

## Possible observation
Give each tested destination a repeating tag whose entered-by trigger shows a counter, and give each entrant a tag with a destroyed-by-anything trigger and a separate counter. Order the entries one at a time and record both counters per tick.

## Current workaround
Transport, pad, dock and refinery entries raise nothing. Garrison, sabotage, spy and ordered building entries go through named seams that keep the order of claim 12 when those abilities are built.
