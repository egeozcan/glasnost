---
id: Q-2181
title: When an effect weapon whose emitter is not matched fires again
status: answered
related: [S-0301, S-0616, S-0613]
answered-by: [S-0301]
---

## Need

S-0301 r58 claim 5 and Terms say when a *matched* particle emitter (its system's behaviour kind
matches the place its weapon flag sets) stops being alive, and so when its weapon fires again.
For an emitter that is not matched, claim 5 and the spec's Open questions leave it unstated: the
let-go events follow the place (S-0616 claim 1) and the end of the system follows its kind
(S-0616 claim 22). Our engine must still decide whether such a weapon's live-effect gate holds it
back. The gap is the gate of S-0301 claim 5 (`emitters::own_effect_alive`,
`emitters::other_effect_alive`), which asks whether the weapon's emitter place holds a system.

## Questions

1. A firer whose weapon has the fire-particle flag and names a particle system type of the spark
   kind, with a stationary durable target in range, keeps its target and stays put. After its
   first shot, does it fire again once the spark system has ended by its own counter, or only
   after a let-go event of the fire place (a retarget to a target not within reach, a move order),
   or never while it keeps its target?
2. The other way round: a weapon with the spark-particle flag whose system type is of the fire
   kind. Does it fire again only once the system has left play as a fire-kind system does (S-0613
   claim 17, S-0616 claim 22), and does a move order or a retarget let it go?
3. Is the answer the same for a railgun-flag weapon whose system type is of another kind, and for
   a fire- or spark-flag weapon whose system type is of the railgun kind?

## Possible observation

S-0301 scenario F4's particle part with each pairing above in a test rules file: record shot
ticks, the tick at which each system leaves play, and the let-go events given.

## Current workaround

`design: Q-2181` interim: an emitter that is not matched is treated as a matched one is. It is
alive while its place holds it: from the shot until a let-go event of its place or its system's
leaving play by its own kind, whichever comes first. So the weapon fires again at the first
attempt after the first of the two, and R′ ticks after the shot. No switch.
