---
id: Q-0636
title: Which held height does a holding aircraft use near a destination on the ground, and how does it steer early in take-off?
status: open
related: [S-0501]
answered-by: []
---

## Need

S-0501 claim 6 marks an aircraft as holding height while, among other cases, it has a target and
ammunition left, and says that within 768 leptons of the destination "guidance instead uses
destination height minus terrain height there when the holding-height mark is set". Read
literally, an attacker whose destination is a target on the ground would then hold a height of
0 and fly at ground level for the end of its approach, which no other claim describes. Claim 8
gives the take-off heading above trunc(*H* / 2) only. The gaps are in `glasnost_sim::air` (the
held height and the take-off steering).

## Questions

1. Does the destination-height rule of claim 6 apply to every holding aircraft within 768
   leptons, or only to one whose destination point is itself more than 120 leptons above the
   terrain? What held height does an attacker with a ground target keep near it?
2. During take-off, below trunc(*H* / 2), does the requested heading stay at the current
   heading, or does it aim toward the destination?

## Possible observation

S-0501 scenarios AC4 and AC7: log held height and height each tick while a strafer and an
ordinary attacker approach a ground target from far away, and the heading during take-off.

## Current workaround

The destination-height rule applies only when the destination point is more than 120 leptons
above the terrain there; otherwise the held height stays the cruise height. Below trunc(*H* / 2)
the requested heading stays the current heading (marked Q-0636 in `glasnost_sim::air` and in
docs/development.md, M6a interim rules).
