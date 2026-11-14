---
id: Q-3082
title: Projectile facing-to-angle scale and zero-displacement direction with quirks.table_direction off
status: answered
related: [S-0204, S-0306]
answered-by: [S-0306]
---

## Need
S-0306 Formulas put the map-plane directions of F11 (the current and goal directions of a
steering turn) and F14 (the drawn facing φ) under S-0204 F3 and its switch
`quirks.table_direction`. With the switch off, S-0204 Edge cases give the direction itself: the
exact angle, rounded to the nearest facing with halves up. The engine now does this (#294,
`glasnost_num::geom::direction_real_b`). Two related points have no off form in the specs:

- S-0306 turns a facing back into an angle with θ(*f*) = (*f* − 16383) × (−π / 32767), for facings
  and pitch facings alike (F1's launch facing *y*, F11's new direction *y*ₙ). S-0204 Edge cases
  say the switch also covers the reverse scale "wherever a rule uses the same short scale", but
  give the off form only for S-0208 F3. They also say pitch conversions stay on the table with
  the switch off.
- F14 gives 16383 for a zero velocity with the table path (both arguments 0). F11 needs a
  direction for a zero displacement too. The exact angle of S-0204 Edge cases is not defined
  for X = Y = 0.

## Questions
1. With `quirks.table_direction` off, does S-0306's θ(*f*) for a map-plane facing (F1's *y* and
   F11's *y*ₙ) become θ = (*f̂* − 16384) × (−2π / 65536), as S-0208 F3's off form does, while θ(*p*)
   for a pitch facing stays (*p* − 16383) × (−π / 32767)? If so, which floating-point steps and
   constants does the off form use?
2. With the switch off, what direction do F11 and F14 give for a zero displacement (both
   arguments 0 after narrowing to single precision): 16383 as with the switch on, 16384 (map east
   on the exact scale), 0, or something else?

## Possible observation
None. The switch-off state is our own fix, not the original's behaviour. This is a decision for
the spec side, like the other switch-off forms of S-0204.

## Current workaround
With the switch off, the code gives the exact direction for a non-zero displacement. A zero
displacement keeps the table path's value (16383), and θ(*f*) is unchanged for every facing. The
default (switch on) is unaffected.
