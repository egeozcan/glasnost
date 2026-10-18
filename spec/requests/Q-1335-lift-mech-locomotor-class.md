---
id: Q-1335
title: Can the walking-vehicle locomotor stand in for a seize
status: answered
related: [S-0607, S-0207]
answered-by: [S-0607]
---

## Need
S-0607 claim 22 sorts the locomotors a seizing warhead can name into those that can stand in temporarily (driving, walking, ship, teleporting, drop-pod) and those that cannot (hovering, flying, rocket, tunnelling). S-0207 table T3 lists one more locomotor identifier, the walking-vehicle (mech) locomotor, which claim 22 does not place in either class. The jumpjet locomotor is claim 5's own case. The gap is in `SeizeLocomotor::of` (crates/deterministic/sim/src/combat/data.rs).

## Questions
1. A seizing warhead naming the walking-vehicle locomotor: is an eligible target seized (the stand-in class) or only stopped and put in guard without a seize (the other class)?

## Possible observation
Scenario L13 with named test data: a magnetic-beam weapon whose warhead names the walking-vehicle locomotor's identifier; fire at a light vehicle.

## Current workaround
The walking-vehicle locomotor is treated as one that can stand in temporarily, like the walking locomotor (and then as Q-1334's workaround says).
