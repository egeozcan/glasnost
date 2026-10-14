---
id: Q-0231
title: Does the area-guard processing that takes its first guard spot run the leash and search table
status: answered
related: [S-0215, S-0309]
answered-by: [S-0215]
---

## Need

S-0215 claim 26 (*area guard*) gives an actor with no guard spot and nothing queued its own
cell as guard spot, and handles an actor with a guard spot by the leash and search table. Its
Open questions leave unestablished whether the own cell taken in a processing already counts
for that table in the same processing, or whether that first processing does none of the
table (no search, no idle action). The two differ in the draws and in the targeting-timer
restart of that first processing. The area-guard processing is
`crates/deterministic/sim/src/process.rs` (the area-guard rows of claim 26).

## Questions

1. An actor enters *area guard* with no guard spot, no target, no destination and nothing
   queued, with its targeting timer run out and passive acquisition allowed. In its first
   *area guard* processing, does it run the area search around its own cell (restarting the
   targeting timer with its draw), or does that processing take no search and no idle action?
2. If the first processing does not search, does an infantry unit in that processing attempt
   its idle action?
3. When an enemy stands within the area-guard search range at that moment, on which processing
   is it first acquired?

## Possible observation

S-0215 scenario B10 (g) with the read-only recorder: record the targeting timer, the
random-stream state and the target on the tick of the first *area guard* processing of a
unit given *area guard* by a scripted order with no guard spot, once with an enemy inside
the search range and once without.

## Current workaround

The table applies only to a guard spot held when the processing begins: the processing that
takes the own cell as guard spot runs no search and no idle action. Interim row in
`docs/development.md`.
