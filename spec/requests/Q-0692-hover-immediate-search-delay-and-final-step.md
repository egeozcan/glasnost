---
id: Q-0692
title: Whether a hover's immediate step-check searches start the retry delay, and its step after a failed search
status: answered
related: [S-0503, S-0212, S-0207]
answered-by: [S-0503]
---

## Need

S-0212 claim 21 says that for the hovering locomotor every fresh search with no stored steps
left starts the retry delay *R* from its tick, found path or not. S-0503 claim 12 (table T1)
also makes a hovering vehicle discard its path and make a fresh search "at once, whatever its
path retry timer shows" at a first check with outcome *cloaked blocker*, *blocked*, *friendly
obstacle*, *enemy obstacle* or *stationary friendly*. Neither spec says whether these immediate
searches also start *R*. For the driving locomotor, S-0212 claim 21 lists the immediate searches
as exceptions to the waiting, and a found path clears the delay.

S-0503 claim 7 also says that when no new heading point is chosen at an arrival, a positive
step length already computed can still give a final step; "absence of a heading point alone
does not suppress it". S-0212 claim 23 says that after a failed search the unit "stops (its
current motion ends)". It is not said whether a hover whose fresh search fails at an arrival
still takes that final step.

The code in `crates/deterministic/sim/src/hover.rs` needs both.

## Questions

1. After a hovering vehicle's immediate fresh search of table T1 (first check), does its path
   retry timer show *R* from that tick, whether the search found a path or not?
2. On an arrival tick whose fresh search fails (S-0212 claim 23 rows 2 and 3), does the hover
   still move by the step length computed for that tick, or does it stay where it was?

## Possible observation

For question 1, use S-0503 scenario H9 (d): after the first *blocked* check and its successful
search, block the new path's first cell too, before *R* has run out, and record the tick of the
next search. For question 2, use S-0212 scenario P19's hovering run and record the hover's
position on the tick its search fails.

## Current workaround

Every fresh search a hovering vehicle makes, including the immediate ones of table T1, starts
*R* from its tick. A failed search at an arrival does not suppress the final step: the hover
moves by the step length already computed, then has no destination. Each choice is marked in
the code with this request's ID.
