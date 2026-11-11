---
id: Q-2972
title: An order a player gives an own object whose owner changes before the order is carried out
status: answered
related: [S-0213, S-0214]
answered-by: [S-0213]
---

## Need

S-0214 (Terms, *Own object*) lets a player order only objects of the player's house, and S-0213
claim 3 holds every house, an AI house included, to S-0214's input rules. S-0213 claim 15 adds
that the original does not check the owner again when an order is applied: an order given while
the actor was the house's own still applies after the actor's owner changes before the order
boundary.

Our engine now checks the owner where an order enters the simulation, `orders::issue`
(GitHub issue #220): an order that names an actor that does not exist or that the issuing house
does not own is refused there. In a network game `issue` runs on every machine at the tick in
which the order is scheduled, which comes some ticks after the player's click (the order delay of
the network game). So an order the player gave while the object was theirs is refused when the
object changes owner between the click and that tick (a mind-control take-over, a capture, a
defection). The spec's claim 15 speaks of the time between issue and application in the
original; we do not know what the original does with an order that is in flight on the network
when the owner changes.

## Questions

1. In a network game of the original, when a player orders an own unit and the unit is taken
   over (or changes owner in another way) in the ticks between the click and the tick in which
   the order is carried out on every machine, is the order carried out on the unit?
2. Is the answer the same for an order given to a building, for example a sell order to a building
   that is captured in those ticks?
3. Is the answer the same in a skirmish game, where the click and the order's tick may be closer?

## Possible observation

A two-player network game with a high order delay: one player orders a unit to move while the
other's mind-control unit takes it over in the same moment. Record whether the unit moves to the
first player's destination after the take-over.

## Current workaround

`orders::issue` refuses the order (`// nocite: our own input validation (#220)`), the same on
every machine at the same tick. An order already taken at issue applies after an owner change, as
claim 15 says. Tests: `crates/deterministic/sim/tests/hostile_orders.rs`.
