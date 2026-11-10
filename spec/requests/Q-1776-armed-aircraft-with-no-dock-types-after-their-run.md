---
id: Q-1776
title: What an armed aircraft with no dock types does after its run, landable or not (and a correction to Q-1756)
status: answered
related: [S-0501, Q-1756, Q-1775]
answered-by: [S-0501]
---

## Need

A follow-up triage of the lead's play note 5 (M8 step A4, docs/m8-claims/a4-play-notes.md), on the
installed data, found two things Q-1756 does not cover.

1. **Correction to Q-1756's premise.** Q-1756 says that after its last run an aircraft with no
   dock types keeps "the last firing spot" as its destination. On the installed data, for a
   strafing aircraft, S-0501 table T1's strafe passes make the target object itself the
   destination, so after the run the aircraft holds above the target's cell, not above a firing
   spot. In our engine such an aircraft (armed, landable, no dock types) ends in *guard* at the end
   of its run (T2), queues *move* to that destination (claim 33), keeps the holding mark (claim 6,
   with the held point as Q-1756's workaround keeps it) and stays above the target at cruise height
   for good. The lead saw the computer player's aircraft "circle over their target" (the lead's
   base), which matches this.
2. **Non-landable aircraft.** The installed data also has armed, non-landable aircraft types with
   no dock types (three of them). T2 gives them the same *guard* and claim 33 the same *move*, and
   claim 40 keeps a holding aircraft in *move*, so they too fly above their destination for good.
   No claim says whether they may leave.

Pad-bound aircraft are not part of this: S-0501 claim 18 and T2 already have an armed, pad-bound
aircraft in flight with no pad crash, and the triage saw that on the installed data. Q-1775 asks
about the other cases of the lead's crash report.

## Questions

1. An armed, landable aircraft with no dock types finishes a strafing attack (ammunition spent).
   Is its destination then the target object or a firing spot, and does it land, circle, leave the
   map or crash? (This is Q-1756 question 1 with the corrected destination; answering either
   answers both.)
2. An armed, non-landable aircraft with no dock types finishes its attack (ammunition spent, or
   the target gone). Does it circle over the destination, fly off the map or to the map edge, go
   back towards its owner's base, or crash? If it stops doing so, after how long?
3. Do the fly-by or fly-back type flags (S-0501's per-type keys), or the owner being human or
   computer-controlled, change either answer?

## Possible observation

A test map with one armed aircraft type of each kind (landable and not), ammunition 1, no dock
types, both fly-by and fly-back off, then both on. Order an attack on a durable enemy vehicle 16
cells away and record behaviour, destination, position, height and whether the aircraft exists
every tick until 1500 ticks after its last round, 5 runs per case, with a human owner and a
computer owner.

## Current workaround

The approved claims as written, with Q-1756's workaround: the aircraft holds above its destination
at cruise height and neither lands nor leaves. No change until this is answered.
