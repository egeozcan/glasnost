---
id: Q-3263
title: A hovering vehicle's repeated assignment and the point of an object destination
status: open
related: [S-0503, S-0502, S-0314, S-0606]
answered-by: []
---

## Need

S-0503 claim 20 (r29) gives a hovering vehicle's *D* for a destination that names an object: the
object's movement point at the assignment, with the height of Terms. It also says that a vehicle
ignores an unchanged order-level destination unless its destination-refresh state is on (S-0502
claims 24 and 27), and the ignored assignment leaves *D* as it was. Four cases are not settled.

- **After a stop request.** Claim 10's stop request clears *D* and leaves the order-level
  destination in place, as at a repair depot's switch-off. Our engine had a rule that a
  destination the locomotor takes gives it a *D* again. When the same object is assigned again
  after the stop request, the order-level destination is unchanged.
- **Cell destinations.** Claim 20 states the ignored assignment for object destinations. Claim 18
  says that an assignment after hold expiry applies the ordinary height adjustment anew, "even for
  the same cell". That reads as a same-cell assignment that is not ignored.
- **The height.** Terms gives *D*'s height as "the surface height of its cell", plus 416 leptons
  on a high-bridge cell. For a cell destination the point is the cell's centre. An object's
  movement point is usually not the centre, and on a slope the surface height differs within a
  cell (S-0106 F1).
- **Depot and bunker contact.** S-0314 claim 18 and S-0606 claim 4 give a vehicle in contact the
  docking point as its destination point. Our engine takes contact in the same order as the
  assignment, just after it, and the driving locomotor takes the docking point then.

## Questions

1. A hovering vehicle whose *D* claim 10's stop request has cleared, with the order-level
   destination left as an object, is given that same object again with no refresh state. Is the
   assignment ignored, so that it has no *D*? Or does it take the object's movement point as a new
   *D*?
2. Is an unchanged cell destination ignored in the same way as an unchanged object destination?
   If it is, what does claim 18's "even for the same cell" refer to?
3. For an object destination, is *D*'s height the surface height at the movement point itself, or
   at the centre of the cell the point is in?
4. When the vehicle takes depot or bunker contact in the same order as the assignment, is *D* the
   docking point? Or is it the building's movement point at the assignment, before the contact?

## Possible observation

Question 1: S-0314 scenario H4's hovering repeat, with the depot's switch-off and then an enter
order to the same depot before the vehicle is down; record *D* (the vehicle's movement) and its
order-level destination each tick. Question 3: a synthetic infantry unit standing off-centre on a
slope cell, a hovering vehicle sent to it by an area guard order; record the vehicle's height
when it stops. Questions 2 and 4 may be answerable from the records already made.

## Current workaround

1. The assignment is not ignored after a stop request: the vehicle takes the object's movement
   point as a new *D*, as before r29.
2. Only object destinations are ignored when unchanged. A cell destination is taken again.
3. The surface height at the movement point itself, plus 416 leptons when the point's cell is a
   high-bridge cell.
4. The docking point: taking the contact in the same order sets *D* to the docking point, as the
   driving locomotor does.
