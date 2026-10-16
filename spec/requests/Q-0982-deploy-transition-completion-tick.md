---
id: Q-0982
title: Which of the two ticks an in-place deploy or undeploy completes on
status: open
related: [S-0517, S-0109]
answered-by: []
---

## Need

S-0517 F1 and F2 (claim 7) say that a deploy begun at tick t0 completes on tick T or T + 1,
where T = t0 + (E - 1) d for a deploy and t0 + (E - 2) d for an undeploy, and that the choice
between the two is the claim's "one-tick uncertainty". The simulation has to pick one tick for every
transition. A rule that depends on something in the game (the tick's parity, the order of two
objects' processing, the stage clock's phase relative to the processing, a draw) would have to be
built into the completion test; a constant choice is a guess.

## Questions

1. Is the choice between T and T + 1 a function of anything the game state determines (the tick
   the transition began, the number of other animations, the position of the vehicle in the
   processing order, a random draw), or is it the same for every transition of the same
   presentation?
2. If it is constant, which one is it for a deploy and for an undeploy: T or T + 1?
3. If it depends on the stage clock, does the animation's first stage advance on the tick the
   presentation is created or d ticks after it?

## Possible observation

A vehicle type with a deploying presentation of E = 5 and d = 2 (and one with d = 1), deployed
and undeployed several times at different start ticks (even and odd, and after other vehicles'
transitions); record the tick of the change to the deployed and mobile states against the start
tick.

## Current workaround

Completion is on T (the first processing at or after t0 + (E - 1) d for a deploy and
t0 + (E - 2) d for an undeploy), in `econ::inplace::completed`; the stage advance of a tick comes
before that tick's processing. Claim 7's own rule that a transition never completes in the
processing that began it holds.
