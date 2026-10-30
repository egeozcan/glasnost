---
id: Q-2136
title: Route points that no route holds, reused committed points, and the holder that gives up and waits
status: open
related: [S-0213, S-0215]
answered-by: []
---

## Need
S-0213 claim 16f says a point that no route holds any more is deleted when
its holder is destroyed and removed from play. Points also stop being held
in other ways: an order ends a route (claim 17), a route without a loop is
used up (claim 16c), and a point delete empties a route (claim 16e). The
engine has to decide whether a point that loses its last holder in those
ways stays on the map (drawn while the route-mode key is held, clickable,
joinable) or goes. The same section leaves two neighbouring cases open.
Code: `crates/deterministic/sim/src/routes.rs` (`drop_orphans`,
`apply_point`, `apply_commit`).

## Questions
1. After a holder's route is ended by an order outside route mode, used up
   at its last point, or emptied by deleting its last remaining point,
   does a route point that no other route holds stay on the map (its marker
   drawn while the key is held, a click on it selecting it with nothing
   selected, a join possible)? Or is it deleted as on destruction?
2. A point add that reuses an existing point of the house (claim 16a) when
   that point is already committed: is the holder that reaches the point
   before the next commit treated as at a committed point (it issues it)
   or at an uncommitted one (it waits for a commit)? Claim 16c says this
   has not been studied.
3. Claim 16c: a holder whose current behaviour is guard or a continual
   action "gives up that behaviour and waits" at the commit, then issues
   its next point. Does "gives up and waits" change anything observable
   before the issued order takes effect at the next order boundary (for
   example, does an area-guarding unit stop firing in that tick)?

## Possible observation
Scenario O8 style: two units sharing a point, one ordered away with an
ordinary move outside route mode; hold the key and look for the other
route's points and the first unit's private points. For question 2, add a
point at a committed point's place to a second unit's route without
releasing the key, and watch whether that unit moves there.

## Current workaround
Question 1: a point no route holds is deleted after any end of holding, not
only destruction (`// design (Q-2136)` in `drop_orphans`). Question 2: the
point keeps its committed mark, so the reused point is followed. Question 3:
the holder issues its next point at the commit and changes nothing else; the
order takes effect at the next order boundary.
