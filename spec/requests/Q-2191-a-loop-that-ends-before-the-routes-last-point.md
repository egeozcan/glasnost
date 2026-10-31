---
id: Q-2191
title: How a holder goes round a loop that ends before its route's last point
status: answered
related: [S-0213]
answered-by: [S-0213]
---

## Need
S-0213 r74 claim 16b, *Who gets a press's orders*, gives the join copy,
added after its own points, to an object that is not selected, and to a
selected holder that has lost the pressed point. Sometimes the copied loop
begins at a later point that the object's own points also hold. The loop
then "begins at that own point and has as many points as the copied loop,
so it ends before its route's last point" (low). The claim says that how a
holder goes round such a loop is not studied (Open questions). Claim 16c's
issue rule (issuing a point of a loop moves it to the end of the route)
covers only loops that end at the route's last point.

Our engine now builds this loop (`Route::loop_to` in
`crates/deterministic/sim/src/routes.rs`). It needs an issue rule for the
loop, and a rule for what a later join copies from such a route.

## Questions
1. When a holder whose loop ends before its route's last point issues a
   point of the loop, where does that point go? To the end of the loop
   (just before the first point after the loop), to the route's end, or
   elsewhere?
2. Are the points after the loop ever issued? If so, when: after one round
   of the loop, when the loop's last point is issued, or at some other
   time?
3. When the holder comes back to the loop's first point, does it go round
   only the loop's points, or the loop's points and then the points after
   it?
4. A join copies such a route (claim 16b, *Joins*). With the joined point
   inside the loop, what does the copy hold, and what is its loop? The same
   question with the joined point after the loop.
5. For such a route, does the point-click test (claim 16b) take the
   route's last point, or the loop's last point, as the route's last point?

## Possible observation
Part w (8) of Scenario O8 makes this shape. W has route P1, P2, P3 and no
loop; X loops P2, P3. A press on X's P2 by a selection whose route ends at
P1 then gives W the copy, and W's loop begins at its own P2. Commit and let
W run for several rounds. Record each point it is sent to, in order, and
whether it ever goes to the places P2 and P3 hold in the copy at the end of
its route. Then join a fresh vehicle to W's P3, with W as P3's longest
holder, and record that vehicle's route and loop.

## Current workaround
Our design, marked `// design (Q-2191)` in `routes.rs`:
- An issued point of such a loop is put back at the loop's end, just
  before the first point after the loop. So the holder goes round the
  loop's points only and never issues the points after it.
- A join on such a route, with the joined point after the loop, copies the
  route from the joined point to the end, with no loop. With the joined
  point inside the loop, it copies the loop entered at the joined point,
  then the points after the loop. The copy's loop is the loop's points.
- The point-click test takes the route's last point.

Tested by `claim_16b_a_press_copy_ignores_route_state_both_ways`
(`crates/deterministic/sim/tests/claims_s0213_routes.rs`). With
`quirks.press_copy_ignores_route_state` off, the loop always begins in the
copy, so the case does not arise.
