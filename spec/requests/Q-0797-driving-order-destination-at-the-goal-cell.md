---
id: Q-0797
title: What removes a driving vehicle's order-level destination when it stands in its goal cell
status: open
related: [S-0207, S-0314, S-0215, S-0212]
---

## Need

S-0207 revision 27 claim 25 and S-0314 revision 20 claim 17 (Q-0794 question 3) separate
the locomotor destination point from the order-level destination. An accepted enter order
from rest in the docking point's cell supplies the point. The retained order-level
destination then makes this a moving destination, and merely standing there does not
clear it. S-0314 scenario H19 (f) names "clearing solely at cell equality" as a falsifier.

Our driving locomotor (`drive::arrive_or_rest`, called from `drive::tick`) removes the
order-level destination when the vehicle comes to rest at a track target with no further
step. It also does so when, at its locomotion update, the vehicle stands without a track
in the cell of that destination. Removing the destination clears the point too (claim 25's
"removal of the order destination" row). So after a same-cell enter order, our point and
the moving destination last only until that vehicle's next locomotion update. No spec we
have says which rule removes an order-level destination at the goal: the locomotor, the
move or enter behaviour's processing (S-0215), or the path search (S-0212).

## Questions

1. When a driving or ship vehicle completes its final track in its destination's cell,
   what removes the order-level destination, and when? In the same tick, at its next
   behaviour processing, or never for some behaviours, such as *enter* to a depot?
2. When a driving or ship vehicle with an order-level destination stands without a track
   in that destination's cell (a same-cell move or enter order), does anything remove the
   destination? Or does it stay until a stop, replacement or behaviour effect?
3. For an ordinary move order into the vehicle's own cell, does the vehicle then stay
   with a moving destination indefinitely?

## Possible observation

- Using S-0207 D29 and S-0314 H19 (f) setups, record the order-level destination,
  locomotor destination and behaviour each tick.
  - Case 1: a move order into the vehicle's own cell.
  - Case 2: an accepted same-cell enter order at a displaced docking point.
  - Case 3: an ordinary move ending with track completion in the goal cell.
- In each case record the tick on which the order-level destination disappears, and
  which processing or update ran on that tick.

## Current workaround

At its locomotion update, a driving or ship vehicle at rest in its order-level
destination's goal cell has that destination removed. This happens when a track ends with
no further step, or when it stands there without a track. The locomotor destination
point is cleared with it. A same-cell enter order's point is therefore present for the
enter processing that runs before that tick's locomotion, but not after it. This choice is
marked in the code with this request's ID.
