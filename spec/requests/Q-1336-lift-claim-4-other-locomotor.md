---
id: Q-1336
title: Claim 4's harvester case under a warhead that names another locomotor
status: answered
related: [S-0607]
answered-by: [S-0607]
---

## Need
S-0607 claim 4 handles a resource-gathering vehicle whose own locomotor is already temporarily replaced (a teleporting harvester driving back): its replacement ends, or it stops and queues stop, and it is not seized. Claim 22 says that claims 1 to 3 decide the seize the same way whatever locomotor the warhead names, but does not say whether claim 4 also applies before claim 22's outcomes. The gap is in `lift::seize_hit` (crates/deterministic/sim/src/lift.rs).

## Questions
1. A seizing warhead that names a locomotor that cannot stand in temporarily (for example hovering) hits such a harvester: does claim 4's special case happen (its replacement ends or it stops and queues stop), or claim 22's (it stops, is put in guard, loses its spawned objects and its parasite)?
2. The same for a warhead naming a locomotor that can stand in (for example driving): claim 4's case, or a seize?

## Possible observation
Scenario L1 part f with named test data: the seizing warhead names the hovering locomotor, then the driving locomotor; fire at a teleporting harvester while it drives back to its refinery.

## Current workaround
Claim 4's special case comes first whatever locomotor the warhead names; claim 22's outcomes apply only to targets claim 4 does not catch.
