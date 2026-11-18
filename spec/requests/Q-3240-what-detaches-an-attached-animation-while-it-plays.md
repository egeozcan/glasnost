---
id: Q-3240
title: What, other than its owner's removal, detaches an attached animation while it plays
status: open
related: [S-0109, S-0611, S-0614, S-0522]
answered-by: [S-0611, S-0614]
---

## Need
S-0109 claim 22 says an attached animation follows its owner, and that "when it is detached it
stays where it is". It names the owner's removal as the event that ends the attachment by deleting
the animation, but it names no event that detaches an animation and leaves it in play. Its backing
rates detachment `unmeasured`: "AN15 provides no action that detaches without removing the owner".
S-0614 claim 25 says the same for the grappling animation ("no skirmish event is known to detach it
during a hold").

S-0611 claim 15 gives rules for an infantry-making animation once it is detached: its point is the
position it had when it was detached, and its first update after the detachment frees the mark of
its last attached update when the two standing positions differ. In our engine an animation is
attached while it has an owner, and only the owner's removal ends that, by deleting the animation.
No path detaches an attached animation while it plays, so the engine keeps no detached state and
claim 15's rules after a detachment cannot be reached or tested (issue #291, a seam left by sim
batch S20). Without a claim that names the event, the engine cannot add one.

## Questions
1. Apart from its owner's removal from the game, is there any event in a skirmish game that detaches
   an attached animation from its owner while the animation stays in play? Examples to check: the
   owner entering a transport or a building, leaving the map through a warp or a teleport, being
   turned into another object (deploy, conversion, mind control), cloaking, sinking, or the
   animation's own follow-up switch to another type.
2. If there is such an event, in which update does the detachment take effect, and is the position
   kept the owner's position plus the displacement at that moment?
3. If there is no such event, may S-0109 claim 22 and S-0611 claim 15 say so, so that the engine
   can treat detachment as unreachable?

## Possible observation
Give an infantry-making test animation type a long loop and attach it to a unit (for example as
the parachute animation of a paradrop, S-0522 claim 13, with `quirks.paratrooper_any_animation_slow_fall`
at its default). While it plays, put the unit through each event in question 1 and record, tick by
tick, whether the animation keeps following the unit, stays where it was, or disappears.

## Current workaround
None needed: no path detaches an attached animation, so S-0611 claim 15's rules after a detachment
are not built. Sim batch S32 leaves this part of #291 open.
