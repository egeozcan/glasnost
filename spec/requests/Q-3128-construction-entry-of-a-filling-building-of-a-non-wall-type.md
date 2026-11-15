---
id: Q-3128
title: Construction entry of a filling building of a type that is not a wall type
status: answered
related: [S-0406, S-0415]
answered-by: [S-0406]
---

## Need
S-0406 claim 11 lets the wall fill of a place order create filling buildings of the placed type.
For a type that is not a wall type (no building wall flag, but a wall overlay that has the overlay
wall flag), each filling building stays a building. Claim 11 says that "whether it shows a
construction state is not established" (Open questions 1, scenario PL8 r).

Claim 17 names two entries into the construction state: a deploy sets the ready mark when the
building enters play, and a place order (and a restored replacement) leaves the mark clear at
entry. With `quirks.placed_buildup_twice` on (the default) the two entries give different build-up
timings: a placed building runs its build-up twice, a deployed one once. A filling building is
neither a deploy nor the building the place order names, so the engine has no rule for which entry
it takes, or whether it takes either (#310).

## Questions
1. Does a filling building of a type that is not a wall type enter play in the construction state
   at all, or does it enter in its idle state with no build-up?
2. If it enters the construction state, is the ready mark set at entry (as for a deploy) or left
   clear (as for the place order's own building)? Does `quirks.placed_buildup_twice` apply to it?
3. Is the answer the same for a filling building created in the same tick as the ordered building
   and for one created by a later fill?

## Possible observation
A type that is not a wall type with a wall overlay that has the overlay wall flag (a rules mod),
with a build-up of at least two stages and a positive delay. Place one so that the fill creates a
second building next to it, and record from the first tick whether the filling building shows its
build-up, and whether it shows it once or twice, beside the ordered building.

## Current workaround
A filling building enters as any insertion other than a place order does: in the construction state
with the ready mark set at entry, so it runs its build-up once. The ordered building is unchanged.
The comment at `econ::placement::put_on_map_facing` and the ledger row of S-0406 claim 11 state this
interim.
