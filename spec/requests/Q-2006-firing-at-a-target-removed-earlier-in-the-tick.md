---
id: Q-2006
title: What a ground unit does with a target that has just left play, and what the firing check answers for it
status: answered
related: [S-0301, S-0309, S-0310, S-0210, S-0215]
answered-by: [S-0309]
---

## Need

Two things can make a unit's target leave play while the unit still holds it. A crusher can drive
over its own target (S-0210 claims 3 and 4), or another unit can remove it earlier in the tick.
The specs we have do not say what the holder does next:

- S-0215 claim 10 (override and revert) says that after a revert by removal "that object is the
  actor's target again", and that "what the actor then does with such a target belongs to the
  target-acquisition specs".
- S-0309 table C's *exists* row keeps a removed object out of a **search**. It says nothing about a
  target the unit already **holds**.
- S-0301 claim 9 (the firing check's ranks) and S-0310 claim 9 (a vehicle's firing step) do not say
  what the firing check answers when the held target object is no longer in play.

In our engine, the firing check treats a target object that is no longer in play as no target.
With no target, the range test passes and the answer is *can fire*. So the vehicle's firing step
makes a shot "at" the removed object.

Our synthetic test `m0011_a_berserk_crusher_holding_a_berserk_friendly_target_drives_over_it`
(`crates/deterministic/sim/tests/claims_s0610.rs`) shows it. A computer-controlled crusher holds
an enemy vehicle as its target and reaches the cell centre. The crush removes the target with no
damage and plays the crush sound (claims 3 and 4). In the same tick, in the same unit's firing
step, it still holds the removed object as its target, and it fires one shot. Before the crush it
never fired at that target: the crusher was berserk and the target berserk-friendly (S-0610
claims 6 and 8). For now our test leaves out the shot made in the crush tick.

## Questions

1. A vehicle drives over and crushes the very object it holds as its target. In the rest of that
   tick, does it fire at that target, or at that target's last place?
2. When a ground unit's target object leaves play (crushed, killed and removed, or taken out of
   play some other way), does the unit drop it at once? Does it drop it at its next turn? Or does
   it keep it until something else clears it?
3. If it keeps it, what does its firing check answer for that target? For example: *illegal*,
   *impossible*, or *can fire* at the last place.

## Possible observation

A computer-owned crusher with an attack target it can crush, less than the crush range away
(S-0210 scenario K7, the attack-target variant). Record per tick the crusher's target, its fire
events and its rearm state around the tick of the crush. A second run: an infantry unit shoots at
an enemy that a third unit kills and removes in the same tick. Record whether the first unit's
next firing step fires.

## Current workaround

The built rule: the firing check treats a removed target as no target and answers as it does for
no target. A vehicle that holds such a target can still fire once in that tick. We have not
changed this, and no arena game depends on it today. Our tests do not count the shot made in the
tick of the crush.
