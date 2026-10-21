---
id: Q-1759
title: Whether a chrono miner teleports or drives when a person orders it to move
status: open
related: [S-0207, S-0402, S-0214, S-0417]
answered-by: []
---

## Need
S-0207 claim 20 (a) says an object with active teleporting locomotion takes every destination, from a behaviour or an order, as a landing point and jumps to it. S-0214 claim 11 says a self-teleporting type's *move* takes the clicked cell. The engine does this (crates/deterministic/sim/src/teleport.rs `take_destination`, targeting.rs). In play the project lead expected a chrono miner to drive on a person's move order and to teleport only when it returns to a refinery. Claim 20 is backed as `measurable` (D22) and has no M-record, so we need to know which is right before the code changes or stays.

## Questions
1. A chrono miner has active teleporting locomotion (it has docked at least once since it left the factory). A person orders it to move to an open cell with no ore, far from any refinery. Does it teleport to that cell, or drive there along a path?
2. Is the answer the same when the order is given while the miner is (a) seeking or loading ore, (b) returning full, (c) standing in guard with nothing to do?
3. Is the answer the same for a person's attack-move, guard-area or waypoint-path order, and for a move order onto an ore cell (harvest order)?
4. If it drives on a person's order, what makes it teleport again: the next harvest-cycle destination it sets itself, docking, or something else?

## Possible observation
S-0207 scenario D22, extended: a chrono miner that has unloaded once, then a person's move order to a cell 20 cells away on open ground with no ore. Record its cell every tick (a teleport is a jump of more than 2 cells in one tick, as in S-0402 scenario H8). Repeat in each state of question 2 and with each order kind of question 3.

## Current workaround
The engine follows S-0207 claim 20 (a) as approved: every accepted destination teleports while teleporting locomotion is active. No change until this is answered.
