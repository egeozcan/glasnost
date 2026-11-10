---
id: Q-2822
title: The same destination given again to a walker waiting for a moving blocker
status: answered
related: [S-0209, S-0208, S-0212]
answered-by: [S-0209]
---

## Need

S-0209 r60 claim 10: "The blockage delay starts afresh only after the walker has taken a walking
step again, or when it is given a new destination, by an order, a scatter or any other rule (S-0208
claim 6)." S-0208 r58 claim 6 says, of a scatter whose cell differs from the walker's destination,
that it ends any running retry delay, and then: "The scatter restarts the blockage delay (S-0209
claim 10)." For a scatter whose cell is already the walker's destination it says only that "the
scatter gives that destination again", and leaves what that does to a running retry delay to
S-0212 claim 21's open case of a destination given again.

Neither spec says whether a destination given again, the same cell or object the walker already
has as its destination, restarts the blockage delay. "A new destination" in S-0209 claim 10 can be
read as a destination given anew (any assignment, the same one included) or as a different
destination. Our engine restarts the delay in `path::destination_changed`
(`crates/deterministic/sim/src/path.rs`), which every order, scatter and other rule that assigns a
walker's destination calls, and in `walk.rs` where a wait starts for a destination other than the
one the running delay was started for.

## Questions

1. A walker waits for a moving blocker in its next cell, with the blockage delay running. It is
   given an order to move to the cell that is already its destination. Does the blockage delay
   start afresh, or does the wait keep counting from its first *moving blocker* answer?
2. The same walker accepts a scatter whose destination is the cell that is already its destination
   (S-0208 claim 6's case of a scatter that gives that destination again). The same question.
3. Does the answer differ when the destination is an object (an enter or attack order on the same
   target again) rather than a cell?

## Possible observation

S-0209 scenario C5 part c with the order of part c replaced by an order to the infantry's present
destination, given while it waits, with a test file that sets the blockage delay to *D* ticks:
record its waiting ticks before its path avoids *V*₁. A wait of at least *D* ticks counted from the
order (as in part c) shows a restart; a wait that ends *D* ticks after its first *moving blocker*
answer shows none.

## Current workaround

`// design: Q-2822` in `path::destination_changed`: the delay starts afresh only when the
destination after the assignment differs from the one before it, a cleared destination included;
the same destination given again leaves a running blockage delay as it was, as our engine keeps a
running retry delay for a destination given again (S-0212 claim 21 ends that delay only for a
destination changed to a different one or cleared, and leaves the case of one given again open).
Test:
`q2822_interim_the_same_destination_again_keeps_the_blockage_delay` in
`crates/deterministic/sim/tests/claims_s0209.rs`.
