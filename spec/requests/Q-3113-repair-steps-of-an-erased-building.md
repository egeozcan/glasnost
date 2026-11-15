---
id: Q-3113
title: Whether a building being erased takes the paid steps of a repair already on
status: answered
related: [S-0314, S-0514]
answered-by: [S-0314]
---

## Need

S-0314 claim 12 says that buildings being erased "do not reach this work",
and the claim's first sentence speaks of a building turn "that reaches repair
work". It also says that the credit split "does not switch off an existing
repair: claim 8 still controls its paid steps". Scenario H13 f turns repair
off before the erasure lock, so it does not show what an erased building does
with a repair that is already on.

The engine now skips claim 12's automatic action for a building being erased
(sim batch S13, issue 256). The code is `econ::sell::building_turn`, which
then takes claim 8's step at each multiple of the building repair period.

## Questions

1. A damaged building whose repair is on (switched on by order, or started
   automatically before the lock) is locked by an erasing unit. While it is
   being erased, does it take claim 8's paid repair steps (credits spent,
   health gained) at the repair period, or are those steps skipped too?
2. If they are skipped, does its repair stay on, so that the steps resume on
   the first repair-period tick after the release?

## Possible observation

H13 f with repair left on: a funded run, damage the building, switch repair
on, let a hostile erasing unit lock it for several repair periods, and record
the house's credits and the building's health at each period tick until the
attacker is removed.

## Current workaround

The automatic action is skipped while the building is being erased; a repair
already on keeps taking claim 8's paid steps, as before the fix.
