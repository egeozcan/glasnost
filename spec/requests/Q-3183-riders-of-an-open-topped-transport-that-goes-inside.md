---
id: Q-3183
title: Whether the riders of an open-topped transport leave play when the transport itself goes inside
status: open
related: [S-0607, S-0507, S-0203]
answered-by: [S-0607]
---

## Need
S-0507 claim 14 puts the passengers of an open-topped transport in play while they ride, and claim
15 takes their targets away when the transport leaves the map, for example when it boards another
transport (claim 6) or enters a building. S-0203 claim 3 and S-0607 claims 12 and 21 make going
inside leaving play for the object that goes in. No claim says whether an open-topped transport's
own riders leave play when the transport itself goes inside, and whether they are still in play
(take turns) while it is inside. The magnetic lift needs this: a lifter that rides an open-topped
transport and grips a unit keeps its grip while the transport moves (S-0607 claim 12, *Lifters and
transports*). When that transport boards another transport or enters a building or bunker, does
the rider's grip end by the leaving-play row, its waves end (claim 21), and its earlier dropped units
lose it as dropper (claim 14)? Our engine's `objects::take_inside` acts only on the object that goes
in (issue #285; S22 ledger seam).

## Questions
1. An open-topped transport carrying a lifter that grips a unit boards a closed transport. Does the
   grip end at the boarding, with the lifter's target cleared, or does the grip last?
2. In the same case, does a live magnetic or sonic wave of the rider end as at the rider's own
   leaving play (S-0607 claim 21), or only through the loss of its target (S-0507 claim 15)?
3. While the open-topped transport is inside the other transport, do its riders still take turns
   (fire, move a gripped unit, advance their own timers), as claim 14 gives for riders of a
   transport on the map?
4. Does the answer differ when the open-topped transport enters a building or a vehicle bunker
   instead of boarding a transport?

## Possible observation
An open-topped vehicle transport that can itself board a larger transport carries a magnetic-lift
unit. The lifter seizes a tank; the transport is then ordered into the larger transport before the
tank arrives. Record whether the tank keeps flying toward the transport, falls or lands, and whether
the lifter's wave stays drawn.

## Current workaround
The riders are untouched by the transport's going inside apart from S-0507 claim 15's loss of
targets (`transport::leaving_map`): a rider's grip lasts (a change of target does not end a grip,
S-0607 claim 12), its waves end only through that loss of target at their next update (claim 21's
target test), and its dropped units keep it as dropper. The riders stay in the update order while
the transport is inside. This is the S22 behaviour, unchanged.
