---
id: Q-2137
title: Which holder's route a point join copies, and a loop that starts before the joined point
status: open
related: [S-0213]
answered-by: []
---

## Need
S-0213 claim 16b: a point join gives the joining object a copy of the
route of one of the point's holders, from the joined point to that route's
end, with that route's loop. It leaves open which holder is copied when the
point has several, and what the copy keeps of a loop whose first point lies
before the joined point. The engine has to pick one. Code:
`crates/deterministic/sim/src/routes.rs` (`apply_join`).

## Questions
1. When the joined point lies on the routes of several holders of the
   house whose routes differ after that point, which holder's route does
   the joining object copy? (The one made first, the one given the point
   first, the one nearest the joining object, or another rule.)
2. When the copied route has a loop whose first point lies before the
   joined point, what does the joining object's copy do after its last
   point: does it loop over the whole copy, loop from some other point,
   go on to the loop's first point (which it does not hold), or stop?
3. When the copied holder has already issued some of its points (so its
   route no longer holds the points it passed), does the copy start at the
   joined point even if that point lies inside the holder's loop and the
   holder is past it in the loop?

## Possible observation
Three units: A with route P1 P2 P3, B with route P2 P4 P5 (P2 shared), C
with no route; select C, hold the key and click P2; release and watch C's
path. For question 2, close a loop at P1 for A before C joins P2.

## Current workaround
Question 1: the copy is that of the point's holder of the house with the
lowest object number (creation order), other than the joining object.
Question 2: a loop starting before the joined point loops the whole copy.
Question 3: the copy takes the holder's remaining steps from the joined
point, whatever the holder's position in its loop. All marked
`design (Q-2137)` in `apply_join`.
