---
id: Q-0036
title: Is there a distance limit on the building that gives a map-placed wall its owner?
status: answered
related: [S-0210, S-0209]
answered-by: [S-0210]
---

## Need
`glasnost_sim::crush::assign_wall_owners` gives each wall the map places
an owner by S-0210 claim 14 (a): "the owner of the nearest building that
... is on the map and not destroyed, and whose owner's country has the
wall-owner flag ... When no building qualifies, the wall has no owner."
The claim states no distance limit, so every wall on a map with at least
one qualifying building gets an owner.

Scenario K12 of the same spec expects otherwise: its wall cell *w*₃ lies
"on a part of the map with no building within 40 cells, on a map with no
other buildings", while the buildings of the map houses *N* and *P* stand
elsewhere on the same map, and the expected result is "*w*₃ owned by no
house". Under the claim as written *w*₃ would be owned by *N* (its building
qualifies and is the nearest one). The claim and the scenario contradict
each other unless there is a distance limit, or another condition, that
the claim does not state.

## Questions
1. Does a building give a wall its owner only within some distance of the
   wall? If so, what is the limit, in leptons or cells, and is it compared
   with the same truncated 3-D distance that claim 14 (a) uses (strictly
   below, or at most)?
2. If there is no limit, which of K12's expectations is wrong, or what
   other condition makes *w*₃ ownerless?

## Possible observation
Scenario K12 as written, with *w*₃ placed at several distances (for
example 10, 20, 30 and 40 cells) from *N*'s building, recording its owner
after loading.

## Current workaround
No limit: the nearest qualifying building on the map gives the owner, as
the claim's text says. Marked "Q-0036" in `crush.rs`.
