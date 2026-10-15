---
id: Q-0794
title: When a driving vehicle's locomotor destination point is present
status: answered
related: [S-0314, S-0207, S-0212]
answered-by: [S-0207]
---

## Need

S-0314 claim 17 (the table answering Q-0790) says a driving or ship vehicle gives the
movement report while its locomotor destination point is present. Terms says that point
is separate from the order-level destination. Our driving locomotor keeps only the
order-level destination, which is S-0207's move destination. The locomotor clears that
destination when it reaches its goal cell. No spec says when the separate destination
point is set or cleared for driving and ship locomotion. Without that rule, a vehicle
that stands at its goal with its destination still set (for example, one given an enter
order while already in the depot's centre cell) is ambiguous. The code in
`crates/deterministic/sim/src/depot.rs` (`moving_destination`) needs the rule.

## Questions

1. When does a driving or ship locomotor set its destination point? On every move order,
   or only when a path search succeeds?
2. When is it cleared? On arrival at the goal cell, at the last track's end, or on give-up?
3. Does an enter order on a depot whose docking point lies in the vehicle's own cell set
   that point?

## Possible observation

The S-0314 claim 17 bench tests (d) and (e), with the read-only trace recording the
locomotor destination point next to the order-level destination: once for a vehicle that
gets an enter order while already standing in a pad depot's centre cell, and once for a
vehicle that is blocked short of it.

## Current workaround

The destination point counts as present while the order-level destination exists and its
goal cell (S-0212's heading point) differs from the vehicle's cell. This choice is marked
in the code with this request's ID.
