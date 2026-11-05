---
id: Q-2646
title: What follows the take-off that a landing's check of its flight point's cell gives
status: open
related: [S-0501]
answered-by: []
---

## Need

S-0501 r132 claim 7 adds a check that each movement update of a landing makes before
touchdown: "For a pad-bound aircraft it refuses a cell that holds no building the aircraft is in
contact with. A refused cell ends the landing with a take-off, and what follows was not traced
(Open question 28)." Claim 24 meets it for an armed pad-bound aircraft whose landing starts over a
cell of its pad's foundation while its docking position lies outside the foundation, and the
engine also meets it for a pad-bound aircraft that lands with no contact (claim 24's docking stage
without a contact, a pad sold while it lands).

The engine has to place the check inside the movement update and fly the aircraft after it.

## Questions

1. Within a movement update of a landing, when is the check made: before that update's forward
   step and descent, or after them? In particular, can the update in which the descent reaches
   height 0 still be refused by the check, or does its touchdown come first?
2. The landing ends "with a take-off toward cruise height". Does the aircraft climb in the same
   movement update, as claim 8's take-off at a new destination does, or from the next one?
3. After that take-off, does the aircraft fly on toward the destination it still has, with claim
   6's triggers (and so claim 6's pad-bound fall-back to rest) applying as in any flight? Or is it
   sent toward a free cell near it, and does it take damage where none is found (Open question
   28)?
4. Does the take-off end the aircraft's tracking or listing in any way other than claim 47 (a)'s
   ordinary take-off (already tracked, so not added a second time)?

## Possible observation

AC31 part h (3)'s set-up (an armed pad-bound aircraft whose docking position lies outside the
foundation), with the docking position placed within 128 leptons of a foundation cell so that a
landing trigger finds the aircraft over that foundation cell. Record each tick its height, landing
and take-off state, destination, position and health from the landing start to 200 ticks after it.

## Current workaround

`air::landing_cell_check` makes the check at the start of each movement update of a landing,
before its forward step (where a new destination's take-off is made). A refused cell starts claim
8's take-off toward cruise height, and the aircraft climbs in that same update. It keeps its
destination and flight point and flies on toward them in ordinary flight, where claim 6's
triggers apply again. No free-cell search and no damage. Cited
`design: S-0501 claim 7, Open question 28 and Q-2646`.
