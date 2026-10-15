---
id: Q-0610
title: The raw arctangent angles that steer a rocket missile's pitch
status: answered
related: [S-0506, S-0204]
answered-by: [S-0204]
---

## Need

S-0506 Formulas F6 (lazy curve) and F7 (descent) take the line-of-sight angle *ℓ* from the raw
table arctangent *A*: for an index *i* ≤ 4096 the magnitude is the original single-precision
angle *T*(*i*). S-0204 claim 8 describes the table but supplies only surrogate angles, and S-0506
says those are not an exact substitute (its last open question). Our engine needs the 4097
single-precision angles in `glasnost_sim::rocket` (lazy cruise and descent pitch), or a rule
that produces them bit-exactly.

## Questions

1. What are the 4097 single-precision angles *T*(0) … *T*(4096) that the rocket pitch uses, as
   observed through a rocket missile's pitch (or a measured table)?
2. Are they the same angles that S-0204 claim 8's facing table is built from, before its
   corrections to integer facings?

## Possible observation

A set-A rocket (lazy curve) and a set-B rocket (descent) fired over flat ground at many
distances; record the per-tick positions and solve F1 for the pitch each tick. Each pitch fixes
one *T*(*i*) through F6 or F7.

## Current workaround

`glasnost_sim::rocket` uses the surrogate table `glasnost_num::tables::ATAN` as *T*(*i*) (marked
Q-0610 in the code and in docs/development.md, M6a interim rules). Pitches during lazy cruise and
descent can differ from the original in the last bits.
