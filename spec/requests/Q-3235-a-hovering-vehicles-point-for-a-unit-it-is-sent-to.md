---
id: Q-3235
title: A hovering vehicle's point for a unit it is sent to, before that unit goes inside
status: open
related: [S-0503, S-0507, S-0212]
answered-by: []
---

## Need

S-0507 claim 6a (r11, Q-3207) says that when an object goes inside, a holder's locomotion keeps "the movement point it last received". For claim 5's transport this is "the object's movement point at that moment", the moment of the assignment, "unless that destination was given to it again later", and "the loss does not change it to where the object went inside". T16 expects a hovering transport's locomotor point to be "the same just before and just after I goes inside", and the transport then "heads along its path toward the recorded point", the point recorded at the claim 5 assignment.

So T16 takes *D* of an unheld hovering vehicle, for a destination that names a unit, to be fixed at the assignment. Our engine reads it from the named unit's current position at every movement update (`hover::destination_point`), so *D* follows the unit as it walks. S-0503 Terms says that assignment sets *D*'s height. No claim of S-0503 says whether *D* for a unit destination then stays where it was given or follows the unit. Claims 17 and 18 cover a point a held vehicle remembers. S-0212's searches use the destination's position.

In T16 the unit I walks from its resting cell to transport C or building B between the assignment and going inside. Under our reading, *D* just before the loss is I's position at that moment, not the recorded point. If the engine kept that point at the loss, it would be where I went inside, which is what T16's falsifier names. If it took the recorded point, the point would change at the loss.

## Questions

1. A hovering vehicle that is not held is given a unit as its destination, for example by S-0507 claim 5's reply on water. The unit then walks a few cells away. Does the vehicle's *D* stay at the unit's position at the assignment, or does it move with the unit at each movement update?
   - Watch its throttle (S-0503 claim 2: ½ within 256 leptons of *D*).
   - Watch its completion test (claim 7) and where it heads on its last leg.
2. If *D* stays at the assignment point, do its path searches (S-0212 claims 20 and 21) also look for that point's cell? Or do they look for the unit's cell at the time of the search?
3. Is the answer the same when the destination names a vehicle, and when the vehicle has a target and chases (S-0207 claim 19 (c))?

## Possible observation

T16 with one more record: at the end of every tick from the assignment until I goes inside, record A's *D* (or, if *D* cannot be read, the throttle and where A heads when it is within 256 leptons) and I's position.

## Current workaround

Our engine's rule until this is answered:

- **Before the loss: unchanged.** *D* for a unit destination is the unit's current position.
- **At the loss** (claim 6a), the hover keeps as its *D* the point recorded when its locomotor last took that destination. This is the unit's position at that assignment, or at a later assignment of the same unit.
  - This follows claim 6a's wording and T16's "heads toward the recorded point".
  - T16's "the same just before and just after" holds only when the unit has not moved since the assignment.

The recorded point is new hover state (`HoverState::taken_point`).
