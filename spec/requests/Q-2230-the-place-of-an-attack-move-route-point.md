---
id: Q-2230
title: The place of a route point added by an attack-move click
status: answered
related: [S-0213, S-0214]
answered-by: [S-0213]
---

## Need
S-0213 r84 claim 16b: a point stands at the order's place. A *move* order
names no target, and its place is the clicked cell's (or, for a click on an
object, that of the object's cell), whatever its destination. Any other
order that names no target has its destination's place.

An attack-move click (S-0213 claim 6, the attack-move key held) gives an
order with the *move* behaviour marked as an attack move, a destination and
no target. The claim does not say whether such an order counts as a *move*
for its place (the clicked cell) or as another order (its destination,
which the nearby-cell search may have moved).

## Questions
1. In route mode with the attack-move key held, a click on a water cell next
   to the shore with a ground vehicle selected: is the marker on the water
   cell (the clicked cell) or on the land cell the search finds (the
   destination)?
2. The same for a click on an object that gives an attack-move order with a
   destination: is the marker on the object's cell or on the destination?

## Possible observation
S-0213 Scenario O8 part q (5) with the attack-move key held as well.

## Current workaround
`routes::order_place` treats an attack-move order as not a *move*: its place
is its destination's (marked `design (Q-2230)` in
`crates/deterministic/sim/src/routes.rs`).
