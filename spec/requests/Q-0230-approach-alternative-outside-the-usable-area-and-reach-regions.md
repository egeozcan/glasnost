---
id: Q-0230
title: Can an approach alternative outside the usable area be in the actor's reach region
status: answered
related: [S-0307, S-0211, S-0212]
answered-by: [S-0307]
---

## Need

S-0307 claim 21 (with `quirks.approach_alternative_outside_area`) says a vehicle's
approach alternative outside the usable area can be chosen when its range, clear-cell
and detour tests pass, and scenario RG20 places such an alternative "on legal terrain in
the same reach region" as the actor. The clear-cell test of claims 15 and 19 requires the
candidate to lie in the actor's current reach region.

S-0211 claim 11 (rule 1) gives every cell outside the usable area the category *off-map*,
and S-0212 claims 2 to 4 say off-map cells form no region and count as lying in a
different reach region from every cell of a reach region. Read together, an alternative
outside the usable area never passes the clear-cell test, so claim 21 and the switch have
no effect in our engine. The code is the approach-position search in
`crates/deterministic/sim/src/combat/range.rs` (`approach_search`).

## Questions

1. When a vehicle standing inside the usable area takes the approach-position search and
   the first alternative of a blocked primary is a legal cell just outside the usable area,
   on open flat ground next to the actor's own ground, does that cell pass the clear-cell
   test's reach-region condition?
2. If it does, what passability category and reach region does the clear-cell test read
   for that cell? Is it the category the cell's land and overlay would give inside the
   usable area, or does the clear-cell test skip the region condition for such cells?
3. Does the answer change for the nearby-cell search of S-0212 claim 25, or for path
   searches whose destination is such a cell?

## Possible observation

RG20 of S-0307 as written: record whether the outside alternative is chosen, and, as a
control, give the vehicle a move order to that same cell and record whether the path
search reaches it.

## Current workaround

The usable-area test is skipped for alternatives under the switch, as claim 21 says, but
the clear-cell test keeps S-0211 and S-0212's off-map rule, so an alternative outside the
usable area is always rejected (the result the switch's off state specifies). Interim
row in `docs/development.md`.
