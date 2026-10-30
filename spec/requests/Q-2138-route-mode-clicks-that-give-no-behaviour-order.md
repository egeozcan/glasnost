---
id: Q-2138
title: Route-mode clicks that give no behaviour order, patrol over a marker, the marker's hit area and the kind refusal
status: open
related: [S-0213, S-0214]
answered-by: []
---

## Need
S-0213 claim 16b turns a route-mode click into the behaviour order the
same click would give outside route mode, marked as a route point. Some
clicks give other orders or no order at all, and the claims leave them
open. Code: `crates/deterministic/sim/src/routes.rs` (`click`, `point_in`).

## Questions
1. A route-mode click whose ordinary action gives an order that is not a
   behaviour order, a detonate order (claim 11) among them: is it refused
   (with which message), does it give its ordinary order at once, or does
   it add a point? Claim 16b says this has not been studied.
2. Patrol (S-0214 row GC3, force-move and force-fire held over a cell that
   holds a route point): how does it meet the join and loop-close test of
   *Clicks on existing points*? (Open questions of S-0213.)
3. What area of the screen counts as a click on a route point's marker: the
   marker's cell, the marker's drawn shape, or another area? When two
   points' markers overlap, which one does the click hit?
4. Claim 16d refuses a route-mode click when a selected object is an
   aircraft or a building. Does that refusal also apply to a click whose
   ordinary action would give no order at all (a click on empty ground
   with only a building selected, which outside route mode deselects), or
   to a click that selects another object? Is the selecting click in
   route mode the ordinary selecting click?

## Possible observation
Scenario O8 setups: a selected engineer-like unit carrying a planted bomb
in route mode clicking itself (detonate); force-move plus force-fire over
an existing point; clicks just outside a marker's cell; a lone selected
building and a click on bare ground.

## Current workaround
Question 1: a click's non-behaviour orders add no point and give nothing
(`design (Q-2138)` in `click`); the deploy refusal of claim 16b stands.
Question 2: patrol is treated as any click on the marker (the point-click
test, then joins or loop closes). Question 3: a click hits a point when it
is in the cell the marker stands in, the lowest numbered point when several
are (`design` in `point_in`). Question 4: a click whose ordinary action
gives no order and is not on a marker is the ordinary click (selection
changes, no refusal); the kind refusal is made only when the click would
give orders or is on a marker.
