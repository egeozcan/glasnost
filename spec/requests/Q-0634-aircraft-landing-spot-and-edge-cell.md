---
id: Q-0634
title: Which landing spot and which map-edge cell does an aircraft without a pad choose?
status: answered
related: [S-0501, S-0212]
answered-by: [S-0212, S-0501]
---

## Need

S-0501 Formula F8 compares the house's buildings for which "a nearby free spot can be found"
(S-0212 claim 25), but does not give that search's settings for an aircraft, and Open question 5
leaves the fallback with no building unresolved. Claim 34 sends *retreat* and *spy-plane
overfly* to "a cell on its house's starting map edge" with an untraced selection and draw
inventory (Open question 10). The gaps are in `glasnost_sim::air` (`landing_spot`,
`edge_cell`).

## Questions

1. Which movement settings (class, zone, deck, reference cell) does F8's nearby-spot search use,
   and what does it pick when the house has no building?
2. Which edge is a house's starting map edge in a skirmish, which cell on it is chosen, and how
   many draws does the choice take?

## Possible observation

S-0501 scenarios AC10 and AC20: exhaust a non-pad-bound aircraft's ammunition with no pads, with
and without house buildings; dispatch spy planes from different starting positions; record the
chosen cells with the random-state recorder.

## Current workaround

F8's search is S-0212 claim 25's nearby search with the aircraft type's own movement zone, on the
ground layer, with no draw; with no building the aircraft's own cell is used. The edge cell is the
nearest legal cell outside the usable area in the aircraft's own row or column, or else the first
cell beyond the usable area there, with no draw (marked Q-0634
in `glasnost_sim::air` and in docs/development.md, M6a interim rules).
