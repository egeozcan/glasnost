---
id: Q-1070
title: When does a jumpjet unit built at a factory stop being tied to the factory
status: open
related: [S-0417, S-0502, S-0404]
answered-by: []
---

## Need

A vehicle or infantry unit with the jumpjet movement that a factory releases keeps its contact with the factory and its tether for the rest of the game under the specs as they stand: S-0417 (claim 10, "Factory contact during jumpjet ascent", and its open question about the clear report) says that no event is established that ends a jumpjet's contact or tether, and that there is no timeout. The cell-arrival updates of walking, driving, hovering and teleporting units end the tether (S-0417 claims 7 and 10); a jumpjet's touchdown is a cell-arrival update only for the cloak and sight purposes of S-0502 claim 15.

The effect in play, found by the local lane (seed 7, AI against AI, the naval maps): the first jumpjet unit a house builds closes the factory that made it for good. A vehicle factory stays in its unloading stage with the unit as its contact, however far the unit has flown, and makes no further vehicle; an infantry factory is held in the same way. Both houses of the naval maps stopped producing and the games did not end.

The own AI cannot tell a person's way out of this (a stop order is refused while tethered, S-0417 claim 7).

## Questions

1. In the running game, does a factory that releases a jumpjet unit (vehicle or infantry) make further units after the first one has flown away? If it does, which event ends the unit's contact with the factory and the tether: the end of its ascent, its first cruise, its first touchdown, its first destination being reached, or something else?
2. If the factory is held only while the unit is still on or over its foundation, what is the distance or the condition?
3. Does the answer differ between a jumpjet vehicle (balloon-hover or not) and jumpjet infantry, and between a unit released with a rally point and one released without?

## Possible observation

Build a barracks and a war factory for a jumpjet type, release one unit of each with no order given, then with an order to move across the map. Order a second unit of the same type after each and note whether the factory accepts the release. Record the ticks of the unit's first movement and of the second release.

## Current workaround

The simulation follows the specs (no event ends the tether). The own AI builds no jumpjet unit (`glasnost_ai::scripted::jams_factory`; its army builder and the jumpjet module's order skip the types) and keeps using the ones it has by other means. When this request is answered the rule and the simulation's behaviour are revisited.
