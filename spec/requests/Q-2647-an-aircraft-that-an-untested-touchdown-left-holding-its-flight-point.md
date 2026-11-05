---
id: Q-2647
title: Tracking, listing and movement of an aircraft that an untested touchdown left holding its flight point
status: answered
related: [S-0501, S-0209, S-0303]
answered-by: [S-0501]
---

## Need

S-0501 r132 claim 7, with `quirks.aircraft_descent_touchdown_untested` on (its default), makes no
cell test at a touchdown: only the touchdown's own forget test applies. A touchdown outside the
flight point's cell whose forget test does not apply leaves the aircraft on the ground by its
height, still holding its flight point, with its destination (it then slides and climbs, claim 7,
Q-2578) or with none (landing where it is, claim 6). Open question 29 leaves open how such an
aircraft is tracked and listed after the touchdown, and what one with no destination does. Claim
47 (c) excepts it from the rule for an untracked aircraft in flight.

The engine meets the case whenever a landing drifts out of its flight point's cell, toward a pad
whose docking position lies off the foundation (AC31 part h (1)), or onto a building's cell.

## Questions

1. Claim 47 (b) ends its tracking at the touchdown. When does it start being tracked again: at its
   first movement update with a positive carried speed fraction, on the ground as in flight (claim
   47 (c)'s rule), only once it is above the terrain, or at some other event? Does
   `quirks.aircraft_corner_sector_untracked` apply to it?
2. While it slides at height 0, is it listed in the ground cell layer of each cell it moves into,
   and does it hold the object-present flag (S-0209 claim 1) in any cell? When it climbs above
   height 0, when does that listing end?
3. With no destination but its flight point kept (claim 6's landing where it is), does it slide
   toward that flight point and climb as one with a destination does (claim 7), or does it stay
   where it touched down? Does claim 6's exact-cell trigger start a new landing for it?
4. A sliding aircraft whose destination is cleared on the ground (a stop order, say), so that it
   holds no flight point any more: does it come to rest where it is, or do its forward steps go
   on?

## Possible observation

AC31 part h (1) with a touchdown in a cell next to the docking position's. Record each tick from
the touchdown to 300 ticks after it: position, height, speed fraction, whether a ground search and
the flying-object search (S-0303 claim 6) find it, and whether its cell's object-present flag is
set.

## Current workaround

At the touchdown its tracking ends and it is listed in its cell layer by its listing height
(S-0209 claim 13, as for any aircraft at height 0); the slide lists it in each cell it moves into
by the same rule. It holds no object-present flag while it slides: that flag is set only when it
comes to rest. It becomes tracked again by claim 47 (c)'s rule for an untracked aircraft in
flight, at its first movement update that starts above the terrain with a positive carried
fraction, with the corner switch applying. With no destination it slides toward its flight point
and climbs as one with a destination does, and claim 6's triggers apply. One whose flight point is
cleared on the ground comes to rest where it is, as after a touchdown with nothing kept. Cited
`design: S-0501 claim 7, Open question 29 and Q-2647`.
