---
id: Q-2231
title: What do a projectile's target-object effects do to a departed warp member, and which render layer is a member that departed from a deck in?
status: answered
related: [S-0306, S-0513, S-0514, S-0102]
answered-by: [S-0306]
---

## Need
S-0306 revision 57, claim 17 (answering Q-2212), lets a projectile keep a superweapon warp member
as its target object from the member's departure until it appears at its landing point, and says
what the detonation's ordinary area damage does to it. Its Open questions leave two things open,
and the engine must do something for both:

- A detonation whose warhead selects a special hit of S-0514 claim 20 (takeover, bomb planting,
  electric charging, temporal erasure, locomotor seizing, air-strike designation, bomb disarming)
  or the disguise of S-0516 claim 31 acts on the projectile's target object itself, not through
  area damage. What it does when that target is a departed member is not established, while the
  member is off the map and while it phases in after it appears (`combat::projectile`,
  `effect_target`).
- Claim 20 (c) chooses a distance threshold of 42 or 128 by whether the target is airborne in the
  ground render layer or in another. A member that departed from a high-bridge deck counts as
  airborne (claim 17, `quirks.warp_deck_departure_airborne`), but the render layer it is in until
  it appears is not established (`combat::projectile::detonation_point`).

## Questions
1. A projectile with a takeover, bomb-planting, charging, erasing, seizing, air-strike or
   disarming warhead (one at a time) detonates while its target, a departed warp member, is off
   the map. Does the effect act on the member (does it change owner, carry a bomb, get erased or
   seized, and so on, when it appears), or on nothing? Is ordinary damage still replaced?
2. The same projectiles detonate on the member after it has appeared, while it is still phasing
   in (before its arrival hold ends). Does each effect act on it as on any other target?
3. A projectile aimed at a vehicle that departed from a high-bridge deck (counted as airborne by
   claim 17) detonates near the point the member stood at. Which threshold of claim 20 (c) is used
   for it: 42 or 128?

## Possible observation
S-0306 scenario P11 run f, with V standing on a high-bridge deck for question 3 and with each of
the warheads above for questions 1 and 2; for question 1 the projectile is slowed so that it
reaches V's old position before V appears. Record the detonation point, the member's owner, bomb,
erasure or seizure state after it appears, and the hits on other objects near the detonation.

## Current workaround
Question 1: no target-object effect acts on a departed member while it is off the map, as when the
projectile had lost its target under the interim of Q-2212. The selected special row is still
chosen with the member as the target and still replaces ordinary damage (S-0514 claim 20).
Question 2: after it appears, the member is a target on the map like any other, so the effects act
on it. Question 3: the render layer its kept state gives: a vehicle on a deck is in the ground
render layer, so the threshold is 42. An aircraft member above the ground is in another layer, so
its threshold is 128.
