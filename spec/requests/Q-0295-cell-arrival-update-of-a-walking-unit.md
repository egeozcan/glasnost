---
id: Q-0295
title: Which updates of a walking infantry unit are its cell-arrival updates for the clear report
status: open
related: [S-0417, S-0208, S-0409]
answered-by: []
---

## Need

S-0417 claim 10 says a tethered infantry unit released by an infantry factory "reports clear on
its next cell arrival", and claims 7 and 8 use "cell-arrival update" for the clear report and
the arrival fallback. Revision 3 of S-0417 defines the cell-arrival updates of a driving vehicle
(Q-0223: reaching a track target, and switching curves at a chain point), but not those of a
walking unit. The engine calls the clear report for a walker from `walk` (`econ::release::arrival`),
and the tick it uses is not specified.

## Questions

1. For a tethered walking infantry unit, which event is its cell-arrival update: the tick it
   crosses into a new cell, the tick it reaches a cell's standing position, the arrival tick of
   S-0208 claim 6 at the end of its walk, or another moment?
2. When an ordinary infantry unit walks from its appear cell in the foundation to its exit cell
   (S-0417 claim 10), is its clear report given in the first cell it enters outside the
   foundation, or only at the exit cell?
3. Does a unit walking across several cells in one move report at the first such update even
   when the walk goes on?

## Possible observation

S-0417 scenario U10 part a with a per-tick recorder of the unit's position, cell, contact and
tether: record the tick the contact with the factory ends, and compare it with the ticks the unit
crosses cell boundaries and reaches cell centres or standing positions.

## Current workaround

A walking unit's cell-arrival update is the arrival tick of S-0208 claim 6 (as it was under
Q-0223's interim rule, which S-0417 revision 3 answered for driving vehicles only). Interim row in
`docs/development.md` (Q-0295).
