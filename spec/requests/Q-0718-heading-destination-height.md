---
id: Q-0718
title: The height of a unit's destination for the bridge heading tests
status: open
related: [S-0511, S-0207, S-0208]
answered-by: []
---

## Need

S-0511 claim 11 hits a unit "when its locomotion is heading to *P*: its current destination lies
in *c* within 104 leptons of *P*'s height" (*P* at the cell's floor + 416), and claim 15 uses the
same test at floor height. For drive and ship locomotion the destination is "the end of the track
it is following, if any, else its destination". Our movement code keeps destinations as cells or
objects, and our drive steps are cell to cell, not tracks with heights. The code in
`crates/deterministic/sim/src/bridges/blow.rs` (`heading_cell`) needs to know which height a destination
has.

## Questions

1. Is the height of a cell destination the height of the layer the unit was ordered to (the deck
   when the cell was a high-bridge cell at the time of the order, else the floor), or the cell's
   current surface height at the moment of the test?
2. For a driving vehicle whose current step ends in *c* but whose final destination lies beyond
   it, does the test use the step's end or the final destination?
3. Does a unit whose destination is an object standing in *c* count?

## Possible observation

S-0511 scenario B6, with the heading vehicle's final destination two cells past the lost section
while its current step ends on the section's cell, and with a vehicle ordered to the cell under the
deck (ground level) instead of onto it.

## Current workaround

For drive and ship locomotion the cell is the target cell of the track step in progress, if any,
else the destination cell; for walk and hover locomotion it is the destination cell. A unit counts
as heading to *c* when that cell is *c*. The height condition is taken as met, for claim 11 and
claim 15 alike. Object destinations do not count. The choices are marked in the code with this request's
ID.
