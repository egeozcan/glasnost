---
id: Q-1404
title: Where in an object's turn the damage sparks and deactivated-vehicle sparks come
status: answered
related: [S-0616, S-0203, S-0405]
answered-by: [S-0616]
---

## Need
S-0616 claim 1 makes damage sparks and the sparks of a deactivated vehicle "in its turn", with
draws, but does not place them among the other draw-taking steps of the object's turn
(self-healing, radiation, behaviour processing, movement, firing). Their order against those
steps decides the draw order. It also leaves to other specs which events deactivate a vehicle.

## Questions
1. In an object's turn, do the damage-sparks draws come before or after its self-healing,
   behaviour processing, movement and firing draws?
2. Same question for a deactivated vehicle's spark draws; and does a deactivated vehicle take
   the turn at all on the ticks it is deactivated?
3. Which events make a vehicle deactivated for this row: losing its house's control building
   (S-0405 claim 13), an electromagnetic pulse, being driverless, others?

## Possible observation
The random-state recorder on a cyborg below yellow that also moves and fires, and on a vehicle
whose control building is sold.

## Current workaround
Both run late in the ordinary living turn of an infantry unit, vehicle or aircraft, after its
movement and attached-parasite step and just before its economy events
(`tick::object_turn`); a deactivated vehicle is one disabled by the loss of its control
building (S-0405 claim 13, `econ::power::unit_disabled`).
