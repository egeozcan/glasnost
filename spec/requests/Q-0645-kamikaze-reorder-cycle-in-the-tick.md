---
id: Q-0645
title: When in the tick does the shared kamikaze re-order cycle run?
status: answered
related: [S-0506, S-0501, S-0201]
answered-by: [S-0506]
---

## Need

S-0506 claim 12 says the game re-orders every released missile every 30 ticks to attack its
release cell, on one shared cycle, and that a salvo release or reset brings the next update
forward to 2 ticks later. No spec places this cycle in the fixed tick order (S-0201), so the
engine cannot run it without choosing an order itself. The gap is
`glasnost_sim::spawn::kamikaze_orders`, which is written but not called from the tick pipeline.

For a rocket missile the orders change nothing (claim 14 fixes its aim). They matter for a flagged
object outside the rocket types, and for the order of RNG draws or events if the queued *attack*
of S-0501 T4 draws anything.

## Questions

1. In which part of a tick does the shared re-order cycle run: before or after the object
   updates, and before or after the spawner updates that release missiles in that tick?
2. On which tick does the cycle first run in a new game, and does it run at all while no
   kamikaze exists?
3. When a salvo release or reset brings the next update forward to 2 ticks later, is that 2 ticks
   after the release tick, counted the same way as the 30-tick interval?
4. Are the kamikazes re-ordered in a fixed order (for example by creation), and can one
   re-order cycle reach an object released earlier in the same tick?

## Possible observation

S-0506 scenario R7 with a named non-rocket aircraft type carrying the missile-spawn flag: release
it from a spawner and log the ticks at which it receives an attack order on its release cell,
relative to the release tick and to other objects' updates in the same tick.

## Current workaround

The cycle is not run: `spawn::kamikaze_orders` exists but the pipeline never calls it (listed as a
seam in docs/development.md, M6a). Rocket missiles behave as specified without it. A non-rocket
flagged object gets no repeated orders.
