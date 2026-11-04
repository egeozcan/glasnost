---
id: Q-2418
title: A deployed infantry firer beyond range, and a SAM building short of power, holding a departed warp member
status: answered
related: [S-0513, S-0215, S-0517, S-0309, S-0405]
answered-by: [S-0513]
---

## Need

S-0513 claim 39a (*The firer*, revision 91, Q-2376 answered) says that a firer whose temporal
beam holds a departed warp member does what a firer holding a target on the map at the member's
old position does. It then names two cases in which the firer keeps the member. In each case
another rule that our engine already builds, for a target on the map, drops the target. The
two statements disagree, and we cannot tell which one the spec means to win.

1. *A deployed infantry firer beyond range.* r91: "A deployed infantry unit of a
   human-controlled owner takes no approach step (S-0215 claim 37 g): it keeps the member and
   its beam and neither moves nor turns for the departure." For a deployed infantry unit, S-0215
   claim 23's *attack* processing applies S-0517 claim 16's deployed-firing rule. That rule
   drops an out-of-range target when its range search finds no other target (`process.rs`,
   `deployed_firing`). So on the map, a deployed unit holding a target beyond its weapon's range
   at that position drops it in its next *attack* processing.
2. *A SAM building short of power.* r91: "While that claim's power row applies it keeps any
   member." S-0215 claim 32's power row (the type needs and drains power, the owner is short of
   power) holds the building in its *aiming* stage, so its own rows drop nothing. But the
   building is then not in operation, so its firing check answers S-0405 T1's *impossible*,
   which S-0301 claim 9 ranks above S-0514 claim 19's *rearming*. S-0309 claim 29's periodic
   check therefore drops the target at its next check tick, on the map as for the member.

## Questions

1. A deployed infantry unit of a human-controlled owner, whose temporal beam is attached to a
   warp member, is beyond its weapon's range of the member's old position at the departure.
   While the member waits to land, does the unit keep the member as its target, as r91 says? Or
   does its deployed-firing rule (S-0517 claim 16) drop it in its first *attack* processing, as
   for a target on the map at that position? If the unit keeps it, does the deployed-firing rule
   not apply to a departed member, or not apply to a held temporal target at all (on the map
   too)?
2. A SAM-flag building short of power (S-0215 claim 32's power row), whose temporal beam is
   attached to the member, holds it while it waits to land. Does S-0309 claim 29's check drop
   the member at its next check tick, because the building's firing check answers *impossible*?
   Or does the building keep it until the power row stops applying, as r91's "keeps any member"
   reads? Is the answer the same for a target on the map?

## Possible observation

Question 1: as S-0513 SW12 part l3, with *T* a deploy-to-fire infantry unit deployed beyond its
weapon's range of the tank, its beam locked from in range before it deployed. Record its target
in each tick from the departure to the appearance. Question 2: as SW12 part l7, with *T* a
SAM-flag test building that needs power, its owner's plants sold before the departure. Record
its target in each tick from the departure to the appearance.

## Current workaround

Both cases are built as for a target on the map. Case 1: S-0517 claim 16's drop in the first
*attack* processing. The unit takes no approach step, so it neither moves nor turns. Case 2:
the power row keeps the member in the building's own rows, and claim 29's check drops it at the
next check tick. Tests
`claims_s0513_r89::r91_claim39a_a_deployed_infantry_firer_beyond_range_takes_no_approach_step_and_stays_put`
and `r91_claim39a_a_sam_building_firer_short_of_power_keeps_the_member_in_its_own_rows` cite
these interims. No quirk switch.
