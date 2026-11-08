---
id: Q-2811
title: A controller target lost while a pending target waits
status: open
related: [S-0506]
answered-by: []
---

## Need

S-0506 r37 claim 11 lists the events at which the controller target is lost: the destroying hit,
going inside, removal, the start of cloaking and going underground. It then says: "The controller
target is cleared as it is lost, before the reset, so such a reset has no target when it releases
(claim 12). A pending target is lost at the same events, and is then cleared with no reset (claim
25)." The reset comes only "when the controller target is lost while no pending target is waiting".

The spec does not say what happens to the controller target when it is lost while a pending target
is waiting. If "cleared as it is lost" holds in that case too, a launching controller then has no
controller target. Claim 5 gives a launching controller without a controller target a reset, and
that reset would release every preparing missile and clear the pending target. That would undo
the "no reset" that claim 11 gives when a pending target waits. If the target is not cleared, the
controller goes on launching at the lost object until a launch adopts the pending target (claim 3).

The second reading has a consequence that the spec does not address either. If the pending target
is then lost too, it is cleared with no reset (claim 25), and the controller is left holding the
lost controller target with no further loss event to reset it. It launches at that object until
claim 8's release. A missile launched at an object that no longer exists has no heading point, so
no aim point is fixed at its launch.

The engine has to pick one. The code is `crate::spawn::target_lost` (the loss) and
`crate::spawn::controller_step` (claim 5's table).

## Questions

1. When the controller target is lost at one of claim 11's events while a pending target is
   waiting, is the controller target cleared at that moment?
2. If it is cleared, what does a launching controller do at its next pass:
   - reset (claim 5, launching without a target);
   - adopt the pending target at once; or
   - something else?
   Do the preparing missiles of a missile type stay preparing?
3. Does the answer depend on the event: a destroying hit on a target that stays in play (a death
   sequence, a sinking), against going inside or removal?

## Possible observation

S-0506 scenario R7 with a launcher of four rocket slots. While its first missile is preparing,
give the spawner a second target, so that a pending target waits (claim 3). Then destroy the first
target (an infantry unit with a death sequence) while the missile is still preparing. Record per
tick:
- the controller's state;
- each slot's state;
- each missile's release tick and release cell;
- which target the next launched missile is given as its destination.

## Current workaround

The controller target is cleared only when the loss resets the controller, that is when no
pending target waits. With a pending target waiting, the pending target is untouched, the
controller target is left as it was (as S-0506 r34 was built at intake 110), and the controller
goes on: the next launch adopts the pending target (claim 3).
