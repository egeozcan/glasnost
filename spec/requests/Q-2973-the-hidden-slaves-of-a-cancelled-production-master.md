---
id: Q-2973
title: What becomes of the hidden slaves of a production item that is cancelled
status: open
related: [S-0402, S-0404, S-0312, S-0601, Q-0193]
answered-by: []
---

## Need

S-0402 claim 19 gives a produced slave master its slaves at production start, when its object is
created (S-0404 claim 11): they are created inside and off the map with the master. S-0404 claim
6 deletes the item's object when the item is cancelled. Neither says what becomes of the slaves
then. S-0312 claim 31 leaves never-placed production masters outside its claim under Q-0193,
which answers the case of items killed when their house is defeated, not a cancel.

GitHub issue #156 showed that our engine left the slaves of every cancelled master in the world:
owned by the house, hidden, counted by the tech tree and by the house's object count, and never
removed. A Build and Cancel pair per tick grew the world by a slave group each time, for no
credits (the cancel refunds the item, and the slaves cost nothing).

Our engine now takes the slaves out with their master when the item is cancelled, silently: no
destruction, no loss counted, no notification, as the spawner slots of S-0506 claim 13 are taken
out. A cancelled controller's mind-control record (S-0601 claim 1) is removed with it, as the
record of any removed controller is. We do not know whether the original counts the hidden slaves
as lost or tells anything about them.

## Questions

1. When a production item whose type names a slave type is cancelled, are its slaves removed from
   the game with it?
2. Are they counted as losses of their house (in the score screen's counts) or as kills of anyone,
   and does anything announce or show their removal?
3. Is the answer the same when the item is cancelled by the removal orders of S-0411 claim 2
   (a type taken from the offer while it is being built)?

## Possible observation

Build a slave miner, cancel it while it is being built, and compare the house's unit losses on the
score screen with a game in which it was not built. Repeat with the slave type's prerequisite
lost while the miner is being built.

## Current workaround

`econ::production::delete_item_object` removes the master's slave group with it, silently
(`objects::remove` per slave after its link is cleared), and the controller record goes with
the master's removal. Tests: `crates/deterministic/sim/tests/cancel_creation_set.rs`.
