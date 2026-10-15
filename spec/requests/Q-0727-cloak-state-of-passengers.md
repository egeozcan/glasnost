---
id: Q-0727
title: Cloak state of a self-cloaking vehicle that leaves a transport
status: open
related: [S-0516, S-0506]
answered-by: []
---

## Need

S-0516's Open questions ask whether a passenger keeps its cloak state inside a transport. Claim
16 says that a vehicle whose own cloak flag is set enters play fully cloaked. It does not say
whether unloading counts as entering play. The code in
`crates/deterministic/sim/src/cloak/machine.rs` (`entered_map`) needs the rule.

## Questions

1. A self-cloaking vehicle was uncloaked when it entered a transport. When it is unloaded, is it
   cloaked, uncloaked or cloaking?
2. Does it keep its cloak delay across the ride?
3. Does a cloaked infantry passenger keep its state?

## Possible observation

Make a self-cloaking vehicle uncloak by firing, load it into a transport at once, and unload it a
few ticks later.

## Current workaround

Leaving a transport counts as entering play. A vehicle with its own cloak flag comes out fully
cloaked (claim 16). Other objects come out in the state they had. The choice is marked in the code
with this request's ID.
