---
id: Q-2716
title: Does a removed object keep its tag while it is a leftover
status: answered
related: [S-0203, S-0011, S-0312]
answered-by: [S-0312, S-0011]
---

## Need

S-0203 r53 claim 13 (g) gives a leftover vehicle with health left a killing hit at the defeat
sweep, and says that "the raises that S-0312 claim 29 gives for a killing hit with no attacker and
no attacking house (point (c)) come again for a tag that it still carries". Its backing adds that
how often a trigger fires at them "depends on the state that the object's first destruction left in
its tag's latches (S-0011)". That reads as if a leftover can still carry the tag it had in play.

No spec says when a removed object loses its tag. Our engine detaches an object's tag when the
object is removed from the game (the removal takes one carrier off the tag's count, S-0011 claim
2). So in our engine a leftover carries no tag, and the raises of claim 13 (g) never fire. If the
original keeps the tag on the removed object until it ends, our engine differs at the sweep's
killing hit, and also in the tag's carrier count during the rest of the tick (S-0011 claim 10's
all-carriers rule reads that count).

## Questions

1. When an infantry unit, vehicle, aircraft or building is removed (leaves play) in tick *t*, does
   it still carry its tag for the rest of tick *t*, as a leftover?
2. If it does: does it still count as one of the tag's carriers for S-0011 claim 10 (an
   all-carriers tag fires only when one carrier is left) for the rest of tick *t*, and is it
   detached when it ends?
3. For a leftover that the sweep's hit sets sinking again and that is kept to the end of the game
   (claim 13 (g)): does it keep its tag, and its place in the carrier count, for the rest of the
   game?

## Possible observation

S-0203 scenario O18 part (h), with W carrying a tag whose repeating trigger has
destroyed-by-anything as its only event and shows a text, and a control in which W carries no tag
of that definition but a second, far object of D's does. Count the firings at the sweep's hit on W
in the tick of D's defeat. For question 2, give the tag the all-carriers repeat type and attach it
to W and to one other object of K's that stays in play: a firing when the other carrier is later
destroyed, in the tick of W's removal, shows whether W still counted.

## Current workaround

Our engine keeps its rule: removal detaches the tag (`objects::remove`, `triggers::detach_object`),
so a leftover carries none and the killing hit of claim 13 (g) raises nothing. Cited `Q-2716` at
`leftover::sweep_hit`.
