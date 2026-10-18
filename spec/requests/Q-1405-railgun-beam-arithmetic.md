---
id: Q-1405
title: The railgun beam's arithmetic: F14's last sample under round-toward-zero, and the line distance
status: open
related: [S-0616, S-0306, S-0205]
answered-by: []
---

## Need
S-0616 F14 and S-0306 claim 27 (revisions 8 and 14) are built (`combat::railgun`), but three
points of their arithmetic are not fixed, and the engine has to pick something at each.

1. **F14's last sample.** F14 gives *t*ᵢ = fl(*i* × δ) in single precision with δ the single
   nearest 1/49, and also says *t*₄₉ = 1.0 exactly, so *s*₄₉ = *Q*. The exact product 49 × δ lies
   11 × 2⁻²⁹ below 1. Rounded to nearest it narrows to 1.0, but in the program's
   round-toward-zero mode (S-0205, `quirks.chop_rounding`, on by default) it narrows to the
   single just below 1.0 (1 − 2⁻²⁴). With that value *s*₄₉ is not always *Q*. For example, a
   positive component at an exact multiple of 256 truncates to one lepton less, which can put
   the last sample in the cell before *Q*'s. The two statements of F14 disagree in that mode.
2. **The line distance of claim 27.** The claim says the distance from the infinite line through
   the start and target positions is computed in floating point, "with single-precision
   coordinate values and mixed single- and double-precision intermediate values, using the
   table square root". It does not give the formula, which values are single and which double,
   or the order of the operations. Near the strict comparison with the railgun hit distance,
   those choices decide whether an object is hit.
3. **Degenerate beams.** Claim 27 leaves open a zero-length beam (start position equal to the
   target position, for example a target in the firer's own position) and a candidate exactly
   on the line.

## Questions
1. In the default (round-toward-zero) mode, is the last sample *s*₄₉ exactly the target position
   *Q*, or the sample computed from the toward-zero product of 49 and δ? Are the other *t*ᵢ formed
   in the program's rounding mode or rounded to nearest?
2. What exactly is the line distance: which formula (for example |(*C* − *P*) × (*Q* − *P*)| /
   |*Q* − *P*|, or the length of *C* − *P* minus its projection onto the beam), which
   intermediate values are single precision and which double, and is the table square root
   applied to both lengths?
3. For a zero-length beam, what distance does a candidate get, and is anything other than the
   shot target hit? Does a candidate exactly on the line (distance 0) qualify when the hit
   distance is 0 or below?

## Possible observation
S-0306 scenario P41 with S-0616 P22 part b: fire across a lane where the target's position has
an *x* that is an exact multiple of 256, and record the line's end point and whether the cell
before the target's is visited. For the distance, place bystanders at distances just inside and
just outside the hit distance from a diagonal beam (positions recorded exactly). For the
degenerate case, fire at a target standing at the firer's own position.

## Current workaround
`combat::railgun`:
1. *t*ᵢ = fl(*i* × δ) in the program's mode for *i* < 49, and *t*₄₉ pinned to 1.0, so *s*₄₉ = *Q*
   as F14 states.
2. The distance is |(*C* − *P*) × (*Q* − *P*)| / |*Q* − *P*|, in double precision from the exact
   integer differences, both lengths by the table square root (S-0204), with no truncation
   before the strict comparison.
3. A zero-length beam measures |*C* − *P*|. A candidate exactly on the line has distance 0 and
   qualifies only when the hit distance is above 0. The shot target always qualifies.
