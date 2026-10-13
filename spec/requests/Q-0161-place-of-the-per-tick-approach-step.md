---
id: Q-0161
title: Where in an actor's tick the per-tick approach step of an unreachable target runs
status: answered
related: [S-0215, S-0203, S-0309]
answered-by: [S-0215]
---

## Need
S-0215 revision 14, claim 37 (a), says that "separately, on every tick",
an infantry unit or vehicle with a target and no destination, not a team
member, that cannot reach its target's cell (or is a passenger of an
open-topped transport) takes the approach step in every behaviour, and
drops a target out of range if it still has no destination afterwards.
The claim does not say where in the actor's update this happens. The
list "Ordering within a tick, as observed" places the per-tick start,
the processing and the per-tick search of claim 33, but not this step.
`glasnost_sim::approach::per_tick` needs a fixed place in
`process::turn`.

## Questions
1. On a tick on which such an actor processes, does the per-tick
   approach step happen before its per-tick start, between the per-tick
   start and the processing, between the processing and the per-tick
   search of claim 33, or after that search?
2. On a tick on which the actor does not process, is the place the same
   relative to its per-tick start and its per-tick search?
3. When the actor's processing on that tick already took the approach
   step (claim 23, 26 or 27), does the per-tick rule take the step a
   second time on the same tick, or is one step per tick all that
   happens?
4. A target that the per-tick search of claim 33 finds on a tick: can
   the per-tick approach step drop it on the same tick (it would if the
   step came after the search and the new target were unreachable and
   out of range)?

## Possible observation
S-0215 scenario B32 (d), with an unreachable target out of range and an
attacker whose processing falls on a known tick, read tick by tick with
the read-only recorder: the tick on which the target is dropped, relative
to the processing tick, and whether a target found by the per-tick search
on that tick survives it.

## Current workaround
The per-tick approach step runs after the actor's processing (or, on a
tick without one, after its per-tick start) and before its per-tick
search, every tick, whether or not the processing took the step too.
