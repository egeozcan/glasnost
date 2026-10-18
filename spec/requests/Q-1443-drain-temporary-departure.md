---
id: Q-1443
title: Drain links of a party that leaves play without being destroyed
status: open
related: [S-0608, S-0402, S-0507, S-0614]
answered-by: []
---

## Need
S-0608 claim 13 ends a link when either party receives a killing hit or is removed from the game,
and Open question 3 leaves temporary departures from play without destruction unresolved (for
example a drainer that goes inside a transport, is taken off the map by a parasite or other
carrier, or a drained building that is taken off the map for a while). Our engine must keep or
end the link in those cases. A departed drainer no longer gets turns, so with the link kept it
would neither check its position nor its alliance until it returns.

## Questions
1. When a drainer leaves the map without being destroyed (any way the rules allow), does its
   link end at once, at its return, or never on that account?
2. While it is off the map, does the drained building stay drained (blackout, payouts, refused
   sale and firing)?
3. The same for a drained building that leaves the map without being destroyed, if any rule
   allows that.

## Possible observation
Named test data giving a transport the capacity to carry a Disc-like hovering drainer, or a
drainable building that can be lifted; record the link state while it is off the map and after it
returns.

## Current workaround
Our engine ends links only at a killing hit or a removal (claim 13); taking an object off the map
for a while does not touch its links. No stock unit reaches this: the Disc does not enter
transports.
