---
id: Q-0224
title: Do recorded upgrades count in power when the building's upgrade count is 0?
status: open
related: [S-0405, S-0408, S-0406]
answered-by: []
---

## Need

S-0408 claim 5 (revision 6): selling a turret-flag upgrade resets the building's upgrade count
to 0 and clears only the sold position, so lower positions keep their upgrades. It adds: "Their
power no longer counts at count 0 (the count gate of S-0405 claims 1 and 2). This revision
requires those claims to state that gate explicitly at hand-off."

S-0405 (revision 4) does not state such a gate. Its Terms say that the upgrades a building holds
are those recorded in its three positions, and F1 and F2 sum the base output and drain of the
upgrades in the three positions, with no condition on the count.

The two specs therefore give different power for a building whose count is 0 while a position
still holds an upgrade.

## Questions

1. When a building's upgrade count is 0 and one or more of its positions still hold upgrades,
   do those upgrades' base output and base drain count in its power?
2. When the count is positive, do all recorded positions count, or only positions 1 to the count?

## Possible observation

S-0408 scenario S4 with two upgrades, the upper one with the turret flag: sell once, then read
the house's output and drain.

## Current workaround

S-0408's statement: no recorded upgrade counts in power while the count is 0; with a positive
count every recorded position counts (S-0405 F1, F2). Interim row in `docs/development.md`
(Q-0224).
