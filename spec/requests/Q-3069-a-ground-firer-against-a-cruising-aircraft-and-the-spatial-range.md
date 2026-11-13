---
id: Q-3069
title: Does a ground firer's range test against a cruising aircraft count the full flight height?
status: answered
related: [S-0307, S-0306, S-0309, S-0506]
answered-by: [S-0307]
---

## Need

Issue #431 reports that a capturable turret building whose weapon hits air and ground does not
fire at aircraft or rocket missiles that pass within its range. Our engine follows S-0307 as
written. A local probe with an installation's rules found the firing check without range
passing for a captured building against a helicopter (in operation, versus test passed, the
aircraft in the flying-object scan). The range test of S-0307 claim 9 then failed at every tick
of the flight.

S-0307 claim 9 (projectile not arcing): the target is in range when
⌊*r*(Δ*x*² + Δ*y*² + Δ*z*²)⌋ ≤ *R* + *E*₁ + β. "The spatial distance is used for every firer,
aircraft included." For a floor-bound firer, claim 8 gives *S* as the firer's reference point
(a building: the centre of its foundation, S-0204 claim 4). Claim 7 replaces the height of *M*_T
only for a floor-bound target, so an airborne target keeps its world height. Δ*z* is then the
target's whole flight height above the firer's reference point. The air bonus of claim 3 is
the only term that adds to *R* for an airborne target.

When a weapon's reach is not much above an aircraft's cruise height, this reading puts a cruising aircraft in range only almost straight overhead. A building type with no
air bonus then hardly ever fires at air. The lead's play report says this does not match the
original. A rocket missile at cruise meets the same geometry. S-0307 claim 26 (built for #253)
changes only when a set-A or set-B rocket counts as airborne. It does not change the distance.

## Questions

1. For a floor-bound firer (a building in particular) and an airborne target, is Δ*z* in claim 9
   the target's full world height above the firer's reference point, as claims 7 to 9 read now?
2. Is the firer's height in that difference its reference point's world height (the terrain at
   its foundation centre), or a raised point such as its fire point (S-0313) or a building height?
   Claim 8 says the fire point is not used for the range test. Does that hold for a building
   firing at air?
3. Does anything else extend a building's reach against an airborne target besides claim 3's air
   bonus (for example a different measure of the distance, a type-level or game-wide value, or the
   acquisition scan of S-0309 using another test)?
4. Is the answer the same for a rocket missile at cruise (S-0506) as for an aircraft?

## Possible observation

A test building type with a turret and a weapon that hits air and ground, with no air bonus and
a non-arcing projectile. Its reach is set a little above a test aircraft type's cruise height.
Both belong to opposed houses on a small flat map with the recorder. Fly the aircraft past at
several planar offsets from the building's centre, from directly overhead out to the weapon
reach. Per tick, record the aircraft's position and height, the building's target, and each shot.
Variants: the same with an air bonus of one cell, and the same with an arcing projectile.
Expected under the current reading: shots only while the spatial distance is within the reach.
Falsified by shots at planar offsets where the spatial distance exceeds it.

## Current workaround

None needed in code: the engine follows claims 7 to 9 as written. #431 stays open as a finding;
no change was made.
