---
id: Q-3353
title: When a driving vehicle's docking point for a building destination is taken
status: answered
related: [S-0207, S-0208, S-0212, S-0606, S-0607]
answered-by: [S-0207]
---

## Need

S-0208 claim 27 (r82) gives the point that a building supplies to an infantry unit's or a
vehicle's locomotion "at each assignment". For a type with the helipad, unit-repair or
vehicle-bunker flag that point is a docking point for the unit, by S-0607 claim 7's precedence, and
it can depend on the unit: S-0606 claim 4's quadrant point is "taken from the unit's position at
the assignment", and the docking-place rule of S-0314 claim 18 and S-0404 claim 15 gives the place
the unit holds through a contact with that building.

For the walking locomotor S-0208 Terms (*Locomotor destination*, r67) and claim 25 (r68) say the
point is fixed at the assignment, and the walker's searches look for that point's cell. For the
driving locomotor S-0207 claim 25 installs the point when the destination is assigned, but
S-0212's searches (claims 20 to 23), its recomputation of claim 20 (the heading point more than
512 leptons away), S-0207 claim 10's braking and claim 15's give-up test read the heading point or
the move destination (S-0207 Terms), and none of these says whether that is the point installed at
the assignment or the point the building would give the vehicle now.

The two readings agree for every building that gives its centre. They differ for a vehicle-bunker
type when the vehicle has no bunker contact and drives round it to another side, so that S-0606
claim 4's quadrant changes, and for a type with the helipad or unit-repair flag and more than one
docking place when the vehicle gains or loses a contact on the way.

## Questions

1. For a driving vehicle whose destination is a building of a docking type, do S-0212's searches
   and claim 20's recomputation look for the cell of the point installed at the last assignment
   (S-0207 claim 25), or of the point the building would give the vehicle at the time of the
   search?
2. Do S-0207 claim 10's braking and claim 15's give-up test measure to the same point as the
   searches?
3. Does the answer differ for the C7-P5 yield heading, S-0402 claim 8's connection test and S-0402
   claim 26's slave connection, which read the heading point's cell for a unit with a building
   destination?

## Possible observation

A driving, bunker-eligible vehicle sent into a 2 × 2 vehicle bunker from the west, with a wall of
impassable cells that makes its path go round to the bunker's south side. Record the cell each of
its searches looks for (the cell its stored path ends next to) before and after it passes the
quadrant boundary, and the cell it arrives in contact from.

## Current workaround

`path::heading_point` and `drive::supplied_point` give claim 27's point for a building to a walking
or driving unit through `path::object_point`. The driver's searches, its recomputation, braking and
give-up test compute that point again at each reading, from the vehicle's position and contacts at
that time; only the installed goal of S-0207 claim 25 is taken at the assignment. The walker's
point is fixed at the assignment, as S-0208 Terms gives.
