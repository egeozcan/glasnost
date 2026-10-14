---
id: Q-0081
title: When a building leaves play after a delayed death or after it explodes
status: answered
related: [S-0302, S-0312]
answered-by: [S-0214, S-0312]
---

## Need
Two rules of the combat specs end a building's time in play "later", and leave
the moment and what happens then to "the structures specs", which are not
written yet:

- S-0302 claim 16 and S-0312 claim 3: a building in a delayed-kill countdown
  keeps 1 health point, and "at the end of the countdown the building is
  destroyed (the structures specs own the end)". The simulation stores the
  tick the countdown ends (`combat::state::CombatState::delay_kill_until`) but
  nothing happens at that tick: the building stays in play at 1 health point
  until another hit destroys it.
- S-0312 claim 21: an exploding building, or one destroyed while being sold,
  "stays in play after its death effects and leaves later (the structures specs
  own when)". The simulation removes it at the end of its death effects, like a
  building that is not exploding (`combat::death::building_death`).

## Questions
1. When a delayed-kill countdown ends, on which tick and in which part of the
   tick (S-0203) is the building destroyed? Does it then take every death
   effect of S-0312 (claims 4 to 7 and 17 to 21, with their draws) at that
   moment, as for a building killed by a hit? Is the destruction credited
   again (to whom), or only at the delayed-kill hit (S-0302 claim 16)?
2. If a building in the countdown is sold, captured or otherwise removed before
   the countdown ends, does anything of question 1 still happen?
3. How long does an exploding building (S-0312 Terms) stay in play after its
   death effects (in ticks, or until which event)? While it stays, can it be
   hit, targeted or selected, does it take turns, and does it still occupy its
   foundation cells? Does anything happen when it finally leaves (more
   animations, draws, its rubble and crate of claim 20)?
4. The same for a building destroyed while being sold.

## Possible observation
For 1 and 2: S-0312 scenario D3 with the random-state recorder running across
the end of the countdown, a recorder of object counts each tick, and a second
run in which the building is sold during the countdown. For 3 and 4: S-0312
scenario D14 variant c with a recorder of object counts and of the building's
cells, each tick from the killing hit.

## Current workaround
1. The countdown's end tick is kept, and nothing happens at it.
2. An exploding building and a building destroyed while being sold leave play at
   the end of their death effects, as claim 21 says of other buildings.
