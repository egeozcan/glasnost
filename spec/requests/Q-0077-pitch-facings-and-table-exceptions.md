---
id: Q-0077
title: Whether pitch facings made from the table angle take S-0204's 50 table exceptions
status: answered
related: [S-0306, S-0204]
answered-by: [S-0306, S-0204]
---

## Need
S-0306 turns a table angle into a pitch facing in two places:
- F12, the pitch of a velocity: *p*(*V*) = trunc((*A*(*V*_z, *H*) − π/2) × *c*₂);
- F13, the straight launch pitch: trunc((*θ*_c − π/2) × *c*₂) with
  *θ*_c = sgn(*Z*) × *A*(|*Z*| − *τ*, max(*d*′, 0.05)).

Both use "the table angle *A* of S-0204 F3". S-0204 F3 defines *A* with a
surrogate table and says that the surrogate gives the original's *direction
facings* exactly only after a correction at 50 table indices, where "the
facing *φ* comes out one facing unit nearer to map east (when *X* > 0) or map
west (when *X* < 0)". It adds that "any other use of the raw angle (outside
this spec) needs the original's table values, not the surrogate".

A pitch facing is made from *A* by the same map as a direction facing, but
it is not a direction. So it is not settled whether the 50 exceptions apply
to it. Our code is not consistent: `combat::projectile::pitch_of` (F12)
applies the exceptions and `straight_pitch` (F13) does not. Under either
answer one of them is off by one facing unit at those indices.

## Questions
1. When the table index of F12's *A*(*V*_z, *H*) is one of S-0204's 50
   exception indices, is the pitch facing one unit nearer to level (16383)
   than the surrogate formula gives, as for a direction facing? (Note that
   F12's *X* argument *H* is never negative.)
2. The same question for F13: when the index of *A*(|*Z*| − *τ*, max(*d*′,
   0.05)) is an exception index, is the launch pitch one unit nearer to level
   than the surrogate gives, for *Z* > 0 and for *Z* < 0 (where the angle is
   negated)?
3. If neither takes the exceptions, which angle values should be used at those
   indices (S-0204 says that other uses need the original's table values)?

## Possible observation
S-0306 scenario P4 with |Δ*z*| and *d*′ chosen so that the table index of F13
is 1024 (an exception index, as in S-0204 scenario C1 run g), comparing the
recorded first velocity's pitch with both candidates. For F12, scenario P9 run
c (direct mode) with the recorder of velocities, at a turn whose
velocity's index is an exception index.

## Current workaround
F12 (`pitch_of`) applies the exceptions and F13 (`straight_pitch`) does not,
as the code stood before this request. The claim tests compute F13 without
the exceptions.
