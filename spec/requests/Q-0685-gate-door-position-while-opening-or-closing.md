---
id: Q-0685
title: The door position p of a moving gate (S-0108 claim 4, S-0215 claim 35)
status: open
related: [S-0108, S-0215]
answered-by: []
---

## Need
S-0108 claim 4: "While the gate is opening or closing, the image frame drawn from the body sprite
is *p* when not damaged and *p* + *G* + 1 when damaged, where *p* is the gate's door position
clamped to 0 … *G* − 1 ... The door position and its timing belong to the gate spec." S-0215
claim 35 (d) gives the door's timing (rest after *T*<sub>d</sub> ticks) and says "The drawn stage
of the gate, from the stage-count field (Import mapping), is drawing only", but no spec gives *p*
as a function of the door's progress, its direction or *G*. Scenario B13 expects the frames to run
0 … *G* − 1 undamaged while the gate moves. The renderer needs *p* in
`crates/app/src/mapview/build.rs` (`building_items`).

## Questions
1. What is *p* while the door opens, as a function of the ticks since the motion began, the door
   time *T*<sub>d</sub> and *G* (rounding, and whether it reaches *G* − 1 or *G* before rest)?
2. What is *p* while it closes: the same expression run backwards (as the factory door of S-0108
   claim 23, *D* − trunc(*q* × *D*)), or another?
3. After `quirks.gate_reverse_snaps` reverses a closing door, which *p* is drawn until it is at
   rest open?
4. With *T*<sub>d</sub> = 0 (door at rest at its first update), is any moving frame drawn?

## Possible observation
B13 with the recorder reading the drawn stage on every tick (as S-0215 B28 does) for a stock gate
opening and closing, and a gate time changed so that *T*<sub>d</sub> is not a multiple of *G*.

## Current workaround
Interim rule, recorded in docs/development.md: claim 23's door-frame rule with *D* = *G*: with
*q* = (*M* − *r*) / *M* (F8), opening *p* = trunc(*q* × *G*), closing *p* = *G* − trunc(*q* × *G*),
then clamped to 0 … *G* − 1; a door at rest uses claim 2's stationary rows.
