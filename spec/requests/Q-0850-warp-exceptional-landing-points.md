---
id: Q-0850
title: The landing point of a warp member whose destination cell is off the map or whose nearby search fails
status: open
related: [S-0513, S-0212, S-0507]
answered-by: []
---

## Need

S-0513 claim 49 (revision 11) says two things about the exceptional warp cells and marks the rest unmeasured.

- When the destination cell that corresponds to a source cell is off the map, the object in that source cell is not refused: claim 34's parasite release and claim 35's organic death still occur, and an otherwise eligible survivor joins the warp. Its eventual position, whether it is placed, and whether it lives are marked unmeasured.
- When a blocked member's nearby-cell search (claim 37, S-0212 claim 25) finds no cell, the failed result replaces the landing point without a success test, the unsuccessful check stays in the retry stage, and a retry sets the held delay. The replacement world point and the later fate are marked unmeasured.

The engine cannot leave a member without a landing point: the warp stages (claim 39) need one to run the landing check and the placement. The code is `examine`, `join` and `redirect` in `crates/deterministic/sim/src/superweapons/warp.rs`.

## Questions

1. For a surviving member whose destination cell is off the map, which world point does its landing take (claim 36 gives the point for an on-map cell only)? Is it placed, and does it live?
2. When the nearby-cell search finds no cell, which world point replaces the landing point? Is it the blank stand-in that S-0212 claim 25 says other callers use (map cell (0, 0)), converted as claim 36 converts a cell, or another point? What happens to the member when the checks at that point pass, and when they fail?
3. May both cases share one answer, so that the engine has a single rule for "no usable destination cell"?

## Possible observation

S-0513 scenarios SW24 parts (a) and (b), recorded with positions and fates every tick after the departure (the spec already proposes them).

## Current workaround

Both cases use the blank stand-in cell (0, 0) as the destination cell and then follow the ordinary rules: the member keeps its position within the cell if it is infantry or an aircraft, lands at the cell centre if it is a vehicle, runs the landing check there, is redirected if blocked, and meets the arrival fate test (claim 38). A note is added to the world's notes when this happens. The rule is marked `Q-0850 interim` in the code and in `docs/development.md`. It is a placeholder for the unmeasured answer, not a claim about the original.
