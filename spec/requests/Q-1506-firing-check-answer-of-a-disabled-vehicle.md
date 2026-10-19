---
id: Q-1506
title: Which firing-check answer and rank a vehicle disabled by its control centre gets
status: open
related: [S-0405, S-0301]
answered-by: []
---

## Need
S-0405 claim 20 (revision 20) says the firing check answers *illegal* for every target and
every weapon of a vehicle that the loss of its control centre has disabled. S-0301 claim 9
lists "the firer is disabled" among the *impossible* reasons, and does not say at which rank
the disabled vehicle's answer stands. The two can be read as different answers for the same
state. The rank matters when another reason also holds, for example a vehicle that is
launching spawned aircraft (*busy*, rank 2), or one whose asked target is its own gripped unit
(*rearming* within rank 4). Our check is in `crates/deterministic/sim/src/combat/check.rs`.

## Questions
1. Is the answer for such a vehicle *illegal* or *impossible*? Is S-0301's "the firer is
   disabled" the same state as S-0405's disabled vehicle, or another one?
2. At which rank of S-0301 claim 9 does the answer stand? Does it outrank *busy* and the
   rank-4 *rearming* cases?

## Possible observation
Hard to separate by observation, since both answers stop every shot. A disabled powered-unit
vehicle given a force-fire order: record the cursor the player sees over an enemy (S-0214 shows
the no-action cursor for *illegal*). A disabled vehicle type that launches spawned aircraft
would separate rank 2 from rank 4.

## Current workaround
*illegal*, as S-0405 claim 20 says. It is placed at the head of rank 4: after the rank-2 *busy*
answer, before the grip refusals and every other rank-4 test.
