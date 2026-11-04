---
id: Q-2438
title: The aircraft ready mark after the move on to the docking stage, when enter is left by force
status: open
related: [S-0501]
answered-by: []
---

## Need

S-0501 r103 claim 24: "The processing that moves an aircraft in contact on to the docking stage
leaves it not ready to change until its next processing, 3 ticks later (claim 31). ... Each
processing of the docking stage leaves it ready to change." Claim 48 (c)'s note describes the
mark as kept state: "the ready mark is set when an aircraft is made and cleared only by some
processings of *enter*, and of *move* for a type that carries vehicles ...; a *guard* processing
sets it again."

Our engine derives the mark from the aircraft's last processing (`air::ready_mark_cleared`:
*enter* current, its docking stage set and a last delay of 3 ticks, which only the move on to the
docking stage returns with that stage set). The two readings differ only when *enter* stops being
current within those 3 ticks without a start at a ready point, for example when a behaviour is
forced (a crash, a scatter answer that forces, an entered report of another path). With the
derived mark the aircraft is ready again at once; with a kept mark it stays not ready until a
processing sets the mark again.

## Questions

1. An aircraft whose processing of *enter* has just moved it on to the docking stage (in
   contact, 3 ticks), and whose behaviour is then forced to another behaviour before that next
   processing: is it ready to change (a queued behaviour starts at claim 31's next point) before
   the forced behaviour's first processing?
2. Which processings set the mark again: every processing of the docking stage and every
   *guard* processing only, or every processing that does not clear it?

## Possible observation

S-0501 AC31's harness with an aircraft docking at its pad in contact: in the tick after the
processing that moves it on to the docking stage, force *guard* (a test action) and queue *area
guard* without an immediate start; record per tick the current and queued behaviour until *area
guard* is current.

## Current workaround

The mark is derived from the last processing: an aircraft is not ready from the processing that
moves it on to the docking stage until its next processing of *enter*; a forced change of
behaviour in between makes it ready again at once. No state is added.
