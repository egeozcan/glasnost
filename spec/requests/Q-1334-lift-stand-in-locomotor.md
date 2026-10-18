---
id: Q-1334
title: How a unit seized with a stand-in locomotor moves and is restored
status: open
related: [S-0607]
answered-by: []
---

## Need
S-0607 claim 22 says that a seizing warhead naming a locomotor that can stand in temporarily (driving, walking, ship, teleporting or drop-pod movement, and the teleporting locomotor when the key is absent) seizes the target "as in claim 5, with that locomotor installed in place of its own", and that how it then moves, and when it is restored, are outside the spec. The engine must still do something with such a unit. The gap is in `lift::seize`, `lift::restore` and the movement dispatch in `tick.rs` (crates/deterministic/sim/src/).

## Questions
1. Does a unit seized with, for example, the driving locomotor move towards its lifter, stay where it stands, or move some other way? Does it keep the occupancy mark of its cell while seized?
2. Do claims 7 and 8 (the movement point fixed at the seize, the arrival drop) have any counterpart for it? Can it be dropped (claim 13), or is it always set down?
3. When is it restored: at its first fall-back to rest after the grip ends (claim 20's rule), at once when the grip ends, or never?
4. While seized and set down, does it refuse destinations (claim 9) and firing (claim 10) as a jumpjet-seized unit does?

## Possible observation
Scenario L13 with named test data: a magnetic-beam weapon whose warhead names the driving locomotor, then the teleporting locomotor, then no locomotor; fire at a light vehicle and record position, occupancy, destination and the tick it can be ordered again.

## Current workaround
The unit is seized and gripped as claim 5 says, but no jumpjet locomotor is installed and no movement point is fixed. It keeps its own movement state and occupancy mark, stops by its own rules (its route and destination cleared) and stays where it stands. Claims 9 and 10 apply (a destination written without the assignment is replaced by none at the start of its behaviour turn). The grip ends by claim 12's events; the unit is then set down, and claim 20's rule restores it at its first fall-back to rest, putting nothing back because nothing was taken away.
