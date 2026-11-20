---
id: Q-3329
title: Whether the end-of-update assignment in move restarts a walker's blockage delay
status: open
related: [S-0208, S-0209, S-0212, S-0215]
answered-by: []
---

## Need

S-0208 claim 25 (r75, *After a failed search*) gives a walker in *move* that keeps its destination
after a failed fresh search (S-0212 claim 23, third row) that destination again at the end of the
same update (S-0215 claim 22). The claim lists what that assignment does: it clears the stored path,
ends the retry delay, gives a locomotor destination again and sets the speed fraction to 1. It does
not mention the blockage delay.

S-0209 claim 10 (r76, Q-2822) says that every assignment that is made starts a walker's blockage
delay afresh, the destination it already holds included. The same claim says that a failed search
after which the walker keeps its destination is not an assignment, so the delay counts on from the
first moving-blocker answer, and S-0209's seam C5 expects that for a walker that an order put in
*move*: the search that avoids the blocker comes no earlier than the blockage delay after the first
answer, and earlier than the blockage delay after the failure.

Read together, the two rules give opposite answers for a walker in *move*: the failure itself is
not an assignment, but the end-of-update check makes one in the same update. Our engine (batch S45)
makes the end-of-update assignment through the same hook as every other assignment, so it restarts
the blockage delay, and the C5 expectation fails for a walker in *move*.

## Questions

1. Does the end-of-update assignment of S-0208 claim 25 (a walker in *move*, not moving, holding
   the destination its failed search kept) start the walker's blockage delay afresh, as S-0209
   claim 10 gives for every assignment?
2. If it does, does S-0209 claim 10's statement and seam C5 (the delay counts on from the first
   moving-blocker answer after a failed search that keeps the destination) hold only for a walker
   in a behaviour other than *move*?
3. If it does not, which of the effects of an assignment does the end-of-update assignment have,
   and which does it not?

## Possible observation

S-0209 seam C5's set-up with the walker in *move* (a move order to the cell next to it, a moving
blocker in that cell, the cell then made impassable for one tick so the fresh search fails and the
walker keeps its destination, then made clear again). Record the tick of the search that avoids the
blocker against the tick of the first moving-blocker answer and of the failed search.

## Current workaround

The end-of-update assignment goes through the ordinary assignment hook and restarts the blockage
delay (S-0209 claim 10 read literally). The test
`seams7b_a_failed_search_that_keeps_the_destination_leaves_the_blockage_delay_running` in
`crates/deterministic/sim/tests/claims_s0209.rs`, whose walker is in *move*, is ignored with this
request's ID.
