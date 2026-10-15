---
id: Q-0616
title: When does a spawner's pending target get cleared without a launch?
status: answered
related: [S-0506, S-0301, S-0309, S-0215]
answered-by: [S-0506]
---

## Need

S-0506 claim 3 says that a spawner weapon's shot hands its target to the spawn controller as a
pending target, and that clearing the spawner's target clears the pending target. A spawner's
target is cleared in many situations: an order, a fall-back to guard, the target leaving play, a
failed acquisition, or the spawner never holding a target of its own when the shot was made. Our
engine needs to know which of these clear the pending target, and when, in
`glasnost_sim::spawn::pass` and `glasnost_sim::spawn::target_cleared`.

## Questions

1. Which events that leave the spawner without a target also clear its controller's pending
   target: a new move or stop order, the target leaving play, the spawner falling back to guard
   after its behaviour ends, a target switch by acquisition, or all of them?
2. If the spawner holds no target of its own at the moment of a pending target's next launch
   pass (for example, because the pending target came from a shot with no target set on the
   spawner), is the pending target adopted or dropped at that pass?
3. If the spawner's target is cleared and then set again to the same object before the next
   launch pass, is the pending target still there at that pass?

## Possible observation

S-0506 scenario R2. Order a launcher to attack a target, then (a) give it a move order,
(b) a stop order, (c) let the target leave play, in the ticks between the shot and the next
launch pass. Record whether a missile is launched at that pass.

## Current workaround

At the start of each pass, if the spawner's own target is empty, the pending target is cleared
before the controller reads it. Otherwise the pending target is kept (marked Q-0616 in
`glasnost_sim::spawn` and in docs/development.md, M6a interim rules).
