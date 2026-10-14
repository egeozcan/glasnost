---
id: Q-0194
title: What an idle harvester does when its step-off search finds no cell
status: answered
related: [S-0402, S-0212]
answered-by: [S-0402]
---

## Need

S-0402 claim 13 (revision 6) says that an idle harvester standing on the cell of a refinery-flag or
weed-flag building moves off it, 105 ticks after it went idle, to the cell that S-0212's nearby-cell
search gives with the claim's settings, and queues *guard*. The same claim says that what a search
with no result does here is not determined, and the spec lists it under Open questions. The code in
`econ::harvest` (the idle phase of *harvest*) has to do something when the search gives no cell, for
example when every cell in reach of the building's reference point is occupied.

## Questions

1. When the step-off search gives no cell, does the harvester keep its destination as it was, get
   no destination, or get some other destination (the building itself, its own cell, a cell from
   another search)?
2. Does it still queue *guard* in that processing, and is the next processing still after the
   standard delay?
3. Does a search with no result take any random draws, or change anything else (facing, contact,
   the house's no-field record)?

## Possible observation

Scenario H11 part d of S-0402 with every cell around the refinery filled: surround the refinery and
its neighbourhood with the house's own buildings or units so that no cell in the reach region of the
reference point's cell qualifies, let a harvester go idle on the refinery's cell, and record its
destination, behaviour and position over the ticks after the 105-tick idle interval.

## Current workaround

`econ::harvest` (the idle step-off): with no result the harvester keeps its destination as it was
(none, since it is idle), stays where it is, still queues *guard* and is processed again after the
standard delay; no draw is taken. Marked as interim in docs/development.md.
