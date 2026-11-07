---
id: Q-2726
title: Does claim 33's move at the movement update also come for an aircraft in sticky
status: answered
related: [S-0501, S-0215]
answered-by: [S-0501]
---

## Need

S-0501 r146 claim 33 moves the *move* that an aircraft in *guard* gets to the movement update:
"When a living aircraft that is not pad-bound and not crashing (claim 9) is in *guard* with a
destination and nothing queued at a movement update, that update queues *move* for it, at any
height, on the ground included." Before r146 that *move* came with the processing of *guard*.
S-0215 claim 5 says that *sticky* is processed exactly as *guard*, so under r134 an aircraft in
*sticky* got the same *move* at its processing. The movement update is not a processing of the
behaviour, so S-0215 claim 5 no longer settles whether an aircraft in *sticky* gets it. The
engine has to decide in `air::guard_move` (crates/deterministic/sim/src/air.rs), which runs in
each movement update after claim 31's first start point.

## Questions

1. Does a living aircraft that is not pad-bound and not crashing, with *sticky* current, a
   destination and nothing queued, get *move* queued at a movement update, as one in *guard*
   does? Or does it keep *sticky* with that destination until something else changes it?
2. If it does not get it there, does a processing of its *sticky* still give it *move*, as the
   processing of *guard* did before r146?

## Possible observation

An aircraft that is not pad-bound, put in *sticky* with a destination cell some cells away and
nothing queued (on the ground and in flight). Record each tick its current and queued
behaviours, its destination and its position. If it gets *move*, say in which tick relative to
the tick at whose end *sticky* is current, and whether it then flies to the destination.

## Current workaround

`air::guard_move` queues *move* for *guard* only, as claim 33 names it; an aircraft in *sticky*
gets no *move* at the movement update or at its processing. Cited `S-0501 claim 33` and
`Q-2726` in the code; the C7 and intake 108 ledgers list it as a seam.
