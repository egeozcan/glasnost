---
id: Q-2927
title: What "may occupy this one" means for an assaulter type that is not an occupier type
status: open
related: [S-0215, S-0309, S-0508]
answered-by: []
---

## Need

S-0215 claim 27 (first infantry rule) and claim 23 (second row of the infantry table) apply to a
computer-controlled infantry unit whose target is a building and whose "type infiltrates, or
occupies or assaults buildings and may occupy this one". S-0309 claim 25 part (e) has a matching
alternative ("may occupy it").

M8 step A6b built "may occupy" as S-0508 claim 1's garrison entry test. That test requires the
unit's type to be an occupier type. For an assaulter type that is not an occupier type it is
therefore always false, so the words "or assaults" add nothing, and such a unit never takes the
building as its destination under these rows. S-0508 claim 14 has its own assault eligibility test
(a garrisonable building that is neither built up nor sold, its centre in the usable area, and the
owner conditions of an assault), which a non-occupier assaulter can pass.

## Questions

1. For an assaulter type that is not an occupier type, is "may occupy this one" S-0508 claim 1's
   test (never true for it), S-0508 claim 14's assault eligibility test, or claim 1's building
   conditions without the unit-type condition?
2. Does the same reading hold for S-0309 claim 25 part (e)?

## Possible observation

S-0215 B11 (b) with a test file that makes an infantry type an assaulter and not an occupier: a
computer-controlled unit of it in *hunt* near an enemy garrisoned building and near a passive
empty one. Whether *capture* becomes current with the building as destination, for each building.

## Current workaround

S-0508 claim 1's test (`garrison::may_enter`), cited in `process::computer_taker_target` and
`acquire::candidate`. No arena row is known to depend on it.
