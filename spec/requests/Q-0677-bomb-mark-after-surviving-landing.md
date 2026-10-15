---
id: Q-0677
title: Whether the bomb mark stays on an object that survives its landing
status: open
related: [S-0511, S-0312, S-0503]
answered-by: []
---

## Need

S-0511 claim 10 gives an object that lands with the bomb mark and positive health a hit of its
current health; S-0302 protections can stop that hit, and S-0511 scenario B3's turn-order
variant uses such a protected survivor. S-0312 Terms define the mark as causing landing damage
"when it finishes falling". Neither says whether the mark is spent by the landing. `fall::land`
needs to know, because a survivor that later falls again (a second bridge loss, a paradrop) or
dies low over water (S-0312 claim 11) would read the mark.

## Questions

1. After a marked object lands and survives the landing hit, does it still carry the bomb mark?
2. If it does, does a later fall without a new mark (for example a parachute drop) end with the
   landing hit again?
3. Does a vehicle that survived its marked landing and later dies low over water count as
   carrying the bomb mark for S-0312 claim 11?

## Possible observation

S-0511 scenario B3 with an invulnerable deck vehicle: after it lands and survives, move it onto
a second bridge deck that is then lost without the vehicle being on it, or make it otherwise
fall, and watch whether its next landing hits it.

## Current workaround

The bomb mark is cleared on landing, after the landing hit (whose death effects read it). The
choice is marked in the code with this request's ID.
