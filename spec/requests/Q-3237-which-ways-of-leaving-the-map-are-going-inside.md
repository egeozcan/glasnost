---
id: Q-3237
title: Which ways of leaving the map are going inside for S-0507 claim 6a
status: open
related: [S-0507, S-0402, S-0614, S-0506, S-0508, S-0605]
answered-by: []
---

## Need

S-0507 claim 6a (r11, Q-3207) defines going inside as leaving the map "by boarding a transport of either kind (claim 6) or by going into a building that takes it off the map, as a garrison occupant does (S-0508)". At that moment the object's vehicle and infantry holders lose it as their destination, and their locomotion keeps its point.

Our engine also takes objects off the map, without ending them, in these cases. For each, it is not clear whether claim 6a's loss applies:

- **A slave going inside its master (S-0402 claim 14).** The master is a vehicle when undeployed, and a building when deployed.
- **A parasite going inside its host (S-0614 claim 1).** The host is a vehicle, not a transport.
- **A spawned aircraft returning into its spawner (S-0506).** The spawner can be a vehicle or a building.
- **A trigger action's lift of a building's occupants** for a building put in its place. The occupants are already inside.
- **An infantry unit going into an absorbing building (S-0605 claims 7 and 8).** This seems to be "a building that takes it off the map". We apply claim 6a to it.

## Questions

1. A unit has a slave as its destination, and the slave goes inside its master: (a) a vehicle; (b) a building. Does the unit lose the slave as its destination at that moment and keep its movement point, as claim 6a says for a transport? Or does something else happen?
2. The same question for a parasite that goes inside its host, and for a spawned aircraft that returns into its spawner.
3. Is going into an absorbing building going inside in claim 6a's sense?

## Current workaround

Our engine's rule until this is answered:

- Claim 6a's loss applies to:
  - boarding a transport of either kind (S-0507 claim 6);
  - going into a building as an occupant (S-0508 claim 5);
  - going into an absorbing building (S-0605 claims 7 and 8).
- The other cases keep what the engine did before r11. A holder of an aircraft follows S-0501 claim 53. A holder that reverts follows S-0309 claim 27a's clear, which assigns no destination. Any other holder keeps the destination as it was.
