---
id: Q-0240
title: Which occupancy record holds a teleporting object's vehicle-present mark at a high-bridge landing point
status: answered
related: [S-0207, S-0209]
answered-by: [S-0207]
---

## Need

S-0207 revision 12 claim 20 (a) moves the vehicle-present mark of the occupancy record (S-0209)
from the cell of the object's old point to the cell of its landing point *L* as soon as *L* is
fixed, before the jump. Claim 20 (e) now says that on a high-bridge cell the object's cell layer
after the jump is the bridge deck. A high-bridge cell has two occupancy records, one for the
ground and one for the deck (S-0209 Terms). Neither claim says which of the two records holds the
mark at *L*'s cell between fixing *L* and the jump, nor which record it is cleared from when a
later destination moves it again. Our engine must pick one record for setting and clearing the
mark. This is in `teleport::take_destination` (crates/deterministic/sim/src/teleport.rs).

## Questions

1. When a teleporting vehicle fixes a landing point on a high-bridge cell, is its
   vehicle-present mark set in that cell's deck record or its ground record? Does the answer
   depend on the deck mode of claim 20 (b) (the object's height more than 312 leptons above the
   surface there)?
2. While the mark stands there, does a ground vehicle trying to drive into the cell under the
   bridge, or a vehicle on the deck trying to drive onto it, meet the mark?
3. After the jump, is the mark in the record of the layer the object is listed in?

## Possible observation

S-0207 scenario D22 on a map with a high bridge, with the read-only recorder: order a teleporting
vehicle onto a deck cell at a tick on which its behaviour sets the destination after its own
locomotor update, so that *L* stays fixed for a tick before the jump. Record the deck and ground
records of that cell on that tick, and whether a vehicle driving under the bridge or along the
deck towards the cell stops or re-routes then.

## Current workaround

The mark is set in, and later cleared from, the record of the layer the object will be listed
in after the jump: the deck's on a high-bridge cell, the ground's elsewhere. Interim row in
`docs/development.md`.
