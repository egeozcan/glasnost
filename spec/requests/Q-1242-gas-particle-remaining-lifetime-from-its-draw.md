---
id: Q-1242
title: How a particle's remaining lifetime is formed from its lifetime draw
status: answered
related: [S-0613, S-0202]
answered-by: [S-0613]
---

## Need

S-0613 claim 5 says a new particle's remaining lifetime *E* "is drawn from its type's lifetime
field as S-0202 claim 15 sets", then reduced modulo 65536. S-0202 claim 15 gives the draw: one raw
value, "magnitude after signed remainder, modulo 10 for railgun behaviour, otherwise modulo the
imported lifetime field", and calls the result a *lifetime variation*. It does not say whether
*E* is that value itself, or the lifetime field plus or minus it, or some other combination.
F8 of S-0613 uses *E*ₘₐₓ − *E*, which suggests *E* does not exceed the field, and claim 5's
"initial zero" suggests the drawn value can be 0.

The code is `crates/deterministic/sim/src/gas.rs` (`lifetime`).

## Questions

1. Is a particle's initial remaining lifetime exactly the drawn value m(r rem *E*ₘₐₓ) (or
   m(r rem 10) for railgun), reduced modulo 65536, so that it lies in 0 … *E*ₘₐₓ − 1?
2. If not, how is it formed from the lifetime field and the drawn value?

## Possible observation

G5: record the random state used for the lifetime draw and the particle's removal tick over
20 runs, and compare with each candidate formula.

## Current workaround

*E* = m(r rem *E*ₘₐₓ) mod 65536 (m(r rem 10) for railgun behaviour), with S-0205 claim 3's
departure for a zero field. Docs: docs/development.md, *Simulation (M7a: gas clouds)*.
