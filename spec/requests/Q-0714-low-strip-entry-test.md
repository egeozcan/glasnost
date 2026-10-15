---
id: Q-0714
title: Which entry test decides that an occupant "could not enter" a changed low-bridge cell
status: answered
related: [S-0511, S-0209, S-0211]
answered-by: [S-0511]
---

## Need

S-0511 claim 15's kill check hits a ground-layer object "when it could not enter the cell as it now
is (the entry test of S-0209 for its own kind)", and a heading unit "when ... it could not enter
that cell". S-0209 claim 4's entry test gives several outcomes, some about other occupants (a
friendly vehicle, a full cell) rather than about the terrain. The object being tested is itself
in the cell, so read literally the test may find the object's own marks or its neighbours there.
The code in `crates/deterministic/sim/src/bridges/low.rs` (`kill_check`) needs the rule.

## Questions

1. Which outcomes of the entry test count as "could not enter" for this check: only the outcome
   for terrain the object's movement cannot use, or also the outcomes for occupants?
2. Does the object's own presence in the cell count against it?
3. For a ship under a low bridge that is repaired (claim 20), is it the cell's new land type alone
   that kills it?

## Possible observation

S-0511 scenario B7 with two friendly vehicles in one strip cell (if they can share it) and a
tank on an intact strip next to a damaged one; and the B12 low variant with a ship under the
bridge.

## Current workaround

"Could not enter" means only that the cell's terrain, as it now is, is not passable for the
object's movement zone on the ground layer (land and passability category, S-0211); occupants,
the object itself included, are not counted. The choice is marked in the code with this
request's ID.
