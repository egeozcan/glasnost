---
id: Q-0738
title: What else a warp member does in its turns while its warp is under way
status: open
related: [S-0513, S-0203, S-0215, S-0314]
answered-by: []
---

## Need
S-0513 claim 39 and T2 give a warp member's stages and say its later draws come in its own
updates. Our engine runs the warp step at the start of the member's living turn (after the
protection tint, S-0513 claim 29). It must also decide what the rest of that turn does in each
stage. The spec says the member stays at its old position through the leaving stage, and that it
counts as being erased then. It does not say whether the member still heals itself, processes its
behaviour, moves, fires or turns its turret while its warp is pending or leaving, while it is off
the map, and in the update in which it is placed.

## Questions
1. While a warp is pending or in its leaving stage (ticks *t*_L + 1 to *t*_L + 62), does the member
   still take the other parts of its turn: self-healing (S-0314 claims 9 and 10), behaviour
   processing (S-0215), movement by its locomotor, and turret turning? Does a moving vehicle stop
   at launch, or keep driving in its old cell until departure?
2. While it is off the map (from departure until it is placed), does it take any part of a turn
   other than its warp step?
3. In the update that places it, and in the update of its arrival effects, does it take the rest of
   its turn after the warp step?
4. During the arrival hold, does the hold-end test come before the rest of its turn, as for the
   teleporting locomotor's phasing in (S-0207 claim 20 f)?

## Possible observation
Extend S-0513 scenario SW12 part d. Warp a damaged self-healing vehicle that was driving towards a
far destination, with a turret turned away from its body. Record per tick, from *t*_L to
*t*_L + 66: its position, health, turret facing and destination.

## Current workaround
Interim rule (docs/development.md, *Simulation (M6b: superweapons)*):
- Joining the group stops the member: its destination, queued destinations, path and driving step
  are cleared.
- From its first update after launch until its arrival effects, the warp step is its whole turn.
- During the arrival hold, the hold-end test runs first, and then the ordinary turn follows.
