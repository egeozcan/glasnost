---
id: Q-0613
title: Does a destroyed rocket missile keep its phase timers, and does it fall?
status: answered
related: [S-0506, S-0312]
answered-by: [S-0506]
---

## Need

S-0506 claim 20 keeps a rocket missile destroyed above the terrain in the rocket-flight movement
mode until its next positive-speed movement attempt. Its first open question asks whether the
pause and tilt timers keep running while it crashes, and whether the crash fall of S-0312
claim 16 also moves it. Our engine needs both answers in `glasnost_sim::rocket::tick` for a dead
rocket.

## Questions

1. A rocket missile destroyed in pause or tilt: do the pause and tilt timers keep counting, so
   that ascent (and its first positive-speed move, which detonates it) comes at the same tick as
   for an undamaged missile?
2. Between the destroying hit and that movement attempt, does the missile's position change by
   anything other than the rocket step of S-0506 Formulas F1?

## Possible observation

S-0506 scenario R13: kill a missile during pause and during tilt, and record its position each
tick and the tick and point of its detonation.

## Current workaround

A destroyed rocket missile keeps running its phases exactly as an undamaged one and moves only by
F1; it detonates after its first positive-speed movement attempt (marked Q-0613 in
`glasnost_sim::rocket` and in docs/development.md, M6a interim rules).
