---
id: Q-2237
title: Which animation an aircraft's damage smoke shows, where, and what its creation takes
status: answered
related: [S-0501, S-0109, S-0312]
answered-by: [S-0501]
---

## Need
S-0501 claim 37 (*Damage smoke*) takes one *U*(0, 99) draw in each update tick
in which an aircraft's health ratio is below the condition-red ratio and its
height is above 0, and says smoke is shown when *u* < 10 for a living aircraft
and *u* < 80 for a dead one. No claim names the animation that is shown, where
it is created, whether it follows the aircraft, or whether its creation takes
the draws of an ordinary animation creation (S-0109 claim 3).
`glasnost_sim::air::smoke` takes the draw and decides whether smoke is shown,
but creates nothing.

M8 round 7 note 40 ("planes use the landing animation when they die ... in the
orig. game, they have a dying animation"): a shot-down aircraft is dead while it
falls, so it would show this smoke on about four ticks in five. The tumble of
S-0113 claim 16 is now drawn; the smoke is not.

## Questions
1. Which animation type does an aircraft's damage smoke show: a fixed type, a
   type named by a game-wide rules key, or one named by the aircraft's type?
   What happens when that type is absent from the rules?
2. Where is it created: the aircraft's position, its centre raised or lowered,
   or a point displaced by a type key? Does it stay where it was created or
   follow the aircraft?
3. Does its creation take S-0109 claim 3's stage-delay draw, or any other draw,
   on the scenario stream, after claim 37's own draw?
4. Is it the same smoke for a living damaged aircraft (u < 10) as for a dead one
   (u < 80)?

## Possible observation
S-0501 scenario AC5 with a recorder of animation creations: a damaged aircraft
below the condition-red ratio flying straight, then shot down, recording each
created animation's type, position and tick, and the draws in each tick in
which one is created.

## Current workaround
None: no smoke animation is created, and the client draws none. The draw is
taken as claim 37 gives.
