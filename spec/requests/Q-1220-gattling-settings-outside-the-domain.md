---
id: Q-1220
title: Gattling spin settings outside the supported domain
status: open
related: [S-0609]
answered-by: []
---

## Need

S-0609 (Formulas, Edge cases, Questions for the project lead) leaves three input cases open,
saying no result is established and the deterministic handling is a design choice:

- a negative stage count, spin-up amount, spin-down amount or threshold;
- arithmetic that leaves the signed 32-bit range (a large amount times a large elapsed-tick
  count, a value grown past the range, an elapsed-tick count that keeps growing);
- a later rules layer that lowers the stage count below 2 after an earlier layer, with the
  gattling flag set and at least 2 stages, populated the thresholds.

The engine has to do something deterministic for each. The code is in
`crates/deterministic/sim/src/gattling.rs` (`SpinType::from_fields`, `f1`, `f2`, `elapse`) and the
importer warnings in `crates/convert/src/sim_combat.rs` (`spin_settings`).

## Questions

1. Should a negative stage count, amount or threshold be used as 0, rejected at import, or kept
   and run through F1 and F2 as written?
2. When a spin product, sum or elapsed-tick count would leave the signed 32-bit range, should the
   arithmetic saturate, wrap, or should the count stop growing at some bound?
3. When a later layer lowers the stage count below 2 after thresholds were populated, should the
   held thresholds still be used (they would give a spin cap from an earlier layer's numbers), or
   ignored as for a type that never had 2 stages?

## Possible observation

None needed: S-0609 classes these as outside the original's supported domain, so this is a
design decision for our engine rather than a measurement.

## Current workaround

Interim rules, noted in `docs/development.md` (Simulation, M7a: Gattling spin-up), each cited as
"Q-1220 interim" in the code:

- Negative stage counts, amounts and thresholds are used as 0. The importer warns for each.
- Spin arithmetic and the elapsed-tick count saturate at the 32-bit bounds.
- With a stage count below 2 as the layers finally leave it, the thresholds are not used: they are
  all 0, the spin cap is 0, the value never grows and the stage stays 0, as for a type that never
  had 2 stages. The importer warns about the reducing layer.
