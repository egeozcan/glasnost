---
id: Q-2579
title: When an entry restarted by the pad's answer in the docking stage is next processed
status: answered
related: [S-0501, S-0215]
answered-by: [S-0501]
---

## Need

S-0501 r120 claim 24 (*A refused touchdown in contact*, Q-2506) covers an aircraft in *enter*'s
docking stage whose refused touchdown forgot its destination. At its next processing, the second
rule asks the pad to be loaded, and a switched-on pad gives itself back as destination. With
*enter* current and nothing queued, that changes only the destination, and the processing goes on
to report entry in flight. "With a behaviour queued, entry starts again instead (the pad's answer
above)".

The pad's answer queues *enter* and starts it at once. That replaces the queued behaviour and gives
*enter* a new start. The claim says entry starts again instead of the report, but not what the
processing that met the answer returns. It also does not say when the restarted entry's first
processing comes. That first processing makes the 1-tick entry delay and sets the ready mark
(claim 31).

## Questions

1. In the processing where the pad's answer restarts entry, does anything else of the docking
   stage happen (a report, a docking step, a fall-back)? Or does the processing end there?
2. What delay does that processing return: 1 tick (the docking stage's own), 3 ticks (the
   approaching stage's), or the standard delay?
3. Does the restarted entry then begin in the approaching stage, so that a later processing that
   finds it landing again moves it on to the docking stage?

## Possible observation

AC31 part h (4)'s set-up, with a behaviour queued on *A* at the end of tick *u*. Use, for example, a
*guard* order given to *A* in the cycle of the refused touchdown. Record *A*'s behaviour slots and
the ticks of its processings from *u* to its next touchdown.

## Current workaround

`air::enter` ends that processing with no report and returns 1 tick. The restarted entry's first
processing comes then. It finds a new start, so it makes the entry delay of 1 tick, sets the ready
mark and puts the aircraft back in the approaching stage. Cited
`design: S-0501 claim 24 and Q-2579`.
