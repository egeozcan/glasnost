---
id: Q-3129
title: Single-precision steps of the flak scatter distance
status: answered
related: [S-0306, S-0204]
answered-by: [S-0306]
---

## Need
S-0306 claim 4 gives a flak-scattering, visible, inaccurate arcing projectile the radius
σ = trunc(R(0, *B*) × *D* / *R*_W), where *D* is "the spatial distance from the fire point to the
aim point (computed in single precision, truncated)". The Terms entry *Point distances* names this
as an exception to its double-precision sum: "claim 4 in single precision".

The spec does not say which steps are single precision. For distances of 4096 leptons or more the
squares and their partial sums no longer fit in 24 significant bits, so the order of the additions,
and whether each square is narrowed before it is added, decide the sum. When the sum moves across a
boundary of the table root's truncation, *D* is one lepton lower, and σ with it. The engine needs a
fixed form to implement (#317).

## Questions
1. Are the three coordinate differences narrowed to single precision before they are squared, and
   are they integer differences first (wrapping or not)?
2. Is each square narrowed to single precision before it is added?
3. In what order are the three squares added (for example (Δ*x*² + Δ*y*²) + Δ*z*²), and is each
   partial sum narrowed to single precision?
4. Is the rounding of each narrowing the rules mode's (S-0205 claims 4 and 5, toward zero with
   `quirks.chop_rounding` on), and is the table root then S-0204 claim 5's real-input root with its
   own input narrowing?
5. Is *D* truncated toward zero after the root, as in *d*₃?

## Possible observation
A rules mod with a weapon of range about 10,000 leptons or more whose projectile is arcing,
inaccurate, flak-scattering and visible. Fire it at a cell displaced by (−6582, 7963, 206) leptons
from the fire point (the exact sum gives *D* = 10333, a sum narrowed after each square and each
partial sum, in the order *x*, *y*, *z*, gives 10332) and record the impact points of several shots
against the draws.

## Current workaround
*D* is S-0306's *d*₃: the squares and their sum formed exactly, then the table root of S-0204, with
its single-precision input narrowing, truncated. It agrees with any single-precision form for
distances under 4096 leptons.
