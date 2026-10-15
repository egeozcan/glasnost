---
id: Q-0670
title: Whether the garrison index reset must precede releases within an eviction or clearing action (S-0313 revision 5 against S-0308, S-0311 and S-0508)
status: answered
related: [S-0313, S-0308, S-0311, S-0508]
answered-by: [S-0313]
---

## Need
The approved copies of S-0308 (revision 9, claim 13 and its occupant-index table), S-0311
(revision 9, claim 15) and S-0508 (revision 4, claim 18) say that an eviction or clearing action
resets the shared occupant/muzzle index to 0 *before any occupant is released or removed*.
S-0313 revision 5 (claim 6) says only that the index is 0 after a completed whole-building
action, states that "no ordering of the reset relative to individual releases or removals within
the action is required", and its Cross-spec changes section asks S-0508 claim 18, S-0308 claim 13
and S-0311 claim 15 to drop the stronger ordering. Those three were revised in the same range
but still carry the ordering.

The engine will need one rule when garrisons (S-0508) are built: whether anything that reads the
index during the action (a shot, a kill credit to the firing occupant under S-0311 claim 8, or a
release order) must already see 0.

## Questions
1. Within an eviction or clearing action, is the index reset required to come before the first
   individual release or removal (S-0308, S-0311, S-0508), or is only the completed-action result
   of 0 required (S-0313 revision 5)?
2. If only the completed result is required, is there any reading of the index during the action
   (for example a kill credited to the building while the action runs) whose answer the specs fix?

## Possible observation
S-0508 scenario G14 and S-0313 scenario P5, with a credited kill or a shot timed inside the
action, if such a timing is reachable.

## Current workaround
Garrisons are not built; nothing reads or resets the index. No code depends on the answer. The
interim rule in docs/development.md (M4 combat table) follows the stronger wording of S-0508
claim 18 until this is settled. Q-0660 question 2 (an independent individual departure or
death) is separate and stays open.
