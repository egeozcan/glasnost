---
id: Q-0615
title: In which ticks do a rocket missile's phases begin and end, and where does a launch pass stand in its spawner's tick?
status: answered
related: [S-0506, S-0203]
answered-by: [S-0506]
---

## Need

S-0506 claim 15 and its Rocket table give the order of a rocket missile's phases, and claim 2
gives the ticks of launch passes, but neither says how the phase durations fall on ticks, or where
a pass stands in the spawner's own tick. Our engine needs both in `glasnost_sim::rocket::tick` and
in `glasnost_sim::tick` (the call of `glasnost_sim::spawn::turn`).

## Questions

1. The tick in which the aim point is fixed (claim 14): does the missile count it as its first
   tick of pause (or of tilt when the pause time is 0), or does pause start in the next tick?
2. With a pause time of *P* ticks and a tilt time of *T*, how many ticks pass from the tick that
   fixes the aim point to the first tick in which the missile moves (its first positive-speed
   movement)?
3. The tick in which ascent reaches the cruise altitude: does the missile already steer and set
   its pitch by the cruise rules in that tick, or does it still accelerate as in ascent and start
   cruise in the next tick?
4. Within the spawner's own tick, does the launch pass come before or after its firing (S-0301),
   so that a shot made in a pass tick is adopted in that same pass?

## Possible observation

S-0506 scenarios R2 and R10: record the tick of the shot, the missile's appearance, its first
change of pitch, its first change of position and its first heading change after reaching the
cruise altitude, for several pause and tilt times including 0.

## Current workaround

The tick that fixes the aim point does nothing else; each later tick of pause, tilt or raise
counts one tick of its time, and the next phase starts in the tick after the time is used up. The
tick that finds the cruise altitude switches to cruise without accelerating, and cruise rules
start in the next tick. The launch pass runs in the spawner's turn after its behaviour processing
and before its movement and firing step (marked Q-0615 in `glasnost_sim::rocket`,
`glasnost_sim::tick` and docs/development.md, M6a interim rules).
