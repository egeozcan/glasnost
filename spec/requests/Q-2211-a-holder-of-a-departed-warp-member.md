---
id: Q-2211
title: What does a holder of a superweapon warp member do with it after the member's departure?
status: answered
related: [S-0513, S-0309, S-0215, S-0301]
answered-by: [S-0513]
---

## Need
S-0513 revision 43, claim 39, says that a warp member's departure clears no holder's target and
takes no holder timer draw, and that the member is off the map from departure until it appears at
its landing point. It leaves open whether a holder's own processing drops the member after
departure, for example its attack behaviour or its target checks, and whether a holder may fire at
the member while it is phasing in after it appears (Open questions).

Our engine must do something with such a target in the holder's own turns. Its firing steps and
the start of each object's turn already have a rule for a held object that is off the map:
S-0309 claim 27's DECIDED rule, written for an object that a revert gave back. That rule treats
such a target as no target in the firing step and drops it at the start of the holder's next turn,
with no timer draw and no revert. The code is `combat::held::firing_target` and
`combat::held::turn_start_drop`. A departed warp member is the first object that a holder keeps
while it is off the map without a revert.

## Questions
1. After a warp member's departure, does a holder keep the member as its target while the member
   is off the map, from the departure until it appears at its landing point? Or does the holder
   drop it in its own processing, and if so, at which point: its next turn, its next search, or its
   attack behaviour's next processing? Does that drop take a targeting-timer draw?
2. Does the answer depend on the holder's kind (infantry, vehicle, building, aircraft) or on its
   behaviour (guard, attack, a behaviour with an override in effect)?
3. A holder that still holds the member when it appears at its landing point: does it go on
   attacking it there, moving towards the new position if it is out of range? May it fire at the
   member while the member is phasing in (from departure to the end of the arrival hold, claims 39
   and 40), and what does its firing check answer then?
4. A holder with an override in effect that keeps the member through the departure (claim 39 and
   S-0215 claim 10): if a later step drops the member, does that drop revert the override?

## Possible observation
S-0513 scenario SW12 part (i), extended past the departure: read each holder's target and timer
every tick from the departure to the end of the arrival hold, with the landing point outside and
inside the holders' range, and with one holder in *attack* and one with an override in effect.

## Current workaround
A holder's own turn drops a departed member with S-0309 claim 27's DECIDED turn-start drop, with no
timer draw and no revert, and its firing step treats the member as no target until then. A holder
whose turn in the departure tick comes before the member's update keeps the member to the end of
that tick and drops it at the start of its next turn. So by the member's arrival effects nobody
holds it, unless a holder picked it up again after it appeared.
