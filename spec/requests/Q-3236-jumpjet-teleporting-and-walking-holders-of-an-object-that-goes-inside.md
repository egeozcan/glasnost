---
id: Q-3236
title: Jumpjet, teleporting and walking holders of an object that goes inside
status: answered
related: [S-0507, S-0502, S-0207, S-0208]
answered-by: [S-0208, S-0507]
---

## Need

S-0507 claim 6a (r11, Q-3207) says that every vehicle and infantry holder of an object that goes inside loses it as its destination, and that "the holder's locomotion is not told of the loss". The movement point the locomotion last received stays as it was. The claim then says how the movement goes on for:

- driving and ship movement;
- hovering movement;
- walking: "S-0208 claim 16, with that point as its locomotor destination".

It names no rule for jumpjet movement (S-0502) or teleporting movement (S-0207 claim 20). For walking it names a point that other claims seem not to keep.

- **Jumpjet.** S-0502 claims 16, 17 and 24 say that an effective clear of the order-level destination invokes claim 17's reroute, "whatever caused the clear". Claim 6a's loss is a clear of the order-level destination, and it says the locomotion is not told. It is not clear whether the reroute runs.
- **Teleporting.** S-0207 claim 20 (h) says that an empty assignment while a jump is pending cancels the pending jump and the moving state. It is not clear whether the loss is such an assignment, or whether the pending jump to the fixed landing point goes on.
- **Walking.** S-0208 Terms defines the locomotor destination as "the point that its destination gives" while the unit walks under a move. S-0208 claim 6 (r52) says that a walker follows a stored path only toward a destination, so a destination that becomes nothing leaves it no path to follow. With no destination after the loss, claim 16's first case ("the unit has no locomotor destination") seems to end the walk at the next arrival tick. Claim 6a instead says to go on "with that point as its locomotor destination".

## Questions

1. A jumpjet vehicle or jumpjet infantry unit flies toward a unit as its destination, and that unit boards a transport. Does the jumpjet go on to the movement point it was given, as an aircraft does under S-0501 claim 53? Or does it reroute or stop as for an ordinary clear of its destination?
2. A teleporting vehicle has a jump pending to a unit as its destination, and that unit boards a transport before the jump. Does the jump still happen to the fixed landing point?
3. A walking infantry unit walks toward a unit as its destination, and that unit boards a transport. At its next arrival tick, does it stop there, as with a cleared destination? Or does it go on along its path toward the point its destination gave before the loss, and if so, how far?

## Possible observation

The same setup as S-0507 T16, with the holder A replaced by a jumpjet unit, a teleporting vehicle (jump pending) or a walking infantry unit that has the boarding unit as its destination. Record A's position, flight state and behaviour every tick for 150 ticks after the boarding.

## Current workaround

Our engine's rule until this is answered:

- **The loss itself** (claim 6a): the order-level destination, the queued destinations and the approach destinations go, as for every holder.
- **Jumpjet and teleporting locomotion are not told.** No reroute runs and no pending jump is cancelled. The locomotor's record of the destination it last took stays as it was. Nothing reads the inside object's position for them, because both fix their point when they take the destination.
- **Walking** is not given a stored point. With no destination, the walk ends at the next arrival tick (S-0208 claim 16, first case), and a stored path is dropped (claim 6).
