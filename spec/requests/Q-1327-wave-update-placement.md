---
id: Q-1327
title: When in the tick a magnetic wave updates, and its impact point
status: answered
related: [S-0607, S-0313, S-0306]
answered-by: [S-0607]
---

## Need
S-0607 F4 gives one update at creation "as well as the subsequent per-tick updates". The engine needs their place in the tick order, because the end test reads the firer's current target. F3 also needs the shot's impact point.

## Questions
1. Do a wave's per-tick updates run in the firer's own turn, or in a separate pass over all waves? Before or after behaviour processing in that tick?
2. Is the F3 impact point the projectile's target point when it is fired (S-0306 claim 1), even for a target that moves while the projectile flies?

## Current workaround
Each living firer updates its wave right after its behaviour processing, in its own object turn. The impact point is the projectile's stored target point at firing. If the projectile is already gone in the firing update, the current target position is used.
