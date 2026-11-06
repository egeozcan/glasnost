---
id: Q-2701
title: Which walkers under enter keep their stored path when given a new destination
status: answered
related: [S-0208, S-0212, S-0213]
answered-by: [S-0208]
---

## Need

S-0208 r46 claim 6, *A new destination while a step is under way*: a walker that has a walking
target keeps it when it is given a new destination, finishes the step and drops its stored path,
but "a walker whose current or queued behaviour is *enter* keeps that path instead", and the Open
questions say "Our engine keeps the path in that case too". S-0208's Changes entry for S-0212
claim 20 gives the same exception without the words "while a step is under way".

Two readings are open, and they differ a great deal in play:

- An ordinary behaviour order applies its parts in S-0213 table T3's order: the order's behaviour
  is queued (part d) before the destination is assigned (part f). If *queued* is read at the
  assignment, every enter order (into a transport, a garrison, a building to capture) given to a
  walker in the middle of a step keeps the stored path of its old destination. The walker then
  walks that whole old path before it first searches for the building or transport.
- If *queued* is read before the order, only a walker that was already under *enter* (walking to
  what it enters) and is given another destination by an order or another rule keeps its path.

The code that would change is every site that assigns a walker a new destination and drops its
stored path: the behaviour order in `crates/deterministic/sim/src/orders.rs` (`ob.path.steps.clear()`
after T3 (f)), the scatter answer in `scatter.rs`, and the other sites that clear `path.steps`
with a destination change.

## Questions

1. A walker in the middle of a step, under *move* to a far cell, is given an enter order on a
   transport or a building next to it. Does it keep walking the stored path to the far cell
   after it finishes the step, or does it search for the transport or building once that step
   ends?
2. A walker in the middle of a step under *enter*, walking to a transport, is given an ordinary
   move order to another cell (or accepts a scatter request). After it finishes the step, does it
   follow its old path towards the transport, or search for the new cell?
3. Does the exception hold only while a step is under way (the walker has a walking target), or
   also for a walker standing between steps with a stored path, as the Changes entry for S-0212
   claim 20 reads?

## Possible observation

S-0208 scenario W17 parts c and d, with the second order an enter order on an empty friendly
transport standing next to the walker's line (question 1), and with the walker first ordered into
that transport and then ordered to a cell to the side (question 2). Record its position, current
and queued behaviour and destination per tick until it stops.

## Current workaround

Not built: every walker given a new destination drops its stored path, *enter* or not, as before
S-0208 r46. A walker with a walking target keeps it and finishes the step (claim 6), so only the
path kept under *enter* is missing.
