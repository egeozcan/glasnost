---
id: Q-2650
title: When an entry restarted by the pad's answer in the approaching stage makes its beginning
status: answered
related: [S-0501]
answered-by: [S-0501]
---

## Need

S-0501 r132 claim 24 (Q-2579) gives the docking stage's case: with a behaviour queued, the pad's
answer to the second rule's request starts entry again "within that processing. It makes entry's
beginning, with its 1-tick delay (below), puts the aircraft back in the approaching stage and
leaves it ready to change, and it neither reports entry, nor makes a docking step, nor falls back.
Its next processing, 1 tick later, is the approaching stage's."

The second rule applies "at every stage" (claim 24: an aircraft in flight with no destination asks
its contact to be loaded), so the pad's answer can also restart entry from a processing in the
approaching stage, for example after a refused touchdown that forgot the destination before the
aircraft reached the docking stage (claim 7, `quirks.aircraft_descent_touchdown_untested` off).
Claim 24 does not say whether that processing too makes entry's beginning at once, or goes on by
the approaching stage's rules (the aircraft now has the pad as destination).

## Questions

1. In the approaching stage, when the pad's answer to the second rule's request restarts entry (a
   behaviour was queued), does that processing make entry's beginning, with its 1-tick delay and
   the ready mark set, as the docking stage's does (Q-2579)?
2. If it does not, does it go on by the approaching stage's rules in contact (move on to the
   docking stage when it finds the aircraft landing, else return 3 ticks), with entry's beginning
   made at the next processing?

## Possible observation

AC31 part h (4)'s set-up with `quirks.aircraft_descent_touchdown_untested` off and the docking
position's cell held, with a behaviour queued on the aircraft and its landing refused before its
first processing that finds it landing. Record each processing's tick and delay, the ready mark
and the stage, until 20 ticks after the refusal.

## Current workaround

As built before r132: the processing goes on by the approaching stage's rules in contact, and the
restarted entry's beginning (1 tick) is made at its next processing. Cited
`design: S-0501 claim 24 and Q-2650`.
