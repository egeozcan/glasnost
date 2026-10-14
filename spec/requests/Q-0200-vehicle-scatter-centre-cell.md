---
id: Q-0200
title: The scatter centre cell of a vehicle, now that S-0208 takes it from the walking target
status: answered
related: [S-0207, S-0208]
answered-by: [S-0207]
---

## Need
S-0207 claim 16 (revision 8) sends a vehicle's scatter request without a
threat point, and its choice among the eight neighbours with one, around its
"scatter centre cell (S-0208 Terms, with the vehicle's move destination as
its locomotor destination)". Up to revision 13, S-0208 Terms took the scatter
centre cell from the *locomotor destination*, so that mapping gave the cell of
the vehicle's move destination. Revision 14 takes it instead from the new
*scatter reference point*: the walking target while there is one, otherwise
the unit's own position; the locomotor destination no longer enters it, and
"it is never the cell of a farther destination". A vehicle has no walking
target, so S-0207's mapping no longer names anything in S-0208's definition.
The engine must choose the cell the vehicle's search and neighbour choice are
centred on (`glasnost_sim::scatter::vehicle_request`).

## Questions
1. Around which cell does a driving vehicle that accepts a scatter request
   look for its new destination: the cell of its move destination (S-0207
   claim 16's wording), its own cell, or the cell its current track is taking
   it into (the analogue of a walker's next cell)?
2. Does the answer differ between a vehicle standing still, one turning in
   place before a step, and one in the middle of a track?

## Possible observation
S-0207's scatter scenarios with the vehicle driving several cells towards a
far destination when an enemy crusher's or a crowding request reaches it;
record the new destination against the vehicle's own cell, the cell its
track ends in and its old destination cell.

## Current workaround
The vehicle keeps S-0207 claim 16's wording: the centre is the cell of its
move destination when it has one (or movement in progress), otherwise its own
cell. Interim row in docs/development.md.
