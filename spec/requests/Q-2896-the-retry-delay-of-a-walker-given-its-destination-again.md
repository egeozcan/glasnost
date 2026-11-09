---
id: Q-2896
title: The retry delay of a walker given the destination it already holds
status: open
related: [S-0209, S-0212, S-0208, S-0417]
answered-by: []
---

## Need

S-0209 r76 claim 10, *A destination given again (Q-2822)*, ends with: "The same assignment also
ends a running retry delay of the walker (Cross-spec coordination, for S-0212 claim 21). That end
is `unmeasured` here." Its Cross-spec coordination item for S-0212 claim 21 says that for the walk
locomotor a destination given again, the same cell or the same object, ends a running retry delay
as a different one does, at every assignment that is made.

S-0212 r97, which owns the retry delay, still says in its Open questions: "For every locomotor,
whether giving a unit the destination it already holds ends the delay is not established; this
spec keeps the delay running." Its claim 21 ends the delay only when the destination "is changed
to a different one or cleared". S-0208 r58 claim 6 leaves the scatter whose cell is already the
walker's destination to that same open case of S-0212.

So two approved specs give different results for one case: a walker whose retry delay runs (after
a re-path for a moving blocker, or after a failed search that kept its destination, S-0212 claim 23
row 3) and that is then given the destination it already holds, by an order or a scatter. S-0209
says the delay ends; S-0212 keeps it. The retry delay decides when the walker searches again, so
the two give different first-search ticks.

The blockage delay part of claim 10 (S-0209's own delay) is built from r76 as written. Only the
retry delay part waits on this request.

## Questions

1. For the walk locomotor, does an assignment of the destination the walker already holds (the
   same cell or the same object), by an order, a scatter or any other rule, end a running retry
   delay, as S-0209 r76 claim 10 says, or keep it running, as S-0212 r97's Open questions keep it?
2. If it ends it: does it end a delay started earlier in the same tick too, as S-0212 claim 21
   says for a different destination?
3. Does S-0212 claim 21's case for the walk locomotor now follow S-0209 claim 10, with the Open
   questions entry limited to the other locomotors, as S-0209's Cross-spec coordination proposes?

## Possible observation

S-0212's walker set-up of claim 23 row 3: a walker given a destination within 1 cell whose search
fails, so that it keeps the destination and waits for *R*. One tick later, order it to the same
cell. Record the tick of its next search (its first step or turn towards the cell). A search at
once shows the delay ended; a search *R* ticks after the failure shows it kept.

## Current workaround

`// design: Q-2896` in `path::destination_changed` (`crates/deterministic/sim/src/path.rs`): the
retry delay follows S-0212 r97. A destination given again keeps a running retry delay for every
locomotor, the walk locomotor included; a different destination or none ends it for the driving
and walk locomotors. The blockage delay restarts at every assignment (S-0209 r76 claim 10).
Tests: `claim_21_the_same_destination_again_keeps_the_delay`
(`crates/deterministic/sim/tests/claims_s0212_r97.rs`) and
`open_question_16_the_kept_destination_given_again`
(`crates/deterministic/sim/tests/claims_s0417_r79.rs`), both citing the interim, and
`q2896_interim_the_same_destination_again_keeps_the_retry_delay`
(`crates/deterministic/sim/tests/claims_s0209.rs`).
