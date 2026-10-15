---
id: Q-0723
title: Does every crush uncloak the crusher
status: answered
related: [S-0516, S-0210]
answered-by: [S-0516]
---

## Need

S-0516 claim 11 (c) says, with medium confidence, that a vehicle that crushes something starts
uncloaking. The spec's Open questions ask whether this happens on every crush. The code in
`crates/deterministic/sim/src/crush.rs` and `cloak/machine.rs` (`crushed`) needs the rule.

## Questions

1. Does a cloaked crusher uncloak alike when it crushes each of these:
   - an infantry unit
   - a vehicle
   - a crushable overlay (a wall or fence)?
2. Does a crusher ever uncloak by this cause when it crushes nothing, for example when it drives
   over an empty cell?

## Possible observation

A cloaked crusher vehicle, cloaked by a field or by itself, is driven over each of these in turn:
an enemy infantry unit, a crushable vehicle and a crushable wall overlay.

## Current workaround

Every crush of an object and every overlay crush makes the crushing vehicle start uncloaking, with
the cloak sound. The choice is marked in the code with this request's ID.
