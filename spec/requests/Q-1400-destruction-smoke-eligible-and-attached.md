---
id: Q-1400
title: Which destruction particle systems are eligible, what counts as attached, and whether the smoke outlives a building that leaves at once
status: open
related: [S-0312, S-0616]
answered-by: []
---

## Need
S-0312 claim 19 (f) makes one system from a destroyed building's destruction particle-system
list "when the building has no particle system attached already, at least one entry is eligible
(the particle-system specs say which kinds are)". S-0616 claim 1's destruction-smoke row names
the owner object (the building) but does not say which kinds of entry are eligible, nor what
"attached already" means. A building that does not explode leaves play at the end of its death
effects (S-0312 claim 21), which by S-0616 claim 4 marks every system it owns before that system
has taken a turn, so the destruction smoke of such a building never emits.

## Questions
1. Which behaviour kinds of entry in the destruction list are eligible: smoke only, or also fire,
   spark, railgun, gas or none?
2. Which systems count as "attached already": a damage-smoke system, a natural-smoke system,
   an effect weapon's emitter, a damage-sparks system, any system whose owner object is the
   building?
3. For a building that does not explode (it leaves play at once), does its destruction smoke
   emit at all? Is it marked at once by the owner's removal, or does it keep emitting for some
   time after the building has gone?

## Possible observation
A test rules file giving a non-exploding building a destruction list of one smoke, one spark
and one fire system; destroy it many times and record which system types appear and how long
each emits.

## Current workaround
Interim rules in `crates/deterministic/sim/src/emitters.rs` (`destruction_smoke`): only
smoke-kind entries are eligible; "attached" means any of the building's four emitter places
holds a system (the natural smoke, which has no owner object, does not count); the system is
owned by the building, so a non-exploding building's smoke is marked when the building leaves
play in the same step (claim 4).
