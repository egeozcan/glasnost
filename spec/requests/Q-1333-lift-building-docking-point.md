---
id: Q-1333
title: Which buildings give their docking point as a lifter's point, and which point
status: answered
related: [S-0607, S-0314, S-0606]
answered-by: [S-0607]
---

## Need
S-0607 claim 7 fixes a gripped unit's movement point from "the lifter's point" at the seize. For a building lifter that is the centre of its foundation, but "a building of a type that services docking units gives its docking point instead". The engine must decide which building types that phrase covers and, where a docking point depends on the docking unit or on a free place (a repair depot's pads, a bunker's quadrant point), whose point it is. The gap is in `lift::lifter_point` (crates/deterministic/sim/src/lift.rs).

## Questions
1. Which building types give their docking point: repair depots, bunkers, refineries, airfields and helipads, others? Is it decided by a type flag the rules can set?
2. For a building with several docking places, which place's point is used when the seized unit is not docked there: the first place, the first free place, or the place the seized unit would be given?
3. Is the docking point taken at the building's height, or at the surface height of its cell?

## Possible observation
Named test data giving a repair-depot type and a refinery type a seizing weapon; seize a vehicle with each and record the cell of the fixed movement point (scenario L2's recorder) against the building's cells and pads.

## Current workaround
A repair depot and a bunker give the docking point they would give the seized unit itself (S-0314 claim 18, S-0606 claim 4). Every other building gives the centre of its foundation.
