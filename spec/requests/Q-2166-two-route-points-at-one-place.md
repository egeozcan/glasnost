---
id: Q-2166
title: Which of two route points at one place a route-mode click selects, joins or closes a loop at
status: answered
related: [S-0213]
answered-by: [S-0213]
---

## Need
S-0213 revision 60 (claim 16a) makes point adds share a point only when
they are applied in the same tick, so two points of one house can now
stand at one place (Scenario O8 part s). A route-mode click on that place
must then pick one of them: for the point-click test and the join or loop
close of claim 16b, for the point selection of claim 16e, and for the
delete key. S-0213's Open questions name this as not studied; the click
area of a marker is Q-2138 question 3. Code:
`crates/deterministic/sim/src/routes.rs` (`point_in`, `point_for`).

## Questions
1. With objects selected, a route-mode click on a place where two points
   of the house stand, one held by a selected object's route and one held
   only by another object's route: which point does the click test and
   join or close a loop at? Does the answer change when the selected
   object holds both, or neither?
2. With nothing selected, which of the two points does the click select
   (claim 16e), and so which one does the delete key then remove?
3. Are the two markers drawn one over the other, and does the drawn one
   decide the click?

## Possible observation
Scenario O8 part s (1) gives two points at the enemy building's place, one
in V's route and one in W's. Select W alone, hold the key and click the
marker: record whether W's route line closes a loop (W's own point) or the
click is refused with the invalid-point text (V's point). Then with
nothing selected click the marker, press delete and see whose route line
loses the place; repeat with the two routes made in the other order.

## Current workaround
Our design: a click with objects selected takes the lowest numbered point
in the cell that a selected object's route holds, else the lowest numbered
point in the cell; with nothing selected, the lowest numbered point in the
cell (the oldest). Tested in `claims_s0213_routes.rs`
(`two_points_at_one_place_a_click_takes_one_the_selection_holds`).
