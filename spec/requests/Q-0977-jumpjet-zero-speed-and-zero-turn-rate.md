---
id: Q-0977
title: Jumpjet types with a speed field of 0 or a turn-rate field of 0
status: open
related: [S-0502]
answered-by: []
---

## Need

S-0502 (Edge cases, Questions for the project lead) says the original divides by zero for a type
with a speed field of 0 (the speed fraction) and for one with a turn-rate field of 0 (the middle
cruise band), and proposes a deterministic departure: a speed fraction of 0 for *S* = 0 and an empty
middle band for *T* = 0, with an importer warning and no quirk switch. The proposal is not yet
approved, and the spec does not prescribe the values.

## Questions

1. Is the proposed departure approved as written (fraction 0 for *S* = 0, no middle band for
   *T* = 0, an importer warning for both)?
2. Is *S* = 0 also to be refused in the bands (every band speed 0), as the formulas give?

## Possible observation

None (the original stops with an error).

## Current workaround

`jump::fly::speed_step` gives fraction 0 for *S* = 0 and the cruise bands skip the middle band for
*T* = 0. The importer warns for either key set to 0 (`JumpjetSpeed=0`, `JumpjetTurnRate=0`).
