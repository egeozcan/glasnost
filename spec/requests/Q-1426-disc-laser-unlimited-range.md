---
id: Q-1426
title: A disc-laser weapon with unlimited range
status: open
related: [S-0615, S-0307]
answered-by: []
---

## Need
S-0615 claim 14 cancels a disc laser when the distance "exceeds the weapon's range field (leptons,
with no additional range term of S-0307)". A weapon whose range field is the unlimited sentinel of
S-0307 claim 2 (−512 leptons) can carry the disc-laser flag. Read literally every distance exceeds
−512, so such a disc laser is cancelled at its first step and never deals damage. Code:
`crates/deterministic/sim/src/disc.rs`, `recheck`.

## Questions
1. Does a disc laser fired from a weapon of unlimited range ever reach its final step, or is it
   cancelled at its first step?
2. Is any other negative range field treated the same way?

## Possible observation
S-0615 scenario DL2 with the disc weapon's range set to unlimited.

## Current workaround
The literal reading: the distance is compared with −512, so the disc laser is cancelled at its
first step.
