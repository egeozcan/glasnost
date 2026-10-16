---
id: Q-0985
title: The acquisition order of equally distant supporters, the delay after a charger starts charging, and the search kind of a charging unit's break-off
status: answered
related: [S-0518, S-0404, S-0215, S-0309]
answered-by: [S-0518]
---

## Need

1. S-0518 claim 3 breaks a tie between two equally distant eligible supporters by "the order in
   which the house acquired its buildings, S-0404 claim 7". Our engine keeps no per-house list of
   acquired buildings: a building a house captured or was given has no acquisition rank apart
   from its object number.
2. S-0518 claim 14 says a charger that takes a building in the processing of *guard* or *area
   guard* gets *attack*, and that in *guard* the ordinary processing "does not run in that
   processing". It gives no delay for that processing (the behaviour delay the processing
   returns).
3. S-0518 claim 15's break-off search is "an automatic search of the range kind centred on its
   position (S-0309 claim 1)" taking the target "instead". Which of S-0309's permission gates
   (claims 2 and 3) apply to it is open (the spec's own Open questions), and whether a charger's
   current building target counts as a usable target that the search keeps.

## Questions

1. Is the tie order the order in which the house's buildings were created or acquired (placement,
   capture, ownership change), and is a building captured later ranked by the capture tick?
2. What delay does a *guard* or *area guard* processing return when the charger found a building
   and took it (one tick, the standard delay, the rate delay)?
3. Does the break-off search replace a current target that is a building of the unit's own house
   without any further test, or only when the search finds a different object than that building?

## Possible observation

TC2 with the processing ticks of the charger recorded (the tick of the processing that took the
building and the next processing); TC3 with two enemies in range at once.

## Current workaround

- `linked::recruit`: the tie goes to the lower object number.
- `process::process_guard` and `process_area_guard`: a charger that took a building returns 1.
- `linked::break_off`: the unit's target is cleared before the search, so the search can find the
  building itself or anything else; a found target (any) replaces the building and clears the
  mark; none found restores the building and the mark.
