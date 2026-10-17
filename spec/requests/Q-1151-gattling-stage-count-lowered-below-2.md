---
id: Q-1151
title: Gattling thresholds when a later rules layer lowers the stage count below 2
status: open
related: [S-0609, S-0006]
answered-by: []
---

## Need

S-0609 revision 4 answers Q-1220's first two points with proposed engine rules (negative settings
used as 0, spin arithmetic saturating at 2³¹ − 1), but leaves its third point open: Edge cases
(*Stage count 0 or 1 from creation*), Import mapping and Questions for the project lead item 3 say
that no result is established when a later rules layer lowers a gattling type's stage count below 2
after an earlier layer, with the flag set and at least 2 stages, populated the thresholds. Our engine
must still build a spin type for such data. The code is `glasnost_sim::gattling::SpinType::from_fields`
(which thresholds the type uses) and the importer's `spin_settings` in `crates/convert/src/sim_combat.rs`
(which warns).

## Questions

1. For such a type, are the thresholds held from the earlier layer used as imported (S-0006 claim 3's
   held value for a key a layer omits, giving a spin cap from the earlier layer's numbers), or are
   all thresholds used as 0, as for a type that never had 2 stages?
2. If they are used as 0, should S-0006 claim 3 and the glossary's *rules layer* entry name this
   exception for the threshold keys?

## Possible observation

None proposed: the original's result for this data is not established, so this is a design decision
for our engine (S-0609 Questions for the project lead, item 3).

## Current workaround

Interim rule, kept from Q-1220's interim and noted in `docs/development.md` (Simulation, M7a:
Gattling spin-up) and cited as "Q-1151 interim" in the code: while the stage count in force is below
2 the held thresholds are not used (all 0), and the importer warns.
