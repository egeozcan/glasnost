---
id: Q-1339
title: A landed aircraft seized with a ground stand-in
status: answered
related: [S-0607, S-0501]
answered-by: [S-0607]
---

## Need
S-0607 claim 3 lets a landed aircraft be an eligible target. Claims 22 and 23 (r4) install a driving, walking, ship or teleporting stand-in on a seized target and move it under S-0207 or S-0504, but those specs describe vehicles, not aircraft, and no spec says how an aircraft moves with such a stand-in. The gap is in `lift::install_stand_in` and `lift::stand_in_tick` (crates/deterministic/sim/src/lift.rs).

## Questions
1. Does a landed aircraft hit by a seizing warhead naming the driving (or the teleporting) locomotor drive (or jump) towards the lifter like a vehicle?
2. Is it later able to take off, or does it stay on the ground for good?

## Possible observation
Scenario L13 part b with a landed aircraft as the victim, once with the driving warhead and once with the teleporting one.

## Current workaround
The aircraft is seized (claims 9 and 10 apply, claim 24 decides restoration), but no stand-in movement is installed: it stays where it stands and does not move or turn under the stand-in.
