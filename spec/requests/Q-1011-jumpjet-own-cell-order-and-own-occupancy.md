---
id: Q-1011
title: Does a grounded jumpjet's own occupancy keep its own cell from qualifying when it is ordered to that cell
status: answered
related: [S-0502, S-0212, S-0209]
answered-by: [S-0502]
---

## Need

S-0502 revision 5 claim 23 ends with "Claim 16's search does not excuse or lift a grounded unit's own
occupancy: that can affect its search like any other occupancy". S-0212 claim 25 only accepts a cell
whose occupancy record is empty as a whole. A grounded jumpjet's own mark (a standing position of an
infantry unit, the vehicle-present flag of a vehicle) is in the record of its own cell, so an order to
its own cell should not find that cell and should go to a neighbouring one.

S-0502 claim 7 and scenario J2 say the opposite for the same order: "Repeat with an order to the
unit's own cell. Expected: it climbs past one quarter of H, then cruises at that height and lands",
and claim 7's early cruise needs the unit to be over its destination cell at that height. That is
measurable in the original (J2), and it holds only when the own cell is found.

## Questions

1. When a grounded jumpjet is ordered to its own cell, does the nearby-cell search of claim 16 count
   the unit's own mark in that cell's record (so another cell is found), or not (so the own cell is
   found, as scenario J2 shows)?
2. If the own mark does count, how does the J2 own-cell result arise?

## Possible observation

S-0502 scenario J2's own-cell repeat, recording the destination cell chosen for an infantry unit and
for a vehicle ordered to the cell they stand in.

## Current workaround

Interim rule: the unit's own mark is lifted for the duration of the claim 16 search and laid again
after it (as before revision 5), so that the own cell qualifies and scenario J2's own-cell result
holds. Claim 23's sentence is therefore not followed for this case. Noted in `docs/development.md`
(section *Simulation (M6c: jumpjets)*).
