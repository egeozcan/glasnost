---
id: Q-0078
title: Which position the flight rows test after a cruise lift, and the height of a vertical projectile's deck detonation
status: answered
related: [S-0306]
answered-by: [S-0306]
---

## Need
Two details of S-0306's flight decide where a projectile is when the rows of
table T1 are tested and where it detonates. The code is in
`combat::projectile::turn` and `steer`.

1. Claim 15 lifts a cruising steering projectile by 18 leptons (or lowers it
   by 18) as "a direct change of position", and says only that the steering
   distance of Terms is measured at *P*′ before this lift. T1 tests *P*′ ("its
   new position") in several other rows: the deep-fall row (*h*′ ≤ −416), the
   flak row (*h*′ < 0), the bridge-deck crossing row, the arrival and
   passed-by rows (η = trunc(*d*₃(*P*′, *K*) / 2)), the anti-air row and the
   closing step *c*ₖ. The spec does not say whether these rows see the
   position before or after the lift, nor whether a detonation "at *P*′"
   includes the lift.
2. T1's vertical row detonates a vertical projectile that "crosses a bridge
   deck height", and T2 gives such a detonation "at *P*′, with the vertical
   change of the first row when that row applies". The first row applies only
   to straight and arcing projectiles, so by T1 and T2 a vertical projectile
   detonates at its own new height. Scenario P33 run c expects the detonation
   height of a vertical projectile launched upward through the deck to be the
   deck height.

## Questions
1. In a turn with a cruise lift of 18 leptons, do the deep-fall, flak,
   bridge-crossing, arrival, passed-by and anti-air rows and the closing step
   use the position before the lift or after it? If a row detonates the
   projectile in that turn, is the detonation point before or after the lift?
2. When a vertical projectile crosses a bridge deck height (going up or
   going down), at which height does it detonate: its new height, the deck
   height, or (going down) 20 leptons below the deck as for a straight or
   arcing projectile?

## Possible observation
1. S-0306 scenario P9 with the recorder of projectile positions: a steering
   projectile cruising low over a plain towards a target on a plateau whose
   edge it reaches in a turn that ends 400 to 420 leptons inside the rising
   ground before the lift. Whether the deep-fall row fires in that turn shows
   which position it tests.
2. S-0306 scenario P33 run c, recording the detonation height, and the same
   with the vertical projectile launched downward through the deck.

## Current workaround
1. Every row of T1 tests *P*′ before the lift, and a detonation in that turn
   is at *P*′ before the lift. The lift is added only to the position the
   projectile keeps when no row applies.
2. A vertical projectile that crosses a deck detonates at its new height *P*′,
   as T2 reads.
