---
id: Q-0681
title: What a depot in its settling phase does while its docked vehicle is moving 200 leptons or more away (S-0314 claims 6 and 21)
status: answered
related: [S-0314, S-0504]
answered-by: [S-0314]
---

## Need
S-0314 claim 21 (*Settling*): "while the vehicle is within 200 leptons of its docking point and
still moving, the depot holds settling and checks again on the next tick ... Beyond 200 leptons
there is no distance test." Claim 6 (*Settling*) asks for the step "provided the vehicle still
stands still and its locomotor has finished settling", and gives the next processing *D* ticks
later only for *cannot pay*. Neither claim says what a settling processing does, and which delay
it chooses, when the docked vehicle is moving and is 200 leptons or more from its docking point
(for example a docked vehicle given a move order after the waiting phase ended). The engine
needs it in `depot::process_repair`.

## Questions
1. In the settling phase, with the docked vehicle moving at 200 leptons or more from its docking
   point, does the depot take a repair step, do nothing, or something else?
2. Which delay does that processing choose: 1 tick, the processing delay *D*, or another?

## Possible observation
H18 b: dock a vehicle, give it a move order away from the depot in the 3 ticks between the
waiting check and the settling processing, and record the depot's processings, the vehicle's
health and the owner's credits each tick while it drives.

## Current workaround
Interim rule, recorded in docs/development.md: no step, and the next processing *D* ticks later
(the delay claim 6 gives settling for a step that is not taken).
