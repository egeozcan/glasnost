---
id: Q-0132
title: What stops the vehicle of a pack-up from being put on the map
status: answered
related: [S-0415, S-0209]
answered-by: [S-0415]
---

## Need

S-0415 claim 11 says that when the vehicle of a pack-up "cannot be put on the map at its cell (for
example because a building now stands there)", the house receives the sale refund instead. Only
the building case is named. The engine must decide, for the return cell, which contents refuse
the vehicle.

The engine needs this in `econ::deploy::can_stand`.

## Questions

1. Is the vehicle refused when another vehicle (own or enemy) stands in the return cell?
2. When infantry stands in the return cell (one or several standing positions taken)?
3. When the return cell holds an overlay (a wall, ore) or a terrain object, or is a cell the
   vehicle's locomotion cannot enter (for example water for a wheeled vehicle)?
4. When the vehicle is refused, is anything else tried (a neighbouring cell), or is the refund
   the only outcome?

## Possible observation

Pack up a construction yard (redeploy setting on, a guard spot given) while, in turn, an enemy
vehicle, an infantry unit and a wall are put into its return cell by triggers before the
build-down ends; record whether the vehicle appears and the credits.

## Current workaround

The vehicle is refused when the return cell is not usable, or when the ground layer's occupancy
record shows a building or a vehicle claim. Infantry, overlays and land do not refuse. Listed in
`docs/development.md`'s provisional table.
