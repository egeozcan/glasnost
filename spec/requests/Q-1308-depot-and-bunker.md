---
id: Q-1308
title: A building type that is both a repair depot and a tank bunker
status: answered
related: [S-0606, S-0314]
answered-by: [S-0606]
---

## Need
Data can set both the unit-repair (depot) role and the bunker flag on one building type. S-0606 and S-0314 each describe the enter processing, the docking point and the repair processing for their role alone.

## Questions
1. Does a vehicle entering such a building dock for repair, become bunkered, or both?

## Current workaround
Interim rule: the depot rules take precedence wherever both apply (enter processing, docking point, early start, order effects).
