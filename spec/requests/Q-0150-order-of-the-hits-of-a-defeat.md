---
id: Q-0150
title: The order in which a defeated house's objects take their hits, and which objects take one
status: open
related: [S-0414, S-0312, S-0404]
answered-by: []
---

## Need

S-0414 claim 5 says every infantry unit, vehicle, aircraft and building of a defeated house takes
a single hit. It does not say in which order the objects are hit. The hits start death effects
(S-0312) that take draws from the scenario stream, so the order decides which draw each object
gets. The order is observable as soon as two of the objects take death draws.

The claim also says "that the house owns". It does not say whether an object off the map is
hit. Examples are an item still in production (an object in existence, S-0404 claim 11) and a
passenger inside a transport. The Edge cases say production items go through S-0404 claim 8 once
the last factory is gone, which suggests they are not hit, but the claim does not say so.

The engine needs this in `econ::victory::destroy_all`.

## Questions

1. In which order are the objects of a defeated house hit: by kind (infantry, vehicles, aircraft,
   buildings, or another order), by creation order, or by another order?
2. Are objects off the map (items in production, objects inside a transport or a building) hit at
   all, or only objects on the map?

## Possible observation

Defeat a house that owns two buildings and two vehicles of types with distinct death draws (debris
or survivors). Record the draw sequence with the random-state recorder and compare it with each
candidate order. For question 2, surrender a house with an infantry item still in production and
an infantry unit inside a transport, and check whether the stream shows a death for either of
them.

## Current workaround

Only objects on the map are hit, one after another in object number order (creation order).
Items in production are left to S-0404 claim 8. Listed in `docs/development.md`'s provisional
table.
