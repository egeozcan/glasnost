---
id: Q-0771
title: In which tick the animations of shots, detonations and deaths take their first update (S-0109 claim 5)
status: open
related: [S-0109, S-0203, S-0302, S-0312]
answered-by: []
---

## Need
S-0109 claim 5 says an animation created before or during the turn pass takes its first update
in its creation tick, after every object already in the update order, and one created after the
turn pass first updates in the next tick; "each creator's spec owns its place in the tick", and
the claim's backing leaves animations created during or after the turn pass `unmeasured` until a
creator's spec names its place. The first update produces no advance, so this place decides how
long the first image frame of a stage delay of 1 is shown: 1 tick when the first update is in the
creation tick, 2 ticks when it is in the next tick. The creators the drawing plays in a game are
the animations of a weapon's shot, of a warhead's detonation (S-0302), of a death or destruction
(S-0312), of projectile trailers and of ship wakes. The drawing needs each creator's place to
start each animation's clock (`mapview::live`).

## Questions
1. For each of these creators (a shot's weapon animation, a detonation's warhead animation, a
   death or destruction animation, a projectile's trailer animation, a moving ship's wake), is
   the animation created before, during or after the turn pass?
2. When it is created during the turn pass, does it then receive its first update in the same
   tick (it joins the update order after every object already in it), or only in the next tick?
3. Does an animation created by a projectile's detonation in the projectile's own turn behave
   differently from one created by an object's death in another object's turn?

## Possible observation
A scenario in the AN1 style with a weapon, warhead and death animation type each of rate giving
*d* = 1 and three image frames, recording the drawn image frame at the end of each tick from the
shot, the detonation and the death: a first image frame shown for 1 tick means a first update in
the creation tick, 2 ticks the next tick.

## Current workaround
Interim rule, recorded in docs/development.md (section *Animation playback (M6b)*): every
animation the simulation reports in a tick's effects takes its first update in that tick (it is
treated as created during the turn pass). The move-target marker, created by input between ticks,
takes its first update in the next tick by claim 5's after-the-turn-pass rule.
