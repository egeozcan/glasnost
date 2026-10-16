---
id: Q-0871
title: Which placements of a warp member count as failing in S-0513 claim 50
status: answered
related: [S-0513, S-0009, S-0209]
answered-by: [S-0513]
---

## Need

S-0513 claim 50 says a placement failure never deletes a warp member: it stays alive in the landing-retry stage with its landing point, owner and passengers kept, and retries on the next update. It does not say which placements fail. The check of claim 37 passes before placement is scheduled, and placement itself happens one update later.

The code is `warp::place`. It treats exactly one case as a failure: a landing point whose cell is not a legal cell (S-0009 claim 1). A cell that a building, a terrain object or a protected object enters between the check and the placement update is not treated as a failure: the member is placed, and an unprotected unit sharing the cell stays as it is.

## Questions

1. Besides a point outside the map, is there any case in which the placement update refuses a member whose landing check passed one update earlier (a blocker that appeared in between, a cell that stopped being enterable, a full cell)?
2. If so, does the member retry from the landing check of claim 37 (blocked then redirected) or only repeat the placement?

## Possible observation

Arrange a landing cell that is free at the check and receives a building or a unit in the one update before placement (S-0513 SW24 part c, engine contract test), and record the member's stage and cell on each update.

## Current workaround

Only a landing point outside the map fails placement; the member returns to the landing-retry stage, which repeats the landing check of claim 37 and its redirect search. Marked `Q-0871 interim` in `docs/development.md`.
