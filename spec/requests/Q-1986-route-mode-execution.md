---
id: Q-1986
title: How does a committed route run, and which orders build and delete it?
status: open
related: [S-0213, S-0215, S-0214, S-0701]
answered-by: []
---

## Need

Play testing (M8 A4 round 5, note 29) found no way to give waypoints. S-0213 claim 16 describes
route mode with the Z key held (claim 22): clicks add route points to each selected object's
planned route, release commits it, and "Route points are created, committed and deleted by their
own order kinds". Claim 17 ends a route on any other order, and S-0215 claim 13's route row issues
the next route point's action as an order at a fall-back to rest. The guard command adds a guard
action to planned routes while route mode is held (S-0213, key commands).

The spec's own confidence for route execution is low, and its Open questions leave "how a route
point's action is carried out, looping routes, and deleting route points; and which selected
objects cannot take part in a route" unsettled. The engine has no route orders and no
route-holder state yet. Building them needs the order kinds' contents and the route's lifecycle,
because they are simulation state and go through the lockstep order stream.

## Questions

1. Which order kinds does route mode send, and what does each name: the actors, the cell or
   object of a point, the point's action, the route it belongs to? Is one route shared by the
   whole selection or does each selected object get its own copy?
2. Which actions can a route point stand for: only *move*, or also the click's ordinary
   action of S-0214 (attack on an enemy, enter, guard, and so on)? What is the point's action
   when the click is over an object rather than a cell?
3. When the key is released, does the first object start at the first point at once, or at its
   next fall-back to rest? What does a route do after its last point: stop, keep the last
   action, or loop back to the first point? Is looping an input (clicking the first point
   again) or automatic?
4. How are route points deleted (which input, which order kind), and can a committed route be
   extended by holding the key again with the same selection?
5. Which selected objects cannot take part (S-0213 claim 16's refusal): buildings, aircraft,
   harvesters, units in transports, objects of another house?
6. What happens to a route when one of its holders is destroyed or leaves the map, and does
   that holder stop counting toward the 128 at once?
7. What does the player see: route point markers or lines between them, their look, and who
   sees them (only the owner, or allies too)?

## Possible observation

S-0213 scenario O8 already sets up route mode. Extend it: (a) route a vehicle through three
cells, with the second click over an enemy unit, and watch what it does there; (b) click the
first point again before releasing and watch whether the route repeats; (c) route two vehicles
at once and compare their paths; (d) select a building or an aircraft with the vehicle and
try a route-mode click.

## Current workaround

None in the simulation: Z is not bound and no route order exists. The client offers rally
points for factories (S-0213 claim 20), which already work, and does not build a queued-orders
substitute, because that would put orders into the lockstep stream that no spec describes.
