---
id: Q-0694
title: The movement heading of a hovering vehicle when it is put on the map
status: open
related: [S-0503, S-0206]
answered-by: []
---

## Need

S-0503 Terms and S-0206 claim 13 give a hovering vehicle a movement heading η besides its body
facing. η turns towards the heading point, and throttle stays 0 while η is more than 45° off
(F1), so the value η starts with decides how long a newly placed or newly produced hover waits
before it accelerates. No spec gives η's value when the vehicle is created or put on the map
(by the map file, by a factory, or by a teleport). Scenario H2 starts from "a resting hover
whose movement heading is south" without saying how that state arises.

The code in `crates/deterministic/sim/src/hover.rs` and `objects.rs` needs it.

## Questions

1. When a hovering vehicle is put on the map, what is its movement heading: its body facing at
   that moment, a fixed facing such as north or east, or another value?
2. Is it set again when the vehicle is put down elsewhere (for example after a teleport) or
   leaves a factory?

## Possible observation

Place two hovering vehicles from the map file with body facings north and south, order both to
move east by 8 cells on the same tick, and record the tick on which each first moves. Repeat
with a factory-produced hover.

## Current workaround

The movement heading is set to the body facing the vehicle is placed with, every time it is put
on the map. The choice is marked in the code with this request's ID.
