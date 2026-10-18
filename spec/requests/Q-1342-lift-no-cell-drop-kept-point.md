---
id: Q-1342
title: Does a dropped unit keep its movement point and commitment when the stop finds no cell
status: answered
related: [S-0607, S-0502]
answered-by: [S-0607]
---

## Need
S-0607 claim 13 (r6) and S-0502 claim 17's magnetic-release safety departure (r14) cover a grip ended by its lifter when the stop's search finds no cell. For a set-down unit they say it "keeps no movement point, its jumpjet movement is no longer moving". For a dropped unit they say only that it "falls as claim 15 says". They do not say what happens to the movement point fixed at the seize (claim 7), or to the movement commitment.

This matters for the fall's height. Under claim 13's "How high it is held during the fall" bullet and S-0502 claim 9 and F3 (r14), a crashing unit uses the look-ahead ground reference when its movement point lies in another cell, and the direct reference when it has no point or the point is in its own cell. Take a unit dropped over a cell centre in mid-pull with no cell found. The seize's point usually lies in another cell, near the lifter. If the point is kept, its fall is held towards its own cell's clearance. If it is cleared, the fall uses bare terrain. Off the cell centre, claim 15's detour replaces the point, so only the over-the-centre case is affected. The gap is in `jump::stop_with` (the `lifter_ended` branch), in crates/deterministic/sim/src/jump.rs.

## Questions
1. When the lifter drops a unit and the stop finds no cell, is the movement point fixed at the seize kept, or cleared as it is for a set-down unit?
2. Is the movement commitment kept, or cleared?

## Possible observation
None in the original: it stops with a fatal error in this case (S-0607 claim 13). This is a design choice.

## Current workaround
A dropped unit is treated like a set-down one: with no cell found, the movement point and the commitment are cleared, so its fall uses the direct ground reference. No self-destruction happens in either quirk position. It falls as claim 15 says.
