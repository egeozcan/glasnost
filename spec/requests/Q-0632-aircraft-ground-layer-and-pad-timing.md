---
id: Q-0632
title: When is an aircraft in the ground layer, how high is it placed on the ground, and when does a pad's first service step come?
status: open
related: [S-0501, S-0404]
answered-by: []
---

## Need

S-0501's on-the-ground test and claim 7 use the ground render layer (the F7 landing test runs on
changing into it), but no height for that change is given. Claim 1 places a non-visiting
aircraft "on the ground", while S-0404 claim 15 gives a docked aircraft the pad position,
including the height of the docking displacement. Claim 26 says that the first service step
depends on when servicing begins. The gaps are in `glasnost_sim::air` (`on_ground`, `placed`,
`report_entered`).

## Questions

1. At which height above the terrain does a descending aircraft change into the ground layer, and
   a climbing one leave it?
2. At which height is a non-visiting aircraft placed when its requested position is above the
   terrain (a carrier launch 10 leptons above the firing point, a pad position with a height
   displacement)?
3. After an aircraft reports that it has entered a pad that is not servicing, on which tick is
   the first service step made?

## Possible observation

S-0501 scenarios AC1, AC4 and AC15: log height and layer each tick around touchdown and take-off;
log the placement height of a carrier launch and of a produced aircraft on a pad with a height
displacement; log the touchdown tick and the first service tick.

## Current workaround

An aircraft is in the ground layer from its ground placement or touchdown (height 0 or below)
until it takes off; ground placement puts it at the terrain height, except that a docked
aircraft stands at its S-0404 claim 15 pad position; the first service step is made at the
pad's first turn at or after the report (marked Q-0632 in `glasnost_sim::air` and in
docs/development.md, M6a interim rules).
