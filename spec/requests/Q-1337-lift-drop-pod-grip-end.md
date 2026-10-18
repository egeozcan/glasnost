---
id: Q-1337
title: What the drop-pod stand-in's grip end does to the lifter
status: open
related: [S-0607]
answered-by: []
---

## Need
S-0607 claim 23 (r4) says that our engine ends the drop-pod stand-in at once at the seize: the unit stays where it is, gets its own locomotor back, and "the grip ends with the unit set down". Claim 12's table lists the events that end a grip and what each does to the lifter (its target, its refusal state), but none of its rows is "the drop-pod stand-in ended at the seize". The engine must decide whether the lifter keeps its target (the victim) and goes on refusing, or loses it as in claim 12's other rows. The gap is in `lift::drop_pod_ends` (crates/deterministic/sim/src/lift.rs).

## Questions
1. When the drop-pod stand-in's grip ends (at touchdown in the original, at once in our engine), does the lifter keep the victim as its target, or is its target cleared as when a grip ends by claim 12's other events?
2. Does the lifter stop refusing new targets at that moment, or only when it would by claim 11?
3. In the original, does the lifter's beam go on being drawn to the victim during and after the drop-pod descent?

## Possible observation
Scenario L13 part c with the drop-pod warhead: record the lifter's target, its refusal state and its firing per tick from the seize until well after touchdown.

## Current workaround
The grip ends (the victim is no longer gripped and is set down where it stands, its own locomotor back, still seized), but the lifter's target is not cleared: the lifter keeps the victim as its target and behaves as with any target it may not seize again (claim 3).
