---
id: Q-1301
title: Is a tank bunker arrival reported once, and is it also detected at the enter processing?
status: answered
related: [S-0606, S-0215, S-0207]
answered-by: [S-0606]
---

## Need
S-0606 claim 5 says that on arrival the vehicle ends its route, stops and makes the completion report, and the bunker queues repair. It does not say whether the bunker stays the vehicle's destination after arrival (which would let the same arrival be reported again at a later step) or whether arrival is only detected when the drive reaches its target, or also during the vehicle's enter processing when it is already standing at the docking cell.

## Questions
1. After arrival, does the entering vehicle still have the bunker as its destination?
2. If a vehicle in enter with bunker contact is already standing still on the docking cell (for example it was stopped there), does it count as arrived at its next enter processing?

## Possible observation
Stop a vehicle on a docking cell, then order it into the bunker; note whether the entry stages start without further movement.

## Current workaround
Interim rule: arrival clears the destination (the report is made once) and arrival is also checked at each enter processing.
