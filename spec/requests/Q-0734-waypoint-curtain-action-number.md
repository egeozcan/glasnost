---
id: Q-0734
title: The action number of the waypoint invulnerability trigger action
status: open
related: [S-0513, S-0524, S-0011]
answered-by: []
---

## Need
S-0513 claim 22 and the S-0524 Summary describe a trigger action that applies invulnerability at
a waypoint. It acts on the ground layer of the waypoint cell's effect block, with no launching
house and no leaving-stage skip.

Neither spec gives the action's number in the trigger action table, so our trigger dispatch
(`triggers.rs`) cannot recognise it. The effect itself is built
(`superweapons::protect::curtain` with `waypoint = true`).

## Questions
1. Which trigger action number applies invulnerability at a waypoint?
2. Which parameter slot holds the waypoint? Does the action use any other parameter (a house, a
   duration)?
3. Does it place the curtain invoke animation and raise the radar event, as a launch does?

## Possible observation
Use a named test map whose trigger uses the action at a waypoint, with a vehicle and an infantry
unit in its effect block. Record which is protected, which dies, and what is shown.

## Current workaround
The action is not dispatched. A trigger that uses it falls under the "outside the subset" note
and does nothing. The waypoint effect is reachable only from code and tests.
