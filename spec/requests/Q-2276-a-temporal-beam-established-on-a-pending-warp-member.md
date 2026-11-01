---
id: Q-2276
title: Can a temporal beam be established on a superweapon warp member after the launch, and what happens to its warp then?
status: open
related: [S-0513, S-0514, S-0301, S-0306]
answered-by: []
---

## Need

S-0513 revision 62, claim 39, says that from the launch until the member's departure update a
firing check about the member answers *illegal* for every weapon, a temporal weapon included,
and that a firer whose temporal beam is attached to the member gets S-0514 claim 19's
*rearming*. It leaves open whether a beam can be attached to the member at all in that span:
by a shot made without a firing check (S-0301 claim 9's exceptions), or by a temporal shot made
before the launch and delivered at its detonation after it (S-0514 claim 1). Its Open questions
add that S-0514 claim 3's lock refusals name no pending warp.

Our engine has to do something when such a delivery happens. As built (intake 71), the
delivery's lock refusals are those of S-0514 claim 3, so a beam locks on a member from the
launch on, and nothing in the warp looks at it afterwards. The engine needs the rule for the
lock and for the warp of a member on which a beam is locked.

The case is not rare in our engine. In the r62 tests, a computer-owned vehicle with a temporal
weapon standing three cells from a tank fired at it on its own before the launch; its shot
detonated in the leaving stage, about ten ticks after the launch, and locked a beam on the tank.
The firer then got *rearming* and kept the tank at every check tick. The tests now hold such a
firer's fire until the launch, so that they test claim 39's answer without a beam.

S-0306 claim 17 (revision 62) says the erasure lock acts on a member after its departure, off
the map or phasing in. This request is about the span before departure, from the launch to the
member's departure update.

## Questions

1. In the original, does a temporal shot that was fired at an object before a warp launch picks
   it up, and that detonates after the launch and before the object's departure, lock a beam on
   it? Does the answer depend on whether the detonation comes before or after the member's first
   warp update in tick t_L + 1?
2. Can a shot made without a firing check lock a beam on a member in that span? Which of S-0301
   claim 9's exceptions can carry a temporal shot there?
3. If a beam is locked on a member in that span: does the member still depart in t_L + 62 and
   land as claim 39 gives, does its warp end and leave it in place, or does something else
   happen? Does the erasure count down while the member waits to depart, and does the beam let
   go at the departure?
4. If a beam is locked on a member in that span: does the beam's firer keep firing at it, given
   that S-0514 claim 19's *rearming* ranks above claim 39's *illegal*?

## Possible observation

On SW12 part k's set-up (S-0513), a human-owned infantry unit with a temporal weapon orders an
attack on the tank so that its shot is in flight at the launch and detonates in tick t_L + 1
or later. Record whether a beam appears on the tank, the tank's departure tick and landing,
the erasure's progress at each tick until departure, and whether the beam is still attached
after departure. A second run with the detonation timed to come before the tank's first warp
update in tick t_L + 1, if it can be arranged, would answer the second part of question 1.

## Current workaround

Interim Q-2276, as built: S-0514 claim 3's refusals only, so a delivery after the launch locks
a beam on the member (`temporal::deliver`). The warp goes on unchanged: the member's turn
remains the warp update alone until its arrival hold (S-0513 claim 51), so the erasure makes no
progress until then. The firer whose beam is attached gets S-0514 claim 19's *rearming* and
keeps the member at S-0309 claim 29's periodic check, as claim 39 gives. Test:
`claims_s0513_r62::claim39_a_firer_whose_beam_holds_the_member_gets_rearming_and_keeps_it`.
