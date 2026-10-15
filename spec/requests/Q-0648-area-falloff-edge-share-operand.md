---
id: Q-0648
title: Whether the area falloff's edge share is formed from the defended amount or its single-precision narrowing
status: answered
related: [S-0302, S-0303, S-0205]
answered-by: [S-0302]
---

## Need

S-0302 (revision 22) claim 6 case (vi) forms the edge share as *a* = RZ₂₄(*f* × *p*) with
*f* = RZ₂₄(*d*₃). S-0303 (revision 13) F3 forms the same quantity as
*E* = RZ₂₄(RZ₅₃(*d*₃ × *p*)), from the unnarrowed *d*₃, and says S-0302 claim 6 "must likewise"
change "at joint sign-off". Both specs are approved and say they describe the same computation,
but they disagree once *d*₃ is above 2²⁴. The code in `glasnost_sim::combat::damage` follows
S-0302's formula, which it cites.

## Questions

1. For a defended amount *d*₃ above 2²⁴, is the falloff's edge share formed from *d*₃ itself (S-0303
   F3) or from its single-precision narrowing (S-0302 claim 6)?
2. If S-0303's form is right, will S-0302 claim 6 be revised to match, so that the two specs agree?

## Possible observation

An area hit with a defended amount above 2²⁴ (a synthetic warhead with a very large damage value
and an edge fraction that is not a power of two), on a target at a distance strictly between 0 and
the spread radius. Compare the resulting damage with both formulas.

## Current workaround

The code keeps S-0302 claim 6's formula. The two forms differ only for defended amounts above
2²⁴, which no ordinary rules produce.
