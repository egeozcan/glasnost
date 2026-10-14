---
id: Q-0341
title: What automatic building repair and sale do when the skill-level gate passes
status: answered
related: [S-0314, S-0408, S-0412]
answered-by: [S-0314]
---

## Need

S-0314 revision 7, claim 8, says that with a repair-and-sell threshold of 0 or below the
skill-level gate passes for every level-0 house (human-controlled houses, AI houses, the
neutral house and the special house), and that passing it is necessary, not sufficient.
It withdraws the earlier proposed departure (never repair or sell by itself, with an importer
warning) and says the gate is "not a licence to disable the rule", but it specifies no
complete automatic-start rule for that case: the further conditions (available credits
against a reserve, random choices for selling, per-building states) are listed as open.

Our engine has no automatic building repair or sale at all (`combat::heal` has the building
repair step only, which the repair order starts). With the threshold at 0 or below it
therefore never repairs or sells by itself, which is the withdrawn departure in effect.

## Questions

1. With the threshold at 0 or below, under which conditions does a house's damaged building
   start repairing by itself? Does it depend on the house's available credits, and against
   which reserve?
2. Under which conditions is a building sold by itself instead? Is a random draw taken, from
   which stream, how many and at which moment of the tick?
3. How often is a building checked (each of its turns, a period, or on damage)?
4. Do the checks differ between human-controlled, neutral and special houses?

## Possible observation

S-0314 scenario H9 runs 2 and 4 (threshold 0, then −1), with buildings at several health
levels and houses at several credit balances, recording the tick of each repair start or
sale and the scenario random stream around it.

## Current workaround

No automatic repair or sale for any house at any threshold; only the repair and sell orders
repair or sell. Documented in docs/development.md, "Provisional rules of combat".
