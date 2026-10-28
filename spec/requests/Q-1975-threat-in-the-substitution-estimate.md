---
id: Q-1975
title: Does a unit's threat coefficient steer the coarse search of the substitution estimate?
status: open
related: [S-0212]
answered-by: []
---

## Need

S-0212 r82 claim 24 says the estimate *E* "comes from the coarse search of claim 13 alone",
run between the coarse end cells for the unit's own movement zone, and that *E* "is not a path
computation" (claim 13). Claim 13 and F9 add the threat term *T* (F5) to the entry cost of a
sector "when the unit's threat coefficient *k* exceeds 0.00001", read from the records of the
moving unit's owner (claim 16a). The spec does not say whether the estimate's coarse search
adds *T* too. With *T*, a unit whose coefficient is above that limit can get a longer level-0
corridor from the found cell, so a larger *E*, and the found cell can be refused where without
*T* it would be adopted.

In our engine `path::coarse_estimate` runs the coarse search with no unit, so with no threat
term, as before r82.

## Questions

1. When a unit with a threat coefficient above 0.00001 has its destination replaced (claim 24),
   does the coarse search that gives the estimate *E* add the threat term of F5, from the unit's
   owner's threat records and with the unit's coefficient, as the coarse search of a path
   computation does? Or does it run with no threat term, whatever the unit's coefficient?
2. If it adds the term: for the subterranean case, where *E* is found for movement zone
   *normal*, is it still the unit's own coefficient and owner?

## Possible observation

P30's threat setup with a P8-style substitution: a unit of a type with a coefficient of 1.0 and a
blocked destination cell whose nearby-cell search finds a cell that is reached in a straight
corridor without threat, and only round a threatened block with it, so that the corridor
length decides whether *E* stays within the Chebyshev distance plus 6. Compare with the same
order given to a unit of a type with coefficient 0.

## Current workaround

The estimate's coarse search adds no threat term (the behaviour before S-0212 r82), cited to
this request.
