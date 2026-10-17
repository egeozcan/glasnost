---
id: Q-1241
title: Order of drift, damage, stage and lifetime within one gas particle's update
status: answered
related: [S-0613, S-0202]
answered-by: [S-0613]
---

## Need

S-0613 claim 6 orders particles within a gas system's turn (newest first, all updates before any
movement), and claim 15 puts the particle stage step after the damage step. It does not say
where the even-tick drift draws (claim 8), the vertical speed (claim 9) and the deflection
(claim 10) come relative to the damage step (claim 13) inside one particle's update, nor whether
the lifetime decrement of claim 6 comes after all of them.

This order is observable: a damaging hit can kill a unit, and the death takes draws from the
scenario stream (death sequences and corpses of S-0107 claim 17, virus clouds of S-0613 claim 3).
Whether the particle's own drift draws come before or after those death draws changes every
later draw.

The code is `crates/deterministic/sim/src/gas.rs` (`update_gas`).

## Questions

1. Within one gas particle's update on an even tick, do its drift draws (claim 8) come before or
   after the hits of its damage step (claim 13), and so before or after the draws of any death
   those hits cause?
2. Do the vertical speed and deflection of claims 9 and 10 come before or after the damage step?
   (They take no draws but read the particle's position and the terrain.)
3. Is the lifetime decrement of claim 6 the last step of the update, after the stage step of
   claim 15?

## Possible observation

G6 with a virus cloud that kills a second unit on an even tick: record the random state before and
after the particle's update and the corpse pick of the dying unit.

## Current workaround

The order follows the claim numbers: drift, vertical speed and deflection (even ticks), then
the damage step, then the particle stage step, then the lifetime decrement. Docs:
docs/development.md, *Simulation (M7a: gas clouds)*.
