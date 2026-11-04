---
id: Q-2567
title: Which of a building's own answers (S-0405 table T1) S-0301 claim 9 ranks above the held-temporal-target rearming
status: open
related: [S-0301, S-0513, S-0405, S-0514]
answered-by: []
---

## Need

S-0513 r115 claim 39a (*The firer*) says a building firer whose beam holds a departed member
answers *rearming* "unless one of the building's own answers that S-0301 claim 9 ranks above it
holds", and: "With an outranking *illegal* or *impossible*, the building drops the member, as
before the departure, at the first of an *attack* processing, where the row for those answers
drops it and starts *guard*, and its checking turn in a check tick of S-0309 claim 29".

S-0301 r71 claim 9 orders a building's pending delayed shot against table T1 ("its *rearming*
yields only to the building's own answers of S-0405 table T1") and places T1's answers before
the rows of S-0306 claim 23. Its paragraph *A held temporal target within rank 4* orders the
held-temporal-target *rearming* only against the hidden-target *impossible* and the protected
target's *illegal*. We find no sentence that orders it against T1's answers: an occupiable
building without firing occupants, a drained building, a building in *construction* or
*selling*, a pulse cannon, and a building not in operation.

Our check asks T1's answers before the held-temporal-target *rearming*, so every T1 answer
outranks it. That order is not cited to a claim, so intake 84's row ("a building's outranking
*illegal* or *impossible* drops the member and starts *guard*") stays untested (M8 seam sweep
5b).

## Questions

1. Which of S-0405 table T1's answers does S-0301 claim 9 rank above the held-temporal-target
   *rearming* (S-0514 claim 19), and which below it?
2. In particular, does a building that is not in operation (switched off, short of power or
   under a pulse) keep a target its temporal beam holds, on the map and for a departed warp
   member?

## Possible observation

SW12 part l7's set-up with a building firer, switched off by the player after the lock and
before the departure (and a second run with it put short of power): record its target and
behaviour at each processing and at each check tick of S-0309 claim 29 until the appearance.

## Current workaround

T1's answers are asked first, as before. No test asserts the drop for a building firer
(`docs/m8-claims/seams-5b.md`).
