---
id: Q-0830
title: Is an order-level destination that names a unit removed at the goal cell like a cell or like a building
status: answered
related: [S-0207, S-0215, S-0507]
answered-by: [S-0207]
---

## Need

S-0207 revision 28 claim 26 distinguishes two kinds of order-level destination for driving
and ship vehicles: one that names a cell, which the first locomotion update without a track
removes while the vehicle stands in that cell, and one that names a building, which cell
equality does not remove (only the exact-point guard row does). It says nothing about a
destination that names a unit, such as a transport vehicle a passenger is ordered to enter
(S-0507) or an object followed or approached by a move order. The engine's driving
locomotor (`drive::goal_cleanup`) must decide for such a destination, and the spec gives no
row.

## Questions

1. When a driving or ship vehicle with an order-level destination that names a unit (a
   vehicle, an infantry unit or an aircraft) stands without a track in the cell of that
   destination's supplied movement point, does the first locomotion update remove the
   destination (as for a cell) or keep it (as for a building)?
2. Does the answer differ for a destination naming a unit that is itself moving, as against
   a stationary one?
3. Does the exact-point guard row (claim 26, third row) apply to such a destination in the
   same way as to a building?

## Possible observation

Order a vehicle to enter a stationary transport, then a moving one, parked in the cell of
the supplied point; record the order-level destination on every locomotion update, as in
S-0207 D30.

## Current workaround

`drive::goal_cleanup` takes a destination naming a unit as a cell destination: removed by the
first locomotion update without a track while the vehicle stands in the cell of its heading
point (the behaviour before claim 26). Marked in the code and in `docs/development.md` with
this request's ID.
