---
id: Q-0735
title: How long a silo stays in its launch sequence and refuses a new launch
status: open
related: [S-0513, S-0512]
answered-by: []
---

## Need
S-0513 claim 7 says a silo "already in its launch sequence with nothing queued need not start a
second sequence". In that case the target is still recorded and the sounds play, but no extra
missile leaves.

Claim 8 says the upward missile leaves on the silo's next update. The silo then plays its launch
animation phases and returns to guarding. Those phases are presentation, and their length is not
given.

Our engine must decide when the silo accepts a launch again (`superweapons::nuke::accepts`).
This also decides S-0512 claim 8's charge when
`quirks.missile_without_launcher_consumes_charge` is off.

## Questions
1. A silo accepts a launch in tick t. Its upward missile leaves in its next update. In which tick
   does it accept a new launch that does start a second missile?
2. Does a second launch made while the first is still in sequence queue a missile ("nothing
   queued" suggests a queue)? If so, when does the queued missile leave, and at which target?
3. Does the answer depend on the silo's animation lengths (the imported launch phases)?

## Possible observation
Use S-0513 scenario SW2 part i (busy silo) with a one-time record and a ready record of the same
house. Fire both at different cells a few ticks apart, and repeat with growing gaps. Record which
orders produce a missile and at which tick each leaves.

## Current workaround
Interim rule (docs/development.md, *Simulation (M6b: superweapons)*):
- A silo is busy only from the tick it accepts a launch until its upward missile leaves, on its
  next update.
- After that it accepts again at once.
- No launch is queued.
