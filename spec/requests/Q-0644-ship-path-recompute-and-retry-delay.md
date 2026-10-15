---
id: Q-0644
title: Does a ship recompute its path and wait out the retry delay like a driving vehicle
status: answered
related: [S-0504, S-0212, S-0207]
answered-by: [S-0504]
---

## Need
S-0504 claim 1 says a ship moves, turns and stops exactly as a drive vehicle and lists the
S-0207 claims that hold for it. S-0212 claim 20 (recomputing as it begins the last stored
step, with the heading point more than 512 leptons away) is stated for the drive locomotor
only, and claim 21 gives the retry delay's rules for the drive, walk and hovering locomotors
but not for the ship locomotor. Claim 23 does name the ship locomotor (its destination is
cleared after a failed search). Our ship locomotor runs the driving code, so it now takes
the driving rules of claims 20 and 21 as well; this needs confirming.

## Questions
1. A ship on a long path begins the last of its stored steps while its destination is still
   far away. Does it search for a new path at that moment, as a driving vehicle does, or only
   once its stored steps run out?
2. After a ship's fresh search succeeds, does a retry delay run before its next search (as for
   the hovering locomotor), or none (as for the driving locomotor)?
3. After a ship re-paths around a moving blocker on its next step, does the retry delay start
   again as for the driving locomotor?

## Possible observation
Order a stock ship along a path longer than the stored-path length, and record the ticks of
its path searches (from changes in its stored path); repeat with another ship blocking its
next step for a few seconds.

## Current workaround
Interim rule: a ship follows the driving locomotor's rules of S-0212 claims 20 to 22, since
it moves by the driving code (S-0504 claim 1).
