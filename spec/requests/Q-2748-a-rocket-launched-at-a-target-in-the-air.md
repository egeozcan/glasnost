---
id: Q-2748
title: A rocket missile launched at a destination whose in-the-air answer is true
status: open
related: [S-0506, S-0307]
answered-by: []
---

## Need

S-0506 r34 claim 7 gives a missile slot's object "the move behaviour, and the controller target
as its destination when that target's in-the-air answer is false (the other case is an Open
question)". Claims 14 and 24 and the Rocket table's launch row fix the aim point and start the
flight at the launch only for that case. The Open questions say: "What a rocket missile does when
its destination's in-the-air answer is true at its launch is not given here".

The engine must still do something when a spawner's controller target is in the air at a launch:
a mod can give a rocket spawner a weapon that may target aircraft, and a set-A or set-B rocket
missile that has reached ascent is itself in the air (S-0506 claim 22). The code is
`crate::spawn::launch` and `crate::rocket::tick`.

## Questions

1. At such a launch, is the missile given the controller target as its destination?
2. Is its aim point fixed at all, and if so when (at the launch, at its first update or at a later
   one) and from which point of the target?
3. Does the aim point follow the target while it flies, and what does the missile do when the
   target lands, leaves play or stops being in the air before the detonation?
4. Does the flight start at the launch, with claim 24's ticks from the launch tick, as for a
   target that is not in the air?

## Possible observation

A named test map gives a stock rocket launcher's weapon a projectile that may target aircraft
(S-0307), and an aircraft of another house hovers or circles within range. Record, per tick from
the launch, the missile's destination, position, height, pitch, phase and speed, the aircraft's
position, and the detonation tick and point (scenario R25's recorder).

## Current workaround

S-0506 r20's rule, the last one approved for this case: the missile is given the controller target
as its destination at the launch, and its aim point is fixed at its first update from that
destination's heading point (claim 14 r20), with claim 24's timer started then. A missile whose
destination is gone at that update stays without a flight, as under r20.
