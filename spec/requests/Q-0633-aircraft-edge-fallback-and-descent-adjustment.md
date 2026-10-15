---
id: Q-0633
title: Where does an aircraft go when its push-back at the map edge fails, and how does descent adjust its position?
status: answered
related: [S-0501]
answered-by: [S-0501]
---

## Need

S-0501 claim 4 and Open question 6: after the 128-lepton push-back toward the map's middle, the
aircraft tries "a nearby fallback position" that was not decoded; descent toward a destination
cell also adjusts the horizontal position in an undecoded way. The gap is in `glasnost_sim::air`
(the horizontal step).

## Questions

1. Which fallback position is tried after the push-back, and with which draws, if any?
2. While descending toward a destination cell, how is the horizontal position adjusted each tick?

## Possible observation

S-0501 scenario AC3's edge-flight variant, and AC4 landings from different approach angles,
logging positions every tick.

## Current workaround

No fallback position (the aircraft does not move in that tick when the push-back fails) and no
descent adjustment (marked Q-0633 in `glasnost_sim::air` and in docs/development.md, M6a
interim rules).
