---
id: Q-2317
title: Whether a firer whose temporal beam is attached keeps a departed warp member as its target
status: open
related: [S-0513, S-0514, S-0309]
answered-by: []
---

## Need

S-0513 claim 39 leaves open what a holder's own processing does with a warp member after its
departure, while the member is off the map (Open questions). Our engine's interim (Q-2211
question 3) drops such a held member at the start of the holder's next turn, as S-0309 claim 27
drops a held object that is off the map.

S-0513 r86 claim 39a now says, for a firer whose temporal beam is attached to the member, that
the firer keeps the member at S-0309 claim 29's checks, that the warp is no release cause, and
that a firer in *attack* whose member lands beyond its weapon's range moves towards it and lets
go on arriving in a new cell (*Release*, SW12 part l3). That approach needs the firer still to
hold the member as its target when the member appears. With the interim drop, the firer would
lose its target at the departure, and in our engine an *attack* firer with no target falls back
to rest, which lets the beam go (S-0514 claim 14) at the departure. We checked this on our
engine: without the workaround below, the firer of SW12 part l1's set-up no longer holds the
beam after the member's hold, and part l3's firer lets go without moving.

The engine's gap is in `combat::held::turn_start_drop`.

## Questions

1. While a warp member is off the map, between its departure and its appearance at the landing
   point, does a firer whose temporal beam is attached to it keep it as its target?
2. If it does, does anything else in that firer's turns change while the member is off the map
   (its behaviour, its facing, its rearm countdown)?

## Possible observation

SW12 part l3's set-up: read the firer's target in each tick from the departure to the member's
appearance, and whether its beam is still attached when the member appears.

## Current workaround

A holder whose beam is attached to a departed member keeps it (no turn-start drop); every
other holder of a departed member drops it as before (Q-2211 question 3). The firer makes no
shot at the member while it is off the map: the firing step counts an off-map target as none
(S-0309 claim 27).
