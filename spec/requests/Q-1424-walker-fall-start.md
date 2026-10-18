---
id: Q-1424
title: In which update a burning walker that leaves a deck starts to descend
status: answered
related: [S-0615, S-0511]
answered-by: [S-0615]
---

## Need
S-0615 claim 10 says a walker "starts to fall" after a running step or a placement on arrival when
it is 416 or more above the terrain with no deck at its cell or at neighbours 6 and 0, and that its
descent follows S-0511 claim 10. It does not say whether the first height change happens in the
same update as the step that started the fall, or at the next update, nor the starting fall rate.
Code: `crates/deterministic/sim/src/walker.rs`, `fall_check` and `descend`.

## Questions
1. In the update in which the walker's step or placement starts the fall, does its height already
   change by S-0511 claim 10, or does the first descent come in the next update?
2. What fall rate does the walker start with (zero, or another value)?

## Possible observation
S-0615 scenario FW4 with the walker's height recorded every tick around the moment it runs off a
deck's end.

## Current workaround
The first descent comes in the walker's next update, with a starting rate of 0; the height
changes by the rate, then the rate follows S-0511's recurrence.
