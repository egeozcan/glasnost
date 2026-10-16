---
id: Q-0986
title: The random draws and burst state of a charge planter's rearm, the moment a strike notices a change of target, and the effect of a missing bomb warhead
status: answered
related: [S-0519, S-0301, S-0213, S-0215, S-0203]
answered-by: [S-0519]
---

## Need

1. S-0519 claim 13 sets the walker's rearm countdown "to the delay that S-0301 claims 1 to 8 give
   a shot of its second weapon". That delay includes the draw R(0, 2) of S-0301 claim 2 (and R(3, 5)
   inside a burst). The claim does not say whether the arrival takes those draws, nor whether the
   walker's burst counter advances as after a real shot.
2. S-0519 claim 19 cancels a strike "in the tick of the cancelling event" when the designator's
   target changes. Our engine has no single place where an object's target changes (orders,
   acquisition, the loss of a dead target, behaviours all write it), so it cannot hook the change
   itself. The claim does not say whether a change that is made and undone within one tick, or one
   made after the designator's own turn in a tick, must cancel in that same tick or may be noticed
   at the designator's next turn.
3. S-0519 Edge cases leave the effect of a missing bomb warhead or demolition warhead open. Stock
   data always gives both, but a mod need not.

## Questions

1. Does the charge arrival take the second-weapon delay's draws and advance the burst counter, or
   is the rearm set from the weapon's fixed rate without them?
2. Is a strike cancelled at the moment of any write of the designator's target, or at the first
   look of the strike's own processing after it (the designator's turn, or a designation shot)?
3. With no bomb warhead, does a bomb's detonation hurt nothing at all but still end the bomb and
   play its animation? With no demolition warhead, does a suicide shot still kill its firer?

## Possible observation

TC13 with the random-state recorder around a charge arrival with a second weapon of rate 30;
TC19 with a target change ordered in the tick of the designation shot.

## Current workaround

- `bomb::charge_arrival`: the rearm takes the delay `combat::fire::second_weapon_delay` computes,
  with its draws (R(0, 2) for a single-shot weapon), and leaves the burst counter alone.
- `strike::turn` and `strike::designation`: the designator's target is compared with the one seen
  last at the designator's own turn and at every designation shot, and a difference cancels then.
- `bomb::detonate`: with no bomb warhead the bomb ends and nothing else happens (no animation);
  `bomb::suicide`: with no demolition warhead the firer still takes its hit (a hit without a
  warhead).
