---
id: Q-2186
title: Whether a building's slots advance in the tick in which a capture re-enables it
status: open
related: [S-0108, S-0109, S-0203, S-0509, M-0014]
answered-by: []
---

## Need

Our client plays building slot animations (`crates/app/src/mapview/slot_play.rs`) from the
simulation's state after each tick. From intake 55b on it places each change of a tick before or
after that tick's slot advances, by S-0108 claim 13's order of places in the tick:

- A polled trigger acts before any object's turn (S-0203 row 1). So a trigger's change of owner of
  a building whose type needs an engineer re-enables it before its slots' turns. M-0014 shows it:
  in b22n-tm1 the trigger's change at 300 re-enables W7, and its slots advance 0 → 1 in 300.
- A pulse's end re-enables the building before its slots' turns. M-0014 shows it: in the recording
  run the slots advance 2 → 3 in 289, the tick in which the hold reaches 0.
- A power order acts after the turn pass (S-0203 row 8). M-0014 shows it in b22n-t, b22o-ep1,
  b22o-ep3 and b22o-ep6.

An engineer's capture has no such fixed place. S-0108 claim 13 puts it in the turn pass, in update
order (S-0203 claim 3), and a slot animation advances in its own turn (S-0109 claim 5). So whether
a capture comes before or after a slot's advance in the same tick depends on the update order of
the engineer and the animation, and no spec states it. M-0014's recording run gives the captures at
133 and 134 only as the run's timeline: *idle* and *active-1* "advance in the next tick, then every
3 ticks". That is consistent with a capture that came after the slots' turns in its tick. It is
also consistent with a slot whose clock was not yet due.

## Questions

1. When an engineer captures a building whose type needs an engineer and whose slots the map's
   placement held (S-0108 claim 26), and the slots' clocks are past their stage delay, do the
   slots first advance in the tick of the capture or in the tick after it?
2. Does the answer depend on anything the game shows, such as which was placed first, the
   engineer or the building (and so the building's slot animations)? Or is it the same for every
   capture?
3. Is the same true for the other captures of S-0509 that change a building's owner in the turn
   pass, such as a takeover or a mind control?

## Possible observation

M-0014's set-up with W7 and W8 and one engineer each. Record each slot's image frame and stage
delay at every tick end from 120 to 140. Run it once as M-0014 placed the objects. Run it again
with the engineers placed before the buildings in the map file, and once more with them placed
after. Check that each slot's clock is past its stage delay at the capture tick.

## Current workaround

The client applies an ownership change that the simulation reports with an announcement (an
engineer's capture, a takeover or a mind control, S-0509 claim 12) after the tick's slot advances,
so the first advance comes in the next tick, as M-0014's timeline shows. It applies every other
ownership change (a trigger's, an occupation's) before them. This is our design (`// design:
Q-2186` in `Slots::enablings`). Only the slot drawing is affected; the simulation is not.
