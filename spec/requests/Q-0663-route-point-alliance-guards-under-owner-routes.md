---
id: Q-0663
title: Whose alliance the repair and reload route-point guards test, given our owner-routes departure (S-0215 claim 13)
status: answered
related: [S-0215, S-0214]
answered-by: [S-0215]
---

## Need
S-0215 claim 13 (*Repair/reload route points*) tests alliance with "the house played on the
machine": that house must count the point object's owner as allied and the owner must count that
house as allied, and an object the local house does not count as allied makes *select*, *toggle
select*, *move* and *no-move* fall back to the cell action. The same claim and its Edge cases
record a departure: our engine takes the next route point from the actor owner's routes on every
machine, not the local player's, so the result does not depend on the machine. The alliance
guards still name the local player's house, which would make the order a route point issues
machine-dependent in lockstep.

## Questions
1. Under the owner-routes departure, should the two alliance guards and the non-allied fallback
   test the actor owner's house in place of the house played on the machine?
2. If the original's local-house test is to be kept, which house stands in for "the house played
   on the machine" in a simulation that must give the same result on every machine (for example
   in a replay or a game with no local player)?
3. Do the two readings differ anywhere in a skirmish other than for an actor whose owner changed
   during a route?

## Current workaround
Routes are not built (the route row of `process` is a seam), so no route point is ever
classified. No code depends on the answer yet.
