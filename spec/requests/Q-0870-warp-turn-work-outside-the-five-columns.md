---
id: Q-0870
title: What a warp member does in the turn parts that S-0513 claim 51 does not list, and the per-tick behaviour start
status: answered
related: [S-0513, S-0203, S-0215, S-0516]
answered-by: [S-0513]
---

## Need

S-0513 claim 51 (revision 13) suppresses five kinds of ordinary work for a warping vehicle, teleporter infantry unit or landed aircraft from its first pending update through arrival effects: self-healing, behaviour processing, ordinary movement, firing and turret turning. Claim 48 says the warp is not thereby the whole turn, and S-0215 claim 2 says the per-tick start of claim 11 takes no effect only on tunnel, erasing and leaving-stage ticks.

The turn has more parts than those five: the per-tick behaviour start of S-0215 claim 11, the ordinary cloak update (S-0516 claim 42), reload and firing-frame timers, an attached parasite's step (S-0614 claim 18), the unit's own economy and power updates, the queued-entry retry of a depot visit (already skipped while phasing in), and the expected-health bookkeeping. The code is `tick::object_turn`, which returns right after `warp::turn` in every stage before the arrival hold.

## Questions

1. In the pending, leaving, departure, off-map retry, placement and arrival-effects updates, does the per-tick behaviour start of S-0215 claim 11 take effect (the literal text of claim 2 lists only the tunnel, erasing and leaving-stage exclusions)?
2. In the same updates, do the cloak update, the reload and firing-frame timers, an attached parasite's step and the unit's own economy and power updates run, or are they part of what the design suppresses?
3. If the spec side wants a rule without measurement, which one: every part of the turn other than the warp step and the intrinsic target search and sensor update is skipped, or only the five listed kinds?

## Possible observation

S-0513 scenario SW23 part f, extended to record the cloak state, the reload counter and the per-tick start on each row of claim 51.

## Current workaround

The warp update is the whole turn in every stage before the arrival hold (nothing else of the turn runs, the per-tick start and the other parts included). In the arrival hold the hold-end test runs first and the ordinary turn follows. This is the rule the code already followed under the old interim for Q-0851; it is a superset of claim 51's five columns and is marked `Q-0870 interim` in `docs/development.md`.
