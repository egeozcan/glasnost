---
id: Q-0070
title: The draws of a smudge choice when a building is destroyed
status: answered
related: [S-0312]
answered-by: [S-0312]
---

## Need
S-0312 claim 19 (a) and (g) give the draws a destroyed building takes for its
smudges up to the percent draw that picks a scorch mark or a crater, "then the
draws of the smudge choice (terrain and smudge specs)". No approved spec gives
those draws, so every draw after the first smudge of a destroyed building
(the explosion animations of claim 19 (b), the destruction animation of (e),
the survivors of claim 18 and the per-cell smudge steps of (g)) lands on other
values of the scenario random stream than in the original
(`combat::death::building_death`).

## Questions
1. When a destroyed building leaves a scorch mark or a crater (S-0312 claim 19
   (a) and (g)), which draws from the scenario random stream does choosing the
   smudge type take, in which order, and over which ranges? Does the choice
   depend on the theater or on the cell (for example a smudge already there)?
2. Does placing the smudge take any further draw?

## Possible observation
A test building of foundation 2 × 2 with an empty explosion list and no crew,
destroyed with the random-state recorder running: the count of raw values
taken between the claim 19 (a) percent draw and the next known draw.

## Current workaround
The percent draw of claim 19 (a) is taken; the smudge choice takes no draw,
and no smudge is placed (smudges are not drawn yet). The world records a note.
