---
id: Q-0220
title: The facing a building gets when a vehicle deploys into it
status: answered
related: [S-0406, S-0415, S-0108]
answered-by: [S-0406, S-0415]
---

## Need

S-0406 claim 16 answers half of Q-0133: a building put on the map by a place order faces 0.
It hands the other half to S-0415: "The facing of a building made by a vehicle deploy is owned
by S-0415, not inferred from the vehicle's turn to its deploy facing." S-0415 (revision 4)
states the deploying vehicle's turn to *D* × 256 (claim 5), the building's position, health
and take-overs (claim 6), and the facing of the vehicle a pack-up makes (claim 11, *D* × 256).
It does not state the facing of the building that claim 6 puts on the map.

The facing matters for a turreted building made by a deploy (its turret's first facing) and for
anything that reads a building's facing, such as passenger release (S-0408 claim 8).

In the code, `econ::deploy` gives the deployed building facing *D* × 256, the facing of the
vehicle at completion.

## Questions

1. Which body facing does a building put on the map by a vehicle deploy (S-0415 claim 6) have:
   0 as a placed building, the deploy facing *D* × 256, the deploying vehicle's facing at
   completion, or another value?
2. For a turreted deployed building, which facing does its turret start with?
3. For a deploy facing outside 0 … 255 (S-0415 Edge cases), is the building's facing reduced
   modulo 65536 like the vehicle's?

## Possible observation

Deploy a vehicle whose deploys-into type is turreted, with a deploy facing other than 0, and
record the building's body and turret facing on its first tick in play.

## Current workaround

The building faces *D* × 256 (interim row in `docs/development.md`, Q-0220).
