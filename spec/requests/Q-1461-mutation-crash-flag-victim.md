---
id: Q-1461
title: Does a converted victim whose type has the crash flag still leave a conversion animation
status: open
related: [S-0611, S-0107]
answered-by: []
---

## Need
S-0611 claim 7 says a converted victim leaves play in its death tick, "its type's crash flag
keeps S-0107 claim 17's exception", and claims 12 and 13 repeat "subject to claim 7's crash
exception". The spec does not say what the conversion still does for such a victim: whether the
conversion animation of claim 8 starts at its position (and later makes a unit) while the victim
crashes, or whether the crash replaces the whole conversion. The code is the crash branch of
`convert` in `crates/deterministic/sim/src/mutation.rs`.

## Questions
1. When a mutating death reaches row 7 for an infantry type with the crash flag and the conversion
   test of claim 6 passes, does a conversion animation start at its position in the death tick?
2. If it does, does that animation make a unit at its end as for any other victim, and to which
   house?
3. Is the victim's standing position shown free from its death tick (as for a refused victim),
   or does it stay taken until the crash ends?

## Possible observation
A named map override that gives an ordinary infantry type the crash flag, killed by a mutating
warhead on open ground; record animations, units and the occupancy record until the crash ends.

## Current workaround
The victim crashes (S-0107 claim 17's exception), keeps its standing position, and no conversion
animation starts. Marked Q-1461 in the code.
