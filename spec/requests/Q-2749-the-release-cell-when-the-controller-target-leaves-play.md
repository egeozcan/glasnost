---
id: Q-2749
title: The release cell of a missile released when the controller target leaves play
status: answered
related: [S-0506]
answered-by: [S-0506]
---

## Need

S-0506 r34 claim 14, *A destination that leaves play after the launch*: when the leaving object is
the controller target and no pending target waits, claim 11's reset follows when the object leaves
play, by its removal or by going inside, and it releases every preparing object of a missile type
(claim 12). Claim 12 gives a released missile "the cell under the target's position when there is a
target, and otherwise the neighbouring cell of the missile's own cell in the step direction nearest
its body facing". The Open questions say which of the two applies here is not given: "the cell of
the object that has left play, or claim 12's cell for no target".

The engine has to pick one. The release cell is the target of every later re-order (claim 26). It
does not change the flight of a rocket missile whose aim point claim 14 fixed at its launch, but it
is the attack target of a released missile of a missile type that is not of a rocket type. The
code is `crate::spawn::reset` and `crate::spawn::release_object`.

## Questions

1. When the controller target goes inside a transport or a building and the reset releases a
   preparing missile, which cell becomes its release cell: the cell the target stood in when it
   went inside, the cell of the transport or building, or claim 12's neighbouring cell for no
   target?
2. The same when the controller target is removed from play (the end of a death sequence, a sale
   of a building, a removal by a trigger).

## Possible observation

S-0506 scenario R21 (an ordinary-flight flagged aircraft as the slot type, whose re-orders show
its release cell), with the controller target an infantry unit that boards a transport next to it
while the aircraft is preparing (question 1), and in a second run a unit destroyed with a death
sequence (question 2). Record the aircraft's target after the first re-order.

## Current workaround

The release reads the target object while it still exists: a target that goes inside is still in
the world, off the map, and gives the cell it stood in when it went inside (it keeps its position);
a removed target gives the cell it stood in at its removal, which happens before it is taken out of
the world. That is claim 12's first case with the leaving object as the target.
