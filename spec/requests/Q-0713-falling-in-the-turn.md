---
id: Q-0713
title: Where a falling object's fall step sits in its turn, and what "its locomotion stops" clears
status: open
related: [S-0511, S-0522, S-0203, S-0215]
answered-by: []
---

## Need

S-0511 claim 9 says a dropped object's "locomotion stops" and it starts falling; claim 10 makes it
fall "on each of its own updates (S-0203)". The spec does not say whether the object's behaviour is
still processed while it falls, whether its ordinary locomotor also runs, whether it keeps its
destination, target and path, nor whether it can fire. The code in
`crates/deterministic/sim/src/tick.rs` (`object_turn`) and `fall.rs` needs the rule.

## Questions

1. While an object falls, does its own update still process its behaviour (S-0215) and its firing,
   or only the fall?
2. What does "its locomotion stops" clear: the destination, a planned path, the movement in
   progress, the target?
3. After landing, does the object keep any order it had on the deck?

## Possible observation

S-0511 scenario B3 with a deck tank that has an attack target in range and a deck infantry unit
with a move order: watch whether either fires or moves while falling and after landing (if a
protection lets it survive).

## Current workaround

At the drop the destination, any queued destinations and the path are cleared and the movement
in progress is reset; the target is kept. While falling, the object's turn processes its
behaviour as usual, but the fall step replaces its locomotor step, and a vehicle takes no firing
step. The choices are marked in the code with this request's ID.
