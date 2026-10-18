---
id: Q-1401
title: What is one "unloading step" of refinery smoke
status: open
related: [S-0402, S-0616]
answered-by: []
---

## Need
S-0616 claim 1 makes refinery smoke "at each unloading step of a delivery to a refinery (and at
each delivery of a resource-carrying slave)". S-0402 claim 11 unloads a harvester in payment rows
(one row per resource it holds, the whole amount at once) and S-0402 claim 14 pays a slave's load
per resource as well. Neither spec names an "unloading step".

## Questions
1. Is one unloading step one payment row of S-0402 claim 11 (so a harvester carrying two
   resources makes the smoke twice), one whole delivery, or something tied to the docking
   animation (which S-0402 does not model)?
2. For a slave delivery, is the smoke made once per delivery, or once per resource paid?
3. When a slave miner is undeployed (a vehicle), do its slaves' deliveries make refinery smoke,
   and at whose position?

## Possible observation
A refinery with one set smoke point; count the systems made by one harvester carrying one
resource, then two resources, and by one slave delivery.

## Current workaround
`emitters::refinery_smoke` runs once after each payment row of S-0402 claim 11 (on the
refinery) and once per slave delivery (on the master, only when it is a building whose type
names a refinery smoke system).
