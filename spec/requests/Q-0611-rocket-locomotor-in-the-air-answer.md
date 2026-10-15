---
id: Q-0611
title: When does a rocket missile of the set-A or set-B type count as in the air?
status: answered
related: [S-0506, S-0501, S-0307, S-0308, S-0303, S-0309]
answered-by: [S-0506]
---

## Need

S-0501 Terms, S-0307 claim 26 and S-0308 *Airborne* say that the aircraft of the game-wide
V3-rocket and dreadnought-missile types take their in-the-air answer from their locomotor, not
from the 208-lepton height test, but no spec states that locomotor's answer. S-0506 defines the
rocket-flight movement mode and does not state it either. Our engine needs it in
`glasnost_sim::combat::weapon::in_air` (weapon choice, range, the area-damage half distance of
S-0303 claim 10, the removal test of `objects::remove`).

## Questions

1. In which flight phases of S-0506 claim 15 (pause, tilt, raise, ascent, cruise, descent) does
   a rocket missile of either type count as in the air?
2. Does the answer depend on its height above the terrain at all, and does it change while the
   missile is destroyed and still flying (S-0506 claim 20)?
3. Does a missile of the set-C rocket type, which is not one of the two named types, use the
   ordinary 208-lepton test?

## Possible observation

An anti-air-only and a ground-only defence beside a launcher (S-0506 scenario R13 setup): record
which one fires at the missile in each phase; and a detonation of known spread near a missile in
pause and in cruise, comparing the damage with the half-distance rule of S-0303 claim 10.

## Current workaround

`glasnost_sim::combat::weapon::in_air` answers for an object of the set-A or set-B rocket type
with the 208-lepton height test of S-0307 Terms (the ordinary aircraft answer), in every phase
(marked Q-0611 in the code and in docs/development.md, M6a interim rules).
