---
id: Q-3103
title: Projectile direction of a non-finite displacement, and a fragment's zero displacement, with quirks.table_direction off
status: answered
related: [S-0204, S-0306, S-0318]
answered-by: [S-0204]
---

## Need
S-0306 revision 90 (answering Q-3082) gives the direction of F11 and F14 with
`quirks.table_direction` off for a non-zero displacement (S-0204's exact angle) and for a zero
one (16384). Both arguments are narrowed to single precision first. A velocity component, or a
difference of positions, that is too large for single precision narrows to an infinity, and a
component that is not a number stays one. The exact angle of S-0204 Edge cases is defined only
for finite arguments, so the off form has no value for such a displacement. The engine needs one
in `glasnost_num::geom::direction_real_b` (#294).

A related zero case: a fragment of S-0318 claim 6 leaves in the direction from the burst point
towards its aim point's reference point, and its velocity is S-0306 F1's (so θ(*y*) takes the
exact scale with the switch off). Revision 90's zero-displacement value names claim 5, F11 and
F14, not this direction. With the switch off and the burst point straight above the reference
point, S-0204's exact angle is undefined.

## Questions
1. With `quirks.table_direction` off, what direction do F11 and F14 give when one or both
   arguments are infinite after the narrowing (for example X = +∞, Y finite; X = +∞, Y = −∞)?
2. And when one or both arguments are not a number after the narrowing?
3. With the switch off, what direction does a fragment (S-0318 claim 6) take when its burst
   point is straight above the reference point (a planar displacement of 0): 16384 as for S-0306
   claim 5, or something else?

## Possible observation
None. The switch-off state is our own fix, not the original's behaviour. This is a decision for
the spec side, like the other switch-off forms of S-0204. Such a displacement is probably out of
reach in play, since speeds and positions stay far below the single-precision range.

## Current workaround
With the switch off, a displacement with an argument that is not finite in single precision
keeps the table path's direction (S-0204 F3), as with the switch on. A fragment's zero planar
displacement takes the engine's exact direction of a zero integer displacement, 0 (map north), as
before revision 90. The default (switch on) is unaffected.
