---
id: Q-1159
title: S-0605 claim 3 refers to a "Flag F1" that the spec does not have
status: open
related: [S-0605]
answered-by: []
---

## Need

S-0605 revision 6, claim 3, has a new vehicle bullet: "a vehicle ordered in to a vehicle
absorber always starts waiting in line". It says that at an empty building "the walk it makes
is at most its movement in that turn (or none, if the retry comes before *enter* is current;
see Flag F1)". The spec has no flag F1. Its F1 is the capacity test (Formulas), which has
nothing to do with the order of the retry and the start of *enter*. Our code follows the turn
order of claim 4 and S-0215 claim 11. If a vehicle is not ready at its first per-tick start
after the order, *enter* is not current yet. The retry still runs after movement, grants
contact and queues *enter* with an immediate start. No approach is made.

## Questions

1. What does "see Flag F1" in claim 3 point to?
2. Does that flag hold any rule beyond the turn order of claim 4 (retry after behaviour
   processing and movement) and S-0215 claims 7 and 11?

## Possible observation

None needed if the reference is editorial. Otherwise, scenario A2c's vehicle variant, with
the vehicle moving when the order takes effect (not ready at its first start).

## Current workaround

None needed. The behaviour is as built: `absorb::order_entered` and `absorb::waiting_turn`,
`claim_3_vehicle_always_waits_in_line`. The request is noted in `docs/m7-claims/S-0605.md`.
