---
id: Q-0701
title: The shore search of a vehicle transport unloading on water
status: open
related: [S-0507, S-0212]
answered-by: []
---

## Need

S-0507 claim 8 (table U, settling) has a vehicle transport on water with no destination choose
"a nearby location where a wheeled vehicle could stand" as its destination and wait 10 ticks. The
spec's Open questions leave "exact shore-search ties and failure behaviour" open.

The code in `crates/deterministic/sim/src/transport.rs` (`shore`) needs the rule.

## Questions

1. Which search is it: the nearby-cell search of S-0212 claim 25, and with which settings (zone,
   reach region, bridges, overlays)?
2. How are ties between equally near cells broken?
3. What happens when no such cell exists: does settling go on to choose an unloading cell, wait
   again, or end the unload?

## Possible observation

S-0507 scenario T8 with an amphibious transport on a lake: vary the shore so that two cells are
equally near, and try a lake with no reachable shore.

## Current workaround

The nearby-cell search of S-0212 claim 25 from the transport's cell for the wheel class in zone
normal, with no reach region; when it finds no cell, settling goes on to claim 9's choice. The
choice is marked in the code with this request's ID.
