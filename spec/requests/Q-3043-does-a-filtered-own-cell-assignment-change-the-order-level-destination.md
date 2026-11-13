---
id: Q-3043
title: Does a jumpjet infantry unit's filtered own-cell assignment change its order-level destination?
status: open
related: [S-0502, S-0213, S-0207, S-0107]
answered-by: []
---

## Need

S-0502 claim 23 and S-0213 claim 21: outside an ion storm a jumpjet-type infantry unit "ignores a
nonempty destination assignment whose cell equals its current cell", and the filter can "suppress
a destination assignment without cancelling the order's other effects". It is not stated whether
the suppressed assignment still writes the unit's order-level destination (the destination
S-0207 claim 27 and S-0107 claim 15 compare against), so that only the locomotor ignores it, or
whether the order-level destination keeps the value it had before.

Our order code writes the order-level destination first and the jumpjet locomotor then filters the
assignment, so the order-level destination holds the filtered one while the locomotor keeps its
preceding order and movement point. GitHub issue #190 found that a later movement update took the
filtered destination after all once the unit had flown out of the cell. We now remember the
filtered destination in the locomotor so that it stays ignored; the order-level destination is
unchanged by that fix. The two readings differ in what later rules see: a repeat of the same
destination (S-0107 claim 15's get-up equality, S-0207 claim 27's effective-assignment test, the
infantry retry and blockage delays of S-0212 claim 21 and S-0209 claim 10), and what a save or an
AI reads as the unit's destination.

## Questions

1. After a filtered own-cell assignment, is the unit's order-level destination the filtered
   destination, or the destination it had before the assignment?
2. If it is the filtered one: when the unit later is over another cell (it was cruising or
   ascending when the order came), does any rule hand that destination to the locomotor again, or
   does the locomotor keep its preceding order until the next assignment?
3. If it is the preceding one: do the other effects of the assignment that come before the
   locomotor in the same assignment (S-0212 claim 21's retry-delay end, S-0209 claim 10's blockage
   delay restart) still happen?

## Possible observation

A jumpjet infantry unit cruising from A to B over cell C is ordered to C. Then, after it has
flown out of C, give it a second order to C (now no longer its cell). Under the first reading of
question 1 the second order repeats the order-level destination; under the second it is a new
one. A difference shows in a prone get-up test or in the draws of S-0212 claim 21's retry.

## Current workaround

The order-level destination holds the filtered destination (as before). The locomotor records the
filtered destination, and its movement updates do not take it again; the record is cleared at the
next assignment the filter lets through. The code (`jump::take_destination`, `jump::tick`) and
the test `a_filtered_own_cell_assignment_stays_ignored_after_the_unit_leaves_the_cell` cite this
request.
