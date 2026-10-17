---
id: Q-1157
title: Does the guard queued by a failed recycle approach start while eaten is current?
status: open
related: [S-0603, S-0212, S-0215]
answered-by: []
---

## Need

S-0603 revision 4, claim 3, says that a unit sent into a recycling building whose path
computation fails more than 1 cell from the building gives up as S-0212 claim 23 row 2 says:
it leaves its team, its destination is cleared and it queues *guard*. The claim then says
that whether this queued *guard* ever starts while *eaten* is current is S-0215's rule and is
not established: S-0215 claims 14 and 15 queue nothing from their own fall-back tables during
*eaten*, but those tables are not the queue of S-0212 claim 23. So the unit "may stay in
*eaten* with *guard* queued, or may start *guard*". Our engine must do one of the two. The
code is `path::failed` (the give-up) and `grinder::process_eaten` with S-0215's start rules
in `behaviour.rs` and `process.rs`.

## Questions

1. After a recycle order whose path computation fails at once (for example S-0212 scenario
   P12 run S, at a failing placement of a 7-line), does the unit's current behaviour change
   from *eaten* to *guard*, and in which tick after the failure?
2. If it stays in *eaten*, does the queued *guard* stay queued until a later order, or is it
   dropped at some point?
3. Is the answer the same for a vehicle and for an infantry unit?

## Possible observation

S-0603 scenario G9's approach-failure test already records, over about 10 processings after
the failure, the unit's current behaviour, its queued behaviours and its destination on
each tick.

## Current workaround

Interim rule, cited as "Q-1157 interim" in `claims_s0603.rs`
(`claim_3_failed_approach_gives_up`) and noted in `docs/development.md` (Simulation, M7a:
Grinder) and `docs/m7-claims/S-0603.md`: S-0215's ordinary start rules apply to the queued
*guard* as to any queued behaviour, so it becomes current at the unit's next ready moment
(in the synthetic test, in the tick of the failure), and the unit leaves *eaten*. This is
the same for a vehicle and an infantry unit.
