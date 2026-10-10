---
id: Q-0017
title: What S-0212 leaves open for an implementation of the path search
status: answered
related: [S-0212, S-0208, S-0211]
answered-by: [S-0208]
---

## Need
`glasnost_sim::path` implements S-0212 behind a swappable interface. Five
parts of the search are stated only by their effect or not at all, and
each changes routes the player sees. Code gap: the corridor (coarse search)
and the parts listed below.

## Questions
1. **Sectors and connections (claim 13).** Which pieces of one region inside
   one aligned block count as one sector (every 8-connected piece, or the
   whole region's part in the block)? How are connections formed across
   block borders (any two neighbouring cells, any height rule?) and through
   links? Is a sector's category its region's?
2. **Order of equal-cost entries.** In the coarse search, which of several
   pending routes with exactly the same cost is continued first? In the cell
   search, which of several pending pairs with the same estimate is expanded
   first, beyond the held-candidate rule that the spec gives (Questions for
   the project lead, "Tie order")? Our engine needs one order; please state
   the original's, or confirm that an order of our own (cost, then the order
   in which entries were recorded) is acceptable.
3. **Anchor cell (F8).** Which cell of a sector are the legs of the estimate
   *E* (claim 24) measured from?
4. **Claim 18 stretch selection.** Which stretches of the first 20 steps are
   examined, in which order, and where does examination resume after a
   replacement?
5. **Claim 25, the further test** of a cell for a subterranean unit, and
   **the visibility test**: is the visibility test exactly the screen-cover
   check of S-0208 Terms applied to the candidate at its own height level
   (S-0208's proposal), including its high-bridge raise?

## Possible observation
Scenario P1 and P12 routes on maps with obstacles that bend a path through
several sectors; a recorder that logs the stored path after each search.

## Current workaround
No corridor: every search runs over the whole map (claim 14 is treated as
never using the corridor), so the retry of claim 13 and the 5-search limit
never arise. Equal costs are broken by the order of recording, after the
held-candidate rule. The estimate *E* of claim 24 is not computed: a
substitute found for a *stationary friendly* destination is adopted when the
distance test passes. Claim 18 is not applied (paths keep their shape after
claim 17). The visibility test is S-0208's screen-cover check. The further
test is treated as passed. Each of these is marked provisional in
`docs/development.md`.
