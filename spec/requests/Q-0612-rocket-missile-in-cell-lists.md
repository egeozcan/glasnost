---
id: Q-0612
title: Is a rocket missile listed in the cell it is over?
status: open
related: [S-0506, S-0303, S-0209, S-0309]
answered-by: []
---

## Need

S-0506 claim 7 places a launched missile on the map at the fire point, and claim 15 says it is
not a tracked flying object before the end of tilt or raise. S-0303 claim 6 asks (Open questions)
whether a tracked flying object can also appear in a scanned cell. No spec says whether a rocket
missile is in the object list of the cell it is over, or whether it marks the occupancy record of
S-0209 claim 1 (an aircraft on the ground sets the object-present flag). Our engine has to decide
in `glasnost_sim::rocket::place` and in the cell updates of each rocket step.

## Questions

1. Is a rocket missile in pause, tilt or raise in the object list of its cell, so that the cell
   scan of an area detonation (S-0303 claim 5) can hit it?
2. Once it flies, does it move between cell lists as it crosses cells, in addition to its
   flying-object registration?
3. Does a rocket missile set the object-present occupancy flag of its cell while it stands on the
   launcher's fire point?

## Possible observation

A low-spread detonation on the launcher's cell while a missile stands in pause or tilt, and again
during cruise at an altitude where the flying-object search would miss it (a ground detonation):
record whether the missile takes damage. For question 3, a vehicle ordered through the launcher's
cell while a missile stands there.

## Current workaround

A rocket missile is in no cell list and marks no occupancy record; area damage reaches it only
through the flying-object search, once it is tracked (marked Q-0612 in `glasnost_sim::rocket` and
in docs/development.md, M6a interim rules).
