---
id: Q-0795
title: The cell-arrival update for locomotors outside S-0516 claim 22's table
status: answered
related: [S-0516, S-0207, S-0208, S-0503, S-0501]
---

## Need

S-0516 revision 7 (claim 22 and Terms) gives a table of cell-arrival updates by locomotor:
driving and ship track arrival, walking arrival and snap, hovering heading-point choices,
and none for ordinary flight. These updates move a mobile sensor's disc and give a fully
cloaked mover the claim 13 adjacency check. The code in
`crates/deterministic/sim/src/cloak/machine.rs` (`arrival_source`, `entered_cell`) needs
the same for every other way a mobile object changes cell. The table has no row for these:

- the teleporting locomotor (S-0207 claim 14: the locomotor of a type with no locomotor
  field);
- the rocket locomotor;
- the locomotors we do not move yet, jumpjets among them;
- relocations that are not locomotion at all, such as a teleport superweapon moving a unit
  (S-0513), an object put down by a carrier, or a unit leaving a transport.

## Questions

1. Does a teleporting mover get a cell-arrival update when its teleport completes? At the
   destination cell? Before or after its other arrival handling?
2. Does a jumpjet get one: at landing only, at each cell crossed in flight, or never (as
   ordinary flight)? Does a quirk like `flying_sensor_disc_stationary` apply to it?
3. Does a rocket, or any missile-like mobile object, ever get one?
4. Does a superweapon teleport, a drop from a carrier, or an unload give the moved object
   a cell-arrival update, or does its disc stay where it was until its next locomotor
   update?

## Possible observation

- A sensor unit of each kind, moved one way at a time. Record the sensor coverage
  around the old and the new cell on every tick from the start of the move until two
  ticks after it ends.
- The same with a fully cloaked mover ending next to an enemy sensor. Record the tick on
  which it starts to uncloak.

## Current workaround

These movers have their cell-arrival update at the cloak update in the first tick in which
their centre cell differs from the recorded one. This was the general rule before
revision 7. The choice is marked in the code with this request's ID.
