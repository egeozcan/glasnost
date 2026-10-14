---
id: Q-0197
title: When a driving harvester "finishes entering" a cell, and how it scatters on a refusal (second route to unload)
status: open
related: [S-0402, S-0207, S-0208, S-0210]
answered-by: []
---

## Need

S-0402 revision 6, claim 10, last paragraph: "While the harvester drives, a tethered harvester in
*enter* that finishes entering a cell whose neighbour in step direction 0 holds its refinery also
asks for *unload* as in row 4; when the refinery refuses, the harvester scatters (S-0208, claim
13)." The code has to hook this into ground-vehicle driving (S-0207), and two points are not
defined anywhere in the approved specs:

- *Finishes entering a cell.* S-0207 has two candidate moments: the position passing into the new
  cell between track points (where `drive::set_pos` moves the object's cell membership, and where
  S-0210 claim 9 runs its "drives into a new cell" test), and the end of the track whose target is
  that cell (S-0207 claims 4 and 12). A track can also pass through an intermediate cell (a
  two-cell curve, S-0207 T1), which the first reading counts and the second does not. The two
  readings give different ticks for the queued *unload*, and so different draws.
- *Scatters (S-0208, claim 13).* S-0208 is the infantry movement spec. A harvester is a vehicle,
  whose scatter is S-0212's vehicle request. It is not said whether the harvester takes S-0208
  claim 13's direction draw *j*, whether the request is forced, and whether the refusal also
  breaks off contact or the tether (row 1 keeps contact for a tethered harvester).

## Questions

1. Which moment is "finishes entering a cell": the cell change during a track, or the end of a
   track at its target cell? Does the intermediate cell of a two-cell curve count?
2. Does the test run in the same tick and at the same point of the driving update as that moment,
   before or after the track's other end-of-track effects (S-0207 claim 12, S-0210 claim 9)?
3. On a refusal, which scatter request does the harvester make (S-0212's vehicle scatter, forced or
   not, with or without a threat point), how many draws does it take, and does it keep its contact,
   its tether and its *enter* behaviour?
4. "As in row 4": does the harvester also need to face exactly 16384 and be stopped on this route,
   or only the refinery's acceptance and not being sold?

## Possible observation

Scenario H10 with a recorder: a harvester tethered to its refinery that approaches the cell below
the refinery's docking column from the south, recording the tick and draw count at which *unload*
is queued, and a second run with the refinery switched off after the tether forms, recording the
harvester's next destination and draws.

## Current workaround

The second route is not implemented: a harvester reaches *unload* only through row 4 at a
processing of *enter* (`econ::harvest::process_enter`). The S-0402 ledger lists claim 10 as
partly implemented for this reason.
