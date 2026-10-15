---
id: Q-0704
title: Which cells count as a tunnel's edge for unloading
status: answered
related: [S-0507, S-0214]
answered-by: [S-0507]
---

## Need

S-0507 claim 7 refuses an unload order to a vehicle transport standing "on a tunnel cell or at a
tunnel's edge", and the spec's Open questions say not to infer that every neighbour of tunnel
land qualifies.

The code in `crates/deterministic/sim/src/transport.rs` (`may_unload`) needs the rule.

## Questions

1. Which cells that are not of the tunnel land type count as a tunnel's edge?

## Possible observation

S-0507 scenario T7 near a tunnel entrance: give the deploy order to a loaded transport on each
cell around it.

## Current workaround

Only cells of the tunnel land type refuse; no edge cell does. The choice is marked in the code
with this request's ID.
