---
id: Q-0133
title: The facing a building gets when it is placed or deployed
status: answered
related: [S-0406, S-0415, S-0204]
answered-by: [S-0406]
---

## Need

S-0406 claim 1 and S-0415 claim 6 give the position of a placed or deployed building but not its
facing. A building's facing matters where another spec reads it: S-0408 claim 8 gives absorbed
passengers released on a sale "each facing the building's facing", and a building with a turret
starts from some facing.

The engine needs this in `econ::placement::put_on_map_facing`.

## Questions

1. Which facing does a building have when a place order puts it on the map?
2. Which facing does a building have when a vehicle deploys into it: facing 0, the deploy facing
   *D* × 256, or the vehicle's facing?

## Possible observation

Deploy a vehicle into an absorbing test building that holds infantry, sell it, and record the
released infantry's facing; repeat for a placed copy of the same type.

## Current workaround

A placed building faces 0; a deployed building faces the deploy facing *D* × 256, the facing the
vehicle had when it deployed. Listed in `docs/development.md`'s provisional table.
