---
id: Q-0237
title: What a close-range attacker does after the approach shortcut sends it to a standing position in its own cell
status: open
related: [S-0307, S-0208, S-0209, S-0215]
answered-by: []
---

## Need

S-0307 claim 22 (b) ends a close-range approach search at once when a standing position *P*
in the attacker's own cell is nearer the target's cell centre: "the attacker's movement is
sent directly to *P*, its destination becomes the cell of *T*, and the target is kept". The
spec leaves the movement itself to S-0208, which has no rule for a walker sent to a point
while it holds a different destination cell. Our engine needs to know how the attacker moves
after the shortcut. This is in `approach::shortcut`.

Claim 22 (b) also says that a vehicle on the ground gets no position there because of its own
vehicle mark. A vehicle on a bridge deck carries its mark in the deck record, not the ground
record that the choice reads, so the shortcut seems able to succeed for it.

## Questions

1. After the shortcut, does an infantry unit walk to *P* and stop there, so that the
   destination (the target's cell) only matters at a later approach step? Or does it go on
   from *P* towards the destination cell, and if so, by which route and standing position?
2. When is the standing position *P* reserved in the own cell's occupancy record, and when is
   the entry for the position the unit stood on cleared: at once, when it starts walking, or
   on arrival?
3. If the unit was already walking to another standing point when the search ran, is that
   walk replaced by the walk to *P*, and is the earlier reservation cleared?
4. Can the shortcut succeed for a vehicle on a bridge deck (its own mark being in the deck
   record), or for any other non-infantry actor? If so, how does that actor move to *P*?

## Possible observation

A close-range infantry unit standing at the centre of the cell next to an enemy infantry unit
standing at a side position, with a weapon range shorter than their distance, given an attack
order. Record its standing point, its destination and its cell's occupancy record each tick
until it fires or moves on. Repeat with the unit walking when the order is given, and with a
close-range vehicle on a high-bridge deck next to its target.

## Current workaround

An infantry unit takes *P* as its walking target at once: the entry for its old walking
target and its own cell are cleared, and *P* is reserved in its own cell's record for its
layer. Its destination is the target's cell. Any other actor only gets the destination.
Interim row in `docs/development.md`.
