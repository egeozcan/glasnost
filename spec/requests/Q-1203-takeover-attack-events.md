---
id: Q-1203
title: The attack events of a takeover hit, their order and how they are judged
status: answered
related: [S-0601, S-0011]
answered-by: [S-0601]
---

## Need
S-0601 claim 5 says a live tagged target of a takeover shot hears attack
events 6 and 44, whether or not the takeover succeeds. It does not say in
which order the two are raised, which object or house counts as the cause,
or how the trigger system judges them. S-0011's judged subset does not yet
include events 6 and 44, so a trigger waiting on them never fires in our
engine.

## Questions
1. When a tagged object is hit by a takeover shot, is event 6 raised before
   event 44, or the other way round? Do the actions of a trigger on event 6
   happen before those of a trigger on event 44?
2. Is the cause of both events the controlling unit (so its house is the
   attacking house), as for an ordinary damaging hit?
3. When are events 6 and 44 judged true: on any raise with their code, or
   only for an attack by a house the event names?
4. Does a building taken by mind control give its owner (old or new) the
   announcement that S-0509 gives for a captured building?

## Possible observation
Attach one tag to a target with two triggers, on events 6 and 44, each
setting a different local variable and playing a different sound; fire a
takeover shot at it from a named house and record the order of the
effects. Repeat with a target the takeover test refuses (an immune unit).

## Current workaround
The engine raises 6, then 44, with the controller as the cause
(`mind::detonation`). The two codes are not judged true by S-0011's subset,
so only triggers whose events are satisfied by any raise react. A building
taken by mind control changes owner with no announcement
(`structures::ownership::Cause::MindControl`).
