---
id: Q-0791
title: When a walking, hovering or flying sensor completes a cell movement
status: answered
related: [S-0516, S-0208, S-0503, S-0502, S-0207]
answered-by: [S-0516]
---

## Need

S-0516 claim 22 moves a mobile sensor's disc at the completed-cell movement event, and
claim 13's adjacency reveal also runs there. For driving vehicles and ships the code uses
the arrival at a track target (`crates/deterministic/sim/src/drive.rs`, `spend`). We know
of no matching event for the other movers: walking infantry (S-0208), hover and amphibious
units (S-0503) and aircraft (S-0502). The code in
`crates/deterministic/sim/src/cloak/machine.rs` (`entered_cell`, `cell_movement_completed`)
needs the rule.

## Questions

1. For a walking infantry sensor, when does its disc move? At the moment its position
   enters the next cell, or when it reaches that cell's sub-cell spot?
2. For a hovering sensor, is it when its position enters the next cell, or when it reaches
   the cell's centre?
3. Does an aircraft with the sensors flag move its disc in flight, and when?
4. Does the adjacency reveal of claim 13 run at the same moment for these movers?

## Possible observation

A non-allied cloaked unit beside a path. Walk, hover and fly a sensor unit along the path,
one cell at a time. Record the tick on which the cloaked unit first becomes sensed, and the
sensor's position on that tick.

## Current workaround

For movement other than drive and ship tracks, the completed-cell event happens at the
object's cloak update in the tick its centre cell first differs from the cell recorded
for it. This choice is marked in the code with this request's ID.
