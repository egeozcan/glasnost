---
id: Q-0075
title: The precision of the red ratio and S-0302's rounding example
status: answered
related: [S-0302, S-0003]
answered-by: [S-0003]
---

## Need
S-0302 F5 compares the integers H and H − m′ exactly with
X = RZ₅₃(S × ρ), ρ the red ratio "(double)", read from `ConditionRed` as
"decimal with optional `%` (S-0003 row C9)". S-0003 claim 6 says such a
decimal is read as a **single** (0.6 gives the single just above 0.6, about
0.6000000238), which is then used as a double. S-0302's paragraph
*Dependence on the rounding mode* instead says that a red ratio of 0.6 "is
held as the nearest double, which can lie just below the decimal", gives
X = 599.99999999999988 for S = 1000 with `quirks.chop_rounding` on, "so a hit
that ends at 600 already crosses the red threshold".

The two readings give different thresholds (just above 600 against just
below 600), and under F5 the paragraph's conclusion holds only for the first:
with X just below 600, a hit ending at 600 does not cross it (600 < X is
false), while a hit from 600 to 599 does. Our engine reads the ratio as
S-0003 claim 6 says (`rules::GeneralRules::condition_red`) and applies F5 as
written; the claim test `claims_s0302::the_red_threshold_is_compared_exactly`
checks F5 against that reading.

The same question applies to the yellow ratio (`ConditionYellow`, claim 13
and F5 with `quirks.yellow_state_at_half` off) and to any other row C9 field
that a damage formula multiplies by.

## Questions
1. Is the red ratio that S-0302 F5 multiplies by the single that S-0003
   claim 6 reads from `ConditionRed` (widened exactly), or the double nearest
   the written decimal?
2. With `ConditionRed=0.6` and a strength of 1000, which hit first reports
   *now red*: one that ends at 600 (601 → 600), or one that ends at 599
   (600 → 599)?
3. If the ratio is the single, should S-0302's rounding paragraph be revised
   to say so (its example value and its "ends at 600" sentence)?

## Possible observation
A test vehicle type of strength 1000 and `ConditionRed=0.6`, damaged by a
weapon of damage 1 against a versus value of 100 %, starting at health 601:
watch for the change of the health bar's colour or of the damage sound
(S-0302 claim 12) at the hit to 600 against the hit to 599.

## Current workaround
None needed in code: the engine reads the ratio by S-0003 claim 6 and
applies F5 exactly. Only the spec text is in question.
