---
id: Q-2158
title: What is a hunter that makes a fire attempt every tick and no projectile doing?
status: answered
related: [S-0301, S-0310, S-0309, S-0215]
answered-by: [S-0306]
---

## Need
M-0005 records, in three of the seven LTNK against ROBO timeout games (A5, B3 and B5), hunters
that make a fire attempt every tick from some tick between 410 and 549 to the end of the game and
never make a projectile again. Another hunter of B5 does the same for 55 ticks and then makes
projectiles again. Each is in *hunt* with a robot tank as its target. M-0005 does not record why
no projectile is made, or the state of those targets at the stall.

Our engine has no such fire event. A fire attempt in our engine is S-0310 claim 9's firing step:
it asks the firing check (S-0301 claim 9) and either fires or, with an answer that is not *fire*,
does what that answer says (turn, approach, wait for the rearm). We record no attempt that neither
fires nor follows one of those answers. So we cannot tell which of the approved answers, if any,
the stock hunters are in.

## Questions
1. What answer does the firing check give such a hunter in those ticks (for example out of range,
   facing, rearming, or the target not targetable)?
2. Is its target then disabled, at height 0, in its cell, and inside the hunter's weapon range,
   or inside its minimum range?
3. Why does the hunter not move to answer it (turn, approach or back away), since it changes no
   cell after tick 700?
4. What ends it for the B5 hunter that makes projectiles again after 55 ticks?

## Possible observation
M-0005 Procedure step 1, adding for each hunter at each fire event the firing check's answer, its
facing and its turret facing, its target's position and height, and the distance between the two.

## Current workaround
The approved rules, unchanged.
