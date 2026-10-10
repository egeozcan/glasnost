---
id: Q-0030
title: Where a gate's door reaches rest within the gate's own update
status: open
related: [S-0215, S-0203, S-0209]
answered-by: []
---

## Need
S-0215 claim 35 (d) says that a moving gate door reaches rest open or rest
closed "at the gate's own update on the first tick on which *T*<sub>d</sub>
ticks have passed since the motion began", and leaves where that update falls
within the tick to S-0203. S-0203 does not say where, within one building's
turn, the door moves relative to the building's per-tick starts and its
behaviour processing (S-0215 claims 2 and 11). `glasnost_sim::process` (the
gate's *open* processing) needs that order.

The order is observable. Take a door that starts opening in the *open*
processing of tick *t*, with *T*<sub>d</sub> = 90. If the door reaches rest
before that turn's processing, the stage-*opening* processing of tick
*t* + 90 already sees the door at rest open and moves to stage *waiting*; its
first Std (one draw of the scenario random stream) comes at tick *t* + 91. If
the door reaches rest after the processing, stage *waiting* starts one tick
later, and so do the draws and the earliest closing.

## Questions
1. Within a gate's turn, does its door reach rest (claim 35 (d)) before or
   after the gate's behaviour processing of that turn, and before or after
   its per-tick starts (claim 11)?

## Possible observation
Scenario B28 (a) of S-0215 with the recorder reading the gate's progress count
and the scenario random stream's position each tick: compare the tick on which
the count first shows *waiting* with the tick on which the door is first at
rest open.

## Current workaround
`process::turn` moves a gate's door at the start of the gate's turn, before
the per-tick start and the processing, on every turn whatever the current
behaviour. The code marks it with this request's ID.
