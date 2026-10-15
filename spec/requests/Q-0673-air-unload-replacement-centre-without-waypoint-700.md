---
id: Q-0673
title: The air-unload replacement centre when map waypoint 700 is undefined
status: open
related: [S-0507, S-0009, S-0501]
answered-by: []
---

## Need
S-0507 claim 13 (revision 4) centres the replacement-destination search on map waypoint 700 and
keeps "the waypoint cell itself" if the search fails. S-0009 claim 15 says a waypoint value of 0
means undefined, and most maps define few waypoints. Claim 13 does not say what the centre is when
waypoint 700 is undefined. `transport::selection` needs it.

## Questions
1. When the map defines no waypoint 700, what centre does an air transport's unload use for the
   replacement search: map cell (0, 0), some other cell (the aircraft's own, its destination's),
   or is there no replacement at all (the destination kept, or the phase changed instead)?
2. If the search around that centre fails, which destination is kept?
3. Does the search still take its per-radius random draws in that case?

## Possible observation
On a tiny flat skirmish map without waypoint 700, fly a loaded invented air transport (named test
data) toward a clear cell and order *unload* while it is airborne. Record its destination after
each *selection* processing and the random-state recorder's draws. Repeat with waypoint 700
defined as a control.

## Current workaround
`transport::selection` (Q-0673 interim): an undefined waypoint 700 stands for map cell (0, 0), as
S-0011 claim 33's actions take an undefined waypoint; the search and its draws run as usual and
cell (0, 0) is kept if it fails.
