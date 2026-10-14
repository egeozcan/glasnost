---
id: Q-0430
title: Does a vehicle slave master's group update come before or after its behaviour processing in its turn?
status: answered
related: [S-0203, S-0402, S-0215]
answered-by: [S-0203]
---

## Need

S-0203 claim 12 places a vehicle master's due group update before the vehicle's movement in
its turn: the update sees the position and destination from before that movement, and a
destination it assigns is available to the same turn's movement. The claim does not place the
update against the other work of the vehicle's turn that comes before movement, chiefly its
behaviour processing (S-0215: the per-tick start and the processing of its current behaviour,
including the slave miner's directed *harvest* of S-0402 claim 22, which can set or replace the
vehicle's destination). The simulation (`tick::object_turn`) must put the group update on one
side of it.

## Questions

1. In a vehicle master's turn, does a due group update see a destination set or cleared by that
   same turn's behaviour processing (for example, a directed *harvest* processing that moves the
   miner to a found site, or a behaviour start that clears the destination)?
2. Conversely, does the behaviour processing in that turn see the group state the update has
   just set (for example, travelling, or working after a deployment attempt starts turning)?
3. Does a behaviour order applied at the end of the previous tick (S-0203 claim 7) start before
   or after the group update in the next turn of the vehicle?

## Possible observation

A slave miner vehicle, explicitly given *harvest* with a cell destination so that its first
*harvest* processing (which searches for a site and moves there) falls in the same tick as a due
group update while the group is field seeking. Record the destination and group state at the end
of that tick: if the group update saw the directed site it enters travelling without a field
search of its own (S-0402 claim 22's seeking-vehicle-with-a-destination row); if not, it makes
its own long-scan field search, whose site may differ.

## Current workaround

The group update runs after the behaviour part of the vehicle's turn and immediately before its
locomotor (`tick::object_turn`), which satisfies claim 12's stated movement ordering. Listed in
docs/development.md (slave miners).
