---
id: Q-0269
title: Whether a satisfied instance's re-arming draws come before or after its actions
status: answered
related: [S-0011, S-0202]
answered-by: [S-0011]
---

## Need
S-0011 claim 15 re-arms a trigger timer when the instance is satisfied in a raise (for a repeating
tag, or when an entered-by event was judged true), and the re-arming of a random-delay trigger takes
its draws (claim 14). Claim 11 says the actions take effect within the raise. When the actions draw
from the scenario random stream too (an explosion at a waypoint, claim 28), the order of the two
sets of draws is observable, and the claims do not give it.

## Questions
1. For a repeating tag whose trigger has a random-delay event and an explosion action, are the
   re-arming draws taken before the explosion's draws or after them?
2. When the trigger's own actions enable that trigger again (claim 25), which arming's length
   stands: the one of the satisfaction or the one of the enable action?

## Possible observation
S-0011 scenario T11 variant B's recorder on a repeating random-delay trigger whose only action is
an explosion at a waypoint.

## Current workaround
Our engine re-arms first and then fires the actions (crates/deterministic/sim/src/triggers.rs,
`raise`), so an enable action of the trigger itself arms it last. Marked in docs/development.md,
section *Map triggers (M5b)*.
