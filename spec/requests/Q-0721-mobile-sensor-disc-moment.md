---
id: Q-0721
title: When a moving sensor's disc moves, and drift when it leaves play
status: answered
related: [S-0516]
answered-by: [S-0516]
---

## Need

S-0516 Open questions ask two things about mobile sensors (claim 22):

- At which tick does a mobile sensor's disc move?
- Can coverage drift when a sensor leaves play between those moments?

The code needs the rule in `crates/deterministic/sim/src/cloak/sensor.rs` (`mobile_moved`,
`mobile_left`) and in `cloak/machine.rs` (`entered_cell`).

## Questions

1. When does a moving sensor unit's disc move? Is it when the unit enters a new cell (its centre
   crosses the cell boundary), or when it reaches the new cell's centre?
2. Suppose the sensor's position has changed cell but its disc has not moved yet. If the sensor
   then leaves play (it is destroyed or enters a transport), which disc is removed?

## Possible observation

A sensor vehicle drives past a cloaked enemy that stands just outside its sensor range. Record the
tick the enemy becomes visible, and compare it with the ticks the sensor crosses cell boundaries
and reaches cell centres.

## Current workaround

- The disc moves in the sensor's own turn: on the first turn in which the cell under its centre
  differs from the cell it was last added around.
- On removal, the disc is taken away around the current cell, never below 0, as the Edge cases
  say.

The choice is marked in the code with this request's ID.
