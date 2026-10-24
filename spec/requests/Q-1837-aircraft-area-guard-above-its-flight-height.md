---
id: Q-1837
title: An aircraft's area guard above its flight height
status: open
related: [S-0501, S-0215]
answered-by: []
---

## Need
S-0501 r40 claim 45 splits an aircraft's *area guard* by height: "at exactly its flight height"
each processing is the fall-back to rest with a delay of 1 tick, and "below its flight height"
the processing has the spent-ammunition fall-back, the held-target *attack* and S-0215 claim 26's
rows with *k* = 2. It does not say what a processing does when the aircraft is above its flight
height. Nothing in the spec rules it out: the height above the terrain grows when the terrain
under a flying aircraft drops (a cliff edge, a plateau's rim), until claim 2's height change
catches up. `glasnost-sim` `air::area_guard` needs one of the two branches for that case.

## Questions
1. When an aircraft in flight in *area guard* is processed at a height above its flight height,
   is the processing the fall-back to rest with a delay of 1 tick (as at exactly its flight
   height), or the below-the-flight-height branch (the spent-ammunition fall-back, the held-target
   *attack* with 1 tick, S-0215 claim 26's rows with *k* = 2), or something else?
2. Is "its flight height" the height above the terrain under the aircraft (as claim 6's held
   height), and is the test made on the height at the start of the processing?

## Possible observation
AC29's computer-owner set-up (claim 45), with a named test type whose flight level is low and a
run that ends over a cliff edge or a raised plateau, so that the aircraft enters *area guard*
above its flight height; the read-only recorder logs per tick its height above the terrain, its
behaviour, its queued behaviour and the delay of each processing.

## Current workaround
A height at or above the flight height is read as at it: each processing is the fall-back to rest
with a delay of 1 tick. It cites Q-1837 in `crates/deterministic/sim/src/air.rs` (`area_guard`).
