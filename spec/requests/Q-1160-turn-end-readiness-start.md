---
id: Q-1160
title: S-0215 claim 11's turning clause, the start at which a turn's end makes a driving vehicle ready, and a turn in place whose move is taken away
status: answered
related: [S-0215, S-0206, S-0207]
answered-by: [S-0215]
---

## Need

S-0215 revision 63, claim 11, makes a vehicle with the driving locomotor "moving", and so not
ready to change, while its body is still turning towards the facing it has been given. It
"becomes ready on the first start or late start after the turn ends", and scenario B47 adds
that "the late start counts that update's turning".

Our turning facing follows S-0206 F1: the facing is a function of the tick number, and from
tick t₀ + Θ it equals its target. So in the tick in which the facing reaches its target, the
first per-tick start already sees the turn ended. The vehicle is ready there, and the queued
behaviour becomes current at that first start and is processed in that same tick (claim 3a).
B47's parenthesis can also be read the other way: the facing's last change is made by that
tick's update, so the first start of that tick still sees the body turning, and the behaviour
becomes current only at the late start and is first processed on the next update (claim 3b).
B47's observable, "*attack* is current after the tick *t*", holds under both readings. Where
the behaviour is processed, in tick *t* or in tick *t* + 1, differs between them.

B47 also says that it is not known what the attack order does to the move in progress. In our
engine the order removes the destination and the route. A turn in place before a step was
counted as movement in progress until the step began, so the vehicle stayed "moving" for ever
and *attack* never became current. We now end the turn in place when the vehicle has no
destination left (S-0206 Terms: a vehicle standing still with neither a move destination nor a
next point is not being moved, even while its body turns). Its body keeps turning, and
readiness then waits only for the turn to end.

## Questions

1. A driving vehicle stands still while its body turns, with a behaviour queued that it will
   start when the turn ends. On the tick on which its body facing reaches the facing it was
   given (the facing's last change), is the queued behaviour made current at that tick's first
   per-tick start, and processed in that tick, or at that tick's late start, and processed
   first on the next tick?
2. In scenario B47 (a move that needs a half turn first, then an attack order on the first
   tick of the turn), does the attack order end the move in progress, so that the vehicle has
   no move destination and no next point? Does its body keep turning towards the step's facing,
   or does it stop or turn elsewhere (for example towards the attack's target)?

## Possible observation

Scenario B47 of S-0215, with the recorder also reading the vehicle's weapon facing or firing
after every tick. For question 1: a turret-less armed vehicle whose target is in range. If
*attack* is processed in tick *t*, the vehicle can fire or begin to aim in tick *t*; if not,
in tick *t* + 1 at the earliest. For question 2: B47's own reads of the destination, the cell
and the body facing after the order.

## Current workaround

Interim rule, cited as "Q-1160 interim" in `docs/m7-claims/step1k-m7a.md` and
`docs/development.md` (Simulation, M7a: S-0215 revision 63):
1. The turning clause reads the turning facing at the tick number (S-0206 F1). In the tick in
   which the facing reaches its target, the first per-tick start sees the turn ended
   (`process::drive_body_turning`).
2. An attack order removes the destination and route. A turn in place with no destination
   left ends in the next drive update, and the body keeps its target facing (`drive::tick`).
   With no destination, S-0310 claim 9's firing step may then give the body a new target
   facing towards the target, and readiness waits for that turn to end.
   Test: `claim_11_b47_turning_vehicle_keeps_a_queued_attack`.
