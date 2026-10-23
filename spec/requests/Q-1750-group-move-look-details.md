---
id: Q-1750
title: Where the members of a group move order end up when their look is cut short
status: answered
related: [S-0213, S-0209, S-0212]
answered-by: [S-0213]
---

## Need

S-0213 claim 7 spreads the destinations of a group move, but its Open questions leave the
placement details unsettled, and our engine runs an interim rule (each look 1 to 6 steps from the
common cell, a spoiled look falls back to the last taken cell or the common cell, a member in the
leader's cell tests only the common cell, a member given a shared cell is not placed again).
The lead's play note (M8 step A4, "weird pathing issues") may come from several members of one
group being sent to the same cell and then jostling there. Code: `glasnost_sim::orders`
(`group_destinations`, `spread`). Scenario O10 already records the needed evidence but does not
test it.

## Questions

1. How many cells out from the clicked cell can a member's destination lie, and is that limit
   counted for each member on its own or for the group as a whole?
2. Can a member's destination be the clicked cell itself when the member is not the leader and the
   ground around is open (that is, does the first cell examined on a member's line lie one cell
   out, or at the clicked cell)?
3. When a member's line is cut short by water, a cliff of 3 or more levels, a building or an
   occupied cell before an open cell, does that member get the clicked cell, a cell already given
   to another member, or some other open cell nearby (a further search)?
4. A member standing in the leader's own cell: which destination does it get?
5. A member that was given the same destination as another unit standing in its cell: is it still
   given a cell of its own later, or does it keep the shared one?
6. In ordinary play with 6 to 12 selected units of one type moved onto open ground, how many end up
   sharing a destination cell?

## Possible observation

S-0213 scenario O10, main part and parts (b) and (d), with the destination cell of every member
recorded per run; add a 12-unit block for question 6.

## Current workaround

S-0213 claim 7's interim rule for our engine (6 steps per member, first step one cell length out,
spoiled look gives the fallback cell with no further search, zero direction in the leader's cell,
shared-cell members not placed again).
