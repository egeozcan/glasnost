---
id: Q-0631
title: Which point does an aircraft fly to in move, and when has its locomotor stopped?
status: open
related: [S-0501]
answered-by: []
---

## Need

S-0501 claim 33 and Open question 2: *move* flies an aircraft to "the flight point of its
destination", which was not decoded, and both *move* and table T1 (an ordinary attacker's
*approaching* row) test that the aircraft's locomotor "has stopped". The gap is in
`glasnost_sim::air` (`flight_point`, `stopped`).

## Questions

1. For a cell destination and for an object destination, which point does the aircraft fly to?
2. When does the flying locomotor report that it has stopped: at touchdown, when the current
   speed fraction reaches 0, or at another moment? Does a hovering aircraft that cannot land
   (not landable, or holding height) ever report that it has stopped?

## Possible observation

S-0501 scenario AC10: order moves to cells and onto objects; log position, destination, speed
fraction and the tick at which *move* falls back to rest.

## Current workaround

The flight point is the destination's own point (a cell's centre at its height, an object's
position); the locomotor has stopped when the current speed fraction is 0 and the height is at
the held height, or it is on the ground (marked Q-0631 in `glasnost_sim::air` and in
docs/development.md, M6a interim rules).
