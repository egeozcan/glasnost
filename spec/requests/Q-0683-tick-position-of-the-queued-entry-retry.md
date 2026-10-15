---
id: Q-0683
title: Where in the tick a vehicle with a queued depot entry asks again for a place (S-0314 claim 23)
status: open
related: [S-0314, S-0203, S-0215]
answered-by: []
---

## Need
S-0314 claim 23: a damaged vehicle with a *queued entry* "on every tick ... asks again for a
contact place (claim 16's alliance and free-place tests). ... When one is free it takes it and
asks permission. If permission is granted, it switches to *enter* and approaches as in claim
17." The claim does not say where in the tick this retry runs: in the vehicle's own turn of the
update order (S-0203 claim 1) and, if so, before or after its behaviour processing (S-0215
claim 2), or in a separate pass. The position decides whether a place freed by a vehicle earlier
in the same tick (a depot sending its serviced vehicle away in its own turn) is taken in that
tick or the next, and whether the switch to *enter* is processed in the same tick. The engine
needs it in `tick::object_turn` (`depot::vehicle_turn`).

## Questions
1. In which part of the tick does a vehicle with a queued entry make its per-tick request: in
   its own turn (before or after its behaviour processing and its movement), or elsewhere?
2. When the depot releases its contact in its own turn earlier in the same tick, does the
   waiting vehicle take the place in that tick?
3. Is the switch to *enter* a queued behaviour with an immediate start, so that its *enter*
   processing can run in the same tick?

## Possible observation
H18 f: two vehicles sent together to a one-place depot, the second damaged, with object IDs
chosen so that the depot comes before and then after the waiting vehicle in the update order;
record contact, queued entry and behaviour each tick around the first vehicle's release.

## Current workaround
Interim rule, recorded in docs/development.md: the retry runs in the vehicle's own turn, after
self-healing and before its behaviour processing; on permission it switches to *enter* with
an immediate start.
