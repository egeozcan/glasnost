---
id: Q-2246
title: When the fire place's retarget test is made for a system that is not of the fire kind
status: open
related: [S-0301, S-0616]
answered-by: []
---

## Need
S-0301 r63 claim 5, *place-and-kind rule*: a retarget to a target that is
not within reach lets the fire place go, whatever the kind of the system that
place holds. S-0616 claim 1 makes this test for a fire-kind system in that
system's own turn. Neither spec says in which tick, or at which step of a
tick, the test is made when the fire place holds a system of another kind
(spark, railgun, smoke, gas, or no kind). Such a system has its own kind's
turn, and S-0616 gives that turn no retarget step.

The engine must pick a moment for the test. It decides the tick in which
the system is marked after its owner's target changes to one out of reach,
and so whether a further turn of the system's own work (for example a spark
system's sparks or a gas system's spread) happens after the retarget.

## Questions
1. A firer's fire place holds a system that is not of the fire kind. The
   firer's target changes to one that is not within reach of the weapon.
   In which tick is that system marked: in the tick of the retarget, in the
   system's next turn, or at another moment?
2. Within that tick, does the system still do its kind's own work (create
   particles, spread, draw) before it is marked, or is it marked first?
3. Is the answer the same for each kind (spark, railgun, smoke, gas, no
   kind)?

## Possible observation
S-0301's F4 unmatched-emitter scenario, part u1 or u2 (a fire-flag weapon
whose system is spark-kind), with a second enemy placed out of the weapon's
range. Force a retarget to the far enemy and record, every tick from the
retarget, whether the system still creates particles and the tick at which
it stops.

## Current workaround
`emitters::fire_place_let_go` makes the test in the held system's own turn,
after its kind's work, as a fire-kind system makes it in its turn (called
from `gas::system_turn`). Marked `design (Q-2246)` in
`crates/deterministic/sim/src/emitters.rs`. Intake 63 made this choice
(commit 890f3db) without a request number; this request gives it one.
