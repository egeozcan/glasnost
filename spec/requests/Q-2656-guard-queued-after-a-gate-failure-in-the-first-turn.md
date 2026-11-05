---
id: Q-2656
title: Why guard is queued at the end of the first turn of a released walker whose ordered search fails at the gate
status: open
related: [S-0417, S-0215, S-0212, S-0516]
answered-by: []
---

## Need

M-0023 run 1: an infantry unit just released from its factory, on a cell that holds the factory,
is given an ordinary move order to a cell in another reach region before its first turn. At the
end of that turn *F* (*F* − *t* = 1) the record shows *move* current, *guard* queued and no
destination, and the next tick *G* = *F* + 1 has *guard* current and the factory's scatter request
with a new destination.

Our engine, built from the specs, does this instead: in turn *F* the unit's due *move* processing
runs before its locomotion (S-0215 claim 2, S-0516 claim 42) and sees the ordered destination,
so it falls under S-0215 claim 22 with a destination and chooses the standard delay. The
locomotion that follows fails the fresh search at the gate and clears the destination
(S-0212 claim 23, the gate row: "no behaviour is queued"; S-0417 claim 17, *No path*). At the
end of *F* we show *move* current, nothing queued, no destination. *move* is next processed only
when that standard delay is over, falls back to *guard* (S-0215 claim 22, no destination), and the
request follows in that turn, so our *G* − *F* is the standard delay rather than 1. The walk after
the request has the same lag as the baseline's (S-0417 claim 17, U10 part g).

The composition of S-0215 claims 2, 3 and 22, S-0516 claim 42 and S-0212 claim 23 gives our
reading; M-0023's row for the end of *F* and its *G* − *F* contradict it. The code is in
`crates/deterministic/sim/src/path.rs` (the gate failure, `failed`) and the behaviour processing
of *move* (S-0215 claim 22).

## Questions

1. In the turn in which a walking unit's fresh search for an ordered destination fails at the
   reachability gate, what puts *guard* in its queue? Is it the gate failure itself, under a
   current *move*, or something else that turn does?
2. Does a walking unit's due *move* processing in such a turn see the destination as it is before
   the turn's locomotion (so the destination is still held), or as it is after it (so none)? Is
   S-0215 claim 2's "processing sees the destination before locomotion" true for an infantry unit
   in its first turn after a release, as it is for an ordinary vehicle turn?
3. Why does *guard* become current in the very next tick (*G* − *F* = 1) rather than after
   S-0215 claim 22's standard delay? Which delay does *move* choose in *F*, and does a queued
   behaviour shorten it?
4. Is the same true for a walking unit that is not newly released: one resting in *guard* on open
   ground, given a move order to a cell in another reach region?

## Possible observation

M-0023's set-up already shows it. For question 4: the same pen, with an infantry unit resting in
*guard* a few cells away on open ground, given the same move order outside the update; record its
current and queued behaviour and destination per tick for the first 20 ticks.

## Current workaround

The engine is unchanged: the order of S-0215 claims 2 and 22 and S-0212 claim 23 as written. The
local-lane check `m0023_rows_hold_in_our_engine` (`crates/tools/tests/arena.rs`) reports the two
rows as failing and expects them to fail until this is answered.
