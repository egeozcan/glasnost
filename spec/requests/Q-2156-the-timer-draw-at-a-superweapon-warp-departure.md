---
id: Q-2156
title: Does a holder take S-0309 claim 27's timer draw when its target leaves the map at a superweapon warp's departure?
status: answered
related: [S-0309, S-0513, S-0207, S-0215]
answered-by: [S-0513]
---

## Need
S-0309 revision 51, claim 27, says that when an object goes off the map, every holder of it as
its target drops it inside that action, and that a holder with more than 10 ticks left on its
targeting timer then restarts the timer with one ranged draw over 4 to 8 ticks. S-0513 claim 39
says that at a superweapon warp's departure the object leaves the map and every object targeting
it "drops it as in S-0207 claim 20 e". S-0207 claim 20 e (a jump's target loss, with S-0215
claim 10's revert and the restored-target distinction) takes no timer draw.

So at a warp departure the two specs disagree on the scenario random stream: claim 27 gives a draw
for each holder with more than 10 ticks left, S-0513 claim 39 by its reference gives none. Neither
names the other, and S-0309's backing says the off-map drop is `unmeasured`.

The code that decides is `teleport::targeters_drop`, which the warp departure
(`superweapons::warp`) calls; it does what S-0207 claim 20 e says and takes no draw. Every other
way off the map (entering a transport, a building or a grinder through `objects::take_inside`)
now goes through `combat::held::holders_drop`, which takes claim 27's draw.

## Questions
1. When an object leaves the map at a superweapon warp's departure, does each holder of it whose
   targeting timer has more than 10 ticks left restart that timer with one ranged draw over 4 to 8
   ticks, as S-0309 claim 27 gives for going off the map? Or does that drop take no draw, as
   S-0207 claim 20 e's drop at a jump?
2. If it takes the draw: are the draws, for several holders, in the same order as claim 27's
   drops, and do they come before or after the warp's other draws of that departure (its warp-out
   animation's creation draws, S-0513 claim 39)?
3. Does a jump of S-0207 claim 20 e, which does not take its object off the map, ever give the
   draw?

## Possible observation
S-0309 scenario A18 part d's timer read, with the held object warped away by a superweapon
instead of crushed: two holders, one with more than 10 ticks left on its targeting timer and one
with fewer, and the per-tick random state read across the departure tick.

## Current workaround
The warp departure keeps S-0513 claim 39's reference to S-0207 claim 20 e: no timer draw. Every
other off-map drop takes claim 27's draw.
